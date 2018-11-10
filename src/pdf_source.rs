use token_reader::{TokenReader, PdfName, PdfString, PdfToken};
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use errors::*;

type Result<T> = ::std::result::Result<T, PdfError>;

type Array = Box<Vec<PdfObject>>;
type Dictionary = Box<HashMap<PdfName, PdfObject>>;

#[derive(Clone, PartialEq, Debug)]
pub struct Reference {
    id: u32,
    gen: u16
}

#[derive(Clone, PartialEq, Debug)]
pub struct Stream {
    header: Dictionary,
    position: u64
}

#[derive(Clone, PartialEq, Debug)]
pub enum Number {
    Integer(i64),
    Real(f64)
}

#[derive(Clone, PartialEq, Debug)]
pub enum PdfObject {
    Keyword(String),
    Null,
    Boolean(bool),
    Number(Number),
    String(PdfString),
    Name(PdfName),
    Array(Array),
    Dictionary(Dictionary),
    Stream(Stream),
    Reference(Reference)
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
        let mut ps = PdfSource { reader: TokenReader::new(source) };
        ps.reader.seek(SeekFrom::Start(trailer_position))?;
        let (trailer, startxref) = ps.read_trailer()?;
        Ok(ps)
    }

    #[cfg(test)]
    pub fn without_validation(source: S) -> PdfSource<S>
    where S: Read + Seek {
        PdfSource { reader: TokenReader::new(source) }
    }

    #[cfg(test)]
    pub fn next_raw(&mut self) -> PdfObject {
        self.next().unwrap().unwrap()
    }

    pub fn next(&mut self) -> Result<Option<PdfObject>> {
        match self.reader.next()? {
            PdfToken::Ident(id) => Ok(Some(self.ident(&id))),
            PdfToken::Integer(i) => Ok(Some(PdfObject::Number(Number::Integer(i)))),
            PdfToken::Real(r) => Ok(Some(PdfObject::Number(Number::Real(r)))),
            PdfToken::Name(name) => Ok(Some(PdfObject::Name(name))),
            PdfToken::Str(s) => Ok(Some(PdfObject::String(s))),
            PdfToken::BeginArray => self.array(),
            PdfToken::BeginDictionary => self.dictionary(),
            PdfToken::EndArray | PdfToken::EndDictionary => Ok(None)
        }
    }

    fn ident(&self, id: &str) -> PdfObject {
        match id {
            "null" => PdfObject::Null,
            "true" => PdfObject::Boolean(true),
            "false" => PdfObject::Boolean(false),
            id @ _ => PdfObject::Keyword(id.to_owned())
        }
    }

    fn array(&mut self) -> Result<Option<PdfObject>> {
        let mut a = Box::new(vec![]);
        loop {
            match self.next()? {
                Some(PdfObject::Keyword(ref keyword))
                    if keyword == "R" => self.reference(&mut a)?,
                Some(obj) => a.push(obj),
                None => return Ok(Some(PdfObject::Array(a)))
            }
        }
    }

    fn dictionary(&mut self) -> Result<Option<PdfObject>> {
        let mut a = vec![];
        loop {
            match self.next()? {
                Some(PdfObject::Keyword(ref keyword))
                    if keyword == "R" => self.reference(&mut a)?,
                Some(obj) => a.push(obj),
                None => {
                    if a.len() % 2 != 0 {
                        a.push(PdfObject::Null);
                    }
                    let mut m: Box<HashMap<PdfName, PdfObject>> = Box::new(HashMap::new());
                    for i in (0..a.len()).step_by(2) {
                        match (&a[i], &a[i + 1]) {
                            (PdfObject::Name(name), obj) => { m.insert(name.clone(), obj.clone()); }
                            _ => return Err(PdfError::InvalidPdf("malformed dictionary"))
                        }
                    }
                    return Ok(Some(PdfObject::Dictionary(m)));
                }
            }
        }
    }

    fn reference(&mut self, a: &mut Vec<PdfObject>) -> Result<()> {
        if a.len() < 2 {
            Err(PdfError::InvalidPdf("not enough arguments for R"))
        } else {
            let gen = a.pop().unwrap();
            let id = a.pop().unwrap();
            match (id, gen) {
                (PdfObject::Number(Number::Integer(id)),
                 PdfObject::Number(Number::Integer(gen)))
                    => {
                        a.push(PdfObject::Reference(Reference {
                            id: id as u32,
                            gen: gen as u16
                        }));
                        Ok(())
                    },
                _ => Err(PdfError::InvalidPdf("invalid arguments to R"))
            }
        }
    }

    fn read_trailer(&mut self) -> Result<(Dictionary, u64)>
    where S: Read + Seek {
        if let Some(PdfObject::Dictionary(trailer_dict)) = self.next()? {
            if let Some(PdfObject::Keyword(ref keyword)) = self.next()? {
                if keyword == "startxref" {
                    if let Some(PdfObject::Number(Number::Integer(addr))) = self.next()? {
                        return Ok((trailer_dict, addr as u64));
                    }
                }
            }
        }
        Err(PdfError::InvalidPdf("invalid pdf trailer"))
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

    fn tokens(source: &'static str) -> impl Read + Seek {
        Cursor::new(source.as_bytes())
    }


    #[test]
    fn keywords() {
        let mut ps = PdfSource::without_validation(tokens(
            " keyword\n\tKeyWord "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Keyword("keyword".to_owned()));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Keyword("KeyWord".to_owned()));
    }

    #[test]
    fn value_keywords() {
        let mut ps = PdfSource::without_validation(tokens(
            "null true false "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Null);
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Boolean(true));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Boolean(false));
    }

    #[test]
    fn numbers() {
        let mut ps = PdfSource::without_validation(tokens(
            "0 0.0 1 1.0 -10.34 10000.5 "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Integer(0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Real(0.0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Integer(1)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Real(1.0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Real(-10.34)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(Number::Real(10000.5)));
    }

    #[test]
    fn strings() {
        let mut ps = PdfSource::without_validation(tokens(
            "() (string) (Another \t (string)) <> <a1b2> <a1b>"));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![115, 116, 114, 105, 110, 103]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![65, 110, 111, 116, 104, 101, 114,
            32, 9, 32, 40, 115, 116, 114, 105, 110, 103, 41]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb2]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb0]));
    }

    #[test]
    fn names() {
        let mut ps = PdfSource::without_validation(tokens(
            "/Name /AnotherName "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Name(PdfName("Name".to_owned())));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Name(PdfName("AnotherName".to_owned())));
    }

    #[test]
    fn array() {
        let mut ps = PdfSource::without_validation(tokens(
            "[0 null [(string)] 1.0] "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Array(Box::new(vec![
            PdfObject::Number(Number::Integer(0)),
            PdfObject::Null,
            PdfObject::Array(Box::new(vec![
                PdfObject::String(vec![115, 116, 114, 105, 110, 103])])),
            PdfObject::Number(Number::Real(1.0))])));
    }

    #[test]
    fn dictionary() {
        let mut ps1 = PdfSource::without_validation(tokens(
            r##"<<
                /Name (Fred)
                /Age 35
                /Friends [(Barney) (Betty)]
                /Ref 10 0 R
                /Wife <<
                    /Name (Wilma)
                    /Age 32
                    /Ref 11 2 R
                >>
            >> "##));
        let n1 = ps1.next_raw();
        let mut ps2 = PdfSource::without_validation(tokens(
            r##"<<
                /Wife <<
                    /Age    32
                    /Name    (Wilma)
                    /Ref 11 2 R
                >>
                /Name (Fred)
                /Age  35
                /Ref    10 0 R
                /Friends   [(Barney)   (Betty)]
            >> "##));
        let n2 = ps2.next_raw();
        assert_eq!(n1, n2);
    }
}
