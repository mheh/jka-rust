// PORT-COMPLETE: g_items.c
//! FAITHFUL port of `oracle/codemp/game/g_items.c`.
//!
//! Filled by the jampgame mega-pass.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use core::ffi::CStr;

use crate::client::gclient::gclient_t;
use mp_bg::bg_misc::{
    BG_AddPredictableEventToPlayerstate, BG_CanItemBeGrabbed, BG_CycleInven, BG_EmplacedView,
    BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta, BG_FindItem, BG_FindItemForHoldable,
    BG_FindItemForWeapon,
};

// Raven `#define ITMSF_*` item spawnflags.
// Source: `oracle/codemp/game/g_items.c:30-35`
pub const ITMSF_SUSPEND: c_int = 1;
pub const ITMSF_NOPLAYER: c_int = 2;
pub const ITMSF_ALLOWNPC: c_int = 4;
pub const ITMSF_NOTSOLID: c_int = 8;
pub const ITMSF_VERTICAL: c_int = 16;
pub const ITMSF_INVISIBLE: c_int = 32;
use crate::client::{CON_CONNECTED, CON_DISCONNECTED};
use crate::entity::flags::{FL_DROPPED_ITEM, FL_NOTARGET, FL_TEAMSLAVE};
use crate::g_combat::G_RadiusDamage;
use crate::g_exphysics::G_RunExPhys;
use crate::g_log::{G_LogWeaponItem, G_LogWeaponPickup, G_LogWeaponPowerup};
use crate::g_main::{G_Error, G_LogPrintf, G_Printf, G_RunThink};
use crate::g_missile::CreateMissile;
use crate::g_object::G_RunObject;
use crate::g_spawn::G_SpawnFloat;
use crate::g_team::{OnSameTeam, Pickup_Team, Team_FreeEntity};
use crate::g_utils::{
    G_AddEvent, G_AddPredictableEvent, G_BoneIndex, G_EffectIndex, G_FreeEntity, G_ModelIndex,
    G_PlayEffect, G_PlayEffectID, G_RadiusList, G_ScaleNetHealth, G_SetAnim, G_SetOrigin, G_Sound,
    G_SoundIndex, G_Spawn, G_TempEntity, G_UseTargets,
};
use crate::g_weapon::WP_FireTurretMissile;
use crate::q_math::{
    AngleNormalize360, AngleSubtract, VectorLength, VectorLengthSquared, VectorNormalize,
};
use crate::teams::npcteam::{NPCTEAM_ENEMY, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER};
use crate::trap;
use crate::w_saber::HasSetSaberOnly;
use crate::NPC_AI_Jedi::{Jedi_Cloak, Jedi_Decloak};
use crate::NPC_combat::G_SetEnemy;
use crate::NPC_spawn::NPC_SpawnType;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::anim_number::animNumber_t::*;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::gametype::{GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL};
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::weaponstate::weaponstate_t::*;
use mp_bg::vehicles::vehicle_s::Vehicle_t;
use mp_bg::weapons::weapon_t::WP_TURRET;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::shared::mdxaBone_t;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

// Raven angle-vector indices (`q_shared.h`): PITCH=0, YAW=1, ROLL=2. Canonical
// in `crate::q_math`. Source: `oracle/codemp/game/q_shared.h:374-376`
use crate::q_math::{PITCH, ROLL, YAW};

// `ITMSF_ALLOWNPC` — item spawnflag defined above with the other `ITMSF_*`
// spawnflags (g_items.c:32); duplicate file-scope const removed at integration.

// Raven `g_items.c:42-44` medpack heal caps.
pub const MAX_MEDPACK_HEAL_AMOUNT: c_int = 25;
pub const MAX_MEDPACK_BIG_HEAL_AMOUNT: c_int = 50;

// Raven `g_items.c:20-26` respawn-time-by-item-class `#define`s, consumed by
// `adjustRespawnTime(float preRespawnTime, ...)` — hence `f32`, not `c_int`.
// Source: `oracle/codemp/game/g_items.c:20-26`
pub const RESPAWN_ARMOR: f32 = 20.0;
pub const RESPAWN_TEAM_WEAPON: f32 = 30.0;
pub const RESPAWN_HEALTH: f32 = 30.0;
pub const RESPAWN_AMMO: f32 = 40.0;
pub const RESPAWN_HOLDABLE: f32 = 60.0;
pub const RESPAWN_MEGAHEALTH: f32 = 120.0;
pub const RESPAWN_POWERUP: f32 = 120.0;

// Raven `g_items.c:1274-1275` tossed-item timing `#define`s.
// Source: `oracle/codemp/game/g_items.c:1274-1275`
pub const TOSSED_ITEM_STAY_PERIOD: c_int = 20000;
pub const TOSSED_ITEM_OWNER_NOTOUCH_DUR: c_int = 1000;

// Raven `g_items.c:1333-1334` dispenser item classnames.
// (referenced from `G_PrecacheDispensers`)

// Raven `surfaceflags.h`/`bg_public.h` CONTENTS_*/MASK_* #defines, canonical in
// `mp_qshared::shared::surface_flags`. Imported explicitly (they also reach here
// via the prelude glob); the former local transcriptions were redundant.
// Source: `oracle/codemp/game/surfaceflags.h:10-36`, `bg_public.h:1172-1177`
use mp_qshared::shared::surface_flags::{
    CONTENTS_LIGHTSABER, CONTENTS_NODROP, CONTENTS_PLAYERCLIP, CONTENTS_SHOTCLIP, CONTENTS_TRIGGER,
    CONTENTS_WATER, MASK_PLAYERSOLID, MASK_SHOT,
};

// Raven `g_public.h` svflags #defines, canonical in `g_public_consts`.
// SVF_SINGLECLIENT was wrongly transcribed locally as 0x40 (that is SVF_PORTAL);
// the correct value is 0x100, so temp-entity item pickups were tagged with the
// wrong svFlag bit. Import fixes it.
// Source: `oracle/codemp/game/g_public.h:22,25`
use crate::g_public_consts::{SVF_BROADCAST, SVF_NOCLIENT, SVF_SINGLECLIENT};

// Raven `bg_public.h:82` `CS_ITEMS` — canonical in `mp_bg::public::configstring`,
// reaches this file via the crate prelude glob (`crate::prelude::*`).

// Raven `bg_public.h` `EF_ITEMPLACEHOLDER`/`EF_CLIENTSMOOTH`/`EF_G2ANIMATING`,
// canonical in `mp_bg::public::entity_flags`.
// Source: `oracle/codemp/game/bg_public.h:560,601,607`
use mp_bg::bg_misc::snap_vector;
use mp_bg::public::entity_flags::{EF_CLIENTSMOOTH, EF_G2ANIMATING, EF_ITEMPLACEHOLDER};

// Raven `ITEM_RADIUS` (`bg_public.h:35`).
pub const ITEM_RADIUS: f32 = 15.0;

// Raven `FRAMETIME` (`g_local.h:37`). `pub` + prelude re-export (pass-3
// symbol backfill) — sibling files (`NPC_AI_GalakMech.rs`, `g_mover.rs`, …)
// carry their own private copy of this same value; this is the one exported
// for bare-use sites without a local copy.
pub const FRAMETIME: c_int = 100;

// Raven `#define REWARD_SPRITE_TIME 2000` (`g_local.h:39`).
pub const REWARD_SPRITE_TIME: c_int = 2000;

// Raven `#define PLAYEREVENT_GAUNTLETREWARD 0x0002` (`bg_public.h:717`).
pub const PLAYEREVENT_GAUNTLETREWARD: c_int = 0x0002;

// Raven `#define TURRET_DEATH_DELAY 2000` / `TURRET_LIFETIME 60000` (`g_items.c:697-698`).
pub const TURRET_DEATH_DELAY: c_int = 2000;
pub const TURRET_LIFETIME: c_int = 60000;

/// Raven `adjustRespawnTime`.
///
/// Source: `oracle/codemp/game/g_items.c:47-88`
pub fn adjustRespawnTime(ctx: &mut GameContext, preRespawnTime: f32, kind: ItemKind) -> c_int {
    // Raven `#define RESPAWN_AMMO 40` (`g_items.c:26`).
    pub const RESPAWN_AMMO: f32 = 40.0;

    let mut respawnTime = preRespawnTime;

    if matches!(
        kind,
        ItemKind::Weapon(WP_THERMAL | WP_TRIP_MINE | WP_DET_PACK)
    ) {
        // special case for these, use ammo respawn rate
        respawnTime = RESPAWN_AMMO;
    }

    if ctx.world.cvars.g_adaptRespawn.integer == 0 {
        return respawnTime as c_int;
    }

    let numPlayingClients = ctx.world.level.numPlayingClients;
    if numPlayingClients > 4 {
        // Start scaling the respawn times.
        if numPlayingClients > 32 {
            // 1/4 time minimum.
            respawnTime *= 0.25;
        } else if numPlayingClients > 12 {
            // From 12-32, scale from 0.5 to 0.25;
            // C: `respawnTime *= 20.0 / (float)(n+8)` — 20.0 is a double, so the
            // divide and the `*=` both run in f64, narrowing once at the store;
            // the int cast to (float) happens first.
            // Source: `oracle/codemp/game/g_items.c:74`
            respawnTime =
                (respawnTime as f64 * (20.0f64 / (numPlayingClients + 8) as f32 as f64)) as f32;
        } else {
            // From 4-12, scale from 1.0 to 0.5;
            // C: `respawnTime *= 8.0 / (float)(n+4)` — same f64 divide/`*=` shape.
            // Source: `oracle/codemp/game/g_items.c:78`
            respawnTime =
                (respawnTime as f64 * (8.0f64 / (numPlayingClients + 4) as f32 as f64)) as f32;
        }
    }

    if respawnTime < 1.0 {
        // No matter what, don't go lower than 1 second, or the pickups become very noisy!
        respawnTime = 1.0;
    }

    respawnTime as c_int
}

/// Raven `ShieldRemove`.
///
/// Source: `oracle/codemp/game/g_items.c:108-119`
pub fn ShieldRemove(ctx: &mut GameContext, self_: EntityId) {
    ctx.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + 100;

    // Play kill sound...
    let shieldDeactivateSound = ctx.world.globals.shieldDeactivateSound;
    G_AddEvent(
        ctx.entity_mut(self_),
        entity_event_t::EV_GENERAL_SOUND as c_int,
        shieldDeactivateSound,
    );
    ctx.entity_mut(self_).s.loopSound = 0;
    ctx.entity_mut(self_).s.loopIsSoundset = qfalse;
}

/// Raven `ShieldThink`.
///
/// Source: `oracle/codemp/game/g_items.c:123-141`
pub fn ShieldThink(ctx: &mut GameContext, self_: EntityId) {
    // Raven `#define SHIELD_HEALTH_DEC 10` / `SHIELD_SIEGE_HEALTH_DEC (2000/25)` (`g_items.c:92,99`).
    pub const SHIELD_SIEGE_HEALTH_DEC: c_int = 2000 / 25;
    pub const SHIELD_HEALTH_DEC: c_int = 10;

    ctx.entity_mut(self_).s.trickedentindex = 0;

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        ctx.entity_mut(self_).health -= SHIELD_SIEGE_HEALTH_DEC;
    } else {
        ctx.entity_mut(self_).health -= SHIELD_HEALTH_DEC;
    }
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + 1000;
    if ctx.entity(self_).health <= 0 {
        ShieldRemove(ctx, self_);
    }
}

/// Raven `ShieldDie`.
///
/// Source: `oracle/codemp/game/g_items.c:145-151`
pub fn ShieldDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    // Play damaging sound...
    let shieldDamageSound = ctx.world.globals.shieldDamageSound;
    G_AddEvent(
        ctx.entity_mut(self_),
        entity_event_t::EV_GENERAL_SOUND as c_int,
        shieldDamageSound,
    );

    ShieldRemove(ctx, self_);
}

/// Raven `ShieldPain`.
///
/// Source: `oracle/codemp/game/g_items.c:155-167`
pub fn ShieldPain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // Set the itemplaceholder flag to indicate the the shield drawing that the shield pain should be drawn.
    ctx.entity_mut(self_).think = Some(EntThink::ShieldThink).into();
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + 400;

    // Play damaging sound...
    let shieldDamageSound = ctx.world.globals.shieldDamageSound;
    G_AddEvent(
        ctx.entity_mut(self_),
        entity_event_t::EV_GENERAL_SOUND as c_int,
        shieldDamageSound,
    );

    ctx.entity_mut(self_).s.trickedentindex = 1;
}

/// Raven `ShieldGoSolid`.
///
/// Source: `oracle/codemp/game/g_items.c:171-207`
pub fn ShieldGoSolid(ctx: &mut GameContext, self_: EntityId) {
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    // see if we're valid
    ctx.entity_mut(self_).health -= 1;
    if ctx.entity(self_).health <= 0 {
        ShieldRemove(ctx, self_);
        return;
    }

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ctx.entity(self_).r.currentOrigin as *const vec3_t,
            &ctx.entity(self_).r.mins as *const vec3_t,
            &ctx.entity(self_).r.maxs as *const vec3_t,
            &ctx.entity(self_).r.currentOrigin as *const vec3_t,
            ctx.entity(self_).s.number,
            CONTENTS_BODY,
        ),
    );
    if tr.startsolid != 0 {
        // gah, we can't activate yet
        ctx.entity_mut(self_).nextthink = ctx.world.level.time + 200;
        ctx.entity_mut(self_).think = Some(EntThink::ShieldGoSolid).into();
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(self_)).cast()),
        );
    } else {
        // get hard... huh-huh...
        ctx.entity_mut(self_).s.eFlags &= !EF_NODRAW;

        ctx.entity_mut(self_).r.contents = CONTENTS_SOLID;
        ctx.entity_mut(self_).nextthink = ctx.world.level.time + 1000;
        ctx.entity_mut(self_).think = Some(EntThink::ShieldThink).into();
        ctx.entity_mut(self_).takedamage = qtrue;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(self_)).cast()),
        );

        // Play raising sound...
        let shieldActivateSound = ctx.world.globals.shieldActivateSound;
        G_AddEvent(
            ctx.entity_mut(self_),
            entity_event_t::EV_GENERAL_SOUND as c_int,
            shieldActivateSound,
        );
        ctx.entity_mut(self_).s.loopSound = ctx.world.globals.shieldLoopSound;
        ctx.entity_mut(self_).s.loopIsSoundset = qfalse;
    }
}

/// Raven `ShieldGoNotSolid`.
///
/// Source: `oracle/codemp/game/g_items.c:211-226`
pub fn ShieldGoNotSolid(ctx: &mut GameContext, self_: EntityId) {
    // make the shield non-solid very briefly
    ctx.entity_mut(self_).r.contents = 0;
    ctx.entity_mut(self_).s.eFlags |= EF_NODRAW;
    // nextthink needs to have a large enough interval to avoid excess accumulation of Activate messages
    ctx.entity_mut(self_).nextthink = ctx.world.level.time + 200;
    ctx.entity_mut(self_).think = Some(EntThink::ShieldGoSolid).into();
    ctx.entity_mut(self_).takedamage = qfalse;
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(self_)).cast()),
    );

    // Play kill sound...
    let shieldDeactivateSound = ctx.world.globals.shieldDeactivateSound;
    G_AddEvent(
        ctx.entity_mut(self_),
        entity_event_t::EV_GENERAL_SOUND as c_int,
        shieldDeactivateSound,
    );
    ctx.entity_mut(self_).s.loopSound = 0;
    ctx.entity_mut(self_).s.loopIsSoundset = qfalse;
}

/// Raven `ShieldTouch`.
///
/// Source: `oracle/codemp/game/g_items.c:230-250`
pub fn ShieldTouch(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let parent = ctx.entity(self_).parent;

    if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
        // let teammates through
        // compare the parent's team to the "other's" team
        // Raven: parent && parent->client && other->client (short-circuit order
        // preserved; the `other` deref runs only after the parent checks pass).
        if parent.is_some_and(|p| !ctx.entity(p).client.is_null())
            && other.is_some_and(|o| !ctx.entity(o).client.is_null())
        {
            if OnSameTeam(ctx, parent, other) != 0 {
                ShieldGoNotSolid(ctx, self_);
            }
        }
    } else {
        // let the person who dropped the shield through
        if let (Some(p), Some(o)) = (parent, other) {
            if ctx.entity(p).s.number == ctx.entity(o).s.number {
                ShieldGoNotSolid(ctx, self_);
            }
        }
    }
}

