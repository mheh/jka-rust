//! Multiplayer server-game ABI surface (`codemp/game`).

pub mod exports;
pub mod imports;
pub mod syscalls;
pub mod vmcalls;

pub use exports::MpGameExport;
pub use imports::MpGameImport;
