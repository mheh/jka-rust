// PORT-COMPLETE: NPC_combat.c 33/38 (pass-3 shard: +28, see NPC_combat.md)
//! FAITHFUL port of `oracle/codemp/game/NPC_combat.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
//!
//! Safe-state migration **Stage 2b** (body sweep): every world reach is a
//! checked `ctx.world.…` borrow — the transitional `(*ctx.world_raw())`
//! raw-deref regime (and its hoisted `let world = &mut *ctx.world_raw()`
//! aliases) is gone. The per-body entity/`gNPC_t`/`gclient_t` re-derives stay
//! raw by design, and the two `Debug_Printf` cvar-pointer sites keep an
//! irreducible `&raw mut ctx.world.cvars.debugNPCAI` alias (marked in-code)
//! passed alongside `ctx` to the raw-ABI callee. Behavior is byte-identical,
//! referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use crate::entity::flags::FL_NOTARGET;
use crate::g_combat::G_AlertTeam;
use crate::g_items::{Add_Ammo, CheckItemCanBePickedUpByNPC};
use crate::g_nav::NAV_FindClosestWaypointForPoint2;
use crate::g_nav::{NAV_ClearPathToPoint, NAV_GetNearestNode, NPC_SetMoveGoal};
use crate::g_team::S_COLOR_RED;
use crate::g_timer::{TIMER_Done, TIMER_Exists, TIMER_Set};
use crate::g_utils::{vtos, G_Sound};
use crate::g_utils::{G_CheckInSolid, G_FreeEntity, G_SetOrigin};
use crate::level::combat_point::MAX_COMBAT_POINTS;
use crate::npc::ai_flags::NPCAI_BURST_WEAPON;
use crate::npc::check_flags::{CHECK_360, CHECK_FOV, CHECK_VISRANGE};
use crate::npc::script_flags::{SCF_ALT_FIRE, SCF_DONT_FIRE, SCF_NO_GROUPS};
use crate::prelude::*;
use crate::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorSubtract, vec3_origin, vectoangles, AngleVectors,
    DistanceHorizontalSquared, VectorLength, VectorLengthSquared, VectorNormalize, PITCH, YAW,
};
use crate::q_shared::Q_stricmp;
use crate::teams::class::*;
use crate::teams::npcteam::{NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER};
use crate::NPC_AI_Default::NPC_LostEnemyDecideChase;
use crate::NPC_AI_Jedi::NPC_Jedi_RateNewEnemy;
use crate::NPC_misc::Debug_Printf;
use crate::NPC_senses::{InVisrange, NPC_CheckVisibility};
use crate::NPC_sounds::G_AddVoiceEvent;
use crate::NPC_utils::G_ActivateBehavior;
use crate::NPC_utils::{
    CalcEntitySpot, NPC_AimWiggle, NPC_CheckLookTarget, NPC_ClearLOS, NPC_ClearLOS3,
    NPC_ClearLookTarget, NPC_UpdateFiringAngles, NPC_ValidEnemy,
};
use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_abi::game::syscalls::G_NAV_GETBESTNODEALT2::GNavGetbestnodealt2Args;
use mp_abi::game::syscalls::G_NAV_GETBESTPATHBETWEENENTS::GNavGetbestpathbetweenentsArgs;
use mp_abi::game::syscalls::G_NAV_GETPATHCOST::GNavGetpathcostArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::entity_event::entity_event_t::{EV_ANGER1, EV_ANGER3};
use mp_bg::public::weaponstate::weaponstate_t::{
    WEAPON_DROPPING, WEAPON_FIRING, WEAPON_IDLE, WEAPON_RAISING, WEAPON_READY,
};
use mp_bg::weapons::weapon_t::*;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::MASK_SHOT;

// Raven `DEBUG_LEVEL_INFO` (`b_local.h:23`) — not yet ported as a central
// const; inlined here from the header value.
const DEBUG_LEVEL_INFO: c_int = 3;

/// `ent - g_entities` base pointer for this file's `ent_id`/`ent_id_opt` calls
/// (entity-id/pointer seam helper), precedent `g_missile.rs`/`g_trigger.rs`.
#[inline]
unsafe fn ent_base(ctx: &mut GameContext) -> *const gentity_t {
    ctx.world.g_entities.as_ptr()
}

/// Resolve a stored `Option<EntityId>` field back to a `gentity_t*` (the
/// id->pointer half of the entity-id seam; `None` -> Raven's NULL).
#[inline]
unsafe fn ent_ptr(ctx: &mut GameContext, id: Option<EntityId>) -> *mut gentity_t {
    match id {
        Some(i) => &mut ctx.world.g_entities[i.index()] as *mut gentity_t,
        None => core::ptr::null_mut(),
    }
}

// Unported types referenced in this file (need porting before this compiles):
// combatPt_t

/// Raven `G_ClearEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:17-36`
pub fn G_ClearEnemy(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        NPC_CheckLookTarget(ctx, ctx.entity_id_of(self_).unwrap());

        if !(*self_).enemy.is_none() {
            let client = (*self_).client as *mut gclient_t;
            let enemy =
                &mut ctx.world.g_entities[(*self_).enemy.unwrap().index()] as *mut gentity_t;
            if !client.is_null() && (*client).renderInfo.lookTarget == (*enemy).s.number {
                NPC_ClearLookTarget(ctx.entity_mut(ctx.entity_id_of(self_).unwrap()));
            }

            let npc = (*self_).NPC as *mut gNPC_t;
            if !npc.is_null() && (*self_).enemy == (*npc).goalEntity {
                (*npc).goalEntity = None;
            }
            //FIXME: set last enemy?
        }

        (*self_).enemy = None;
    }
}

/// Raven `G_AngerAlert`.
///
/// Raven: `ANGER_ALERT_RADIUS` (512), `ANGER_ALERT_SOUND_RADIUS` (256).
/// Source: `oracle/codemp/game/NPC_combat.c:44-59`
pub fn G_AngerAlert(ctx: &mut GameContext, self_: Option<EntityId>) {
    pub const ANGER_ALERT_RADIUS: f32 = 512.0;
    pub const ANGER_ALERT_SOUND_RADIUS: f32 = 256.0;
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ent_ptr(ctx, self_);
        let self_id = ctx.entity_id_of(self_).unwrap();
        let ent = ent_ptr(ctx, (*self_).enemy);
        let ent_id = ctx.entity_id_of(ent);
        if !self_.is_null() {
            let npc = (*self_).NPC as *mut gNPC_t;
            if !npc.is_null() && ((*npc).scriptFlags & SCF_NO_GROUPS) != 0 {
                //I'm not a team playa...
                return;
            }
        }
        if TIMER_Done(
            ctx,
            ctx.entity_id_of(self_),
            c"interrogating".as_ptr() as *const c_char,
        ) == 0
        {
            //I'm interrogating, don't wake everyone else up yet...
            return;
        }
        G_AlertTeam(
            ctx,
            self_id,
            ent_id,
            ANGER_ALERT_RADIUS,
            ANGER_ALERT_SOUND_RADIUS,
        );
    }
}

/// Raven `G_TeamEnemy`.
///
/// Raven: FIXME - Probably a better way to do this, is a linked list of your
/// teammates already available?
/// Source: `oracle/codemp/game/NPC_combat.c:67-115`
pub fn G_TeamEnemy(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let self_client = (*self_).client as *mut gclient_t;
        if self_client.is_null() || (*self_client).playerTeam == NPCTEAM_FREE {
            return 0;
        }
        let self_npc = (*self_).NPC as *mut gNPC_t;
        if !self_npc.is_null() && ((*self_npc).scriptFlags & SCF_NO_GROUPS) != 0 {
            //I'm not a team playa...
            return 0;
        }

        let num_entities = ctx.world.level.num_entities;
        for i in 1..num_entities {
            let ent = &mut ctx.world.g_entities[i as usize] as *mut gentity_t;
            if ent == self_ {
                continue;
            }
            if (*ent).health <= 0 {
                continue;
            }
            let ent_client = (*ent).client as *mut gclient_t;
            if ent_client.is_null() {
                continue;
            }
            if (*ent_client).playerTeam != (*self_client).playerTeam {
                //ent is not on my team
                continue;
            }
            if !(*ent).enemy.is_none() {
                //they have an enemy
                let enemy_client = (*ent_ptr(ctx, (*ent).enemy)).client as *mut gclient_t;
                if enemy_client.is_null() || (*enemy_client).playerTeam != (*self_client).playerTeam
                {
                    //the ent's enemy is either a normal ent or is a player/NPC that is not on my team
                    return 1;
                }
            }
        }

        0
    }
}

/// Raven `G_AttackDelay`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:117-310`
pub fn G_AttackDelay(ctx: &mut GameContext, self_: EntityId, enemy: Option<EntityId>) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let enemy: *mut gentity_t = ent_ptr(ctx, enemy);
        if enemy.is_null() || (*self_).client.is_null() || (*self_).NPC.is_null() {
            return;
        }
        //delay their attack based on how far away they're facing from enemy
        let client = (*self_).client as *mut gclient_t;
        let npc = (*self_).NPC as *mut gNPC_t;

        // VectorSubtract( self->client->renderInfo.eyePoint, enemy->r.currentOrigin, dir );//purposely backwards
        let eye = (*client).renderInfo.eyePoint;
        let enemy_org = (*enemy).r.currentOrigin;
        let mut dir = [
            eye[0] - enemy_org[0],
            eye[1] - enemy_org[1],
            eye[2] - enemy_org[2],
        ];
        VectorNormalize(&mut dir);
        let mut fwd = [0.0f32; 3];
        AngleVectors((*client).renderInfo.eyeAngles, Some(&mut fwd), None, None);
        //dir[2] = fwd[2] = 0;//ignore z diff?

        let g_spskill = ctx.world.cvars.g_spskill.integer;
        let mut attDelay = (4 - g_spskill) * 500; //initial: from 1000ms delay on hard to 2000ms delay on easy
        if (*client).playerTeam == NPCTEAM_PLAYER {
            //invert
            attDelay = 2000 - attDelay;
        }
        let dot = fwd[0] * dir[0] + fwd[1] * dir[1] + fwd[2] * dir[2];
        attDelay += ((dot + 1.0) * 2000.0).floor() as c_int; //add up to 4000ms delay if they're facing away

        //FIXME: should distance matter, too?

        //Now modify the delay based on NPC_class, weapon, and team
        //NOTE: attDelay should be somewhere between 1000 to 6000 milliseconds
        match (*client).NPC_class {
            class_t::CLASS_IMPERIAL => {
                //they give orders and hang back
                attDelay += ctx.world.bg_state.rng.Q_irand(500, 1500);
            }
            class_t::CLASS_STORMTROOPER => {
                //stormtroopers shoot sooner
                if (*npc).rank >= RANK_LT {
                    //officers shoot even sooner
                    attDelay -= ctx.world.bg_state.rng.Q_irand(500, 1500);
                } else {
                    //normal stormtroopers don't have as fast reflexes as officers
                    attDelay -= ctx.world.bg_state.rng.Q_irand(0, 1000);
                }
            }
            class_t::CLASS_SWAMPTROOPER => {
                //shoot very quickly?  What about guys in water?
                attDelay -= ctx.world.bg_state.rng.Q_irand(1000, 2000);
            }
            class_t::CLASS_IMPWORKER => {
                //they panic, don't fire right away
                attDelay += ctx.world.bg_state.rng.Q_irand(1000, 2500);
            }
            class_t::CLASS_TRANDOSHAN => {
                attDelay -= ctx.world.bg_state.rng.Q_irand(500, 1500);
            }
            class_t::CLASS_JAN
            | class_t::CLASS_LANDO
            | class_t::CLASS_PRISONER
            | class_t::CLASS_REBEL => {
                attDelay -= ctx.world.bg_state.rng.Q_irand(500, 1500);
            }
            class_t::CLASS_GALAKMECH | class_t::CLASS_ATST => {
                attDelay -= ctx.world.bg_state.rng.Q_irand(1000, 2000);
            }
            class_t::CLASS_REELO | class_t::CLASS_UGNAUGHT | class_t::CLASS_JAWA => {
                return;
            }
            class_t::CLASS_MINEMONSTER | class_t::CLASS_MURJJ => {
                return;
            }
            class_t::CLASS_INTERROGATOR
            | class_t::CLASS_PROBE
            | class_t::CLASS_MARK1
            | class_t::CLASS_MARK2
            | class_t::CLASS_SENTRY => {
                return;
            }
            class_t::CLASS_REMOTE | class_t::CLASS_SEEKER => {
                return;
            }
            /*
            CLASS_GRAN, CLASS_RODIAN, CLASS_WEEQUAY,
            CLASS_JEDI, CLASS_SHADOWTROOPER, CLASS_TAVION, CLASS_REBORN,
            CLASS_LUKE, CLASS_DESANN,
            */
            _ => {}
        }

        match (*self_).s.weapon {
            w if w == WP_NONE as c_int || w == WP_SABER as c_int => {
                return;
            }
            w if w == WP_BRYAR_PISTOL as c_int => {}
            w if w == WP_BLASTER as c_int => {
                if ((*npc).scriptFlags & SCF_ALT_FIRE) != 0 {
                    //rapid-fire blasters
                    attDelay += ctx.world.bg_state.rng.Q_irand(0, 500);
                } else {
                    //regular blaster
                    attDelay -= ctx.world.bg_state.rng.Q_irand(0, 500);
                }
            }
            w if w == WP_BOWCASTER as c_int => {
                attDelay += ctx.world.bg_state.rng.Q_irand(0, 500);
            }
            w if w == WP_REPEATER as c_int => {
                if ((*npc).scriptFlags & SCF_ALT_FIRE) == 0 {
                    //rapid-fire blasters
                    attDelay += ctx.world.bg_state.rng.Q_irand(0, 500);
                }
            }
            w if w == WP_FLECHETTE as c_int => {
                attDelay += ctx.world.bg_state.rng.Q_irand(500, 1500);
            }
            w if w == WP_ROCKET_LAUNCHER as c_int => {
                attDelay += ctx.world.bg_state.rng.Q_irand(500, 1500);
            }
            //rwwFIXMEFIXME: Have this weapon for NPCs?
            w if w == WP_DISRUPTOR as c_int => {
                //sniper's don't delay?
                return;
            }
            w if w == WP_THERMAL as c_int => {
                //grenade-throwing has a built-in delay
                return;
            }
            w if w == WP_STUN_BATON as c_int => {
                // Any ol' melee attack
                return;
            }
            w if w == WP_EMPLACED_GUN as c_int => {
                return;
            }
            w if w == WP_TURRET as c_int => {
                // turret guns
                return;
            }
            _ => {}
        }

        if (*client).playerTeam == NPCTEAM_PLAYER {
            //clamp it
            if attDelay > 2000 {
                attDelay = 2000;
            }
        }

        //don't shoot right away
        let cap = 4000 + ((2 - g_spskill) * 3000);
        if attDelay > cap {
            attDelay = cap;
        }
        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"attackDelay".as_ptr() as *const c_char,
            attDelay,
        ); //ctx.world.bg_state.rng.Q_irand( 1500, 4500 ) );
           //don't move right away either
        if attDelay > 4000 {
            attDelay = 4000 - ctx.world.bg_state.rng.Q_irand(500, 1500);
        } else {
            attDelay -= ctx.world.bg_state.rng.Q_irand(500, 1500);
        }

        TIMER_Set(
            ctx,
            ctx.entity_id_of(self_),
            c"roamTime".as_ptr() as *const c_char,
            attDelay,
        ); //was ctx.world.bg_state.rng.Q_irand( 1000, 3500 );
    }
}

