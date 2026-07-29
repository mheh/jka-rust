//! Port of `oracle/codemp/cgame/cg_predict.c` — client-side movement prediction and its trace helpers. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_abi::cgame::public::snapshot_t::MAX_ENTITIES_IN_SNAPSHOT;
use mp_bg::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanItemBeGrabbed, BG_EvaluateTrajectory,
    BG_PlayerTouchesItem, BG_TouchJumpPad,
};
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::entity_event::entity_event_t::EV_ITEM_PICKUP;
use mp_bg::public::entity_flags::{EF_ITEMPLACEHOLDER, EF_NODRAW};
use mp_bg::public::entity_type::entityType_t::{
    ET_ITEM, ET_MISSILE, ET_NPC, ET_PLAYER, ET_PUSH_TRIGGER, ET_TELEPORT_TRIGGER, ET_TERRAIN,
};
use mp_bg::public::gametype::{GT_CTF, GT_CTY};
use mp_bg::public::item_type::{IT_POWERUP, IT_WEAPON};
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t::{PM_FLOAT, PM_JETPACK, PM_NORMAL, PM_SPECTATOR};
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT, PW_REDFLAG,
};
use mp_bg::public::stat_index::statIndex_t::{STAT_HEALTH, STAT_WEAPONS};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED};
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_bg::weapons::weapon_t::{WP_EMPLACED_GUN, WP_NONE};
use mp_qshared::common::mp::game::class_t::class_t::CLASS_VEHICLE;
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::qcommon::PMF_FOLLOW;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::force_powers::{FORCE_DARKSIDE, FORCE_LIGHTSIDE};
use mp_qshared::shared::q_math::{_VectorSubtract, vec3_origin, LerpAngle};
use mp_qshared::shared::surface_flags::SOLID_BMODEL;
use mp_qshared::shared::{
    qfalse, qtrue, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS_I32, MAX_GENTITIES,
};

use crate::cg_players::CG_G2TraceCollide;
use crate::local::player_state_ref::PlayerStateRef;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `#define CG_SEND_PS_POOL_SIZE 64` — the `_XBOX` playerstate pool's
/// length. Retail PC takes the `#else` arm (a full `MAX_GENTITIES` pool), so
/// nothing reads this; it is kept as the file's one `#define`.
/// Source: `oracle/codemp/cgame/cg_predict.c:848`
pub const CG_SEND_PS_POOL_SIZE: usize = 64;

/// Raven `CG_Piloting` — is this client piloting this veh?
///
/// Source: `oracle/codemp/cgame/cg_predict.c:18-35`
pub fn CG_Piloting(world: &CgWorld, vehNum: c_int) -> bool {
    if vehNum == 0 {
        return false;
    }

    let veh = &world.entities[vehNum as usize];

    if veh.currentState.owner != world.cg.predictedPlayerState.clientNum {
        //the owner should be the current pilot
        return false;
    }

    true
}

