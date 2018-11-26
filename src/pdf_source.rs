use std::io::Error;
use std::io::{Read, Seek, SeekFrom};
use std::result::Result;

pub trait Source {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error>;
    fn getch(&mut self) -> Option<char>;
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
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        self.seek(pos)
    }

    fn getch(&mut self) -> Option<char> {
        readch(&mut self.source)
    }

    fn backup(&mut self) {
        let _ = self.seek(SeekFrom::Current(-1));
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
    T: Read + Seek,
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
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        panic!("internal error - cannot seek on StreamSource");
    }

    fn getch(&mut self) -> Option<char> {
        if let Some(ch) = self.next {
            self.next = None;
            self.last = Some(ch);
            Some(ch)
        } else {
            if let Some(ch) = readch(&mut self.source) {
                self.last = Some(ch);
                Some(ch)
            } else {
                None
            }
        }
    }

    fn backup(&mut self) {
        if let Some(ch) = self.last {
            self.next = Some(ch);
        }
    }
}

fn readch(source: &mut Read) -> Option<char> {
    let mut buffer = [0];
    match source.read(&mut buffer) {
        Ok(n) if n == 1 => Some(buffer[0] as char),
        _ => None,
    }
}
