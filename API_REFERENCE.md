# `bench_matrix` API Reference

## 1. Introduction / Core Concepts

`bench_matrix` is a Rust utility crate designed to simplify the creation and execution of parameterized benchmarks, especially when using the Criterion benchmarking harness. It allows you to define a matrix of input parameters and automatically run your benchmark logic for every combination of these parameters.

**Core Concepts:**

*   **Parameter Axis:** A list of possible values for a single dimension of your benchmark configuration. For example, an axis could represent different buffer sizes `[64, 128, 256]` or different algorithms `["AlgorithmA", "AlgorithmB"]`.
*   **`MatrixCellValue`:** An enum representing a single value within a parameter axis (e.g., a specific tag, string, integer, or boolean).
*   **`AbstractCombination`:** A struct representing one unique combination of `MatrixCellValue`s, one from each defined axis. This forms a single "row" in your conceptual parameter matrix.
*   **Configuration Extraction (`ExtractorFn`):** A user-provided function that takes an `AbstractCombination` and translates it into a concrete, strongly-typed configuration struct (defined by the user) that the benchmark logic will use.
*   **Benchmark Suites (`SyncBenchmarkSuite`, `AsyncBenchmarkSuite`):** The primary structures for defining and running a set of parameterized benchmarks. They manage the generation of combinations, configuration extraction, and integration with Criterion.
    *   `SyncBenchmarkSuite`: For benchmarking synchronous code.
    *   `AsyncBenchmarkSuite`: For benchmarking asynchronous code, typically with a Tokio runtime.
*   **Benchmark Lifecycle Functions:** Users provide several callback functions to the suites to define the benchmark behavior:
    *   **Setup:** Prepares the necessary state and context for a benchmark iteration or batch.
    *   **Logic:** Contains the actual code to be measured.
    *   **Teardown:** Cleans up resources after an iteration or batch.
    *   **Global Setup/Teardown:** Run once per concrete configuration, before and after all benchmark iterations for that configuration.

**Main Entry Points:**

The primary way to interact with `bench_matrix` is by:

1.  Defining your parameter axes using `Vec<Vec<MatrixCellValue>>`.
2.  Creating an instance of either `SyncBenchmarkSuite` or `AsyncBenchmarkSuite` using their `new` methods.
3.  Providing the necessary callback functions (extractor, setup, logic, teardown) to the suite.
4.  Optionally configuring the suite further using its builder methods (e.g., for global setup/teardown, Criterion group configuration).
5.  Calling the `run()` method on the suite instance within your Criterion benchmark functions.

**Pervasive Types and Patterns:**

*   **`MatrixCellValue` and `AbstractCombination`:** These are fundamental for defining and working with parameter combinations.
*   **Function Pointers/Closures:** The library extensively uses function pointers (e.g., `fn(...) -> ...`) and `Box<dyn Fn(...)>` for user-provided callbacks. This allows for flexible and type-safe customization of the benchmarking process.
*   **User-Defined Configuration (`Cfg`), State (`S`), and Context (`CtxT`):** The benchmark suites are generic over these types, which are defined by the user to match the specific needs of their benchmarks.
    *   `Cfg`: Your concrete configuration struct derived from an `AbstractCombination`.
    *   `S`: The state that is set up before benchmarking and potentially modified by it.
    *   `CtxT`: An optional context that can be passed through benchmark iterations.
*   **Error Handling:** User-provided functions like the `ExtractorFn` or `GlobalSetupFn` typically return `Result<T, UserErrorType>` where `UserErrorType` is often `String` for simplicity or a custom error enum defined by the user. The suites themselves will report these errors and may skip benchmark variants accordingly.

## 2. Modules and Public API

This reference is organized by the public modules of the `bench_matrix` crate.

---

### Crate Root (`bench_matrix`)

These are items re-exported at the top level of the `bench_matrix` crate for convenience.

**Type Aliases (Common for Criterion Integration):**

*   `pub type ExtractorFn<Cfg, ExtErr = String> = Box<dyn Fn(&AbstractCombination) -> Result<Cfg, ExtErr>>;`
    *   A function to extract/resolve a user-defined concrete configuration (`Cfg`) from an `AbstractCombination`.
    *   `Cfg`: The user's concrete configuration struct.
    *   `ExtErr`: User-defined error type for the extractor function (defaults to `String`).
