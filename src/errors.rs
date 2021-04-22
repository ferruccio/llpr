use thiserror::Error;

use crate::pdf_types::*;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("decompression error: {0:?}")]
    DecompressionError (String),

    #[error("internal error: {0:?}")]
    InternalError (&'static str),

    #[error("invalid pdf file: {0:?}")]
    InvalidPdf (&'static str),

    #[error("pdf keyword expected: {0:?}")]
    KeywordExpected (PdfKeyword),

    #[error("reference not found")]
    InvalidReference,

    #[error("target does not match reference")]
    InvalidReferenceTarget,

    #[error("invalid page number")]
    InvalidPageNumber,

    #[error("unexpected end of file")]
    EndOfFile,

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("failed to parse int")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("failed to parse float")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}