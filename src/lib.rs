mod components;
mod consts;
mod preprocessor;
mod recalc_swap_list;

#[cfg(test)]
mod tests;

use crate::recalc_swap_list::RecalcSwapList;
use crate::{
    components::{Node, Transistor},
    consts::*,
};
use std::{
    cell::Cell,
    io::{Read, Seek},
};

#[allow(dead_code)]
enum MemoryType {
    /// $0000-$07FF (mirrored to $1FFF)
    CpuRam,
    /// $8000-$FFFF
    PrgRam,
    /// $0000-$1FFF
    ChrRam,
    /// $2000-$2FFF ($2000-$23FF is nametable A, $2400-$27FF is nametable B)
    NametableRam,
    /// Internal to the PPU - 32 bytes (including some mirrors)
    PaletteRam,
    /// Internal to the PPU.  256 bytes for primary + 32 bytes for secondary
    SpriteRam,
    /// All of the above put together + a state of all of the nodes in the simulation (used for
    /// save/load state)
    FullState,
}

#[allow(dead_code)]
enum MirroringType {
    Horizontal,
    Vertical,
    FourScreens,
    ScreenAOnly,
    ScreenBOnly,
}

pub struct SimulationState {
    all_recalc_nodes: Vec<u16>,
    transistors_initial_power_state: Vec<bool>,
    nodes: Vec<Node>,
    has_ground: bool,
    has_power: bool,
    group: Vec<u16>,
    node_counts: Vec<u8>,
    transistors: Vec<Transistor>,
    nodes_c1_c2: Vec<Vec<u16>>,
    processed_nodes: Vec<Cell<bool>>,
    step_cycle_count: u8,
    prev_ppu_ale: bool,
    prev_ppu_write: bool,
    prev_ppu_read: bool,
    chr_address: u16,
    last_address: u16,
    mirroring_type: MirroringType,
    chr_ram: Box<[u8; 0x2000]>,
    nametable_ram: Box<[[u8; 0x400]; 4]>,
    cpu_ram: Box<[u8; 0x800]>,
    prg_ram: Box<[u8; 0x8000]>,
    last_cpu_db_value: u8,
    last_data: u8,
    prev_hpos: i32,
    ppu_framebuffer: Box<[u32; 256 * 240]>,
    sprite_nodes: Vec<Vec<(i32, i32)>>,
    palette_nodes: Vec<Vec<(i32, i32)>>,
    recalc_swap_list: RecalcSwapList,
}

impl SimulationState {
    pub fn new() -> Self {
        use crate::preprocessor::{
            id_conversion_table, load_ppu_nodes, load_segment_definitions,
            load_transistor_definitions, setup_nodes, setup_transistors,
        };
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let trans_defs = load_transistor_definitions(&conversion_table);
        let mut nodes = setup_nodes(&seg_defs);
        let (palette_nodes, sprite_nodes) = load_ppu_nodes();
        let transistors_initial_power_state = trans_defs
            .iter()
            .map(|def| def.gate == NODE_PWR)
            .collect::<Vec<bool>>();
        let (transistors, node_counts, nodes_c1_c2, _) = setup_transistors(&mut nodes, trans_defs);
        let all_recalc_nodes = nodes
            .iter()
            .filter(|n| n.num != NODE_PWR && n.num != NODE_GND && n.num != EMPTYNODE)
            .map(|n| n.num)
            .collect::<Vec<u16>>();

        SimulationState {
            all_recalc_nodes,
            transistors_initial_power_state,
            nodes: nodes.into_iter().map(|def| Node::new(def)).collect(),
            node_counts,
            has_ground: false,
            has_power: false,
            group: Vec::new(),
            transistors,
            nodes_c1_c2,
            processed_nodes: vec![Cell::new(false); NUM_NODES],
            step_cycle_count: 0,
            prev_ppu_ale: false,
            prev_ppu_read: true,
            prev_ppu_write: true,
            chr_address: 0,
            last_address: 0,
            mirroring_type: MirroringType::Horizontal,
            chr_ram: Box::new([0; 0x2000]),
            nametable_ram: Box::new([[0; 0x400]; 4]),
            cpu_ram: Box::new([0; 0x800]),
            prg_ram: Box::new([0; 0x8000]),
            last_cpu_db_value: 0,
            last_data: 0,
            prev_hpos: -1,
            ppu_framebuffer: Box::new([0; 256 * 240]),
            sprite_nodes,
            palette_nodes,
            recalc_swap_list: RecalcSwapList::new(),
        }
    }

