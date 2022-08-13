#![forbid(unsafe_code)]
use std::collections::VecDeque;
use std::io::{self, Write};

use anyhow::{bail, Result};
use crc::{Crc, Digest, CRC_32_ISO_HDLC};

////////////////////////////////////////////////////////////////////////////////

const HISTORY_SIZE: usize = 32768;
static CRC_CHECKER: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
pub struct TrackingWriter<'a, T> {
    inner: T,
    len: usize,
    hist: VecDeque<u8>,
    digest: Option<Digest<'a, u32>>,
}

impl<T: Write> Write for TrackingWriter<'_, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write(buf) {
            Ok(written) => {
                self.digest.as_mut().unwrap().update(&buf[..written]);
                self.len += written;
                if self.hist.len() + std::cmp::min(written, HISTORY_SIZE) > HISTORY_SIZE {
                    self.hist.drain(
                        ..std::cmp::min(
                            self.hist.len(),
                            self.hist.len() + std::cmp::min(written, HISTORY_SIZE) - HISTORY_SIZE,
                        ),
                    );
                }
                let start = match written > HISTORY_SIZE {
                    true => written - HISTORY_SIZE,
                    _ => 0,
                };
                self.hist.extend(buf[start..].iter());
                Ok(written)
            }
            Err(err) => Err(err),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.len = 0;
        self.hist.clear();
        self.digest = Some(CRC_CHECKER.digest());
        self.inner.flush()
    }
}

impl<T: Write> TrackingWriter<'_, T> {
    pub fn new(inner: T) -> Self {
        let mut hist = VecDeque::new();
        hist.reserve(HISTORY_SIZE);
        Self {
            inner,
            len: 0,
            hist,
            digest: Some(CRC_CHECKER.digest()),
        }
    }

    /// Write a sequence of `len` bytes written `dist` bytes ago.
    pub fn write_previous(&mut self, dist: usize, len: usize) -> Result<()> {
        if self.len < dist || dist > HISTORY_SIZE {
            bail!("distance is bigger than length of written")
        }

        let begin = std::cmp::min(HISTORY_SIZE, self.len) - dist;
        self.hist.make_contiguous();
        let data = self.hist.as_slices().0[begin..begin + dist].to_vec();
        let mut bytes_written = 0;
        while bytes_written < len {
            match self.write(&data.as_slice()[0..std::cmp::min(dist, len - bytes_written)]) {
                Ok(written) => {
                    if written < std::cmp::min(dist, len - bytes_written) {
                        bail!("bad writing!")
                    } else {
                        bytes_written += written;
                    }
                }
                Err(err) => panic!("{}", err),
            }
        }
        Ok(())
    }

    pub fn byte_count(&self) -> usize {
        self.len
    }

    pub fn crc32(&mut self) -> u32 {
        let dig = self.digest.take().unwrap();
        dig.finalize()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;

    #[test]
    fn write() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 10];
        let mut writer = TrackingWriter::new(&mut buf);

        assert_eq!(writer.write(&[1, 2, 3, 4])?, 4);
        assert_eq!(writer.byte_count(), 4);

        assert_eq!(writer.write(&[4, 8, 15, 16, 23])?, 5);
        assert_eq!(writer.byte_count(), 9);

        assert_eq!(writer.write(&[0, 0, 123])?, 1);
        assert_eq!(writer.byte_count(), 10);

        assert_eq!(writer.write(&[42, 124, 234, 27])?, 0);
        assert_eq!(writer.byte_count(), 10);
        assert_eq!(writer.crc32(), 2992191065);

        Ok(())
    }

    #[test]
    fn write_previous() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 512];
        let mut writer = TrackingWriter::new(&mut buf);

        for i in 0..=255 {
            writer.write_u8(i)?;
        }

        writer.write_previous(192, 128)?;
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(10000, 20).is_err());
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(256, 256).is_err());
        assert_eq!(writer.byte_count(), 512);

        assert!(writer.write_previous(1, 1).is_err());
        assert_eq!(writer.byte_count(), 512);
        assert_eq!(writer.crc32(), 2733545866);

        Ok(())
    }
}
