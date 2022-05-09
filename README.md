# Rust native JSON deserializer

This repository contains a performant Rust implementation to parse JSON by reference.

## Why not `serde-json`?

`serde-json` is both a JSON parser and data model based on serde's model (`Value`).
`Value` is an owning data structure.

In many use cases, JSON can be parsed into references. In particular, ownership of strings
is only required when the JSON string contains non-ascii characters.

There is a performance oportunity if JSON is parsed by reference instead of by value when possible.

This crate fills this gap. When parsing e.g. a list of strings, this crate
is ~2x faster than `serde-json` (see below).

## Safety

This crate is `#![forbid(unsafe_code)]` and only panics on failed allocations.

### Benches

Run

```bash
python3 write_bench_files.py && cargo bench --bench parse
```

for a comparison with `serde_json`. Broadly speaking, this crate is either faster or equally fast.
Some examples:

### Array of bools
```
bool json_deserializer 2^20   time:   [26.067 ms 26.114 ms 26.172 ms]
bool serde_json 2^20    time:   [29.350 ms 29.405 ms 29.459 ms]
```

### Array of strings:
```
string json_deserializer 2^18 time:   [10.106 ms 10.138 ms 10.173 ms]
string serde_json 2^18  time:   [24.352 ms 24.470 ms 24.616 ms]
```

### Array of an object with a string
```
object_string json_deserializer 2^14
                        time:   [3.1260 ms 3.1408 ms 3.1588 ms]
object_string serde_json 2^14
                        time:   [4.3516 ms 4.3628 ms 4.3754 ms]
```

### Array of an object with a bool

```
object_bool json_deserializer 2^10
                        time:   [165.67 us 166.27 us 166.99 us]
object_bool serde_json 2^10
                        time:   [199.96 us 200.71 us 201.56 us]
```