    pub fn load_rom<R: Read + Seek>(&mut self, input: &mut R) {
        use nes_rom_loader::{Mirroring, NesRom};
        // TODO: Return Result so failure can be handled gracefully
        let rom = NesRom::load(input).unwrap();

        if rom.mapper != 0 {
            panic!("Only mapper 0 is supported");
        }

        let mirroring_type = match rom.mirroring {
            Mirroring::Horizontal => MirroringType::Horizontal,
            Mirroring::Vertical => MirroringType::Vertical,
            _ => unimplemented!(),
        };

        self.mirroring_type = mirroring_type;

        let mut prg = rom.prg.clone();

        if prg.len() == 0x4000 {
            prg.extend_from_slice(&rom.prg);
        }

        self.init(false);
        self.set_memory_state(MemoryType::ChrRam, &rom.chr);
        self.set_memory_state(MemoryType::PrgRam, &prg);
    }

    fn set_memory_state(&mut self, memory_type: MemoryType, buffer: &[u8]) {
        match memory_type {
            MemoryType::PrgRam => self.prg_ram.copy_from_slice(buffer),
            MemoryType::ChrRam => self.chr_ram.copy_from_slice(buffer),
            MemoryType::CpuRam => self.cpu_ram.copy_from_slice(buffer),
            MemoryType::NametableRam => {
                for i in 0..4 {
                    let start_index = i * 0x400;
                    self.nametable_ram[i]
                        .copy_from_slice(&buffer[start_index..(start_index + 0x400)]);
                }
            }
            MemoryType::PaletteRam => {
                for (i, byte) in buffer.iter().enumerate().take(PALETTE_RAM_SIZE) {
                    self.palette_write(i as u16, *byte);
                }
            }
            MemoryType::SpriteRam => {
                for (i, byte) in buffer.iter().enumerate().take(SPRITE_RAM_SIZE) {
                    self.sprite_write(i as u16, *byte);
                }
            }
            MemoryType::FullState => unimplemented!(),
        }
    }

    fn palette_write(&mut self, addr: u16, val: u8) {
        for b in 0..6 {
            let (n0, n1) = self.palette_nodes[addr as usize][b as usize];

            if val & (1 << b) > 0 {
                self.set_bit(n1, n0);
            } else {
                self.set_bit(n0, n1);
            }
        }
    }

    fn sprite_write(&mut self, addr: u16, val: u8) {
        for b in 0..8 {
            let (n0, n1) = self.sprite_nodes[addr as usize][b as usize];
            if val & (1 << b) > 0 {
                self.set_bit(n1, n0);
            } else {
                self.set_bit(n0, n1);
            }
        }
    }

    fn set_bit(&mut self, n1: i32, n2: i32) {
        if n1 < 0 || n2 < 0 {
            return;
        }

        for gate in &self.nodes[n1 as usize].gates {
            self.transistors[*gate as usize].on.set(true);
        }

        for gate in &self.nodes[n2 as usize].gates {
            self.transistors[*gate as usize].on.set(false);
        }

        self.nodes[n1 as usize].state.set(true);
        self.nodes[n2 as usize].state.set(false);
        self.recalc_node_list(&[n1 as u16, n2 as u16]);
    }

    fn init(&mut self, soft_reset: bool) {
        self.prev_hpos = -1;

        if soft_reset {
            self.set_low(NODE_RESET);
            for _ in 0..=(12 * 8 * 2) {
                if self.is_node_high(NODE_CLK0) {
                    self.set_low(NODE_CLK0);
                } else {
                    self.set_high(NODE_CLK0);
                }
            }
            self.set_high(NODE_RESET);
        } else {
            self.ppu_framebuffer.iter_mut().for_each(|b| *b = 0);
            self.cpu_ram.iter_mut().for_each(|b| *b = 0);
            self.prg_ram.iter_mut().for_each(|b| *b = 0);
            self.chr_ram.iter_mut().for_each(|b| *b = 0);
            self.nametable_ram
                .iter_mut()
                .for_each(|nt| nt.iter_mut().for_each(|b| *b = 0));

            for node in self.nodes.iter() {
                node.state.set(false);
                node.floating.set(true);
            }

            self.nodes[NODE_GND as usize].state.set(false);
            self.nodes[NODE_GND as usize].floating.set(false);
            self.nodes[NODE_PWR as usize].state.set(true);
            self.nodes[NODE_PWR as usize].floating.set(false);

            for (i, on) in self.transistors_initial_power_state.iter().enumerate() {
                self.transistors[i].on.set(*on);
            }

            self.set_low(NODE_RESET);
            self.set_low(NODE_CLK0);
            self.set_high(NODE_IO_CE);
            self.set_high(NODE_INT);

            for _ in 0..6 {
                self.set_high(NODE_CLK0);
                self.set_low(NODE_CLK0);
            }

            self.set_low(NODE_CPU_SO);
            self.set_high(NODE_CPU_IRQ);
            self.set_high(NODE_CPU_NMI);

            self.recalc_node_list(&self.all_recalc_nodes.clone());

            for _ in 0..(12 * 8) {
                self.set_high(NODE_CLK0);
                self.set_low(NODE_CLK0);
            }

            self.set_high(NODE_RESET);
        }

        self.chr_address = 0;
        self.prev_ppu_read = true;
        self.prev_ppu_write = true;
        self.prev_ppu_ale = false;
    }

