// PORT-COMPLETE: g_active.c 31/5
//! Port of `oracle/oracle/codemp/game/g_active.c` (jampgame pass 2).
//!
//! State reached through `ctx.world` (STATE-D6 leaf reborrows), traps through
//! `trap::X(ctx.engine, …)`, file-scope globals/cvars via the pre-merged
//! `GameWorld` fields (fork ruling 1). Five functions remain parked: the three
//! entity-`touch` fn-pointer callers (`ClientImpacts`, `G_TouchTriggers`,
//! `G_MoverTouchPushTriggers`) and the two `Pmove` drivers (`SpectatorThink`,
//! `ClientThink_real`) — both blocked on storing engine-bearing/ctx-carrying
//! handlers in the frozen ABI struct's raw `Option<extern "C" fn>` fields.
#![allow(non_snake_case, non_camel_case_types, unused, clippy::all)]

use crate::prelude::*;
use crate::trap;
use crate::q_math::vec3_origin;

// Const families ported elsewhere but not in the shared prelude glob.
use crate::client::client_connected::CON_CONNECTED;
use crate::client::spectator_state::spectatorState_t::*;
use crate::entity::flags::*; // FL_GODMODE / FL_BBRUSH / …
use crate::level::damage_flags::*; // DAMAGE_NO_ARMOR / …
use mp_bg::public::anim_number::animNumber_t::*; // BOTH_* / TORSO_* / MAX_ANIMATIONS
use mp_bg::public::entity_event::entity_event_t::*; // EV_*
use mp_bg::public::gametype::*; // GT_*
use mp_bg::public::weaponstate::weaponstate_t::*; // WEAPON_*
use mp_bg::vehicles::vehicle_type_t::vehicleType_t::*; // VH_SPEEDER
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t::*; // SS_*
use mp_qshared::common::mp::qcommon::usercmd_button::*; // BUTTON_*
use mp_qshared::shared::saber_blocked_type::saberBlockedType_t::*; // BLOCKED_*
use mp_qshared::shared::trajectory::trType_t::*; // TR_GRAVITY

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites
// (same convention as `g_combat.rs`).
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Raven `PITCH`/`YAW`/`ROLL` — Euler-angle component indices.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

// Raven `PMF_FOLLOW` (`playerState_t::pm_flags` bit).
// Source: `oracle/oracle/codemp/game/bg_public.h:415`
const PMF_FOLLOW: c_int = 4096;

// MAT_*/SVF_*/PMF_SCOREBOARD now resolve via the crate prelude (pass-3 symbol
// backfill: `mp_qshared::common::mp::gentity`, `crate::g_public_consts`,
// `mp_qshared::common::mp::qcommon::pm_flags`).

// Raven `#define MAX_SIGHT_DISTANCE`/`MAX_SIGHT_FOV`/`MAX_JEDIMASTER_DISTANCE`/
// `MAX_JEDIMASTER_FOV` — file-scope in `g_active.c` (not referenced elsewhere).
// Source: `oracle/oracle/codemp/game/g_active.c:1097-1101`
const MAX_SIGHT_DISTANCE: c_float = 1500.0;
const MAX_SIGHT_FOV: c_float = 100.0;
const MAX_JEDIMASTER_DISTANCE: c_float = 2500.0;
const MAX_JEDIMASTER_FOV: c_float = 100.0;

// Raven's taunt selector is a file-scope anonymous `enum { TAUNT_TAUNT = 0,
// TAUNT_BOW, TAUNT_MEDITATE, TAUNT_FLOURISH, TAUNT_GLOAT };` in `g_active.c`
// (no typedef name), so per enum-vs-alias fidelity these are plain `c_int`
// consts, private to this file like the Raven original.
// Source: `oracle/oracle/codemp/game/g_active.c:1652-1659`
const TAUNT_TAUNT: c_int = 0;
const TAUNT_BOW: c_int = 1;
const TAUNT_MEDITATE: c_int = 2;
const TAUNT_FLOURISH: c_int = 3;
const TAUNT_GLOAT: c_int = 4;

//TODO: Port VectorCompare           // Source: oracle/oracle/codemp/game/q_shared.h

// Resolved cross-module fns (verbatim post-retrofit signatures — call surface).
use crate::bg_misc::{BG_PlayerStateToEntityState, BG_PlayerStateToEntityStateExtraPolate, vectoyaw};
use crate::bg_panimate::BG_AnimLength;
use crate::g_client::{respawn, ClientBegin, SetClientViewAngle};
use crate::g_cmds::{Cmd_FollowCycle_f, SetTeam, StopFollowing};
use crate::g_combat::{G_ApplyKnockback, G_Damage};
use crate::g_items::{
    ItemUse_Binoculars, ItemUse_Jetpack, ItemUse_MedPack, ItemUse_MedPack_Big, ItemUse_Seeker,
    ItemUse_Sentry, ItemUse_Shield, ItemUse_UseCloak, ItemUse_UseEWeb,
};
use crate::g_nav::FlyingCreature;
use crate::g_utils::{
    G_AddEvent, G_SetAngles, G_SetAnim, G_SetOrigin, G_Sound, G_SoundIndex, G_TempEntity, TryUse,
};
use crate::g_weapon::FireWeapon;
use crate::ai_main::InFieldOfVision;
use crate::bg_g2_utils::BG_AttachToRancor;
use crate::npc_c::NPC_SetAnim;
use crate::q_math::{vectoangles, AngleVectors, Q_irand, VectorLength, VectorLengthSquared, VectorNormalize};
use mp_bg::public::pmove_t::Pmove;

use crate::npc::g_npc_t::gNPC_t;

/// Raven `P_SetTwitchInfo`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:20-24`
pub fn P_SetTwitchInfo(ctx: GameContext<'_>, client: *mut gclient_t) {
    unsafe {
        (*client).ps.painTime = (*ctx.world).level.time;
        (*client).ps.painDirection ^= 1;
    }
}

/// Raven `P_DamageFeedback` — send the damage-blend/pain feedback for a frame.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:36-118`
pub fn P_DamageFeedback(ctx: GameContext<'_>, player: *mut gentity_t) {
    unsafe {
        let client = (*player).client as *mut gclient_t;
        if (*client).ps.pm_type == PM_DEAD {
            return;
        }

        // total points of damage shot at the player this frame
        let mut count: f32 = ((*client).damage_blood + (*client).damage_armor) as f32;
        if count == 0.0 {
            return; // didn't take any damage
        }

        if count > 255.0 {
            count = 255.0;
        }

        // world damage (falling, slime, etc) uses a special code to make the
        // blend blob centered instead of positional
        if (*client).damage_fromWorld != 0 {
            (*client).ps.damagePitch = 255;
            (*client).ps.damageYaw = 255;

            (*client).damage_fromWorld = qfalse;
        } else {
            let mut angles: vec3_t = [0.0; 3];
            vectoangles((*client).damage_from, &mut angles);
            (*client).ps.damagePitch = (angles[PITCH] / 360.0 * 256.0) as c_int;
            (*client).ps.damageYaw = (angles[YAW] / 360.0 * 256.0) as c_int;

            //cap them since we can't send negative values in here across the net
            if (*client).ps.damagePitch < 0 {
                (*client).ps.damagePitch = 0;
            }
            if (*client).ps.damageYaw < 0 {
                (*client).ps.damageYaw = 0;
            }
        }

        // play an apropriate pain sound
        if ((*ctx.world).level.time > (*player).pain_debounce_time)
            && ((*player).flags & FL_GODMODE) == 0
            && ((*player).s.eFlags & EF_DEAD) == 0
        {
            // don't do more than two pain sounds a second
            // nmckenzie: also don't make him loud and whiny if he's only getting nicked.
            if (*ctx.world).level.time - (*client).ps.painTime < 500 || count < 10.0 {
                return;
            }
            P_SetTwitchInfo(ctx, client);
            (*player).pain_debounce_time = (*ctx.world).level.time + 700;

            G_AddEvent(player, EV_PAIN as c_int, (*player).health);
            (*client).ps.damageEvent += 1;

            if (*client).damage_armor != 0 && (*client).damage_blood == 0 {
                (*client).ps.damageType = 1; //pure shields
            } else if (*client).damage_armor != 0 {
                (*client).ps.damageType = 2; //shields and health
            } else {
                (*client).ps.damageType = 0; //pure health
            }
        }

        (*client).ps.damageCount = count as c_int;

        // clear totals
        (*client).damage_blood = 0;
        (*client).damage_armor = 0;
        (*client).damage_knockback = 0;
    }
}

