// PORT-COMPLETE: NPC.c
//! FAITHFUL port of `oracle/codemp/game/NPC.c`.
//!
//! Filled by the jampgame mega-pass; all bodies are live. Functions that reach
//! file-scope NPC-AI state (the `NPC`/`NPCInfo`/`client`/`ucmd` file-scope
//! globals, `level`, `g_entities`, cvars) read it through `ctx.world.globals`
//! and keep raw-pointer internals (Stage-2 debt), matching the g_utils.c
//! precedent.
//!
//! Safe-state migration **Stage 1**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5), not raw `gentity_t*`; ctx-free leaf helpers
//! take `&mut`/`&gentity_t`. Bodies re-derive the raw pointers verbatim at the
//! top (`// STAGE-1:` markers) — Stage-2 debt. Callers bridge at the boundary
//! via `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::teams::class::class_t;
use mp_qshared::common::mp::qcommon::b_state_t::bState_t;

use crate::g_utils::G_SetAnim;
use crate::NPC_AI_Atst::NPC_BSATST_Default;
use crate::NPC_AI_Default::NPC_BSDefault;
use crate::NPC_AI_Droid::NPC_BSDroid_Default;
use crate::NPC_AI_Grenadier::NPC_BSGrenadier_Default;
use crate::NPC_AI_Howler::NPC_BSHowler_Default;
use crate::NPC_AI_ImperialProbe::NPC_BSImperialProbe_Default;
use crate::NPC_AI_Interrogator::NPC_BSInterrogator_Default;
use crate::NPC_AI_Jedi::{NPC_BSJedi_Default, NPC_BSJedi_FollowLeader};
use crate::NPC_AI_Mark1::NPC_BSMark1_Default;
use crate::NPC_AI_Mark2::NPC_BSMark2_Default;
use crate::NPC_AI_MineMonster::NPC_BSMineMonster_Default;
use crate::NPC_AI_Rancor::NPC_BSRancor_Default;
use crate::NPC_AI_Remote::NPC_BSRemote_Default;
use crate::NPC_AI_Seeker::NPC_BSSeeker_Default;
use crate::NPC_AI_Sentry::NPC_BSSentry_Default;
use crate::NPC_AI_Sniper::NPC_BSSniper_Default;
use crate::NPC_AI_Stormtrooper::{NPC_BSST_Default, NPC_BSST_Investigate, NPC_BSST_Sleep};
use crate::NPC_behavior::{
    NPC_BSAdvanceFight, NPC_BSCinematic, NPC_BSFlee, NPC_BSFollowLeader, NPC_BSJump, NPC_BSNoClip,
    NPC_BSRemove, NPC_BSSearch, NPC_BSSleep, NPC_BSWait, NPC_BSWander,
};
use crate::NPC_stats::NPC_LoadParms;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Raven `CorpsePhysics`.
///
/// Source: `oracle/codemp/game/NPC.c:46-103`
pub fn CorpsePhysics(ctx: &mut GameContext, self_: EntityId) {
    // `EF_DISINTEGRATION` (entity_effects) and `CONTENTS_TRIGGER` (surface_flags)
    // resolve to their canonical workspace consts through the prelude glob.
    // `ALERT_CLEAR_TIME` — single-owner header, deliberately kept local (not
    // consolidated; the peer copy in NPC_senses.rs is fn-local too).
    // Source: `oracle/codemp/game/b_local.h:164`
    const ALERT_CLEAR_TIME: c_int = 200;

    ctx.world.globals.ucmd = usercmd_t::default();
    // STAGE-2b: irreducible — raw &ucmd alias into ctx.world handed alongside
    // ctx to the raw-ABI ClientThink.
    let ucmd_ptr = &raw mut ctx.world.globals.ucmd;
    let self_num = ctx.world.entity(self_).s.number;
    crate::g_active::ClientThink(ctx, self_num, ucmd_ptr);

    // FLAG: NPC pool `gclient_t` (`gClPtrs`, g_utils.c:430) — not a
    // `level.clients` slot; pointer read via the safe entity borrow, dereffed
    // raw exactly as Raven does (recipe 2b).
    // §19: oracle derefs `self->client` (NPC_class, ps, respawnTime, …)
    // unconditionally throughout; the `!client.is_null()` guards below are
    // defensive. Source: oracle/codemp/game/NPC.c:54-103
    let client = ctx.world.entity(self_).client;
    unsafe {
        if !client.is_null() && (*client).NPC_class == class_t::CLASS_GALAKMECH {
            crate::NPC_AI_GalakMech::GM_Dying(ctx, self_);
        }

        //FIXME: match my pitch and roll for the slope of my groundPlane
        if (*client).ps.groundEntityNum != ENTITYNUM_NONE
            && (ctx.world.entity(self_).s.eFlags & EF_DISINTEGRATION) == 0
        {
            //on the ground
            //FIXME: check 4 corners
            pitch_roll_for_slope(ctx, self_, None);
        }

        if ctx.world.globals.eventClearTime == ctx.world.level.time + ALERT_CLEAR_TIME {
            //events were just cleared out so add me again
            if !client.is_null() && ((*client).ps.eFlags & EF_NODRAW) == 0 {
                let enemy = ctx.world.entity(self_).enemy;
                let origin = ctx.world.entity(self_).r.currentOrigin;
                crate::NPC_senses::AddSightEvent(
                    ctx,
                    enemy,
                    origin,
                    384.0,
                    alertEventLevel_e::AEL_DISCOVERED,
                    0.0,
                );
            }
        }

        if ctx.world.level.time - ctx.world.entity(self_).s.time > 3000 {
            //been dead for 3 seconds
            if ctx.world.cvars.g_dismember.integer < 11381138
                && ctx.world.cvars.g_saberRealisticCombat.integer == 0
            {
                //can't be dismembered once dead
                if !client.is_null() && (*client).NPC_class != class_t::CLASS_PROTOCOL {
                    //	self->client->dismembered = qtrue;
                }
            }
        }

        //if ( level.time - self->s.time > 500 )
        if !client.is_null() && (*client).respawnTime < ctx.world.level.time + 500 {
            //don't turn "nonsolid" until about 1 second after actual death
            if !client.is_null() && ((*client).ps.eFlags & EF_DISINTEGRATION) != 0 {
                ctx.world.entity_mut(self_).r.contents = 0;
            } else if !client.is_null()
                && (*client).NPC_class != class_t::CLASS_MARK1
                && (*client).NPC_class != class_t::CLASS_INTERROGATOR
            {
                // The Mark1 & Interrogator stays solid.
                ctx.world.entity_mut(self_).r.contents = CONTENTS_CORPSE;
                //self->r.maxs[2] = -8;
            }

            if !ctx.world.entity(self_).message.is_null() {
                ctx.world.entity_mut(self_).r.contents |= CONTENTS_TRIGGER;
            }
        }
    }
}

