//! `l_script.cpp`-local constants.
//!
//! Source: `oracle/codemp/botlib/l_script.cpp:66`

/// Raven `PUNCTABLE` — unconditionally-defined feature guard enabling the
/// punctuation lookup table fast path. Ported as `bool` since Raven never
/// gives it a value, only tests it with `#ifdef`, and it is defined
/// unconditionally at this site.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:66`
pub const PUNCTABLE: bool = true;