/// Raven `P_WorldEffects` — drowning + lava/slime sizzle damage.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:129-205`
pub fn P_WorldEffects(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        if (*client).noclip != 0 {
            (*client).airOutTime = (*ctx.world).level.time + 12000; // don't need air
            return;
        }

        let waterlevel = (*ent).waterlevel;

        let envirosuit: qboolean =
            ((*client).ps.powerups[PW_BATTLESUIT as usize] > (*ctx.world).level.time) as c_int;

        // check for drowning
        if waterlevel == 3 {
            // envirosuit give air
            if envirosuit != 0 {
                (*client).airOutTime = (*ctx.world).level.time + 10000;
            }

            // if out of air, start drowning
            if (*client).airOutTime < (*ctx.world).level.time {
                // drown!
                (*client).airOutTime += 1000;
                if (*ent).health > 0 {
                    // take more damage the longer underwater
                    (*ent).damage += 2;
                    if (*ent).damage > 15 {
                        (*ent).damage = 15;
                    }

                    // play a gurp sound instead of a normal pain sound
                    if (*ent).health <= (*ent).damage {
                        G_Sound(ctx, ent, CHAN_VOICE as c_int, G_SoundIndex(cstr("sound/player/gurp1.wav").as_ptr()));
                    } else if (*ctx.world).bg_state.rng.rand() & 1 != 0 {
                        G_Sound(ctx, ent, CHAN_VOICE as c_int, G_SoundIndex(cstr("sound/player/gurp1.wav").as_ptr()));
                    } else {
                        G_Sound(ctx, ent, CHAN_VOICE as c_int, G_SoundIndex(cstr("sound/player/gurp2.wav").as_ptr()));
                    }

                    // don't play a normal pain sound
                    (*ent).pain_debounce_time = (*ctx.world).level.time + 200;

                    G_Damage(
                        ent,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        [0.0; 3],
                        [0.0; 3],
                        (*ent).damage,
                        DAMAGE_NO_ARMOR,
                        MOD_WATER as c_int,
                    );
                }
            }
        } else {
            (*client).airOutTime = (*ctx.world).level.time + 12000;
            (*ent).damage = 2;
        }

        // check for sizzle damage (move to pmove?)
        if waterlevel != 0 && ((*ent).watertype & (CONTENTS_LAVA | CONTENTS_SLIME)) != 0 {
            if (*ent).health > 0 && (*ent).pain_debounce_time <= (*ctx.world).level.time {
                if envirosuit != 0 {
                    G_AddEvent(ent, EV_POWERUP_BATTLESUIT as c_int, 0);
                } else {
                    if (*ent).watertype & CONTENTS_LAVA != 0 {
                        G_Damage(
                            ent,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            [0.0; 3],
                            [0.0; 3],
                            30 * waterlevel,
                            0,
                            MOD_LAVA as c_int,
                        );
                    }

                    if (*ent).watertype & CONTENTS_SLIME != 0 {
                        G_Damage(
                            ent,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            [0.0; 3],
                            [0.0; 3],
                            10 * waterlevel,
                            0,
                            MOD_SLIME as c_int,
                        );
                    }
                }
            }
        }
    }
}

/// Raven `DoImpact` — collision impact damage (crush/fall).
///
/// Source: `oracle/oracle/codemp/game/g_active.c:213-405`
pub fn DoImpact(ctx: GameContext<'_>, self_: *mut gentity_t, other: *mut gentity_t, damageSelf: qboolean) {
    unsafe {
        let mut velocity: vec3_t = [0.0; 3];
        let mut my_mass: f32;
        let cont: c_int;
        let mut easyBreakBrush: qboolean = qtrue;

        let selfCl = (*self_).client as *mut gclient_t;
        if !selfCl.is_null() {
            velocity = (*selfCl).ps.velocity;
            if (*self_).mass == 0.0 {
                my_mass = 10.0;
            } else {
                my_mass = (*self_).mass;
            }
        } else {
            velocity = (*self_).s.pos.trDelta;
            if (*self_).s.pos.trType == TR_GRAVITY {
                velocity[2] -= 0.25 * (*ctx.world).cvars.g_gravity.value;
            }
            if (*self_).mass == 0.0 {
                my_mass = 1.0;
            } else if (*self_).mass <= 10.0 {
                my_mass = 10.0;
            } else {
                my_mass = (*self_).mass; ///10;
            }
        }

        let mut magnitude: f32 = VectorLength(velocity) * my_mass / 10.0;

        if (*other).material == MAT_GLASS
            || (*other).material == MAT_GLASS_METAL
            || (*other).material == MAT_GRATE1
            || (((*other).flags & FL_BBRUSH) != 0 && ((*other).spawnflags & 8 /*THIN*/) != 0)
            || ((*other).r.svFlags & SVF_GLASS_BRUSH) != 0
        {
            easyBreakBrush = qtrue;
        }

        if selfCl.is_null()
            || (*selfCl).ps.lastOnGround + 300 < (*ctx.world).level.time
            || ((*selfCl).ps.lastOnGround + 100 < (*ctx.world).level.time && easyBreakBrush != 0)
        {
            let mut dir1: vec3_t = [0.0; 3];
            let mut dir2: vec3_t = [0.0; 3];
            let mut force: f32 = 0.0;
            let dot: f32;

            if easyBreakBrush != 0 {
                magnitude *= 2.0;
            }

            //damage them
            if magnitude >= 100.0 && (*other).s.number < ENTITYNUM_WORLD {
                dir1 = velocity;
                VectorNormalize(&mut dir1);
                if VectorCompare((*other).r.currentOrigin, vec3_origin) != qfalse {
                    //a brush with no origin
                    dir2 = dir1;
                } else {
                    for i in 0..3 {
                        dir2[i] = (*other).r.currentOrigin[i] - (*self_).r.currentOrigin[i];
                    }
                    VectorNormalize(&mut dir2);
                }

                dot = dir1[0] * dir2[0] + dir1[1] * dir2[1] + dir1[2] * dir2[2];

                if dot >= 0.2 {
                    force = dot;
                } else {
                    force = 0.0;
                }

                force *= magnitude / 50.0;

                cont = trap::PointContents(
                    ctx.engine,
                    mp_abi::game::syscalls::G_POINT_CONTENTS::GPointContentsArgs::new(
                        &(*other).r.absmax as *const vec3_t,
                        (*other).s.number,
                    ),
                );
                if (cont & CONTENTS_WATER) != 0 {
                    force /= 3.0; //water absorbs 2/3 velocity
                }

                if (force >= 1.0 && (*other).s.number != 0) || force >= 10.0 {
                    if (*other).r.svFlags & SVF_GLASS_BRUSH != 0 {
                        (*other).splashRadius =
                            (((*self_).r.maxs[0] - (*self_).r.mins[0]) / 4.0) as c_int;
                    }
                    if (*other).takedamage != 0 {
                        G_Damage(
                            other,
                            self_,
                            self_,
                            velocity,
                            (*self_).r.currentOrigin,
                            force as c_int,
                            DAMAGE_NO_ARMOR,
                            MOD_CRUSH as c_int,
                        ); //FIXME: MOD_IMPACT
                    } else {
                        G_ApplyKnockback(ctx, other, dir2, force);
                    }
                }
            }

            if damageSelf != 0 && (*self_).takedamage != 0 {
                //Now damage me
                if !selfCl.is_null() && (*selfCl).ps.fd.forceJumpZStart != 0.0 {
                    //we were force-jumping
                    if (*self_).r.currentOrigin[2] >= (*selfCl).ps.fd.forceJumpZStart {
                        //we landed at same height or higher than we landed
                        magnitude = 0.0;
                    } else {
                        magnitude =
                            ((*selfCl).ps.fd.forceJumpZStart - (*self_).r.currentOrigin[2]) / 3.0;
                    }
                }
                if (magnitude >= 100.0 + (*self_).health as f32
                    && (*self_).s.number != 0
                    && (*self_).s.weapon != WP_SABER)
                    || (magnitude >= 700.0)
                {
                    //health here is used to simulate structural integrity
                    if ((*self_).s.weapon == WP_SABER || (*self_).s.number == 0)
                        && !selfCl.is_null()
                        && (*selfCl).ps.groundEntityNum < ENTITYNUM_NONE
                        && magnitude < 1000.0
                    {
                        //players and jedi take less impact damage
                        magnitude /= 2.0;
                    }
                    magnitude /= 40.0;
                    magnitude = magnitude - force / 2.0; //If damage other, subtract half of that damage off of own injury
                    if magnitude >= 1.0 {
                        G_Damage(
                            self_,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            [0.0; 3],
                            (*self_).r.currentOrigin,
                            (magnitude / 2.0) as c_int,
                            DAMAGE_NO_ARMOR,
                            MOD_FALLING as c_int,
                        ); //FIXME: MOD_IMPACT
                    }
                }
            }
        }
    }
}

/// Raven `Client_CheckImpactBBrush` — clients only do impact damage vs easy-break breakables.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:407-436`
pub fn Client_CheckImpactBBrush(ctx: GameContext<'_>, self_: *mut gentity_t, other: *mut gentity_t) {
    unsafe {
        if other.is_null() || (*other).inuse == 0 {
            return;
        }
        let selfCl = if self_.is_null() {
            core::ptr::null_mut()
        } else {
            (*self_).client as *mut gclient_t
        };
        if self_.is_null()
            || (*self_).inuse == 0
            || selfCl.is_null()
            || (*selfCl).tempSpectate >= (*ctx.world).level.time
            || (*selfCl).sess.sessionTeam == TEAM_SPECTATOR
        {
            //hmm.. let's not let spectators ram into breakables.
            return;
        }

        if (*other).material == MAT_GLASS
            || (*other).material == MAT_GLASS_METAL
            || (*other).material == MAT_GRATE1
            || (((*other).flags & FL_BBRUSH) != 0 && ((*other).spawnflags & 8 /*THIN*/) != 0)
            || (((*other).flags & FL_BBRUSH) != 0 && (*other).health <= 10)
            || ((*other).r.svFlags & SVF_GLASS_BRUSH) != 0
        {
            //clients only do impact damage against easy-break breakables
            DoImpact(ctx, self_, other, qfalse);
        }
    }
}