    pub fn half_step(&mut self) {
        let cpu_clk0 = self.is_node_high(NODE_CPU_CLK0);
        let clk = self.is_node_high(NODE_CLK0);

        if clk {
            self.set_low(NODE_CLK0);
        } else {
            self.set_high(NODE_CLK0);
        }

        if self.step_cycle_count > 0 {
            self.step_cycle_count -= 1;
            if self.step_cycle_count == 0 {
                self.set_high(NODE_IO_CE);
            }
        } else if self.is_node_high(NODE_CPU_AB13)
            && !self.is_node_high(NODE_CPU_AB14)
            && !self.is_node_high(NODE_CPU_AB15)
            && self.is_node_high(NODE_CPU_CLK0)
        {
            // Simulate the 74139's logic
            self.set_low(NODE_IO_CE);
            self.step_cycle_count = 11;
        }

        self.handle_chr_bus();

        if cpu_clk0 != self.is_node_high(NODE_CPU_CLK0) {
            if cpu_clk0 {
                self.handle_cpu_bus_read();
            } else {
                self.handle_cpu_bus_write();
            }
        }

        if self.read_bit(NODE_PCLK1) > 0 {
            let hpos = i32::from(self.read_hpos()) - 2;
            if hpos != self.prev_hpos {
                let vpos = self.read_vpos();
                if hpos >= 0 && hpos < 256 && vpos < 240 {
                    let palette_entry = self.read_bit(NODE_PAL_D0_OUT)
                        | (self.read_bit(NODE_PAL_D1_OUT) << 1)
                        | (self.read_bit(NODE_PAL_D2_OUT) << 2)
                        | (self.read_bit(NODE_PAL_D3_OUT) << 3)
                        | (self.read_bit(NODE_PAL_D4_OUT) << 4)
                        | (self.read_bit(NODE_PAL_D5_OUT) << 5);
                    self.ppu_framebuffer[((vpos << 8) | (hpos as u16)) as usize] =
                        PALETTE_ARGB[palette_entry as usize];
                }
                self.prev_hpos = hpos;
            }
        }
    }

    fn handle_cpu_bus_read(&mut self) {
        if self.is_node_high(NODE_CPU_RW) {
            let a = self.read_cpu_address_bus();
            let (d, open_bus) = self.cpu_read(a);

            if open_bus {
                self.float_cpu_db();
            } else {
                self.write_cpu_db(d);
            }
        }
    }

    fn handle_cpu_bus_write(&mut self) {
        if !self.is_node_high(NODE_CPU_RW) {
            let a = self.read_cpu_address_bus();
            let d = self.read_cpu_data_bus();
            self.cpu_write(a, d);
        }
    }

    fn read_cpu_address_bus(&mut self) -> u16 {
        let mut res = 0_u16;
        for (i, nn) in [
            NODE_CPU_AB0,
            NODE_CPU_AB1,
            NODE_CPU_AB2,
            NODE_CPU_AB3,
            NODE_CPU_AB4,
            NODE_CPU_AB5,
            NODE_CPU_AB6,
            NODE_CPU_AB7,
            NODE_CPU_AB8,
            NODE_CPU_AB9,
            NODE_CPU_AB10,
            NODE_CPU_AB11,
            NODE_CPU_AB12,
            NODE_CPU_AB13,
            NODE_CPU_AB14,
            NODE_CPU_AB15,
        ]
        .iter()
        .enumerate()
        {
            res += (self.is_node_high(*nn) as u16) << i;
        }
        res
    }

