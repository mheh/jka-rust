//! `l_precomp.h` path-separator constants.
//!
//! Guarded by `#if defined(WIN32)|defined(_WIN32)|defined(__NT__)|...`; this
//! project only builds for Linux, so the `else` branch applies.
//!
//! Source: `oracle/codemp/botlib/l_precomp.h:17-29`

/// Raven `PATHSEPERATOR_STR` (Raven's spelling, not "SEPARATOR").
/// Source: `oracle/codemp/botlib/l_precomp.h:21`
pub const PATHSEPERATOR_STR: &str = "/";

/// Raven `PATHSEPERATOR_CHAR` (Raven's spelling, not "SEPARATOR").
/// Source: `oracle/codemp/botlib/l_precomp.h:28`
pub const PATHSEPERATOR_CHAR: u8 = b'/';
