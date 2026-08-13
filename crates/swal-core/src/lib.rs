//! swal-core — Platform-agnostic agent loop + tools trait (wasm32-clean)
//! Wave skeleton. Implementation lands in Wave 1+.

pub mod tool;

// Re-export public dependencies to facilitate implementing the Tool trait
// in downstream crates and integration tests without duplicate dependency declarations.
pub use schemars;
pub use async_trait;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
