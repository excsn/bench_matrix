#![cfg(feature = "criterion_integration")]

use crate::params::AbstractCombination;
use std::fmt::Debug;

// --- Common User-Provided Function Signature Types ---
// These are types that might be used by both async and sync suites,
// primarily dealing with configuration rather than execution specifics.
// Cfg: User's concrete configuration struct, derived by the ExtractorFn.
// ExtErr: User-defined error type for the extractor function. Defaults to String.

/// Function to extract/resolve a user-defined concrete configuration (`Cfg`)
/// from an `AbstractCombination`.
///
/// It takes a reference to an `AbstractCombination` (one "row" of abstract parameters)
/// and should return a `Result` containing either the successfully resolved `Cfg`
/// or an error of type `ExtErr` if the combination is invalid or resolution fails.
pub type ExtractorFn<Cfg, ExtErr = String> =
    Box<dyn Fn(&AbstractCombination) -> Result<Cfg, ExtErr>>;

/// Function to perform global setup before a Criterion benchmark group for a specific
/// resolved configuration (`Cfg`) begins.
///
/// This is useful for initializing shared resources or global state (like an io_uring backend)
/// that pertains to all benchmark iterations run under this specific `Cfg`.
/// Returns `Result<(), String>` where `String` is an error message if setup fails,
/// which would typically cause benchmarks for this `Cfg` to be skipped.
pub type GlobalSetupFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;

/// Function to perform global teardown after a Criterion benchmark group for a specific
/// resolved configuration (`Cfg`) has completed.
///
/// Used for cleaning up any resources initialized by `GlobalSetupFn`.
pub type GlobalTeardownFn<Cfg> = Box<dyn FnMut(&Cfg) -> Result<(), String>>;


// Declare the submodules for async and sync benchmark suites.
pub mod async_suite;
pub mod sync_suite;