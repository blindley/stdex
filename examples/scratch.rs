#![allow(unused_imports, dead_code)]
extern crate stdex;

fn main() {
    use stdex::random::{MT19937_32, MT19937_64};

    let mut rng: MT19937_64 = unsafe { std::mem::zeroed() };
    unsafe {
        let ptr = &mut rng as *mut _ as *mut u8;
        let ptr = ptr.offset(100);
        *ptr = 1;
    }
    let mut non_zero_count = 0;
    for i in 0..6000 {
        if non_zero_count >= 1000 { break; }

        let value = rng.generate();
        if value != 0 {
            println!("{}: {}", i, value);
            non_zero_count += 1;
        }
    }
}