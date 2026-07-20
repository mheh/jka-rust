//! `native_platform` — Raven-free OS/platform primitives (cross-mode).
#![allow(non_camel_case_types)]

pub mod entrypoints;
pub mod mem;
pub mod module_loader;
pub mod platform;

// Raven ships the platform layer as per-OS source trees (`unix/` vs `win32/`);
// only the unix tree is ported. Gated so non-unix builds of dependents (the
// Windows jampgame cdylib lanes) compile without the unported win32 twin.
#[cfg(unix)]
pub mod net;
#[cfg(unix)]
pub mod sys_main;
#[cfg(unix)]
pub mod sys_shared;

pub use mem::{zeroed_box, ZeroValid};

// The native `Sys_*` OS surface (libc transcription of Raven's unix tree),
// re-exported at the crate root where qcommon call sites import them.
#[cfg(unix)]
pub use sys_main::{
    Sys_BeginStreamedFile, Sys_CheckCD, Sys_EndStreamedFile, Sys_LowPhysicalMemory, Sys_UnloadDll,
};
#[cfg(unix)]
pub use sys_shared::{
    sys_fopen, sys_remove, sys_rename, Sys_DefaultCDPath, Sys_DefaultHomePath,
    Sys_DefaultInstallPath, Sys_ListFiles, Sys_Mkdir,
};