/// Raven `G_ForceSaberOn`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:312-340`
pub fn G_ForceSaberOn(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ctx.entity_mut(ent);
        let client = (*ent).client as *mut gclient_t;
        if (*client).ps.saberInFlight != 0 {
            //alright, can't turn it on now in any case, so forget it.
            return;
        }
        if (*client).ps.saberHolstered == 0 {
            //it's already on!
            return;
        }
        if (*client).ps.weapon != WP_SABER {
            //This probably should never happen. But if it does we'll just return without complaining.
            return;
        }

        //Well then, turn it on.
        (*client).ps.saberHolstered = 0;

        if (*client).saber[0].soundOn != 0 {
            G_Sound(
                ctx,
                ctx.entity_id_of(ent),
                mp_qshared::shared::sound_channel::CHAN_AUTO,
                (*client).saber[0].soundOn,
            );
        }
        if (*client).saber[1].soundOn != 0 {
            G_Sound(
                ctx,
                ctx.entity_id_of(ent),
                mp_qshared::shared::sound_channel::CHAN_AUTO,
                (*client).saber[1].soundOn,
            );
        }
    }
}

/// Raven `G_SetEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:349-523`
pub fn G_SetEnemy(ctx: &mut GameContext, self_: EntityId, enemy: Option<EntityId>) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let enemy: *mut gentity_t = ent_ptr(ctx, enemy);
        let base = ent_base(ctx);
        let mut event: c_int = 0;

        //Must be valid
        if enemy.is_null() {
            return;
        }

        //Must be valid
        if (*enemy).inuse == 0 {
            return;
        }

        //Don't take the enemy if in notarget
        if ((*enemy).flags & FL_NOTARGET) != 0 {
            return;
        }

        let npc = (*self_).NPC as *mut gNPC_t;
        if npc.is_null() {
            (*self_).enemy = ent_id_opt(base, enemy);
            return;
        }

        let level_time = ctx.world.level.time;
        if (*npc).confusionTime > level_time {
            //can't pick up enemies if confused
            return;
        }

        // (debug assert( enemy != self ) omitted — release build)

        //	if ( enemy->client && enemy->client->playerTeam == TEAM_DISGUISE )
        //	{//unmask the player
        //		enemy->client->playerTeam = TEAM_PLAYER;
        //	}

        let client = (*self_).client as *mut gclient_t;
        let enemy_client = (*enemy).client as *mut gclient_t;
        if !client.is_null()
            && !enemy_client.is_null()
            && (*enemy_client).playerTeam == (*client).playerTeam
        {
            //Probably a damn script!
            if (*npc).charmedTime > level_time {
                //Probably a damn script!
                return;
            }
        }

        if !client.is_null() && (*client).ps.weapon == WP_SABER {
            //when get new enemy, set a base aggression based on what that enemy is using, how far they are, etc.
            NPC_Jedi_RateNewEnemy(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                ctx.entity_id_of(enemy),
            );
        }

        //NOTE: this is not necessarily true!
        //self->NPC->enemyLastSeenTime = level.time;

        if (*self_).enemy.is_none() {
            //TEMP HACK: turn on our saber
            if (*self_).health > 0 {
                G_ForceSaberOn(ctx, ctx.entity_id_of(self_).unwrap());
            }

            //FIXME: Have to do this to prevent alert cascading
            G_ClearEnemy(ctx, ctx.entity_id_of(self_).unwrap());
            (*self_).enemy = ent_id_opt(base, enemy);

            //Special case- if player is being hunted by his own people, set their enemy team correctly
            if (*client).playerTeam == NPCTEAM_PLAYER && (*enemy).s.number == 0 {
                (*client).enemyTeam = NPCTEAM_PLAYER;
            }

            //If have an anger script, run that instead of yelling
            if G_ActivateBehavior(ctx, ctx.entity_id_of(self_), BSET_ANGER as c_int) != 0 {
                // handled by the script
            } else if !client.is_null()
                && !enemy_client.is_null()
                && (*client).playerTeam != (*enemy_client).playerTeam
            {
                //FIXME: Use anger when entire team has no enemy.
                //		 Basically, you're first one to notice enemies
                //if ( self->forcePushTime < level.time ) // not currently being pushed
                //rwwFIXMEFIXME: Set forcePushTime
                if G_TeamEnemy(ctx, ctx.entity_id_of(self_).unwrap()) == 0 {
                    //team did not have an enemy previously
                    event = ctx
                        .world
                        .bg_state
                        .rng
                        .Q_irand(EV_ANGER1 as c_int, EV_ANGER3 as c_int);
                }

                if event != 0 {
                    //yell
                    G_AddVoiceEvent(ctx, ctx.entity_id_of(self_).unwrap(), event, 2000);
                }
            }

            if (*self_).s.weapon == WP_BLASTER
                || (*self_).s.weapon == WP_REPEATER
                || (*self_).s.weapon == WP_THERMAL
                /*|| self->s.weapon == WP_BLASTER_PISTOL */
                //rwwFIXMEFIXME: Blaster pistol useable by npcs?
                || (*self_).s.weapon == WP_BOWCASTER
            {
                //Hmm, how about sniper and bowcaster?
                //When first get mad, aim is bad
                //Hmm, base on game difficulty, too?  Rank?
                let g_spskill = ctx.world.cvars.g_spskill.integer;
                if (*client).playerTeam == NPCTEAM_PLAYER {
                    let self_id = ctx.entity_id_of(self_).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(
                        (*npc).stats.aim - (5 * g_spskill),
                        (*npc).stats.aim - g_spskill,
                    );
                    G_AimSet(ctx, self_id, delay);
                } else {
                    let mut minErr = 3;
                    let mut maxErr = 12;
                    let self_id = ctx.entity_id_of(self_).unwrap();
                    let delay = ctx.world.bg_state.rng.Q_irand(
                        (*npc).stats.aim - (maxErr * (3 - g_spskill)),
                        (*npc).stats.aim - (minErr * (3 - g_spskill)),
                    );
                    if (*client).NPC_class == class_t::CLASS_IMPWORKER {
                        minErr = 15;
                        maxErr = 30;
                    } else if (*client).NPC_class == class_t::CLASS_STORMTROOPER
                        && !npc.is_null()
                        && (*npc).rank <= RANK_CREWMAN
                    {
                        minErr = 5;
                        maxErr = 15;
                    }

                    G_AimSet(ctx, self_id, delay);
                }
            }

            //Alert anyone else in the area
            if Q_stricmp(c"desperado".as_ptr(), (*self_).NPC_type as *const c_char) != 0
                && Q_stricmp(c"paladin".as_ptr(), (*self_).NPC_type as *const c_char) != 0
            {
                //special holodeck enemies exception
                if (*client).ps.fd.forceGripBeingGripped < level_time as f32 {
                    //gripped people can't call for help
                    G_AngerAlert(ctx, ctx.entity_id_of(self_));
                }
            }

            //Stormtroopers don't fire right away!
            G_AttackDelay(
                ctx,
                ctx.entity_id_of(self_).unwrap(),
                ctx.entity_id_of(enemy),
            );

            //rwwFIXMEFIXME: Deal with this some other way.
            /*
            //FIXME: this is a disgusting hack that is supposed to make the Imperials start with their weapon holstered- need a better way
            (dead code, oracle-commented-out)
            */
            return;
        }

        //Otherwise, just picking up another enemy

        if event != 0 {
            G_AddVoiceEvent(ctx, ctx.entity_id_of(self_).unwrap(), event, 2000);
        }

        //Take the enemy
        G_ClearEnemy(ctx, ctx.entity_id_of(self_).unwrap());
        (*self_).enemy = ent_id_opt(base, enemy);
    }
}

/// Raven `ChangeWeapon`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:570-842`
pub fn ChangeWeapon(ctx: &mut GameContext, ent: Option<EntityId>, newWeapon: c_int) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ent_ptr(ctx, ent);
        if ent.is_null() || (*ent).client.is_null() || (*ent).NPC.is_null() {
            return;
        }

        let client = (*ent).client as *mut gclient_t;
        let npc = (*ent).NPC as *mut gNPC_t;

        (*client).ps.weapon = newWeapon;
        (*client).pers.cmd.weapon = newWeapon as u8;
        (*npc).shotTime = 0;
        (*npc).burstCount = 0;
        (*npc).attackHold = 0;
        (*npc).currentAmmo = (*client).ps.ammo[weaponData[newWeapon as usize].ammoIndex as usize];

        let g_spskill = ctx.world.cvars.g_spskill.integer;

        match newWeapon {
            WP_BRYAR_PISTOL => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                (*npc).burstSpacing = 1000;
            }
            WP_SABER => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                (*npc).burstSpacing = 0;
            }
            WP_DISRUPTOR => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                if ((*npc).scriptFlags & SCF_ALT_FIRE) != 0 {
                    match g_spskill {
                        0 => (*npc).burstSpacing = 2500,
                        1 => (*npc).burstSpacing = 2000,
                        2 => (*npc).burstSpacing = 1500,
                        _ => {}
                    }
                } else {
                    (*npc).burstSpacing = 1000;
                }
            }
            WP_BOWCASTER => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                if g_spskill == 0 {
                    (*npc).burstSpacing = 1000;
                } else if g_spskill == 1 {
                    (*npc).burstSpacing = 750;
                } else {
                    (*npc).burstSpacing = 500;
                }
            }
            WP_REPEATER => {
                if ((*npc).scriptFlags & SCF_ALT_FIRE) != 0 {
                    (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                    (*npc).burstSpacing = 2000;
                } else {
                    (*npc).aiFlags |= NPCAI_BURST_WEAPON;
                    (*npc).burstMin = 3;
                    (*npc).burstMean = 6;
                    (*npc).burstMax = 10;
                    if g_spskill == 0 {
                        (*npc).burstSpacing = 1500;
                    } else if g_spskill == 1 {
                        (*npc).burstSpacing = 1000;
                    } else {
                        (*npc).burstSpacing = 500;
                    }
                }
            }
            WP_DEMP2 => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                (*npc).burstSpacing = 1000;
            }
            WP_FLECHETTE => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                if ((*npc).scriptFlags & SCF_ALT_FIRE) != 0 {
                    (*npc).burstSpacing = 2000;
                } else {
                    (*npc).burstSpacing = 1000;
                }
            }
            WP_ROCKET_LAUNCHER => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                if g_spskill == 0 {
                    (*npc).burstSpacing = 2500;
                } else if g_spskill == 1 {
                    (*npc).burstSpacing = 2000;
                } else {
                    (*npc).burstSpacing = 1500;
                }
            }
            WP_THERMAL => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                if g_spskill == 0 {
                    (*npc).burstSpacing = 3000;
                } else if g_spskill == 1 {
                    (*npc).burstSpacing = 2500;
                } else {
                    (*npc).burstSpacing = 2000;
                }
            }
            WP_BLASTER => {
                if ((*npc).scriptFlags & SCF_ALT_FIRE) != 0 {
                    (*npc).aiFlags |= NPCAI_BURST_WEAPON;
                    (*npc).burstMin = 3;
                    (*npc).burstMean = 3;
                    (*npc).burstMax = 3;
                    if g_spskill == 0 {
                        (*npc).burstSpacing = 1500;
                    } else if g_spskill == 1 {
                        (*npc).burstSpacing = 1000;
                    } else {
                        (*npc).burstSpacing = 500;
                    }
                } else {
                    (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                    if g_spskill == 0 {
                        (*npc).burstSpacing = 1000;
                    } else if g_spskill == 1 {
                        (*npc).burstSpacing = 750;
                    } else {
                        (*npc).burstSpacing = 500;
                    }
                }
            }
            WP_STUN_BATON => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                (*npc).burstSpacing = 1000;
            }
            WP_EMPLACED_GUN => {
                if !client.is_null() && (*client).NPC_class == class_t::CLASS_REELO {
                    (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
                    (*npc).burstSpacing = 1000;
                } else {
                    (*npc).aiFlags |= NPCAI_BURST_WEAPON;
                    (*npc).burstMin = 2;
                    (*npc).burstMean = 2;
                    (*npc).burstMax = 2;

                    if !(*ent).parent.is_none() {
                        let parent = ent_ptr(ctx, (*ent).parent);
                        if g_spskill == 0 {
                            (*npc).burstSpacing = (*parent).wait as c_int + 400;
                            (*npc).burstMin = 1;
                            (*npc).burstMax = 1;
                        } else if g_spskill == 1 {
                            (*npc).burstSpacing = (*parent).wait as c_int + 200;
                        } else {
                            (*npc).burstSpacing = (*parent).wait as c_int;
                        }
                    } else if g_spskill == 0 {
                        (*npc).burstSpacing = 1200;
                        (*npc).burstMin = 1;
                        (*npc).burstMax = 1;
                    } else if g_spskill == 1 {
                        (*npc).burstSpacing = 1000;
                    } else {
                        (*npc).burstSpacing = 800;
                    }
                }
            }
            _ => {
                (*npc).aiFlags &= !NPCAI_BURST_WEAPON;
            }
        }
    }
}

