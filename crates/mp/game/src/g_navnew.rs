// PORT-COMPLETE: g_navnew.c
//! Port of `oracle/codemp/game/g_navnew.c` (jampgame mega-pass).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (fork rulings 1/4 + `docs/architecture/engine-seam.md`, precedent
//! `NPC_reactions.rs`/`w_force.rs`): logic fns that reach `level`/cvars/traps
//! thread the `GameContext<'_>` receiver (`.world: *mut GameWorld`, `.engine`)
//! as an ADDITIVE first parameter (the faithful C signature carries none).
//! `level` -> `ctx.world.level`, cvars -> `ctx.world.cvars`. Traps go
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
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`.
//!
//! Safe-state **campaign 2c** (task #7): the Stage-1 fn-top raw re-derives are
//! retired — entity fields read/write through `ctx.world.entity(id)` /
//! `entity_mut(id)` at the point of use; seam `.cast()` entity-pointer passes to
//! `trap_Nav_*` derive their raw `*mut gentity_t` locally at the call site. The
//! only raw derefs left are the sanctioned ones, each FLAGged inline: gNPC_t
//! (`self->NPC`/NPCInfo, no safe accessor), the NPC pool `gclient_t`
//! (`blocker->client`, `gClPtrs` — not a `level.clients` slot), and the
//! caller-owned `*mut navInfo_t` scratch pointer. Callers bridge stored raw
//! pointers at the boundary via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_nav::{G_DrawEdge, G_DrawNode, NAV_CheckAhead, NAV_TestBestNode, NAV_TestForBlocked};
use crate::prelude::*;
use crate::trap;
use crate::world::GameContext;

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_qshared::common::mp::gentity::MAX_FAILED_NODES;
use mp_qshared::shared::MAX_GENTITIES;

use crate::q_math::VectorNormalize;

// `ENTITYNUM_NONE`/`ENTITYNUM_WORLD` resolve via the crate prelude glob
// (`mp_qshared::shared::limits`); the shadowing local copies were removed by
// the placeholder-const sweep.

// `DEFAULT_MINS_2`/`DEFAULT_MAXS_2` canonical in `mp_bg::public::viewheight`
// (`c_int`, cast here to match the `vec3_t` components they seed).
// Source: `oracle/codemp/game/bg_public.h:41-42`
const DEFAULT_MINS_2: f32 = mp_bg::public::viewheight::DEFAULT_MINS_2 as f32;
const DEFAULT_MAXS_2: f32 = mp_bg::public::viewheight::DEFAULT_MAXS_2 as f32;

/// Raven `NAV_CheckNodeFailedForEnt`.
///
/// Raven: "FIXME: must be a better way to do this". `+1` because 0 is a valid
/// nodeNum but also the default (unset) slot value.
///
/// Source: `oracle/codemp/game/g_navnew.c:15-28`
pub fn NAV_CheckNodeFailedForEnt(ent: &gentity_t, nodeNum: c_int) -> qboolean {
    for j in 0..MAX_FAILED_NODES {
        if ent.failedWaypoints[j] == nodeNum + 1 {
            //we failed against this node
            return qtrue;
        }
    }
    qfalse
}

/// Raven `NPC_ClearBlocked`.
///
/// Source: `oracle/codemp/game/g_navnew.c:34-41`
pub fn NPC_ClearBlocked(self_: &mut gentity_t) {
    // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; read the pointer via
    // the safe field access and deref raw exactly as Raven does.
    let npc = self_.NPC;
    if npc.is_null() {
        return;
    }
    //self->NPC->aiFlags &= ~NPCAI_BLOCKED;
    unsafe {
        (*npc).blockingEntNum = ENTITYNUM_NONE;
    }
}

/// Raven `NPC_SetBlocked`.
///
/// Source: `oracle/codemp/game/g_navnew.c:43-51`
pub fn NPC_SetBlocked(ctx: &mut GameContext, self_: EntityId, blocker: Option<EntityId>) {
    // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; read the pointer via
    // the safe borrow and deref raw exactly as Raven does.
    let npc = ctx.world.entity(self_).NPC;
    if npc.is_null() {
        return;
    }

    //self->NPC->aiFlags |= NPCAI_BLOCKED;
    // RHS reads level.time + draws RNG (order-preserving) before the raw writes
    // to the gNPC_t pool struct.
    let debounce = ctx.world.level.time
        + MIN_BLOCKED_SPEECH_TIME
        + ((ctx.world.bg_state.rng.random() * 4000.0) as c_int);
    let blocker_number = ctx.world.entity(blocker.unwrap()).s.number;
    unsafe {
        (*npc).blockedSpeechDebounceTime = debounce;
        (*npc).blockingEntNum = blocker_number;
    }
}

/// Raven `NAVNEW_ClearPathBetweenPoints`.
///
/// Source: `oracle/codemp/game/g_navnew.c:58-77`
pub fn NAVNEW_ClearPathBetweenPoints(
    ctx: &mut GameContext,
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
        ) == qfalse
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

/// Raven `NAVNEW_PushBlocker`.
///
/// Raven: try pushing blocker to one side.
///
/// Source: `oracle/codemp/game/g_navnew.c:84-171`
pub fn NAVNEW_PushBlocker(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    right: vec3_t,
    setBlockedInfo: qboolean,
) {
    let blocker = blocker.unwrap();
    // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; read the pointer via
    // the safe borrow and deref raw exactly as Raven does.
    let npc = ctx.world.entity(self_).NPC;
    if unsafe { (*npc).shoveCount } > 30 {
        //don't push for more than 3 seconds;
        return;
    }

    if ctx.world.entity(blocker).s.number == 0 {
        //never push the player
        return;
    }

    // FLAG (task #7): NPC pool `gclient_t` (`gClPtrs`, g_utils.c:430) — not a
    // `level.clients` slot; read the pointer via the safe borrow, deref raw.
    let client = ctx.world.entity(blocker).client;
    // Oracle: `!blocker->client || !VectorCompare(pushVec, vec3_origin)` — bail
    // when the blocker has no client OR is already being pushed elsewhere.
    if client.is_null() || !unsafe { VectorCompare((*client).pushVec, [0.0f32, 0.0, 0.0]) } {
        //someone else is pushing him, wait until they give up?
        return;
    }

    let mut mins = [0.0f32; 3];
    crate::q_math::_VectorCopy(ctx.world.entity(blocker).r.mins, &mut mins);
    mins[2] += STEPSIZE;

    // Raven: `(float sum) * 1.2` — the f32 maxs sum promotes to f64 for the
    // double literal, multiplied in f64 and narrowed once at the float store.
    // Source: `oracle/codemp/game/g_navnew.c:108`
    let moveamt = ((ctx.world.entity(self_).r.maxs[1] + ctx.world.entity(blocker).r.maxs[1]) as f64
        * 1.2) as f32; //yes, magic number

    let blocker_origin = ctx.world.entity(blocker).r.currentOrigin;
    let blocker_maxs = ctx.world.entity(blocker).r.maxs;
    let blocker_number = ctx.world.entity(blocker).s.number;
    let blocker_clipmask = ctx.world.entity(blocker).clipmask;

    let mut end = [0.0f32; 3];
    crate::q_math::_VectorMA(blocker_origin, -moveamt, right, &mut end);
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &blocker_origin as *const vec3_t,
            &mins as *const vec3_t,
            &blocker_maxs as *const vec3_t,
            &end as *const vec3_t,
            blocker_number,
            blocker_clipmask | CONTENTS_BOTCLIP,
        ),
    );
    let leftSucc = if tr.startsolid == 0 && tr.allsolid == 0 {
        tr.fraction
    } else {
        0.0f32
    };

    // SAFETY: `client` is the blocker's pool `gclient_t` read above; the writes
    // mirror Raven's `blocker->client->pushVec*` stores.
    unsafe {
        if leftSucc >= 1.0f32 {
            //it's clear, shove him that way
            crate::q_math::_VectorScale(right, -moveamt, &mut (*client).pushVec);
            (*client).pushVecTime = ctx.world.level.time + 2000;
        } else {
            crate::q_math::_VectorMA(blocker_origin, moveamt, right, &mut end);
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &blocker_origin as *const vec3_t,
                    &mins as *const vec3_t,
                    &blocker_maxs as *const vec3_t,
                    &end as *const vec3_t,
                    blocker_number,
                    blocker_clipmask | CONTENTS_BOTCLIP,
                ),
            );
            let rightSucc = if tr.startsolid == 0 && tr.allsolid == 0 {
                tr.fraction
            } else {
                0.0f32
            };

            if leftSucc == 0.0f32 && rightSucc == 0.0f32 {
                //both sides failed
                if ctx.world.cvars.d_patched.integer != 0 {
                    //use patch-style navigation
                    (*client).pushVecTime = 0;
                }
                return;
            }

            if rightSucc >= 1.0f32 {
                //it's clear, shove him that way
                crate::q_math::_VectorScale(right, moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = ctx.world.level.time + 2000;
            }
            //if neither are enough, we probably can't get around him, but keep trying
            else if leftSucc >= rightSucc {
                //favor the left, all things being equal
                crate::q_math::_VectorScale(right, -moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = ctx.world.level.time + 2000;
            } else {
                crate::q_math::_VectorScale(right, moveamt, &mut (*client).pushVec);
                (*client).pushVecTime = ctx.world.level.time + 2000;
            }
        }

        if setBlockedInfo != qfalse {
            //we tried pushing
            (*npc).shoveCount += 1;
        }
    }
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
/// Source: `oracle/codemp/game/g_navnew.c:178-215`
pub fn NAVNEW_DanceWithBlocker(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    blocker: Option<EntityId>,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    // `self_` is unused by Raven's body (kept for signature fidelity).
    let blocker = blocker.unwrap();
    // FLAG (task #7): NPC pool `gclient_t` (`gClPtrs`, g_utils.c:430) — not a
    // `level.clients` slot; read the pointer via the safe borrow, deref raw.
    let client = ctx.world.entity(blocker).client;
    unsafe {
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
                return qtrue;
            } else if dot > -50.0 {
                //he's moving to the left of me at a relatively good speed
                //go to my right
                movedir[0] += right[0];
                movedir[1] += right[1];
                movedir[2] += right[2];
                VectorNormalize(movedir);
                return qtrue;
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
    qfalse
}

/// Raven `NAVNEW_SidestepBlocker`.
///
/// Raven: trace to sides of blocker and see if either is clear.
///
/// Source: `oracle/codemp/game/g_navnew.c:222-340`
pub fn NAVNEW_SidestepBlocker(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    right: vec3_t,
) -> qboolean {
    let blocker = blocker.unwrap();
    // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; the pointer is read
    // via the safe borrow and dereffed raw exactly as Raven does.
    let npc = ctx.world.entity(self_).NPC;
    unsafe {
        let mut mins = [0.0f32; 3];
        crate::q_math::_VectorCopy(ctx.world.entity(self_).r.mins, &mut mins);
        mins[2] += STEPSIZE;

        //Get the blocked direction
        let yaw = mp_bg::bg_misc::vectoyaw(blocked_dir);

        //Get the avoid radius
        // Raven: `sqrt(a) + sqrt(b)` — the f32 products promote to f64 for libm
        // sqrt, summed in f64 and narrowed once at the float store.
        // Source: `oracle/codemp/game/g_navnew.c:236-237`
        let blocker_maxs = ctx.world.entity(blocker).r.maxs;
        let self_maxs = ctx.world.entity(self_).r.maxs;
        let avoidRadius = (((blocker_maxs[0] * blocker_maxs[0] + blocker_maxs[1] * blocker_maxs[1])
            as f64)
            .sqrt()
            + ((self_maxs[0] * self_maxs[0] + self_maxs[1] * self_maxs[1]) as f64).sqrt())
            as f32;

        //See if we're inside our avoidance radius
        let arcAngle = if blocked_dist <= avoidRadius {
            135.0f32
        } else {
            (avoidRadius / blocked_dist) * 90.0f32
        };

        let mut avoidAngles = [0.0f32; 3];

        //need to stop it from ping-ponging, so we have a bit of a debounce time on which side you try
        if (*npc).sideStepHoldTime > ctx.world.level.time {
            let adj_arcAngle = if (*npc).lastSideStepSide == -1 {
                -arcAngle
            } else {
                arcAngle
            };
            avoidAngles[1] = crate::q_math::AngleNormalize360(yaw + adj_arcAngle);
            let mut avoidRight_dir = [0.0f32; 3];
            crate::q_math::AngleVectors(avoidAngles, Some(&mut *movedir), None, None);
            let self_origin = ctx.world.entity(self_).r.currentOrigin;
            let mut block_pos = [0.0f32; 3];
            crate::q_math::_VectorMA(self_origin, blocked_dist, *movedir, &mut block_pos);
            let self_number = ctx.world.entity(self_).s.number;
            let self_clipmask = ctx.world.entity(self_).clipmask;
            let mut tr: trace_t = core::mem::zeroed();
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &self_origin as *const vec3_t,
                    &mins as *const vec3_t,
                    &self_maxs as *const vec3_t,
                    &block_pos as *const vec3_t,
                    self_number,
                    self_clipmask | CONTENTS_BOTCLIP,
                ),
            );
            return if tr.fraction == 1.0 && tr.allsolid == 0 && tr.startsolid == 0 {
                qtrue
            } else {
                qfalse
            };
        }

        let self_origin = ctx.world.entity(self_).r.currentOrigin;
        let self_number = ctx.world.entity(self_).s.number;
        let self_clipmask = ctx.world.entity(self_).clipmask;

        //test right
        avoidAngles[crate::q_math::YAW] = crate::q_math::AngleNormalize360(yaw + arcAngle);
        let mut avoidRight_dir = [0.0f32; 3];
        crate::q_math::AngleVectors(avoidAngles, Some(&mut avoidRight_dir), None, None);

        let mut block_pos = [0.0f32; 3];
        crate::q_math::_VectorMA(self_origin, blocked_dist, avoidRight_dir, &mut block_pos);

        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &self_origin as *const vec3_t,
                &mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &block_pos as *const vec3_t,
                self_number,
                self_clipmask | CONTENTS_BOTCLIP,
            ),
        );

        // Oracle computes rightSucc then ALWAYS runs the left trace and computes
        // leftSucc; the combined `rightSucc*dist>=avoidRadius || leftSucc*dist>=..`
        // partial-success check must run for both sides, so keep this flat.
        let rightSucc = if tr.allsolid == 0 && tr.startsolid == 0 {
            if tr.fraction >= 1.0f32 {
                //all clear, go for it (favor the right if both are equal)
                crate::q_math::_VectorCopy(avoidRight_dir, movedir);
                (*npc).lastSideStepSide = 1;
                (*npc).sideStepHoldTime = ctx.world.level.time + 2000;
                return qtrue;
            }
            tr.fraction
        } else {
            0.0f32
        };

        //now test left
        let arcAngle = -arcAngle;
        avoidAngles[crate::q_math::YAW] = crate::q_math::AngleNormalize360(yaw + arcAngle);
        let mut avoidLeft_dir = [0.0f32; 3];
        crate::q_math::AngleVectors(avoidAngles, Some(&mut avoidLeft_dir), None, None);

        crate::q_math::_VectorMA(self_origin, blocked_dist, avoidLeft_dir, &mut block_pos);

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &self_origin as *const vec3_t,
                &mins as *const vec3_t,
                &self_maxs as *const vec3_t,
                &block_pos as *const vec3_t,
                self_number,
                self_clipmask | CONTENTS_BOTCLIP,
            ),
        );

        let leftSucc = if tr.allsolid == 0 && tr.startsolid == 0 {
            if tr.fraction >= 1.0f32 {
                //all clear, go for it (right side would have already succeeded if as good as this)
                crate::q_math::_VectorCopy(avoidLeft_dir, movedir);
                (*npc).lastSideStepSide = -1;
                (*npc).sideStepHoldTime = ctx.world.level.time + 2000;
                return qtrue;
            }
            tr.fraction
        } else {
            0.0f32
        };

        if leftSucc == 0.0f32 && rightSucc == 0.0f32 {
            //both sides failed
            return qfalse;
        }

        if rightSucc * blocked_dist >= avoidRadius || leftSucc * blocked_dist >= avoidRadius {
            //the traces hit something, but got a relatively good distance
            if rightSucc >= leftSucc {
                //favor the right, all things being equal
                crate::q_math::_VectorCopy(avoidRight_dir, movedir);
                (*npc).lastSideStepSide = 1;
                (*npc).sideStepHoldTime = ctx.world.level.time + 2000;
            } else {
                crate::q_math::_VectorCopy(avoidLeft_dir, movedir);
                (*npc).lastSideStepSide = -1;
                (*npc).sideStepHoldTime = ctx.world.level.time + 2000;
            }
            return qtrue;
        }

        //if neither are enough, we probably can't get around him
        qfalse
    }
}

/// Raven `NAVNEW_Bypass`.
///
/// Source: `oracle/codemp/game/g_navnew.c:347-377`
pub fn NAVNEW_Bypass(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    blocked_dir: vec3_t,
    blocked_dist: f32,
    movedir: &mut vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    // `movedir` is threaded `&mut` (Raven's out-param): DanceWithBlocker /
    // SidestepBlocker write the avoid-direction back for the caller to copy.
    //Draw debug info if requested
    if ctx.world.globals.NAVDEBUG_showCollision != 0 {
        let self_origin = ctx.world.entity(self_).r.currentOrigin;
        let blocker_origin = ctx.world.entity(blocker.unwrap()).r.currentOrigin;
        G_DrawEdge(self_origin, blocker_origin, EDGE_NORMAL);
    }

    let mut moveangles = [0.0f32; 3];
    let mut right = [0.0f32; 3];
    crate::q_math::vectoangles(*movedir, &mut moveangles);
    moveangles[2] = 0.0;
    crate::q_math::AngleVectors(moveangles, None, Some(&mut right), None);

    //Check to see what dir the other guy is moving in (if any) and pick the opposite dir
    if NAVNEW_DanceWithBlocker(ctx, Some(self_), blocker, movedir, right) != qfalse {
        return qtrue;
    }

    //Okay, so he's not moving to my side, see which side of him is most clear
    if NAVNEW_SidestepBlocker(
        ctx,
        self_,
        blocker,
        blocked_dir,
        blocked_dist,
        movedir,
        right,
    ) != qfalse
    {
        return qtrue;
    }

    //Neither side is clear, tell him to step aside
    NAVNEW_PushBlocker(ctx, self_, blocker, right, setBlockedInfo);

    qfalse
}

/// Raven `NAVNEW_CheckDoubleBlock`.
///
/// Raven: stop double waiting.
///
/// Source: `oracle/codemp/game/g_navnew.c:384-391`
pub fn NAVNEW_CheckDoubleBlock(
    self_: &gentity_t,
    blocker: &gentity_t,
    blocked_dir: vec3_t,
) -> qboolean {
    // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; read the pointer via
    // the safe field access and deref raw as Raven does.
    let npc = blocker.NPC;
    if !npc.is_null() && unsafe { (*npc).blockingEntNum } == self_.s.number {
        return qtrue;
    }
    qfalse
}

/// Raven `NAVNEW_ResolveEntityCollision`.
///
/// Source: `oracle/codemp/game/g_navnew.c:399-435`
pub fn NAVNEW_ResolveEntityCollision(
    ctx: &mut GameContext,
    self_: EntityId,
    blocker: Option<EntityId>,
    movedir: &mut vec3_t,
    pathDir: vec3_t,
    setBlockedInfo: qboolean,
) -> qboolean {
    // `movedir` threaded `&mut` down to Bypass so the avoid-direction write
    // reaches AvoidCollision's `VectorCopy(movedir, info->direction)`.
    let blocker = blocker.unwrap();
    //Doors are ignored
    let blocker_classname = ctx.world.entity(blocker).classname;
    if crate::q_shared::Q_stricmp(blocker_classname, cstr("func_door").as_ptr()) == 0 {
        let mut center = [0.0f32; 3];
        CalcTeamDoorCenter(ctx, blocker, &mut center);
        let self_origin = ctx.world.entity(self_).r.currentOrigin;
        if crate::q_math::DistanceSquared(self_origin, center) > MIN_DOOR_BLOCK_DIST_SQR as f32 {
            return qtrue;
        }
    }

    let blocker_origin = ctx.world.entity(blocker).r.currentOrigin;
    let self_origin = ctx.world.entity(self_).r.currentOrigin;
    let mut blocked_dir = [0.0f32; 3];
    crate::q_math::_VectorSubtract(blocker_origin, self_origin, &mut blocked_dir);
    let blocked_dist = crate::q_math::VectorNormalize(&mut blocked_dir);

    //Make sure an actual collision is going to happen
    //	if ( NAVNEW_PredictCollision( self, blocker, movedir, blocked_dir ) == qfalse )
    //		return qtrue;

    //First, attempt to walk around the blocker or shove him out of the way
    if NAVNEW_Bypass(
        ctx,
        self_,
        Some(blocker),
        blocked_dir,
        blocked_dist,
        movedir,
        setBlockedInfo,
    ) != qfalse
    {
        return qtrue;
    }

    //Can't get around him... see if I'm blocking him too... if so, I need to just keep moving?
    if NAVNEW_CheckDoubleBlock(
        ctx.world.entity(self_),
        ctx.world.entity(blocker),
        blocked_dir,
    ) != qfalse
    {
        return qtrue;
    }

    if setBlockedInfo != qfalse {
        //Complain about it if we can
        NPC_SetBlocked(ctx, self_, Some(blocker));
    }

    qfalse
}

/// Raven `NAVNEW_AvoidCollision`.
///
/// Source: `oracle/codemp/game/g_navnew.c:442-518`
pub fn NAVNEW_AvoidCollision(
    ctx: &mut GameContext,
    self_: EntityId,
    goal: Option<EntityId>,
    info: *mut navInfo_t,
    setBlockedInfo: qboolean,
    blockedMovesLimit: c_int,
) -> qboolean {
    // `info` is a caller-owned `*mut navInfo_t` scratch pointer (not an entity);
    // `(*info).*` derefs stay raw exactly as Raven does. gNPC_t (`NPC`) likewise
    // has no safe accessor. FLAG (task #7).
    unsafe {
        //Cap our distance
        if (*info).distance > MAX_COLL_AVOID_DIST as f32 {
            (*info).distance = MAX_COLL_AVOID_DIST as f32;
        }

        //Get an end position
        let mut movedir = [0.0f32; 3];
        let mut movepos = [0.0f32; 3];
        let self_origin = ctx.world.entity(self_).r.currentOrigin;
        crate::q_math::_VectorMA(
            self_origin,
            (*info).distance,
            (*info).direction,
            &mut movepos,
        );
        crate::q_math::_VectorCopy((*info).direction, &mut movedir);

        //Now test against entities
        if NAV_CheckAhead(
            ctx,
            self_,
            movepos,
            &mut (*info).trace as *mut trace_t,
            CONTENTS_BODY,
        ) == qfalse
        {
            //Get the blocker
            (*info).blocker =
                &mut ctx.world.g_entities[(*info).trace.entityNum as usize] as *mut gentity_t;
            (*info).flags |= NIF_COLLISION;

            //Ok to hit our goal entity
            if goal == ctx.entity_id_of((*info).blocker) {
                return qtrue;
            }

            let npc = ctx.world.entity(self_).NPC;
            if setBlockedInfo != qfalse {
                if (*npc).consecutiveBlockedMoves > blockedMovesLimit {
                    if ctx.world.cvars.d_patched.integer != 0 {
                        //use patch-style navigation
                        (*npc).consecutiveBlockedMoves += 1;
                    }
                    NPC_SetBlocked(ctx, self_, ctx.entity_id_of((*info).blocker));
                    return qfalse;
                }
                (*npc).consecutiveBlockedMoves += 1;
            }
            //See if we're moving along with them
            //if ( NAVNEW_TrueCollision( self, info->blocker, movedir, info->direction ) == qfalse )
            //	return qtrue;

            //Test for blocking by standing on goal
            if NAV_TestForBlocked(
                ctx,
                self_,
                goal,
                ctx.entity_id_of((*info).blocker),
                (*info).distance,
                &mut (*info).flags as *mut c_int,
            ) == qtrue
            {
                return qfalse;
            }

            //If the above function said we're blocked, don't do the extra checks
            /*
            if ( info->flags & NIF_BLOCKED )
                return qtrue;
            */

            //See if we can get that entity to move out of our way
            if NAVNEW_ResolveEntityCollision(
                ctx,
                self_,
                ctx.entity_id_of((*info).blocker),
                &mut movedir,
                (*info).pathDirection,
                setBlockedInfo,
            ) == qfalse
            {
                return qfalse;
            }

            crate::q_math::_VectorCopy(movedir, &mut (*info).direction);

            return qtrue;
        } else {
            if setBlockedInfo != qfalse {
                let npc = ctx.world.entity(self_).NPC;
                (*npc).consecutiveBlockedMoves = 0;
            }
        }

        //Our path is clear, just move there
        if ctx.world.globals.NAVDEBUG_showCollision != 0 {
            let self_origin = ctx.world.entity(self_).r.currentOrigin;
            G_DrawEdge(self_origin, movepos, EDGE_MOVEDIR);
        }

        qtrue
    }
}

/// Raven `NAVNEW_TestNodeConnectionBlocked`.
///
/// Raven: see if the direct path between 2 nodes is blocked by architecture
/// or an ent.
///
/// Source: `oracle/codemp/game/g_navnew.c:520-572`
pub fn NAVNEW_TestNodeConnectionBlocked(
    ctx: &mut GameContext,
    wp1: c_int,
    wp2: c_int,
    ignoreEnt: Option<EntityId>,
    goalEntNum: c_int,
    checkWorld: qboolean,
    checkEnts: qboolean,
) -> qboolean {
    if checkWorld == qfalse && checkEnts == qfalse {
        //duh, nothing to trace against
        return qfalse;
    }
    let mut localPlayerMins = [0.0f32; 3];
    let mut localPlayerMaxs = [0.0f32; 3];
    localPlayerMins[0] = -15.0;
    localPlayerMins[1] = -15.0;
    localPlayerMins[2] = DEFAULT_MINS_2;
    localPlayerMaxs[0] = 15.0;
    localPlayerMaxs[1] = 15.0;
    localPlayerMaxs[2] = DEFAULT_MAXS_2;

    let mut pos1 = [0.0f32; 3];
    let mut pos2 = [0.0f32; 3];
    trap::Nav_GetNodePosition(
        ctx.engine,
        mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
            wp1,
            &mut pos1 as *mut vec3_t,
        ),
    );
    trap::Nav_GetNodePosition(
        ctx.engine,
        mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
            wp2,
            &mut pos2 as *mut vec3_t,
        ),
    );

    let mut clipmask = MASK_NPCSOLID | CONTENTS_BOTCLIP;
    if checkWorld == qfalse {
        clipmask &= !(CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP);
    }
    if checkEnts == qfalse {
        clipmask &= !CONTENTS_BODY;
    }

    let mut mins = [0.0f32; 3];
    let mut maxs = [0.0f32; 3];
    let ignoreEntNum: c_int;
    if let Some(ignore_id) = ignoreEnt {
        crate::q_math::_VectorCopy(ctx.world.entity(ignore_id).r.mins, &mut mins);
        crate::q_math::_VectorCopy(ctx.world.entity(ignore_id).r.maxs, &mut maxs);
        ignoreEntNum = ctx.world.entity(ignore_id).s.number;
    } else {
        // §19: Raven copies playerMaxs into `mins` here (its own bug, preserved),
        // leaving `maxs` uninitialized before the reads below; the zeroed `maxs`
        // declared above is the defined-behavior choice for that C UB.
        crate::q_math::_VectorCopy(localPlayerMins, &mut mins);
        crate::q_math::_VectorCopy(localPlayerMaxs, &mut mins);
        ignoreEntNum = ENTITYNUM_NONE;
    }
    mins[2] += STEPSIZE;
    //don't let box get inverted
    if mins[2] > maxs[2] {
        mins[2] = maxs[2];
    }

    let mut trace: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut trace as *mut trace_t,
            &pos1 as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &pos2 as *const vec3_t,
            ignoreEntNum,
            clipmask,
        ),
    );
    if trace.fraction >= 1.0f32 || trace.entityNum as c_int == goalEntNum {
        //clear or hit goal
        return qfalse;
    }
    //hit something we weren't supposed to
    qtrue
}

/// Raven `NAVNEW_MoveToGoal`.
///
/// Source: `oracle/codemp/game/g_navnew.c:578-865`
pub fn NAVNEW_MoveToGoal(ctx: &mut GameContext, self_: EntityId, info: *mut navInfo_t) -> c_int {
    // `info` is a caller-owned `*mut navInfo_t` scratch pointer (not an entity);
    // its `(*info).*` derefs, the seam `.cast()` entity-pointer passes, and the
    // gNPC_t (`npc`) derefs stay raw exactly as Raven does. FLAG (task #7). The
    // stored `goalEntity` is a valid `Some(EntityId)`, so — matching Raven's
    // unconditional `self->NPC->goalEntity->*` derefs — its fields read straight
    // through the accessor (the port's always-true null guards are dropped).
    unsafe {
        // FLAG (task #7): gNPC_t (`NPC`) has no safe accessor; held raw as `self->NPC`.
        let npc = ctx.world.entity(self_).NPC;
        let mut bestNode = WAYPOINT_NONE;
        let mut foundClearPath = qfalse;
        let mut origin = [0.0f32; 3];
        let mut tempInfo: navInfo_t = core::mem::zeroed();
        let mut setBlockedInfo = qtrue;
        let mut inBestWP = qfalse;
        let mut inGoalWP = qfalse;
        let mut goalWPFailed = qfalse;
        let mut numTries = 0;

        core::ptr::copy_nonoverlapping(
            info as *const navInfo_t,
            &mut tempInfo as *mut navInfo_t,
            1,
        );

        //Must have a goal entity to move there
        if (*npc).goalEntity.is_none() {
            return WAYPOINT_NONE;
        }

        let goal_entity = match (*npc).goalEntity {
            Some(eid) => eid,
            None => return WAYPOINT_NONE,
        };

        if ctx.world.entity(self_).waypoint == WAYPOINT_NONE
            && ctx.world.entity(self_).noWaypointTime > ctx.world.level.time
        {
            //didn't have a valid one in about the past second, don't look again just yet
            return WAYPOINT_NONE;
        }

        if ctx.world.entity(goal_entity).waypoint == WAYPOINT_NONE
            && ctx.world.entity(goal_entity).noWaypointTime > ctx.world.level.time
        {
            //didn't have a valid one in about the past second, don't look again just yet
            return WAYPOINT_NONE;
        }

        if ctx.world.entity(self_).noWaypointTime > ctx.world.level.time
            && ctx.world.entity(goal_entity).noWaypointTime > ctx.world.level.time
        {
            //just use current waypoints
            let self_wp = ctx.world.entity(self_).waypoint;
            let goal_wp = ctx.world.entity(goal_entity).waypoint;
            bestNode = trap::Nav_GetBestNodeAltRoute2(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_GETBESTNODEALT2::GNavGetbestnodealt2Args::new(
                    self_wp, goal_wp, bestNode,
                ),
            );
        }
        //FIXME!!!!: this is making them wiggle back and forth between waypoints
        else if {
            let self_ptr = &mut ctx.world.g_entities[self_.index()] as *mut gentity_t;
            let goal_ptr = &mut ctx.world.g_entities[goal_entity.index()] as *mut gentity_t;
            bestNode = trap::Nav_GetBestPathBetweenEnts(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_GETBESTPATHBETWEENENTS::GNavGetbestpathbetweenentsArgs::new(self_ptr.cast(), goal_ptr.cast(), NF_CLEAR_PATH),
            );
            bestNode == NODE_NONE
        } {
            //one of us didn't have a valid waypoint!
            if ctx.world.entity(self_).waypoint == NODE_NONE {
                //don't even try to find one again for a bit
                let t = ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1500);
                ctx.world.entity_mut(self_).noWaypointTime = t;
            }
            if ctx.world.entity(goal_entity).waypoint == NODE_NONE {
                //don't even try to find one again for a bit
                let t = ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1500);
                ctx.world.entity_mut(goal_entity).noWaypointTime = t;
            }
            return WAYPOINT_NONE;
        } else {
            if ctx.world.entity(goal_entity).noWaypointTime < ctx.world.level.time {
                let t = ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1500);
                ctx.world.entity_mut(goal_entity).noWaypointTime = t;
            }
        }

        while foundClearPath == qfalse {
            inBestWP = qfalse;
            inGoalWP = qfalse;

            if bestNode == WAYPOINT_NONE {
                // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                // Source: `oracle/codemp/game/g_navnew.c:637,844`
                trap::Nav_GetNodePosition(
                    ctx.engine,
                    mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                        ctx.world.entity(self_).waypoint,
                        &mut origin as *mut vec3_t,
                    ),
                );
                return WAYPOINT_NONE;
            }

            trap::Nav_GetNodePosition(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                    bestNode,
                    &mut origin as *mut vec3_t,
                ),
            );

            if inGoalWP == qfalse {
                //not heading straight for goal
                if bestNode == ctx.world.entity(self_).waypoint {
                    //we know it's clear or architecture
                } else {
                    //heading to an edge off our confirmed clear waypoint... make sure it's clear
                    //it it's not, bestNode will fall back to our waypoint
                    let oldBestNode = bestNode;
                    let self_wp = ctx.world.entity(self_).waypoint;
                    bestNode = NAV_TestBestNode(ctx, self_, self_wp, bestNode, qtrue);
                    if bestNode == ctx.world.entity(self_).waypoint {
                        //we fell back to our waypoint, reset the origin
                        (*npc).aiFlags |= NPCAI_BLOCKED;
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(oldBestNode, &mut (*npc).blockedDest as *mut vec3_t),
                        );
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(bestNode, &mut origin as *mut vec3_t),
                        );
                    }
                }
            }

            core::ptr::copy_nonoverlapping(
                info as *const navInfo_t,
                &mut tempInfo as *mut navInfo_t,
                1,
            );
            let self_origin = ctx.world.entity(self_).r.currentOrigin;
            crate::q_math::_VectorSubtract(origin, self_origin, &mut tempInfo.direction);
            crate::q_math::VectorNormalize(&mut tempInfo.direction);

            //NOTE: One very important thing NAVNEW_AvoidCollision does is
            //		it actually CHANGES the value of "direction" - it changes it to
            //		whatever dir you need to go in to avoid the obstacle...
            foundClearPath = NAVNEW_AvoidCollision(
                ctx,
                self_,
                Some(goal_entity),
                &mut tempInfo,
                setBlockedInfo,
                5,
            );

            if foundClearPath == qfalse {
                //blocked by an ent
                if inGoalWP != qfalse {
                    //we were heading straight for the goal, head for the goal's wp instead
                    trap::Nav_GetNodePosition(
                        ctx.engine,
                        mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                            bestNode,
                            &mut origin as *mut vec3_t,
                        ),
                    );
                    foundClearPath = NAVNEW_AvoidCollision(
                        ctx,
                        self_,
                        Some(goal_entity),
                        &mut tempInfo,
                        setBlockedInfo,
                        5,
                    );
                }
            }

            if foundClearPath != qfalse {
                //clear!
                //If we got set to blocked, clear it
                NPC_ClearBlocked(ctx.world.entity_mut(self_));
                //Take the dir
                core::ptr::copy_nonoverlapping(
                    &tempInfo as *const navInfo_t,
                    info as *mut navInfo_t,
                    1,
                );
                if ctx.world.entity(self_).s.weapon == WP_SABER {
                    //jedi
                    if (*info).direction[2] * (*info).distance > 64.0 {
                        (*npc).aiFlags |= NPCAI_BLOCKED;
                        crate::q_math::_VectorCopy(origin, &mut (*npc).blockedDest);
                        // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                        // Source: `oracle/codemp/game/g_navnew.c:730,844`
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                                ctx.world.entity(self_).waypoint,
                                &mut origin as *mut vec3_t,
                            ),
                        );
                        return WAYPOINT_NONE;
                    }
                }
            } else {
                //blocked by ent!
                if setBlockedInfo != qfalse {
                    (*npc).aiFlags |= NPCAI_BLOCKED;
                    trap::Nav_GetNodePosition(
                        ctx.engine,
                        mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                            bestNode,
                            &mut (*npc).blockedDest as *mut vec3_t,
                        ),
                    );
                }
                //Only set blocked info first time
                setBlockedInfo = qfalse;

                if inGoalWP != qfalse {
                    //we headed for our goal and failed and our goal's WP and failed
                    if ctx.world.entity(self_).waypoint == ctx.world.entity(goal_entity).waypoint {
                        //our waypoint is our goal's waypoint, nothing we can do
                        //remember that this node is blocked
                        let self_wp = ctx.world.entity(self_).waypoint;
                        let self_ptr = &mut ctx.world.g_entities[self_.index()] as *mut gentity_t;
                        trap::Nav_AddFailedNode(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_ADDFAILEDNODE::GNavAddfailednodeArgs::new(
                                self_ptr.cast(),
                                self_wp,
                            ),
                        );
                        // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                        // Source: `oracle/codemp/game/g_navnew.c:750,844`
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                                ctx.world.entity(self_).waypoint,
                                &mut origin as *mut vec3_t,
                            ),
                        );
                        return WAYPOINT_NONE;
                    } else {
                        //try going for our waypoint this time
                        goalWPFailed = qtrue;
                        inGoalWP = qfalse;
                    }
                } else if bestNode != ctx.world.entity(self_).waypoint {
                    //we headed toward our next waypoint (instead of our waypoint) and failed
                    if ctx.world.cvars.d_altRoutes.integer != 0 {
                        //mark this edge failed and try our waypoint
                        //NOTE: don't assume there is something blocking the direct path
                        //			between my waypoint and the bestNode... I could be off
                        //			that path because of collision avoidance...
                        let self_wp = ctx.world.entity(self_).waypoint;
                        let goal_num = ctx.world.entity(goal_entity).s.number;
                        if ctx.world.cvars.d_patched.integer != 0 && //use patch-style navigation
                           (trap::Nav_NodesAreNeighbors(
                               ctx.engine,
                               mp_abi::game::syscalls::G_NAV_NODESARENEIGHBORS::GNavNodesareneighborsArgs::new(self_wp, bestNode),
                           ) == qfalse
                           || NAVNEW_TestNodeConnectionBlocked(ctx, self_wp, bestNode, Some(self_), goal_num, qfalse, qtrue) != qfalse)
                        {
                            //the direct path between these 2 nodes is blocked by an ent
                            let self_num = ctx.world.entity(self_).s.number;
                            trap::Nav_AddFailedEdge(
                                ctx.engine,
                                mp_abi::game::syscalls::G_NAV_ADDFAILEDEDGE::GNavAddfailededgeArgs::new(self_num, self_wp, bestNode),
                            );
                        }
                        bestNode = self_wp;
                    } else {
                        //we should stop
                        // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                        // Source: `oracle/codemp/game/g_navnew.c:776,844`
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                                ctx.world.entity(self_).waypoint,
                                &mut origin as *mut vec3_t,
                            ),
                        );
                        return WAYPOINT_NONE;
                    }
                } else {
                    //we headed for *our* waypoint and couldn't get to it
                    if ctx.world.cvars.d_altRoutes.integer != 0 {
                        //remember that this node is blocked
                        let self_wp = ctx.world.entity(self_).waypoint;
                        let self_ptr = &mut ctx.world.g_entities[self_.index()] as *mut gentity_t;
                        trap::Nav_AddFailedNode(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_ADDFAILEDNODE::GNavAddfailednodeArgs::new(
                                self_ptr.cast(),
                                self_wp,
                            ),
                        );
                        //Now we should get our waypoints again
                        //FIXME: cache the trace-data for subsequent calls as only the route info would have changed
                        // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                        // Source: `oracle/codemp/game/g_navnew.c:789,844`
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                                ctx.world.entity(self_).waypoint,
                                &mut origin as *mut vec3_t,
                            ),
                        );
                        return WAYPOINT_NONE;
                    } else {
                        //we should stop
                        // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                        // Source: `oracle/codemp/game/g_navnew.c:795,844`
                        trap::Nav_GetNodePosition(
                            ctx.engine,
                            mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                                ctx.world.entity(self_).waypoint,
                                &mut origin as *mut vec3_t,
                            ),
                        );
                        return WAYPOINT_NONE;
                    }
                }

                if {
                    numTries += 1;
                    numTries
                } >= 10
                {
                    // failed: label — trap_Nav_GetNodePosition(waypoint, origin) before bail.
                    // Source: `oracle/codemp/game/g_navnew.c:801,844`
                    trap::Nav_GetNodePosition(
                        ctx.engine,
                        mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                            ctx.world.entity(self_).waypoint,
                            &mut origin as *mut vec3_t,
                        ),
                    );
                    return WAYPOINT_NONE;
                }
            }
        }

        //Draw any debug info, if requested
        if ctx.world.globals.NAVDEBUG_showEnemyPath != 0 {
            let mut dest = [0.0f32; 3];
            let mut start = [0.0f32; 3];

            //Get the positions
            trap::Nav_GetNodePosition(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                    ctx.world.entity(goal_entity).waypoint,
                    &mut dest as *mut vec3_t,
                ),
            );
            trap::Nav_GetNodePosition(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                    bestNode,
                    &mut start as *mut vec3_t,
                ),
            );

            //Draw the route
            G_DrawNode(start, NODE_START);
            if bestNode != ctx.world.entity(self_).waypoint {
                let mut wpPos = [0.0f32; 3];
                trap::Nav_GetNodePosition(
                    ctx.engine,
                    mp_abi::game::syscalls::G_NAV_GETNODEPOSITION::GNavGetnodepositionArgs::new(
                        ctx.world.entity(self_).waypoint,
                        &mut wpPos as *mut vec3_t,
                    ),
                );
                G_DrawNode(wpPos, NODE_NAVGOAL);
            }
            G_DrawNode(dest, NODE_GOAL);
            let goal_origin = ctx.world.entity(goal_entity).r.currentOrigin;
            G_DrawEdge(dest, goal_origin, EDGE_PATH);
            G_DrawNode(goal_origin, NODE_GOAL);
            trap::Nav_ShowPath(
                ctx.engine,
                mp_abi::game::syscalls::G_NAV_SHOWPATH::GNavShowpathArgs::new(
                    bestNode,
                    ctx.world.entity(goal_entity).waypoint,
                ),
            );
        }

        (*npc).shoveCount = 0;

        //let me keep this waypoint for a while
        if ctx.world.entity(self_).noWaypointTime < ctx.world.level.time {
            let t = ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(500, 1500);
            ctx.world.entity_mut(self_).noWaypointTime = t;
        }
        bestNode
    }
}
