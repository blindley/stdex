
macro_rules! mersenne_twister_impl {
    ($name:ident, $out_type:ty,
    $w:expr, $n:expr, $m:expr, $r:expr,
    $a:expr, $u:expr, $d:expr, $s:expr, $b:expr, $t:expr,
    $c:expr, $l:expr, $f:expr) => {

        pub struct $name {
            state: [$out_type;$n],
            index: usize,
        }

        impl $name {
            /// Initialized the generator with the seed 5489
            pub fn new() -> $name {
                $name::from_seed(5489)
            }

            pub fn from_seed(seed: $out_type) -> $name {
                let mut rng: $name = unsafe { std::mem::uninitialized() };
                rng.reseed(seed);
                rng
            }

            pub fn reseed(&mut self, seed: $out_type) {
                self.state[0] = seed;
                for i in 1..$n {
                    let prev = self.state[i-1];
                    self.state[i] = ($f as $out_type)
                        .wrapping_mul(prev ^ (prev >> ($w - 2))) + i as $out_type;
                }

                self.twist();
                self.index = 0;
            }

            pub fn generate(&mut self) -> $out_type {
                if self.index >= $n {
                    self.twist();
                    self.index = 0;
                }

                let mut x = self.state[self.index];
                self.index += 1;

                x ^=  (x >> $u) & $d;
                x ^= (x <<  $s) & $b;
                x ^= (x << $t) & $c;
                x ^=  x >> $l;

                x
            }

            pub fn skip(&mut self, mut skip_count: usize) {
                while skip_count >= $n {
                    self.twist();
                    skip_count -= $n;
                }

                self.index += skip_count;

                if self.index >= $n {
                    self.twist();
                    self.index -= $n;
                }
            }

            fn twist(&mut self) {
                const LMASK: $out_type = (1 << $r) - 1;
                const UMASK: $out_type = !LMASK;
                const FIRST_HALF: usize = $n - $m;

                let mut i = 0;
                while i < FIRST_HALF {
                    let bits =
                        (self.state[i] & UMASK) | (self.state[i + 1] & LMASK);
                    self.state[i] =
                        self.state[i + $m] ^ (bits  >> 1) ^ ((bits & 1) * $a);
                    i += 1;
                }

                while i < $n - 1 {
                    let bits =
                        (self.state[i] & UMASK) | (self.state[i + 1] & LMASK);
                    self.state[i] =
                        self.state[i - FIRST_HALF] ^ (bits >> 1) ^ ((bits & 1) * $a);
                    i += 1;
                }

                let bits = (self.state[i] & UMASK) | (self.state[0] & LMASK);
                self.state[i] = self.state[$m - 1] ^ (bits >> 1) ^ ((bits & 1) * $a);
            }
        }
    };
}

mersenne_twister_impl!(
    MT19937_32, u32,
    32, 624, 397, 31, // w, n, m, r
    0x9908b0df, 11,   // a, u
    0xffffffff, 7,    // d, s
    0x9d2c5680, 15,   // b, t
    0xefc60000, 18,   // c, l
    1812433253        // f
);

mersenne_twister_impl!(
    MT19937_64, u64, 64, 312, 156, 31,
    0xb5026f5aa96619e9, 29,
    0x5555555555555555, 17,
    0x71d67fffeda60000, 37,
    0xfff7eee000000000, 43,
    6364136223846793005
);

impl super::Rng32 for MT19937_32 {
    fn generate_u32(&mut self) -> u32 {
        self.generate()
    }
}

impl super::Rng64 for MT19937_32 {
    /// Generates two 32 bit values and combines them with bitwise OR
    fn generate_u64(&mut self) -> u64 {
        ((self.generate() as u64) << 32) | (self.generate() as u64)
    }
}

impl super::Rng32 for MT19937_64 {
    /// Generates a 64 bit value and discards the 32 most significant bits
    fn generate_u32(&mut self) -> u32 {
        self.generate() as u32
    }
}

impl super::Rng64 for MT19937_64 {
    fn generate_u64(&mut self) -> u64 {
        self.generate()
    }
}

mod tests {
    #[test]
    fn test_standard_mt19937_values() {
        let seeds_and_values_32 = [
            (1, [ 1791095845, 4282876139, 3093770124, 4005303368, 491263 ]),
            (1112, [ 339905540, 1014708198, 934252836, 2454405075, 1539988327 ]),
            (2223, [ 2703965020, 1645400881, 2741065023, 589793706, 68050488 ]),
        ];

        for (seed, values) in seeds_and_values_32.iter() {
            let mut gen = super::MT19937_32::from_seed(*seed);
            for i in 0..5 {
                assert_eq!(gen.generate(), values[i]);
            }
        }

        let seeds_and_values_64 = [
            (1, [ 2469588189546311528, 2516265689700432462, 8323445853463659930, 387828560950575246, 6472927700900931384 ]),
            (1112, [ 2030023689557577270, 17891585337015847365, 10964359332531572785, 7461730276270468400, 9580021312583915945 ]),
            (2223, [ 14826023173755866876, 7155474859988725726, 9451480599440685931, 7860867063210863592, 1592382917952193713 ]),
        ];

        for (seed, values) in seeds_and_values_64.iter() {
            let mut gen = super::MT19937_64::from_seed(*seed);
            for i in 0..5 {
                assert_eq!(gen.generate(), values[i]);
            }
        }

        {
            let mut gen32 = super::MT19937_32::new();
            let mut gen64 = super::MT19937_64::new();
            gen32.skip(9999);
            gen64.skip(9999);
            assert_eq!(gen32.generate(), 4123659995);
            assert_eq!(gen64.generate(), 9981545732273789042);
        }
    }
}