/// Raven `CreateShield`.
///
/// Source: `oracle/codemp/game/g_items.c:254-380`
pub fn CreateShield(ctx: &mut GameContext, ent: EntityId) {
    // Raven `g_items.c:91-99` shield #defines.
    pub const SHIELD_HEALTH: f32 = 250.0;
    pub const SHIELD_SIEGE_HEALTH: f32 = 2000.0;
    pub const MAX_SHIELD_HEIGHT: f32 = 254.0;
    pub const MAX_SHIELD_HALFWIDTH: f32 = 255.0;
    pub const SHIELD_HALFTHICKNESS: f32 = 4.0;

    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    // trace upward to find height of shield
    let mut end = ctx.entity(ent).r.currentOrigin;
    end[2] += MAX_SHIELD_HEIGHT;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ctx.entity(ent).r.currentOrigin as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &end as *const vec3_t,
            ctx.entity(ent).s.number,
            MASK_SHOT,
        ),
    );
    let height = (MAX_SHIELD_HEIGHT * tr.fraction) as c_int;

    // use angles to find the proper axis along which to align the shield
    let mut mins: vec3_t = [-SHIELD_HALFTHICKNESS, -SHIELD_HALFTHICKNESS, 0.0];
    let mut maxs: vec3_t = [SHIELD_HALFTHICKNESS, SHIELD_HALFTHICKNESS, height as f32];
    let mut posTraceEnd = ctx.entity(ent).r.currentOrigin;
    let mut negTraceEnd = ctx.entity(ent).r.currentOrigin;

    let xaxis;
    if ctx.entity(ent).s.angles[YAW] as c_int == 0 {
        // shield runs along y-axis
        posTraceEnd[1] += MAX_SHIELD_HALFWIDTH;
        negTraceEnd[1] -= MAX_SHIELD_HALFWIDTH;
        xaxis = qfalse;
    } else {
        // shield runs along x-axis
        posTraceEnd[0] += MAX_SHIELD_HALFWIDTH;
        negTraceEnd[0] -= MAX_SHIELD_HALFWIDTH;
        xaxis = qtrue;
    }

    // trace horizontally to find extend of shield
    // positive trace
    let mut start = ctx.entity(ent).r.currentOrigin;
    start[2] += (height >> 1) as f32;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &start as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &posTraceEnd as *const vec3_t,
            ctx.entity(ent).s.number,
            MASK_SHOT,
        ),
    );
    let posWidth = (MAX_SHIELD_HALFWIDTH * tr.fraction) as c_int;
    // negative trace
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &start as *const vec3_t,
            core::ptr::null(),
            core::ptr::null(),
            &negTraceEnd as *const vec3_t,
            ctx.entity(ent).s.number,
            MASK_SHOT,
        ),
    );
    let negWidth = (MAX_SHIELD_HALFWIDTH * tr.fraction) as c_int;

    // kef -- monkey with dimensions and place origin in center
    let halfWidth = (posWidth + negWidth) >> 1;
    if xaxis != 0 {
        ctx.entity_mut(ent).r.currentOrigin[0] =
            ctx.entity(ent).r.currentOrigin[0] - negWidth as f32 + halfWidth as f32;
    } else {
        ctx.entity_mut(ent).r.currentOrigin[1] =
            ctx.entity(ent).r.currentOrigin[1] - negWidth as f32 + halfWidth as f32;
    }
    ctx.entity_mut(ent).r.currentOrigin[2] += (height >> 1) as f32;

    // set entity's mins and maxs to new values, make it solid, and link it
    if xaxis != 0 {
        ctx.entity_mut(ent).r.mins = [
            -(halfWidth as f32),
            -SHIELD_HALFTHICKNESS,
            -((height >> 1) as f32),
        ];
        ctx.entity_mut(ent).r.maxs = [halfWidth as f32, SHIELD_HALFTHICKNESS, (height >> 1) as f32];
    } else {
        ctx.entity_mut(ent).r.mins = [
            -SHIELD_HALFTHICKNESS,
            -(halfWidth as f32),
            -((height >> 1) as f32),
        ];
        ctx.entity_mut(ent).r.maxs = [SHIELD_HALFTHICKNESS, halfWidth as f32, height as f32];
    }
    ctx.entity_mut(ent).clipmask = MASK_SHOT;

    // Information for shield rendering.
    //	xaxis - 1 bit
    //	height - 0-254 8 bits
    //	posWidth - 0-255 8 bits
    //  negWidth - 0 - 255 8 bits
    let paramData = (xaxis << 24) | (height << 16) | (posWidth << 8) | negWidth;
    ctx.entity_mut(ent).s.time2 = paramData;

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        ctx.entity_mut(ent).health = SHIELD_SIEGE_HEALTH.ceil() as c_int;
    } else {
        ctx.entity_mut(ent).health = SHIELD_HEALTH.ceil() as c_int;
    }

    ctx.entity_mut(ent).s.time = ctx.entity(ent).health; // ???
    ctx.entity_mut(ent).pain = Some(EntPain::ShieldPain).into();
    ctx.entity_mut(ent).die = Some(EntDie::ShieldDie).into();
    ctx.entity_mut(ent).touch = Some(EntTouch::ShieldTouch).into();

    // see if we're valid
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ctx.entity(ent).r.currentOrigin as *const vec3_t,
            &ctx.entity(ent).r.mins as *const vec3_t,
            &ctx.entity(ent).r.maxs as *const vec3_t,
            &ctx.entity(ent).r.currentOrigin as *const vec3_t,
            ctx.entity(ent).s.number,
            CONTENTS_BODY,
        ),
    );

    if tr.startsolid != 0 {
        // Something in the way!
        // make the shield non-solid very briefly
        ctx.entity_mut(ent).r.contents = 0;
        ctx.entity_mut(ent).s.eFlags |= EF_NODRAW;
        // nextthink needs to have a large enough interval to avoid excess accumulation of Activate messages
        ctx.entity_mut(ent).nextthink = ctx.world.level.time + 200;
        ctx.entity_mut(ent).think = Some(EntThink::ShieldGoSolid).into();
        ctx.entity_mut(ent).takedamage = qfalse;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
        );
    } else {
        // Get solid.
        ctx.entity_mut(ent).r.contents = CONTENTS_PLAYERCLIP | CONTENTS_SHOTCLIP; //CONTENTS_SOLID;

        ctx.entity_mut(ent).nextthink = ctx.world.level.time;
        ctx.entity_mut(ent).think = Some(EntThink::ShieldThink).into();

        ctx.entity_mut(ent).takedamage = qtrue;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
        );

        // Play raising sound...
        let shieldActivateSound = ctx.world.globals.shieldActivateSound;
        G_AddEvent(
            ctx.entity_mut(ent),
            entity_event_t::EV_GENERAL_SOUND as c_int,
            shieldActivateSound,
        );
        ctx.entity_mut(ent).s.loopSound = ctx.world.globals.shieldLoopSound;
        ctx.entity_mut(ent).s.loopIsSoundset = qfalse;
    }

    ShieldGoSolid(ctx, ent);
}

/// Raven `PlaceShield`.
///
/// Source: `oracle/codemp/game/g_items.c:382-470`
pub fn PlaceShield(ctx: &mut GameContext, playerent: EntityId) -> qboolean {
    pub const SHIELD_PLACEDIST: f32 = 64.0;

    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut fwd: vec3_t = [0.0; 3];
    let mut pos: vec3_t;
    let mut dest: vec3_t;
    let mins: vec3_t = [-4.0, -4.0, 0.0];
    let maxs: vec3_t = [4.0, 4.0, 4.0];

    if ctx.world.globals.shieldAttachSound == 0 {
        ctx.world.globals.shieldLoopSound =
            G_SoundIndex(ctx, "sound/movers/doors/forcefield_lp.wav");
        ctx.world.globals.shieldAttachSound =
            G_SoundIndex(ctx, "sound/weapons/detpack/stick.wav");
        ctx.world.globals.shieldActivateSound =
            G_SoundIndex(ctx, "sound/movers/doors/forcefield_on.wav");
        ctx.world.globals.shieldDeactivateSound =
            G_SoundIndex(ctx, "sound/movers/doors/forcefield_off.wav");
        ctx.world.globals.shieldDamageSound = G_SoundIndex(ctx, "sound/effects/bumpfield.wav");
        // `shieldItem` (`static const gitem_t *`) is a function-scope
        // cache; recomputed each call here since the fn-scope caching
        // scheme isn't threaded through GameWorld for this local.
    }
    let shieldItem = BG_FindItemForHoldable(HI_SHIELD);

    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let client = ctx.entity(playerent).client;

    // can we place this in front of us?
    let viewangles = unsafe { (*client).ps.viewangles };
    AngleVectors(viewangles, Some(&mut fwd), None, None);
    fwd[2] = 0.0;
    dest = unsafe { (*client).ps.origin };
    for i in 0..3 {
        dest[i] += SHIELD_PLACEDIST * fwd[i];
    }
    let ps_origin = unsafe { (*client).ps.origin };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ps_origin as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &dest as *const vec3_t,
            ctx.entity(playerent).s.number,
            MASK_SHOT,
        ),
    );
    if tr.fraction > 0.9 {
        // room in front
        pos = tr.endpos;
        // drop to floor
        dest = [pos[0], pos[1], pos[2] - 4096.0];
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &pos as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &dest as *const vec3_t,
                ctx.entity(playerent).s.number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid == 0 && tr.allsolid == 0 {
            // got enough room so place the portable shield
            let shield = G_Spawn(ctx);

            // Figure out what direction the shield is facing.
            if fwd[0].abs() > fwd[1].abs() {
                // shield is north/south, facing east.
                ctx.entity_mut(shield).s.angles[YAW] = 0.0;
            } else {
                // shield is along the east/west axis, facing north
                ctx.entity_mut(shield).s.angles[YAW] = 90.0;
            }
            ctx.entity_mut(shield).think = Some(EntThink::CreateShield).into();
            ctx.entity_mut(shield).nextthink = ctx.world.level.time + 500; // power up after .5 seconds
            ctx.entity_mut(shield).parent = Some(playerent);

            // Set team number.
            let sessionTeam = unsafe { (*client).sess.sessionTeam };
            ctx.entity_mut(shield).s.otherEntityNum2 = sessionTeam;

            ctx.entity_mut(shield).s.eType = ET_SPECIAL as c_int;
            ctx.entity_mut(shield).s.modelindex = HI_SHIELD as c_int; // this'll be used in CG_Useable() for rendering.
            let classname: &'static CStr = unsafe { CStr::from_ptr(shieldItem.classname_cstr()) };
            ctx.ent_set(shield, PrefixSet::ClassnameStatic(classname));

            ctx.entity_mut(shield).r.contents = CONTENTS_TRIGGER;

            ctx.entity_mut(shield).touch = FnId::NONE;
            // using an item causes it to respawn
            ctx.entity_mut(shield).use_ = FnId::NONE; //Use_Item;

            // allow to ride movers
            ctx.entity_mut(shield).s.groundEntityNum = tr.entityNum as c_int;

            G_SetOrigin(ctx.entity_mut(shield), tr.endpos);

            ctx.entity_mut(shield).s.eFlags &= !EF_NODRAW;
            ctx.entity_mut(shield).r.svFlags &= !SVF_NOCLIENT;

            trap::LinkEntity(
                ctx.engine,
                GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(shield)).cast()),
            );

            ctx.entity_mut(shield).s.owner = ctx.entity(playerent).s.number;
            ctx.entity_mut(shield).s.shouldtarget = qtrue;
            if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
                let sessionTeam = unsafe { (*client).sess.sessionTeam };
                ctx.entity_mut(shield).s.teamowner = sessionTeam;
            } else {
                ctx.entity_mut(shield).s.teamowner = 16;
            }

            // Play placing sound...
            let shieldAttachSound = ctx.world.globals.shieldAttachSound;
            G_AddEvent(
                ctx.entity_mut(shield),
                entity_event_t::EV_GENERAL_SOUND as c_int,
                shieldAttachSound,
            );

            return qtrue;
        }
    }
    // no room
    qfalse
}

/// Raven `ItemUse_Binoculars`.
///
/// Source: `oracle/codemp/game/g_items.c:472-502`
pub fn ItemUse_Binoculars(ctx: &mut GameContext, ent: Option<EntityId>) {
    let Some(ent) = ent else {
        return;
    };
    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let client = ctx.entity(ent).client;
    if client.is_null() {
        return;
    }

    unsafe {
        if (*client).ps.weaponstate != WEAPON_READY as c_int {
            // So we can't fool it and reactivate while switching to the saber or something.
            return;
        }

        if (*client).ps.zoomMode == 0 {
            // not zoomed or currently zoomed with the disruptor
            (*client).ps.zoomMode = 2;
            (*client).ps.zoomLocked = qfalse;
            (*client).ps.zoomFov = 40.0;
        } else if (*client).ps.zoomMode == 2 {
            (*client).ps.zoomMode = 0;
            (*client).ps.zoomTime = ctx.world.level.time;
        }
    }
}

/// Raven `ItemUse_Shield`.
///
/// Source: `oracle/codemp/game/g_items.c:504-507`
pub fn ItemUse_Shield(ctx: &mut GameContext, ent: EntityId) {
    PlaceShield(ctx, ent);
}

/// Raven `SentryTouch`.
///
/// Source: `oracle/codemp/game/g_items.c:515-518`
pub fn SentryTouch(ent: EntityId, other: Option<EntityId>, trace: *mut trace_t) {
    return;
}

/// Raven `pas_fire`.
///
/// Source: `oracle/codemp/game/g_items.c:521-542`
pub fn pas_fire(ctx: &mut GameContext, ent: EntityId) {
    let mut myOrg = ctx.entity(ent).r.currentOrigin;
    myOrg[2] += 24.0;

    // Raven derefs `ent->enemy` unconditionally; callers only invoke
    // `pas_fire` when `ent->enemy` is non-null (see `pas_think`).
    let enemy = ctx.entity(ent).enemy.unwrap();
    // FLAG: enemy client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2b — the enemy may be any entity).
    let enemy_client = ctx.entity(enemy).client;
    let mut enOrg = unsafe { (*enemy_client).ps.origin };
    enOrg[2] += 24.0;

    let mut fwd = [
        enOrg[0] - myOrg[0],
        enOrg[1] - myOrg[1],
        enOrg[2] - myOrg[2],
    ];
    VectorNormalize(&mut fwd);

    myOrg[0] += fwd[0] * 16.0;
    myOrg[1] += fwd[1] * 16.0;
    myOrg[2] += fwd[2] * 16.0;

    let target = EntityId(ctx.entity(ent).genericValue3 as u32);
    WP_FireTurretMissile(
        ctx,
        target,
        myOrg,
        fwd,
        false,
        10,
        2300,
        MOD_SENTRY as c_int,
        Some(ent),
    );

    G_RunObject(ctx, ent);
}

/// Raven `pas_find_enemies`.
///
/// Source: `oracle/codemp/game/g_items.c:547-639`
pub fn pas_find_enemies(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    // Raven `#define TURRET_RADIUS 800` (`g_items.c:544`).
    pub const TURRET_RADIUS: f32 = 800.0;
    const MAX_GENTITIES: usize = mp_qshared::shared::MAX_GENTITIES;

    let mut found = qfalse;
    let mut bestDist = TURRET_RADIUS * TURRET_RADIUS;

    if ctx.entity(self_).aimDebounceTime > ctx.world.level.time {
        // time since we've been shut off
        if ctx.entity(self_).painDebounceTime < ctx.world.level.time {
            let sound = G_SoundIndex(ctx, "sound/chars/turret/ping.wav");
            G_Sound(ctx, Some(self_), CHAN_BODY, sound);
            ctx.entity_mut(self_).painDebounceTime = ctx.world.level.time + 1000;
        }
    }

    let org2 = ctx.entity(self_).s.pos.trBase;

    let mut entity_list: Vec<*mut gentity_t> = vec![core::ptr::null_mut(); MAX_GENTITIES];
    let count = G_RadiusList(
        ctx,
        org2,
        TURRET_RADIUS,
        Some(self_),
        qtrue,
        entity_list.as_mut_ptr(),
    );

    for i in 0..count {
        let target = ctx.entity_id_of(entity_list[i as usize]).unwrap();

        // FLAG: target client pointer (`gclient_t*`); deref stays raw through the
        // copied pointer value (recipe 2b — the target may be any entity).
        let target_client = ctx.entity(target).client;
        if target_client.is_null() {
            continue;
        }
        if target == self_
            || ctx.entity(target).takedamage == 0
            || ctx.entity(target).health <= 0
            || (ctx.entity(target).flags & FL_NOTARGET) != 0
        {
            continue;
        }
        if ctx.entity(self_).alliedTeam != 0
            && unsafe { (*target_client).sess.sessionTeam } == ctx.entity(self_).alliedTeam
        {
            continue;
        }
        if ctx.entity(self_).genericValue3 == ctx.entity(target).s.number {
            continue;
        }
        if trap::InPVS(
            ctx.engine,
            GInPvsArgs::new(
                &org2 as *const vec3_t,
                &ctx.entity(target).r.currentOrigin as *const vec3_t,
            ),
        ) == 0
        {
            continue;
        }

        if ctx.entity(target).s.eType == ET_NPC as c_int
            && ctx.entity(target).s.NPC_class == CLASS_VEHICLE as c_int
        {
            // don't get mad at vehicles, silly.
            continue;
        }

        let org = if !target_client.is_null() {
            unsafe { (*target_client).ps.origin }
        } else {
            ctx.entity(target).r.currentOrigin
        };

        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &org2 as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &org as *const vec3_t,
                ctx.entity(self_).s.number,
                MASK_SHOT,
            ),
        );

        if tr.allsolid == 0
            && tr.startsolid == 0
            && (tr.fraction == 1.0 || tr.entityNum as c_int == ctx.entity(target).s.number)
        {
            // Only acquire if have a clear shot, Is it in range and closer than our best?
            let enemyDir = [
                ctx.entity(target).r.currentOrigin[0] - ctx.entity(self_).r.currentOrigin[0],
                ctx.entity(target).r.currentOrigin[1] - ctx.entity(self_).r.currentOrigin[1],
                ctx.entity(target).r.currentOrigin[2] - ctx.entity(self_).r.currentOrigin[2],
            ];
            let enemyDist = VectorLengthSquared(enemyDir);

            if enemyDist < bestDist {
                // all things equal, keep current
                if ctx.entity(self_).attackDebounceTime + 100 < ctx.world.level.time {
                    // We haven't fired or acquired an enemy in the last 2 seconds-start-up sound
                    let sound = G_SoundIndex(ctx, "sound/chars/turret/startup.wav");
                    G_Sound(ctx, Some(self_), CHAN_BODY, sound);

                    // Wind up turrets for a bit
                    // C: `level.time + 900 + random()*200` — the int sum promotes
                    // to f32 against `random()*200`, truncating once at the store.
                    // Source: `oracle/codemp/game/g_items.c:628`
                    ctx.entity_mut(self_).attackDebounceTime = ((ctx.world.level.time + 900) as f32
                        + ctx.world.bg_state.rng.random() * 200.0)
                        as c_int;
                }

                G_SetEnemy(ctx, self_, Some(target));
                bestDist = enemyDist;
                found = qtrue;
            }
        }
    }

    found
}

/// Raven `pas_adjust_enemy`.
///
/// Source: `oracle/codemp/game/g_items.c:642-695`
pub fn pas_adjust_enemy(ctx: &mut GameContext, ent: EntityId) {
    let mut keep = qtrue;
    // Raven derefs `ent->enemy` unconditionally here; callers only invoke
    // `pas_adjust_enemy` when `ent->enemy` is non-null (see `pas_think`).
    let enemy = ctx.entity(ent).enemy.unwrap();

    if ctx.entity(enemy).health <= 0 {
        keep = qfalse;
    } else {
        let org2 = ctx.entity(ent).s.pos.trBase;
        // FLAG: enemy client pointer (`gclient_t*`); deref stays raw through the
        // copied pointer value (recipe 2b — the enemy may be any entity).
        let enemy_client = ctx.entity(enemy).client;
        let org = if !enemy_client.is_null() {
            let mut o = unsafe { (*enemy_client).ps.origin };
            o[2] -= 15.0;
            o
        } else {
            ctx.entity(enemy).r.currentOrigin
        };

        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &org2 as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &org as *const vec3_t,
                ctx.entity(ent).s.number,
                MASK_SHOT,
            ),
        );

        if tr.allsolid != 0
            || tr.startsolid != 0
            || tr.fraction < 0.9
            || tr.entityNum as c_int == ctx.entity(ent).s.number
        {
            if tr.entityNum as c_int != ctx.entity(enemy).s.number {
                // trace failed
                keep = qfalse;
            }
        }
    }

    if keep != 0 {
        //ent->bounceCount = level.time + 500 + random() * 150;
    } else if ctx.entity(ent).bounceCount < ctx.world.level.time && ctx.entity(ent).enemy.is_some()
    {
        // don't ping pong on and off
        ctx.entity_mut(ent).enemy = None;
        // shut-down sound
        let sound = G_SoundIndex(ctx, "sound/chars/turret/shutdown.wav");
        G_Sound(ctx, Some(ent), CHAN_BODY, sound);

        // C: `level.time + 500 + random()*150` — single truncation of the
        // promoted-to-f32 sum. Source: `oracle/codemp/game/g_items.c:690`
        ctx.entity_mut(ent).bounceCount = ((ctx.world.level.time + 500) as f32
            + ctx.world.bg_state.rng.random() * 150.0)
            as c_int;

        // make turret play ping sound for 5 seconds
        ctx.entity_mut(ent).aimDebounceTime = ctx.world.level.time + 5000;
    }
}

