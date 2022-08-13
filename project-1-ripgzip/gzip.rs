#![forbid(unsafe_code)]

use std::io::BufRead;

use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::Crc;
use log::debug;
use std::str;

////////////////////////////////////////////////////////////////////////////////

const ID1: u8 = 0x1f;
const ID2: u8 = 0x8b;

const CM_DEFLATE: u8 = 8;

const FTEXT_OFFSET: u8 = 0;
const FHCRC_OFFSET: u8 = 1;
const FEXTRA_OFFSET: u8 = 2;
const FNAME_OFFSET: u8 = 3;
const FCOMMENT_OFFSET: u8 = 4;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberHeader {
    pub compression_method: CompressionMethod,
    pub modification_time: u32,
    pub extra: Option<Vec<u8>>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub extra_flags: u8,
    pub os: u8,
    pub has_crc: bool,
    pub is_text: bool,
}

impl MemberHeader {
    pub fn crc16(&self) -> u16 {
        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        digest.update(&[ID1, ID2, self.compression_method.into(), self.flags().0]);
        digest.update(&self.modification_time.to_le_bytes());
        digest.update(&[self.extra_flags, self.os]);

        if let Some(extra) = &self.extra {
            digest.update(&(extra.len() as u16).to_le_bytes());
            digest.update(extra);
        }

        if let Some(name) = &self.name {
            digest.update(name.as_bytes());
            digest.update(&[0]);
        }

        if let Some(comment) = &self.comment {
            digest.update(comment.as_bytes());
            digest.update(&[0]);
        }

        (digest.finalize() & 0xffff) as u16
    }

    pub fn flags(&self) -> MemberFlags {
        let mut flags = MemberFlags(0);
        flags.set_is_text(self.is_text);
        flags.set_has_crc(self.has_crc);
        flags.set_has_extra(self.extra.is_some());
        flags.set_has_name(self.name.is_some());
        flags.set_has_comment(self.comment.is_some());
        flags
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum CompressionMethod {
    Deflate,
    Unknown(u8),
}

impl From<u8> for CompressionMethod {
    fn from(value: u8) -> Self {
        match value {
            CM_DEFLATE => Self::Deflate,
            x => Self::Unknown(x),
        }
    }
}

impl From<CompressionMethod> for u8 {
    fn from(method: CompressionMethod) -> u8 {
        match method {
            CompressionMethod::Deflate => CM_DEFLATE,
            CompressionMethod::Unknown(x) => x,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberFlags(u8);

#[allow(unused)]
impl MemberFlags {
    fn bit(&self, n: u8) -> bool {
        (self.0 >> n) & 1 != 0
    }

    fn set_bit(&mut self, n: u8, value: bool) {
        if value {
            self.0 |= 1 << n;
        } else {
            self.0 &= !(1 << n);
        }
    }

    pub fn is_text(&self) -> bool {
        self.bit(FTEXT_OFFSET)
    }

    pub fn set_is_text(&mut self, value: bool) {
        self.set_bit(FTEXT_OFFSET, value)
    }

    pub fn has_crc(&self) -> bool {
        self.bit(FHCRC_OFFSET)
    }

    pub fn set_has_crc(&mut self, value: bool) {
        self.set_bit(FHCRC_OFFSET, value)
    }

    pub fn has_extra(&self) -> bool {
        self.bit(FEXTRA_OFFSET)
    }

    pub fn set_has_extra(&mut self, value: bool) {
        self.set_bit(FEXTRA_OFFSET, value)
    }

    pub fn has_name(&self) -> bool {
        self.bit(FNAME_OFFSET)
    }

    pub fn set_has_name(&mut self, value: bool) {
        self.set_bit(FNAME_OFFSET, value)
    }

    pub fn has_comment(&self) -> bool {
        self.bit(FCOMMENT_OFFSET)
    }

    pub fn set_has_comment(&mut self, value: bool) {
        self.set_bit(FCOMMENT_OFFSET, value)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberFooter {
    pub data_crc32: u32,
    pub data_size: u32,
}

////////////////////////////////////////////////////////////////////////////////

pub struct GzipReader<T> {
    reader: T,
}

impl<T: BufRead> GzipReader<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn get_header(&mut self) -> Option<Result<[u8; 10]>> {
        let mut header = [0_u8; 10];
        debug!("begin reading");
        match self.reader.read(&mut header) {
            Ok(sz) => {
                if sz == 0 {
                    return None;
                } else if sz < 10 {
                    Some(anyhow!("unexpected eof"));
                }
            }
            Err(err) => {
                return Some(Err(anyhow!(err)));
            }
        }
        Some(Ok(header))
    }

    pub fn parse_header(mut self, header: &[u8]) -> Result<(MemberHeader, MemberReader<T>)> {
        if header[0] != ID1 || header[1] != ID2 {
            return Err(anyhow!("wrong id values"));
        }
        if let CompressionMethod::Unknown(_) = CompressionMethod::from(header[2]) {
            return Err(anyhow!("unsupported compression method"));
        }

        let (fl, extra_flags, os) = (MemberFlags(header[3]), header[8], header[9]);
        let modification_time = (&header[4..8]).read_u32::<LittleEndian>().unwrap();

        let mut extra = None;

        if fl.has_extra() {
            let mut len_extra_fields_bytes = [0_u8, 2];
            self.reader.read_exact(&mut len_extra_fields_bytes)?;
            let len_extra_fields = u16::from_le_bytes(len_extra_fields_bytes);
            let mut data_extra = Vec::new();
            data_extra.resize(len_extra_fields as usize, 0_u8);
            self.reader.read_exact(data_extra.as_mut_slice())?;
            extra = Some(data_extra);
        }

        let mut name = None;
        if fl.has_name() {
            let mut data = Vec::new();
            self.reader.read_until(b'\x00', &mut data)?;
            let str_data = str::from_utf8(&data)?;
            name = Some(str_data.to_string());
        }

        let mut comment = None;

        if fl.has_comment() {
            let mut data = Vec::new();
            self.reader.read_until(b'\x00', &mut data)?;
            let str_data = str::from_utf8(&data)?;
            comment = Some(str_data.to_string());
        }

        let has_crc = fl.has_crc();
        let mut crc16 = 0;
        if has_crc {
            let mut crc_bytes = [0_u8, 2];
            self.reader.read_exact(&mut crc_bytes)?;
            crc16 = u16::from_le_bytes(crc_bytes);
        }

        let res = MemberHeader {
            compression_method: CompressionMethod::Deflate,
            modification_time,
            extra,
            name,
            comment,
            extra_flags,
            os,
            has_crc,
            is_text: fl.is_text(),
        };
        if has_crc && crc16 != res.crc16() {
            return Err(anyhow!("header crc16 check failed"));
        }
        Ok((res, MemberReader { inner: self.reader }))
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct MemberReader<T> {
    inner: T,
}

impl<T: BufRead> MemberReader<T> {
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn read_footer(mut self) -> Result<(MemberFooter, GzipReader<T>)> {
        debug!("read footer");
        let mut footer = [0_u8; 8];
        self.inner.read_exact(&mut footer)?;
        let (data_crc32, data_size) = (
            (&footer[0..4]).read_u32::<LittleEndian>()?,
            (&footer[4..8]).read_u32::<LittleEndian>()?,
        );
        Ok((
            MemberFooter {
                data_crc32,
                data_size,
            },
            GzipReader::new(self.inner),
        ))
    }
}
