use std::cell::Cell;

pub struct ProcessedNodesSet {
    set: Vec<Cell<u8>>,
}

impl ProcessedNodesSet {
    pub fn new(node_count: usize) -> Self {
        let elem_count = (node_count / 8) + (if node_count % 8 != 0 { 1 } else { 0 });
        ProcessedNodesSet {
            set: vec![Cell::new(0); elem_count],
        }
    }

    pub fn contains(&self, node_number: u16) -> bool {
        let byte_index = (node_number / 8) as usize;
        let bit_index = node_number % 8;
        let mask = 1 << bit_index;
        self.set[byte_index].get() & mask > 0
    }

    pub fn set(&self, node_number: u16) {
        let byte_index = (node_number / 8) as usize;
        let bit_index = node_number % 8;
        let mask = 1 << bit_index;
        let byte = self.set[byte_index].get();
        self.set[byte_index].set(byte | mask);
    }

    pub fn clear(&self, nodes: &[u16]) {
        for node_number in nodes.iter() {
            let byte_index = (node_number / 8) as usize;
            let bit_index = node_number % 8;
            let mask = 1 << bit_index;
            let byte = self.set[byte_index].get();
            self.set[byte_index].set(byte & !(byte & mask));
        }
    }
}

#[test]
fn test_create() {
    let set = ProcessedNodesSet::new(8);
    assert_eq!(1, set.set.len());

    let set = ProcessedNodesSet::new(9);
    assert_eq!(2, set.set.len());

    let set = ProcessedNodesSet::new(16);
    assert_eq!(2, set.set.len());

    let set = ProcessedNodesSet::new(17);
    assert_eq!(3, set.set.len());
}

#[test]
fn test_insert() {
    let set = ProcessedNodesSet::new(20);
    assert_eq!(false, set.contains(0));
    assert_eq!(false, set.contains(1));
    assert_eq!(false, set.contains(2));
    set.set(0);
    assert_eq!(true, set.contains(0));
    assert_eq!(false, set.contains(1));
    assert_eq!(false, set.contains(2));
    set.set(2);
    assert_eq!(true, set.contains(0));
    assert_eq!(false, set.contains(1));
    assert_eq!(true, set.contains(2));
    set.set(9);
    assert_eq!(true, set.contains(9));
}

#[test]
fn test_clear() {
    let set = ProcessedNodesSet::new(20);
    set.set(0);
    set.set(1);
    set.set(8);
    set.set(9);

    assert_eq!(true, set.contains(0));
    assert_eq!(true, set.contains(1));
    assert_eq!(false, set.contains(2));
    assert_eq!(false, set.contains(3));
    assert_eq!(false, set.contains(4));
    assert_eq!(false, set.contains(5));
    assert_eq!(false, set.contains(6));
    assert_eq!(false, set.contains(7));
    assert_eq!(true, set.contains(8));
    assert_eq!(true, set.contains(9));
    assert_eq!(false, set.contains(10));

    set.clear(&[1, 2, 9]);

    assert_eq!(true, set.contains(0));
    assert_eq!(false, set.contains(1));
    assert_eq!(false, set.contains(2));
    assert_eq!(false, set.contains(3));
    assert_eq!(false, set.contains(4));
    assert_eq!(false, set.contains(5));
    assert_eq!(false, set.contains(6));
    assert_eq!(false, set.contains(7));
    assert_eq!(true, set.contains(8));
    assert_eq!(false, set.contains(9));
    assert_eq!(false, set.contains(10));

    set.clear(&[0, 8]);

    assert_eq!(false, set.contains(0));
    assert_eq!(false, set.contains(1));
    assert_eq!(false, set.contains(2));
    assert_eq!(false, set.contains(3));
    assert_eq!(false, set.contains(4));
    assert_eq!(false, set.contains(5));
    assert_eq!(false, set.contains(6));
    assert_eq!(false, set.contains(7));
    assert_eq!(false, set.contains(8));
    assert_eq!(false, set.contains(9));
    assert_eq!(false, set.contains(10));
}
