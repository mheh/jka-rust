//! `native_platform` — Raven-free OS/platform primitives (cross-mode).
#![allow(non_camel_case_types)]

pub mod entrypoints;
pub mod mem;
pub mod module_loader;
pub mod net;
pub mod platform;
pub mod sys_main;
pub mod sys_shared;

pub use mem::{zeroed_box, ZeroValid};

// The native `Sys_*` OS surface (libc transcription of Raven's unix tree),
// re-exported at the crate root where qcommon call sites import them.
pub use sys_main::{
    Sys_BeginStreamedFile, Sys_EndStreamedFile, Sys_LowPhysicalMemory, Sys_UnloadDll,
};
pub use sys_shared::{
    Sys_DefaultCDPath, Sys_DefaultHomePath, Sys_DefaultInstallPath, Sys_FreeFileList, Sys_ListFiles,
    Sys_Mkdir,
};
