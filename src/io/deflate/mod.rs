use std::io::Read;
use crate::huffman::{Node, Code, CodeString};
use crate::io::{BitRead, BitReaderLSB};
use crate::collections::BitString;
use crate::io::{write_u8};

mod ring_buffer;

struct Codes {
    litlen: Vec<Code<u16>>,
    distance: Vec<Code<u16>>,
}

fn fixed_huffman_codes() -> Codes {
    let mut litlen = Vec::with_capacity(288);

    let mut bits = 0b00110000;
    for value in 0..=143 {
        let code = CodeString::new(8, bits);
        litlen.push(Code { value, code });
        bits += 1;
    }

    let mut bits = 0b110010000;
    for value in 144..=255 {
        let code = CodeString::new(9, bits);
        litlen.push(Code { value, code });
        bits += 1;
    }

    let mut bits = 0;
    for value in 256..=279 {
        let code = CodeString::new(7, bits);
        litlen.push(Code { value, code });
        bits += 1;
    }

    let mut bits = 0b11000000;
    for value in 280..=287 {
        let code = CodeString::new(8, bits);
        litlen.push(Code { value, code });
        bits += 1;
    }

    let mut distance = Vec::new();
    for value in 0..=31 {
        let code = CodeString::new(5, value as u32);
        distance.push(Code { value, code });
    }

    Codes { litlen, distance }
}

fn dynamic_huffman_codes<R: Read>(bitreader: &mut BitReaderLSB<R>)
-> std::io::Result<Codes> {
    let hlit = bitreader.read_bits_32(5)? as usize + 257;
    let hdist = bitreader.read_bits_32(5)? as usize + 1;
    let hclen = bitreader.read_bits_32(4)? as usize + 4;

    const SWIZZLE: [usize;19] =
        [ 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15 ];

    let mut code_lengths = [0;19];
    for i in 0..hclen {
        code_lengths[SWIZZLE[i]] = bitreader.read_bits_32(3)? as u32;
    }

    let codes = Code::canonical_from_lengths(0, &code_lengths).or_else(
        |_| Err(DeflateDecompressorError::BadHuffmanCodes)
    )?;

    let tree = Node::from_codes(&codes).or_else(
        |_| Err(DeflateDecompressorError::BadHuffmanCodes)
    )?;

    let mut code_lengths = [0;288 + 32];
    let mut i = 0;
    let mut previous = 0;
    while i < hlit + hdist {
        match tree.read_value(bitreader)? {
            n @ 0...15 => {
                code_lengths[i] = n as u32;
                previous = n;
                i += 1;
            },
            16 => {
                let repeat_len = bitreader.read_bits_32(2)? + 3;
                for _ in 0..repeat_len {
                    code_lengths[i] = previous as u32;
                    i += 1;
                }
            },
            17 => {
                let repeat_len = bitreader.read_bits_32(3)? + 3;
                previous = 0;
                i += repeat_len as usize;
            },
            18 => {
                let repeat_len = bitreader.read_bits_32(7)? + 11;
                previous = 0;
                i += repeat_len as usize;
            },
            _ => {
                return Err(DeflateDecompressorError::BadHuffmanCodes.into());
            }
        }
    }

    let litlen = Code::canonical_from_lengths(0, &code_lengths[..hlit]).or_else(
        |_| Err(DeflateDecompressorError::BadHuffmanCodes)
    )?;

    let distance = Code::canonical_from_lengths(0, &code_lengths[hlit..]).or_else(
        |_| Err(DeflateDecompressorError::BadHuffmanCodes)
    )?;

    Ok(Codes {
        litlen,
        distance,
    })
}