/// Raven `NPC_RemoveBody`.
///
/// Source: `oracle/codemp/game/NPC.c:115-223`
pub fn NPC_RemoveBody(ctx: &mut GameContext, self_: EntityId) {
    // `EF_DISINTEGRATION` (entity_effects) resolves via the prelude glob.
    // Raven `g_local.h:37`: `#define FRAMETIME 100` — single-owner header,
    // deliberately kept local (not consolidated).
    const FRAMETIME: c_int = 100;
    // Raven `entity_effects.rs` doesn't yet re-export `EF2_HELD_BY_MONSTER`
    // through the prelude glob — imported explicitly below.
    use mp_bg::public::entity_effects::EF2_HELD_BY_MONSTER;

    CorpsePhysics(ctx, self_);

    ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;

    // FLAG: gNPC_t (NPCInfo) and NPC pool `gclient_t` have no accessor; the
    // pointers are read via the safe entity borrow and dereffed raw exactly as
    // Raven does (recipe 2b/2c).
    // §19: oracle derefs `self->NPC` and `self->client` unconditionally
    // throughout; the `!npc.is_null()`/`!client.is_null()` guards below are
    // defensive. Source: oracle/codemp/game/NPC.c:115-223
    let npc = ctx.world.entity(self_).NPC;
    unsafe {
        if !npc.is_null() && (*npc).nextBStateThink <= ctx.world.level.time {
            let self_num = ctx.world.entity(self_).s.number;
            trap::ICARUS_MaintainTaskManager(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_MAINTAINTASKMANAGER::GIcarusMaintaintaskmanagerArgs::new(self_num),
            );
        }
        if !npc.is_null() {
            (*npc).nextBStateThink = ctx.world.level.time + FRAMETIME;
        }

        if !ctx.world.entity(self_).message.is_null() {
            //I still have a key
            return;
        }

        let client = ctx.world.entity(self_).client;

        // I don't consider this a hack, it's creative coding . . .
        // I agree, very creative... need something like this for ATST and GALAKMECH too!
        if !client.is_null() && (*client).NPC_class == class_t::CLASS_MARK1 {
            crate::NPC_AI_Mark1::Mark1_dying(ctx, Some(self_));
        }

        // Since these blow up, remove the bounding box.
        if !client.is_null()
            && ((*client).NPC_class == class_t::CLASS_REMOTE
                || (*client).NPC_class == class_t::CLASS_SENTRY
                || (*client).NPC_class == class_t::CLASS_PROBE
                || (*client).NPC_class == class_t::CLASS_INTERROGATOR
                || (*client).NPC_class == class_t::CLASS_MARK2)
        {
            //if ( !self->taskManager || !self->taskManager->IsRunning() )
            let self_num = ctx.world.entity(self_).s.number;
            if trap::ICARUS_IsRunning(
                ctx.engine,
                mp_abi::game::syscalls::G_ICARUS_ISRUNNING::GIcarusIsrunningArgs::new(self_num),
            ) == 0
            {
                let activator_id = ctx.world.entity(self_).activator;
                let activator_client = match activator_id {
                    Some(id) => ctx.world.entity(id).client,
                    None => core::ptr::null_mut(),
                };
                if activator_id.is_none()
                    || activator_client.is_null()
                    || ((*activator_client).ps.eFlags2 & EF2_HELD_BY_MONSTER) == 0
                {
                    //not being held by a Rancor
                    crate::g_utils::G_FreeEntity(ctx, Some(self_));
                }
            }
            return;
        }

        //FIXME: don't ever inflate back up?
        if !client.is_null() {
            let eye_z = (*client).renderInfo.eyePoint[2];
            let origin_z = ctx.world.entity(self_).r.currentOrigin[2];
            ctx.world.entity_mut(self_).r.maxs[2] = eye_z - origin_z + 4.0;
        }
        if ctx.world.entity(self_).r.maxs[2] < -8.0 {
            ctx.world.entity_mut(self_).r.maxs[2] = -8.0;
        }

        if !client.is_null() && (*client).NPC_class == class_t::CLASS_GALAKMECH {
            //never disappears
            return;
        }
        if !npc.is_null() && (*npc).timeOfDeath <= ctx.world.level.time {
            (*npc).timeOfDeath = ctx.world.level.time + 1000;
            // Only do all of this nonsense for Scav boys ( and girls )
            // should I check NPC_class here instead of TEAM ? - dmv
            if !client.is_null()
                && ((*client).playerTeam == crate::teams::npcteam::NPCTEAM_ENEMY
                    || (*client).NPC_class == class_t::CLASS_PROTOCOL)
            {
                ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;
                // try back in a second
                //Don't care about this for MP I guess.
            }

            //FIXME: there are some conditions - such as heavy combat - in which we want
            //			to remove the bodies... but in other cases it's just weird, like
            //			when they're right behind you in a closed room and when they've been
            //			placed as dead NPCs by a designer...
            //			For now we just assume that a corpse with no enemy was
            //			placed in the map as a corpse
            if !ctx.world.entity(self_).enemy.is_none() {
                //if ( !self->taskManager || !self->taskManager->IsRunning() )
                let self_num = ctx.world.entity(self_).s.number;
                if trap::ICARUS_IsRunning(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ICARUS_ISRUNNING::GIcarusIsrunningArgs::new(self_num),
                ) == 0
                {
                    let activator_id = ctx.world.entity(self_).activator;
                    let activator_client = match activator_id {
                        Some(id) => ctx.world.entity(id).client,
                        None => core::ptr::null_mut(),
                    };
                    if activator_id.is_none()
                        || activator_client.is_null()
                        || ((*activator_client).ps.eFlags2 & EF2_HELD_BY_MONSTER) == 0
                    {
                        //not being held by a Rancor
                        if !client.is_null()
                            && (*client).ps.saberEntityNum > 0
                            && (*client).ps.saberEntityNum < ENTITYNUM_WORLD
                        {
                            let saber_id = EntityId::from_num((*client).ps.saberEntityNum);
                            crate::g_utils::G_FreeEntity(ctx, saber_id);
                        }
                        crate::g_utils::G_FreeEntity(ctx, Some(self_));
                    }
                }
            }
        }
    }
}

/// Raven `BodyRemovalPadTime`.
///
/// Raven: team no longer indicates species/race, so this switches on
/// `NPC_class` instead (comment preserved from source).
/// Source: `oracle/codemp/game/NPC.c:233-312`
pub fn BodyRemovalPadTime(ent: &gentity_t) -> c_int {
    // Ctx-free leaf takes `&gentity_t`; the `ent.is_null()` guard is vacuous
    // behind a reference (dropped); the `client` null guard is preserved.
    // FLAG: NPC pool `gclient_t` has no accessor; deref stays raw (recipe 2b).
    let client = ent.client;
    if client.is_null() {
        return 0;
    }
    unsafe {
        match (*client).NPC_class {
            class_t::CLASS_MOUSE
            | class_t::CLASS_GONK
            | class_t::CLASS_R2D2
            | class_t::CLASS_R5D2
            | class_t::CLASS_MARK1
            | class_t::CLASS_MARK2
            | class_t::CLASS_PROBE
            | class_t::CLASS_SEEKER
            | class_t::CLASS_REMOTE
            | class_t::CLASS_SENTRY
            | class_t::CLASS_INTERROGATOR => 0,
            // never go away; for now made default 10000 (Raven comment preserved).
            _ => 10000,
        }
    }
}

/// Raven `NPC_RemoveBodyEffect`.
///
/// Source: `oracle/codemp/game/NPC.c:323-378`
pub fn NPC_RemoveBodyEffect(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPC pool `gclient_t` has no accessor; pointer read via the safe
    // entity borrow (only null-checked here, never dereffed) (recipe 2b).
    let client = ctx.world.entity(npc_id).client;
    if client.is_null() || (ctx.world.entity(npc_id).s.eFlags & EF_NODRAW) != 0 {
        return;
    }
    // Raven: the per-class droid/species branches below are `stub code` —
    // every arm is commented-out upstream (dead debug-effect scaffolding);
    // the switch itself has no live behavior beyond the guard above.
}

/// Raven `pitch_roll_for_slope`.
///
/// `pass_slope` is NULL-able (`!pass_slope` guard: NULL from `NPC_Pain`,
/// non-NULL `tr.plane.normal` from `G_RunObject`) and never written
/// through, so it takes the AngleVectors-idiom shape
/// (`Option<&mut [f32;3]>`) per the mechanical out-param rule.
/// Source: `oracle/codemp/game/NPC.c:395-470`
pub fn pitch_roll_for_slope(
    ctx: &mut GameContext,
    forwhom: EntityId,
    pass_slope: Option<&mut vec3_t>,
) {
    // `PITCH`/`ROLL` resolve to the canonical `crate::q_math` consts via the prelude.
    unsafe {
        let currentOrigin = ctx.world.entity(forwhom).r.currentOrigin;
        let mins = ctx.world.entity(forwhom).r.mins;
        let number = ctx.world.entity(forwhom).s.number;
        // if we don't have a slope, get one
        let slope: vec3_t = match pass_slope {
            None => {
                let mut startspot = currentOrigin;
                startspot[2] += mins[2] + 4.0;
                let mut endspot = startspot;
                endspot[2] -= 300.0;

                let mut trace: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &currentOrigin as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &endspot as *const vec3_t,
                        number,
                        MASK_SOLID,
                    ),
                );
                //		if(trace_fraction>0.05&&forwhom.movetype==MOVETYPE_STEP)
                //			forwhom.flags(-)FL_ONGROUND;
                if trace.fraction >= 1.0 {
                    return;
                }
                // `!( &trace.plane )` is always false in the oracle (address of a
                // struct member is never null) — dead condition, dropped.
                if trace.plane.normal == VEC3_ORIGIN {
                    return;
                }
                trace.plane.normal
            }
            Some(p) => {
                if *p == VEC3_ORIGIN {
                    // Raven falls through the `!pass_slope` branch (retracing) when
                    // an all-zero slope is explicitly passed too.
                    let mut startspot = currentOrigin;
                    startspot[2] += mins[2] + 4.0;
                    let mut endspot = startspot;
                    endspot[2] -= 300.0;
                    let mut trace: trace_t = core::mem::zeroed();
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &currentOrigin as *const vec3_t,
                            core::ptr::null(),
                            core::ptr::null(),
                            &endspot as *const vec3_t,
                            number,
                            MASK_SOLID,
                        ),
                    );
                    if trace.fraction >= 1.0 {
                        return;
                    }
                    if trace.plane.normal == VEC3_ORIGIN {
                        return;
                    }
                    trace.plane.normal
                } else {
                    *p
                }
            }
        };

        let mut ovf: vec3_t = [0.0; 3];
        let mut ovr: vec3_t = [0.0; 3];
        let currentAngles = ctx.world.entity(forwhom).r.currentAngles;
        crate::q_math::AngleVectors(currentAngles, Some(&mut ovf), Some(&mut ovr), None);

        let mut new_angles: vec3_t = [0.0, 0.0, 0.0];
        crate::q_math::vectoangles(slope, &mut new_angles);
        let pitch = new_angles[PITCH] + 90.0;
        new_angles[ROLL] = 0.0;
        new_angles[PITCH] = 0.0;

        let mut nvf: vec3_t = [0.0; 3];
        crate::q_math::AngleVectors(new_angles, Some(&mut nvf), None, None);

        // Raven `DotProduct(a,b)` macro (`q_shared.h`) has no ported fn; inlined
        // elementwise, matching the codebase's other unmacro'd C idioms.
        let mut mod_ = nvf[0] * ovr[0] + nvf[1] * ovr[1] + nvf[2] * ovr[2];
        mod_ = if mod_ < 0.0 { -1.0 } else { 1.0 };

        let dot = nvf[0] * ovf[0] + nvf[1] * ovf[1] + nvf[2] * ovf[2];

        // FLAG: NPC pool `gclient_t` has no accessor; deref stays raw (recipe 2b).
        let client = ctx.world.entity(forwhom).client;
        if !client.is_null() {
            (*client).ps.viewangles[PITCH] = dot * pitch;
            (*client).ps.viewangles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * mod_;
            let oldmins2 = ctx.world.entity(forwhom).r.mins[2];
            // C promotes through `double`: `fabs()` is double libm and `/180.0f`
            // widens the quotient, so the whole expr is f64, narrowed on store.
            let new_mins2 =
                (-24.0_f64 + 12.0 * ((*client).ps.viewangles[PITCH] as f64).abs() / 180.0) as f32;
            ctx.world.entity_mut(forwhom).r.mins[2] = new_mins2;
            //FIXME: if it gets bigger, move up
            if oldmins2 > new_mins2 {
                //our mins is now lower, need to move up
                //FIXME: trace?
                (*client).ps.origin[2] += oldmins2 - new_mins2;
                let origin_z = (*client).ps.origin[2];
                ctx.world.entity_mut(forwhom).r.currentOrigin[2] = origin_z;
                let forwhom_ptr = ctx.world.entity_mut(forwhom) as *mut gentity_t;
                trap::LinkEntity(
                    ctx.engine,
                    mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(forwhom_ptr.cast()),
                );
            }
        } else {
            ctx.world.entity_mut(forwhom).r.currentAngles[PITCH] = dot * pitch;
            ctx.world.entity_mut(forwhom).r.currentAngles[ROLL] =
                (1.0 - Q_fabs(dot)) * pitch * mod_;
        }
    }
}

