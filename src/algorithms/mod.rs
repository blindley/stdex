use crate::random::{
    generate_uniform_u64,
    Rng64
};

mod heap;
pub use self::heap::*;

mod deflate;
pub use self::deflate::inflate;

pub fn shuffle<T, G: Rng64>(data: &mut [T], eng: &mut G) {
    let n = data.len();
    for i in 0..=(n - 2) {
        let j = generate_uniform_u64(eng, (n-i-1) as u64) as usize + i;
        data.swap(i, j);
    }
}