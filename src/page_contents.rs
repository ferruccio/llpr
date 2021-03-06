use crate::next_object::next_object;
use crate::pdf_source::{ByteSource, Source};
use crate::pdf_types::*;

pub struct PageContents {
    source: Box<dyn Source>,
}

impl PageContents {
    pub fn new(contents: Vec<u8>) -> PageContents {
        PageContents {
            source: Box::new(ByteSource::new(contents)),
        }
    }

    pub fn next_object(&mut self) -> crate::Result<Option<PdfObject>> {
        next_object(&mut self.source)
    }
}