/// Raven `DeadThink`.
///
/// Source: `oracle/codemp/game/NPC.c:478-607`
pub fn DeadThink(ctx: &mut GameContext) {
    // `CONTENTS_NODROP` (surface_flags) resolves via the prelude glob.
    // `FRAMETIME` (`g_local.h:37` = 100) — single-owner header, deliberately
    // kept local (not consolidated).
    const FRAMETIME: c_int = 100;

    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) and NPC pool `gclient_t` have no accessor; the
    // pointers are read via the safe entity borrow and dereffed raw exactly as
    // Raven does (recipe 2b/2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        //HACKHACKHACKHACKHACK
        //We should really have a seperate G2 bounding box (seperate from the physics bbox) for G2 collisions only
        //FIXME: don't ever inflate back up?
        let eye_z = (*client).renderInfo.eyePoint[2];
        let origin_z = ctx.world.entity(npc_id).r.currentOrigin[2];
        ctx.world.entity_mut(npc_id).r.maxs[2] = eye_z - origin_z + 4.0;
        if ctx.world.entity(npc_id).r.maxs[2] < -8.0 {
            ctx.world.entity_mut(npc_id).r.maxs[2] = -8.0;
        }
        if (*client).ps.velocity == VEC3_ORIGIN {
            //not flying through the air
            let mut trace: trace_t = core::mem::zeroed();
            if ctx.world.entity(npc_id).r.mins[0] > -32.0 {
                ctx.world.entity_mut(npc_id).r.mins[0] -= 1.0;
                let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
                let mins = ctx.world.entity(npc_id).r.mins;
                let maxs = ctx.world.entity(npc_id).r.maxs;
                let number = ctx.world.entity(npc_id).s.number;
                let clipmask = ctx.world.entity(npc_id).clipmask;
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &currentOrigin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &currentOrigin as *const vec3_t,
                        number,
                        clipmask,
                    ),
                );
                if trace.allsolid != 0 {
                    ctx.world.entity_mut(npc_id).r.mins[0] += 1.0;
                }
            }
            if ctx.world.entity(npc_id).r.maxs[0] < 32.0 {
                ctx.world.entity_mut(npc_id).r.maxs[0] += 1.0;
                let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
                let mins = ctx.world.entity(npc_id).r.mins;
                let maxs = ctx.world.entity(npc_id).r.maxs;
                let number = ctx.world.entity(npc_id).s.number;
                let clipmask = ctx.world.entity(npc_id).clipmask;
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &currentOrigin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &currentOrigin as *const vec3_t,
                        number,
                        clipmask,
                    ),
                );
                if trace.allsolid != 0 {
                    ctx.world.entity_mut(npc_id).r.maxs[0] -= 1.0;
                }
            }
            if ctx.world.entity(npc_id).r.mins[1] > -32.0 {
                ctx.world.entity_mut(npc_id).r.mins[1] -= 1.0;
                let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
                let mins = ctx.world.entity(npc_id).r.mins;
                let maxs = ctx.world.entity(npc_id).r.maxs;
                let number = ctx.world.entity(npc_id).s.number;
                let clipmask = ctx.world.entity(npc_id).clipmask;
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &currentOrigin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &currentOrigin as *const vec3_t,
                        number,
                        clipmask,
                    ),
                );
                if trace.allsolid != 0 {
                    ctx.world.entity_mut(npc_id).r.mins[1] += 1.0;
                }
            }
            if ctx.world.entity(npc_id).r.maxs[1] < 32.0 {
                ctx.world.entity_mut(npc_id).r.maxs[1] += 1.0;
                let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
                let mins = ctx.world.entity(npc_id).r.mins;
                let maxs = ctx.world.entity(npc_id).r.maxs;
                let number = ctx.world.entity(npc_id).s.number;
                let clipmask = ctx.world.entity(npc_id).clipmask;
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut trace as *mut trace_t,
                        &currentOrigin as *const vec3_t,
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        &currentOrigin as *const vec3_t,
                        number,
                        clipmask,
                    ),
                );
                if trace.allsolid != 0 {
                    ctx.world.entity_mut(npc_id).r.maxs[1] -= 1.0;
                }
            }
        }
        //HACKHACKHACKHACKHACK

        //FIXME: tilt and fall off of ledges?
        //NPC_PostDeathThink();

        // Raven's commented-out `!NPCInfo->timeOfDeath` branch is dead code
        // upstream (`/* ... */`); only the live `else` block runs.
        if ctx.world.level.time
            >= (*npc_info).timeOfDeath + BodyRemovalPadTime(ctx.world.entity(npc_id))
        {
            //death anim done (or were given a specific amount of time to wait before removal), wait the requisite amount of time them remove
            if ((*client).ps.eFlags & EF_NODRAW) != 0 {
                let number = ctx.world.entity(npc_id).s.number;
                if trap::ICARUS_IsRunning(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ICARUS_ISRUNNING::GIcarusIsrunningArgs::new(number),
                ) == 0
                {
                    //if ( !NPC->taskManager || !NPC->taskManager->IsRunning() )
                    ctx.world.entity_mut(npc_id).think =
                        Some(crate::ent_fn_enums::EntThink::G_FreeEntity).into();
                    ctx.world.entity_mut(npc_id).nextthink = ctx.world.level.time + FRAMETIME;
                }
            } else {
                // Start the body effect first, then delay 400ms before ditching the corpse
                NPC_RemoveBodyEffect(ctx);

                //FIXME: keep it running through physics somehow?
                ctx.world.entity_mut(npc_id).think =
                    Some(crate::ent_fn_enums::EntThink::NPC_RemoveBody).into();
                ctx.world.entity_mut(npc_id).nextthink = ctx.world.level.time + FRAMETIME;
                let npc_class = (*client).NPC_class;
                // check for droids
                if npc_class == class_t::CLASS_SEEKER
                    || npc_class == class_t::CLASS_REMOTE
                    || npc_class == class_t::CLASS_PROBE
                    || npc_class == class_t::CLASS_MOUSE
                    || npc_class == class_t::CLASS_GONK
                    || npc_class == class_t::CLASS_R2D2
                    || npc_class == class_t::CLASS_R5D2
                    || npc_class == class_t::CLASS_MARK2
                    || npc_class == class_t::CLASS_SENTRY
                {
                    (*client).ps.eFlags |= EF_NODRAW;
                    (*npc_info).timeOfDeath = ctx.world.level.time + FRAMETIME * 8;
                } else {
                    (*npc_info).timeOfDeath = ctx.world.level.time + FRAMETIME * 4;
                }
            }
            return;
        }

        // If the player is on the ground and the resting position contents haven't been set yet...(BounceCount tracks the contents)
        if ctx.world.entity(npc_id).bounceCount < 0
            && ctx.world.entity(npc_id).s.groundEntityNum >= 0
        {
            // if client is in a nodrop area, make him/her nodraw
            let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
            let contents = trap::PointContents(
                ctx.engine,
                mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs::new(
                    &currentOrigin as *const vec3_t,
                    -1,
                ),
            );
            ctx.world.entity_mut(npc_id).bounceCount = contents;

            if (contents & CONTENTS_NODROP) != 0 {
                (*client).ps.eFlags |= EF_NODRAW;
            }
        }

        CorpsePhysics(ctx, npc_id);
    }
}

/// Raven `SetNPCGlobals`.
///
/// Source: `oracle/codemp/game/NPC.c:617-623`
pub fn SetNPCGlobals(ctx: &mut GameContext, ent: EntityId) {
    // FLAG: gNPC_t (NPCInfo) and NPC pool `gclient_t` globals hold raw pointers
    // with no accessor; the fields are read via the safe entity borrow and
    // stored verbatim (recipe 2b/2c).
    let npc_info = ctx.world.entity(ent).NPC;
    let client = ctx.world.entity(ent).client;
    let ent_ptr = ctx.world.entity_mut(ent) as *mut gentity_t;
    ctx.world.globals.NPC = ent_ptr;
    ctx.world.globals.NPCInfo = npc_info;
    ctx.world.globals.client = client;
    ctx.world.globals.ucmd = usercmd_t::default();
}

