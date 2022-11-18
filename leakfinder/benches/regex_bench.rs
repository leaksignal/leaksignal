use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fancy_regex::Regex as FRegex;
use regex::Regex;

pub fn benchmark_fancy_regex(c: &mut Criterion) {
    let regex =
        FRegex::new(r#"[^0-9](?:\+1[\s.-])?(?:\d{3}|\(\d{3}\))[\s.-]\d{3}[\s.-]\d{4}[^0-9]"#)
            .unwrap();
    c.bench_function("phone number fancy_regex", |b| {
        b.iter(|| {
            for x in regex.find_iter(include_str!("../../scripts/local/pub/ssn001.html")) {
                black_box(x.unwrap());
            }
        })
    });
}

pub fn benchmark_regex(c: &mut Criterion) {
    let regex =
        Regex::new(r#"[^0-9](?:\+1[\s.-])?(?:\d{3}|\(\d{3}\))[\s.-]\d{3}[\s.-]\d{4}[^0-9]"#)
            .unwrap();
    c.bench_function("phone number regex", |b| {
        b.iter(|| {
            for x in regex.find_iter(include_str!("../../scripts/local/pub/ssn001.html")) {
                black_box(x);
            }
        })
    });
}

criterion_group!(benches, benchmark_fancy_regex, benchmark_regex);
criterion_main!(benches);
