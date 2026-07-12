//! `RestartKind` — VM-restart semantics (LOAD-D2).

/// VM-restart semantics (LOAD-D2). NativeDll and Static restart ONLY by
/// drop+recreate — Raven's actual native path. There is NO in-place native
/// reset (the QVM in-place arm is out of scope).
///
/// Source: `oracle/codemp/qcommon/vm.cpp:391-458`
pub enum RestartKind {
    /// `unload_module(old)` then `sys_load_dll(...)` — native map-change path
    /// (`sv_init.cpp:484,662`) and Raven's native `VM_Restart` (`vm.cpp:399-409`).
    DropRecreate,
}
