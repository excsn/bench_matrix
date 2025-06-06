### `bench_matrix` API Reference (Updated)

## 1. Introduction / Core Concepts

`bench_matrix` is a Rust utility crate designed to simplify the creation and execution of parameterized benchmarks, especially when using the Criterion benchmarking harness. It allows you to define a matrix of input parameters and automatically run your benchmark logic for every combination of these parameters.

**Core Concepts:**

*   **Parameter Axis:** A list of possible values for a single dimension of your benchmark configuration. For example, an axis could represent different buffer sizes `[64, 128, 256]` or different algorithms `["AlgorithmA", "AlgorithmB"]`.
*   **Parameter Names:** A list of human-readable names corresponding to each parameter axis, used for generating descriptive benchmark IDs.
*   **`MatrixCellValue`:** An enum representing a single value within a parameter axis (e.g., a specific tag, string, integer, or boolean).
*   **`AbstractCombination`:** A struct representing one unique combination of `MatrixCellValue`s, one from each defined axis. This forms a single "row" in your conceptual parameter matrix.
*   **Configuration Extraction (`ExtractorFn`):** A user-provided function that takes an `AbstractCombination` and translates it into a concrete, strongly-typed configuration struct (defined by the user) that the benchmark logic will use.
*   **Benchmark Suites (`SyncBenchmarkSuite`, `AsyncBenchmarkSuite`):** The primary structures for defining and running a set of parameterized benchmarks. They orchestrate combination generation, configuration extraction, and integration with Criterion.
    *   `SyncBenchmarkSuite`: For benchmarking synchronous code.
    *   `AsyncBenchmarkSuite`: For benchmarking asynchronous code, typically with a Tokio runtime.
*   **Benchmark Lifecycle Functions:** Users provide several callback functions to the suites to define the benchmark behavior:
    *   **Setup:** Prepares the necessary state and context for a benchmark sample.
    *   **Logic:** Contains the actual code to be measured.
    *   **Teardown:** Cleans up resources after a benchmark sample.
    *   **Global Setup/Teardown:** Run once per concrete configuration, before and after all benchmarks for that configuration are defined.

**Main Entry Points:**

The primary way to interact with `bench_matrix` is by:

1.  Defining your parameter axes using `Vec<Vec<MatrixCellValue>>` and their corresponding names using `Vec<String>`.
2.  Creating an instance of either `SyncBenchmarkSuite` or `AsyncBenchmarkSuite`.
3.  Providing the necessary callback functions (extractor, setup, logic, teardown) to the suite.
4.  Optionally configuring the suite further using its builder methods (e.g., `.global_setup()`, `.throughput()`).
5.  Calling the `run()` method on the suite instance within your Criterion benchmark functions. This will create a single benchmark group named after your suite, with each parameter combination registered as a separate benchmark within that group (e.g., `MySuite/ParamA-Val1_ParamB-Val2`).

**Pervasive Types and Patterns:**

*   **`MatrixCellValue` and `AbstractCombination`:** These are fundamental for defining and working with parameter combinations.
*   **Function Pointers/Closures:** The library extensively uses function pointers (e.g., `fn(...) -> ...`) and `Box<dyn Fn(...)>` for user-provided callbacks. This allows for flexible and type-safe customization of the benchmarking process.
*   **User-Defined Configuration (`Cfg`), State (`S`), and Context (`CtxT`):** The benchmark suites are generic over these types, which are defined by the user to match the specific needs of their benchmarks.
    *   `Cfg`: Your concrete configuration struct derived from an `AbstractCombination`.
    *   `S`: The state that is set up before benchmarking and potentially modified by it.
    *   `CtxT`: An optional context that can be passed through benchmark iterations within a sample.
*   **Error Handling:** User-provided functions like the `ExtractorFn` or `GlobalSetupFn` typically return `Result<T, UserErrorType>`. The suites will report these errors and skip benchmark variants accordingly.

## 2. Modules and Public API

This reference is organized by the public modules of the `bench_matrix` crate.

---

### Crate Root (`bench_matrix`)

These are items re-exported at the top level of the `bench_matrix` crate for convenience.

**(The Type Alias sections remain largely the same as they describe the function signatures, so they are omitted here for brevity. They are still correct.)**

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

Represents a single value that can be part of a parameter axis.
`#[derive(Clone, PartialEq, Eq, Hash, Debug)]`
(Also implements `Display`, and various `From<T>` traits for ergonomic construction.)

*   **Variants:** `Tag(String)`, `String(String)`, `Int(i64)`, `Unsigned(u64)`, `Bool(bool)`.

**Struct `AbstractCombination`:**

Represents one specific combination of abstract parameter values.
`#[derive(Debug, Clone)]`

*   **Public Fields:**
    *   `pub cells: Vec<MatrixCellValue>`
