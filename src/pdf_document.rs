use std::io::{Read, SeekFrom};

use crate::dictionary::Access;
use crate::next_object::{need_dictionary, need_keyword, need_u32, next_object};
use crate::page_contents::PageContents;
use crate::pdf_source::Source;
use crate::pdf_types::*;
use crate::streams::decode_stream;
use crate::PdfError;

const FREE_GEN: u16 = 0xffff;

#[derive(Debug, PartialEq, Clone)]
pub struct XRefEntry {
    gen: u16,
    position: u64,
}

pub struct PdfDocument {
    source: Box<dyn Source>,
    xref: Vec<XRefEntry>,
    pages: Vec<Dictionary>,
}

impl PdfDocument {
    pub fn new(mut source: Box<dyn Source>) -> crate::Result<PdfDocument> {
        PdfDocument::validate_pdf(&mut source)?;
        let (position, buffer) = PdfDocument::read_tail(&mut source)?;
        let trailer_position = find_trailer(position, &buffer)?;
        let mut document = PdfDocument {
            source: source,
            xref: vec![],
            pages: vec![],
        };
        document.source.seek(SeekFrom::Start(trailer_position))?;
        let (trailer_dict, startxref) = PdfDocument::read_trailer(&mut document.source)?;
        document.source.seek(SeekFrom::Start(startxref))?;
        let size = match trailer_dict.get_u32(PdfName::Size) {
            Some(s) => s,
            _ => return Err(PdfError::InvalidPdf("Size missing in trailer")),
        };
        document.read_xref(size)?;
        let catalog_ref = match trailer_dict.get_reference(PdfName::Root) {
            Some(r) => r,
            _ => return Err(PdfError::InvalidPdf("Root missing from trailer")),
        };
        document.seek_reference(catalog_ref)?;
        let catalog = document.read_dictionary(catalog_ref)?;
        let page_root_ref = match catalog.get_reference(PdfName::Pages) {
            Some(r) => r,
            _ => return Err(PdfError::InvalidPdf("document page tree missing")),
        };
        document.seek_reference(page_root_ref)?;
        let mut page_root = document.read_dictionary(page_root_ref)?;
        document.pages = document.read_pages(&mut page_root)?;
        Ok(document)
    }

    pub fn page_count(&self) -> u32 {
        self.pages.len() as u32
    }

    pub fn page_contents(&mut self, pageno: u32) -> crate::Result<PageContents> {
        if pageno < self.pages.len() as u32 {
            let page_dict = self.pages[pageno as usize].clone();
            Ok(PageContents::new(self.contents(&page_dict)?))
        } else {
            Err(PdfError::InvalidPageNumber)
        }
    }
}

impl PdfDocument {
    fn validate_pdf(source: &mut Box<dyn Source>) -> crate::Result<()> {
        source.seek(SeekFrom::Start(0))?;
        let expected_header = "%PDF-1.";
        let mut buffer = [0; 7];
        source.read(&mut buffer)?;
        if buffer != expected_header.as_bytes() {
            return Err(PdfError::InvalidPdf("bad pdf header"));
        }
        Ok(())
    }

    fn read_tail(source: &mut Box<dyn Source>) -> crate::Result<(u64, Vec<u8>)> {
        const BUFFER_SIZE: usize = 8 * 1024;
        let filesize = source.seek(SeekFrom::End(0))? as usize;
        let position = if filesize < BUFFER_SIZE {
            source.seek(SeekFrom::Start(0))?
        } else {
            source.seek(SeekFrom::End(-(BUFFER_SIZE as i64)))?
        };
        let mut buffer = vec![];
        let _ = source.read_to_end(&mut buffer)?;
        Ok((position, buffer))
    }

    fn read_trailer(source: &mut Box<dyn Source>) -> crate::Result<(Dictionary, u64)> {
        if let Some(PdfObject::Dictionary(trailer_dict)) = next_object(source)? {
            need_keyword(source, PdfKeyword::startxref)?;
            if let Some(PdfObject::Number(PdfNumber::Integer(addr))) = next_object(source)? {
                return Ok((trailer_dict, addr as u64));
            }
        }
        Err(PdfError::InvalidPdf("invalid pdf trailer"))
    }