/// Raven `G_SetClientSound` — loop-sound selection (hack/heal/supply/lava).
///
/// Source: `oracle/oracle/codemp/game/g_active.c:444-467`
pub fn G_SetClientSound(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;
        if !client.is_null() && (*client).isHacking != 0 {
            //loop hacking sound
            (*client).ps.loopSound = (*ctx.world).level.snd_hack;
            (*ent).s.loopIsSoundset = qfalse;
        } else if !client.is_null() && (*client).isMedHealed > level_time {
            //loop healing sound
            (*client).ps.loopSound = (*ctx.world).level.snd_medHealed;
            (*ent).s.loopIsSoundset = qfalse;
        } else if !client.is_null() && (*client).isMedSupplied > level_time {
            //loop supplying sound
            (*client).ps.loopSound = (*ctx.world).level.snd_medSupplied;
            (*ent).s.loopIsSoundset = qfalse;
        } else if (*ent).waterlevel != 0 && ((*ent).watertype & (CONTENTS_LAVA | CONTENTS_SLIME)) != 0 {
            (*client).ps.loopSound = (*ctx.world).level.snd_fry;
            (*ent).s.loopIsSoundset = qfalse;
        } else {
            (*client).ps.loopSound = 0;
            (*ent).s.loopIsSoundset = qfalse;
        }
    }
}

/// Raven `ClientImpacts` — dispatch `touch` for pmove touch-ents.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:478-506`
// PORT-ESCALATION(fn-pointer-dispatch): calls `ent->touch(...)` / `other->touch(...)`; the frozen ABI `gentity_t.touch` field is a raw `Option<extern "C" fn>`, but the fork-2 ruling stores dispatch as the `EntTouch` fn-ID enum + ctx-carrying handlers — the two cannot be reconciled here.
pub fn ClientImpacts(ctx: GameContext<'_>, ent: *mut gentity_t, pm: *mut pmove_t) {
    todo!("Port ClientImpacts — parked: fn-pointer-dispatch")
}

/// Raven `G_TouchTriggers` — fire trigger `touch` handlers around a client.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:516-590`
// PORT-ESCALATION(fn-pointer-dispatch): calls `hit->touch(...)` and identity-compares `hit->touch != Touch_DoorTrigger`; the raw `Option<extern "C" fn>` field cannot express the fork-2 `EntTouch` enum dispatch/PartialEq nor hold ctx-carrying handlers.
pub fn G_TouchTriggers(ctx: GameContext<'_>, ent: *mut gentity_t) {
    todo!("Port G_TouchTriggers — parked: fn-pointer-dispatch")
}

/// Raven `G_MoverTouchPushTriggers` — fire push-trigger `touch` along a mover's path.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:601-671`
// PORT-ESCALATION(fn-pointer-dispatch): calls `hit->touch(...)`; the raw `Option<extern "C" fn>` field cannot hold the fork-2 ctx-carrying `EntTouch` handlers. (`oldOrg` is read-only here, so the fork-9 reshape leaves it by-value `vec3_t`.)
pub fn G_MoverTouchPushTriggers(ctx: GameContext<'_>, ent: *mut gentity_t, oldOrg: vec3_t) {
    todo!("Port G_MoverTouchPushTriggers — parked: fn-pointer-dispatch")
}

/// Raven `SpectatorThink`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:678-740`
// PORT-ESCALATION(pmove-trace-seam): sets `pm.trace = trap_Trace` / `pm.pointcontents = trap_PointContents` then calls `Pmove`; the engine-bearing `trap::Trace(engine, …)` cannot be stored in `pmove_t`'s raw `Option<extern "C" fn>` field, and `Pmove` itself is parked with no established trace-threading convention.
pub fn SpectatorThink(ctx: GameContext<'_>, ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    todo!("Port SpectatorThink — parked: pmove-trace-seam")
}

/// Raven `ClientInactivityTimer`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:751-774`
pub fn ClientInactivityTimer(ctx: GameContext<'_>, client: *mut gclient_t) -> qboolean {
    unsafe {
        let level_time = (*ctx.world).level.time;
        let clients = (*ctx.world).level.clients;
        if (*ctx.world).cvars.g_inactivity.integer == 0 {
            // give everyone some time, so if the operator sets g_inactivity during
            // gameplay, everyone isn't kicked
            (*client).inactivityTime = level_time + 60 * 1000;
            (*client).inactivityWarning = qfalse;
        } else if (*client).pers.cmd.forwardmove != 0
            || (*client).pers.cmd.rightmove != 0
            || (*client).pers.cmd.upmove != 0
            || ((*client).pers.cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0
        {
            (*client).inactivityTime = level_time + (*ctx.world).cvars.g_inactivity.integer * 1000;
            (*client).inactivityWarning = qfalse;
        } else if (*client).pers.localClient == 0 {
            let clientNum = client.offset_from(clients) as c_int;
            if level_time > (*client).inactivityTime {
                trap::DropClient(
                    ctx.engine,
                    mp_abi::game::syscalls::G_DROP_CLIENT::GDropClientArgs::new(
                        clientNum,
                        cstr("Dropped due to inactivity"),
                    ),
                );
                return qfalse;
            }
            if level_time > (*client).inactivityTime - 10000 && (*client).inactivityWarning == 0 {
                (*client).inactivityWarning = qtrue;
                trap::SendServerCommand(
                    ctx.engine,
                    mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(
                        clientNum,
                        cstr("cp \"Ten seconds until inactivity drop!\n\""),
                    ),
                );
            }
        }
        qtrue
    }
}

/// Raven `ClientTimerActions` — once-a-second health/armor decay over max.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:783-803`
pub fn ClientTimerActions(ent: *mut gentity_t, msec: c_int) {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        (*client).timeResidual += msec;

        while (*client).timeResidual >= 1000 {
            (*client).timeResidual -= 1000;

            // count down health when over max
            if (*ent).health > (*client).ps.stats[STAT_MAX_HEALTH as usize] {
                (*ent).health -= 1;
            }

            // count down armor when over max
            if (*client).ps.stats[STAT_ARMOR as usize] > (*client).ps.stats[STAT_MAX_HEALTH as usize]
            {
                (*client).ps.stats[STAT_ARMOR as usize] -= 1;
            }
        }
    }
}

/// Raven `ClientIntermissionThink`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:810-823`
pub fn ClientIntermissionThink(client: *mut gclient_t) {
    unsafe {
        (*client).ps.eFlags &= !EF_TALK;
        (*client).ps.eFlags &= !EF_FIRING;

        // swap and latch button actions
        (*client).oldbuttons = (*client).buttons;
        (*client).buttons = (*client).pers.cmd.buttons;
        if (*client).buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) & ((*client).oldbuttons ^ (*client).buttons)
            != 0
        {
            // this used to be an ^1 but once a player says ready, it should stick
            (*client).readyToExit = 1;
        }
    }
}

/// Raven `G_VehicleAttachDroidUnit` — snap a droid unit to its vehicle bolt.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:826-854`
pub fn G_VehicleAttachDroidUnit(ctx: GameContext<'_>, vehEnt: *mut gentity_t) {
    unsafe {
        let veh = (*vehEnt).m_pVehicle as *mut Vehicle_t;
        if !vehEnt.is_null() && !veh.is_null() && !(*veh).m_pDroidUnit.is_null() {
            let droidEnt = (*veh).m_pDroidUnit as *mut gentity_t;
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut fwd: vec3_t = [0.0; 3];

            trap::G2API_GetBoltMatrix(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                    (*vehEnt).ghoul2,
                    0,
                    (*veh).m_iDroidUnitTag,
                    &mut boltMatrix as *mut mdxaBone_t,
                    &(*vehEnt).r.currentAngles as *const vec3_t,
                    &(*vehEnt).r.currentOrigin as *const vec3_t,
                    (*ctx.world).level.time,
                    core::ptr::null_mut(),
                    (*vehEnt).modelScale,
                ),
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::ORIGIN as c_int,
                &mut (*droidEnt).r.currentOrigin,
            );
            BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut fwd);
            vectoangles(fwd, &mut (*droidEnt).r.currentAngles);

            let droidCl = (*droidEnt).client as *mut gclient_t;
            if !droidCl.is_null() {
                (*droidCl).ps.viewangles = (*droidEnt).r.currentAngles;
                (*droidCl).ps.origin = (*droidEnt).r.currentOrigin;
            }

            G_SetOrigin(droidEnt, (*droidEnt).r.currentOrigin);
            trap::LinkEntity(
                ctx.engine,
                mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(droidEnt),
            );

            if !(*droidEnt).NPC.is_null() {
                NPC_SetAnim(
                    droidEnt,
                    SETANIM_BOTH as c_int,
                    BOTH_STAND2 as c_int,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                );
            }
        }
    }
}

