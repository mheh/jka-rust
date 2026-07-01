//! MP `saberType_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/q_shared.h:601-631`

#![allow(non_camel_case_types)]

/// Raven `saberType_t` — none, single, staff, etc.
///
/// `typedef enum` → int-wide discriminants.
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:601-631`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum saberType_t {
    SABER_NONE = 0,
    SABER_SINGLE,
    SABER_STAFF,
    SABER_DAGGER,
    SABER_BROAD,
    SABER_PRONG,
    SABER_ARC,
    SABER_SAI,
    SABER_CLAW,
    SABER_LANCE,
    SABER_STAR,
    SABER_TRIDENT,
    SABER_SITH_SWORD,
    NUM_SABERS,
}
