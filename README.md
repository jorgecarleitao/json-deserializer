[![test](https://github.com/jorgecarleitao/json-deserializer/actions/workflows/test.yml/badge.svg)](https://github.com/jorgecarleitao/json-deserializer/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/jorgecarleitao/json-deserializer/branch/main/graph/badge.svg?token=AgyTF60R3D)](https://codecov.io/gh/jorgecarleitao/json-deserializer)

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
bool json_deserializer 2^20   time:   [26.022 ms 26.056 ms 26.090 ms]
bool serde_json 2^20          time:   [30.419 ms 30.468 ms 30.516 ms]
bool simd_json 2^20           time:   [31.440 ms 31.486 ms 31.531 ms] 
```

### Array of strings
```
string json_deserializer 2^18 time:   [10.106 ms 10.138 ms 10.173 ms]
string serde_json 2^18        time:   [23.177 ms 23.209 ms 23.243 ms]
string simd_json 2^18         time:   [10.924 ms 10.941 ms 10.959 ms]

# with `RUSTFLAGS='-C target-cpu=native'` (skilake in this case)
string simd_json 2^18         time:   [8.0735 ms 8.0887 ms 8.1046 ms]
```

### Array of an object with a string
```
object_string json_deserializer 2^14
                        time:   [2.7631 ms 2.7681 ms 2.7736 ms]
object_string serde_json 2^14
                        time:   [4.3729 ms 4.3823 ms 4.3922 ms]
object_string simd_json 2^14
                        time:   [2.6313 ms 2.6357 ms 2.6401 ms]
```

### Array of an object with a bool

```
object_bool json_deserializer 2^10
                        time:   [144.14 us 144.35 us 144.62 us]
object_bool serde_json 2^10
                        time:   [197.12 us 197.62 us 198.31 us]
object_bool simd_json 2^10
                        time:   [160.87 us 161.33 us 161.77 us]
```