/// Raven `sentryExpire`.
///
/// Source: `oracle/codemp/game/g_items.c:702-705`
pub fn sentryExpire(ctx: &mut GameContext, self_: EntityId) {
    turret_die(
        ctx,
        self_,
        Some(self_),
        Some(self_),
        1000,
        MOD_UNKNOWN as c_int,
    );
}

/// Raven `pas_think`.
///
/// Source: `oracle/codemp/game/g_items.c:708-937`
pub fn pas_think(ctx: &mut GameContext, ent: EntityId) {
    const MAX_GENTITIES: usize = mp_qshared::shared::MAX_GENTITIES;

    let testMins: vec3_t = [
        ctx.entity(ent).r.currentOrigin[0] + ctx.entity(ent).r.mins[0] + 4.0,
        ctx.entity(ent).r.currentOrigin[1] + ctx.entity(ent).r.mins[1] + 4.0,
        ctx.entity(ent).r.currentOrigin[2] + ctx.entity(ent).r.mins[2] + 4.0,
    ];
    let testMaxs: vec3_t = [
        ctx.entity(ent).r.currentOrigin[0] + ctx.entity(ent).r.maxs[0] - 4.0,
        ctx.entity(ent).r.currentOrigin[1] + ctx.entity(ent).r.maxs[1] - 4.0,
        ctx.entity(ent).r.currentOrigin[2] + ctx.entity(ent).r.maxs[2] - 4.0,
    ];

    let mut iEntityList: Vec<c_int> = vec![0; MAX_GENTITIES];
    let mut numListedEntities = trap::EntitiesInBox(
        ctx.engine,
        GEntitiesInBoxArgs::new(
            &testMins as *const vec3_t,
            &testMaxs as *const vec3_t,
            iEntityList.as_mut_ptr(),
            MAX_GENTITIES as c_int,
        ),
    );

    let mut i = 0;
    let mut clTrapped = qfalse;
    while i < numListedEntities {
        if iEntityList[i as usize] < mp_qshared::shared::MAX_CLIENTS as c_int {
            // client stuck inside me. go nonsolid.
            let clNum = iEntityList[i as usize];

            numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &ctx.world.g_entities[clNum as usize].r.absmin as *const vec3_t,
                    &ctx.world.g_entities[clNum as usize].r.absmax as *const vec3_t,
                    iEntityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            i = 0;
            while i < numListedEntities {
                if iEntityList[i as usize] == ctx.entity(ent).s.number {
                    clTrapped = qtrue;
                    break;
                }
                i += 1;
            }
            break;
        }

        i += 1;
    }

    if clTrapped != 0 {
        ctx.entity_mut(ent).r.contents = 0;
        ctx.entity_mut(ent).s.fireflag = 0;
        ctx.entity_mut(ent).nextthink = ctx.world.level.time + FRAMETIME;
        return;
    } else {
        ctx.entity_mut(ent).r.contents = CONTENTS_SOLID;
    }

    let ownerIdx = ctx.entity(ent).genericValue3 as usize;
    // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2b — read exactly what Raven derefs, and
    // only after the `inuse`/null short-circuit passes).
    let owner_client = ctx.world.g_entities[ownerIdx].client;
    if ctx.world.g_entities[ownerIdx].inuse == 0
        || owner_client.is_null()
        || unsafe { (*owner_client).sess.sessionTeam } != ctx.entity(ent).genericValue2
    {
        ctx.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
        ctx.entity_mut(ent).nextthink = ctx.world.level.time;
        return;
    }

    //	G_RunObject(ent);

    if ctx.entity(ent).damage == 0 {
        ctx.entity_mut(ent).damage = 1;
        ctx.entity_mut(ent).nextthink = ctx.world.level.time + FRAMETIME;
        return;
    }

    if ctx.entity(ent).genericValue8 + TURRET_LIFETIME < ctx.world.level.time {
        let sound = G_SoundIndex(ctx, "sound/chars/turret/shutdown.wav");
        G_Sound(ctx, Some(ent), CHAN_BODY, sound);
        ctx.entity_mut(ent).s.bolt2 = ENTITYNUM_NONE;
        ctx.entity_mut(ent).s.fireflag = 2;

        ctx.entity_mut(ent).think = Some(EntThink::sentryExpire).into();
        ctx.entity_mut(ent).nextthink = ctx.world.level.time + TURRET_DEATH_DELAY;
        return;
    }

    ctx.entity_mut(ent).nextthink = ctx.world.level.time + FRAMETIME;

    if ctx.entity(ent).enemy.is_some() {
        // make sure that the enemy is still valid
        pas_adjust_enemy(ctx, ent);
    }

    if let Some(enemy_id) = ctx.entity(ent).enemy {
        // FLAG: enemy client pointer (`gclient_t*`); deref stays raw (recipe 2b).
        let enemy_client = ctx.entity(enemy_id).client;
        if enemy_client.is_null() {
            ctx.entity_mut(ent).enemy = None;
        } else if ctx.entity(enemy_id).s.number == ctx.entity(ent).s.number {
            ctx.entity_mut(ent).enemy = None;
        } else if ctx.entity(enemy_id).health < 1 {
            ctx.entity_mut(ent).enemy = None;
        }
    }

    if ctx.entity(ent).enemy.is_none() {
        pas_find_enemies(ctx, ent);
    }

    if let Some(enemy_id) = ctx.entity(ent).enemy {
        let num = ctx.entity(enemy_id).s.number;
        ctx.entity_mut(ent).s.bolt2 = num;
    } else {
        ctx.entity_mut(ent).s.bolt2 = ENTITYNUM_NONE;
    }

    let mut moved = qfalse;
    let mut diffYaw: f32 = 0.0;
    let mut diffPitch: f32 = 0.0;

    let speed = AngleNormalize360(ctx.entity(ent).speed);
    ctx.entity_mut(ent).speed = speed;
    let random = AngleNormalize360(ctx.entity(ent).random);
    ctx.entity_mut(ent).random = random;

    if let Some(enemy_id) = ctx.entity(ent).enemy {
        // ...then we'll calculate what new aim adjustments we should attempt to make this frame
        // Aim at enemy
        // FLAG: enemy client pointer (`gclient_t*`); deref stays raw (recipe 2b).
        let enemy_client = ctx.entity(enemy_id).client;
        let org = if !enemy_client.is_null() {
            unsafe { (*enemy_client).ps.origin }
        } else {
            ctx.entity(enemy_id).r.currentOrigin
        };

        let enemyDir = [
            org[0] - ctx.entity(ent).r.currentOrigin[0],
            org[1] - ctx.entity(ent).r.currentOrigin[1],
            org[2] - ctx.entity(ent).r.currentOrigin[2],
        ];
        let mut desiredAngles: vec3_t = [0.0; 3];
        vectoangles(enemyDir, &mut desiredAngles);

        diffYaw = AngleSubtract(ctx.entity(ent).speed, desiredAngles[YAW]);
        diffPitch = AngleSubtract(ctx.entity(ent).random, desiredAngles[PITCH]);
    } else {
        // no enemy, so make us slowly sweep back and forth as if searching for a new one
        // `sin` is the double libm function: the float argument is widened to f64,
        // evaluated in f64, then narrowed back to the f32 result.
        diffYaw = (((ctx.world.level.time as f32 * 0.0001 + ctx.entity(ent).count as f32) as f64)
            .sin()
            * 2.0) as f32;
    }

    if diffYaw.abs() > 0.25 {
        moved = qtrue;

        if diffYaw.abs() > 10.0 {
            // cap max speed
            ctx.entity_mut(ent).speed += if diffYaw > 0.0 { -10.0 } else { 10.0 };
        } else {
            // small enough
            ctx.entity_mut(ent).speed -= diffYaw;
        }
    }

    if diffPitch.abs() > 0.25 {
        moved = qtrue;

        if diffPitch.abs() > 4.0 {
            // cap max speed
            ctx.entity_mut(ent).random += if diffPitch > 0.0 { -4.0 } else { 4.0 };
        } else {
            // small enough
            ctx.entity_mut(ent).random -= diffPitch;
        }
    }

    // the bone axes are messed up, so hence some dumbness here
    let _frontAngles: vec3_t = [-ctx.entity(ent).random, 0.0, 0.0];
    let _backAngles: vec3_t = [0.0, 0.0, ctx.entity(ent).speed];

    if moved != 0 {
        //ent->s.loopSound = G_SoundIndex( "sound/chars/turret/move.wav" );
    } else {
        ctx.entity_mut(ent).s.loopSound = 0;
        ctx.entity_mut(ent).s.loopIsSoundset = qfalse;
    }

    if ctx.entity(ent).enemy.is_some() && ctx.entity(ent).attackDebounceTime < ctx.world.level.time
    {
        ctx.entity_mut(ent).count -= 1;

        if ctx.entity(ent).count != 0 {
            pas_fire(ctx, ent);
            ctx.entity_mut(ent).s.fireflag = 1;
            ctx.entity_mut(ent).attackDebounceTime = ctx.world.level.time + 200;
        } else {
            //ent->nextthink = 0;
            let sound = G_SoundIndex(ctx, "sound/chars/turret/shutdown.wav");
            G_Sound(ctx, Some(ent), CHAN_BODY, sound);
            ctx.entity_mut(ent).s.bolt2 = ENTITYNUM_NONE;
            ctx.entity_mut(ent).s.fireflag = 2;
            ctx.entity_mut(ent).nextthink = ctx.world.level.time + TURRET_DEATH_DELAY;
        }
    } else {
        ctx.entity_mut(ent).s.fireflag = 0;
    }
}

/// Raven `turret_die`.
///
/// Source: `oracle/codemp/game/g_items.c:940-973`
pub fn turret_die(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    // Turn off the thinking of the base & use it's targets
    ctx.entity_mut(self_).think = FnId::NONE;
    ctx.entity_mut(self_).use_ = FnId::NONE;

    if ctx.entity(self_).target.is_some() {
        G_UseTargets(ctx, Some(self_), attacker);
    }

    let owner = EntityId(ctx.entity(self_).genericValue3 as u32);
    // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2b — read exactly what Raven derefs).
    let owner_client = ctx.entity(owner).client;
    if ctx.entity(owner).inuse == 0 || owner_client.is_null() {
        G_FreeEntity(ctx, Some(self_));
        return;
    }

    // clear my data
    ctx.entity_mut(self_).die = FnId::NONE;
    ctx.entity_mut(self_).takedamage = qfalse;
    ctx.entity_mut(self_).health = 0;

    // hack the effect angle so that explode death can orient the effect properly
    ctx.entity_mut(self_).s.angles = [0.0, 0.0, 1.0];

    G_PlayEffect(
        EFFECT_EXPLOSION_PAS as c_int,
        ctx.entity(self_).s.pos.trBase,
        ctx.entity(self_).s.angles,
    );
    let trBase = ctx.entity(self_).s.pos.trBase;
    G_RadiusDamage(
        ctx,
        trBase,
        Some(owner),
        30.0,
        256.0,
        Some(self_),
        Some(self_),
        MOD_UNKNOWN as c_int,
    );

    unsafe {
        (*owner_client).ps.fd.sentryDeployed = qfalse;
    }

    //ExplodeDeath( self );
    G_FreeEntity(ctx, Some(self_));
}

/// Raven `SP_PAS`.
///
/// Source: `oracle/codemp/game/g_items.c:978-1011`
pub fn SP_PAS(ctx: &mut GameContext, base: EntityId) {
    // Raven `#define TURRET_AMMO_COUNT 40` (`g_items.c:975`).
    pub const TURRET_AMMO_COUNT: c_int = 40;

    if ctx.entity(base).count == 0 {
        // give ammo
        ctx.entity_mut(base).count = TURRET_AMMO_COUNT;
    }

    ctx.entity_mut(base).s.bolt1 = 1; // This is a sort of hack to indicate that this model needs special turret things done to it
    ctx.entity_mut(base).s.bolt2 = ENTITYNUM_NONE; // store our current enemy index

    ctx.entity_mut(base).damage = 0; // start animation flag

    ctx.entity_mut(base).r.mins = [-8.0, -8.0, 0.0];
    ctx.entity_mut(base).r.maxs = [8.0, 8.0, 24.0];

    G_RunObject(ctx, base);

    ctx.entity_mut(base).think = Some(EntThink::pas_think).into();
    ctx.entity_mut(base).nextthink = ctx.world.level.time + FRAMETIME;

    if ctx.entity(base).health == 0 {
        ctx.entity_mut(base).health = 50;
    }

    ctx.entity_mut(base).takedamage = qtrue;
    ctx.entity_mut(base).die = Some(EntDie::turret_die).into();

    ctx.entity_mut(base).physicsObject = qtrue;

    let sound = G_SoundIndex(ctx, "sound/chars/turret/startup.wav");
    G_Sound(ctx, Some(base), CHAN_BODY, sound);
}

/// Raven `ItemUse_Sentry`.
///
/// Source: `oracle/codemp/game/g_items.c:1014-1093`
pub fn ItemUse_Sentry(ctx: &mut GameContext, ent: Option<EntityId>) {
    let Some(ent) = ent else {
        return;
    };
    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let client = ctx.entity(ent).client;
    if client.is_null() {
        return;
    }

    let mins: vec3_t = [-8.0, -8.0, 0.0];
    let maxs: vec3_t = [8.0, 8.0, 24.0];

    let sessionTeam = unsafe { (*client).sess.sessionTeam };

    let mut yawonly: vec3_t = [0.0; 3];
    yawonly[ROLL] = 0.0;
    yawonly[PITCH] = 0.0;
    yawonly[YAW] = unsafe { (*client).ps.viewangles[YAW] };

    let mut fwd: vec3_t = [0.0; 3];
    AngleVectors(yawonly, Some(&mut fwd), None, None);

    let mut fwdorg: vec3_t = [0.0; 3];
    let origin = unsafe { (*client).ps.origin };
    fwdorg[0] = origin[0] + fwd[0] * 64.0;
    fwdorg[1] = origin[1] + fwd[1] * 64.0;
    fwdorg[2] = origin[2] + fwd[2] * 64.0;

    let sentry = G_Spawn(ctx);

    ctx.ent_set(sentry, PrefixSet::ClassnameStatic(c"sentryGun"));
    ctx.entity_mut(sentry).s.modelindex = G_ModelIndex(ctx, "models/items/psgun.glm"); // replace ASAP

    ctx.entity_mut(sentry).s.g2radius = 30;
    ctx.entity_mut(sentry).s.modelGhoul2 = 1;

    G_SetOrigin(ctx.entity_mut(sentry), fwdorg);
    ctx.entity_mut(sentry).parent = Some(ent);
    ctx.entity_mut(sentry).r.contents = CONTENTS_SOLID;
    ctx.entity_mut(sentry).s.solid = 2;
    ctx.entity_mut(sentry).clipmask = MASK_SOLID;
    ctx.entity_mut(sentry).r.mins = mins;
    ctx.entity_mut(sentry).r.maxs = maxs;
    ctx.entity_mut(sentry).genericValue3 = ctx.entity(ent).s.number;
    ctx.entity_mut(sentry).genericValue2 = sessionTeam; // so we can remove ourself if our owner changes teams
    let trBase = ctx.entity(sentry).s.pos.trBase;
    ctx.entity_mut(sentry).r.absmin[0] = trBase[0] + mins[0];
    ctx.entity_mut(sentry).r.absmin[1] = trBase[1] + mins[1];
    ctx.entity_mut(sentry).r.absmin[2] = trBase[2] + mins[2];
    ctx.entity_mut(sentry).r.absmax[0] = trBase[0] + maxs[0];
    ctx.entity_mut(sentry).r.absmax[1] = trBase[1] + maxs[1];
    ctx.entity_mut(sentry).r.absmax[2] = trBase[2] + maxs[2];
    ctx.entity_mut(sentry).s.eType = ET_GENERAL as c_int;
    ctx.entity_mut(sentry).s.pos.trType = trType_t::TR_GRAVITY; //STATIONARY;
    ctx.entity_mut(sentry).s.pos.trTime = ctx.world.level.time;
    ctx.entity_mut(sentry).touch = Some(EntTouch::SentryTouch).into();
    ctx.entity_mut(sentry).nextthink = ctx.world.level.time;
    ctx.entity_mut(sentry).genericValue4 = ENTITYNUM_NONE; // genericValue4 used as enemy index

    ctx.entity_mut(sentry).genericValue5 = 1000;

    ctx.entity_mut(sentry).genericValue8 = ctx.world.level.time;

    ctx.entity_mut(sentry).alliedTeam = sessionTeam;

    unsafe {
        (*client).ps.fd.sentryDeployed = qtrue;
    }

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(sentry)).cast()),
    );

    ctx.entity_mut(sentry).s.owner = ctx.entity(ent).s.number;
    ctx.entity_mut(sentry).s.shouldtarget = qtrue;
    if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
        ctx.entity_mut(sentry).s.teamowner = sessionTeam;
    } else {
        ctx.entity_mut(sentry).s.teamowner = 16;
    }

    SP_PAS(ctx, sentry);
}

/// Raven `ItemUse_Seeker`.
///
/// Source: `oracle/codemp/game/g_items.c:1096-1125`
pub fn ItemUse_Seeker(ctx: &mut GameContext, ent: EntityId) {
    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let ent_client = ctx.entity(ent).client;
    if ctx.world.cvars.g_gametype.integer == GT_SIEGE
        && ctx.world.cvars.d_siegeSeekerNPC.integer != 0
    {
        // actualy spawn a remote NPC
        let remote = NPC_SpawnType(ctx, Some(ent), "remote", None, qfalse);
        if !remote.is_null() {
            let remote_id = ctx.entity_id_of(remote).unwrap();
            // FLAG: NPC client pointer; deref stays raw (recipe 2b — remote is an NPC).
            let remote_client = ctx.entity(remote_id).client;
            if !remote_client.is_null() {
                // set it to my team
                ctx.entity_mut(remote_id).r.ownerNum = ctx.entity(ent).s.number;
                ctx.entity_mut(remote_id).s.owner = ctx.entity(ent).s.number;
                ctx.entity_mut(remote_id).activator = Some(ent);
                let sessionTeam = unsafe { (*ent_client).sess.sessionTeam };
                if sessionTeam == TEAM_BLUE {
                    unsafe {
                        (*remote_client).playerTeam = NPCTEAM_PLAYER;
                    }
                } else if sessionTeam == TEAM_RED {
                    unsafe {
                        (*remote_client).playerTeam = NPCTEAM_ENEMY;
                    }
                } else {
                    unsafe {
                        (*remote_client).playerTeam = NPCTEAM_NEUTRAL;
                    }
                }
            }
        }
    } else {
        unsafe {
            (*ent_client).ps.eFlags |= EF_SEEKERDRONE;
            (*ent_client).ps.droneExistTime = (ctx.world.level.time + 30000) as f32;
            (*ent_client).ps.droneFireTime = (ctx.world.level.time + 1500) as f32;
        }
    }
}

