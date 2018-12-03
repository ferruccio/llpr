use errors::*;
use next_object::next_object;
use pdf_source::{Source, VByteSource};
use pdf_types::*;

type Result<R> = ::std::result::Result<R, PdfError>;

pub struct PageContents {
    source: Box<Source>,
}

impl PageContents {
    pub fn new(mut contents: Vec<u8>) -> PageContents {
        // shove a space at the end so we don't get a premature PdfError::EndOfFile
        contents.push(b' ');
        PageContents {
            source: Box::new(VByteSource::new(contents)),
        }
    }

    pub fn next_object(&mut self) -> Result<Option<PdfObject>> {
        next_object(&mut self.source)
    }
}
