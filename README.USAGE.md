# Usage Guide: bench_matrix

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
    *   [Providing Parameter Names for Group IDs](#providing-parameter-names-for-group-ids)
    *   [Global Setup and Teardown](#global-setup-and-teardown)
    *   [Customizing Criterion Groups](#customizing-criterion-groups)
    *   [Defining Throughput](#defining-throughput)
*   [Error Handling](#error-handling)

## Core Concepts

Understanding these concepts is key to effectively using `bench_matrix`:

*   **Parameter Axis:** A `Vec<MatrixCellValue>` representing all possible values for a single dimension of your benchmark configuration. For example, an axis could define different buffer sizes: `vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(128)]`.
*   **Parameter Names:** An optional `Vec<String>` where each string is a human-readable name for the corresponding parameter axis. These names are used by `bench_matrix` to generate more descriptive benchmark group IDs in Criterion (e.g., `_ParamName1-Value1_ParamName2-Value2`).
*   **`MatrixCellValue`:** An enum (`Tag(String)`, `String(String)`, `Int(i64)`, `Unsigned(u64)`, `Bool(bool)`) representing a single, discrete value within a parameter axis.
*   **`AbstractCombination`:** A struct holding a `Vec<MatrixCellValue>`, where each cell value is taken from a different parameter axis. This represents one unique configuration to be benchmarked (one "row" in your parameter matrix).
*   **Configuration Extraction (`ExtractorFn`):** A user-provided function `Box<dyn Fn(&AbstractCombination) -> Result<Cfg, ExtErr>>`. Its role is to take an `AbstractCombination` and convert it into a concrete, strongly-typed configuration struct (`Cfg`) that your benchmark logic will consume. `ExtErr` is a user-defined error type, often `String`.
*   **Benchmark Suites (`SyncBenchmarkSuite`, `AsyncBenchmarkSuite`):** These are the main entry points for defining and running parameterized benchmarks. They orchestrate the generation of combinations, configuration extraction, setup, execution of benchmark logic via Criterion, and teardown.
*   **Benchmark Lifecycle Functions:** You provide these functions to the suites:
    *   **Setup Function:** Prepares the necessary state (`S`) and an optional context (`CtxT`) for a benchmark iteration (async) or a batch of iterations (sync).
    *   **Benchmark Logic Function:** Contains the actual code to be measured. It receives the `S` and `CtxT`, performs operations, and returns the updated `S`, `CtxT`, and the measured `Duration`.
    *   **Teardown Function:** Cleans up resources after the benchmark logic.
    *   **Global Setup/Teardown Functions (`GlobalSetupFn`, `GlobalTeardownFn`):** These run once per concrete configuration (`Cfg`), bracketing all benchmark iterations for that specific configuration.
*   **User-Defined Types (`Cfg`, `S`, `CtxT`):**
    *   `Cfg`: Your custom struct holding the specific parameters for a benchmark variant (e.g., packet size, algorithm choice).
    *   `S`: Your custom struct holding the state needed for the benchmark (e.g., a data buffer, a network connection simulator).
    *   `CtxT`: Your custom struct for carrying context or accumulators across iterations if needed (e.g., counting operations).

## Quick Start Examples

These examples demonstrate the basic structure of using `SyncBenchmarkSuite` and `AsyncBenchmarkSuite`, including providing parameter names.

### Synchronous Benchmark Example

This example benchmarks a simple data processing task with varying data sizes and processing intensities, now with named parameters for clearer group IDs.

```rust
// In your benches/my_sync_bench.rs

use bench_matrix::{
  criterion_runner::{
    sync_suite::{SyncBenchmarkLogicFn, SyncBenchmarkSuite, SyncSetupFn, SyncTeardownFn},
    ExtractorFn,
  },
  AbstractCombination, MatrixCellValue,
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::time::{Duration, Instant};

// 1. Define Configuration
#[derive(Debug, Clone)]
pub struct ConfigSync {
  pub data_elements: usize,
  pub intensity_level: String, // Changed name for clarity
}

// 2. Define State and Context (optional context)
#[derive(Debug, Default)]
struct SyncContext {
  items_processed: usize,
}
struct SyncState {
  dataset: Vec<u64>,
}

// 3. Implement Extractor Function
//    (Assumes parameter_names are provided to the suite for group ID naming)
fn extract_config(combo: &AbstractCombination) -> Result<ConfigSync, String> {
  Ok(ConfigSync {
    // First axis (index 0) corresponds to "Elements" name
    data_elements: combo.get_u64(0)? as usize,
    // Second axis (index 1) corresponds to "Intensity" name
    intensity_level: combo.get_string(1)?.to_string(),
  })
}

// 4. Implement Setup Function
fn setup_fn(cfg: &ConfigSync) -> Result<(SyncContext, SyncState), String> {
  let dataset = (0..cfg.data_elements).map(|i| i as u64).collect();
  Ok((SyncContext::default(), SyncState { dataset }))
}

// 5. Implement Benchmark Logic
fn benchmark_logic_fn(
  mut ctx: SyncContext,
  state: SyncState,
  cfg: &ConfigSync,
) -> (SyncContext, SyncState, Duration) {
  let start_time = Instant::now();
  let mut sum = 0;
  let multiplier = if cfg.intensity_level == "High" { 10 } else { 1 }; // Use intensity_level
  for &val in &state.dataset {
    for _ in 0..multiplier {
      sum = sum.wrapping_add(val);
    }
  }
  if sum == u64::MAX { println!("Overflow"); } // Pretend sum is used
  let duration = start_time.elapsed();
  ctx.items_processed += state.dataset.len();
  (ctx, state, duration)
}

// 6. Implement Teardown Function
fn teardown_fn(_ctx: SyncContext, _state: SyncState, _cfg: &ConfigSync) { /* ... */ }

// 7. Define Benchmark Suite
fn benchmark_sync(c: &mut Criterion) {
  let parameter_axes = vec![
    vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(1000)], // For "Elements"
    vec![MatrixCellValue::String("Low".to_string()), MatrixCellValue::String("High".to_string())], // For "Intensity"
  ];

  let parameter_names = vec![
    "Elements".to_string(),
    "Intensity".to_string(),
  ];

  let suite = SyncBenchmarkSuite::new(
    c,
    "MySyncSuite".to_string(),
    Some(parameter_names), // Provide parameter names
    parameter_axes,
    Box::new(extract_config),
    setup_fn,
    benchmark_logic_fn,
    teardown_fn,
  )
  .throughput(|cfg: &ConfigSync| Throughput::Elements(cfg.data_elements as u64));

  suite.run();
}

criterion_group!(benches, benchmark_sync);
criterion_main!(benches);
```
*Resulting group ID example: `MySyncSuite_Elements-100_Intensity-Low`*

### Asynchronous Benchmark Example

This example simulates an asynchronous network operation, now with named parameters.

```rust
// In your benches/my_async_bench.rs

use bench_matrix::{
  criterion_runner::{
    async_suite::{AsyncBenchmarkLogicFn, AsyncBenchmarkSuite, AsyncSetupFn, AsyncTeardownFn},
    ExtractorFn,
  },
  AbstractCombination, MatrixCellValue,
};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::{future::Future, pin::Pin, time::{Duration, Instant}};
use tokio::runtime::Runtime;

// 1. Define Configuration
#[derive(Debug, Clone)]
pub struct ConfigAsync {
  pub packet_size_bytes: u32, // Renamed for clarity
  pub concurrent_operations: u16, // Renamed for clarity
}

// 2. Define State and Context
#[derive(Default)]
struct AsyncContext { ops_this_iteration: u32 }
struct AsyncState { data_packet: Vec<u8> }

// 3. Implement Extractor Function
fn extract_config_async(combo: &AbstractCombination) -> Result<ConfigAsync, String> {
  Ok(ConfigAsync {
    // First axis (index 0) corresponds to "PktSize"
    packet_size_bytes: combo.get_u64(0)? as u32,
    // Second axis (index 1) corresponds to "ConcurrentOps"
    concurrent_operations: combo.get_u64(1)? as u16,
  })
}

// 4. Implement Async Setup Function
fn setup_fn_async(
  _runtime: &Runtime,
  cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = Result<(AsyncContext, AsyncState), String>> + Send>> {
  let cfg_clone = cfg.clone();
  Box::pin(async move {
    let data_packet = vec![0u8; cfg_clone.packet_size_bytes as usize];
    Ok((AsyncContext::default(), AsyncState { data_packet }))
  })
}

// 5. Implement Async Benchmark Logic
fn benchmark_logic_fn_async(
  mut ctx: AsyncContext,
  state: AsyncState,
  cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = (AsyncContext, AsyncState, Duration)> + Send>> {
  let concurrent_ops_count = cfg.concurrent_operations; // Use renamed field
  Box::pin(async move {
    let start_time = Instant::now();
    let mut tasks = Vec::new();
    for _ in 0..concurrent_ops_count { // Use renamed field
      let packet_clone = state.data_packet.clone();
      tasks.push(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_micros(10)).await;
        let _checksum = packet_clone.iter().sum::<u8>();
      }));
    }
    for task in tasks { task.await.unwrap(); }
    let duration = start_time.elapsed();
    ctx.ops_this_iteration += concurrent_ops_count as u32;
    (ctx, state, duration)
  })
}

// 6. Implement Async Teardown Function
fn teardown_fn_async(
  _ctx: AsyncContext, _state: AsyncState, _runtime: &Runtime, _cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
  Box::pin(async move { /* Async cleanup */ })
}

// 7. Define Benchmark Suite
fn benchmark_async(c: &mut Criterion) {
  let rt = Runtime::new().expect("Failed to create Tokio runtime");

  let parameter_axes = vec![
    vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(512)], // For "PktSize"
    vec![MatrixCellValue::Unsigned(1), MatrixCellValue::Unsigned(4)],   // For "ConcurrentOps"
  ];

  let parameter_names = vec![
      "PktSize".to_string(),
      "ConcurrentOps".to_string(),
  ];

  let suite = AsyncBenchmarkSuite::new(
    c,
    &rt,
    "MyAsyncSuite".to_string(),
    Some(parameter_names), // Provide parameter names
    parameter_axes,
    Box::new(extract_config_async),
    setup_fn_async,
    benchmark_logic_fn_async,
    teardown_fn_async,
  )
  .throughput(|cfg: &ConfigAsync| Throughput::Elements(cfg.concurrent_operations as u64));

  suite.run();
}

criterion_group!(benches, benchmark_async);
criterion_main!(benches);
```
*Resulting group ID example: `MyAsyncSuite_PktSize-64_ConcurrentOps-1`*

## Defining Parameters and Configurations

Configuration in `bench_matrix` revolves around defining parameter axes with `MatrixCellValue` along with optional names for these axes. These are then combined into `AbstractCombination` instances, and the user-provided `ExtractorFn` translates them into concrete, typed configuration structs.

### `MatrixCellValue`

An enum representing a single value for a parameter.
`pub enum MatrixCellValue`

*   **Variants:**
    *   `Tag(String)`: A semantic tag or identifier (e.g., `"AlgorithmA"`).
    *   `String(String)`: A general-purpose string value.
    *   `Int(i64)`: A signed integer value.
    *   `Unsigned(u64)`: An unsigned integer value.
    *   `Bool(bool)`: A boolean value.

### Parameter Axes and Names

*   **Parameter Axes (`Vec<Vec<MatrixCellValue>>`):** A list where each inner vector represents one dimension of parameters. Each element in the inner vector is a `MatrixCellValue` representing a possible value for that parameter.
*   **Parameter Names (`Option<Vec<String>>`):** An optional list of strings, where each string names the corresponding parameter axis. If provided, these names are used to create more descriptive benchmark group IDs (e.g., `_ParamName1-Value1_ParamName2-Value2`). The length of this vector must match the length of `parameter_axes`.

Example:
```rust
use bench_matrix::MatrixCellValue;

// Define axes
let algorithms_axis = vec![
    MatrixCellValue::Tag("QuickSort".to_string()),
    MatrixCellValue::Tag("MergeSort".to_string()),
];
let data_sizes_axis = vec![
    MatrixCellValue::Unsigned(100),
    MatrixCellValue::Unsigned(1000),
];
let parameter_axes = vec![algorithms_axis, data_sizes_axis];

// Define corresponding names
let parameter_names = Some(vec![
    "Algorithm".to_string(),
    "DataSize".to_string(),
]);

// These are then passed to the benchmark suite constructor.
```

### `AbstractCombination`

Represents one specific combination of `MatrixCellValue`s.
`pub struct AbstractCombination { pub cells: Vec<MatrixCellValue> }`

It provides helper methods to extract typed values by index:
*   `pub fn get_tag(&self, index: usize) -> Result<&str, String>`
*   `pub fn get_string(&self, index: usize) -> Result<&str, String>`
*   `pub fn get_i64(&self, index: usize) -> Result<i64, String>`
*   `pub fn get_u64(&self, index: usize) -> Result<u64, String>`
*   `pub fn get_bool(&self, index: usize) -> Result<bool, String>`

The struct also has methods `id_suffix()` and `id_suffix_with_names(&[String])` used internally by the suites to generate parts of the benchmark group ID.

### Extractor Function (`ExtractorFn`)

This is a crucial user-defined function that bridges `AbstractCombination` to your specific configuration struct.

`pub type ExtractorFn<Cfg, ExtErr = String> = Box<dyn Fn(&AbstractCombination) -> Result<Cfg, ExtErr>>;`

Example:
```rust
use bench_matrix::{AbstractCombination, ExtractorFn};

#[derive(Debug, Clone)] // Your concrete config struct
pub struct MyConfig {
    pub algorithm_name: String, // Field in your config
    pub item_count: usize,      // Another field
}

fn my_extractor(combo: &AbstractCombination) -> Result<MyConfig, String> {
    // Assuming the first axis (index 0) was named "Algorithm" and contains Tags
    let algorithm_name = combo.get_tag(0)?.to_string();
    // Assuming the second axis (index 1) was named "ItemCount" and contains Unsigned
    let item_count = combo.get_u64(1)? as usize;

    Ok(MyConfig { algorithm_name, item_count })
}

// Usage when creating a suite:
// Box::new(my_extractor)
```

## Main API Sections

### Generating Parameter Combinations

The `generate_combinations` function is used to create all unique combinations from a set of parameter axes. This is typically called internally by the benchmark suites but can be used independently.

*   **Signature:** `pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> Vec<AbstractCombination>`
*   **Description:** Takes a slice of axes (each axis is a `Vec<MatrixCellValue>`) and returns a `Vec<AbstractCombination>` representing the Cartesian product.

### Synchronous Benchmarking (`SyncBenchmarkSuite`)

Used for orchestrating benchmarks of synchronous code.

*   **Primary Struct:** `pub struct SyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
    *   `S`: Your benchmark state type.
    *   `Cfg`: Your concrete configuration type.
    *   `CtxT`: Your benchmark context type.
*   **Constructor:**
    `pub fn new(criterion: &'s mut Criterion<WallTime>, suite_base_name: String, parameter_names: Option<Vec<String>>, parameter_axes: Vec<Vec<MatrixCellValue>>, extractor_fn: ExtractorFn<Cfg, ExtErr>, setup_fn: SyncSetupFn<S, Cfg, CtxT, SetupErr>, benchmark_logic_fn: SyncBenchmarkLogicFn<S, Cfg, CtxT>, teardown_fn: SyncTeardownFn<S, Cfg, CtxT>) -> Self`
    *   Creates a new synchronous benchmark suite.
    *   `parameter_names`: Optional names for each axis, used for group ID generation.
*   **Key Type Aliases for Callbacks:**
    *   `pub type SyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&Cfg) -> Result<(CtxT, S), SetupErr>;`
        *   Logic to set up state and context for a batch of benchmark iterations.
    *   `pub type SyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> (CtxT, S, Duration);`
        *   The synchronous code to be benchmarked. Returns updated context, state, and measured duration.
    *   `pub type SyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> ();`
        *   Logic to clean up after a batch of iterations.
*   **Execution:**
    *   `pub fn run(mut self)`
        *   Runs all benchmark combinations through Criterion.

### Asynchronous Benchmarking (`AsyncBenchmarkSuite`)

Used for orchestrating benchmarks of asynchronous code, typically with Tokio.

*   **Primary Struct:** `pub struct AsyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
    *   `S`: Your benchmark state type.
    *   `Cfg`: Your concrete configuration type.
    *   `CtxT`: Your benchmark context type.
*   **Constructor:**
    `pub fn new(criterion: &'s mut Criterion<WallTime>, runtime: &'s Runtime, suite_base_name: String, parameter_names: Option<Vec<String>>, parameter_axes: Vec<Vec<MatrixCellValue>>, extractor_fn: ExtractorFn<Cfg, ExtErr>, setup_fn: AsyncSetupFn<S, Cfg, CtxT, SetupErr>, benchmark_logic_fn: AsyncBenchmarkLogicFn<S, Cfg, CtxT>, teardown_fn: AsyncTeardownFn<S, Cfg, CtxT>) -> Self`
    *   Creates a new asynchronous benchmark suite. Requires a Tokio `Runtime`.
    *   `parameter_names`: Optional names for each axis, used for group ID generation.
*   **Key Type Aliases for Callbacks:**
    *   `pub type AsyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&Runtime, &Cfg) -> Pin<Box<dyn Future<Output = Result<(CtxT, S), SetupErr>> + Send>>;`
        *   Async logic to set up state and context for each benchmark iteration.
    *   `pub type AsyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> Pin<Box<dyn Future<Output = (CtxT, S, Duration)> + Send>>;`
        *   The asynchronous code to be benchmarked. Returns updated context, state, and measured duration.
    *   `pub type AsyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &Runtime, &Cfg) -> Pin<Box<dyn Future<Output = ()> + Send>>;`
        *   Async logic to clean up after each benchmark iteration.
*   **Execution:**
    *   `pub fn run(mut self)`
        *   Runs all benchmark combinations through Criterion.

## Customizing Benchmark Execution

Both `SyncBenchmarkSuite` and `AsyncBenchmarkSuite` offer builder-style methods for further customization:

### Providing Parameter Names for Group IDs

While `parameter_names` can be passed to the `new` constructor, you can also set or override them using a builder method:

*   `pub fn parameter_names(self, names: Vec<String>) -> Self` (Available on both suites)
    *   Sets the names for parameter axes. If `names.len()` does not match the number of defined axes, these names will be ignored for ID generation, and a warning will be printed.

### Global Setup and Teardown

These functions run once per concrete configuration (`Cfg`), bracketing all Criterion iterations for that specific `Cfg`.

*   `pub fn global_setup(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
    *   Sets a function to run before any benchmarks for a given `Cfg`.
*   `pub fn global_teardown(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
    *   Sets a function to run after all benchmarks for a given `Cfg`.

**Type Aliases:**
*   `pub type GlobalSetupFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;`
*   `pub type GlobalTeardownFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;`

### Customizing Criterion Groups

Allows direct configuration of the `criterion::BenchmarkGroup` for each parameter combination.

*   `pub fn configure_criterion_group(self, f: impl for<'g> Fn(&mut BenchmarkGroup<'g, WallTime>) + 'static) -> Self`
    *   Example usage: `.configure_criterion_group(|group| group.sample_size(100).measurement_time(Duration::from_secs(5)))`

### Defining Throughput

Allows specifying throughput for Criterion, which can be based on the concrete configuration.

*   `pub fn throughput(self, f: impl Fn(&Cfg) -> Throughput + 'static) -> Self`
    *   Example usage: `.throughput(|config: &MyConfig| Throughput::Bytes(config.item_count as u64))`

## Error Handling

`bench_matrix` handles errors from user-provided functions in the following ways:

*   **`ExtractorFn` and `GlobalSetupFn` Errors:** If the `ExtractorFn` (for converting `AbstractCombination` to `Cfg`) or the `GlobalSetupFn` fails for a particular combination, that combination and its associated benchmarks will be skipped. An error message is printed to `stderr`.
*   **`SetupFn` (within Criterion loop) Errors:** If the `SyncSetupFn` or `AsyncSetupFn` (which are called by Criterion during its sampling process) return an `Err`, `bench_matrix` will `panic!`. This is because Criterion expects setup within its iteration logic to succeed to ensure valid measurements.
*   **`GlobalTeardownFn` Errors:** Failures in `GlobalTeardownFn` are reported as warnings to `stderr` but do not stop other benchmarks.
*   **Parameter Name Mismatch:** If `parameter_names` are provided but their count doesn't match the number of `parameter_axes`, a warning is printed, and the names are ignored for ID generation (falling back to default ID suffixes).
*   **Error Types:** User-provided functions typically return `Result<T, String>` or `Result<T, UserDefinedError>`. The default error type for `ExtractorFn` and `SetupFn` generic parameters is `String`.

The `benchmark_logic_fn` and `teardown_fn` (the ones called repeatedly by Criterion) do not directly return `Result` to `bench_matrix`. Any panics within these will be handled by Criterion as usual.