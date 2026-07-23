#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `indent_t` — a preprocessor `#if`/`#ifdef` indent stack entry.
///
/// Idiomatic redesign (porting-rules §F17): the `script_s *script` back-pointer
/// (compared for identity against `source->scriptstack` to tell whether an
/// indent belongs to the current script) becomes a `usize` index into
/// `Source::scriptstack`; the identity test is an index compare. The
/// `indent_s *next` link dissolves — the indent stack is `Source::indentstack`,
/// a `Vec<Indent>` (LIFO).
///
/// Type definition source: `oracle/codemp/botlib/l_precomp.h:71-77`
#[derive(Clone, Copy)]
pub struct Indent {
    /// indent type
    pub type_: c_int,
    /// true if skipping current indent
    pub skip: c_int,
    /// index (into `Source::scriptstack`) of the script the indent was in
    pub script: usize,
}
