use crate::collections::BitString;
use crate::error::{error_if, SimpleResult};
use crate::Increment;
use crate::collections::CompactBitString27 as CodeString;


#[derive(Debug, Clone)]
pub enum Node<T> {
    Leaf(T),
    Branch(Box<Node<T>>, Box<Node<T>>),
}

#[derive(Debug, Clone, Copy)]
pub struct Code<T> {
    pub value: T,
    pub code: CodeString,
}

impl<T> Node<T> {
    pub fn from_codes(codes: &[Code<T>]) -> SimpleResult<Node<T>>
    where T: Clone {
        match codes.len() {
            0 => {
                Err(error_if(true, "empty huffman code list").unwrap_err())
            }
            1 => {
                error_if(codes[0].code.len() != 0, "not a huffman code, superfluous bits")?;
                Ok(Node::Leaf(codes[0].value.clone()))
            },
            _ => {
                let mut child_0 = Vec::new();
                let mut child_1 = Vec::new();
                for Code{ value, code } in codes.iter() {
                    error_if(code.len() == 0, "not a prefix code")?;
                    let mut code = code.clone();
                    let bit = code.pop_bit_front();
                    match bit {
                        0 => child_0.push(Code{ value: value.clone(), code }),
                        _ => child_1.push(Code{ value: value.clone(), code }),
                    }
                }

                let child_0 = Node::from_codes(&child_0)?;
                let child_1 = Node::from_codes(&child_1)?;

                Ok(Node::Branch(Box::new(child_0), Box::new(child_1)))
            }
        }
    }

    pub fn read_value<R: crate::io::BitRead>(&self, bitreader: &mut R)
    -> std::io::Result<T> where T: Clone {
        match self {
            Node::Leaf(value) => Ok(value.clone()),
            Node::Branch(child_0, child_1) => {
                match bitreader.read_bit()? {
                    0 => child_0.read_value(bitreader),
                    _ => child_1.read_value(bitreader),
                }
            },
        }
    }
}

impl<T: Increment + Clone> Code<T> {
    pub fn canonical_from_lengths(first_value: T, code_lengths: &[u32])
    -> SimpleResult<Vec<Code<T>>> {
        assert_ne!(code_lengths.len(), 0, "no code lengths");
        let max_length = *code_lengths.iter().max().unwrap();
        
        Code::canonical_from_lengths_known_max(first_value, code_lengths, max_length)
    }

    pub fn canonical_from_lengths_known_max(first_value: T, code_lengths: &[u32], max_length: u32)
    -> SimpleResult<Vec<Code<T>>> {
        const MAX_STACK_BUFFER_LENGTH: usize = 4096;
        let buffer_len = max_length as usize + 1;
        if buffer_len <= MAX_STACK_BUFFER_LENGTH {
            let mut length_counts: [u32;MAX_STACK_BUFFER_LENGTH] = unsafe {
                std::mem::uninitialized()
            };

            for i in 0..buffer_len {
                length_counts[i] = 0;
            }

            let mut next_code: [u32;MAX_STACK_BUFFER_LENGTH] = unsafe {
                std::mem::uninitialized()
            };

            Code::canonical_from_lengths_impl(first_value, code_lengths, max_length,
                &mut length_counts[..buffer_len], &mut next_code[..buffer_len])
        } else {
            let mut length_counts = Vec::new();
            length_counts.resize(buffer_len, 0);
            let mut next_code = Vec::with_capacity(buffer_len);
            unsafe { next_code.set_len(buffer_len); }
            Code::canonical_from_lengths_impl(first_value, code_lengths, max_length,
                &mut length_counts, &mut next_code)
        }
    }

    fn canonical_from_lengths_impl(first_value: T, code_lengths: &[u32], max_length: u32,
    length_counts: &mut [u32], next_code: &mut [u32]) -> SimpleResult<Vec<Code<T>>> {
        for i in code_lengths {
            length_counts[*i as usize] += 1;
        }
        let mut code = 0;
        for bits in 1..=(max_length as usize) {
            code = (code + length_counts[bits - 1] as u32) << 1;
            next_code[bits] = code;
        }

        let mut codes = Vec::new();
        let mut value = first_value.clone();
        for len in code_lengths {
            let j = *len as usize;
            if j != 0 {
                codes.push(Code {
                    value: value.clone(),
                    code: CodeString::new(j, next_code[j]),
                });
                next_code[j] += 1;
            }
            value.increment();
        }

        Ok(codes)
    }
}

macro_rules! impl_increment_for_integer {
    ($($t:ty)*) => {
        $(
            impl Increment for $t {
                #[inline]
                fn increment(&mut self) { *self += 1; }
            }
        )*
    };
}

impl_increment_for_integer! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}

mod tests {
    #[cfg(test)]
    use super::{Code, CodeString};
    #[cfg(test)]
    use crate::collections::BitString;

    #[test]
    fn test_decode() {
        // test deflate format fixed huffman codes
        let mut code_lengths: [u32;288] = unsafe { std::mem::uninitialized() };
        for i in 0..=143 { code_lengths[i] = 8; }
        for i in 144..=255 { code_lengths[i] = 9; }
        for i in 256..=279 { code_lengths[i] = 7; }
        for i in 280..=287 { code_lengths[i] = 8; }

        let codes = Code::canonical_from_lengths(0u16, &code_lengths).unwrap();

        let mut code_bits = 0b00110000;
        for i in 0..=143 {
            let code_string = CodeString::new(8, code_bits);
            assert_eq!(codes[i].code, code_string);
            assert_eq!(codes[i].value, i as u16);
            code_bits += 1;
        }

        code_bits = 0b110010000;
        for i in 144..=255 {
            let code_string = CodeString::new(9, code_bits);
            assert_eq!(codes[i].code, code_string);
            assert_eq!(codes[i].value, i as u16);
            code_bits += 1;
        }

        code_bits = 0;
        for i in 256..=279 {
            let code_string = CodeString::new(7, code_bits);
            assert_eq!(codes[i].code, code_string);
            assert_eq!(codes[i].value, i as u16);
            code_bits += 1;
        }

        code_bits = 0b11000000;
        for i in 280..=287 {
            let code_string = CodeString::new(8, code_bits);
            assert_eq!(codes[i].code, code_string);
            assert_eq!(codes[i].value, i as u16);
            code_bits += 1;
        }
    }
}