/// Raven `SaveNPCGlobals`.
///
/// Source: `oracle/codemp/game/NPC.c:630-636`
pub fn SaveNPCGlobals(ctx: &mut GameContext) {
    ctx.world.globals._saved_NPC = ctx.world.globals.NPC;
    ctx.world.globals._saved_NPCInfo = ctx.world.globals.NPCInfo;
    ctx.world.globals._saved_client = ctx.world.globals.client;
    ctx.world.globals._saved_ucmd = ctx.world.globals.ucmd;
}

/// Raven `RestoreNPCGlobals`.
///
/// Source: `oracle/codemp/game/NPC.c:638-644`
pub fn RestoreNPCGlobals(ctx: &mut GameContext) {
    ctx.world.globals.NPC = ctx.world.globals._saved_NPC;
    ctx.world.globals.NPCInfo = ctx.world.globals._saved_NPCInfo;
    ctx.world.globals.client = ctx.world.globals._saved_client;
    ctx.world.globals.ucmd = ctx.world.globals._saved_ucmd;
}

/// Raven `ClearNPCGlobals`.
///
/// Raven: "We MUST do this, other funcs were using NPC illegally when
/// 'self' wasn't the global NPC" (comment preserved from source).
/// Source: `oracle/codemp/game/NPC.c:647-652`
pub fn ClearNPCGlobals(ctx: &mut GameContext) {
    ctx.world.globals.NPC = core::ptr::null_mut();
    ctx.world.globals.NPCInfo = core::ptr::null_mut();
    ctx.world.globals.client = core::ptr::null_mut();
}

/// Raven `NPC_ShowDebugInfo`.
///
/// Source: `oracle/codemp/game/NPC.c:664-681`
pub fn NPC_ShowDebugInfo(ctx: &mut GameContext) {
    if ctx.world.globals.showBBoxes == 0 {
        return;
    }
    // Raven `NPCDEBUG_RED` (`NPC.c:658`) — const color, not GameWorld state.
    const NPCDEBUG_RED: vec3_t = [1.0, 0.0, 0.0];
    // Raven `FOFS(classname)` macro.
    let fieldofs = core::mem::offset_of!(gentity_t, classname) as c_int;

    let mut found: *mut gentity_t = core::ptr::null_mut();
    loop {
        let found_id = ctx.entity_id_of(found);
        found = crate::g_utils::G_Find(ctx, found_id, fieldofs, c"NPC".as_ptr());
        if found.is_null() {
            break;
        }
        let fid = ctx.entity_id_of(found).unwrap();
        let found_origin = ctx.world.entity(fid).r.currentOrigin;
        let player_origin = ctx.world.g_entities[0].r.currentOrigin;
        if trap::InPVS(
            ctx.engine,
            mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                &found_origin as *const vec3_t,
                &player_origin as *const vec3_t,
            ),
        ) != 0
        {
            let f_origin = ctx.world.entity(fid).r.currentOrigin;
            let f_mins = ctx.world.entity(fid).r.mins;
            let f_maxs = ctx.world.entity(fid).r.maxs;
            let mins = [
                f_origin[0] + f_mins[0],
                f_origin[1] + f_mins[1],
                f_origin[2] + f_mins[2],
            ];
            let maxs = [
                f_origin[0] + f_maxs[0],
                f_origin[1] + f_maxs[1],
                f_origin[2] + f_maxs[2],
            ];
            crate::g_nav::G_Cube(mins, maxs, NPCDEBUG_RED, 0.25);
        }
    }
}

/// Raven `NPC_ApplyScriptFlags`.
///
/// Source: `oracle/codemp/game/NPC.c:683-735`
pub fn NPC_ApplyScriptFlags(ctx: &mut GameContext) {
    // Raven `b_public.h:27-43` scriptFlags bits (`SCF_*`) resolve to the
    // canonical `crate::npc::script_flags` consts through the prelude glob.
    use mp_qshared::common::mp::qcommon::usercmd_button::{
        BUTTON_ALT_ATTACK, BUTTON_ATTACK, BUTTON_USE, BUTTON_WALKING,
    };

    // FLAG: gNPC_t (NPCInfo) has no accessor; all derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        let scriptFlags = (*npc_info).scriptFlags;
        let level_time = ctx.world.level.time;

        if (scriptFlags & SCF_CROUCHED) != 0 {
            if (*npc_info).charmedTime > level_time
                && (ctx.world.globals.ucmd.forwardmove != 0
                    || ctx.world.globals.ucmd.rightmove != 0)
            {
                //ugh, if charmed and moving, ignore the crouched command
            } else {
                ctx.world.globals.ucmd.upmove = -127;
            }
        }

        if (scriptFlags & SCF_RUNNING) != 0 {
            ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        } else if (scriptFlags & SCF_WALKING) != 0 {
            if (*npc_info).charmedTime > level_time
                && (ctx.world.globals.ucmd.forwardmove != 0
                    || ctx.world.globals.ucmd.rightmove != 0)
            {
                //ugh, if charmed and moving, ignore the walking command
            } else {
                ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            }
        }

        if (scriptFlags & SCF_LEAN_RIGHT) != 0 {
            ctx.world.globals.ucmd.buttons |= BUTTON_USE;
            ctx.world.globals.ucmd.rightmove = 127;
            ctx.world.globals.ucmd.forwardmove = 0;
            ctx.world.globals.ucmd.upmove = 0;
        } else if (scriptFlags & SCF_LEAN_LEFT) != 0 {
            ctx.world.globals.ucmd.buttons |= BUTTON_USE;
            ctx.world.globals.ucmd.rightmove = -127;
            ctx.world.globals.ucmd.forwardmove = 0;
            ctx.world.globals.ucmd.upmove = 0;
        }

        if (scriptFlags & SCF_ALT_FIRE) != 0
            && (ctx.world.globals.ucmd.buttons & BUTTON_ATTACK) != 0
        {
            //Use altfire instead
            ctx.world.globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
        }
    }
}

/// Raven `NPC_HandleAIFlags`.
///
/// Source: `oracle/codemp/game/NPC.c:738-833`
pub fn NPC_HandleAIFlags(ctx: &mut GameContext) {
    // `NPCAI_LOST` (b_public.h) resolves to the canonical `crate::npc::ai_flags`
    // const through the prelude glob.
    use mp_bg::public::entity_event::entity_event_t;

    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; all derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        //FIXME: make these flags checks a function call like NPC_CheckAIFlagsAndTimers
        if ((*npc_info).aiFlags & NPCAI_LOST) != 0 {
            //Print that you need help!
            //FIXME: shouldn't remove this just yet if cg_draw needs it
            (*npc_info).aiFlags &= !NPCAI_LOST;

            if !(*npc_info).goalEntity.is_none()
                && (*npc_info).goalEntity == ctx.world.entity(npc_id).enemy
            {
                //We can't nav to our enemy
                //Drop enemy and see if we should search for him
                crate::NPC_AI_Default::NPC_LostEnemyDecideChase(ctx);
            }
        }

        //MRJ Request: greet-allies block is `/* ... */`'d out upstream — dead code, not ported.

        //been told to play a victory sound after a delay
        if (*npc_info).greetingDebounceTime != 0
            && (*npc_info).greetingDebounceTime < ctx.world.level.time
        {
            // Two Q_irand draws as call args; C order is unspecified. Verified with
            // the referee-oracle compiler (g++-16): args evaluate left-to-right, so
            // the event draw precedes the delay draw. Source: oracle/codemp/game/NPC.c:813
            let ev = ctx.world.bg_state.rng.Q_irand(
                entity_event_t::EV_VICTORY1 as c_int,
                entity_event_t::EV_VICTORY3 as c_int,
            );
            let debounce = ctx.world.bg_state.rng.Q_irand(2000, 4000);
            crate::NPC_sounds::G_AddVoiceEvent(ctx, npc_id, ev, debounce);
            (*npc_info).greetingDebounceTime = 0;
        }

        if (*npc_info).ffireCount > 0 && (*npc_info).ffireFadeDebounce < ctx.world.level.time {
            (*npc_info).ffireCount -= 1;
            //Com_Printf( "drop: %d < %d\n", NPCInfo->ffireCount, 3+((2-g_spskill.integer)*2) );
            (*npc_info).ffireFadeDebounce = ctx.world.level.time + 3000;
        }
        if ctx.world.cvars.d_patched.integer != 0 {
            //use patch-style navigation
            if (*npc_info).consecutiveBlockedMoves > 20 {
                //been stuck for a while, try again?
                (*npc_info).consecutiveBlockedMoves = 0;
            }
        }
    }
}

/// Raven `NPC_AvoidWallsAndCliffs`.
///
/// Raven: body is `//...` — an intentional no-op stub upstream. Ported
/// faithfully as a callable no-op.
/// Source: `oracle/codemp/game/NPC.c:835-838`
pub fn NPC_AvoidWallsAndCliffs() {}

