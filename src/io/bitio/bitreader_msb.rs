use std::io::Read;
use crate::io::read_u8;

use super::Bit;

/// Adapts an input stream to read one or more bits at a time
/// 
/// Bits are read starting from the most significant bit of each successive
/// byte.
pub struct BitReaderMSB<R: Read> {
    reader: R,
    buffer: u32,
    mask: u32,
}

impl<R: Read> BitReaderMSB<R> {
    pub fn new(reader: R) -> BitReaderMSB<R> {
        BitReaderMSB {
            reader,
            buffer: 0,
            mask: 0x80,
        }
    }

    /// Returns a reference to the underlying `Read` object.
    /// 
    /// Any partially read bytes will not be accessible through the reference.
    pub fn as_read(&self) -> &R {
        &self.reader
    }

    /// Returns a mutable reference to the underlying `Read` object.
    /// 
    /// Any partially read bytes will not be accessible through the reference.
    /// If you partially read a byte, and then use the `Read` interface to
    /// read one or more bytes, when you go back to the BitReader, you will
    /// first read the unfinished byte, and then skip to after the last byte
    /// read from the `Read` object.
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderMSB};
    /// # use stdex::io::read_u8;
    /// let cursor = std::io::Cursor::new([0xab, 0xcd, 0xef]);
    /// let mut bitreader = BitReaderMSB::new(cursor);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xa));
    /// {
    ///     let reader = bitreader.as_read_mut();
    ///     assert_eq!(read_u8(reader).ok(), Some(0xcd));
    /// }
    /// assert_eq!(bitreader.read_bits_32(12).ok(), Some(0xbef));
    /// ```
    pub fn as_read_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Drops self and returns the underlying `Read` object.
    /// 
    /// Any remaining bits of partially read bytes will be lost.
    pub fn into_read(self) -> R {
        self.reader
    }
}

impl<R: Read> crate::io::BitRead for BitReaderMSB<R> {
    /// Reads a single bit from the stream
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderMSB};
    /// let buffer = std::io::Cursor::new([0b10101010]);
    /// let mut bitreader = BitReaderMSB::new(buffer);
    /// for i in 0..4 {
    ///     assert_eq!(bitreader.read_bit().ok(), Some(1));
    ///     assert_eq!(bitreader.read_bit().ok(), Some(0));
    /// }
    /// ```
    fn read_bit(&mut self) -> std::io::Result<Bit> {
        if self.mask == 0x80 {
            self.buffer = read_u8(&mut self.reader)? as u32;
        }

        let result = match self.mask & self.buffer {
            0 => 0,
            _ => 1,
        };

        self.mask >>= 1;
        if self.mask == 0 {
            self.mask = 0x80;
        }

        Ok(result)
    }

    /// Reads up to 32 bits from the stream
    /// 
    /// Bits are placed in the lower portion of the resulting u32, with
    /// the first bits being placed in the most significant position.
    /// So if you read 9 bits you will get a value less than 512.
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderMSB};
    /// let buffer = std::io::Cursor::new([0xab, 0xcd, 0xef]);
    /// let mut bitreader = BitReaderMSB::new(buffer);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xa));
    /// assert_eq!(bitreader.read_bits_32(8).ok(), Some(0xbc));
    /// assert_eq!(bitreader.read_bits_32(12).ok(), Some(0xdef))
    /// ```
    /// 
    /// # Panic
    /// Panics if `count > 32`.
    fn read_bits_32(&mut self, mut count: usize) -> std::io::Result<u32> {
        assert!(count <= 32);
        let mut result = 0;
        while count > 0 && self.mask != 0x80 {
            result <<= 1;
            if self.mask & self.buffer != 0 {
                result |= 1;
            }
            self.mask >>= 1;
            if self.mask == 0 {
                self.mask = 0x80;
            }
            count -= 1;
        }

        while count >= 8 {
            let buffer = read_u8(&mut self.reader)? as u32;
            result = (result << 8) | buffer;
            count -= 8;
        }

        while count > 0 {
            if self.mask == 0x80 {
                self.buffer = read_u8(&mut self.reader)? as u32;
            }

            result <<= 1;
            if self.mask & self.buffer != 0 {
                result |= 1;
            }

            self.mask >>= 1;
            if self.mask == 0 {
                self.mask = 0x80;
            }
            count -= 1;
        }

        Ok(result)
    }

    /// Discards any remaining bits of a partially read byte
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderMSB};
    /// let cursor = std::io::Cursor::new([0xab, 0xcd]);
    /// let mut bitreader = BitReaderMSB::new(cursor);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xa));
    /// bitreader.flush_byte();
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xc));
    /// ```
    fn flush_byte(&mut self) {
        self.buffer = 0;
        self.mask = 0x80;
    }
}

