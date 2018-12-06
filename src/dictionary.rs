use crate::pdf_types::*;

pub trait Access {
    // lookup methods
    fn get_reference(&self, name: PdfName) -> Option<Reference>;
    fn get_i32(&self, name: PdfName) -> Option<i32>;
    fn get_u32(&self, name: PdfName) -> Option<u32>;
    fn get_u64(&self, name: PdfName) -> Option<u64>;
    fn get_string(&self, name: PdfName) -> Option<PdfString>;
    fn get_name(&self, name: PdfName) -> Option<PdfName>;
    fn get_symbol(&self, name: PdfName) -> Option<PdfString>;
    fn get_number(&self, name: PdfName) -> Option<PdfNumber>;
    fn get_array(&self, name: PdfName) -> Option<Array>;
    fn get_dictionary(&self, name: PdfName) -> Option<Dictionary>;

    // extraction methods
    fn remove_string(&mut self, name: PdfName) -> Option<PdfString>;
    fn remove_symbol(&mut self, name: PdfName) -> Option<PdfString>;
    fn remove_dictionary(&mut self, name: PdfName) -> Option<Dictionary>;
    fn remove_array(&mut self, name: PdfName) -> Option<Array>;
}

impl Access for Dictionary {
    // lookup methods
    fn get_reference(&self, name: PdfName) -> Option<Reference> {
        match self.get(&name) {
            Some(PdfObject::Reference(ref r)) => Some(*r),
            _ => None,
        }
    }

    fn get_i32(&self, name: PdfName) -> Option<i32> {
        match self.get(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(i))) => Some(*i as i32),
            _ => None,
        }
    }

    fn get_u32(&self, name: PdfName) -> Option<u32> {
        match self.get(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(u))) => Some(*u as u32),
            _ => None,
        }
    }

    fn get_u64(&self, name: PdfName) -> Option<u64> {
        match self.get(&name) {
            Some(PdfObject::Number(PdfNumber::Integer(u))) => Some(*u as u64),
            _ => None,
        }
    }

    fn get_string(&self, name: PdfName) -> Option<PdfString> {
        match self.get(&name) {
            Some(PdfObject::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn get_name(&self, name: PdfName) -> Option<PdfName> {
        match self.get(&name) {
            Some(PdfObject::Name(name)) => Some(name.clone()),
            _ => None,
        }
    }

    fn get_symbol(&self, name: PdfName) -> Option<PdfString> {
        match self.get(&name) {
            Some(PdfObject::Symbol(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn get_number(&self, name: PdfName) -> Option<PdfNumber> {
        match self.get(&name) {
            Some(PdfObject::Number(n)) => Some(n.clone()),
            _ => None,
        }
    }

    fn get_array(&self, name: PdfName) -> Option<Array> {
        match self.get(&name) {
            Some(PdfObject::Array(a)) => Some(a.clone()),
            _ => None,
        }
    }

    fn get_dictionary(&self, name: PdfName) -> Option<Dictionary> {
        match self.get(&name) {
            Some(PdfObject::Dictionary(d)) => Some(d.clone()),
            _ => None,
        }
    }

    // extraction methods
    fn remove_string(&mut self, name: PdfName) -> Option<PdfString> {
        match self.remove(&name) {
            Some(PdfObject::String(s)) => Some(s),
            _ => None,
        }
    }

    fn remove_symbol(&mut self, name: PdfName) -> Option<PdfString> {
        match self.remove(&name) {
            Some(PdfObject::Symbol(s)) => Some(s),
            _ => None,
        }
    }

    fn remove_dictionary(&mut self, name: PdfName) -> Option<Dictionary> {
        match self.remove(&name) {
            Some(PdfObject::Dictionary(d)) => Some(d),
            _ => None,
        }
    }

    fn remove_array(&mut self, name: PdfName) -> Option<Array> {
        match self.remove(&name) {
            Some(PdfObject::Array(a)) => Some(a),
            _ => None,
        }
    }
}