*   **Public Methods:**
    *   `pub fn id_suffix(&self) -> String`
        *   Generates a string suffix for benchmark IDs (e.g., `_Value1_Value2`).
    *   `pub fn id_suffix_with_names(&self, param_names: &[String]) -> String`
        *   Generates a descriptive string suffix, incorporating parameter names (e.g., `_ParamName1-Value1_ParamName2-Value2`).
    *   `pub fn get_tag(...)`, `get_string(...)`, `get_i64(...)`, `get_u64(...)`, `get_bool(...)`
        *   Helpers to get a cell by index and interpret it as a specific type.

---

### Module `bench_matrix::generator`

This module provides utilities for generating parameter combinations.

**Struct `CombinationIterator`:**

An iterator that lazily generates the Cartesian product of benchmark parameter axes.
`#[derive(Debug, Clone)]`

*   **Description:** This struct is highly memory-efficient as it generates each `AbstractCombination` on the fly. It implements `ExactSizeIterator`, allowing the use of `.len()` to get the total number of combinations without consuming the iterator.
*   **Implementation Details:**
    *   `impl<'a> Iterator for CombinationIterator<'a>`
    *   `impl<'a> ExactSizeIterator for CombinationIterator<'a>`

**Function `generate_combinations`:**

Creates a `CombinationIterator` over a set of parameter axes.

*   **Signature:**
    `pub fn generate_combinations(axes: &[Vec<MatrixCellValue>]) -> CombinationIterator`
*   **Parameters:**
    *   `axes`: A slice of `Vec<MatrixCellValue>`. Each inner `Vec` represents one parameter axis.
*   **Returns:**
    *   A `CombinationIterator` that will lazily yield all generated combinations. If `axes` is empty or if any individual axis is empty, the returned iterator will have a length of 0.

---

### Module `bench_matrix::criterion_runner::sync_suite`

Provides the `SyncBenchmarkSuite` for orchestrating synchronous benchmarks.

**Struct `SyncBenchmarkSuite`:**

Orchestrates a suite of synchronous benchmarks.

*   **Signature:**
    `pub struct SyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
*   **Public Methods:**
    *   `pub fn new(...) -> Self`
        *   Constructs a new `SyncBenchmarkSuite`.
    *   `pub fn parameter_names(self, names: Vec<String>) -> Self`
        *   Builder method to set or override the parameter names.
    *   `pub fn global_setup(self, f: ...) -> Self`
        *   Sets the global setup function.
    *   `pub fn global_teardown(self, f: ...) -> Self`
        *   Sets the global teardown function.
    *   `pub fn configure_criterion_group(self, f: ...) -> Self`
        *   Provides a closure to customize the `criterion::BenchmarkGroup`.
    *   `pub fn throughput(self, f: ...) -> Self`
        *   Provides a closure to calculate `criterion::Throughput` for each benchmark variant.
    *   `pub fn run(mut self)`
        *   Executes the benchmark suite. This creates a single benchmark group (named `suite_base_name`) and registers each parameter combination as a benchmark within it.

---

### Module `bench_matrix::criterion_runner::async_suite`

Provides the `AsyncBenchmarkSuite` for orchestrating asynchronous benchmarks.

**Struct `AsyncBenchmarkSuite`:**

Orchestrates a suite of asynchronous benchmarks.

*   **Signature:**
    `pub struct AsyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String>`
*   **Public Methods:**
    *   `pub fn new(...) -> Self`
        *   Constructs a new `AsyncBenchmarkSuite`. Requires a reference to a Tokio `Runtime`.
    *   `pub fn parameter_names(self, names: Vec<String>) -> Self`
        *   Builder method to set or override the parameter names.
    *   `pub fn global_setup(self, f: ...) -> Self`
        *   Sets the global setup function.
    *   `pub fn global_teardown(self, f: ...) -> Self`
        *   Sets the global teardown function.
    *   `pub fn configure_criterion_group(self, f: ...) -> Self`
        *   Provides a closure to customize the `criterion::BenchmarkGroup`.
    *   `pub fn throughput(self, f: ...) -> Self`
        *   Provides a closure to calculate `criterion::Throughput` for each benchmark variant.
    *   `pub fn run(mut self)`
        *   Executes the asynchronous benchmark suite, creating a single benchmark group and registering each parameter combination as a benchmark within it.

## 3. Error Handling

`bench_matrix` itself does not define a global error enum. Instead, error handling relies on:

1.  **User-Provided Error Types:**
    *   The `ExtractorFn` and `SetupFn`s have generic error types (`ExtErr`, `SetupErr`) that default to `String`.
2.  **Panics:** If a setup function (e.g., `AsyncSetupFn` or `SyncSetupFn` called within Criterion's sampling loop) fails with an error, the benchmark suites will `panic!` with a descriptive message.
3.  **Reporting:** The benchmark suites print error messages to `stderr` when:
    *   Combination extraction fails (variant skipped).
    *   Global setup for a configuration fails (variant skipped).
    *   A mismatch occurs between the length of `parameter_names` and `parameter_axes` (warning).

Users are responsible for handling errors within their `benchmark_logic_fn` and `teardown_fn` implementations.