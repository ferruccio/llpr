use crate::errors::PdfError;
use inflate::inflate_bytes_zlib;
use crate::pdf_types::*;

pub type Result<T> = std::result::Result<T, PdfError>;

struct Filter {
    name: PdfName,
    _decode_parms: Option<Dictionary>,
}

pub fn decode_stream(mut stream: Vec<u8>, stream_dict: Dictionary) -> Result<Vec<u8>> {
    let filters = filters(stream_dict)?;
    for filter in filters.iter() {
        match filter.name {
            PdfName::ASCIIHexDecode => {
                return Err(PdfError::InternalError(
                    "ASCIIHexDecode filter not implemented",
                ))
            }
            PdfName::ASCII85Decode => {
                return Err(PdfError::InternalError(
                    "ASCII85Decode filter not implemented",
                ))
            }
            PdfName::LZWDecode => {
                return Err(PdfError::InternalError("LZWDecode filter not implemented"))
            }
            PdfName::FlateDecode => {
                stream = match inflate_bytes_zlib(&stream[..]) {
                    Ok(stream) => stream,
                    Err(e) => return Err(PdfError::DecompressionError(e)),
                }
            }
            PdfName::RunLengthDecode => {
                return Err(PdfError::InternalError(
                    "RunLengthDecode filter not implemented",
                ))
            }
            PdfName::CCITTFaxDecode => {
                return Err(PdfError::InternalError(
                    "CCITTFaxDecode filter not implemented",
                ))
            }
            PdfName::JBIG2Decode => {
                return Err(PdfError::InternalError(
                    "JBIG2Decode filter not implemented",
                ))
            }
            PdfName::DCTDecode => {
                return Err(PdfError::InternalError(
                    "JBIG2Decode filter not implemented",
                ))
            }
            PdfName::Crypt => return Err(PdfError::InternalError("Crypt filter not implemented")),
            _ => return Err(PdfError::InvalidPdf("unknown filter")),
        }
    }
    Ok(stream)
}

fn filters(mut stream_dict: Dictionary) -> Result<Vec<Filter>> {
    match (
        stream_dict.remove(&PdfName::Filter),
        stream_dict.remove(&PdfName::DecodeParms),
    ) {
        (Some(PdfObject::Name(name)), None) => Ok(vec![Filter {
            name: name,
            _decode_parms: None,
        }]),
        (Some(PdfObject::Name(name)), Some(PdfObject::Dictionary(dp))) => Ok(vec![Filter {
            name: name,
            _decode_parms: Some(dp),
        }]),
        (Some(PdfObject::Array(names)), None) => {
            fn name_to_filter(name: &PdfObject) -> Result<Filter> {
                match name {
                    PdfObject::Name(name) => Ok(Filter {
                        name: name.clone(),
                        _decode_parms: None,
                    }),
                    _ => Err(PdfError::InvalidPdf("name expected")),
                }
            }

            names.iter().map(name_to_filter).collect()
        }
        (Some(PdfObject::Array(names)), Some(PdfObject::Array(dps))) => {
            fn filter(item: (&PdfObject, &PdfObject)) -> Result<Filter> {
                match item {
                    (PdfObject::Name(name), PdfObject::Dictionary(dp)) => Ok(Filter {
                        name: name.clone(),
                        _decode_parms: Some(dp.clone()),
                    }),
                    _ => Err(PdfError::InvalidPdf("name/dictionary expected")),
                }
            }

            names.iter().zip(dps.iter()).map(filter).collect()
        }
        (None, None) => Ok(vec![]),
        (_, _) => Err(PdfError::InvalidPdf("invalid stream dictionary")),
    }
}
