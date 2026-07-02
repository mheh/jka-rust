#![allow(non_camel_case_types, non_snake_case)]

/// Raven `awardType_t` — award type enumeration.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:1064-1071`
#[repr(i32)]
pub enum awardType_t {
    AWARD_ACCURACY,
    AWARD_IMPRESSIVE,
    AWARD_EXCELLENT,
    AWARD_GAUNTLET,
    AWARD_FRAGS,
    AWARD_PERFECT,
}
