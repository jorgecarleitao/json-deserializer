# Rust native JSON parser

This repository contains an implementation to parse JSON.

## Why not serde-json?

Serde-json is both a JSON parser and data model (`Value`).
This implies that it parses JSON by value, making it quite expensive when
we do not intent to use the values directly but just pass them through.

Two important examples of this requirement are: 
* writing JSON data to storage formats such as Apache Parquet (writing directly)
* reading JSON into Apache Arrow, an in-memory columnar format (e.g. not backed by `String`)

## Safety

This crate is `#![forbid(unsafe_code)]` only panics on failed allocations.

### Benches

Run

```bash
python3 write_bench_files.py && cargo bench
```

for a comparison with `serde_json`.
