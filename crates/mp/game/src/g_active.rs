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

// Raven `bg_public.h`/`surfaceflags.h` `MASK_PLAYERSOLID` derived contents
// mask (base `CONTENTS_*` flags live in `mp_qshared::shared::surface_flags`
// and are prelude-globbed; the combo mask itself is transcribed locally,
// matching the `g_items.rs:106` precedent).
// Source: `oracle/oracle/codemp/game/bg_public.h:29`
const MASK_PLAYERSOLID: c_int = CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN;

// Raven `#define FALL_FADE_TIME 3000` (q_shared.h).
// Source: `oracle/oracle/codemp/game/q_shared.h:2148`
pub const FALL_FADE_TIME: c_int = 3000;


// MAT_*/SVF_*/PMF_SCOREBOARD now resolve via the crate prelude (pass-3 symbol
// backfill: `mp_qshared::common::mp::gentity`, `crate::g_public_consts`,
// `mp_qshared::common::mp::qcommon::pm_flags`).

// Raven `#define MAX_SIGHT_DISTANCE`/`MAX_SIGHT_FOV`/`MAX_JEDIMASTER_DISTANCE`/
// `MAX_JEDIMASTER_FOV` — file-scope in `g_active.c` (not referenced elsewhere).
// Source: `oracle/oracle/codemp/game/g_active.c:1097-1101`
pub const MAX_SIGHT_DISTANCE: c_float = 1500.0;
pub const MAX_SIGHT_FOV: c_float = 100.0;
pub const MAX_JEDIMASTER_DISTANCE: c_float = 2500.0;
pub const MAX_JEDIMASTER_FOV: c_float = 100.0;

// Raven's taunt selector is a file-scope anonymous `enum { TAUNT_TAUNT = 0,
// TAUNT_BOW, TAUNT_MEDITATE, TAUNT_FLOURISH, TAUNT_GLOAT };` in `g_active.c`
// (no typedef name), so per enum-vs-alias fidelity these are plain `c_int`
// consts, private to this file like the Raven original.
// Source: `oracle/oracle/codemp/game/g_active.c:1652-1659`
pub const TAUNT_TAUNT: c_int = 0;
pub const TAUNT_BOW: c_int = 1;
pub const TAUNT_MEDITATE: c_int = 2;
pub const TAUNT_FLOURISH: c_int = 3;
pub const TAUNT_GLOAT: c_int = 4;

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
use crate::ent_fn_enums::EntTouch;
use mp_bg::public::dm_flags::DF_NO_FOOTSTEPS;
use crate::bg_channel::{GameBgTraps, GameCallbacksImpl};

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
                        None,
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
                            None,
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
                            None,
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
                my_mass = (*self_).mass; // /10;
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
                            Some(&mut velocity),
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
                            None,
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
/// Raven `ClientImpacts` — bot/other touch dispatch over `pm->touchents`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:478-506`
pub fn ClientImpacts(ctx: GameContext<'_>, ent: *mut gentity_t, pm: *mut pmove_t) {
    unsafe {
        let mut trace: trace_t = core::mem::zeroed();
        let mut i = 0;
        while i < (*pm).numtouch {
            let mut j = 0;
            while j < i {
                if (*pm).touchents[j as usize] == (*pm).touchents[i as usize] {
                    break;
                }
                j += 1;
            }
            if j != i {
                i += 1;
                continue; // duplicated
            }
            let other =
                &mut (*ctx.world).entities[(*pm).touchents[i as usize] as usize] as *mut gentity_t;

            if (*ent).r.svFlags & SVF_BOT != 0 {
                if let Some(t) = (*ent).touch {
                    crate::ent_fn_enums::dispatch_touch(ctx, t, ent, other, &mut trace as *mut trace_t);
                }
            }

            let other_touch = (*other).touch;
            let Some(other_touch) = other_touch else {
                i += 1;
                continue;
            };

            crate::ent_fn_enums::dispatch_touch(ctx, other_touch, other, ent, &mut trace as *mut trace_t);
            i += 1;
        }
    }
}

/// Raven `G_TouchTriggers` — fire trigger `touch` handlers around a client.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:516-590`
/// Raven `G_TouchTriggers` — check nearby trigger volumes against `ent`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:516-590`
pub fn G_TouchTriggers(ctx: GameContext<'_>, ent: *mut gentity_t) {
    unsafe {
        if (*ent).client.is_null() {
            return;
        }
        let client = (*ent).client as *mut gclient_t;

        // dead clients don't activate triggers!
        if (*client).ps.stats[STAT_HEALTH as usize] <= 0 {
            return;
        }

        // Raven `static vec3_t range = { 40, 40, 52 };`
        let range: vec3_t = [40.0, 40.0, 52.0];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        crate::q_math::_VectorSubtract((*client).ps.origin, range, &mut mins);
        crate::q_math::_VectorAdd((*client).ps.origin, range, &mut maxs);

        let mut touch: [c_int; mp_qshared::shared::MAX_GENTITIES] =
            [0; mp_qshared::shared::MAX_GENTITIES];
        let num = trap::EntitiesInBox(
            ctx.engine,
            mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                touch.as_mut_ptr(),
                mp_qshared::shared::MAX_GENTITIES as c_int,
            ),
        );

        // can't use ent->r.absmin, because that has a one unit pad
        crate::q_math::_VectorAdd((*client).ps.origin, (*ent).r.mins, &mut mins);
        crate::q_math::_VectorAdd((*client).ps.origin, (*ent).r.maxs, &mut maxs);

        let mut trace: trace_t = core::mem::zeroed();
        let mut i = 0;
        while i < num {
            let hit = &mut (*ctx.world).entities[touch[i as usize] as usize] as *mut gentity_t;

            if (*hit).touch.is_none() && (*ent).touch.is_none() {
                i += 1;
                continue;
            }
            if (*hit).r.contents & CONTENTS_TRIGGER == 0 {
                i += 1;
                continue;
            }

            // ignore most entities if a spectator
            if (*client).sess.sessionTeam == TEAM_SPECTATOR {
                if (*hit).s.eType != ET_TELEPORT_TRIGGER as c_int
                    // this is ugly but adding a new ET_? type will
                    // most likely cause network incompatibilities
                    && (*hit).touch != Some(EntTouch::Touch_DoorTrigger)
                {
                    i += 1;
                    continue;
                }
            }

            // use seperate code for determining if an item is picked up
            // so you don't have to actually contact its bounding box
            if (*hit).s.eType == ET_ITEM as c_int {
                if BG_PlayerTouchesItem(&mut (*client).ps, &mut (*hit).s, (*ctx.world).level.time) == qfalse {
                    i += 1;
                    continue;
                }
            } else if trap::EntityContact(
                ctx.engine,
                mp_abi::game::syscalls::G_ENTITY_CONTACT::GEntityContactArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    hit as *const gentity_t,
                ),
            ) == qfalse
            {
                i += 1;
                continue;
            }

            trace = core::mem::zeroed();

            if let Some(t) = (*hit).touch {
                crate::ent_fn_enums::dispatch_touch(ctx, t, hit, ent, &mut trace as *mut trace_t);
            }

            if (*ent).r.svFlags & SVF_BOT != 0 {
                if let Some(t) = (*ent).touch {
                    crate::ent_fn_enums::dispatch_touch(ctx, t, ent, hit, &mut trace as *mut trace_t);
                }
            }

            i += 1;
        }

        // if we didn't touch a jump pad this pmove frame
        if (*client).ps.jumppad_frame != (*client).ps.pmove_framecount {
            (*client).ps.jumppad_frame = 0;
            (*client).ps.jumppad_ent = 0;
        }
    }
}

/// Raven `G_MoverTouchPushTriggers` — fire push-trigger `touch` along a mover's path.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:601-671`
/// Raven `G_MoverTouchPushTriggers` — sweep a mover's motion against push triggers.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:601-671`
pub fn G_MoverTouchPushTriggers(ctx: GameContext<'_>, ent: *mut gentity_t, oldOrg: vec3_t) {
    unsafe {
        // non-moving movers don't hit triggers!
        if VectorLengthSquared((*ent).s.pos.trDelta) == 0.0 {
            return;
        }

        let range: vec3_t = [40.0, 40.0, 52.0];
        let mut size: vec3_t = [0.0; 3];
        crate::q_math::_VectorSubtract((*ent).r.mins, (*ent).r.maxs, &mut size);
        let mut stepSize = VectorLength(size);
        if stepSize < 1.0 {
            stepSize = 1.0;
        }

        let mut dir: vec3_t = [0.0; 3];
        crate::q_math::_VectorSubtract((*ent).r.currentOrigin, oldOrg, &mut dir);
        let dist = VectorNormalize(&mut dir);

        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut touch: [c_int; mp_qshared::shared::MAX_GENTITIES] =
            [0; mp_qshared::shared::MAX_GENTITIES];
        let mut trace: trace_t = core::mem::zeroed();

        let mut step = 0.0f32;
        while step <= dist {
            let mut checkSpot: vec3_t = [0.0; 3];
            crate::q_math::_VectorMA((*ent).r.currentOrigin, step, dir, &mut checkSpot);
            crate::q_math::_VectorSubtract(checkSpot, range, &mut mins);
            crate::q_math::_VectorAdd(checkSpot, range, &mut maxs);

            let num = trap::EntitiesInBox(
                ctx.engine,
                mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    touch.as_mut_ptr(),
                    mp_qshared::shared::MAX_GENTITIES as c_int,
                ),
            );

            // can't use ent->r.absmin, because that has a one unit pad
            crate::q_math::_VectorAdd(checkSpot, (*ent).r.mins, &mut mins);
            crate::q_math::_VectorAdd(checkSpot, (*ent).r.maxs, &mut maxs);

            let mut i = 0;
            while i < num {
                let hit = &mut (*ctx.world).entities[touch[i as usize] as usize] as *mut gentity_t;

                if (*hit).s.eType != ET_PUSH_TRIGGER as c_int {
                    i += 1;
                    continue;
                }

                if (*hit).touch.is_none() {
                    i += 1;
                    continue;
                }

                if (*hit).r.contents & CONTENTS_TRIGGER == 0 {
                    i += 1;
                    continue;
                }

                if trap::EntityContact(
                    ctx.engine,
                    mp_abi::game::syscalls::G_ENTITY_CONTACT::GEntityContactArgs::new(
                        &mins as *const vec3_t,
                        &maxs as *const vec3_t,
                        hit as *const gentity_t,
                    ),
                ) == qfalse
                {
                    i += 1;
                    continue;
                }

                trace = core::mem::zeroed();

                if let Some(t) = (*hit).touch {
                    crate::ent_fn_enums::dispatch_touch(ctx, t, hit, ent, &mut trace as *mut trace_t);
                }

                i += 1;
            }

            step += stepSize;
        }
    }
}

