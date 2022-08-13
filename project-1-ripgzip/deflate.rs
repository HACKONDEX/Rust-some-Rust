#![forbid(unsafe_code)]

use std::io::BufRead;

use crate::bit_reader::BitReader;
use anyhow::{anyhow, Result};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BlockHeader {
    pub is_final: bool,
    pub compression_type: CompressionType,
}

#[derive(Debug)]
pub enum CompressionType {
    Uncompressed = 0,
    FixedTree = 1,
    DynamicTree = 2,
    Reserved = 3,
}

////////////////////////////////////////////////////////////////////////////////

pub struct DeflateReader<T> {
    bit_reader: BitReader<T>,
}

impl<T: BufRead> DeflateReader<T> {
    pub fn new(bit_reader: BitReader<T>) -> Self {
        Self { bit_reader }
    }

    pub fn next_block(&mut self) -> Option<Result<(BlockHeader, &mut BitReader<T>)>> {
        let mut result = BlockHeader {
            is_final: false,
            compression_type: CompressionType::Uncompressed,
        };
        match self.bit_reader.read_bits(1) {
            Ok(seq) => {
                result.is_final = seq.bits() != 0;
            }
            Err(err) => {
                return Some(Err(anyhow!(err)));
            }
        }
        let compression_bits = self.bit_reader.read_bits(2);
        if compression_bits.is_err() {
            return Some(Err(anyhow!(compression_bits.err().unwrap())));
        }
        match compression_bits.unwrap().bits() {
            0 => {
                result.compression_type = CompressionType::Uncompressed;
            }
            1 => {
                result.compression_type = CompressionType::FixedTree;
            }
            2 => {
                result.compression_type = CompressionType::DynamicTree;
            }
            3 => {
                result.compression_type = CompressionType::Reserved;
            }
            _ => {
                return None;
            }
        }
        Some(Ok((result, &mut self.bit_reader)))
    }
}