/// Raven `NPC_CheckAttackScript`.
///
/// Source: `oracle/codemp/game/NPC.c:840-848`
pub fn NPC_CheckAttackScript(ctx: &mut GameContext) {
    use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_ATTACK;

    let npc_id = ctx.entity_id_of(ctx.world.globals.NPC);
    if (ctx.world.globals.ucmd.buttons & BUTTON_ATTACK) == 0 {
        return;
    }
    crate::NPC_utils::G_ActivateBehavior(ctx, npc_id, bSet_t::BSET_ATTACK as c_int);
}

/// Raven `NPC_CheckAttackHold`.
///
/// Source: `oracle/codemp/game/NPC.c:851-913`
pub fn NPC_CheckAttackHold(ctx: &mut GameContext) {
    use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_ATTACK;

    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        // If they don't have an enemy they shouldn't hold their attack anim.
        if ctx.world.entity(npc_id).enemy.is_none() {
            (*npc_info).attackHoldTime = 0;
            return;
        }

        // Raven's borg-specific `/* ... */`'d branch is dead code upstream —
        // only the live `else` block (everyone else) runs.
        // Guaranteed `Some` — the early return above covers the `None` case.
        let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
        let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
        let self_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let vec: vec3_t = [
            enemy_origin[0] - self_origin[0],
            enemy_origin[1] - self_origin[1],
            enemy_origin[2] - self_origin[2],
        ];
        if crate::q_math::VectorLengthSquared(vec)
            > crate::NPC_combat::NPC_MaxDistSquaredForWeapon(ctx)
        {
            (*npc_info).attackHoldTime = 0;
        } else if (*npc_info).attackHoldTime != 0
            && (*npc_info).attackHoldTime > ctx.world.level.time
        {
            ctx.world.globals.ucmd.buttons |= BUTTON_ATTACK;
        } else if (*npc_info).attackHold != 0
            && (ctx.world.globals.ucmd.buttons & BUTTON_ATTACK) != 0
        {
            (*npc_info).attackHoldTime = ctx.world.level.time + (*npc_info).attackHold;
        } else {
            (*npc_info).attackHoldTime = 0;
        }
    }
}

/// Raven `NPC_KeepCurrentFacing`.
///
/// Source: `oracle/codemp/game/NPC.c:920-931`
pub fn NPC_KeepCurrentFacing(ctx: &mut GameContext) {
    // `PITCH`/`YAW` resolve to the canonical `crate::q_math` consts via the prelude.

    // FLAG: NPC pool `gclient_t` has no accessor; derefs stay raw (recipe 2b).
    let client = ctx.world.globals.client;
    unsafe {
        if ctx.world.globals.ucmd.angles[YAW] == 0 {
            // Raven `ANGLE2SHORT(x)` == `((int)((x)*65536/360) & 65535)`.
            let angle2short = |x: f32| -> c_int { ((x * 65536.0 / 360.0) as c_int) & 65535 };
            ctx.world.globals.ucmd.angles[YAW] =
                angle2short((*client).ps.viewangles[YAW]) - (*client).ps.delta_angles[YAW];
        }

        if ctx.world.globals.ucmd.angles[PITCH] == 0 {
            let angle2short = |x: f32| -> c_int { ((x * 65536.0 / 360.0) as c_int) & 65535 };
            ctx.world.globals.ucmd.angles[PITCH] =
                angle2short((*client).ps.viewangles[PITCH]) - (*client).ps.delta_angles[PITCH];
        }
    }
}

/// Raven `NPC_BehaviorSet_Charmed`.
///
/// Source: `oracle/codemp/game/NPC.c:939-963`
pub fn NPC_BehaviorSet_Charmed(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_FOLLOW_LEADER as c_int => NPC_BSFollowLeader(ctx),
        x if x == bState_t::BS_REMOVE as c_int => NPC_BSRemove(ctx),
        x if x == bState_t::BS_SEARCH as c_int => NPC_BSSearch(ctx),
        x if x == bState_t::BS_WANDER as c_int => NPC_BSWander(ctx),
        x if x == bState_t::BS_FLEE as c_int => NPC_BSFlee(ctx),
        _ => NPC_BSDefault(ctx),
    }
}

/// Raven `NPC_BehaviorSet_Default`.
///
/// Source: `oracle/codemp/game/NPC.c:970-1012`
pub fn NPC_BehaviorSet_Default(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_ADVANCE_FIGHT as c_int => NPC_BSAdvanceFight(ctx),
        x if x == bState_t::BS_SLEEP as c_int => NPC_BSSleep(ctx),
        x if x == bState_t::BS_FOLLOW_LEADER as c_int => NPC_BSFollowLeader(ctx),
        x if x == bState_t::BS_JUMP as c_int => NPC_BSJump(ctx),
        x if x == bState_t::BS_REMOVE as c_int => NPC_BSRemove(ctx),
        x if x == bState_t::BS_SEARCH as c_int => NPC_BSSearch(ctx),
        x if x == bState_t::BS_NOCLIP as c_int => NPC_BSNoClip(ctx),
        x if x == bState_t::BS_WANDER as c_int => NPC_BSWander(ctx),
        x if x == bState_t::BS_FLEE as c_int => NPC_BSFlee(ctx),
        x if x == bState_t::BS_WAIT as c_int => NPC_BSWait(ctx),
        x if x == bState_t::BS_CINEMATIC as c_int => NPC_BSCinematic(ctx),
        _ => NPC_BSDefault(ctx),
    }
}

