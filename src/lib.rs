mod dictionary;
mod errors;
mod next_object;
mod next_token;
mod page_contents;
mod pdf_document;
mod pdf_source;
mod pdf_types;
mod streams;

pub type Result<T> = std::result::Result<T, errors::PdfError>;

pub use crate::errors::PdfError;
pub use crate::pdf_document::PdfDocument;
pub use crate::pdf_source::{ByteSliceSource, ByteSource, PdfSource, Source};
