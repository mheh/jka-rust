//! SP `bg_public.h` powerup definitions.
//!
//! Type definition source: `oracle/code/game/bg_public.h:248-267`

#![allow(non_camel_case_types)]

/// Raven SP `powerup_t`.
///
/// Unlike MP (`typedef int powerup_t` + an anonymous value-space enum), SP's
/// `powerup_t` is a **named** enum, and its member set diverges heavily from MP
/// (e.g. `PW_HASTE`, `PW_UNCLOAKING`, `PW_DISRUPTION`, `PW_GALAK_SHIELD`,
/// `PW_SEEKER`, `PW_SHOCKED`, `PW_DRAINED`, `PW_INVINCIBLE`, `PW_FORCE_PUSH*`
/// replace MP's flag/force-power powerups).
/// Type definition source: `oracle/code/game/bg_public.h:248-267`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum powerup_t {
    PW_NONE = 0,
    /// Raven: This can go away.
    PW_QUAD,
    PW_BATTLESUIT,
    /// Raven: This can go away.
    PW_HASTE,
    PW_CLOAKED,
    PW_UNCLOAKING,
    PW_DISRUPTION,
    PW_GALAK_SHIELD,
    PW_SEEKER,
    /// Raven: electricity effect.
    PW_SHOCKED,
    /// Raven: drain effect.
    PW_DRAINED,
    /// Raven: ghost.
    PW_DISINT_2,
    PW_INVINCIBLE,
    PW_FORCE_PUSH,
    PW_FORCE_PUSH_RHAND,

    PW_NUM_POWERUPS,
}

const _: () = assert!(core::mem::size_of::<powerup_t>() == 4);