/// Raven `NPC_BehaviorSet_Interrogator`.
///
/// Source: `oracle/codemp/game/NPC.c:1019-1034`
pub fn NPC_BehaviorSet_Interrogator(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSInterrogator_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_ImperialProbe`.
///
/// Source: `oracle/codemp/game/NPC.c:1045-1060`
pub fn NPC_BehaviorSet_ImperialProbe(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSImperialProbe_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Seeker`.
///
/// Source: `oracle/codemp/game/NPC.c:1070-1085`
pub fn NPC_BehaviorSet_Seeker(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSSeeker_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Remote`.
///
/// Source: `oracle/codemp/game/NPC.c:1094-1097`
pub fn NPC_BehaviorSet_Remote(ctx: &mut GameContext, bState: c_int) {
    NPC_BSRemote_Default(ctx);
}

/// Raven `NPC_BehaviorSet_Sentry`.
///
/// Source: `oracle/codemp/game/NPC.c:1106-1121`
pub fn NPC_BehaviorSet_Sentry(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSSentry_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Grenadier`.
///
/// Source: `oracle/codemp/game/NPC.c:1128-1144`
pub fn NPC_BehaviorSet_Grenadier(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSGrenadier_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Sniper`.
///
/// Source: `oracle/codemp/game/NPC.c:1150-1166`
pub fn NPC_BehaviorSet_Sniper(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSSniper_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Stormtrooper`.
///
/// Source: `oracle/codemp/game/NPC.c:1173-1197`
pub fn NPC_BehaviorSet_Stormtrooper(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSST_Default(ctx);
        }
        x if x == bState_t::BS_INVESTIGATE as c_int => NPC_BSST_Investigate(ctx),
        x if x == bState_t::BS_SLEEP as c_int => NPC_BSST_Sleep(ctx),
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Jedi`.
///
/// Source: `oracle/codemp/game/NPC.c:1205-1225`
pub fn NPC_BehaviorSet_Jedi(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSJedi_Default(ctx);
        }
        x if x == bState_t::BS_FOLLOW_LEADER as c_int => NPC_BSJedi_FollowLeader(ctx),
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Droid`.
///
/// Source: `oracle/codemp/game/NPC.c:1232-1245`
pub fn NPC_BehaviorSet_Droid(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_DEFAULT as c_int
            || x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int =>
        {
            NPC_BSDroid_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Mark1`.
///
/// Source: `oracle/codemp/game/NPC.c:1252-1265`
pub fn NPC_BehaviorSet_Mark1(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_DEFAULT as c_int
            || x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int =>
        {
            NPC_BSMark1_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Mark2`.
///
/// Source: `oracle/codemp/game/NPC.c:1272-1286`
pub fn NPC_BehaviorSet_Mark2(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_DEFAULT as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int =>
        {
            NPC_BSMark2_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_ATST`.
///
/// Source: `oracle/codemp/game/NPC.c:1293-1307`
pub fn NPC_BehaviorSet_ATST(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_DEFAULT as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int =>
        {
            NPC_BSATST_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_MineMonster`.
///
/// Source: `oracle/codemp/game/NPC.c:1314-1329`
pub fn NPC_BehaviorSet_MineMonster(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSMineMonster_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Howler`.
///
/// Source: `oracle/codemp/game/NPC.c:1336-1351`
pub fn NPC_BehaviorSet_Howler(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSHowler_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_BehaviorSet_Rancor`.
///
/// Source: `oracle/codemp/game/NPC.c:1358-1373`
pub fn NPC_BehaviorSet_Rancor(ctx: &mut GameContext, bState: c_int) {
    match bState {
        x if x == bState_t::BS_STAND_GUARD as c_int
            || x == bState_t::BS_PATROL as c_int
            || x == bState_t::BS_STAND_AND_SHOOT as c_int
            || x == bState_t::BS_HUNT_AND_KILL as c_int
            || x == bState_t::BS_DEFAULT as c_int =>
        {
            NPC_BSRancor_Default(ctx);
        }
        _ => NPC_BehaviorSet_Default(ctx, bState),
    }
}

/// Raven `NPC_RunBehavior`.
///
/// Source: `oracle/codemp/game/NPC.c:1384-1564`
pub fn NPC_RunBehavior(ctx: &mut GameContext, team: c_int, bState: c_int) {
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) and NPC pool `gclient_t` have no accessor; the
    // pointers are read via the safe entity borrow and dereffed raw exactly as
    // Raven does (recipe 2b/2c). `npc_ent` is retained only as the raw ABI
    // handle passed across the trap seam.
    let npc_info = ctx.world.globals.NPCInfo;
    let client = ctx.world.entity(npc_id).client;
    unsafe {
        if ctx.world.entity(npc_id).s.NPC_class == class_t::CLASS_VEHICLE as c_int
            && !ctx.world.entity(npc_id).m_pVehicle.is_null()
        {
            //vehicles don't do AI!
            return;
        }

        if bState == bState_t::BS_CINEMATIC as c_int {
            crate::NPC_behavior::NPC_BSCinematic(ctx);
        } else if (*client).ps.weapon == WP_EMPLACED_GUN {
            crate::NPC_behavior::NPC_BSEmplaced(ctx);
            crate::NPC_utils::NPC_CheckCharmed(ctx);
            return;
        } else if (*client).ps.weapon == WP_SABER {
            //jedi
            NPC_BehaviorSet_Jedi(ctx, bState);
        } else if (*client).NPC_class == class_t::CLASS_WAMPA {
            //wampa
            crate::NPC_AI_Wampa::NPC_BSWampa_Default(ctx);
        } else if (*client).NPC_class == class_t::CLASS_RANCOR {
            //rancor
            NPC_BehaviorSet_Rancor(ctx, bState);
        } else if (*client).NPC_class == class_t::CLASS_REMOTE {
            NPC_BehaviorSet_Remote(ctx, bState);
        } else if (*client).NPC_class == class_t::CLASS_SEEKER {
            NPC_BehaviorSet_Seeker(ctx, bState);
        } else if (*client).NPC_class == class_t::CLASS_BOBAFETT {
            //bounty hunter
            if crate::NPC_AI_Jedi::Boba_Flying(ctx.world.entity(npc_id)) != 0 {
                NPC_BehaviorSet_Seeker(ctx, bState);
            } else {
                NPC_BehaviorSet_Jedi(ctx, bState);
            }
        } else if ((*npc_info).scriptFlags & 0x00010000) != 0 {
            //being forced to march (SCF_FORCED_MARCH)
            crate::NPC_AI_Default::NPC_BSDefault(ctx);
        } else {
            match team {
                x if x == crate::teams::npcteam::NPCTEAM_ENEMY => {
                    // special cases for enemy droids
                    match (*client).NPC_class {
                        class_t::CLASS_ATST => {
                            NPC_BehaviorSet_ATST(ctx, bState);
                            return;
                        }
                        class_t::CLASS_PROBE => {
                            NPC_BehaviorSet_ImperialProbe(ctx, bState);
                            return;
                        }
                        class_t::CLASS_REMOTE => {
                            NPC_BehaviorSet_Remote(ctx, bState);
                            return;
                        }
                        class_t::CLASS_SENTRY => {
                            NPC_BehaviorSet_Sentry(ctx, bState);
                            return;
                        }
                        class_t::CLASS_INTERROGATOR => {
                            NPC_BehaviorSet_Interrogator(ctx, bState);
                            return;
                        }
                        class_t::CLASS_MINEMONSTER => {
                            NPC_BehaviorSet_MineMonster(ctx, bState);
                            return;
                        }
                        class_t::CLASS_HOWLER => {
                            NPC_BehaviorSet_Howler(ctx, bState);
                            return;
                        }
                        class_t::CLASS_MARK1 => {
                            NPC_BehaviorSet_Mark1(ctx, bState);
                            return;
                        }
                        class_t::CLASS_MARK2 => {
                            NPC_BehaviorSet_Mark2(ctx, bState);
                            return;
                        }
                        class_t::CLASS_GALAKMECH => {
                            crate::NPC_AI_GalakMech::NPC_BSGM_Default(ctx);
                            return;
                        }
                        _ => {}
                    }

                    if !ctx.world.entity(npc_id).enemy.is_none()
                        && ctx.world.entity(npc_id).s.weapon == WP_NONE
                        && bState != bState_t::BS_HUNT_AND_KILL as c_int
                        && trap::ICARUS_TaskIDPending(
                            ctx.engine,
                            mp_abi::game::syscalls::G_ICARUS_TASKIDPENDING::GIcarusTaskidpendingArgs::new(
                                npc_ent.cast(),
                                taskID_t::TID_MOVE_NAV as c_int,
                            ),
                        ) == 0
                    {
                        //if in battle and have no weapon, run away, fixme: when in BS_HUNT_AND_KILL, they just stand there
                        if bState != bState_t::BS_FLEE as c_int {
                            // Guaranteed `Some` — covered by the `enemy.is_none()` guard above.
                            let enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
                            let enemy_origin = ctx.world.entity(enemy_id).r.currentOrigin;
                            crate::NPC_behavior::NPC_StartFlee(ctx, Some(enemy_id), enemy_origin, alertEventLevel_e::AEL_DANGER_GREAT as c_int, 5000, 10000);
                        } else {
                            crate::NPC_behavior::NPC_BSFlee(ctx);
                        }
                        return;
                    }
                    if (*client).ps.weapon == WP_SABER {
                        //special melee exception
                        NPC_BehaviorSet_Default(ctx, bState);
                        return;
                    }
                    if (*client).ps.weapon == WP_DISRUPTOR
                        && ((*npc_info).scriptFlags & 0x00000040) != 0
                    {
                        //a sniper (SCF_ALT_FIRE)
                        NPC_BehaviorSet_Sniper(ctx, bState);
                        return;
                    }
                    if (*client).ps.weapon == WP_THERMAL || (*client).ps.weapon == WP_STUN_BATON {
                        //a grenadier //FIXME: separate AI for melee fighters
                        NPC_BehaviorSet_Grenadier(ctx, bState);
                        return;
                    }
                    if crate::NPC_behavior::NPC_CheckSurrender(ctx) != 0 {
                        return;
                    }
                    NPC_BehaviorSet_Stormtrooper(ctx, bState);
                }
                x if x == crate::teams::npcteam::NPCTEAM_NEUTRAL => {
                    // special cases for enemy droids
                    if (*client).NPC_class == class_t::CLASS_PROTOCOL
                        || (*client).NPC_class == class_t::CLASS_UGNAUGHT
                        || (*client).NPC_class == class_t::CLASS_JAWA
                    {
                        NPC_BehaviorSet_Default(ctx, bState);
                    } else if (*client).NPC_class == class_t::CLASS_VEHICLE {
                        // TODO: Add vehicle behaviors here.
                        crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1); //just face our spawn angles for now
                    } else {
                        // Just one of the average droids
                        NPC_BehaviorSet_Droid(ctx, bState);
                    }
                }
                _ => {
                    if (*client).NPC_class == class_t::CLASS_SEEKER {
                        NPC_BehaviorSet_Seeker(ctx, bState);
                    } else {
                        if (*npc_info).charmedTime > ctx.world.level.time {
                            NPC_BehaviorSet_Charmed(ctx, bState);
                        } else {
                            NPC_BehaviorSet_Default(ctx, bState);
                        }
                        crate::NPC_utils::NPC_CheckCharmed(ctx);
                    }
                }
            }
        }
    }
}

/// Raven `NPC_ExecuteBState`.
///
/// Source: `oracle/codemp/game/NPC.c:1576-1762`
pub fn NPC_ExecuteBState(ctx: &mut GameContext, self_: EntityId) {
    // STAGE-1: `self_` is unused by the body (it drives off the `NPC` global set
    // by the preceding `SetNPCGlobals`); signature is `EntityId`, no re-derive.
    let _ = self_;
    use mp_bg::public::anim_number::animNumber_t;
    use mp_bg::public::weaponstate::weaponstate_t::{WEAPON_IDLE, WEAPON_READY};
    use mp_qshared::common::mp::qcommon::usercmd_button::{BUTTON_ALT_ATTACK, BUTTON_ATTACK};

    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) and NPC pool `gclient_t` have no accessor; the
    // pointers are read via the safe entity borrow and dereffed raw exactly as
    // Raven does (recipe 2b/2c).
    let npc_info = ctx.world.globals.NPCInfo;
    let client = ctx.world.globals.client;
    unsafe {
        NPC_HandleAIFlags(ctx);

        //FIXME: these next three bits could be a function call, some sort of setup/cleanup func
        //Lookmode must be reset every think cycle
        if ctx.world.entity(npc_id).delayScriptTime != 0
            && ctx.world.entity(npc_id).delayScriptTime <= ctx.world.level.time
        {
            crate::NPC_utils::G_ActivateBehavior(ctx, Some(npc_id), bSet_t::BSET_DELAYED as c_int);
            ctx.world.entity_mut(npc_id).delayScriptTime = 0;
        }

        //Clear this and let bState set it itself, so it automatically handles changing bStates... but we need a set bState wrapper func
        (*npc_info).combatMove = 0;

        //Execute our bState
        let bState = if (*npc_info).tempBehavior as c_int != 0 {
            //Overrides normal behavior until cleared
            (*npc_info).tempBehavior
        } else {
            if (*npc_info).behaviorState as c_int == 0 {
                (*npc_info).behaviorState = (*npc_info).defaultBehavior;
            }
            (*npc_info).behaviorState
        };

        //Pick the proper bstate for us and run it
        NPC_RunBehavior(ctx, (*client).playerTeam as c_int, bState as c_int);

        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            if ctx.world.entity(enemy_id).inuse == 0 {
                //just in case bState doesn't catch this
                crate::NPC_combat::G_ClearEnemy(ctx, npc_id);
            }
        }

        if (*client).ps.saberLockTime != 0 && (*client).ps.saberLockEnemy != ENTITYNUM_NONE {
            let look_time = ctx.world.level.time + 1000;
            let saber_enemy = (*client).ps.saberLockEnemy;
            let look_ent = ctx.entity_mut(npc_id);
            crate::NPC_utils::NPC_SetLookTarget(look_ent, saber_enemy, look_time);
        } else if crate::NPC_utils::NPC_CheckLookTarget(ctx, npc_id) == 0 {
            if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
                let enemy_number = ctx.world.entity(enemy_id).s.number;
                let look_ent = ctx.entity_mut(npc_id);
                crate::NPC_utils::NPC_SetLookTarget(look_ent, enemy_number, 0);
            }
        }

        if let Some(enemy_id) = ctx.world.entity(npc_id).enemy {
            if (ctx.world.entity(enemy_id).flags & FL_DONT_SHOOT) != 0 {
                ctx.world.globals.ucmd.buttons &= !BUTTON_ATTACK;
                ctx.world.globals.ucmd.buttons &= !BUTTON_ALT_ATTACK;
            } else if (*client).playerTeam != crate::teams::npcteam::NPCTEAM_ENEMY {
                // FLAG: enemy's gNPC_t (NPCInfo) has no accessor; deref stays raw.
                let enemy_npc = ctx.world.entity(enemy_id).NPC;
                if !enemy_npc.is_null()
                    && ((*enemy_npc).surrenderTime > ctx.world.level.time
                        || ((*enemy_npc).scriptFlags & 0x00010000) != 0)
                {
                    //don't shoot someone who's surrendering if you're a good guy (SCF_FORCED_MARCH)
                    ctx.world.globals.ucmd.buttons &= !BUTTON_ATTACK;
                    ctx.world.globals.ucmd.buttons &= !BUTTON_ALT_ATTACK;
                }
            }

            if (*client).ps.weaponstate == WEAPON_IDLE as c_int {
                (*client).ps.weaponstate = WEAPON_READY as c_int;
            }
        } else if (*client).ps.weaponstate == WEAPON_READY as c_int {
            (*client).ps.weaponstate = WEAPON_IDLE as c_int;
        }

        if (ctx.world.globals.ucmd.buttons & BUTTON_ATTACK) == 0
            && ctx.world.entity(npc_id).attackDebounceTime > ctx.world.level.time
        {
            //We just shot but aren't still shooting, so hold the gun up for a while
            if (*client).ps.weapon == WP_SABER {
                //One-handed
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_TORSO,
                    animNumber_t::TORSO_WEAPONREADY1 as c_int,
                    SETANIM_FLAG_NORMAL,
                );
            } else if (*client).ps.weapon == WP_BRYAR_PISTOL {
                //Sniper pose
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_TORSO,
                    animNumber_t::TORSO_WEAPONREADY3 as c_int,
                    SETANIM_FLAG_NORMAL,
                );
            }
        } else if ctx.world.entity(npc_id).enemy.is_none() {
            //HACK!
            if ctx.world.entity(npc_id).s.torsoAnim == animNumber_t::TORSO_WEAPONREADY1 as c_int
                || ctx.world.entity(npc_id).s.torsoAnim == animNumber_t::TORSO_WEAPONREADY3 as c_int
            {
                //we look ready for action, using one of the first 2 weapon, let's rest our weapon on our shoulder
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_TORSO,
                    animNumber_t::TORSO_WEAPONIDLE3 as c_int,
                    SETANIM_FLAG_NORMAL,
                );
            }
        }

        NPC_CheckAttackHold(ctx);
        NPC_ApplyScriptFlags(ctx);

        //cliff and wall avoidance
        NPC_AvoidWallsAndCliffs();

        // run the bot through the server like it was a real client
        //=== Save the ucmd for the second no-think Pmove ============================
        ctx.world.globals.ucmd.serverTime = ctx.world.level.time - 50;
        (*npc_info).last_ucmd = ctx.world.globals.ucmd;
        if (*npc_info).attackHoldTime == 0 {
            (*npc_info).last_ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
            //so we don't fire twice in one think
        }
        //============================================================================
        NPC_CheckAttackScript(ctx);
        NPC_KeepCurrentFacing(ctx);

        if ctx.world.entity(npc_id).next_roff_time == 0
            || ctx.world.entity(npc_id).next_roff_time < ctx.world.level.time
        {
            //If we were following a roff, we don't do normal pmoves.
            let mut ucmd = ctx.world.globals.ucmd;
            let number = ctx.world.entity(npc_id).s.number;
            crate::g_active::ClientThink(ctx, number, &mut ucmd as *mut usercmd_t);
        } else {
            crate::NPC_move::NPC_ApplyRoff(ctx);
        }

        // end of thinking cleanup
        (*npc_info).touchedByPlayer = None;

        crate::NPC_reactions::NPC_CheckPlayerAim();
        crate::NPC_reactions::NPC_CheckAllClear();
    }
}

