//! SP `saber_styles_t`.
//!
//! Type definition source: `oracle/code/game/q_shared.h:1660-1671`

#![allow(non_camel_case_types)]

/// Raven SP `saber_styles_t` — identical to MP.
///
/// Type definition source: `oracle/code/game/q_shared.h:1660-1671`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum saber_styles_t {
    SS_NONE = 0,
    SS_FAST,
    SS_MEDIUM,
    SS_STRONG,
    SS_DESANN,
    SS_TAVION,
    SS_DUAL,
    SS_STAFF,
    SS_NUM_SABER_STYLES,
}
