use fnv::FnvHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

const EMPTYNODE: u16 = 65535;
const CPU_OFFSET: u16 = 13000;
const NGND: u16 = 2;
const NPWR: u16 = 1;

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

struct DataDefinitions {
    segdefs: Vec<Vec<i32>>,
    transdefs: Vec<TransistorDefinition>,
    nodenames: FnvHashMap<String, u16>,
    pallete_nodes: Vec<Vec<(u16, u16)>>,
    sprite_nodes: Vec<Vec<(u16, u16)>>,
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

fn load_segment_definitions<R: Read>(
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

fn load_transistor_definitions<R: Read>(
    reader: R,
    name_prefix: &'static str,
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

fn load_node_names<R: Read>(
    reader: R,
    name_prefix: &'static str,
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

fn load_ppu_nodes<R: Read>(reader: R) -> Vec<Vec<(i32, i32)>> {
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
        // TODO: area will overflow here if using i32, is this a bug in the C++ source?
        nodes[w_idx].area += area.abs();
        nodes[w_idx].segs.push((seg[3], *seg.last().unwrap()))
    }
    nodes
}

fn main() {
    let conversion_table = id_conversion_table();
    let seg_defs = {
        let mut seg_defs = load_segment_definitions(
            File::open("data/segdefs.txt").unwrap(),
            0,
            &conversion_table,
        );

        let cpu_seg_defs = load_segment_definitions(
            File::open("data/cpusegdefs.txt").unwrap(),
            CPU_OFFSET,
            &conversion_table,
        );

        seg_defs.extend(cpu_seg_defs);
        seg_defs
    };

    let trans_defs = {
        let mut trans_defs = load_transistor_definitions(
            File::open("data/transdefs.txt").unwrap(),
            "",
            0,
            &conversion_table,
        );

        let cpu_transistor_defs = load_transistor_definitions(
            File::open("data/cputransdefs.txt").unwrap(),
            "cpu_",
            CPU_OFFSET,
            &conversion_table,
        );

        trans_defs.extend(cpu_transistor_defs);
        trans_defs
    };

    let node_names = {
        let mut node_names = load_node_names(
            File::open("data/nodenames.txt").unwrap(),
            "",
            0,
            &conversion_table,
        );

        let cpu_node_names = load_node_names(
            File::open("data/cpunodenames.txt").unwrap(),
            "cpu_",
            CPU_OFFSET,
            &conversion_table,
        );

        node_names.extend(cpu_node_names);

        node_names
    };

    {
        let mut conversion_table: Vec<(u16, u16)> =
            conversion_table.into_iter().map(|v| v).collect();

        conversion_table.sort_by(|(a1, b1), (a2, b2)| a1.cmp(a2));

        let mut file = File::create("conversion_table.txt").unwrap();
        file.write(format!("Entries: {}\n", conversion_table.len()).as_bytes())
            .unwrap();
        conversion_table.iter().for_each(|(a, b)| {
            file.write(format!("{},{}\n", a, b).as_bytes()).unwrap();
        });
    }

    {
        let node_names: std::collections::BTreeSet<_> = node_names
            .iter()
            .map(|(k, v)| format!("{},{}", k, v))
            .collect();

        let mut file = File::create("node_names.txt").unwrap();
        file.write(format!("Entries: {}\n", node_names.len()).as_bytes())
            .unwrap();
        node_names.iter().for_each(|l| {
            file.write(format!("{}\n", l).as_bytes()).unwrap();
        });
    }

    let palette_nodes = load_ppu_nodes(File::open("data/palettenodes.txt").unwrap());
    let sprite_nodes = load_ppu_nodes(File::open("data/spritenodes.txt").unwrap());

    let nodes = setup_nodes(&seg_defs);
    println!("segdef entries: {}", seg_defs.len());
    println!("transdef entries: {}", trans_defs.len());
    println!("palette_nodes entries: {}", palette_nodes.len());
    println!("sprite_nodes entries: {}", sprite_nodes.len());
    println!("nodes entries: {}", nodes.len());
}
