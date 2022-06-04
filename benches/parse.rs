use std::io::Read;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

fn parse_serde_json(data: &[u8]) {
    let a: serde_json::Value = serde_json::from_slice(data).unwrap();
    assert!(a.is_array());
}

fn parse_simd_json(data: &mut [u8]) {
    use simd_json::Value;
    let a = simd_json::to_borrowed_value(data).unwrap();
    assert!(a.is_array());
}

fn parse_json(data: &[u8]) {
    let a = json_deserializer::parse(data).unwrap();
    if let json_deserializer::Value::Array(_) = a {
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
    for type_ in [
        "integer",
        "float",
        "string",
        "bool",
        "object_string",
        "object_bool",
    ] {
        (10..=20usize).step_by(2).for_each(|log2_size| {
            let bytes = read(type_, log2_size);

            c.bench_function(
                &format!("{} json_deserializer 2^{}", type_, log2_size),
                |b| b.iter(|| parse_json(&bytes)),
            );

            c.bench_function(&format!("{} serde_json 2^{}", type_, log2_size), |b| {
                b.iter(|| parse_serde_json(&bytes))
            });

            let bytes = bytes.clone();
            c.bench_function(&format!("{} simd_json 2^{}", type_, log2_size), move |b| {
                b.iter_batched(
                    || bytes.clone(),
                    |mut data| parse_simd_json(&mut data),
                    BatchSize::SmallInput,
                )
            });
        })
    }
}

criterion_group!(benches, add_benchmark);
criterion_main!(benches);