/// Raven `MedPackGive`.
///
/// Source: `oracle/codemp/game/g_items.c:1127-1152`
pub fn MedPackGive(ent: &mut gentity_t, amount: c_int) {
    // FLAG: client pointer (`gclient_t*`); deref stays raw (recipe 2 — ctx-free
    // leaf has no accessor). The `ent.is_null()` arm is vacuous (a live borrow is
    // never null) and dropped.
    let cl = ent.client;
    if cl.is_null() {
        return;
    }

    unsafe {
        if ent.health <= 0
            || (*cl).ps.stats[statIndex_t::STAT_HEALTH as usize] <= 0
            || ((*cl).ps.eFlags & EF_DEAD) != 0
        {
            return;
        }

        if ent.health >= (*cl).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] {
            return;
        }

        ent.health += amount;

        if ent.health > (*cl).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] {
            ent.health = (*cl).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize];
        }
    }
}

/// Raven `ItemUse_MedPack_Big`.
///
/// Source: `oracle/codemp/game/g_items.c:1154-1157`
pub fn ItemUse_MedPack_Big(ent: &mut gentity_t) {
    MedPackGive(ent, MAX_MEDPACK_BIG_HEAL_AMOUNT);
}

/// Raven `ItemUse_MedPack`.
///
/// Source: `oracle/codemp/game/g_items.c:1159-1162`
pub fn ItemUse_MedPack(ent: &mut gentity_t) {
    MedPackGive(ent, MAX_MEDPACK_HEAL_AMOUNT);
}

/// Raven `Jetpack_Off`.
///
/// Source: `oracle/codemp/game/g_items.c:1165-1175`
pub fn Jetpack_Off(ent: &mut gentity_t) {
    // FLAG: client pointer (`gclient_t*`); deref stays raw (recipe 2 — ctx-free
    // leaf has no accessor).
    let cl = ent.client;
    debug_assert!(!cl.is_null());

    unsafe {
        if (*cl).jetPackOn == qfalse {
            // already off
            return;
        }

        (*cl).jetPackOn = qfalse;
    }
}

/// Raven `Jetpack_On`.
///
/// Source: `oracle/codemp/game/g_items.c:1177-1199`
pub fn Jetpack_On(ctx: &mut GameContext, ent: EntityId) {
    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let cl = ctx.entity(ent).client;
    debug_assert!(!cl.is_null());

    unsafe {
        if (*cl).jetPackOn != qfalse {
            // aready on
            return;
        }

        if (*cl).ps.fd.forceGripBeingGripped >= ctx.world.level.time as f32 {
            // can't turn on during grip interval
            return;
        }

        if (*cl).ps.fallingToDeath != qfalse {
            // too late!
            return;
        }
    }

    let sound = G_SoundIndex(ctx, "sound/boba/JETON");
    G_Sound(ctx, Some(ent), CHAN_AUTO, sound);

    unsafe {
        (*cl).jetPackOn = qtrue;
    }
}

/// Raven `ItemUse_Jetpack`.
///
/// Source: `oracle/codemp/game/g_items.c:1201-1234`
pub fn ItemUse_Jetpack(ctx: &mut GameContext, ent: EntityId) {
    // Raven `#define JETPACK_TOGGLE_TIME` (`g_items.c`).
    pub const JETPACK_TOGGLE_TIME: c_int = 1000;

    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let cl = ctx.entity(ent).client;
    debug_assert!(!cl.is_null());

    if unsafe { (*cl).jetPackToggleTime } >= ctx.world.level.time {
        return;
    }

    if ctx.entity(ent).health <= 0
        || unsafe {
            (*cl).ps.stats[STAT_HEALTH as usize] <= 0
                || ((*cl).ps.eFlags & EF_DEAD) != 0
                || (*cl).ps.pm_type == PM_DEAD as c_int
        }
    {
        // can't use it when dead under any circumstances.
        return;
    }

    if unsafe { (*cl).jetPackOn == qfalse && (*cl).ps.jetpackFuel < 5 } {
        // too low on fuel to start it up
        return;
    }

    if unsafe { (*cl).jetPackOn } != qfalse {
        Jetpack_Off(ctx.entity_mut(ent));
    } else {
        Jetpack_On(ctx, ent);
    }

    unsafe {
        (*cl).jetPackToggleTime = ctx.world.level.time + JETPACK_TOGGLE_TIME;
    }
}

/// Raven `ItemUse_UseCloak`.
///
/// Source: `oracle/codemp/game/g_items.c:1239-1272`
pub fn ItemUse_UseCloak(ctx: &mut GameContext, ent: EntityId) {
    // Raven `#define CLOAK_TOGGLE_TIME` (`g_items.c`).
    pub const CLOAK_TOGGLE_TIME: c_int = 1000;

    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let cl = ctx.entity(ent).client;
    debug_assert!(!cl.is_null());

    if unsafe { (*cl).cloakToggleTime } >= ctx.world.level.time {
        return;
    }

    if ctx.entity(ent).health <= 0
        || unsafe {
            (*cl).ps.stats[STAT_HEALTH as usize] <= 0
                || ((*cl).ps.eFlags & EF_DEAD) != 0
                || (*cl).ps.pm_type == PM_DEAD as c_int
        }
    {
        // can't use it when dead under any circumstances.
        return;
    }

    if unsafe { (*cl).ps.powerups[PW_CLOAKED as usize] == 0 && (*cl).ps.cloakFuel < 5 } {
        // too low on fuel to start it up
        return;
    }

    if unsafe { (*cl).ps.powerups[PW_CLOAKED as usize] } != 0 {
        // decloak
        Jedi_Decloak(ctx, Some(ent));
    } else {
        // cloak
        Jedi_Cloak(ctx, Some(ent));
    }

    unsafe {
        (*cl).cloakToggleTime = ctx.world.level.time + CLOAK_TOGGLE_TIME;
    }
}

/// Raven `SpecialItemThink`.
///
/// Source: `oracle/codemp/game/g_items.c:1277-1293`
pub fn SpecialItemThink(ctx: &mut GameContext, ent: EntityId) {
    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    if ctx.entity(ent).genericValue5 < ctx.world.level.time {
        ctx.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
        ctx.entity_mut(ent).nextthink = ctx.world.level.time;
        return;
    }

    G_RunExPhys(
        ctx,
        ent,
        gravity,
        mass,
        bounce,
        false,
        core::ptr::null_mut(),
        0,
    );
    let currentOrigin = ctx.entity(ent).r.currentOrigin;
    ctx.entity_mut(ent).s.origin = currentOrigin;
    ctx.entity_mut(ent).nextthink = ctx.world.level.time + 50;
}

/// Raven `G_SpecialSpawnItem`.
///
/// Source: `oracle/codemp/game/g_items.c:1295-1331`
pub fn G_SpecialSpawnItem(ctx: &mut GameContext, ent: EntityId, item: ItemId) {
    RegisterItem(ctx, item);
    ctx.entity_mut(ent).item = Some(item);

    // go away if no one wants me
    ctx.entity_mut(ent).genericValue5 = ctx.world.level.time + TOSSED_ITEM_STAY_PERIOD;
    ctx.entity_mut(ent).think = Some(EntThink::SpecialItemThink).into();
    ctx.entity_mut(ent).nextthink = ctx.world.level.time + 50;
    ctx.entity_mut(ent).clipmask = MASK_SOLID;

    ctx.entity_mut(ent).physicsBounce = 0.50; // items are bouncy
    ctx.entity_mut(ent).r.mins = [-8.0, -8.0, 0.0];
    ctx.entity_mut(ent).r.maxs = [8.0, 8.0, 16.0];

    ctx.entity_mut(ent).s.eType = ET_ITEM as c_int;
    // store item number in modelindex
    let modelindex = item.modelindex();
    ctx.entity_mut(ent).s.modelindex = modelindex;

    ctx.entity_mut(ent).r.contents = CONTENTS_TRIGGER;
    ctx.entity_mut(ent).touch = Some(EntTouch::Touch_Item).into();

    // can't touch owner for x seconds
    ctx.entity_mut(ent).genericValue11 = ctx.entity(ent).r.ownerNum;
    ctx.entity_mut(ent).genericValue10 = ctx.world.level.time + TOSSED_ITEM_OWNER_NOTOUCH_DUR;

    // so we know to remove when picked up, not respawn
    ctx.entity_mut(ent).genericValue9 = 1;

    // kind of a lame value to use, but oh well. This means don't
    // pick up this item clientside with prediction, because we
    // aren't sending over all the data necessary for the player
    // to know if he can.
    ctx.entity_mut(ent).s.brokenLimbs = 1;

    // since it uses my server-only physics
    ctx.entity_mut(ent).s.eFlags |= EF_CLIENTSMOOTH;
}

/// Raven `G_PrecacheDispensers`.
///
/// Source: `oracle/codemp/game/g_items.c:1336-1351`
pub fn G_PrecacheDispensers(ctx: &mut GameContext) {
    if let Some(item) = BG_FindItem("item_medpak_instant") {
        crate::g_items::RegisterItem(ctx, item);
    }

    if let Some(item) = BG_FindItem("ammo_all") {
        crate::g_items::RegisterItem(ctx, item);
    }
}

/// Raven `ItemUse_UseDisp`.
///
/// Source: `oracle/codemp/game/g_items.c:1353-1410`
pub fn ItemUse_UseDisp(ctx: &mut GameContext, ent: EntityId, r#type: c_int) {
    // Raven `#define TOSS_DEBOUNCE_TIME 5000` (`bg_public.h:181`).
    pub const TOSS_DEBOUNCE_TIME: c_int = 5000;

    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let cl = ctx.entity(ent).client;
    if cl.is_null() || unsafe { (*cl).tossableItemDebounce } > ctx.world.level.time {
        // can't use it again yet
        return;
    }

    if unsafe { (*cl).ps.weaponTime > 0 || (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int } {
        // busy doing something else
        return;
    }

    unsafe {
        (*cl).tossableItemDebounce = ctx.world.level.time + TOSS_DEBOUNCE_TIME;
    }

    let item = if r#type == HI_HEALTHDISP as c_int {
        BG_FindItem("item_medpak_instant")
    } else {
        BG_FindItem("ammo_all")
    };

    if let Some(item) = item {
        let eItem = G_Spawn(ctx);
        ctx.entity_mut(eItem).r.ownerNum = ctx.entity(ent).s.number;
        let classname: &'static CStr = unsafe { CStr::from_ptr(item.classname_cstr()) };
        ctx.ent_set(eItem, PrefixSet::ClassnameStatic(classname));

        let mut pos = unsafe { (*cl).ps.origin };
        pos[2] += unsafe { (*cl).ps.viewheight } as f32;

        G_SetOrigin(ctx.entity_mut(eItem), pos);
        let currentOrigin = ctx.entity(eItem).r.currentOrigin;
        ctx.entity_mut(eItem).s.origin = currentOrigin;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(eItem)).cast()),
        );

        G_SpecialSpawnItem(ctx, eItem, item);

        let mut fwd: vec3_t = [0.0; 3];
        let viewangles = unsafe { (*cl).ps.viewangles };
        AngleVectors(viewangles, Some(&mut fwd), None, None);
        ctx.entity_mut(eItem).epVelocity = [fwd[0] * 128.0, fwd[1] * 128.0, fwd[2] * 128.0];
        ctx.entity_mut(eItem).epVelocity[2] = 16.0;

        //	G_SetAnim( ent, NULL, SETANIM_TORSO, BOTH_THERMAL_THROW, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD, 0 );

        let origin = unsafe { (*cl).ps.origin };
        let te = G_TempEntity(ctx, origin, entity_event_t::EV_LOCALTIMER as c_int);
        ctx.entity_mut(te).s.time = ctx.world.level.time;
        ctx.entity_mut(te).s.time2 = TOSS_DEBOUNCE_TIME;
        let clientNum = unsafe { (*cl).ps.clientNum };
        ctx.entity_mut(te).s.owner = clientNum;
    }
}

/// Raven `EWebDisattach`.
///
/// Source: `oracle/codemp/game/g_items.c:1417-1431`
pub fn EWebDisattach(ctx: &mut GameContext, owner: EntityId, eweb: EntityId) {
    // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let owner_client = ctx.entity(owner).client;
    unsafe {
        (*owner_client).ewebIndex = 0;
        (*owner_client).ps.emplacedIndex = 0;
    }
    if ctx.entity(owner).health > 0 {
        let genericValue11 = ctx.entity(eweb).genericValue11;
        unsafe {
            (*owner_client).ps.stats[STAT_WEAPONS as usize] = genericValue11;
        }
    } else {
        unsafe {
            (*owner_client).ps.stats[STAT_WEAPONS as usize] = 0;
        }
    }
    ctx.entity_mut(eweb).think = Some(EntThink::G_FreeEntity).into();
    ctx.entity_mut(eweb).nextthink = ctx.world.level.time;
}

/// Raven `EWebPrecache`.
///
/// Source: `oracle/codemp/game/g_items.c:1434-1439`
pub fn EWebPrecache(ctx: &mut GameContext) {
    RegisterItem(ctx, BG_FindItemForWeapon(WP_TURRET));
    G_EffectIndex(ctx, "detpack/explosion.efx");
    G_EffectIndex(ctx, "turret/muzzle_flash.efx");
}

/// Raven `EWebDie`.
///
/// Source: `oracle/codemp/game/g_items.c:1449-1481`
pub fn EWebDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    // Raven `#define EWEB_DEATH_DMG 90` / `EWEB_DEATH_RADIUS 128` (`g_items.c:1442-1443`).
    pub const EWEB_DEATH_DMG: f32 = 90.0;
    pub const EWEB_DEATH_RADIUS: f32 = 128.0;

    let currentOrigin = ctx.entity(self_).r.currentOrigin;
    G_RadiusDamage(
        ctx,
        currentOrigin,
        Some(self_),
        EWEB_DEATH_DMG,
        EWEB_DEATH_RADIUS,
        Some(self_),
        Some(self_),
        MOD_SUICIDE as c_int,
    );

    let fxDir: vec3_t = [1.0, 0.0, 0.0];
    G_PlayEffect(
        EFFECT_EXPLOSION_DETPACK as c_int,
        ctx.entity(self_).r.currentOrigin,
        fxDir,
    );

    if ctx.entity(self_).r.ownerNum != ENTITYNUM_NONE {
        let owner = EntityId(ctx.entity(self_).r.ownerNum as u32);
        // FLAG: owner client pointer (`gclient_t*`); deref stays raw (recipe 2b).
        let owner_client = ctx.entity(owner).client;

        if ctx.entity(owner).inuse != 0 && !owner_client.is_null() {
            EWebDisattach(ctx, owner, self_);

            // make sure it resets next time we spawn one in case we someone obtain one before death
            unsafe {
                (*owner_client).ewebHealth = -1;

                // take it away from him, it is gone forever.
                (*owner_client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_EWEB);
            }

            let hi = unsafe { (*owner_client).ps.stats[STAT_HOLDABLE_ITEM as usize] };
            if hi > 0 && bg_itemlist[hi as usize].kind == ItemKind::Holdable(HI_EWEB) {
                //he has it selected so deselect it and select the first thing available
                unsafe {
                    (*owner_client).ps.stats[STAT_HOLDABLE_ITEM as usize] = 0;
                    BG_CycleInven(&mut (*owner_client).ps as *mut playerState_t, 1);
                }
            }
        }
    }
}

/// Raven `EWebPain`.
///
/// Source: `oracle/codemp/game/g_items.c:1484-1496`
pub fn EWebPain(ctx: &mut GameContext, self_: EntityId, attacker: Option<EntityId>, damage: c_int) {
    // update the owner's health status of me
    if ctx.entity(self_).r.ownerNum != ENTITYNUM_NONE {
        let owner = EntityId(ctx.entity(self_).r.ownerNum as u32);
        // FLAG: owner client pointer (`gclient_t*`); deref stays raw (recipe 2b).
        let owner_client = ctx.entity(owner).client;

        if ctx.entity(owner).inuse != 0 && !owner_client.is_null() {
            let health = ctx.entity(self_).health;
            unsafe {
                (*owner_client).ewebHealth = health;
            }
        }
    }
}

/// Raven `EWeb_SetBoneAngles`.
///
/// Source: `oracle/codemp/game/g_items.c:1499-1598`
pub fn EWeb_SetBoneAngles(ctx: &mut GameContext, ent: EntityId, bone: *mut c_char, angles: vec3_t) {
    // Raven `BONE_ANGLES_POSTMULT` resolves via the canonical
    // `mp_qshared::common::mp::ghoul2::bone_flags` module (crate prelude glob).
    // Orientations (`POSITIVE_Y`/`NEGATIVE_Z`/`NEGATIVE_X`) come from the
    // already-ported `Eorientations` enum (prelude glob import).

    // `bone` is a `char*` — stays raw.
    let boneIndex = G_BoneIndex(ctx, &(unsafe { cstr_to_str(bone) }));

    // Walk the 4 fixed bone-index/bone-angle slot pairs looking for an
    // existing match, else the first free slot (`g_items.c:1499-1550`).
    let mut slot: Option<usize> = None;
    let mut freeSlot: Option<usize> = None;
    for i in 0..4 {
        let idx = match i {
            0 => ctx.entity(ent).s.boneIndex1,
            1 => ctx.entity(ent).s.boneIndex2,
            2 => ctx.entity(ent).s.boneIndex3,
            _ => ctx.entity(ent).s.boneIndex4,
        };
        if idx == 0 && freeSlot.is_none() {
            freeSlot = Some(i);
        } else if idx != 0 && idx == boneIndex {
            slot = Some(i);
            break;
        }
    }

    let slotIdx = match slot {
        Some(s) => s,
        None => match freeSlot {
            Some(s) => {
                match s {
                    0 => ctx.entity_mut(ent).s.boneIndex1 = boneIndex,
                    1 => ctx.entity_mut(ent).s.boneIndex2 = boneIndex,
                    2 => ctx.entity_mut(ent).s.boneIndex3 = boneIndex,
                    _ => ctx.entity_mut(ent).s.boneIndex4 = boneIndex,
                }
                s
            }
            None => {
                // WARNING: E-Web has no free bone indexes
                return;
            }
        },
    };

    // Copy the angles over the vector in the entitystate, so we can use the
    // corresponding index to set the bone angles on the client.
    match slotIdx {
        0 => ctx.entity_mut(ent).s.boneAngles1 = angles,
        1 => ctx.entity_mut(ent).s.boneAngles2 = angles,
        2 => ctx.entity_mut(ent).s.boneAngles3 = angles,
        _ => ctx.entity_mut(ent).s.boneAngles4 = angles,
    }

    // Now set the angles on our server instance if we have one.
    if ctx.entity(ent).ghoul2.is_null() {
        return;
    }

    let flags = BONE_ANGLES_POSTMULT;
    let up = POSITIVE_Y as c_int;
    let right = NEGATIVE_Z as c_int;
    let forward = NEGATIVE_X as c_int;

    // first 3 bits is forward, second 3 bits is right, third 3 bits is up
    ctx.entity_mut(ent).s.boneOrient = forward | (right << 3) | (up << 6);

    // FLAG: `bone` is a raw `char*`; `cstr_to_str` deref stays unsafe.
    let boneName = unsafe { cstr_to_str(bone as *const c_char) };
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ctx.entity(ent).ghoul2,
        0,
        &boneName,
        &angles as *const vec3_t,
        flags,
        up,
        right,
        forward,
        core::ptr::null_mut(),
        100,
        ctx.world.level.time,
    );
}

