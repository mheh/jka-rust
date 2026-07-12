//! `be_ai_weight.cpp`-local weight-AI constants.
//!
//! Source: `oracle/codemp/botlib/be_ai_weight.cpp:31`

/// Raven `EVALUATERECURSIVELY` — unconditionally-defined feature guard.
/// Ported as `bool` since Raven never gives it a value, only tests it with
/// `#ifdef`, and it is defined unconditionally at this site.
///
/// Source: `oracle/codemp/botlib/be_ai_weight.cpp:31`
pub const EVALUATERECURSIVELY: bool = true;
