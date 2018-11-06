use std::io::Read;
use crate::io::read_u8;

use super::Bit;

/// Adapts an input stream to read one or more bits at a time
/// 
/// Bits are read starting from the least significant bit of each successive
/// byte.
pub struct BitReaderLSB<R: Read> {
    reader: R,
    buffer: u32,
    mask: u32,
}

impl<R: Read> BitReaderLSB<R> {
    pub fn new(reader: R) -> BitReaderLSB<R> {
        BitReaderLSB {
            reader,
            buffer: 0,
            mask: 0x1,
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
    /// # use stdex::io::{BitRead, BitReaderLSB};
    /// # use stdex::io::read_u8;
    /// let cursor = std::io::Cursor::new([0xab, 0xcd, 0xef]);
    /// let mut bitreader = BitReaderLSB::new(cursor);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xb));
    /// {
    ///     let reader = bitreader.as_read_mut();
    ///     assert_eq!(read_u8(reader).ok(), Some(0xcd));
    /// }
    /// assert_eq!(bitreader.read_bits_32(12).ok(), Some(0xefa));
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

impl<R: Read> crate::io::BitRead for BitReaderLSB<R> {
    /// Reads a single bit from the stream
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderLSB};
    /// let buffer = std::io::Cursor::new([0b10101010]);
    /// let mut bitreader = BitReaderLSB::new(buffer);
    /// for i in 0..4 {
    ///     assert_eq!(bitreader.read_bit().ok(), Some(0));
    ///     assert_eq!(bitreader.read_bit().ok(), Some(1));
    /// }
    /// ```
    fn read_bit(&mut self) -> std::io::Result<Bit> {
        if self.mask == 0x1 {
            self.buffer = read_u8(&mut self.reader)? as u32;
        }

        let result = match self.mask & self.buffer {
            0 => 0,
            _ => 1,
        };

        self.mask <<= 1;
        if self.mask == 0x100 {
            self.mask = 0x1;
        }

        Ok(result)
    }

    /// Reads up to 32 bits from the stream
    /// 
    /// Bits are placed in the lower portion of the resulting u32, with
    /// the first bits being placed in the least significant position.
    /// So if you read 9 bits you will get a value less than 512.
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderLSB};
    /// let buffer = std::io::Cursor::new([0xab, 0xcd, 0xef]);
    /// let mut bitreader = BitReaderLSB::new(buffer);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xb));
    /// assert_eq!(bitreader.read_bits_32(8).ok(), Some(0xda));
    /// assert_eq!(bitreader.read_bits_32(12).ok(), Some(0xefc))
    /// ```
    /// 
    /// # Panic
    /// Panics if `count > 32`.
    fn read_bits_32(&mut self, mut count: usize) -> std::io::Result<u32> {
        assert!(count <= 32);
        let mut result = 0;
        let mut mask_shift = 0;
        while count > 0 && self.mask != 0x1 {
            if self.mask & self.buffer != 0 {
                result |= 1 << mask_shift;
            }
            
            self.mask <<= 1;
            if self.mask == 0x100 {
                self.mask = 0x1;
            }
            count -= 1;
            mask_shift += 1;
        }

        while count >= 8 {
            let buffer = read_u8(&mut self.reader)? as u32;
            result = result | (buffer << mask_shift);
            count -= 8;
            mask_shift += 8;
        }

        while count > 0 {
            if self.mask == 0x1 {
                self.buffer = read_u8(&mut self.reader)? as u32;
            }

            if self.mask & self.buffer != 0 {
                result |= 1 << mask_shift;
            }

            self.mask <<= 1;
            if self.mask == 0x100 {
                self.mask = 0x1;
            }
            count -= 1;
            mask_shift += 1;
        }

        Ok(result)
    }

    /// Discards any remaining bits of a partially read byte
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::{BitRead, BitReaderLSB};
    /// let cursor = std::io::Cursor::new([0xab, 0xcd]);
    /// let mut bitreader = BitReaderLSB::new(cursor);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xb));
    /// bitreader.flush_byte();
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xd));
    /// ```
    fn flush_byte(&mut self) {
        self.buffer = 0;
        self.mask = 0x1;
    }
}

mod bitreaderlsb_tests {
    #[test]
    fn test_read_bit() {
        use crate::io::{BitRead, BitReaderLSB};
        let data = [0b10101010, 0b10101010];
        let reader = std::io::Cursor::new(data);
        let mut reader = BitReaderLSB::new(reader);
        let mut index = 0;
        while let Ok(bit) = reader.read_bit() {
            assert_eq!(bit, index % 2);
            index += 1;
        }
    }

    #[test]
    fn read_bits_32() {
        use crate::io::{BitRead, BitReaderLSB};
        let data = [0xab, 0xcd, 0xef];
        let reader = std::io::Cursor::new(data);
        let mut reader = BitReaderLSB::new(reader);
        assert_eq!(reader.read_bits_32(4).ok(), Some(0xb));
        assert_eq!(reader.read_bits_32(8).ok(), Some(0xda));
        assert_eq!(reader.read_bits_32(12).ok(), Some(0xefc));
    }
}
