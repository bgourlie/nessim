use crate::consts::EMPTYNODE;
use std::cell::Cell;

#[derive(Clone)]
pub struct NodeDefinition {
    pub area: u64,
    pub num: u16,
    pub pullup: bool,
    pub state: bool,
    pub gates: Vec<u16>,
    pub segs: Vec<(u16, u16)>,
}

impl Default for NodeDefinition {
    fn default() -> Self {
        NodeDefinition {
            area: 0,
            num: EMPTYNODE,
            pullup: false,
            state: false,
            gates: Vec::new(),
            segs: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub state: Cell<bool>,
    pub pullup: Cell<bool>,
    pub pulldown: Cell<bool>,
    pub floating: Cell<bool>,
    pub area: u64,
    pub gates: Vec<u16>,
}

impl Node {
    pub fn new(def: NodeDefinition) -> Self {
        Node {
            state: Cell::new(def.state),
            pullup: Cell::new(def.pullup),
            pulldown: Cell::new(false),
            floating: Cell::new(true),
            area: def.area,
            gates: def.gates,
        }
    }
}

pub struct Transistor {
    pub on: Cell<bool>,
    pub c1: u16,
    pub c2: u16,
    pub gate: u16,
}
