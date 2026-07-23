#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::l_script::token_s::Token;

/// Raven `define_t` — a preprocessor `#define` macro entry.
///
/// Idiomatic redesign (porting-rules §F17): `char *name` owns its bytes as a
/// `String`, and the malloc'd `token_t *parms` / `token_t *tokens` lists become
/// owned `Vec<Token>`. The three intrusive links — `next` (definition list),
/// `hashnext` (hash chain), `globalnext` (global-defines chain) — dissolve into
/// the owner's arena-plus-buckets shape: a `Source` keeps `defines: Vec<Define>`
/// with `definehash: Vec<Vec<usize>>` prepend-buckets, and `BotLib` keeps the
/// global `globaldefines: Vec<Define>` arena.
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.h:55-66`
#[derive(Clone, Default)]
pub struct Define {
    /// define name
    pub name: String,
    /// define flags
    pub flags: c_int,
    /// > 0 if builtin define
    pub builtin: c_int,
    /// number of define parameters
    pub numparms: c_int,
    /// define parameters
    pub parms: Vec<Token>,
    /// macro tokens (possibly containing parm tokens)
    pub tokens: Vec<Token>,
}
