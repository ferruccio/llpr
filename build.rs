extern crate phf_codegen;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let names = [ "Unknown",
        "Ascent", "ASCII85Decode", "ASCIIHexDecode", "CCITTFaxDecode", "Catalog", "Crypt",
        "DCTDecode", "DL", "Encrypt", "FlateDecode", "Font", "FontDescriptor",
        "FontFile", "ID", "Info", "JBIG2Decode", "JPXDecode", "Length",
        "LZWDecode", "Page", "Prev", "Root", "RunLengthDecode", "Size", "SubType",
        "ToUnicode", "Type"
    ];
    generate("codegen_names.rs", "NAMES", "PdfName", &names[..]);

    let keywords = [ "Unknown",
        "endobj", "endstream", "f", "false", "n", "null", "obj", "R", "stream",
        "trailer", "true", "startxref", "xref"
    ];
    generate("codegen_keywords.rs", "KEYWORDS", "PdfKeyword", &keywords[..]);
}

fn generate(filename: &str, target: &str, typename: &str, entries: &[&str]) {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join(filename);
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(&mut file, "#[derive(Debug, Clone, PartialEq, Eq, Hash)]");
    writeln!(&mut file, "#[allow(non_camel_case_types)]");
    writeln!(&mut file, "pub enum {} {{", typename);
    for &entry in entries {
        writeln!(&mut file, "    r#{},", entry);
    }
    writeln!(&mut file, "}}\n");

    write!(&mut file, "static {}: phf::Map<&'static str, {}> = ", target, typename).unwrap();
    let mut builder = phf_codegen::Map::new();
    for &entry in entries {
        builder.entry(entry, &format!("{}::r#{}", typename, entry));
    }
    builder.build(&mut file).unwrap();
    writeln!(&mut file, ";").unwrap();
}
