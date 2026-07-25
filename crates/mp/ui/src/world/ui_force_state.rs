//! `UiForceState` — `ui_force.c`'s file-scope globals as one `UiWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::force_mastery::FORCE_MASTERY_JEDI_KNIGHT;
use mp_qshared::common::mp::qcommon::saber::saber_colors::NUM_SABER_COLORS;
use mp_qshared::shared::force_powers::{
    FORCE_DARKSIDE, FORCE_LIGHTSIDE, MAX_FORCE_RANK, NUM_FORCE_POWERS,
};
use mp_qshared::shared::qhandle_t;

/// Raven `#define NUM_FORCE_STAR_IMAGES 9`.
///
/// Source: `oracle/codemp/ui/ui_force.h:3`
pub const NUM_FORCE_STAR_IMAGES: usize = 9;

/// The force-allocation screen's state — Raven's free-floating `ui_force.c`
/// globals, folded onto `UiWorld` because they are ui state that sits outside
/// `uiInfo_t` only by file organisation (DEC-36 D1).
///
/// PORT-NOTE: `uiForcePowersDisabled`, `uiForcePowersRank`,
/// `uiForcePowerDarkLight` and `gCustPowersRank` are declared non-`const` and
/// seeded with the tables below; the first two are rewritten every time the
/// player edits a template, so all four stay state rather than becoming
/// `const`s.
///
/// Source: `oracle/codemp/ui/ui_force.c:15-98,1081-1103`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiForceState {
    /// Raven `int uiForceSide`.
    /// Source: `oracle/codemp/ui/ui_force.c:15`
    pub uiForceSide: c_int,
    /// Raven `int uiJediNonJedi`.
    /// Source: `oracle/codemp/ui/ui_force.c:16`
    pub uiJediNonJedi: c_int,
    /// Raven `int uiForceRank`.
    /// Source: `oracle/codemp/ui/ui_force.c:17`
    pub uiForceRank: c_int,
    /// Raven `int uiMaxRank`.
    /// Source: `oracle/codemp/ui/ui_force.c:18`
    pub uiMaxRank: c_int,
    /// Raven `int uiMaxPoints`.
    /// Source: `oracle/codemp/ui/ui_force.c:19`
    pub uiMaxPoints: c_int,
    /// Raven `int uiForceUsed`.
    /// Source: `oracle/codemp/ui/ui_force.c:20`
    pub uiForceUsed: c_int,
    /// Raven `int uiForceAvailable`.
    /// Source: `oracle/codemp/ui/ui_force.c:21`
    pub uiForceAvailable: c_int,

    /// Raven `qboolean gTouchedForce`.
    /// Source: `oracle/codemp/ui/ui_force.c:25`
    pub gTouchedForce: bool,

    /// Raven `qboolean uiForcePowersDisabled[NUM_FORCE_POWERS]`.
    /// Source: `oracle/codemp/ui/ui_force.c:32-51`
    pub uiForcePowersDisabled: [bool; NUM_FORCE_POWERS as usize],
    /// Raven `int uiForcePowersRank[NUM_FORCE_POWERS]`.
    /// Source: `oracle/codemp/ui/ui_force.c:53-72`
    pub uiForcePowersRank: [c_int; NUM_FORCE_POWERS as usize],
    /// Raven `int uiForcePowerDarkLight[NUM_FORCE_POWERS]` — 0 == neutral.
    /// Source: `oracle/codemp/ui/ui_force.c:74-95`
    pub uiForcePowerDarkLight: [c_int; NUM_FORCE_POWERS as usize],

    /// Raven `int uiForceStarShaders[NUM_FORCE_STAR_IMAGES][2]`.
    /// Source: `oracle/codemp/ui/ui_force.c:97`
    pub uiForceStarShaders: [[qhandle_t; 2]; NUM_FORCE_STAR_IMAGES],
    /// Raven `int uiSaberColorShaders[NUM_SABER_COLORS]`.
    /// Source: `oracle/codemp/ui/ui_force.c:98`
    pub uiSaberColorShaders: [qhandle_t; NUM_SABER_COLORS as usize],

    /// Raven `int gCustRank` — the rank the loaded force template carries.
    /// Source: `oracle/codemp/ui/ui_force.c:1081`
    pub gCustRank: c_int,
    /// Raven `int gCustSide`.
    /// Source: `oracle/codemp/ui/ui_force.c:1082`
    pub gCustSide: c_int,
    /// Raven `int gCustPowersRank[NUM_FORCE_POWERS]`.
    /// Source: `oracle/codemp/ui/ui_force.c:1084-1103`
    pub gCustPowersRank: [c_int; NUM_FORCE_POWERS as usize],
}

impl Default for UiForceState {
    /// Raven's static initializers (`ui_force.c:15-98,1081-1103`).
    fn default() -> Self {
        UiForceState {
            uiForceSide: FORCE_LIGHTSIDE,
            uiJediNonJedi: -1,
            uiForceRank: FORCE_MASTERY_JEDI_KNIGHT,
            uiMaxRank: MAX_FORCE_RANK,
            uiMaxPoints: 20,
            uiForceUsed: 0,
            uiForceAvailable: 0,
            gTouchedForce: false,
            uiForcePowersDisabled: [false; NUM_FORCE_POWERS as usize],
            uiForcePowersRank: [
                0, // FP_HEAL
                1, // FP_LEVITATION — this one defaults to 1 (gives a free point)
                0, // FP_SPEED
                0, // FP_PUSH
                0, // FP_PULL
                0, // FP_TELEPATHY
                0, // FP_GRIP
                0, // FP_LIGHTNING
                0, // FP_RAGE
                0, // FP_PROTECT
                0, // FP_ABSORB
                0, // FP_TEAM_HEAL
                0, // FP_TEAM_FORCE
                0, // FP_DRAIN
                0, // FP_SEE
                1, // FP_SABER_OFFENSE — default to 1 point in attack
                1, // FP_SABER_DEFENSE — defualt to 1 point in defense
                0, // FP_SABERTHROW
            ],
            // nothing should be usable at rank 0..
            uiForcePowerDarkLight: [
                FORCE_LIGHTSIDE, // FP_HEAL
                0,               // FP_LEVITATION
                0,               // FP_SPEED
                0,               // FP_PUSH
                0,               // FP_PULL
                FORCE_LIGHTSIDE, // FP_TELEPATHY
                FORCE_DARKSIDE,  // FP_GRIP
                FORCE_DARKSIDE,  // FP_LIGHTNING
                FORCE_DARKSIDE,  // FP_RAGE
                FORCE_LIGHTSIDE, // FP_PROTECT
                FORCE_LIGHTSIDE, // FP_ABSORB
                FORCE_LIGHTSIDE, // FP_TEAM_HEAL
                FORCE_DARKSIDE,  // FP_TEAM_FORCE
                FORCE_DARKSIDE,  // FP_DRAIN
                0,               // FP_SEE
                0,               // FP_SABER_OFFENSE
                0,               // FP_SABER_DEFENSE
                0,               // FP_SABERTHROW
            ],
            uiForceStarShaders: [[0; 2]; NUM_FORCE_STAR_IMAGES],
            uiSaberColorShaders: [0; NUM_SABER_COLORS as usize],
            gCustRank: 0,
            gCustSide: 0,
            gCustPowersRank: [
                0, // FP_HEAL
                1, // FP_LEVITATION — this one defaults to 1 (gives a free point)
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }
}