/// Raven `EWeb_SetBoneAnim`.
///
/// Source: `oracle/codemp/game/g_items.c:1601-1620`
pub fn EWeb_SetBoneAnim(ctx: &mut GameContext, eweb: EntityId, startFrame: c_int, endFrame: c_int) {
    // set info on the entity so it knows to start the anim on the client next snapshot.
    ctx.entity_mut(eweb).s.eFlags |= EF_G2ANIMATING;

    if ctx.entity(eweb).s.torsoAnim == startFrame && ctx.entity(eweb).s.legsAnim == endFrame {
        // already playing this anim, let's flag it to restart
        let torsoFlip = if ctx.entity(eweb).s.torsoFlip == 0 {
            1
        } else {
            0
        };
        ctx.entity_mut(eweb).s.torsoFlip = torsoFlip;
    } else {
        ctx.entity_mut(eweb).s.torsoAnim = startFrame;
        ctx.entity_mut(eweb).s.legsAnim = endFrame;
    }

    // Raven `ghoul2/G2.h:22-25` bone-anim flags resolve via the canonical
    // `mp_qshared::common::mp::ghoul2::bone_flags` module (crate prelude glob).

    // now set the animation on the server ghoul2 instance.
    debug_assert!(!ctx.entity(eweb).ghoul2.is_null());
    trap::G2API_SetBoneAnim(
        ctx.engine,
        ctx.entity(eweb).ghoul2,
        0,
        "model_root",
        startFrame,
        endFrame,
        BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
        1.0,
        ctx.world.level.time,
        -1.0,
        100,
    );
}

/// Raven `EWebFire`.
///
/// Source: `oracle/codemp/game/g_items.c:1624-1664`
pub fn EWebFire(ctx: &mut GameContext, owner: EntityId, eweb: EntityId) {
    // Raven `#define EWEB_MISSILE_DAMAGE 20` (`g_items.c:1623`).
    pub const EWEB_MISSILE_DAMAGE: c_int = 20;
    // Raven `DAMAGE_DEATH_KNOCKBACK` == 0x80, canonical in `crate::level::damage_flags`.
    // The former local value 0x08 was wrong (that bit is `DAMAGE_NO_PROTECTION`),
    // so the e-web missile's dflags were set incorrectly.
    // Source: `oracle/codemp/game/g_local.h:1178`
    use crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;

    if ctx.entity(eweb).genericValue10 == -1 {
        // oh no
        debug_assert!(false, "Bad e-web bolt");
        return;
    }

    // get the muzzle point
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ctx.entity(eweb).ghoul2,
            0,
            ctx.entity(eweb).genericValue10,
            &mut boltMatrix as *mut mdxaBone_t,
            &ctx.entity(eweb).s.apos.trBase as *const vec3_t,
            &ctx.entity(eweb).r.currentOrigin as *const vec3_t,
            ctx.world.level.time,
            core::ptr::null_mut(),
            &ctx.entity(eweb).modelScale as *const vec3_t,
        ),
    );
    let mut p: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    BG_GiveMeVectorFromMatrix(&boltMatrix as *const mdxaBone_t, ORIGIN as c_int, &mut p);
    BG_GiveMeVectorFromMatrix(
        &boltMatrix as *const mdxaBone_t,
        NEGATIVE_Y as c_int,
        &mut d,
    );

    // Start the thing backwards into the bounding box so it can't start inside other solid things
    let bPoint = [p[0] - 16.0 * d[0], p[1] - 16.0 * d[1], p[2] - 16.0 * d[2]];

    // create the missile
    let missile = CreateMissile(ctx, bPoint, d, 1200.0, 10000, owner, false);

    ctx.ent_set(missile, PrefixSet::ClassnameStatic(c"generic_proj"));
    ctx.entity_mut(missile).s.weapon = WP_TURRET as c_int;

    ctx.entity_mut(missile).damage = EWEB_MISSILE_DAMAGE;
    ctx.entity_mut(missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.entity_mut(missile).methodOfDeath = MOD_TURBLAST as c_int;
    ctx.entity_mut(missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // ignore the e-web entity
    let ewebNum = ctx.entity(eweb).s.number;
    ctx.entity_mut(missile).passThroughNum = ewebNum + 1;

    // times it can bounce before it dies
    ctx.entity_mut(missile).bounceCount = 8;

    // play the muzzle flash
    let mut dAng: vec3_t = [0.0; 3];
    vectoangles(d, &mut dAng);
    let fx = G_EffectIndex(ctx, "turret/muzzle_flash.efx");
    G_PlayEffectID(fx, p, dAng);
}

/// Raven `EWebPositionUser`.
///
/// Source: `oracle/codemp/game/g_items.c:1667-1732`
pub fn EWebPositionUser(ctx: &mut GameContext, owner: EntityId, eweb: EntityId) {
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ctx.entity(eweb).ghoul2,
            0,
            ctx.entity(eweb).genericValue9,
            &mut boltMatrix as *mut mdxaBone_t,
            &ctx.entity(eweb).s.apos.trBase as *const vec3_t,
            &ctx.entity(eweb).r.currentOrigin as *const vec3_t,
            ctx.world.level.time,
            core::ptr::null_mut(),
            &ctx.entity(eweb).modelScale as *const vec3_t,
        ),
    );
    let mut p: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    BG_GiveMeVectorFromMatrix(&boltMatrix as *const mdxaBone_t, ORIGIN as c_int, &mut p);
    BG_GiveMeVectorFromMatrix(
        &boltMatrix as *const mdxaBone_t,
        NEGATIVE_X as c_int,
        &mut d,
    );

    p[0] += 32.0 * d[0];
    p[1] += 32.0 * d[1];
    p[2] += 32.0 * d[2];
    p[2] = ctx.entity(eweb).r.currentOrigin[2];

    p[2] += 4.0;

    // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let owner_client = ctx.entity(owner).client;
    let owner_origin = unsafe { (*owner_client).ps.origin };

    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &owner_origin as *const vec3_t,
            &ctx.entity(owner).r.mins as *const vec3_t,
            &ctx.entity(owner).r.maxs as *const vec3_t,
            &p as *const vec3_t,
            ctx.entity(owner).s.number,
            MASK_PLAYERSOLID,
        ),
    );

    if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction == 1.0 {
        // all clear, we can move there
        let mut pDown = p;
        pDown[2] -= 7.0;
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &p as *const vec3_t,
                &ctx.entity(owner).r.mins as *const vec3_t,
                &ctx.entity(owner).r.maxs as *const vec3_t,
                &pDown as *const vec3_t,
                ctx.entity(owner).s.number,
                MASK_PLAYERSOLID,
            ),
        );

        if tr.startsolid == 0 && tr.allsolid == 0 {
            let d2 = [
                owner_origin[0] - tr.endpos[0],
                owner_origin[1] - tr.endpos[1],
                owner_origin[2] - tr.endpos[2],
            ];
            if VectorLength(d2) > 1.0 {
                // we moved, do some animating
                let mut dAng: vec3_t = [0.0; 3];
                let mut aFlags = SETANIM_FLAG_HOLD as c_int;

                vectoangles(d2, &mut dAng);
                let viewYaw = unsafe { (*owner_client).ps.viewangles[YAW] };
                dAng[YAW] = AngleSubtract(viewYaw, dAng[YAW]);
                if dAng[YAW] > 0.0 {
                    if unsafe { (*owner_client).ps.legsAnim } == BOTH_STRAFE_RIGHT1 as c_int {
                        // reset to change direction
                        aFlags |= SETANIM_FLAG_OVERRIDE as c_int;
                    }
                    let cmd_ptr = unsafe { core::ptr::addr_of_mut!((*owner_client).pers.cmd) };
                    G_SetAnim(
                        ctx,
                        owner,
                        cmd_ptr,
                        SETANIM_LEGS as c_int,
                        BOTH_STRAFE_LEFT1 as c_int,
                        aFlags,
                        0,
                    );
                } else {
                    if unsafe { (*owner_client).ps.legsAnim } == BOTH_STRAFE_LEFT1 as c_int {
                        // reset to change direction
                        aFlags |= SETANIM_FLAG_OVERRIDE as c_int;
                    }
                    let cmd_ptr = unsafe { core::ptr::addr_of_mut!((*owner_client).pers.cmd) };
                    G_SetAnim(
                        ctx,
                        owner,
                        cmd_ptr,
                        SETANIM_LEGS as c_int,
                        BOTH_STRAFE_RIGHT1 as c_int,
                        aFlags,
                        0,
                    );
                }
            } else if unsafe { (*owner_client).ps.legsAnim } == BOTH_STRAFE_RIGHT1 as c_int
                || unsafe { (*owner_client).ps.legsAnim } == BOTH_STRAFE_LEFT1 as c_int
            {
                // don't keep animating in place
                unsafe {
                    (*owner_client).ps.legsTimer = 0;
                }
            }

            G_SetOrigin(ctx.entity_mut(owner), tr.endpos);
            unsafe {
                (*owner_client).ps.origin = tr.endpos;
            }
        }
    } else {
        // can't move here.. stop using the thing I guess
        EWebDisattach(ctx, owner, eweb);
    }
}

/// Raven `EWebUpdateBoneAngles`.
///
/// Source: `oracle/codemp/game/g_items.c:1735-1769`
pub fn EWebUpdateBoneAngles(ctx: &mut GameContext, owner: EntityId, eweb: EntityId) {
    let turnCap: f32 = 4.0;

    // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let owner_client = ctx.entity(owner).client;

    let mut yAng: vec3_t = [0.0, 0.0, 0.0];
    let ideal = AngleSubtract(
        unsafe { (*owner_client).ps.viewangles[YAW] },
        ctx.entity(eweb).s.angles[YAW],
    );
    let mut incr = AngleSubtract(ideal, ctx.entity(eweb).angle);

    if incr > turnCap {
        incr = turnCap;
    } else if incr < -turnCap {
        incr = -turnCap;
    }

    ctx.entity_mut(eweb).angle += incr;

    yAng[0] = ctx.entity(eweb).angle;
    EWeb_SetBoneAngles(ctx, eweb, c"cannon_Yrot".as_ptr() as *mut c_char, yAng);

    EWebPositionUser(ctx, owner, eweb);
    if unsafe { (*owner_client).ewebIndex } == 0 {
        // was removed during position function
        return;
    }

    let mut yAng2: vec3_t = [0.0, 0.0, 0.0];
    yAng2[2] = AngleSubtract(
        unsafe { (*owner_client).ps.viewangles[PITCH] },
        ctx.entity(eweb).s.angles[PITCH],
    ) * 0.8;
    EWeb_SetBoneAngles(ctx, eweb, c"cannon_Xrot".as_ptr() as *mut c_char, yAng2);
}

/// Raven `EWebThink`.
///
/// Source: `oracle/codemp/game/g_items.c:1776-1854`
pub fn EWebThink(ctx: &mut GameContext, self_: EntityId) {
    let mut killMe = qfalse;
    let gravity: f32 = 3.0;
    let mass: f32 = 0.09;
    let bounce: f32 = 1.1;

    if ctx.entity(self_).r.ownerNum == ENTITYNUM_NONE {
        killMe = qtrue;
    } else {
        let owner = EntityId(ctx.entity(self_).r.ownerNum as u32);
        // FLAG: owner client pointer (`gclient_t*`); deref stays raw through the
        // copied pointer value (recipe 2/2b — read exactly what Raven derefs, and
        // only after the `inuse`/null short-circuit passes).
        let owner_client = ctx.entity(owner).client;

        if ctx.entity(owner).inuse == 0
            || owner_client.is_null()
            || unsafe { (*owner_client).pers.connected } != CON_CONNECTED as c_int
            || unsafe { (*owner_client).ewebIndex } != ctx.entity(self_).s.number
            || ctx.entity(owner).health < 1
        {
            killMe = qtrue;
        } else if unsafe { (*owner_client).ps.emplacedIndex } != ctx.entity(self_).s.number {
            // just go back to the inventory then
            EWebDisattach(ctx, owner, self_);
            return;
        }

        if killMe == qfalse {
            let mut yaw: f32 = 0.0;

            if BG_EmplacedView(
                unsafe { (*owner_client).ps.viewangles },
                ctx.entity(self_).s.angles,
                &mut yaw as *mut f32,
                ctx.entity(self_).s.origin2[0],
            ) != 0
            {
                unsafe {
                    (*owner_client).ps.viewangles[YAW] = yaw;
                }
            }
            unsafe {
                (*owner_client).ps.weapon = WP_EMPLACED_GUN as c_int;
                (*owner_client).ps.stats[STAT_WEAPONS as usize] = WP_EMPLACED_GUN as c_int;
            }

            if ctx.entity(self_).genericValue8 < ctx.world.level.time {
                // make sure the anim timer is done
                EWebUpdateBoneAngles(ctx, owner, self_);
                if unsafe { (*owner_client).ewebIndex } == 0 {
                    // was removed during position function
                    return;
                }

                if unsafe { (*owner_client).pers.cmd.buttons & BUTTON_ATTACK } != 0 {
                    if ctx.entity(self_).genericValue5 < ctx.world.level.time {
                        // we can fire another shot off
                        EWebFire(ctx, owner, self_);

                        // cheap firing anim
                        EWeb_SetBoneAnim(ctx, self_, 2, 4);
                        ctx.entity_mut(self_).genericValue3 = 1;

                        // set fire debounce time
                        ctx.entity_mut(self_).genericValue5 = ctx.world.level.time + 100;
                    }
                } else if ctx.entity(self_).genericValue5 < ctx.world.level.time
                    && ctx.entity(self_).genericValue3 != 0
                {
                    // reset the anim back to non-firing
                    EWeb_SetBoneAnim(ctx, self_, 0, 1);
                    ctx.entity_mut(self_).genericValue3 = 0;
                }
            }
        }
    }

    if killMe != qfalse {
        // something happened to the owner, let's explode
        EWebDie(
            ctx,
            self_,
            Some(self_),
            Some(self_),
            999,
            MOD_SUICIDE as c_int,
        );
        return;
    }

    // run some physics on it real quick so it falls and stuff properly
    G_RunExPhys(
        ctx,
        self_,
        gravity,
        mass,
        bounce,
        false,
        core::ptr::null_mut(),
        0,
    );

    ctx.entity_mut(self_).nextthink = ctx.world.level.time;
}

