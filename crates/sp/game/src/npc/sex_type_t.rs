#![allow(non_camel_case_types, non_snake_case)]

/// Raven `sexType_t` — character sex type enumeration.
///
/// Type definition source: `oracle/oracle/code/game/b_public.h:106-112`
#[repr(i32)]
pub enum sexType_t {
    SEX_NEUTRAL = 0,
    SEX_MALE,
    SEX_FEMALE,
    SEX_SHEMALE, // what the Hell, ya never know...
}
