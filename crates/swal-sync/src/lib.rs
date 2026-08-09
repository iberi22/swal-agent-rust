//! swal-sync — SyncEngine behind transport trait; CRDT/merge logic, wasm-clean
//!
//! This crate contains the SyncEngine and core CRDT operational/state merge logic.
//! It is completely wasm-clean (pure Rust, no standard I/O, no network).

pub mod engine;
pub mod transport;
