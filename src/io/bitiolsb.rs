use std::io::{self, Read, Write};
use crate::io::{read_u8, write_u8};

type Bit = u8;

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

    /// Reads a single bit from the stream
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::BitReaderLSB;
    /// let buffer = std::io::Cursor::new([0b10101010]);
    /// let mut bitreader = BitReaderLSB::new(buffer);
    /// for i in 0..4 {
    ///     assert_eq!(bitreader.read_bit().ok(), Some(0));
    ///     assert_eq!(bitreader.read_bit().ok(), Some(1));
    /// }
    /// ```
    pub fn read_bit(&mut self) -> io::Result<Bit> {
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
    /// # use stdex::io::BitReaderLSB;
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
    pub fn read_bits_32(&mut self, mut count: usize) -> io::Result<u32> {
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
    /// # use stdex::io::BitReaderLSB;
    /// let cursor = std::io::Cursor::new([0xab, 0xcd]);
    /// let mut bitreader = BitReaderLSB::new(cursor);
    /// 
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xb));
    /// bitreader.flush_byte();
    /// assert_eq!(bitreader.read_bits_32(4).ok(), Some(0xd));
    /// ```
    pub fn flush_byte(&mut self) {
        self.buffer = 0;
        self.mask = 0x1;
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
    /// # use stdex::io::BitReaderLSB;
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

mod bitreaderlsb_tests {
    #[test]
    fn test_read_bit() {
        let data = [0b10101010, 0b10101010];
        let reader = std::io::Cursor::new(data);
        let mut reader = super::BitReaderLSB::new(reader);
        let mut index = 0;
        while let Ok(bit) = reader.read_bit() {
            assert_eq!(bit, index % 2);
            index += 1;
        }
    }

    #[test]
    fn read_bits_32() {
        let data = [0xab, 0xcd, 0xef];
        let reader = std::io::Cursor::new(data);
        let mut reader = super::BitReaderLSB::new(reader);
        assert_eq!(reader.read_bits_32(4).ok(), Some(0xb));
        assert_eq!(reader.read_bits_32(8).ok(), Some(0xda));
        assert_eq!(reader.read_bits_32(12).ok(), Some(0xefc));
    }
}

/// Adapts an output stream to write one or more bits at a time
pub struct BitWriterLSB<W: Write> {
    writer: W,
    buffer: u32,
    mask: u32,
}

impl<W: Write> BitWriterLSB<W> {
    pub fn new(writer: W) -> BitWriterLSB<W> {
        BitWriterLSB {
            writer,
            buffer: 0,
            mask: 0x1,
        }
    }

    /// Writes a single bit to the stream.
    /// 
    /// If `bit == 0`, writes a 0, otherwise writes a 1.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use stdex::io::BitWriterLSB;
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterLSB::new(output.by_ref());
    ///     for i in 0..24 {
    ///         bitwriter.write_bit(i % 3).unwrap();
    ///     }
    /// }
    /// assert_eq!(output, vec![0b10110110, 0b01101101, 0b11011011]);
    /// ```
    pub fn write_bit(&mut self, bit: Bit) -> io::Result<()> {
        if bit != 0 {
            self.buffer |= self.mask;
        }

        self.mask <<= 1;
        if self.mask == 0x100 {
            write_u8(&mut self.writer, self.buffer as u8)?;
            self.buffer = 0;
            self.mask = 0x1;
        }

        Ok(())
    }

    /// Writes up to 32 bits to the stream.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use stdex::io::BitWriterLSB;
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterLSB::new(output.by_ref());
    ///     bitwriter.write_bits_32(0xabc, 12).unwrap();
    ///     bitwriter.write_bits_32(0xd, 4).unwrap();
    /// }
    /// assert_eq!(output, vec![0xbc, 0xda]);
    /// ```
    /// 
    /// # Panics
    /// 
    /// Panics if `count > 32`.
    pub fn write_bits_32(&mut self, value: u32, mut count: usize)
    -> io::Result<()> {
        assert!(count <= 32);
        if count == 0 {
            return Ok(());
        }

        let mut mask_shift = 0;
        while count > 0 && self.mask != 0x1 {
            if value & (1 << mask_shift) != 0 {
                self.buffer |= self.mask;
            }
            self.mask <<= 1;
            if self.mask == 0x100 {
                write_u8(&mut self.writer, self.buffer as u8)?;
                self.buffer = 0;
                self.mask = 0x1;
            }
            mask_shift += 1;
            count -= 1;
        }

        while count >= 8 {
            let buffer = value >> mask_shift;
            write_u8(&mut self.writer, buffer as u8)?;
            mask_shift += 8;
            count -= 8;
        }

        while count > 0 {
            if value & (1 << mask_shift) != 0 {
                self.buffer |= self.mask;
            }

            self.mask <<= 1;
            if self.mask == 0x100 {
                write_u8(&mut self.writer, self.buffer as u8)?;
                self.buffer = 0;
                self.mask = 0x1;
            }
            mask_shift += 1;
            count -= 1;
        }

        Ok(())
    }

    /// Finishes writing any partially written byte.
    /// 
    /// Fills in remaining bits with `fill_bit`. If there are no partially
    /// written bytes, does nothing.
    pub fn finish_byte(&mut self, fill_bit: Bit) -> io::Result<()> {
        while self.mask != 0x1 {
            self.write_bit(fill_bit)?;
        }
        Ok(())
    }

    /// Returns a reference to the underlying `Write` object.
    /// 
    /// Partially written bytes will not be output to the stream and remain
    /// in the buffer.
    pub fn as_write(&self) -> &W {
        &self.writer
    }

    /// Returns a reference to the underlying `Write` object.
    /// 
    /// Partially written bytes will not be output to the stream and remain
    /// in the buffer.
    /// 
    /// # Example
    /// ```
    /// # use stdex::io::BitWriterLSB;
    /// # use stdex::io::write_u8;
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterLSB::new(output.by_ref());
    ///     bitwriter.write_bits_32(0xa, 4).unwrap();
    ///     {
    ///         let writer = bitwriter.as_write_mut();
    ///         write_u8(writer, 0xbc).unwrap();
    ///     }
    ///     bitwriter.write_bits_32(0xd, 4).unwrap();
    /// }
    /// assert_eq!(output, vec![0xbc, 0xda]);
    /// ```
    pub fn as_write_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Drops self and returns the underlying `Write` object
    /// 
    /// Partially written bytes will not be output to the stream and will
    /// be lost.
    pub fn into_write(self) -> W {
        self.writer
    }
}

mod bitwriterlsb_tests {
    #[test]
    fn test_write_bit() {
        use std::io::Write;
        let mut output = Vec::new();
        {
            let mut writer = super::BitWriterLSB::new(output.by_ref());
            let bits = [
                1, 0, 0, 0, 0, 0, 0, 0,
                1, 1, 0, 0, 0, 1, 0, 0,
                1, 0, 1, 0, 0, 0, 1, 0,
                1, 1, 1, 0, 0, 1, 1, 0,
                1, 0, 0, 1, 0, 0, 0, 1,
                1, 1, 0, 1, 0, 1, 0, 1,
                1, 0, 1, 1, 0, 0, 1, 1,
                1, 1, 1, 1, 0, 1, 1, 1,
            ];

            for bit in bits.iter() {
                writer.write_bit(*bit).unwrap();
            }
        }
        assert_eq!(output, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
    }

    #[test]
    fn test_write_bits_32() {
        
        use std::io::Write;
        let mut output = Vec::new();
        {
            let mut writer = super::BitWriterLSB::new(output.by_ref());
            writer.write_bits_32(0xa, 4).unwrap();
            writer.write_bits_32(0xbc, 8).unwrap();
            writer.write_bits_32(0xdef, 12).unwrap();
        }

        assert_eq!(output, vec![0xca, 0xfb, 0xde]);
    }
}