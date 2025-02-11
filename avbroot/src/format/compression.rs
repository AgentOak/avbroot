/*
 * SPDX-FileCopyrightText: 2023 Andrew Gunnerson
 * SPDX-License-Identifier: GPL-3.0-only
 */

use std::io::{self, Read, Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use lz4_flex::frame::FrameDecoder;
use thiserror::Error;

static GZIP_MAGIC: &[u8; 2] = b"\x1f\x8b";
static LZ4_LEGACY_MAGIC: &[u8; 4] = b"\x02\x21\x4c\x18";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown compression format")]
    UnknownFormat,
    #[error("I/O error")]
    IoError(#[from] io::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Lz4LegacyEncoder<W: Write> {
    writer: Option<W>,
    buf: Vec<u8>,
    n_filled: usize,
}

impl<W: Write> Lz4LegacyEncoder<W> {
    pub fn new(mut writer: W) -> io::Result<Self> {
        writer.write_all(LZ4_LEGACY_MAGIC)?;

        Ok(Self {
            writer: Some(writer),
            // We always use the max block size.
            buf: vec![0u8; 8 * 1024 * 1024],
            n_filled: 0,
        })
    }

    pub fn write_block(&mut self, force: bool) -> io::Result<()> {
        if !force && self.n_filled < self.buf.len() {
            // Block not fully filled yet.
            return Ok(());
        }

        // HC is currently not supported:
        // https://github.com/PSeitz/lz4_flex/issues/21
        let compressed = lz4_flex::block::compress(&self.buf[..self.n_filled]);

        let writer = self.writer.as_mut().unwrap();
        writer.write_u32::<LittleEndian>(compressed.len() as u32)?;
        writer.write_all(&compressed)?;

        self.n_filled = 0;

        Ok(())
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.write_block(true)?;
        Ok(self.writer.take().unwrap())
    }
}

impl<W: Write> Drop for Lz4LegacyEncoder<W> {
    fn drop(&mut self) {
        if self.writer.is_some() {
            let _ = self.write_block(true);
        }
    }
}

impl<W: Write> Write for Lz4LegacyEncoder<W> {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let total = buf.len();

        while !buf.is_empty() {
            let to_write = buf.len().min(self.buf.len() - self.n_filled);
            self.buf[self.n_filled..self.n_filled + to_write].copy_from_slice(&buf[..to_write]);

            self.n_filled += to_write;
            self.write_block(false)?;

            buf = &buf[to_write..];
        }

        Ok(total)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_block(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressedFormat {
    None,
    Gzip,
    Lz4Legacy,
}

pub enum CompressedReader<R: Read> {
    None(R),
    Gzip(GzDecoder<R>),
    Lz4(FrameDecoder<R>),
}

impl<R: Read + Seek> CompressedReader<R> {
    pub fn new(mut reader: R, raw_if_unknown: bool) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        reader.rewind()?;

        if &magic[0..2] == GZIP_MAGIC {
            Ok(Self::Gzip(GzDecoder::new(reader)))
        } else if &magic == LZ4_LEGACY_MAGIC {
            Ok(Self::Lz4(FrameDecoder::new(reader)))
        } else if raw_if_unknown {
            Ok(Self::None(reader))
        } else {
            Err(Error::UnknownFormat)
        }
    }

    pub fn format(&self) -> CompressedFormat {
        match self {
            Self::None(_) => CompressedFormat::None,
            Self::Gzip(_) => CompressedFormat::Gzip,
            Self::Lz4(_) => CompressedFormat::Lz4Legacy,
        }
    }

    pub fn into_inner(self) -> R {
        match self {
            Self::None(r) => r,
            Self::Gzip(r) => r.into_inner(),
            Self::Lz4(r) => r.into_inner(),
        }
    }
}

impl<R: Read> Read for CompressedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::None(r) => r.read(buf),
            Self::Gzip(r) => r.read(buf),
            Self::Lz4(r) => r.read(buf),
        }
    }
}

pub enum CompressedWriter<W: Write> {
    None(W),
    Gzip(GzEncoder<W>),
    Lz4Legacy(Lz4LegacyEncoder<W>),
}

impl<W: Write> CompressedWriter<W> {
    pub fn new(writer: W, format: CompressedFormat) -> Result<Self> {
        match format {
            CompressedFormat::None => Ok(Self::None(writer)),
            CompressedFormat::Gzip => {
                Ok(Self::Gzip(GzEncoder::new(writer, Compression::default())))
            }
            CompressedFormat::Lz4Legacy => Ok(Self::Lz4Legacy(Lz4LegacyEncoder::new(writer)?)),
        }
    }

    pub fn format(&self) -> CompressedFormat {
        match self {
            Self::None(_) => CompressedFormat::None,
            Self::Gzip(_) => CompressedFormat::Gzip,
            Self::Lz4Legacy(_) => CompressedFormat::Lz4Legacy,
        }
    }

    pub fn finish(self) -> io::Result<W> {
        match self {
            Self::None(w) => Ok(w),
            Self::Gzip(w) => w.finish(),
            Self::Lz4Legacy(w) => w.finish(),
        }
    }
}

impl<W: Write> Write for CompressedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::None(w) => w.write(buf),
            Self::Gzip(w) => w.write(buf),
            Self::Lz4Legacy(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::None(w) => w.flush(),
            Self::Gzip(w) => w.flush(),
            Self::Lz4Legacy(w) => w.flush(),
        }
    }
}