/// Raven `NPC_ChangeWeapon`.
///
/// Raven: entire body is commented out (dead code, rwwFIXMEFIXME note that
/// NPC weapon-changing should work "the same way as players"); faithfully a
/// no-op.
/// Source: `oracle/codemp/game/NPC_combat.c:844-873`
pub fn NPC_ChangeWeapon(newWeapon: c_int) {
    //rwwFIXMEFIXME: Change the same way as players, all this stuff is just crazy.
}

/// Raven `NPC_ApplyWeaponFireDelay`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:878-918`
pub fn NPC_ApplyWeaponFireDelay(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let client = ctx.world.globals.client;
        let level_time = ctx.world.level.time;

        if (*npc).attackDebounceTime > level_time {
            //Just fired, if attacking again, must be a burst fire, so don't add delay
            //NOTE: Borg AI uses attackDebounceTime "incorrectly", so this will always return for them!
            return;
        }

        match (*client).ps.weapon {
            WP_THERMAL => {
                if (*client).ps.clientNum != 0 {
                    //NPCs delay...
                    (*client).ps.weaponTime = 700;
                }
            }
            WP_STUN_BATON => {
                //if ( !PM_DroidMelee( client->NPC_class ) )
                //rwwFIXMEFIXME: ...
                (*client).ps.weaponTime = 300;
            }
            _ => {
                (*client).ps.weaponTime = 0;
            }
        }
    }
}

/// Raven `ShootThink`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:925-1028`
pub fn ShootThink(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let client = ctx.world.globals.client;

        ctx.world.globals.ucmd.buttons &= !BUTTON_ATTACK;

        if (*client).ps.weapon == WP_NONE {
            return;
        }

        if (*client).ps.weaponstate != WEAPON_READY as c_int
            && (*client).ps.weaponstate != WEAPON_FIRING as c_int
            && (*client).ps.weaponstate != WEAPON_IDLE as c_int
        {
            return;
        }

        if ctx.world.level.time < (*npc_info).shotTime {
            return;
        }

        ctx.world.globals.ucmd.buttons |= BUTTON_ATTACK;

        (*npc_info).currentAmmo =
            (*client).ps.ammo[weaponData[(*client).ps.weapon as usize].ammoIndex as usize];

        NPC_ApplyWeaponFireDelay(ctx);

        let mut delay: c_int = 0;
        if ((*npc_info).aiFlags & NPCAI_BURST_WEAPON) != 0 {
            if (*npc_info).burstCount == 0 {
                (*npc_info).burstCount = ctx
                    .world
                    .bg_state
                    .rng
                    .Q_irand((*npc_info).burstMin, (*npc_info).burstMax);
                delay = 0;
            } else {
                (*npc_info).burstCount -= 1;
                if (*npc_info).burstCount == 0 {
                    delay = (*npc_info).burstSpacing;
                } else {
                    delay = 0;
                }
            }

            if delay == 0 {
                // HACK: dirty little emplaced bits, but is done because it would otherwise require some sort of new variable...
                if (*client).ps.weapon == WP_EMPLACED_GUN {
                    let g_spskill = ctx.world.cvars.g_spskill.integer;
                    if !(*npc).parent.is_none() {
                        // try and get the debounce values from the chair if we can
                        let parent = ent_ptr(ctx, (*npc).parent);
                        if g_spskill == 0 {
                            delay = (*parent).random as c_int + 150;
                        } else if g_spskill == 1 {
                            delay = (*parent).random as c_int + 100;
                        } else {
                            delay = (*parent).random as c_int;
                        }
                    } else if g_spskill == 0 {
                        delay = 350;
                    } else if g_spskill == 1 {
                        delay = 300;
                    } else {
                        delay = 200;
                    }
                }
            }
        } else {
            delay = (*npc_info).burstSpacing;
        }

        (*npc_info).shotTime = ctx.world.level.time + delay;
        (*npc).attackDebounceTime = ctx.world.level.time + NPC_AttackDebounceForWeapon(ctx);
    }
}

/// Raven `WeaponThink`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1036-1103`
pub fn WeaponThink(ctx: &mut GameContext, inCombat: qboolean) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let client = ctx.world.globals.client;

        if (*client).ps.weaponstate == WEAPON_RAISING as c_int
            || (*client).ps.weaponstate == WEAPON_DROPPING as c_int
        {
            ctx.world.globals.ucmd.weapon = (*client).ps.weapon as u8;
            ctx.world.globals.ucmd.buttons &= !BUTTON_ATTACK;
            return;
        }

        //MCG - Begin
        //For now, no-one runs out of ammo
        let npc_client = (*npc).client as *mut gclient_t;
        if (*npc_client).ps.ammo[weaponData[(*client).ps.weapon as usize].ammoIndex as usize] < 10 {
            let npc_id = ctx.entity_id_of(npc).unwrap();
            Add_Ammo(ctx, npc_id, (*client).ps.weapon, 100);
        }
        //MCG - End

        ctx.world.globals.ucmd.weapon = (*client).ps.weapon as u8;
        ShootThink(ctx);
    }
}

/// Raven `HaveWeapon`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1109-1112`
pub fn HaveWeapon(ctx: &mut GameContext, weapon: c_int) -> qboolean {
    unsafe {
        let client = ctx.world.globals.client;
        (((*client).ps.stats[statIndex_t::STAT_WEAPONS as usize] & (1 << weapon)) != 0) as qboolean
    }
}

/// Raven `EntIsGlass`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1114-1124`
pub fn EntIsGlass(check: &gentity_t) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let check: *const gentity_t = check;
        if !(*check).classname.is_null()
            && Q_stricmp(
                c"func_breakable".as_ptr(),
                (*check).classname as *const c_char,
            ) == 0
            && (*check).count == 1
            && (*check).health <= 100
        {
            return 1;
        }
        0
    }
}

/// Raven `ShotThroughGlass`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1126-1140`
pub fn ShotThroughGlass(
    ctx: &mut GameContext,
    tr: *mut trace_t,
    target: Option<EntityId>,
    spot: vec3_t,
    mask: c_int,
) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let target: *mut gentity_t = ent_ptr(ctx, target);
        let hit = &mut ctx.world.g_entities[(*tr).entityNum as usize] as *mut gentity_t;
        if hit != target && EntIsGlass(ctx.entity(ctx.entity_id_of(hit).unwrap())) != 0 {
            //ok to shoot through breakable glass
            let skip = (*hit).s.number;
            let muzzle = (*tr).endpos;

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    tr,
                    &muzzle as *const vec3_t,
                    std::ptr::null(),
                    std::ptr::null(),
                    &spot as *const vec3_t,
                    skip,
                    mask,
                ),
            );
            return 1;
        }

        0
    }
}

/// Raven `CanShoot`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1150-1221`
pub fn CanShoot(ctx: &mut GameContext, ent: EntityId, shooter: EntityId) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ctx.entity_mut(ent);
        let shooter: *mut gentity_t = ctx.entity_mut(shooter);
        let mut tr: trace_t = std::mem::zeroed();
        let mut muzzle: vec3_t = [0.0; 3];
        let mut spot: vec3_t = [0.0; 3];

        CalcEntitySpot(
            ctx,
            ctx.entity_id_of(shooter),
            spot_t::SPOT_WEAPON,
            &mut muzzle,
        );
        CalcEntitySpot(ctx, ctx.entity_id_of(ent), spot_t::SPOT_ORIGIN, &mut spot); //FIXME preferred target locations for some weapons (feet for R/L)

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &muzzle as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &spot as *const vec3_t,
                (*shooter).s.number,
                MASK_SHOT,
            ),
        );
        let mut traceEnt = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;

        // point blank, baby!
        let shooter_npc = (*shooter).NPC as *mut gNPC_t;
        if tr.startsolid != 0 && !shooter_npc.is_null() && !(*shooter_npc).touchedByPlayer.is_none()
        {
            traceEnt = &mut ctx.world.g_entities[(*shooter_npc).touchedByPlayer.unwrap().index()]
                as *mut gentity_t;
        }

        if ShotThroughGlass(
            ctx,
            &mut tr as *mut trace_t,
            ctx.entity_id_of(ent),
            spot,
            MASK_SHOT,
        ) != 0
        {
            traceEnt = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
        }

        // shot is dead on
        if traceEnt == ent {
            return 1;
        }
        //MCG - Begin
        //ok, can't hit them in center, try their head
        CalcEntitySpot(ctx, ctx.entity_id_of(ent), spot_t::SPOT_HEAD, &mut spot);
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &muzzle as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &spot as *const vec3_t,
                (*shooter).s.number,
                MASK_SHOT,
            ),
        );
        traceEnt = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
        if traceEnt == ent {
            return 1;
        }

        //Actually, we should just check to fire in dir we're facing and if it's close enough,
        //and we didn't hit someone on our own team, shoot
        let diff = [
            spot[0] - tr.endpos[0],
            spot[1] - tr.endpos[1],
            spot[2] - tr.endpos[2],
        ];
        // Raven `random()` (`q_shared.h:1591`, `(rand()&0x7fff)/32767.0`) —
        // the `bg_lib.c` `randSeed` LCG (distinct from the game's own
        // `Q_flrand`/`Q_irand` LCG), reached via `bg_state.rng`.
        let random = ctx.world.bg_state.rng.random();
        if VectorLength(diff) < random * 32.0 {
            return 1;
        }
        //MCG - End
        // shot would hit a non-client
        if (*traceEnt).client.is_null() {
            return 0;
        }

        // shot is blocked by another player

        // he's already dead, so go ahead
        if (*traceEnt).health <= 0 {
            return 1;
        }

        // don't deliberately shoot a teammate
        let traceEnt_client = (*traceEnt).client as *mut gclient_t;
        let shooter_client = (*shooter).client as *mut gclient_t;
        if !traceEnt_client.is_null()
            && !shooter_client.is_null()
            && (*traceEnt_client).playerTeam == (*shooter_client).playerTeam
        {
            return 0;
        }

        // he's just in the wrong place, go ahead
        1
    }
}

/// Raven `NPC_CheckPossibleEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1229-1274`
pub fn NPC_CheckPossibleEnemy(ctx: &mut GameContext, other: Option<EntityId>, vis: visibility_t) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let other: *mut gentity_t = ent_ptr(ctx, other);
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ent_base(ctx);
        let other_id = ent_id_opt(base, other);

        // is he is already our enemy?
        if (*npc).enemy == other_id {
            return;
        }

        if ((*other).flags & FL_NOTARGET) != 0 {
            return;
        }

        // we already have an enemy and this guy is in our FOV, see if this guy would be better
        if !(*npc).enemy.is_none() && vis == visibility_t::VIS_FOV {
            if (*npc_info).enemyLastSeenTime - ctx.world.level.time < 2000 {
                return;
            }
            if ctx.world.globals.enemyVisibility == visibility_t::VIS_UNKNOWN {
                let ent = ent_ptr(ctx, (*npc).enemy);
                let ent_id = ctx.entity_id_of(ent);
                ctx.world.globals.enemyVisibility =
                    NPC_CheckVisibility(ctx, ent_id, CHECK_360 | CHECK_FOV);
            }
            if ctx.world.globals.enemyVisibility == visibility_t::VIS_FOV {
                return;
            }
        }

        if (*npc).enemy.is_none() {
            let npc_id = ctx.entity_id_of(npc).unwrap();
            let other_id = ctx.entity_id_of(other);
            //only take an enemy if you don't have one yet
            G_SetEnemy(ctx, npc_id, other_id);
        }

        if vis == visibility_t::VIS_FOV {
            (*npc_info).enemyLastSeenTime = ctx.world.level.time;
            (*npc_info).enemyLastSeenLocation = (*other).r.currentOrigin;
            (*npc_info).enemyLastHeardTime = 0;
            (*npc_info).enemyLastHeardLocation = [0.0, 0.0, 0.0];
        } else {
            (*npc_info).enemyLastSeenTime = 0;
            (*npc_info).enemyLastSeenLocation = [0.0, 0.0, 0.0];
            (*npc_info).enemyLastHeardTime = ctx.world.level.time;
            (*npc_info).enemyLastHeardLocation = (*other).r.currentOrigin;
        }
    }
}