/// Raven `G_CheapWeaponFire` — server-driven weapon fire event (with speeder gate).
///
/// Source: `oracle/oracle/codemp/game/g_active.c:857-895`
pub fn G_CheapWeaponFire(ctx: GameContext<'_>, entNum: c_int, ev: c_int) {
    unsafe {
        let ent = &mut (*ctx.world).entities[entNum as usize] as *mut gentity_t;

        if (*ent).inuse == 0 || (*ent).client.is_null() {
            return;
        }

        let cl = (*ent).client as *mut gclient_t;

        if ev == EV_FIRE_WEAPON as c_int {
            let veh = (*ent).m_pVehicle as *mut Vehicle_t;
            if !veh.is_null()
                && (*(*veh).m_pVehicleInfo).r#type == VH_SPEEDER
                && (*cl).ps.m_iVehicleNum != 0
            {
                //a speeder with a pilot
                let rider = &mut (*ctx.world).entities[((*cl).ps.m_iVehicleNum - 1) as usize]
                    as *mut gentity_t;
                if (*rider).inuse != 0 && !(*rider).client.is_null() {
                    //pilot is valid...
                    let rcl = (*rider).client as *mut gclient_t;
                    if (*rcl).ps.weapon != WP_MELEE
                        && ((*rcl).ps.weapon != WP_SABER || (*rcl).ps.saberHolstered == 0)
                    {
                        //can only attack on speeder when using melee or when saber is holstered
                        return;
                    }
                }
            }

            FireWeapon(ctx, ent, qfalse);
            (*cl).dangerTime = (*ctx.world).level.time;
            (*cl).ps.eFlags &= !EF_INVULNERABLE;
            (*cl).invulnerableTimer = 0;
        } else if ev == EV_ALT_FIRE as c_int {
            FireWeapon(ctx, ent, qtrue);
            (*cl).dangerTime = (*ctx.world).level.time;
            (*cl).ps.eFlags &= !EF_INVULNERABLE;
            (*cl).invulnerableTimer = 0;
        }
    }
}

/// Raven `ClientEvents` — process predictable client events for the frame.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:909-1052`
pub fn ClientEvents(ctx: GameContext<'_>, ent: *mut gentity_t, oldEventSequence: c_int) {
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let mut oldEventSequence = oldEventSequence;

        if oldEventSequence < (*client).ps.eventSequence - MAX_PS_EVENTS as c_int {
            oldEventSequence = (*client).ps.eventSequence - MAX_PS_EVENTS as c_int;
        }
        let mut i = oldEventSequence;
        while i < (*client).ps.eventSequence {
            let event = (*client).ps.events[(i & (MAX_PS_EVENTS as c_int - 1)) as usize];

            if event == EV_FALL as c_int || event == EV_ROLL as c_int {
                let delta = (*client).ps.eventParms[(i & (MAX_PS_EVENTS as c_int - 1)) as usize];
                let mut knockDownage: qboolean = qfalse;
                let mut damage: c_int;

                'blk: loop {
                    if !(*ent).client.is_null() && (*client).ps.fallingToDeath != 0 {
                        break 'blk;
                    }

                    if (*ent).s.eType != ET_PLAYER as c_int {
                        break 'blk; // not in the player model
                    }

                    if (*ctx.world).cvars.g_dmflags.integer & DF_NO_FALLING != 0 {
                        break 'blk;
                    }

                    if BG_InKnockDownOnly((*client).ps.legsAnim) != 0 {
                        if delta <= 14 {
                            break 'blk;
                        }
                        knockDownage = qtrue;
                    } else {
                        if delta <= 44 {
                            break 'blk;
                        }
                    }

                    if knockDownage != 0 {
                        damage = delta; //you suffer for falling unprepared.
                    } else {
                        if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE && delta > 60 {
                            //longer falls hurt more
                            damage = delta;
                        } else {
                            damage = (delta as f32 * 0.16) as c_int;
                        }
                    }

                    let dir: vec3_t = [0.0, 0.0, 1.0];
                    (*ent).pain_debounce_time = (*ctx.world).level.time + 200; // no normal pain sound
                    G_Damage(
                        ent,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        [0.0; 3],
                        [0.0; 3],
                        damage,
                        DAMAGE_NO_ARMOR,
                        MOD_FALLING as c_int,
                    );

                    if (*ent).health < 1 {
                        G_Sound(ctx, ent, CHAN_AUTO as c_int, G_SoundIndex(cstr("sound/player/fallsplat.wav").as_ptr()));
                    }
                    break 'blk;
                }
            } else if event == EV_FIRE_WEAPON as c_int {
                FireWeapon(ctx, ent, qfalse);
                (*client).dangerTime = (*ctx.world).level.time;
                (*client).ps.eFlags &= !EF_INVULNERABLE;
                (*client).invulnerableTimer = 0;
            } else if event == EV_ALT_FIRE as c_int {
                FireWeapon(ctx, ent, qtrue);
                (*client).dangerTime = (*ctx.world).level.time;
                (*client).ps.eFlags &= !EF_INVULNERABLE;
                (*client).invulnerableTimer = 0;
            } else if event == EV_SABER_ATTACK as c_int {
                (*client).dangerTime = (*ctx.world).level.time;
                (*client).ps.eFlags &= !EF_INVULNERABLE;
                (*client).invulnerableTimer = 0;
            } else if event == EV_USE_ITEM1 as c_int {
                //seeker droid
                ItemUse_Seeker(ctx, ent);
            } else if event == EV_USE_ITEM2 as c_int {
                //shield
                ItemUse_Shield(ctx, ent);
            } else if event == EV_USE_ITEM3 as c_int {
                //medpack
                ItemUse_MedPack(ent);
            } else if event == EV_USE_ITEM4 as c_int {
                //big medpack
                ItemUse_MedPack_Big(ent);
            } else if event == EV_USE_ITEM5 as c_int {
                //binoculars
                ItemUse_Binoculars(ctx, ent);
            } else if event == EV_USE_ITEM6 as c_int {
                //sentry gun
                ItemUse_Sentry(ctx, ent);
            } else if event == EV_USE_ITEM7 as c_int {
                //jetpack
                ItemUse_Jetpack(ctx, ent);
            } else if event == EV_USE_ITEM8 as c_int {
                //health disp — ItemUse_UseDisp(ent, HI_HEALTHDISP);
            } else if event == EV_USE_ITEM9 as c_int {
                //ammo disp — ItemUse_UseDisp(ent, HI_AMMODISP);
            } else if event == EV_USE_ITEM10 as c_int {
                //eweb
                ItemUse_UseEWeb(ctx, ent);
            } else if event == EV_USE_ITEM11 as c_int {
                //cloak
                ItemUse_UseCloak(ctx, ent);
            }

            i += 1;
        }
    }
}

/// Raven `SendPendingPredictableEvents` — spawn a temp-ent for a pending event.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1059-1087`
pub fn SendPendingPredictableEvents(ctx: GameContext<'_>, ps: *mut playerState_t) {
    unsafe {
        // if there are still events pending
        if (*ps).entityEventSequence < (*ps).eventSequence {
            // create a temporary entity for this event which is sent to everyone
            // except the client who generated the event
            let seq = (*ps).entityEventSequence & (MAX_PS_EVENTS as c_int - 1);
            let event = (*ps).events[seq as usize] | (((*ps).entityEventSequence & 3) << 8);
            // set external event to zero before calling BG_PlayerStateToEntityState
            let extEvent = (*ps).externalEvent;
            (*ps).externalEvent = 0;
            // create temporary entity for event
            let t = G_TempEntity(ctx, (*ps).origin, event);
            let number = (*t).s.number;
            BG_PlayerStateToEntityState(ps, &mut (*t).s, qtrue);
            (*t).s.number = number;
            (*t).s.eType = ET_EVENTS as c_int + event;
            (*t).s.eFlags |= EF_PLAYER_EVENT;
            (*t).s.otherEntityNum = (*ps).clientNum;
            // send to everyone except the client who generated the event
            (*t).r.svFlags |= SVF_NOTSINGLECLIENT;
            (*t).r.singleClient = (*ps).clientNum;
            // set back external event
            (*ps).externalEvent = extEvent;
        }
    }
}

/// Raven `G_UpdateForceSightBroadcasts` — broadcast this client to force-sight viewers.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1103-1147`
pub fn G_UpdateForceSightBroadcasts(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        // Any clients with force sight on should see this client
        let numConnectedClients = (*ctx.world).level.numConnectedClients;
        let selfCl = (*self_).client as *mut gclient_t;
        for i in 0..numConnectedClients {
            let ent = &mut (*ctx.world).entities
                [(*ctx.world).level.sortedClients[i as usize] as usize]
                as *mut gentity_t;
            let dist: f32;
            let mut angles: vec3_t = [0.0; 3];

            if ent == self_ {
                continue;
            }

            let entCl = (*ent).client as *mut gclient_t;
            // Not using force sight so we shouldnt broadcast to this one
            if (*entCl).ps.fd.forcePowersActive & (1 << FP_SEE) == 0 {
                continue;
            }

            for k in 0..3 {
                angles[k] = (*selfCl).ps.origin[k] - (*entCl).ps.origin[k];
            }
            dist = VectorLengthSquared(angles);
            let anglesIn = angles;
            vectoangles(anglesIn, &mut angles);

            // Too far away then just forget it
            if dist > MAX_SIGHT_DISTANCE * MAX_SIGHT_DISTANCE {
                continue;
            }

            // If not within the field of view then forget it
            if InFieldOfVision((*entCl).ps.viewangles, MAX_SIGHT_FOV, angles) == 0 {
                break;
            }

            // Turn on the broadcast bit for the master and since there is only one
            // master we are done
            (*self_).r.broadcastClients[((*ent).s.clientNum / 32) as usize] |=
                1 << ((*ent).s.clientNum % 32);

            break;
        }
    }
}

