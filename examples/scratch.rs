#![allow(unused_imports, dead_code)]
extern crate stdex;
use stdex::{
    random::MT19937_32,
    collections::BinaryHeap
};

fn main() {
    let mut heap = BinaryHeap::max_heap();
    let mut rng = MT19937_32::new();
    let mut values_generated = 0;

    for _ in 0..15 {
        heap.push(rng.generate());
        values_generated += 1;
    }

    while *heap.peek().unwrap() >= 1000000000 {
        let value = rng.generate();
        values_generated += 1;
        if value < *heap.peek().unwrap() {
            heap.pop();
            heap.push(value);
        }
    }

    while let Some(value) = heap.pop() {
        println!("{}", value);
    }

    println!("{} values generated", values_generated);
}