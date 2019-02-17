use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct RecalcSwapList {
    lists: [Rc<RefCell<Vec<u16>>>; 2],
    indices: (u8, u8),
}

impl RecalcSwapList {
    pub fn new() -> Self {
        RecalcSwapList {
            lists: [
                Rc::new(RefCell::new(Vec::with_capacity(14330))), // init() recalculates all nodes
                Rc::new(RefCell::new(Vec::with_capacity(5120))),
            ],
            indices: (0, 1),
        }
    }

    pub fn init(&mut self, nodes: &[u16]) {
        self.lists[0].borrow_mut().clear();
        self.lists[0].borrow_mut().extend_from_slice(nodes);
        self.indices = (0, 1);
    }

    pub fn cur_list(&self) -> Rc<RefCell<Vec<u16>>> {
        let (cur_list, _) = self.indices;
        self.lists[cur_list as usize].clone()
    }

    pub fn push_next_list(&self, node: u16) {
        let (_, next_list) = self.indices;
        self.lists[next_list as usize].borrow_mut().push(node);
    }

    pub fn next_list(&self) -> Ref<Vec<u16>> {
        let (_, next_list) = self.indices;
        self.lists[next_list as usize].borrow()
    }

    pub fn is_next_list_empty(&self) -> bool {
        let (_, next_list) = self.indices;
        self.lists[next_list as usize].borrow().is_empty()
    }

    pub fn swap(&mut self) {
        let (cur_list, next_list) = self.indices;
        self.lists[cur_list as usize].borrow_mut().clear();
        self.indices = (next_list, cur_list);
    }
}
