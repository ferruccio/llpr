use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum PdfError {
        InvalidPdf(detail: &'static str) {
            display("Invalid PDF file: {}", detail)
        }
        EndOfFile {
            display("unexpected end of file")
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