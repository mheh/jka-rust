//! SP `saberType_t`.
//!
//! Type definition source: `oracle/oracle/code/game/q_shared.h:1561-1577`

#![allow(non_camel_case_types)]

/// Raven SP `saberType_t` — identical to MP.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1561-1577`
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
