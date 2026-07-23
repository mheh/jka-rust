#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::source_s::Source;
use crate::BotLib;

/// Raven `directive_t` — one row of the preprocessor directive dispatch table
/// (`directives[]`/`dollardirectives[]`, l_precomp.cpp:2535/2648). File-local to
/// `l_precomp.cpp`; never crosses the ABI seam.
///
/// Idiomatic redesign (porting-rules §F17): the malloc-free static tables are
/// `&'static [Directive]` consts, so `char *name` becomes a `&'static str` and
/// Raven's trailing `{NULL, NULL}` sentinel row dissolves — slice iteration ends
/// at the slice bound, exactly as the `default_punctuations` table does. Raven's
/// handler `int (*func)(source_t *source)` threads the port's `&mut BotLib`
/// receiver plus the owned `&mut Source` it drives.
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.cpp:86-90`
pub struct Directive {
    pub name: &'static str,
    pub func: fn(&mut BotLib, &mut Source) -> c_int,
}
