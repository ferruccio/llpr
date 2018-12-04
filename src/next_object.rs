use errors::*;
use next_token::next_token;
use pdf_source::Source;
use pdf_types::*;
use std::collections::HashMap;

type Result<T> = ::std::result::Result<T, PdfError>;

pub fn next_object(source: &mut Box<Source>) -> Result<Option<PdfObject>> {
    match next_token(source)? {
        Some(PdfToken::Keyword(PdfKeyword::null)) => Ok(Some(PdfObject::Null)),
        Some(PdfToken::Keyword(PdfKeyword::r#true)) => Ok(Some(PdfObject::Boolean(true))),
        Some(PdfToken::Keyword(PdfKeyword::r#false)) => Ok(Some(PdfObject::Boolean(false))),
        Some(PdfToken::Keyword(keyword)) => Ok(Some(PdfObject::Keyword(keyword))),
        Some(PdfToken::Integer(i)) => Ok(Some(PdfObject::Number(PdfNumber::Integer(i)))),
        Some(PdfToken::Real(r)) => Ok(Some(PdfObject::Number(PdfNumber::Real(r)))),
        Some(PdfToken::Name(name)) => Ok(Some(PdfObject::Name(name))),
        Some(PdfToken::Symbol(symbol)) => Ok(Some(PdfObject::Symbol(symbol))),
        Some(PdfToken::Str(s)) => Ok(Some(PdfObject::String(s))),
        Some(PdfToken::BeginArray) => array(source),
        Some(PdfToken::BeginDictionary) => dictionary(source),
        Some(PdfToken::EndArray) | Some(PdfToken::EndDictionary) => Ok(None),
        None => Ok(None),
    }
}

pub fn need_keyword(source: &mut Box<Source>, keyword: PdfKeyword) -> Result<()> {
    match next_object(source)? {
        Some(PdfObject::Keyword(ref k)) if k == &keyword => Ok(()),
        _ => Err(PdfError::KeywordExpected(keyword)),
    }
}

pub fn need_u32(source: &mut Box<Source>, value: u32) -> Result<()> {
    match next_object(source)? {
        Some(PdfObject::Number(PdfNumber::Integer(i))) if i == value as i64 => Ok(()),
        _ => Err(PdfError::InvalidReferenceTarget),
    }
}

pub fn need_dictionary(source: &mut Box<Source>) -> Result<Dictionary> {
    match next_object(source)? {
        Some(PdfObject::Dictionary(d)) => Ok(d),
        _ => Err(PdfError::InvalidPdf("dictionary expected")),
    }
}

fn array(source: &mut Box<Source>) -> Result<Option<PdfObject>> {
    let mut array = Box::new(vec![]);
    loop {
        match next_object(source)? {
            Some(PdfObject::Keyword(PdfKeyword::R)) => reference(&mut array)?,
            Some(obj) => array.push(obj),
            None => return Ok(Some(PdfObject::Array(array))),
        }
    }
}

fn dictionary(source: &mut Box<Source>) -> Result<Option<PdfObject>> {
    let mut array = vec![];
    loop {
        match next_object(source)? {
            Some(PdfObject::Keyword(PdfKeyword::R)) => reference(&mut array)?,
            Some(obj) => array.push(obj),
            None => {
                if array.len() % 2 != 0 {
                    array.push(PdfObject::Null);
                }
                let mut dict: Dictionary = Box::new(HashMap::new());
                while array.len() != 0 {
                    let value = array.pop().unwrap();
                    let name = array.pop().unwrap();
                    match name {
                        PdfObject::Name(name) => {
                            dict.insert(name, value);
                        }
                        PdfObject::Symbol(_) => {}
                        _ => return Err(PdfError::InvalidPdf("malformed dictionary")),
                    }
                }
                return Ok(Some(PdfObject::Dictionary(dict)));
            }
        }
    }
}

fn reference(array: &mut Vec<PdfObject>) -> Result<()> {
    if array.len() < 2 {
        Err(PdfError::InvalidPdf("not enough arguments for R"))
    } else {
        let gen = array.pop().unwrap();
        let id = array.pop().unwrap();
        match (id, gen) {
            (
                PdfObject::Number(PdfNumber::Integer(id)),
                PdfObject::Number(PdfNumber::Integer(gen)),
            ) => {
                array.push(PdfObject::Reference(Reference::new(id as u32, gen as u16)));
                Ok(())
            }
            _ => Err(PdfError::InvalidPdf("invalid arguments to R")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_source::ByteSliceSource;

    fn next(source: &mut Box<Source>) -> PdfObject {
        next_object(source).unwrap().unwrap()
    }

    #[test]
    fn keywords() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b" trailer\n\txref "));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Keyword(PdfKeyword::trailer));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Keyword(PdfKeyword::xref));
    }

    #[test]
    fn value_keywords() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b"null true false "));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Null);
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Boolean(true));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Boolean(false));
    }

    #[test]
    fn numbers() {
        let mut source: Box<Source> =
            Box::new(ByteSliceSource::new(b"0 0.0 1 1.0 -10.34 10000.5 "));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Integer(0)));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(0.0)));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Integer(1)));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(1.0)));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(-10.34)));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Number(PdfNumber::Real(10000.5)));
    }

    #[test]
    fn strings() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(
            b"() (string) (Another \t (string)) <> <a1b2> <a1b>",
        ));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::String(vec![]));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::String(vec![115, 116, 114, 105, 110, 103]));
        let n = next(&mut source);
        assert_eq!(
            n,
            PdfObject::String(vec![
                65, 110, 111, 116, 104, 101, 114, 32, 9, 32, 40, 115, 116, 114, 105, 110, 103, 41
            ])
        );
        let n = next(&mut source);
        assert_eq!(n, PdfObject::String(vec![]));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb2]));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::String(vec![0xa1, 0xb0]));
    }

    #[test]
    fn names() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b"/Root /Size "));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Name(PdfName::Root));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Name(PdfName::Size));
    }

    #[test]
    fn symbols() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b"/Who /What "));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Symbol("Who".as_bytes().to_vec()));
        let n = next(&mut source);
        assert_eq!(n, PdfObject::Symbol("What".as_bytes().to_vec()));
    }

    #[test]
    fn array() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b"[0 null [(string)] 1.0] "));
        let n = next(&mut source);
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
    fn array_of_references() {
        let mut source: Box<Source> = Box::new(ByteSliceSource::new(b"[0 1 R 2 3 R 4 5 R] "));
        let a = next(&mut source);
        assert_eq!(
            a,
            PdfObject::Array(Box::new(vec![
                PdfObject::Reference(Reference { id: 0, gen: 1 }),
                PdfObject::Reference(Reference { id: 2, gen: 3 }),
                PdfObject::Reference(Reference { id: 4, gen: 5 })
            ]))
        );
    }

    #[test]
    fn dictionary() {
        let mut source1: Box<Source> = Box::new(ByteSliceSource::new(
            br##"<<
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
        let n1 = next(&mut source1);
        let mut source2: Box<Source> = Box::new(ByteSliceSource::new(
            br##"<<
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
        let n2 = next(&mut source2);
        assert_eq!(n1, n2);
    }
}
