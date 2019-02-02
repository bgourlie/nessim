#![allow(unused_variables, dead_code)]
//
use fnv::FnvHashMap;
use std;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

const NUM_NODES: usize = 33001;
const EMPTYNODE: u16 = 65535;
const CPU_OFFSET: u16 = 13000;
const NGND: u16 = 2;
const NPWR: u16 = 1;
const NUMBERS: [&str; 32] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
];

#[allow(clippy::unreadable_literal)]
const PALETTE_ARGB: [u32; 64] = [
    0xFF666666, 0xFF002A88, 0xFF1412A7, 0xFF3B00A4, 0xFF5C007E, 0xFF6E0040, 0xFF6C0600, 0xFF561D00,
    0xFF333500, 0xFF0B4800, 0xFF005200, 0xFF004F08, 0xFF00404D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFADADAD, 0xFF155FD9, 0xFF4240FF, 0xFF7527FE, 0xFFA01ACC, 0xFFB71E7B, 0xFFB53120, 0xFF994E00,
    0xFF6B6D00, 0xFF388700, 0xFF0C9300, 0xFF008F32, 0xFF007C8D, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFF64B0FF, 0xFF9290FF, 0xFFC676FF, 0xFFF36AFF, 0xFFFE6ECC, 0xFFFE8170, 0xFFEA9E22,
    0xFFBCBE00, 0xFF88D800, 0xFF5CE430, 0xFF45E082, 0xFF48CDDE, 0xFF4F4F4F, 0xFF000000, 0xFF000000,
    0xFFFFFEFF, 0xFFC0DFFF, 0xFFD3D2FF, 0xFFE8C8FF, 0xFFFBC2FF, 0xFFFEC4EA, 0xFFFECCC5, 0xFFF7D8A5,
    0xFFE4E594, 0xFFCFEF96, 0xFFBDF4AB, 0xFFB3F3CC, 0xFFB5EBF2, 0xFFB8B8B8, 0xFF000000, 0xFF000000,
];

enum MirroringType {
    Horizontal,
    Vertical,
    FourScreens,
    ScreenAOnly,
    ScreenBOnly,
}

pub struct SimulationState {
    cycle: u16,
    nodes: Vec<Node>,
    node_number_by_name: FnvHashMap<String, u16>,
    has_ground: bool,
    has_power: bool,
    group: Vec<u16>,
    node_counts: Vec<u8>,
    transistors: Vec<Transistor>,
    nodes_c1_c2: Vec<Vec<u16>>,
    processed_nodes: Vec<u16>,
    recalc_lists: [Option<Vec<u16>>; 2],
    cur_recalc_list_index: u8,
    group_empty: bool,
    step_cycle_count: u8,
    prev_ppu_ale: bool,
    prev_ppu_write: bool,
    prev_ppu_read: bool,
    chr_address: u16,
    node_number_cache: FnvHashMap<String, Vec<u16>>,
    bit_count_cache: FnvHashMap<String, u8>,
    last_address: u16,
    mirroring_type: MirroringType,
    chr_ram: Box<[u8; 0x2000]>,
    nametable_ram: Box<[[u8; 0x400]; 4]>,
    cpu_ram: Box<[u8; 0x800]>,
    prg_ram: Box<[u8; 0x8000]>,
    last_cpu_db_value: u8,
    last_data: u8,
    prev_hpos: Option<u16>,
    ppu_framebuffer: Box<[u32; 256 * 240]>,
}

impl SimulationState {
    pub fn new(
        nodes: Vec<Node>,
        node_counts: Vec<u8>,
        node_number_by_name: FnvHashMap<String, u16>,
        nodes_c1_c2: Vec<Vec<u16>>,
        transistors: Vec<Transistor>,
    ) -> Self {
        SimulationState {
            cycle: 0,
            nodes,
            node_number_by_name,
            node_counts,
            has_ground: false,
            has_power: false,
            group: Vec::new(),
            transistors,
            nodes_c1_c2,
            processed_nodes: Vec::new(),
            recalc_lists: [None, None],
            cur_recalc_list_index: 0,
            group_empty: true,
            step_cycle_count: 0,
            prev_ppu_ale: false,
            prev_ppu_read: true,
            prev_ppu_write: true,
            chr_address: 0,
            node_number_cache: FnvHashMap::default(),
            bit_count_cache: FnvHashMap::default(),
            last_address: 0,
            mirroring_type: MirroringType::Horizontal,
            chr_ram: Box::new([0; 0x2000]),
            nametable_ram: Box::new([[0; 0x400]; 4]),
            cpu_ram: Box::new([0; 0x800]),
            prg_ram: Box::new([0; 0x8000]),
            last_cpu_db_value: 0,
            last_data: 0,
            prev_hpos: None,
            ppu_framebuffer: Box::new([0; 256 * 240]),
        }
    }

