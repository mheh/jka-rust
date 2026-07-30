//! Port of `oracle/codemp/cgame/cg_predict.c` — client-side movement prediction and its trace helpers. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::mem::size_of;
use core::ptr::null_mut;

use mp_abi::cgame::public::snapshot_t::MAX_ENTITIES_IN_SNAPSHOT;
use mp_bg::bg_channel::PmoveContext;
use mp_bg::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanItemBeGrabbed, BG_EvaluateTrajectory,
    BG_PlayerTouchesItem, BG_TouchJumpPad,
};
use mp_bg::bg_pmove::{BG_VehicleAdjustBBoxForOrientation, Pmove};
use mp_bg::public::bg_entity::bgEntity_t;
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::dm_flags::DF_NO_FOOTSTEPS;
use mp_bg::public::entity_event::entity_event_t::EV_ITEM_PICKUP;
use mp_bg::public::entity_flags::{EF_ITEMPLACEHOLDER, EF_NODRAW};
use mp_bg::public::entity_type::entityType_t::{
    ET_ITEM, ET_MISSILE, ET_NPC, ET_PLAYER, ET_PUSH_TRIGGER, ET_TELEPORT_TRIGGER, ET_TERRAIN,
};
use mp_bg::public::gametype::{GT_CTF, GT_CTY};
use mp_bg::public::item_type::{IT_POWERUP, IT_WEAPON};
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t::{
    PM_DEAD, PM_FLOAT, PM_INTERMISSION, PM_JETPACK, PM_NORMAL, PM_SPECTATOR,
};
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT, PW_REDFLAG,
};
use mp_bg::public::stat_index::statIndex_t::{STAT_HEALTH, STAT_WEAPONS};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_bg::weapons::weapon_t::{WP_EMPLACED_GUN, WP_NONE};
use mp_qshared::common::mp::game::class_t::class_t::CLASS_VEHICLE;
use mp_qshared::common::mp::qcommon::playerState_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_PS_EVENTS;
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t::{SS_DUAL, SS_STAFF};
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_TALK;
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::qcommon::PMF_FOLLOW;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::force_powers::{FORCE_DARKSIDE, FORCE_LIGHTSIDE};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorScale, _VectorSubtract, vec3_origin, vectoangles, AngleSubtract, LerpAngle,
    VectorClear, VectorCompare, VectorLength,
};
use mp_qshared::shared::surface_flags::{CONTENTS_BODY, MASK_PLAYERSOLID, SOLID_BMODEL};
use mp_qshared::shared::{
    qfalse, qtrue, vec3_t, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS_I32, MAX_GENTITIES,
};
use mp_uishared::shared::display_state::DisplayState;

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_ents::{CG_AdjustPositionForMover, CG_Cube};
use crate::cg_main::CG_Printf;
use crate::cg_players::CG_G2TraceCollide;
use crate::cg_playerstate::CG_TransitionPlayerState;
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
/// Raven's `cg_pmove.baseEnt = (bgEntity_t *)cg_entities` overlay pun cannot
/// read the DEC-46.2 reshaped `centity_t`, so `baseEnt` aims at the
/// `CgWorld.bg_ents` shadow rows instead (DEC-47.2) - real `bgEntity_t`s whose
/// `playerState` pointers wire up here and whose entity-state fields
/// `CG_PredictPlayerState` syncs before each `Pmove`. The stride is
/// `sizeof(bgEntity_t)`, not Raven's `sizeof(centity_t)`, because the shadow
/// array is the thing being walked.
/// Source: `oracle/codemp/cgame/cg_predict.c:883-918`
pub fn CG_PmoveClientPointerUpdate(world: &mut CgWorld) {
    // Raven: memset(&cgSendPSPool[0], 0, sizeof(cgSendPSPool));
    for ps in world.cgSendPSPool.iter_mut() {
        *ps = playerState_t::zeroed();
    }

    for i in 0..MAX_GENTITIES {
        // Raven stores `&cgSendPSPool[i]`, i.e. entity `i`'s own snapshot
        // playerstate — the DEC-46.2 `Snap` arm, and the live pointer on the
        // bg view row (raw-derived so later pool borrows don't retag it).
        world.entities[i].playerState = PlayerStateRef::Snap;
        world.bg_ents[i].playerState = &raw mut world.cgSendPSPool[i];
    }

    // Set up bg entity data
    world.predict.cg_pmove.baseEnt = world.bg_ents.as_mut_ptr();
    world.predict.cg_pmove.entSize = size_of::<bgEntity_t>() as c_int;

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
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
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

            let mut bmins: vec3_t = [-x, -x, -zd];
            let mut bmaxs: vec3_t = [x, x, zu];

            let eType = ctx.world.entities[num].currentState.eType;
            let npcClass = ctx.world.entities[num].currentState.NPC_class;
            if eType == ET_NPC as c_int && npcClass == CLASS_VEHICLE as c_int {
                if let Some(id) = ctx.world.entities[num].m_pVehicle {
                    //try to dynamically adjust his bbox dynamically, if possible
                    let row_idx = id.ent_num() as usize;
                    // Raven swaps `m_vOrientation` at the cent's lerpAngles
                    // around the call and back; the None trace channel is
                    // Raven's NULL localTrace arm (always accept)
                    let old = ctx.world.vehicle_pool[row_idx].m_vOrientation;
                    ctx.world.vehicle_pool[row_idx].m_vOrientation =
                        &raw mut ctx.world.entities[num].lerpAngles[0];
                    let pVeh = &raw mut ctx.world.vehicle_pool[row_idx];
                    let lerpOrigin = ctx.world.entities[num].lerpOrigin;
                    BG_VehicleAdjustBBoxForOrientation(
                        pVeh,
                        lerpOrigin,
                        &mut bmins,
                        &mut bmaxs,
                        entNumber,
                        MASK_PLAYERSOLID,
                        None,
                    );
                    ctx.world.vehicle_pool[row_idx].m_vOrientation = old;
                }
            }

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
                CG_G2TraceCollide(ctx, &mut trace, mins, maxs, start, end);

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
            ctx.engine,
            &mut trace,
            &origin,
            &origin,
            Some(&pmins),
            Some(&pmaxs),
            cmodel,
            -1,
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
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
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
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
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
/// The DEC-47.2 seam carries the body: `cg_pmove.ps` is the raw self-pointer
/// into `cg.predictedPlayerState` Raven stores, `trace`/`pointcontents` live
/// on `CgBgTraps`, and `baseEnt` walks the `CgWorld.bg_ents` shadow rows this
/// fn syncs from the entities up front (Raven's overlay read `cg_entities`
/// directly - nothing here writes those synced fields back except Raven's own
/// explicit `revertES` pump). The piloted-vehicle sub-blocks deref the
/// DEC-47.3 `vehicle_pool` rows behind `m_pVehicle`.
///
/// Source: `oracle/codemp/cgame/cg_predict.c:963-1511`
pub fn CG_PredictPlayerState(ctx: &mut CgContext, ds: &DisplayState) {
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

    // prepare for pmove
    // Raven also rebinds cg_pmove.trace/pointcontents here; those live on
    // CgBgTraps (DEC-47.2), and pmove_t's fn-ptr fields stay for layout only.
    ctx.world.predict.cg_pmove.ps = &raw mut ctx.world.cg.predictedPlayerState;

    // sync the bg view rows before bg walks them through baseEnt - port-only
    // plumbing for the overlay Raven read straight off cg_entities
    {
        let world = &mut *ctx.world;
        for i in 0..MAX_GENTITIES {
            let m_pVehicle = match world.entities[i].m_pVehicle {
                // bg reads the referent through pm_entVeh during vehicle
                // prediction, so the row carries the live pool address
                Some(id) => &raw mut world.vehicle_pool[id.ent_num() as usize],
                None => null_mut(),
            };
            let ent = &world.entities[i];
            let row = &mut world.bg_ents[i];
            row.s = ent.currentState;
            row.ghoul2 = ent.ghoul2;
            row.localAnimIndex = ent.localAnimIndex;
            row.modelScale = ent.modelScale;
            row.m_pVehicle = m_pVehicle;
        }
    }

    let pEntNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
    //rww - bgghoul2
    if ctx.world.predict.cg_pmove.ghoul2 != ctx.world.entity(pEntNum).ghoul2 {
        //only update it if the g2 instance has changed
        let pGhoul2 = ctx.world.entity(pEntNum).ghoul2;
        let snapOk = ctx.world.cg.snap_ref().is_some_and(|snap| {
            snap.ps.pm_flags & PMF_FOLLOW == 0
                && snap.ps.persistant[PERS_TEAM as usize] != TEAM_SPECTATOR
        });
        if snapOk && !pGhoul2.is_null() {
            ctx.world.predict.cg_pmove.ghoul2 = pGhoul2;
            ctx.world.predict.cg_pmove.g2Bolts_LFoot =
                trap::G2API_AddBolt(ctx.engine, pGhoul2, 0, "*l_leg_foot");
            ctx.world.predict.cg_pmove.g2Bolts_RFoot =
                trap::G2API_AddBolt(ctx.engine, pGhoul2, 0, "*r_leg_foot");
        } else {
            ctx.world.predict.cg_pmove.ghoul2 = null_mut();
        }
    }

    // Raven grabs `ci = &cgs.clientinfo[clientNum]` here; the saberHolstered
    // block below reads the row at its use site instead.

    //I'll just do this every frame in case the scale changes in realtime (don't need to update the g2 inst for that)
    ctx.world.predict.cg_pmove.modelScale = ctx.world.entity(pEntNum).modelScale;
    //rww end bgghoul2

    if ctx.world.cg.predictedPlayerState.pm_type == PM_DEAD as c_int {
        ctx.world.predict.cg_pmove.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY;
    } else {
        ctx.world.predict.cg_pmove.tracemask = MASK_PLAYERSOLID;
    }
    // §F19: the `cg.snap->` derefs from here down are unguarded in Raven;
    // CG_DrawActiveFrame never runs prediction without a snapshot, so a
    // missing snap reads as its zero value.
    if ctx
        .world
        .cg
        .snap_ref()
        .is_some_and(|s| s.ps.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR)
    {
        // spectators can fly through bodies
        ctx.world.predict.cg_pmove.tracemask &= !CONTENTS_BODY;
    }
    ctx.world.predict.cg_pmove.noFootsteps = if ctx.world.cgs.dmflags & DF_NO_FOOTSTEPS > 0 {
        qtrue
    } else {
        qfalse
    };

    // save the state before the pmove so we can detect transitions
    let mut oldPlayerState = ctx.world.cg.predictedPlayerState;
    // Raven's `oldVehicleState` local is uninitialized unless piloting. The
    // copy gate reads the CURRENT ps while the later reads gate on
    // oldPlayerState's vehicle, so a vehicle-num change between them can read
    // the seed - Raven read uninitialized stack there; zeros are the defined
    // stand-in
    let mut oldVehicleState = playerState_t::zeroed();
    if CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum) {
        oldVehicleState = ctx.world.cg.predictedVehicleState;
    }

    let current = trap::GetCurrentCmdNumber(ctx.engine);

    // if we don't have the commands right after the snapshot, we
    // can't accurately predict a current position, so just freeze at
    // the last good position we had
    let cmdNum = current - CMD_BACKUP + 1;
    let mut oldestCmd = usercmd_t::default();
    trap::GetUserCmd(ctx.engine, cmdNum, &mut oldestCmd);
    let snapCommandTime = ctx.world.cg.snap_ref().map_or(0, |s| s.ps.commandTime);
    if oldestCmd.serverTime > snapCommandTime && oldestCmd.serverTime < ctx.world.cg.time {
        // special check for map_restart
        if ctx.world.cvars.cg_showmiss.integer != 0 {
            CG_Printf(ctx, "exceeded PACKET_BACKUP on commands\n");
        }
        return;
    }

    // get the latest command so we can know which commands are from previous map_restarts
    let mut latestCmd = usercmd_t::default();
    trap::GetUserCmd(ctx.engine, current, &mut latestCmd);

    // get the most recent information we have, even if
    // the server time is beyond our current cg.time,
    // because predicted player positions are going to
    // be ahead of everything else anyway
    let slopeRecalcTime = ctx.world.cg.predictedPlayerState.slopeRecalcTime;
    if ctx.world.cg.next_snap_ref().is_some()
        && ctx.world.cg.nextFrameTeleport == qfalse
        && ctx.world.cg.thisFrameTeleport == qfalse
    {
        let (ps, vps, serverTime) = {
            let snap = ctx.world.cg.next_snap_mut().unwrap();
            //this is the only value we want to maintain seperately on server/client
            snap.ps.slopeRecalcTime = slopeRecalcTime;
            (snap.ps, snap.vps, snap.serverTime)
        };
        ctx.world.cg.predictedPlayerState = ps;
        if CG_Piloting(ctx.world, ps.m_iVehicleNum) {
            ctx.world.cg.predictedVehicleState = vps;
        }
        ctx.world.cg.physicsTime = serverTime;
    } else if let Some((ps, vps, serverTime)) = {
        let snap = ctx.world.cg.snap_mut();
        snap.map(|snap| {
            //this is the only value we want to maintain seperately on server/client
            snap.ps.slopeRecalcTime = slopeRecalcTime;
            (snap.ps, snap.vps, snap.serverTime)
        })
    } {
        ctx.world.cg.predictedPlayerState = ps;
        if CG_Piloting(ctx.world, ps.m_iVehicleNum) {
            ctx.world.cg.predictedVehicleState = vps;
        }
        ctx.world.cg.physicsTime = serverTime;
    }

    if ctx.world.cvars.pmove_msec.integer < 8 {
        trap::Cvar_Set(ctx.engine, "pmove_msec", "8");
    } else if ctx.world.cvars.pmove_msec.integer > 33 {
        trap::Cvar_Set(ctx.engine, "pmove_msec", "33");
    }

    ctx.world.predict.cg_pmove.pmove_fixed = ctx.world.cvars.pmove_fixed.integer; // | cg_pmove_fixed.integer;
    ctx.world.predict.cg_pmove.pmove_msec = ctx.world.cvars.pmove_msec.integer;

    {
        let world = &mut *ctx.world;
        for i in 0..MAX_GENTITIES {
            //Written this way for optimal speed, even though it doesn't look pretty.
            //(we don't want to spend the time assigning pointers as it does take
            //a small precious fraction of time and adds up in the loop.. so says
            //the precision timer!)

            let es = &world.entities[i].currentState;
            if es.eType == ET_PLAYER as c_int || es.eType == ET_NPC as c_int {
                let ps = &mut world.cgSendPSPool[i];
                ps.origin = es.pos.trBase;
                ps.velocity = es.pos.trDelta;
                ps.saberLockFrame = es.forceFrame;
                ps.legsAnim = es.legsAnim;
                ps.torsoAnim = es.torsoAnim;
                ps.legsFlip = es.legsFlip;
                ps.torsoFlip = es.torsoFlip;
                ps.clientNum = es.clientNum;
                ps.saberMove = es.saberMove;
            }
        }
    }

    if CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum) {
        let clientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
        let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;
        let world = &mut *ctx.world;
        // both rows repoint at the predicted states - enum arm for the cgame
        // readers, raw pointer for the bg view (DEC-47.2)
        world.entities[clientNum].playerState = PlayerStateRef::Predicted;
        world.bg_ents[clientNum].playerState = &raw mut world.cg.predictedPlayerState;
        world.entities[vehNum].playerState = PlayerStateRef::PredictedVehicle;
        world.bg_ents[vehNum].playerState = &raw mut world.cg.predictedVehicleState;

        //use the player command time, because we are running with the player cmds (this is even the case
        //on the server)
        world.cg.predictedVehicleState.commandTime = world.cg.predictedPlayerState.commandTime;
    }

    // run cmds
    let mut moved = qfalse;
    for cmdNum in (current - CMD_BACKUP + 1)..=current {
        // get the command
        trap::GetUserCmd(ctx.engine, cmdNum, &mut ctx.world.predict.cg_pmove.cmd);

        if ctx.world.predict.cg_pmove.pmove_fixed != 0 {
            let ps_ptr = &raw mut ctx.world.cg.predictedPlayerState;
            let cmd_ptr = &raw const ctx.world.predict.cg_pmove.cmd;
            let clientNum = ctx.world.cg.predictedPlayerState.clientNum;
            let m_iVehicleNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            let mut pmctx = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
            // Raven's pm/pm_entSelf/pm_entVeh are TU statics still holding
            // the last PmoveSingle's values at this call; the fresh context
            // re-derives them from the current ps (PmoveSingle's own rule) so
            // the fighter pitch-clamp arm in PM_UpdateViewAngles can see the
            // vehicle
            pmctx.pm = &raw mut ctx.world.predict.cg_pmove;
            pmctx.pm_entSelf = pmctx.PM_BGEntForNum(clientNum);
            pmctx.pm_entVeh = if m_iVehicleNum != 0 {
                if clientNum < MAX_CLIENTS_I32 {
                    // player riding vehicle
                    pmctx.PM_BGEntForNum(m_iVehicleNum)
                } else {
                    // vehicle with player pilot
                    pmctx.PM_BGEntForNum(m_iVehicleNum - 1)
                }
            } else {
                null_mut()
            };
            pmctx.PM_UpdateViewAngles(ps_ptr, cmd_ptr);
        }

        // don't do anything if the time is before the snapshot player time
        if ctx.world.predict.cg_pmove.cmd.serverTime
            <= ctx.world.cg.predictedPlayerState.commandTime
        {
            continue;
        }

        // don't do anything if the command was from a previous map_restart
        if ctx.world.predict.cg_pmove.cmd.serverTime > latestCmd.serverTime {
            continue;
        }

        // check for a prediction error from last frame
        // on a lan, this will often be the exact value
        // from the snapshot, but on a wan we will have
        // to predict several commands to get to the point
        // we want to compare
        if CG_Piloting(ctx.world, oldPlayerState.m_iVehicleNum)
            && ctx.world.cg.predictedVehicleState.commandTime == oldVehicleState.commandTime
        {
            if ctx.world.cg.thisFrameTeleport != qfalse {
                // a teleport will not cause an error decay
                VectorClear(&mut ctx.world.cg.predictedError);
                if ctx.world.cvars.cg_showVehMiss.integer != 0 {
                    CG_Printf(ctx, "VEH PredictionTeleport\n");
                }
                ctx.world.cg.thisFrameTeleport = qfalse;
            } else {
                let mut adjusted: vec3_t = [0.0; 3];
                CG_AdjustPositionForMover(
                    ctx.world,
                    ctx.world.cg.predictedVehicleState.origin,
                    ctx.world.cg.predictedVehicleState.groundEntityNum,
                    ctx.world.cg.physicsTime,
                    ctx.world.cg.oldTime,
                    &mut adjusted,
                );

                if ctx.world.cvars.cg_showVehMiss.integer != 0
                    && !VectorCompare(oldVehicleState.origin, adjusted)
                {
                    CG_Printf(ctx, "VEH prediction error\n");
                }
                let mut delta: vec3_t = [0.0; 3];
                _VectorSubtract(oldVehicleState.origin, adjusted, &mut delta);
                let len = VectorLength(delta);
                if len > 0.1 {
                    if ctx.world.cvars.cg_showVehMiss.integer != 0 {
                        CG_Printf(ctx, &format!("VEH Prediction miss: {:.6}\n", len));
                    }
                    if ctx.world.cvars.cg_errorDecay.integer != 0 {
                        let t = ctx.world.cg.time - ctx.world.cg.predictedErrorTime;
                        let mut f = (ctx.world.cvars.cg_errorDecay.value - t as f32)
                            / ctx.world.cvars.cg_errorDecay.value;
                        if f < 0.0 {
                            f = 0.0;
                        }
                        if f > 0.0 && ctx.world.cvars.cg_showVehMiss.integer != 0 {
                            CG_Printf(ctx, &format!("VEH Double prediction decay: {:.6}\n", f));
                        }
                        let pe = ctx.world.cg.predictedError;
                        _VectorScale(pe, f, &mut ctx.world.cg.predictedError);
                    } else {
                        VectorClear(&mut ctx.world.cg.predictedError);
                    }
                    let pe = ctx.world.cg.predictedError;
                    _VectorAdd(delta, pe, &mut ctx.world.cg.predictedError);
                    ctx.world.cg.predictedErrorTime = ctx.world.cg.oldTime;
                }
                //
                if ctx.world.cvars.cg_showVehMiss.integer != 0
                    && !VectorCompare(
                        oldVehicleState.vehOrientation,
                        ctx.world.cg.predictedVehicleState.vehOrientation,
                    )
                {
                    let pvs = ctx.world.cg.predictedVehicleState.vehOrientation;
                    CG_Printf(ctx, "VEH orient prediction error\n");
                    CG_Printf(
                        ctx,
                        &format!(
                            "VEH pitch prediction miss: {:.6}\n",
                            AngleSubtract(oldVehicleState.vehOrientation[0], pvs[0])
                        ),
                    );
                    CG_Printf(
                        ctx,
                        &format!(
                            "VEH yaw prediction miss: {:.6}\n",
                            AngleSubtract(oldVehicleState.vehOrientation[1], pvs[1])
                        ),
                    );
                    CG_Printf(
                        ctx,
                        &format!(
                            "VEH roll prediction miss: {:.6}\n",
                            AngleSubtract(oldVehicleState.vehOrientation[2], pvs[2])
                        ),
                    );
                }
            }
        } else if oldPlayerState.m_iVehicleNum == 0 //don't do pred err on ps while riding veh
            && ctx.world.cg.predictedPlayerState.commandTime == oldPlayerState.commandTime
        {
            if ctx.world.cg.thisFrameTeleport != qfalse {
                // a teleport will not cause an error decay
                VectorClear(&mut ctx.world.cg.predictedError);
                if ctx.world.cvars.cg_showmiss.integer != 0 {
                    CG_Printf(ctx, "PredictionTeleport\n");
                }
                ctx.world.cg.thisFrameTeleport = qfalse;
            } else {
                let mut adjusted: vec3_t = [0.0; 3];
                CG_AdjustPositionForMover(
                    ctx.world,
                    ctx.world.cg.predictedPlayerState.origin,
                    ctx.world.cg.predictedPlayerState.groundEntityNum,
                    ctx.world.cg.physicsTime,
                    ctx.world.cg.oldTime,
                    &mut adjusted,
                );

                if ctx.world.cvars.cg_showmiss.integer != 0
                    && !VectorCompare(oldPlayerState.origin, adjusted)
                {
                    CG_Printf(ctx, "prediction error\n");
                }
                let mut delta: vec3_t = [0.0; 3];
                _VectorSubtract(oldPlayerState.origin, adjusted, &mut delta);
                let len = VectorLength(delta);
                if len > 0.1 {
                    if ctx.world.cvars.cg_showmiss.integer != 0 {
                        CG_Printf(ctx, &format!("Prediction miss: {:.6}\n", len));
                    }
                    if ctx.world.cvars.cg_errorDecay.integer != 0 {
                        let t = ctx.world.cg.time - ctx.world.cg.predictedErrorTime;
                        let mut f = (ctx.world.cvars.cg_errorDecay.value - t as f32)
                            / ctx.world.cvars.cg_errorDecay.value;
                        if f < 0.0 {
                            f = 0.0;
                        }
                        if f > 0.0 && ctx.world.cvars.cg_showmiss.integer != 0 {
                            CG_Printf(ctx, &format!("Double prediction decay: {:.6}\n", f));
                        }
                        let pe = ctx.world.cg.predictedError;
                        _VectorScale(pe, f, &mut ctx.world.cg.predictedError);
                    } else {
                        VectorClear(&mut ctx.world.cg.predictedError);
                    }
                    let pe = ctx.world.cg.predictedError;
                    _VectorAdd(delta, pe, &mut ctx.world.cg.predictedError);
                    ctx.world.cg.predictedErrorTime = ctx.world.cg.oldTime;
                }
            }
        }

        if ctx.world.predict.cg_pmove.pmove_fixed != 0 {
            let msec = ctx.world.cvars.pmove_msec.integer;
            ctx.world.predict.cg_pmove.cmd.serverTime =
                ((ctx.world.predict.cg_pmove.cmd.serverTime + msec - 1) / msec) * msec;
        }

        let localAnimIndex = ctx.world.entity(pEntNum).localAnimIndex;
        // §F19: Raven indexes `bgAllAnims[pEnt->localAnimIndex]` unchecked -
        // an unparsed skeleton's -1 read garbage there. The defined answer is
        // a null table, which Pmove's own animations guards read as "no
        // anims" (same guard shape as CG_SetLerpFrameAnimation's).
        ctx.world.predict.cg_pmove.animations = if localAnimIndex >= 0
            && (localAnimIndex as usize) < ctx.world.bg_state.bgAllAnims.len()
        {
            ctx.world.bg_state.bgAllAnims[localAnimIndex as usize].anims
        } else {
            null_mut()
        };
        ctx.world.predict.cg_pmove.gametype = ctx.world.cgs.gametype;

        ctx.world.predict.cg_pmove.debugMelee = ctx.world.cgs.debugMelee;
        ctx.world.predict.cg_pmove.stepSlideFix = ctx.world.cgs.stepSlideFix;
        ctx.world.predict.cg_pmove.noSpecMove = ctx.world.cgs.noSpecMove;

        ctx.world.predict.cg_pmove.nonHumanoid = if localAnimIndex > 0 { qtrue } else { qfalse };

        let saberLock = ctx
            .world
            .cg
            .snap_ref()
            .map(|s| (s.ps.saberLockTime, s.ps.saberLockEnemy, s.ps.origin));
        if let Some((saberLockTime, saberLockEnemy, snapOrigin)) = saberLock {
            if saberLockTime > ctx.world.cg.time {
                // Raven's `if (blockOpp)` tests the address of
                // `&cg_entities[...]` - always true - so the block is
                // unconditional
                let blockOppLerpOrigin = ctx.world.entity(saberLockEnemy as usize).lerpOrigin;

                let mut lockDir: vec3_t = [0.0; 3];
                let mut lockAng: vec3_t = [0.0; 3];
                _VectorSubtract(blockOppLerpOrigin, snapOrigin, &mut lockDir);
                vectoangles(lockDir, &mut lockAng);

                ctx.world.cg.predictedPlayerState.viewangles = lockAng;
            }
        }

        //THIS is pretty much bad, but...
        ctx.world.cg.predictedPlayerState.fd.saberAnimLevelBase =
            ctx.world.cg.predictedPlayerState.fd.saberAnimLevel;
        if ctx.world.cg.predictedPlayerState.saberHolstered == 1 {
            let ci = &ctx.world.cgs.clientinfo[pEntNum];
            if ci.saber[0].numBlades > 0 {
                ctx.world.cg.predictedPlayerState.fd.saberAnimLevelBase = SS_STAFF as c_int;
            } else if ci.saber[1].model[0] != 0 {
                ctx.world.cg.predictedPlayerState.fd.saberAnimLevelBase = SS_DUAL as c_int;
            }
        }

        {
            let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
            let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
            let pm_ptr = &raw mut ctx.world.predict.cg_pmove;
            Pmove(pm_ptr, &mut ctx.world.bg_state, &traps, &mut callbacks);
        }

        if CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum)
            && ctx.world.cg.predictedPlayerState.pm_type != PM_INTERMISSION as c_int
        {
            //we're riding a vehicle, let's predict it
            let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;
            if let Some(id) = ctx.world.entity(vehNum).m_pVehicle {
                let row_idx = id.ent_num() as usize;

                //make sure pointer is set up to go to our predicted state
                ctx.world.vehicle_pool[row_idx].m_vOrientation =
                    &raw mut ctx.world.cg.predictedVehicleState.vehOrientation[0];

                //keep this updated based on what the playerstate says
                ctx.world.vehicle_pool[row_idx].m_iRemovedSurfaces =
                    ctx.world.cg.predictedVehicleState.vehSurfaces;

                trap::GetUserCmd(
                    ctx.engine,
                    cmdNum,
                    &mut ctx.world.vehicle_pool[row_idx].m_ucmd,
                );

                if ctx.world.vehicle_pool[row_idx].m_ucmd.buttons & BUTTON_TALK != 0 {
                    //forced input if "chat bubble" is up
                    let ucmd = &mut ctx.world.vehicle_pool[row_idx].m_ucmd;
                    ucmd.buttons = BUTTON_TALK;
                    ucmd.forwardmove = 0;
                    ucmd.rightmove = 0;
                    ucmd.upmove = 0;
                }
                ctx.world.predict.cg_vehPmove.ps = &raw mut ctx.world.cg.predictedVehicleState;
                let vehLocalAnimIndex = ctx.world.entity(vehNum).localAnimIndex;
                // §F19: same unchecked `bgAllAnims[localAnimIndex]` as
                // cg_pmove above - a -1 reads as the null table
                ctx.world.predict.cg_vehPmove.animations = if vehLocalAnimIndex >= 0
                    && (vehLocalAnimIndex as usize) < ctx.world.bg_state.bgAllAnims.len()
                {
                    ctx.world.bg_state.bgAllAnims[vehLocalAnimIndex as usize].anims
                } else {
                    null_mut()
                };

                ctx.world.predict.cg_vehPmove.cmd = ctx.world.vehicle_pool[row_idx].m_ucmd;

                ctx.world.predict.cg_vehPmove.gametype = ctx.world.cgs.gametype;
                ctx.world.predict.cg_vehPmove.ghoul2 = ctx.world.entity(vehNum).ghoul2;

                ctx.world.predict.cg_vehPmove.nonHumanoid =
                    if vehLocalAnimIndex > 0 { qtrue } else { qfalse };

                //I think this was actually wrong.. just copy-pasted from id code. Oh well.
                let solid = ctx.world.entity(vehNum).currentState.solid;
                let x = solid & 255;
                let mut zd = (solid >> 8) & 255;
                let mut zu = (solid >> 15) & 255;

                zu -= 32; //I don't quite get the reason for this.
                zd = -zd;

                //z/y must be symmetrical (blah)
                ctx.world.predict.cg_vehPmove.mins[0] = -(x as f32);
                ctx.world.predict.cg_vehPmove.mins[1] = -(x as f32);
                ctx.world.predict.cg_vehPmove.maxs[0] = x as f32;
                ctx.world.predict.cg_vehPmove.maxs[1] = x as f32;
                ctx.world.predict.cg_vehPmove.mins[2] = zd as f32;
                ctx.world.predict.cg_vehPmove.maxs[2] = zu as f32;

                ctx.world.predict.cg_vehPmove.modelScale = ctx.world.entity(vehNum).modelScale;

                if ctx.world.predict.cg_vehPmoveSet == qfalse {
                    //do all the one-time things
                    // Raven binds cg_vehPmove.trace/pointcontents here; those
                    // live on CgBgTraps (DEC-47.2), pmove_t's fn-ptr fields
                    // stay for layout only
                    ctx.world.predict.cg_vehPmove.tracemask = MASK_PLAYERSOLID;
                    ctx.world.predict.cg_vehPmove.debugLevel = 0;
                    ctx.world.predict.cg_vehPmove.g2Bolts_LFoot = -1;
                    ctx.world.predict.cg_vehPmove.g2Bolts_RFoot = -1;

                    // the bg_ents shadow walk, sizeof(bgEntity_t) stride -
                    // same DEC-47.2 divergence as cg_pmove (Raven casts
                    // cg_entities and strides sizeof(centity_t))
                    ctx.world.predict.cg_vehPmove.baseEnt = ctx.world.bg_ents.as_mut_ptr();
                    ctx.world.predict.cg_vehPmove.entSize = size_of::<bgEntity_t>() as c_int;

                    ctx.world.predict.cg_vehPmoveSet = qtrue;
                }

                ctx.world.predict.cg_vehPmove.noFootsteps =
                    if ctx.world.cgs.dmflags & DF_NO_FOOTSTEPS > 0 {
                        qtrue
                    } else {
                        qfalse
                    };
                ctx.world.predict.cg_vehPmove.pmove_fixed = ctx.world.cvars.pmove_fixed.integer;
                ctx.world.predict.cg_vehPmove.pmove_msec = ctx.world.cvars.pmove_msec.integer;

                {
                    let clientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
                    let world = &mut *ctx.world;
                    // both rows repoint at the predicted states - enum arm for
                    // the cgame readers, raw pointer for the bg view (DEC-47.2)
                    world.entities[clientNum].playerState = PlayerStateRef::Predicted;
                    world.bg_ents[clientNum].playerState = &raw mut world.cg.predictedPlayerState;
                    world.entities[vehNum].playerState = PlayerStateRef::PredictedVehicle;
                    world.bg_ents[vehNum].playerState = &raw mut world.cg.predictedVehicleState;
                }

                //update boarding value sent from server. boarding is not predicted, but no big deal
                ctx.world.vehicle_pool[row_idx].m_iBoarding =
                    ctx.world.cg.predictedVehicleState.vehBoarding;

                {
                    let traps = CgBgTraps::new(ctx.engine, ctx.world_raw());
                    let mut callbacks = CgGameCallbacks::new(ctx.engine, ctx.world_raw());
                    let pm_ptr = &raw mut ctx.world.predict.cg_vehPmove;
                    Pmove(pm_ptr, &mut ctx.world.bg_state, &traps, &mut callbacks);
                }

                if ctx.world.cvars.cg_showVehBounds.integer != 0 {
                    let NPCDEBUG_RED: vec3_t = [1.0, 0.0, 0.0];
                    // Raven reads `cg_vehPmove.ps->origin` - that ps was aimed
                    // at predictedVehicleState just above
                    let origin = ctx.world.cg.predictedVehicleState.origin;
                    let mut absmin: vec3_t = [0.0; 3];
                    let mut absmax: vec3_t = [0.0; 3];
                    _VectorAdd(origin, ctx.world.predict.cg_vehPmove.mins, &mut absmin);
                    _VectorAdd(origin, ctx.world.predict.cg_vehPmove.maxs, &mut absmax);
                    CG_Cube(ctx, absmin, absmax, NPCDEBUG_RED, 0.25);
                }
            }
        }

        moved = qtrue;

        // add push trigger movement effects
        CG_TouchTriggerPrediction(ctx);

        // check for predictable events that changed from previous predictions
        //CG_CheckChangedPredictableEvents(&cg.predictedPlayerState);
    }

    if ctx.world.cvars.cg_showmiss.integer > 1 {
        CG_Printf(
            ctx,
            &format!(
                "[{} : {}] ",
                ctx.world.predict.cg_pmove.cmd.serverTime, ctx.world.cg.time
            ),
        );
    }

    // Raven's `if (!moved) goto revertES;` - the moved tail runs here, the
    // revertES section below runs either way
    if moved == qfalse {
        if ctx.world.cvars.cg_showmiss.integer != 0 {
            CG_Printf(ctx, "not moved\n");
        }
    } else {
        if CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum) {
            let mut adjusted: vec3_t = [0.0; 3];
            CG_AdjustPositionForMover(
                ctx.world,
                ctx.world.cg.predictedVehicleState.origin,
                ctx.world.cg.predictedVehicleState.groundEntityNum,
                ctx.world.cg.physicsTime,
                ctx.world.cg.time,
                &mut adjusted,
            );
            ctx.world.cg.predictedVehicleState.origin = adjusted;
        } else {
            // adjust for the movement of the groundentity
            let mut adjusted: vec3_t = [0.0; 3];
            CG_AdjustPositionForMover(
                ctx.world,
                ctx.world.cg.predictedPlayerState.origin,
                ctx.world.cg.predictedPlayerState.groundEntityNum,
                ctx.world.cg.physicsTime,
                ctx.world.cg.time,
                &mut adjusted,
            );
            ctx.world.cg.predictedPlayerState.origin = adjusted;
        }

        if ctx.world.cvars.cg_showmiss.integer != 0
            && ctx.world.cg.predictedPlayerState.eventSequence
                > oldPlayerState.eventSequence + MAX_PS_EVENTS as c_int
        {
            CG_Printf(ctx, "WARNING: dropped event\n");
        }

        // fire events and other transition triggered things
        let pps = ctx.world.cg.predictedPlayerState;
        CG_TransitionPlayerState(
            ctx,
            ds,
            &pps,
            &mut oldPlayerState,
            PlayerStateRef::Predicted,
        );

        if ctx.world.cvars.cg_showmiss.integer != 0
            && ctx.world.cg.eventSequence > ctx.world.cg.predictedPlayerState.eventSequence
        {
            CG_Printf(ctx, "WARNING: double event\n");
            ctx.world.cg.eventSequence = ctx.world.cg.predictedPlayerState.eventSequence;
        }

        if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
            && !CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum)
        {
            //a passenger on this vehicle, bolt them in
            let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;
            let lerpOrigin = ctx.world.entity(vehNum).lerpOrigin;
            ctx.world.cg.predictedPlayerState.origin = lerpOrigin;
        }
    }

    // revertES:
    if CG_Piloting(ctx.world, ctx.world.cg.predictedPlayerState.m_iVehicleNum) {
        let vehNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum as usize;

        if let Some(id) = ctx.world.entity(vehNum).m_pVehicle {
            //switch ptr back for this ent in case we stop riding it
            let sn = ctx.world.entity(vehNum).currentState.number as usize;
            ctx.world.vehicle_pool[id.ent_num() as usize].m_vOrientation =
                &raw mut ctx.world.cgSendPSPool[sn].vehOrientation[0];
        }

        let clientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
        // Raven keys the vehicle's pool row by `veh->currentState.number`
        // (== vehNum whenever the snapshot is coherent); the raw bg row keeps
        // the literal semantic, the Snap arm resolves by entity number. The
        // index is wire-bounded (GENTITYNUM_BITS decode < MAX_GENTITIES)
        let vehStateNum = ctx.world.entity(vehNum).currentState.number as usize;
        let world = &mut *ctx.world;
        world.entities[clientNum].playerState = PlayerStateRef::Snap;
        world.bg_ents[clientNum].playerState = &raw mut world.cgSendPSPool[clientNum];
        world.entities[vehNum].playerState = PlayerStateRef::Snap;
        world.bg_ents[vehNum].playerState = &raw mut world.cgSendPSPool[vehStateNum];
    }

    //copy some stuff back into the entstates to help actually "predict" them if applicable
    {
        let world = &mut *ctx.world;
        for i in 0..MAX_GENTITIES {
            let es = &mut world.entities[i].currentState;
            if es.eType == ET_PLAYER as c_int || es.eType == ET_NPC as c_int {
                let ps = &world.cgSendPSPool[i];
                es.torsoAnim = ps.torsoAnim;
                es.legsAnim = ps.legsAnim;
                es.forceFrame = ps.saberLockFrame;
                es.saberMove = ps.saberMove;
            }
        }
    }
}

/// Raven `#define CMD_BACKUP 64`.
///
/// Source: `oracle/codemp/game/q_shared.h:2914`
const CMD_BACKUP: c_int = 64;
