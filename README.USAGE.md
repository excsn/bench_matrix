# Usage Guide: `bench_matrix`

This guide provides a detailed overview of `bench_matrix`, its core concepts, and how to use its API to create and run parameterized benchmarks with the Criterion harness in Rust.

## Table of Contents

*   [Core Concepts](#core-concepts)
*   [Quick Start Examples](#quick-start-examples)
    *   [Synchronous Benchmark Example](#synchronous-benchmark-example)
    *   [Asynchronous Benchmark Example](#asynchronous-benchmark-example)
*   [Defining Parameters and Configurations](#defining-parameters-and-configurations)
    *   [`MatrixCellValue`](#matrixcellvalue)
    *   [Parameter Axes and Names](#parameter-axes-and-names)
    *   [`AbstractCombination`](#abstractcombination)
    *   [Extractor Function (`ExtractorFn`)](#extractor-function-extractorfn)
*   [Main API Sections](#main-api-sections)
    *   [Generating Parameter Combinations](#generating-parameter-combinations)
    *   [Synchronous Benchmarking (`SyncBenchmarkSuite`)](#synchronous-benchmarking-syncbenchmarksuite)
    *   [Asynchronous Benchmarking (`AsyncBenchmarkSuite`)](#asynchronous-benchmarking-asyncbenchmarksuite)
*   [Customizing Benchmark Execution](#customizing-benchmark-execution)
    *   [Providing Parameter Names for Benchmark IDs](#providing-parameter-names-for-benchmark-ids)
    *   [Global Setup and Teardown](#global-setup-and-teardown)
    *   [Customizing Criterion Groups](#customizing-criterion-groups)
    *   [Defining Throughput](#defining-throughput)
*   [Error Handling](#error-handling)

## Core Concepts

Understanding these concepts is key to effectively using `bench_matrix`:

*   **Parameter Axis:** A `Vec<MatrixCellValue>` representing all possible values for a single dimension of your benchmark configuration. For example, an axis could define different buffer sizes: `vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(128)]`.
*   **Parameter Names:** An optional `Vec<String>` where each string is a human-readable name for the corresponding parameter axis. These names are used by `bench_matrix` to generate descriptive benchmark IDs in Criterion (e.g., `MySuite/Algorithm-QuickSort_DataSize-1000`).
*   **`MatrixCellValue`:** An enum (`Tag`, `String`, `Int`, `Unsigned`, `Bool`) representing a single, discrete value within a parameter axis.
*   **`AbstractCombination`:** A struct holding a `Vec<MatrixCellValue>`, where each cell value is taken from a different parameter axis. This represents one unique configuration to be benchmarked.
*   **Configuration Extraction (`ExtractorFn`):** A user-provided function that takes an `AbstractCombination` and converts it into a concrete, strongly-typed configuration struct (`Cfg`) that your benchmark logic will consume. This is the crucial bridge between the generic framework and your specific code.
*   **Benchmark Suites (`SyncBenchmarkSuite`, `AsyncBenchmarkSuite`):** These are the main entry points for defining and running parameterized benchmarks. They create a single Criterion benchmark group and register each parameter combination as a separate, named benchmark within it.
*   **Benchmark Lifecycle Functions:** You provide these functions to the suites:
    *   **Setup Function (`setup_fn`):** Prepares the necessary state (`S`) and an optional context (`CtxT`) for a benchmark *sample* (a batch of iterations). This runs once per sample and is excluded from timing.
    *   **Benchmark Logic Function (`benchmark_logic_fn`):** Contains the actual code to be measured. It receives the `S` and `CtxT`, performs operations, and returns the updated `S`, `CtxT`, and the measured `Duration`.
    *   **Teardown Function (`teardown_fn`):** Cleans up resources after the benchmark sample. This runs once per sample and is excluded from timing.
    *   **Global Setup/Teardown Functions (`GlobalSetupFn`, `GlobalTeardownFn`):** These run once per concrete configuration (`Cfg`), bracketing all benchmark definitions for that specific configuration. They are ideal for expensive setup that can be shared across multiple samples of the same configuration.
*   **User-Defined Types (`Cfg`, `S`, `CtxT`):**
    *   `Cfg`: Your custom struct holding the specific parameters for a benchmark variant (e.g., `packet_size`, `algorithm_type`).
    *   `S` (State): Your custom struct holding the state needed for the benchmark (e.g., a data buffer, a list of connections).
    *   `CtxT` (Context): Your custom struct for carrying context across iterations within a single sample if needed (e.g., counting total operations performed).

## Quick Start Examples

### Synchronous Benchmark Example

This example benchmarks a simple data processing task with varying data sizes and processing intensities.

```rust
// In your benches/my_sync_bench.rs

use bench_matrix::{
  criterion_runner::sync_suite::SyncBenchmarkSuite,
  AbstractCombination, MatrixCellValue, SyncSetupFn, SyncBenchmarkLogicFn, SyncTeardownFn,
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::time::{Duration, Instant};

// 1. Define Configuration, State, and Context
#[derive(Debug, Clone)]
pub struct ConfigSync {
  pub data_elements: usize,
  pub intensity_level: String,
}
#[derive(Debug, Default)]
struct SyncContext { items_processed: usize }
struct SyncState { dataset: Vec<u64> }

// 2. Implement Extractor Function
fn extract_config(combo: &AbstractCombination) -> Result<ConfigSync, String> {
  Ok(ConfigSync {
    data_elements: combo.get_u64(0)? as usize, // Corresponds to "Elements"
    intensity_level: combo.get_string(1)?.to_string(), // Corresponds to "Intensity"
  })
}

// 3. Implement Lifecycle Functions
fn setup_fn(cfg: &ConfigSync) -> Result<(SyncContext, SyncState), String> {
    // Setup logic here...
    Ok((SyncContext::default(), SyncState { dataset: vec![0; cfg.data_elements] }))
}
fn benchmark_logic_fn(mut ctx: SyncContext, state: SyncState, _cfg: &ConfigSync) -> (SyncContext, SyncState, Duration) {
    let start = Instant::now();
    // Your benchmark logic...
    ctx.items_processed += state.dataset.len();
    (ctx, state, start.elapsed())
}
fn teardown_fn(_ctx: SyncContext, _state: SyncState, _cfg: &ConfigSync) {
    // Teardown logic here...
}

// 4. Define Benchmark Suite in main benchmark function
fn benchmark_sync(c: &mut Criterion) {
  let parameter_axes = vec![
    vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(1000)],
    vec![MatrixCellValue::String("Low".to_string()), MatrixCellValue::String("High".to_string())],
  ];
  let parameter_names = vec!["Elements".to_string(), "Intensity".to_string()];

  let suite = SyncBenchmarkSuite::new(
    c, "MySyncSuite".to_string(), None, parameter_axes,
    Box::new(extract_config),
    setup_fn,
    benchmark_logic_fn,
    teardown_fn,
  )
  .parameter_names(parameter_names) // Set names using the builder method
  .throughput(|cfg: &ConfigSync| Throughput::Elements(cfg.data_elements as u64));

  suite.run();
}

criterion_group!(benches, benchmark_sync);
criterion_main!(benches);
```
*Resulting benchmark ID example: `MySyncSuite/Elements-100_Intensity-High`*

### Asynchronous Benchmark Example

This example simulates an asynchronous network operation.

```rust
// In your benches/my_async_bench.rs

use bench_matrix::{
  criterion_runner::async_suite::AsyncBenchmarkSuite,
  AbstractCombination, MatrixCellValue, AsyncSetupFn, AsyncBenchmarkLogicFn, AsyncTeardownFn
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::{future::Future, pin::Pin, time::{Duration, Instant}};
use tokio::runtime::Runtime;

// 1. Define Configuration, State, and Context
#[derive(Debug, Clone)]
pub struct ConfigAsync {
  pub packet_size_bytes: u32,
  pub concurrent_ops: u16,
}
#[derive(Debug, Default)]
struct AsyncContext { ops_this_iteration: u32 }
struct AsyncState { data: Vec<u8> }

// 2. Implement Extractor Function
fn extract_config_async(combo: &AbstractCombination) -> Result<ConfigAsync, String> {
  Ok(ConfigAsync {
    packet_size_bytes: combo.get_u64(0)? as u32, // Corresponds to "PktSize"
    concurrent_ops: combo.get_u64(1)? as u16, // Corresponds to "ConcurrentOps"
  })
}

// 3. Implement Async Lifecycle Functions
fn setup_fn_async(_rt: &Runtime, cfg: &ConfigAsync) -> Pin<Box<dyn Future<Output = Result<(AsyncContext, AsyncState), String>> + Send>> {
    let cfg_clone = cfg.clone();
    Box::pin(async move {
        Ok((AsyncContext::default(), AsyncState { data: vec![0; cfg_clone.packet_size_bytes as usize] }))
    })
}
// ... (benchmark_logic_fn_async and teardown_fn_async implementation omitted)

// 4. Define Benchmark Suite
fn benchmark_async(c: &mut Criterion) {
  let rt = Runtime::new().expect("Failed to create Tokio runtime");
  let parameter_axes = vec![
    vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(512)],
    vec![MatrixCellValue::Unsigned(1), MatrixCellValue::Unsigned(4)],
  ];
  let parameter_names = vec!["PktSize".to_string(), "ConcurrentOps".to_string()];

  let suite = AsyncBenchmarkSuite::new(
    c, &rt, "MyAsyncSuite".to_string(), None, parameter_axes,
    Box::new(extract_config_async),
    setup_fn_async, benchmark_logic_fn_async, teardown_fn_async,
  )
  .parameter_names(parameter_names)
  .throughput(|cfg: &ConfigAsync| Throughput::Elements(cfg.concurrent_ops as u64));

  suite.run();
}

criterion_group!(benches, benchmark_async);
criterion_main!(benches);
```
*Resulting benchmark ID example: `MyAsyncSuite/PktSize-64_ConcurrentOps-1`*

## Defining Parameters and Configurations

### `MatrixCellValue`
An enum that represents a single value on a parameter axis.
*   **Variants:** `Tag(String)`, `String(String)`, `Int(i64)`, `Unsigned(u64)`, `Bool(bool)`.
*   **Usage:** It includes `From<T>` implementations for native types like `&'static str`, `u64`, `bool`, etc., making axis definitions more ergonomic.

### Parameter Axes and Names
You define your parameter space as a `Vec<Vec<MatrixCellValue>>`. Each inner vector is an axis. You can optionally provide a `Vec<String>` of the same length containing human-readable names for these axes.

### `AbstractCombination`
A struct containing a `Vec<MatrixCellValue>`, representing one complete benchmark variant. It's the input to your `ExtractorFn`.
*   **Key Methods:**
    *   `get_u64(index)`, `get_string(index)`, etc.: For safely extracting typed values by index.
    *   `id_suffix()` and `id_suffix_with_names()`: Used internally to create benchmark IDs.

### Extractor Function (`ExtractorFn`)
This function is your responsibility. It bridges `bench_matrix`'s generic representation to your specific code.
*   **Signature:** `Fn(&AbstractCombination) -> Result<Cfg, Err>`
*   **Purpose:** To take an `AbstractCombination` and produce your strongly-typed `Cfg` struct. You will use the `get_*` methods on the combination to access values by index.

## Main API Sections

### Generating Parameter Combinations

The `generate_combinations` function is used to create an iterator over all unique combinations from a set of parameter axes. This is called internally by the suites but is part of the public API.

*   **Signature:** `pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> CombinationIterator`
*   **Description:** Takes a slice of axes and returns a `CombinationIterator`. This iterator is **lazy**, meaning it generates combinations on the fly, making it highly memory-efficient. It also implements `ExactSizeIterator`, so you can call `.len()` to get the total number of combinations without consuming it.

### Synchronous Benchmarking (`SyncBenchmarkSuite`)

*   **Description:** Orchestrates benchmarks of synchronous code. It creates a single benchmark group and registers each parameter combination as a separate benchmark within that group.
*   **Constructor:** `pub fn new(...) -> Self`. Requires a `&mut Criterion`, a suite name, parameter axes, and the lifecycle function pointers.
*   **Key Type Aliases:** `SyncSetupFn`, `SyncBenchmarkLogicFn`, `SyncTeardownFn`.
*   **Execution:** The `pub fn run(mut self)` method consumes the suite and executes all defined benchmark combinations.

### Asynchronous Benchmarking (`AsyncBenchmarkSuite`)

*   **Description:** Orchestrates benchmarks of asynchronous code. Like the sync suite, it creates one group for all variants. It requires a reference to a `tokio::runtime::Runtime`.
*   **Constructor:** `pub fn new(...) -> Self`. Requires a `&mut Criterion`, `&Runtime`, a suite name, axes, and async lifecycle function pointers.
*   **Key Type Aliases:** These all involve `Pin<Box<dyn Future<...>>>`:
    *   `AsyncSetupFn`: Async logic to set up state for a benchmark *sample*.
    *   `AsyncBenchmarkLogicFn`: The async code to be benchmarked.
    *   `AsyncTeardownFn`: Async logic to clean up after a benchmark *sample*.
*   **Execution:** The `pub fn run(mut self)` method consumes the suite and executes the benchmarks.

## Customizing Benchmark Execution

Both `SyncBenchmarkSuite` and `AsyncBenchmarkSuite` use a builder pattern, allowing you to chain these methods after `new()`.

### Providing Parameter Names for Benchmark IDs

While `parameter_names` can be passed to the `new` constructor as `None`, it's often cleaner to use the dedicated builder method. This is the recommended approach.

*   `pub fn parameter_names(self, names: Vec<String>) -> Self` (Available on both suites)

### Global Setup and Teardown

These functions are executed once per concrete `Cfg` variant, outside of the Criterion sampling loop.

*   `pub fn global_setup(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
*   `pub fn global_teardown(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`

### Customizing Criterion Groups

This allows you to configure properties of the entire benchmark group, such as sample size, measurement time, or plot settings.

*   `pub fn configure_criterion_group(self, f: impl for<'g> Fn(&mut BenchmarkGroup<'g, WallTime>) + 'static) -> Self`
    *   Provides a closure to customize the main `criterion::BenchmarkGroup` for the entire suite. Example: `.configure_criterion_group(|group| group.sample_size(100).measurement_time(Duration::from_secs(5)))`

### Defining Throughput

Set the throughput for each benchmark variant dynamically based on its configuration.

*   `pub fn throughput(self, f: impl Fn(&Cfg) -> Throughput + 'static) -> Self`
    *   Provides a closure to calculate `criterion::Throughput` for each individual benchmark variant based on its `Cfg`. For example, `Throughput::Bytes(cfg.packet_size as u64)`.

## Error Handling

`bench_matrix` is designed to be robust, preventing a single faulty configuration from halting the entire benchmark suite.

*   **Extraction & Global Setup Failures:** If your `ExtractorFn` or `GlobalSetupFn` returns an `Err`, `bench_matrix` will print a descriptive error message to `stderr` and skip all benchmarks for that specific combination. The suite will then continue with the next combination.
*   **Per-Sample Setup Failures:** If the `setup_fn` (sync or async) called within Criterion's sampling loop returns an `Err`, the suite will `panic!` with a detailed message. This is considered a non-recoverable error for that specific benchmark variant.
*   **User Logic:** You are responsible for handling errors within your `benchmark_logic_fn` and `teardown_fn` as appropriate for your use case.