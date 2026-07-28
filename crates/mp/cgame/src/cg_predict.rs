//! Port of `oracle/codemp/cgame/cg_predict.c` — client-side movement prediction and its trace helpers. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_abi::cgame::public::snapshot_t::MAX_ENTITIES_IN_SNAPSHOT;
use mp_bg::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanItemBeGrabbed, BG_PlayerTouchesItem,
};
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::entity_event::entity_event_t::EV_ITEM_PICKUP;
use mp_bg::public::entity_flags::{EF_ITEMPLACEHOLDER, EF_NODRAW};
use mp_bg::public::entity_type::entityType_t::{
    ET_ITEM, ET_NPC, ET_PLAYER, ET_PUSH_TRIGGER, ET_TELEPORT_TRIGGER, ET_TERRAIN,
};
use mp_bg::public::gametype::{GT_CTF, GT_CTY};
use mp_bg::public::item_type::{IT_POWERUP, IT_WEAPON};
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT, PW_REDFLAG,
};
use mp_bg::public::stat_index::statIndex_t::STAT_WEAPONS;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED};
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_bg::weapons::weapon_t::{WP_EMPLACED_GUN, WP_NONE};
use mp_qshared::common::mp::game::class_t::class_t::CLASS_VEHICLE;
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::force_powers::{FORCE_DARKSIDE, FORCE_LIGHTSIDE};
use mp_qshared::shared::q_math::{_VectorSubtract, LerpAngle};
use mp_qshared::shared::surface_flags::SOLID_BMODEL;
use mp_qshared::shared::{qfalse, qtrue, vec3_t, ENTITYNUM_WORLD, MAX_GENTITIES};

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
