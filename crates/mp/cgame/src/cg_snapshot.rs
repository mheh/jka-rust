//! Port of `oracle/codemp/cgame/cg_snapshot.c` — snapshot transition — entity reset, interpolation setup. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_bg::bg_misc::BG_PlayerStateToEntityState;
use mp_bg::public::entity_event::EVENT_VALID_MSEC;
use mp_bg::public::entity_flags::{EF_G2ANIMATING, EF_TELEPORT_BIT};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL;
use mp_qshared::common::mp::qcommon::pm_flags::PMF_FOLLOW;
use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};
use mp_qshared::shared::q_math::_VectorCopy;
use mp_qshared::shared::{qfalse, qtrue, SNAPFLAG_NOT_ACTIVE, SNAPFLAG_SERVERCOUNT};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;

use crate::cg_draw::CG_AddLagometerSnapshotInfo;
use crate::cg_event::CG_CheckEvents;
use crate::cg_main::{CG_Error, CG_Printf};
use crate::cg_players::{zeroed_client_info, CG_ResetPlayerEntity};
use crate::cg_playerstate::{CG_Respawn, CG_TransitionPlayerState};
use crate::cg_predict::{CG_BuildSolidList, CG_UsingEWeb};
use crate::cg_servercmds::CG_ExecuteNewServerCommands;
use crate::cg_weapons::CG_CopyG2WeaponInstance;
use crate::local::player_state_ref::PlayerStateRef;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `FIRST_WEAPON` - the first weapon for next/prev weapon switching.
///
/// Source: `oracle/codemp/game/bg_weapons.h:100`
const FIRST_WEAPON: c_int = WP_BRYAR_PISTOL;

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
/// the same two-step [`CG_ProcessSnapshots`] does with its `snap` local.
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
    // bitwise copy-in/copy-back, original left in place - a zeroed-swap here is
    // visible to every ctx-reading helper down the call chain (CG_CopyG2WeaponInstance
    // reads entity(centNum).currentState.number mid-call; the zeroed placeholder
    // made every client copy client 0's saber - C6b referee catch).
    // SAFETY: centity_t is #[repr(C)] plain data; the copy is written back whole.
    let mut cent = unsafe { core::ptr::read(ctx.world.entity(centNum)) };

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
            // bitwise copy-in/copy-back, ORIGINAL LEFT IN PLACE - the earlier
            // swap-out left a zeroed placeholder in the world slot, and
            // CG_CopyG2WeaponInstance re-reads cgs.clientinfo[n] mid-call, so
            // dual-saber clients lost their second saber copy (C6b referee
            // caught it: shared-instance tokens where Raven had per-client
            // ghoul2Weapons). ci mutations before any helper read are limited
            // to fields the helpers never touch, so value-aliasing is safe.
            // SAFETY: clientInfo_t is #[repr(C)] plain data (see the scratch
            // arm note); the bit-copy is written back whole after the call.
            let mut ci = unsafe { core::ptr::read(&ctx.world.cgs.clientinfo[clientNum]) };
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

