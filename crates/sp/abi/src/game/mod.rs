//! Single-player server-game ABI surface (`code/game`).
//!
//! Raven SP game uses the `GetGameAPI` function-table ABI, not the `vmMain` /
//! syscall shape. Keep that surface deferred until the table ABI is modeled.

pub mod exports;
pub mod imports;
pub mod public;

pub use exports::{SpGameExport, SpGameExportTable};
pub use imports::{SpGameImport, SpGameImportTable};
