mod bitreader_msb;
pub use self::bitreader_msb::BitReaderMSB;

mod bitwriter_msb;
pub use self::bitwriter_msb::BitWriterMSB;

mod bitreader_lsb;
pub use self::bitreader_lsb::BitReaderLSB;

mod bitwriter_lsb;
pub use self::bitwriter_lsb::BitWriterLSB;

pub type Bit = u8;

pub trait BitRead {
    /// Reads a single bit from the stream
    fn read_bit(&mut self) -> std::io::Result<Bit>;

    /// Reads up to 32 bits from the stream
    fn read_bits_32(&mut self, count: usize) -> std::io::Result<u32>;

    /// Discards any remaining bits of a partially read byte
    fn flush_byte(&mut self);
}

pub trait BitWrite {
    /// Writes a single bit to the stream.
    fn write_bit(&mut self, bit: Bit) -> std::io::Result<()>;

    /// Writes up to 32 bits to the stream.
    fn write_bits_32(&mut self, value: u32, count: usize)
        -> std::io::Result<()>;

    /// Finishes writing any partially written byte.
    fn finish_byte(&mut self, fill_bit: Bit) -> std::io::Result<()>;

    /// Returns the number of bits left in any partially written byte.
    fn remaining_bits(&self) -> u8;
}
