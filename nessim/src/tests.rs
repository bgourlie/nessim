use crate::{MemoryType, SimulationState, NUM_NODES};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

const NUM_TRANSISTORS: usize = 27703;

#[test]
fn reference_tests() {
    use std::fs::File;
    let mut sim = SimulationState::new();

    let reader = File::open("test_data/reference_samples.zip").unwrap();
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut file = zip.by_index(0).unwrap();

    // Test initial state
    verify_state(&sim, &mut file);

    sim.init(false);

    // Test post-init state
    verify_state(&sim, &mut file);

    let num_steps = file.read_i32::<LittleEndian>().unwrap();
    let half_cycles_per_step = file.read_i32::<LittleEndian>().unwrap();

    let mut prg_ram = vec![0_u8; 0x8000];
    let mut chr_ram = vec![0_u8; 0x2000];
    file.read_exact(&mut prg_ram).unwrap();
    file.read_exact(&mut chr_ram).unwrap();

    sim.set_memory_state(MemoryType::ChrRam, &chr_ram);
    sim.set_memory_state(MemoryType::PrgRam, &prg_ram);

    // Test post load state
    verify_ram_state(&sim, &prg_ram, &chr_ram);
    verify_state(&sim, &mut file);

    for _ in 0..num_steps {
        for _ in 0..half_cycles_per_step {
            sim.half_step();
        }

        let mut prg_ram = vec![0_u8; 0x8000];
        let mut chr_ram = vec![0_u8; 0x2000];
        file.read_exact(&mut prg_ram).unwrap();
        file.read_exact(&mut chr_ram).unwrap();

        // Verifying state at step
        verify_ram_state(&sim, &prg_ram, &chr_ram);
        verify_state(&sim, &mut file);
    }
}

fn verify_ram_state(sim: &SimulationState, reference_prg: &[u8], reference_chr: &[u8]) {
    assert_eq!(reference_prg.len(), sim.prg_ram.len());
    assert_eq!(reference_chr.len(), sim.chr_ram.len());

    for (i, byte) in reference_prg.iter().enumerate() {
        assert_eq!(
            *byte, sim.prg_ram[i],
            "PRG RAM value mismatch at index {}",
            i
        );
    }

    for (i, byte) in reference_chr.iter().enumerate() {
        assert_eq!(
            *byte, sim.chr_ram[i],
            "CHR RAM value mismatch at index {}",
            i
        );
    }
}

fn verify_state<R: Read>(sim: &SimulationState, reader: &mut R) {
    let mut node_bytes = vec![0_u8; 16501];
    reader.read_exact(&mut node_bytes).unwrap();
    let mut reference_nodes = Vec::new();
    for i in 0..NUM_NODES {
        let byte_index = i / 2;
        let bit_position = (i % 2) * 4;
        let bits = node_bytes[byte_index] >> bit_position;
        let floating = bits & 0b0000_0001 > 0;
        let pulldown = bits & 0b0000_0010 > 0;
        let pullup = bits & 0b0000_0100 > 0;
        let state = bits & 0b0000_1000 > 0;
        reference_nodes.push((floating, pulldown, pullup, state));
    }

    assert_eq!(
        reference_nodes.len(),
        sim.nodes.len(),
        "reference node count != node count"
    );

    let mut transistor_bytes = vec![0_u8; 3463];
    reader.read_exact(&mut transistor_bytes).unwrap();
    let mut reference_transistors = Vec::new();
    for i in 0..NUM_TRANSISTORS {
        let byte_index = i / 8;
        let bit_position = i % 8;
        let on = (transistor_bytes[byte_index] >> bit_position) & 1 > 0;
        reference_transistors.push(on);
    }

    assert_eq!(
        reference_transistors.len(),
        sim.transistors.len(),
        "reference transistors count {} != transistors count {}",
        reference_transistors.len(),
        sim.transistors.len()
    );

    for (i, reference_node) in reference_nodes.iter().enumerate() {
        let (floating, pulldown, pullup, state) = *reference_node;
        let node = &sim.nodes[i];

        assert_eq!(
            floating,
            node.floating.get(),
            "Floating expected was {} but was {} at node {}",
            floating,
            node.floating.get(),
            i
        );

        assert_eq!(
            pullup,
            node.pullup.get(),
            "Pullup expected was {} but was {} at node {}",
            pullup,
            node.pullup.get(),
            i
        );

        assert_eq!(
            pullup,
            node.pullup.get(),
            "Pulldown expected was {} but was {} at node {}",
            pulldown,
            node.pulldown.get(),
            i
        );

        assert_eq!(
            state,
            node.state.get(),
            "State expected was {} but was {} at node {}",
            state,
            node.state.get(),
            i
        );
    }
    for (i, reference_transistor) in reference_transistors.iter().enumerate() {
        assert_eq!(
            *reference_transistor,
            sim.transistors[i].on.get(),
            "Expected transistor {} to be {}, was {}",
            i,
            reference_transistor,
            sim.transistors[i].on.get()
        );
    }
}
