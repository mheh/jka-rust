//! MP `bg_public.h` field descriptor.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1263-1269`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use native_types::byte;

use super::fieldtype::fieldtype_t;

/// Typed setter for an [`fieldtype_t::F_STRING_OWNED`] tail field: given the
/// entity base pointer and the decoded value, stores it into the entity's owned
/// `String`/`Option<String>` field. The base is type-erased (`*mut byte`) because
/// bg cannot name the game tier's `gentity_t`; the game-tier setter casts it. No
/// Raven counterpart — Raven set these fields as `F_LSTRING` pool pointers.
pub type SpawnStringSetter = fn(*mut byte, &str);

/// Raven `BG_field_t`, plus the jka-rust `set` slot for owned tail fields.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1263-1269`
// No `PartialEq`/`Eq`: the `set` fn-pointer slot has no meaningful equality
// (function addresses are not unique), and no caller compares `BG_field_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BG_field_t {
    pub name: *mut c_char,
    pub ofs: c_int,
    pub r#type: fieldtype_t,
    pub flags: c_int,
    /// `Some` only for [`fieldtype_t::F_STRING_OWNED`] entries; `None` for every
    /// offset-written (POD/`F_LSTRING`) entry. No Raven counterpart.
    pub set: Option<SpawnStringSetter>,
}

// `name` is always initialized from a `'static` string literal (Raven's
// spawn-field tables are `static const fieldDescriptor_t` arrays of C-string
// literals cast to `char *`) and never mutated, so sharing a `BG_field_t`
// table across threads is sound despite the raw pointer.
unsafe impl Sync for BG_field_t {}

// `BG_field_t` is game-internal (the spawn-field table never crosses the engine
// ABI seam), so it carries no layout contract with the engine; the asserts below
// pin the Raven prefix (`name`/`ofs`/`type`/`flags`) and the appended `set` slot.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<BG_field_t>() == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, ofs) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, r#type) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, flags) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, set) == 24);
