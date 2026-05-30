#![no_std]
#![allow(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic, clippy::cargo)]

//! # Summary
//! A pure `no_std`, zero-heap, single-threaded CLI chat state machine.
//!
//! # Description
//! Designed for resource-constrained embedded environments. Provides deterministic
//! memory layout, hardware-abstraction traits, and a guard-checked command parser.
//! Intended for static BSS allocation and explicit single-threaded polling.
//!
//! # Examples
//! ```rust,ignore
//! // Core library does not run on its own. See runner_std for usage.
//! ```
//!
//! # Panics
//! This crate is designed to never panic under normal operation.
//!
//! # Errors
//! Fallible operations return Result with static string slices.
//!
//! # Safety
//! This crate strictly forbids unsafe code globally, with isolated exceptions
//! for uninitialized memory management in specific data structures.
//!
//! # See Also
//! [`state::AppState`], [`serial::SerialPort`], [`storage::Storage`]

pub mod serial;
pub mod state;
pub mod storage;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod tests;

pub use serial::SerialPort;
pub use state::AppState;
pub use storage::Storage;
