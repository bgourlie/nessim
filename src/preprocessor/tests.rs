use super::*;
use crate::consts::*;
use std::fs::File;

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
    let mut conversion_table: Vec<(u16, u16)> = conversion_table.into_iter().map(|v| v).collect();

    conversion_table.sort_by(|(a1, _), (a2, _)| a1.cmp(a2));

    let processed_data = conversion_table
        .iter()
        .map(|(a, b)| format!("{},{}", a, b))
        .collect::<Vec<String>>()
        .join("\r\n");

    assert_eq!(reference_data, processed_data);
}

#[test]
fn segment_definitions_reference_test() {
    let reference_data = string_from_zip("test_data/segment_definitions_reference.zip");
    let conversion_table = id_conversion_table();
    let seg_defs = load_segment_definitions(&conversion_table);

    let processed_data = seg_defs
        .iter()
        .map(|seg| {
            seg.iter()
                .map(|s| format!("{}", s))
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn transistor_definition_reference_test() {
    let reference_data = string_from_zip("test_data/transistor_definition_reference.zip");
    let conversion_table = id_conversion_table();
    let mut trans_defs = load_transistor_definitions(&conversion_table);

    trans_defs.sort_by(|td1, td2| td1.name.cmp(&td2.name));

    let processed_data = trans_defs
        .iter()
        .map(|td| format!("{}:{},{},{}", td.name, td.c1, td.c2, td.gate))
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn node_names_reference_test() {
    let reference_data = string_from_zip("test_data/node_names_reference.zip");
    let conversion_table = id_conversion_table();
    let node_names: std::collections::BTreeSet<_> = load_node_number_by_name_map(&conversion_table)
        .iter()
        .map(|(k, v)| format!("{},{}", k, v))
        .collect();

    let processed_data = node_names
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn sprite_nodes_reference_test() {
    let reference_data = string_from_zip("test_data/sprite_nodes_reference.zip");
    let (_, sprite_nodes) = load_ppu_nodes();

    let processed_data = sprite_nodes
        .iter()
        .map(|seg| {
            seg.iter()
                .map(|(i, j)| format!("{},{}", i, j))
                .collect::<Vec<String>>()
                .join("|")
        })
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn palette_nodes_reference_test() {
    let reference_data = string_from_zip("test_data/palette_nodes_reference.zip");
    let (palette_nodes, _) = load_ppu_nodes();

    let processed_data = palette_nodes
        .iter()
        .map(|seg| {
            seg.iter()
                .map(|(i, j)| format!("{},{}", i, j))
                .collect::<Vec<String>>()
                .join("|")
        })
        .collect::<Vec<String>>()
        .join("\r\n");
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

    let processed_data = transistors
        .iter()
        .map(|trans| {
            format!(
                "{},{},{},{},{}",
                trans.name,
                trans.c1,
                trans.c2,
                trans.gate,
                if trans.on.get() { 1 } else { 0 }
            )
        })
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn node_area_reference_test() {
    let reference_data = string_from_zip("test_data/node_area_reference.zip");
    let conversion_table = id_conversion_table();
    let seg_defs = load_segment_definitions(&conversion_table);
    let nodes = setup_nodes(&seg_defs);

    let processed_data = nodes
        .iter()
        .map(|node| format!("{}:{}", node.num, node.area))
        .collect::<Vec<String>>()
        .join("\r\n");
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

    let processed_data = node_counts
        .iter()
        .map(|node| format!("{}", node))
        .collect::<Vec<String>>()
        .join("\r\n");
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

    let processed_data = nodes_c1_c2
        .iter()
        .map(|nodes| {
            nodes
                .iter()
                .map(|n| format!("{}", n))
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\r\n");

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

    let processed_data = transistor_index_by_name
        .iter()
        .map(|(a, b)| format!("{},{}", a, b))
        .collect::<Vec<String>>()
        .join("\r\n");
    assert_eq!(reference_data, processed_data);
}

#[test]
fn node_constant_tests() {
    // Ensure that the NUM_NODES constant always reflects the number of processed nodes.
    let conversion_table = id_conversion_table();
    let seg_defs = load_segment_definitions(&conversion_table);
    let nodes = setup_nodes(&seg_defs);
    let node_number_by_name_map = load_node_number_by_name_map(&conversion_table);

    assert_eq!(nodes.len(), NUM_NODES);
    assert_eq!(
        node_number_by_name_map["clk0"], NODE_CLK0,
        "Wrong CLK0 constant value"
    );
    assert_eq!(
        node_number_by_name_map["res"], NODE_RESET,
        "Wrong RESET constant value"
    );
    assert_eq!(
        node_number_by_name_map["io_ce"], NODE_IO_CE,
        "Wrong IO_CE constant value"
    );
    assert_eq!(
        node_number_by_name_map["int"], NODE_INT,
        "Wrong INT constant value"
    );
    assert_eq!(
        node_number_by_name_map["ale"], NODE_ALE,
        "Wrong ALE constant value"
    );
    assert_eq!(
        node_number_by_name_map["rd"], NODE_RD,
        "Wrong RD constant value"
    );
    assert_eq!(
        node_number_by_name_map["wr"], NODE_WR,
        "Wrong WR constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_so"], NODE_CPU_SO,
        "Wrong CPU_SO constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_irq"], NODE_CPU_IRQ,
        "Wrong CPU_IRQ constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_nmi"], NODE_CPU_NMI,
        "Wrong CPU_NMI constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_clk0"], NODE_CPU_CLK0,
        "Wrong CPU_CLK0 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab0"], NODE_CPU_AB0,
        "Wrong CPU_AB0 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab1"], NODE_CPU_AB1,
        "Wrong CPU_AB1 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab2"], NODE_CPU_AB2,
        "Wrong CPU_AB2 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab3"], NODE_CPU_AB3,
        "Wrong CPU_AB3 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab4"], NODE_CPU_AB4,
        "Wrong CPU_AB4 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab5"], NODE_CPU_AB5,
        "Wrong CPU_AB5 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab6"], NODE_CPU_AB6,
        "Wrong CPU_AB6 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab7"], NODE_CPU_AB7,
        "Wrong CPU_AB7 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab8"], NODE_CPU_AB8,
        "Wrong CPU_AB8 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab9"], NODE_CPU_AB9,
        "Wrong CPU_AB9 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab10"], NODE_CPU_AB10,
        "Wrong CPU_AB10 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab11"], NODE_CPU_AB11,
        "Wrong CPU_AB11 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab12"], NODE_CPU_AB12,
        "Wrong CPU_AB12 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab13"], NODE_CPU_AB13,
        "Wrong CPU_AB13 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab14"], NODE_CPU_AB14,
        "Wrong CPU_AB14 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_ab15"], NODE_CPU_AB15,
        "Wrong CPU_AB15 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db0"], NODE_CPU_DB0,
        "Wrong CPU_DB0 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db1"], NODE_CPU_DB1,
        "Wrong CPU_DB1 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db2"], NODE_CPU_DB2,
        "Wrong CPU_DB2 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db3"], NODE_CPU_DB3,
        "Wrong CPU_DB3 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db4"], NODE_CPU_DB4,
        "Wrong CPU_DB4 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db5"], NODE_CPU_DB5,
        "Wrong CPU_DB5 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db6"], NODE_CPU_DB6,
        "Wrong CPU_DB6 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_db7"], NODE_CPU_DB7,
        "Wrong CPU_DB7 constant value"
    );
    assert_eq!(
        node_number_by_name_map["cpu_rw"], NODE_CPU_RW,
        "Wrong CPU_RW constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d0_out"], NODE_PAL_D0_OUT,
        "Wrong PAL_D0_OUT constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d1_out"], NODE_PAL_D1_OUT,
        "Wrong PAL_D1_OUT constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d2_out"], NODE_PAL_D2_OUT,
        "Wrong PAL_D2_OUT constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d3_out"], NODE_PAL_D3_OUT,
        "Wrong PAL_D3_OUT constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d4_out"], NODE_PAL_D4_OUT,
        "Wrong PAL_D4_OUT constant value"
    );
    assert_eq!(
        node_number_by_name_map["pal_d5_out"], NODE_PAL_D5_OUT,
        "Wrong PAL_D5_OUT constant value"
    );
}
