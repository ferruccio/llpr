#[macro_use]
extern crate criterion;
extern crate llpr;

use criterion::Criterion;
use llpr::*;

fn parse_all_streams(buffer: &'static [u8]) {
    let source = Box::new(ByteSliceSource::new(buffer));
    let mut pdf = PdfDocument::new(source).unwrap();
    for pageno in 0..pdf.page_count() {
        let mut contents = pdf.page_contents(pageno).unwrap();
        while contents.next_object().unwrap() != None {}
    }
}

static TRACEMONKEY_PDF: &'static [u8] = include_bytes!("../testing/tracemonkey.pdf");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tracemonkey.pdf streams", |b| {
        b.iter(|| parse_all_streams(TRACEMONKEY_PDF))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
