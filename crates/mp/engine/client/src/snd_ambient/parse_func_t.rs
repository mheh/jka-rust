#![allow(non_camel_case_types, non_snake_case)]

use super::ambient_set_s::ambientSet_t;

/// Raven `parseFunc_t` — function pointer type for parsing ambient sets.
///
/// Type definition source: `oracle/oracle/codemp/client/snd_ambient.h:75`
pub type parseFunc_t = unsafe extern "C" fn(*mut ambientSet_t) -> ();
