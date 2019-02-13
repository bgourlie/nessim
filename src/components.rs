use crate::consts::EMPTYNODE;
use std::cell::Cell;

#[derive(Clone)]
pub struct Node {
    pub state: Cell<bool>,
    pub pullup: Cell<bool>,
    pub pulldown: Cell<bool>,
    pub floating: Cell<bool>,
    pub area: i64,
    pub num: u16,
    pub gates: Vec<u16>,
    pub segs: Vec<(u16, u16)>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            state: Cell::new(false),
            pullup: Cell::new(false),
            pulldown: Cell::new(false),
            floating: Cell::new(true),
            area: 0,
            num: EMPTYNODE,
            gates: Vec::new(),
            segs: Vec::new(),
        }
    }
}

pub struct Transistor {
    pub on: Cell<bool>,
    pub c1: u16,
    pub c2: u16,
    pub gate: u16,
    pub name: String,
}