/// Raven `EWeb_Create`.
///
/// Source: `oracle/codemp/game/g_items.c:1859-1980`
pub fn EWeb_Create(ctx: &mut GameContext, spawner: EntityId) -> *mut gentity_t {
    // Raven `#define EWEB_HEALTH 200` (`g_items.c:1856`).
    pub const EWEB_HEALTH: c_int = 200;

    let modelName = c"models/map_objects/hoth/eweb_model.glm";
    let failSound = G_SoundIndex(ctx, "sound/interface/shieldcon_empty");

    let mins: vec3_t = [-32.0, -32.0, -24.0];
    let maxs: vec3_t = [32.0, 32.0, 24.0];

    // FLAG: spawner client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let spawner_client = ctx.entity(spawner).client;

    let fAng: vec3_t = [0.0, unsafe { (*spawner_client).ps.viewangles[1] }, 0.0];
    let mut fwd: vec3_t = [0.0; 3];
    AngleVectors(fAng, Some(&mut fwd), None, None);

    let mut s = unsafe { (*spawner_client).ps.origin };
    // allow some fudge
    s[2] += 12.0;

    let pos = [
        s[0] + 48.0 * fwd[0],
        s[1] + 48.0 * fwd[1],
        s[2] + 48.0 * fwd[2],
    ];

    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &s as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &pos as *const vec3_t,
            ctx.entity(spawner).s.number,
            MASK_PLAYERSOLID,
        ),
    );

    if tr.allsolid != 0 || tr.startsolid != 0 || tr.fraction != 1.0 {
        // can't spawn here, we are in solid
        G_Sound(ctx, Some(spawner), CHAN_AUTO, failSound);
        return core::ptr::null_mut();
    }

    let ent = G_Spawn(ctx);

    ctx.entity_mut(ent).clipmask = MASK_PLAYERSOLID;
    ctx.entity_mut(ent).r.contents = MASK_PLAYERSOLID;

    ctx.entity_mut(ent).physicsObject = qtrue;

    // for the sake of being able to differentiate client-side between this and an emplaced gun
    ctx.entity_mut(ent).s.weapon = WP_NONE as c_int;

    let mut downPos = pos;
    downPos[2] -= 18.0;
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &pos as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &downPos as *const vec3_t,
            ctx.entity(spawner).s.number,
            MASK_PLAYERSOLID,
        ),
    );

    if tr.startsolid != 0
        || tr.allsolid != 0
        || tr.fraction == 1.0
        || (tr.entityNum as c_int) < ENTITYNUM_WORLD
    {
        // didn't hit ground.
        G_FreeEntity(ctx, Some(ent));
        G_Sound(ctx, Some(spawner), CHAN_AUTO, failSound);
        return core::ptr::null_mut();
    }

    let pos = tr.endpos;

    G_SetOrigin(ctx.entity_mut(ent), pos);

    ctx.entity_mut(ent).s.apos.trBase = fAng;
    ctx.entity_mut(ent).r.currentAngles = fAng;

    ctx.entity_mut(ent).s.owner = ctx.entity(spawner).s.number;
    let sessionTeam = unsafe { (*spawner_client).sess.sessionTeam };
    ctx.entity_mut(ent).s.teamowner = sessionTeam;

    ctx.entity_mut(ent).takedamage = qtrue;

    if unsafe { (*spawner_client).ewebHealth } <= 0 {
        // refresh the owner's e-web health if its last e-web did not exist or was killed
        unsafe {
            (*spawner_client).ewebHealth = EWEB_HEALTH;
        }
    }

    // resume health of last deployment
    ctx.entity_mut(ent).maxHealth = EWEB_HEALTH;
    let ewebHealth = unsafe { (*spawner_client).ewebHealth };
    ctx.entity_mut(ent).health = ewebHealth;
    G_ScaleNetHealth(ctx.entity_mut(ent));

    ctx.entity_mut(ent).die = Some(EntDie::EWebDie).into();
    ctx.entity_mut(ent).pain = Some(EntPain::EWebPain).into();

    ctx.entity_mut(ent).think = Some(EntThink::EWebThink).into();
    ctx.entity_mut(ent).nextthink = ctx.world.level.time;

    // set up the g2 model info
    ctx.entity_mut(ent).s.modelGhoul2 = 1;
    ctx.entity_mut(ent).s.g2radius = 128;
    ctx.entity_mut(ent).s.modelindex = G_ModelIndex(ctx, modelName.to_str().unwrap());

    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut ctx.entity_mut(ent).ghoul2 as *mut *mut c_void,
        modelName.to_str().unwrap(),
        0,
        0,
        0,
        0,
        0,
    );

    if ctx.entity(ent).ghoul2.is_null() {
        // should not happen, but just to be safe.
        G_FreeEntity(ctx, Some(ent));
        return core::ptr::null_mut();
    }

    // initialize bone angles (Raven `vec3_origin` — now resolved via the
    // crate prelude, pass-3 symbol backfill).
    EWeb_SetBoneAngles(
        ctx,
        ent,
        c"cannon_Yrot".as_ptr() as *mut c_char,
        vec3_origin,
    );
    EWeb_SetBoneAngles(
        ctx,
        ent,
        c"cannon_Xrot".as_ptr() as *mut c_char,
        vec3_origin,
    );

    let genericValue10 =
        trap::G2API_AddBolt(ctx.engine, ctx.entity(ent).ghoul2, 0, "*cannonflash"); // muzzle bolt
    ctx.entity_mut(ent).genericValue10 = genericValue10;
    let genericValue9 = trap::G2API_AddBolt(ctx.engine, ctx.entity(ent).ghoul2, 0, "cannon_Yrot"); // for placing the owner relative to rotation
    ctx.entity_mut(ent).genericValue9 = genericValue9;

    // set the constraints for this guy as an emplaced weapon, and his constraint angles
    ctx.entity_mut(ent).s.origin2[0] = 360.0; // 360 degrees in either direction

    ctx.entity_mut(ent).s.angles = fAng; // consider "angle 0" for constraint

    // angle of y rot bone
    ctx.entity_mut(ent).angle = 0.0;

    ctx.entity_mut(ent).r.ownerNum = ctx.entity(spawner).s.number;
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    );

    // store off the owner's current weapons, we will be forcing him to use the "emplaced" weapon
    let stats_weapons = unsafe { (*spawner_client).ps.stats[STAT_WEAPONS as usize] };
    ctx.entity_mut(ent).genericValue11 = stats_weapons;

    // start the "unfolding" anim
    EWeb_SetBoneAnim(ctx, ent, 4, 20);
    // don't allow use until the anim is done playing (rough time estimate)
    ctx.entity_mut(ent).genericValue8 = ctx.world.level.time + 500;

    ctx.entity_mut(ent).r.mins = mins;
    ctx.entity_mut(ent).r.maxs = maxs;

    core::ptr::from_mut(ctx.entity_mut(ent))
}

/// Raven `ItemUse_UseEWeb`.
///
/// Source: `oracle/codemp/game/g_items.c:1984-2018`
pub fn ItemUse_UseEWeb(ctx: &mut GameContext, ent: EntityId) {
    // Raven `#define EWEB_USE_DEBOUNCE 1000` (`g_items.c:1982`).
    pub const EWEB_USE_DEBOUNCE: c_int = 1000;

    // FLAG: player client pointer (`gclient_t*`); deref stays raw through the
    // copied pointer value (recipe 2/2b — read exactly what Raven derefs).
    let ent_client = ctx.entity(ent).client;
    if unsafe { (*ent_client).ewebTime } > ctx.world.level.time {
        // can't use again yet
        return;
    }

    if unsafe {
        (*ent_client).ps.weaponTime > 0
            || (*ent_client).ps.forceHandExtend != HANDEXTEND_NONE as c_int
    } {
        // busy doing something else
        return;
    }

    if unsafe { (*ent_client).ps.emplacedIndex != 0 && (*ent_client).ewebIndex == 0 } {
        // using an emplaced gun already that isn't our own e-web
        return;
    }

    if unsafe { (*ent_client).ewebIndex } != 0 {
        // put it away
        let eweb = EntityId(unsafe { (*ent_client).ewebIndex } as u32);
        EWebDisattach(ctx, ent, eweb);
    } else {
        // create it
        let eweb_ptr = EWeb_Create(ctx, ent);

        if !eweb_ptr.is_null() {
            // if it's null the thing couldn't spawn (probably no room)
            let eweb = ctx.entity_id_of(eweb_ptr).unwrap();
            let number = ctx.entity(eweb).s.number;
            unsafe {
                (*ent_client).ewebIndex = number;
                (*ent_client).ps.emplacedIndex = number;
            }
        }
    }

    unsafe {
        (*ent_client).ewebTime = ctx.world.level.time + EWEB_USE_DEBOUNCE;
    }
}

/// Raven `Pickup_Powerup`.
///
/// Source: `oracle/codemp/game/g_items.c:2024-2100`
pub fn Pickup_Powerup(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    // Raven `#define RESPAWN_POWERUP 120` (`g_items.c:27`).
    pub const RESPAWN_POWERUP: c_int = 120;
    // Raven `PLAYEREVENT_DENIEDREWARD` (`bg_public.h:716`) — not yet ported; transcribed locally.
    pub const PLAYEREVENT_DENIEDREWARD: c_int = 0x0001;

    let it = ctx.entity(ent).item.unwrap().item();
    // Only the Touch_Item dispatch reaches here, always with a powerup item.
    let ItemKind::Powerup(tag) = it.kind else {
        unreachable!("Pickup_Powerup on non-powerup item {}", it.classname);
    };
    // FLAG: picker-upper client pointer (`gclient_t*`); the toucher may be an NPC
    // with a pool client, so deref stays raw (recipe 2b).
    let other_client = ctx.entity(other).client;
    if unsafe { (*other_client).ps.powerups[tag as usize] } == 0 {
        // round timing to seconds to make multiple powerup timers
        // count in sync
        unsafe {
            (*other_client).ps.powerups[tag as usize] =
                ctx.world.level.time - (ctx.world.level.time % 1000);
        }

        let number = ctx.entity(other).s.number;
        G_LogWeaponPowerup(ctx, number, tag);
    }

    let quantity = if ctx.entity(ent).count != 0 {
        ctx.entity(ent).count
    } else {
        it.quantity
    };

    unsafe {
        (*other_client).ps.powerups[tag as usize] += quantity * 1000;
    }

    if tag == PW_YSALAMIRI as c_int {
        unsafe {
            (*other_client).ps.powerups[PW_FORCE_ENLIGHTENED_LIGHT as usize] = 0;
            (*other_client).ps.powerups[PW_FORCE_ENLIGHTENED_DARK as usize] = 0;
            (*other_client).ps.powerups[PW_FORCE_BOON as usize] = 0;
        }
    }

    // give any nearby players a "denied" anti-reward
    for i in 0..ctx.world.level.maxclients {
        // `i` is a real `level.clients` slot (i < maxclients), safe to index.
        let client_ptr = core::ptr::addr_of!(ctx.world.clients[i as usize]);
        if client_ptr == other_client as *const gclient_t {
            continue;
        }
        if ctx.world.clients[i as usize].pers.connected == CON_DISCONNECTED {
            continue;
        }
        if ctx.world.clients[i as usize].ps.stats[STAT_HEALTH as usize] <= 0 {
            continue;
        }

        // if same team in team game, no sound
        // cannot use OnSameTeam as it expects to g_entities, not clients
        if ctx.world.cvars.g_gametype.integer >= GT_TEAM
            && unsafe { (*other_client).sess.sessionTeam }
                == ctx.world.clients[i as usize].sess.sessionTeam
        {
            continue;
        }

        // if too far away, no sound
        let mut delta = [
            ctx.entity(ent).s.pos.trBase[0] - ctx.world.clients[i as usize].ps.origin[0],
            ctx.entity(ent).s.pos.trBase[1] - ctx.world.clients[i as usize].ps.origin[1],
            ctx.entity(ent).s.pos.trBase[2] - ctx.world.clients[i as usize].ps.origin[2],
        ];
        let len = VectorNormalize(&mut delta);
        if len > 192.0 {
            continue;
        }

        // if not facing, no sound
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors(
            ctx.world.clients[i as usize].ps.viewangles,
            Some(&mut forward),
            None,
            None,
        );
        let dot = delta[0] * forward[0] + delta[1] * forward[1] + delta[2] * forward[2];
        if dot < 0.4 {
            continue;
        }

        // if not line of sight, no sound
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &ctx.world.clients[i as usize].ps.origin as *const vec3_t,
                core::ptr::null(),
                core::ptr::null(),
                &ctx.entity(ent).s.pos.trBase as *const vec3_t,
                ENTITYNUM_NONE,
                CONTENTS_SOLID,
            ),
        );
        if tr.fraction != 1.0 {
            continue;
        }

        // anti-reward
        ctx.world.clients[i as usize].ps.persistant[PERS_PLAYEREVENTS as usize] ^=
            PLAYEREVENT_DENIEDREWARD;
    }
    RESPAWN_POWERUP
}

/// Raven `Pickup_Holdable`.
///
/// Source: `oracle/codemp/game/g_items.c:2104-2113`
pub fn Pickup_Holdable(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    let item = ctx.entity(ent).item.unwrap();
    let it = item.item();
    // Only the Touch_Item dispatch reaches here, always with a holdable item.
    let ItemKind::Holdable(tag) = it.kind else {
        unreachable!("Pickup_Holdable on non-holdable item {}", it.classname);
    };
    // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool client,
    // so deref stays raw (recipe 2b).
    let other_client = ctx.entity(other).client;
    unsafe {
        (*other_client).ps.stats[statIndex_t::STAT_HOLDABLE_ITEM as usize] = item.modelindex();

        (*other_client).ps.stats[statIndex_t::STAT_HOLDABLE_ITEMS as usize] |= 1 << tag;
    }

    let number = ctx.entity(other).s.number;
    G_LogWeaponItem(ctx, number, tag);

    adjustRespawnTime(ctx, RESPAWN_HOLDABLE, it.kind)
}

/// Raven `Add_Ammo`.
///
/// Source: `oracle/codemp/game/g_items.c:2118-2128`
pub fn Add_Ammo(ctx: &mut GameContext, ent: EntityId, weapon: c_int, count: c_int) {
    // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool client,
    // so deref stays raw (recipe 2b).
    let cl = ctx.entity(ent).client;
    unsafe {
        if (*cl).ps.ammo[weapon as usize] < ammoData[weapon as usize].max {
            (*cl).ps.ammo[weapon as usize] += count;
            if (*cl).ps.ammo[weapon as usize] > ammoData[weapon as usize].max {
                (*cl).ps.ammo[weapon as usize] = ammoData[weapon as usize].max;
            }
        }
    }
}

/// Raven `Pickup_Ammo`.
///
/// Source: `oracle/codemp/game/g_items.c:2130-2175`
pub fn Pickup_Ammo(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    // Raven `#define RESPAWN_AMMO 40` (`g_items.c:26`).
    const RESPAWN_AMMO: f32 = 40.0;

    // below take `other` as a handle.
    let it = ctx.entity(ent).item.unwrap().item();
    // Only the Touch_Item dispatch reaches here, always with an ammo item.
    let ItemKind::Ammo(tag) = it.kind else {
        unreachable!("Pickup_Ammo on non-ammo item {}", it.classname);
    };
    let quantity = if ctx.entity(ent).count != 0 {
        ctx.entity(ent).count
    } else {
        it.quantity
    };

    if tag == -1 {
        // an ammo_all, give them a bit of everything
        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool
            // client, so deref stays raw (recipe 2b).
            let other_client = ctx.entity(other).client;
            // complaints that siege tech's not giving enough ammo.  Does anything else use ammo all?
            Add_Ammo(ctx, other, AMMO_BLASTER as c_int, 100);
            Add_Ammo(ctx, other, AMMO_POWERCELL as c_int, 100);
            Add_Ammo(ctx, other, AMMO_METAL_BOLTS as c_int, 100);
            Add_Ammo(ctx, other, AMMO_ROCKETS as c_int, 5);
            if (unsafe { (*other_client).ps.stats[STAT_WEAPONS as usize] }
                & (1 << WP_DET_PACK as c_int))
                != 0
            {
                Add_Ammo(ctx, other, AMMO_DETPACK as c_int, 2);
            }
            if (unsafe { (*other_client).ps.stats[STAT_WEAPONS as usize] }
                & (1 << WP_THERMAL as c_int))
                != 0
            {
                Add_Ammo(ctx, other, AMMO_THERMAL as c_int, 2);
            }
            if (unsafe { (*other_client).ps.stats[STAT_WEAPONS as usize] }
                & (1 << WP_TRIP_MINE as c_int))
                != 0
            {
                Add_Ammo(ctx, other, AMMO_TRIPMINE as c_int, 2);
            }
        } else {
            Add_Ammo(ctx, other, AMMO_BLASTER as c_int, 50);
            Add_Ammo(ctx, other, AMMO_POWERCELL as c_int, 50);
            Add_Ammo(ctx, other, AMMO_METAL_BOLTS as c_int, 50);
            Add_Ammo(ctx, other, AMMO_ROCKETS as c_int, 2);
        }
    } else {
        Add_Ammo(ctx, other, tag, quantity);
    }

    adjustRespawnTime(ctx, RESPAWN_AMMO, it.kind)
}

/// Raven `Pickup_Weapon`.
///
/// Source: `oracle/codemp/game/g_items.c:2180-2232`
pub fn Pickup_Weapon(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    let it = ctx.entity(ent).item.unwrap().item();
    // Only the Touch_Item dispatch reaches here, always with a weapon item.
    let ItemKind::Weapon(weapon) = it.kind else {
        unreachable!("Pickup_Weapon on non-weapon item {}", it.classname);
    };
    // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool client,
    // so deref stays raw (recipe 2b).
    let other_client = ctx.entity(other).client;
    let mut quantity: c_int;

    if ctx.entity(ent).count < 0 {
        quantity = 0; // None for you, sir!
    } else {
        quantity = if ctx.entity(ent).count != 0 {
            ctx.entity(ent).count
        } else {
            it.quantity
        };

        // dropped items and teamplay weapons always have full ammo
        if (ctx.entity(ent).flags & FL_DROPPED_ITEM) == 0
            && ctx.world.cvars.g_gametype.integer != GT_TEAM
        {
            // respawning rules

            // New method:  If the player has less than half the minimum, give them the minimum, else add 1/2 the min.

            // drop the quantity if the already have over the minimum
            if (unsafe { (*other_client).ps.ammo[weapon as usize] } as f32) < quantity as f32 * 0.5
            {
                quantity -= unsafe { (*other_client).ps.ammo[weapon as usize] };
            } else {
                quantity = (quantity as f32 * 0.5) as c_int; // only add half the value.
            }
        }
    }

    // add the weapon
    unsafe {
        (*other_client).ps.stats[STAT_WEAPONS as usize] |= 1 << weapon;
    }

    Add_Ammo(ctx, other, weaponData[weapon as usize].ammoIndex, quantity);

    let number = ctx.entity(other).s.number;
    G_LogWeaponPickup(ctx, number, weapon);

    // team deathmatch has slow weapon respawns
    if ctx.world.cvars.g_gametype.integer == GT_TEAM {
        return adjustRespawnTime(ctx, RESPAWN_TEAM_WEAPON, it.kind);
    }

    let weapon_respawn = ctx.world.cvars.g_weaponRespawn.integer as f32;
    adjustRespawnTime(ctx, weapon_respawn, it.kind)
}

/// Raven `Pickup_Health`.
///
/// Source: `oracle/codemp/game/g_items.c:2237-2266`
pub fn Pickup_Health(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    // Raven local `#define`s (`g_items.c:23-27`).
    pub const RESPAWN_HEALTH: f32 = 30.0;
    pub const RESPAWN_MEGAHEALTH: c_int = 120;

    let it = ctx.entity(ent).item.unwrap().item();
    // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool client,
    // so deref stays raw (recipe 2b).
    let other_client = ctx.entity(other).client;

    // small and mega healths will go over the max
    let maxHealth = unsafe { (*other_client).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] };
    let max = if it.quantity != 5 && it.quantity != 100 {
        maxHealth
    } else {
        maxHealth * 2
    };

    let quantity = if ctx.entity(ent).count != 0 {
        ctx.entity(ent).count
    } else {
        it.quantity
    };

    ctx.entity_mut(other).health += quantity;

    if ctx.entity(other).health > max {
        ctx.entity_mut(other).health = max;
    }
    let health = ctx.entity(other).health;
    unsafe {
        (*other_client).ps.stats[statIndex_t::STAT_HEALTH as usize] = health;
    }

    if it.quantity == 100 {
        // mega health respawns slow
        return RESPAWN_MEGAHEALTH;
    }

    adjustRespawnTime(ctx, RESPAWN_HEALTH, it.kind)
}

/// Raven `Pickup_Armor`.
///
/// Source: `oracle/codemp/game/g_items.c:2270-2279`
pub fn Pickup_Armor(ctx: &mut GameContext, ent: EntityId, other: EntityId) -> c_int {
    // Raven local `#define` (`g_items.c:21`).
    pub const RESPAWN_ARMOR: f32 = 20.0;

    let it = ctx.entity(ent).item.unwrap().item();
    // Only the Touch_Item dispatch reaches here, always with an armor item.
    let ItemKind::Armor { rating } = it.kind else {
        unreachable!("Pickup_Armor on non-armor item {}", it.classname);
    };
    // FLAG: picker-upper client pointer (`gclient_t*`); may be an NPC pool client,
    // so deref stays raw (recipe 2b).
    let cl = ctx.entity(other).client;

    unsafe {
        (*cl).ps.stats[statIndex_t::STAT_ARMOR as usize] += it.quantity;
        // Raven caps armor at maxhealth * giTag (the shield's 1|2 rating).
        let cap = (*cl).ps.stats[statIndex_t::STAT_MAX_HEALTH as usize] * rating;
        if (*cl).ps.stats[statIndex_t::STAT_ARMOR as usize] > cap {
            (*cl).ps.stats[statIndex_t::STAT_ARMOR as usize] = cap;
        }
    }

    adjustRespawnTime(ctx, RESPAWN_ARMOR, it.kind)
}