/// Raven `CG_BuildSolidList`.
///
/// Raven: When a new cg.snap has been set, this function builds a sublist of
/// the entities that are actually solid, to make for more efficient collision
/// detection.
/// Source: `oracle/codemp/cgame/cg_predict.c:46-141`
pub fn CG_BuildSolidList(world: &mut CgWorld) {
    world.predict.cg_numSolidEntities = 0;
    world.predict.cg_numTriggerEntities = 0;

    let useNext = !world.cg.nextSnap.is_null()
        && world.cg.nextFrameTeleport == qfalse
        && world.cg.thisFrameTeleport == qfalse;

    // §F19: Raven walks `snap->numEntities` with no null check - before the first
    // snapshot that's a null deref, so the port leaves both lists empty.
    if !useNext && world.cg.snap.is_null() {
        return;
    }

    let snap = if useNext {
        world.cg.next_snap_ref()
    } else {
        world.cg.snap_ref()
    };
    let Some(snap) = snap else {
        unreachable!("CG_BuildSolidList: cg.snap points outside activeSnapshots")
    };

    let numEntities = snap.numEntities;
    let origin = snap.ps.origin;
    for i in 0..numEntities as usize {
        let num = snap.entities[i].number as usize;
        let eType = world.entities[num].currentState.eType;

        if eType == ET_ITEM as c_int
            || eType == ET_PUSH_TRIGGER as c_int
            || eType == ET_TELEPORT_TRIGGER as c_int
        {
            let n = world.predict.cg_numTriggerEntities as usize;
            world.predict.cg_triggerEntities[n] = num as c_int;
            world.predict.cg_numTriggerEntities += 1;
            continue;
        }

        if world.entities[num].nextState.solid != 0 {
            let n = world.predict.cg_numSolidEntities as usize;
            world.predict.cg_solidEntities[n] = num as c_int;
            world.predict.cg_numSolidEntities += 1;
            continue;
        }
    }

    //rww - Horrible, terrible, awful hack.
    //We don't send your client entity from the server,
    //so it isn't added into the solid list from the snapshot,
    //and in addition, it has no solid data. So we will force
    //adding it in based on a hardcoded player bbox size.
    //This will cause issues if the player box size is ever
    //changed..
    if (world.predict.cg_numSolidEntities as usize) < MAX_ENTITIES_IN_SNAPSHOT {
        let playerMins: vec3_t = [-15.0, -15.0, DEFAULT_MINS_2 as f32];
        let playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];

        let mut i = playerMaxs[0] as c_int;
        if i < 1 {
            i = 1;
        }
        if i > 255 {
            i = 255;
        }

        // z is not symetric
        let mut j = (-playerMins[2]) as c_int;
        if j < 1 {
            j = 1;
        }
        if j > 255 {
            j = 255;
        }

        // and z playerMaxs can be negative...
        let mut k = (playerMaxs[2] + 32.0) as c_int;
        if k < 1 {
            k = 1;
        }
        if k > 255 {
            k = 255;
        }

        let n = world.predict.cg_numSolidEntities as usize;
        let clientNum = world.cg.predictedPlayerState.clientNum;
        world.predict.cg_solidEntities[n] = clientNum;
        world.entities[clientNum as usize].currentState.solid = (k << 16) | (j << 8) | i;

        world.predict.cg_numSolidEntities += 1;
    }

    let mut dsquared: f32 = /*RMG_distancecull.value*/ 5000.0 + 500.0;
    dsquared *= dsquared;

    // Raven's cg_permanents/cg_numpermanents pair (`cg_main.c:695-696`) is
    // `CgMainState::cg_permanents` — a Vec of entity numbers, so the count is
    // its len.
    for i in 0..world.main.cg_permanents.len() {
        let num = world.main.cg_permanents[i];
        let mut difference: vec3_t = [0.0; 3];
        _VectorSubtract(world.entities[num].lerpOrigin, origin, &mut difference);
        if world.entities[num].currentState.eType == ET_TERRAIN as c_int
            || ((difference[0] * difference[0])
                + (difference[1] * difference[1])
                + (difference[2] * difference[2]))
                <= dsquared
        {
            world.entities[num].currentValid = qtrue;
            if world.entities[num].nextState.solid != 0 {
                let n = world.predict.cg_numSolidEntities as usize;
                // §F19: Raven's permanents pass has no cap check, so enough
                // permanents walk `cg_solidEntities` off its end; the port's
                // bounds check panics on that instead of corrupting neighbours.
                // Source: `oracle/codemp/cgame/cg_predict.c:132-133`
                world.predict.cg_solidEntities[n] = num as c_int;
                world.predict.cg_numSolidEntities += 1;
            }
        } else {
            world.entities[num].currentValid = qfalse;
        }
    }
}

