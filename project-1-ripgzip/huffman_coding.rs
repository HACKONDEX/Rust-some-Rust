#![forbid(unsafe_code)]

use std::{collections::HashMap, convert::TryFrom, io::BufRead};

use anyhow::{anyhow, bail, Result};
use log::debug;

use crate::bit_reader::{BitReader, BitSequence};

////////////////////////////////////////////////////////////////////////////////

fn find_sizes<T: BufRead>(
    bit_reader: &mut BitReader<T>,
    num_tokens: usize,
    encoder_tree: &HuffmanCoding<TreeCodeToken>,
) -> Result<Vec<u8>> {
    let mut sizes = Vec::<u8>::new();
    sizes.reserve(num_tokens);
    while sizes.len() < num_tokens {
        match encoder_tree.read_symbol(bit_reader)? {
            TreeCodeToken::Length(len) => {
                sizes.push(len);
            }
            TreeCodeToken::CopyPrev => {
                let num_copies = bit_reader.read_bits(2)?.bits() + 3;
                for _ in 0..num_copies {
                    sizes.push(*sizes.last().unwrap());
                }
            }
            TreeCodeToken::RepeatZero { base, extra_bits } => {
                let num_copies = bit_reader.read_bits(extra_bits)?.bits() + base;
                sizes.resize(sizes.len() + num_copies as usize, 0);
            }
        }
    }
    Ok(sizes)
}

