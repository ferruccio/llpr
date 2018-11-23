use quick_error::quick_error;
use token_reader::PdfKeyword;

quick_error! {
    #[derive(Debug)]
    pub enum PdfError {
        InvalidPdf(detail: &'static str) {
            description("invalid pdf file")
            display("Invalid PDF file: {}", detail)
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
        InvalidTrailerDictionary {
            description("trailer dictionary missing required item(s)")
        }
        InvalidCatalog {
            description("document catalog missing required item(s)")
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
