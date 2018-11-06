use std::io::Write;
use crate::io::write_u8;

use super::Bit;

/// Adapts an output stream to write one or more bits at a time
pub struct BitWriterMSB<W: Write> {
    writer: W,
    buffer: u32,
    mask: u32,
}

impl<W: Write> BitWriterMSB<W> {
    pub fn new(writer: W) -> BitWriterMSB<W> {
        BitWriterMSB {
            writer,
            buffer: 0,
            mask: 0x80,
        }
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
    /// # use stdex::io::{BitWrite, BitWriterMSB};
    /// # use stdex::io::write_u8;
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterMSB::new(output.by_ref());
    ///     bitwriter.write_bits_32(0xa, 4).unwrap();
    ///     {
    ///         let writer = bitwriter.as_write_mut();
    ///         write_u8(writer, 0xbc).unwrap();
    ///     }
    ///     bitwriter.write_bits_32(0xd, 4).unwrap();
    /// }
    /// assert_eq!(output, vec![0xbc, 0xad]);
    /// ```
    pub fn as_write_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Drops self and returns the underlying `Write` object
    /// 
    /// # Panics
    /// Panics if any partially written bytes are left in the buffer. Call
    /// `remaining_bits()` to check if there are any.
    pub fn into_write(mut self) -> W {
        use crate::io::BitWrite;
        assert_eq!(self.remaining_bits(), 0, "bits remaining in BitWriter before dropping");
        unsafe {
            let writer = std::mem::replace(&mut self.writer, std::mem::uninitialized());
            std::mem::forget(self);
            writer
        }
    }
}

impl<W: Write> crate::io::BitWrite for BitWriterMSB<W> {

    /// Writes a single bit to the stream.
    /// 
    /// If `bit == 0`, writes a 0, otherwise writes a 1.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use stdex::io::{BitWrite, BitWriterMSB};
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterMSB::new(output.by_ref());
    ///     for i in 0..24 {
    ///         bitwriter.write_bit(i % 3).unwrap();
    ///     }
    /// }
    /// assert_eq!(output, vec![0b01101101, 0b10110110, 0b11011011]);
    /// ```
    fn write_bit(&mut self, bit: Bit) -> std::io::Result<()> {
        if bit != 0 {
            self.buffer |= self.mask;
        }

        self.mask >>= 1;
        if self.mask == 0 {
            write_u8(&mut self.writer, self.buffer as u8)?;
            self.buffer = 0;
            self.mask = 0x80;
        }

        Ok(())
    }

    /// Writes up to 32 bits to the stream.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use stdex::io::{BitWrite, BitWriterMSB};
    /// # use std::io::Write;
    /// let mut output: Vec<u8> = Vec::new();
    /// {
    ///     let mut bitwriter = BitWriterMSB::new(output.by_ref());
    ///     bitwriter.write_bits_32(0xabc, 12).unwrap();
    ///     bitwriter.write_bits_32(0xd, 4).unwrap();
    /// }
    /// assert_eq!(output, vec![0xab, 0xcd]);
    /// ```
    /// 
    /// # Panics
    /// 
    /// Panics if `count > 32`.
    fn write_bits_32(&mut self, value: u32, mut count: usize)
    -> std::io::Result<()> {
        assert!(count <= 32);
        if count == 0 {
            return Ok(());
        }

        let mut mask = 1 << (count - 1);
        while count > 0 && self.mask != 0x80 {
            if value & mask != 0 {
                self.buffer |= self.mask;
            }
            self.mask >>= 1;
            if self.mask == 0 {
                write_u8(&mut self.writer, self.buffer as u8)?;
                self.buffer = 0;
                self.mask = 0x80;
            }
            mask >>= 1;
            count -= 1;
        }

        while count >= 8 {
            let buffer = value >> (count - 8);
            write_u8(&mut self.writer, buffer as u8)?;
            mask >>= 8;
            count -= 8;
        }

        while count > 0 {
            if value & mask != 0 {
                self.buffer |= self.mask;
            }

            self.mask >>= 1;
            if self.mask == 0 {
                write_u8(&mut self.writer, self.buffer as u8)?;
                self.buffer = 0;
                self.mask = 0x80;
            }
            mask >>= 1;
            count -= 1;
        }

        Ok(())
    }

    /// Finishes writing any partially written byte.
    /// 
    /// Fills in remaining bits with `fill_bit`. If there are no partially
    /// written bytes, does nothing.
    fn finish_byte(&mut self, fill_bit: Bit) -> std::io::Result<()> {
        while self.mask != 0x80 {
            self.write_bit(fill_bit)?;
        }
        Ok(())
    }

    fn remaining_bits(&self) -> u8 {
        match self.mask {
            0x1 => 1,
            0x2 => 2,
            0x4 => 3,
            0x8 => 4,
            0x10 => 5,
            0x20 => 6,
            0x40 => 7,
            0x80 => 0,
            _ => unreachable!(),
        }
    }
}

impl<W: Write> Drop for BitWriterMSB<W> {
    fn drop(&mut self) {
        use crate::io::BitWrite;
        assert_eq!(self.remaining_bits(), 0, "bits remaining in BitWriter before dropping");
    }
}

mod bitwriter_tests {
    #[test]
    fn test_write_bit() {
        use crate::io::{BitWrite, BitWriterMSB};
        let mut vec: Vec<u8> = Vec::new();
        {
            use std::io::Write;
            let mut writer = BitWriterMSB::new(vec.by_ref());
            let bits = [
                0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 1, 0, 0, 0, 1, 1,
                0, 1, 0, 0, 0, 1, 0, 1,
                0, 1, 1, 0, 0, 1, 1, 1,
                1, 0, 0, 0, 1, 0, 0, 1,
                1, 0, 1, 0, 1, 0, 1, 1,
                1, 1, 0, 0, 1, 1, 0, 1,
                1, 1, 1, 0, 1, 1, 1, 1,
            ];

            for b in bits.iter() {
                assert_eq!(writer.write_bit(*b).ok(), Some(()));
            }
        }

        assert_eq!(vec, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
    }

    #[test]
    fn test_write_bits_32() {
        use crate::io::{BitRead, BitWrite, BitWriterMSB, BitReaderMSB};

        let fibonacci = generate_fibonacci();
        let mut vec: Vec<u8> = Vec::new();
        {
            use std::io::Write;
            let mut writer = BitWriterMSB::new(vec.by_ref());

            for x in fibonacci.iter() {
                assert_eq!(
                    writer.write_bits_32(*x, count_bits(*x)).ok(),
                    Some(())
                );
            }
            
            assert_eq!(writer.finish_byte(0).ok(), Some(()));
        }

        {
            let reader = std::io::Cursor::new(&vec);
            let mut reader = BitReaderMSB::new(reader);

            for x in fibonacci.iter() {
                assert_eq!(
                    reader.read_bits_32(count_bits(*x)).ok(),
                    Some(*x)
                );
            }
        }
    }

    #[cfg(test)]
    fn count_bits(n: u32) -> usize {
        (32 - n.leading_zeros()) as usize
    }

    #[cfg(test)]
    fn generate_fibonacci() -> Vec<u32> {
        let mut fib = Vec::new();
        let mut parents: [u32;2] = [0, 1];
        let mut index = 0;
        loop {
            let i = index % 2;
            let j = (index + 1) % 2;
            fib.push(parents[i]);
            if parents[j].leading_zeros() == 0 {
                break;
            }
            parents[index % 2] += parents[(index + 1) % 2];
            index += 1;
        }
        fib
    }
}
