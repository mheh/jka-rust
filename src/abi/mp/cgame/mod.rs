//! Multiplayer client-game ABI surface (`codemp/cgame`).

pub mod exports;
pub mod imports;
pub mod shared_buffer;
pub mod syscalls;
pub mod vmcalls;

pub use exports::MpCgameExport;
pub use imports::MpCgameImport;
