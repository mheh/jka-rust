// PORT-COMPLETE: g_navnew.c 5/8
//! Port of `oracle/oracle/codemp/game/g_navnew.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `NPC_reactions.rs`/`w_force.rs`): logic fns that reach `level`/cvars/traps
//! thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`, `.engine`)
//! as an ADDITIVE first parameter (the faithful C signature carries none).
//! `level` -> `(*ctx.world).level`, cvars -> `(*ctx.world).cvars`. Traps go
//! through `trap::X(ctx.engine, ..)`. `vec3_t` (`[f32;3]`) is `Copy`, so
//! `VectorCopy`/`VectorMA`/`VectorScale`/`VectorAdd`/`VectorSubtract`/
//! `VectorClear`/`VectorSet`/`DotProduct`/`VectorCompare` (all inline macros
//! in the oracle) transcribe as plain array arithmetic per the bless-the-rule
//! appendix, not function calls. Raw `gentity_t*` chains are `unsafe`
//! raw-pointer field access mirroring the C exactly (`gentity_t::NPC`/
//! `::client` are opaque `*mut c_void`, cast to `gNPC_t`/`gclient_t` per the
//! `NPC_reactions.rs` precedent).
//!
//! `NAVNEW_DanceWithBlocker`/`NAVNEW_SidestepBlocker`'s faithful `movedir`
//! out-param cannot be received as a by-value `vec3_t` (`vec3-outparam-seam`,
//! `g_combat.rs` precedent) — but unlike that cross-file precedent, these
//! `movedir` out-params are OWNED by this same file (not a frozen callee
//! signature), so per §C7 (out-params -> return values / mutable refs) they
//! are declared `movedir: &mut vec3_t` here instead of parking.
//!
//! PARKED (see PORT-ESCALATION markers): `NPC_SetBlocked`,
//! `NAVNEW_PushBlocker`, `NAVNEW_SidestepBlocker`, `NAVNEW_Bypass`,
//! `NAVNEW_ResolveEntityCollision`, `NAVNEW_AvoidCollision`,
//! `NAVNEW_TestNodeConnectionBlocked`, `NAVNEW_MoveToGoal` all read JKA-added
//! `CONTENTS_*`/`MASK_NPCSOLID` collision-mask bits, `NIF_*`/`EDGE_*`/`NODE_*`
//! debug-draw enum values, or magic-number defines (`MIN_BLOCKED_SPEECH_TIME`,
//! `MIN_DOOR_BLOCK_DIST_SQR`, `DEFAULT_MINS_2`/`DEFAULT_MAXS_2`,
//! `NPCAI_BLOCKED`) that are not resolved anywhere in this packet and are not
//! confidently re-derivable the way `ENTITYNUM_NONE`/`STEPSIZE` are (topic
//! `const-value`); `NAVNEW_MoveToGoal` additionally reaches the ambient
//! "current NPC" global `NPCInfo` with no `GameContext` channel (topic
//! `ai-context`, matching `NPC_reactions.rs`/`NPC_utils.rs`).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_qshared::common::mp::gentity::MAX_FAILED_NODES;
use mp_qshared::shared::MAX_GENTITIES;
use mp_qshared::shared::{QFALSE, QTRUE};

use crate::q_math::VectorNormalize;

/// Raven `ENTITYNUM_NONE`/`ENTITYNUM_WORLD` — derived from `MAX_GENTITIES`,
/// an invariant unchanged across every id-tech-3 engine build including JKA.
/// Source: `oracle/oracle/codemp/game/q_shared.h` (ENTITYNUM_NONE/ENTITYNUM_WORLD defines)
const ENTITYNUM_NONE: c_int = (MAX_GENTITIES - 1) as c_int;
const ENTITYNUM_WORLD: c_int = (MAX_GENTITIES - 2) as c_int;

/// Raven `NAV_CheckNodeFailedForEnt`.
///
/// Raven: "FIXME: must be a better way to do this". `+1` because 0 is a valid
/// nodeNum but also the default (unset) slot value.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:15-28`
pub fn NAV_CheckNodeFailedForEnt(ent: *mut gentity_t, nodeNum: c_int) -> qboolean {
    unsafe {
        for j in 0..MAX_FAILED_NODES {
            if (*ent).failedWaypoints[j] == nodeNum + 1 {
                //we failed against this node
                return QTRUE;
            }
        }
    }
    QFALSE
}

/// Raven `NPC_ClearBlocked`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:34-41`
pub fn NPC_ClearBlocked(self_: *mut gentity_t) {
    unsafe {
        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            return;
        }
        //self->NPC->aiFlags &= ~NPCAI_BLOCKED;
        (*npc).blockingEntNum = ENTITYNUM_NONE;
    }
}

