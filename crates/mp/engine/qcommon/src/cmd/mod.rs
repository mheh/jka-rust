//! `cmd` types.

pub mod cmd_consts;
pub mod cmd_function_t;

// `cmd_pc.cpp`'s registered-command functions live in `cmd_pc.rs`; re-export the
// registration entrypoints here so importers reach them at `crate::cmd::*`.
pub use crate::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
