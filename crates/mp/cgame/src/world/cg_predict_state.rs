//! `CgPredictState` — `cg_predict.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_abi::cgame::public::snapshot_t::MAX_ENTITIES_IN_SNAPSHOT;
use mp_bg::public::pmove_t::{pmove_t, MAXTOUCH};
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::shared::{qboolean, qfalse};

/// `cg_predict.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// The two `centity_t *` snapshot sublists become entity-number lists (§B5:
/// entities by index, no aliasing pointers into the owned arena).
///
/// Source: `oracle/codemp/cgame/cg_predict.c:10-15,849-856,878,959-960`
#[derive(Debug)]
pub struct CgPredictState {
    /// Raven `static pmove_t cg_pmove` — the local-prediction move block. It
    /// persists between frames: `CG_PredictPlayerState` only re-adds the foot
    /// bolts when `cg_pmove.ghoul2` changed, so the `NULL` that
    /// `CG_PmoveClientPointerUpdate` stores is observable.
    /// Source: `oracle/codemp/cgame/cg_predict.c:10`
    pub cg_pmove: pmove_t,

    /// Raven `static int cg_numSolidEntities`.
    /// Source: `oracle/codemp/cgame/cg_predict.c:12`
    pub cg_numSolidEntities: c_int,
    /// Raven `static centity_t *cg_solidEntities[MAX_ENTITIES_IN_SNAPSHOT]`,
    /// as entity numbers.
    /// Source: `oracle/codemp/cgame/cg_predict.c:13`
    pub cg_solidEntities: [c_int; MAX_ENTITIES_IN_SNAPSHOT],
    /// Raven `static int cg_numTriggerEntities`.
    /// Source: `oracle/codemp/cgame/cg_predict.c:14`
    pub cg_numTriggerEntities: c_int,
    /// Raven `static centity_t *cg_triggerEntities[MAX_ENTITIES_IN_SNAPSHOT]`,
    /// as entity numbers.
    /// Source: `oracle/codemp/cgame/cg_predict.c:15`
    pub cg_triggerEntities: [c_int; MAX_ENTITIES_IN_SNAPSHOT],

    /// Raven `pmove_t cg_vehPmove` — the piloted-vehicle prediction move
    /// block; its one-time field setup is latched by `cg_vehPmoveSet`.
    /// Source: `oracle/codemp/cgame/cg_predict.c:959`
    pub cg_vehPmove: pmove_t,
    /// Raven `qboolean cg_vehPmoveSet`.
    /// Source: `oracle/codemp/cgame/cg_predict.c:960`
    pub cg_vehPmoveSet: qboolean,
}

impl Default for CgPredictState {
    /// Raven's BSS start: every one of these is a file-scope static, so the C
    /// module boots with the whole set zero/NULL. Hand-written rather than
    /// derived because `pmove_t` carries raw pointers and the two sublists are
    /// longer than the 32-element array `Default` impls.
    /// Source: `oracle/codemp/cgame/cg_predict.c:10-15`
    fn default() -> Self {
        Self {
            cg_pmove: zeroed_pmove(),
            cg_numSolidEntities: 0,
            cg_solidEntities: [0; MAX_ENTITIES_IN_SNAPSHOT],
            cg_numTriggerEntities: 0,
            cg_triggerEntities: [0; MAX_ENTITIES_IN_SNAPSHOT],
            cg_vehPmove: zeroed_pmove(),
            cg_vehPmoveSet: qfalse,
        }
    }
}

/// The all-zero/NULL BSS start both move blocks boot from.
fn zeroed_pmove() -> pmove_t {
    pmove_t {
        ps: null_mut(),
        ghoul2: null_mut(),
        g2Bolts_LFoot: 0,
        g2Bolts_RFoot: 0,
        modelScale: [0.0; 3],
        nonHumanoid: qfalse,
        cmd: usercmd_t::default(),
        tracemask: 0,
        debugLevel: 0,
        noFootsteps: qfalse,
        gauntletHit: qfalse,
        framecount: 0,
        numtouch: 0,
        touchents: [0; MAXTOUCH],
        useEvent: 0,
        mins: [0.0; 3],
        maxs: [0.0; 3],
        watertype: 0,
        waterlevel: 0,
        gametype: 0,
        debugMelee: 0,
        stepSlideFix: 0,
        noSpecMove: 0,
        animations: null_mut(),
        xyspeed: 0.0,
        pmove_fixed: 0,
        pmove_msec: 0,
        trace: None,
        pointcontents: None,
        checkDuelLoss: 0,
        baseEnt: null_mut(),
        entSize: 0,
    }
}