pub fn decode_litlen_distance_trees<T: BufRead>(
    bit_reader: &mut BitReader<T>,
) -> Result<(HuffmanCoding<LitLenToken>, HuffmanCoding<DistanceToken>)> {
    let hlit = bit_reader.read_bits(5)?.bits() + 257;
    let hdist = bit_reader.read_bits(5)?.bits() + 1;
    let hclen = bit_reader.read_bits(4)?.bits() + 4;
    debug!("hlit {}, hdist{}, hclen {}", hlit, hdist, hclen);
    let mapper = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut code_len: [u8; 19] = [0; 19];
    for ind in 0..hclen {
        code_len[mapper[ind as usize]] = bit_reader.read_bits(3)?.bits() as u8;
    }

    let encoder_tree = HuffmanCoding::<TreeCodeToken>::from_lengths(&code_len)?;

    let litlen_sizes = find_sizes(bit_reader, hlit as usize, &encoder_tree)?;

    let distance_sizes = find_sizes(bit_reader, hdist as usize, &encoder_tree)?;

    let litlen_coder = HuffmanCoding::<LitLenToken>::from_lengths(litlen_sizes.as_slice())?;
    let distance_coder = HuffmanCoding::<DistanceToken>::from_lengths(distance_sizes.as_slice())?;
    Ok((litlen_coder, distance_coder))
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum TreeCodeToken {
    Length(u8),
    CopyPrev,
    RepeatZero { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for TreeCodeToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            0..=15 => Ok(Self::Length(value.0 as u8)),
            16 => Ok(Self::CopyPrev),
            17 => Ok(Self::RepeatZero {
                base: 3,
                extra_bits: 3,
            }),
            18 => Ok(Self::RepeatZero {
                base: 11,
                extra_bits: 7,
            }),
            19.. => Err(anyhow!("no such code for TreeCodeToken")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum LitLenToken {
    Literal(u8),
    EndOfBlock,
    Length { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for LitLenToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            0..=255 => Ok(Self::Literal(value.0 as u8)),
            256 => Ok(Self::EndOfBlock),
            257..=264 => Ok(Self::Length {
                base: value.0 - 254,
                extra_bits: 0,
            }),
            265..=268 => Ok(Self::Length {
                base: 11 + (value.0 - 265) * 2,
                extra_bits: 1,
            }),
            269..=272 => Ok(Self::Length {
                base: 19 + (value.0 - 269) * 4,
                extra_bits: 2,
            }),
            273..=276 => Ok(Self::Length {
                base: 35 + (value.0 - 273) * 8,
                extra_bits: 3,
            }),
            277..=280 => Ok(Self::Length {
                base: 67 + (value.0 - 277) * 16,
                extra_bits: 4,
            }),
            281..=284 => Ok(Self::Length {
                base: 131 + (value.0 - 281) * 32,
                extra_bits: 5,
            }),
            285 => Ok(Self::Length {
                base: 258,
                extra_bits: 0,
            }),
            _ => bail!("No such LitLen Token"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub struct DistanceToken {
    pub base: u16,
    pub extra_bits: u8,
}

impl TryFrom<HuffmanCodeWord> for DistanceToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        // See RFC 1951, section 3.2.5.
        match value.0 {
            0..=3 => Ok(DistanceToken {
                base: value.0 + 1,
                extra_bits: 0,
            }),
            4 => Ok(DistanceToken {
                base: 5,
                extra_bits: 1,
            }),
            5 => Ok(DistanceToken {
                base: 7,
                extra_bits: 1,
            }),
            6 => Ok(DistanceToken {
                base: 9,
                extra_bits: 2,
            }),
            7 => Ok(DistanceToken {
                base: 13,
                extra_bits: 2,
            }),
            8 => Ok(DistanceToken {
                base: 17,
                extra_bits: 3,
            }),
            9 => Ok(DistanceToken {
                base: 25,
                extra_bits: 3,
            }),
            10 => Ok(DistanceToken {
                base: 33,
                extra_bits: 4,
            }),
            11 => Ok(DistanceToken {
                base: 49,
                extra_bits: 4,
            }),
            12 => Ok(DistanceToken {
                base: 65,
                extra_bits: 5,
            }),
            13 => Ok(DistanceToken {
                base: 97,
                extra_bits: 5,
            }),
            14 => Ok(DistanceToken {
                base: 129,
                extra_bits: 6,
            }),
            15 => Ok(DistanceToken {
                base: 193,
                extra_bits: 6,
            }),
            16 => Ok(DistanceToken {
                base: 257,
                extra_bits: 7,
            }),
            17 => Ok(DistanceToken {
                base: 385,
                extra_bits: 7,
            }),
            18 => Ok(DistanceToken {
                base: 513,
                extra_bits: 8,
            }),
            19 => Ok(DistanceToken {
                base: 769,
                extra_bits: 8,
            }),
            20 => Ok(DistanceToken {
                base: 1025,
                extra_bits: 9,
            }),
            21 => Ok(DistanceToken {
                base: 1537,
                extra_bits: 9,
            }),
            22 => Ok(DistanceToken {
                base: 2049,
                extra_bits: 10,
            }),
            23 => Ok(DistanceToken {
                base: 3073,
                extra_bits: 10,
            }),
            24 => Ok(DistanceToken {
                base: 4097,
                extra_bits: 11,
            }),
            25 => Ok(DistanceToken {
                base: 6145,
                extra_bits: 11,
            }),
            26 => Ok(DistanceToken {
                base: 8193,
                extra_bits: 12,
            }),
            27 => Ok(DistanceToken {
                base: 12289,
                extra_bits: 12,
            }),
            28 => Ok(DistanceToken {
                base: 16385,
                extra_bits: 13,
            }),
            29 => Ok(DistanceToken {
                base: 24577,
                extra_bits: 13,
            }),
            30..=u16::MAX => Err(anyhow!("Distance token wrong code")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

const MAX_BITS: usize = 15;

pub struct HuffmanCodeWord(pub u16);

pub struct HuffmanCoding<T> {
    map: HashMap<BitSequence, T>,
}

impl<T> HuffmanCoding<T>
where
    T: Copy + TryFrom<HuffmanCodeWord, Error = anyhow::Error>,
{
    #[allow(unused)]
    pub fn decode_symbol(&self, seq: BitSequence) -> Option<T> {
        self.map.get(&seq).copied()
    }

    pub fn read_symbol<U: BufRead>(&self, bit_reader: &mut BitReader<U>) -> Result<T> {
        let mut result_symbol = BitSequence::new(0, 0);
        loop {
            match bit_reader.read_bits(1) {
                Ok(seq) => {
                    result_symbol = seq.concat(result_symbol);
                }
                Err(err) => {
                    debug!("{}", err);
                    bail!(err)
                }
            }
            if let Some(val) = self.decode_symbol(result_symbol) {
                return Ok(val);
            }
        }
    }

    pub fn from_lengths(code_lengths: &[u8]) -> Result<Self> {
        let mut map: HashMap<BitSequence, T> = HashMap::new();
        let mut bl_count: [usize; 256] = [0; 256];
        let mut next_code: [u16; MAX_BITS + 1] = [0; MAX_BITS + 1];
        for &len in code_lengths {
            if len > 0 {
                bl_count[len as usize] += 1;
            }
        }
        let mut code = 0;
        for bits in 1..(MAX_BITS + 1) {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code as u16;
        }

        for (ind, &code_len) in code_lengths.iter().enumerate() {
            if code_len != 0 {
                let value = T::try_from(HuffmanCodeWord(ind as u16));
                map.insert(
                    BitSequence::new(next_code[code_len as usize], code_len),
                    value.unwrap(),
                );
                next_code[code_len as usize] += 1;
            }
        }
        Ok(Self { map })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Value(u16);

    impl TryFrom<HuffmanCodeWord> for Value {
        type Error = anyhow::Error;

        fn try_from(x: HuffmanCodeWord) -> Result<Self> {
            Ok(Self(x.0))
        }
    }

    #[test]
    fn from_lengths() -> Result<()> {
        let code = HuffmanCoding::<Value>::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;

        assert_eq!(
            code.decode_symbol(BitSequence::new(0b00, 2)),
            Some(Value(0)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b100, 3)),
            Some(Value(1)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1110, 4)),
            Some(Value(2)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b101, 3)),
            Some(Value(3)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b110, 3)),
            Some(Value(4)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1111, 4)),
            Some(Value(5)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b01, 2)),
            Some(Value(6)),
        );

        assert_eq!(code.decode_symbol(BitSequence::new(0b0, 1)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b10, 2)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b111, 3)), None,);

        Ok(())
    }

    #[test]
    fn read_symbol() -> Result<()> {
        let code = HuffmanCoding::<Value>::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;
        let mut data: &[u8] = &[0b10111001, 0b11001010, 0b11101101];
        let mut reader = BitReader::new(&mut data);

        assert_eq!(code.read_symbol(&mut reader)?, Value(1));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(3));
        assert_eq!(code.read_symbol(&mut reader)?, Value(6));
        assert_eq!(code.read_symbol(&mut reader)?, Value(0));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(4));
        assert!(code.read_symbol(&mut reader).is_err());

        Ok(())
    }
}
