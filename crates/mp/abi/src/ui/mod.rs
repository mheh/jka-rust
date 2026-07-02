//! Multiplayer UI ABI surface (`codemp/ui`).

pub mod exports;
pub mod imports;
pub mod public;
pub mod syscalls;
pub mod vmcalls;

pub use exports::MpUiExport;
pub use imports::MpUiImport;
