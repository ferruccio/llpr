extern crate phf_codegen;

use std::collections::hash_set::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let names = load("names.txt");
    generate("codegen_names.rs", "NAMES", "PdfName", names);

    let keywords = load("keywords.txt");
    generate("codegen_keywords.rs", "KEYWORDS", "PdfKeyword", keywords);
}

fn load(filename: &str) -> Vec<String> {
    let mut strings = HashSet::<String>::new();
    strings.insert("Unknown".to_owned());
    let of = File::open(filename).unwrap();
    let file = BufReader::new(&of);
    for (_, line) in file.lines().enumerate() {
        let l = line.unwrap().trim().to_owned();
        if l.len() > 0 && l.chars().next().unwrap() != '#' {
            strings.insert(l.to_owned());
        }
    }
    strings.into_iter().collect()
}

fn generate(filename: &str, target: &str, typename: &str, entries: Vec<String>) {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join(filename);
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(&mut file, "#[derive(Debug, Clone, PartialEq, Eq, Hash)]");
    writeln!(&mut file, "#[allow(non_camel_case_types)]");
    writeln!(&mut file, "pub enum {} {{", typename);
    for entry in entries.iter() {
        writeln!(&mut file, "    r#{},", entry);
    }
    writeln!(&mut file, "}}\n");

    write!(
        &mut file,
        "static {}: phf::Map<&'static str, {}> = ",
        target, typename
    ).unwrap();
    let mut builder = phf_codegen::Map::new();
    for entry in entries.iter() {
        builder.entry(&entry[..], &format!("{}::r#{}", typename, entry));
    }
    builder.build(&mut file).unwrap();
    writeln!(&mut file, ";").unwrap();
}