    fn read_xref(&mut self, size: u32) -> crate::Result<()> {
        self.xref.resize(
            size as usize,
            XRefEntry {
                gen: FREE_GEN,
                position: 0,
            },
        );
        need_keyword(&mut self.source, PdfKeyword::xref)?;
        loop {
            let first = next_object(&mut self.source)?;
            let count = next_object(&mut self.source)?;
            let (first, count) = match (first, count) {
                (
                    Some(PdfObject::Number(PdfNumber::Integer(f))),
                    Some(PdfObject::Number(PdfNumber::Integer(c))),
                ) => (f as usize, c as usize),
                _ => return Ok(()),
            };
            for index in first..first + count {
                let entry = self.read_xref_entry()?;
                if index < self.xref.len() {
                    self.xref[index] = entry;
                }
            }
        }
    }

    fn read_xref_entry(&mut self) -> crate::Result<XRefEntry> {
        let position = next_object(&mut self.source)?;
        let gen = next_object(&mut self.source)?;
        let flag = next_object(&mut self.source)?;
        match (position, gen, flag) {
            (
                Some(PdfObject::Number(PdfNumber::Integer(p))),
                Some(PdfObject::Number(PdfNumber::Integer(g))),
                Some(PdfObject::Keyword(PdfKeyword::n)),
            ) => Ok(XRefEntry {
                gen: g as u16,
                position: p as u64,
            }),
            (
                Some(PdfObject::Number(PdfNumber::Integer(_))),
                Some(PdfObject::Number(PdfNumber::Integer(_))),
                Some(PdfObject::Keyword(PdfKeyword::f)),
            ) => Ok(XRefEntry {
                gen: FREE_GEN,
                position: 0,
            }),

            _ => Err(PdfError::InvalidPdf("invalid xref entry")),
        }
    }

    fn read_pages(&mut self, pages_node: &mut Dictionary) -> crate::Result<Vec<Dictionary>> {
        let mut pages = vec![];
        let kids = match pages_node.get_array(PdfName::Kids) {
            Some(a) => a,
            _ => return Err(PdfError::InvalidPdf("Kids missing from pages node")),
        };
        for kid in kids.iter() {
            match kid {
                PdfObject::Reference(r) => {
                    self.seek_reference(r.clone())?;
                    let mut dict = self.read_dictionary(r.clone())?;
                    match dict.get_name(PdfName::Type) {
                        Some(ref name) if *name == PdfName::Pages => {
                            pages.append(&mut self.read_pages(&mut dict)?);
                        }
                        Some(ref name) if *name == PdfName::Page => {
                            pages.push(dict);
                        }
                        _ => return Err(PdfError::InvalidPdf("invalid page tree entry")),
                    }
                }
                _ => return Err(PdfError::InvalidPdf("invalid Kids entry")),
            }
        }
        Ok(pages)
    }

    fn read_object(&mut self, reference: Reference) -> crate::Result<PdfObject> {
        need_u32(&mut self.source, reference.id)?;
        need_u32(&mut self.source, reference.gen as u32)?;
        need_keyword(&mut self.source, PdfKeyword::obj)?;
        match next_object(&mut self.source)? {
            Some(obj) => {
                need_keyword(&mut self.source, PdfKeyword::endobj)?;
                Ok(obj)
            }
            None => Err(PdfError::InvalidPdf("pdf object expected")),
        }
    }

    fn read_prefix(&mut self, reference: Reference) -> crate::Result<Dictionary> {
        need_u32(&mut self.source, reference.id)?;
        need_u32(&mut self.source, reference.gen as u32)?;
        need_keyword(&mut self.source, PdfKeyword::obj)?;
        let dictionary = need_dictionary(&mut self.source)?;
        Ok(dictionary)
    }

    fn read_dictionary(&mut self, reference: Reference) -> crate::Result<Dictionary> {
        let dictionary = self.read_prefix(reference)?;
        need_keyword(&mut self.source, PdfKeyword::endobj)?;
        Ok(dictionary)
    }

    fn seek_reference(&mut self, reference: Reference) -> crate::Result<u64> {
        let id = reference.id as usize;
        if id >= self.xref.len() || self.xref[id].gen == FREE_GEN {
            Err(PdfError::InvalidReference)
        } else {
            Ok(self.source.seek(SeekFrom::Start(self.xref[id].position))?)
        }
    }

