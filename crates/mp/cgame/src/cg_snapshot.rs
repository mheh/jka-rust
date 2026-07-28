//! Port of `oracle/codemp/cgame/cg_snapshot.c` — snapshot transition — entity reset, interpolation setup. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_bg::bg_misc::BG_PlayerStateToEntityState;
use mp_bg::public::entity_flags::EF_TELEPORT_BIT;
use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};
use mp_qshared::shared::{qfalse, qtrue, SNAPFLAG_SERVERCOUNT};

use crate::cg_draw::CG_AddLagometerSnapshotInfo;
use crate::cg_main::CG_Printf;
use crate::cg_predict::CG_BuildSolidList;
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