/// Raven `CG_SetInitialSnapshot` — latches the very first snapshot in as
/// `cg.snap`, spins up the local player's ghoul2 instance if the model hasn't
/// been duplicated yet, and folds every entity in the snapshot into
/// `cg_entities[]` as a hard (non-interpolated) reset.
///
/// PORT-NOTE: Raven's `snapshot_t *snap` param is always the value
/// [`CG_ReadNextSnapshot`] just handed the caller (`cg_snapshot.c:365`), a
/// `cg.activeSnapshots` slot - `slot` names it directly, same handle shape as
/// [`CG_SetNextSnap`].
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:80-122`
pub fn CG_SetInitialSnapshot(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    slot: usize,
) {
    ctx.world.cg.snap = &mut ctx.world.cg.activeSnapshots[slot] as *mut snapshot_t;

    let clientNum = ctx.world.cg.activeSnapshots[slot].ps.clientNum as usize;

    if ctx.world.entity(clientNum).ghoul2.is_null()
        && trap::G2_HaveWeGhoul2Models(ctx.engine, ctx.world.cgs.clientinfo[clientNum].ghoul2Model)
    {
        let ghoul2Model = ctx.world.cgs.clientinfo[clientNum].ghoul2Model;
        let ghoul2Slot: &mut *mut c_void = &mut ctx.world.entity_mut(clientNum).ghoul2;
        trap::G2API_DuplicateGhoul2Instance(
            ctx.engine,
            ghoul2Model,
            ghoul2Slot as *mut *mut c_void,
        );
        let ghoul2 = ctx.world.entity(clientNum).ghoul2;
        CG_CopyG2WeaponInstance(ctx, clientNum, FIRST_WEAPON, ghoul2);

        // check now to see if we have this bone for setting anims and such
        let ghoul2 = ctx.world.entity(clientNum).ghoul2;
        if trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "face") == -1 {
            ctx.world.entity_mut(clientNum).noFace = qtrue;
        }
    }

    let ps_ptr = &mut ctx.world.cg.activeSnapshots[slot].ps as *mut playerState_t;
    let es_ptr = &mut ctx.world.entity_mut(clientNum).currentState as *mut entityState_t;
    BG_PlayerStateToEntityState(ps_ptr, es_ptr, qfalse);

    // sort out solid entities
    CG_BuildSolidList(ctx.world);

    let serverCommandSequence = ctx.world.cg.activeSnapshots[slot].serverCommandSequence;
    CG_ExecuteNewServerCommands(ctx, serverCommandSequence, menus, ds);

    // set our local weapon selection pointer to
    // what the server has indicated the current weapon is
    CG_Respawn(ctx.world);

    let numEntities = ctx.world.cg.activeSnapshots[slot].numEntities;
    for i in 0..numEntities as usize {
        let state = ctx.world.cg.activeSnapshots[slot].entities[i];
        let entNum = state.number as usize;

        let cent = ctx.world.entity_mut(entNum);
        cent.currentState = state;
        cent.interpolate = qfalse;
        cent.currentValid = qtrue;

        CG_ResetEntity(ctx, entNum);

        // check for events
        CG_CheckEvents(ctx, ds, entNum);
    }
}

/// Raven `CG_TransitionSnapshot` — retires the frame's old snapshot in favor
/// of `cg.nextSnap`, transitioning every entity's `nextState` into
/// `currentState`, then, if client-side movement prediction isn't running
/// this frame for any reason, drives the local playerstate's
/// respawn/damage/sound/event side effects directly off the snap-to-snap
/// delta.
///
/// PORT-NOTE: Raven addresses the old/new snapshot pair as `oldFrame`/
/// `cg.snap` pointers; the port names the two `cg.activeSnapshots` slots
/// (`oldSlot`/`newSlot`) the same way [`CG_ReadNextSnapshot`]'s `dest`
/// computation already does, so `ops`/`ps` read/write through real slots
/// instead of a dangling local `snapshot_t*`.
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:133-196`
pub fn CG_TransitionSnapshot(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    if ctx.world.cg.snap_ref().is_none() {
        CG_Error(ctx, "CG_TransitionSnapshot: NULL cg.snap");
        return;
    }
    if ctx.world.cg.next_snap_ref().is_none() {
        CG_Error(ctx, "CG_TransitionSnapshot: NULL cg.nextSnap");
        return;
    }

    // execute any server string commands before transitioning entities
    let nextSnapCmdSeq = ctx.world.cg.next_snap_ref().unwrap().serverCommandSequence;
    CG_ExecuteNewServerCommands(ctx, nextSnapCmdSeq, menus, ds);

    // Raven's `if ( !cg.snap ) { }` guard here has an empty body (a gutted
    // map_restart special-case) - nothing to transcribe.

    // which activeSnapshots slots cg.snap/cg.nextSnap currently name, same
    // address-compare idiom CG_ReadNextSnapshot's `dest` computation uses
    let snapPtr = ctx.world.cg.snap as *const snapshot_t;
    let oldSlot: usize = if snapPtr == &ctx.world.cg.activeSnapshots[0] as *const snapshot_t {
        0
    } else {
        1
    };
    let nextPtr = ctx.world.cg.nextSnap as *const snapshot_t;
    let newSlot: usize = if nextPtr == &ctx.world.cg.activeSnapshots[0] as *const snapshot_t {
        0
    } else {
        1
    };

    // clear the currentValid flag for all entities in the existing snapshot
    let numEntities = ctx.world.cg.activeSnapshots[oldSlot].numEntities;
    for i in 0..numEntities as usize {
        let entNum = ctx.world.cg.activeSnapshots[oldSlot].entities[i].number as usize;
        ctx.world.entity_mut(entNum).currentValid = qfalse;
    }

    // move nextSnap to snap and do the transitions
    ctx.world.cg.snap = ctx.world.cg.nextSnap;

    // CG_CheckPlayerG2Weapons calls here are commented out in the oracle -
    // nothing to transcribe.
    let snapClientNum = ctx.world.cg.activeSnapshots[newSlot].ps.clientNum as usize;
    let ps_ptr = &mut ctx.world.cg.activeSnapshots[newSlot].ps as *mut playerState_t;
    let es_ptr = &mut ctx.world.entity_mut(snapClientNum).currentState as *mut entityState_t;
    BG_PlayerStateToEntityState(ps_ptr, es_ptr, qfalse);
    ctx.world.entity_mut(snapClientNum).interpolate = qfalse;

    let newNumEntities = ctx.world.cg.activeSnapshots[newSlot].numEntities;
    for i in 0..newNumEntities as usize {
        let entNum = ctx.world.cg.activeSnapshots[newSlot].entities[i].number as usize;
        CG_TransitionEntity(ctx, ds, entNum);

        // remember time of snapshot this entity was last updated in
        let serverTime = ctx.world.cg.activeSnapshots[newSlot].serverTime;
        ctx.world.entity_mut(entNum).snapShotTime = serverTime;
    }

    ctx.world.cg.nextSnap = core::ptr::null_mut();

    // check for playerstate transition events
    //
    // `oldFrame` is always non-null here - the fn errors out above when
    // `cg.snap` starts NULL, and `oldSlot` was captured from that non-null
    // value before the reassignment above.
    let opsEFlags = ctx.world.cg.activeSnapshots[oldSlot].ps.eFlags;
    let psEFlags = ctx.world.cg.activeSnapshots[newSlot].ps.eFlags;
    // teleporting checks are irrespective of prediction
    if (psEFlags ^ opsEFlags) & EF_TELEPORT_BIT != 0 {
        ctx.world.cg.thisFrameTeleport = qtrue;
    }

    // if we are not doing client side movement prediction for any
    // reason, then the client events and view changes will be issued now
    let psPmFlags = ctx.world.cg.activeSnapshots[newSlot].ps.pm_flags;
    if ctx.world.cg.demoPlayback == qtrue
        || (psPmFlags & PMF_FOLLOW) != 0
        || ctx.world.cvars.cg_nopredict.integer != 0
        || ctx.world.cvars.cg_synchronousClients.integer != 0
        || CG_UsingEWeb(ctx.world)
    {
        let ps = ctx.world.cg.activeSnapshots[newSlot].ps;
        let mut ops = ctx.world.cg.activeSnapshots[oldSlot].ps;
        // PORT-NOTE: Raven's referent here is `&cg.snap->ps`, not entity
        // clientNum's cgSendPSPool row that `Snap` documents. Today's sole
        // consumer only tests `!= None`, so the arms coincide; when a consumer
        // resolves `Snap` through cgSendPSPool this call site needs its own arm
        // (cgSendPSPool-home ruling, design queue item 1/10).
        CG_TransitionPlayerState(ctx, ds, &ps, &mut ops, PlayerStateRef::Snap);
        // Raven writes through `ops` in place (`*ops = *ps` on the follow-mode
        // branch) - `oldFrame`'s slot is never read again once the next
        // CG_SetNextSnap overwrites it, so the write-back below reproduces
        // that in-place mutation faithfully.
        ctx.world.cg.activeSnapshots[oldSlot].ps = ops;
    }
}

