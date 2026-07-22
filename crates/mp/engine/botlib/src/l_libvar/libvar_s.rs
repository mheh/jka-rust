#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `libvar_t` — a bot library variable (cvar-like entry).
///
/// Idiomatic redesign (porting-rules §F17): Raven's malloc'd
/// `{char *name; char *string; int flags; qboolean modified; float value;
/// libvar_s *next;}` linked-list node becomes an owned entry in `BotLib`'s
/// `libvars: Vec<LibVar>` arena (§B); `name`/`string` own their bytes and the
/// `next` link is replaced by the vector's ordering. Raven's `flags` is
/// dropped — it is zero-initialized in `LibVarAlloc` and never read or written
/// anywhere in botlib.
///
/// Type definition source: `oracle/codemp/botlib/l_libvar.h:16-24`
pub struct LibVar {
    pub name: String,
    pub string: String,
    /// set each time the cvar is changed
    pub modified: qboolean,
    pub value: f32,
}

/// Stable handle to a `LibVar` slot in `BotLib::libvars`.
///
/// Raven's `LibVar()` returned a `libvar_t *` that callers cached and later
/// dereferenced to re-read `.value`/`.string` live; those callers hold this
/// index instead (porting-rules §B5 arena handle — an index, not a pointer
/// wrapper). Slots are only ever appended (`LibVarAlloc`) or cleared en masse
/// (`LibVarDeAllocAll`), never individually removed, so an index stays valid
/// for the lifetime of the arena.
#[derive(Clone, Copy)]
pub struct LibVarHandle(pub usize);
