#[macro_use]
extern crate criterion;
extern crate llpr;

use criterion::Criterion;
use llpr::{ByteSliceSource, PdfDocument};

fn parse_all_streams(buffer: &'static [u8]) {
    let source = Box::new(ByteSliceSource::new(buffer));
    let mut pdf = PdfDocument::new(source).unwrap();
    for pageno in 0..pdf.page_count() {
        let mut contents = pdf.page_contents(pageno).unwrap();
        while contents.next_object().unwrap() != None {}
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("tracemonkey.pdf streams", |b| {
        b.iter(|| {
            let pdf = include_bytes!("../testing/tracemonkey.pdf");
            parse_all_streams(pdf)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
