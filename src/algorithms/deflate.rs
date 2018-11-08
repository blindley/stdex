use std::io::Read;
use crate::error::{error_if, BoxResult};
use crate::huffman::{Node, Code, CodeString};
use crate::io::{BitRead, BitReaderLSB};
use crate::collections::BitString;

struct Codes {
    litlen: Vec<Code<u16>>,
    distance: Vec<Code<u16>>,
}

pub fn inflate<R: Read>(reader: &mut R, output: &mut Vec<u8>) -> BoxResult<()> {
    let mut bitreader = BitReaderLSB::new(reader);
    loop {
        let bfinal = bitreader.read_bit()? != 0;
        match bitreader.read_bits_32(2)? {
            0 => {
                read_uncompressed_block(&mut bitreader, output)?;
            },
            btype @ 1...2 => {
                let codes = match btype {
                    1 => fixed_huffman_codes(),
                    2 => dynamic_huffman_codes(&mut bitreader)?,
                    _ => unreachable!(),
                };

                read_huffman_compressed_block(&codes, &mut bitreader, output)?;
            },
            3 => {
                error_if(true, "bad btype value")?;
            },
            _ => unreachable!(),
        }

        if bfinal { break; }
    }

    Ok(())
}

fn read_uncompressed_block<R: Read>(bitreader: &mut BitReaderLSB<R>,
output: &mut Vec<u8>) -> Result<(), Box<std::error::Error>> {
    bitreader.flush_byte();
    let reader = bitreader.as_read_mut();
    let len = crate::io::read_u16_le(reader)?;
    let nlen = crate::io::read_u16_le(reader)?;

    error_if(len != !nlen, "non-matching len/nlen")?;

    let old_len = output.len();
    let new_len = old_len + len as usize;
    output.reserve(len as usize);
    unsafe {
        output.set_len(new_len);
    }

    reader.read_exact(&mut output[old_len..new_len])?;    

    Ok(())
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
-> Result<Codes, Box<std::error::Error>> {
    let hlit = bitreader.read_bits_32(5)? as usize + 257;
    let hdist = bitreader.read_bits_32(5)? as usize + 1;
    let hclen = bitreader.read_bits_32(4)? as usize + 4;

    const SWIZZLE: [usize;19] =
        [ 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15 ];

    let mut code_lengths = [0;19];
    for i in 0..hclen {
        code_lengths[SWIZZLE[i]] = bitreader.read_bits_32(3)? as u32;
    }

    let codes = Code::canonical_from_lengths(0, &code_lengths)?;
    let tree = Node::from_codes(&codes)?;

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
                error_if(true, "bad code lengths")?;
                unreachable!();
            }
        }
    }

    let litlen = Code::canonical_from_lengths(0, &code_lengths[..hlit])?;
    let distance = Code::canonical_from_lengths(0, &code_lengths[hlit..])?;

    Ok(Codes {
        litlen,
        distance,
    })
}

fn read_huffman_compressed_block<R: Read>(codes: &Codes,
bitreader: &mut BitReaderLSB<R>, output: &mut Vec<u8>)
-> Result<(), Box<std::error::Error>> {
    let litlen_tree = Node::from_codes(&codes.litlen)?;
    let dist_tree = Node::from_codes(&codes.distance)?;

    loop {
        match litlen_tree.read_value(bitreader)? {
            value @ 0...255 => {
                output.push(value as u8);
            },
            256 => break,
            value @ 257...285 => {
                read_length_distance_pair(value - 257, &dist_tree, bitreader, output)?;
            },
            286...287 => { // error in data
                error_if(true, "invalid litlen code")?;
            },
            _ => unreachable!(), // error in my code
        }
    }

    Ok(())
}

fn read_length_distance_pair<R: Read>(code : u16, dist_tree: &Node<u16>,
bitreader: &mut BitReaderLSB<R>, output: &mut Vec<u8>)
-> Result<(), Box<std::error::Error>> {
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

    let mut from_index = output.len() - dist;
    for _ in 0..len {
        let value = output[from_index];
        output.push(value);
        from_index += 1;
    }

    Ok(())
}