fn read_length_distance_pair<R: Read>(code : u16, dist_tree: &Node<u16>, bitreader: &mut BitReaderLSB<R>)
-> std::io::Result<(u16, u16)> {
    const LENGTH_BASE: [usize;29] = [
        3,4,5,6,7,8,9,10,11,13,
        15,17,19,23,27,31,35,43,51,59,
        67,83,99,115,131,163,195,227,258
    ];

    const LENGTH_EXTRA: [usize;29] = [
        0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0
    ];

    const DIST_BASE: [usize;32] = [
        1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,
        257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577,0,0
    ];

    const DIST_EXTRA: [usize;32] = [
        0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13,0,0
    ];

    let mut len = LENGTH_BASE[code as usize];
    if LENGTH_EXTRA[code as usize] != 0 {
        len += bitreader.read_bits_32(LENGTH_EXTRA[code as usize])? as usize;
    }

    let code = dist_tree.read_value(bitreader)?;
    let mut dist = DIST_BASE[code as usize];
    if DIST_EXTRA[code as usize] != 0 {
        dist += bitreader.read_bits_32(DIST_EXTRA[code as usize])? as usize;
    }

    Ok((len as u16, dist as u16))
}

pub struct DeflateDecompressor<R: Read> {
    bitreader: crate::io::BitReaderLSB<R>,
    state: DeflateDecompressorState,
    window: ring_buffer::RingBuffer,
    available: usize,
}

struct HuffmanState {
    litlen_tree: crate::huffman::Node<u16>,
    distance_tree: crate::huffman::Node<u16>,
    distance: u16,
    copy_len: u16,
}

enum DeflateDecompressorState {
    Uncompressed((bool, u16)),
    Huffman((bool, HuffmanState)),
    Complete,
}

impl DeflateDecompressorState {
    fn from_bitreader<R: Read>(bitreader: &mut crate::io::BitReaderLSB<R>)
    -> std::io::Result<DeflateDecompressorState> {
        let bfinal = bitreader.read_bit()? != 0;
        let state = match bitreader.read_bits_32(2)? {
            0 => {
                bitreader.flush_byte();
                let reader = bitreader.as_read_mut();
                let len = crate::io::read_u16_le(reader)?;
                let nlen = crate::io::read_u16_le(reader)?;

                if len != !nlen {
                    return Err(DeflateDecompressorError::NonMatchingLenNLen.into());
                }

                DeflateDecompressorState::Uncompressed((bfinal, len))
            },
            btype @ 1...2 => {
                let codes = match btype {
                    1 => fixed_huffman_codes(),
                    2 => dynamic_huffman_codes(bitreader)?,
                    _ => unreachable!(),
                };

                let litlen_tree = crate::huffman::Node::from_codes(&codes.litlen).or_else(
                    |_| Err(DeflateDecompressorError::BadHuffmanCodes)
                )?;

                let distance_tree = crate::huffman::Node::from_codes(&codes.distance).or_else(
                    |_| Err(DeflateDecompressorError::BadHuffmanCodes)
                )?;

                DeflateDecompressorState::Huffman(
                    (bfinal, HuffmanState {
                        litlen_tree, distance_tree, distance: 0, copy_len: 0,
                    })
                )
            },
            3 => {
                return Err(DeflateDecompressorError::InvalidBType.into());
            },
            _ => unreachable!(),
        };

        Ok(state)
    }
}

impl<R: Read> DeflateDecompressor<R> {
    pub fn new(reader: R) -> std::io::Result<DeflateDecompressor<R>> {
        let mut bitreader = crate::io::BitReaderLSB::new(reader);
        let state = DeflateDecompressorState::from_bitreader(&mut bitreader)?;
        let window = ring_buffer::RingBuffer::new(32768);

        Ok(DeflateDecompressor {
            bitreader,
            state,
            window,
            available: 0,
        })
    }

