#![allow(non_camel_case_types)]

/// Raven `genCmds_t` — generic (button-driven) client commands.
///
/// SP-vs-MP: SP's list is much shorter and differently ordered than MP's (no
/// saber/duel/item commands); it runs `GENCMD_FORCE_HEAL == 1` through
/// `GENCMD_FORCE_SEEING == 12`.
///
/// Type definition source: `oracle/code/game/q_shared.h:2389-2403`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum genCmds_t {
    GENCMD_FORCE_HEAL = 1,
    GENCMD_FORCE_SPEED,
    GENCMD_FORCE_THROW,
    GENCMD_FORCE_PULL,
    GENCMD_FORCE_DISTRACT,
    GENCMD_FORCE_GRIP,
    GENCMD_FORCE_LIGHTNING,
    GENCMD_FORCE_RAGE,
    GENCMD_FORCE_PROTECT,
    GENCMD_FORCE_ABSORB,
    GENCMD_FORCE_DRAIN,
    GENCMD_FORCE_SEEING,
}