mod bitreader_tests {
    #[test]
    fn test_read_bit() {
        use crate::io::{BitRead, BitReaderMSB};

        let vec: Vec<u8> = (0..=255).collect();
        let reader = std::io::Cursor::new(vec.clone());
        let mut reader = BitReaderMSB::new(reader);
        let mut bit_count = 0;
        let mut index = 0;
        let mut byte = 0;
        while let Ok(bit) = reader.read_bit() {
            bit_count += 1;
            byte = (byte << 1) | bit;
            if bit_count == 8 {
                assert_eq!(byte, vec[index]);
                index += 1;
                bit_count = 0;
            }
        }
        assert_eq!(index, 256); // make sure we've consumed the whole vector
    }

    #[test]
    fn test_read_bits_32() {
        use crate::io::{BitRead, BitReaderMSB};

        let vec: Vec<u8> = (0..=255).collect();
        let reader = std::io::Cursor::new(vec.clone());
        let mut reader = BitReaderMSB::new(reader);

        let mut bits_remaining = 256 * 8;

        // read 0x0 - 0x11, by 32, 28, 24 ... bits at a time
        assert_eq!(reader.read_bits_32(32).ok(), Some(0x00010203));
        assert_eq!(reader.read_bits_32(28).ok(), Some(0x0405060));
        assert_eq!(reader.read_bits_32(24).ok(), Some(0x708090));
        assert_eq!(reader.read_bits_32(20).ok(), Some(0xa0b0c));
        assert_eq!(reader.read_bits_32(16).ok(), Some(0x0d0e));
        assert_eq!(reader.read_bits_32(12).ok(), Some(0x0f1));
        assert_eq!(reader.read_bits_32(8) .ok(), Some(0x01));
        assert_eq!(reader.read_bits_32(4) .ok(), Some(0x1));
        assert_eq!(reader.read_bits_32(0) .ok(), Some(0x0));

        bits_remaining -= 144;

        // 0x121314 = 000 100 100 001 001 100 010 100
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b000));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b100));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b100));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b001));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b001));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b100));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b010));
        assert_eq!(reader.read_bits_32(3).ok(), Some(0b100));

        bits_remaining -= 24;

        // 0x15161718 191a1b1c 1d1e1f20 =
        // 00010101 000101100 0010111000 11000
        // 000110 010001101000 0110110001110 0
        // 0001110100011 110000111110010 0000
        assert_eq!(reader.read_bits_32(8).ok(), Some(0b00010101));
        assert_eq!(reader.read_bits_32(9).ok(), Some(0b000101100));
        assert_eq!(reader.read_bits_32(10).ok(), Some(0b0010111000));
        assert_eq!(reader.read_bits_32(11).ok(), Some(0b11000000110));
        assert_eq!(reader.read_bits_32(12).ok(), Some(0b010001101000));
        assert_eq!(reader.read_bits_32(13).ok(), Some(0b0110110001110));
        assert_eq!(reader.read_bits_32(14).ok(), Some(0b00001110100011));
        assert_eq!(reader.read_bits_32(15).ok(), Some(0b110000111110010));
        assert_eq!(reader.read_bits_32(4).ok(), Some(0b0000));

        bits_remaining -= 96;

        while bits_remaining >= 32 {
            match reader.read_bits_32(32) {
                Err(_) => panic!("not enough bits"),
                _ => (),
            }
            bits_remaining -= 32;
        }

        match reader.read_bits_32(bits_remaining) {
            Err(_) => panic!("not enough bits"),
            _ => (),
        }

        match reader.read_bits_32(1) {
            Ok(_) => panic!("too many bits"),
            _ => (),
        }
    }
}
