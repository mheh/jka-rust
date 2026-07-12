#![allow(non_camel_case_types)]

//! `be_aas_routealt.cpp`-local constants.
//!
//! Source: `oracle/codemp/botlib/be_aas_routealt.cpp:29`

/// Raven `ENABLE_ALTROUTING` — unconditionally-defined feature guard for the
/// alternate-route computation path. Ported as `bool` since Raven never
/// gives it a value, only tests it with `#ifdef`, and it is defined
/// unconditionally at this site.
///
/// Source: `oracle/codemp/botlib/be_aas_routealt.cpp:29`
pub const ENABLE_ALTROUTING: bool = true;