    fn read_cpu_data_bus(&mut self) -> u8 {
        let mut res = 0_u8;
        for (i, nn) in [
            NODE_CPU_DB0,
            NODE_CPU_DB1,
            NODE_CPU_DB2,
            NODE_CPU_DB3,
            NODE_CPU_DB4,
            NODE_CPU_DB5,
            NODE_CPU_DB6,
            NODE_CPU_DB7,
        ]
        .iter()
        .enumerate()
        {
            res += (self.is_node_high(*nn) as u8) << i;
        }
        res
    }

    fn is_node_high(&self, node_number: u16) -> bool {
        self.nodes[node_number as usize].state.get()
    }

    fn recalc_node_list(&mut self, recalc_list: &[u16]) {
        self.recalc_swap_list.init(recalc_list);
        for iter_count in 0..100 {
            if iter_count >= 99 {
                panic!("iter count exceeded");
            }
            for node_number in self.recalc_swap_list.cur_list().borrow().iter() {
                let node_number = *node_number;
                if node_number == NODE_GND || node_number == NODE_PWR {
                    continue;
                } else {
                    self.get_node_group(node_number);
                    let new_state = self.get_node_value();
                    for node_number in &self.group {
                        let node_number = *node_number as usize;
                        if self.nodes[node_number].state.get() != new_state {
                            self.nodes[node_number].state.set(new_state);
                            for i in &self.nodes[node_number].gates {
                                if self.nodes[node_number].state.get() {
                                    self.turn_transistor_on(*i);
                                } else {
                                    self.turn_transistor_off(*i);
                                }
                            }
                        }
                    }
                }
            }

            if self.recalc_swap_list.is_next_list_empty() {
                return;
            }

            for node_number in self.recalc_swap_list.next_list().iter() {
                self.processed_nodes[*node_number as usize].set(false);
            }
            self.recalc_swap_list.swap();
        }
    }

    fn turn_transistor_on(&self, i: u16) {
        let i = i as usize;
        if !self.transistors[i].on.get() {
            self.transistors[i].on.set(true);
            self.add_recalc_node(self.transistors[i].c1);
        }
    }

    fn turn_transistor_off(&self, i: u16) {
        let i = i as usize;
        if self.transistors[i].on.get() {
            self.transistors[i].on.set(false);
            self.add_recalc_node(self.transistors[i].c1);
            self.add_recalc_node(self.transistors[i].c2);
        }
    }

    fn add_recalc_node(&self, node_number: u16) {
        if node_number == NODE_GND || node_number == NODE_PWR {
            return;
        }

        if !self.processed_nodes[node_number as usize].get() {
            self.recalc_swap_list.push_next_list(node_number);
            self.processed_nodes[node_number as usize].set(true);
        }
    }

    fn set_high(&mut self, node_number: u16) {
        self.nodes[node_number as usize].pullup.set(true);
        self.nodes[node_number as usize].pulldown.set(false);
        self.recalc_node_list(&[node_number])
    }

    fn set_low(&mut self, node_number: u16) {
        self.nodes[node_number as usize].pullup.set(false);
        self.nodes[node_number as usize].pulldown.set(true);
        self.recalc_node_list(&[node_number])
    }

    fn read_db(&self) -> u8 {
        let mut res = 0_u8;
        for (i, node_number) in [
            NODE_DB0, NODE_DB1, NODE_DB2, NODE_DB3, NODE_DB4, NODE_DB5, NODE_DB6, NODE_DB7,
        ]
        .iter()
        .enumerate()
        {
            let node_number = *node_number;
            res += (self.is_node_high(node_number) as u8) << i;
        }
        res
    }

    fn read_hpos(&self) -> u16 {
        let mut res = 0_u16;
        for (i, node_number) in [
            NODE_HPOS0, NODE_HPOS1, NODE_HPOS2, NODE_HPOS3, NODE_HPOS4, NODE_HPOS5, NODE_HPOS6,
            NODE_HPOS7, NODE_HPOS8,
        ]
        .iter()
        .enumerate()
        {
            let node_number = *node_number;
            res += (self.is_node_high(node_number) as u16) << i;
        }
        res
    }