/// Raven `SpectatorThink`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:678-740`
/// Raven `SpectatorThink`.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:678-740`
pub fn SpectatorThink(ctx: GameContext<'_>, ent: *mut gentity_t, ucmd: *mut usercmd_t) {
    unsafe {
        let client = (*ent).client as *mut gclient_t;

        if (*client).sess.spectatorState != SPECTATOR_FOLLOW {
            (*client).ps.pm_type = PM_SPECTATOR;
            (*client).ps.speed = 400.0; // faster than normal
            (*client).ps.basespeed = 400.0;

            // hmm, shouldn't have an anim if you're a spectator, make sure
            // it gets cleared.
            (*client).ps.legsAnim = 0;
            (*client).ps.legsTimer = 0;
            (*client).ps.torsoAnim = 0;
            (*client).ps.torsoTimer = 0;

            // set up for pmove
            let mut pm: pmove_t = core::mem::zeroed();
            pm.ps = &mut (*client).ps as *mut playerState_t;
            pm.cmd = *ucmd;
            // spectators can fly through bodies
            pm.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY;
            // ruling 21: pm.trace/pointcontents fields stay for layout only;
            // bg logic reaches the engine via BgTraps, threaded into Pmove below.

            pm.noSpecMove = (*ctx.world).cvars.g_noSpecMove.integer;

            pm.animations = core::ptr::null_mut();
            pm.nonHumanoid = qfalse;

            // Set up bg entity data
            pm.baseEnt = (*ctx.world).entities.as_mut_ptr() as *mut _;
            pm.entSize = core::mem::size_of::<gentity_t>() as c_int;

            // perform a pmove
            let traps = GameBgTraps::new(ctx.engine);
            let mut callbacks = GameCallbacksImpl { world: ctx.world, engine: ctx.engine };
            Pmove(&mut pm as *mut pmove_t, &mut (*ctx.world).bg_state, &traps, &mut callbacks);
            // save results of pmove
            crate::q_math::_VectorCopy((*client).ps.origin, &mut (*ent).s.origin);

            if (*client).tempSpectate < (*ctx.world).level.time {
                G_TouchTriggers(ctx, ent);
            }
            trap::UnlinkEntity(
                ctx.engine,
                mp_abi::game::syscalls::G_UNLINKENTITY::GUnlinkentityArgs::new(ent),
            );
        }

        (*client).oldbuttons = (*client).buttons;
        (*client).buttons = (*ucmd).buttons;

        if (*client).tempSpectate < (*ctx.world).level.time {
            // attack button cycles through spectators
            if (*client).buttons & BUTTON_ATTACK != 0 && (*client).oldbuttons & BUTTON_ATTACK == 0 {
                Cmd_FollowCycle_f(ctx, ent, 1);
            }

            if (*client).sess.spectatorState == SPECTATOR_FOLLOW && (*ucmd).upmove > 0 {
                // jump now removes you from follow mode
                StopFollowing(ctx, ent);
            }
        }
    }
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
                        None,
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
/// Raven `ClientThink_real` — the per-client server-side think/Pmove driver.
///
/// Source: `oracle/oracle/codemp/game/g_active.c:1939-3611`
pub fn ClientThink_real(ctx: GameContext<'_>, ent: *mut gentity_t) {
    use mp_qshared::shared::gen_cmds::genCmds_t::{self, *};
    unsafe {
        let client = (*ent).client as *mut gclient_t;
        let mut isNPC = qfalse;
        let mut controlledByPlayer = qfalse;
        let mut killJetFlags = qtrue;

        if (*ent).s.eType == ET_NPC as c_int {
            isNPC = qtrue;
        }

        // don't think if the client is not yet connected (and thus not yet spawned in)
        if (*client).pers.connected != CON_CONNECTED && isNPC == qfalse {
            return;
        }

        // This code was moved here from clientThink to fix a problem with
        // g_synchronousClients being set to 1 when in vehicles.
        if (*ent).s.number < MAX_CLIENTS as c_int && (*client).ps.m_iVehicleNum != 0 {
            // driving a vehicle
            if !(*ctx.world).entities[(*client).ps.m_iVehicleNum as usize]
                .client
                .is_null()
            {
                let veh = &mut (*ctx.world).entities[(*client).ps.m_iVehicleNum as usize]
                    as *mut gentity_t;
                let vehVehicle = (*veh).m_pVehicle as *mut Vehicle_t;

                if !vehVehicle.is_null() && (*vehVehicle).m_pPilot == ent as *mut _ {
                    // only take input from the pilot...
                    let vehClient = (*veh).client as *mut gclient_t;
                    (*vehClient).ps.commandTime = (*client).ps.commandTime;
                    (*vehVehicle).m_ucmd = (*client).pers.cmd;
                    if (*vehVehicle).m_ucmd.buttons & BUTTON_TALK != 0 {
                        // forced input if "chat bubble" is up
                        (*vehVehicle).m_ucmd.buttons = BUTTON_TALK;
                        (*vehVehicle).m_ucmd.forwardmove = 0;
                        (*vehVehicle).m_ucmd.rightmove = 0;
                        (*vehVehicle).m_ucmd.upmove = 0;
                    }
                }
            }
        }

        if (*client).ps.pm_flags & PMF_FOLLOW == 0 {
            if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE
                && (*client).siegeClass != -1
                && (*ctx.world).bg_state.bgSiegeClasses[(*client).siegeClass as usize].saberStance
                    != 0
            {
                // the class says we have to use this stance set.
                if (*ctx.world).bg_state.bgSiegeClasses[(*client).siegeClass as usize].saberStance
                    & (1 << (*client).ps.fd.saberAnimLevel)
                    == 0
                {
                    // the current stance is not in the bitmask, so find the first one that is.
                    let mut i = SS_FAST as c_int;

                    while i < SS_NUM_SABER_STYLES as c_int {
                        if (*ctx.world).bg_state.bgSiegeClasses[(*client).siegeClass as usize]
                            .saberStance
                            & (1 << i)
                            != 0
                        {
                            if i == SS_DUAL as c_int && (*client).ps.saberHolstered == 1 {
                                // one saber should be off, adjust saberAnimLevel accordinly
                                (*client).ps.fd.saberAnimLevelBase = i;
                                (*client).ps.fd.saberAnimLevel = SS_FAST as c_int;
                                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                            } else if i == SS_STAFF as c_int
                                && (*client).ps.saberHolstered == 1
                                && (*client).saber[0].singleBladeStyle != SS_NONE
                            {
                                // one saber or blade should be off, adjust saberAnimLevel accordinly
                                (*client).ps.fd.saberAnimLevelBase = i;
                                (*client).ps.fd.saberAnimLevel = (*client).saber[0].singleBladeStyle as c_int;
                                (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                            } else {
                                (*client).ps.fd.saberAnimLevelBase = i;
                                (*client).ps.fd.saberAnimLevel = i;
                                (*client).ps.fd.saberDrawAnimLevel = i;
                            }
                            break;
                        }
                        i += 1;
                    }
                }
            } else if (*client).saber[0].model[0] != 0 && (*client).saber[1].model[0] != 0 {
                // with two sabs always use akimbo style
                if (*client).ps.saberHolstered == 1 {
                    // one saber should be off, adjust saberAnimLevel accordinly
                    (*client).ps.fd.saberAnimLevelBase = SS_DUAL as c_int;
                    (*client).ps.fd.saberAnimLevel = SS_FAST as c_int;
                    (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                } else {
                    if WP_SaberStyleValidForSaber(
                        &mut (*client).saber[0],
                        &mut (*client).saber[1],
                        (*client).ps.saberHolstered,
                        (*client).ps.fd.saberAnimLevel,
                    ) == qfalse
                    {
                        // only use dual style if the style we're trying to use isn't valid
                        (*client).ps.fd.saberAnimLevelBase = SS_DUAL as c_int;
                        (*client).ps.fd.saberAnimLevel = SS_DUAL as c_int;
                    }
                    (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                }
            } else {
                if (*client).saber[0].stylesLearned == (1 << SS_STAFF as c_int) {
                    // then *always* use the staff style
                    (*client).ps.fd.saberAnimLevelBase = SS_STAFF as c_int;
                }
                if (*client).ps.fd.saberAnimLevelBase == SS_STAFF as c_int {
                    // using staff style
                    if (*client).ps.saberHolstered == 1
                        && (*client).saber[0].singleBladeStyle != SS_NONE
                    {
                        // one blade should be off, adjust saberAnimLevel accordinly
                        (*client).ps.fd.saberAnimLevel = (*client).saber[0].singleBladeStyle as c_int;
                        (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                    } else {
                        (*client).ps.fd.saberAnimLevel = SS_STAFF as c_int;
                        (*client).ps.fd.saberDrawAnimLevel = (*client).ps.fd.saberAnimLevel;
                    }
                }
            }
        }

        // mark the time, so the connection sprite can be removed
        let mut ucmd: *mut usercmd_t = &mut (*client).pers.cmd as *mut usercmd_t;

        if (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER != 0 {
            G_HeldByMonster(ctx, ent, &mut ucmd as *mut *mut usercmd_t);
        }

        // sanity check the command time to prevent speedup cheating
        if (*ucmd).serverTime > (*ctx.world).level.time + 200 {
            (*ucmd).serverTime = (*ctx.world).level.time + 200;
        }
        if (*ucmd).serverTime < (*ctx.world).level.time - 1000 {
            (*ucmd).serverTime = (*ctx.world).level.time - 1000;
        }

        if isNPC != qfalse && ((*ucmd).serverTime - (*client).ps.commandTime) < 1 {
            (*ucmd).serverTime = (*client).ps.commandTime + 100;
        }

        let mut msec = (*ucmd).serverTime - (*client).ps.commandTime;
        // following others may result in bad times, but we still want
        // to check for follow toggles
        if msec < 1 && (*client).sess.spectatorState != SPECTATOR_FOLLOW {
            return;
        }

        if msec > 200 {
            msec = 200;
        }

        if (*ctx.world).cvars.pmove_msec.integer < 8 {
            trap::Cvar_Set(
                ctx.engine,
                mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs::new(cstr("pmove_msec"), cstr("8")),
            );
        } else if (*ctx.world).cvars.pmove_msec.integer > 33 {
            trap::Cvar_Set(
                ctx.engine,
                mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs::new(cstr("pmove_msec"), cstr("33")),
            );
        }

        if (*ctx.world).cvars.pmove_fixed.integer != 0 || (*client).pers.pmoveFixed != qfalse {
            (*ucmd).serverTime = ((*ucmd).serverTime + (*ctx.world).cvars.pmove_msec.integer - 1)
                / (*ctx.world).cvars.pmove_msec.integer
                * (*ctx.world).cvars.pmove_msec.integer;
        }

        // check for exiting intermission
        if (*ctx.world).level.intermissiontime != 0 {
            if (*ent).s.number < MAX_CLIENTS as c_int || (*client).NPC_class == CLASS_VEHICLE as c_int
            {
                // players and vehicles do nothing in intermissions
                ClientIntermissionThink(client);
                return;
            }
        }

        // spectators don't do much
        if (*client).sess.sessionTeam == TEAM_SPECTATOR || (*client).tempSpectate > (*ctx.world).level.time {
            if (*client).sess.spectatorState == SPECTATOR_SCOREBOARD {
                return;
            }
            SpectatorThink(ctx, ent, ucmd);
            return;
        }

        if !ent.is_null() && !(*ent).client.is_null() && (*client).ps.eFlags & EF_INVULNERABLE != 0 {
            if (*client).invulnerableTimer <= (*ctx.world).level.time {
                (*client).ps.eFlags &= !EF_INVULNERABLE;
            }
        }

        if (*ent).s.eType != ET_NPC as c_int {
            // check for inactivity timer, but never drop the local client of a non-dedicated server
            if ClientInactivityTimer(ctx, client) == qfalse {
                return;
            }
        }

        // Check if we should have a fullbody push effect around the player
        if (*client).pushEffectTime > (*ctx.world).level.time {
            (*client).ps.eFlags |= EF_BODYPUSH;
        } else if (*client).pushEffectTime != 0 {
            (*client).pushEffectTime = 0;
            (*client).ps.eFlags &= !EF_BODYPUSH;
        }

        if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_JETPACK as c_int) != 0 {
            (*client).ps.eFlags |= EF_JETPACK;
        } else {
            (*client).ps.eFlags &= !EF_JETPACK;
        }

        if (*client).noclip != qfalse {
            (*client).ps.pm_type = PM_NOCLIP;
        } else if (*client).ps.eFlags & EF_DISINTEGRATION != 0 {
            (*client).ps.pm_type = PM_NOCLIP;
        } else if (*client).ps.stats[STAT_HEALTH as usize] <= 0 {
            (*client).ps.pm_type = PM_DEAD;
        } else if (*client).ps.forceGripChangeMovetype != 0 {
            (*client).ps.pm_type = (*client).ps.forceGripChangeMovetype;
        } else if (*client).jetPackOn != qfalse {
            (*client).ps.pm_type = PM_JETPACK;
            (*client).ps.eFlags |= EF_JETPACK_ACTIVE;
            killJetFlags = qfalse;
        } else {
            (*client).ps.pm_type = PM_NORMAL;
        }

        if killJetFlags != qfalse {
            (*client).ps.eFlags &= !EF_JETPACK_ACTIVE;
            (*client).ps.eFlags &= !EF_JETPACK_FLAMING;
        }

        // Raven `#define SLOWDOWN_DIST 128.0f` / `#define MIN_NPC_SPEED 16.0f` (g_active.c:2203-2204)
        const SLOWDOWN_DIST: f32 = 128.0;
        const MIN_NPC_SPEED: c_int = 16;

        if (*client).bodyGrabIndex != ENTITYNUM_NONE as c_int {
            let grabbed = &mut (*ctx.world).entities[(*client).bodyGrabIndex as usize] as *mut gentity_t;

            if (*grabbed).inuse == qfalse
                || (*grabbed).s.eType != ET_BODY as c_int
                || (*grabbed).s.eFlags & EF_DISINTEGRATION != 0
                || (*grabbed).s.eFlags & EF_NODRAW != 0
            {
                if (*grabbed).inuse != qfalse && (*grabbed).s.eType == ET_BODY as c_int {
                    (*grabbed).s.ragAttach = 0;
                }
                (*client).bodyGrabIndex = ENTITYNUM_NONE as c_int;
            } else {
                let mut rhMat: mdxaBone_t = core::mem::zeroed();
                let mut rhOrg: vec3_t = [0.0; 3];
                let mut tAng: vec3_t = [0.0; 3];
                let mut bodyDir: vec3_t = [0.0; 3];

                (*client).ps.forceHandExtend = HANDEXTEND_DRAGGING as c_int;

                if (*client).ps.forceHandExtendTime < (*ctx.world).level.time + 500 {
                    (*client).ps.forceHandExtendTime = (*ctx.world).level.time + 1000;
                }

                crate::q_math::VectorSet(&mut tAng, 0.0, (*client).ps.viewangles[YAW], 0.0);
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                        (*ent).ghoul2,
                        0,
                        0, // 0 is always going to be right hand bolt
                        &mut rhMat as *mut mdxaBone_t,
                        &tAng as *const vec3_t,
                        &(*client).ps.origin as *const vec3_t,
                        (*ctx.world).level.time,
                        core::ptr::null_mut(),
                        &(*ent).modelScale as *const vec3_t,
                    ),
                );
                BG_GiveMeVectorFromMatrix(&rhMat, Eorientations::ORIGIN as c_int, &mut rhOrg);

                crate::q_math::_VectorSubtract(rhOrg, (*grabbed).r.currentOrigin, &mut bodyDir);
                let bodyDist = VectorLength(bodyDir);

                if bodyDist > 40.0 {
                    // can no longer reach
                    (*grabbed).s.ragAttach = 0;
                    (*client).bodyGrabIndex = ENTITYNUM_NONE as c_int;
                } else if bodyDist > 24.0 {
                    bodyDir[2] = 0.0; // don't want it floating
                    crate::q_math::_VectorAdd((*grabbed).epVelocity, bodyDir, &mut (*grabbed).epVelocity);
                    G_Sound(ctx, grabbed, CHAN_AUTO as c_int, G_SoundIndex(cstr("sound/player/roll1.wav").as_ptr()));
                }
            }
        } else if (*client).ps.forceHandExtend == HANDEXTEND_DRAGGING as c_int {
            (*client).ps.forceHandExtend = HANDEXTEND_WEAPONREADY as c_int;
        }

        if !(*ent).NPC.is_null() && (*ent).s.NPC_class != CLASS_VEHICLE as c_int {
            // vehicles manage their own speed
            let npc = (*ent).NPC as *mut gNPC_t;
            // FIXME: swoop should keep turning (and moving forward?) for a little bit?
            if (*npc).combatMove == qfalse {
                // Not leaning
                let Flying = (*ucmd).upmove != 0 && (*client).ps.eFlags2 & EF2_FLYING != 0;
                let Climbing = (*ucmd).upmove != 0 && (*ent).watertype & CONTENTS_LADDER != 0;

                if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 || Flying {
                    // In-Formation NPCs set thier desiredSpeed themselves
                    if (*ucmd).buttons & BUTTON_WALKING != 0 {
                        (*npc).desiredSpeed = NPC_GetWalkSpeed(ent);
                    } else {
                        // running
                        (*npc).desiredSpeed = NPC_GetRunSpeed(ent);
                    }

                    if (*npc).currentSpeed >= 80 && controlledByPlayer == qfalse {
                        // At higher speeds, need to slow down close to stuff
                        // Slow down as you approach your goal
                        if (*npc).distToGoal < SLOWDOWN_DIST && (*npc).aiFlags & NPCAI_NO_SLOWDOWN == 0 {
                            if (*npc).desiredSpeed > MIN_NPC_SPEED {
                                let slowdownSpeed = ((*npc).desiredSpeed as f32) * (*npc).distToGoal / SLOWDOWN_DIST;

                                (*npc).desiredSpeed = slowdownSpeed.ceil() as c_int;
                                if (*npc).desiredSpeed < MIN_NPC_SPEED {
                                    // don't slow down too much
                                    (*npc).desiredSpeed = MIN_NPC_SPEED;
                                }
                            }
                        }
                    }
                } else if Climbing {
                    (*npc).desiredSpeed = (*npc).stats.walkSpeed;
                } else {
                    // We want to stop
                    (*npc).desiredSpeed = 0;
                }

                NPC_Accelerate(ent, qfalse, qfalse);

                if (*npc).currentSpeed <= 24 && (*npc).desiredSpeed < (*npc).currentSpeed {
                    // No-one walks this slow
                    (*client).ps.speed = 0.0;
                    (*npc).currentSpeed = 0; // Full stop
                    (*ucmd).forwardmove = 0;
                    (*ucmd).rightmove = 0;
                } else {
                    if (*npc).currentSpeed <= (*npc).stats.walkSpeed {
                        // Play the walkanim
                        (*ucmd).buttons |= BUTTON_WALKING;
                    } else {
                        (*ucmd).buttons &= !BUTTON_WALKING;
                    }

                    if (*npc).currentSpeed > 0 {
                        // We should be moving
                        if Climbing || Flying {
                            if (*ucmd).upmove == 0 {
                                // We need to force them to take a couple more steps until stopped
                                (*ucmd).upmove = (*npc).last_ucmd.upmove;
                            }
                        } else if (*ucmd).forwardmove == 0 && (*ucmd).rightmove == 0 {
                            // We need to force them to take a couple more steps until stopped
                            (*ucmd).forwardmove = (*npc).last_ucmd.forwardmove;
                            (*ucmd).rightmove = (*npc).last_ucmd.rightmove;
                        }
                    }

                    (*client).ps.speed = (*npc).currentSpeed as f32;
                    // rwwFIXMEFIXME: do this and also check for all real client
                    // Slow down on turns - don't orbit!!!
                    let mut turndelta = 0.0f32;
                    // rwwFIXMEFIXME: locked-yaw RF_LOCKEDANGLE path is unreachable
                    // (Raven guards it with `if (0)`) — port the always-taken branch.
                    turndelta = (180.0 - crate::q_math::AngleDelta((*ent).r.currentAngles[YAW], (*npc).desiredYaw).abs()) / 180.0;

                    if turndelta < 0.75 {
                        (*client).ps.speed = 0.0;
                    } else if (*npc).distToGoal < 100.0 && turndelta < 1.0 {
                        // Turn is greater than 45 degrees or closer than 100 to goal
                        (*client).ps.speed = ((*client).ps.speed * turndelta).floor();
                    }
                }
            } else {
                (*npc).desiredSpeed = if (*ucmd).buttons & BUTTON_WALKING != 0 {
                    NPC_GetWalkSpeed(ent)
                } else {
                    NPC_GetRunSpeed(ent)
                };

                (*client).ps.speed = (*npc).desiredSpeed as f32;
            }

            if (*ucmd).buttons & BUTTON_WALKING != 0 {
                // sort of a hack I guess since MP handles walking differently from SP
                // (has some proxy cheat prevention methods)
                if (*ucmd).forwardmove > 64 {
                    (*ucmd).forwardmove = 64;
                } else if (*ucmd).forwardmove < -64 {
                    (*ucmd).forwardmove = -64;
                }

                if (*ucmd).rightmove > 64 {
                    (*ucmd).rightmove = 64;
                } else if (*ucmd).rightmove < -64 {
                    (*ucmd).rightmove = -64;
                }
            }
            (*client).ps.basespeed = (*client).ps.speed;
        } else if (*client).ps.m_iVehicleNum == 0
            && ((*ent).NPC.is_null() || (*ent).s.NPC_class != CLASS_VEHICLE as c_int)
        {
            // if riding a vehicle it will manage our speed and such
            // set speed
            (*client).ps.speed = (*ctx.world).cvars.g_speed.value;

            // Check for a siege class speed multiplier
            if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE && (*client).siegeClass != -1 {
                (*client).ps.speed *= (*ctx.world).bg_state.bgSiegeClasses[(*client).siegeClass as usize].speed;
            }

            if (*client).bodyGrabIndex != ENTITYNUM_NONE as c_int {
                // can't go nearly as fast when dragging a body around
                (*client).ps.speed *= 0.2;
            }

            (*client).ps.basespeed = (*client).ps.speed;
        }

        if (*ent).NPC.is_null()
            || (*((*ent).NPC as *mut gNPC_t)).aiFlags & NPCAI_CUSTOM_GRAVITY == 0
        {
            // use global gravity
            let vehVehicle = (*ent).m_pVehicle as *mut Vehicle_t;
            if !(*ent).NPC.is_null()
                && (*ent).s.NPC_class == CLASS_VEHICLE as c_int
                && !vehVehicle.is_null()
                && (*(*vehVehicle).m_pVehicleInfo).gravity != 0
            {
                // use custom veh gravity
                (*client).ps.gravity = (*(*vehVehicle).m_pVehicleInfo).gravity;
            } else if (*client).inSpaceIndex != 0 && (*client).inSpaceIndex != ENTITYNUM_NONE as c_int {
                // in space, so no gravity...
                (*client).ps.gravity = 1;
                if (*ent).s.number < MAX_CLIENTS as c_int {
                    crate::q_math::_VectorScale((*client).ps.velocity, 0.8, &mut (*client).ps.velocity);
                }
            } else if (*client).ps.eFlags2 & EF2_SHIP_DEATH != 0 {
                // float there
                (*client).ps.velocity = [0.0; 3];
                (*client).ps.gravity = 1;
            } else {
                (*client).ps.gravity = (*ctx.world).cvars.g_gravity.value as c_int;
            }
        }

        if (*client).ps.duelInProgress != qfalse {
            let duelAgainst = &mut (*ctx.world).entities[(*client).ps.duelIndex as usize] as *mut gentity_t;

            // Keep the time updated, so once this duel ends this player can't engage in a duel for another
            // 10 seconds. This will give other people a chance to engage in duels in case this player wants
            // to engage again right after he's done fighting and someone else is waiting.
            (*client).ps.fd.privateDuelTime = (*ctx.world).level.time + 10000;

            if (*client).ps.duelTime < (*ctx.world).level.time {
                // Bring out the sabers
                if (*client).ps.weapon == WP_SABER
                    && (*client).ps.saberHolstered != 0
                    && (*client).ps.duelTime != 0
                {
                    (*client).ps.saberHolstered = 0;

                    if (*client).saber[0].soundOn != 0 {
                        G_Sound(ctx, ent, CHAN_AUTO as c_int, (*client).saber[0].soundOn);
                    }
                    if (*client).saber[1].soundOn != 0 {
                        G_Sound(ctx, ent, CHAN_AUTO as c_int, (*client).saber[1].soundOn);
                    }

                    G_AddEvent(ent, EV_PRIVATE_DUEL as c_int, 2);

                    (*client).ps.duelTime = 0;
                }

                if !duelAgainst.is_null()
                    && !(*duelAgainst).client.is_null()
                    && (*duelAgainst).inuse != qfalse
                    && (*((*duelAgainst).client as *mut gclient_t)).ps.weapon == WP_SABER
                    && (*((*duelAgainst).client as *mut gclient_t)).ps.saberHolstered != 0
                    && (*((*duelAgainst).client as *mut gclient_t)).ps.duelTime != 0
                {
                    let daClient = (*duelAgainst).client as *mut gclient_t;
                    (*daClient).ps.saberHolstered = 0;

                    if (*daClient).saber[0].soundOn != 0 {
                        G_Sound(ctx, duelAgainst, CHAN_AUTO as c_int, (*daClient).saber[0].soundOn);
                    }
                    if (*daClient).saber[1].soundOn != 0 {
                        G_Sound(ctx, duelAgainst, CHAN_AUTO as c_int, (*daClient).saber[1].soundOn);
                    }

                    G_AddEvent(duelAgainst, EV_PRIVATE_DUEL as c_int, 2);

                    (*daClient).ps.duelTime = 0;
                }
            } else {
                (*client).ps.speed = 0.0;
                (*client).ps.basespeed = 0.0;
                (*ucmd).forwardmove = 0;
                (*ucmd).rightmove = 0;
                (*ucmd).upmove = 0;
            }

            if duelAgainst.is_null()
                || (*duelAgainst).client.is_null()
                || (*duelAgainst).inuse == qfalse
                || (*((*duelAgainst).client as *mut gclient_t)).ps.duelIndex != (*ent).s.number
            {
                (*client).ps.duelInProgress = 0;
                G_AddEvent(ent, EV_PRIVATE_DUEL as c_int, 0);
            } else if (*duelAgainst).health < 1
                || (*((*duelAgainst).client as *mut gclient_t)).ps.stats[STAT_HEALTH as usize] < 1
            {
                let daClient = (*duelAgainst).client as *mut gclient_t;
                (*client).ps.duelInProgress = 0;
                (*daClient).ps.duelInProgress = 0;

                G_AddEvent(ent, EV_PRIVATE_DUEL as c_int, 0);
                G_AddEvent(duelAgainst, EV_PRIVATE_DUEL as c_int, 0);

                // Winner gets full health.. providing he's still alive
                if (*ent).health > 0 && (*client).ps.stats[STAT_HEALTH as usize] > 0 {
                    if (*ent).health < (*client).ps.stats[STAT_MAX_HEALTH as usize] {
                        (*client).ps.stats[STAT_HEALTH as usize] = (*client).ps.stats[STAT_MAX_HEALTH as usize];
                        (*ent).health = (*client).ps.stats[STAT_HEALTH as usize];
                    }

                    if (*ctx.world).cvars.g_spawnInvulnerability.integer != 0 {
                        (*client).ps.eFlags |= EF_INVULNERABLE;
                        (*client).invulnerableTimer =
                            (*ctx.world).level.time + (*ctx.world).cvars.g_spawnInvulnerability.integer;
                    }
                }

                // Private duel announcements are now made globally because we only want one duel at a time.
                if (*ent).health > 0 && (*client).ps.stats[STAT_HEALTH as usize] > 0 {
                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELWINNER".as_ptr() as *mut c_char,
                    );
                    let s = format!(
                        "cp \"{} {} {}!\n\"",
                        cstr_to_str((*client).pers.netname.as_ptr()),
                        cstr_to_str(m),
                        cstr_to_str((*daClient).pers.netname.as_ptr())
                    );
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(-1, cstr(&s)),
                    );
                } else {
                    // it was a draw, because we both managed to die in the same frame
                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELTIE".as_ptr() as *mut c_char,
                    );
                    let s = format!("cp \"{}\n\"", cstr_to_str(m));
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(-1, cstr(&s)),
                    );
                }
            } else {
                let mut vSub: vec3_t = [0.0; 3];
                crate::q_math::_VectorSubtract((*client).ps.origin, (*((*duelAgainst).client as *mut gclient_t)).ps.origin, &mut vSub);
                let subLen = VectorLength(vSub);

                if subLen >= 1024.0 {
                    let daClient = (*duelAgainst).client as *mut gclient_t;
                    (*client).ps.duelInProgress = 0;
                    (*daClient).ps.duelInProgress = 0;

                    G_AddEvent(ent, EV_PRIVATE_DUEL as c_int, 0);
                    G_AddEvent(duelAgainst, EV_PRIVATE_DUEL as c_int, 0);

                    let m = crate::g_main::G_GetStringEdString(
                        ctx,
                        c"MP_SVGAME".as_ptr() as *mut c_char,
                        c"PLDUELSTOP".as_ptr() as *mut c_char,
                    );
                    let s = format!("print \"{}\n\"", cstr_to_str(m));
                    trap::SendServerCommand(
                        ctx.engine,
                        mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs::new(-1, cstr(&s)),
                    );
                }
            }
        }

        if (*client).doingThrow > (*ctx.world).level.time {
            let throwee = &mut (*ctx.world).entities[(*client).throwingIndex as usize] as *mut gentity_t;

            if (*throwee).inuse == qfalse
                || (*throwee).client.is_null()
                || (*throwee).health < 1
                || (*((*throwee).client as *mut gclient_t)).sess.sessionTeam == TEAM_SPECTATOR
                || (*((*throwee).client as *mut gclient_t)).ps.pm_flags & PMF_FOLLOW != 0
                || (*((*throwee).client as *mut gclient_t)).throwingIndex != (*ent).s.number
            {
                (*client).doingThrow = 0;
                (*client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;

                if (*throwee).inuse != qfalse && !(*throwee).client.is_null() {
                    let toClient = (*throwee).client as *mut gclient_t;
                    (*toClient).ps.heldByClient = 0;
                    (*toClient).beingThrown = 0;

                    if (*toClient).ps.forceHandExtend != HANDEXTEND_POSTTHROWN as c_int {
                        (*toClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                    }
                }
            }
        }

        if (*client).beingThrown > (*ctx.world).level.time {
            let thrower = &mut (*ctx.world).entities[(*client).throwingIndex as usize] as *mut gentity_t;

            if (*thrower).inuse == qfalse
                || (*thrower).client.is_null()
                || (*thrower).health < 1
                || (*((*thrower).client as *mut gclient_t)).sess.sessionTeam == TEAM_SPECTATOR
                || (*((*thrower).client as *mut gclient_t)).ps.pm_flags & PMF_FOLLOW != 0
                || (*((*thrower).client as *mut gclient_t)).throwingIndex != (*ent).s.number
            {
                (*client).ps.heldByClient = 0;
                (*client).beingThrown = 0;

                if (*client).ps.forceHandExtend != HANDEXTEND_POSTTHROWN as c_int {
                    (*client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                }

                if (*thrower).inuse != qfalse && !(*thrower).client.is_null() {
                    let thClient = (*thrower).client as *mut gclient_t;
                    (*thClient).doingThrow = 0;
                    (*thClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                }
            } else {
                let thClient = (*thrower).client as *mut gclient_t;
                if !(*thrower).ghoul2.is_null()
                    && trap::G2_HaveWeGhoul2Models(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_HAVEWEGHOULMODELS::GG2HaveweghoulmodelsArgs::new(
                            (*thrower).ghoul2,
                        ),
                    ) != qfalse
                {
                    // Raven's `#if 0` bolt-index computation is dead code (never
                    // taken); the always-taken `#else` path below is transcribed.
                    let pDif = 40.0f32;
                    let mut boltOrg: vec3_t = [0.0; 3];
                    let mut pBoltOrg: vec3_t = [0.0; 3];
                    let mut tAngles: vec3_t = [0.0; 3];
                    let mut vDif: vec3_t = [0.0; 3];
                    let mut entDir: vec3_t = [0.0; 3];
                    let mut otherAngles: vec3_t = [0.0; 3];
                    let mut fwd: vec3_t = [0.0; 3];
                    let mut right: vec3_t = [0.0; 3];

                    // Always look at the thrower.
                    crate::q_math::_VectorSubtract((*thClient).ps.origin, (*client).ps.origin, &mut entDir);
                    crate::q_math::_VectorCopy((*client).ps.viewangles, &mut otherAngles);
                    otherAngles[YAW] = vectoyaw(entDir);
                    SetClientViewAngle(ent, otherAngles);

                    crate::q_math::_VectorCopy((*thClient).ps.viewangles, &mut tAngles);
                    tAngles[PITCH] = 0.0;
                    tAngles[ROLL] = 0.0;

                    // Get the direction between the pelvis and position of the hand
                    crate::q_math::_VectorCopy((*thClient).ps.origin, &mut pBoltOrg);
                    AngleVectors(tAngles, Some(&mut fwd), Some(&mut right), None);
                    boltOrg[0] = pBoltOrg[0] + fwd[0] * 8.0 + right[0] * pDif;
                    boltOrg[1] = pBoltOrg[1] + fwd[1] * 8.0 + right[1] * pDif;
                    boltOrg[2] = pBoltOrg[2];

                    crate::q_math::_VectorSubtract((*client).ps.origin, boltOrg, &mut vDif);
                    if VectorLength(vDif) > 32.0 && ((*thClient).doingThrow - (*ctx.world).level.time) < 4500 {
                        // the hand is too far away, and can no longer hold onto us, so escape.
                        (*client).ps.heldByClient = 0;
                        (*client).beingThrown = 0;
                        (*thClient).doingThrow = 0;

                        (*thClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                        G_EntitySound(ctx, thrower, CHAN_VOICE as c_int, G_SoundIndex(cstr("*pain25.wav").as_ptr()));

                        (*client).ps.forceDodgeAnim = 2;
                        (*client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                        (*client).ps.forceHandExtendTime = (*ctx.world).level.time + 500;
                        (*client).ps.velocity[2] = 400.0;
                        G_PreDefSound(ctx, (*client).ps.origin, PDSOUND_FORCEJUMP as c_int);
                    } else if ((*client).beingThrown - (*ctx.world).level.time) < 4000 {
                        // step into the next part of the throw, and go flying back
                        let vScale = 400.0f32;
                        (*client).ps.forceHandExtend = HANDEXTEND_POSTTHROWN as c_int;
                        (*client).ps.forceHandExtendTime = (*ctx.world).level.time + 1200;
                        (*client).ps.forceDodgeAnim = 0;

                        (*thClient).ps.forceHandExtend = HANDEXTEND_POSTTHROW as c_int;
                        (*thClient).ps.forceHandExtendTime = (*ctx.world).level.time + 200;

                        (*client).ps.heldByClient = 0;
                        (*client).beingThrown = 0;
                        (*thClient).doingThrow = 0;

                        AngleVectors((*thClient).ps.viewangles, Some(&mut vDif), None, None);
                        (*client).ps.velocity[0] = vDif[0] * vScale;
                        (*client).ps.velocity[1] = vDif[1] * vScale;
                        (*client).ps.velocity[2] = 400.0;

                        G_EntitySound(ctx, ent, CHAN_VOICE as c_int, G_SoundIndex(cstr("*pain100.wav").as_ptr()));
                        G_EntitySound(ctx, thrower, CHAN_VOICE as c_int, G_SoundIndex(cstr("*jump1.wav").as_ptr()));

                        // Set the thrower as the "other killer", so if we die from
                        // fall/impact damage he is credited.
                        (*client).ps.otherKiller = (*thrower).s.number;
                        (*client).ps.otherKillerTime = (*ctx.world).level.time + 8000;
                        (*client).ps.otherKillerDebounceTime = (*ctx.world).level.time + 100;
                        (*client).otherKillerMOD = MOD_FALLING as c_int;
                        (*client).otherKillerVehWeapon = 0;
                        (*client).otherKillerWeaponType = WP_NONE as c_int;
                    } else {
                        // see if we can move to be next to the hand.. if it's not
                        // clear, break the throw.
                        let mut intendedOrigin: vec3_t = [0.0; 3];
                        let mut tr: trace_t = core::mem::zeroed();
                        let mut tr2: trace_t = core::mem::zeroed();

                        crate::q_math::_VectorSubtract(boltOrg, pBoltOrg, &mut vDif);
                        VectorNormalize(&mut vDif);

                        (*client).ps.velocity = [0.0; 3];
                        intendedOrigin[0] = pBoltOrg[0] + vDif[0] * pDif;
                        intendedOrigin[1] = pBoltOrg[1] + vDif[1] * pDif;
                        intendedOrigin[2] = (*thClient).ps.origin[2];

                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &intendedOrigin as *const vec3_t,
                                &(*ent).r.mins as *const vec3_t,
                                &(*ent).r.maxs as *const vec3_t,
                                &intendedOrigin as *const vec3_t,
                                (*ent).s.number,
                                (*ent).clipmask,
                            ),
                        );
                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr2 as *mut trace_t,
                                &(*client).ps.origin as *const vec3_t,
                                &(*ent).r.mins as *const vec3_t,
                                &(*ent).r.maxs as *const vec3_t,
                                &intendedOrigin as *const vec3_t,
                                (*ent).s.number,
                                CONTENTS_SOLID,
                            ),
                        );

                        if tr.fraction == 1.0 && tr.startsolid == qfalse && tr2.fraction == 1.0 && tr2.startsolid == qfalse {
                            crate::q_math::_VectorCopy(intendedOrigin, &mut (*client).ps.origin);

                            if ((*client).beingThrown - (*ctx.world).level.time) < 4800 {
                                (*client).ps.heldByClient = (*thrower).s.number + 1;
                            }
                        } else {
                            // if the guy can't be put here then it's time to break the throw off.
                            (*client).ps.heldByClient = 0;
                            (*client).beingThrown = 0;
                            (*thClient).doingThrow = 0;

                            (*thClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                            G_EntitySound(ctx, thrower, CHAN_VOICE as c_int, G_SoundIndex(cstr("*pain25.wav").as_ptr()));

                            (*client).ps.forceDodgeAnim = 2;
                            (*client).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                            (*client).ps.forceHandExtendTime = (*ctx.world).level.time + 500;
                            (*client).ps.velocity[2] = 400.0;
                            G_PreDefSound(ctx, (*client).ps.origin, PDSOUND_FORCEJUMP as c_int);
                        }
                    }
                }
            }
        } else if (*client).ps.heldByClient != 0 {
            (*client).ps.heldByClient = 0;
        }

        // rww - moved this stuff into the pmove code so that it's predicted properly
        // set up for pmove
        let oldEventSequence = (*client).ps.eventSequence;

        let mut pm: pmove_t = core::mem::zeroed();

        if (*ent).flags & FL_FORCE_GESTURE != 0 {
            (*ent).flags &= !FL_FORCE_GESTURE;
            (*client).pers.cmd.buttons |= BUTTON_GESTURE;
        }

        if !(*ent).client.is_null()
            && (*client).ps.fallingToDeath != 0
            && ((*ctx.world).level.time - FALL_FADE_TIME) > (*client).ps.fallingToDeath
        {
            // die!
            if (*ent).health > 0 {
                let mut otherKiller = ent;
                if (*client).ps.otherKillerTime > (*ctx.world).level.time
                    && (*client).ps.otherKiller != ENTITYNUM_NONE as c_int
                {
                    otherKiller = &mut (*ctx.world).entities[(*client).ps.otherKiller as usize] as *mut gentity_t;

                    if (*otherKiller).inuse == qfalse {
                        otherKiller = ent;
                    }
                }
                G_Damage(
                    ctx,
                    ent,
                    otherKiller,
                    otherKiller,
                    None,
                    (*client).ps.origin,
                    9999,
                    DAMAGE_NO_PROTECTION,
                    MOD_FALLING as c_int,
                );

                G_MuteSound(ctx, (*ent).s.number, CHAN_VOICE as c_int); // stop screaming, because you are dead!
            }
        }

        if (*client).ps.otherKillerTime > (*ctx.world).level.time
            && (*client).ps.groundEntityNum != ENTITYNUM_NONE as c_int
            && (*client).ps.otherKillerDebounceTime < (*ctx.world).level.time
        {
            (*client).ps.otherKillerTime = 0;
            (*client).ps.otherKiller = ENTITYNUM_NONE as c_int;
        } else if (*client).ps.otherKillerTime > (*ctx.world).level.time
            && (*client).ps.groundEntityNum == ENTITYNUM_NONE as c_int
        {
            if (*client).ps.otherKillerDebounceTime < ((*ctx.world).level.time + 100) {
                (*client).ps.otherKillerDebounceTime = (*ctx.world).level.time + 100;
            }
        }

        // NOTE: can't put USE here *before* PMove!!
        if (*client).ps.useDelay > (*ctx.world).level.time && (*client).ps.m_iVehicleNum != 0 {
            // when in a vehicle, debounce the use...
            (*ucmd).buttons &= !BUTTON_USE;
        }

        // FIXME: need to do this before check to avoid walls and cliffs (or just cliffs?)
        G_AddPushVecToUcmd(ctx, ent, ucmd);

        // play/stop any looping sounds tied to controlled movement
        G_CheckMovingLoopingSounds(ctx, ent, ucmd);

        pm.ps = &mut (*client).ps as *mut playerState_t;
        pm.cmd = *ucmd;
        if (*pm.ps).pm_type == PM_DEAD {
            pm.tracemask = MASK_PLAYERSOLID & !CONTENTS_BODY;
        } else if (*ent).r.svFlags & SVF_BOT != 0 {
            pm.tracemask = MASK_PLAYERSOLID | CONTENTS_MONSTERCLIP;
        } else {
            pm.tracemask = MASK_PLAYERSOLID;
        }
        // ruling 21: pm.trace/pointcontents fields stay for layout only; bg logic
        // reaches the engine through BgTraps threaded into Pmove below.
        pm.debugLevel = (*ctx.world).cvars.g_debugMove.integer;
        pm.noFootsteps = (((*ctx.world).cvars.g_dmflags.integer & DF_NO_FOOTSTEPS) > 0) as qboolean;

        pm.pmove_fixed = (*ctx.world).cvars.pmove_fixed.integer | (*client).pers.pmoveFixed;
        pm.pmove_msec = (*ctx.world).cvars.pmove_msec.integer;

        pm.animations = (*ctx.world).bg_state.bgAllAnims[(*ent).localAnimIndex as usize]
            .anims
            .as_mut_ptr();

        // rww - bgghoul2
        pm.ghoul2 = core::ptr::null_mut();

        if !(*ent).ghoul2.is_null() {
            if (*ent).localAnimIndex > 1 {
                // if it isn't humanoid then we will be having none of this.
                pm.ghoul2 = core::ptr::null_mut();
            } else {
                pm.ghoul2 = (*ent).ghoul2;
                pm.g2Bolts_LFoot = trap::G2API_AddBolt(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*ent).ghoul2, 0, c"*l_leg_foot".as_ptr()),
                );
                pm.g2Bolts_RFoot = trap::G2API_AddBolt(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*ent).ghoul2, 0, c"*r_leg_foot".as_ptr()),
                );
            }
        }

        // I'll just do this every frame in case the scale changes in realtime
        // (don't need to update the g2 inst for that)
        crate::q_math::_VectorCopy((*ent).modelScale, &mut pm.modelScale);
        // rww end bgghoul2

        pm.gametype = (*ctx.world).cvars.g_gametype.integer;
        pm.debugMelee = (*ctx.world).cvars.g_debugMelee.integer;
        pm.stepSlideFix = (*ctx.world).cvars.g_stepSlideFix.integer;

        pm.noSpecMove = (*ctx.world).cvars.g_noSpecMove.integer;

        pm.nonHumanoid = ((*ent).localAnimIndex > 0) as qboolean;

        crate::q_math::_VectorCopy((*client).ps.origin, &mut (*client).oldOrigin);

        // Set up bg entity data
        pm.baseEnt = (*ctx.world).entities.as_mut_ptr() as *mut _;
        pm.entSize = core::mem::size_of::<gentity_t>() as c_int;

        if (*client).ps.saberLockTime > (*ctx.world).level.time {
            let blockOpp = &mut (*ctx.world).entities[(*client).ps.saberLockEnemy as usize] as *mut gentity_t;

            if !(*blockOpp).client.is_null() && (*blockOpp).inuse != qfalse {
                let mut lockDir: vec3_t = [0.0; 3];
                let mut lockAng: vec3_t = [0.0; 3];

                crate::q_math::_VectorSubtract((*blockOpp).r.currentOrigin, (*ent).r.currentOrigin, &mut lockDir);
                vectoangles(lockDir, &mut lockAng);
                SetClientViewAngle(ent, lockAng);
            }

            if (*client).ps.saberLockHitCheckTime < (*ctx.world).level.time {
                // have moved to next frame since last lock push
                (*client).ps.saberLockHitCheckTime = (*ctx.world).level.time; // so we don't push more than once per server frame
                if (*client).buttons & BUTTON_ATTACK != 0 && (*client).oldbuttons & BUTTON_ATTACK == 0 {
                    if (*client).ps.saberLockHitIncrementTime < (*ctx.world).level.time {
                        // have moved to next frame since last saberlock attack button press
                        let mut lockHits: c_int;
                        (*client).ps.saberLockHitIncrementTime = (*ctx.world).level.time; // so we don't register an attack key press more than once per server frame
                        // NOTE: FP_SABER_OFFENSE level already taken into account in PM_SaberLocked
                        if (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                            // raging: push harder
                            lockHits = 1 + (*client).ps.fd.forcePowerLevel[FP_RAGE as usize];
                        } else {
                            // normal attack
                            lockHits = match (*client).ps.fd.saberAnimLevel {
                                x if x == SS_FAST as c_int => 1,
                                x if x == SS_MEDIUM as c_int
                                    || x == SS_TAVION as c_int
                                    || x == SS_DUAL as c_int
                                    || x == SS_STAFF as c_int =>
                                {
                                    2
                                }
                                x if x == SS_STRONG as c_int || x == SS_DESANN as c_int => 3,
                                _ => 0,
                            };
                        }
                        if (*client).ps.fd.forceRageRecoveryTime > (*ctx.world).level.time
                            && (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0
                        {
                            // finished raging: weak
                            lockHits -= 1;
                        }
                        lockHits += (*client).saber[0].lockBonus;
                        if (*client).saber[1].model[0] != 0 && (*client).ps.saberHolstered == 0 {
                            lockHits += (*client).saber[1].lockBonus;
                        }
                        (*client).ps.saberLockHits += lockHits;
                        if (*ctx.world).cvars.g_saberLockRandomNess.integer != 0 {
                            (*client).ps.saberLockHits += (*ctx.world)
                                .bg_state
                                .rng
                                .Q_irand(0, (*ctx.world).cvars.g_saberLockRandomNess.integer);
                            if (*client).ps.saberLockHits < 0 {
                                (*client).ps.saberLockHits = 0;
                            }
                        }
                    }
                }
                if (*client).ps.saberLockHits > 0 {
                    if (*client).ps.saberLockAdvance == qfalse {
                        (*client).ps.saberLockHits -= 1;
                    }
                    (*client).ps.saberLockAdvance = qtrue;
                }
            }
        } else {
            (*client).ps.saberLockFrame = 0;
            // check for taunt
            if pm.cmd.generic_cmd as c_int == GENCMD_ENGAGE_DUEL as c_int
                && ((*ctx.world).cvars.g_gametype.integer == GT_DUEL
                    || (*ctx.world).cvars.g_gametype.integer == GT_POWERDUEL)
            {
                // already in a duel, make it a taunt command
                pm.cmd.buttons |= BUTTON_GESTURE;
            }
        }

        if (*ent).s.number >= MAX_CLIENTS as c_int {
            crate::q_math::_VectorCopy((*ent).r.mins, &mut pm.mins);
            crate::q_math::_VectorCopy((*ent).r.maxs, &mut pm.maxs);

            let vehVehicle = (*ent).m_pVehicle as *mut Vehicle_t;
            if (*ent).s.NPC_class == CLASS_VEHICLE as c_int && !vehVehicle.is_null() {
                if !(*vehVehicle).m_pPilot.is_null() {
                    // vehicles want to use their last pilot ucmd I guess
                    if ((*ctx.world).level.time - (*vehVehicle).m_ucmd.serverTime) > 2000 {
                        // Previous owner disconnected, maybe
                        (*vehVehicle).m_ucmd.serverTime = (*ctx.world).level.time;
                        (*client).ps.commandTime = (*ctx.world).level.time - 100;
                        msec = 100;
                    }

                    pm.cmd = (*vehVehicle).m_ucmd;

                    // no veh can strafe
                    pm.cmd.rightmove = 0;
                    // no crouching or jumping!
                    pm.cmd.upmove = 0;

                    // NOTE: button presses were getting lost!
                    let pilotEnt = (*vehVehicle).m_pPilot as *mut gentity_t;
                    let pilotClient = (*pilotEnt).client as *mut gclient_t;
                    pm.cmd.buttons = (*pilotClient).pers.cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK);
                }
                if (*(*vehVehicle).m_pVehicleInfo).r#type == VH_WALKER {
                    if (*client).ps.groundEntityNum != ENTITYNUM_NONE as c_int {
                        // ATST crushes anything underneath it
                        let under =
                            &mut (*ctx.world).entities[(*client).ps.groundEntityNum as usize] as *mut gentity_t;
                        if (*under).health != 0 && (*under).takedamage != qfalse {
                            let down: vec3_t = [0.0, 0.0, -1.0];
                            // FIXME: we'll be doing traces down from each foot, so we'll have a real impact origin
                            G_Damage(
                                ctx,
                                under,
                                ent,
                                ent,
                                Some(&mut down.clone()),
                                (*under).r.currentOrigin,
                                100,
                                0,
                                MOD_CRUSH as c_int,
                            );
                        }
                    }
                }
            }
        }

        {
            let traps = GameBgTraps::new(ctx.engine);
            let mut callbacks = GameCallbacksImpl { world: ctx.world, engine: ctx.engine };
            Pmove(&mut pm as *mut pmove_t, &mut (*ctx.world).bg_state, &traps, &mut callbacks);
        }

        if (*client).solidHack != 0 {
            if (*client).solidHack > (*ctx.world).level.time {
                // whee!
                (*ent).r.contents = 0;
            } else {
                (*ent).r.contents = CONTENTS_BODY;
                (*client).solidHack = 0;
            }
        }

        if !(*ent).NPC.is_null() {
            crate::q_math::_VectorCopy((*client).ps.viewangles, &mut (*ent).r.currentAngles);
        }

        if pm.checkDuelLoss != 0 {
            if pm.checkDuelLoss > 0
                && (pm.checkDuelLoss <= MAX_CLIENTS as c_int
                    || (pm.checkDuelLoss < (mp_qshared::shared::MAX_GENTITIES as c_int - 1)
                        && (*ctx.world).entities[(pm.checkDuelLoss - 1) as usize].s.eType == ET_NPC as c_int))
            {
                let clientLost =
                    &mut (*ctx.world).entities[(pm.checkDuelLoss - 1) as usize] as *mut gentity_t;

                if !(*clientLost).client.is_null()
                    && (*clientLost).inuse != qfalse
                    && (*ctx.world).bg_state.rng.Q_irand(0, 40) > (*clientLost).health
                {
                    let clClient = (*clientLost).client as *mut gclient_t;
                    let mut attDir: vec3_t = [0.0; 3];
                    crate::q_math::_VectorSubtract((*client).ps.origin, (*clClient).ps.origin, &mut attDir);
                    VectorNormalize(&mut attDir);

                    (*clClient).ps.velocity = [0.0; 3];
                    (*clClient).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                    (*clClient).ps.forceHandExtendTime = 0;

                    (*ctx.world).globals.gGAvoidDismember = 1;
                    G_Damage(
                        ctx,
                        clientLost,
                        ent,
                        ent,
                        Some(&mut attDir),
                        (*clClient).ps.origin,
                        9999,
                        DAMAGE_NO_PROTECTION,
                        MOD_SABER as c_int,
                    );

                    if (*clientLost).health < 1 {
                        (*ctx.world).globals.gGAvoidDismember = 2;
                        G_CheckForDismemberment(
                            ctx,
                            clientLost,
                            ent,
                            (*clClient).ps.origin,
                            999,
                            (*clClient).ps.legsAnim,
                            qfalse,
                        );
                    }

                    (*ctx.world).globals.gGAvoidDismember = 0;
                } else if !(*clientLost).client.is_null()
                    && (*clientLost).inuse != qfalse
                    && (*(( *clientLost).client as *mut gclient_t)).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN as c_int
                    && (*(( *clientLost).client as *mut gclient_t)).ps.saberEntityNum != 0
                {
                    // if we didn't knock down it was a circle lock. So as punishment,
                    // make them lose their saber and go into a proper anim
                    let clClient = (*clientLost).client as *mut gclient_t;
                    let saberEnt = &mut (*ctx.world).entities[(*clClient).ps.saberEntityNum as usize] as *mut gentity_t;
                    saberCheckKnockdown_DuelLoss(ctx, saberEnt, clientLost, ent);
                }
            }

            pm.checkDuelLoss = 0;
        }

        if (*client).ps.groundEntityNum < ENTITYNUM_WORLD as c_int {
            // standing on an ent
            let groundEnt = &mut (*ctx.world).entities[(*client).ps.groundEntityNum as usize] as *mut gentity_t;
            if (*groundEnt).s.eType == ET_NPC as c_int
                && (*groundEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && (*groundEnt).inuse != qfalse
                && (*groundEnt).health > 0
                && !(*groundEnt).m_pVehicle.is_null()
            {
                // standing on a valid, living vehicle
                let groundClient = (*groundEnt).client as *mut gclient_t;
                let groundVeh = (*groundEnt).m_pVehicle as *mut Vehicle_t;
                if (*groundClient).ps.speed == 0.0 && (*groundVeh).m_ucmd.upmove > 0 {
                    // a vehicle that's trying to take off!
                    // just kill me
                    let up: vec3_t = [0.0, 0.0, 1.0];
                    G_Damage(
                        ctx,
                        ent,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        Some(&mut up.clone()),
                        (*ent).r.currentOrigin,
                        9999999,
                        DAMAGE_NO_PROTECTION,
                        MOD_CRUSH as c_int,
                    );
                    return;
                }
            }
        }

        if pm.cmd.generic_cmd != 0
            && (pm.cmd.generic_cmd as c_int != (*client).lastGenCmd || (*client).lastGenCmdTime < (*ctx.world).level.time)
        {
            (*client).lastGenCmd = pm.cmd.generic_cmd as c_int;
            if pm.cmd.generic_cmd as c_int != GENCMD_FORCE_THROW as c_int
                && pm.cmd.generic_cmd as c_int != GENCMD_FORCE_PULL as c_int
            {
                // these are the only two where you wouldn't care about a delay between
                (*client).lastGenCmdTime = (*ctx.world).level.time + 300; // default 100ms debounce between issuing the same command.
            }

            let gc = pm.cmd.generic_cmd as c_int;
            if gc == GENCMD_SABERSWITCH as c_int {
                Cmd_ToggleSaber_f(ctx, ent);
            } else if gc == GENCMD_ENGAGE_DUEL as c_int {
                if (*ctx.world).cvars.g_gametype.integer == GT_DUEL || (*ctx.world).cvars.g_gametype.integer == GT_POWERDUEL {
                    // already in a duel, made it a taunt command
                } else {
                    Cmd_EngageDuel_f(ctx, ent);
                }
            } else if gc == GENCMD_FORCE_HEAL as c_int {
                ForceHeal(ctx, ent);
            } else if gc == GENCMD_FORCE_SPEED as c_int {
                ForceSpeed(ctx, ent, 0);
            } else if gc == GENCMD_FORCE_THROW as c_int {
                ForceThrow(ctx, ent, qfalse);
            } else if gc == GENCMD_FORCE_PULL as c_int {
                ForceThrow(ctx, ent, qtrue);
            } else if gc == GENCMD_FORCE_DISTRACT as c_int {
                ForceTelepathy(ctx, ent);
            } else if gc == GENCMD_FORCE_RAGE as c_int {
                ForceRage(ctx, ent);
            } else if gc == GENCMD_FORCE_PROTECT as c_int {
                ForceProtect(ctx, ent);
            } else if gc == GENCMD_FORCE_ABSORB as c_int {
                ForceAbsorb(ctx, ent);
            } else if gc == GENCMD_FORCE_HEALOTHER as c_int {
                ForceTeamHeal(ctx, ent);
            } else if gc == GENCMD_FORCE_FORCEPOWEROTHER as c_int {
                ForceTeamForceReplenish(ctx, ent);
            } else if gc == GENCMD_FORCE_SEEING as c_int {
                ForceSeeing(ctx, ent);
            } else if gc == GENCMD_USE_SEEKER as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SEEKER as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_SEEKER as c_int) != 0
                {
                    ItemUse_Seeker(ctx, ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_SEEKER as c_int, 0);
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_SEEKER as c_int);
                }
            } else if gc == GENCMD_USE_FIELD as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SHIELD as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_SHIELD as c_int) != 0
                {
                    ItemUse_Shield(ctx, ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_SHIELD as c_int, 0);
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_SHIELD as c_int);
                }
            } else if gc == GENCMD_USE_BACTA as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_MEDPAC as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_MEDPAC as c_int) != 0
                {
                    ItemUse_MedPack(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_MEDPAC as c_int, 0);
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_MEDPAC as c_int);
                }
            } else if gc == GENCMD_USE_BACTABIG as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_MEDPAC_BIG as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_MEDPAC_BIG as c_int) != 0
                {
                    ItemUse_MedPack_Big(ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_MEDPAC_BIG as c_int, 0);
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_MEDPAC_BIG as c_int);
                }
            } else if gc == GENCMD_USE_ELECTROBINOCULARS as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_BINOCULARS as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_BINOCULARS as c_int) != 0
                {
                    ItemUse_Binoculars(ctx, ent);
                    if (*client).ps.zoomMode == 0 {
                        G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_BINOCULARS as c_int, 1);
                    } else {
                        G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_BINOCULARS as c_int, 2);
                    }
                }
            } else if gc == GENCMD_ZOOM as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_BINOCULARS as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_BINOCULARS as c_int) != 0
                {
                    ItemUse_Binoculars(ctx, ent);
                    if (*client).ps.zoomMode == 0 {
                        G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_BINOCULARS as c_int, 1);
                    } else {
                        G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_BINOCULARS as c_int, 2);
                    }
                }
            } else if gc == GENCMD_USE_SENTRY as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_SENTRY_GUN as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_SENTRY_GUN as c_int) != 0
                {
                    ItemUse_Sentry(ctx, ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_SENTRY_GUN as c_int, 0);
                    (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] &= !(1 << HI_SENTRY_GUN as c_int);
                }
            } else if gc == GENCMD_USE_JETPACK as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_JETPACK as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_JETPACK as c_int) != 0
                {
                    ItemUse_Jetpack(ctx, ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_JETPACK as c_int, 0);
                }
            } else if gc == GENCMD_USE_HEALTHDISP as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_HEALTHDISP as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_HEALTHDISP as c_int) != 0
                {
                    // ItemUse_UseDisp(ent, HI_HEALTHDISP);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_HEALTHDISP as c_int, 0);
                }
            } else if gc == GENCMD_USE_AMMODISP as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_AMMODISP as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_AMMODISP as c_int) != 0
                {
                    // ItemUse_UseDisp(ent, HI_AMMODISP);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_AMMODISP as c_int, 0);
                }
            } else if gc == GENCMD_USE_EWEB as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_EWEB as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_EWEB as c_int) != 0
                {
                    ItemUse_UseEWeb(ctx, ent);
                    G_AddEvent(ent, EV_USE_ITEM0 as c_int + HI_EWEB as c_int, 0);
                }
            } else if gc == GENCMD_USE_CLOAK as c_int {
                if (*client).ps.stats[STAT_HOLDABLE_ITEMS as usize] & (1 << HI_CLOAK as c_int) != 0
                    && G_ItemUsable(ctx, &mut (*client).ps, HI_CLOAK as c_int) != 0
                {
                    if (*client).ps.powerups[PW_CLOAKED as usize] != 0 {
                        // decloak
                        Jedi_Decloak(ctx, ent);
                    } else {
                        // cloak
                        Jedi_Cloak(ctx, ent);
                    }
                }
            } else if gc == GENCMD_SABERATTACKCYCLE as c_int {
                Cmd_SaberAttackCycle_f(ctx, ent);
            } else if gc == GENCMD_TAUNT as c_int {
                G_SetTauntAnim(ctx, ent, TAUNT_TAUNT);
            } else if gc == GENCMD_BOW as c_int {
                G_SetTauntAnim(ctx, ent, TAUNT_BOW);
            } else if gc == GENCMD_MEDITATE as c_int {
                G_SetTauntAnim(ctx, ent, TAUNT_MEDITATE);
            } else if gc == GENCMD_FLOURISH as c_int {
                G_SetTauntAnim(ctx, ent, TAUNT_FLOURISH);
            } else if gc == GENCMD_GLOAT as c_int {
                G_SetTauntAnim(ctx, ent, TAUNT_GLOAT);
            }
        }

        // save results of pmove
        if (*client).ps.eventSequence != oldEventSequence {
            (*ent).eventTime = (*ctx.world).level.time;
        }
        if (*ctx.world).cvars.g_smoothClients.integer != 0 {
            BG_PlayerStateToEntityStateExtraPolate(&mut (*client).ps, &mut (*ent).s, (*client).ps.commandTime, qfalse);
            // rww - 12-03-02 - Don't snap the origin of players! It screws prediction all up.
        } else {
            BG_PlayerStateToEntityState(&mut (*client).ps, &mut (*ent).s, qfalse);
        }

        if isNPC != qfalse {
            (*ent).s.eType = ET_NPC as c_int;
        }

        SendPendingPredictableEvents(ctx, &mut (*client).ps);

        if (*client).ps.eFlags & EF_FIRING == 0 {
            (*client).fireHeld = qfalse; // for grapple
        }

        // use the snapped origin for linking so it matches client predicted versions
        crate::q_math::_VectorCopy((*ent).s.pos.trBase, &mut (*ent).r.currentOrigin);

        let vehVehicle = (*ent).m_pVehicle as *mut Vehicle_t;
        if (*ent).s.eType != ET_NPC as c_int
            || (*ent).s.NPC_class != CLASS_VEHICLE as c_int
            || vehVehicle.is_null()
            || (*vehVehicle).m_iRemovedSurfaces == 0
        {
            // let vehicles that are getting broken apart do their own crazy sizing stuff
            crate::q_math::_VectorCopy(pm.mins, &mut (*ent).r.mins);
            crate::q_math::_VectorCopy(pm.maxs, &mut (*ent).r.maxs);
        }

        (*ent).waterlevel = pm.waterlevel;
        (*ent).watertype = pm.watertype;

        // execute client events
        ClientEvents(ctx, ent, oldEventSequence);

        if pm.useEvent != 0 {
            //TODO: Use
            // TryUse( ent );
        }
        if (*client).pers.cmd.buttons & BUTTON_USE != 0 && (*client).ps.useDelay < (*ctx.world).level.time {
            TryUse(ctx, ent);
            (*client).ps.useDelay = (*ctx.world).level.time + 100;
        }

        // link entity now, after any personal teleporters have been used
        trap::LinkEntity(ctx.engine, mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent));
        if (*client).noclip == qfalse {
            G_TouchTriggers(ctx, ent);
        }

        // NOTE: now copy the exact origin over otherwise clients can be snapped into solid
        crate::q_math::_VectorCopy((*client).ps.origin, &mut (*ent).r.currentOrigin);

        // touch other objects
        ClientImpacts(ctx, ent, &mut pm as *mut pmove_t);

        // save results of triggers and client events
        if (*client).ps.eventSequence != oldEventSequence {
            (*ent).eventTime = (*ctx.world).level.time;
        }

        // swap and latch button actions
        (*client).oldbuttons = (*client).buttons;
        (*client).buttons = (*ucmd).buttons;
        (*client).latched_buttons |= (*client).buttons & !(*client).oldbuttons;

        // Did we kick someone in our pmove sequence?
        if (*client).ps.forceKickFlip != 0 {
            let faceKicked =
                &mut (*ctx.world).entities[((*client).ps.forceKickFlip - 1) as usize] as *mut gentity_t;

            if !(*faceKicked).client.is_null()
                && (OnSameTeam(ctx, ent, faceKicked) == qfalse || (*ctx.world).cvars.g_friendlyFire.integer != 0)
                && ((*((*faceKicked).client as *mut gclient_t)).ps.duelInProgress == qfalse
                    || (*((*faceKicked).client as *mut gclient_t)).ps.duelIndex == (*ent).s.number)
                && ((*client).ps.duelInProgress == qfalse || (*client).ps.duelIndex == (*faceKicked).s.number)
            {
                let fkClient = (*faceKicked).client as *mut gclient_t;
                if !(*faceKicked).client.is_null() && (*faceKicked).health != 0 && (*faceKicked).takedamage != qfalse {
                    // push them away and do pain
                    let mut oppDir: vec3_t = [0.0; 3];
                    let mut strength = crate::q_math::VectorNormalize2((*client).ps.velocity, &mut oppDir) as c_int;

                    strength = ((strength as f64) * 0.05) as c_int;

                    crate::q_math::_VectorScale(oppDir, -1.0, &mut oppDir);

                    G_Damage(
                        ctx,
                        faceKicked,
                        ent,
                        ent,
                        Some(&mut oppDir),
                        (*client).ps.origin,
                        strength,
                        DAMAGE_NO_ARMOR,
                        MOD_MELEE as c_int,
                    );

                    if (*fkClient).ps.weapon != WP_SABER
                        || (*fkClient).ps.fd.saberAnimLevel != FORCE_LEVEL_3
                        || (BG_SaberInAttack((*fkClient).ps.saberMove) == qfalse
                            && PM_SaberInStart((*fkClient).ps.saberMove) == qfalse
                            && PM_SaberInReturn((*fkClient).ps.saberMove) == qfalse
                            && PM_SaberInTransition((*fkClient).ps.saberMove) == qfalse)
                    {
                        if (*faceKicked).health > 0
                            && (*fkClient).ps.stats[STAT_HEALTH as usize] > 0
                            && (*fkClient).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN as c_int
                        {
                            if BG_KnockDownable(&mut (*fkClient).ps) != qfalse
                                && (*ctx.world).bg_state.rng.Q_irand(1, 10) <= 3
                            {
                                // only actually knock over sometimes, but always do velocity hit
                                (*fkClient).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                                (*fkClient).ps.forceHandExtendTime = (*ctx.world).level.time + 1100;
                                (*fkClient).ps.forceDodgeAnim = 0; // this toggles between 1 and 0, when it's 1 we should play the get up anim
                            }

                            (*fkClient).ps.otherKiller = (*ent).s.number;
                            (*fkClient).ps.otherKillerTime = (*ctx.world).level.time + 5000;
                            (*fkClient).ps.otherKillerDebounceTime = (*ctx.world).level.time + 100;
                            (*fkClient).otherKillerMOD = MOD_MELEE as c_int;
                            (*fkClient).otherKillerVehWeapon = 0;
                            (*fkClient).otherKillerWeaponType = WP_NONE as c_int;

                            (*fkClient).ps.velocity[0] = oppDir[0] * (strength as f32 * 40.0);
                            (*fkClient).ps.velocity[1] = oppDir[1] * (strength as f32 * 40.0);
                            (*fkClient).ps.velocity[2] = 200.0;
                        }
                    }

                    G_Sound(
                        ctx,
                        faceKicked,
                        CHAN_AUTO as c_int,
                        G_SoundIndex(cstr(&format!("sound/weapons/melee/punch{}", (*ctx.world).bg_state.rng.Q_irand(1, 4))).as_ptr()),
                    );
                }
            }

            (*client).ps.forceKickFlip = 0;
        }

        // check for respawning
        if (*client).ps.stats[STAT_HEALTH as usize] <= 0
            && (*client).ps.eFlags2 & EF2_HELD_BY_MONSTER == 0 // can't respawn while being eaten
            && (*ent).s.eType != ET_NPC as c_int
        {
            // wait for the attack button to be pressed
            if (*ctx.world).level.time > (*client).respawnTime && (*ctx.world).globals.gDoSlowMoDuel == qfalse {
                // forcerespawn is to prevent users from waiting out powerups
                let mut forceRes = (*ctx.world).cvars.g_forcerespawn.integer;

                if (*ctx.world).cvars.g_gametype.integer == GT_POWERDUEL {
                    forceRes = 1;
                } else if (*ctx.world).cvars.g_gametype.integer == GT_SIEGE && (*ctx.world).cvars.g_siegeRespawn.integer != 0 {
                    // wave respawning on
                    forceRes = 1;
                }

                if forceRes > 0 && ((*ctx.world).level.time - (*client).respawnTime) > forceRes * 1000 {
                    respawn(ctx, ent);
                    return;
                }

                // pressing attack or use is the normal respawn method
                if (*ucmd).buttons & (BUTTON_ATTACK | BUTTON_USE_HOLDABLE) != 0 {
                    respawn(ctx, ent);
                }
            } else if (*ctx.world).globals.gDoSlowMoDuel != qfalse {
                (*client).respawnTime = (*ctx.world).level.time + 1000;
            }
            return;
        }

        // perform once-a-second actions
        ClientTimerActions(ent, msec);

        G_UpdateClientBroadcasts(ctx, ent);

        // try some idle anims on ent if getting no input and not moving for some time
        G_CheckClientIdle(ctx, ent, ucmd);

        // This code was moved here from clientThink to fix a problem with
        // g_synchronousClients being set to 1 when in vehicles.
        if (*ent).s.number < MAX_CLIENTS as c_int && (*client).ps.m_iVehicleNum != 0 {
            // driving a vehicle
            // run it
            let vehEnt = &mut (*ctx.world).entities[(*client).ps.m_iVehicleNum as usize] as *mut gentity_t;
            if (*vehEnt).inuse != qfalse && !(*vehEnt).client.is_null() {
                let vehVehicle = (*vehEnt).m_pVehicle as *mut Vehicle_t;
                ClientThink(ctx, (*client).ps.m_iVehicleNum, &mut (*vehVehicle).m_ucmd as *mut usercmd_t);
            } else {
                // vehicle no longer valid?
                (*client).ps.m_iVehicleNum = 0;
            }
        }
    }
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
