//! `LoadedModule` — a live native module handle (LOAD-D8/D12f).

use crate::entrypoints::RawVmMain;

/// A live native module: the library handle + its resolved `vmMain` entrypoint,
/// held inside a `ModuleSlot` in the `ModuleRegistry` (module-loading § State
/// ownership). The bare name it was loaded under and the SEAM-D11 engine cell
/// live on the owning `ModuleSlot`, not here (LOAD-D8 round-3 — `vm_s`'s
/// `name`/`dllHandle`/`entryPoint` are mirrored across `ModuleSlot` + its
/// `LoadedModule`).
///
/// Source: `oracle/oracle/codemp/qcommon/vm_local.h:111-146`
pub struct LoadedModule {
    /// `vm->dllHandle` (`win_main.cpp:855-863`); `pub(crate)` per LOAD-D12f.
    pub(crate) lib: libloading::Library,
    /// `"vmMain"` (`win_main.cpp:880`); `RawVmMain` defined in this crate
    /// (LOAD-D6), re-exported by `abi-transport`.
    pub(crate) entry: RawVmMain,
}

impl LoadedModule {
    /// The resolved `vmMain` entrypoint (`vm->entryPoint`, `vm_local.h:123`).
    /// Accessor over the `pub(crate)` field (LOAD-D12f) for the registry's
    /// engine→module calls (`VM_Call`'s `vm->entryPoint` read, `vm.cpp:808`).
    pub fn entry(&self) -> RawVmMain {
        self.entry
    }
}
