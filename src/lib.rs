//! `bench_matrix` is a utility crate for defining and orchestrating
//! parameterized benchmarks, allowing for the generation and execution
//! of tests across a matrix of configurations. It offers optional
//! integration with the Criterion benchmarking harness.

// Define modules
#[cfg(feature = "criterion_integration")]
pub mod criterion_runner;
pub mod generator; // For generate_combinations
pub mod params; // For MatrixCellValue, AbstractCombination, etc. // For the Criterion-specific orchestrator

// Re-export key types for easier public use
pub use generator::generate_combinations;
pub use params::{AbstractCombination, MatrixCellValue};

// --- Re-exports for Criterion Integration (from the submodules) ---

// Common types used by both async and sync criterion runners
#[cfg(feature = "criterion_integration")]
pub use criterion_runner::{ExtractorFn, GlobalSetupFn, GlobalTeardownFn};

// Async specific exports
#[cfg(feature = "criterion_integration")]
pub use criterion_runner::async_suite::{
  AsyncBenchmarkLogicFn,
  AsyncBenchmarkSuite,
  // Function signature types for async benchmarks
  AsyncSetupFn,
  AsyncTeardownFn,
};

// Sync specific exports
#[cfg(feature = "criterion_integration")]
pub use criterion_runner::sync_suite::{
  SyncBenchmarkLogicFn,
  SyncBenchmarkSuite,
  // Function signature types for sync benchmarks
  SyncSetupFn,
  SyncTeardownFn,
};
