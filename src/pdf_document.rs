use catalog::Catalog;
use dictionary::{Dictionary, GetFrom};
use errors::*;
use object_reader::{ObjectReader, PdfNumber, PdfObject, Reference};
use pages::{Page, Pages};
use std::io::{Read, Seek, SeekFrom};
use token_reader::{PdfKeyword, PdfName, TokenReader};
use trailer::Trailer;

type Result<T> = ::std::result::Result<T, PdfError>;

const FREE_GEN: u16 = 0xffff;

#[derive(Debug, PartialEq, Clone)]
pub struct XRefEntry {
    gen: u16,
    position: u64,
}

pub struct PdfDocument<S>
where
    S: Read + Seek,
{
    reader: ObjectReader<S>,
    xref: Vec<XRefEntry>,
    pages: Vec<Page>,
}

impl<S> PdfDocument<S>
where
    S: Read + Seek,
{
    pub fn new(mut source: S) -> Result<PdfDocument<S>>
    where
        S: Read + Seek,
    {
        PdfDocument::validate_pdf(&mut source)?;
        let (position, buffer) = PdfDocument::read_tail(&mut source)?;
        let trailer_position = find_trailer(position, &buffer)?;
        let mut document = PdfDocument {
            reader: ObjectReader::new(TokenReader::new(source)),
            xref: vec![],
            pages: vec![],
        };
        document.reader.seek(SeekFrom::Start(trailer_position))?;
        let (trailer_dict, startxref) = PdfDocument::read_trailer(&mut document.reader)?;
        let trailer = Trailer::new(trailer_dict)?;
        document.reader.seek(SeekFrom::Start(startxref))?;
        document.read_xref(trailer.size)?;
        document.seek_reference(trailer.root)?;
        let catalog = Catalog::new(document.read_dictionary(trailer.root)?)?;
        document.seek_reference(catalog.pages)?;
        let page_root = Pages::new(document.read_dictionary(catalog.pages)?)?;
        document.pages = document.read_pages(&page_root)?;
        Ok(document)
    }

    pub fn page_count(&self) -> u32 {
        self.pages.len() as u32
    }
}

impl<S> PdfDocument<S>
where
    S: Read + Seek,
{
    fn validate_pdf(source: &mut S) -> Result<()> {
        source.seek(SeekFrom::Start(0))?;
        let expected_header = "%PDF-1.";
        let mut buffer = [0; 7];
        source.read(&mut buffer)?;
        if buffer != expected_header.as_bytes() {
            return Err(PdfError::InvalidPdf("bad pdf header"));
        }
        Ok(())
    }

    fn read_tail(source: &mut S) -> Result<(u64, Vec<u8>)> {
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

    fn read_trailer(reader: &mut ObjectReader<S>) -> Result<(Dictionary, u64)> {
        if let Some(PdfObject::Dictionary(trailer_dict)) = reader.next()? {
            reader.need_keyword(PdfKeyword::startxref)?;
            if let Some(PdfObject::Number(PdfNumber::Integer(addr))) = reader.next()? {
                return Ok((trailer_dict, addr as u64));
            }
        }
        Err(PdfError::InvalidPdf("invalid pdf trailer"))
    }

    fn read_xref(&mut self, size: u32) -> Result<()> {
        self.xref.resize(
            size as usize,
            XRefEntry {
                gen: FREE_GEN,
                position: 0,
            },
        );
        self.reader.need_keyword(PdfKeyword::xref)?;
        loop {
            let first = self.reader.next()?;
            let count = self.reader.next()?;
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

    fn read_xref_entry(&mut self) -> Result<XRefEntry> {
        let position = self.reader.next()?;
        let gen = self.reader.next()?;
        let flag = self.reader.next()?;
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

    fn read_pages(&mut self, page_root: &Pages) -> Result<(Vec<Page>)> {
        let mut pages = vec![];
        for kid in page_root.kids.iter() {
            match kid {
                PdfObject::Reference(r) => {
                    self.seek_reference(r.clone())?;
                    let dict = self.read_dictionary(r.clone())?;
                    match dict.get_name(PdfName::Type) {
                        Some(ref name) if *name == PdfName::Pages => {
                            pages.append(&mut self.read_pages(&Pages::new(dict)?)?);
                        }
                        Some(ref name) if *name == PdfName::Page => {
                            pages.push(Page::new(dict)?);
                        }
                        _ => return Err(PdfError::InvalidPdf("invalid page tree entry")),
                    }
                }
                _ => return Err(PdfError::InvalidPdf("invalid Kids entry")),
            }
        }
        Ok(pages)
    }

    fn read_dictionary(&mut self, reference: Reference) -> Result<Dictionary> {
        self.reader.need_u32(reference.id)?;
        self.reader.need_u32(reference.gen as u32)?;
        self.reader.need_keyword(PdfKeyword::obj)?;
        let dictionary = self.reader.need_dictionary()?;
        self.reader.need_keyword(PdfKeyword::endobj)?;
        Ok(dictionary)
    }

    fn seek_reference(&mut self, reference: Reference) -> Result<u64> {
        let id = reference.id as usize;
        if id >= self.xref.len() || self.xref[id].gen == FREE_GEN {
            Err(PdfError::InvalidReference)
        } else {
            Ok(self.reader.seek(SeekFrom::Start(self.xref[id].position))?)
        }
    }
}

fn find_trailer(position: u64, buffer: &[u8]) -> Result<u64> {
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
    use std::fs::File;
    use std::io::{Cursor, Read, Seek};

    fn open_test_file(name: &str) -> File {
        let src = env!("CARGO_MANIFEST_DIR");
        let fullname = format!("{}/testing/{}", src, name);
        File::open(&fullname).unwrap()
    }

    fn pdf_str(source: &'static str) -> impl Read + Seek {
        Cursor::new(source.as_bytes())
    }

    #[test]
    fn open_minimal_pdf() {
        let pdf = open_test_file("minimal.pdf");
        assert!(PdfDocument::new(pdf).is_ok());
    }

    #[test]
    fn bad_header() {
        let pdf = pdf_str("%PDx-1.3\n% bad pdf header\n");
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
}