*   `pub type GlobalSetupFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;`
    *   Function to perform global setup before a Criterion benchmark group for a specific `Cfg` begins.
*   `pub type GlobalTeardownFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;`
    *   Function to perform global teardown after a Criterion benchmark group for a specific `Cfg` has completed.

**Type Aliases (Async Suite Specific):**

*   `pub type AsyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&tokio::runtime::Runtime, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(CtxT, S), SetupErr>> + Send>>;`
    *   Function to set up state (`S`) and context (`CtxT`) for an asynchronous benchmark iteration.
    *   `S`: User-defined state type.
    *   `Cfg`: User-defined configuration type.
    *   `CtxT`: User-defined context type.
    *   `SetupErr`: User-defined error type for setup (defaults to `String`).
*   `pub type AsyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = (CtxT, S, std::time::Duration)> + Send>>;`
    *   Function containing the asynchronous benchmark logic to be measured.
*   `pub type AsyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &tokio::runtime::Runtime, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;`
    *   Function to tear down resources after an asynchronous benchmark iteration.

**Type Aliases (Sync Suite Specific):**

*   `pub type SyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&Cfg) -> Result<(CtxT, S), SetupErr>;`
    *   Function to set up state (`S`) and context (`CtxT`) for a synchronous benchmark batch.
    *   `S`: User-defined state type.
    *   `Cfg`: User-defined configuration type.
    *   `CtxT`: User-defined context type.
    *   `SetupErr`: User-defined error type for setup (defaults to `String`).
*   `pub type SyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> (CtxT, S, std::time::Duration);`
    *   Function containing the synchronous benchmark logic to be measured.
*   `pub type SyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> ();`
    *   Function to tear down resources after a synchronous benchmark batch.

**Structs (Async Suite Specific):**

*   `pub use criterion_runner::async_suite::AsyncBenchmarkSuite;` (See `bench_matrix::criterion_runner::async_suite` module for details)

**Structs (Sync Suite Specific):**

*   `pub use criterion_runner::sync_suite::SyncBenchmarkSuite;` (See `bench_matrix::criterion_runner::sync_suite` module for details)

**Structs (Parameter Definition):**

*   `pub use params::AbstractCombination;` (See `bench_matrix::params` module for details)

**Enums (Parameter Definition):**

*   `pub use params::MatrixCellValue;` (See `bench_matrix::params` module for details)

**Functions (Combination Generation):**

*   `pub use generator::generate_combinations;` (See `bench_matrix::generator` module for details)

---

### Module `bench_matrix::params`

This module provides types for defining individual parameter values and combinations of them.

**Enum `MatrixCellValue`:**

Represents a single "custom value" that can be part of a parameter axis.
`#[derive(Clone, PartialEq, Eq, Hash, Debug)]`
(Also implements `Display`, and various `From<T>` traits for ergonomic construction.)

*   **Variants:**
    *   `Tag(String)`: A semantic tag or identifier.
    *   `String(String)`: A general-purpose string value.
    *   `Int(i64)`: A signed integer value.
    *   `Unsigned(u64)`: An unsigned integer value.
    *   `Bool(bool)`: A boolean value.

*   **`From` Implementations for `MatrixCellValue`:**
    *   `impl From<&'static str> for MatrixCellValue` (creates a `Tag`)
    *   `impl From<String> for MatrixCellValue` (creates a `String`)
    *   `impl From<i64> for MatrixCellValue`
    *   `impl From<i32> for MatrixCellValue`
    *   `impl From<u64> for MatrixCellValue`
    *   `impl From<u32> for MatrixCellValue`
    *   `impl From<bool> for MatrixCellValue`

**Struct `AbstractCombination`:**

Represents one specific combination of abstract parameter values.
`#[derive(Debug, Clone)]`

*   **Public Fields:**
    *   `pub cells: Vec<MatrixCellValue>`: The collection of `MatrixCellValue`s that make up this unique combination.
*   **Public Methods:**
    *   `pub fn id_suffix(&self) -> String`
        *   Generates a string suffix suitable for use in benchmark IDs.
    *   `pub fn get_tag(&self, index: usize) -> Result<&str, String>`
        *   Helper to get a cell by index and interpret it as a `Tag`.
    *   `pub fn get_string(&self, index: usize) -> Result<&str, String>`
        *   Helper to get a cell by index and interpret it as a `String`.
    *   `pub fn get_i64(&self, index: usize) -> Result<i64, String>`
        *   Helper to get a cell by index and interpret it as an `Int`.
    *   `pub fn get_u64(&self, index: usize) -> Result<u64, String>`
        *   Helper to get a cell by index and interpret it as an `Unsigned`.
    *   `pub fn get_bool(&self, index: usize) -> Result<bool, String>`
        *   Helper to get a cell by index and interpret it as a `Bool`.

