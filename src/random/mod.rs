mod mt19937;

pub use self::mt19937::{MT19937_32, MT19937_64};

/// convenience function to provide a simple seed from the system time
pub fn time_seed_seconds_32() -> u32 {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as u32,
        Err(_) => 0,
    }
}

/// convenience function to provide a simple seed from the system time
pub fn time_seed_seconds_64() -> u64 {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() as u64,
        Err(_) => 0,
    }
}

pub trait Rng32 {
    fn generate_u32(&mut self) -> u32;
}

pub trait Rng64 {
    fn generate_u64(&mut self) -> u64;
}

/// Generate a `u32` uniformly distributed across the range `[0, max_value]`
/// 
/// Uniformity is contingent upon the uniformity of `eng` across
/// `[0, std::u32::MAX]`
pub fn generate_uniform_u32(eng: &mut impl Rng32, max_value: u32) -> u32 {
    const ENGINE_MAX: u32 = std::u32::MAX;
    if max_value == 0 {
        max_value
    } else if ENGINE_MAX == max_value {
        eng.generate_u32()
    } else {
        let bucket_size = ((ENGINE_MAX as u64 + 1) / (max_value as u64 + 1)) as u32;
        loop {
            let result = eng.generate_u32() / bucket_size;
            if result <= max_value {
                break result;
            }
        }
    }
}

/// Generate a `u64` uniformly distributed across the range `[0, max_value]`
/// 
/// Uniformity is contingent upon the uniformity of `eng` across
/// `[0, std::u64::MAX]`
pub fn generate_uniform_u64(eng: &mut impl Rng64, max_value: u64) -> u64 {
    const ENGINE_MAX: u64 = std::u64::MAX;
    if max_value == 0 {
        max_value
    } else if ENGINE_MAX == max_value {
        eng.generate_u64()
    } else {
        let bucket_size = ((ENGINE_MAX as u128 + 1) / (max_value as u128 + 1)) as u64;
        loop {
            let result = eng.generate_u64() / bucket_size;
            if result <= max_value {
                break result;
            }
        }
    }
}

/// Generate an `f32` in the range `[0,1)`
pub fn generate_canonical_f32(eng: &mut impl Rng32) -> f32 {
    let result = (eng.generate_u32() as f32) / (std::u32::MAX as f32);
    if result == 1.0 {
        result - std::f32::EPSILON / 2.0
    } else {
        result
    }
}

/// Generate an `f64` in the range `[0,1)`
pub fn generate_canonical_f64(eng: &mut impl Rng64) -> f64 {
    let result = (eng.generate_u64() as f64) / (std::u64::MAX as f64);
    if result == 1.0 {
        result - std::f64::EPSILON / 2.0
    } else {
        result
    }
}