    fn dereference(&mut self, object: PdfObject) -> crate::Result<PdfObject> {
        match object {
            PdfObject::Reference(r) => {
                self.seek_reference(r)?;
                let obj = self.read_object(r)?;
                Ok(self.dereference(obj)?)
            }
            PdfObject::Array(array) => {
                let a: crate::Result<Vec<_>> =
                    array.into_iter().map(|o| self.dereference(o)).collect();
                Ok(PdfObject::Array(Box::new(a?)))
            }
            PdfObject::Dictionary(dict) => {
                let d = dict
                    .into_iter()
                    .map(|(k, v)| match k {
                        PdfName::Parent | PdfName::Contents | PdfName::Resources => (k, v),
                        _ => (k, self.dereference(v).unwrap_or(PdfObject::Null)),
                    })
                    .collect();
                Ok(PdfObject::Dictionary(Box::new(d)))
            }
            obj @ _ => Ok(obj),
        }
    }

    fn dereference_dictionary(&mut self, dictionary: Dictionary) -> crate::Result<Dictionary> {
        match self.dereference(PdfObject::Dictionary(dictionary))? {
            PdfObject::Dictionary(dictionary) => Ok(dictionary),
            _ => Err(PdfError::InternalError(
                "unexpected result from dereference_dictionary",
            )),
        }
    }

    fn read_stream(&mut self, reference: Reference) -> crate::Result<Vec<u8>> {
        self.seek_reference(reference)?;
        let stream_dict = self.read_prefix(reference)?;
        need_keyword(&mut self.source, PdfKeyword::stream)?;
        while match self.source.getch()? {
            None => return Err(PdfError::EndOfFile),
            Some('\n') => false,
            _ => true,
        } {}
        let pos = self.source.seek(SeekFrom::Current(0))?;
        let stream_dict = self.dereference_dictionary(stream_dict)?;
        let _ = self.source.seek(SeekFrom::Start(pos))?;
        let length = match stream_dict.get_u32(PdfName::Length) {
            Some(length) => length as usize,
            None => {
                return Err(PdfError::InvalidPdf(
                    "Length missing from stream dictionary",
                ))
            }
        };
        let mut buffer = vec![];
        buffer.resize(length, 0);
        let nread = self.source.read(&mut buffer)?;
        if nread != length {
            return Err(PdfError::InternalError("failed to read stream"));
        }
        need_keyword(&mut self.source, PdfKeyword::endstream)?;
        decode_stream(buffer, stream_dict)
    }

    fn read_streams(&mut self, _streams: &Array) -> crate::Result<Vec<u8>> {
        Err(PdfError::InternalError("read_streams not implemented"))
    }

    fn contents(&mut self, page_dict: &Dictionary) -> crate::Result<Vec<u8>> {
        match page_dict.get(&PdfName::Contents) {
            Some(PdfObject::Reference(reference)) => self.read_stream(*reference),
            Some(PdfObject::Array(array)) => self.read_streams(array),
            _ => Err(PdfError::InvalidPdf("invalid page contents")),
        }
    }
}