/// Raven `CG_VehicleClipCheck` — does the traced-against entity belong to the
/// same vehicle/rider pair as `ignored`, i.e. may the two pass through each
/// other.
///
/// PORT-NOTE: Raven takes `centity_t *ignored` plus a `!trace` guard. Both call
/// sites (`cg_predict.c:305,338`) reach here only inside `if (ignored &&
/// ignored->currentState.m_iVehicleNum)` with `&trace` of a live local, so
/// `ignored` is an entity number (`ignored = &cg_entities[skipNumber]`,
/// `cg_predict.c:227-230`) and the null-trace arm is unreachable.
/// Source: `oracle/codemp/cgame/cg_predict.c:143-200`
pub fn CG_VehicleClipCheck(world: &CgWorld, ignored: usize, trace: &trace_t) -> bool {
    // trace_t.entityNum is Raven's c_short; widen for the ENTITYNUM_WORLD test
    if trace.entityNum < 0 || trace.entityNum as c_int >= ENTITYNUM_WORLD {
        //it's alright then
        return true;
    }

    let ignored = &world.entities[ignored].currentState;

    if ignored.eType != ET_PLAYER as c_int && ignored.eType != ET_NPC as c_int {
        //can't possibly be valid then
        return true;
    }

    if ignored.m_iVehicleNum != 0 {
        //see if the ignore ent is a vehicle/rider - if so, see if the ent we supposedly hit is a vehicle/rider.
        //if they belong to each other, we don't want to collide them.
        let otherguy = &world.entities[trace.entityNum as usize].currentState;

        if otherguy.eType != ET_PLAYER as c_int && otherguy.eType != ET_NPC as c_int {
            //can't possibly be valid then
            return true;
        }

        if otherguy.m_iVehicleNum != 0 {
            //alright, both of these are either a vehicle or a player who is on a vehicle
            let index;

            if ignored.eType == ET_PLAYER as c_int
                || (ignored.eType == ET_NPC as c_int && ignored.NPC_class != CLASS_VEHICLE as c_int)
            {
                //must be a player or NPC riding a vehicle
                index = ignored.m_iVehicleNum;
            } else {
                //a vehicle
                index = ignored.m_iVehicleNum - 1;
            }

            if index == otherguy.number {
                //this means we're riding or being ridden by this guy, so don't collide
                return false;
            } else {
                //see if I'm hitting one of my own passengers
                if otherguy.eType == ET_PLAYER as c_int
                    || (otherguy.eType == ET_NPC as c_int
                        && otherguy.NPC_class != CLASS_VEHICLE as c_int)
                {
                    //must be a player or NPC riding a vehicle
                    if otherguy.m_iVehicleNum == ignored.number {
                        //this means we're other guy is riding the ignored ent
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Raven `CG_PointContents`.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:393-424`
pub fn CG_PointContents(ctx: &mut CgContext, point: &vec3_t, passEntityNum: c_int) -> c_int {
    let engine = ctx.engine;

    let mut contents = trap::CM_PointContents(engine, point, 0);

    for i in 0..ctx.world.predict.cg_numSolidEntities as usize {
        let num = ctx.world.predict.cg_solidEntities[i] as usize;
        let ent = &ctx.world.entities[num].currentState;
        let (number, solid, modelindex, origin, angles) = (
            ent.number,
            ent.solid,
            ent.modelindex,
            ent.origin,
            ent.angles,
        );

        if number == passEntityNum {
            continue;
        }

        if solid != SOLID_BMODEL {
            // special value for bmodel
            continue;
        }

        let cmodel = trap::CM_InlineModel(engine, modelindex);
        if cmodel == 0 {
            continue;
        }

        contents |= trap::CM_TransformedPointContents(engine, point, cmodel, &origin, &angles);
    }

    contents
}

/// Raven `CG_InterpolatePlayerState` — fills `cg.predictedPlayerState` by
/// lerping between the two snapshots.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:435-485`
pub fn CG_InterpolatePlayerState(ctx: &mut CgContext, grabAngles: bool) {
    // §F19: Raven derefs `cg.snap` unguarded here too; with no snapshot the port
    // leaves `cg.predictedPlayerState` alone.
    let Some(prev) = ctx.world.cg.snap_ref() else {
        return;
    };
    let prevPs = prev.ps;
    let prevServerTime = prev.serverTime;

    ctx.world.cg.predictedPlayerState = prevPs;

    // if we are still allowing local input, short circuit the view angles
    if grabAngles {
        let cmdNum = trap::GetCurrentCmdNumber(ctx.engine);
        let mut cmd = usercmd_t::default();
        trap::GetUserCmd(ctx.engine, cmdNum, &mut cmd);

        // DEFERRED: PM_UpdateViewAngles — `oracle/codemp/game/bg_pmove.c:7897`.
        // The ported body is an `mp_bg::bg_channel::PmoveContext` method, and a
        // `PmoveContext` needs a `&dyn BgTraps`; cgame has no `BgTraps`
        // implementor yet (`crates/mp/cgame/src/bg_channel/mod.rs`: "the
        // `BgTraps` one follows with the transcription waves").
    }

    // if the next frame is a teleport, we can't lerp to it
    if ctx.world.cg.nextFrameTeleport != qfalse {
        return;
    }

    let Some(next) = ctx.world.cg.next_snap_ref() else {
        return;
    };
    let nextPs = next.ps;
    let nextServerTime = next.serverTime;
    if nextServerTime <= prevServerTime {
        return;
    }

    let f = (ctx.world.cg.time - prevServerTime) as f32 / (nextServerTime - prevServerTime) as f32;

    let prevBobCycle = prevPs.bobCycle;
    let mut i = nextPs.bobCycle;
    if i < prevBobCycle {
        i += 256; // handle wraparound
    }
    ctx.world.cg.predictedPlayerState.bobCycle =
        (prevBobCycle as f32 + f * (i - prevBobCycle) as f32) as c_int;

    let prevOrigin = prevPs.origin;
    let nextOrigin = nextPs.origin;
    let prevViewangles = prevPs.viewangles;
    let nextViewangles = nextPs.viewangles;
    let prevVelocity = prevPs.velocity;
    let nextVelocity = nextPs.velocity;
    let out = &mut ctx.world.cg.predictedPlayerState;
    for i in 0..3 {
        out.origin[i] = prevOrigin[i] + f * (nextOrigin[i] - prevOrigin[i]);
        if !grabAngles {
            out.viewangles[i] = LerpAngle(prevViewangles[i], nextViewangles[i], f);
        }
        out.velocity[i] = prevVelocity[i] + f * (nextVelocity[i] - prevVelocity[i]);
    }
}

/// Raven `CG_InterpolateVehiclePlayerState` — the same lerp against the
/// snapshot's `vps`, filling `cg.predictedVehicleState`.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:487-537`
pub fn CG_InterpolateVehiclePlayerState(ctx: &mut CgContext, grabAngles: bool) {
    // §F19: same unguarded `cg.snap` deref as its player-state twin.
    let Some(prev) = ctx.world.cg.snap_ref() else {
        return;
    };
    let prevVps = prev.vps;
    let prevServerTime = prev.serverTime;

    ctx.world.cg.predictedVehicleState = prevVps;

    // if we are still allowing local input, short circuit the view angles
    if grabAngles {
        let cmdNum = trap::GetCurrentCmdNumber(ctx.engine);
        let mut cmd = usercmd_t::default();
        trap::GetUserCmd(ctx.engine, cmdNum, &mut cmd);

        // DEFERRED: PM_UpdateViewAngles — `oracle/codemp/game/bg_pmove.c:7897`;
        // same missing cgame `BgTraps` implementor as
        // `CG_InterpolatePlayerState`.
    }

    // if the next frame is a teleport, we can't lerp to it
    if ctx.world.cg.nextFrameTeleport != qfalse {
        return;
    }

    let Some(next) = ctx.world.cg.next_snap_ref() else {
        return;
    };
    let nextVps = next.vps;
    let nextServerTime = next.serverTime;
    if nextServerTime <= prevServerTime {
        return;
    }

    let f = (ctx.world.cg.time - prevServerTime) as f32 / (nextServerTime - prevServerTime) as f32;

    let prevBobCycle = prevVps.bobCycle;
    let mut i = nextVps.bobCycle;
    if i < prevBobCycle {
        i += 256; // handle wraparound
    }
    ctx.world.cg.predictedVehicleState.bobCycle =
        (prevBobCycle as f32 + f * (i - prevBobCycle) as f32) as c_int;

    let prevOrigin = prevVps.origin;
    let nextOrigin = nextVps.origin;
    let prevViewangles = prevVps.viewangles;
    let nextViewangles = nextVps.viewangles;
    let prevVelocity = prevVps.velocity;
    let nextVelocity = nextVps.velocity;
    let out = &mut ctx.world.cg.predictedVehicleState;
    for i in 0..3 {
        out.origin[i] = prevOrigin[i] + f * (nextOrigin[i] - prevOrigin[i]);
        if !grabAngles {
            out.viewangles[i] = LerpAngle(prevViewangles[i], nextViewangles[i], f);
        }
        out.velocity[i] = prevVelocity[i] + f * (nextVelocity[i] - prevVelocity[i]);
    }
}

/// Raven `CG_TouchItem` — predicts picking up the item entity `cent`.
///
/// PORT-NOTE: the flag and enlightenment checks read `GItem::giTag()` rather
/// than matching `ItemKind`, because the CTF block below compares `giTag` with
/// no `giType` qualifier — any item whose tag happens to equal
/// `PW_REDFLAG`/`PW_BLUEFLAG` takes that early return. That is Raven's behavior
/// and it stays (§A2).
/// Source: `oracle/codemp/cgame/cg_predict.c:544-660`
pub fn CG_TouchItem(world: &mut CgWorld, cent: usize) {
    if world.cvars.cg_predictItems.integer == 0 {
        return;
    }
    let time = world.cg.time;
    if BG_PlayerTouchesItem(
        &mut world.cg.predictedPlayerState,
        &mut world.entities[cent].currentState,
        time,
    ) == qfalse
    {
        return;
    }

    if world.entities[cent].currentState.brokenLimbs != 0 {
        //dropped item
        return;
    }

    if (world.entities[cent].currentState.eFlags & EF_ITEMPLACEHOLDER) != 0 {
        return;
    }

    if (world.entities[cent].currentState.eFlags & EF_NODRAW) != 0 {
        return;
    }

    // never pick an item up twice in a prediction
    if world.entities[cent].miscTime == world.cg.time {
        return;
    }

    if BG_CanItemBeGrabbed(
        world.cgs.gametype,
        &world.entities[cent].currentState,
        &world.cg.predictedPlayerState,
    ) == qfalse
    {
        return; // can't hold it
    }

    let item = &bg_itemlist[world.entities[cent].currentState.modelindex as usize];

    //Currently there is no reliable way of knowing if the client has touched a certain item before another if they are next to each other, or rather
    //if the server has touched them in the same order. This results often in grabbing an item in the prediction and the server giving you the other
    //item. So for now prediction of armor, health, and ammo is disabled.

    // Special case for flags.
    // We don't predict touching our own flag
    if world.cgs.gametype == GT_CTF || world.cgs.gametype == GT_CTY {
        if world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == TEAM_RED
            && item.giTag() == PW_REDFLAG
        {
            return;
        }
        if world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == TEAM_BLUE
            && item.giTag() == PW_BLUEFLAG
        {
            return;
        }
    }

    if item.giType() == IT_POWERUP
        && (item.giTag() == PW_FORCE_ENLIGHTENED_LIGHT || item.giTag() == PW_FORCE_ENLIGHTENED_DARK)
    {
        if item.giTag() == PW_FORCE_ENLIGHTENED_LIGHT {
            if world.cg.predictedPlayerState.fd.forceSide != FORCE_LIGHTSIDE {
                return;
            }
        } else if world.cg.predictedPlayerState.fd.forceSide != FORCE_DARKSIDE {
            return;
        }
    }

    // grab it
    let number = world.entities[cent].currentState.number;
    BG_AddPredictableEventToPlayerstate(
        EV_ITEM_PICKUP as c_int,
        number,
        &mut world.cg.predictedPlayerState,
    );

    // remove it from the frame so it won't be drawn
    world.entities[cent].currentState.eFlags |= EF_NODRAW;

    // don't touch it again this prediction
    world.entities[cent].miscTime = world.cg.time;

    // if its a weapon, give them some predicted ammo so the autoswitch will work
    if item.giType() == IT_WEAPON {
        let tag = item.giTag() as usize;
        world.cg.predictedPlayerState.stats[STAT_WEAPONS as usize] |= 1 << item.giTag();
        if world.cg.predictedPlayerState.ammo[tag] == 0 {
            world.cg.predictedPlayerState.ammo[tag] = 1;
        }
    }
}

/// Raven `CG_PmoveClientPointerUpdate`.
///
/// Raven: Assign all the entity playerstate pointers to the corresponding one
/// so that we can access playerstate stuff in bg code (and then translate it
/// back to entitystate data).
///
/// PORT-NOTE: Raven's `cg_pmove.baseEnt = (bgEntity_t *)cg_entities` /
/// `cg_pmove.entSize = sizeof(centity_t)` pair is deliberately not set here.
/// `pmove_t` still carries both fields and `PM_BGEntForNum` is still the
/// head-overlay pun that reads through them (`bg_pmove.rs`), but DEC-46.2
/// reshaped `centity_t`'s head — `playerState`/`m_pVehicle` are now a
/// `PlayerStateRef` and an `Option<VehicleId>` where Raven had two 8-byte
/// pointers — so an overlay onto `cg_entities` would misread every field. The
/// cgame pmove entity seam is an open DEC-46 design point and it blocks the
/// `CG_PredictPlayerState` wave.
/// Source: `oracle/codemp/cgame/cg_predict.c:913-914`
pub fn CG_PmoveClientPointerUpdate(world: &mut CgWorld) {
    // DEFERRED: cgSendPSPool — `oracle/codemp/cgame/cg_predict.c:853,888`. The
    // `playerState_t cgSendPSPool[MAX_GENTITIES]` backing store for
    // `PlayerStateRef::Snap` has no home in DEC-46 yet, and `playerState_t` has
    // no safe zeroed constructor for this wave to build one from (no `Default`,
    // no `zeroed`, and `unsafe` is off the table here). Raven's `memset` of the
    // pool belongs at this line.

    for i in 0..MAX_GENTITIES {
        // Raven stores `&cgSendPSPool[i]`, i.e. entity `i`'s own snapshot
        // playerstate — the DEC-46.2 `Snap` arm.
        world.entities[i].playerState = PlayerStateRef::Snap;
    }

    world.predict.cg_pmove.ghoul2 = null_mut();
}

/// Raven `CG_UsingEWeb` — check if local client is on an eweb.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:920-929`
pub fn CG_UsingEWeb(world: &CgWorld) -> bool {
    if world.cg.predictedPlayerState.weapon == WP_EMPLACED_GUN
        && world.cg.predictedPlayerState.emplacedIndex != 0
        && world.entities[world.cg.predictedPlayerState.emplacedIndex as usize]
            .currentState
            .weapon
            == WP_NONE
    {
        return true;
    }

    false
}

/// Raven `CG_ClipMoveToEntities` — sweeps a box against `cg_solidEntities`,
/// tightening `tr` against whichever solid entity it lands on first.
///
/// PORT-NOTE: the dynamic vehicle-bbox-orientation adjust (`cent->m_pVehicle
/// ->m_vOrientation` swap around `BG_VehicleAdjustBBoxForOrientation`) is
/// dropped — DEC-46.2's `Option<VehicleId>` on `centity_t` carries only the
/// vehicle cent's entity number, presence-only, until the `Vehicle_t`
/// referent pool lands (`CG_VehicleEffects`, `cg_players.rs:7981-8305`, is the
/// established precedent for this exact deferral). The encoded bbox still
/// gets built and traced with the un-adjusted extents.
/// Source: `oracle/codemp/cgame/cg_predict.c:216-351`
#[allow(clippy::too_many_arguments)]
pub fn CG_ClipMoveToEntities(
    ctx: &mut CgContext,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
    tr: &mut trace_t,
    g2Check: bool,
) {
    let ignored: Option<usize> = if skipNumber != -1 && skipNumber != ENTITYNUM_NONE {
        Some(skipNumber as usize)
    } else {
        None
    };
    let ignoredHasVeh = ignored
        .map(|idx| ctx.world.entities[idx].currentState.m_iVehicleNum != 0)
        .unwrap_or(false);

    let numSolid = ctx.world.predict.cg_numSolidEntities as usize;
    for i in 0..numSolid {
        let num = ctx.world.predict.cg_solidEntities[i] as usize;
        let entNumber = ctx.world.entities[num].currentState.number;

        if entNumber == skipNumber {
            continue;
        }

        let genericenemyindex = ctx.world.entities[num].currentState.genericenemyindex;
        if entNumber > MAX_CLIENTS_I32
            && (genericenemyindex - MAX_GENTITIES as c_int
                == ctx.world.cg.predictedPlayerState.clientNum
                || genericenemyindex - MAX_GENTITIES as c_int
                    == ctx.world.cg.predictedVehicleState.clientNum)
        {
            //rww - method of keeping objects from colliding in client-prediction (in case of ownership)
            continue;
        }

        let solid = ctx.world.entities[num].currentState.solid;
        let cmodel;
        let origin: vec3_t;
        let angles: vec3_t;

        if solid == SOLID_BMODEL {
            // special value for bmodel
            let modelindex = ctx.world.entities[num].currentState.modelindex;
            cmodel = trap::CM_InlineModel(ctx.engine, modelindex);
            angles = ctx.world.entities[num].lerpAngles;
            let mut o: vec3_t = [0.0; 3];
            BG_EvaluateTrajectory(
                &ctx.world.entities[num].currentState.pos,
                ctx.world.cg.physicsTime,
                &mut o,
            );
            origin = o;
        } else {
            // encoded bbox
            let x = (solid & 255) as f32;
            let zd = ((solid >> 8) & 255) as f32;
            let zu = (((solid >> 16) & 255) - 32) as f32;

            let bmins: vec3_t = [-x, -x, -zd];
            let bmaxs: vec3_t = [x, x, zu];

            // PORT-NOTE: Raven dynamically widens `bmins`/`bmaxs` here for a
            // vehicle NPC ("if (ent->eType == ET_NPC && ent->NPC_class ==
            // CLASS_VEHICLE && cent->m_pVehicle) BG_VehicleAdjustBBoxForOrientation(...)").
            // See the fn doc — unreachable until the Vehicle_t referent pool
            // lands, so the un-adjusted encoded bbox is traced instead.

            cmodel = trap::CM_TempBoxModel(ctx.engine, &bmins, &bmaxs);
            angles = vec3_origin;
            origin = ctx.world.entities[num].lerpOrigin;
        }

        let mut trace = trace_t::zeroed();
        trap::CM_TransformedBoxTrace(
            ctx.engine, &mut trace, start, end, mins, maxs, cmodel, mask, &origin, &angles,
        );
        trace.entityNum = if trace.fraction != 1.0 {
            entNumber as i16
        } else {
            ENTITYNUM_NONE as i16
        };

        let mut oldTrace = trace_t::zeroed();
        if g2Check || ignoredHasVeh {
            oldTrace = *tr;
        }

        if trace.allsolid != 0 || trace.fraction < tr.fraction {
            trace.entityNum = entNumber as i16;
            *tr = trace;
        } else if trace.startsolid != 0 {
            tr.startsolid = qtrue as u8;

            //rww 12-02-02
            trace.entityNum = entNumber as i16;
            tr.entityNum = trace.entityNum;
        }

        if tr.allsolid != 0 {
            if ignoredHasVeh {
                let ignoredIdx = ignored.unwrap();
                trace.entityNum = entNumber as i16;
                if CG_VehicleClipCheck(ctx.world, ignoredIdx, &trace) {
                    //this isn't our vehicle, we're really stuck
                    return;
                } else {
                    //it's alright, keep going
                    trace = oldTrace;
                    *tr = trace;
                }
            } else {
                return;
            }
        }

        if g2Check {
            let ghoul2 = ctx.world.entities[num].ghoul2;
            if trace.entityNum == entNumber as i16 && !ghoul2.is_null() {
                CG_G2TraceCollide(ctx, &mut trace, Some(mins), Some(maxs), start, end);

                if trace.entityNum == ENTITYNUM_NONE as i16 {
                    //g2 trace failed, so put it back where it was.
                    trace = oldTrace;
                    *tr = trace;
                }
            }
        }

        if ignoredHasVeh {
            //see if this is the vehicle we hit
            let ignoredIdx = ignored.unwrap();
            let hitIdx = trace.entityNum as usize;
            if !CG_VehicleClipCheck(ctx.world, ignoredIdx, &trace) {
                //looks like it
                trace = oldTrace;
                *tr = trace;
            } else if ctx.world.entities[hitIdx].currentState.eType == ET_MISSILE as c_int
                && ctx.world.entities[hitIdx].currentState.owner
                    == ctx.world.entities[ignoredIdx].currentState.number
            {
                //hack, don't hit own missiles
                trace = oldTrace;
                *tr = trace;
            }
        }
    }
}

/// Raven `CG_TouchTriggerPrediction` — fires predicted item pickups and
/// trigger touches (teleport/push) against `cg_triggerEntities`.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:670-726`
pub fn CG_TouchTriggerPrediction(ctx: &mut CgContext) {
    // dead clients don't activate triggers
    if ctx.world.cg.predictedPlayerState.stats[STAT_HEALTH as usize] <= 0 {
        return;
    }

    let spectator = ctx.world.cg.predictedPlayerState.pm_type == PM_SPECTATOR as c_int;

    if ctx.world.cg.predictedPlayerState.pm_type != PM_NORMAL as c_int
        && ctx.world.cg.predictedPlayerState.pm_type != PM_JETPACK as c_int
        && ctx.world.cg.predictedPlayerState.pm_type != PM_FLOAT as c_int
        && !spectator
    {
        return;
    }

    let numTrigger = ctx.world.predict.cg_numTriggerEntities as usize;
    for i in 0..numTrigger {
        let num = ctx.world.predict.cg_triggerEntities[i] as usize;
        let eType = ctx.world.entities[num].currentState.eType;

        if eType == ET_ITEM as c_int && !spectator {
            CG_TouchItem(ctx.world, num);
            continue;
        }

        if ctx.world.entities[num].currentState.solid != SOLID_BMODEL {
            continue;
        }

        let modelindex = ctx.world.entities[num].currentState.modelindex;
        let cmodel = trap::CM_InlineModel(ctx.engine, modelindex);
        if cmodel == 0 {
            continue;
        }

        let mut trace = trace_t::zeroed();
        let origin = ctx.world.cg.predictedPlayerState.origin;
        let pmins = ctx.world.predict.cg_pmove.mins;
        let pmaxs = ctx.world.predict.cg_pmove.maxs;
        trap::CM_BoxTrace(
            ctx.engine, &mut trace, &origin, &origin, &pmins, &pmaxs, cmodel, -1,
        );

        if trace.startsolid == 0 {
            continue;
        }

        if eType == ET_TELEPORT_TRIGGER as c_int {
            ctx.world.cg.hyperspace = qtrue;
        } else if eType == ET_PUSH_TRIGGER as c_int {
            BG_TouchJumpPad(
                &raw mut ctx.world.cg.predictedPlayerState,
                &raw mut ctx.world.entities[num].currentState,
            );
        }
    }

    // if we didn't touch a jump pad this pmove frame
    if ctx.world.cg.predictedPlayerState.jumppad_frame
        != ctx.world.cg.predictedPlayerState.pmove_framecount
    {
        ctx.world.cg.predictedPlayerState.jumppad_frame = 0;
        ctx.world.cg.predictedPlayerState.jumppad_ent = 0;
    }
}

/// Raven `CG_Trace` — a straight `trap_CM_BoxTrace` sweep, then narrowed
/// against `cg_solidEntities` (no ghoul2 sub-model check).
/// Source: `oracle/codemp/cgame/cg_predict.c:359-369`
#[allow(clippy::too_many_arguments)]
pub fn CG_Trace(
    ctx: &mut CgContext,
    result: &mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
) {
    let mut t = trace_t::zeroed();

    trap::CM_BoxTrace(ctx.engine, &mut t, start, end, mins, maxs, 0, mask);
    t.entityNum = if t.fraction != 1.0 {
        ENTITYNUM_WORLD as i16
    } else {
        ENTITYNUM_NONE as i16
    };
    // check all other solid models
    CG_ClipMoveToEntities(ctx, start, mins, maxs, end, skipNumber, mask, &mut t, false);

    *result = t;
}

/// Raven `CG_G2Trace` — identical to [`CG_Trace`] but flips the
/// `CG_ClipMoveToEntities` `g2Check` arm on, so the sweep also probes ghoul2
/// sub-models.
/// Source: `oracle/codemp/cgame/cg_predict.c:376-386`
#[allow(clippy::too_many_arguments)]
pub fn CG_G2Trace(
    ctx: &mut CgContext,
    result: &mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
) {
    let mut t = trace_t::zeroed();

    trap::CM_BoxTrace(ctx.engine, &mut t, start, end, mins, maxs, 0, mask);
    t.entityNum = if t.fraction != 1.0 {
        ENTITYNUM_WORLD as i16
    } else {
        ENTITYNUM_NONE as i16
    };
    // check all other solid models
    CG_ClipMoveToEntities(ctx, start, mins, maxs, end, skipNumber, mask, &mut t, true);

    *result = t;
}

/// Raven `CG_PredictPlayerState` — generates `cg.predictedPlayerState` for the
/// current cg.time from the last snapshot plus the pending usercmds, running
/// the same `Pmove` the server did.
///
/// PORT-NOTE: only the three non-predicting prologue paths are transcribed
/// here — the first-frame `validPPS` seed and the demo/follow +
/// nopredict/synchronous/eweb interpolation early-returns, all of which land
/// clean. The predicting body (the whole `Pmove`-driven remainder from Raven's
/// "prepare for pmove", `cg_predict.c:1007`) is blocked on two DEC-46 design
/// points, both already cited by [`CG_PmoveClientPointerUpdate`]:
///
/// 1. The cgame pmove entity seam. Raven binds `cg_pmove.ps =
///    &cg.predictedPlayerState`, `cg_pmove.trace = CG_Trace`,
///    `cg_pmove.pointcontents = CG_PointContents`, and
///    `cg_pmove.baseEnt = (bgEntity_t *)cg_entities` before `Pmove`. The
///    ported `Pmove` drives `self.traps.trace()` / `self.traps.pointcontents()`
///    (`bg_pmove.rs`), and `CgBgTraps`'s two methods are still `todo!()`s
///    because the seam carries `&Engine`, not the `&mut CgContext` the ported
///    `CG_Trace`/`CG_PointContents` need
///    (`bg_channel/cg_bg_traps.rs:67-96`). The raw self-pointer into
///    `predictedPlayerState` and the `baseEnt` overlay pun are the open DEC-46
///    ruling `CG_PmoveClientPointerUpdate` names as blocking this wave.
/// 2. `cgSendPSPool`. The `VectorCopy(... cgSendPS[i]->origin)` pump and the
///    `revertES` copy-back read/write `playerState_t cgSendPSPool[MAX_GENTITIES]`
///    (`cg_predict.c:853,888`), which has no DEC-46 home yet — the same
///    deferral `CG_PmoveClientPointerUpdate` records.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:963-1511`
pub fn CG_PredictPlayerState(ctx: &mut CgContext) {
    ctx.world.cg.hyperspace = qfalse; // will be set if touching a trigger_teleport

    // if this is the first frame we must guarantee predictedPlayerState is
    // valid even if there is some other error condition
    if ctx.world.cg.validPPS == qfalse {
        ctx.world.cg.validPPS = qtrue;
        // §F19: Raven derefs `cg.snap` unguarded; before the first snapshot the
        // port leaves `predictedPlayerState`/`predictedVehicleState` alone.
        if let Some((ps, vps)) = ctx.world.cg.snap_ref().map(|s| (s.ps, s.vps)) {
            ctx.world.cg.predictedPlayerState = ps;
            if CG_Piloting(ctx.world, ps.m_iVehicleNum) {
                ctx.world.cg.predictedVehicleState = vps;
            }
        }
    }

    // demo playback just copies the moves
    // §F19: `cg.snap->ps.pm_flags` is an unguarded deref in Raven; with no
    // snapshot the follow test reads as unset.
    let demo_or_follow = ctx.world.cg.demoPlayback != qfalse
        || ctx
            .world
            .cg
            .snap_ref()
            .map_or(false, |s| s.ps.pm_flags & PMF_FOLLOW != 0);
    if demo_or_follow {
        CG_InterpolatePlayerState(ctx, false);
        let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
        if CG_Piloting(ctx.world, vehNum) {
            CG_InterpolateVehiclePlayerState(ctx, false);
        }
        return;
    }

    // non-predicting local movement will grab the latest angles
    if ctx.world.cvars.cg_nopredict.integer != 0
        || ctx.world.cvars.cg_synchronousClients.integer != 0
        || CG_UsingEWeb(ctx.world)
    {
        CG_InterpolatePlayerState(ctx, true);
        let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
        if CG_Piloting(ctx.world, vehNum) {
            CG_InterpolateVehiclePlayerState(ctx, true);
        }
        return;
    }

    //TODO: Port CG_PredictPlayerState
    // Source: `oracle/codemp/cgame/cg_predict.c:1007-1511`
    // The predicting body ("prepare for pmove" onward) is blocked on the two
    // DEC-46 design points documented in this fn's doc comment: the pmove
    // entity seam (raw `cg_pmove.ps`/`baseEnt`, plus `CgBgTraps::trace` /
    // `pointcontents` still `todo!()`) and the unhomed `cgSendPSPool`.
    todo!("Port CG_PredictPlayerState predicting body — oracle/codemp/cgame/cg_predict.c:1007-1511 (blocked on the DEC-46 pmove entity seam + cgSendPSPool; see CG_PmoveClientPointerUpdate)")
}