/// Raven `CG_ProcessSnapshots` — pulls in every snapshot the client system has
/// queued since last frame: latches the very first one via
/// [`CG_SetInitialSnapshot`], keeps [`CG_SetNextSnap`] fed so there's always a
/// `cg.nextSnap` to interpolate towards, and walks [`CG_TransitionSnapshot`]
/// forward through however many frames it takes for `cg.time` to land inside
/// the `[cg.snap, cg.nextSnap)` window (or until snapshots run out and we fall
/// back to extrapolating off the last one).
///
/// Source: `oracle/codemp/cgame/cg_snapshot.c:338-413`
pub fn CG_ProcessSnapshots(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    // see what the latest snapshot the client system has is
    let (n, latestSnapshotTime) = trap::GetCurrentSnapshotNumber(ctx.engine);
    ctx.world.cg.latestSnapshotTime = latestSnapshotTime;
    if n != ctx.world.cg.latestSnapshotNum {
        if n < ctx.world.cg.latestSnapshotNum {
            // this should never happen
            CG_Error(ctx, "CG_ProcessSnapshots: n < cg.latestSnapshotNum");
            return;
        }
        ctx.world.cg.latestSnapshotNum = n;
    }

    // If we have yet to receive a snapshot, check for it.
    // Once we have gotten the first snapshot, cg.snap will
    // always have valid data for the rest of the game
    while ctx.world.cg.snap_ref().is_none() {
        let slot = CG_ReadNextSnapshot(ctx);
        let slot = match slot {
            Some(slot) => slot,
            None => {
                // we can't continue until we get a snapshot
                return;
            }
        };

        // set our weapon selection to what
        // the playerstate is currently using
        if (ctx.world.cg.activeSnapshots[slot].snapFlags & SNAPFLAG_NOT_ACTIVE) == 0 {
            CG_SetInitialSnapshot(ctx, menus, ds, slot);
        }
    }

    // loop until we either have a valid nextSnap with a serverTime
    // greater than cg.time to interpolate towards, or we run
    // out of available snapshots
    loop {
        // if we don't have a nextframe, try and read a new one in
        if ctx.world.cg.next_snap_ref().is_none() {
            let slot = CG_ReadNextSnapshot(ctx);

            // if we still don't have a nextframe, we will just have to
            // extrapolate
            let slot = match slot {
                Some(slot) => slot,
                None => break,
            };

            CG_SetNextSnap(ctx.world, slot);

            // if time went backwards, we have a level restart
            if ctx.world.cg.next_snap_ref().unwrap().serverTime
                < ctx.world.cg.snap_ref().unwrap().serverTime
            {
                CG_Error(ctx, "CG_ProcessSnapshots: Server time went backwards");
                return;
            }
        }

        // if our time is < nextFrame's, we have a nice interpolating state
        if ctx.world.cg.time >= ctx.world.cg.snap_ref().unwrap().serverTime
            && ctx.world.cg.time < ctx.world.cg.next_snap_ref().unwrap().serverTime
        {
            break;
        }

        // we have passed the transition from nextFrame to frame
        CG_TransitionSnapshot(ctx, menus, ds);
    }

    // assert our valid conditions upon exiting
    if ctx.world.cg.snap_ref().is_none() {
        CG_Error(ctx, "CG_ProcessSnapshots: cg.snap == NULL");
        return;
    }
    if ctx.world.cg.time < ctx.world.cg.snap_ref().unwrap().serverTime {
        // this can happen right after a vid_restart
        ctx.world.cg.time = ctx.world.cg.snap_ref().unwrap().serverTime;
    }
    if ctx.world.cg.next_snap_ref().is_some()
        && ctx.world.cg.next_snap_ref().unwrap().serverTime <= ctx.world.cg.time
    {
        CG_Error(
            ctx,
            "CG_ProcessSnapshots: cg.nextSnap->serverTime <= cg.time",
        );
        return;
    }
}
