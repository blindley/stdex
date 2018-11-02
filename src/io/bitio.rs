use std::io::{self, Read, Write};
use crate::io::{read_u8, write_u8};

type Bit = u8;

pub struct BitReader<R: Read> {
    reader: R,
    buffer: u32,
    mask: u32,
}

impl<R: Read> BitReader<R> {
    pub fn new(reader: R) -> BitReader<R> {
        BitReader {
            reader,
            buffer: 0,
            mask: 0x80,
        }
    }

    pub fn read_bit(&mut self) -> io::Result<Bit> {
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

    pub fn read_bits_32(&mut self, mut count: usize) -> io::Result<u32> {
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
}

mod bitreader_tests {
    #[test]
    fn test_read_bit() {
        let vec: Vec<u8> = (0..=255).collect();
        let reader = std::io::Cursor::new(vec.clone());
        let mut reader = super::BitReader::new(reader);
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
        let vec: Vec<u8> = (0..=255).collect();
        let reader = std::io::Cursor::new(vec.clone());
        let mut reader = super::BitReader::new(reader);

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

pub struct BitWriter<W: Write> {
    writer: W,
    buffer: u32,
    mask: u32,
}

impl<W: Write> BitWriter<W> {
    pub fn new(writer: W) -> BitWriter<W> {
        BitWriter {
            writer,
            buffer: 0,
            mask: 0x80,
        }
    }

    pub fn write_bit(&mut self, bit: Bit) -> io::Result<()> {
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

    pub fn write_bits_32(&mut self, mut value: u32, mut count: usize)
    -> io::Result<()> {
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

    pub fn finish_byte(&mut self, fill_bit: Bit) -> io::Result<()> {
        while self.mask != 0x80 {
            self.write_bit(fill_bit)?;
        }
        Ok(())
    }
}

mod bitwriter_tests {
    #[test]
    fn test_write_bit() {
        let mut vec: Vec<u8> = Vec::new();
        {
            use std::io::Write;
            let mut writer = super::BitWriter::new(vec.by_ref());
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
        let fibonacci = generate_fibonacci();
        let mut vec: Vec<u8> = Vec::new();
        {
            use std::io::Write;
            let mut writer = super::BitWriter::new(vec.by_ref());

            for x in fibonacci.iter() {
                assert_eq!(
                    writer.write_bits_32(*x, count_bits(*x)).ok(),
                    Some(())
                );
            }
            
            writer.finish_byte(0);
        }

        {
            let reader = std::io::Cursor::new(&vec);
            let mut reader = super::BitReader::new(reader);

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