// PORT-ESCALATION(const-value): reads `MIN_BLOCKED_SPEECH_TIME` (a JKA
// `b_local.h`-family time-interval define with no resolved value anywhere in
// this packet) plus `random()`; guessing the interval would silently corrupt
// NPC blocked-speech debounce timing.
/// Raven `NPC_SetBlocked`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:43-51`
pub fn NPC_SetBlocked(
    ctx: GameContext<'_>,self_: *mut gentity_t, blocker: *mut gentity_t) {
    todo!("Port NPC_SetBlocked — parked: const-value")
}

/// Raven `NAVNEW_ClearPathBetweenPoints`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:58-77`
pub fn NAVNEW_ClearPathBetweenPoints(
    ctx: GameContext<'_>,
    start: vec3_t,
    end: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    ignore: c_int,
    clipmask: c_int,
) -> c_int {
    unsafe {
        //Test if they're even conceivably close to one another
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(&start as *const vec3_t, &end as *const vec3_t),
        ) == QFALSE
        {
            return ENTITYNUM_WORLD;
        }

        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut trace as *mut trace_t,
                &start as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &end as *const vec3_t,
                ignore,
                clipmask,
            ),
        );

        //if( ( ( trace.startsolid == false ) && ( trace.allsolid == false ) ) && ( trace.fraction < 1.0f ) )
        //{//FIXME: check for drops?
        //FIXME: if startsolid or allsolid, then the path isn't clear... but returning ENTITYNUM_NONE indicates to CheckFailedEdge that is is clear...?
        trace.entityNum as c_int
        //}

        //return ENTITYNUM_NONE;
    }
}

// PORT-ESCALATION(const-value): reads `CONTENTS_BOTCLIP` (a JKA-added
// content-mask bit not resolved anywhere in this packet, not a stock
// id-tech-3 constant) via `blocker->clipmask|CONTENTS_BOTCLIP`; guessing the
// bit would silently corrupt collision-mask parity.
/// Raven `NAVNEW_PushBlocker`.
///
/// Raven: try pushing blocker to one side.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:84-171`
pub fn NAVNEW_PushBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    right: vec3_t,
    setBlockedInfo: qboolean,
) {
    todo!("Port NAVNEW_PushBlocker — parked: const-value")
}

/// Raven `NAVNEW_DanceWithBlocker`.
///
/// Raven: sees if blocker has any lateral movement.
///
/// The faithful `movedir` out-param is declared `&mut vec3_t` (§C7) rather
/// than by-value — this is this file's own signature, not a frozen cross-file
/// callee, so unlike the `vec3-outparam-seam` park precedent (`g_combat.rs`)
/// it is fixed here rather than parked.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:178-215`
pub fn NAVNEW_DanceWithBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    unsafe {
        let client = (*blocker).client as *mut gclient_t;
        if !client.is_null() && (*client).ps.velocity != [0.0f32, 0.0, 0.0] {
            let mut blocker_movedir = (*client).ps.velocity;
            blocker_movedir[2] = 0.0; //cancel any vertical motion
            let dot = blocker_movedir[0] * right[0]
                + blocker_movedir[1] * right[1]
                + blocker_movedir[2] * right[2];
            if dot > 50.0 {
                //he's moving to the right of me at a relatively good speed
                //go to my left
                movedir[0] += -1.0 * right[0];
                movedir[1] += -1.0 * right[1];
                movedir[2] += -1.0 * right[2];
                VectorNormalize(movedir);
                return QTRUE;
            } else if dot > -50.0 {
                //he's moving to the left of me at a relatively good speed
                //go to my right
                movedir[0] += right[0];
                movedir[1] += right[1];
                movedir[2] += right[2];
                VectorNormalize(movedir);
                return QTRUE;
            }
            /*
            vec3_t	block_pos;
            trace_t	tr;
            VectorScale( blocker_movedir, -1, blocker_movedir );
            VectorMA( self->r.currentOrigin, blocked_dist, blocker_movedir, block_pos );
            if ( NAVNEW_CheckAhead( self, block_pos, tr, ( self->clipmask & ~CONTENTS_BODY )|CONTENTS_BOTCLIP ) )
            {
                VectorCopy( blocker_movedir, movedir );
                return qtrue;
            }
            */
        }
    }
    QFALSE
}

// PORT-ESCALATION(const-value): reads `CONTENTS_BOTCLIP` (JKA-added
// content-mask bit, not resolved in this packet) via
// `self->clipmask|CONTENTS_BOTCLIP` at every `trap_Trace` call; guessing the
// bit would silently corrupt collision-mask parity.
/// Raven `NAVNEW_SidestepBlocker`.
///
/// Raven: trace to sides of blocker and see if either is clear.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:222-340`
pub fn NAVNEW_SidestepBlocker(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    todo!("Port NAVNEW_SidestepBlocker — parked: const-value")
}