/// Raven `NPC_AttackDebounceForWeapon`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1288-1331`
pub fn NPC_AttackDebounceForWeapon(ctx: &mut GameContext) -> c_int {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let npc_client = (*npc).client as *mut gclient_t;
        match (*npc_client).ps.weapon {
            WP_SABER => 0,
            _ => (*npc_info).burstSpacing, //was 100 by default
        }
    }
}

/// Raven `NPC_MaxDistSquaredForWeapon`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1334-1392`
pub fn NPC_MaxDistSquaredForWeapon(ctx: &mut GameContext) -> f32 {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        if (*npc_info).stats.shootDistance > 0.0 {
            //overrides default weapon dist
            return (*npc_info).stats.shootDistance * (*npc_info).stats.shootDistance;
        }

        match (*npc).s.weapon {
            WP_BLASTER => 1024.0 * 1024.0, //should be shorter?
            WP_BRYAR_PISTOL => 1024.0 * 1024.0,
            WP_DISRUPTOR => {
                if ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0 {
                    4096.0 * 4096.0
                } else {
                    1024.0 * 1024.0
                }
            }
            WP_SABER => {
                let npc_client = (*npc).client as *mut gclient_t;
                if !npc_client.is_null() && (*npc_client).saber[0].blade[0].lengthMax != 0.0 {
                    //FIXME: account for whether enemy and I are heading towards each other!
                    // C's `1.5` is a double literal: the `*1.5`, the sum, and the
                    // square evaluate in f64, narrowing to float only on return.
                    let reach = (*npc_client).saber[0].blade[0].lengthMax as f64
                        + (*npc).r.maxs[0] as f64 * 1.5;
                    (reach * reach) as f32
                } else {
                    48.0 * 48.0
                }
            }
            _ => 1024.0 * 1024.0, //was 0
        }
    }
}

/// Raven `ValidEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1400-1460`
pub fn ValidEnemy(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ent_ptr(ctx, ent);
        let npc = ctx.world.globals.NPC;

        if ent.is_null() {
            return qfalse;
        }
        if ent == npc {
            return qfalse;
        }

        if ((*ent).flags & FL_NOTARGET) == 0 && (*ent).health > 0 {
            let ent_client = (*ent).client as *mut gclient_t;
            if ent_client.is_null() {
                return qtrue;
            } else if (*ent_client).sess.sessionTeam == TEAM_SPECTATOR {
                //don't go after spectators
                return qfalse;
            } else {
                let mut entTeam: c_int = NPCTEAM_FREE;
                let ent_npc = (*ent).NPC as *mut gNPC_t;
                if !ent_npc.is_null() && !ent_client.is_null() {
                    entTeam = (*ent_client).playerTeam;
                } else if !ent_client.is_null() {
                    if (*ent_client).sess.sessionTeam == TEAM_BLUE {
                        entTeam = NPCTEAM_PLAYER;
                    } else if (*ent_client).sess.sessionTeam == TEAM_RED {
                        entTeam = NPCTEAM_ENEMY;
                    } else {
                        entTeam = NPCTEAM_NEUTRAL;
                    }
                }
                let npc_client = (*npc).client as *mut gclient_t;
                if entTeam == NPCTEAM_FREE
                    || (*npc_client).enemyTeam == NPCTEAM_FREE
                    || entTeam == (*npc_client).enemyTeam
                {
                    if entTeam != (*npc_client).playerTeam {
                        return qtrue;
                    }
                }
            }
        }

        qfalse
    }
}

/// Raven `NPC_EnemyTooFar`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1462-1486`
pub fn NPC_EnemyTooFar(
    ctx: &mut GameContext,
    enemy: Option<EntityId>,
    dist: f32,
    toShoot: qboolean,
) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let enemy: *mut gentity_t = ent_ptr(ctx, enemy);
        let npc = ctx.world.globals.NPC;
        let mut dist = dist;

        if toShoot == qfalse {
            //Not trying to actually press fire button with this check
            let npc_client = (*npc).client as *mut gclient_t;
            if (*npc_client).ps.weapon == WP_SABER {
                //Just have to get to him
                return qfalse;
            }
        }

        if dist == 0.0 {
            let mut vec: vec3_t = [0.0; 3];
            _VectorSubtract((*npc).r.currentOrigin, (*enemy).r.currentOrigin, &mut vec);
            dist = VectorLengthSquared(vec);
        }

        if dist > NPC_MaxDistSquaredForWeapon(ctx) {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `NPC_PickEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1505-1796`
pub fn NPC_PickEnemy(
    ctx: &mut GameContext,
    closestTo: Option<EntityId>,
    enemyTeam: c_int,
    checkVis: qboolean,
    findPlayersFirst: qboolean,
    findClosest: qboolean,
) -> *mut gentity_t {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let closestTo: *mut gentity_t = ent_ptr(ctx, closestTo);
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ent_base(ctx);

        let mut num_choices: usize = 0;
        // §19: a >128th valid candidate silently overruns the stack array in C
        // (UB); here the index panics — unreachable with realistic enemy counts.
        let mut choice: [c_int; 128] = [0; 128];
        let mut closestEnemy: *mut gentity_t = core::ptr::null_mut();
        let mut bestDist: f32 = crate::g_public_consts::Q3_INFINITE as f32;
        let mut failed;
        let mut visChecks = CHECK_360 | CHECK_FOV | CHECK_VISRANGE;
        let mut minVis = visibility_t::VIS_FOV;

        if enemyTeam == NPCTEAM_NEUTRAL {
            return core::ptr::null_mut();
        }

        if (*npc_info).behaviorState == bState_t::BS_STAND_AND_SHOOT
            || (*npc_info).behaviorState == bState_t::BS_HUNT_AND_KILL
        {
            //Formations guys don't require inFov to pick up a target
            //These other behavior states are active battle states and should not
            //use FOV.  FOV checks are for enemies who are patrolling, guarding, etc.
            visChecks &= !CHECK_FOV;
            minVis = visibility_t::VIS_360;
        }

        if findPlayersFirst != qfalse {
            //try to find a player first
            let newenemy = &mut ctx.world.g_entities[0] as *mut gentity_t;
            if !(*newenemy).client.is_null()
                && ((*newenemy).flags & FL_NOTARGET) == 0
                && ((*newenemy).s.eFlags & EF_NODRAW) == 0
                && (*newenemy).health > 0
                && NPC_ValidEnemy(ctx, ctx.entity_id_of(newenemy)) != qfalse
                && ent_id_opt(base, newenemy) != (*npc).lastEnemy
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*newenemy).r.currentOrigin as *const vec3_t,
                        &(*npc).r.currentOrigin as *const vec3_t,
                    ),
                ) != qfalse
            {
                failed = false;
                if ((*npc_info).behaviorState == bState_t::BS_INVESTIGATE
                    || (*npc_info).behaviorState == bState_t::BS_PATROL)
                    && (*npc).enemy.is_none()
                {
                    if InVisrange(ctx, ctx.entity_id_of(newenemy)) == qfalse {
                        failed = true;
                    } else if NPC_CheckVisibility(
                        ctx,
                        ctx.entity_id_of(newenemy),
                        CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                    ) != visibility_t::VIS_FOV
                    {
                        failed = true;
                    }
                }

                if !failed {
                    let mut diff: vec3_t = [0.0; 3];
                    _VectorSubtract(
                        (*closestTo).r.currentOrigin,
                        (*newenemy).r.currentOrigin,
                        &mut diff,
                    );
                    let mut relDist = VectorLengthSquared(diff);
                    let newenemy_client = (*newenemy).client as *mut gclient_t;
                    if (*newenemy_client).hiddenDist > 0.0 {
                        if relDist > (*newenemy_client).hiddenDist * (*newenemy_client).hiddenDist {
                            //out of hidden range
                            if VectorLengthSquared((*newenemy_client).hiddenDir) != 0.0 {
                                //They're only hidden from a certain direction, check
                                let mut normDiff = diff;
                                VectorNormalize(&mut normDiff);
                                let dot = _DotProduct((*newenemy_client).hiddenDir, normDiff);
                                if dot > 0.5 {
                                    //I'm not looking in the right dir toward them to see them
                                    failed = true;
                                } else {
                                    let vtos_str =
                                        vtos(ctx, (*newenemy_client).hiddenDir) as *const c_char;
                                    // PORT-NOTE(varargs-seam): Debug_Printf's ported signature has no
                                    let name_str = cstr_to_str(vtos_str);
                                    let vtos_str_2 = vtos(ctx, normDiff) as *const c_char;
                                    let name_str_2 = cstr_to_str(vtos_str_2);
                                    // variadic slot; pre-format via format! and pass as fmt.
                                    let s = format!(
                                        "{} saw {} trying to hide - hiddenDir {} targetDir {} dot {}\n",
                                        cstr_to_str((*npc).targetname as *const c_char),
                                        cstr_to_str((*newenemy).targetname as *const c_char), name_str, name_str_2,
                                        dot
                                    );
                                    let cs = cstr(&s);
                                    // STAGE-2b: irreducible — raw `&debugNPCAI` alias passed alongside `ctx`
                                    // to the raw-ABI `Debug_Printf`.
                                    let dbg_cvar = &raw mut ctx.world.cvars.debugNPCAI;
                                    Debug_Printf(
                                        ctx,
                                        dbg_cvar,
                                        DEBUG_LEVEL_INFO,
                                        cs.as_ptr() as *mut c_char,
                                    );
                                }
                            } else {
                                failed = true;
                            }
                        } else {
                            let s = format!(
                                "{} saw {} trying to hide - hiddenDist {}\n",
                                cstr_to_str((*npc).targetname as *const c_char),
                                cstr_to_str((*newenemy).targetname as *const c_char),
                                (*newenemy_client).hiddenDist
                            );
                            let cs = cstr(&s);
                            // STAGE-2b: irreducible — raw `&debugNPCAI` alias passed alongside `ctx`
                            // to the raw-ABI `Debug_Printf`.
                            let dbg_cvar = &raw mut ctx.world.cvars.debugNPCAI;
                            Debug_Printf(
                                ctx,
                                dbg_cvar,
                                DEBUG_LEVEL_INFO,
                                cs.as_ptr() as *mut c_char,
                            );
                        }
                    }

                    if !failed {
                        if findClosest != qfalse {
                            if relDist < bestDist
                                && NPC_EnemyTooFar(ctx, ctx.entity_id_of(newenemy), relDist, qfalse)
                                    == qfalse
                            {
                                if checkVis != qfalse {
                                    if NPC_CheckVisibility(
                                        ctx,
                                        ctx.entity_id_of(newenemy),
                                        visChecks,
                                    ) == minVis
                                    {
                                        bestDist = relDist;
                                        closestEnemy = newenemy;
                                    }
                                } else {
                                    bestDist = relDist;
                                    closestEnemy = newenemy;
                                }
                            }
                        } else if NPC_EnemyTooFar(ctx, ctx.entity_id_of(newenemy), 0.0, qfalse)
                            == qfalse
                        {
                            if checkVis != qfalse {
                                if NPC_CheckVisibility(
                                    ctx,
                                    ctx.entity_id_of(newenemy),
                                    CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                                ) == visibility_t::VIS_FOV
                                {
                                    choice[num_choices] = (*newenemy).s.number;
                                    num_choices += 1;
                                }
                            } else {
                                choice[num_choices] = (*newenemy).s.number;
                                num_choices += 1;
                            }
                        }
                    }
                }
            }
        }

        if findClosest != qfalse && !closestEnemy.is_null() {
            return closestEnemy;
        }

        if num_choices != 0 {
            let idx = (ctx.world.bg_state.rng.rand() as usize) % num_choices;
            return &mut ctx.world.g_entities[choice[idx] as usize] as *mut gentity_t;
        }

        num_choices = 0;
        bestDist = crate::g_public_consts::Q3_INFINITE as f32;
        closestEnemy = core::ptr::null_mut();

        for entNum in 0..ctx.world.level.num_entities {
            let newenemy = &mut ctx.world.g_entities[entNum as usize] as *mut gentity_t;

            if newenemy == npc
                || (*newenemy).client.is_null()
                || ((*newenemy).flags & FL_NOTARGET) != 0
                || ((*newenemy).s.eFlags & EF_NODRAW) != 0
                || (*newenemy).health <= 0
            {
                continue;
            }

            let newenemy_client = (*newenemy).client as *mut gclient_t;
            let newenemy_id = ctx.entity_id_of(newenemy);
            let ok = (!newenemy_client.is_null() && NPC_ValidEnemy(ctx, newenemy_id) != qfalse)
                || (newenemy_client.is_null() && (*newenemy).alliedTeam == enemyTeam);
            if !ok {
                continue;
            }

            let npc_client = (*npc).client as *mut gclient_t;
            if (*npc_client).playerTeam == NPCTEAM_PLAYER
                && enemyTeam == NPCTEAM_PLAYER
                && (*newenemy).s.number != 0
            {
                //player allies turning on ourselves?  How?
                //only turn on the player, not other player allies
                continue;
            }

            if ent_id_opt(base, newenemy) == (*npc).lastEnemy {
                continue;
            }

            if trap::InPVS(
                ctx.engine,
                GInPvsArgs::new(
                    &(*newenemy).r.currentOrigin as *const vec3_t,
                    &(*npc).r.currentOrigin as *const vec3_t,
                ),
            ) == qfalse
            {
                continue;
            }

            if ((*npc_info).behaviorState == bState_t::BS_INVESTIGATE
                || (*npc_info).behaviorState == bState_t::BS_PATROL)
                && (*npc).enemy.is_none()
            {
                if InVisrange(ctx, ctx.entity_id_of(newenemy)) == qfalse {
                    continue;
                } else if NPC_CheckVisibility(
                    ctx,
                    ctx.entity_id_of(newenemy),
                    CHECK_360 | CHECK_FOV | CHECK_VISRANGE,
                ) != visibility_t::VIS_FOV
                {
                    continue;
                }
            }

            let mut diff: vec3_t = [0.0; 3];
            _VectorSubtract(
                (*closestTo).r.currentOrigin,
                (*newenemy).r.currentOrigin,
                &mut diff,
            );
            let relDist = VectorLengthSquared(diff);
            if !newenemy_client.is_null() && (*newenemy_client).hiddenDist > 0.0 {
                if relDist > (*newenemy_client).hiddenDist * (*newenemy_client).hiddenDist {
                    if VectorLengthSquared((*newenemy_client).hiddenDir) != 0.0 {
                        let mut normDiff = diff;
                        VectorNormalize(&mut normDiff);
                        let dot = _DotProduct((*newenemy_client).hiddenDir, normDiff);
                        if dot > 0.5 {
                            continue;
                        } else {
                            let vtos_str = vtos(ctx, (*newenemy_client).hiddenDir) as *const c_char;
                            let name_str = cstr_to_str(vtos_str);
                            let vtos_str_2 = vtos(ctx, normDiff) as *const c_char;
                            let name_str_2 = cstr_to_str(vtos_str_2);
                            let s = format!(
                                "{} saw {} trying to hide - hiddenDir {} targetDir {} dot {}\n",
                                cstr_to_str((*npc).targetname as *const c_char),
                                cstr_to_str((*newenemy).targetname as *const c_char),
                                name_str,
                                name_str_2,
                                dot
                            );
                            let cs = cstr(&s);
                            // STAGE-2b: irreducible — raw `&debugNPCAI` alias passed alongside `ctx`
                            // to the raw-ABI `Debug_Printf`.
                            let dbg_cvar = &raw mut ctx.world.cvars.debugNPCAI;
                            Debug_Printf(
                                ctx,
                                dbg_cvar,
                                DEBUG_LEVEL_INFO,
                                cs.as_ptr() as *mut c_char,
                            );
                        }
                    } else {
                        continue;
                    }
                } else {
                    let s = format!(
                        "{} saw {} trying to hide - hiddenDist {}\n",
                        cstr_to_str((*npc).targetname as *const c_char),
                        cstr_to_str((*newenemy).targetname as *const c_char),
                        (*newenemy_client).hiddenDist
                    );
                    let cs = cstr(&s);
                    // STAGE-2b: irreducible — raw `&debugNPCAI` alias passed alongside `ctx`
                    // to the raw-ABI `Debug_Printf`.
                    let dbg_cvar = &raw mut ctx.world.cvars.debugNPCAI;
                    Debug_Printf(ctx, dbg_cvar, DEBUG_LEVEL_INFO, cs.as_ptr() as *mut c_char);
                }
            }

            if findClosest != qfalse {
                if relDist < bestDist
                    && NPC_EnemyTooFar(ctx, ctx.entity_id_of(newenemy), relDist, qfalse) == qfalse
                {
                    if checkVis != qfalse {
                        //FIXME: NPCs need to be able to pick up other NPCs behind them,
                        //but for now, commented out because it was picking up enemies it shouldn't
                        if NPC_CheckVisibility(ctx, ctx.entity_id_of(newenemy), visChecks) == minVis
                        {
                            bestDist = relDist;
                            closestEnemy = newenemy;
                        }
                    } else {
                        bestDist = relDist;
                        closestEnemy = newenemy;
                    }
                }
            } else if NPC_EnemyTooFar(ctx, ctx.entity_id_of(newenemy), 0.0, qfalse) == qfalse {
                if checkVis != qfalse {
                    if NPC_CheckVisibility(
                        ctx,
                        ctx.entity_id_of(newenemy),
                        CHECK_360 | CHECK_VISRANGE,
                    ) as i32
                        >= visibility_t::VIS_360 as i32
                    {
                        choice[num_choices] = (*newenemy).s.number;
                        num_choices += 1;
                    }
                } else {
                    choice[num_choices] = (*newenemy).s.number;
                    num_choices += 1;
                }
            }
        }

        if findClosest != qfalse {
            //FIXME: you can pick up an enemy around a corner this way.
            return closestEnemy;
        }

        if num_choices == 0 {
            return core::ptr::null_mut();
        }

        let idx = (ctx.world.bg_state.rng.rand() as usize) % num_choices;
        &mut ctx.world.g_entities[choice[idx] as usize] as *mut gentity_t
    }
}

