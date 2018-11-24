use dictionary::Dictionary;
use errors::*;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use token_reader::{PdfKeyword, PdfName, PdfString, PdfToken, TokenReader};

type Result<T> = ::std::result::Result<T, PdfError>;

pub type Array = Box<Vec<PdfObject>>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Reference {
    pub id: u32,
    pub gen: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Stream {
    header: Dictionary,
    position: u64,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PdfNumber {
    Integer(i64),
    Real(f64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum PdfObject {
    Null,
    Keyword(PdfKeyword),
    Boolean(bool),
    Number(PdfNumber),
    String(PdfString),
    Name(PdfName),
    Symbol(String), // a Symbol is an unrecognized Name
    Array(Array),
    Dictionary(Dictionary),
    Stream(Stream),
    Reference(Reference),
}

pub struct ObjectReader<S>
where
    S: Seek + Read,
{
    tokens: TokenReader<S>,
}

impl<S> ObjectReader<S>
where
    S: Read + Seek,
{
    pub fn new(tokens: TokenReader<S>) -> ObjectReader<S> {
        ObjectReader { tokens: tokens }
    }

    #[cfg(test)]
    pub fn without_validation(source: S) -> ObjectReader<S>
    where
        S: Read + Seek,
    {
        ObjectReader {
            tokens: TokenReader::new(source),
        }
    }

    #[cfg(test)]
    pub fn next_raw(&mut self) -> PdfObject {
        self.next().unwrap().unwrap()
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        Ok(self.tokens.seek(pos)?)
    }

    pub fn next(&mut self) -> Result<Option<PdfObject>> {
        match self.tokens.next()? {
            PdfToken::Keyword(PdfKeyword::null) => Ok(Some(PdfObject::Null)),
            PdfToken::Keyword(PdfKeyword::r#true) => Ok(Some(PdfObject::Boolean(true))),
            PdfToken::Keyword(PdfKeyword::r#false) => Ok(Some(PdfObject::Boolean(false))),
            PdfToken::Keyword(keyword) => Ok(Some(PdfObject::Keyword(keyword))),
            PdfToken::Integer(i) => Ok(Some(PdfObject::Number(PdfNumber::Integer(i)))),
            PdfToken::Real(r) => Ok(Some(PdfObject::Number(PdfNumber::Real(r)))),
            PdfToken::Name(name) => Ok(Some(PdfObject::Name(name))),
            PdfToken::Symbol(symbol) => Ok(Some(PdfObject::Symbol(symbol))),
            PdfToken::Str(s) => Ok(Some(PdfObject::String(s))),
            PdfToken::BeginArray => self.array(),
            PdfToken::BeginDictionary => self.dictionary(),
            PdfToken::EndArray | PdfToken::EndDictionary => Ok(None),
        }
    }

    pub fn need_keyword(&mut self, keyword: PdfKeyword) -> Result<()> {
        match self.next()? {
            Some(PdfObject::Keyword(ref k)) if k == &keyword => Ok(()),
            _ => Err(PdfError::KeywordExpected(keyword)),
        }
    }

    pub fn need_u32(&mut self, value: u32) -> Result<()> {
        match self.next()? {
            Some(PdfObject::Number(PdfNumber::Integer(i))) if i == value as i64 => Ok(()),
            _ => Err(PdfError::InvalidReferenceTarget),
        }
    }

    pub fn need_dictionary(&mut self) -> Result<Dictionary> {
        match self.next()? {
            Some(PdfObject::Dictionary(d)) => Ok(d),
            _ => Err(PdfError::InvalidPdf("dictionary expected")),
        }
    }

    fn array(&mut self) -> Result<Option<PdfObject>> {
        let mut source = Box::new(vec![]);
        loop {
            match self.next()? {
                Some(PdfObject::Keyword(PdfKeyword::R)) => self.reference(&mut source)?,
                Some(obj) => source.push(obj),
                None => return Ok(Some(PdfObject::Array(source))),
            }
        }
    }

    fn dictionary(&mut self) -> Result<Option<PdfObject>> {
        let mut source = vec![];
        loop {
            match self.next()? {
                Some(PdfObject::Keyword(PdfKeyword::R)) => self.reference(&mut source)?,
                Some(obj) => source.push(obj),
                None => {
                    if source.len() % 2 != 0 {
                        source.push(PdfObject::Null);
                    }
                    let mut dict: Dictionary = Box::new(HashMap::new());
                    while source.len() != 0 {
                        let value = source.pop().unwrap();
                        let name = source.pop().unwrap();
                        match name {
                            PdfObject::Name(name) => {
                                dict.insert(name, value);
                            }
                            _ => return Err(PdfError::InvalidPdf("malformed dictionary")),
                        }
                    }
                    return Ok(Some(PdfObject::Dictionary(dict)));
                }
            }
        }
    }

    fn reference(&mut self, source: &mut Vec<PdfObject>) -> Result<()> {
        if source.len() < 2 {
            Err(PdfError::InvalidPdf("not enough arguments for R"))
        } else {
            let gen = source.pop().unwrap();
            let id = source.pop().unwrap();
            match (id, gen) {
                (
                    PdfObject::Number(PdfNumber::Integer(id)),
                    PdfObject::Number(PdfNumber::Integer(gen)),
                ) => {
                    source.push(PdfObject::Reference(Reference {
                        id: id as u32,
                        gen: gen as u16,
                    }));
                    Ok(())
                }
                _ => Err(PdfError::InvalidPdf("invalid arguments to R")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Seek};

    fn tokens(source: &'static str) -> impl Read + Seek {
        Cursor::new(source.as_bytes())
    }

    #[test]
    fn keywords() {
        let mut ps = ObjectReader::without_validation(tokens(" trailer\n\txref "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Keyword(PdfKeyword::trailer));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Keyword(PdfKeyword::xref));
    }

    #[test]
    fn value_keywords() {
        let mut ps = ObjectReader::without_validation(tokens("null true false "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Null);
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Boolean(true));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Boolean(false));
    }

    #[test]
    fn numbers() {
        let mut ps = ObjectReader::without_validation(tokens("0 0.0 1 1.0 -10.34 10000.5 "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Integer(0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(0.0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Integer(1)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(1.0)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(-10.34)));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(10000.5)));
    }

    #[test]
    fn strings() {
        let mut ps = ObjectReader::without_validation(tokens(
            "() (string) (Another \t (string)) <> <a1b2> <a1b>",
        ));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![115, 116, 114, 105, 110, 103]));
        let n = ps.next_raw();
        assert_eq!(
            n,
            PdfObject::String(vec![
                65, 110, 111, 116, 104, 101, 114, 32, 9, 32, 40, 115, 116, 114, 105, 110, 103, 41
            ])
        );
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb2]));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb0]));
    }

    #[test]
    fn names() {
        let mut ps = ObjectReader::without_validation(tokens("/Root /Size "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Name(PdfName::Root));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Name(PdfName::Size));
    }

    #[test]
    fn symbols() {
        let mut ps = ObjectReader::without_validation(tokens("/Who /What "));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Symbol("Who".to_owned()));
        let n = ps.next_raw();
        assert_eq!(n, PdfObject::Symbol("What".to_owned()));
    }

    #[test]
    fn array() {
        let mut ps = ObjectReader::without_validation(tokens("[0 null [(string)] 1.0] "));
        let n = ps.next_raw();
        assert_eq!(
            n,
            PdfObject::Array(Box::new(vec![
                PdfObject::Number(PdfNumber::Integer(0)),
                PdfObject::Null,
                PdfObject::Array(Box::new(vec![PdfObject::String(vec![
                    115, 116, 114, 105, 110, 103
                ])])),
                PdfObject::Number(PdfNumber::Real(1.0))
            ]))
        );
    }

    #[test]
    fn dictionary() {
        let mut ps1 = ObjectReader::without_validation(tokens(
            r##"<<
                /Root 10 0 R
                /Size 35
                /Info [(xyzzy) (plover)]
                /ID <<
                    /Type (some type)
                    /Prev 32
                    /Metadata 11 2 R
                >>
            >> "##,
        ));
        let n1 = ps1.next_raw();
        let mut ps2 = ObjectReader::without_validation(tokens(
            r##"<<
                /Root    10  0   R
                /Size   35
                /Info [(xyzzy)      (plover)]
                /ID <<
                    /Type  (some type)

                    /Prev     32
                    /Metadata 11 2 R
                >>
            >> "##,
        ));
        let n2 = ps2.next_raw();
        assert_eq!(n1, n2);
    }
}
