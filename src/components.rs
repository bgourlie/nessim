use crate::consts::EMPTYNODE;

#[derive(Clone)]
pub struct Node {
    pub state: bool,
    pub pullup: bool,
    pub pulldown: bool,
    pub floating: bool,
    pub area: i64,
    pub num: u16,
    pub gates: Vec<u16>,
    pub segs: Vec<(u16, u16)>,
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

pub struct Transistor {
    pub on: bool,
    pub c1: u16,
    pub c2: u16,
    pub gate: u16,
    pub name: String,
}
