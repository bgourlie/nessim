use super::*;
use crate::consts::NUM_NODES;
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
fn node_names_length_constant_test() {
    // Ensure that the NUM_NODES constant always reflects the number of processed nodes.
    let conversion_table = id_conversion_table();
    let seg_defs = load_segment_definitions(&conversion_table);
    let nodes = setup_nodes(&seg_defs);
    assert_eq!(nodes.len(), NUM_NODES);
}