/// Raven `G_UpdateJediMasterBroadcasts` — broadcast the Jedi Master to nearby clients.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1149-1197`
pub fn G_UpdateJediMasterBroadcasts(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        let selfCl = (*self_).client as *mut gclient_t;

        // Not jedi master mode then nothing to do
        if (*ctx.world).cvars.g_gametype.integer != GT_JEDIMASTER {
            return;
        }

        // This client isnt the jedi master so it shouldnt broadcast
        if (*selfCl).ps.isJediMaster == qfalse {
            return;
        }

        // Broadcast ourself to all clients within range
        let numConnectedClients = (*ctx.world).level.numConnectedClients;
        for i in 0..numConnectedClients {
            let ent = &mut (*ctx.world).entities
                [(*ctx.world).level.sortedClients[i as usize] as usize]
                as *mut gentity_t;
            let dist: f32;
            let mut angles: vec3_t = [0.0; 3];

            if ent == self_ {
                continue;
            }

            let entCl = (*ent).client as *mut gclient_t;
            for k in 0..3 {
                angles[k] = (*selfCl).ps.origin[k] - (*entCl).ps.origin[k];
            }
            dist = VectorLengthSquared(angles);
            let anglesIn = angles;
            vectoangles(anglesIn, &mut angles);

            // Too far away then just forget it
            if dist > MAX_JEDIMASTER_DISTANCE * MAX_JEDIMASTER_DISTANCE {
                continue;
            }

            // If not within the field of view then forget it
            if InFieldOfVision((*entCl).ps.viewangles, MAX_JEDIMASTER_FOV, angles) == 0 {
                continue;
            }

            // Turn on the broadcast bit for the master and since there is only one
            // master we are done
            (*self_).r.broadcastClients[((*ent).s.clientNum / 32) as usize] |=
                1 << ((*ent).s.clientNum % 32);
        }
    }
}

/// Raven `G_UpdateClientBroadcasts`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1199-1209`
pub fn G_UpdateClientBroadcasts(ctx: GameContext<'_>, self_: *mut gentity_t) {
    unsafe {
        // Clear all the broadcast bits for this client
        (*self_).r.broadcastClients = [0; 2];

        // The jedi master is broadcast to everyone in range
        G_UpdateJediMasterBroadcasts(ctx, self_);

        // Anyone with force sight on should see this client
        G_UpdateForceSightBroadcasts(ctx, self_);
    }
}

/// Raven `G_AddPushVecToUcmd` — fold a client's push vector into its ucmd.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1211-1244`
pub fn G_AddPushVecToUcmd(ctx: GameContext<'_>, self_: *mut gentity_t, ucmd: *mut usercmd_t) {
    unsafe {
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut moveDir: vec3_t = [0.0; 3];

        let cl = (*self_).client as *mut gclient_t;
        if cl.is_null() {
            return;
        }
        let pushSpeed = VectorLengthSquared((*cl).pushVec);
        if pushSpeed == 0.0 {
            //not being pushed
            return;
        }

        AngleVectors((*cl).ps.viewangles, Some(&mut forward), Some(&mut right), None);
        for i in 0..3 {
            moveDir[i] = forward[i] * ((*ucmd).forwardmove as f32 / 127.0 * (*cl).ps.speed);
        }
        for i in 0..3 {
            moveDir[i] += (*ucmd).rightmove as f32 / 127.0 * (*cl).ps.speed * right[i];
        }
        //moveDir is now our intended move velocity

        for i in 0..3 {
            moveDir[i] += (*cl).pushVec[i];
        }
        (*cl).ps.speed = VectorNormalize(&mut moveDir);
        //moveDir is now our intended move velocity plus our push Vector

        let fMove =
            127.0 * (forward[0] * moveDir[0] + forward[1] * moveDir[1] + forward[2] * moveDir[2]);
        let rMove =
            127.0 * (right[0] * moveDir[0] + right[1] * moveDir[1] + right[2] * moveDir[2]);
        (*ucmd).forwardmove = fMove.floor() as i8; //If in the same dir , will be positive
        (*ucmd).rightmove = rMove.floor() as i8; //If in the same dir , will be positive

        if (*cl).pushVecTime < (*ctx.world).level.time {
            (*cl).pushVec = [0.0; 3];
        }
    }
}

/// Raven `G_StandingAnim` — is this a plain standing anim? (not idles/cinematics).
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1246-1258`
pub fn G_StandingAnim(anim: c_int) -> qboolean {
    if anim == BOTH_STAND1 as c_int
        || anim == BOTH_STAND2 as c_int
        || anim == BOTH_STAND3 as c_int
        || anim == BOTH_STAND4 as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `G_ActionButtonPressed`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1260-1300`
