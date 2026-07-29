//! Port of `oracle/codemp/cgame/cg_snapshot.c` — snapshot transition — entity reset, interpolation setup. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_bg::bg_misc::BG_PlayerStateToEntityState;
use mp_bg::public::entity_event::EVENT_VALID_MSEC;
use mp_bg::public::entity_flags::{EF_G2ANIMATING, EF_TELEPORT_BIT};
use mp_bg::public::entity_type::entityType_t;
use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};
use mp_qshared::shared::q_math::_VectorCopy;
use mp_qshared::shared::{qfalse, qtrue, SNAPFLAG_SERVERCOUNT};
use mp_uishared::shared::display_state::DisplayState;

use crate::cg_draw::CG_AddLagometerSnapshotInfo;
use crate::cg_event::CG_CheckEvents;
use crate::cg_main::CG_Printf;
use crate::cg_players::{zeroed_client_info, CG_ResetPlayerEntity};
use crate::cg_predict::CG_BuildSolidList;
use crate::local::centity_s::centity_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `CG_SetNextSnap` — latches a freshly-read snapshot in as `cg.nextSnap`,
/// folds its entity states into `cg_entities[].nextState`, and figures out
/// whether the transition is a teleport (so the interpolator knows not to
/// blend across it).
///
/// PORT-NOTE: Raven takes the already-filled `snapshot_t *` directly. Here
/// `slot` names which of `cg.activeSnapshots` [`CG_ReadNextSnapshot`] just
/// filled (0 or 1) instead - a live `&snapshot_t` borrowed off `world.cg`
/// can't coexist with the `&mut CgWorld` this function also needs to mutate
/// `world.entities`/`world.cg.nextFrameTeleport`, so the index is the handle
/// (matches the address-compare idiom `cg_t::snap_ref` already centralizes).
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:206-257`
pub fn CG_SetNextSnap(world: &mut CgWorld, slot: usize) {
    world.cg.nextSnap = &mut world.cg.activeSnapshots[slot] as *mut snapshot_t;

    // BG_PlayerStateToEntityState( &snap->ps, &cg_entities[ snap->ps.clientNum ].nextState, qfalse );
    let clientNum = world.cg.activeSnapshots[slot].ps.clientNum as usize;
    let ps_ptr = &mut world.cg.activeSnapshots[slot].ps as *mut playerState_t;
    let es_ptr = &mut world.entities[clientNum].nextState as *mut entityState_t;
    BG_PlayerStateToEntityState(ps_ptr, es_ptr, qfalse);

    // check for extrapolation errors
    let numEntities = world.cg.activeSnapshots[slot].numEntities;
    for num in 0..numEntities as usize {
        let es = world.cg.activeSnapshots[slot].entities[num];
        let entNum = es.number as usize;

        world.entities[entNum].nextState = es;

        // if this frame is a teleport, or the entity wasn't in the
        // previous frame, don't interpolate
        let dontInterpolate = world.entities[entNum].currentValid == qfalse
            || (world.entities[entNum].currentState.eFlags ^ es.eFlags) & EF_TELEPORT_BIT != 0;
        world.entities[entNum].interpolate = if dontInterpolate { qfalse } else { qtrue };
    }

    let nextPsEFlags = world.cg.activeSnapshots[slot].ps.eFlags;
    let nextClientNum = world.cg.activeSnapshots[slot].ps.clientNum;
    let nextSnapFlags = world.cg.activeSnapshots[slot].snapFlags;

    // if the next frame is a teleport for the playerstate, we
    // can't interpolate during demos
    //
    // §F19: Raven dereferences `cg.snap` unconditionally a few lines below
    // this first check (`cg.snap->ps.clientNum`, `cg.snap->snapFlags`), with
    // no null guard of its own - `CG_SetNextSnap` is only ever reached once
    // the first snapshot has landed, so `cg.snap` is non-null in practice.
    // The `None` arm below takes the answer Raven's own short-circuit
    // `cg.snap && (...)` already gives the first check: not a teleport.
    let snapVals = world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.eFlags, snap.ps.clientNum, snap.snapFlags));

    match snapVals {
        Some((eFlags, clientNumSnap, snapFlagsSnap)) => {
            world.cg.nextFrameTeleport = if (nextPsEFlags ^ eFlags) & EF_TELEPORT_BIT != 0 {
                qtrue
            } else {
                qfalse
            };

            // if changing follow mode, don't interpolate
            if nextClientNum != clientNumSnap {
                world.cg.nextFrameTeleport = qtrue;
            }

            // if changing server restarts, don't interpolate
            if (nextSnapFlags ^ snapFlagsSnap) & SNAPFLAG_SERVERCOUNT != 0 {
                world.cg.nextFrameTeleport = qtrue;
            }
        }
        None => {
            world.cg.nextFrameTeleport = qfalse;
        }
    }

    // sort out solid entities
    CG_BuildSolidList(world);
}

/// Raven `CG_ReadNextSnapshot` — pulls snapshots out of the client system
/// until one loads cleanly (or none are left), logging a lagometer sample for
/// every attempt (hits and drops alike).
///
/// PORT-NOTE: Raven returns the filled `snapshot_t *`. Returning `Option<usize>`
/// (the `cg.activeSnapshots` slot) instead of a reference keeps this composable
/// with [`CG_SetNextSnap`] - a caller does
/// `if let Some(slot) = CG_ReadNextSnapshot(ctx) { CG_SetNextSnap(ctx.world, slot); }`,
/// the same two-step Raven's `CG_ProcessSnapshots` (not in this wave) does with
/// its `snap` local.
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:270-316`
pub fn CG_ReadNextSnapshot(ctx: &mut CgContext) -> Option<usize> {
    if ctx.world.cg.latestSnapshotNum > ctx.world.cgs.processedSnapshotNum + 1000 {
        let latestSnapshotNum = ctx.world.cg.latestSnapshotNum;
        let processedSnapshotNum = ctx.world.cgs.processedSnapshotNum;
        CG_Printf(
            ctx,
            &format!(
                "WARNING: CG_ReadNextSnapshot: way out of range, {} > {}",
                latestSnapshotNum, processedSnapshotNum
            ),
        );
    }

    while ctx.world.cgs.processedSnapshotNum < ctx.world.cg.latestSnapshotNum {
        // decide which of the two slots to load it into
        let snapPtr = ctx.world.cg.snap as *const snapshot_t;
        let dest: usize = if snapPtr == &ctx.world.cg.activeSnapshots[0] as *const snapshot_t {
            1
        } else {
            0
        };

        // try to read the snapshot from the client system
        ctx.world.cgs.processedSnapshotNum += 1;
        let processedSnapshotNum = ctx.world.cgs.processedSnapshotNum;
        let r = trap::GetSnapshot(
            ctx.engine,
            processedSnapshotNum,
            &mut ctx.world.cg.activeSnapshots[dest],
        );

        // Raven's `FIXME: why would trap_GetSnapshot return a snapshot with
        // the same server time` guard body is entirely commented out
        // (`//continue`) in the oracle - dead code, nothing to transcribe.

        // if it succeeded, return
        if r {
            // hand the lagometer the two fields it reads, releasing the
            // world.cg borrow
            let src = &ctx.world.cg.activeSnapshots[dest];
            let logged = (src.ping, src.snapFlags);
            CG_AddLagometerSnapshotInfo(ctx.world, Some(logged));
            return Some(dest);
        }

        // a GetSnapshot will return failure if the snapshot
        // never arrived, or is so old that its entities
        // have been shoved off the end of the circular
        // buffer in the client system.

        // record as a dropped packet
        CG_AddLagometerSnapshotInfo(ctx.world, None);

        // If there are additional snapshots, continue trying to
        // read them.
    }

    // nothing left to read
    None
}