    fn half_step(&mut self) {
        let cpu_clk0 = self.is_node_high(self.node_number_by_name["cpu_clk0"]);
        let clk = self.is_node_high(self.node_number_by_name["clk0"]);

        if clk {
            self.set_low("clk0");
        } else {
            self.set_high("clk0");
        }

        if self.step_cycle_count > 0 {
            self.step_cycle_count -= 1;
            if self.step_cycle_count == 0 {
                self.set_high("io_ce");
            }
        } else if self.is_node_high(self.node_number_by_name["cpu_ab13"])
            && !self.is_node_high(self.node_number_by_name["cpu_ab14"])
            && !self.is_node_high(self.node_number_by_name["cpu_ab15"])
            && self.is_node_high(self.node_number_by_name["cpu_clk0"])
        {
            // Simulate the 74139's logic
            self.set_low("io_ce");
            self.step_cycle_count = 11;
        }

        self.handle_chr_bus();

        if cpu_clk0 != self.is_node_high(self.node_number_by_name["cpu_clk0"]) {
            if cpu_clk0 {
                self.handle_cpu_bus_read();
            } else {
                self.handle_cpu_bus_write();
            }
        }

        if self.read_bits("pclk1", 0) > 0 {
            let hpos = self.read_bits("hpos", 0) - 2;
            if self.prev_hpos.is_none() || hpos != self.prev_hpos.unwrap() {
                let vpos = self.read_bits("vpos", 0);
                if hpos <= 255 && vpos < 240 {
                    let palette_entry = self.read_bit("pal_d0_out")
                        | (self.read_bit("pal_d1_out") << 1)
                        | (self.read_bit("pal_d2_out") << 2)
                        | (self.read_bit("pal_d3_out") << 3)
                        | (self.read_bit("pal_d4_out") << 4)
                        | (self.read_bit("pal_d5_out") << 5);
                    self.ppu_framebuffer[((vpos << 8) | hpos) as usize] =
                        PALETTE_ARGB[palette_entry as usize];
                }
                self.prev_hpos = Some(hpos);
            }
        }

        self.cycle += 1;
    }

    fn handle_cpu_bus_read(&mut self) {
        if self.is_node_high(self.node_number_by_name["cpu_rw"]) {
            let a = self.read_cpu_address_bus();
            let (d, open_bus) = self.cpu_read(a);

            if open_bus {
                self.float_bits("cpu_db", 8);
            } else {
                self.write_bits("cpu_db", 8, u16::from(d));
            }
        }
    }

    fn handle_cpu_bus_write(&mut self) {
        if !self.is_node_high(self.node_number_by_name["cpu_rw"]) {
            let a = self.read_cpu_address_bus();
            let d = self.read_cpu_data_bus();
            self.cpu_write(a, d);
        }
    }

    fn read_cpu_address_bus(&mut self) -> u16 {
        self.read_bits("cpu_ab", 16)
    }

    fn read_cpu_data_bus(&mut self) -> u8 {
        self.last_cpu_db_value = self.read_bits("cpu_db", 8) as u8;
        self.last_cpu_db_value
    }

    fn is_node_high(&self, node_number: u16) -> bool {
        self.nodes[node_number as usize].state
    }