pub fn G_ActionButtonPressed(buttons: c_int) -> qboolean {
    if buttons & BUTTON_ATTACK != 0 {
        qtrue
    } else if buttons & BUTTON_USE_HOLDABLE != 0 {
        qtrue
    } else if buttons & BUTTON_GESTURE != 0 {
        qtrue
    } else if buttons & BUTTON_USE != 0 {
        qtrue
    } else if buttons & BUTTON_FORCEGRIP != 0 {
        qtrue
    } else if buttons & BUTTON_ALT_ATTACK != 0 {
        qtrue
    } else if buttons & BUTTON_FORCEPOWER != 0 {
        qtrue
    } else if buttons & BUTTON_FORCE_LIGHTNING != 0 {
        qtrue
    } else if buttons & BUTTON_FORCE_DRAIN != 0 {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `G_CheckClientIdle` — enter/exit idle animations after inactivity.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1302-1430`
pub fn G_CheckClientIdle(ctx: GameContext<'_>, ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    unsafe {
        let mut viewChange: vec3_t = [0.0; 3];
        let actionPressed: qboolean;
        let mut buttons: c_int;

        let cl = if ent.is_null() {
            core::ptr::null_mut()
        } else {
            (*ent).client as *mut gclient_t
        };
        if ent.is_null()
            || cl.is_null()
            || (*ent).health <= 0
            || (*cl).ps.stats[STAT_HEALTH as usize] <= 0
            || (*cl).sess.sessionTeam == TEAM_SPECTATOR
            || ((*cl).ps.pm_flags & PMF_FOLLOW) != 0
        {
            return;
        }

        buttons = (*ucmd).buttons;

        if (*ent).r.svFlags & SVF_BOT != 0 {
            //they press use all the time..
            buttons &= !BUTTON_USE;
        }
        actionPressed = G_ActionButtonPressed(buttons);

        for i in 0..3 {
            viewChange[i] = (*cl).ps.viewangles[i] - (*cl).idleViewAngles[i];
        }
        let level_time = (*ctx.world).level.time;
        if VectorCompare(vec3_origin, (*cl).ps.velocity) == qfalse
            || actionPressed != 0
            || (*ucmd).forwardmove != 0
            || (*ucmd).rightmove != 0
            || (*ucmd).upmove != 0
            || G_StandingAnim((*cl).ps.legsAnim) == qfalse
            || ((*ent).health + (*cl).ps.stats[STAT_ARMOR as usize]) != (*cl).idleHealth
            || VectorLength(viewChange) > 10.0
            || (*cl).ps.legsTimer > 0
            || (*cl).ps.torsoTimer > 0
            || (*cl).ps.weaponTime > 0
            || (*cl).ps.weaponstate == WEAPON_CHARGING as c_int
            || (*cl).ps.weaponstate == WEAPON_CHARGING_ALT as c_int
            || (*cl).ps.zoomMode != 0
            || ((*cl).ps.weaponstate != WEAPON_READY as c_int && (*cl).ps.weapon != WP_SABER)
            || (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int
            || (*cl).ps.saberBlocked != BLOCKED_NONE as c_int
            || (*cl).ps.saberBlocking >= level_time
            || (*cl).ps.weapon == WP_MELEE
            || ((*cl).ps.weapon != (*cl).pers.cmd.weapon && (*ent).s.eType != ET_NPC as c_int)
        {
            //FIXME: also check for turning?
            let mut brokeOut: qboolean = qfalse;

            if VectorCompare(vec3_origin, (*cl).ps.velocity) == qfalse
                || actionPressed != 0
                || (*ucmd).forwardmove != 0
                || (*ucmd).rightmove != 0
                || (*ucmd).upmove != 0
                || ((*ent).health + (*cl).ps.stats[STAT_ARMOR as usize]) != (*cl).idleHealth
                || (*cl).ps.zoomMode != 0
                || ((*cl).ps.weaponstate != WEAPON_READY as c_int && (*cl).ps.weapon != WP_SABER)
                || ((*cl).ps.weaponTime > 0 && (*cl).ps.weapon == WP_SABER)
                || (*cl).ps.weaponstate == WEAPON_CHARGING as c_int
                || (*cl).ps.weaponstate == WEAPON_CHARGING_ALT as c_int
                || (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int
                || (*cl).ps.saberBlocked != BLOCKED_NONE as c_int
                || (*cl).ps.saberBlocking >= level_time
                || (*cl).ps.weapon == WP_MELEE
                || ((*cl).ps.weapon != (*cl).pers.cmd.weapon && (*ent).s.eType != ET_NPC as c_int)
            {
                //if in an idle, break out
                let la = (*cl).ps.legsAnim;
                if la == BOTH_STAND1IDLE1 as c_int
                    || la == BOTH_STAND2IDLE1 as c_int
                    || la == BOTH_STAND2IDLE2 as c_int
                    || la == BOTH_STAND3IDLE1 as c_int
                    || la == BOTH_STAND5IDLE1 as c_int
                {
                    (*cl).ps.legsTimer = 0;
                    brokeOut = qtrue;
                }
                let ta = (*cl).ps.torsoAnim;
                if ta == BOTH_STAND1IDLE1 as c_int
                    || ta == BOTH_STAND2IDLE1 as c_int
                    || ta == BOTH_STAND2IDLE2 as c_int
                    || ta == BOTH_STAND3IDLE1 as c_int
                    || ta == BOTH_STAND5IDLE1 as c_int
                {
                    (*cl).ps.torsoTimer = 0;
                    (*cl).ps.weaponTime = 0;
                    (*cl).ps.saberMove = LS_READY;
                    brokeOut = qtrue;
                }
            }
            //
            (*cl).idleHealth = (*ent).health + (*cl).ps.stats[STAT_ARMOR as usize];
            (*cl).idleViewAngles = (*cl).ps.viewangles;
            if (*cl).idleTime < level_time {
                (*cl).idleTime = level_time;
            }

            if brokeOut != 0
                && ((*cl).ps.weaponstate == WEAPON_CHARGING as c_int
                    || (*cl).ps.weaponstate == WEAPON_CHARGING_ALT as c_int)
            {
                (*cl).ps.torsoAnim = TORSO_RAISEWEAP1 as c_int;
            }
        } else if level_time - (*cl).idleTime > 5000 {
            //been idle for 5 seconds
            let mut idleAnim: c_int = -1;
            let la = (*cl).ps.legsAnim;
            if la == BOTH_STAND1 as c_int {
                idleAnim = BOTH_STAND1IDLE1 as c_int;
            } else if la == BOTH_STAND2 as c_int {
                idleAnim = BOTH_STAND2IDLE1 as c_int;
            } else if la == BOTH_STAND3 as c_int {
                idleAnim = BOTH_STAND3IDLE1 as c_int;
            } else if la == BOTH_STAND5 as c_int {
                idleAnim = BOTH_STAND5IDLE1 as c_int;
            }

            if idleAnim == BOTH_STAND2IDLE1 as c_int && Q_irand(1, 10) <= 5 {
                idleAnim = BOTH_STAND2IDLE2 as c_int;
            }

            if idleAnim != -1 && idleAnim > 0 && idleAnim < MAX_ANIMATIONS as c_int {
                G_SetAnim(
                    ent,
                    ucmd,
                    SETANIM_BOTH as c_int,
                    idleAnim,
                    (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD) as c_int,
                    0,
                );

                //don't idle again after this anim for a while
                (*cl).idleTime = level_time + (*cl).ps.legsTimer + Q_irand(0, 2000);
            }
        }
    }
}

/// Raven `NPC_Accelerate`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1432-1486`
pub fn NPC_Accelerate(ent: *mut gentity_t, fullWalkAcc: qboolean, fullRunAcc: qboolean) {
    unsafe {
        if (*ent).client.is_null() || (*ent).NPC.is_null() {
            return;
        }
        let npc = (*ent).NPC as *mut gNPC_t;

        if (*npc).stats.acceleration == 0 {
            //No acceleration means just start and stop
            (*npc).currentSpeed = (*npc).desiredSpeed;
        }
        //FIXME:  in cinematics always accel/decel?
        else if (*npc).desiredSpeed <= (*npc).stats.walkSpeed {
            //Only accelerate if at walkSpeeds
            if (*npc).desiredSpeed > (*npc).currentSpeed + (*npc).stats.acceleration {
                (*npc).currentSpeed += (*npc).stats.acceleration;
            } else if (*npc).desiredSpeed > (*npc).currentSpeed {
                (*npc).currentSpeed = (*npc).desiredSpeed;
            } else if fullWalkAcc != 0
                && (*npc).desiredSpeed < (*npc).currentSpeed - (*npc).stats.acceleration
            {
                //decelerate even when walking
                (*npc).currentSpeed -= (*npc).stats.acceleration;
            } else if (*npc).desiredSpeed < (*npc).currentSpeed {
                //stop on a dime
                (*npc).currentSpeed = (*npc).desiredSpeed;
            }
        } else {
            //Only decelerate if at runSpeeds
            if fullRunAcc != 0
                && (*npc).desiredSpeed > (*npc).currentSpeed + (*npc).stats.acceleration
            {
                //Accelerate to runspeed
                (*npc).currentSpeed += (*npc).stats.acceleration;
            } else if (*npc).desiredSpeed > (*npc).currentSpeed {
                //accelerate instantly
                (*npc).currentSpeed = (*npc).desiredSpeed;
            } else if fullRunAcc != 0
                && (*npc).desiredSpeed < (*npc).currentSpeed - (*npc).stats.acceleration
            {
                (*npc).currentSpeed -= (*npc).stats.acceleration;
            } else if (*npc).desiredSpeed < (*npc).currentSpeed {
                (*npc).currentSpeed = (*npc).desiredSpeed;
            }
        }
    }
}

/// Raven `NPC_GetWalkSpeed`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1494-1510`
pub fn NPC_GetWalkSpeed(ent: *mut gentity_t) -> c_int {
    unsafe {
        if (*ent).client.is_null() || (*ent).NPC.is_null() {
            return 0;
        }
        let npc = (*ent).NPC as *mut gNPC_t;
        // Raven's switch on playerTeam has only the NPCTEAM_PLAYER / default arm,
        // both yielding walkSpeed (stub code).
        (*npc).stats.walkSpeed
    }
}

/// Raven `NPC_GetRunSpeed`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1517-1573`
pub fn NPC_GetRunSpeed(ent: *mut gentity_t) -> c_int {
    unsafe {
        if (*ent).client.is_null() || (*ent).NPC.is_null() {
            return 0;
        }
        let cl = (*ent).client as *mut gclient_t;
        let npc = (*ent).NPC as *mut gNPC_t;

        // team no longer indicates species/race. Use NPC_class to adjust speed.
        let runSpeed: c_int = match (*cl).NPC_class {
            CLASS_PROBE | CLASS_GONK | CLASS_R2D2 | CLASS_R5D2 | CLASS_MARK1 | CLASS_MARK2
            | CLASS_PROTOCOL | CLASS_ATST | CLASS_MOUSE | CLASS_SEEKER | CLASS_REMOTE => {
                (*npc).stats.runSpeed
            }
            _ => ((*npc).stats.runSpeed as f32 * 1.3) as c_int, //rww - seems to slow in MP for some reason.
        };

        runSpeed
    }
}

/// Raven `G_CheckMovingLoopingSounds` — NPC movement loop-sounds.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1577-1614`
pub fn G_CheckMovingLoopingSounds(ctx: GameContext<'_>, ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    unsafe {
        let cl = (*ent).client as *mut gclient_t;
        if !cl.is_null() {
            if (!(*ent).NPC.is_null() && VectorCompare(vec3_origin, (*cl).ps.moveDir) == qfalse) //moving using moveDir
                || (*ucmd).forwardmove != 0
                || (*ucmd).rightmove != 0 //moving using ucmds
                || ((*ucmd).upmove != 0 && FlyingCreature(ent) != 0) //flier using ucmds to move
                || (FlyingCreature(ent) != 0
                    && VectorCompare(vec3_origin, (*cl).ps.velocity) == qfalse
                    && (*ent).health > 0)
            {
                //flier using velocity to move
                match (*cl).NPC_class {
                    CLASS_R2D2 => {
                        (*ent).s.loopSound =
                            G_SoundIndex(cstr("sound/chars/r2d2/misc/r2_move_lp.wav").as_ptr());
                    }
                    CLASS_R5D2 => {
                        (*ent).s.loopSound =
                            G_SoundIndex(cstr("sound/chars/r2d2/misc/r2_move_lp2.wav").as_ptr());
                    }
                    CLASS_MARK2 => {
                        (*ent).s.loopSound =
                            G_SoundIndex(cstr("sound/chars/mark2/misc/mark2_move_lp").as_ptr());
                    }
                    CLASS_MOUSE => {
                        (*ent).s.loopSound =
                            G_SoundIndex(cstr("sound/chars/mouse/misc/mouse_lp").as_ptr());
                    }
                    CLASS_PROBE => {
                        (*ent).s.loopSound =
                            G_SoundIndex(cstr("sound/chars/probe/misc/probedroidloop").as_ptr());
                    }
                    _ => {}
                }
            } else {
                //not moving under your own control, stop loopSound
                if (*cl).NPC_class == CLASS_R2D2
                    || (*cl).NPC_class == CLASS_R5D2
                    || (*cl).NPC_class == CLASS_MARK2
                    || (*cl).NPC_class == CLASS_MOUSE
                    || (*cl).NPC_class == CLASS_PROBE
                {
                    (*ent).s.loopSound = 0;
                }
            }
        }
    }
}

/// Raven `G_HeldByMonster` — clamp a player being held by a monster (Rancor).
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1616-1651`
pub fn G_HeldByMonster(ctx: GameContext<'_>, ent: *mut gentity_t, ucmd: *mut *mut usercmd_t) {
    unsafe {
        let cl = if ent.is_null() {
            core::ptr::null_mut()
        } else {
            (*ent).client as *mut gclient_t
        };
        //NOTE: lookTarget is an entity number, so this presumes that client 0 is NOT a Rancor...
        if !ent.is_null() && !cl.is_null() && (*cl).ps.hasLookTarget != 0 {
            let monster =
                &mut (*ctx.world).entities[(*cl).ps.lookTarget as usize] as *mut gentity_t;
            let mcl = (*monster).client as *mut gclient_t;
            if !monster.is_null() && !mcl.is_null() {
                //take the monster's waypoint as your own
                (*ent).waypoint = (*monster).waypoint;
                if (*monster).s.NPC_class == CLASS_RANCOR as c_int {
                    //only possibility right now, may add Wampa and Sand Creature later
                    BG_AttachToRancor(
                        (*monster).ghoul2, //ghoul2 info
                        (*monster).r.currentAngles[YAW],
                        (*monster).r.currentOrigin,
                        (*ctx.world).level.time,
                        core::ptr::null_mut(),
                        (*monster).modelScale,
                        (*mcl).ps.eFlags2 & EF2_GENERIC_NPC_FLAG,
                        (*cl).ps.origin,
                        (*cl).ps.viewangles,
                        core::ptr::null_mut(),
                    );
                }
                (*cl).ps.velocity = [0.0; 3];
                G_SetOrigin(ent, (*cl).ps.origin);
                SetClientViewAngle(ent, (*cl).ps.viewangles);
                G_SetAngles(ent, (*cl).ps.viewangles);
                trap::LinkEntity(
                    ctx.engine,
                    mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent),
                ); //redundant?
            }
        }
        // don't allow movement, weapon switching, and most kinds of button presses
        (**ucmd).forwardmove = 0;
        (**ucmd).rightmove = 0;
        (**ucmd).upmove = 0;
    }
}

/// Raven `G_SetTauntAnim` — play a taunt/bow/meditate/flourish/gloat animation.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1662-1926`
pub fn G_SetTauntAnim(ctx: GameContext<'_>, ent: *mut gentity_t, taunt: c_int) {
    unsafe {
        let cl = (*ent).client as *mut gclient_t;
        let level_time = (*ctx.world).level.time;

        if (*cl).pers.cmd.upmove != 0 || (*cl).pers.cmd.forwardmove != 0 || (*cl).pers.cmd.rightmove != 0
        {
            //hack, don't do while moving
            return;
        }
        if taunt != TAUNT_TAUNT {
            //normal taunt always allowed
            if (*ctx.world).cvars.g_gametype.integer != GT_DUEL
                && (*ctx.world).cvars.g_gametype.integer != GT_POWERDUEL
            {
                //no taunts unless in Duel
                return;
            }
        }
        if (*cl).ps.torsoTimer < 1
            && (*cl).ps.forceHandExtend == HANDEXTEND_NONE as c_int
            && (*cl).ps.legsTimer < 1
            && (*cl).ps.weaponTime < 1
            && (*cl).ps.saberLockTime < level_time
        {
            let mut anim: c_int = -1;
            let saberAnimLevel = (*cl).ps.fd.saberAnimLevel;
            if taunt == TAUNT_TAUNT {
                if (*cl).ps.weapon != WP_SABER {
                    anim = BOTH_ENGAGETAUNT as c_int;
                } else if (*cl).saber[0].tauntAnim != -1 {
                    anim = (*cl).saber[0].tauntAnim;
                } else if (*cl).saber[1].model[0] != 0 && (*cl).saber[1].tauntAnim != -1 {
                    anim = (*cl).saber[1].tauntAnim;
                } else {
                    if saberAnimLevel == SS_FAST as c_int || saberAnimLevel == SS_TAVION as c_int {
                        if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                            //turn off second saber
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOff);
                        } else if (*cl).ps.saberHolstered == 0 {
                            //turn off first
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOff);
                        }
                        (*cl).ps.saberHolstered = 2;
                        anim = BOTH_GESTURE1 as c_int;
                    } else if saberAnimLevel == SS_MEDIUM as c_int
                        || saberAnimLevel == SS_STRONG as c_int
                        || saberAnimLevel == SS_DESANN as c_int
                    {
                        anim = BOTH_ENGAGETAUNT as c_int;
                    } else if saberAnimLevel == SS_DUAL as c_int {
                        if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                            //turn on second saber
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOn);
                        } else if (*cl).ps.saberHolstered == 2 {
                            //turn on first
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                        }
                        (*cl).ps.saberHolstered = 0;
                        anim = BOTH_DUAL_TAUNT as c_int;
                    } else if saberAnimLevel == SS_STAFF as c_int {
                        if (*cl).ps.saberHolstered > 0 {
                            //turn on all blades
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                        }
                        (*cl).ps.saberHolstered = 0;
                        anim = BOTH_STAFF_TAUNT as c_int;
                    }
                }
            } else if taunt == TAUNT_BOW {
                if (*cl).saber[0].bowAnim != -1 {
                    anim = (*cl).saber[0].bowAnim;
                } else if (*cl).saber[1].model[0] != 0 && (*cl).saber[1].bowAnim != -1 {
                    anim = (*cl).saber[1].bowAnim;
                } else {
                    anim = BOTH_BOW as c_int;
                }
                if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                    //turn off second saber
                    G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOff);
                } else if (*cl).ps.saberHolstered == 0 {
                    //turn off first
                    G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOff);
                }
                (*cl).ps.saberHolstered = 2;
            } else if taunt == TAUNT_MEDITATE {
                if (*cl).saber[0].meditateAnim != -1 {
                    anim = (*cl).saber[0].meditateAnim;
                } else if (*cl).saber[1].model[0] != 0 && (*cl).saber[1].meditateAnim != -1 {
                    anim = (*cl).saber[1].meditateAnim;
                } else {
                    anim = BOTH_MEDITATE as c_int;
                }
                if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                    //turn off second saber
                    G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOff);
                } else if (*cl).ps.saberHolstered == 0 {
                    //turn off first
                    G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOff);
                }
                (*cl).ps.saberHolstered = 2;
            } else if taunt == TAUNT_FLOURISH {
                if (*cl).ps.weapon == WP_SABER {
                    if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                        //turn on second saber
                        G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOn);
                    } else if (*cl).ps.saberHolstered == 2 {
                        //turn on first
                        G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                    }
                    (*cl).ps.saberHolstered = 0;
                    if (*cl).saber[0].flourishAnim != -1 {
                        anim = (*cl).saber[0].flourishAnim;
                    } else if (*cl).saber[1].model[0] != 0 && (*cl).saber[1].flourishAnim != -1 {
                        anim = (*cl).saber[1].flourishAnim;
                    } else {
                        if saberAnimLevel == SS_FAST as c_int || saberAnimLevel == SS_TAVION as c_int {
                            anim = BOTH_SHOWOFF_FAST as c_int;
                        } else if saberAnimLevel == SS_MEDIUM as c_int {
                            anim = BOTH_SHOWOFF_MEDIUM as c_int;
                        } else if saberAnimLevel == SS_STRONG as c_int
                            || saberAnimLevel == SS_DESANN as c_int
                        {
                            anim = BOTH_SHOWOFF_STRONG as c_int;
                        } else if saberAnimLevel == SS_DUAL as c_int {
                            anim = BOTH_SHOWOFF_DUAL as c_int;
                        } else if saberAnimLevel == SS_STAFF as c_int {
                            anim = BOTH_SHOWOFF_STAFF as c_int;
                        }
                    }
                }
            } else if taunt == TAUNT_GLOAT {
                if (*cl).saber[0].gloatAnim != -1 {
                    anim = (*cl).saber[0].gloatAnim;
                } else if (*cl).saber[1].model[0] != 0 && (*cl).saber[1].gloatAnim != -1 {
                    anim = (*cl).saber[1].gloatAnim;
                } else {
                    if saberAnimLevel == SS_FAST as c_int || saberAnimLevel == SS_TAVION as c_int {
                        anim = BOTH_VICTORY_FAST as c_int;
                    } else if saberAnimLevel == SS_MEDIUM as c_int {
                        anim = BOTH_VICTORY_MEDIUM as c_int;
                    } else if saberAnimLevel == SS_STRONG as c_int
                        || saberAnimLevel == SS_DESANN as c_int
                    {
                        if (*cl).ps.saberHolstered != 0 {
                            //turn on first
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                        }
                        (*cl).ps.saberHolstered = 0;
                        anim = BOTH_VICTORY_STRONG as c_int;
                    } else if saberAnimLevel == SS_DUAL as c_int {
                        if (*cl).ps.saberHolstered == 1 && (*cl).saber[1].model[0] != 0 {
                            //turn on second saber
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[1].soundOn);
                        } else if (*cl).ps.saberHolstered == 2 {
                            //turn on first
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                        }
                        (*cl).ps.saberHolstered = 0;
                        anim = BOTH_VICTORY_DUAL as c_int;
                    } else if saberAnimLevel == SS_STAFF as c_int {
                        if (*cl).ps.saberHolstered != 0 {
                            //turn on first
                            G_Sound(ctx, ent, CHAN_WEAPON as c_int, (*cl).saber[0].soundOn);
                        }
                        (*cl).ps.saberHolstered = 0;
                        anim = BOTH_VICTORY_STAFF as c_int;
                    }
                }
            }
            if anim != -1 {
                if (*cl).ps.groundEntityNum != ENTITYNUM_NONE {
                    (*cl).ps.forceHandExtend = HANDEXTEND_TAUNT as c_int;
                    (*cl).ps.forceDodgeAnim = anim;
                    (*cl).ps.forceHandExtendTime =
                        level_time + BG_AnimLength((*ent).localAnimIndex, anim);
                }
                if taunt != TAUNT_MEDITATE && taunt != TAUNT_BOW {
                    //no sound for meditate or bow
                    G_AddEvent(ent, EV_TAUNT as c_int, taunt);
                }
            }
        }
    }
}

