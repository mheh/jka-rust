use core::ffi::c_void;

// The five raw ABI aliases relocated to `native/platform` per module-loading.md
// LOAD-D6 (base tier) and re-exported here so existing `abi_transport`
// consumers (the module cdylib shells) are unaffected and no tier inversion
// occurs. `abi-transport` takes a downhill dep on `native_platform`.
pub use native_platform::entrypoints::{AbiCommand, AbiWord, RawDllEntry, RawSyscall, RawVmMain};

pub type RawImportTable = *mut c_void;
pub type RawExportTable = *mut c_void;

pub type RawGetModuleApi =
    extern "C" fn(api_version: AbiCommand, import: RawImportTable) -> RawExportTable;
pub type RawGetGameApi = extern "C" fn(import: RawImportTable) -> RawExportTable;

// The former `qvm`/`sp_game` `#[no_mangle]` stub-export modules were the
// pre-decision state superseded by LOAD-D4/SEAM-D10: live exports are declared
// per module cdylib shell crate (`jampgame`/`jagame` hold theirs; `cgame`/`ui`
// carry interim stubs pending their live match). One shared entrypoints.rs
// cannot carry per-module exports, and the shared `#[no_mangle]` symbols
// collide with the shells' live ones at cdylib link time.
