    #![forbid(unsafe_code)]

use std::io::{self, BufRead};

use log::debug;
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitSequence {
    bits: u16,
    len: u8,
}

impl BitSequence {
    pub fn new(bits: u16, len: u8) -> Self {
        // NB: make sure to zero unused bits so that Eq and Hash work as expected.
        let mut right_data = bits;
        match len {
            1..=15 => {
                right_data &= (1 << len) - 1;
            }
            17.. => {
                panic!("We can read at most 16 bits at time");
            }
            _ => (),
        }
        Self {
            bits: right_data,
            len,
        }
    }

    pub fn bits(&self) -> u16 {
        self.bits
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn concat(self, other: Self) -> Self {
        if self.len + other.len > 16 {
            panic!("We can read at most 16 bits at time");
        }
        Self {
            bits: self.bits | (other.bits << self.len),
            len: self.len + other.len,
        }
    }

    pub fn shrink(&mut self, len: u8) -> Self {
        if len > self.len {
            panic!("shrink big len");
        }
        let res = BitSequence::new(self.bits & ((1 << len) - 1), len);
        self.bits >>= len;
        self.len -= len;
        res
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct BitReader<T> {
    stream: T,
    buf: BitSequence,
}

impl<T: BufRead> BitReader<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            buf: BitSequence::new(0, 0),
        }
    }

    pub fn read_bits(&mut self, len: u8) -> io::Result<BitSequence> {
        if len > 16 {
            panic!("We can read at most 16 bits at time");
        }
        if self.buf.len() >= len {
            return Ok(self.buf.shrink(len));
        }
        if self.buf.len() + 8 < len {
            let mut read_bytes: [u8; 2] = [0, 0];
            match self.stream.read_exact(&mut read_bytes) {
                Ok(()) => {
                    let value = (read_bytes[0] as u16) + (256_u16) * (read_bytes[1] as u16);
                    let tail = BitSequence::new(value, len - self.buf.len());
                    let mut new_buf = BitSequence::new(
                        value >> (len - self.buf.len()),
                        16 - (len - self.buf.len()),
                    );
                    std::mem::swap(&mut new_buf, &mut self.buf);
                    Ok(new_buf.concat(tail))
                }
                Err(err) => Err(err),
            }
        } else {
            let mut read_byte: [u8; 1] = [0];
            match self.stream.read_exact(&mut read_byte) {
                Ok(()) => {
                    let tail = BitSequence::new(read_byte[0] as u16, len - self.buf.len());
                    let mut new_buf = BitSequence::new(
                        read_byte[0] as u16 >> (len - self.buf.len()),
                        8 - (len - self.buf.len()),
                    );
                    std::mem::swap(&mut new_buf, &mut self.buf);
                    Ok(new_buf.concat(tail))
                }
                Err(err) => Err(err),
            }
        }
    }

    /// Discard all the unread bits in the current byte and return a mutable reference
    /// to the underlying reader.
    pub fn borrow_reader_from_boundary(&mut self) -> &mut T {
        debug!("skipped {} bits", { self.buf.len });
        self.buf.len = 0;
        &mut self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;

    #[test]
    fn read_bits() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(1)?, BitSequence::new(0b1, 1));
        assert_eq!(reader.read_bits(2)?, BitSequence::new(0b01, 2));
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b100, 3));
        assert_eq!(reader.read_bits(4)?, BitSequence::new(0b1101, 4));
        assert_eq!(reader.read_bits(5)?, BitSequence::new(0b10110, 5));
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b01011111, 8));
        assert_eq!(
            reader.read_bits(2).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        Ok(())
    }

    #[test]
    fn borrow_reader_from_boundary() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b011, 3));
        assert_eq!(reader.borrow_reader_from_boundary().read_u8()?, 0b11011011);
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b10101111, 8));
        Ok(())
    }
}
