//! swal-core — Platform-agnostic agent loop + tools trait (wasm32-clean)
//! Wave skeleton. Implementation lands in Wave 1+.

pub mod tool;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