/// Raven `NPC_CheckInSolid`.
///
/// Source: `oracle/codemp/game/NPC.c:1764-1785`
pub fn NPC_CheckInSolid(ctx: &mut GameContext) {
    let npc_ent = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc_ent).unwrap();
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    // `npc_ent` is retained only as the raw ABI handle passed across the seam.
    let npc_info = ctx.world.globals.NPCInfo;
    unsafe {
        let mut point = ctx.world.entity(npc_id).r.currentOrigin;
        point[2] -= 0.25;

        let currentOrigin = ctx.world.entity(npc_id).r.currentOrigin;
        let mins = ctx.world.entity(npc_id).r.mins;
        let maxs = ctx.world.entity(npc_id).r.maxs;
        let number = ctx.world.entity(npc_id).s.number;
        let clipmask = ctx.world.entity(npc_id).clipmask;
        let mut trace: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &currentOrigin as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &point as *const vec3_t,
                number,
                clipmask,
            ),
        );
        if trace.startsolid == 0 && trace.allsolid == 0 {
            (*npc_info).lastClearOrigin = ctx.world.entity(npc_id).r.currentOrigin;
        } else if crate::q_math::VectorLengthSquared((*npc_info).lastClearOrigin) != 0.0 {
            //			Com_Printf("%s stuck in solid at %s: fixing...\n", NPC->script_targetname, vtos(NPC->r.currentOrigin));
            let lco = (*npc_info).lastClearOrigin;
            crate::g_utils::G_SetOrigin(ctx.world.entity_mut(npc_id), lco);
            trap::LinkEntity(
                ctx.engine,
                mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(npc_ent.cast()),
            );
        }
    }
}

/// Raven `G_DroidSounds`.
///
/// Source: `oracle/codemp/game/NPC.c:1787-1814`
pub fn G_DroidSounds(ctx: &mut GameContext, self_: EntityId) {
    // FLAG: NPC pool `gclient_t` has no accessor; pointer read via the safe
    // entity borrow, dereffed raw (recipe 2b).
    let client = ctx.world.entity(self_).client;
    if client.is_null() {
        return;
    }
    unsafe {
        // Raven: make the noises
        if crate::g_timer::TIMER_Done(ctx, Some(self_), c"patrolNoise".as_ptr()) != 0
            && ctx.world.bg_state.rng.Q_irand(0, 20) == 0
        {
            let npc_class = (*client).NPC_class;
            let sound_path = match npc_class {
                class_t::CLASS_R2D2 => {
                    let idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                    Some(format!("sound/chars/r2d2/misc/r2d2talk0{}.wav", idx))
                }
                class_t::CLASS_R5D2 => {
                    let idx = ctx.world.bg_state.rng.Q_irand(1, 4);
                    Some(format!("sound/chars/r5d2/misc/r5talk{}.wav", idx))
                }
                class_t::CLASS_PROBE => {
                    let idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                    Some(format!("sound/chars/probe/misc/probetalk{}.wav", idx))
                }
                class_t::CLASS_MOUSE => {
                    let idx = ctx.world.bg_state.rng.Q_irand(1, 3);
                    Some(format!("sound/chars/mouse/misc/mousego{}.wav", idx))
                }
                class_t::CLASS_GONK => {
                    let idx = ctx.world.bg_state.rng.Q_irand(1, 2);
                    Some(format!("sound/chars/gonk/misc/gonktalk{}.wav", idx))
                }
                // Oracle switch has no default: unmatched classes skip the per-class
                // sound but still fall through to the TIMER_Set + draw below.
                // Source: oracle/codemp/game/NPC.c:1793-1810
                _ => None,
            };

            if let Some(sound_path) = sound_path {
                crate::g_utils::G_SoundOnEnt(ctx, self_, CHAN_AUTO, cstr(&sound_path).as_ptr());
            }

            let duration = ctx.world.bg_state.rng.Q_irand(2000, 4000);
            crate::g_timer::TIMER_Set(ctx, Some(self_), c"patrolNoise".as_ptr(), duration);
        }
    }
}

