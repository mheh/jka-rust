#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EMatImpactEffect` — material impact effect enumeration.
///
/// Raven: (no comment).
/// Type definition source: `oracle/codemp/client/FxPrimitives.h:101-105`
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum EMatImpactEffect {
    #[default]
    MATIMPACTFX_NONE = 0,
    MATIMPACTFX_SHELLSOUND,
}
