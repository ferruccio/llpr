use dictionary::{ExtractOption, ExtractRequired};
use errors::*;
use pdf_types::*;

type Result<T> = ::std::result::Result<T, PdfError>;

pub struct Trailer {
    pub size: u32,
    pub prev: Option<u64>,
    pub root: Reference,
    pub encrypt: Option<Dictionary>,
    pub info: Option<Dictionary>,
    pub id: Option<Array>,
}

impl Trailer {
    pub fn new(mut dict: Dictionary) -> Result<Trailer> {
        Ok(Trailer {
            size: dict.need_u32(PdfName::Size, PdfError::InvalidTrailerDictionary)?,
            prev: dict.want_u64(PdfName::Prev),
            root: dict.need_reference(PdfName::Root, PdfError::InvalidTrailerDictionary)?,
            encrypt: dict.want_dictionary(PdfName::Encrypt),
            info: dict.want_dictionary(PdfName::Info),
            id: dict.want_array(PdfName::ID),
        })
    }
}
