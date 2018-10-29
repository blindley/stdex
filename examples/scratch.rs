#![allow(unused_imports, dead_code)]
extern crate stdex;
use stdex::{
    random::MT19937_32,
    collections::binary_heap::BinaryHeap
};

fn main() {
    let mut heap = BinaryHeap::min_heap();
    let mut rng = MT19937_32::new();
    for _ in 0..20 {
        heap.push(rng.generate());
    }

    while let Some(value) = heap.pop() {
        println!("{}", value);
    }
}