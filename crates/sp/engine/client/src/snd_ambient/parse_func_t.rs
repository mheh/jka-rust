#![allow(non_camel_case_types, non_snake_case)]

use super::ambient_set_s::ambientSet_t;

/// Raven `parseFunc_t` — function pointer for parsing ambient sets.
///
/// Type definition source: `oracle/oracle/code/client/snd_ambient.h:75-75`
pub type parseFunc_t = extern "C" fn(*mut ambientSet_t);
const _: () = assert!(core::mem::size_of::<parseFunc_t>() == 8);
