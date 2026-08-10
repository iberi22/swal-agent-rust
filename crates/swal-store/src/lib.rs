//! swal-store — Store trait: rusqlite (native) / IndexedDB (web) backends, shared serde schema
//! Wave skeleton. Implementation lands in Wave 1+.

pub mod session;

#[cfg(target_arch = "wasm32")]
pub mod indexeddb;