/// Raven `NPC_PickAlly`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1804-1893`
pub fn NPC_PickAlly(
    ctx: &mut GameContext,
    facingEachOther: qboolean,
    range: f32,
    ignoreGroup: qboolean,
    movingOnly: qboolean,
) -> *mut gentity_t {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let base = ent_base(ctx);
        let npc_client = (*npc).client as *mut gclient_t;

        let mut closestAlly: *mut gentity_t = core::ptr::null_mut();
        let mut bestDist = range;

        for entNum in 0..ctx.world.level.num_entities {
            let ally = &mut ctx.world.g_entities[entNum as usize] as *mut gentity_t;

            if (*ally).client.is_null() || (*ally).health <= 0 {
                continue;
            }

            let ally_client = (*ally).client as *mut gclient_t;
            if (*ally_client).playerTeam == (*npc_client).playerTeam
                || (*npc_client).playerTeam == NPCTEAM_ENEMY
            {
                //if on same team or if player is disguised as your team
                if ignoreGroup != qfalse {
                    if ent_id_opt(base, ally) == (*npc_client).leader {
                        //reject
                        continue;
                    }
                    if !(*ally_client).leader.is_none()
                        && ent_ptr(ctx, (*ally_client).leader) == npc
                    {
                        //reject
                        continue;
                    }
                }

                if trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*ally).r.currentOrigin as *const vec3_t,
                        &(*npc).r.currentOrigin as *const vec3_t,
                    ),
                ) == qfalse
                {
                    continue;
                }

                if movingOnly != qfalse {
                    //They have to be moving relative to each other
                    if DistanceSquared((*ally_client).ps.velocity, (*npc_client).ps.velocity) == 0.0
                    {
                        continue;
                    }
                }

                let mut diff: vec3_t = [0.0; 3];
                _VectorSubtract((*npc).r.currentOrigin, (*ally).r.currentOrigin, &mut diff);
                let relDist = VectorNormalize(&mut diff);
                if relDist < bestDist {
                    if facingEachOther != qfalse {
                        let mut vf: vec3_t = [0.0; 3];
                        AngleVectors((*ally_client).ps.viewangles, Some(&mut vf), None, None);
                        VectorNormalize(&mut vf);
                        let mut dot = _DotProduct(diff, vf);

                        if dot < 0.5 {
                            //Not facing in dir to me
                            continue;
                        }
                        //He's facing me, am I facing him?
                        AngleVectors((*npc_client).ps.viewangles, Some(&mut vf), None, None);
                        VectorNormalize(&mut vf);
                        dot = _DotProduct(diff, vf);

                        if dot > -0.5 {
                            //I'm not facing opposite of dir to me
                            continue;
                        }
                        //I am facing him
                    }

                    if NPC_CheckVisibility(ctx, ctx.entity_id_of(ally), CHECK_360 | CHECK_VISRANGE)
                        as i32
                        >= visibility_t::VIS_360 as i32
                    {
                        bestDist = relDist;
                        closestAlly = ally;
                    }
                }
            }
        }

        closestAlly
    }
}

/// Raven `NPC_CheckEnemy`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:1895-2102`
pub fn NPC_CheckEnemy(
    ctx: &mut GameContext,
    findNew: qboolean,
    tooFarOk: qboolean,
    setEnemy: qboolean,
) -> *mut gentity_t {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let base = ent_base(ctx);

        let mut forcefindNew = qfalse;
        let mut newEnemy: *mut gentity_t = core::ptr::null_mut();

        if !(*npc).enemy.is_none() {
            let enemy = ent_ptr(ctx, (*npc).enemy);
            if (*enemy).inuse == 0 {
                if setEnemy != qfalse {
                    G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                }
            }
        }

        // if ( NPC->svFlags & SVF_IGNORE_ENEMIES )
        // PORT-NOTE(SVF_IGNORE_ENEMIES): flag not yet ported; oracle-commented `if(0)` faithfully skips this branch.

        if !(*npc).enemy.is_none() {
            let enemy = ent_ptr(ctx, (*npc).enemy);
            if NPC_EnemyTooFar(ctx, ctx.entity_id_of(enemy), 0.0, qfalse) != qfalse {
                if findNew != qfalse {
                    //See if there is a close one and take it if so, else keep this one
                    forcefindNew = qtrue;
                } else if tooFarOk == qfalse {
                    if setEnemy != qfalse {
                        G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                    }
                }
            } else if trap::InPVS(
                ctx.engine,
                GInPvsArgs::new(
                    &(*npc).r.currentOrigin as *const vec3_t,
                    &(*enemy).r.currentOrigin as *const vec3_t,
                ),
            ) == qfalse
            {
                //FIXME: should this be a line-of site check?
                let enemy_client = (*enemy).client as *mut gclient_t;
                if !enemy_client.is_null() && (*enemy_client).hiddenDist != 0.0 {
                    //He ducked into shadow while we weren't looking
                    NPC_LostEnemyDecideChase(ctx);
                }
                //else: not chasing him — logic left commented in oracle, never give him up
            }
        }

        if !(*npc).enemy.is_none() {
            let enemy = ent_ptr(ctx, (*npc).enemy);
            if (*enemy).health <= 0 || ((*enemy).flags & FL_NOTARGET) != 0 {
                if setEnemy != qfalse {
                    G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                }
            }
        }

        let mut closestTo = npc;
        //FIXME: check your defendEnt, if you have one, see if their enemy is different
        //than yours, or, if they don't have one, pick the closest enemy to THEM?
        if !(*npc_info).defendEnt.is_none() {
            //Trying to protect someone
            let defendEnt = ent_ptr(ctx, (*npc_info).defendEnt);
            if (*defendEnt).health > 0 {
                //Still alive, We presume we're close to them, navigation should handle this?
                if !(*defendEnt).enemy.is_none() {
                    //They were shot or acquired an enemy
                    if (*npc).enemy != (*defendEnt).enemy {
                        //They have a different enemy, take it!
                        newEnemy = ent_ptr(ctx, (*defendEnt).enemy);
                        if setEnemy != qfalse {
                            G_SetEnemy(
                                ctx,
                                ctx.entity_id_of(npc).unwrap(),
                                ctx.entity_id_of(newEnemy),
                            );
                        }
                    }
                } else if (*npc).enemy.is_none() {
                    //We don't have an enemy, so find closest to defendEnt
                    closestTo = defendEnt;
                }
            }
        }

        let enemy_dead = (*npc).enemy.is_some() && (*ent_ptr(ctx, (*npc).enemy)).health <= 0;
        if (*npc).enemy.is_none() || enemy_dead || forcefindNew != qfalse {
            //FIXME: NPCs that are moving after an enemy should ignore the can't hit enemy counter- that should only be for NPCs that are standing still
            let mut foundenemy = qfalse;

            if findNew == qfalse {
                if setEnemy != qfalse {
                    (*npc).lastEnemy = (*npc).enemy;
                    G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                }
                return core::ptr::null_mut();
            }

            //If enemy dead or unshootable, look for others on out enemy's team
            let npc_client = (*npc).client as *mut gclient_t;
            if (*npc_client).enemyTeam != NPCTEAM_NEUTRAL {
                //NOTE:  this only checks vis if can't hit enemy for 10 tries, which I suppose
                //means they need to find one that in more than just PVS
                //For now, made it so you ALWAYS have to check VIS
                newEnemy = NPC_PickEnemy(
                    ctx,
                    ctx.entity_id_of(closestTo),
                    (*npc_client).enemyTeam,
                    qtrue,
                    qfalse,
                    qtrue,
                );
                if !newEnemy.is_null() {
                    foundenemy = qtrue;
                    if setEnemy != qfalse {
                        G_SetEnemy(
                            ctx,
                            ctx.entity_id_of(npc).unwrap(),
                            ctx.entity_id_of(newEnemy),
                        );
                    }
                }
            }

            if forcefindNew == qfalse {
                if foundenemy == qfalse {
                    if setEnemy != qfalse {
                        (*npc).lastEnemy = (*npc).enemy;
                        G_ClearEnemy(ctx, ctx.entity_id_of(npc).unwrap());
                    }
                }

                (*npc).cantHitEnemyCounter = 0;
            }
            //FIXME: if we can't find any at all, go into INdependant NPC AI, pursue and kill
        }

        if !(*npc).enemy.is_none() {
            let enemy = ent_ptr(ctx, (*npc).enemy);
            let enemy_client = (*enemy).client as *mut gclient_t;
            if !enemy_client.is_null() && (*enemy_client).playerTeam != 0 {
                let npc_client = (*npc).client as *mut gclient_t;
                if (*npc_client).playerTeam != (*enemy_client).playerTeam {
                    (*npc_client).enemyTeam = (*enemy_client).playerTeam;
                }
            }
        }

        newEnemy
    }
}