fn find_trailer(position: u64, buffer: &[u8]) -> crate::Result<u64> {
    let trailer = "trailer".as_bytes();
    for i in (0..=buffer.len() - trailer.len()).rev() {
        if &buffer[i..i + trailer.len()] == trailer {
            return Ok(position + i as u64 + trailer.len() as u64);
        }
    }
    Err(PdfError::InvalidPdf("no trailer"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf_source::{ByteSliceSource, PdfSource};
    use std::fs::File;

    fn open_test_file(name: &str) -> Box<PdfSource<File>> {
        let src = env!("CARGO_MANIFEST_DIR");
        let fullname = format!("{}/testing/{}", src, name);
        Box::new(PdfSource::new(File::open(&fullname).unwrap()))
    }

    #[test]
    fn bad_header() {
        let pdf = Box::new(ByteSliceSource::new(b"%PDx-1.3\n% bad pdf header\n"));
        assert!(PdfDocument::new(pdf).is_err());
    }

    #[test]
    fn find_trailer_middle() {
        let buffer = "blah blah blah trailer blah blah blah".as_bytes();
        let position = find_trailer(0, &buffer);
        assert!(position.is_ok());
        assert_eq!(position.unwrap(), 22);
    }

    #[test]
    fn find_trailer_middle_offset() {
        let buffer = "blah blah blah trailer blah blah blah".as_bytes();
        let position = find_trailer(1000, &buffer);
        assert!(position.is_ok());
        assert_eq!(position.unwrap(), 1000 + 22);
    }

    #[test]
    fn find_trailer_end() {
        let buffer = "blah blah blah trailer".as_bytes();
        let position = find_trailer(0, &buffer);
        assert!(position.is_ok());
        assert_eq!(position.unwrap(), 22);
    }

    #[test]
    fn find_trailer_start() {
        let buffer = "trailer blah blah blah".as_bytes();
        let position = find_trailer(0, &buffer);
        assert!(position.is_ok());
        assert_eq!(position.unwrap(), 7);
    }

    #[test]
    fn no_trailer() {
        let buffer = "railer blah blah blah".as_bytes();
        let position = find_trailer(0, &buffer);
        assert!(position.is_err());
    }

    #[test]
    fn minimal_pdf_xref() {
        let pdf = PdfDocument::new(open_test_file("minimal.pdf")).unwrap();
        assert_eq!(pdf.xref.len(), 5);
        assert_eq!(pdf.page_count(), 1);
        assert_eq!(
            pdf.xref[0],
            XRefEntry {
                gen: FREE_GEN,
                position: 0
            }
        );
        assert_eq!(
            pdf.xref[1],
            XRefEntry {
                gen: 0,
                position: 18
            }
        );
        assert_eq!(
            pdf.xref[2],
            XRefEntry {
                gen: 0,
                position: 77
            }
        );
        assert_eq!(
            pdf.xref[3],
            XRefEntry {
                gen: 0,
                position: 178
            }
        );
        assert_eq!(
            pdf.xref[4],
            XRefEntry {
                gen: 0,
                position: 457
            }
        );
    }

    #[test]
    fn tracemonkey_pdf_xref() {
        let pdf = PdfDocument::new(open_test_file("tracemonkey.pdf")).unwrap();
        assert_eq!(pdf.xref.len(), 997);
        assert_eq!(pdf.page_count(), 14);
    }

    #[test]
    fn minimal_pdf_stream() {
        let mut pdf = PdfDocument::new(open_test_file("minimal.pdf")).unwrap();
        let buffer = pdf.read_stream(Reference::new(4, 0)).unwrap();
        assert_eq!(
            buffer,
            vec![
                32, 32, 66, 84, 10, 32, 32, 32, 32, 47, 70, 49, 32, 49, 56, 32, 84, 102, 10, 32,
                32, 32, 32, 48, 32, 48, 32, 84, 100, 10, 32, 32, 32, 32, 40, 72, 101, 108, 108,
                111, 32, 87, 111, 114, 108, 100, 41, 32, 84, 106, 10, 32, 32, 69, 84
            ]
        );
    }

    #[test]
    fn tracemonkey_pdf_stream() {
        let mut pdf = PdfDocument::new(open_test_file("tracemonkey.pdf")).unwrap();
        let contents_refs: Vec<_> = pdf
            .pages
            .iter()
            .map(|p| p.get_reference(PdfName::Contents).unwrap())
            .collect();
        let src = env!("CARGO_MANIFEST_DIR");
        for reference in contents_refs.into_iter() {
            let stream_buffer = pdf.read_stream(reference).unwrap();
            let filename = format!(
                "{}/testing/tracemonkey-streams/{}-{}.txt",
                src, reference.id, reference.gen
            );
            let mut file = ::std::fs::File::open(&filename).unwrap();
            let mut file_buffer = vec![];
            let nbytes = file.read_to_end(&mut file_buffer).unwrap();
            assert_eq!(stream_buffer, &file_buffer[0..nbytes]);
        }
    }

    #[test]
    fn minimal_pdf_contents_iter() {
        let mut pdf = PdfDocument::new(open_test_file("minimal.pdf")).unwrap();
        let mut pc = pdf.page_contents(0).unwrap();
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Keyword(PdfKeyword::BT)
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Symbol(b"F1".to_vec())
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Number(PdfNumber::Integer(18))
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Keyword(PdfKeyword::Tf)
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Number(PdfNumber::Integer(0))
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Number(PdfNumber::Integer(0))
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Keyword(PdfKeyword::Td)
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::String(b"Hello World".to_vec())
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Keyword(PdfKeyword::Tj)
        );
        assert_eq!(
            pc.next_object().unwrap().unwrap(),
            PdfObject::Keyword(PdfKeyword::ET)
        );
        assert!(pc.next_object().unwrap() == None);
    }
}