/// Raven `ClientThink_real` — the per-frame client/NPC think core.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1939-3611`
// PORT-ESCALATION(pmove-trace-seam): the 1673-LOC core sets `pm.trace = trap_Trace` / `pm.pointcontents = trap_PointContents` and calls the (parked) `Pmove`; the engine-bearing `trap::Trace(engine, …)` cannot be stored in `pmove_t`'s raw `Option<extern "C" fn>` field, and it also relies on the pilot pointer-identity `m_pPilot == (bgEntity_t*)ent` compare (fork 4) with `m_pVehicle` still `*mut c_void`. No established pmove trace-threading convention exists in the packet.
pub fn ClientThink_real(ctx: GameContext<'_>, ent: *mut gentity_t) {
    todo!("Port ClientThink_real — parked: pmove-trace-seam")
}

/// Raven `G_CheckClientTimeouts` — force idle clients to spectator.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:3620-3640`
pub fn G_CheckClientTimeouts(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let cl = (*ent).client as *mut gclient_t;
        // Only timeout supported right now is the timeout to spectator mode
        if (*ctx.world).cvars.g_timeouttospec.integer == 0 {
            return;
        }

        // Already a spectator, no need to boot them to spectator
        if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
            return;
        }

        // See how long its been since a command was received by the client and if
        // its longer than the timeout to spectator then force this client into
        // spectator mode
        if (*ctx.world).level.time - (*cl).pers.cmd.serverTime
            > (*ctx.world).cvars.g_timeouttospec.integer * 1000
        {
            let s = cstr("spectator");
            SetTeam(ctx, ent, s.as_ptr() as *mut c_char);
        }
    }
}

