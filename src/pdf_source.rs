use catalog::Catalog;
use dictionary::Dictionary;
use errors::*;
use object_reader::{ObjectReader, PdfNumber, PdfObject, Reference};
use std::io::{Read, Seek, SeekFrom};
use token_reader::{PdfKeyword, TokenReader};
use trailer::Trailer;

type Result<T> = ::std::result::Result<T, PdfError>;

const FREE_GEN: u16 = 0xffff;

#[derive(Debug, PartialEq, Clone)]
pub struct XRefEntry {
    gen: u16,
    position: u64,
}

pub struct PdfSource<S>
where
    S: Read + Seek,
{
    reader: ObjectReader<S>,
    trailer: Trailer,
    xref: Vec<XRefEntry>,
    catalog: Catalog,
}

impl<S> PdfSource<S>
where
    S: Read + Seek,
{
    pub fn new(mut source: S) -> Result<PdfSource<S>>
    where
        S: Read + Seek,
    {
        validate_pdf(&mut source)?;
        let (position, buffer) = read_tail(&mut source)?;
        let trailer_position = find_trailer(position, &buffer)?;
        let mut reader = ObjectReader::new(TokenReader::new(source));
        reader.seek(SeekFrom::Start(trailer_position))?;
        let (trailer_dict, startxref) = read_trailer(&mut reader)?;
        let trailer = Trailer::new(trailer_dict)?;
        reader.seek(SeekFrom::Start(startxref))?;
        let xref = read_xref(&mut reader, trailer.size)?;
        reader.seek(SeekFrom::Start(object_position(&xref, trailer.root.id)?))?;
        let catalog_dict = read_object(&mut reader, trailer.root)?;
        let catalog = Catalog::new(catalog_dict)?;
        Ok(PdfSource {
            reader: reader,
            trailer: trailer,
            xref: xref,
            catalog: catalog,
        })
    }
}

fn validate_pdf<S>(source: &mut S) -> Result<()>
where
    S: Read + Seek,
{
    source.seek(SeekFrom::Start(0))?;
    let expected_header = "%PDF-1.";
    let mut buffer = [0; 7];
    source.read(&mut buffer)?;
    if buffer != expected_header.as_bytes() {
        return Err(PdfError::InvalidPdf("bad pdf header"));
    }
    Ok(())
}

fn read_tail<S>(source: &mut S) -> Result<(u64, Vec<u8>)>
where
    S: Read + Seek,
{
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

fn find_trailer(position: u64, buffer: &[u8]) -> Result<u64> {
    let trailer = "trailer".as_bytes();
    for i in (0..=buffer.len() - trailer.len()).rev() {
        if &buffer[i..i + trailer.len()] == trailer {
            return Ok(position + i as u64 + trailer.len() as u64);
        }
    }
    Err(PdfError::InvalidPdf("no trailer"))
}

fn read_trailer<S>(reader: &mut ObjectReader<S>) -> Result<(Dictionary, u64)>
where
    S: Read + Seek,
{
    if let Some(PdfObject::Dictionary(trailer_dict)) = reader.next()? {
        reader.need_keyword(PdfKeyword::startxref)?;
        if let Some(PdfObject::Number(PdfNumber::Integer(addr))) = reader.next()? {
            return Ok((trailer_dict, addr as u64));
        }
    }
    Err(PdfError::InvalidPdf("invalid pdf trailer"))
}

fn read_xref<S>(reader: &mut ObjectReader<S>, size: u32) -> Result<Vec<XRefEntry>>
where
    S: Read + Seek,
{
    let mut xref = vec![];
    xref.resize(
        size as usize,
        XRefEntry {
            gen: FREE_GEN,
            position: 0,
        },
    );
    reader.need_keyword(PdfKeyword::xref)?;
    loop {
        let first = reader.next()?;
        let count = reader.next()?;
        let (first, count) = match (first, count) {
            (
                Some(PdfObject::Number(PdfNumber::Integer(f))),
                Some(PdfObject::Number(PdfNumber::Integer(c))),
            ) => (f as usize, c as usize),
            _ => return Ok(xref),
        };
        for index in first..first + count {
            let entry = read_xref_entry(reader)?;
            if index < xref.len() {
                xref[index] = entry;
            }
        }
    }
}

fn read_xref_entry<S>(reader: &mut ObjectReader<S>) -> Result<XRefEntry>
where
    S: Read + Seek,
{
    let position = reader.next()?;
    let gen = reader.next()?;
    let flag = reader.next()?;
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

fn object_position(xref: &Vec<XRefEntry>, id: u32) -> Result<u64> {
    let id = id as usize;
    if id >= xref.len() || xref[id].gen == FREE_GEN {
        Err(PdfError::InvalidReference)
    } else {
        Ok(xref[id].position)
    }
}

fn read_object<S>(reader: &mut ObjectReader<S>, reference: Reference) -> Result<Dictionary>
where
    S: Read + Seek,
{
    reader.need_u32(reference.id)?;
    reader.need_u32(reference.gen as u32)?;
    reader.need_keyword(PdfKeyword::obj)?;
    reader.need_dictionary()
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
        assert!(PdfSource::new(pdf).is_ok());
    }

    #[test]
    fn bad_header() {
        let pdf = pdf_str("%PDx-1.3\n% bad pdf header\n");
        assert!(PdfSource::new(pdf).is_err());
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
        let pdf = PdfSource::new(open_test_file("minimal.pdf")).unwrap();
        assert_eq!(pdf.xref.len(), 5);
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
        let pdf = PdfSource::new(open_test_file("tracemonkey.pdf")).unwrap();
        assert_eq!(pdf.xref.len(), 997);
    }
}
