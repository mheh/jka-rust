#![allow(non_camel_case_types, non_snake_case)]

/// Raven `set_e` — ambient set types.
///
/// Type definition source: `oracle/code/client/snd_ambient.h:33-40`
#[repr(i32)]
pub enum set_e {
    /// General sets
    AS_SET_GENERAL,
    /// Local sets (regional)
    AS_SET_LOCAL,
    /// Brush model sets (doors, plats, etc.)
    AS_SET_BMODEL,
    NUM_AS_SETS,
}
const _: () = assert!(core::mem::size_of::<set_e>() == 4);
