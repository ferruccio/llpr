use crate::pdf_types::*;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum PdfError {
        DecompressionError(detail: String) {
            description("decompression error")
            display("decompression error: {}", detail)
        }
        InternalError(detail: &'static str) {
            description("internal error")
            display("internal error: {}", detail)
        }
        InvalidPdf(detail: &'static str) {
            description("invalid pdf file")
            display("invalid pdf file: {}", detail)
        }
        KeywordExpected(keyword: PdfKeyword) {
            description("pdf keyword expected")
            display("pdf keyword expected: {:?}", keyword)
        }
        InvalidReference {
            description("reference not found")
        }
        InvalidReferenceTarget{
            description("target does not match reference")
        }
        InvalidPageNumber{
            description("invalid page number")
        }
        EndOfFile {
            description("unexpected end of file")
        }
        IoError(err: std::io::Error) {
            description(err.description())
            from()
        }
        ParseIntError(err: std::num::ParseIntError) {
            description(err.description())
            from()
        }
        ParseFloatError(err: std::num::ParseFloatError) {
            description(err.description())
            from()
        }
    }
}
