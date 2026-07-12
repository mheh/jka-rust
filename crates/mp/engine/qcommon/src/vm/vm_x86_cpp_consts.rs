//! `vm_x86.cpp`-local constants.
//!
//! Source: `oracle/codemp/qcommon/vm_x86.cpp:36`

/// Raven `FTOL_PTR` — selects the `_ftol` float-to-int helper pointer path;
/// its effect is further gated by `#ifdef _WIN32` at each use site, which
/// this project's Linux target never satisfies. Ported as `bool` since Raven
/// never gives it a value, only tests it with `#if defined(...)`, and it is
/// defined unconditionally at this site.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:36`
pub const FTOL_PTR: bool = true;
