use fnv::FnvHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

const EMPTYNODE: u16 = 65535;
const CPU_OFFSET: u16 = 13000;
const NGND: u16 = 2;
const NPWR: u16 = 1;

pub struct SimulationState<'a> {
    half_steps: u64,
    nodes: Vec<Node>,
    cpu_clk0_index: usize,
    clk0_index: usize,
}

impl<'a> SimulationState<'a> {
    pub fn new(nodes: Vec<Node>, node_name_to_index_map: &FnvHashMap<String, u16>) -> Self {
        let cpu_clk0_index = node_name_to_index_map["cpu_clk0"] as usize;
        let clk0_index = node_name_to_index_map["clk0"] as usize;
        SimulationState {
            half_steps: 0,
            nodes,
            cpu_clk0_index,
            clk0_index,
        }
    }

    pub fn set_high(&mut self, node_index: usize) {
        self.nodes[self.clk0_index].pullup = true;
        self.nodes[self.clk0_index].pulldown = false;
//        recalcNodeList(shared_ptr<vector<uint16_t>>(new vector<uint16_t>{ nn }));
    }

    pub fn set_low(&mut self, node_index: usize) {
        self.nodes[self.clk0_index].pullup = false;
        self.nodes[self.clk0_index].pulldown = true;
//        recalcNodeList(shared_ptr<vector<uint16_t>>(new vector<uint16_t>{ nn }));
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
//
fn load_node_names(conversion_table: &FnvHashMap<u16, u16>) -> FnvHashMap<String, u16> {
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
    let mut node_count = vec![0_u8; MAX_NODES];
    let mut node_c1_c2 = vec![vec![0_u16; MAX_C1_C2]; MAX_NODES];
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
            node_c1_c2[c1 as usize][node_count[c1 as usize] as usize] = i as u16;
            node_count[c1 as usize] += 1;
        }

        if c2 != NPWR && c2 != NGND {
            node_c1_c2[c2 as usize][node_count[c2 as usize] as usize] = i as u16;
            node_count[c2 as usize] += 1;
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
        node_count,
        node_c1_c2,
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
        let node_names: std::collections::BTreeSet<_> = load_node_names(&conversion_table)
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
}
