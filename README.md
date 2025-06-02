# bench_matrix

[![Crates.io](https://img.shields.io/crates/v/bench_matrix.svg)](https://crates.io/crates/bench_matrix)
[![Docs.rs](https://docs.rs/bench_matrix/badge.svg)](https://docs.rs/bench_matrix)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](LICENSE)

`bench_matrix` is a Rust utility crate designed to simplify the definition and orchestration of parameterized benchmarks, particularly when used with the Criterion benchmarking harness. It helps you systematically test your code across a wide range of configurations by automating the generation and execution of benchmark combinations.

The core problem `bench_matrix` solves is the reduction of boilerplate and manual effort required to set up benchmarks for multiple parameter sets. Instead of writing numerous, slightly varied benchmark functions, you define parameter "axes," and `bench_matrix` takes care of running your benchmark logic for every resulting combination, integrating smoothly with Criterion for robust measurement and reporting.

## Key Features

### Parameterized Benchmarking
Easily define multiple axes of parameters (e.g., different data sizes, algorithm types, concurrency levels). `bench_matrix` then generates the Cartesian product of these axes, ensuring comprehensive coverage of all specified configurations.

### Seamless Criterion Integration
Designed to work hand-in-hand with the [Criterion](https://crates.io/crates/criterion) benchmarking harness. It leverages Criterion's powerful statistical analysis, reporting, and plotting capabilities for the generated benchmark matrix.

### Synchronous & Asynchronous Support
Offers dedicated benchmark suites for both synchronous (`SyncBenchmarkSuite`) and asynchronous (`AsyncBenchmarkSuite`) code. The async suite integrates with Tokio runtimes.

### Customizable Benchmark Lifecycle
Provides hooks for user-defined functions at various stages of the benchmark lifecycle for each configuration:
*   **Setup:** Prepare state and context before measurement.
*   **Core Logic:** The actual code to be benchmarked.
*   **Teardown:** Clean up resources after measurement.
*   **Global Setup/Teardown:** Execute code once before and after all benchmarks for a specific concrete configuration.

### Type-Safe Configuration Extraction
Uses a user-provided "extractor" function (`ExtractorFn`) to convert abstract parameter combinations (`AbstractCombination`) into your own strongly-typed configuration structs. This ensures that your benchmark logic receives well-defined, type-safe configuration.

## Installation

Add `bench_matrix` to your `Cargo.toml` file:

```toml
[dev-dependencies]
bench_matrix = "0.1.0" # Replace with the latest version
criterion = "0.5"     # bench_matrix is often used with criterion
tokio = { version = "1", features = ["full"] } # If using async benchmarks
```

By default, `bench_matrix` includes Criterion integration. The core logic for parameter generation is available even without it, but its primary utility shines with Criterion. The `criterion_integration` feature is enabled by default.

## Getting Started / Documentation

For a detailed guide on how to use `bench_matrix`, including core concepts, API overview, and examples, please see the **[Usage Guide](./README.USAGE.md)**.

Example benchmark suites demonstrating various features can be found in the `benches/` directory of the project (e.g., `benches/async.rs`, `benches/sync.rs`).

For the full API reference, please visit [docs.rs/bench_matrix](https://docs.rs/bench_matrix/latest/bench_matrix/).