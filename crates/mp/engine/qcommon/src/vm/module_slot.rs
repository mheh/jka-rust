//! `ModuleSlot` — one occupied registry slot (LOAD-D8 round-3 composed shape).

use native_platform::module_loader::LoadedModule;

use super::engine_slot::EngineSlot;

/// One occupied registry slot — the composed per-slot struct reconciling LOAD-D8
/// with engine-seam SEAM-D11. Faithful `vm_s` mirror: the reuse-by-name key sits
/// beside the handle/entry it identifies (`vm_local.h:119,122-123`).
///
/// Source: `oracle/oracle/codemp/qcommon/vm_local.h:111-146`
pub struct ModuleSlot {
    /// `vm->name` (`vm_local.h:119`): the bare module name, the reuse-by-name key
    /// `load_module`'s scan compares case-insensitively. `pub(crate)` (LOAD-D12f).
    pub(crate) name: String,
    /// The loaded native artifact (lib + `vmMain` entry). NativeDll-only today;
    /// transport-polymorphic content for Static/Wasm is LOAD-Q9 (open).
    pub(crate) module: LoadedModule,
    /// SEAM-D11's per-slot engine syscall target the inbound trampoline forwards
    /// to — the injected ctx + `system_calls` pair (`vm->systemCall`,
    /// `vm.cpp:506`), one per slot, stored at load (skeleton-findings
    /// resolution 2, 2026-07-03).
    pub(crate) engine: EngineSlot,
}
