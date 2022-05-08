use std::io::Read;

use criterion::{criterion_group, criterion_main, Criterion};

fn parse_serde_json(data: &[u8]) {
    let a: serde_json::Value = serde_json::from_slice(data).unwrap();
    assert!(a.is_array());
}

fn parse_json(data: &[u8]) {
    let a = json_parser::parse(data).unwrap();
    if let json_parser::Value::Array(_) = a {
    } else {
        panic!()
    }
}

fn read(file: &str, log2_size: usize) -> Vec<u8> {
    let mut f = std::fs::File::open(format!("data/{}_{}.json", file, log2_size)).unwrap();
    let mut data = vec![];
    f.read_to_end(&mut data).unwrap();
    data
}

fn add_benchmark(c: &mut Criterion) {
    for type_ in ["number", "string", "bool"] {
        (10..=20usize).step_by(2).for_each(|log2_size| {
            let bytes = read(type_, log2_size);

            c.bench_function(&format!("{} json_parser 2^{}", type_, log2_size), |b| {
                b.iter(|| parse_json(&bytes))
            });

            c.bench_function(&format!("{} serde_json 2^{}", type_, log2_size), |b| {
                b.iter(|| parse_serde_json(&bytes))
            });
        })
    }
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);