// PORT-ESCALATION(const-value): reads `NAVDEBUG_showCollision` (unresolved
// file-scope debug toggle, no GameWorld field ruled for it here) and
// `EDGE_NORMAL` (a `g_nav.c` debug-draw enum value not resolved in this
// packet); guessing the enum ordinal would silently corrupt debug rendering
// call parity, and none of the useful behavior in this fn is reachable
// without the two blocked-out calls it wraps.
/// Raven `NAVNEW_Bypass`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:347-377`
pub fn NAVNEW_Bypass(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    todo!("Port NAVNEW_Bypass — parked: const-value")
}

/// Raven `NAVNEW_CheckDoubleBlock`.
///
/// Raven: stop double waiting.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:384-391`
pub fn NAVNEW_CheckDoubleBlock(
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    blocked_dir: vec3_t,
) -> qboolean {
    unsafe {
        let npc = (*blocker).NPC as *mut gNPC_t;
        if !npc.is_null() && (*npc).blockingEntNum == (*self_).s.number {
            return QTRUE;
        }
    }
    QFALSE
}

// PORT-ESCALATION(const-value): reads `MIN_DOOR_BLOCK_DIST_SQR` (a
// `g_navnew.c`-local magic-number define not resolved anywhere in this
// packet); guessing the distance-squared threshold would silently corrupt
// door-blocker parity.
/// Raven `NAVNEW_ResolveEntityCollision`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:399-435`
pub fn NAVNEW_ResolveEntityCollision(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    blocker: *mut gentity_t,
    movedir: vec3_t,
    pathDir: vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    todo!("Port NAVNEW_ResolveEntityCollision — parked: const-value")
}

// PORT-ESCALATION(const-value): reads `navInfo_t`'s `NIF_COLLISION` flag bit
// and `EDGE_MOVEDIR` debug-draw enum value (both unresolved in this packet);
// guessing either would silently corrupt `info->flags` parity for every
// caller of `NAVNEW_MoveToGoal`.
/// Raven `NAVNEW_AvoidCollision`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:442-518`
pub fn NAVNEW_AvoidCollision(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    goal: *mut gentity_t,
    info: *mut navInfo_t,
    setBlockedInfo: qboolean,
    blockedMovesLimit: c_int,
) -> qboolean {
    todo!("Port NAVNEW_AvoidCollision — parked: const-value")
}

// PORT-ESCALATION(const-value): reads `MASK_NPCSOLID`, `CONTENTS_BOTCLIP`,
// `CONTENTS_SOLID`, `CONTENTS_MONSTERCLIP`, `CONTENTS_BODY`,
// `DEFAULT_MINS_2`/`DEFAULT_MAXS_2` (JKA content-mask bits and default NPC
// bbox constants, none resolved in this packet); guessing any of these
// bitmasks or bbox extents would silently corrupt the clipmask/box built for
// `trap_Trace`.
/// Raven `NAVNEW_TestNodeConnectionBlocked`.
///
/// Raven: see if the direct path between 2 nodes is blocked by architecture
/// or an ent.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:520-572`
pub fn NAVNEW_TestNodeConnectionBlocked(
    ctx: GameContext<'_>,
    wp1: c_int,
    wp2: c_int,
    ignoreEnt: *mut gentity_t,
    goalEntNum: c_int,
    checkWorld: qboolean,
    checkEnts: qboolean,
) -> qboolean {
    todo!("Port NAVNEW_TestNodeConnectionBlocked — parked: const-value")
}

// PORT-ESCALATION(ai-context): reads the ambient "current NPC" global
// `NPCInfo` (`NPCInfo->blockedDest`) that Raven's `ai_main.c` think-loop sets
// per NPC frame — no `GameWorld`/`GameContext` field carries it and no entity
// param substitutes (same unresolved fork as `NPC_reactions.rs`/
// `NPC_utils.rs`'s `ai-context` sites). Also reads `NPCAI_BLOCKED`,
// `NF_CLEAR_PATH`, `WAYPOINT_NONE`/`NODE_NONE`, `NAVDEBUG_showEnemyPath`,
// `NODE_START`/`NODE_NAVGOAL`/`NODE_GOAL`, `EDGE_PATH` — none resolved in this
// packet (topic `const-value`, folded into the same park since the
// `ai-context` gap alone already blocks a faithful body).
/// Raven `NAVNEW_MoveToGoal`.
///
/// Source: `oracle/oracle/codemp/game/g_navnew.c:578-865`
pub fn NAVNEW_MoveToGoal(
    ctx: GameContext<'_>,self_: *mut gentity_t, info: *mut navInfo_t) -> c_int {
    todo!("Port NAVNEW_MoveToGoal — parked: ai-context")
}