---

### Module `bench_matrix::generator`

This module provides utilities for generating parameter combinations.

**Function `generate_combinations`:**

Generates all possible unique combinations (Cartesian product) from a set of parameter axes.

*   **Signature:**
    `pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> Vec<AbstractCombination>`
*   **Parameters:**
    *   `axes`: A slice of `Vec<MatrixCellValue>`. Each inner `Vec` represents one parameter axis.
*   **Returns:**
    *   A `Vec<AbstractCombination>` containing all generated combinations. Returns an empty `Vec` if `axes` is empty or if any individual axis is empty.

---

### Module `bench_matrix::criterion_runner`

This module contains common types used by the Criterion integration suites. The types themselves (`ExtractorFn`, `GlobalSetupFn`, `GlobalTeardownFn`) are re-exported at the crate root and documented there. This section refers to the submodules for specific benchmark suites.

*   `pub mod async_suite;`
*   `pub mod sync_suite;`

---

### Module `bench_matrix::criterion_runner::sync_suite`

Provides the `SyncBenchmarkSuite` for orchestrating synchronous benchmarks with Criterion.

**Type Aliases (Specific to `SyncBenchmarkSuite`):**

(These are also re-exported at the crate root `bench_matrix` and documented there.)

*   `pub type SyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&Cfg) -> Result<(CtxT, S), SetupErr>;`
*   `pub type SyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> (CtxT, S, std::time::Duration);`
*   `pub type SyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> ();`

**Struct `SyncBenchmarkSuite`:**

Orchestrates a suite of synchronous benchmarks.

*   **Signature:**
    `pub struct SyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
    *   `'s`: Lifetime parameter tied to the `Criterion` instance.
    *   `S`: User-defined state type, must be `Send + 'static`.
    *   `Cfg`: User-defined concrete configuration type, must be `Clone + Debug + Send + Sync + 'static`.
    *   `CtxT`: User-defined context type, must be `Send + 'static`.
    *   `ExtErr`: Error type for the `ExtractorFn`, must be `Debug` (defaults to `String`).
    *   `SetupErr`: Error type for the `SyncSetupFn`, must be `Debug` (defaults to `String`).

