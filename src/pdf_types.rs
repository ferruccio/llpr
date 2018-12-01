use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/codegen_names.rs"));
include!(concat!(env!("OUT_DIR"), "/codegen_keywords.rs"));

pub type PdfString = Vec<u8>;
pub type Dictionary = Box<HashMap<PdfName, PdfObject>>;
pub type Array = Box<Vec<PdfObject>>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Reference {
    pub id: u32,
    pub gen: u16,
}

impl Reference {
    pub fn new(id: u32, gen: u16) -> Reference {
        Reference { id: id, gen: gen }
    }
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
    Symbol(PdfString), // a Symbol is an unrecognized Name
    Array(Array),
    Dictionary(Dictionary),
    Reference(Reference),
}

#[derive(Debug, PartialEq)]
pub enum PdfToken {
    Keyword(PdfKeyword),
    Integer(i64),
    Real(f64),
    Name(PdfName),
    Symbol(PdfString), // a Symbol is an unrecognized Name
    Str(PdfString),
    BeginArray,
    EndArray,
    BeginDictionary,
    EndDictionary,
}
