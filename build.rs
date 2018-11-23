extern crate phf_codegen;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let names = [
        "Unknown",
        "AA",
        "AcroForm",
        "Ascent",
        "ASCII85Decode",
        "ASCIIHexDecode",
        "CCITTFaxDecode",
        "Catalog",
        "Collection",
        "Crypt",
        "DCTDecode",
        "Dests",
        "DL",
        "Encrypt",
        "Extensions",
        "FlateDecode",
        "Font",
        "FontDescriptor",
        "FontFile",
        "ID",
        "Info",
        "JBIG2Decode",
        "JPXDecode",
        "Lang",
        "Legal",
        "Length",
        "LZWDecode",
        "MarkInfo",
        "Metadata",
        "Names",
        "NeedsRendering",
        "OCProperties",
        "OpenAction",
        "Outlines",
        "OutputIntents",
        "Page",
        "Pages",
        "PageLabels",
        "PageLayout",
        "PageMode",
        "Perms",
        "PieceInfo",
        "Prev",
        "Requirements",
        "Root",
        "RunLengthDecode",
        "Size",
        "SpiderInfo",
        "StructTreeRoot",
        "SubType",
        "Threads",
        "ToUnicode",
        "Type",
        "URI",
        "Version",
        "ViewerPreferences",
    ];
    generate("codegen_names.rs", "NAMES", "PdfName", &names[..]);

    let keywords = [
        "Unknown",
        "endobj",
        "endstream",
        "f",
        "false",
        "n",
        "null",
        "obj",
        "R",
        "stream",
        "trailer",
        "true",
        "startxref",
        "xref",
    ];
    generate(
        "codegen_keywords.rs",
        "KEYWORDS",
        "PdfKeyword",
        &keywords[..],
    );
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

    write!(
        &mut file,
        "static {}: phf::Map<&'static str, {}> = ",
        target, typename
    ).unwrap();
    let mut builder = phf_codegen::Map::new();
    for &entry in entries {
        builder.entry(entry, &format!("{}::r#{}", typename, entry));
    }
    builder.build(&mut file).unwrap();
    writeln!(&mut file, ";").unwrap();
}
