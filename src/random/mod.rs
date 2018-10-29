mod mt19937;

pub use self::mt19937::{MT19937_32, MT19937_64};

pub trait Rng32 {
    fn generate_u32(&mut self) -> u32;
}

pub trait Rng64 {
    fn generate_u64(&mut self) -> u64;
}