/// Raven `NPC_ClearShot`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2110-2144`
pub fn NPC_ClearShot(ctx: &mut GameContext, ent: Option<EntityId>) -> qboolean {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ent_ptr(ctx, ent);
        let npc = ctx.world.globals.NPC;
        if npc.is_null() || ent.is_null() {
            return qfalse;
        }

        let mut muzzle: vec3_t = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();
        CalcEntitySpot(ctx, ctx.entity_id_of(npc), spot_t::SPOT_WEAPON, &mut muzzle);

        // add aim error
        // use weapon instead of specific npc types, although you could add certain npc classes if you wanted
        if (*npc).s.weapon == WP_BLASTER {
            let mins: vec3_t = [-2.0, -2.0, -2.0];
            let maxs: vec3_t = [2.0, 2.0, 2.0];
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &muzzle as *const vec3_t,
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    &(*ent).r.currentOrigin as *const vec3_t,
                    (*npc).s.number,
                    MASK_SHOT,
                ),
            );
        } else {
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &muzzle as *const vec3_t,
                    std::ptr::null(),
                    std::ptr::null(),
                    &(*ent).r.currentOrigin as *const vec3_t,
                    (*npc).s.number,
                    MASK_SHOT,
                ),
            );
        }

        if tr.startsolid != 0 || tr.allsolid != 0 {
            return qfalse;
        }

        if tr.entityNum as c_int == (*ent).s.number {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `NPC_ShotEntity`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2152-2206`
pub fn NPC_ShotEntity(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    impactPos: Option<&mut vec3_t>,
) -> c_int {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let ent: *mut gentity_t = ent_ptr(ctx, ent);
        let npc = ctx.world.globals.NPC;
        let mut muzzle: vec3_t = [0.0; 3];
        let mut targ: vec3_t = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();

        if npc.is_null() || ent.is_null() {
            return qfalse as c_int;
        }

        if (*npc).s.weapon == WP_THERMAL {
            //thermal aims from slightly above head
            //FIXME: what about low-angle shots, rolling the thermal under something?
            let npc_client = (*npc).client as *mut gclient_t;
            CalcEntitySpot(ctx, ctx.entity_id_of(npc), spot_t::SPOT_HEAD, &mut muzzle);
            let angles: vec3_t = [0.0, (*npc_client).ps.viewangles[1], 0.0];
            let mut forward: vec3_t = [0.0; 3];
            AngleVectors(angles, Some(&mut forward), None, None);
            let mut end: vec3_t = [0.0; 3];
            _VectorMA(muzzle, 8.0, forward, &mut end);
            end[2] += 24.0;
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &muzzle as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    (*npc).s.number,
                    MASK_SHOT,
                ),
            );
            muzzle = tr.endpos;
        } else {
            CalcEntitySpot(ctx, ctx.entity_id_of(npc), spot_t::SPOT_WEAPON, &mut muzzle);
        }
        CalcEntitySpot(ctx, ctx.entity_id_of(ent), spot_t::SPOT_CHEST, &mut targ);

        // add aim error
        // use weapon instead of specific npc types, although you could add certain npc classes if you wanted
        if (*npc).s.weapon == WP_BLASTER {
            let mins: vec3_t = [-2.0, -2.0, -2.0];
            let maxs: vec3_t = [2.0, 2.0, 2.0];
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &muzzle as *const vec3_t,
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    &targ as *const vec3_t,
                    (*npc).s.number,
                    MASK_SHOT,
                ),
            );
        } else {
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &muzzle as *const vec3_t,
                    std::ptr::null(),
                    std::ptr::null(),
                    &targ as *const vec3_t,
                    (*npc).s.number,
                    MASK_SHOT,
                ),
            );
        }
        //FIXME: if using a bouncing weapon like the bowcaster, should we check the reflection of the wall, too?
        if let Some(p) = impactPos {
            //they want to know *where* the hit would be, too
            *p = tr.endpos;
        }
        tr.entityNum as c_int
    }
}

/// Raven `NPC_EvaluateShot`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2208-2220`
pub fn NPC_EvaluateShot(ctx: &mut GameContext, hit: c_int, glassOK: qboolean) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        if (*npc).enemy.is_none() {
            return qfalse;
        }

        let enemy = ent_ptr(ctx, (*npc).enemy);
        let hitEnt = &mut ctx.world.g_entities[hit as usize] as *mut gentity_t;
        if hit == (*enemy).s.number
            || ((*hitEnt).r.svFlags & crate::g_public_consts::SVF_GLASS_BRUSH) != 0
        {
            //can hit enemy or will hit glass, so shoot anyway
            return qtrue;
        }
        qfalse
    }
}

/// Raven `NPC_CheckAttack`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2228-2242`
pub fn NPC_CheckAttack(ctx: &mut GameContext, scale: f32) -> qboolean {
    unsafe {
        let npc_info = ctx.world.globals.NPCInfo;
        let mut scale = scale;
        if scale == 0.0 {
            scale = 1.0;
        }

        if ((*npc_info).stats.aggression as f32) * scale < ctx.world.bg_state.rng.flrand(0.0, 4.0) {
            return qfalse;
        }

        if (*npc_info).shotTime > ctx.world.level.time {
            return qfalse;
        }

        qtrue
    }
}

/// Raven `NPC_CheckDefend`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2250-2259`
pub fn NPC_CheckDefend(ctx: &mut GameContext, scale: f32) -> qboolean {
    unsafe {
        let npc_info = ctx.world.globals.NPCInfo;
        let mut scale = scale;
        if scale == 0.0 {
            scale = 1.0;
        }

        if (*npc_info).stats.evasion as f32 > ctx.world.bg_state.rng.random() * 4.0 * scale {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `NPC_CheckCanAttack`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2263-2465`
pub fn NPC_CheckCanAttack(
    ctx: &mut GameContext,
    attack_scale: f32,
    stationary: qboolean,
) -> qboolean {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let client = ctx.world.globals.client;

        let mut attack_scale = attack_scale;
        let mut attack_ok = qfalse;
        let mut dead_on = qfalse;
        let max_aim_off = 128.0 - (16.0 * (*npc_info).stats.aim as f32);

        let enemy = ent_ptr(ctx, (*npc).enemy);
        if ((*enemy).flags & FL_NOTARGET) != 0 {
            return qfalse;
        }

        //FIXME: only check to see if should duck if that provides cover from the
        //enemy!!!
        if attack_scale == 0.0 {
            attack_scale = 1.0;
        }

        //Yaw to enemy
        let mut enemy_org: vec3_t = [0.0; 3];
        let enemy_id = ctx.entity_id_of(enemy);
        CalcEntitySpot(ctx, enemy_id, spot_t::SPOT_HEAD, &mut enemy_org);
        NPC_AimWiggle(ctx, &mut enemy_org);

        let mut muzzle: vec3_t = [0.0; 3];
        let npc_id = ctx.entity_id_of(npc);
        CalcEntitySpot(ctx, npc_id, spot_t::SPOT_WEAPON, &mut muzzle);

        let mut delta: vec3_t = [0.0; 3];
        _VectorSubtract(enemy_org, muzzle, &mut delta);
        let mut angleToEnemy: vec3_t = [0.0; 3];
        vectoangles(delta, &mut angleToEnemy);
        let distanceToEnemy = VectorNormalize(&mut delta);

        let npc_npc = (*npc).NPC as *mut gNPC_t;
        (*npc_npc).desiredYaw = angleToEnemy[YAW];
        NPC_UpdateFiringAngles(ctx, qfalse, qtrue);

        let enemy_id_2 = ctx.entity_id_of(enemy);
        if NPC_EnemyTooFar(ctx, enemy_id_2, distanceToEnemy * distanceToEnemy, qtrue) != qfalse {
            //Too far away?  Do not attack
            return qfalse;
        }

        if (*client).ps.weaponTime > 0 {
            //already waiting for a shot to fire
            (*npc_npc).desiredPitch = angleToEnemy[PITCH];
            NPC_UpdateFiringAngles(ctx, qtrue, qfalse);
            return qfalse;
        }

        if ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0 {
            return qfalse;
        }

        (*npc_info).enemyLastVisibility = ctx.world.globals.enemyVisibility;
        let enemy_id_3 = ctx.entity_id_of(enemy);
        //See if they're in our FOV and we have a clear shot to them
        ctx.world.globals.enemyVisibility =
            NPC_CheckVisibility(ctx, enemy_id_3, CHECK_360 | CHECK_FOV);

        if ctx.world.globals.enemyVisibility as c_int >= visibility_t::VIS_FOV as c_int {
            //He's in our FOV
            attack_ok = qtrue;

            //Check to duck
            let enemy_client = (*enemy).client as *mut gclient_t;
            if !enemy_client.is_null() {
                if (*enemy).enemy == ent_id_opt(ent_base(ctx), npc) {
                    if ((*enemy_client).buttons & BUTTON_ATTACK) != 0 {
                        //FIXME: determine if enemy fire angles would hit me or get close
                        if NPC_CheckDefend(ctx, 1.0) != qfalse {
                            //duck and don't shoot
                            attack_ok = qfalse;
                            ctx.world.globals.ucmd.upmove = -127;
                        }
                    }
                }
            }

            let mut hitspot: vec3_t = [0.0; 3];
            let mut traceEnt: *mut gentity_t = core::ptr::null_mut();
            let mut tr: trace_t = std::mem::zeroed();
            if attack_ok != qfalse {
                //are we gonna hit him
                //NEW: use actual forward facing
                let mut forward: vec3_t = [0.0; 3];
                AngleVectors((*client).ps.viewangles, Some(&mut forward), None, None);
                _VectorMA(muzzle, distanceToEnemy, forward, &mut hitspot);
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &muzzle as *const vec3_t,
                        std::ptr::null(),
                        std::ptr::null(),
                        &hitspot as *const vec3_t,
                        (*npc).s.number,
                        MASK_SHOT,
                    ),
                );
                let enemy_id_4 = ctx.entity_id_of(enemy);
                ShotThroughGlass(ctx, &mut tr as *mut trace_t, enemy_id_4, hitspot, MASK_SHOT);

                traceEnt = &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;

                hitspot = tr.endpos;

                let traceEnt_client = (*traceEnt).client as *mut gclient_t;
                let npc_client = (*npc).client as *mut gclient_t;
                if traceEnt == enemy
                    || (!traceEnt_client.is_null()
                        && !npc_client.is_null()
                        && (*npc_client).enemyTeam != 0
                        && (*npc_client).enemyTeam == (*traceEnt_client).playerTeam)
                {
                    dead_on = qtrue;
                } else {
                    attack_scale *= 0.5;
                    if (*npc_client).playerTeam != 0
                        && !traceEnt.is_null()
                        && !traceEnt_client.is_null()
                        && (*traceEnt_client).playerTeam != 0
                        && (*npc_client).playerTeam == (*traceEnt_client).playerTeam
                    {
                        //Don't shoot our own team
                        attack_ok = qfalse;
                    }
                }
            }

            if attack_ok != qfalse {
                //ok, now adjust pitch aim
                _VectorSubtract(hitspot, muzzle, &mut delta);
                vectoangles(delta, &mut angleToEnemy);
                (*npc_npc).desiredPitch = angleToEnemy[PITCH];
                NPC_UpdateFiringAngles(ctx, qtrue, qfalse);

                if dead_on == qfalse {
                    //We're not going to hit him directly, try a suppressing fire
                    //see if where we're going to shoot is too far from his origin
                    if !traceEnt.is_null()
                        && ((*traceEnt).health <= 30
                            || EntIsGlass(ctx.entity(ctx.entity_id_of(traceEnt).unwrap())) != 0)
                    {
                        //easy to kill - go for it
                        //rwwFIXMEFIXME: ExplodeDeath_Wait? — dead code path, faithfully skipped (if(0) in oracle)
                    } else {
                        let mut forward: vec3_t = [0.0; 3];
                        AngleVectors((*client).ps.viewangles, Some(&mut forward), None, None);
                        _VectorMA(muzzle, distanceToEnemy, forward, &mut hitspot);
                        let mut diff: vec3_t = [0.0; 3];
                        _VectorSubtract(hitspot, enemy_org, &mut diff);
                        let mut aim_off = VectorLength(diff);
                        if aim_off > ctx.world.bg_state.rng.random() * max_aim_off {
                            //FIXME: use aim value to allow poor aim?
                            attack_scale *= 0.75;
                            //see if where we're going to shoot is too far from his head
                            _VectorSubtract(hitspot, enemy_org, &mut diff);
                            aim_off = VectorLength(diff);
                            if aim_off > ctx.world.bg_state.rng.random() * max_aim_off {
                                attack_ok = qfalse;
                            }
                        }
                        attack_scale *= (max_aim_off - aim_off + 1.0) / max_aim_off;
                    }
                }
            }
        } else {
            //Update pitch anyway
            (*npc_npc).desiredPitch = angleToEnemy[PITCH];
            NPC_UpdateFiringAngles(ctx, qtrue, qfalse);
        }

        if attack_ok != qfalse {
            if NPC_CheckAttack(ctx, attack_scale) != qfalse {
                //check aggression to decide if we should shoot
                ctx.world.globals.enemyVisibility = visibility_t::VIS_SHOOT;
                WeaponThink(ctx, qtrue);
            } else {
                attack_ok = qfalse;
            }
        }

        attack_ok
    }
}

/// Raven `IdealDistance`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2475-2503`
pub fn IdealDistance(ctx: &mut GameContext, self_: EntityId) -> f32 {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        let mut ideal = 225.0 - 20.0 * (*npc_info).stats.aggression as f32;
        match (*npc).s.weapon {
            WP_ROCKET_LAUNCHER => {
                ideal += 200.0;
            }
            WP_THERMAL => {
                ideal += 50.0;
            }
            _ => {}
        }

        ideal
    }
}

/// Raven `SP_point_combat`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2516-2546`
pub fn SP_point_combat(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let numCombatPoints = ctx.world.level.numCombatPoints as usize;
        if numCombatPoints >= MAX_COMBAT_POINTS {
            let s = format!(
                "{}ERROR:  Too many combat points, limit is {}\n",
                S_COLOR_RED.to_str().unwrap(),
                MAX_COMBAT_POINTS
            );
            Com_Printf(cstr(&s).as_ptr());
            G_FreeEntity(ctx, ctx.entity_id_of(self_));
            return;
        }

        (*self_).s.origin[2] += 0.125;
        G_SetOrigin(&mut *(self_), (*self_).s.origin);
        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(self_),
        );

        if G_CheckInSolid(ctx, ctx.entity_id_of(self_).unwrap(), 1) != 0 {
            let s = format!(
                "{}ERROR: combat point at {} in solid!\n",
                S_COLOR_RED.to_str().unwrap(),
                cstr_to_str(vtos(ctx, (*self_).r.currentOrigin))
            );
            Com_Printf(cstr(&s).as_ptr());
        }

        ctx.world.level.combatPoints[numCombatPoints].origin = (*self_).r.currentOrigin;
        ctx.world.level.combatPoints[numCombatPoints].flags = (*self_).spawnflags;
        ctx.world.level.combatPoints[numCombatPoints].occupied = 0;

        ctx.world.level.numCombatPoints += 1;

        G_FreeEntity(ctx, ctx.entity_id_of(self_));
    }
}

