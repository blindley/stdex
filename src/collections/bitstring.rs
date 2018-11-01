pub trait BitString {
    const MAX_LEN: usize;
    type BitsType;
    fn new(len: usize, bits: Self::BitsType) -> Self;
    fn len(&self) -> usize;
    fn bits(&self) -> Self::BitsType;
    fn push_bit_back(&mut self, bit: u8);
    fn push_bit_front(&mut self, bit: u8);
    fn pop_bit_back(&mut self) -> u8;
    fn pop_bit_front(&mut self) -> u8;
    fn append(&mut self, other: Self);
    fn prepend(&mut self, other: Self);
    fn clear(&mut self);
}

/// A BitString that can hold at least 8 bits
pub trait BitString8: BitString {
    fn push_u8_back(&mut self, byte: u8);
    fn push_u8_front(&mut self, byte: u8);
    fn pop_u8_back(&mut self) -> u8;
    fn pop_u8_front(&mut self) -> u8;
}

macro_rules! impl_compact_bitstring {
    ($name:ident, $base:ty, $maxlen:expr) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name($base);

        impl BitString for $name {
            const MAX_LEN: usize = $maxlen;
            type BitsType = $base;
            fn new(len: usize, bits: Self::BitsType) -> Self {
                $name(((len as Self::BitsType) << Self::MAX_LEN) | (bits & ((1 << len) - 1)))
            }
            fn len(&self) -> usize { (self.0 >> Self::MAX_LEN) as usize }
            fn bits(&self) -> Self::BitsType { self.0 & ((1 << Self::MAX_LEN) - 1) }

            fn push_bit_back(&mut self, bit: u8) {
                let len = self.len();
                let new_bits = (self.bits() << 1) | (bit as Self::BitsType);
                *self = Self::new(len + 1, new_bits);
            }

            fn push_bit_front(&mut self, bit: u8) {
                let len = self.len();
                let new_bits = ((bit as Self::BitsType) << len) | self.bits();
                *self = Self::new(len + 1, new_bits);
            }

            fn pop_bit_back(&mut self) -> u8 {
                let result = (self.0 as u8) & 1;
                let len = self.len();
                let new_bits = self.bits() >> 1;
                *self = Self::new(len - 1, new_bits);
                result
            }

            fn pop_bit_front(&mut self) -> u8 {
                let new_len = self.len() - 1;
                let result = ((self.0 >> new_len) as u8) & 1;
                let new_bits = self.bits() & ((1 << new_len) - 1);
                *self = Self::new(new_len, new_bits);
                result
            }

            fn append(&mut self, other: Self) {
                let new_len = self.len() + other.len();
                let new_bits = (self.bits() << other.len()) | other.bits();
                *self = Self::new(new_len, new_bits);
            }

            fn prepend(&mut self, other: Self) {
                let new_len = self.len() + other.len();
                let new_bits = (other.bits() << self.len()) | self.bits();
                *self = Self::new(new_len, new_bits);
            }

            fn clear(&mut self) {
                self.0 = 0;
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let mut buffer: [u8;Self::MAX_LEN] = unsafe { std::mem::uninitialized() };
                let len = self.len();
                let mut bits = self.bits();
                for i in 0..len {
                    buffer[len - i - 1] = (bits as u8 & 1) + b'0';
                    bits >>= 1;
                }

                let s = unsafe { std::str::from_utf8_unchecked(&buffer[..len]) };
                f.write_str(s)
            }
        }
    };
}

impl_compact_bitstring!(CompactBitString5, u8, 5);
impl_compact_bitstring!(CompactBitString12, u16, 12);
impl_compact_bitstring!(CompactBitString27, u32, 27);
impl_compact_bitstring!(CompactBitString58, u64, 58);
impl_compact_bitstring!(CompactBitString121, u128, 121);

macro_rules! impl_compact_bitstring8 {
    ($name:ident) => {
        impl BitString8 for $name {
            fn push_u8_back(&mut self, byte: u8) {
                let len = self.len();
                let new_bits = (self.bits() << 8) | (byte as Self::BitsType);
                *self = Self::new(len + 8, new_bits);
            }

            fn push_u8_front(&mut self, byte: u8) {
                let len = self.len();
                let new_bits = ((byte as Self::BitsType) << len) | self.bits();
                *self = Self::new(len + 8, new_bits);
            }

            fn pop_u8_back(&mut self) -> u8 {
                let result = self.0 as u8;
                let len = self.len();
                let new_bits = self.bits() >> 8;
                *self = Self::new(len - 8, new_bits);
                result
            }

            fn pop_u8_front(&mut self) -> u8 {
                let new_len = self.len() - 8;
                let result = (self.0 >> new_len) as u8;
                let new_bits = self.bits() & ((1 << new_len) - 1);
                *self = Self::new(new_len, new_bits);
                result
            }
        }
    };
}

impl_compact_bitstring8!(CompactBitString12);
impl_compact_bitstring8!(CompactBitString27);
impl_compact_bitstring8!(CompactBitString58);
impl_compact_bitstring8!(CompactBitString121);

mod tests {
    #![allow(unused_imports, overflowing_literals, non_snake_case)]
    use super::*;

    #[test]
    fn test_CompactBitString5() {
        let mut bs = CompactBitString5::new(0, 0);
        assert_eq!(bs.len(), 0);
        assert_eq!(bs.bits(), 0);

        bs.push_bit_back(1);
        assert_eq!(bs.len(), 1);
        assert_eq!(bs.bits(), 0b1);

        bs.push_bit_front(0);
        assert_eq!(bs.len(), 2);
        assert_eq!(bs.bits(), 0b01);

        bs.push_bit_front(1);
        assert_eq!(bs.len(), 3);
        assert_eq!(bs.bits(), 0b101);

        bs.push_bit_back(1);
        assert_eq!(bs.len(), 4);
        assert_eq!(bs.bits(), 0b1011);

        bs.push_bit_front(0);
        assert_eq!(bs.len(), 5);
        assert_eq!(bs.bits(), 0b01011);

        assert_eq!(bs.pop_bit_back(), 1);
        assert_eq!(bs.len(), 4);
        assert_eq!(bs.bits(), 0b0101);

        assert_eq!(bs.pop_bit_front(), 0);
        assert_eq!(bs.len(), 3);
        assert_eq!(bs.bits(), 0b101);

        assert_eq!(bs.pop_bit_front(), 1);
        assert_eq!(bs.len(), 2);
        assert_eq!(bs.bits(), 0b01);
        
        assert_eq!(bs.pop_bit_back(), 1);
        assert_eq!(bs.len(), 1);
        assert_eq!(bs.bits(), 0b0);

        assert_eq!(bs.pop_bit_back(), 0);
        assert_eq!(bs.len(), 0);
        assert_eq!(bs.bits(), 0);

        bs.push_bit_back(1);
        bs.push_bit_back(0);
        assert_eq!(bs.len(), 2);
        assert_eq!(bs.bits(), 0b10);

        let x = bs;
        bs.append(x);
        assert_eq!(bs.len(), 4);
        assert_eq!(bs.bits(), 0b1010);

        bs.clear();
        assert_eq!(bs.len(), 0);
        assert_eq!(bs.bits(), 0);

        bs.push_bit_back(1);
        bs.prepend(x);
        assert_eq!(bs.len(), 3);
        assert_eq!(bs.bits(), 0b101);

        bs.prepend(x);
        assert_eq!(bs.len(), 5);
        assert_eq!(bs.bits(), 0b10101);
    }
}