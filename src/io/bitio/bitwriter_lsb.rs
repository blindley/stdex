use std::io::Write;
use crate::io::write_u8;

use super::Bit;

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
    /// # use stdex::io::{BitWrite, BitWriterLSB};
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

impl<W: Write> crate::io::BitWrite for BitWriterLSB<W> {

    /// Writes a single bit to the stream.
    /// 
    /// If `bit == 0`, writes a 0, otherwise writes a 1.
    /// 
    /// # Example
    /// 
    /// ```
    /// # use stdex::io::{BitWrite, BitWriterLSB};
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
    fn write_bit(&mut self, bit: Bit) -> std::io::Result<()> {
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
    /// # use stdex::io::{BitWrite, BitWriterLSB};
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
    fn write_bits_32(&mut self, value: u32, mut count: usize)
    -> std::io::Result<()> {
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
    fn finish_byte(&mut self, fill_bit: Bit) -> std::io::Result<()> {
        while self.mask != 0x1 {
            self.write_bit(fill_bit)?;
        }
        Ok(())
    }

    fn remaining_bits(&self) -> u8 {
        match self.mask {
            0x1 => 0,
            0x2 => 7,
            0x4 => 6,
            0x8 => 5,
            0x10 => 4,
            0x20 => 3,
            0x40 => 2,
            0x80 => 1,
            _ => unreachable!(),
        }
    }
}

impl<W: Write> Drop for BitWriterLSB<W> {
    fn drop(&mut self) {
        use crate::io::BitWrite;
        assert_eq!(self.remaining_bits(), 0, "bits remaining in BitWriter before dropping");
    }
}

mod bitwriterlsb_tests {
    #[test]
    fn test_write_bit() {
        use std::io::Write;
        use crate::io::{BitWrite, BitWriterLSB};
        let mut output = Vec::new();
        {
            let mut writer = BitWriterLSB::new(output.by_ref());
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
        use crate::io::{BitWriterLSB, BitWrite};
        let mut output = Vec::new();
        {
            let mut writer = BitWriterLSB::new(output.by_ref());
            writer.write_bits_32(0xa, 4).unwrap();
            writer.write_bits_32(0xbc, 8).unwrap();
            writer.write_bits_32(0xdef, 12).unwrap();
        }

        assert_eq!(output, vec![0xca, 0xfb, 0xde]);
    }
}
