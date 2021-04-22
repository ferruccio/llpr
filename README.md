## llpr

**llpr** (low-level PDF reader) is intended to support applications which need
fast access to raw PDF data streams.

### usage:

- Instantiate a `PDFDocument` with `PDFDocument::new` which expects a
  `PDFSource`. Currently `ByteSource` and `ByteSliceSource` are supported
  which expect the PDF source to be a `Vec<u8>` and a `&[u8]` respectively.
  Other sources can be used by implementing the `Source` trait.

- Once you've created a `PDFDocument` the `page_contents` function will accept
  a page number (zero-based) and return a `PageContents` object. Use the `page_count`
  function to determine how many pages are in the document.

- Repeatedly call the `next_object` function on the `PageContents` object to
  retrieve `PdfObject`s.

A `PDFObject` is **_really_** low-level. See `pdf_types.rs` for a definition.
This crate may be useful for building up higher level abstractions but can't
help you if you're looking for something that can easily extract text or images
from a PDF file.
