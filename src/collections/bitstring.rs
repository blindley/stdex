
macro_rules! impl_compact_bitstring {
    ($name:ident, $type:ty, $len:expr) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name(pub $type);

        impl $name {
            const MAX_LEN: usize = $len;
            const BITS_MASK: $type = (1 << $len) - 1;
            const LEN_MASK: $type = !$name::BITS_MASK;
            const LEN_ONE: $type = 1 << $len;

            pub fn new() -> $name { $name(0) }
            pub fn from_len_and_bits(len: usize, bits: $type) -> $name {
                $name(((len as $type) << $len) | bits)
            }
            pub fn len(&self) -> usize { (self.0 >> $len) as usize }
            pub fn bits(&self) -> $type { self.0 & $name::BITS_MASK }
            pub fn push_bit(&mut self, bit: u8) {
                self.0 =
                    ((self.0 & $name::LEN_MASK) + $name::LEN_ONE) |
                    ((self.0 << 1) & $name::BITS_MASK | (bit as $type));
            }

            pub fn push_bit_front(&mut self, bit: u8) {
                self.0 =
                    ((self.0 & $name::LEN_MASK) + $name::LEN_ONE) |
                    (self.bits() | ((bit as $type) << self.len()))
            }

            pub fn append(&mut self, other: $name) {
                self.0 =
                    ((self.0 & $name::LEN_MASK) + (other.0 & $name::LEN_MASK)) |
                    ((self.0 << other.len()) & $name::BITS_MASK | other.bits())
            }

            pub fn prepend(&mut self, other: $name) {
                self.0 =
                    ((self.0 & $name::LEN_MASK) + (other.0 & $name::LEN_MASK)) |
                    (self.bits() | (other.bits() << self.len()))
            }

            pub fn pop_bit(&mut self) -> u8 {
                let result = (self.0 & 1) as u8;
                self.0 =
                    ((self.0 & $name::LEN_MASK) - $name::LEN_ONE) |
                    (self.bits() >> 1);
                result
            }

            pub fn pop_bit_front(&mut self) -> u8 {
                let new_len = self.len() - 1;
                let result = ((self.0 >> new_len) & 1) as u8;
                self.0 =
                    ((self.0 & $name::LEN_MASK) - $name::LEN_ONE) |
                    (self.bits() & ((1 << new_len) - 1));
                result
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let mut buffer: [u8;$len] = unsafe { std::mem::uninitialized() };
                let len = self.len();
                assert!(len <= $len);
                let mut bits = self.bits();
                for i in 0..len {
                    buffer[len - i - 1] = b'0' + (bits as u8 & 1);
                    bits >>= 1;
                }

                let s = unsafe { std::str::from_utf8_unchecked(&buffer[..len]) };

                f.write_str(s)
            }
        }

        impl std::fmt::Binary for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:b}", self.bits())
            }
        }

        impl std::fmt::LowerHex for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:x}", self.bits())
            }
        }

        impl std::fmt::UpperHex for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:X}", self.bits())
            }
        }

        impl std::fmt::Octal for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:o}", self.bits())
            }
        }
    };
}

impl_compact_bitstring!(CompactBitString5, u8, 5);
impl_compact_bitstring!(CompactBitString12, u16, 12);
impl_compact_bitstring!(CompactBitString27, u32, 27);
impl_compact_bitstring!(CompactBitString58, u64, 58);
impl_compact_bitstring!(CompactBitString121, u128, 121);

macro_rules! impl_from {
    ($from:ty, $into:ident, $into_t:ty) => {
        impl From<$from> for $into {
            fn from(item: $from) -> $into {
                const SHIFT: usize = <$into>::MAX_LEN - <$from>::MAX_LEN;
                $into(
                    (((item.0 as $into_t) << SHIFT) & Self::LEN_MASK) |
                    ((item.0 & <$from>::BITS_MASK) as $into_t)
                )
            }
        }
    };
}

impl_from!(CompactBitString5, CompactBitString12, u16);
impl_from!(CompactBitString5, CompactBitString27, u32);
impl_from!(CompactBitString5, CompactBitString58, u64);
impl_from!(CompactBitString5, CompactBitString121, u128);
impl_from!(CompactBitString12, CompactBitString27, u32);
impl_from!(CompactBitString12, CompactBitString58, u64);
impl_from!(CompactBitString12, CompactBitString121, u128);
impl_from!(CompactBitString27, CompactBitString58, u64);
impl_from!(CompactBitString27, CompactBitString121, u128);
impl_from!(CompactBitString58, CompactBitString121, u128);

mod tests {
    #![allow(unused_imports, overflowing_literals)]
    use super::*;

    macro_rules! impl_tests {
        ($tname:ident, $name:ident, $type:ty, $len:expr) => {
            #[test]
            fn $tname() {
                let mut b = $name::new();
                assert_eq!(b.len(), 0);
                assert_eq!(b.bits(), 0);
                b.push_bit(1);
                assert_eq!(b.len(), 1);
                assert_eq!(b.bits(), 1);
                b.push_bit_front(0);
                assert_eq!(b.len(), 2);
                assert_eq!(b.bits(), 1);
                b.push_bit(1);
                assert_eq!(b.len(), 3);
                assert_eq!(b.bits(), 3);
                b.push_bit(0);
                assert_eq!(b.len(), 4);
                assert_eq!(b.bits(), 6);

                let mut b2 = $name::new();
                b2.append(b);
                assert_eq!(b2.len(), 4);
                assert_eq!(b2.bits(), 6);
                assert_eq!(b, b2);

                

                if $len >= 8 {
                    b2.prepend(b);
                    assert_eq!(b2.len(), 8);
                    assert_eq!(b2.bits(), 0b01100110);
                    b2.append(b);
                    assert_eq!(b2.len(), 12);
                    assert_eq!(b2.bits(), 0b011001100110);
                }

                let old_len = b2.len();
                assert_eq!(b2.pop_bit(), 0);
                assert_eq!(b2.len(), old_len - 1);
                assert_eq!(b2.pop_bit_front(), 0);
                assert_eq!(b2.len(), old_len - 2);
                assert_eq!(b2.pop_bit_front(), 1);
                assert_eq!(b2.len(), old_len - 3);
                assert_eq!(b2.pop_bit(), 1);
                assert_eq!(b2.len(), old_len - 4);
            }
        };
    }
    
    impl_tests!(test_bitstring5, CompactBitString5, u8, 5);
    impl_tests!(test_bitstring12, CompactBitString12, u16, 12);
    impl_tests!(test_bitstring27, CompactBitString27, u32, 27);
    impl_tests!(test_bitstring58, CompactBitString58, u64, 58);
    impl_tests!(test_bitstring121, CompactBitString121, u128, 121);

    #[test]
    fn a_few_more_tests() {
        let mut bits = 0x102030405060708090a0b0c0d0e0f0;
        let mut b = CompactBitString121::from_len_and_bits(120, bits);
        assert_eq!(b.len(), 120);
        assert_eq!(b.bits(), bits);

        for i in 0..120 {
            let bit = (bits & 1) as u8;
            assert_eq!(b.pop_bit(), bit);
            bits >>= 1;
            assert_eq!(b.len(), 119 - i);
            assert_eq!(b.bits(), bits);
        }
    }
}