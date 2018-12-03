use errors::*;
use std::io::{Cursor, Error, Read, Seek, SeekFrom};

type StdResult<R, E> = ::std::result::Result<R, E>;
type Result<R> = StdResult<R, PdfError>;

pub trait Source: Read {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error>;
    fn getch(&mut self) -> Result<char>;
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

    fn getch(&mut self) -> Result<char> {
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

pub struct StreamSource<T>
where
    T: Read,
{
    source: T,
    last: Option<char>,
    next: Option<char>,
}

impl<T> StreamSource<T>
where
    T: Read,
{
    pub fn new(source: T) -> StreamSource<T> {
        StreamSource {
            source: source,
            last: None,
            next: None,
        }
    }
}

impl<T> Source for StreamSource<T>
where
    T: Read,
{
    fn seek(&mut self, _pos: SeekFrom) -> StdResult<u64, Error> {
        panic!("internal error - cannot seek on StreamSource");
    }

    fn getch(&mut self) -> Result<char> {
        if let Some(ch) = self.next {
            self.next = None;
            self.last = Some(ch);
            Ok(ch)
        } else {
            let ch = readch(&mut self.source)?;
            self.last = Some(ch);
            Ok(ch)
        }
    }

    fn backup(&mut self) {
        if let Some(ch) = self.last {
            self.next = Some(ch);
        }
    }
}

impl<T> Read for StreamSource<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.source.read(buf)
    }
}

pub struct ByteSource<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> ByteSource<'a> {
    pub fn new(source: &'a [u8]) -> ByteSource<'a> {
        ByteSource {
            cursor: Cursor::new(source),
        }
    }
}

impl<'a> Source for ByteSource<'a> {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error> {
        self.cursor.seek(pos)
    }

    fn getch(&mut self) -> Result<char> {
        readch(&mut self.cursor)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }
}

impl<'a> Read for ByteSource<'a> {
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.cursor.read(buf)
    }
}

pub struct VByteSource {
    cursor: Cursor<Vec<u8>>,
}

impl VByteSource {
    pub fn new(bytes: Vec<u8>) -> VByteSource {
        VByteSource {
            cursor: Cursor::new(bytes),
        }
    }
}

impl Source for VByteSource {
    fn seek(&mut self, pos: SeekFrom) -> StdResult<u64, Error> {
        self.cursor.seek(pos)
    }

    fn getch(&mut self) -> Result<char> {
        readch(&mut self.cursor)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
    }
}

impl Read for VByteSource {
    fn read(&mut self, buf: &mut [u8]) -> StdResult<usize, Error> {
        self.cursor.read(buf)
    }
}

fn readch(source: &mut Read) -> Result<char> {
    let mut buffer = [0];
    match source.read(&mut buffer)? {
        0 => Err(PdfError::EndOfFile),
        _ => Ok(buffer[0] as char),
    }
}