    fn read_vpos(&self) -> u16 {
        let mut res = 0_u16;
        for (i, node_number) in [
            NODE_VPOS0, NODE_VPOS1, NODE_VPOS2, NODE_VPOS3, NODE_VPOS4, NODE_VPOS5, NODE_VPOS6,
            NODE_VPOS7, NODE_VPOS8,
        ]
        .iter()
        .enumerate()
        {
            let node_number = *node_number;
            res += (self.is_node_high(node_number) as u16) << i;
        }
        res
    }

    fn read_ab(&self) -> u16 {
        let mut res = 0_u16;
        for (i, node_number) in [
            NODE_AB0, NODE_AB1, NODE_AB2, NODE_AB3, NODE_AB4, NODE_AB5, NODE_AB6, NODE_AB7,
            NODE_AB8, NODE_AB9, NODE_AB10, NODE_AB11, NODE_AB12, NODE_AB13,
        ]
        .iter()
        .enumerate()
        {
            let node_number = *node_number;
            res += (self.is_node_high(node_number) as u16) << i;
        }
        res
    }

    fn read_bit(&self, node_number: u16) -> u8 {
        self.is_node_high(node_number) as u8
    }

    fn handle_chr_bus(&mut self) {
        let ale = self.is_node_high(NODE_ALE);
        let rd = self.is_node_high(NODE_RD);
        let wr = self.is_node_high(NODE_WR);

        // rising edge of ALE
        if self.prev_ppu_ale && ale {
            self.chr_address = self.read_ppu_address_bus();
        }

        // falling edge of /RD - put bits on bus
        if self.prev_ppu_read && !rd {
            self.write_db(self.ppu_read(self.chr_address));
        }

        // rising edge of /RD - flaot the data bus
        if !self.prev_ppu_read && rd {
            self.float_db();
        }

        // rising edge of /WR - store data in RAM
        if !self.prev_ppu_write && wr {
            let ppu_data_bus_val = self.read_ppu_data_bus();
            self.ppu_write(self.chr_address, ppu_data_bus_val);
        }

        self.read_ppu_data_bus();
        self.prev_ppu_ale = ale;
        self.prev_ppu_read = rd;
        self.prev_ppu_write = wr;
    }

    fn read_ppu_data_bus(&mut self) -> u8 {
        if !self.is_node_high(NODE_RD) || !self.is_node_high(NODE_WR) {
            self.last_data = self.read_db();
        }
        self.last_data
    }

    fn float_byte(&mut self, nodes: [u16; 8]) {
        let mut recalc_nodes = [0_u16; 8];
        for (i, node_number) in nodes.iter().enumerate() {
            let node_number = *node_number;
            self.nodes[node_number as usize].pulldown.set(false);
            self.nodes[node_number as usize].pullup.set(false);
            recalc_nodes[i as usize] = node_number;
        }
        self.recalc_node_list(&recalc_nodes);
    }

    fn float_db(&mut self) {
        self.float_byte([
            NODE_DB0, NODE_DB1, NODE_DB2, NODE_DB3, NODE_DB4, NODE_DB5, NODE_DB6, NODE_DB7,
        ])
    }

    fn float_cpu_db(&mut self) {
        self.float_byte([
            NODE_CPU_DB0,
            NODE_CPU_DB1,
            NODE_CPU_DB2,
            NODE_CPU_DB3,
            NODE_CPU_DB4,
            NODE_CPU_DB5,
            NODE_CPU_DB6,
            NODE_CPU_DB7,
        ])
    }

    /// Read byte at address in memory, returning the byte at that address and a boolean
    /// indicating an open bus.
    fn cpu_read(&self, a: u16) -> (u8, bool) {
        if a < 0x2000 {
            (self.cpu_ram[(a & 0x7ff) as usize], false)
        } else if a >= 0x8000 {
            (self.prg_ram[(a - 0x8000) as usize], false)
        } else {
            // TODO: proper open bus implementation
            (self.last_cpu_db_value, true)
        }
    }

    fn cpu_write(&mut self, a: u16, d: u8) {
        if a < 0x2000 {
            self.cpu_ram[(a & 0x7ff) as usize] = d;
        } else if a >= 0x8000 {
            self.prg_ram[(a - 0x8000) as usize] = d;
        }
        // else external device (i.e. PPU)
    }

    fn ppu_write(&mut self, mut a: u16, d: u8) {
        a &= 0x3fff;
        if a >= 0x3000 {
            a -= 0x1000;
        }

        if a < 0x2000 {
            self.chr_ram[a as usize] = d
        } else {
            self.nametable_ram[self.get_nametable(a) as usize][(a & 0x3ff) as usize] = d;
        }
    }