/// Raven `CG_ResetEntity` — wipes an entity's event/lerp state when it
/// (re)appears in a snapshot: clears a stale previous-event window, snaps
/// the interpolation origin/angles straight to the fresh snapshot values,
/// and for G2-animating models resets the torso/legs animation slot so the
/// next frame picks a fresh anim rather than blending from garbage.
///
/// §F19: Raven reads `cg.snap->serverTime` unconditionally here; `CG_ResetEntity`
/// only runs once a snapshot has landed (same reasoning as `CG_SetNextSnap`'s
/// note above), so the `None` arm leaves `trailTime` untouched - unreachable
/// in practice, never a panic.
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:15-43`
pub fn CG_ResetEntity(ctx: &mut CgContext, centNum: usize) {
    // take the body out for the CG_ResetPlayerEntity call below - same
    // pattern as cg_ents.rs's CG_General ragdoll/bolt calls
    let mut cent = core::mem::replace(ctx.world.entity_mut(centNum), centity_t::zeroed());

    // if the previous snapshot this entity was updated in is at least
    // an event window back in time then we can reset the previous event
    if cent.snapShotTime < ctx.world.cg.time - EVENT_VALID_MSEC {
        cent.previousEvent = 0;
    }

    if let Some(snap) = ctx.world.cg.snap_ref() {
        cent.trailTime = snap.serverTime;
    }

    _VectorCopy(cent.currentState.origin, &mut cent.lerpOrigin);
    _VectorCopy(cent.currentState.angles, &mut cent.lerpAngles);

    if (cent.currentState.eFlags & EF_G2ANIMATING) != 0 {
        //reset the animation state
        cent.pe.torso.animationNumber = -1;
        cent.pe.legs.animationNumber = -1;
    }

    // Raven's `#if 0` ragdoll lerpOriginOffset block is dead code in the
    // oracle - nothing to transcribe.

    if cent.currentState.eType == entityType_t::ET_PLAYER as c_int
        || cent.currentState.eType == entityType_t::ET_NPC as c_int
    {
        let isNpc = cent.currentState.eType == entityType_t::ET_NPC as c_int;

        if isNpc {
            // CG_ResetPlayerEntity's own npcClient alloc always leaves its
            // internal `npcCi` populated by the time `ci` would be read, so
            // the param goes unused here - hand it a scratch value instead
            // of indexing cgs.clientinfo with an NPC's clientNum, which
            // isn't a client slot.
            //
            // SAFETY: clientInfo_t is #[repr(C)] scalars/arrays/qhandle_t
            // and opaque ghoul2 pointers, and its two enum members
            // (team_t/gender_t) both have a 0 discriminant, so all-zero is
            // a legal value (same reasoning as cg_players.rs's private
            // zeroed_client_info).
            let mut scratch = zeroed_client_info();
            CG_ResetPlayerEntity(ctx, &mut cent, &mut scratch);
        } else if (cent.currentState.clientNum as usize) < ctx.world.cgs.clientinfo.len() {
            let clientNum = cent.currentState.clientNum as usize;
            let mut ci = core::mem::replace(
                &mut ctx.world.cgs.clientinfo[clientNum],
                zeroed_client_info(),
            );
            CG_ResetPlayerEntity(ctx, &mut cent, &mut ci);
            ctx.world.cgs.clientinfo[clientNum] = ci;
        } else {
            // §F19: Raven indexes cgs.clientinfo with a server-supplied
            // clientNum and reads OOB garbage; the port hands a zeroed scratch
            // (the NPC-arm treatment) rather than panicking.
            let mut scratch = zeroed_client_info();
            CG_ResetPlayerEntity(ctx, &mut cent, &mut scratch);
        }
    }

    *ctx.world.entity_mut(centNum) = cent;
}

