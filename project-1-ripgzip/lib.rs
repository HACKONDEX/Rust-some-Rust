#![forbid(unsafe_code)]

use std::io::{BufRead, Write};

use crate::gzip::GzipReader;
use anyhow::{bail, Result};
use bit_reader::BitReader;
use byteorder::{LittleEndian, ReadBytesExt};
use deflate::DeflateReader;
use huffman_coding::decode_litlen_distance_trees;
use log::debug;
use tracking_writer::TrackingWriter;

mod bit_reader;
mod deflate;
mod gzip;
mod huffman_coding;
mod tracking_writer;

pub fn decompress<R: BufRead, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut writer = TrackingWriter::new(&mut output);
    let mut main_reader = GzipReader::new(input);
    debug!("we are here!");
    loop {
        writer.flush()?;
        let header_inf = main_reader.get_header();
        if header_inf.is_none() {
            break;
        }
        let header = header_inf.unwrap()?;
        debug!("read header");
        match main_reader.parse_header(&header) {
            Ok((_, mut member_reader)) => {
                let before_len = writer.byte_count();
                let mut deflate_reader =
                    DeflateReader::new(BitReader::new(member_reader.inner_mut()));
                while let Some(block) = deflate_reader.next_block() {
                    if let Err(err) = block {
                        bail!(err)
                    }
                    let (block_header, reader) = block?;
                    match block_header.compression_type {
                        deflate::CompressionType::Uncompressed => {
                            debug!("uncompressed");
                            let reader = reader.borrow_reader_from_boundary();
                            let (len, nlen) = (
                                reader.read_u16::<LittleEndian>()?,
                                reader.read_u16::<LittleEndian>()?,
                            );
                            match len != !nlen {
                                true => {
                                    bail!("nlen check failed")
                                }
                                _ => {
                                    let mut data = Vec::new();
                                    data.resize(len as usize, 0_u8);
                                    reader.read_exact(data.as_mut_slice())?;
                                    writer.write_all(data.as_slice())?;
                                }
                            }
                        }
                        deflate::CompressionType::DynamicTree => {
                            debug!("dynamic");
                            let (lit_len, distance) = decode_litlen_distance_trees(reader)?;
                            loop {
                                match lit_len.read_symbol(reader)? {
                                    huffman_coding::LitLenToken::Literal(val) => {
                                        writer.write_all(&[val])?;
                                    }
                                    huffman_coding::LitLenToken::EndOfBlock => {
                                        break;
                                    }
                                    huffman_coding::LitLenToken::Length { base, extra_bits } => {
                                        //  debug!("read len");
                                        let len = base + reader.read_bits(extra_bits)?.bits();
                                        //  debug!("read dist token");
                                        let dist_token = distance.read_symbol(reader)?;
                                        //debug!("hmm");
                                        let dist = dist_token.base
                                            + reader.read_bits(dist_token.extra_bits)?.bits();
                                        // debug!("before write");
                                        writer.write_previous(dist as usize, len as usize)?;
                                    }
                                }
                            }
                        }
                        _ => {
                            bail!("unsupported block type")
                        }
                    }
                    if block_header.is_final {
                        break;
                    }
                }
                let (footer, reader) = member_reader.read_footer()?;
                if writer.byte_count() - before_len != footer.data_size as usize {
                    bail!("length check failed");
                }

                if footer.data_crc32 != writer.crc32() {
                    bail!("crc32 check failed");
                }
                main_reader = reader;
            }
            Err(err) => bail!(err),
        }
    }
    Ok(())
}