/// Raven `CP_FindCombatPointWaypoints`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2548-2562`
pub fn CP_FindCombatPointWaypoints(ctx: &mut GameContext) {
    unsafe {
        let numCombatPoints = ctx.world.level.numCombatPoints as usize;
        for i in 0..numCombatPoints {
            let origin = ctx.world.level.combatPoints[i].origin;
            ctx.world.level.combatPoints[i].waypoint =
                NAV_FindClosestWaypointForPoint2(ctx, origin);

            if ctx.world.level.combatPoints[i].waypoint == WAYPOINT_NONE {
                let cp_origin = ctx.world.level.combatPoints[i].origin;
                let vtos_str = vtos(ctx, cp_origin);
                let name_str = cstr_to_str(vtos_str);
                let s = format!(
                    "{}ERROR: Combat Point at {} has no waypoint!\n",
                    S_COLOR_RED.to_str().unwrap(),
                    name_str
                );
                Com_Printf(cstr(&s).as_ptr());
            }
        }
    }
}

// Raven `CP_*` request-flag bits: `crate::npc::combat_point_flags`
// (`b_local.h:243-261`).

/// `combatPt_t` — local mirror of the oracle's file-local `typedef struct
/// { float dist; int index; } combatPt_t` used only as the collector's
/// scratch array.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2569-2574`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CombatPt {
    dist: f32,
    index: c_int,
}

const MAX_COMBAT_POINTS_LOCAL: usize = MAX_COMBAT_POINTS;

/// Raven `NPC_CollectCombatPoints`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2575-2644`
pub fn NPC_CollectCombatPoints(
    ctx: &mut GameContext,
    origin: vec3_t,
    radius: f32,
    points: *mut CombatPt,
    flags: c_int,
) -> c_int {
    unsafe {
        let radiusSqr = radius * radius;
        let mut bestDistance = crate::g_public_consts::Q3_INFINITE as f32;
        let mut bestPoint: c_int = 0;
        let mut numPoints: c_int = 0;

        //Collect all nearest
        let numCombatPoints = ctx.world.level.numCombatPoints as usize;
        for i in 0..numCombatPoints {
            if numPoints as usize >= MAX_COMBAT_POINTS_LOCAL {
                break;
            }

            //Must be vacant
            if ctx.world.level.combatPoints[i].occupied != 0 {
                continue;
            }

            //If we want a duck space, make sure this is one
            if (flags & CP_DUCK) != 0 && (ctx.world.level.combatPoints[i].flags & CPF_DUCK) != 0 {
                continue;
            }

            //If we want a duck space, make sure this is one
            if (flags & CP_FLEE) != 0 && (ctx.world.level.combatPoints[i].flags & CPF_FLEE) != 0 {
                continue;
            }

            ///Make sure this is an investigate combat point
            if (flags & CP_INVESTIGATE) != 0
                && (ctx.world.level.combatPoints[i].flags & CPF_INVESTIGATE) != 0
            {
                continue;
            }

            //Squad points are only valid if we're looking for them
            if (ctx.world.level.combatPoints[i].flags & CPF_SQUAD) != 0 && (flags & CP_SQUAD) == 0 {
                continue;
            }

            if (flags & CP_NO_PVS) != 0 {
                //must not be within PVS of mu current origin
                if trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &origin as *const vec3_t,
                        &ctx.world.level.combatPoints[i].origin as *const vec3_t,
                    ),
                ) != qfalse
                {
                    continue;
                }
            }

            let distance = if (flags & CP_HORZ_DIST_COLL) != 0 {
                DistanceHorizontalSquared(origin, ctx.world.level.combatPoints[i].origin)
            } else {
                DistanceSquared(origin, ctx.world.level.combatPoints[i].origin)
            };

            if distance < radiusSqr {
                if distance < bestDistance {
                    bestDistance = distance;
                    bestPoint = numPoints;
                }

                (*points.add(numPoints as usize)).dist = distance;
                (*points.add(numPoints as usize)).index = i as c_int;
                numPoints += 1;
            }
        }

        let _ = bestPoint;
        numPoints //bestPoint
    }
}

/// Raven `NPC_FindCombatPoint`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2657-2874`
pub fn NPC_FindCombatPoint(
    ctx: &mut GameContext,
    position: vec3_t,
    avoidPosition: vec3_t,
    enemyPosition: vec3_t,
    flags: c_int,
    avoidDist: f32,
    ignorePoint: c_int,
) -> c_int {
    unsafe {
        const MIN_AVOID_DOT: f32 = 0.75;
        const MIN_AVOID_DISTANCE: f32 = 128.0;
        const MIN_AVOID_DISTANCE_SQUARED: f32 = MIN_AVOID_DISTANCE * MIN_AVOID_DISTANCE;
        const CP_COLLECT_RADIUS: f32 = 512.0;

        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        let mut points: [CombatPt; MAX_COMBAT_POINTS_LOCAL] = [CombatPt {
            dist: 0.0,
            index: 0,
        }; MAX_COMBAT_POINTS_LOCAL];
        let mut best: c_int = -1;
        let mut bestCost: c_int = crate::g_public_consts::Q3_INFINITE;
        let mut waypoint: c_int = WAYPOINT_NONE;
        let mut collRad = CP_COLLECT_RADIUS;
        let mut modifiedAvoidDist = avoidDist;

        if modifiedAvoidDist <= 0.0 {
            modifiedAvoidDist = MIN_AVOID_DISTANCE_SQUARED;
        } else {
            modifiedAvoidDist *= modifiedAvoidDist;
        }

        if (flags & CP_HAS_ROUTE) != 0 || (flags & CP_NEAREST) != 0 {
            //going to be doing macro nav tests
            if (*npc).waypoint == WAYPOINT_NONE {
                let npc_id = ctx.entity_id_of(npc).unwrap();
                waypoint = NAV_GetNearestNode(ctx, npc_id, (*npc).lastWaypoint);
            } else {
                waypoint = (*npc).waypoint;
            }
        }

        //Collect our nearest points
        if (flags & CP_NO_PVS) != 0 {
            //much larger radius since most will be dropped?
            collRad = CP_COLLECT_RADIUS * 4.0;
        }
        let numPoints =
            NPC_CollectCombatPoints(ctx, enemyPosition, collRad, points.as_mut_ptr(), flags); //position

        for j in 0..(numPoints as usize) {
            let i = points[j].index as usize;
            let pdist = points[j].dist;

            //Must not be one we want to ignore
            if i as c_int == ignorePoint {
                continue;
            }

            //FIXME: able to mark certain ones as too dangerous to go to for now?  Like a tripmine/thermal/detpack is near or something?
            //If we need a cover point, check this point
            if (flags & CP_COVER) != 0
                && NPC_ClearLOS(ctx, ctx.world.level.combatPoints[i].origin, enemyPosition)
                    != qfalse
            {
                continue;
            }

            //Need a clear LOS to our target... and be within shot range to enemy position (FIXME: make this a separate CS_ flag? and pass in a range?)
            if (flags & CP_CLEAR) != 0 {
                let enemy = ent_ptr(ctx, (*npc).enemy);
                let enemy_id = ctx.entity_id_of(enemy);
                if NPC_ClearLOS3(ctx, ctx.world.level.combatPoints[i].origin, enemy_id) == qfalse {
                    continue;
                }
                let dist = if (*npc).s.weapon == WP_THERMAL {
                    //horizontal
                    DistanceHorizontalSquared(
                        ctx.world.level.combatPoints[i].origin,
                        (*enemy).r.currentOrigin,
                    )
                } else {
                    //actual
                    DistanceSquared(
                        ctx.world.level.combatPoints[i].origin,
                        (*enemy).r.currentOrigin,
                    )
                };
                if dist > (*npc_info).stats.visrange * (*npc_info).stats.visrange {
                    continue;
                }
            }

            //Avoid this position?
            if (flags & CP_AVOID) != 0
                && DistanceSquared(ctx.world.level.combatPoints[i].origin, position)
                    < modifiedAvoidDist
            {
                //was using MIN_AVOID_DISTANCE_SQUARED, not passed in modifiedAvoidDist
                continue;
            }

            //Try to find a point closer to the enemy than where we are
            if (flags & CP_APPROACH_ENEMY) != 0 {
                if (flags & CP_HORZ_DIST_COLL) != 0 {
                    if pdist > DistanceHorizontalSquared(position, enemyPosition) {
                        continue;
                    }
                } else if pdist > DistanceSquared(position, enemyPosition) {
                    continue;
                }
            }
            //Try to find a point farther from the enemy than where we are
            if (flags & CP_RETREAT) != 0 {
                if (flags & CP_HORZ_DIST_COLL) != 0 {
                    if pdist < DistanceHorizontalSquared(position, enemyPosition) {
                        //it's closer, don't use it
                        continue;
                    }
                } else if pdist < DistanceSquared(position, enemyPosition) {
                    //it's closer, don't use it
                    continue;
                }
            }

            //We want a point on other side of the enemy from current pos
            if (flags & CP_FLANK) != 0 {
                let mut eDir2Me: vec3_t = [0.0; 3];
                _VectorSubtract(position, enemyPosition, &mut eDir2Me);
                VectorNormalize(&mut eDir2Me);

                let mut eDir2CP: vec3_t = [0.0; 3];
                _VectorSubtract(
                    ctx.world.level.combatPoints[i].origin,
                    enemyPosition,
                    &mut eDir2CP,
                );
                VectorNormalize(&mut eDir2CP);

                let dot = _DotProduct(eDir2Me, eDir2CP);

                //Not far enough behind enemy from current pos
                if dot >= 0.4 {
                    continue;
                }
            }

            //See if we're trying to avoid our enemy
            //FIXME: this needs to check for the waypoint you'll be taking to get to that combat point
            if (flags & CP_AVOID_ENEMY) != 0 {
                let mut eDir: vec3_t = [0.0; 3];
                _VectorSubtract(position, enemyPosition, &mut eDir);
                VectorNormalize(&mut eDir);

                let wpOrg = ctx.world.level.combatPoints[i].origin;

                let mut gDir: vec3_t = [0.0; 3];
                _VectorSubtract(position, wpOrg, &mut gDir);
                VectorNormalize(&mut gDir);

                let dot = _DotProduct(gDir, eDir);

                //Don't want to run at enemy
                if dot >= MIN_AVOID_DOT {
                    continue;
                }

                //Can't be too close to the enemy
                if DistanceSquared(wpOrg, enemyPosition) < modifiedAvoidDist {
                    continue;
                }
            }

            //Okay, now make sure it's not blocked
            let mut tr: trace_t = std::mem::zeroed();
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &ctx.world.level.combatPoints[i].origin as *const vec3_t,
                    &(*npc).r.mins as *const vec3_t,
                    &(*npc).r.maxs as *const vec3_t,
                    &ctx.world.level.combatPoints[i].origin as *const vec3_t,
                    (*npc).s.number,
                    (*npc).clipmask,
                ),
            );
            if tr.allsolid != 0 || tr.startsolid != 0 {
                continue;
            }

            //we must have a route to the combat point
            if (flags & CP_HAS_ROUTE) != 0 {
                if waypoint == WAYPOINT_NONE
                    || ctx.world.level.combatPoints[i].waypoint == WAYPOINT_NONE
                    || trap::Nav_GetBestNodeAltRoute2(
                        ctx.engine,
                        GNavGetbestnodealt2Args::new(
                            waypoint,
                            ctx.world.level.combatPoints[i].waypoint,
                            NODE_NONE,
                        ),
                    ) == WAYPOINT_NONE
                {
                    //can't possibly have a route to any OR can't possibly have a route to this one OR don't have a route to this one
                    if NAV_ClearPathToPoint(
                        ctx,
                        ctx.entity_id_of(npc).unwrap(),
                        (*npc).r.mins,
                        (*npc).r.maxs,
                        ctx.world.level.combatPoints[i].origin,
                        (*npc).clipmask,
                        ENTITYNUM_NONE_LOCAL,
                    ) == qfalse
                    {
                        //don't even have a clear straight path to this one
                        continue;
                    }
                }
            }

            //We want the one with the shortest path from current pos
            if (flags & CP_NEAREST) != 0
                && waypoint != WAYPOINT_NONE
                && ctx.world.level.combatPoints[i].waypoint != WAYPOINT_NONE
            {
                let cost = trap::Nav_GetPathCost(
                    ctx.engine,
                    GNavGetpathcostArgs::new(waypoint, ctx.world.level.combatPoints[i].waypoint),
                );
                if cost < bestCost {
                    bestCost = cost;
                    best = i as c_int;
                }
                continue;
            }

            //we want the combat point closest to the enemy
            //if ( flags & CP_CLOSEST )
            //they are sorted by this distance, so the first one to get this far is the closest
            return i as c_int;
        }

        best
    }
}