/// Raven `CG_TransitionEntity` — latches an entity's nextState in as its
/// currentState once a new snapshot lands, resetting event/lerp state first
/// if the entity wasn't interpolating from the previous frame (new arrival or
/// teleport), then checks for events on the fresh currentState.
///
/// PORT-NOTE: Raven's fn signature is `(centity_t *cent)`; [`CG_ResetEntity`]
/// and [`CG_CheckEvents`] - both already ported by an earlier wave - take
/// `(ctx, centNum)` instead, so this fn threads `ctx`/`centNum` too and
/// mutates the entity through `ctx.world.entity_mut` rather than holding a
/// `&mut centity_t` across those calls.
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:52-66`
pub fn CG_TransitionEntity(ctx: &mut CgContext, ds: &DisplayState, centNum: usize) {
    {
        let cent = ctx.world.entity_mut(centNum);
        cent.currentState = cent.nextState;
        cent.currentValid = qtrue;
    }

    // reset if the entity wasn't in the last frame or was teleported
    if ctx.world.entity_mut(centNum).interpolate == qfalse {
        CG_ResetEntity(ctx, centNum);
    }

    // clear the next state.  if will be set by the next CG_SetNextSnap
    ctx.world.entity_mut(centNum).interpolate = qfalse;

    // check for events
    CG_CheckEvents(ctx, ds, centNum);
}