    fn recalc_node_list(&mut self, mut recalc_list: Option<Vec<u16>>) {
        if self.processed_nodes.is_empty() {
            self.processed_nodes.extend_from_slice(&[0; NUM_NODES]);
            self.recalc_lists[0] = Some(vec![100; 0]);
            self.recalc_lists[1] = Some(vec![100; 0]);
        } else {
            self.recalc_lists[0].take();
        }

        self.cur_recalc_list_index = 0;

        for j in 0..100 {
            if j == 99 {
                panic!("Maximum loop exceeded")
            }

            for node_number in recalc_list.take().unwrap() {
                self.recalc_node(node_number);
            }

            if self.group_empty {
                return;
            }

            for node_number in self.recalc_lists[self.cur_recalc_list_index as usize]
                .as_ref()
                .unwrap()
            {
                self.processed_nodes[*node_number as usize] = 0;
            }

            recalc_list = Some(
                self.recalc_lists[self.cur_recalc_list_index as usize]
                    .take()
                    .unwrap(),
            );
            self.cur_recalc_list_index = if self.cur_recalc_list_index == 0 {
                1
            } else {
                0
            };

            self.recalc_lists[self.cur_recalc_list_index as usize].take();
            self.group_empty = true;
        }
    }

    fn set_high(&mut self, node_name: &str) {
        let node_number = self.node_number_by_name[node_name];
        self.nodes[node_number as usize].pullup = true;
        self.nodes[node_number as usize].pulldown = false;
        self.recalc_node_list(Some(vec![node_number]))
    }

    fn set_low(&mut self, node_name: &str) {
        let node_number = self.node_number_by_name[node_name];
        self.nodes[node_number as usize].pullup = false;
        self.nodes[node_number as usize].pulldown = true;
        self.recalc_node_list(Some(vec![node_number]))
    }

    fn recalc_node(&mut self, node_number: u16) {
        if node_number == NGND || node_number == NPWR {
            return;
        }

        self.get_node_group(node_number);
        let new_state = self.get_node_value();

        // TODO(perf): Get rid of clone
        for node_number in self.group.clone() {
            let node_number = node_number as usize;
            if self.nodes[node_number].state != new_state {
                self.nodes[node_number].state = new_state;
                // TODO(perf): Get rid of clone
                for i in self.nodes[node_number].gates.clone() {
                    if self.nodes[node_number].state {
                        self.turn_transistor_on(i);
                    } else {
                        self.turn_transistor_off(i);
                    }
                }
            }
        }
    }

    fn read_bits(&mut self, name: &str, mut n: u8) -> u16 {
        if name == "cycle" {
            self.cycle >> 1
        } else {
            let mut res = 0_u16;
            if n == 0 {
                let last_char = name.chars().last().unwrap();
                if last_char >= '0' && last_char <= '9' {
                    return u16::from(self.read_bit(name));
                } else {
                    if let Some(bit_count) = self.bit_count_cache.get(name) {
                        n = *bit_count;
                    } else {
                        self.node_number_cache.insert(name.to_owned(), Vec::new());
                        while self
                            .node_number_by_name
                            .contains_key(format!("{}{}", name, NUMBERS[n as usize]).as_str())
                        {
                            self.node_number_cache.get_mut(name).unwrap().push(
                                self.node_number_by_name
                                    [format!("{}{}", name, NUMBERS[n as usize]).as_str()],
                            );
                            n += 1;
                        }

                        self.bit_count_cache.insert(name.to_owned(), n);
                        if n == 0 && self.node_number_by_name.contains_key(name) {
                            self.bit_count_cache.insert(name.to_owned(), 1);
                        }
                    }

                    if n == 1 {
                        return u16::from(self.read_bit(name));
                    }
                }
                for (i, nn) in self.node_number_cache[name].iter().enumerate() {
                    res += if self.is_node_high(*nn) { 1 } else { 0 } << i;
                }
            } else {
                for i in 0..n {
                    let nn = self.node_number_by_name
                        [format!("{}{}", name, NUMBERS[i as usize]).as_str()];
                    res += (if self.is_node_high(nn) { 1 } else { 0 }) << i;
                }
            }
            res
        }
    }

    fn read_bit(&self, name: &str) -> u8 {
        self.is_node_high(self.node_number_by_name[name]) as u8
    }

