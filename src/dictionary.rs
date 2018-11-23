use errors::*;
use object_reader::{Array, PdfNumber, PdfObject, Reference};
use std::collections::HashMap;
use token_reader::PdfName;

type Result<T> = ::std::result::Result<T, PdfError>;

pub type Dictionary = Box<HashMap<PdfName, PdfObject>>;

pub trait ExtractOption {
    fn want_u32(&mut self, name: PdfName) -> Option<u32>;
    fn want_u64(&mut self, name: PdfName) -> Option<u64>;
    fn want_name(&mut self, name: PdfName) -> Option<PdfName>;
    fn want_reference(&mut self, name: PdfName) -> Option<Reference>;
    fn want_dictionary(&mut self, name: PdfName) -> Option<Dictionary>;
    fn want_array(&mut self, name: PdfName) -> Option<Array>;
}

pub trait ExtractRequired {
    fn need_u32(&mut self, name: PdfName, err: PdfError) -> Result<u32>;
    fn need_reference(&mut self, name: PdfName, err: PdfError) -> Result<Reference>;
    fn need_dictionary(&mut self, name: PdfName, err: PdfError) -> Result<Dictionary>;
    fn need_type(&mut self, r#type: PdfName, err: PdfError) -> Result<PdfName>;
}

impl ExtractOption for Dictionary {
    fn want_u32(&mut self, name: PdfName) -> Option<u32> {
        match self.remove(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(i))) => Some(i as u32),
            _ => None,
        }
    }

    fn want_u64(&mut self, name: PdfName) -> Option<u64> {
        match self.remove(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(i))) => Some(i as u64),
            _ => None,
        }
    }

    fn want_name(&mut self, name: PdfName) -> Option<PdfName> {
        match self.remove(&name) {
            Some(PdfObject::Name(n)) => Some(n),
            _ => None,
        }
    }

    fn want_reference(&mut self, name: PdfName) -> Option<Reference> {
        match self.remove(&name) {
            Some(PdfObject::Reference(r)) => Some(r),
            _ => None,
        }
    }

    fn want_dictionary(&mut self, name: PdfName) -> Option<Dictionary> {
        match self.remove(&name) {
            Some(PdfObject::Dictionary(d)) => Some(d),
            _ => None,
        }
    }

    fn want_array(&mut self, name: PdfName) -> Option<Array> {
        match self.remove(&name) {
            Some(PdfObject::Array(a)) => Some(a),
            _ => None,
        }
    }
}

impl ExtractRequired for Dictionary {
    fn need_u32(&mut self, name: PdfName, err: PdfError) -> Result<u32> {
        match self.want_u32(name) {
            Some(i) => Ok(i),
            None => Err(err),
        }
    }

    fn need_reference(&mut self, name: PdfName, err: PdfError) -> Result<Reference> {
        match self.want_reference(name) {
            Some(r) => Ok(r),
            None => Err(err),
        }
    }

    fn need_dictionary(&mut self, name: PdfName, err: PdfError) -> Result<Dictionary> {
        match self.want_dictionary(name) {
            Some(d) => Ok(d),
            None => Err(err),
        }
    }

    fn need_type(&mut self, r#type: PdfName, err: PdfError) -> Result<PdfName> {
        match self.remove(&PdfName::Type) {
            Some(PdfObject::Name(ref name)) if name == &r#type => Ok(name.clone()),
            _ => Err(err),
        }
    }
}
