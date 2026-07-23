#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
use std::collections::VecDeque;

use super::define_s::Define;
use super::indent_s::Indent;
use crate::l_script::script_s::Script;
use crate::l_script::token_s::Token;

/// Raven `source_t` — an open preprocessor source: its script stack, macro
/// definitions, pending tokens, and conditional-indent stack.
///
/// Idiomatic redesign (porting-rules §F17): `filename`/`includepath` own their
/// bytes as `String`; every malloc'd intrusive list becomes an owned
/// collection. `scriptstack` (a `next`-linked LIFO) → `Vec<Script>` with the
/// last element as the top; `tokens` (unread-token push-front / read-pop-front
/// list) → `VecDeque<Token>` front-stack; `indentstack` (LIFO) → `Vec<Indent>`;
/// and `defines`/`definehash` (the macro list plus its hash chains) → a
/// `Vec<Define>` arena indexed by `definehash: Vec<Vec<usize>>` prepend-buckets
/// (RULED: prepend insertion, first-match-wins lookup — a duplicate-named global
/// can coexist, so lookup stays chain-order-dependent; a `HashMap` was
/// rejected). Raven's `punctuation_t *punctuations` field is dropped: it is only
/// ever written (`PC_SetPunctuations`) and never read — scripts always take the
/// static default set.
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.h:80-92`
#[derive(Default)]
pub struct Source {
    /// file name of the script
    pub filename: String,
    /// path to include files
    pub includepath: String,
    /// stack with scripts of the source (LIFO; last element is the top)
    pub scriptstack: Vec<Script>,
    /// tokens to read first (front-stack: unread pushes front, read pops front)
    pub tokens: VecDeque<Token>,
    /// arena of macro definitions
    pub defines: Vec<Define>,
    /// hash chains into `defines`: `definehash[hash]` is a prepend-ordered bucket
    /// of indices (sized to `DEFINEHASHSIZE` by the loader)
    pub definehash: Vec<Vec<usize>>,
    /// stack with indents (LIFO)
    pub indentstack: Vec<Indent>,
    /// > 0 if skipping conditional code
    pub skip: c_int,
    /// last read token
    pub token: Token,
}
