use std::io::{self, Read, Write};

mod bitio;
pub use self::bitio::{
    BitRead, BitWrite,
    BitReaderMSB, BitWriterMSB,
    BitReaderLSB, BitWriterLSB
};

mod deflate;
pub use self::deflate::DeflateDecompressor;

unsafe fn as_u8_slice<T>(data: &T) -> &[u8] {
    let ptr = data as *const T as *const u8;
    let len = std::mem::size_of::<T>();
    std::slice::from_raw_parts(ptr, len)
}

unsafe fn as_u8_slice_mut<T>(data: &mut T) -> &mut [u8] {
    let ptr = data as *mut T as *mut u8;
    let len = std::mem::size_of::<T>();
    std::slice::from_raw_parts_mut(ptr, len)
}

fn read_item<T, R: Read>(reader: &mut R) -> io::Result<T> {
    let mut result: T = unsafe { std::mem::uninitialized() };
    unsafe {
        let data = as_u8_slice_mut(&mut result);
        reader.read_exact(data)?;
    }
    Ok(result)
}

fn write_item<T, W: Write>(writer: &mut W, item: T) -> io::Result<()> {
    unsafe {
        let data = as_u8_slice(&item);
        writer.write_all(data)
    }
}

pub fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    read_item::<u8,_>(reader)
}

pub fn write_u8(reader: &mut impl Write, item: u8) -> io::Result<()> {
    write_item(reader, item)
}

macro_rules! impl_endian_readers {
    ($type:ty, $read_be:ident, $read_le:ident,
     $write_be:ident, $write_le:ident) => {
        pub fn $read_be(reader: &mut impl Read) -> io::Result<$type> {
            Ok(<$type>::from_be(read_item::<$type,_>(reader)?))
        }

        pub fn $read_le(reader: &mut impl Read) -> io::Result<$type> {
            Ok(<$type>::from_le(read_item::<$type,_>(reader)?))
        }

        pub fn $write_be(writer: &mut impl Write, item: $type)
        -> io::Result<()> {
            let item = <$type>::to_be(item);
            write_item(writer, &item)
        }

        pub fn $write_le(writer: &mut impl Write, item: $type)
        -> io::Result<()> {
            let item = <$type>::to_le(item);
            write_item(writer, &item)
        }
    };
}

impl_endian_readers!(i8, read_i8_be, read_i8_le, write_i8_be, write_i8_le);
impl_endian_readers!(i16, read_i16_be, read_i16_le, write_i16_be, write_i16_le);
impl_endian_readers!(i32, read_i32_be, read_i32_le, write_i32_be, write_i32_le);
impl_endian_readers!(i64, read_i64_be, read_i64_le, write_i64_be, write_i64_le);
impl_endian_readers!(i128, read_i128_be, read_i128_le, write_i128_be, write_i128_le);
impl_endian_readers!(u8, read_u8_be, read_u8_le, write_u8_be, write_u8_le);
impl_endian_readers!(u16, read_u16_be, read_u16_le, write_u16_be, write_u16_le);
impl_endian_readers!(u32, read_u32_be, read_u32_le, write_u32_be, write_u32_le);
impl_endian_readers!(u64, read_u64_be, read_u64_le, write_u64_be, write_u64_le);
impl_endian_readers!(u128, read_u128_be, read_u128_le, write_u128_be, write_u128_le);