/// Raven `RespawnItem`.
///
/// Source: `oracle/codemp/game/g_items.c:2288-2334`
pub fn RespawnItem(ctx: &mut GameContext, ent: EntityId) {
    let mut ent = ent;
    // randomly select from teamed entities
    if ctx.entity(ent).team.is_some() {
        if ctx.entity(ent).teammaster.is_none() {
            G_Error(ctx, "RespawnItem: bad teammaster");
        }
        let master = ctx.entity(ent).teammaster;

        let mut count = 0;
        let mut e = master;
        while let Some(eid) = e {
            e = ctx.entity(eid).teamchain;
            count += 1;
        }

        let choice = ctx.world.bg_state.rng.rand() % count;

        let mut i = 0;
        let mut e = master;
        while i < choice {
            e = ctx.entity(e.unwrap()).teamchain;
            i += 1;
        }
        ent = e.unwrap();
    }

    let e = ctx.entity_mut(ent);
    e.r.contents = CONTENTS_TRIGGER;
    //ent->s.eFlags &= ~EF_NODRAW;
    e.s.eFlags &= !(EF_NODRAW | EF_ITEMPLACEHOLDER);
    e.r.svFlags &= !SVF_NOCLIENT;
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    );

    let it = ctx.entity(ent).item.unwrap().item();
    if matches!(it.kind, ItemKind::Powerup(_)) {
        // play powerup spawn sound to all clients
        let trBase = ctx.entity(ent).s.pos.trBase;

        // if the powerup respawn sound should Not be global
        let te_ptr = if ctx.entity(ent).speed != 0.0 {
            G_TempEntity(ctx, trBase, entity_event_t::EV_GENERAL_SOUND as c_int)
        } else {
            G_TempEntity(ctx, trBase, entity_event_t::EV_GLOBAL_SOUND as c_int)
        };
        let te = te_ptr;
        ctx.entity_mut(te).s.eventParm = G_SoundIndex(ctx, "sound/items/respawn1");
        ctx.entity_mut(te).r.svFlags |= SVF_BROADCAST;
    }

    // play the normal respawn sound only to nearby clients
    G_AddEvent(
        ctx.entity_mut(ent),
        entity_event_t::EV_ITEM_RESPAWN as c_int,
        0,
    );

    ctx.entity_mut(ent).nextthink = 0;
}

/// Raven `CheckItemCanBePickedUpByNPC`.
///
/// Source: `oracle/codemp/game/g_items.c:2336-2355`
pub fn CheckItemCanBePickedUpByNPC(
    ctx: &mut GameContext,
    item: EntityId,
    pickerupper: EntityId,
) -> qboolean {
    // Raven `SCF_FORCED_MARCH` (`b_public.h:43`), canonical in
    // `crate::npc::script_flags`. Source: `oracle/codemp/game/b_public.h:43`
    use crate::npc::script_flags::SCF_FORCED_MARCH;

    // `item` is the item *entity* (not a `gitem_t`). FLAG: `pickerupper.NPC`
    // (`gNPC_t*`) has no accessor; deref stays raw (recipe 2c).
    let npc = ctx.entity(pickerupper).NPC;
    // Raven's `resolve(g_entities, item->activator) != &g_entities[0]` is exactly
    // "activator is not entity 0" — expressed directly on the handle.
    if (ctx.entity(item).flags & FL_DROPPED_ITEM) != 0
        && ctx.entity(item).activator != Some(EntityId(0))
        && ctx.entity(pickerupper).s.number != 0
        && ctx.entity(pickerupper).s.weapon == WP_NONE as c_int
        && ctx.entity(pickerupper).enemy.is_some()
        && ctx.entity(pickerupper).painDebounceTime < ctx.world.level.time
        && !npc.is_null()
        && unsafe { (*npc).surrenderTime } < ctx.world.level.time // not surrendering
        && (unsafe { (*npc).scriptFlags } & SCF_FORCED_MARCH) == 0
    /*&& item->item->giTag != INV_SECURITY_KEY*/
    {
        // non-player, in combat, picking up a dropped item that does NOT belong to the player and it *not* a security key
        if ctx.world.level.time - ctx.entity(item).s.time < 3000 {
            // was 5000
            return qfalse;
        }
        return qtrue;
    }
    qfalse
}

/// Raven `Touch_Item`.
///
/// Source: `oracle/codemp/game/g_items.c:2362-2646`
pub fn Touch_Item(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // `ent` is the item entity (never NULL); `other` is the toucher. Raven hard-
    // derefs `other` past the first guard, so the touch dispatch always passes a
    // live entity — resolve it once here.
    if ctx.entity(ent).genericValue10 > ctx.world.level.time
        && other.is_some_and(|o| ctx.entity(o).s.number == ctx.entity(ent).genericValue11)
    {
        // this is the ent that we don't want to be able to touch us for x seconds
        return;
    }

    if (ctx.entity(ent).s.eFlags & EF_ITEMPLACEHOLDER) != 0 {
        return;
    }

    if (ctx.entity(ent).s.eFlags & EF_NODRAW) != 0 {
        return;
    }

    let it = ctx.entity(ent).item.unwrap().item();

    if matches!(it.kind, ItemKind::Weapon(_))
        && ctx.entity(ent).s.powerups != 0
        && ctx.entity(ent).s.powerups < ctx.world.level.time
    {
        ctx.entity_mut(ent).s.generic1 = 0;
        ctx.entity_mut(ent).s.powerups = 0;
    }

    let other = other.unwrap();
    // FLAG: toucher client pointer (`gclient_t*`); may be an NPC pool client, so
    // deref stays raw (recipe 2b).
    if ctx.entity(other).client.is_null() {
        return;
    }
    let other_client = ctx.entity(other).client;
    if ctx.entity(other).health < 1 {
        return; // dead people can't pickup
    }

    if let ItemKind::Powerup(tag @ (PW_FORCE_ENLIGHTENED_LIGHT | PW_FORCE_ENLIGHTENED_DARK)) =
        it.kind
    {
        if tag == PW_FORCE_ENLIGHTENED_LIGHT {
            if unsafe { (*other_client).ps.fd.forceSide } != FORCE_LIGHTSIDE as c_int {
                return;
            }
        } else if unsafe { (*other_client).ps.fd.forceSide } != FORCE_DARKSIDE as c_int {
            return;
        }
    }

    // the same pickup rules are used for client side and server side
    if BG_CanItemBeGrabbed(
        ctx.world.cvars.g_gametype.integer,
        &ctx.entity(ent).s as *const entityState_t,
        unsafe { &(*other_client).ps as *const playerState_t },
    ) == 0
    {
        return;
    }

    let npc_class = unsafe { (*other_client).NPC_class };
    if npc_class == CLASS_ATST
        || npc_class == CLASS_GONK
        || npc_class == CLASS_MARK1
        || npc_class == CLASS_MARK2
        || npc_class == CLASS_MOUSE
        || npc_class == CLASS_PROBE
        || npc_class == CLASS_PROTOCOL
        || npc_class == CLASS_R2D2
        || npc_class == CLASS_R5D2
        || npc_class == CLASS_SEEKER
        || npc_class == CLASS_REMOTE
        || npc_class == CLASS_RANCOR
        || npc_class == CLASS_WAMPA
        //|| npc_class == CLASS_JAWA //FIXME: in some cases it's okay?
        || npc_class == CLASS_UGNAUGHT //FIXME: in some cases it's okay?
        || npc_class == CLASS_SENTRY
    {
        //FIXME: some flag would be better
        // droids can't pick up items/weapons!
        return;
    }

    if CheckItemCanBePickedUpByNPC(ctx, ent, other) != 0 {
        // FLAG: `other.NPC` (`gNPC_t*`) has no accessor; deref stays raw (recipe 2c).
        let npc = ctx.entity(other).NPC;
        if !npc.is_null() {
            if let Some(goal_id) = unsafe { (*npc).goalEntity } {
                if ctx.entity(goal_id).enemy == Some(ent) {
                    // they were running to pick me up, they did, so clear goal
                    unsafe {
                        (*npc).goalEntity = None;
                        (*npc).squadState = SQUAD_STAND_AND_SHOOT;
                    }
                }
            }
        }
    } else if (ctx.entity(ent).spawnflags & ITMSF_ALLOWNPC) == 0 {
        // NPCs cannot pick it up
        if ctx.entity(other).s.eType == ET_NPC as c_int {
            // Not the player?
            let mut dontGo = qfalse;
            // FLAG: `other.m_pVehicle` (`Vehicle_t*`)/`m_pVehicleInfo` derefs stay
            // raw (recipe 2b — vehicle structs have no accessor).
            let veh = ctx.entity(other).m_pVehicle;
            if it.kind == ItemKind::Ammo(-1)
                && ctx.entity(other).s.NPC_class == CLASS_VEHICLE as c_int
                && !veh.is_null()
                && unsafe { (*(*veh).m_pVehicleInfo).r#type } == VH_WALKER
            {
                // yeah, uh, atst gets healed by these things
                if ctx.entity(other).maxHealth != 0
                    && ctx.entity(other).health < ctx.entity(other).maxHealth
                {
                    ctx.entity_mut(other).health += 80;
                    let maxHealth = ctx.entity(other).maxHealth;
                    if ctx.entity(other).health > maxHealth {
                        ctx.entity_mut(other).health = maxHealth;
                    }
                    G_ScaleNetHealth(ctx.entity_mut(other));
                    dontGo = qtrue;
                }
            }

            if dontGo == qfalse {
                return;
            }
        }
    }

    let other_number = ctx.entity(other).s.number;
    let logmsg = format!("Item: {} {}\n", other_number, it.classname);
    G_LogPrintf(ctx, &logmsg);

    let mut predict = unsafe { (*other_client).pers.predictItemPickup } != 0;

    // call the item-specific pickup function
    let mut respawn = match it.kind {
        ItemKind::Weapon(_) => {
            predict = true;
            Pickup_Weapon(ctx, ent, other)
        }
        ItemKind::Ammo(tag) => {
            let respawn = Pickup_Ammo(ctx, ent, other);
            if tag == AMMO_THERMAL as c_int
                || tag == AMMO_TRIPMINE as c_int
                || tag == AMMO_DETPACK as c_int
            {
                let weapForAmmo = if tag == AMMO_THERMAL as c_int {
                    WP_THERMAL as c_int
                } else if tag == AMMO_TRIPMINE as c_int {
                    WP_TRIP_MINE as c_int
                } else {
                    WP_DET_PACK as c_int
                };

                if !other_client.is_null()
                    && unsafe {
                        (*other_client).ps.ammo[weaponData[weapForAmmo as usize].ammoIndex as usize]
                    } > 0
                {
                    unsafe {
                        (*other_client).ps.stats[STAT_WEAPONS as usize] |= 1 << weapForAmmo;
                    }
                }
            }
            predict = true;
            respawn
        }
        ItemKind::Armor { .. } => {
            predict = true;
            Pickup_Armor(ctx, ent, other)
        }
        ItemKind::Health => {
            predict = true;
            Pickup_Health(ctx, ent, other)
        }
        ItemKind::Powerup(_) => {
            predict = false;
            Pickup_Powerup(ctx, ent, other)
        }
        ItemKind::Team(_) => Pickup_Team(ctx, ent, other),
        ItemKind::Holdable(_) => Pickup_Holdable(ctx, ent, other),
        ItemKind::Bad => return,
    };

    if respawn == 0 {
        return;
    }

    let ent_number = ctx.entity(ent).s.number;

    // play the normal pickup sound
    if predict {
        if !other_client.is_null() {
            BG_AddPredictableEventToPlayerstate(
                entity_event_t::EV_ITEM_PICKUP as c_int,
                ent_number,
                unsafe { core::ptr::addr_of_mut!((*other_client).ps) },
            );
        } else {
            G_AddPredictableEvent(
                Some(ctx.entity_mut(other)),
                entity_event_t::EV_ITEM_PICKUP as c_int,
                ent_number,
            );
        }
    } else {
        G_AddEvent(
            ctx.entity_mut(other),
            entity_event_t::EV_ITEM_PICKUP as c_int,
            ent_number,
        );
    }

    // powerup pickups are global broadcasts
    // Raven: `/*ent->item->giType == IT_POWERUP ||*/ ent->item->giType == IT_TEAM`.
    if matches!(it.kind, ItemKind::Team(_)) {
        // if we want the global sound to play
        let trBase = ctx.entity(ent).s.pos.trBase;
        if ctx.entity(ent).speed == 0.0 {
            let te = G_TempEntity(ctx, trBase, entity_event_t::EV_GLOBAL_ITEM_PICKUP as c_int);
            ctx.entity_mut(te).s.eventParm = ctx.entity(ent).s.modelindex;
            ctx.entity_mut(te).r.svFlags |= SVF_BROADCAST;
        } else {
            let te = G_TempEntity(ctx, trBase, entity_event_t::EV_GLOBAL_ITEM_PICKUP as c_int);
            ctx.entity_mut(te).s.eventParm = ctx.entity(ent).s.modelindex;
            // only send this temp entity to a single client
            ctx.entity_mut(te).r.svFlags |= SVF_SINGLECLIENT;
            ctx.entity_mut(te).r.singleClient = ctx.entity(other).s.number;
        }
    }

    // fire item targets
    G_UseTargets(ctx, Some(ent), Some(other));

    // wait of -1 will not respawn
    if ctx.entity(ent).wait == -1.0 {
        let e = ctx.entity_mut(ent);
        e.r.svFlags |= SVF_NOCLIENT;
        e.s.eFlags |= EF_NODRAW;
        e.r.contents = 0;
        e.unlinkAfterEvent = qtrue;
        return;
    }

    // non zero wait overrides respawn time
    if ctx.entity(ent).wait != 0.0 {
        respawn = ctx.entity(ent).wait as c_int;
    }

    // random can be used to vary the respawn time
    if ctx.entity(ent).random != 0.0 {
        // C `respawn += crandom() * ent->random`: `crandom()` is `double`, so
        // the sum is computed in `double` and truncated once into `int respawn`.
        let random = ctx.entity(ent).random as f64;
        respawn = (respawn as f64 + ctx.world.bg_state.rng.crandom() * random) as c_int;
        if respawn < 1 {
            respawn = 1;
        }
    }

    let level_time = ctx.world.level.time;
    let e = ctx.entity_mut(ent);

    // dropped items will not respawn
    if (e.flags & FL_DROPPED_ITEM) != 0 {
        e.freeAfterEvent = qtrue;
    }

    // picked up items still stay around, they just don't
    // draw anything.  This allows respawnable items
    // to be placed on movers.
    if (e.flags & FL_DROPPED_ITEM) == 0
        && matches!(it.kind, ItemKind::Weapon(_) | ItemKind::Powerup(_))
    {
        e.s.eFlags |= EF_ITEMPLACEHOLDER;
        e.s.eFlags &= !EF_NODRAW;
    } else {
        e.s.eFlags |= EF_NODRAW;
        e.r.svFlags |= SVF_NOCLIENT;
    }
    e.r.contents = 0;

    if e.genericValue9 != 0 {
        // dropped item, should be removed when picked up
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time;
        return;
    }

    // ZOID
    // A negative respawn times means to never respawn this item (but don't
    // delete it).  This is used by items that are respawned by third party
    // events such as ctf flags
    if respawn <= 0 {
        e.nextthink = 0;
        e.think = FnId::NONE;
    } else {
        e.nextthink = level_time + respawn * 1000;
        e.think = Some(EntThink::RespawnItem).into();
    }
    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    );
}

/// Raven `LaunchItem`.
///
/// Source: `oracle/codemp/game/g_items.c:2658-2733`
pub fn LaunchItem(
    ctx: &mut GameContext,
    item: ItemId,
    origin: vec3_t,
    velocity: vec3_t,
) -> EntityId {
    let dropped = G_Spawn(ctx);
    let it = item.item();

    let e = ctx.entity_mut(dropped);
    e.s.eType = ET_ITEM as c_int;
    e.s.modelindex = item.modelindex(); // store item number in modelindex
    if e.s.modelindex < 0 {
        e.s.modelindex = 0;
    }
    e.s.modelindex2 = 1; // This is non-zero is it's a dropped item

    // Raven `e->classname = item->classname`: alias the item table's `'static`
    // classname pointer (no pool copy).
    let classname: &'static CStr = unsafe { CStr::from_ptr(item.classname_cstr()) };
    ctx.ent_set(dropped, PrefixSet::ClassnameStatic(classname));
    let e = ctx.entity_mut(dropped);
    e.item = Some(item);
    e.r.mins = [-ITEM_RADIUS, -ITEM_RADIUS, -ITEM_RADIUS];
    e.r.maxs = [ITEM_RADIUS, ITEM_RADIUS, ITEM_RADIUS];

    e.r.contents = CONTENTS_TRIGGER;

    e.touch = Some(EntTouch::Touch_Item).into();

    G_SetOrigin(ctx.entity_mut(dropped), origin);
    let level_time = ctx.world.level.time;
    let e = ctx.entity_mut(dropped);
    e.s.pos.trType = trType_t::TR_GRAVITY;
    e.s.pos.trTime = level_time;
    e.s.pos.trDelta = velocity;

    e.flags |= FL_BOUNCE_HALF;
    if (ctx.world.cvars.g_gametype.integer == GT_CTF
        || ctx.world.cvars.g_gametype.integer == GT_CTY)
        && matches!(it.kind, ItemKind::Team(_))
    {
        // Special case for CTF flags
        let e = ctx.entity_mut(dropped);
        e.think = Some(EntThink::Team_DroppedFlagThink).into();
        e.nextthink = level_time + 30000;
        Team_CheckDroppedItem(ctx, dropped);

        // rww - so bots know (`droppedRedFlag`/`droppedBlueFlag` are raw seam
        // globals; derive the pointer at the write).
        if it.classname == "team_CTF_redflag" {
            ctx.world.globals.droppedRedFlag = ctx.entity_mut(dropped) as *mut gentity_t;
        } else if it.classname == "team_CTF_blueflag" {
            ctx.world.globals.droppedBlueFlag = ctx.entity_mut(dropped) as *mut gentity_t;
        }
    } else {
        // auto-remove after 30 seconds
        let e = ctx.entity_mut(dropped);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = level_time + 30000;
    }

    let e = ctx.entity_mut(dropped);
    e.flags = FL_DROPPED_ITEM;

    if matches!(it.kind, ItemKind::Weapon(_) | ItemKind::Powerup(_)) {
        e.s.eFlags |= EF_DROPPEDWEAPON;
    }

    vectoangles(velocity, &mut e.s.angles);
    e.s.angles[PITCH] = 0.0;

    // Raven compares the raw giTag with NO giType check, so the WP_* values pun
    // against every kind's tag space (e.g. ammo_thermal's AMMO_THERMAL == 7 ==
    // WP_BOWCASTER skips the roll below) — extract the raw tag per that.
    let tag = match it.kind {
        ItemKind::Bad | ItemKind::Health => 0,
        ItemKind::Armor { rating } => rating,
        ItemKind::Holdable(t)
        | ItemKind::Powerup(t)
        | ItemKind::Weapon(t)
        | ItemKind::Ammo(t)
        | ItemKind::Team(t) => t,
    };
    if tag == WP_TRIP_MINE as c_int || tag == WP_DET_PACK as c_int {
        e.s.angles[PITCH] = -90.0;
    }

    if tag != WP_BOWCASTER as c_int && tag != WP_DET_PACK as c_int && tag != WP_THERMAL as c_int {
        e.s.angles[ROLL] = -90.0;
    }

    e.physicsObject = qtrue;

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(dropped)).cast()),
    );

    dropped
}

