//! MP `bg_public.h` field type definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1231-1258`

#![allow(non_camel_case_types)]

/// Raven `fieldtype_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1231-1258`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fieldtype_t {
    F_INT = 0,
    F_FLOAT = 1,
    F_LSTRING = 2,
    F_GSTRING = 3,
    F_VECTOR = 4,
    F_ANGLEHACK = 5,
    F_ENTITY = 6,
    F_ITEM = 7,
    F_CLIENT = 8,
    F_PARM1 = 9,
    F_PARM2 = 10,
    F_PARM3 = 11,
    F_PARM4 = 12,
    F_PARM5 = 13,
    F_PARM6 = 14,
    F_PARM7 = 15,
    F_PARM8 = 16,
    F_PARM9 = 17,
    F_PARM10 = 18,
    F_PARM11 = 19,
    F_PARM12 = 20,
    F_PARM13 = 21,
    F_PARM14 = 22,
    F_PARM15 = 23,
    F_PARM16 = 24,
    F_IGNORE = 25,
    /// jka-rust tail-field migration: the field's value is owned by the entity
    /// (`String`/`Option<String>`), set through a typed setter carried on
    /// [`super::bg_field::BG_field_t::set`] rather than an offset write. No Raven
    /// counterpart — Raven kept these as `F_LSTRING` pool pointers.
    F_STRING_OWNED = 26,
}
