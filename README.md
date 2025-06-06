# bench_matrix

[![Crates.io](https://img.shields.io/crates/v/bench_matrix.svg)](https://crates.io/crates/bench_matrix)
[![Docs.rs](https://docs.rs/bench_matrix/badge.svg)](https://docs.rs/bench_matrix)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](LICENSE)

`bench_matrix` is a Rust utility crate that supercharges your parameterized benchmarks. It provides a powerful and ergonomic framework for running benchmarks across a complex matrix of configurations, integrating seamlessly with the [Criterion](https://crates.io/crates/criterion) harness.

Stop writing repetitive benchmark functions. Define your parameter axes once, and let `bench_matrix` handle the rest, generating a full suite of benchmarks with clean, hierarchical reporting.

## Why use `bench_matrix`?

*   **Eliminate Boilerplate:** Define your parameters (e.g., data sizes, algorithms, concurrency levels) in one place. `bench_matrix` generates the Cartesian product, ensuring every combination is tested without repetitive code.
*   **Memory Efficient:** Lazily generates benchmark combinations on the fly. You can define a test matrix with millions of variants without consuming gigabytes of memory upfront.
*   **Clean, Hierarchical Reports:** Automatically creates well-named Criterion groups, leading to organized and readable benchmark results (e.g., `MySuite/Algorithm-QuickSort_DataSize-1000`).
*   **Seamless Criterion Integration:** Built from the ground up to work with Criterion, leveraging its powerful statistical analysis and plotting features.
*   **Async & Sync Ready:** Provides dedicated, consistent APIs for both synchronous (`SyncBenchmarkSuite`) and asynchronous (`AsyncBenchmarkSuite`) code.
*   **Type-Safe & Customizable:** Use your own strongly-typed configuration structs and hook into a flexible lifecycle with `setup`, `teardown`, and `global_setup` functions.

## A Quick Look

Here's how you can set up a benchmark for a function across multiple data sizes and processing intensities:

```rust
// In benches/my_bench.rs
use bench_matrix::{criterion_runner::sync_suite::SyncBenchmarkSuite, MatrixCellValue};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn my_benchmark_function(c: &mut Criterion) {
    let parameter_axes = vec![
        // Axis 1: Number of data elements
        vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(1000)],
        // Axis 2: Processing intensity
        vec![MatrixCellValue::String("Low".to_string()), MatrixCellValue::String("High".to_string())],
    ];

    let parameter_names = vec!["Elements".to_string(), "Intensity".to_string()];

    // Define your config struct, state, extractor, and lifecycle functions...
    // (See the Usage Guide for full details)

    let suite = SyncBenchmarkSuite::new(
        c, "DataProcessingSuite".to_string(), None, parameter_axes,
        Box::new(my_extractor_fn),
        my_setup_fn,
        my_logic_fn,
        my_teardown_fn,
    )
    .parameter_names(parameter_names)
    .throughput(|cfg: &MyConfig| Throughput::Elements(cfg.data_elements as u64));

    suite.run();
}

criterion_group!(benches, my_benchmark_function);
criterion_main!(benches);
```
This will produce benchmark results like:
*   `DataProcessingSuite/Elements-100_Intensity-Low`
*   `DataProcessingSuite/Elements-100_Intensity-High`
*   `DataProcessingSuite/Elements-1000_Intensity-Low`
*   `DataProcessingSuite/Elements-1000_Intensity-High`

## Installation

Add `bench_matrix` and its companions to the `[dev-dependencies]` section of your `Cargo.toml`:

```toml
[dev-dependencies]
bench_matrix = "0.2.0" # Replace with the latest version
criterion = "0.5"
tokio = { version = "1", features = ["full"] } # Required for async benchmarks
```

The `criterion_integration` feature is enabled by default.

## Documentation

*   **[Usage Guide](./README.USAGE.md):** A comprehensive guide on concepts, API, and examples. **Start here!**
*   **[API Reference (docs.rs):](https://docs.rs/bench_matrix/latest/bench_matrix/)** Detailed documentation for every public type and function.
*   **[Examples (`benches/` directory):](https://github.com/excsn/bench_matrix/tree/main/benches)** Fully working examples demonstrating synchronous and asynchronous suites.