    fn ppu_read(&self, mut a: u16) -> u8 {
        a &= 0x3fff;
        if a >= 0x3000 {
            a -= 0x1000;
        }

        if a < 0x2000 {
            self.chr_ram[a as usize]
        } else {
            self.nametable_ram[self.get_nametable(a) as usize][(a & 0x3ff) as usize]
        }
    }

    fn get_nametable(&self, a: u16) -> u16 {
        match self.mirroring_type {
            MirroringType::Horizontal => {
                if a & 0x800 > 0 {
                    1
                } else {
                    0
                }
            }
            MirroringType::Vertical => {
                if a & 0x400 > 0 {
                    1
                } else {
                    0
                }
            }
            MirroringType::FourScreens => {
                // TODO: Wouldn't this always equal 0?
                // (a & 0xc00) >> 16
                unimplemented!()
            }
            MirroringType::ScreenAOnly => 0,
            MirroringType::ScreenBOnly => 1,
        }
    }

    fn write_cpu_db(&mut self, val: u8) {
        self.write_byte(
            [
                NODE_CPU_DB0,
                NODE_CPU_DB1,
                NODE_CPU_DB2,
                NODE_CPU_DB3,
                NODE_CPU_DB4,
                NODE_CPU_DB5,
                NODE_CPU_DB6,
                NODE_CPU_DB7,
            ],
            val,
        );
    }

    fn write_db(&mut self, val: u8) {
        self.write_byte(
            [
                NODE_DB0, NODE_DB1, NODE_DB2, NODE_DB3, NODE_DB4, NODE_DB5, NODE_DB6, NODE_DB7,
            ],
            val,
        );
    }

    fn write_byte(&mut self, nodes: [u16; 8], mut val: u8) {
        let mut recalc_nodes = [0_u16; 8];
        for (i, node_number) in nodes.iter().enumerate() {
            let node_number = *node_number;
            if val % 2 == 0 {
                self.nodes[node_number as usize].pulldown.set(true);
                self.nodes[node_number as usize].pullup.set(false);
            } else {
                self.nodes[node_number as usize].pulldown.set(false);
                self.nodes[node_number as usize].pullup.set(true);
            }
            recalc_nodes[i as usize] = node_number;
            val >>= 1;
        }

        self.recalc_node_list(&recalc_nodes);
    }

    fn read_ppu_address_bus(&mut self) -> u16 {
        if self.is_node_high(NODE_ALE) {
            self.last_address = self.read_ab();
        }

        self.last_address
    }

    fn get_node_value(&mut self) -> bool {
        if self.has_ground && self.has_power {
            for i in &self.group {
                let i = *i;
                if i == 359
                    || i == 566
                    || i == 691
                    || i == 871
                    || i == 870
                    || i == 864
                    || i == 856
                    || i == 818
                {
                    self.has_ground = false;
                    self.has_power = false;
                    break;
                }
            }
        }

        if self.has_ground {
            false
        } else if self.has_power {
            true
        } else {
            let mut hi_area = 0_u64;
            let mut lo_area = 0_u64;
            for node_number in &self.group {
                let node = &self.nodes[*node_number as usize];
                if node.pullup.get() {
                    return true;
                } else if node.pulldown.get() {
                    return false;
                } else if node.state.get() {
                    hi_area += node.area
                } else {
                    lo_area += node.area
                }
            }

            hi_area > lo_area
        }
    }

    fn get_node_group(&mut self, node_number: u16) {
        self.has_ground = false;
        self.has_power = false;
        self.group.clear();
        self.add_node_to_group(node_number);
    }

    fn add_node_to_group(&mut self, node_number: u16) {
        if node_number == NODE_GND {
            self.has_ground = true;
            return;
        }

        if node_number == NODE_PWR {
            self.has_power = true;
            return;
        }

        if self.group.contains(&node_number) {
            return;
        }

        self.group.push(node_number);

        for i in 0..(self.node_counts[node_number as usize] as usize) {
            let transistor_index = self.nodes_c1_c2[node_number as usize][i] as usize;
            let transistor = &self.transistors[transistor_index];
            if transistor.on.get() {
                let node_to_add = if transistor.c1 == node_number {
                    transistor.c2
                } else {
                    transistor.c1
                };
                self.add_node_to_group(node_to_add);
            }
        }
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::new()
    }
}