/// Raven `ClientThink` — vmMain client-think entry.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:3649-3720`
pub fn ClientThink(ctx: GameContext<'_>, clientNum: c_int, ucmd: *mut usercmd_t) {
    unsafe {
        let ent = &mut (*ctx.world).entities[clientNum as usize] as *mut gentity_t;
        let cl = (*ent).client as *mut gclient_t;
        if clientNum < MAX_CLIENTS {
            trap::GetUsercmd(
                ctx.engine,
                mp_abi::game::syscalls::G_GET_USERCMD::GGetUsercmdArgs::new(
                    clientNum,
                    &mut (*cl).pers.cmd as *mut usercmd_t,
                ),
            );
        }

        // mark the time we got info, so we can display the phone jack if they
        // don't get any for a while
        (*cl).lastCmdTime = (*ctx.world).level.time;

        if !ucmd.is_null() {
            (*cl).pers.cmd = *ucmd;
        }

        if (*ent).r.svFlags & SVF_BOT == 0 && (*ctx.world).cvars.g_synchronousClients.integer == 0 {
            ClientThink_real(ctx, ent);
        }
        // vehicles are clients and when running synchronous they still need to
        // think here so special case them.
        else if clientNum >= MAX_CLIENTS {
            ClientThink_real(ctx, ent);
        }
    }
}

/// Raven `G_RunClient`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:3723-3729`
pub fn G_RunClient(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        if (*ent).r.svFlags & SVF_BOT == 0 && (*ctx.world).cvars.g_synchronousClients.integer == 0 {
            return;
        }
        let cl = (*ent).client as *mut gclient_t;
        (*cl).pers.cmd.serverTime = (*ctx.world).level.time;
        ClientThink_real(ctx, ent);
    }
}

/// Raven `SpectatorClientEndFrame` — follow-cam / scoreboard bookkeeping.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:3738-3783`
pub fn SpectatorClientEndFrame(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let entCl = (*ent).client as *mut gclient_t;

        if (*ent).s.eType == ET_NPC as c_int {
            debug_assert!(false, "SpectatorClientEndFrame called on ET_NPC");
            return;
        }

        // if we are doing a chase cam or a remote view, grab the latest info
        if (*entCl).sess.spectatorState == SPECTATOR_FOLLOW {
            let mut clientNum = (*entCl).sess.spectatorClient;

            // team follow1 and team follow2 go to whatever clients are playing
            if clientNum == -1 {
                clientNum = (*ctx.world).level.follow1;
            } else if clientNum == -2 {
                clientNum = (*ctx.world).level.follow2;
            }
            if clientNum >= 0 {
                let cl = (*ctx.world).level.clients.add(clientNum as usize);
                if (*cl).pers.connected == CON_CONNECTED
                    && (*cl).sess.sessionTeam != TEAM_SPECTATOR
                {
                    (*entCl).ps.eFlags = (*cl).ps.eFlags;
                    (*entCl).ps = (*cl).ps;
                    (*entCl).ps.pm_flags |= PMF_FOLLOW;
                    return;
                } else {
                    // drop them to free spectators unless they are dedicated camera followers
                    if (*entCl).sess.spectatorClient >= 0 {
                        (*entCl).sess.spectatorState = SPECTATOR_FREE;
                        let idx = (entCl as *mut gclient_t)
                            .offset_from((*ctx.world).level.clients)
                            as c_int;
                        ClientBegin(ctx, idx, qtrue);
                    }
                }
            }
        }

        if (*entCl).sess.spectatorState == SPECTATOR_SCOREBOARD {
            (*entCl).ps.pm_flags |= PMF_SCOREBOARD;
        } else {
            (*entCl).ps.pm_flags &= !PMF_SCOREBOARD;
        }
    }
}

/// Raven `ClientEndFrame` — end-of-frame per-client fixups.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:3794-3874`
pub fn ClientEndFrame(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        let mut isNPC: qboolean = qfalse;

        if (*ent).s.eType == ET_NPC as c_int {
            isNPC = qtrue;
        }

        let entCl = (*ent).client as *mut gclient_t;

        if (*entCl).sess.sessionTeam == TEAM_SPECTATOR {
            SpectatorClientEndFrame(ctx, ent);
            return;
        }

        // turn off any expired powerups
        for i in 0..MAX_POWERUPS {
            if (*entCl).ps.powerups[i] < (*ctx.world).level.time {
                (*entCl).ps.powerups[i] = 0;
            }
        }

        // If the end of unit layout is displayed, don't give the player any
        // normal movement attributes
        if (*ctx.world).level.intermissiontime != 0 {
            if (*ent).s.number < MAX_CLIENTS || (*entCl).NPC_class == CLASS_VEHICLE {
                //players and vehicles do nothing in intermissions
                return;
            }
        }

        // burn from lava, etc
        P_WorldEffects(ctx, ent);

        // apply all the damage taken this frame
        P_DamageFeedback(ctx, ent);

        // add the EF_CONNECTION flag if we haven't gotten commands recently
        if (*ctx.world).level.time - (*entCl).lastCmdTime > 1000 {
            (*ent).s.eFlags |= EF_CONNECTION;
        } else {
            (*ent).s.eFlags &= !EF_CONNECTION;
        }

        (*entCl).ps.stats[STAT_HEALTH as usize] = (*ent).health; // FIXME: get rid of ent->health...

        G_SetClientSound(ctx, ent);

        // set the latest infor
        if (*ctx.world).cvars.g_smoothClients.integer != 0 {
            BG_PlayerStateToEntityStateExtraPolate(
                &mut (*entCl).ps,
                &mut (*ent).s,
                (*entCl).ps.commandTime,
                qfalse,
            );
        } else {
            BG_PlayerStateToEntityState(&mut (*entCl).ps, &mut (*ent).s, qfalse);
        }

        if isNPC != 0 {
            (*ent).s.eType = ET_NPC as c_int;
        }

        SendPendingPredictableEvents(ctx, &mut (*entCl).ps);
    }
}