*   **Public Methods:**
    *   `#[allow(clippy::too_many_arguments)]`
        `pub fn new(`
        `  criterion: &'s mut criterion::Criterion<criterion::measurement::WallTime>,`
        `  suite_base_name: String,`
        `  parameter_axes: Vec<Vec<MatrixCellValue>>,`
        `  extractor_fn: ExtractorFn<Cfg, ExtErr>,`
        `  setup_fn: SyncSetupFn<S, Cfg, CtxT, SetupErr>,`
        `  benchmark_logic_fn: SyncBenchmarkLogicFn<S, Cfg, CtxT>,`
        `  teardown_fn: SyncTeardownFn<S, Cfg, CtxT>,`
        `) -> Self`
        *   Constructs a new `SyncBenchmarkSuite`.
    *   `pub fn global_setup(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
        *   Sets the global setup function to be run once per concrete configuration `Cfg` before its benchmark group.
    *   `pub fn global_teardown(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
        *   Sets the global teardown function to be run once per `Cfg` after its benchmark group.
    *   `pub fn configure_criterion_group(self, f: impl for<'g> Fn(&mut criterion::BenchmarkGroup<'g, criterion::measurement::WallTime>) + 'static) -> Self`
        *   Provides a closure to customize the `criterion::BenchmarkGroup` for each variant.
    *   `pub fn throughput(self, f: impl Fn(&Cfg) -> criterion::Throughput + 'static) -> Self`
        *   Provides a closure to calculate `criterion::Throughput` based on the `Cfg`.
    *   `pub fn run(mut self)`
        *   Executes the benchmark suite, generating combinations, running benchmarks through Criterion, and handling setup/teardown.

---

### Module `bench_matrix::criterion_runner::async_suite`

Provides the `AsyncBenchmarkSuite` for orchestrating asynchronous benchmarks with Criterion, typically using Tokio.

**Type Aliases (Specific to `AsyncBenchmarkSuite`):**

(These are also re-exported at the crate root `bench_matrix` and documented there.)

*   `pub type AsyncSetupFn<S, Cfg, CtxT, SetupErr = String> = fn(&tokio::runtime::Runtime, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(CtxT, S), SetupErr>> + Send>>;`
*   `pub type AsyncBenchmarkLogicFn<S, Cfg, CtxT> = fn(CtxT, S, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = (CtxT, S, std::time::Duration)> + Send>>;`
*   `pub type AsyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &tokio::runtime::Runtime, &Cfg) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;`

**Struct `AsyncBenchmarkSuite`:**

Orchestrates a suite of asynchronous benchmarks.

*   **Signature:**
    `pub struct AsyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
    *   `'s`: Lifetime parameter tied to the `Criterion` instance and `Runtime`.
    *   `S`: User-defined state type, must be `Send + 'static`.
    *   `Cfg`: User-defined concrete configuration type, must be `Clone + Debug + Send + Sync + 'static`.
    *   `CtxT`: User-defined context type, must be `Send + 'static`.
    *   `ExtErr`: Error type for the `ExtractorFn`, must be `Debug` (defaults to `String`).
    *   `SetupErr`: Error type for the `AsyncSetupFn`, must be `Debug` (defaults to `String`).

*   **Public Methods:**
    *   `#[allow(clippy::too_many_arguments)]`
        `pub fn new(`
        `  criterion: &'s mut criterion::Criterion<criterion::measurement::WallTime>,`
        `  runtime: &'s tokio::runtime::Runtime,`
        `  suite_base_name: String,`
        `  parameter_axes: Vec<Vec<MatrixCellValue>>,`
        `  extractor_fn: ExtractorFn<Cfg, ExtErr>,`
        `  setup_fn: AsyncSetupFn<S, Cfg, CtxT, SetupErr>,`
        `  benchmark_logic_fn: AsyncBenchmarkLogicFn<S, Cfg, CtxT>,`
        `  teardown_fn: AsyncTeardownFn<S, Cfg, CtxT>,`
        `) -> Self`
        *   Constructs a new `AsyncBenchmarkSuite`. Requires a reference to a Tokio `Runtime`.
    *   `pub fn global_setup(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
        *   Sets the global setup function.
    *   `pub fn global_teardown(self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self`
        *   Sets the global teardown function.
    *   `pub fn configure_criterion_group(self, f: impl for<'g> Fn(&mut criterion::BenchmarkGroup<'g, criterion::measurement::WallTime>) + 'static) -> Self`
        *   Provides a closure to customize the `criterion::BenchmarkGroup`.
    *   `pub fn throughput(self, f: impl Fn(&Cfg) -> criterion::Throughput + 'static) -> Self`
        *   Provides a closure to calculate `criterion::Throughput`.
    *   `pub fn run(mut self)`
        *   Executes the asynchronous benchmark suite.

## 3. Error Handling

`bench_matrix` itself does not define a global error enum. Instead, error handling relies on:

1.  **User-Provided Error Types:**
    *   The `ExtractorFn` has a generic error type `ExtErr` which defaults to `String`. Users can define their own error enum for extraction failures.
    *   The `SetupFn` (both sync and async) has a generic error type `SetupErr` which also defaults to `String`.
    *   Other callbacks like `GlobalSetupFn` and `GlobalTeardownFn` return `Result<(), String>`, indicating failure with a string message.

2.  **Panics:** If a setup function (e.g., `AsyncSetupFn` or `SyncSetupFn` called within Criterion's sampling loop) fails with an error, the benchmark suites will typically `panic!` with a descriptive message. This is because Criterion's `iter_custom` and `to_async().iter_custom()` expect the setup phase within them to succeed to proceed with measurements. Failures in `GlobalSetupFn` or `ExtractorFn` lead to skipping the respective benchmark variants with an error message printed to `stderr`, but do not panic the entire benchmark run.

3.  **Reporting:** The benchmark suites print error messages to `stderr` when:
    *   Combination extraction fails.
    *   Global setup for a configuration fails.
    *   Global teardown for a configuration fails (reported as a warning).

Users are responsible for handling errors within their `benchmark_logic_fn` and `teardown_fn` implementations as these functions are not expected to return `Result` to the `bench_matrix` framework directly (though they can, of course, propagate panics).