    pub fn make_available(&mut self, required: usize) -> std::io::Result<usize> {
        assert!(required <= 32768, "too many bytes requested at once");
        while required > self.available {
            let mut state_change = None;
            match &mut self.state {
                DeflateDecompressorState::Uncompressed((bfinal,uncompressed)) => {
                    let can_read = 32768 - self.available;
                    let to_read = std::cmp::min(*uncompressed as usize, can_read);
                    io_copy(self.bitreader.as_read_mut(), &mut self.window, to_read)?;
                    self.available += to_read;

                    *uncompressed -= to_read as u16;
                    if *uncompressed == 0 {
                        state_change = Some(*bfinal);
                    }
                },
                DeflateDecompressorState::Huffman((bfinal, huffstate)) => {
                    if huffstate.copy_len == 0 {
                        match huffstate.litlen_tree.read_value(&mut self.bitreader)? {
                            value @ 0...255 => {
                                write_u8(&mut self.window, value as u8)?;
                                self.available += 1;
                            },
                            256 => {
                                state_change = Some(*bfinal);
                            },
                            value @ 257...285 => {
                                let (len, dist) = read_length_distance_pair(value - 257,
                                    &huffstate.distance_tree, &mut self.bitreader)?;
                                huffstate.distance = dist;
                                huffstate.copy_len = len;
                            },
                            286...287 => { // error in data
                                return Err(DeflateDecompressorError::InvalidLitLenCode.into());
                            },
                            _ => unreachable!(), // error in my code
                        }
                    } else {
                        let required = required - self.available;
                        let copy_len = std::cmp::min(huffstate.copy_len as usize, required);
                        self.window.self_copy(huffstate.distance as usize, copy_len)?;
                        self.available += copy_len;
                        huffstate.copy_len -= copy_len as u16;
                    }
                },
                DeflateDecompressorState::Complete => {
                    break;
                }
            }

            if let Some(bfinal) = state_change {
                self.state = match bfinal {
                    true => DeflateDecompressorState::Complete,
                    false => DeflateDecompressorState::from_bitreader(&mut self.bitreader)?,
                }
            }
        }
        Ok(self.available)
    }
}

impl<R: Read> Read for DeflateDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read_so_far = 0;
        while buf.len() - read_so_far > 32768 {
            let end = read_so_far + 32768;
            match self.read(&mut buf[read_so_far..end]) {
                Ok(0) => return Ok(read_so_far),
                Ok(n) => read_so_far += n,
                Err(e) => return match read_so_far {
                    0 => Err(e),
                    _ => Ok(read_so_far)
                },
            }
        }

        let to_read = buf.len() - read_so_far;
        match self.make_available(to_read) {
            Ok(0) => Ok(0),
            Ok(available) => {
                let to_read = std::cmp::min(available, to_read);
                let write_end = read_so_far + to_read;
                self.window.copy_out(&mut buf[read_so_far..write_end], available);
                self.available -= to_read;
                read_so_far += to_read;
                Ok(read_so_far)
            },
            Err(e) => match read_so_far {
                0 => Err(e.into()),
                _ => Ok(read_so_far),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DeflateDecompressorError {
    General,
    NonMatchingLenNLen,
    BadHuffmanCodes,
    InvalidBType,
    UnexpectedEOF,
    InvalidLitLenCode,
}

impl std::fmt::Display for DeflateDecompressorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::DeflateDecompressorError::*;
        match self {
            General => write!(f, "Deflate Decompressor Error"),
            NonMatchingLenNLen => write!(f, "Non-matching len/nlen in uncompressed block"),
            BadHuffmanCodes => write!(f, "Bad Huffman codes"),
            InvalidBType => write!(f, "Invalie btype code"),
            UnexpectedEOF => write!(f, "Unexpected end of file"),
            InvalidLitLenCode => write!(f, "Invalid Lit/Len code"),
        }
    }
}

impl std::error::Error for DeflateDecompressorError {}

impl From<DeflateDecompressorError> for std::io::Error {
    fn from(e: DeflateDecompressorError) -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e
        )
    }
}

fn io_copy<R: std::io::Read, W: std::io::Write>(
    reader: &mut R, writer: &mut W, byte_count: usize,
) -> std::io::Result<()> {
    let mut buffer: [u8; 16384] = unsafe { std::mem::uninitialized() };
    let mut remaining = byte_count;
    while remaining > 0 {
        let to_read = if remaining > 16384 { 16384 } else { remaining };

        reader.read_exact(&mut buffer[0..to_read])?;
        remaining -= to_read;
        writer.write(&buffer[0..to_read])?;
    }

    Ok(())
}

mod tests {
    #[test]
    fn test_deflate_decompressor() {
        
    }
}