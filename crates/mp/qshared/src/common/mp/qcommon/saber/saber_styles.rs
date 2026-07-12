//! MP `saber_styles_t`.
//!
//! Type definition source: `oracle/codemp/game/q_shared.h:671-683`

#![allow(non_camel_case_types)]

/// Raven `saber_styles_t` — fast, medium, strong, etc.
///
/// `typedef enum` → int-wide discriminants.
/// Type definition source: `oracle/codemp/game/q_shared.h:671-683`
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
