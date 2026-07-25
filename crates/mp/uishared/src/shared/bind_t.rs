//! `Bind` — Raven `bind_t`.

use core::ffi::c_int;

/// Raven `bind_t` — one row of the controls-menu key-binding table.
///
/// The table is seeded from a static command list but is **not** read-only:
/// `Controls_GetConfig` writes the live `bind1`/`bind2` back into every row, so
/// the rows are [`MenuSystem`](super::menu_system::MenuSystem) state, not a
/// `const`. `command` keeps `&'static str` — it is the compiled-in console
/// command name and is never rewritten.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.c:5173-5180`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(alias = "bind_t")]
#[allow(non_snake_case)]
pub struct Bind {
    pub command: &'static str,
    pub id: c_int,
    pub defaultbind1: c_int,
    pub defaultbind2: c_int,
    pub bind1: c_int,
    pub bind2: c_int,
}
