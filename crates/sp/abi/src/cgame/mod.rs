//! Single-player client-game ABI surface (`code/cgame`).

pub mod exports;
pub mod imports;
pub mod public;
pub mod syscalls;
pub mod types;
pub mod vmcalls;

pub use exports::SpCgameExport;
pub use imports::SpCgameImport;
