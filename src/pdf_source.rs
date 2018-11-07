use token_reader::{TokenReader, PdfName};
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use errors::*;

type Result<T> = ::std::result::Result<T, PdfError>;

type Array = Vec<PdfObject>;
type Dictionary = HashMap<PdfName, PdfObject>;

struct Stream {
    header: Dictionary,
    position: u64
}

enum Number {
    Integer(i64),
    Real(f64)
}

enum PdfObject {
    Null,
    Boolean(bool),
    Number(Number),
    String(String),
    Name(PdfName),
    Array(Box<Array>),
    Dictionary(Box<Dictionary>),
    Stream(Stream)
}

pub struct PdfSource<S> where S: Read + Seek {
    reader: TokenReader<S>
}

impl<S> PdfSource<S> where S: Read + Seek {
    pub fn new(mut source: S) -> Result<PdfSource<S>>
    where S: Read + Seek {
        validate_pdf(&mut source)?;
        let (position, buffer) = read_tail(&mut source)?;
        let trailer_position = find_trailer(position, &buffer)?;
        let tr = TokenReader::new(source);
        let mut ps = PdfSource { reader: tr };
        ps.reader.seek(SeekFrom::Start(trailer_position))?;
        Ok(ps)
    }

}

fn validate_pdf<S>(source: &mut S) -> Result<()>
where S: Read + Seek {
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
where S: Read + Seek {
    const BUFFER_SIZE:usize = 8 * 1024;
    let filesize = source.seek(SeekFrom::End(0))? as usize;
    let position = if filesize < BUFFER_SIZE {
        source.seek(SeekFrom::Start(0))?
    } else {
        source.seek(SeekFrom::End(BUFFER_SIZE as i64))?
    };
    let mut buffer = vec![];
    let _ = source.read_to_end(&mut buffer)?;
    Ok((position, buffer))
}

fn find_trailer(position: u64, buffer: &[u8]) -> Result<u64> {
    let trailer = "trailer".as_bytes();
    for i in (0 ..= buffer.len() - trailer.len()).rev() {
        if &buffer[i..i + trailer.len()] == trailer {
            return Ok(position + i as u64 + trailer.len() as u64);
        }
    }
    Err(PdfError::InvalidPdf("no trailer"))
}

fn read_trailer<S>(source: &mut S)
where S: Read + Seek {

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Seek, Cursor};

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
    fn good_header() {
        let pdf = pdf_str("%PDF-1.5\n% good pdf header\ntrailer\n");
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
        assert_eq!(position.unwrap(),  1000 + 22);
    }

    #[test]
    fn find_trailer_end() {
        let buffer = "blah blah blah trailer".as_bytes();
        let position = find_trailer(0, &buffer);
        assert!(position.is_ok());
        assert_eq!(position.unwrap(),  22);
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

}