/// Raven `Drop_Item`.
///
/// Source: `oracle/codemp/game/g_items.c:2742-2755`
pub fn Drop_Item(ctx: &mut GameContext, ent: EntityId, item: ItemId, angle: f32) -> EntityId {
    let mut angles = ctx.entity(ent).s.apos.trBase;
    angles[YAW] += angle;
    angles[PITCH] = 0.0; // always forward

    let mut velocity: vec3_t = [0.0; 3];
    AngleVectors(angles, Some(&mut velocity), None, None);
    velocity[0] *= 150.0;
    velocity[1] *= 150.0;
    velocity[2] *= 150.0;
    // C: `200 + crandom() * 50` is `double`; the sum widens `velocity[2]`,
    // then narrows back to the `float` component.
    velocity[2] = (velocity[2] as f64 + (200.0 + ctx.world.bg_state.rng.crandom() * 50.0)) as f32;

    let trBase = ctx.entity(ent).s.pos.trBase;
    LaunchItem(ctx, item, trBase, velocity)
}

/// Raven `Use_Item`.
///
/// Source: `oracle/codemp/game/g_items.c:2765-2767`
pub fn Use_Item(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    RespawnItem(ctx, ent);
}

/// Raven `FinishSpawningItem`.
///
/// Source: `oracle/codemp/game/g_items.c:2779-2963`
pub fn FinishSpawningItem(ctx: &mut GameContext, ent: EntityId) {
    let item = ctx.entity(ent).item.unwrap();
    let it = item.item();
    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        // in siege remove all powerups
        if matches!(it.kind, ItemKind::Powerup(_)) {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    if ctx.world.cvars.g_gametype.integer != GT_JEDIMASTER {
        if HasSetSaberOnly(ctx) {
            if matches!(it.kind, ItemKind::Ammo(_)) {
                G_FreeEntity(ctx, Some(ent));
                return;
            }

            if matches!(
                it.kind,
                ItemKind::Holdable(HI_SEEKER | HI_SHIELD | HI_SENTRY_GUN)
            ) {
                G_FreeEntity(ctx, Some(ent));
                return;
            }
        }
    } else {
        // no powerups in jedi master
        if matches!(it.kind, ItemKind::Powerup(_)) {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    if ctx.world.cvars.g_gametype.integer == GT_HOLOCRON {
        if matches!(
            it.kind,
            ItemKind::Powerup(PW_FORCE_ENLIGHTENED_LIGHT | PW_FORCE_ENLIGHTENED_DARK)
        ) {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    if ctx.world.cvars.g_forcePowerDisable.integer != 0 {
        // if force powers disabled, don't add force powerups
        if matches!(
            it.kind,
            ItemKind::Powerup(
                PW_FORCE_ENLIGHTENED_LIGHT | PW_FORCE_ENLIGHTENED_DARK | PW_FORCE_BOON
            )
        ) {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    if ctx.world.cvars.g_gametype.integer == GT_DUEL
        || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
    {
        if matches!(
            it.kind,
            ItemKind::Armor { .. }
                | ItemKind::Health
                | ItemKind::Holdable(HI_MEDPAC | HI_MEDPAC_BIG)
        ) {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    // Raven kills only the three flags here; the IT_TEAM red/blue cubes
    // (giTag 0) survive outside CTF/CTY.
    if ctx.world.cvars.g_gametype.integer != GT_CTF
        && ctx.world.cvars.g_gametype.integer != GT_CTY
        && matches!(
            it.kind,
            ItemKind::Team(PW_REDFLAG | PW_BLUEFLAG | PW_NEUTRALFLAG)
        )
    {
        G_FreeEntity(ctx, Some(ent));
        return;
    }

    let e = ctx.entity_mut(ent);
    e.r.mins = [-8.0, -8.0, 0.0];
    e.r.maxs = [8.0, 8.0, 16.0];

    e.s.eType = ET_ITEM as c_int;
    e.s.modelindex = item.modelindex(); // store item number in modelindex
    e.s.modelindex2 = 0; // zero indicates this isn't a dropped item

    e.r.contents = CONTENTS_TRIGGER;
    e.touch = Some(EntTouch::Touch_Item).into();
    // useing an item causes it to respawn
    e.use_ = Some(EntUse::Use_Item).into();

    if (ctx.entity(ent).spawnflags & ITMSF_SUSPEND) != 0 {
        // suspended
        let origin = ctx.entity(ent).s.origin;
        G_SetOrigin(ctx.entity_mut(ent), origin);
    } else {
        // drop to floor

        // if it is directly even with the floor it will return startsolid, so raise up by 0.1
        // and temporarily subtract 0.1 from the z maxs so that going up doesn't push into the ceiling
        let dest: vec3_t = {
            let e = ctx.entity_mut(ent);
            e.s.origin[2] += 0.1;
            e.r.maxs[2] -= 0.1;
            [e.s.origin[0], e.s.origin[1], e.s.origin[2] - 4096.0]
        };
        let mut tr: trace_t = unsafe { core::mem::zeroed() };
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &ctx.entity(ent).s.origin as *const vec3_t,
                &ctx.entity(ent).r.mins as *const vec3_t,
                &ctx.entity(ent).r.maxs as *const vec3_t,
                &dest as *const vec3_t,
                ctx.entity(ent).s.number,
                MASK_SOLID,
            ),
        );
        if tr.startsolid != 0 {
            G_Printf(ctx, "FinishSpawningItem: %s startsolid at %s\n");
            G_FreeEntity(ctx, Some(ent));
            return;
        }

        let e = ctx.entity_mut(ent);
        // add the 0.1 back after the trace
        e.r.maxs[2] += 0.1;

        // allow to ride movers
        e.s.groundEntityNum = tr.entityNum as c_int;

        G_SetOrigin(e, tr.endpos);
    }

    // team slaves and targeted items aren't present at start
    let e = ctx.entity_mut(ent);
    if (e.flags & FL_TEAMSLAVE) != 0 || e.targetname_str().is_some() {
        e.s.eFlags |= EF_NODRAW;
        e.r.contents = 0;
        return;
    }

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    );
}

/// Raven `G_CheckTeamItems`.
///
/// Source: `oracle/codemp/game/g_items.c:2973-2991`
pub fn G_CheckTeamItems(ctx: &mut GameContext) {
    // Set up team stuff
    Team_InitGame(ctx);

    if ctx.world.cvars.g_gametype.integer == GT_CTF || ctx.world.cvars.g_gametype.integer == GT_CTY
    {
        // check for the two flags
        let mut item = BG_FindItem("team_CTF_redflag");
        if item.is_none()
            || ctx.world.globals.itemRegistered.0[item.unwrap().modelindex() as usize] == 0
        {
            G_Printf(ctx, "WARNING: No team_CTF_redflag in map");
        }
        item = BG_FindItem("team_CTF_blueflag");
        if item.is_none()
            || ctx.world.globals.itemRegistered.0[item.unwrap().modelindex() as usize] == 0
        {
            G_Printf(ctx, "WARNING: No team_CTF_blueflag in map");
        }
    }
}

/// Raven `ClearRegisteredItems`.
///
/// Source: `oracle/codemp/game/g_items.c:2998-3011`
pub fn ClearRegisteredItems(ctx: &mut GameContext) {
    ctx.world.globals.itemRegistered = crate::game_globals::ItemRegistered::default();

    // players always start with the base weapon
    RegisterItem(ctx, BG_FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_STUN_BATON));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_MELEE));
    RegisterItem(ctx, BG_FindItemForWeapon(WP_SABER));

    if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
        // kind of cheesy, maybe check if siege class with disp's is gonna be on this map too
        G_PrecacheDispensers(ctx);
    }
}

/// Raven `RegisterItem`.
///
/// Source: `oracle/codemp/game/g_items.c:3020-3025`
pub fn RegisterItem(ctx: &mut GameContext, item: ItemId) {
    // Raven's `if (!item) Com_Error(...)` NULL guard is unreachable: `ItemId` is
    // non-null by construction (callers pass `Some` after a find).
    ctx.world.globals.itemRegistered.0[item.modelindex() as usize] = qtrue;
}

/// Raven `SaveRegisteredItems`.
///
/// Source: `oracle/codemp/game/g_items.c:3036-3054`
pub fn SaveRegisteredItems(ctx: &mut GameContext) {
    let mut string: Vec<c_char> = vec![0; crate::game_globals::MAX_ITEMS + 1];
    let mut count = 0;
    for i in 0..bg_numItems {
        if ctx.world.globals.itemRegistered.0[i as usize] != 0 {
            count += 1;
            string[i as usize] = b'1' as c_char;
        } else {
            string[i as usize] = b'0' as c_char;
        }
    }
    string[bg_numItems as usize] = 0;

    //	G_Printf( "%i items registered\n", count );
    let s = cstr_from_chars(&string).to_string_lossy().into_owned();
    trap::SetConfigstring(ctx.engine, CS_ITEMS, &s);
}

/// Raven `G_ItemDisabled`.
///
/// Source: `oracle/codemp/game/g_items.c:3061-3067`
pub fn G_ItemDisabled(ctx: &mut GameContext, item: ItemId) -> c_int {
    let name = format!("disable_{}", item.item().classname);
    trap::Cvar_VariableIntegerValue(ctx.engine, &name)
}

/// Raven `G_SpawnItem`.
///
/// Source: `oracle/codemp/game/g_items.c:3079-3121`
pub fn G_SpawnItem(ctx: &mut GameContext, ent: EntityId, item: ItemId) {
    // `G_SpawnFloat` writes an out-param into the entity field: form the raw
    // pointer (ending the arena borrow) before threading `ctx` into the call.
    let random_ptr = core::ptr::addr_of_mut!(ctx.world.g_entities[ent.index()].random);
    G_SpawnFloat(ctx, c"random".as_ptr(), c"0".as_ptr(), random_ptr);
    let wait_ptr = core::ptr::addr_of_mut!(ctx.world.g_entities[ent.index()].wait);
    G_SpawnFloat(ctx, c"wait".as_ptr(), c"0".as_ptr(), wait_ptr);

    let wDisable = if ctx.world.cvars.g_gametype.integer == GT_DUEL
        || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
    {
        ctx.world.cvars.g_duelWeaponDisable.integer
    } else {
        ctx.world.cvars.g_weaponDisable.integer
    };

    let it = item.item();
    if let ItemKind::Weapon(weapon) = it.kind {
        if wDisable != 0
            && (wDisable & (1 << weapon)) != 0
            && ctx.world.cvars.g_gametype.integer != GT_JEDIMASTER
        {
            G_FreeEntity(ctx, Some(ent));
            return;
        }
    }

    RegisterItem(ctx, item);
    if G_ItemDisabled(ctx, item) != 0 {
        return;
    }

    let level_time = ctx.world.level.time;
    let e = ctx.entity_mut(ent);
    e.item = Some(item);
    // some movers spawn on the second frame, so delay item
    // spawns until the third frame so they can ride trains
    e.nextthink = level_time + FRAMETIME * 2;
    e.think = Some(EntThink::FinishSpawningItem).into();

    e.physicsBounce = 0.50; // items are bouncy

    if matches!(it.kind, ItemKind::Powerup(_)) {
        G_SoundIndex(ctx, "sound/items/respawn1");
        let speed_ptr = core::ptr::addr_of_mut!(ctx.world.g_entities[ent.index()].speed);
        G_SpawnFloat(ctx, c"noglobalsound".as_ptr(), c"0".as_ptr(), speed_ptr);
    }
}

/// Raven `G_BounceItem`.
///
/// Source: `oracle/codemp/game/g_items.c:3130-3174`
pub fn G_BounceItem(ctx: &mut GameContext, ent: EntityId, trace: *mut trace_t) {
    // `trace` is a raw `trace_t*` — stays raw.
    // reflect the velocity on the trace plane
    // C: `previousTime + (level.time - previousTime) * trace->fraction` — the
    // int base promotes to f32 against the float product; one truncation.
    // Source: `oracle/codemp/game/g_items.c:3136`
    let fraction = unsafe { (*trace).fraction };
    let hitTime = (ctx.world.level.previousTime as f32
        + (ctx.world.level.time - ctx.world.level.previousTime) as f32 * fraction)
        as c_int;
    let mut velocity: vec3_t = [0.0; 3];
    BG_EvaluateTrajectoryDelta(
        &ctx.entity(ent).s.pos as *const trajectory_t,
        hitTime,
        &mut velocity,
    );
    let normal = unsafe { (*trace).plane.normal };
    let dot = velocity[0] * normal[0] + velocity[1] * normal[1] + velocity[2] * normal[2];
    ctx.entity_mut(ent).s.pos.trDelta = [
        velocity[0] + -2.0 * dot * normal[0],
        velocity[1] + -2.0 * dot * normal[1],
        velocity[2] + -2.0 * dot * normal[2],
    ];

    // cut the velocity to keep from bouncing forever
    let trDelta = ctx.entity(ent).s.pos.trDelta;
    let physicsBounce = ctx.entity(ent).physicsBounce;
    ctx.entity_mut(ent).s.pos.trDelta = [
        trDelta[0] * physicsBounce,
        trDelta[1] * physicsBounce,
        trDelta[2] * physicsBounce,
    ];

    if ctx.entity(ent).s.weapon == WP_DET_PACK as c_int
        && ctx.entity(ent).s.eType == ET_GENERAL as c_int
        && ctx.entity(ent).physicsObject != 0
    {
        // detpacks only
        if let Some(touch) = ctx.entity(ent).touch.get() {
            let entityNum = unsafe { (*trace).entityNum };
            let self_ptr: *mut gentity_t = ctx.entity_mut(ent);
            let other_ptr: *mut gentity_t = ctx.entity_mut(EntityId(entityNum as u32));
            crate::ent_fn_enums::dispatch_touch(ctx, touch, self_ptr, other_ptr, trace);
            return;
        }
    }

    // check for stop
    if unsafe { (*trace).plane.normal[2] } > 0.0 && ctx.entity(ent).s.pos.trDelta[2] < 40.0 {
        unsafe {
            (*trace).endpos[2] += 1.0; // make sure it is off ground
            snap_vector(&mut (*trace).endpos);
        }
        let endpos = unsafe { (*trace).endpos };
        G_SetOrigin(ctx.entity_mut(ent), endpos);
        ctx.entity_mut(ent).s.groundEntityNum = unsafe { (*trace).entityNum } as c_int;
        return;
    }

    let currentOrigin = ctx.entity(ent).r.currentOrigin;
    ctx.entity_mut(ent).r.currentOrigin = [
        currentOrigin[0] + normal[0],
        currentOrigin[1] + normal[1],
        currentOrigin[2] + normal[2],
    ];
    let currentOrigin = ctx.entity(ent).r.currentOrigin;
    ctx.entity_mut(ent).s.pos.trBase = currentOrigin;
    ctx.entity_mut(ent).s.pos.trTime = ctx.world.level.time;

    if ctx.entity(ent).s.eType == ET_HOLOCRON as c_int
        || (ctx.entity(ent).s.shouldtarget != 0
            && ctx.entity(ent).s.eType == ET_GENERAL as c_int
            && ctx.entity(ent).physicsObject != 0)
    {
        // holocrons and sentry guns
        if let Some(touch) = ctx.entity(ent).touch.get() {
            let entityNum = unsafe { (*trace).entityNum };
            let self_ptr: *mut gentity_t = ctx.entity_mut(ent);
            let other_ptr: *mut gentity_t = ctx.entity_mut(EntityId(entityNum as u32));
            crate::ent_fn_enums::dispatch_touch(ctx, touch, self_ptr, other_ptr, trace);
        }
    }
}

/// Raven `G_RunItem`.
///
/// Source: `oracle/codemp/game/g_items.c:3183-3242`
pub fn G_RunItem(ctx: &mut GameContext, ent: EntityId) {
    // if groundentity has been set to -1, it may have been pushed off an edge
    if ctx.entity(ent).s.groundEntityNum == -1
        && ctx.entity(ent).s.pos.trType != trType_t::TR_GRAVITY
    {
        ctx.entity_mut(ent).s.pos.trType = trType_t::TR_GRAVITY;
        ctx.entity_mut(ent).s.pos.trTime = ctx.world.level.time;
    }

    if ctx.entity(ent).s.pos.trType == trType_t::TR_STATIONARY {
        // check think function
        G_RunThink(ctx, ent);
        return;
    }

    // get current position
    let mut origin: vec3_t = [0.0; 3];
    BG_EvaluateTrajectory(
        &ctx.entity(ent).s.pos as *const trajectory_t,
        ctx.world.level.time,
        &mut origin,
    );

    // trace a line from the previous position to the current position
    let mask = if ctx.entity(ent).clipmask != 0 {
        ctx.entity(ent).clipmask
    } else {
        MASK_PLAYERSOLID & !CONTENTS_BODY
    };
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ctx.entity(ent).r.currentOrigin as *const vec3_t,
            &ctx.entity(ent).r.mins as *const vec3_t,
            &ctx.entity(ent).r.maxs as *const vec3_t,
            &origin as *const vec3_t,
            ctx.entity(ent).r.ownerNum,
            mask,
        ),
    );

    ctx.entity_mut(ent).r.currentOrigin = tr.endpos;

    if tr.startsolid != 0 {
        tr.fraction = 0.0;
    }

    trap::LinkEntity(
        ctx.engine,
        GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(ent)).cast()),
    ); // FIXME: avoid this for stationary?

    // check think function
    G_RunThink(ctx, ent);

    if tr.fraction == 1.0 {
        return;
    }

    // if it is in a nodrop volume, remove it
    let contents = trap::PointContents(
        ctx.engine,
        mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs::new(
            &ctx.entity(ent).r.currentOrigin as *const vec3_t,
            -1,
        ),
    );
    if (contents & CONTENTS_NODROP) != 0 {
        let item = ctx.entity(ent).item;
        if item.is_some_and(|it| matches!(it.item().kind, ItemKind::Team(_))) {
            Team_FreeEntity(ctx, ent);
        } else {
            G_FreeEntity(ctx, Some(ent));
        }
        return;
    }

    G_BounceItem(ctx, ent, &mut tr as *mut trace_t);
}
