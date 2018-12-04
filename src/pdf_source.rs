use errors::*;
use std::io::{Cursor, Error, Read, Seek, SeekFrom};

type StdResult<R, E> = ::std::result::Result<R, E>;
type Result<R> = StdResult<R, PdfError>;

pub trait Source: Read {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error>;
    fn getch(&mut self) -> Result<Option<char>>;
    fn backup(&mut self);
}

pub struct PdfSource<T>
where
    T: Read + Seek,
{
    source: T,
}

impl<T> PdfSource<T>
where
    T: Read + Seek,
{
    pub fn new(source: T) -> PdfSource<T>
    where
        T: Read + Seek,
    {
        PdfSource { source: source }
    }
}

impl<T> Source for PdfSource<T>
where
    T: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error> {
        self.source.seek(pos)
    }

    fn getch(&mut self) -> Result<Option<char>> {
        readch(&mut self.source)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }
}

impl<T> Read for PdfSource<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.source.read(buf)
    }
}

pub struct ByteSliceSource<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> ByteSliceSource<'a> {
    pub fn new(source: &'a [u8]) -> ByteSliceSource<'a> {
        ByteSliceSource {
            cursor: Cursor::new(source),
        }
    }
}

impl<'a> Source for ByteSliceSource<'a> {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error> {
        self.cursor.seek(pos)
    }

    fn getch(&mut self) -> Result<Option<char>> {
        readch(&mut self.cursor)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }
}

impl<'a> Read for ByteSliceSource<'a> {
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.cursor.read(buf)
    }
}

pub struct ByteSource {
    cursor: Cursor<Vec<u8>>,
}

impl ByteSource {
    pub fn new(bytes: Vec<u8>) -> ByteSource {
        ByteSource {
            cursor: Cursor::new(bytes),
        }
    }
}

impl Source for ByteSource {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error> {
        self.cursor.seek(pos)
    }

    fn getch(&mut self) -> Result<Option<char>> {
        readch(&mut self.cursor)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }
}

impl Read for ByteSource {
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.cursor.read(buf)
    }
}

fn readch(source: &mut Read) -> Result<Option<char>> {
    let mut buffer = [0];
    match source.read(&mut buffer)? {
        0 => Ok(None),
        _ => Ok(Some(buffer[0] as char)),
    }
}