/// Raven `NPC_Think`.
///
/// Source: `oracle/codemp/game/NPC.c:1826-1979`
pub fn NPC_Think(ctx: &mut GameContext, self_: EntityId) {
    // `PMF_FOLLOW` (pm_flags) resolves to its canonical const via the prelude glob.
    // `FRAMETIME` (`g_local.h:37` = 100) — single-owner header, deliberately
    // kept local (not consolidated).
    const FRAMETIME: c_int = 100;
    use mp_bg::vehicles::vehicle_s::Vehicle_t;

    ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME;

    SetNPCGlobals(ctx, self_);

    ctx.world.globals.ucmd = usercmd_t::default();

    // FLAG: gNPC_t (NPCInfo, `npc`) and NPC pool `gclient_t` (`client`) have no
    // accessor; the pointers are read via the safe entity borrow and dereffed
    // raw exactly as Raven does (recipe 2b/2c).
    // Raven reads `self->client->ps.moveDir` unconditionally before the
    // null check below (`self->client` is always valid by the time
    // `NPC_Think` is wired as an entity think — matching the oracle's
    // implicit non-null assumption here).
    let client = ctx.world.entity(self_).client;
    unsafe {
        let oldMoveDir = (*client).ps.moveDir;
        if ctx.world.entity(self_).s.NPC_class != class_t::CLASS_VEHICLE as c_int {
            //YOU ARE BREAKING MY PREDICTION. Bad clear.
            (*client).ps.moveDir = VEC3_ORIGIN;
        }

        // Raven's `self` NULL guard is vacuous behind the `EntityId` handle
        // (dropped, §F2); the `NPC`/`client` NULL guards are preserved.
        if ctx.world.entity(self_).NPC.is_null() || ctx.world.entity(self_).client.is_null() {
            return;
        }

        let npc = ctx.world.entity(self_).NPC;

        // dead NPCs have a special think, don't run scripts (for now)
        //FIXME: this breaks deathscripts
        if ctx.world.entity(self_).health <= 0 {
            DeadThink(ctx);
            if (*npc).nextBStateThink <= ctx.world.level.time {
                let number = ctx.world.entity(self_).s.number;
                trap::ICARUS_MaintainTaskManager(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ICARUS_MAINTAINTASKMANAGER::GIcarusMaintaintaskmanagerArgs::new(number),
                );
            }
            (*client).ps.origin = ctx.world.entity(self_).r.currentOrigin;
            return;
        }

        // see if NPC ai is frozen. `SVF_ICARUS_FREEZE` (g_public.h = 0x8000)
        // resolves to the canonical `crate::g_public_consts` const via the
        // prelude glob (the former local const here had a guessed 0x400, so the
        // freeze check masked the wrong svFlags bit — a live bug).
        if ctx.world.cvars.debugNPCFreeze.value != 0.0
            || (ctx.world.entity(self_).r.svFlags & SVF_ICARUS_FREEZE) != 0
        {
            crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
            let mut ucmd = ctx.world.globals.ucmd;
            let number = ctx.world.entity(self_).s.number;
            crate::g_active::ClientThink(ctx, number, &mut ucmd as *mut usercmd_t);
            (*client).ps.origin = ctx.world.entity(self_).r.currentOrigin;
            return;
        }

        ctx.world.entity_mut(self_).nextthink = ctx.world.level.time + FRAMETIME / 2;

        for i in 0..MAX_CLIENTS {
            let player_id = EntityId::from_num(i as c_int).unwrap();
            // FLAG: player-slot `gclient_t` read via the safe entity borrow;
            // condition is pure and its body is dead (below), so the raw derefs
            // are only reads.
            let player_client = ctx.world.entity(player_id).client;
            if ctx.world.entity(player_id).inuse != 0
                && !player_client.is_null()
                && (*player_client).sess.sessionTeam != TEAM_SPECTATOR
                && (((*player_client).ps.pm_flags & PMF_FOLLOW) == 0)
            {
                // Raven `if (0) //rwwFIXMEFIXME: Allow controlling ents` — this
                // whole arm is dead in the shipped oracle (condition always
                // false); dropped per porting-rules §20 (preserve emergent
                // quirks, drop dead surface) with this note.
            }
        }

        if (*client).NPC_class == class_t::CLASS_VEHICLE {
            if (*client).ps.m_iVehicleNum != 0 {
                //we don't think on our own
                //well, run scripts, though...
                let number = ctx.world.entity(self_).s.number;
                trap::ICARUS_MaintainTaskManager(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ICARUS_MAINTAINTASKMANAGER::GIcarusMaintaintaskmanagerArgs::new(number),
                );
                return;
            } else {
                (*client).ps.moveDir = VEC3_ORIGIN;
                (*client).pers.cmd.forwardmove = 0;
                (*client).pers.cmd.rightmove = 0;
                (*client).pers.cmd.upmove = 0;
                (*client).pers.cmd.buttons = 0;
                // §19: oracle derefs `self->m_pVehicle->m_ucmd` unconditionally.
                // Source: oracle/codemp/game/NPC.c:1914
                // FLAG: `m_pVehicle` (Vehicle_t*) has no accessor; deref stays raw.
                if !ctx.world.entity(self_).m_pVehicle.is_null() {
                    let veh = ctx.world.entity(self_).m_pVehicle;
                    let cmd = (*client).pers.cmd;
                    (*veh).m_ucmd = cmd;
                }
            }
        } else if ctx.world.entity(self_).s.m_iVehicleNum != 0 {
            //droid in a vehicle?
            G_DroidSounds(ctx, self_);
        }

        if (*npc).nextBStateThink <= ctx.world.level.time
            && ctx.world.entity(self_).s.m_iVehicleNum == 0
        {
            //NPCs sitting in Vehicles do NOTHING
            if ctx.world.entity(self_).s.eType != entityType_t::ET_NPC as c_int {
                //Something drastic happened in our script
                return;
            }

            if ctx.world.entity(self_).s.weapon == WP_SABER
                && ctx.world.cvars.g_spskill.integer >= 2
                && (*npc).rank > RANK_LT_JG
            {
                //Jedi think faster on hard difficulty, except low-rank (reborn)
                (*npc).nextBStateThink = ctx.world.level.time + FRAMETIME / 2;
            } else {
                //Maybe even 200 ms?
                (*npc).nextBStateThink = ctx.world.level.time + FRAMETIME;
            }

            //nextthink is set before this so something in here can override it
            if ctx.world.entity(self_).s.NPC_class != class_t::CLASS_VEHICLE as c_int
                || ctx.world.entity(self_).m_pVehicle.is_null()
            {
                //ok, let's not do this at all for vehicles.
                NPC_ExecuteBState(ctx, self_);
            }
        } else {
            (*client).ps.moveDir = oldMoveDir;
            //or use client->pers.lastCommand?
            (*npc).last_ucmd.serverTime = ctx.world.level.time - 50;
            if ctx.world.entity(self_).next_roff_time == 0
                || ctx.world.entity(self_).next_roff_time < ctx.world.level.time
            {
                //If we were following a roff, we don't do normal pmoves.
                //FIXME: firing angles (no aim offset) or regular angles?
                crate::NPC_utils::NPC_UpdateAngles(ctx, 1, 1);
                ctx.world.globals.ucmd = (*npc).last_ucmd;
                let mut ucmd = ctx.world.globals.ucmd;
                let number = ctx.world.entity(self_).s.number;
                crate::g_active::ClientThink(ctx, number, &mut ucmd as *mut usercmd_t);
            } else {
                crate::NPC_move::NPC_ApplyRoff(ctx);
            }
        }
        //must update icarus *every* frame because of certain animation completions in the pmove stuff that can leave a 50ms gap between ICARUS animation commands
        let number = ctx.world.entity(self_).s.number;
        trap::ICARUS_MaintainTaskManager(
            ctx.engine,
            mp_abi::game::syscalls::G_ICARUS_MAINTAINTASKMANAGER::GIcarusMaintaintaskmanagerArgs::new(number),
        );
        (*client).ps.origin = ctx.world.entity(self_).r.currentOrigin;
    }
}

/// Raven `NPC_InitAI`.
///
/// Raven: real body is `#if 0`'d out (cvar registration commented upstream);
/// live function is an empty no-op — ported faithfully as-is.
/// Source: `oracle/codemp/game/NPC.c:1981-2009`
pub fn NPC_InitAI() {}

/// Raven `NPC_InitGame`.
///
/// Source: `oracle/codemp/game/NPC.c:2041-2056`
pub fn NPC_InitGame(ctx: &mut GameContext) {
    NPC_LoadParms(ctx);
    NPC_InitAI();
}

/// Raven `NPC_SetAnim`.
///
/// Raven: forwards straight to `G_SetAnim` with a null `usercmd_t*` — the
/// real per-torso/legs anim-timer logic below it in the oracle is `#if 0`'d
/// out upstream, so this is the whole live body.
/// Source: `oracle/codemp/game/NPC.c:2058-2110`
pub fn NPC_SetAnim(
    ctx: &mut GameContext,
    ent: EntityId,
    setAnimParts: c_int,
    anim: c_int,
    setAnimFlags: c_int,
) {
    G_SetAnim(
        ctx,
        ent,
        core::ptr::null_mut(),
        setAnimParts,
        anim,
        setAnimFlags,
        0,
    );
}
