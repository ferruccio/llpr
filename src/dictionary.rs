use errors::*;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

pub trait Access {
    // lookup methods
    fn get_name(&self, name: PdfName) -> Option<PdfName>;
    fn get_reference(&self, name: PdfName) -> Option<Reference>;

    // optional extraction methods
    fn want_i32(&mut self, name: PdfName) -> Option<i32>;
    fn want_u32(&mut self, name: PdfName) -> Option<u32>;
    fn want_u64(&mut self, name: PdfName) -> Option<u64>;
    fn want_string(&mut self, name: PdfName) -> Option<PdfString>;
    fn want_name(&mut self, name: PdfName) -> Option<PdfName>;
    fn want_symbol(&mut self, name: PdfName) -> Option<PdfString>;
    fn want_number(&mut self, name: PdfName) -> Option<PdfNumber>;
    fn want_reference(&mut self, name: PdfName) -> Option<Reference>;
    fn want_dictionary(&mut self, name: PdfName) -> Option<Dictionary>;
    fn want_array(&mut self, name: PdfName) -> Option<Array>;

    // required extraction methods
    fn need_u32(&mut self, name: PdfName, err: PdfError) -> Result<u32>;
    fn need_reference(&mut self, name: PdfName, err: PdfError) -> Result<Reference>;
    fn need_dictionary(&mut self, name: PdfName, err: PdfError) -> Result<Dictionary>;
    fn need_array(&mut self, name: PdfName, err: PdfError) -> Result<Array>;
}

impl Access for Dictionary {
    fn get_name(&self, name: PdfName) -> Option<PdfName> {
        match self.get(&name) {
            Some(PdfObject::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }

    fn get_reference(&self, name: PdfName) -> Option<Reference> {
        match self.get(&name) {
            Some(PdfObject::Reference(ref r)) => Some(*r),
            _ => None,
        }
    }

    fn want_i32(&mut self, name: PdfName) -> Option<i32> {
        match self.remove(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(i))) => Some(i as i32),
            _ => None,
        }
    }

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

    fn want_string(&mut self, name: PdfName) -> Option<PdfString> {
        match self.remove(&name) {
            Some(PdfObject::String(s)) => Some(s),
            _ => None,
        }
    }

    fn want_name(&mut self, name: PdfName) -> Option<PdfName> {
        match self.remove(&name) {
            Some(PdfObject::Name(n)) => Some(n),
            _ => None,
        }
    }

    fn want_symbol(&mut self, name: PdfName) -> Option<PdfString> {
        match self.remove(&name) {
            Some(PdfObject::Symbol(n)) => Some(n),
            _ => None,
        }
    }

    fn want_number(&mut self, name: PdfName) -> Option<PdfNumber> {
        match self.remove(&name) {
            Some(PdfObject::Number(n)) => Some(n),
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

    fn need_array(&mut self, name: PdfName, err: PdfError) -> Result<Array> {
        match self.want_array(name) {
            Some(a) => Ok(a),
            None => Err(err),
        }
    }
}