    fn handle_chr_bus(&mut self) {
        let ale = self.is_node_high(self.node_number_by_name["ale"]);
        let rd = self.is_node_high(self.node_number_by_name["rd"]);
        let wr = self.is_node_high(self.node_number_by_name["wr"]);

        // rising edge of ALE
        if self.prev_ppu_ale && ale {
            self.chr_address = self.read_ppu_address_bus();
        }

        // falling edge of /RD - put bits on bus
        if self.prev_ppu_read && !rd {
            self.write_bits("db", 8, u16::from(self.ppu_read(self.chr_address)));
        }

        // rising edge of /RD - flaot the data bus
        if !self.prev_ppu_read && rd {
            self.float_bits("db", 8);
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
        if !self.is_node_high(self.node_number_by_name["rd"])
            || !self.is_node_high(self.node_number_by_name["wr"])
        {
            self.last_data = self.read_bits("db", 8) as u8;
        }
        self.last_data
    }

    fn float_bits(&mut self, name: &str, n: u16) {
        let mut recalc_nodes = Vec::with_capacity(n as usize);
        for i in 0..n {
            let node_number = self.node_number_by_name[format!("{}{}", name, i).as_str()];
            self.nodes[node_number as usize].pulldown = false;
            self.nodes[node_number as usize].pullup = false;
            recalc_nodes.push(node_number);
        }
        self.recalc_node_list(Some(recalc_nodes));
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

    fn write_bits(&mut self, name: &str, n: u16, mut x: u16) {
        let mut recalc_nodes = Vec::with_capacity(n as usize);
        for i in 0..n {
            let node_number = self.node_number_by_name[format!("{}{}", name, i).as_str()];
            if x % 2 == 0 {
                self.nodes[node_number as usize].pulldown = true;
                self.nodes[node_number as usize].pullup = false;
            } else {
                self.nodes[node_number as usize].pulldown = false;
                self.nodes[node_number as usize].pullup = true;
            }
            recalc_nodes.push(node_number);
            x >>= 1;
        }

        self.recalc_node_list(Some(recalc_nodes));
    }

    fn read_ppu_address_bus(&mut self) -> u16 {
        if self.is_node_high(self.node_number_by_name["ale"]) {
            self.last_address = self.read_bits("ab", 14);
        }

        self.last_address
    }

    fn turn_transistor_on(&mut self, i: u16) {
        let i = i as usize;
        if !self.transistors[i].on {
            self.transistors[i].on = true;
            self.add_recalc_node(self.transistors[i].c1);
        }
    }

    fn turn_transistor_off(&mut self, i: u16) {
        let i = i as usize;
        if self.transistors[i].on {
            self.transistors[i].on = false;
            self.add_recalc_node(self.transistors[i].c1);
            self.add_recalc_node(self.transistors[i].c2);
        }
    }

    fn add_recalc_node(&mut self, node_number: u16) {
        if node_number == NGND || node_number == NPWR {
            return;
        }

        if self.processed_nodes[node_number as usize] == 0 {
            self.recalc_lists[self.cur_recalc_list_index as usize]
                .as_mut()
                .unwrap()
                .push(node_number);
            self.processed_nodes[node_number as usize] = 1;
        }

        self.group_empty = false;
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
            let mut hi_area = 0_i64;
            let mut lo_area = 0_i64;
            for node_number in &self.group {
                let node = &self.nodes[*node_number as usize];
                if node.pullup {
                    return true;
                } else if node.pulldown {
                    return false;
                } else if node.state {
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
        if node_number == NGND {
            self.has_ground = true;
            return;
        }

        if node_number == NPWR {
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
            if transistor.on {
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

pub struct TransistorDefinition {
    name: String,
    gate: u16,
    c1: u16,
    c2: u16,
}

pub struct Transistor {
    on: bool,
    c1: u16,
    c2: u16,
    gate: u16,
    name: String,
}

#[derive(Clone)]
pub struct Node {
    state: bool,
    pullup: bool,
    pulldown: bool,
    floating: bool,
    area: i64,
    num: u16,
    gates: Vec<u16>,
    segs: Vec<(u16, u16)>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            state: false,
            pullup: false,
            pulldown: false,
            floating: true,
            area: 0,
            num: EMPTYNODE,
            gates: Vec::new(),
            segs: Vec::new(),
        }
    }
}

pub fn id_conversion_table() -> FnvHashMap<u16, u16> {
    let mut map = FnvHashMap::default();
    map.insert(10000 + CPU_OFFSET, 1); // vcc
    map.insert(10001 + CPU_OFFSET, 2); // vss
    map.insert(10004 + CPU_OFFSET, 1934); // reset

    map.insert(11669 + CPU_OFFSET, 772); // cpu_clk_in -> clk0

    map.insert(1013, 11819 + CPU_OFFSET); // io_db0 -> cpu_db0
    map.insert(765, 11966 + CPU_OFFSET); // db1
    map.insert(431, 12056 + CPU_OFFSET); // db2
    map.insert(87, 12091 + CPU_OFFSET); // db3
    map.insert(11, 12090 + CPU_OFFSET); // db4
    map.insert(10, 12089 + CPU_OFFSET); // db5
    map.insert(9, 12088 + CPU_OFFSET); // db6
    map.insert(8, 12087 + CPU_OFFSET); // db7

    map.insert(12, 10020 + CPU_OFFSET); // io_ab0 -> cpu_ab0
    map.insert(6, 10019 + CPU_OFFSET); // io_ab1 -> cpu_ab1
    map.insert(7, 10030 + CPU_OFFSET); // io_ab2 -> cpu_ab2

    map.insert(10331 + CPU_OFFSET, 1031); // nmi -> int
    map.insert(10092 + CPU_OFFSET, 1224); // cpu_rw -> io_rw

    map
}

fn convert_id(id: u16, conversion_table: &FnvHashMap<u16, u16>) -> u16 {
    *conversion_table.get(&id).unwrap_or(&id)
}

fn load_segment_definitions(conversion_table: &FnvHashMap<u16, u16>) -> Vec<Vec<u16>> {
    fn load_from_file<R: Read>(
        reader: R,
        segment_id_offset: u16,
        conversion_table: &FnvHashMap<u16, u16>,
    ) -> Vec<Vec<u16>> {
        BufReader::new(reader)
            .lines()
            .map(|line| {
                let values = line
                    .unwrap()
                    .split(',')
                    .map(|seg| seg.parse::<u16>().unwrap())
                    .collect::<Vec<u16>>();

                let mut seg_def = Vec::with_capacity(values.len());

                let id = values[0];
                seg_def.push(convert_id(id + segment_id_offset, conversion_table));
                if values.len() > 1 {
                    seg_def.extend_from_slice(&values[1..]);
                }

                seg_def
            })
            .collect::<Vec<Vec<u16>>>()
    }

    let mut seg_defs = load_from_file(File::open("data/segdefs.txt").unwrap(), 0, conversion_table);

    let cpu_seg_defs = load_from_file(
        File::open("data/cpusegdefs.txt").unwrap(),
        CPU_OFFSET,
        conversion_table,
    );

    seg_defs.extend(cpu_seg_defs);
    seg_defs
}
fn load_transistor_definitions(
    conversion_table: &FnvHashMap<u16, u16>,
) -> Vec<TransistorDefinition> {
    fn load_from_file<R: Read>(
        reader: R,
        name_prefix: &str,
        segment_id_offset: u16,
        conversion_table: &FnvHashMap<u16, u16>,
    ) -> Vec<TransistorDefinition> {
        BufReader::new(reader)
            .lines()
            .map(|line| {
                let values = line
                    .unwrap()
                    .split(',')
                    .map(|val| val.to_owned())
                    .collect::<Vec<String>>();
                TransistorDefinition {
                    name: format!("{}{}", name_prefix, values[0]),
                    gate: convert_id(
                        values[1].parse::<u16>().unwrap() + segment_id_offset,
                        conversion_table,
                    ),
                    c1: convert_id(
                        values[2].parse::<u16>().unwrap() + segment_id_offset,
                        conversion_table,
                    ),
                    c2: convert_id(
                        values[3].parse::<u16>().unwrap() + segment_id_offset,
                        conversion_table,
                    ),
                }
            })
            .collect()
    }

    let mut trans_defs = load_from_file(
        File::open("data/transdefs.txt").unwrap(),
        "",
        0,
        conversion_table,
    );

    let cpu_transistor_defs = load_from_file(
        File::open("data/cputransdefs.txt").unwrap(),
        "cpu_",
        CPU_OFFSET,
        conversion_table,
    );

    trans_defs.extend(cpu_transistor_defs);
    trans_defs
}

fn setup_node_names_by_number_map(node_names: &FnvHashMap<String, u16>) -> FnvHashMap<u16, String> {
    node_names.iter().map(|(k, v)| (*v, k.clone())).collect()
}

fn load_node_number_by_name_map(
    conversion_table: &FnvHashMap<u16, u16>,
) -> FnvHashMap<String, u16> {
    fn load_from_file<R: Read>(
        reader: R,
        name_prefix: &str,
        segment_id_offset: u16,
        conversion_table: &FnvHashMap<u16, u16>,
    ) -> FnvHashMap<String, u16> {
        BufReader::new(reader)
            .lines()
            .map(|line| {
                let values = line
                    .unwrap()
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .collect::<Vec<String>>();

                let id = (values[1].parse::<i64>().unwrap() + i64::from(segment_id_offset)) as u16;
                (
                    format!("{}{}", name_prefix, values[0]),
                    convert_id(id, conversion_table),
                )
            })
            .collect::<FnvHashMap<String, u16>>()
    }

    let mut node_names = load_from_file(
        File::open("data/nodenames.txt").unwrap(),
        "",
        0,
        conversion_table,
    );

    let cpu_node_names = load_from_file(
        File::open("data/cpunodenames.txt").unwrap(),
        "cpu_",
        CPU_OFFSET,
        conversion_table,
    );

    node_names.extend(cpu_node_names);

    node_names
}

#[allow(clippy::type_complexity)]
fn load_ppu_nodes() -> (Vec<Vec<(i32, i32)>>, Vec<Vec<(i32, i32)>>) {
    fn load_from_file<R: Read>(reader: R) -> Vec<Vec<(i32, i32)>> {
        BufReader::new(reader)
            .lines()
            .map(|line| {
                line.unwrap()
                    .split(',')
                    .map(|values| {
                        let value = values.split('|').collect::<Vec<&str>>();
                        (
                            value[0].parse::<i32>().unwrap(),
                            value[1].parse::<i32>().unwrap(),
                        )
                    })
                    .collect::<Vec<(i32, i32)>>()
            })
            .collect()
    }

    let palette_nodes = load_from_file(File::open("data/palettenodes.txt").unwrap());
    let sprite_nodes = load_from_file(File::open("data/spritenodes.txt").unwrap());

    (palette_nodes, sprite_nodes)
}

fn setup_nodes(segdefs: &[Vec<u16>]) -> Vec<Node> {
    let max_id = usize::from(
        segdefs
            .iter()
            .max_by(|seg1, seg2| seg1[0].cmp(&seg2[0]))
            .unwrap()[0],
    );
    let mut nodes = vec![Node::default(); max_id + 1];
    for seg in segdefs.iter() {
        let w = seg[0];
        let w_idx = w as usize;
        if nodes[w_idx].num == EMPTYNODE {
            nodes[w_idx].num = w as _;
            nodes[w_idx].pullup = seg[1] == 1;
            nodes[w_idx].state = false;
            nodes[w_idx].area = 0;
        }

        if w == NGND || w == NPWR {
            continue;
        }

        let mut area = i64::from(seg[seg.len() - 2]) * i64::from(seg[4])
            - i64::from(seg[3]) * i64::from(seg[seg.len() - 1]);
        let mut j = 3;
        loop {
            if j + 4 >= seg.len() {
                break;
            }

            area += i64::from(seg[j]) * i64::from(seg[j + 3])
                - i64::from(seg[j + 2]) * i64::from(seg[j - 1]);
            j += 2;
        }

        nodes[w_idx].area += area.abs();
        nodes[w_idx].segs.push((seg[3], *seg.last().unwrap()))
    }
    nodes
}

#[allow(clippy::type_complexity)]
fn setup_transistors(
    nodes: &mut Vec<Node>,
    trans_defs: Vec<TransistorDefinition>,
) -> (
    Vec<Transistor>,
    Vec<u8>,
    Vec<Vec<u16>>,
    FnvHashMap<String, u16>,
) {
    const MAX_NODES: usize = 34000;
    const MAX_C1_C2: usize = 95;
    let mut node_counts = vec![0_u8; MAX_NODES];
    let mut nodes_c1_c2 = vec![vec![0_u16; MAX_C1_C2]; MAX_NODES];
    let mut transistors = Vec::new();
    let mut transistor_index_by_name = FnvHashMap::<String, u16>::default();
    for (i, trans_def) in trans_defs.into_iter().enumerate() {
        let mut c1 = trans_def.c1;
        let mut c2 = trans_def.c2;
        let name = trans_def.name;
        let gate = trans_def.gate;

        if c1 == NGND {
            c1 = c2;
            c2 = NGND;
        }

        if c1 == NPWR {
            c1 = c2;
            c2 = NPWR;
        }

        nodes[gate as usize].gates.push(i as u16);
        if c1 != NPWR && c1 != NGND {
            nodes_c1_c2[c1 as usize][node_counts[c1 as usize] as usize] = i as u16;
            node_counts[c1 as usize] += 1;
        }

        if c2 != NPWR && c2 != NGND {
            nodes_c1_c2[c2 as usize][node_counts[c2 as usize] as usize] = i as u16;
            node_counts[c2 as usize] += 1;
        }

        transistors.push(Transistor {
            c1,
            c2,
            gate,
            on: false,
            name: name.clone(),
        });
        transistor_index_by_name.insert(name, i as u16);
    }

    (
        transistors,
        node_counts,
        nodes_c1_c2,
        transistor_index_by_name,
    )
}

fn main() {
    println!("woot");
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{fmt::Write, fs::File};

    fn string_from_zip(file: &str) -> String {
        let reader = File::open(file).unwrap();
        let mut zip = zip::ZipArchive::new(reader).unwrap();
        let mut orig_file = zip.by_index(0).unwrap();
        let mut reference_data = String::new();
        orig_file.read_to_string(&mut reference_data).unwrap();
        reference_data
    }

    #[test]
    fn conversion_table_reference_test() {
        let reference_data = string_from_zip("test_data/conversion_table_reference.zip");
        let conversion_table = id_conversion_table();
        let mut conversion_table: Vec<(u16, u16)> =
            conversion_table.into_iter().map(|v| v).collect();

        conversion_table.sort_by(|(a1, _), (a2, _)| a1.cmp(a2));

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", conversion_table.len()).as_str())
            .unwrap();
        conversion_table.iter().for_each(|(a, b)| {
            processed_data
                .write_str(format!("{},{}\r\n", a, b).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn segment_definitions_reference_test() {
        let reference_data = string_from_zip("test_data/segment_definitions_reference.zip");
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", seg_defs.len()).as_str())
            .unwrap();

        seg_defs.iter().for_each(|seg| {
            let line = seg
                .iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join(",");
            processed_data
                .write_str(format!("{}\r\n", line).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn transistor_definition_reference_test() {
        let reference_data = string_from_zip("test_data/transistor_definition_reference.zip");
        let conversion_table = id_conversion_table();
        let mut trans_defs = load_transistor_definitions(&conversion_table);

        trans_defs.sort_by(|td1, td2| td1.name.cmp(&td2.name));

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", trans_defs.len()).as_str())
            .unwrap();
        trans_defs.iter().for_each(|td| {
            processed_data
                .write_str(format!("{}:{},{},{}\r\n", td.name, td.c1, td.c2, td.gate).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn node_names_reference_test() {
        let reference_data = string_from_zip("test_data/node_names_reference.zip");
        let conversion_table = id_conversion_table();
        let node_names: std::collections::BTreeSet<_> =
            load_node_number_by_name_map(&conversion_table)
                .iter()
                .map(|(k, v)| format!("{},{}", k, v))
                .collect();

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", node_names.len()).as_str())
            .unwrap();
        node_names.iter().for_each(|l| {
            processed_data
                .write_str(format!("{}\r\n", l).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn sprite_nodes_reference_test() {
        let reference_data = string_from_zip("test_data/sprite_nodes_reference.zip");
        let (_, sprite_nodes) = load_ppu_nodes();

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", sprite_nodes.len()).as_str())
            .unwrap();
        sprite_nodes.iter().for_each(|seg| {
            let line = seg
                .iter()
                .map(|(i, j)| format!("{},{}", i, j))
                .collect::<Vec<String>>()
                .join("|");
            processed_data
                .write_str(format!("{}\r\n", line).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn palette_nodes_reference_test() {
        let reference_data = string_from_zip("test_data/palette_nodes_reference.zip");
        let (palette_nodes, _) = load_ppu_nodes();

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", palette_nodes.len()).as_str())
            .unwrap();
        palette_nodes.iter().for_each(|seg| {
            let line = seg
                .iter()
                .map(|(i, j)| format!("{},{}", i, j))
                .collect::<Vec<String>>()
                .join("|");
            processed_data
                .write_str(format!("{}\r\n", line).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn transistors_reference_test() {
        let reference_data = string_from_zip("test_data/transistors_reference.zip");
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let trans_defs = load_transistor_definitions(&conversion_table);
        let mut nodes = setup_nodes(&seg_defs);

        let (transistors, _, _, _) = setup_transistors(&mut nodes, trans_defs);

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", transistors.len()).as_str())
            .unwrap();
        transistors.iter().for_each(|trans| {
            processed_data
                .write_str(
                    format!(
                        "{},{},{},{},{}\r\n",
                        trans.name,
                        trans.c1,
                        trans.c2,
                        trans.gate,
                        if trans.on { 1 } else { 0 }
                    )
                    .as_str(),
                )
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn node_counts_reference_test() {
        let reference_data = string_from_zip("test_data/node_counts_reference.zip");
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let trans_defs = load_transistor_definitions(&conversion_table);
        let mut nodes = setup_nodes(&seg_defs);

        let (_, node_counts, _, _) = setup_transistors(&mut nodes, trans_defs);

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", node_counts.len()).as_str())
            .unwrap();
        node_counts.iter().for_each(|node| {
            processed_data
                .write_str(format!("{}\r\n", node).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn nodes_c1_c2_reference_test() {
        let reference_data = string_from_zip("test_data/nodes_c1_c2_reference.zip");
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let trans_defs = load_transistor_definitions(&conversion_table);
        let mut nodes = setup_nodes(&seg_defs);
        let (_, _, nodes_c1_c2, _) = setup_transistors(&mut nodes, trans_defs);

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", nodes_c1_c2.len()).as_str())
            .unwrap();
        for nodes in nodes_c1_c2 {
            let line = nodes
                .iter()
                .map(|n| format!("{}", n))
                .collect::<Vec<String>>()
                .join(",");
            processed_data
                .write_str(format!("{}\r\n", line).as_str())
                .unwrap();
        }
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn transistor_index_by_name_reference_test() {
        let reference_data = string_from_zip("test_data/transistor_index_by_name_reference.zip");
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let trans_defs = load_transistor_definitions(&conversion_table);
        let mut nodes = setup_nodes(&seg_defs);
        let (_, _, _, transistor_index_by_name) = setup_transistors(&mut nodes, trans_defs);

        let mut transistor_index_by_name: Vec<(String, u16)> =
            transistor_index_by_name.into_iter().map(|v| v).collect();

        transistor_index_by_name.sort_by(|(a1, _b1), (a2, _b2)| a1.cmp(a2));

        let mut processed_data = String::new();
        processed_data
            .write_str(format!("Entries: {}\r\n", transistor_index_by_name.len()).as_str())
            .unwrap();
        transistor_index_by_name.iter().for_each(|(a, b)| {
            processed_data
                .write_str(format!("{},{}\r\n", a, b).as_str())
                .unwrap();
        });
        assert_eq!(reference_data, processed_data);
    }

    #[test]
    fn node_names_length_constant_test() {
        // Ensure that the NUM_NODES constant always reflects the number of processed nodes.
        let conversion_table = id_conversion_table();
        let seg_defs = load_segment_definitions(&conversion_table);
        let nodes = setup_nodes(&seg_defs);
        assert_eq!(nodes.len(), NUM_NODES);
    }
}