// Raven `ENTITYNUM_NONE` (`MAX_GENTITIES - 1`) — file-local alias of the
// canonical `mp_qshared::shared::ENTITYNUM_NONE` (kept to avoid churning this
// file's use sites).
const ENTITYNUM_NONE_LOCAL: c_int = mp_qshared::shared::ENTITYNUM_NONE;

// Raven `CPF_DUCK`/`CPF_FLEE`/`CPF_INVESTIGATE`/`CPF_SQUAD`
// (`combatPoint_t::flags` bits): `crate::npc::combat_point_flags`
// (`b_local.h:264-267`).

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy).

// Raven `WORLD_SIZE` (`MAX_WORLD_COORD - MIN_WORLD_COORD`, `64*1024 -
// (-64*1024)`) — not yet ported as a central const.
// Source: `oracle/codemp/game/q_shared.h:18-20`
pub const WORLD_SIZE: f32 = 131072.0;

/// Raven `NPC_FindSquadPoint`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2882-2915`
pub fn NPC_FindSquadPoint(ctx: &mut GameContext, position: vec3_t) -> c_int {
    let mut nearestDist = WORLD_SIZE * WORLD_SIZE;
    let mut nearestPoint: c_int = -1;

    //float			playerDist = DistanceSquared( g_entities[0].currentOrigin, NPC->r.currentOrigin );

    let numCombatPoints = ctx.world.level.numCombatPoints as usize;
    for i in 0..numCombatPoints {
        let cp = ctx.world.level.combatPoints[i];
        //Squad points are only valid if we're looking for them
        if (cp.flags & CPF_SQUAD) == 0 {
            continue;
        }

        //Must be vacant
        if cp.occupied != 0 {
            continue;
        }

        let dist = DistanceSquared(position, cp.origin);

        //The point cannot take us past the player
        //if ( dist > ( playerDist * DotProduct( dirToPlayer, playerDir ) ) )	//FIXME: Retain this

        //See if this is closer than the others
        if dist < nearestDist {
            nearestPoint = i as c_int;
            nearestDist = dist;
        }
    }

    nearestPoint
}

/// Raven `NPC_ReserveCombatPoint`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2923-2937`
pub fn NPC_ReserveCombatPoint(ctx: &mut GameContext, combatPointID: c_int) -> qboolean {
    //Make sure it's valid
    // §19: Raven only guards the upper bound; a -1 id reads combatPoints[-1] (UB,
    // reads as not-occupied → returns qfalse). We reject negatives to that same effect.
    if combatPointID > ctx.world.level.numCombatPoints || combatPointID < 0 {
        return 0;
    }

    //Make sure it's not already occupied
    if ctx.world.level.combatPoints[combatPointID as usize].occupied != 0 {
        return 0;
    }

    //Reserve it
    ctx.world.level.combatPoints[combatPointID as usize].occupied = 1;

    1
}

/// Raven `NPC_FreeCombatPoint`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2945-2963`
pub fn NPC_FreeCombatPoint(
    ctx: &mut GameContext,
    combatPointID: c_int,
    failed: qboolean,
) -> qboolean {
    unsafe {
        let npc_info = ctx.world.globals.NPCInfo;

        if failed != qfalse {
            //remember that this one failed for us
            (*npc_info).lastFailedCombatPoint = combatPointID;
        }
        //Make sure it's valid
        // §19: Raven only guards the upper bound; a -1 id reads combatPoints[-1] (UB,
        // reads as not-occupied → returns qfalse). We reject negatives to that same effect.
        if combatPointID > ctx.world.level.numCombatPoints || combatPointID < 0 {
            return qfalse;
        }

        //Make sure it's currently occupied
        if ctx.world.level.combatPoints[combatPointID as usize].occupied == qfalse {
            return qfalse;
        }

        //Free it
        ctx.world.level.combatPoints[combatPointID as usize].occupied = qfalse;

        qtrue
    }
}

/// Raven `NPC_SetCombatPoint`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2971-2985`
pub fn NPC_SetCombatPoint(ctx: &mut GameContext, combatPointID: c_int) -> qboolean {
    unsafe {
        let npc_info = ctx.world.globals.NPCInfo;

        //Free a combat point if we already have one
        if (*npc_info).combatPoint != -1 {
            NPC_FreeCombatPoint(ctx, (*npc_info).combatPoint, qfalse);
        }

        if NPC_ReserveCombatPoint(ctx, combatPointID) == qfalse {
            return qfalse;
        }

        (*npc_info).combatPoint = combatPointID;

        qtrue
    }
}

/// Raven `NPC_SearchForWeapons`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:2988-3045`
pub fn NPC_SearchForWeapons(ctx: &mut GameContext) -> *mut gentity_t {
    unsafe {
        let npc = ctx.world.globals.NPC;

        let mut bestFound: *mut gentity_t = core::ptr::null_mut();
        let mut bestDist = crate::g_public_consts::Q3_INFINITE as f32;

        let num_entities = ctx.world.level.num_entities as usize;
        for i in 0..num_entities {
            if ctx.world.g_entities[i].inuse == 0 {
                continue;
            }

            let found = &mut ctx.world.g_entities[i] as *mut gentity_t;

            //FIXME: Also look for ammo_racks that have weapons on them?
            if (*found).s.eType != ET_ITEM as c_int {
                continue;
            }
            if (*(*found).item).giType != IT_WEAPON {
                continue;
            }
            if ((*found).s.eFlags & EF_NODRAW) != 0 {
                continue;
            }
            if CheckItemCanBePickedUpByNPC(
                ctx,
                ctx.entity_id_of(found).unwrap(),
                ctx.entity_id_of(npc).unwrap(),
            ) != qfalse
            {
                if trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*found).r.currentOrigin as *const vec3_t,
                        &(*npc).r.currentOrigin as *const vec3_t,
                    ),
                ) != qfalse
                {
                    let dist = DistanceSquared((*found).r.currentOrigin, (*npc).r.currentOrigin);
                    if dist < bestDist {
                        if trap::Nav_GetBestPathBetweenEnts(
                            ctx.engine,
                            GNavGetbestpathbetweenentsArgs::new(npc, found, NF_CLEAR_PATH_LOCAL),
                        ) == qfalse as c_int
                            || trap::Nav_GetBestNodeAltRoute2(
                                ctx.engine,
                                GNavGetbestnodealt2Args::new(
                                    (*npc).waypoint,
                                    (*found).waypoint,
                                    NODE_NONE,
                                ),
                            ) == WAYPOINT_NONE
                        {
                            //can't possibly have a route to any OR can't possibly have a route to this one OR don't have a route to this one
                            if NAV_ClearPathToPoint(
                                ctx,
                                ctx.entity_id_of(npc).unwrap(),
                                (*npc).r.mins,
                                (*npc).r.maxs,
                                (*found).r.currentOrigin,
                                (*npc).clipmask,
                                ENTITYNUM_NONE_LOCAL,
                            ) != qfalse
                            {
                                //have a clear straight path to this one
                                bestDist = dist;
                                bestFound = found;
                            }
                        } else {
                            //can nav to it
                            bestDist = dist;
                            bestFound = found;
                        }
                    }
                }
            }
        }

        bestFound
    }
}

// Raven `NF_CLEAR_PATH` (`g_nav.h`, nav-flags bit) — not yet ported as a
// central const; inlined here from the header value.
// Source: `oracle/codemp/game/g_nav.h:36`
const NF_CLEAR_PATH_LOCAL: c_int = 0x0000_0002;

/// Raven `NPC_SetPickUpGoal`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:3047-3058`
pub fn NPC_SetPickUpGoal(ctx: &mut GameContext, foundWeap: Option<EntityId>) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let foundWeap: *mut gentity_t = ent_ptr(ctx, foundWeap);
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        //NPCInfo->goalEntity = foundWeap;
        let mut org = (*foundWeap).r.currentOrigin;
        org[2] += 24.0 - ((*foundWeap).r.mins[2] * -1.0); //adjust the origin so that I am on the ground
        NPC_SetMoveGoal(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            org,
            ((*foundWeap).r.maxs[0] * 0.75) as c_int,
            qfalse,
            -1,
            ctx.entity_id_of(foundWeap),
        );
        let tempGoal = ent_ptr(ctx, (*npc_info).tempGoal);
        (*tempGoal).waypoint = (*foundWeap).waypoint;
        (*npc_info).tempBehavior = bState_t::BS_DEFAULT;
        (*npc_info).squadState = SQUAD_TRANSITION_LOCAL;
    }
}

// Raven `SQUAD_TRANSITION` (`squadState_t`-family int, `b_local.h`) — not yet
// ported as a central const; per-file precedent (`NPC_AI_Grenadier.rs`/
// `NPC_AI_Sniper.rs` each keep their own private copy).
const SQUAD_TRANSITION_LOCAL: i32 = 4;

/// Raven `NPC_CheckGetNewWeapon`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:3060-3096`
pub fn NPC_CheckGetNewWeapon(ctx: &mut GameContext) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;

        if (*npc).s.weapon == WP_NONE && !(*npc).enemy.is_none() {
            //if running away because dropped weapon...
            if !(*npc_info).goalEntity.is_none() && (*npc_info).goalEntity == (*npc_info).tempGoal {
                let goalEntity = ent_ptr(ctx, (*npc_info).goalEntity);
                if !(*goalEntity).enemy.is_none() {
                    let goal_enemy = ent_ptr(ctx, (*goalEntity).enemy);
                    if (*goal_enemy).inuse == 0 {
                        //maybe was running at a weapon that was picked up
                        (*npc_info).goalEntity = None;
                    }
                }
            }
            if TIMER_Done(
                ctx,
                ctx.entity_id_of(npc),
                c"panic".as_ptr() as *const c_char,
            ) != qfalse
                && (*npc_info).goalEntity.is_none()
            {
                //need a weapon, any lying around?
                let foundWeap = NPC_SearchForWeapons(ctx);
                if !foundWeap.is_null() {
                    //try to nav to it
                    NPC_SetPickUpGoal(ctx, ctx.entity_id_of(foundWeap));
                }
            }
        }
    }
}

/// Raven `NPC_AimAdjust`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:3098-3129`
pub fn NPC_AimAdjust(ctx: &mut GameContext, change: c_int) {
    unsafe {
        let npc = ctx.world.globals.NPC;
        let npc_info = ctx.world.globals.NPCInfo;
        let g_spskill = ctx.world.cvars.g_spskill.integer;

        if TIMER_Exists(
            ctx,
            ctx.entity_id_of(npc),
            c"aimDebounce".as_ptr() as *const c_char,
        ) == qfalse
        {
            let debounce = 500 + (3 - g_spskill) * 100;
            let npc_id = ctx.entity_id_of(npc);
            let delay = ctx.world.bg_state.rng.Q_irand(debounce, debounce + 1000);
            TIMER_Set(ctx, npc_id, c"aimDebounce".as_ptr() as *const c_char, delay);
            return;
        }
        if TIMER_Done(
            ctx,
            ctx.entity_id_of(npc),
            c"aimDebounce".as_ptr() as *const c_char,
        ) != qfalse
        {
            (*npc_info).currentAim += change;
            if (*npc_info).currentAim > (*npc_info).stats.aim {
                //can never be better than max aim
                (*npc_info).currentAim = (*npc_info).stats.aim;
            } else if (*npc_info).currentAim < -30 {
                //can never be worse than this
                (*npc_info).currentAim = -30;
            }

            //Com_Printf( "%s new aim = %d\n", NPC->NPC_type, NPCInfo->currentAim );

            let debounce = 500 + (3 - g_spskill) * 100;
            let npc_id = ctx.entity_id_of(npc);
            let delay = ctx.world.bg_state.rng.Q_irand(debounce, debounce + 1000);
            TIMER_Set(ctx, npc_id, c"aimDebounce".as_ptr() as *const c_char, delay);
        }
    }
}

/// Raven `G_AimSet`.
///
/// Source: `oracle/codemp/game/NPC_combat.c:3131-3145`
pub fn G_AimSet(ctx: &mut GameContext, self_: EntityId, aim: c_int) {
    unsafe {
        // STAGE-1: EntityId/Option params, raw body re-derived verbatim (Stage-2 debt).
        let self_: *mut gentity_t = ctx.entity_mut(self_);
        let npc = (*self_).NPC as *mut gNPC_t;
        if !npc.is_null() {
            (*npc).currentAim = aim;
            //Com_Printf( "%s new aim = %d\n", self->NPC_type, self->NPC->currentAim );

            let g_spskill = ctx.world.cvars.g_spskill.integer;
            let debounce = 500 + (3 - g_spskill) * 100;
            let self_id = ctx.entity_id_of(self_);
            let delay = ctx.world.bg_state.rng.Q_irand(debounce, debounce + 1000);
            TIMER_Set(
                ctx,
                self_id,
                c"aimDebounce".as_ptr() as *const c_char,
                delay,
            );
            //	int debounce = 1000+(3-g_spskill.integer)*500;
            //	TIMER_Set( self, "aimDebounce", ctx.world.bg_state.rng.Q_irand( debounce,debounce+2000 ) );
        }
    }
}
