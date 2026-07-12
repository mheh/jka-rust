//! `ModuleNaming` — per-platform artifact naming (LOAD-D1).

/// Per-platform artifact naming. Windows/Unix entries are faithful to oracle;
/// the macOS entry is our documented extension (no oracle precedent — MP has no
/// Mac loader, dossier §3). Exact macOS base string: LOAD-Q1 (open).
///
/// Source: `oracle/codemp/win32/win_main.cpp:826` ("x86.dll");
/// `oracle/codemp/unix/unix_main.c:346` ("i386.so").
pub struct ModuleNaming {
    /// Appended to the bare module name, e.g. "x86.dll" → "jampgamex86.dll".
    /// `None` until the macOS host is wired (LOAD-Q1, round-4 amendment widening
    /// `&'static str` → `Option`: a macOS suffix cannot be "unset" as a bare
    /// `&'static str`).
    //TODO: Port ModuleNaming macOS suffix
    // Source: oracle/codemp/win32/win_main.cpp:826
    pub suffix: Option<&'static str>,
}
