#![allow(non_camel_case_types, non_snake_case)]

/// Raven `set_e` — ambient set type categories.
///
/// Raven: General sets, local sets (regional), and brush model sets (doors, plats, etc.).
/// Type definition source: `oracle/codemp/client/snd_ambient.h:33-40`
/// Type definition source: `oracle/code/client/snd_ambient.h:33-40`
#[repr(i32)]
pub enum set_e {
    /// General sets
    AS_SET_GENERAL = 0,
    /// Local sets (regional)
    AS_SET_LOCAL = 1,
    /// Brush model sets (doors, plats, etc.)
    AS_SET_BMODEL = 2,
    NUM_AS_SETS = 3,
}

const _: () = assert!(core::mem::size_of::<set_e>() == 4);
