//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Jedi.c`.
//!
//! Pass-3 transcription: `ctx: GameContext` threads the ai_main globals
//! (`NPC`, `NPCInfo`, `ucmd`, `level`, `g_entities`) via `(*ctx.world)`, RNG
//! routes to the one `BgState.rng`, and stored `enemy`/`goalEntity`/
//! `activator`/`lastEnemy` fields are `Option<EntityId>`. See `PORT-NOTE`s
//! for the two open items:
//! the `jediSpeechDebounceTime` global is still a `()` placeholder in
//! `game_globals.rs`, and `BG_AnimLength` resolved as a `PmoveContext` method
//! with no game-tier receiver at its single call site.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Pass-2: constants this file needs that the prelude does not glob. `entity_event_t`
// (voice/entity events) and `animNumber_t` (anim ids) are `#[repr(i32)] enum`s —
// used as `<Type>::<VARIANT> as c_int` at the `c_int`-typed call sites. `FL_NOTARGET`
// is a `g_local.h` entity flag.
use crate::entity::flags::FL_NOTARGET;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::entity_event::entity_event_t;
use crate::saber::evasion_type_t::evasionType_t;
// Consts defined in sibling modules but not re-exported through the prelude glob;
// imported here so call sites keep the bare Raven spelling.
use crate::bg_slidemove::{SOLID_BMODEL, STEPSIZE};
use crate::NPC_AI_Stormtrooper::MIN_ROCKET_DIST_SQUARED;
// Explicit import to dedupe an E0659 glob ambiguity (known SFL_*/SVF_* debt);
// canonical path per crate::saber::saber_flags.
use crate::saber::saber_flags::SFL_NO_CARTWHEELS;
// Dedupe MASK_SHOT glob ambiguity (surface_flags::* / mp_qshared::shared::* both
// re-export it): canonical home is surface_flags per house convention.
use mp_qshared::shared::surface_flags::MASK_SHOT;

// Raven `rank_t` (anonymous enum, `ai.h:31-40`) — values pinned per file,
// matching `NPC_combat.rs`'s `RANK_CREWMAN`/`NPC_AI_Utils.rs`'s `RANK_ENSIGN`.
// Source: `oracle/oracle/codemp/game/ai.h:31-40`
pub const RANK_CIVILIAN: c_int = 0;
pub const RANK_CREWMAN: c_int = 1;
pub const RANK_ENSIGN: c_int = 2;
pub const RANK_LT_JG: c_int = 3;
pub const RANK_LT: c_int = 4;
pub const RANK_LT_COMM: c_int = 5;
pub const RANK_COMMANDER: c_int = 6;
pub const RANK_CAPTAIN: c_int = 7;

/// Raven `G_StartMatrixEffect`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:16-19`
pub fn G_StartMatrixEffect(
    ent: *mut gentity_t,
) {
    //perhaps write this at some point?
}

/// Raven `NPC_ShadowTrooper_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:103-108`
pub fn NPC_ShadowTrooper_Precache(ctx: GameContext<'_>) {
    crate::g_items::RegisterItem(ctx, crate::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_FORCE));
    crate::g_utils::G_SoundIndex(c"sound/chars/shadowtrooper/cloak.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/shadowtrooper/decloak.wav".as_ptr());
}

/// Raven `Jedi_ClearTimers`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:110-135`
pub fn Jedi_ClearTimers(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) {
    crate::g_timer::TIMER_Set(ctx, ent, c"roamTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"chatter".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"strafeLeft".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"strafeRight".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noStrafe".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"walking".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"taunting".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"parryTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"parryReCalcTime".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"forceJumpChasing".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"jumpChaseDebounce".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveforward".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveback".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"movenone".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveright".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"moveleft".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"movecenter".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"saberLevelDebounce".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noRetreat".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"holdLightning".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"gripping".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"draining".as_ptr(), 0);
    crate::g_timer::TIMER_Set(ctx, ent, c"noturn".as_ptr(), 0);
}

/// Raven `Jedi_PlayBlockedPushSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:137-148`
pub fn Jedi_PlayBlockedPushSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let level_time = (*ctx.world).level.time;
        if (*self_).s.number == 0 {
            crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, entity_event_t::EV_PUSHFAIL as c_int, 3000);
        } else {
            let npc = (*self_).NPC as *mut gNPC_t;
            if (*self_).health > 0 && !npc.is_null() && (*npc).blockedSpeechDebounceTime < level_time {
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, entity_event_t::EV_PUSHFAIL as c_int, 3000);
                (*npc).blockedSpeechDebounceTime = level_time + 3000;
            }
        }
    }
}

/// Raven `Jedi_PlayDeflectSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:150-161`
pub fn Jedi_PlayDeflectSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        // Q_irand is drawn inside each emitting branch (as in Raven) so the LCG
        // sequence matches: no draw occurs when nothing is emitted.
        let level_time = (*ctx.world).level.time;
        if (*self_).s.number == 0 {
            let ev = (*ctx.world).bg_state.rng.Q_irand(
                entity_event_t::EV_DEFLECT1 as c_int,
                entity_event_t::EV_DEFLECT3 as c_int,
            );
            crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, ev, 3000);
        } else {
            let npc = (*self_).NPC as *mut gNPC_t;
            if (*self_).health > 0 && !npc.is_null() && (*npc).blockedSpeechDebounceTime < level_time {
                let ev = (*ctx.world).bg_state.rng.Q_irand(
                    entity_event_t::EV_DEFLECT1 as c_int,
                    entity_event_t::EV_DEFLECT3 as c_int,
                );
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, ev, 3000);
                (*npc).blockedSpeechDebounceTime = level_time + 3000;
            }
        }
    }
}

/// Raven `NPC_Jedi_PlayConfusionSound`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:163-180`
pub fn NPC_Jedi_PlayConfusionSound(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        if (*self_).health > 0 {
            let client = (*self_).client as *mut gclient_t;
            if !client.is_null()
                && ((*client).NPC_class == CLASS_TAVION || (*client).NPC_class == CLASS_DESANN)
            {
                let ev = (*ctx.world).bg_state.rng.Q_irand(
                    entity_event_t::EV_CONFUSE1 as c_int,
                    entity_event_t::EV_CONFUSE3 as c_int,
                );
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, ev, 2000);
            } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) != 0 {
                let ev = (*ctx.world).bg_state.rng.Q_irand(
                    entity_event_t::EV_TAUNT1 as c_int,
                    entity_event_t::EV_TAUNT3 as c_int,
                );
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, ev, 2000);
            } else {
                let ev = (*ctx.world).bg_state.rng.Q_irand(
                    entity_event_t::EV_GLOAT1 as c_int,
                    entity_event_t::EV_GLOAT3 as c_int,
                );
                crate::NPC_sounds::G_AddVoiceEvent(ctx, self_, ev, 2000);
            }
        }
    }
}

/// Raven `Boba_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:182-189`
pub fn Boba_Precache(ctx: GameContext<'_>) {
    crate::g_utils::G_SoundIndex(c"sound/boba/jeton.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/boba/jethover.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/effects/combustfire.mp3".as_ptr());
    crate::g_utils::G_EffectIndex(c"boba/jet".as_ptr());
    crate::g_utils::G_EffectIndex(c"boba/fthrw".as_ptr());
}

/// Raven `Boba_ChangeWeapon`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:193-201`
pub fn Boba_ChangeWeapon(
    ctx: GameContext<'_>,
    wp: c_int,
) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        if (*npc).s.weapon == wp {
            return;
        }
        crate::NPC_combat::NPC_ChangeWeapon(wp);
        crate::g_utils::G_AddEvent(
            npc,
            entity_event_t::EV_GENERAL_SOUND as c_int,
            crate::g_utils::G_SoundIndex(c"sound/weapons/change.wav".as_ptr()),
        );
    }
}

/// Raven `WP_ResistForcePush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:203-270`
pub fn WP_ResistForcePush(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    noPenalty: qboolean,
) {
    unsafe {
        let world = ctx.world;
        let parts: c_int;
        let mut runningResist: qboolean = qfalse;

        if self_.is_null()
            || (*self_).health <= 0
            || (*self_).client.is_null()
            || pusher.is_null()
            || (*pusher).client.is_null()
        {
            return;
        }
        let client = (*self_).client as *mut gclient_t;
        if ((*self_).s.number == 0
            || (*client).NPC_class == CLASS_DESANN
            || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
            || (*client).NPC_class == CLASS_LUKE)
            && (crate::q_math::VectorLengthSquared((*client).ps.velocity) > 10000.0
                || (*client).ps.fd.forcePowerLevel[FP_PUSH as usize] >= FORCE_LEVEL_3
                || (*client).ps.fd.forcePowerLevel[FP_PULL as usize] >= FORCE_LEVEL_3)
        {
            runningResist = qtrue;
        }
        if runningResist == qfalse
            && (*client).ps.groundEntityNum != ENTITYNUM_NONE
            && crate::bg_panimate::BG_SpinningSaberAnim((*client).ps.legsAnim) == qfalse
            && crate::bg_panimate::BG_FlippingAnim((*client).ps.legsAnim) == qfalse
            && crate::bg_pmove::PM_RollingAnim((*client).ps.legsAnim) == qfalse
            && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
            && crate::bg_panimate::BG_CrouchAnim((*client).ps.legsAnim) == qfalse
        {
            //if on a surface and not in a spin or flip, play full body resist
            parts = SETANIM_BOTH;
        } else {
            //play resist just in torso
            parts = SETANIM_TORSO;
        }
        crate::npc_c::NPC_SetAnim(
            self_,
            parts,
            animNumber_t::BOTH_RESISTPUSH as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        if noPenalty == qfalse {
            let mut buf: [c_char; 128] = [0; 128];
            let tFVal: f32;

            crate::trap::Cvar_VariableStringBuffer(
                ctx.engine,
                mp_abi::game::syscalls::G_CVAR_VARIABLE_STRING_BUFFER::GCvarVariableStringBufferArgs::new(
                    cstr("timescale"),
                    buf.as_mut_ptr(),
                    buf.len() as c_int,
                ),
            );

            tFVal = crate::bg_lib::atof(buf.as_ptr()) as f32;

            if runningResist == qfalse {
                (*client).ps.velocity = [0.0, 0.0, 0.0];
                //still stop them from attacking or moving for a bit, though
                (*client).ps.weaponTime = 1000;
                if (*client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                    (*client).ps.weaponTime =
                        ((*client).ps.weaponTime as f32 * tFVal).floor() as c_int;
                }
                (*client).ps.pm_time = (*client).ps.weaponTime;
                (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
            } else {
                (*client).ps.weaponTime = 600;
                if (*client).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
                    (*client).ps.weaponTime =
                        ((*client).ps.weaponTime as f32 * tFVal).floor() as c_int;
                }
            }
        }
        //play my force push effect on my hand
        (*client).ps.powerups[PW_DISINT_4 as usize] =
            (*world).level.time + (*client).ps.torsoTimer + 500;
        (*client).ps.powerups[PW_PULL as usize] = 0;
        Jedi_PlayBlockedPushSound(ctx, self_);
    }
}

/// Raven `Boba_StopKnockdown`.
///
/// `pushDir` is read-only here (never written), so it stays by-value.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:272-343`
pub fn Boba_StopKnockdown(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    pusher: *mut gentity_t,
    pushDir: vec3_t,
    forceKnockdown: qboolean,
) -> qboolean {
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if (*client).NPC_class != CLASS_BOBAFETT {
            return qfalse;
        }

        if ((*client).ps.eFlags2 & EF2_FLYING) != 0 {
            //can't knock me down when I'm flying
            return qtrue;
        }

        let ang: vec3_t = [0.0, (*self_).r.currentAngles[YAW], 0.0];
        let strafeTime = (*ctx.world).bg_state.rng.Q_irand(1000, 2000);

        let mut fwd: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        crate::q_math::AngleVectors(ang, Some(&mut fwd), Some(&mut right), None);
        let mut pDir: vec3_t = [0.0; 3];
        crate::q_math::VectorNormalize2(pushDir, &mut pDir);
        let fDot = pDir[0] * fwd[0] + pDir[1] * fwd[1] + pDir[2] * fwd[2];
        let rDot = pDir[0] * right[0] + pDir[1] * right[1] + pDir[2] * right[2];

        if (*ctx.world).bg_state.rng.Q_irand(0, 2) != 0 {
            //flip or roll with it
            // C leaves tempCmd's other fields uninitialized (UB read in ForceJump);
            // zero-initialize as the one defined behavior (porting-rules §19).
            let mut tempCmd: usercmd_t = core::mem::zeroed();
            if fDot >= 0.4 {
                tempCmd.forwardmove = 127;
                crate::g_timer::TIMER_Set(ctx, self_, c"moveforward".as_ptr(), strafeTime);
            } else if fDot <= -0.4 {
                tempCmd.forwardmove = -127;
                crate::g_timer::TIMER_Set(ctx, self_, c"moveback".as_ptr(), strafeTime);
            } else if rDot > 0.0 {
                tempCmd.rightmove = 127;
                crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), strafeTime);
                crate::g_timer::TIMER_Set(ctx, self_, c"strafeLeft".as_ptr(), -1);
            } else {
                tempCmd.rightmove = -127;
                crate::g_timer::TIMER_Set(ctx, self_, c"strafeLeft".as_ptr(), strafeTime);
                crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), -1);
            }
            crate::g_utils::G_AddEvent(self_, entity_event_t::EV_JUMP as c_int, 0);
            if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 {
                //flip
                (*client).ps.fd.forceJumpCharge = 280.0; //FIXME: calc this intelligently?
                crate::w_force::ForceJump(ctx, self_, &mut tempCmd);
            } else {
                //roll
                crate::g_timer::TIMER_Set(ctx, self_, c"duck".as_ptr(), strafeTime);
            }
            (*self_).painDebounceTime = 0; //so we do something
        } else if (*ctx.world).bg_state.rng.Q_irand(0, 1) == 0 && forceKnockdown != qfalse {
            //resist
            WP_ResistForcePush(ctx, self_, pusher, qtrue);
        } else {
            //fall down
            return qfalse;
        }

        qtrue
    }
}

/// Raven `Boba_FlyStart`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:345-365`
pub fn Boba_FlyStart(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    //switch to seeker AI for a while
    unsafe {
        let world = ctx.world;
        if crate::g_timer::TIMER_Done(ctx, self_, c"jetRecharge".as_ptr()) != qfalse {
            let client = (*self_).client as *mut gclient_t;
            (*client).ps.gravity = 0;
            if !(*self_).NPC.is_null() {
                let snpc = (*self_).NPC as *mut gNPC_t;
                (*snpc).aiFlags |= NPCAI_CUSTOM_GRAVITY;
            }
            (*client).ps.eFlags2 |= EF2_FLYING; //moveType = MT_FLYSWIM;
            (*client).jetPackTime =
                (*world).level.time + (*world).bg_state.rng.Q_irand(3000, 10000);
            //take-off sound
            crate::g_utils::G_SoundOnEnt(ctx, self_, CHAN_ITEM as c_int, c"sound/boba/jeton.wav".as_ptr());
            //jet loop sound
            (*self_).s.loopSound = crate::g_utils::G_SoundIndex(c"sound/boba/jethover.wav".as_ptr());
            if !(*self_).NPC.is_null() {
                (*self_).count = Q3_INFINITE; // SEEKER shot ammo count
            }
        }
    }
}

/// Raven `Boba_FlyStop`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:367-384`
pub fn Boba_FlyStop(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let world = ctx.world;
        let client = (*self_).client as *mut gclient_t;
        (*client).ps.gravity = (*world).cvars.g_gravity.value as c_int;
        if !(*self_).NPC.is_null() {
            let snpc = (*self_).NPC as *mut gNPC_t;
            (*snpc).aiFlags &= !NPCAI_CUSTOM_GRAVITY;
        }
        (*client).ps.eFlags2 &= !EF2_FLYING;
        (*client).jetPackTime = 0;
        //stop jet loop sound
        (*self_).s.loopSound = 0;
        if !(*self_).NPC.is_null() {
            (*self_).count = 0; // SEEKER shot ammo count
            crate::g_timer::TIMER_Set(
                ctx,
                self_,
                c"jetRecharge".as_ptr(),
                (*world).bg_state.rng.Q_irand(1000, 5000),
            );
            crate::g_timer::TIMER_Set(
                ctx,
                self_,
                c"jumpChaseDebounce".as_ptr(),
                (*world).bg_state.rng.Q_irand(500, 2000),
            );
        }
    }
}

/// Raven `Boba_Flying`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:386-389`
pub fn Boba_Flying(
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if ((*client).ps.eFlags2 & EF2_FLYING) != 0 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `Boba_FireFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:391-416`
pub fn Boba_FireFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let world = ctx.world;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*self_).client as *mut gclient_t;
        let damage = (*world).bg_state.rng.Q_irand(20, 30);
        let mut tr: trace_t = core::mem::zeroed();
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let traceMins: vec3_t = [-4.0, -4.0, -4.0];
        let traceMaxs: vec3_t = [4.0, 4.0, 4.0];

        crate::trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                (*self_).ghoul2,
                0,
                (*client).renderInfo.handLBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &(*self_).r.currentAngles as *const vec3_t,
                &(*self_).r.currentOrigin as *const vec3_t,
                (*world).level.time,
                core::ptr::null_mut(),
                &(*self_).modelScale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut start);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut dir);
        //G_PlayEffect( "boba/fthrw", start, dir );
        crate::q_math::_VectorMA(start, 128.0, dir, &mut end);

        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &start as *const vec3_t,
                &traceMins as *const vec3_t,
                &traceMaxs as *const vec3_t,
                &end as *const vec3_t,
                (*self_).s.number,
                MASK_SHOT,
            ),
        );

        let traceEnt = ge.add(tr.entityNum as usize);
        if (tr.entityNum as c_int) < ENTITYNUM_WORLD && (*traceEnt).takedamage != qfalse {
            crate::g_combat::G_Damage(
                ctx,
                traceEnt,
                self_,
                self_,
                Some(&mut dir),
                tr.endpos,
                damage,
                (DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK | DAMAGE_IGNORE_TEAM) as c_int,
                MOD_LAVA as c_int,
            );
        }
    }
}

/// Raven `Boba_StartFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:419-469`
pub fn Boba_StartFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let client = (*self_).client as *mut gclient_t;
        let flameTime = 4000; //Q_irand( 1000, 3000 );
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        (*client).ps.torsoTimer = flameTime; //+1000;
        if !(*self_).NPC.is_null() {
            crate::g_timer::TIMER_Set(ctx, self_, c"nextAttackDelay".as_ptr(), flameTime);
            crate::g_timer::TIMER_Set(ctx, self_, c"walking".as_ptr(), 0);
        }
        crate::g_timer::TIMER_Set(ctx, self_, c"flameTime".as_ptr(), flameTime);
        crate::g_utils::G_SoundOnEnt(
            ctx,
            self_,
            CHAN_WEAPON as c_int,
            c"sound/effects/combustfire.mp3".as_ptr(),
        );

        let npc_client = (*npc).client as *mut gclient_t;
        crate::trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                (*npc).ghoul2,
                0,
                (*npc_client).renderInfo.handRBolt,
                &mut boltMatrix as *mut mdxaBone_t,
                &(*npc).r.currentAngles as *const vec3_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                (*world).level.time,
                core::ptr::null_mut(),
                &(*npc).modelScale as *const vec3_t,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_Y as c_int, &mut dir);

        crate::g_utils::G_PlayEffectID(
            crate::g_utils::G_EffectIndex(c"boba/fthrw".as_ptr()),
            org,
            dir,
        );
    }
}

/// Raven `Boba_DoFlameThrower`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:471-479`
pub fn Boba_DoFlameThrower(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        crate::npc_c::NPC_SetAnim(
            self_,
            SETANIM_TORSO,
            animNumber_t::BOTH_FORCELIGHTNING_HOLD as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        if crate::g_timer::TIMER_Done(ctx, self_, c"nextAttackDelay".as_ptr()) != qfalse
            && crate::g_timer::TIMER_Done(ctx, self_, c"flameTime".as_ptr()) != qfalse
        {
            Boba_StartFlameThrower(ctx, self_);
        }
        Boba_FireFlameThrower(ctx, self_);
    }
}

/// Raven `Boba_FireDecide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:481-797`
pub fn Boba_FireDecide(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        let mut enemyLOS: qboolean = qfalse;
        let mut enemyCS: qboolean = qfalse;
        let mut enemyInFOV: qboolean = qfalse;
        let mut faceEnemy: qboolean = qfalse;
        let mut shoot: qboolean = qfalse;
        let mut hitAlly: qboolean = qfalse;
        let mut impactPos: vec3_t = [0.0; 3];
        let enemyDist: f32;
        let dot: f32;
        let mut enemyDir: vec3_t = [0.0; 3];
        let mut shootDir: vec3_t = [0.0; 3];

        if (*client).ps.groundEntityNum == ENTITYNUM_NONE
            && (*client).ps.fd.forceJumpZStart != 0.0
            && crate::bg_panimate::BG_FlippingAnim((*client).ps.legsAnim) == qfalse
            && (*world).bg_state.rng.Q_irand(0, 10) == 0
        {
            //take off
            Boba_FlyStart(ctx, npc);
        }

        if (*npc).enemy.is_none() {
            return;
        }
        let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
        let enemy_client = (*enemy).client as *mut gclient_t;

        if (*enemy).s.weapon == WP_SABER as c_int {
            (*npc_info).scriptFlags &= !SCF_ALT_FIRE;
            Boba_ChangeWeapon(ctx, WP_ROCKET_LAUNCHER as c_int);
        } else {
            if ((*npc).health as f32) < (*client).pers.maxHealth as f32 * 0.5f32 {
                (*npc_info).scriptFlags |= SCF_ALT_FIRE;
                Boba_ChangeWeapon(ctx, WP_BLASTER as c_int);
                (*npc_info).burstMin = 3;
                (*npc_info).burstMean = 12;
                (*npc_info).burstMax = 20;
                (*npc_info).burstSpacing = (*world).bg_state.rng.Q_irand(300, 750); //attack debounce
            } else {
                (*npc_info).scriptFlags &= !SCF_ALT_FIRE;
                Boba_ChangeWeapon(ctx, WP_BLASTER as c_int);
            }
        }

        impactPos = [0.0, 0.0, 0.0];
        enemyDist = crate::q_math::DistanceSquared((*npc).r.currentOrigin, (*enemy).r.currentOrigin);

        crate::q_math::_VectorSubtract(
            (*enemy).r.currentOrigin,
            (*npc).r.currentOrigin,
            &mut enemyDir,
        );
        crate::q_math::VectorNormalize(&mut enemyDir);
        crate::q_math::AngleVectors((*client).ps.viewangles, Some(&mut shootDir), None, None);
        dot = crate::q_math::_DotProduct(enemyDir, shootDir);
        if dot > 0.5f32 || (enemyDist * (1.0f32 - dot)) < 10000.0 {
            //enemy is in front of me or they're very close and not behind me
            enemyInFOV = qtrue;
        }

        if (enemyDist < (128.0 * 128.0) && enemyInFOV != qfalse)
            || crate::g_timer::TIMER_Done(ctx, npc, c"flameTime".as_ptr()) == qfalse
        {
            //flamethrower
            Boba_DoFlameThrower(ctx, npc);
            enemyCS = qfalse;
            shoot = qfalse;
            (*npc_info).enemyLastSeenTime = (*world).level.time;
            faceEnemy = qtrue;
            (*world).globals.ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
        } else if enemyDist < MIN_ROCKET_DIST_SQUARED {
            //128
            //enemy within 128
            if ((*client).ps.weapon == WP_FLECHETTE as c_int
                || (*client).ps.weapon == WP_REPEATER as c_int)
                && ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0
            {
                //shooting an explosive, but enemy too close, switch to primary fire
                (*npc_info).scriptFlags &= !SCF_ALT_FIRE;
            }
        } else if enemyDist > 65536.0 {
            //256 squared
            if (*client).ps.weapon == WP_DISRUPTOR as c_int {
                //sniping... should be assumed
                if ((*npc_info).scriptFlags & SCF_ALT_FIRE) == 0 {
                    //use primary fire
                    (*npc_info).scriptFlags |= SCF_ALT_FIRE;
                    //reset fire-timing variables
                    crate::NPC_combat::NPC_ChangeWeapon(WP_DISRUPTOR as c_int);
                    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
        }

        //can we see our target?
        if crate::g_timer::TIMER_Done(ctx, npc, c"nextAttackDelay".as_ptr()) != qfalse
            && crate::g_timer::TIMER_Done(ctx, npc, c"flameTime".as_ptr()) != qfalse
        {
            if crate::NPC_utils::NPC_ClearLOS4(ctx, enemy) != qfalse {
                (*npc_info).enemyLastSeenTime = (*world).level.time;
                enemyLOS = qtrue;

                if (*client).ps.weapon == WP_NONE as c_int {
                    enemyCS = qfalse; //not true, but should stop us from firing
                } else {
                    //can we shoot our target?
                    if ((*client).ps.weapon == WP_ROCKET_LAUNCHER as c_int
                        || ((*client).ps.weapon == WP_FLECHETTE as c_int
                            && ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0))
                        && enemyDist < MIN_ROCKET_DIST_SQUARED
                    {
                        enemyCS = qfalse; //not true, but should stop us from firing
                        hitAlly = qtrue; //us!
                    } else if enemyInFOV != qfalse {
                        //if enemy is FOV, go ahead and check for shooting
                        let hit = crate::NPC_combat::NPC_ShotEntity(ctx, enemy, impactPos);
                        let hitEnt = ge.add(hit as usize);
                        let hitEnt_client = (*hitEnt).client as *mut gclient_t;

                        if hit == (*enemy).s.number
                            || (!hitEnt.is_null()
                                && !(*hitEnt).client.is_null()
                                && (*hitEnt_client).playerTeam == (*client).enemyTeam)
                            || (!hitEnt.is_null()
                                && (*hitEnt).takedamage != qfalse
                                && (((*hitEnt).r.svFlags & SVF_GLASS_BRUSH) != 0
                                    || (*hitEnt).health < 40
                                    || (*npc).s.weapon == WP_EMPLACED_GUN as c_int))
                        {
                            //can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                            enemyCS = qtrue;
                            crate::q_math::_VectorCopy(
                                (*enemy).r.currentOrigin,
                                &mut (*npc_info).enemyLastSeenLocation,
                            );
                        } else {
                            //Hmm, have to get around this bastard
                            if !hitEnt.is_null()
                                && !(*hitEnt).client.is_null()
                                && (*hitEnt_client).playerTeam == (*client).playerTeam
                            {
                                //would hit an ally, don't fire!!!
                                hitAlly = qtrue;
                            }
                        }
                    } else {
                        enemyCS = qfalse; //not true, but should stop us from firing
                    }
                }
            } else if crate::trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &(*enemy).r.currentOrigin as *const vec3_t,
                    &(*npc).r.currentOrigin as *const vec3_t,
                ),
            ) != 0 {
                (*npc_info).enemyLastSeenTime = (*world).level.time;
                faceEnemy = qtrue;
            }

            if (*client).ps.weapon == WP_NONE as c_int {
                faceEnemy = qfalse;
                shoot = qfalse;
            } else {
                if enemyLOS != qfalse {
                    faceEnemy = qtrue;
                }
                if enemyCS != qfalse {
                    shoot = qtrue;
                }
            }

            if enemyCS == qfalse {
                //if have a clear shot, always try
                if hitAlly == qfalse //we're not going to hit an ally
                    && enemyInFOV != qfalse //enemy is in our FOV
                    && (*npc_info).enemyLastSeenTime > 0
                {
                    if (*world).level.time - (*npc_info).enemyLastSeenTime < 10000 {
                        if (*world).bg_state.rng.Q_irand(0, 10) == 0 {
                            //Fire on the last known position
                            let mut muzzle: vec3_t = [0.0; 3];
                            let mut dir: vec3_t = [0.0; 3];
                            let mut angles: vec3_t = [0.0; 3];
                            let mut tooClose: qboolean = qfalse;
                            let mut tooFar: qboolean = qfalse;
                            let mut distThreshold: f32;
                            let mut dist: f32;

                            crate::NPC_utils::CalcEntitySpot(ctx, npc, SPOT_HEAD, &mut muzzle);
                            if crate::q_math::VectorCompare(impactPos, vec3_origin) != qfalse {
                                //never checked ShotEntity this frame, so must do a trace...
                                let mut tr: trace_t = core::mem::zeroed();
                                let mut forward: vec3_t = [0.0; 3];
                                let mut end: vec3_t = [0.0; 3];
                                crate::q_math::AngleVectors(
                                    (*client).ps.viewangles,
                                    Some(&mut forward),
                                    None,
                                    None,
                                );
                                crate::q_math::_VectorMA(muzzle, 8192.0, forward, &mut end);
                                crate::trap::Trace(
                                    ctx.engine,
                                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                        &mut tr as *mut trace_t,
                                        &muzzle as *const vec3_t,
                                        &vec3_origin as *const vec3_t,
                                        &vec3_origin as *const vec3_t,
                                        &end as *const vec3_t,
                                        (*npc).s.number,
                                        MASK_SHOT,
                                    ),
                                );
                                crate::q_math::_VectorCopy(tr.endpos, &mut impactPos);
                            }

                            //see if impact would be too close to me
                            distThreshold = 16384.0; //128*128 default
                            match (*npc).s.weapon {
                                w if w == WP_ROCKET_LAUNCHER as c_int
                                    || w == WP_FLECHETTE as c_int
                                    || w == WP_THERMAL as c_int
                                    || w == WP_TRIP_MINE as c_int
                                    || w == WP_DET_PACK as c_int =>
                                {
                                    distThreshold = 65536.0; //256*256
                                }
                                w if w == WP_REPEATER as c_int => {
                                    if ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0 {
                                        distThreshold = 65536.0; //256*256
                                    }
                                }
                                _ => {}
                            }

                            dist = crate::q_math::DistanceSquared(impactPos, muzzle);

                            if dist < distThreshold {
                                //impact would be too close to me
                                tooClose = qtrue;
                            } else if (*world).level.time - (*npc_info).enemyLastSeenTime > 5000
                                || (!(*npc_info).group.is_null()
                                    && (*world).level.time
                                        - (*(*npc_info).group).lastSeenEnemyTime
                                        > 5000)
                            {
                                //we've haven't seen them in the last 5 seconds
                                distThreshold = 65536.0; //256*256 default
                                match (*npc).s.weapon {
                                    w if w == WP_ROCKET_LAUNCHER as c_int
                                        || w == WP_FLECHETTE as c_int
                                        || w == WP_THERMAL as c_int
                                        || w == WP_TRIP_MINE as c_int
                                        || w == WP_DET_PACK as c_int =>
                                    {
                                        distThreshold = 262144.0; //512*512
                                    }
                                    w if w == WP_REPEATER as c_int => {
                                        if ((*npc_info).scriptFlags & SCF_ALT_FIRE) != 0 {
                                            distThreshold = 262144.0; //512*512
                                        }
                                    }
                                    _ => {}
                                }
                                dist = crate::q_math::DistanceSquared(
                                    impactPos,
                                    (*npc_info).enemyLastSeenLocation,
                                );
                                if dist > distThreshold {
                                    //impact would be too far from enemy
                                    tooFar = qtrue;
                                }
                            }

                            if tooClose == qfalse && tooFar == qfalse {
                                //okay too shoot at last pos
                                crate::q_math::_VectorSubtract(
                                    (*npc_info).enemyLastSeenLocation,
                                    muzzle,
                                    &mut dir,
                                );
                                crate::q_math::VectorNormalize(&mut dir);
                                crate::q_math::vectoangles(dir, &mut angles);

                                (*npc_info).desiredYaw = angles[YAW as usize];
                                (*npc_info).desiredPitch = angles[PITCH as usize];

                                shoot = qtrue;
                                faceEnemy = qfalse;
                            }
                        }
                    }
                }
            }

            //FIXME: don't shoot right away!
            if (*client).ps.weaponTime > 0 {
                if (*npc).s.weapon == WP_ROCKET_LAUNCHER as c_int {
                    if enemyLOS == qfalse || enemyCS == qfalse {
                        //cancel it
                        (*client).ps.weaponTime = 0;
                    } else {
                        //delay our next attempt
                        crate::g_timer::TIMER_Set(
                            ctx,
                            npc,
                            c"nextAttackDelay".as_ptr(),
                            (*world).bg_state.rng.Q_irand(500, 1000),
                        );
                    }
                }
            } else if shoot != qfalse {
                //try to shoot if it's time
                if crate::g_timer::TIMER_Done(ctx, npc, c"nextAttackDelay".as_ptr()) != qfalse {
                    if ((*npc_info).scriptFlags & SCF_FIRE_WEAPON) == 0 {
                        // we've already fired, no need to do it again here
                        crate::NPC_combat::WeaponThink(ctx, qtrue);
                    }
                    //NASTY
                    if (*npc).s.weapon == WP_ROCKET_LAUNCHER as c_int
                        && ((*world).globals.ucmd.buttons & BUTTON_ATTACK) != 0
                        && (*world).bg_state.rng.Q_irand(0, 3) == 0
                    {
                        //every now and then, shoot a homing rocket
                        (*world).globals.ucmd.buttons &= !BUTTON_ATTACK;
                        (*world).globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
                        (*client).ps.weaponTime = (*world).bg_state.rng.Q_irand(500, 1500);
                    }
                }
            }
        }
        let _ = (faceEnemy, enemy_client);
    }
}

/// Raven `Jedi_Cloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:799-816`
pub fn Jedi_Cloak(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        if !self_.is_null() {
            (*self_).flags |= FL_NOTARGET;
            if !(*self_).client.is_null() {
                let client = (*self_).client as *mut gclient_t;
                if (*client).ps.powerups[PW_CLOAKED as usize] == 0 {
                    //cloak
                    (*client).ps.powerups[PW_CLOAKED as usize] = Q3_INFINITE;

                    crate::g_utils::G_Sound(
                        ctx,
                        self_,
                        CHAN_ITEM as c_int,
                        crate::g_utils::G_SoundIndex(c"sound/chars/shadowtrooper/cloak.wav".as_ptr()),
                    );
                }
            }
        }
    }
}

/// Raven `Jedi_Decloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:818-833`
pub fn Jedi_Decloak(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        if !self_.is_null() {
            (*self_).flags &= !FL_NOTARGET;
            let client = (*self_).client as *mut gclient_t;
            if !client.is_null() {
                if (*client).ps.powerups[PW_CLOAKED as usize] != 0 {
                    //Uncloak
                    (*client).ps.powerups[PW_CLOAKED as usize] = 0;

                    crate::g_utils::G_Sound(
                        ctx,
                        self_,
                        CHAN_ITEM as c_int,
                        crate::g_utils::G_SoundIndex(
                            c"sound/chars/shadowtrooper/decloak.wav".as_ptr(),
                        ),
                    );
                }
            }
        }
    }
}

/// Raven `Jedi_CheckCloak`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:835-857`
pub fn Jedi_CheckCloak(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        if npc.is_null() {
            return;
        }
        let client = (*npc).client as *mut gclient_t;
        if !(*npc).client.is_null() && (*client).NPC_class == CLASS_SHADOWTROOPER {
            if (*client).ps.saberHolstered == 0
                || (*npc).health <= 0
                || (*client).ps.saberInFlight != qfalse
                || (*npc).painDebounceTime > (*world).level.time
            {
                //can't be cloaked if saber is on, or dead or saber in flight or taking pain or being gripped
                Jedi_Decloak(ctx, npc);
            } else if (*npc).health > 0
                && (*client).ps.saberInFlight == qfalse
                && (*npc).painDebounceTime < (*world).level.time
            {
                //still alive, have saber in hand, not taking pain and not being gripped
                Jedi_Cloak(ctx, npc);
            }
        }
    }
}

/// Raven `Jedi_Aggression`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:863-898`
pub fn Jedi_Aggression(
    self_: *mut gentity_t,
    change: c_int,
) {
    unsafe {
        let upper_threshold: c_int;
        let lower_threshold: c_int;
        let snpc = (*self_).NPC as *mut gNPC_t;
        let client = (*self_).client as *mut gclient_t;

        (*snpc).stats.aggression += change;

        //FIXME: base this on initial NPC stats
        if (*client).playerTeam == NPCTEAM_PLAYER as c_int {
            //good guys are less aggressive
            upper_threshold = 7;
            lower_threshold = 1;
        } else {
            //bad guys are more aggressive
            if (*client).NPC_class == CLASS_DESANN {
                upper_threshold = 20;
                lower_threshold = 5;
            } else {
                upper_threshold = 10;
                lower_threshold = 3;
            }
        }

        if (*snpc).stats.aggression > upper_threshold {
            (*snpc).stats.aggression = upper_threshold;
        } else if (*snpc).stats.aggression < lower_threshold {
            (*snpc).stats.aggression = lower_threshold;
        }
    }
}

/// Raven `Jedi_AggressionErosion`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:900-912`
pub fn Jedi_AggressionErosion(
    ctx: GameContext<'_>,
    amt: c_int,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let client = (*npc).client as *mut gclient_t;
        if crate::g_timer::TIMER_Done(ctx, npc, c"roamTime".as_ptr()) != qfalse {
            //the longer we're not alerted and have no enemy, the more our aggression goes down
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"roamTime".as_ptr(),
                (*world).bg_state.rng.Q_irand(2000, 5000),
            );
            Jedi_Aggression(npc, amt);
        }

        if (*npc_info).stats.aggression < 4
            || ((*npc_info).stats.aggression < 6 && (*client).NPC_class == CLASS_DESANN)
        {
            //turn off the saber
            crate::w_saber::WP_DeactivateSaber(ctx, npc, qfalse);
        }
    }
}

/// Raven `NPC_Jedi_RateNewEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:914-950`
pub fn NPC_Jedi_RateNewEnemy(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    enemy: *mut gentity_t,
) {
    let healthAggression: f32;
    let weaponAggression: f32;
    let newAggression: c_int;

    unsafe {
        match (*enemy).s.weapon {
            w if w == WP_SABER as c_int => {
                healthAggression = (*self_).health as f32 / 200.0 * 6.0;
                weaponAggression = 7.0; //go after him
            }
            w if w == WP_BLASTER as c_int => {
                // DistanceSquared( self->r.currentOrigin, enemy->r.currentOrigin )
                let s = (*self_).r.currentOrigin;
                let e = (*enemy).r.currentOrigin;
                let v0 = e[0] - s[0];
                let v1 = e[1] - s[1];
                let v2 = e[2] - s[2];
                if v0 * v0 + v1 * v1 + v2 * v2 < 65536.0
                //256 squared
                {
                    healthAggression = (*self_).health as f32 / 200.0 * 8.0;
                    weaponAggression = 8.0; //go after him
                } else {
                    healthAggression = 8.0 - ((*self_).health as f32 / 200.0 * 8.0);
                    weaponAggression = 2.0; //hang back for a second
                }
            }
            _ => {
                healthAggression = (*self_).health as f32 / 200.0 * 8.0;
                weaponAggression = 6.0; //approach
            }
        }
        //Average these with current aggression
        newAggression =
            ((healthAggression + weaponAggression + (*((*self_).NPC as *mut gNPC_t)).stats.aggression as f32) / 3.0)
                .ceil() as c_int;
        Jedi_Aggression(self_, newAggression - (*((*self_).NPC as *mut gNPC_t)).stats.aggression);

        //don't taunt right away
        crate::g_timer::TIMER_Set(ctx, self_, c"chatter".as_ptr(), (*ctx.world).bg_state.rng.Q_irand(4000, 7000));
    }
}

/// Raven `Jedi_Rage`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:952-964`
pub fn Jedi_Rage(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        Jedi_Aggression(
            npc,
            10 - (*npc_info).stats.aggression + (*world).bg_state.rng.Q_irand(-2, 2),
        );
        crate::g_timer::TIMER_Set(ctx, npc, c"roamTime".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"chatter".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"walking".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"jumpChaseDebounce".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"movenone".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"movecenter".as_ptr(), 0);
        crate::g_timer::TIMER_Set(ctx, npc, c"noturn".as_ptr(), 0);
        crate::w_force::ForceRage(ctx, npc);
    }
}

/// Raven `Jedi_RageStop`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:966-973`
pub fn Jedi_RageStop(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        if !(*self_).NPC.is_null() {
            //calm down and back off
            crate::g_timer::TIMER_Set(ctx, self_, c"roamTime".as_ptr(), 0);
            Jedi_Aggression(self_, (*ctx.world).bg_state.rng.Q_irand(-5, 0));
        }
    }
}

/// Raven `Jedi_BattleTaunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:980-1013`
pub fn Jedi_BattleTaunt(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        // PORT-NOTE(jediSpeechDebounceTime): field is a `()` placeholder in
        // game_globals.rs; needs porting to `[c_int; TEAM_NUM_TEAMS]`.
        if crate::g_timer::TIMER_Done(ctx, npc, c"chatter".as_ptr()) != qfalse
            && (*world).bg_state.rng.Q_irand(0, 3) == 0
            && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
            && (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize]
                < (*world).level.time
        {
            let mut event: c_int = -1;
            let enemy_opt = (*npc).enemy;
            let enemy: *mut gentity_t = match enemy_opt {
                Some(id) => ge.add(id.0 as usize),
                None => core::ptr::null_mut(),
            };
            let enemy_client = if enemy.is_null() {
                core::ptr::null_mut()
            } else {
                (*enemy).client as *mut gclient_t
            };
            if (*client).playerTeam == NPCTEAM_PLAYER as c_int
                && !enemy.is_null()
                && !(*enemy).client.is_null()
                && (*enemy_client).NPC_class == CLASS_JEDI
            {
                //a jedi fighting a jedi - training
                if (*client).NPC_class == CLASS_JEDI && (*npc_info).rank == RANK_COMMANDER as c_int {
                    //only trainer taunts
                    event = entity_event_t::EV_TAUNT1 as c_int;
                }
            } else {
                //reborn or a jedi fighting an enemy
                event = (*world).bg_state.rng.Q_irand(
                    entity_event_t::EV_TAUNT1 as c_int,
                    entity_event_t::EV_TAUNT3 as c_int,
                );
            }
            if event != -1 {
                crate::NPC_sounds::G_AddVoiceEvent(ctx, npc, event, 3000);
                (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 6000;
                (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                    (*world).level.time + 6000;
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"chatter".as_ptr(),
                    (*world).bg_state.rng.Q_irand(5000, 10000),
                );
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `Jedi_ClearPathToSpot`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1020-1077`
pub fn Jedi_ClearPathToSpot(
    ctx: GameContext<'_>,
    dest: vec3_t,
    impactEntNum: c_int,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let mut trace: trace_t = core::mem::zeroed();
        let mut start: vec3_t = [0.0; 3];
        let mut end: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let dist: f32;
        let drop: f32;
        let mut i: f32;

        //Offset the step height
        let mins: vec3_t = [(*npc).r.mins[0], (*npc).r.mins[1], (*npc).r.mins[2] + STEPSIZE];

        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                &mins as *const vec3_t,
                &(*npc).r.maxs as *const vec3_t,
                &dest as *const vec3_t,
                (*npc).s.number,
                (*npc).clipmask,
            ),
        );

        //Do a simple check
        if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
            //inside solid
            return qfalse;
        }

        if trace.fraction < 1.0f32 {
            //hit something
            if impactEntNum != ENTITYNUM_NONE && trace.entityNum == (impactEntNum) as i16 {
                //hit what we're going after
                return qtrue;
            } else {
                return qfalse;
            }
        }

        //otherwise, clear path in a straight line.
        crate::q_math::_VectorSubtract(dest, (*npc).r.currentOrigin, &mut dir);
        dist = crate::q_math::VectorNormalize(&mut dir);
        if dest[2] > (*npc).r.currentOrigin[2] {
            //going up, check for steps
            drop = STEPSIZE;
        } else {
            //going down or level, check for moderate drops
            drop = 64.0;
        }
        i = (*npc).r.maxs[0] * 2.0;
        while i < dist {
            crate::q_math::_VectorMA((*npc).r.currentOrigin, i, dir, &mut start);
            crate::q_math::_VectorCopy(start, &mut end);
            end[2] -= drop;
            crate::trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &start as *const vec3_t,
                    &mins as *const vec3_t,
                    &(*npc).r.maxs as *const vec3_t,
                    &end as *const vec3_t,
                    (*npc).s.number,
                    (*npc).clipmask,
                ),
            );
            if trace.fraction < 1.0f32 || trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
                //good to go
                i += (*npc).r.maxs[0] * 2.0;
                continue;
            }
            //no floor here! (or a long drop?)
            return qfalse;
        }
        //we made it!
        qtrue
    }
}

/// Raven `NPC_MoveDirClear`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1079-1193`
pub fn NPC_MoveDirClear(
    ctx: GameContext<'_>,
    forwardmove: c_int,
    rightmove: c_int,
    reset: qboolean,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        let mut testPos: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let fwdDist: f32;
        let rtDist: f32;
        let mut bottom_max: f32 = -STEPSIZE * 4.0 - 1.0;

        if forwardmove == 0 && rightmove == 0 {
            //not even moving
            return qtrue;
        }

        if (*world).globals.ucmd.upmove > 0 || (*client).ps.fd.forceJumpCharge != 0.0 {
            //Going to jump
            return qtrue;
        }

        if (*client).ps.groundEntityNum == ENTITYNUM_NONE {
            //in the air
            return qtrue;
        }

        let mut mins: vec3_t = [0.0; 3];
        crate::q_math::_VectorCopy((*npc).r.mins, &mut mins);
        mins[2] += STEPSIZE;
        angles[PITCH as usize] = 0.0;
        angles[ROLL as usize] = 0.0;
        angles[YAW as usize] = (*client).ps.viewangles[YAW as usize];
        crate::q_math::AngleVectors(angles, Some(&mut forward), Some(&mut right), None);
        fwdDist = (forwardmove as f32) / 2.0f32;
        rtDist = (rightmove as f32) / 2.0f32;
        crate::q_math::_VectorMA((*npc).r.currentOrigin, fwdDist, forward, &mut testPos);
        let testPos_in = testPos;
        crate::q_math::_VectorMA(testPos_in, rtDist, right, &mut testPos);
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &(*npc).r.currentOrigin as *const vec3_t,
                &mins as *const vec3_t,
                &(*npc).r.maxs as *const vec3_t,
                &testPos as *const vec3_t,
                (*npc).s.number,
                (*npc).clipmask | CONTENTS_BOTCLIP,
            ),
        );
        if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
            //hmm, trace started inside this brush
            if reset != qfalse {
                trace.fraction = 1.0f32;
            }
            crate::q_math::_VectorCopy(testPos, &mut trace.endpos);
        }
        if trace.fraction < 0.6 {
            //Going to bump into something very close, don't move, just turn
            let enemy: *mut gentity_t = match (*npc).enemy {
                Some(id) => ge.add(id.0 as usize),
                None => core::ptr::null_mut(),
            };
            let goal: *mut gentity_t = match (*npc_info).goalEntity {
                Some(id) => ge.add(id.0 as usize),
                None => core::ptr::null_mut(),
            };
            if (!enemy.is_null() && trace.entityNum == ((*enemy).s.number) as i16)
                || (!goal.is_null() && trace.entityNum == ((*goal).s.number) as i16)
            {
                //okay to bump into enemy or goal
                return qtrue;
            } else if reset != qfalse {
                //actually want to screw with the ucmd
                (*world).globals.ucmd.forwardmove = 0;
                (*world).globals.ucmd.rightmove = 0;
                (*client).ps.moveDir = [0.0, 0.0, 0.0];
            }
            return qfalse;
        }

        if (*npc_info).goalEntity.is_some() {
            let goal: *mut gentity_t = ge.add((*npc_info).goalEntity.unwrap().0 as usize);
            if (*goal).r.currentOrigin[2] < (*npc).r.currentOrigin[2] {
                //goal is below me, okay to step off at least that far plus stepheight
                bottom_max += (*goal).r.currentOrigin[2] - (*npc).r.currentOrigin[2];
            }
        }
        crate::q_math::_VectorCopy(trace.endpos, &mut testPos);
        testPos[2] += bottom_max;

        let trace_endpos = trace.endpos;
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace as *mut trace_t,
                &trace_endpos as *const vec3_t,
                &mins as *const vec3_t,
                &(*npc).r.maxs as *const vec3_t,
                &testPos as *const vec3_t,
                (*npc).s.number,
                (*npc).clipmask,
            ),
        );

        if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
            //Not going off a cliff
            return qtrue;
        }

        if trace.fraction < 1.0 {
            //Not going off a cliff
            return qtrue;
        }

        //going to fall at least bottom_max, don't move, just turn
        if reset != qfalse {
            //actually want to screw with the ucmd
            (*world).globals.ucmd.forwardmove =
                ((*world).globals.ucmd.forwardmove as f32 * -1.0) as c_schar;
            (*world).globals.ucmd.rightmove =
                ((*world).globals.ucmd.rightmove as f32 * -1.0) as c_schar;
            let md = (*client).ps.moveDir;
            crate::q_math::_VectorScale(md, -1.0, &mut (*client).ps.moveDir);
        }
        qfalse
    }
}

/// Raven `Jedi_HoldPosition`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1200-1211`
pub fn Jedi_HoldPosition(ctx: GameContext<'_>) {
    unsafe {
        let npc_info = (*ctx.world).globals.NPCInfo;
        (*npc_info).goalEntity = None;
    }
}

/// Raven `Jedi_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1219-1251`
pub fn Jedi_Move(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
    retreat: qboolean,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let moved: qboolean;
        let mut info: navInfo_t = core::mem::zeroed();

        (*npc_info).combatMove = qtrue;
        (*npc_info).goalEntity = ent_id_opt(ge, goal);

        moved = crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);

        //FIXME: temp retreat behavior
        if retreat != qfalse {
            (*world).globals.ucmd.forwardmove =
                ((*world).globals.ucmd.forwardmove as i32 * -1) as c_schar;
            (*world).globals.ucmd.rightmove =
                ((*world).globals.ucmd.rightmove as i32 * -1) as c_schar;
            let md = (*client).ps.moveDir;
            crate::q_math::_VectorScale(md, -1.0, &mut (*client).ps.moveDir);
        }

        //Get the move info
        crate::NPC_move::NAV_GetLastMove(ctx, &mut info);

        //If we hit our target, then stop and fire!
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        if (info.flags & NIF_COLLISION) != 0 && info.blocker == enemy {
            Jedi_HoldPosition(ctx);
        }

        //If our move failed, then reset
        if moved == qfalse {
            Jedi_HoldPosition(ctx);
        }
    }
}

/// Raven `Jedi_Hunt`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1253-1280`
pub fn Jedi_Hunt(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        //if we're at all interested in fighting, go after him
        if (*npc_info).stats.aggression > 1 {
            //approach enemy
            (*npc_info).combatMove = qtrue;
            if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) == 0 {
                crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                return qtrue;
            } else {
                if (*npc_info).goalEntity.is_none() {
                    //hunt
                    (*npc_info).goalEntity = (*npc).enemy;
                }
                if crate::NPC_move::NPC_MoveToGoal(ctx, qfalse) != qfalse {
                    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return qtrue;
                }
            }
        }
        qfalse
    }
}

/// Raven `Jedi_Retreat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1300-1310`
pub fn Jedi_Retreat(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let ge = (*world).g_entities.as_mut_ptr();
        if crate::g_timer::TIMER_Done(ctx, npc, c"noRetreat".as_ptr()) == qfalse {
            //don't actually move
            return;
        }
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        Jedi_Move(ctx, enemy, qtrue);
    }
}

/// Raven `Jedi_Advance`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1312-1325`
pub fn Jedi_Advance(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        if (*client).ps.saberInFlight == qfalse {
            crate::w_saber::WP_ActivateSaber(ctx, npc);
        }
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        Jedi_Move(ctx, enemy, qfalse);
    }
}

/// Raven `Jedi_AdjustSaberAnimLevel`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1327-1394`
pub fn Jedi_AdjustSaberAnimLevel(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    newLevel: c_int,
) {
    unsafe {
        let world = ctx.world;
        if self_.is_null() || (*self_).client.is_null() {
            return;
        }
        let client = (*self_).client as *mut gclient_t;
        let snpc = (*self_).NPC as *mut gNPC_t;
        if (*client).NPC_class == CLASS_TAVION {
            //special attacks
            (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_5;
            return;
        } else if (*client).NPC_class == CLASS_DESANN {
            //special attacks
            (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_4;
            return;
        }
        if (*client).playerTeam == NPCTEAM_ENEMY as c_int {
            if (*snpc).rank == RANK_CIVILIAN as c_int || (*snpc).rank == RANK_LT_JG as c_int {
                //grunt and fencer always uses quick attacks
                (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
                return;
            }
            if (*snpc).rank == RANK_CREWMAN as c_int || (*snpc).rank == RANK_ENSIGN as c_int {
                //acrobat & force-users always use medium attacks
                (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_2;
                return;
            }
        }
        //use the different attacks
        if newLevel > (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] {
            //cap it
            (*client).ps.fd.saberAnimLevel =
                (*client).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize];
        } else if newLevel < FORCE_LEVEL_1 {
            (*client).ps.fd.saberAnimLevel = FORCE_LEVEL_1;
        } else {
            //go ahead and set it
            (*client).ps.fd.saberAnimLevel = newLevel;
        }

        if (*world).cvars.d_JediAI.integer != 0 {
            let ty = cstr_to_str((*self_).NPC_type);
            if (*client).ps.fd.saberAnimLevel == FORCE_LEVEL_1 {
                crate::g_main::Com_Printf(cstr(&format!("^2{} Saber Attack Set: fast\n", ty)).as_ptr());
            } else if (*client).ps.fd.saberAnimLevel == FORCE_LEVEL_2 {
                crate::g_main::Com_Printf(cstr(&format!("^3{} Saber Attack Set: medium\n", ty)).as_ptr());
            } else if (*client).ps.fd.saberAnimLevel == FORCE_LEVEL_3 {
                crate::g_main::Com_Printf(cstr(&format!("^1{} Saber Attack Set: strong\n", ty)).as_ptr());
            }
        }
    }
}

/// Raven `Jedi_CheckDecreaseSaberAnimLevel`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1396-1411`
pub fn Jedi_CheckDecreaseSaberAnimLevel(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let client = (*npc).client as *mut gclient_t;
        if (*client).ps.weaponTime == 0
            && ((*world).globals.ucmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) == 0
        {
            //not attacking
            if crate::g_timer::TIMER_Done(ctx, npc, c"saberLevelDebounce".as_ptr()) != qfalse
                && (*world).bg_state.rng.Q_irand(0, 10) == 0
            {
                Jedi_AdjustSaberAnimLevel(
                    ctx,
                    npc,
                    (*world).bg_state.rng.Q_irand(FORCE_LEVEL_1, FORCE_LEVEL_3),
                );
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"saberLevelDebounce".as_ptr(),
                    (*world).bg_state.rng.Q_irand(3000, 10000),
                );
            }
        } else {
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"saberLevelDebounce".as_ptr(),
                (*world).bg_state.rng.Q_irand(1000, 5000),
            );
        }
    }
}

/// Raven `Jedi_CombatDistance`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1413-1874`
pub fn Jedi_CombatDistance(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };

        if (*client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0
            && (*client).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
        {
            //when gripping, don't move
            return;
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"gripping".as_ptr()) == qfalse {
            //stopped gripping, clear timers just in case
            crate::g_timer::TIMER_Set(ctx, npc, c"gripping".as_ptr(), -(*world).level.time);
            crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), (*world).bg_state.rng.Q_irand(0, 1000));
        }

        if (*client).ps.fd.forcePowersActive & (1 << FP_DRAIN) != 0
            && (*client).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_1
        {
            //when draining, don't move
            return;
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"draining".as_ptr()) == qfalse {
            //stopped draining, clear timers just in case
            crate::g_timer::TIMER_Set(ctx, npc, c"draining".as_ptr(), -(*world).level.time);
            crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), (*world).bg_state.rng.Q_irand(0, 1000));
        }

        if (*client).NPC_class == CLASS_BOBAFETT {
            if crate::g_timer::TIMER_Done(ctx, npc, c"flameTime".as_ptr()) == qfalse {
                if enemy_dist > 50 {
                    Jedi_Advance(ctx);
                } else if enemy_dist <= 0 {
                    Jedi_Retreat(ctx);
                }
            } else if enemy_dist < 200 {
                Jedi_Retreat(ctx);
            } else if enemy_dist > 1024 {
                Jedi_Advance(ctx);
            }
        } else if (*client).ps.saberInFlight != qfalse
            && crate::bg_saber::PM_SaberInBrokenParry((*client).ps.saberMove) == qfalse
            && (*client).ps.saberBlocked != BLOCKED_PARRY_BROKEN as c_int
        {
            //maintain distance
            if (enemy_dist as f32) < (*client).ps.saberEntityDist {
                Jedi_Retreat(ctx);
            } else if (enemy_dist as f32) > (*client).ps.saberEntityDist && enemy_dist > 100 {
                Jedi_Advance(ctx);
            }
            if (*client).ps.weapon == WP_SABER as c_int
                && (*client).ps.saberEntityState == SES_LEAVING as c_int
                && (*client).ps.fd.forcePowerLevel[FP_SABERTHROW as usize] > FORCE_LEVEL_1
                && ((*client).ps.fd.forcePowersActive & (1 << FP_SPEED)) == 0
                && ((*client).ps.saberEventFlags & SEF_INWATER as c_int) == 0
            {
                //hold it out there
                (*world).globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
            }
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"taunting".as_ptr()) == qfalse {
            if enemy_dist <= 64 {
                //he's getting too close
                (*world).globals.ucmd.buttons |= BUTTON_ATTACK;
                if (*client).ps.saberInFlight == qfalse {
                    crate::w_saber::WP_ActivateSaber(ctx, npc);
                }
                crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), -(*world).level.time);
            } else if (*client).ps.forceHandExtend == HANDEXTEND_JEDITAUNT as c_int
                && ((*client).ps.forceHandExtendTime - (*world).level.time) < 200
            {
                //we're almost done with our special taunt
                if (*client).ps.saberInFlight == qfalse {
                    crate::w_saber::WP_ActivateSaber(ctx, npc);
                }
            }
        } else if (*client).ps.saberEventFlags & SEF_LOCK_WON as c_int != 0 {
            //we won a saber lock, press the advantage
            if enemy_dist > 0 {
                //get closer so we can hit!
                Jedi_Advance(ctx);
            }
            if enemy_dist > 128 {
                //lost 'em
                (*client).ps.saberEventFlags &= !SEF_LOCK_WON as c_int;
            }
            if !enemy.is_null() && (*enemy).painDebounceTime + 2000 < (*world).level.time {
                //the window of opportunity is gone
                (*client).ps.saberEventFlags &= !SEF_LOCK_WON as c_int;
            }
            //don't strafe?
            crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), -1);
            crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), -1);
        } else if !enemy.is_null()
            && !(*enemy).client.is_null()
            && (*enemy).s.weapon == WP_SABER as c_int
            && (*enemy_client).ps.saberLockTime > (*world).level.time
            && (*client).ps.saberLockTime < (*world).level.time
        {
            //enemy is in a saberLock and we are not
            if enemy_dist < 64 {
                Jedi_Retreat(ctx);
            }
        } else if enemy_dist <= 64
            && (((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0
                || (crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) == 0
                    && (*world).bg_state.rng.Q_irand(0, 10) == 0))
        {
            //can't use saber and they're in striking range
            if (*world).bg_state.rng.Q_irand(0, 5) == 0
                && crate::NPC_senses::InFront(
                    (*enemy).r.currentOrigin,
                    (*npc).r.currentOrigin,
                    (*client).ps.viewangles,
                    0.2f32,
                ) != qfalse
            {
                if (((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0
                    || (*client).pers.maxHealth - (*npc).health
                        > ((*client).pers.maxHealth as f32 * 0.25f32) as c_int)
                    && (*client).ps.fd.forcePowersKnown & (1 << FP_DRAIN) != 0
                    && crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_DRAIN, 20) != qfalse
                    && (*world).bg_state.rng.Q_irand(0, 2) == 0
                {
                    //drain
                    crate::g_timer::TIMER_Set(ctx, npc, c"draining".as_ptr(), 3000);
                    crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), 3000);
                    Jedi_Advance(ctx);
                    return;
                } else {
                    crate::w_force::ForceThrow(ctx, npc, qfalse);
                }
            }
            Jedi_Retreat(ctx);
        } else if enemy_dist <= 64
            && (*client).pers.maxHealth - (*npc).health
                > ((*client).pers.maxHealth as f32 * 0.25f32) as c_int
            && (*client).ps.fd.forcePowersKnown & (1 << FP_DRAIN) != 0
            && crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_DRAIN, 20) != qfalse
            && (*world).bg_state.rng.Q_irand(0, 10) == 0
            && crate::NPC_senses::InFront(
                (*enemy).r.currentOrigin,
                (*npc).r.currentOrigin,
                (*client).ps.viewangles,
                0.2f32,
            ) != qfalse
        {
            crate::g_timer::TIMER_Set(ctx, npc, c"draining".as_ptr(), 3000);
            crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), 3000);
            Jedi_Advance(ctx);
            return;
        } else if enemy_dist <= -16 {
            //we're too damn close!
            Jedi_Retreat(ctx);
        } else if enemy_dist <= 0 {
            //we're within striking range
            if (*npc_info).stats.aggression < 4 {
                //back off and defend
                Jedi_Retreat(ctx);
            }
        } else if enemy_dist > 256 {
            //we're way out of range
            let mut usedForce: qboolean = qfalse;
            if (*npc_info).stats.aggression < (*world).bg_state.rng.Q_irand(0, 20)
                && (*npc).health < ((*client).pers.maxHealth as f32 * 0.75f32) as c_int
                && (*world).bg_state.rng.Q_irand(0, 2) == 0
            {
                if ((*client).ps.fd.forcePowersKnown & (1 << FP_HEAL)) != 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_HEAL)) == 0
                    && (*world).bg_state.rng.Q_irand(0, 1) != 0
                {
                    crate::w_force::ForceHeal(ctx, npc);
                    usedForce = qtrue;
                } else if ((*client).ps.fd.forcePowersKnown & (1 << FP_PROTECT)) != 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_PROTECT)) == 0
                    && (*world).bg_state.rng.Q_irand(0, 1) != 0
                {
                    crate::w_force::ForceProtect(ctx, npc);
                    usedForce = qtrue;
                } else if ((*client).ps.fd.forcePowersKnown & (1 << FP_ABSORB)) != 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_ABSORB)) == 0
                    && (*world).bg_state.rng.Q_irand(0, 1) != 0
                {
                    crate::w_force::ForceAbsorb(ctx, npc);
                    usedForce = qtrue;
                } else if ((*client).ps.fd.forcePowersKnown & (1 << FP_RAGE)) != 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                    && (*world).bg_state.rng.Q_irand(0, 1) != 0
                {
                    Jedi_Rage(ctx);
                    usedForce = qtrue;
                }
            }
            if enemy_dist > 384 {
                // PORT-NOTE(jediSpeechDebounceTime): `()` placeholder field indexed by team.
                if (*world).bg_state.rng.Q_irand(0, 10) == 0
                    && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
                    && (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize]
                        < (*world).level.time
                {
                    if crate::NPC_utils::NPC_ClearLOS4(ctx, enemy) != qfalse {
                        crate::NPC_sounds::G_AddVoiceEvent(
                            ctx,
                            npc,
                            (*world).bg_state.rng.Q_irand(
                                entity_event_t::EV_JCHASE1 as c_int,
                                entity_event_t::EV_JCHASE3 as c_int,
                            ),
                            3000,
                        );
                    }
                    (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 3000;
                    (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                        (*world).level.time + 3000;
                }
            }
            //Unless we're totally hiding, go after him
            if (*npc_info).stats.aggression > 0 {
                if usedForce == qfalse {
                    Jedi_Advance(ctx);
                }
            }
        } else if enemy_dist > 50 {
            //we're out of striking range and we are allowed to attack
            //first, check some tactical force power decisions
            if !enemy.is_null()
                && !(*enemy).client.is_null()
                && (*enemy_client).ps.fd.forceGripBeingGripped > (*world).level.time
            {
                //They're being gripped, rush them!
                if (*enemy_client).ps.groundEntityNum != ENTITYNUM_NONE {
                    if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse
                        || (*npc_info).rank > RANK_LT as c_int
                    {
                        if enemy_dist > 200 || ((*npc_info).scriptFlags & SCF_DONT_FIRE) == 0 {
                            Jedi_Advance(ctx);
                        }
                    }
                }
                if (*npc_info).rank >= RANK_LT_JG as c_int
                    && (*world).bg_state.rng.Q_irand(0, 5) == 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_SPEED)) == 0
                    && ((*client).ps.saberEventFlags & SEF_INWATER as c_int) == 0
                {
                    //throw saber
                    (*world).globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
                }
            } else if !enemy.is_null()
                && !(*enemy).client.is_null()
                && (*enemy_client).ps.saberInFlight != qfalse
                && (*enemy_client).ps.saberEntityNum != 0
                && (*client).ps.weaponTime <= 0
                && crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_GRIP, 0) != qfalse
                && (*world).bg_state.rng.Q_irand(0, 10) == 0
                && (*world).bg_state.rng.Q_irand(0, 6) < (*world).cvars.g_spskill.integer
                && (*world).bg_state.rng.Q_irand(RANK_CIVILIAN as c_int, RANK_CAPTAIN as c_int) < (*npc_info).rank
            {
                //They're throwing their saber, grip them!
                if crate::g_timer::TIMER_Done(ctx, npc, c"chatter".as_ptr()) != qfalse
                    && (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize]
                        < (*world).level.time
                    && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
                {
                    crate::NPC_sounds::G_AddVoiceEvent(
                        ctx,
                        npc,
                        (*world).bg_state.rng.Q_irand(
                            entity_event_t::EV_TAUNT1 as c_int,
                            entity_event_t::EV_TAUNT3 as c_int,
                        ),
                        3000,
                    );
                    (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 3000;
                    (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                        (*world).level.time + 3000;
                    crate::g_timer::TIMER_Set(ctx, npc, c"chatter".as_ptr(), 3000);
                }
                //grip
                crate::g_timer::TIMER_Set(ctx, npc, c"gripping".as_ptr(), 3000);
                crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), 3000);
            } else {
                let chanceScale: c_int;

                if !enemy.is_null()
                    && !(*enemy).client.is_null()
                    && ((*enemy_client).ps.fd.forcePowersActive & (1 << FP_GRIP)) != 0
                {
                    //They're choking someone, run at them
                    if (*enemy_client).ps.groundEntityNum != ENTITYNUM_NONE {
                        if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse
                            || (*npc_info).rank > RANK_LT as c_int
                        {
                            if enemy_dist > 200 || ((*npc_info).scriptFlags & SCF_DONT_FIRE) == 0 {
                                Jedi_Advance(ctx);
                            }
                        }
                    }
                }
                if (*client).NPC_class == CLASS_DESANN
                    || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) == 0
                {
                    chanceScale = 1;
                } else if (*npc_info).rank == RANK_ENSIGN as c_int {
                    chanceScale = 2;
                } else if (*npc_info).rank >= RANK_LT_JG as c_int {
                    chanceScale = 5;
                } else {
                    chanceScale = 0;
                }
                if chanceScale != 0
                    && (enemy_dist > (*world).bg_state.rng.Q_irand(100, 200)
                        || ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0
                        || (crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) == 0
                            && (*world).bg_state.rng.Q_irand(0, 3) == 0))
                    && enemy_dist < 500
                    && ((*world).bg_state.rng.Q_irand(0, chanceScale * 10) < 5
                        || (!enemy.is_null()
                            && !(*enemy).client.is_null()
                            && (*enemy_client).ps.weapon != WP_SABER as c_int
                            && (*world).bg_state.rng.Q_irand(0, chanceScale) == 0))
                {
                    //else, randomly try some kind of attack every now and then
                    if ((*npc_info).rank == RANK_ENSIGN as c_int
                        || (*npc_info).rank > RANK_LT_JG as c_int)
                        && (*world).bg_state.rng.Q_irand(0, 1) == 0
                    {
                        if crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_PULL, 0) != qfalse
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            //force pull the guy to me!
                            crate::w_force::ForceThrow(ctx, npc, qtrue);
                            crate::g_timer::TIMER_Set(ctx, npc, c"duck".as_ptr(), enemy_dist * 3);
                            if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                                (*world).globals.ucmd.buttons |= BUTTON_ATTACK;
                            }
                        } else if crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_LIGHTNING, 0)
                            != qfalse
                            && (*world).bg_state.rng.Q_irand(0, 1) != 0
                        {
                            crate::w_force::ForceLightning(ctx, npc);
                            if (*client).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_1
                            {
                                (*client).ps.weaponTime = (*world).bg_state.rng
                                    .Q_irand(1000, 3000 + ((*world).cvars.g_spskill.integer * 500));
                                crate::g_timer::TIMER_Set(
                                    ctx,
                                    npc,
                                    c"holdLightning".as_ptr(),
                                    (*client).ps.weaponTime,
                                );
                            }
                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"attackDelay".as_ptr(),
                                (*client).ps.weaponTime,
                            );
                        } else if crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_GRIP, 0)
                            != qfalse
                        {
                            //taunt
                            if crate::g_timer::TIMER_Done(ctx, npc, c"chatter".as_ptr()) != qfalse
                                && (*world).globals.jediSpeechDebounceTime
                                    [(*client).playerTeam as usize]
                                    < (*world).level.time
                                && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
                            {
                                crate::NPC_sounds::G_AddVoiceEvent(
                                    ctx,
                                    npc,
                                    (*world).bg_state.rng.Q_irand(
                                        entity_event_t::EV_TAUNT1 as c_int,
                                        entity_event_t::EV_TAUNT3 as c_int,
                                    ),
                                    3000,
                                );
                                (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 3000;
                                (*world).globals.jediSpeechDebounceTime
                                    [(*client).playerTeam as usize] = (*world).level.time + 3000;
                                crate::g_timer::TIMER_Set(ctx, npc, c"chatter".as_ptr(), 3000);
                            }
                            //grip
                            crate::g_timer::TIMER_Set(ctx, npc, c"gripping".as_ptr(), 3000);
                            crate::g_timer::TIMER_Set(ctx, npc, c"attackDelay".as_ptr(), 3000);
                        } else {
                            if crate::w_force::WP_ForcePowerAvailable(ctx, npc, FP_SABERTHROW, 0)
                                != qfalse
                                && ((*client).ps.fd.forcePowersActive & (1 << FP_SPEED)) == 0
                                && ((*client).ps.saberEventFlags & SEF_INWATER as c_int) == 0
                            {
                                //throw saber
                                (*world).globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
                            }
                        }
                    } else {
                        if (*npc_info).rank >= RANK_LT_JG as c_int
                            && ((*client).ps.fd.forcePowersActive & (1 << FP_SPEED)) == 0
                            && ((*client).ps.saberEventFlags & SEF_INWATER as c_int) == 0
                        {
                            //throw saber
                            (*world).globals.ucmd.buttons |= BUTTON_ALT_ATTACK;
                        }
                    }
                }
                //see if we should advance now
                else if (*npc_info).stats.aggression > 5 {
                    if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse
                        || (*npc_info).rank > RANK_LT as c_int
                    {
                        if enemy.is_null()
                            || (*enemy).client.is_null()
                            || (*enemy_client).ps.groundEntityNum != ENTITYNUM_NONE
                        {
                            if enemy_dist > 200 || ((*npc_info).scriptFlags & SCF_DONT_FIRE) == 0 {
                                Jedi_Advance(ctx);
                            }
                        }
                    }
                }
            }
        } else {
            //we're not close enough to attack, but not far enough away to be safe
            if (*npc_info).stats.aggression < 4 {
                //back off and defend
                Jedi_Retreat(ctx);
            } else if (*npc_info).stats.aggression > 5 {
                //try to get closer
                if enemy_dist > 0 && ((*npc_info).scriptFlags & SCF_DONT_FIRE) == 0 {
                    if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse
                        || (*npc_info).rank > RANK_LT as c_int
                    {
                        if enemy.is_null()
                            || (*enemy).client.is_null()
                            || (*enemy_client).ps.groundEntityNum != ENTITYNUM_NONE
                        {
                            Jedi_Advance(ctx);
                        }
                    }
                }
            }
        }
        //if really really mad, rage!
        if (*npc_info).stats.aggression > (*world).bg_state.rng.Q_irand(5, 15)
            && (*npc).health < ((*client).pers.maxHealth as f32 * 0.75f32) as c_int
            && (*world).bg_state.rng.Q_irand(0, 2) == 0
        {
            if ((*client).ps.fd.forcePowersKnown & (1 << FP_RAGE)) != 0
                && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
            {
                Jedi_Rage(ctx);
            }
        }
    }
}

/// Raven `Jedi_Strafe`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1876-1929`
pub fn Jedi_Strafe(
    ctx: GameContext<'_>,
    strafeTimeMin: c_int,
    strafeTimeMax: c_int,
    nextStrafeTimeMin: c_int,
    nextStrafeTimeMax: c_int,
    walking: qboolean,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        if (*client).ps.saberEventFlags & SEF_LOCK_WON as c_int != 0
            && !enemy.is_null()
            && (*enemy).painDebounceTime > (*world).level.time
        {
            //don't strafe if pressing the advantage of winning a saberLock
            return qfalse;
        }
        if crate::g_timer::TIMER_Done(ctx, npc, c"strafeLeft".as_ptr()) != qfalse
            && crate::g_timer::TIMER_Done(ctx, npc, c"strafeRight".as_ptr()) != qfalse
        {
            let mut strafed: qboolean = qfalse;
            let strafeTime = (*world).bg_state.rng.Q_irand(strafeTimeMin, strafeTimeMax);

            if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                if NPC_MoveDirClear(ctx, (*world).globals.ucmd.forwardmove as c_int, -127, qfalse)
                    != qfalse
                {
                    crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), strafeTime);
                    strafed = qtrue;
                } else if NPC_MoveDirClear(
                    ctx,
                    (*world).globals.ucmd.forwardmove as c_int,
                    127,
                    qfalse,
                ) != qfalse
                {
                    crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), strafeTime);
                    strafed = qtrue;
                }
            } else {
                if NPC_MoveDirClear(ctx, (*world).globals.ucmd.forwardmove as c_int, 127, qfalse)
                    != qfalse
                {
                    crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), strafeTime);
                    strafed = qtrue;
                } else if NPC_MoveDirClear(
                    ctx,
                    (*world).globals.ucmd.forwardmove as c_int,
                    -127,
                    qfalse,
                ) != qfalse
                {
                    crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), strafeTime);
                    strafed = qtrue;
                }
            }

            if strafed != qfalse {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"noStrafe".as_ptr(),
                    strafeTime + (*world).bg_state.rng.Q_irand(nextStrafeTimeMin, nextStrafeTimeMax),
                );
                if walking != qfalse {
                    //should be a slow strafe
                    crate::g_timer::TIMER_Set(ctx, npc, c"walking".as_ptr(), strafeTime);
                }
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `Jedi_CheckFlipEvasions`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:1969-2303`
pub fn Jedi_CheckFlipEvasions(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    rightdot: f32,
    zdiff: f32,
) -> evasionType_t {
    unsafe {
        let world = ctx.world;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*self_).client as *mut gclient_t;
        let snpc = (*self_).NPC as *mut gNPC_t;

        if !(*self_).NPC.is_null() && ((*snpc).scriptFlags & SCF_NO_ACROBATICS) != 0 {
            return evasionType_t::EVASION_NONE;
        }
        if !(*self_).client.is_null()
            && ((*client).ps.fd.forceRageRecoveryTime > (*world).level.time
                || ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) != 0)
        {
            //no fancy dodges when raging
            return evasionType_t::EVASION_NONE;
        }
        if (*client).ps.legsAnim == BOTH_WALL_RUN_LEFT as c_int
            || (*client).ps.legsAnim == BOTH_WALL_RUN_RIGHT as c_int
        {
            //already running on a wall
            let mut right: vec3_t = [0.0; 3];
            let mut anim: c_int = -1;
            let animLength: f32;

            let fwdAngles: vec3_t = [0.0, (*client).ps.viewangles[YAW as usize], 0.0];
            crate::q_math::AngleVectors(fwdAngles, None, Some(&mut right), None);

            // PORT-NOTE(BG_AnimLength): resolved as a free-function that takes
            // a reference to the BgState from GameWorld.
            animLength = crate::bg_panimate::BG_AnimLength(
                &(*world).bg_state,
                (*self_).localAnimIndex,
                (*client).ps.legsAnim as c_int,
            ) as f32;
            if (*client).ps.legsAnim == BOTH_WALL_RUN_LEFT as c_int && rightdot < 0.0 {
                if animLength - (*client).ps.legsTimer as f32 > 400.0
                    && (*client).ps.legsTimer > 400
                {
                    anim = BOTH_WALL_RUN_LEFT_FLIP as c_int;
                }
            } else if (*client).ps.legsAnim == BOTH_WALL_RUN_RIGHT as c_int && rightdot > 0.0 {
                if animLength - (*client).ps.legsTimer as f32 > 400.0
                    && (*client).ps.legsTimer > 400
                {
                    anim = BOTH_WALL_RUN_RIGHT_FLIP as c_int;
                }
            }
            if anim != -1 {
                //flip off the wall!
                let parts: c_int;
                if anim == BOTH_WALL_RUN_LEFT_FLIP as c_int {
                    (*client).ps.velocity[0] *= 0.5f32;
                    (*client).ps.velocity[1] *= 0.5f32;
                    let v = (*client).ps.velocity;
                    crate::q_math::_VectorMA(v, 150.0, right, &mut (*client).ps.velocity);
                } else if anim == BOTH_WALL_RUN_RIGHT_FLIP as c_int {
                    (*client).ps.velocity[0] *= 0.5f32;
                    (*client).ps.velocity[1] *= 0.5f32;
                    let v = (*client).ps.velocity;
                    crate::q_math::_VectorMA(v, -150.0, right, &mut (*client).ps.velocity);
                }
                if (*client).ps.weaponTime == 0 {
                    parts = SETANIM_BOTH;
                } else {
                    parts = SETANIM_LEGS;
                }
                crate::npc_c::NPC_SetAnim(
                    self_,
                    parts,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_utils::G_AddEvent(self_, entity_event_t::EV_JUMP as c_int, 0);
                return evasionType_t::EVASION_OTHER;
            }
        } else if (*client).NPC_class != CLASS_DESANN
            && ((*snpc).rank == RANK_CREWMAN as c_int || (*snpc).rank >= RANK_LT as c_int)
            && (*world).bg_state.rng.Q_irand(0, 1) != 0
            && crate::bg_panimate::BG_InRoll(&mut (*client).ps, (*client).ps.legsAnim) == qfalse
            && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
            && crate::bg_panimate::BG_SaberInSpecialAttack((*client).ps.torsoAnim) == qfalse
        {
            let mut fwd: vec3_t = [0.0; 3];
            let mut right: vec3_t = [0.0; 3];
            let mut traceto: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();
            let parts: c_int;
            let mut anim: c_int;
            let speed: f32;
            let mut checkDist: f32;
            let mut allowCartWheels: qboolean = qtrue;
            let mut allowWallFlips: qboolean = qtrue;

            if (*client).ps.weapon == WP_SABER as c_int {
                if (*client).saber[0].model[0] != 0
                    && ((*client).saber[0].saberFlags & SFL_NO_CARTWHEELS) != 0
                {
                    allowCartWheels = qfalse;
                } else if (*client).saber[1].model[0] != 0
                    && ((*client).saber[1].saberFlags & SFL_NO_CARTWHEELS) != 0
                {
                    allowCartWheels = qfalse;
                }
                if (*client).saber[0].model[0] != 0
                    && ((*client).saber[0].saberFlags & SFL_NO_WALL_FLIPS) != 0
                {
                    allowWallFlips = qfalse;
                } else if (*client).saber[1].model[0] != 0
                    && ((*client).saber[1].saberFlags & SFL_NO_WALL_FLIPS) != 0
                {
                    allowWallFlips = qfalse;
                }
            }

            let mins: vec3_t = [(*self_).r.mins[0], (*self_).r.mins[1], 0.0];
            let maxs: vec3_t = [(*self_).r.maxs[0], (*self_).r.maxs[1], 24.0];
            let fwdAngles: vec3_t = [0.0, (*client).ps.viewangles[YAW as usize], 0.0];

            crate::q_math::AngleVectors(fwdAngles, Some(&mut fwd), Some(&mut right), None);

            parts = if crate::bg_panimate::BG_SaberInAttack((*client).ps.saberMove) != qfalse
                || crate::bg_panimate::PM_SaberInStart((*client).ps.saberMove) != qfalse
            {
                SETANIM_LEGS
            } else {
                SETANIM_BOTH
            };
            if rightdot >= 0.0 {
                anim = if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                    BOTH_ARIAL_LEFT as c_int
                } else {
                    BOTH_CARTWHEEL_LEFT as c_int
                };
                checkDist = -128.0;
                speed = -200.0;
            } else {
                anim = if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                    BOTH_ARIAL_RIGHT as c_int
                } else {
                    BOTH_CARTWHEEL_RIGHT as c_int
                };
                checkDist = 128.0;
                speed = 200.0;
            }
            //trace in the dir that we want to go
            crate::q_math::_VectorMA((*self_).r.currentOrigin, checkDist, right, &mut traceto);
            crate::trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    &traceto as *const vec3_t,
                    (*self_).s.number,
                    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                ),
            );
            if trace.fraction >= 1.0f32 && allowCartWheels != qfalse {
                //it's clear, let's do it
                let mut fwdAngles2: vec3_t = [0.0; 3];
                let mut jumpRt: vec3_t = [0.0; 3];

                crate::npc_c::NPC_SetAnim(
                    self_,
                    parts,
                    anim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                (*client).ps.weaponTime = (*client).ps.legsTimer;
                crate::q_math::_VectorCopy((*client).ps.viewangles, &mut fwdAngles2);
                fwdAngles2[PITCH as usize] = 0.0;
                fwdAngles2[ROLL as usize] = 0.0;
                crate::q_math::AngleVectors(fwdAngles2, None, Some(&mut jumpRt), None);
                crate::q_math::_VectorScale(jumpRt, speed, &mut (*client).ps.velocity);
                (*client).ps.fd.forceJumpCharge = 0.0;
                (*client).ps.velocity[2] = 200.0;
                (*client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2];
                if (*client).NPC_class == CLASS_BOBAFETT {
                    crate::g_utils::G_AddEvent(self_, entity_event_t::EV_JUMP as c_int, 0);
                } else {
                    crate::g_utils::G_SoundOnEnt(
                        ctx,
                        self_,
                        CHAN_BODY as c_int,
                        c"sound/weapons/force/jump.wav".as_ptr(),
                    );
                }
                return evasionType_t::EVASION_CARTWHEEL;
            } else if (trace.contents & CONTENTS_BOTCLIP) == 0 {
                //hit a wall, not a do-not-enter brush
                let mut idealNormal: vec3_t = [0.0; 3];

                crate::q_math::_VectorSubtract((*self_).r.currentOrigin, traceto, &mut idealNormal);
                crate::q_math::VectorNormalize(&mut idealNormal);
                let traceEnt = ge.add(trace.entityNum as usize);
                if (trace.entityNum < (ENTITYNUM_WORLD) as i16
                    && !traceEnt.is_null()
                    && (*traceEnt).s.solid != SOLID_BMODEL)
                    || crate::q_math::_DotProduct(trace.plane.normal, idealNormal) > 0.7f32
                {
                    //it's a ent of some sort or it's a wall roughly facing us
                    let mut bestCheckDist: f32 = 0.0;
                    //hmm, see if we're moving forward
                    if crate::q_math::_DotProduct((*client).ps.velocity, fwd) < 200.0 {
                        //not running forward very fast
                        if (trace.fraction * checkDist) <= 32.0 {
                            //wall on that side is close enough to wall-flip off of or wall-run on
                            bestCheckDist = checkDist;
                            checkDist *= -1.0f32;
                            crate::q_math::_VectorMA(
                                (*self_).r.currentOrigin,
                                checkDist,
                                right,
                                &mut traceto,
                            );
                            crate::trap::Trace(
                                ctx.engine,
                                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                    &mut trace as *mut trace_t,
                                    &(*self_).r.currentOrigin as *const vec3_t,
                                    &mins as *const vec3_t,
                                    &maxs as *const vec3_t,
                                    &traceto as *const vec3_t,
                                    (*self_).s.number,
                                    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                                ),
                            );
                            if trace.fraction >= 1.0f32 {
                                //it's clear, let's do it
                                if allowWallFlips != qfalse {
                                    let parts2: c_int;
                                    //turn the cartwheel into a wallflip in the other dir
                                    if rightdot > 0.0 {
                                        anim = BOTH_WALL_FLIP_LEFT as c_int;
                                        (*client).ps.velocity[0] = 0.0;
                                        (*client).ps.velocity[1] = 0.0;
                                        let v = (*client).ps.velocity;
                                        crate::q_math::_VectorMA(
                                            v,
                                            150.0,
                                            right,
                                            &mut (*client).ps.velocity,
                                        );
                                    } else {
                                        anim = BOTH_WALL_FLIP_RIGHT as c_int;
                                        (*client).ps.velocity[0] = 0.0;
                                        (*client).ps.velocity[1] = 0.0;
                                        let v = (*client).ps.velocity;
                                        crate::q_math::_VectorMA(
                                            v,
                                            -150.0,
                                            right,
                                            &mut (*client).ps.velocity,
                                        );
                                    }
                                    (*client).ps.velocity[2] =
                                        forceJumpStrength[FORCE_LEVEL_2 as usize] as f32 / 2.25f32;
                                    parts2 = if (*client).ps.weaponTime == 0 {
                                        SETANIM_BOTH
                                    } else {
                                        SETANIM_LEGS
                                    };
                                    crate::npc_c::NPC_SetAnim(
                                        self_,
                                        parts2,
                                        anim,
                                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                    );
                                    (*client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2];
                                    if (*client).NPC_class == CLASS_BOBAFETT {
                                        crate::g_utils::G_AddEvent(
                                            self_,
                                            entity_event_t::EV_JUMP as c_int,
                                            0,
                                        );
                                    } else {
                                        crate::g_utils::G_SoundOnEnt(
                                            ctx,
                                            self_,
                                            CHAN_BODY as c_int,
                                            c"sound/weapons/force/jump.wav".as_ptr(),
                                        );
                                    }
                                    return evasionType_t::EVASION_OTHER;
                                }
                            } else {
                                //boxed in on both sides
                                if crate::q_math::_DotProduct((*client).ps.velocity, fwd) < 0.0 {
                                    //moving backwards
                                    return evasionType_t::EVASION_NONE;
                                }
                                if (trace.fraction * checkDist) <= 32.0
                                    && (trace.fraction * checkDist) < bestCheckDist
                                {
                                    bestCheckDist = checkDist;
                                }
                            }
                        } else {
                            //too far from that wall to flip or run off it, check other side
                            checkDist *= -1.0f32;
                            crate::q_math::_VectorMA(
                                (*self_).r.currentOrigin,
                                checkDist,
                                right,
                                &mut traceto,
                            );
                            crate::trap::Trace(
                                ctx.engine,
                                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                    &mut trace as *mut trace_t,
                                    &(*self_).r.currentOrigin as *const vec3_t,
                                    &mins as *const vec3_t,
                                    &maxs as *const vec3_t,
                                    &traceto as *const vec3_t,
                                    (*self_).s.number,
                                    CONTENTS_SOLID | CONTENTS_MONSTERCLIP | CONTENTS_BOTCLIP,
                                ),
                            );
                            if (trace.fraction * checkDist) <= 32.0 {
                                bestCheckDist = checkDist;
                            } else {
                                return evasionType_t::EVASION_NONE;
                            }
                        }
                    }
                    //Try wall run?
                    if bestCheckDist != 0.0 {
                        //one of the walls was close enough to wall-run on
                        let mut allowWallRuns: qboolean = qtrue;
                        if (*client).ps.weapon == WP_SABER as c_int {
                            if (*client).saber[0].model[0] != 0
                                && ((*client).saber[0].saberFlags & SFL_NO_WALL_RUNS) != 0
                            {
                                allowWallRuns = qfalse;
                            } else if (*client).saber[1].model[0] != 0
                                && ((*client).saber[1].saberFlags & SFL_NO_WALL_RUNS) != 0
                            {
                                allowWallRuns = qfalse;
                            }
                        }
                        if allowWallRuns != qfalse {
                            let parts2: c_int;
                            if bestCheckDist > 0.0 {
                                anim = BOTH_WALL_RUN_RIGHT as c_int;
                            } else {
                                anim = BOTH_WALL_RUN_LEFT as c_int;
                            }
                            (*client).ps.velocity[2] =
                                forceJumpStrength[FORCE_LEVEL_2 as usize] as f32 / 2.25f32;
                            parts2 = if (*client).ps.weaponTime == 0 {
                                SETANIM_BOTH
                            } else {
                                SETANIM_LEGS
                            };
                            crate::npc_c::NPC_SetAnim(
                                self_,
                                parts2,
                                anim,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                            (*client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2];
                            if (*client).NPC_class == CLASS_BOBAFETT {
                                crate::g_utils::G_AddEvent(
                                    self_,
                                    entity_event_t::EV_JUMP as c_int,
                                    0,
                                );
                            } else {
                                crate::g_utils::G_SoundOnEnt(
                                    ctx,
                                    self_,
                                    CHAN_BODY as c_int,
                                    c"sound/weapons/force/jump.wav".as_ptr(),
                                );
                            }
                            return evasionType_t::EVASION_OTHER;
                        }
                    }
                }
            }
        }
        evasionType_t::EVASION_NONE
    }
}

/// Raven `Jedi_ReCalcParryTime`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2305-2441`
pub fn Jedi_ReCalcParryTime(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    evasionType: evasionType_t,
) -> c_int {
    unsafe {
        let world = ctx.world;
        let client = (*self_).client as *mut gclient_t;
        let snpc = (*self_).NPC as *mut gNPC_t;
        if (*self_).client.is_null() {
            return 0;
        }
        if (*self_).s.number == 0 {
            //player
            return bg_parryDebounce
                [(*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] as usize];
        } else if !(*self_).NPC.is_null() {
            if (*world).cvars.g_saberRealisticCombat.integer == 0
                && ((*world).cvars.g_spskill.integer == 2
                    || ((*world).cvars.g_spskill.integer == 1
                        && (*client).NPC_class == CLASS_TAVION))
            {
                if (*client).NPC_class == CLASS_TAVION {
                    return 0;
                } else {
                    return (*world).bg_state.rng.Q_irand(0, 150);
                }
            } else {
                let mut baseTime: c_int;
                if evasionType == evasionType_t::EVASION_DODGE {
                    baseTime = (*client).ps.torsoTimer;
                } else if evasionType == evasionType_t::EVASION_CARTWHEEL {
                    baseTime = (*client).ps.torsoTimer;
                } else if (*client).ps.saberInFlight != qfalse {
                    baseTime = (*world).bg_state.rng.Q_irand(1, 3) * 50;
                } else {
                    if (*world).cvars.g_saberRealisticCombat.integer != 0 {
                        baseTime = 500;
                        match (*world).cvars.g_spskill.integer {
                            0 => baseTime = 500,
                            1 => baseTime = 300,
                            _ => baseTime = 100,
                        }
                    } else {
                        baseTime = 150;
                        match (*world).cvars.g_spskill.integer {
                            0 => baseTime = 200,
                            1 => baseTime = 100,
                            _ => baseTime = 50,
                        }
                    }

                    if (*client).NPC_class == CLASS_TAVION {
                        //Tavion is faster
                        baseTime = (baseTime as f32 / 2.0f32).ceil() as c_int;
                    } else if (*snpc).rank >= RANK_LT_JG as c_int {
                        if (*world).bg_state.rng.Q_irand(0, 2) != 0 {
                            //medium speed parry
                        } else {
                            //with the occasional fast parry
                            baseTime = (baseTime as f32 / 2.0f32).ceil() as c_int;
                        }
                    } else if (*snpc).rank == RANK_CIVILIAN as c_int {
                        //grunts are slowest
                        baseTime = baseTime * (*world).bg_state.rng.Q_irand(1, 3);
                    } else if (*snpc).rank == RANK_CREWMAN as c_int {
                        //acrobats aren't so bad
                        if evasionType == evasionType_t::EVASION_PARRY
                            || evasionType == evasionType_t::EVASION_DUCK_PARRY
                            || evasionType == evasionType_t::EVASION_JUMP_PARRY
                        {
                            //slower with parries
                            baseTime = baseTime * (*world).bg_state.rng.Q_irand(1, 2);
                        }
                    } else {
                        //force users are kinda slow
                        baseTime = baseTime * (*world).bg_state.rng.Q_irand(1, 2);
                    }
                    if evasionType == evasionType_t::EVASION_DUCK
                        || evasionType == evasionType_t::EVASION_DUCK_PARRY
                    {
                        baseTime += 100;
                    } else if evasionType == evasionType_t::EVASION_JUMP
                        || evasionType == evasionType_t::EVASION_JUMP_PARRY
                    {
                        baseTime += 50;
                    } else if evasionType == evasionType_t::EVASION_OTHER {
                        baseTime += 100;
                    } else if evasionType == evasionType_t::EVASION_FJUMP {
                        baseTime += 100;
                    }
                }

                return baseTime;
            }
        }
        0
    }
}

/// Raven `Jedi_QuickReactions`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2443-2453`
pub fn Jedi_QuickReactions(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc_info = (*world).globals.NPCInfo;
        let client = (*self_).client as *mut gclient_t;
        if ((*client).NPC_class == CLASS_JEDI && (*npc_info).rank == RANK_COMMANDER as c_int)
            || (*client).NPC_class == CLASS_TAVION
            || ((*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_1
                && (*world).cvars.g_spskill.integer > 1)
            || ((*client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] > FORCE_LEVEL_2
                && (*world).cvars.g_spskill.integer > 0)
        {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Jedi_SaberBusy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2455-2470`
pub fn Jedi_SaberBusy(
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if (*client).ps.torsoTimer > 300
            && ((crate::bg_panimate::BG_SaberInAttack((*client).ps.saberMove) != qfalse
                && (*client).ps.fd.saberAnimLevel == FORCE_LEVEL_3)
                || crate::bg_panimate::BG_SpinningSaberAnim((*client).ps.torsoAnim) != qfalse
                || crate::bg_panimate::BG_SaberInSpecialAttack((*client).ps.torsoAnim) != qfalse
                || crate::bg_saber::PM_SaberInBrokenParry((*client).ps.saberMove) != qfalse
                || crate::bg_panimate::BG_FlippingAnim((*client).ps.torsoAnim) != qfalse
                || crate::bg_pmove::PM_RollingAnim((*client).ps.torsoAnim) != qfalse)
        {
            //my saber is not in a parrying position
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Jedi_SaberBlockGo`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:2485-3139`
pub fn Jedi_SaberBlockGo(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    cmd: *mut usercmd_t,
    pHitloc: vec3_t,
    phitDir: vec3_t,
    incoming: *mut gentity_t,
    dist: f32,
) -> evasionType_t {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*self_).client as *mut gclient_t;

        let mut hitloc: vec3_t = [0.0; 3];
        let mut hitdir: vec3_t = [0.0; 3];
        let mut diff: vec3_t = [0.0; 3];
        let mut fwdangles: vec3_t = [0.0, 0.0, 0.0];
        let mut right: vec3_t = [0.0; 3];
        let rightdot: f32;
        let zdiff: f32;
        let mut duckChance: c_int = 0;
        let mut dodgeAnim: c_int = -1;
        let mut saberBusy: qboolean = qfalse;
        let mut evaded: qboolean = qfalse;
        let mut doDodge: qboolean = qfalse;
        let mut evasionType: evasionType_t = evasionType_t::EVASION_NONE;
        let d_jedi = (*world).cvars.d_JediAI.integer != 0;

        if incoming.is_null() {
            crate::q_math::_VectorCopy(pHitloc, &mut hitloc);
            crate::q_math::_VectorCopy(phitDir, &mut hitdir);
            if (*client).ps.saberInFlight != qfalse {
                //DOH!  do non-saber evasion!
                saberBusy = qtrue;
            } else if Jedi_QuickReactions(ctx, self_) != qfalse {
                //trainer/tavion faster at parrying
            } else {
                saberBusy = Jedi_SaberBusy(self_);
            }
        } else {
            crate::q_math::_VectorCopy((*incoming).r.currentOrigin, &mut hitloc);
            crate::q_math::VectorNormalize2((*incoming).s.pos.trDelta, &mut hitdir);
        }
        if !(*self_).client.is_null() && (*client).NPC_class == CLASS_BOBAFETT {
            saberBusy = qtrue;
        }

        crate::q_math::_VectorSubtract(hitloc, (*client).renderInfo.eyePoint, &mut diff);
        diff[2] = 0.0;
        fwdangles[1] = (*client).ps.viewangles[1];
        crate::q_math::AngleVectors(fwdangles, None, Some(&mut right), None);

        rightdot = crate::q_math::_DotProduct(right, diff);
        zdiff = hitloc[2] - (*client).renderInfo.eyePoint[2];

        //see if we can dodge if need-be
        if (dist > 16.0 && ((*world).bg_state.rng.Q_irand(0, 2) != 0 || saberBusy != qfalse))
            || (*client).ps.saberInFlight != qfalse
            || crate::bg_pmove::BG_SabersOff(&mut (*client).ps) != qfalse
            || (*client).NPC_class == CLASS_BOBAFETT
        {
            let snpc = (*self_).NPC as *mut gNPC_t;
            if !(*self_).NPC.is_null()
                && ((*snpc).rank == RANK_CREWMAN as c_int || (*snpc).rank >= RANK_LT_JG as c_int)
            {
                if (*client).ps.groundEntityNum != ENTITYNUM_NONE
                    && ((*client).ps.pm_flags & PMF_DUCKED) == 0
                    && (*cmd).upmove >= 0
                    && crate::g_timer::TIMER_Done(ctx, self_, c"duck".as_ptr()) != qfalse
                    && crate::bg_panimate::BG_InRoll(&mut (*client).ps, (*client).ps.legsAnim)
                        == qfalse
                    && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                    && ((*client).ps.saberInFlight != qfalse
                        || (*client).NPC_class == CLASS_BOBAFETT
                        || (crate::bg_panimate::BG_SaberInAttack((*client).ps.saberMove) == qfalse
                            && crate::bg_panimate::PM_SaberInStart((*client).ps.saberMove)
                                == qfalse
                            && crate::bg_panimate::BG_SpinningSaberAnim((*client).ps.torsoAnim)
                                == qfalse
                            && crate::bg_panimate::BG_SaberInSpecialAttack((*client).ps.torsoAnim)
                                == qfalse))
                {
                    doDodge = qtrue;
                }
            }
        }
        if d_jedi {
            crate::g_main::Com_Printf(
                cstr(&format!(
                    "({}) evading attack from height {:.2}, zdiff: {:.2}, rightdot: {:.2}\n",
                    (*world).level.time,
                    hitloc[2] - (*self_).r.absmin[2],
                    zdiff,
                    rightdot
                ))
                .as_ptr(),
            );
        }

        if zdiff >= -5.0 {
            if !incoming.is_null() || saberBusy == qfalse {
                if rightdot > 12.0
                    || (rightdot > 3.0 && zdiff < 5.0)
                    || (incoming.is_null() && hitdir[2].abs() < 0.25f32)
                {
                    //coming from right
                    if doDodge != qfalse {
                        if (*client).NPC_class == CLASS_BOBAFETT
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            //roll!
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"duck".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"strafeLeft".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), 0);
                            evasionType = evasionType_t::EVASION_DUCK;
                            evaded = qtrue;
                        } else if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                            dodgeAnim = BOTH_DODGE_FL as c_int;
                        } else {
                            dodgeAnim = BOTH_DODGE_BL as c_int;
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_UPPER_RIGHT as c_int;
                        evasionType = evasionType_t::EVASION_PARRY;
                        if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                            if zdiff > 5.0 {
                                crate::g_timer::TIMER_Start(
                                    ctx,
                                    self_,
                                    c"duck".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1500),
                                );
                                evasionType = evasionType_t::EVASION_DUCK_PARRY;
                                evaded = qtrue;
                                if d_jedi {
                                    crate::g_main::Com_Printf(cstr("duck ").as_ptr());
                                }
                            } else {
                                duckChance = 6;
                            }
                        }
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("UR block\n").as_ptr());
                    }
                } else if rightdot < -12.0
                    || (rightdot < -3.0 && zdiff < 5.0)
                    || (incoming.is_null() && hitdir[2].abs() < 0.25f32)
                {
                    //coming from left
                    if doDodge != qfalse {
                        if (*client).NPC_class == CLASS_BOBAFETT
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            //roll!
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"duck".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"strafeRight".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Set(ctx, self_, c"strafeLeft".as_ptr(), 0);
                            evasionType = evasionType_t::EVASION_DUCK;
                            evaded = qtrue;
                        } else if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                            dodgeAnim = BOTH_DODGE_FR as c_int;
                        } else {
                            dodgeAnim = BOTH_DODGE_BR as c_int;
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT as c_int;
                        evasionType = evasionType_t::EVASION_PARRY;
                        if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                            if zdiff > 5.0 {
                                crate::g_timer::TIMER_Start(
                                    ctx,
                                    self_,
                                    c"duck".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1500),
                                );
                                evasionType = evasionType_t::EVASION_DUCK_PARRY;
                                evaded = qtrue;
                                if d_jedi {
                                    crate::g_main::Com_Printf(cstr("duck ").as_ptr());
                                }
                            } else {
                                duckChance = 6;
                            }
                        }
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("UL block\n").as_ptr());
                    }
                } else {
                    (*client).ps.saberBlocked = BLOCKED_TOP as c_int;
                    evasionType = evasionType_t::EVASION_PARRY;
                    if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                        duckChance = 4;
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("TOP block\n").as_ptr());
                    }
                }
                evaded = qtrue;
            } else {
                if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                    crate::g_timer::TIMER_Start(
                        ctx,
                        self_,
                        c"duck".as_ptr(),
                        (*world).bg_state.rng.Q_irand(500, 1500),
                    );
                    evasionType = evasionType_t::EVASION_DUCK;
                    evaded = qtrue;
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("duck ").as_ptr());
                    }
                }
            }
        } else if zdiff > -22.0 {
            //hmm, pretty low, need to duck
            if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                crate::g_timer::TIMER_Start(
                    ctx,
                    self_,
                    c"duck".as_ptr(),
                    (*world).bg_state.rng.Q_irand(500, 1500),
                );
                evasionType = evasionType_t::EVASION_DUCK;
                evaded = qtrue;
                if d_jedi {
                    crate::g_main::Com_Printf(cstr("duck ").as_ptr());
                }
            }
            if !incoming.is_null() || saberBusy == qfalse {
                if rightdot > 8.0 || (rightdot > 3.0 && zdiff < -11.0) {
                    if doDodge != qfalse {
                        if (*client).NPC_class == CLASS_BOBAFETT
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"strafeLeft".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), 0);
                        } else {
                            dodgeAnim = BOTH_DODGE_L as c_int;
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_UPPER_RIGHT as c_int;
                        if evasionType == evasionType_t::EVASION_DUCK {
                            evasionType = evasionType_t::EVASION_DUCK_PARRY;
                        } else {
                            evasionType = evasionType_t::EVASION_PARRY;
                        }
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("mid-UR block\n").as_ptr());
                    }
                } else if rightdot < -8.0 || (rightdot < -3.0 && zdiff < -11.0) {
                    if doDodge != qfalse {
                        if (*client).NPC_class == CLASS_BOBAFETT
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            crate::g_timer::TIMER_Start(
                                ctx,
                                self_,
                                c"strafeLeft".as_ptr(),
                                (*world).bg_state.rng.Q_irand(500, 1500),
                            );
                            crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), 0);
                        } else {
                            dodgeAnim = BOTH_DODGE_R as c_int;
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_UPPER_LEFT as c_int;
                        if evasionType == evasionType_t::EVASION_DUCK {
                            evasionType = evasionType_t::EVASION_DUCK_PARRY;
                        } else {
                            evasionType = evasionType_t::EVASION_PARRY;
                        }
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("mid-UL block\n").as_ptr());
                    }
                } else {
                    (*client).ps.saberBlocked = BLOCKED_TOP as c_int;
                    if evasionType == evasionType_t::EVASION_DUCK {
                        evasionType = evasionType_t::EVASION_DUCK_PARRY;
                    } else {
                        evasionType = evasionType_t::EVASION_PARRY;
                    }
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("mid-TOP block\n").as_ptr());
                    }
                }
                evaded = qtrue;
            }
        } else if saberBusy != qfalse
            || (zdiff < -36.0
                && (zdiff < -44.0 || (*world).bg_state.rng.Q_irand(0, 2) == 0))
        {
            //jump!
            let snpc = (*self_).NPC as *mut gNPC_t;
            if (*client).ps.groundEntityNum == ENTITYNUM_NONE {
                //already in air, duck to pull up legs
                crate::g_timer::TIMER_Start(
                    ctx,
                    self_,
                    c"duck".as_ptr(),
                    (*world).bg_state.rng.Q_irand(500, 1500),
                );
                evasionType = evasionType_t::EVASION_DUCK;
                evaded = qtrue;
                if d_jedi {
                    crate::g_main::Com_Printf(cstr("legs up\n").as_ptr());
                }
                if !incoming.is_null() || saberBusy == qfalse {
                    if rightdot >= 0.0 {
                        (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT as c_int;
                        evasionType = evasionType_t::EVASION_DUCK_PARRY;
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("LR block\n").as_ptr());
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT as c_int;
                        evasionType = evasionType_t::EVASION_DUCK_PARRY;
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("LL block\n").as_ptr());
                        }
                    }
                    evaded = qtrue;
                }
            } else {
                //gotta jump!
                if !(*self_).NPC.is_null()
                    && ((*snpc).rank == RANK_CREWMAN as c_int
                        || (*snpc).rank > RANK_LT_JG as c_int)
                    && ((*world).bg_state.rng.Q_irand(0, 10) == 0
                        || ((*world).bg_state.rng.Q_irand(0, 2) == 0
                            && ((*cmd).forwardmove != 0 || (*cmd).rightmove != 0)))
                {
                    //superjump
                    if !(*self_).NPC.is_null()
                        && ((*snpc).scriptFlags & SCF_NO_ACROBATICS) == 0
                        && (*client).ps.fd.forceRageRecoveryTime < (*world).level.time
                        && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                        && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                    {
                        (*client).ps.fd.forceJumpCharge = 320.0;
                        evasionType = evasionType_t::EVASION_FJUMP;
                        evaded = qtrue;
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("force jump + ").as_ptr());
                        }
                    }
                } else {
                    //normal jump
                    if !(*self_).NPC.is_null()
                        && ((*snpc).scriptFlags & SCF_NO_ACROBATICS) == 0
                        && (*client).ps.fd.forceRageRecoveryTime < (*world).level.time
                        && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                    {
                        if (*client).NPC_class == CLASS_BOBAFETT
                            && (*world).bg_state.rng.Q_irand(0, 1) == 0
                        {
                            //roll!
                            if rightdot > 0.0 {
                                crate::g_timer::TIMER_Start(
                                    ctx,
                                    self_,
                                    c"strafeLeft".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1500),
                                );
                                crate::g_timer::TIMER_Set(ctx, self_, c"strafeRight".as_ptr(), 0);
                                crate::g_timer::TIMER_Set(ctx, self_, c"walking".as_ptr(), 0);
                            } else {
                                crate::g_timer::TIMER_Start(
                                    ctx,
                                    self_,
                                    c"strafeRight".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1500),
                                );
                                crate::g_timer::TIMER_Set(ctx, self_, c"strafeLeft".as_ptr(), 0);
                                crate::g_timer::TIMER_Set(ctx, self_, c"walking".as_ptr(), 0);
                            }
                        } else {
                            if self_ == npc {
                                (*cmd).upmove = 127;
                            } else {
                                (*client).ps.velocity[2] = JUMP_VELOCITY;
                            }
                        }
                        evasionType = evasionType_t::EVASION_JUMP;
                        evaded = qtrue;
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("jump + ").as_ptr());
                        }
                    }
                    if (*client).NPC_class == CLASS_TAVION {
                        if incoming.is_null()
                            && (*client).ps.groundEntityNum < ENTITYNUM_NONE
                            && (*world).bg_state.rng.Q_irand(0, 2) == 0
                        {
                            if crate::bg_panimate::BG_SaberInAttack((*client).ps.saberMove)
                                == qfalse
                                && crate::bg_panimate::PM_SaberInStart((*client).ps.saberMove)
                                    == qfalse
                                && crate::bg_panimate::BG_InRoll(
                                    &mut (*client).ps,
                                    (*client).ps.legsAnim,
                                ) == qfalse
                                && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                                && crate::bg_panimate::BG_SaberInSpecialAttack(
                                    (*client).ps.torsoAnim,
                                ) == qfalse
                            {
                                //do the butterfly!
                                let butterflyAnim = if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                                    BOTH_BUTTERFLY_LEFT as c_int
                                } else {
                                    BOTH_BUTTERFLY_RIGHT as c_int
                                };
                                evasionType = evasionType_t::EVASION_CARTWHEEL;
                                crate::npc_c::NPC_SetAnim(
                                    self_,
                                    SETANIM_BOTH,
                                    butterflyAnim,
                                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                );
                                (*client).ps.velocity[2] = 225.0;
                                (*client).ps.fd.forceJumpZStart = (*self_).r.currentOrigin[2];
                                if (*client).NPC_class == CLASS_BOBAFETT {
                                    crate::g_utils::G_AddEvent(
                                        self_,
                                        entity_event_t::EV_JUMP as c_int,
                                        0,
                                    );
                                } else {
                                    crate::g_utils::G_Sound(
                                        ctx,
                                        self_,
                                        CHAN_BODY as c_int,
                                        crate::g_utils::G_SoundIndex(
                                            c"sound/weapons/force/jump.wav".as_ptr(),
                                        ),
                                    );
                                }
                                (*cmd).upmove = 0;
                                saberBusy = qtrue;
                                evaded = qtrue;
                            }
                        }
                    }
                }
                evasionType = Jedi_CheckFlipEvasions(ctx, self_, rightdot, zdiff);
                if evasionType != evasionType_t::EVASION_NONE {
                    let enemy: *mut gentity_t = match (*self_).enemy {
                        Some(id) => ge.add(id.0 as usize),
                        None => core::ptr::null_mut(),
                    };
                    if (*world).cvars.d_slowmodeath.integer > 5
                        && !enemy.is_null()
                        && (*enemy).s.number == 0
                    {
                        G_StartMatrixEffect(self_);
                    }
                    saberBusy = qtrue;
                    evaded = qtrue;
                } else if !incoming.is_null() || saberBusy == qfalse {
                    if rightdot >= 0.0 {
                        (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT as c_int;
                        if evasionType == evasionType_t::EVASION_JUMP {
                            evasionType = evasionType_t::EVASION_JUMP_PARRY;
                        } else if evasionType == evasionType_t::EVASION_NONE {
                            evasionType = evasionType_t::EVASION_PARRY;
                        }
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("LR block\n").as_ptr());
                        }
                    } else {
                        (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT as c_int;
                        if evasionType == evasionType_t::EVASION_JUMP {
                            evasionType = evasionType_t::EVASION_JUMP_PARRY;
                        } else if evasionType == evasionType_t::EVASION_NONE {
                            evasionType = evasionType_t::EVASION_PARRY;
                        }
                        if d_jedi {
                            crate::g_main::Com_Printf(cstr("LL block\n").as_ptr());
                        }
                    }
                    evaded = qtrue;
                }
            }
        } else {
            if !incoming.is_null() || saberBusy == qfalse {
                let snpc = (*self_).NPC as *mut gNPC_t;
                if rightdot >= 0.0 {
                    (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT as c_int;
                    evasionType = evasionType_t::EVASION_PARRY;
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("LR block\n").as_ptr());
                    }
                } else {
                    (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT as c_int;
                    evasionType = evasionType_t::EVASION_PARRY;
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("LL block\n").as_ptr());
                    }
                }
                if !incoming.is_null() && (*incoming).s.weapon == WP_SABER as c_int {
                    //thrown saber!
                    if !(*self_).NPC.is_null()
                        && ((*snpc).rank == RANK_CREWMAN as c_int
                            || (*snpc).rank > RANK_LT_JG as c_int)
                        && ((*world).bg_state.rng.Q_irand(0, 10) == 0
                            || ((*world).bg_state.rng.Q_irand(0, 2) == 0
                                && ((*cmd).forwardmove != 0 || (*cmd).rightmove != 0)))
                    {
                        //superjump
                        if !(*self_).NPC.is_null()
                            && ((*snpc).scriptFlags & SCF_NO_ACROBATICS) == 0
                            && (*client).ps.fd.forceRageRecoveryTime < (*world).level.time
                            && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                            && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                        {
                            (*client).ps.fd.forceJumpCharge = 320.0;
                            evasionType = evasionType_t::EVASION_FJUMP;
                            if d_jedi {
                                crate::g_main::Com_Printf(cstr("force jump + ").as_ptr());
                            }
                        }
                    } else {
                        //normal jump
                        if !(*self_).NPC.is_null()
                            && ((*snpc).scriptFlags & SCF_NO_ACROBATICS) == 0
                            && (*client).ps.fd.forceRageRecoveryTime < (*world).level.time
                            && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                        {
                            if self_ == npc {
                                (*cmd).upmove = 127;
                            } else {
                                (*client).ps.velocity[2] = JUMP_VELOCITY;
                            }
                            evasionType = evasionType_t::EVASION_JUMP_PARRY;
                            if d_jedi {
                                crate::g_main::Com_Printf(cstr("jump + ").as_ptr());
                            }
                        }
                    }
                }
                evaded = qtrue;
            }
        }

        if evasionType == evasionType_t::EVASION_NONE {
            return evasionType_t::EVASION_NONE;
        }
        //stop taunting
        crate::g_timer::TIMER_Set(ctx, self_, c"taunting".as_ptr(), 0);
        //stop gripping
        crate::g_timer::TIMER_Set(ctx, self_, c"gripping".as_ptr(), -(*world).level.time);
        crate::w_force::WP_ForcePowerStop(ctx, self_, FP_GRIP);
        //stop draining
        crate::g_timer::TIMER_Set(ctx, self_, c"draining".as_ptr(), -(*world).level.time);
        crate::w_force::WP_ForcePowerStop(ctx, self_, FP_DRAIN);

        if dodgeAnim != -1 {
            //dodged
            evasionType = evasionType_t::EVASION_DODGE;
            crate::npc_c::NPC_SetAnim(
                self_,
                SETANIM_BOTH,
                dodgeAnim,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            (*client).ps.weaponTime = (*client).ps.torsoTimer;
            (*client).ps.pm_time = (*client).ps.torsoTimer;
            (*client).ps.pm_flags |= PMF_TIME_KNOCKBACK;
            let enemy: *mut gentity_t = match (*self_).enemy {
                Some(id) => ge.add(id.0 as usize),
                None => core::ptr::null_mut(),
            };
            if (*world).cvars.d_slowmodeath.integer > 5 && !enemy.is_null() && (*enemy).s.number == 0
            {
                G_StartMatrixEffect(self_);
            }
        } else {
            if duckChance != 0 {
                if (*world).bg_state.rng.Q_irand(0, duckChance) == 0 {
                    crate::g_timer::TIMER_Start(
                        ctx,
                        self_,
                        c"duck".as_ptr(),
                        (*world).bg_state.rng.Q_irand(500, 1500),
                    );
                    if evasionType == evasionType_t::EVASION_PARRY {
                        evasionType = evasionType_t::EVASION_DUCK_PARRY;
                    } else {
                        evasionType = evasionType_t::EVASION_DUCK;
                    }
                }
            }

            if !incoming.is_null() {
                (*client).ps.saberBlocked =
                    crate::w_saber::WP_MissileBlockForBlock((*client).ps.saberBlocked);
            }
        }
        {
            let parryReCalcTime = Jedi_ReCalcParryTime(ctx, self_, evasionType);
            if (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize]
                < (*world).level.time + parryReCalcTime
            {
                (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                    (*world).level.time + parryReCalcTime;
            }
        }
        evasionType
    }
}

/// Raven `Jedi_SaberBlock`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3143-3372`
pub fn Jedi_SaberBlock(
    ctx: GameContext<'_>,
    saberNum: c_int,
    bladeNum: c_int,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let d_jedi = (*world).cvars.d_JediAI.integer != 0;

        let mut hitloc: vec3_t = [0.0; 3];
        let mut saberTipOld: vec3_t = [0.0; 3];
        let mut saberTip: vec3_t = [0.0; 3];
        let mut top: vec3_t = [0.0; 3];
        let mut bottom: vec3_t = [0.0; 3];
        let mut axisPoint: vec3_t = [0.0; 3];
        let mut saberPoint: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut pointDir: vec3_t = [0.0; 3];
        let mut baseDir: vec3_t = [0.0; 3];
        let mut tipDir: vec3_t = [0.0; 3];
        let mut saberHitPoint: vec3_t = [0.0; 3];
        let pointDist: f32;
        let baseDirPerc: f32;
        let mut dist: f32;
        let bladeLen: f32;
        let mut tr: trace_t = core::mem::zeroed();
        let evasionType: evasionType_t;

        if crate::g_timer::TIMER_Done(ctx, npc, c"parryReCalcTime".as_ptr()) == qfalse {
            //can't do our own re-think of which parry to use yet
            return qfalse;
        }

        if (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] > (*world).level.time {
            //can't move the saber to another position yet
            return qfalse;
        }

        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };
        if (*enemy).health <= 0 || (*enemy).client.is_null() {
            //don't keep blocking him once he's dead (or if not a client)
            return qfalse;
        }
        let saberMins: vec3_t = [-4.0, -4.0, -4.0];
        let saberMaxs: vec3_t = [4.0, 4.0, 4.0];

        let e_blade = &(*enemy_client).saber[saberNum as usize].blade[bladeNum as usize];
        crate::q_math::_VectorMA(
            e_blade.muzzlePointOld,
            e_blade.length,
            e_blade.muzzleDirOld,
            &mut saberTipOld,
        );
        crate::q_math::_VectorMA(
            e_blade.muzzlePoint,
            e_blade.length,
            e_blade.muzzleDir,
            &mut saberTip,
        );

        crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut top);
        top[2] = (*npc).r.absmax[2];
        crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut bottom);
        bottom[2] = (*npc).r.absmin[2];

        dist = crate::g_utils::ShortestLineSegBewteen2LineSegs(
            (*enemy_client).renderInfo.muzzlePoint,
            saberTip,
            bottom,
            top,
            &mut saberPoint,
            &mut axisPoint,
        );
        if dist > (*npc).r.maxs[0] * 5.0 {
            //too far away to actually hit him
            if d_jedi {
                crate::g_main::Com_Printf(
                    cstr(&format!("^1enemy saber dist: {:.2}\n", dist)).as_ptr(),
                );
            }
            crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), -1);
            return qfalse;
        }
        if d_jedi {
            crate::g_main::Com_Printf(cstr(&format!("^2enemy saber dist: {:.2}\n", dist)).as_ptr());
        }

        crate::q_math::_VectorSubtract(
            saberPoint,
            (*enemy_client).renderInfo.muzzlePoint,
            &mut pointDir,
        );
        pointDist = crate::q_math::VectorLength(pointDir);

        bladeLen = (*enemy_client).saber[saberNum as usize].blade[bladeNum as usize].length;

        if bladeLen <= 0.0 {
            baseDirPerc = 0.5f32;
        } else {
            baseDirPerc = pointDist / bladeLen;
        }
        crate::q_math::_VectorSubtract(
            (*enemy_client).renderInfo.muzzlePoint,
            (*enemy_client).renderInfo.muzzlePointOld,
            &mut baseDir,
        );
        crate::q_math::_VectorSubtract(saberTip, saberTipOld, &mut tipDir);
        let baseDir_in = baseDir;
        crate::q_math::_VectorScale(baseDir_in, baseDirPerc, &mut baseDir);
        crate::q_math::_VectorMA(baseDir, 1.0f32 - baseDirPerc, tipDir, &mut dir);
        crate::q_math::_VectorMA(saberPoint, 200.0, dir, &mut hitloc);

        //get the actual point of impact
        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &saberPoint as *const vec3_t,
                &saberMins as *const vec3_t,
                &saberMaxs as *const vec3_t,
                &hitloc as *const vec3_t,
                (*enemy).s.number,
                CONTENTS_BODY,
            ),
        );
        if tr.allsolid != (qfalse) as u8 || tr.startsolid != (qfalse) as u8 || tr.fraction >= 1.0f32 {
            //estimate
            let mut dir2Me: vec3_t = [0.0; 3];
            crate::q_math::_VectorSubtract(axisPoint, saberPoint, &mut dir2Me);
            dist = crate::q_math::VectorNormalize(&mut dir2Me);
            if crate::q_math::_DotProduct(dir, dir2Me) < 0.2f32 {
                //saber is not swinging in my direction
                crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), -1);
                return qfalse;
            }
            let mut hitloc_out: vec3_t = [0.0; 3];
            crate::g_utils::ShortestLineSegBewteen2LineSegs(
                saberPoint,
                hitloc,
                bottom,
                top,
                &mut saberHitPoint,
                &mut hitloc_out,
            );
            hitloc = hitloc_out;
        } else {
            crate::q_math::_VectorCopy(tr.endpos, &mut hitloc);
        }

        if d_jedi {
            crate::ai_wpnav::G_TestLine(ctx, saberPoint, hitloc, 0x0000ff, FRAMETIME as c_int);
        }

        evasionType = Jedi_SaberBlockGo(
            ctx,
            npc,
            &mut (*world).globals.ucmd,
            hitloc,
            dir,
            core::ptr::null_mut(),
            dist,
        );
        if evasionType != evasionType_t::EVASION_DODGE {
            //we did block (not dodge)
            let parryReCalcTime: c_int;

            if (*client).ps.saberInFlight == qfalse {
                crate::w_saber::WP_ActivateSaber(ctx, npc);
            }

            parryReCalcTime = Jedi_ReCalcParryTime(ctx, npc, evasionType);
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"parryReCalcTime".as_ptr(),
                (*world).bg_state.rng.Q_irand(0, parryReCalcTime),
            );
            if d_jedi {
                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "Keep parry choice until: {}\n",
                        (*world).level.time + parryReCalcTime
                    ))
                    .as_ptr(),
                );
            }

            if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse {
                if (*client).NPC_class == CLASS_TAVION {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"parryTime".as_ptr(),
                        (*world).bg_state.rng.Q_irand(
                            parryReCalcTime / 2,
                            (parryReCalcTime as f32 * 1.5) as c_int,
                        ),
                    );
                } else if (*npc_info).rank >= RANK_LT_JG as c_int {
                    crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), parryReCalcTime);
                } else {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"parryTime".as_ptr(),
                        (*world).bg_state.rng.Q_irand(1, 2) * parryReCalcTime,
                    );
                }
            }
        } else {
            let mut dodgeTime = (*client).ps.torsoTimer;
            if (*npc_info).rank > RANK_LT_COMM as c_int && (*client).NPC_class != CLASS_DESANN {
                dodgeTime -= 200;
            }
            crate::g_timer::TIMER_Set(ctx, npc, c"parryReCalcTime".as_ptr(), dodgeTime);
            crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), dodgeTime);
        }
        let _ = pointDist;
        qtrue
    }
}

/// Raven `Jedi_EvasionSaber`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3380-3666`
pub fn Jedi_EvasionSaber(
    ctx: GameContext<'_>,
    enemy_movedir: vec3_t,
    enemy_dist: f32,
    enemy_dir: vec3_t,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let d_jedi = (*world).cvars.d_JediAI.integer != 0;

        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };

        let mut dirEnemy2Me: vec3_t = [0.0; 3];
        let mut evasionChance: c_int = 30;
        let mut enemy_attacking: qboolean = qfalse;
        let mut throwing_saber: qboolean = qfalse;
        let mut shooting_lightning: qboolean = qfalse;

        if (*enemy).client.is_null() {
            return;
        } else if !(*enemy).client.is_null()
            && (*enemy).s.weapon == WP_SABER as c_int
            && (*enemy_client).ps.saberLockTime > (*world).level.time
        {
            //don't try to block/evade an enemy who is in a saberLock
            return;
        } else if (*client).ps.saberEventFlags & SEF_LOCK_WON as c_int != 0
            && (*enemy).painDebounceTime > (*world).level.time
        {
            //pressing the advantage of winning a saber lock
            return;
        }

        if (*enemy_client).ps.saberInFlight != qfalse
            && crate::g_timer::TIMER_Done(ctx, npc, c"taunting".as_ptr()) == qfalse
        {
            //if he's throwing his saber, stop taunting
            crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), -(*world).level.time);
            if (*client).ps.saberInFlight == qfalse {
                crate::w_saber::WP_ActivateSaber(ctx, npc);
            }
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse {
            if (*client).ps.saberBlocked != BLOCKED_ATK_BOUNCE as c_int
                && (*client).ps.saberBlocked != BLOCKED_PARRY_BROKEN as c_int
            {
                //wasn't blocked myself
                (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
            }
        }

        if (*enemy_client).ps.weaponTime != 0
            && (*enemy_client).ps.weaponstate == (WEAPON_FIRING) as i32
        {
            if (*client).ps.saberInFlight == qfalse && Jedi_SaberBlock(ctx, 0, 0) != qfalse {
                return;
            }
        }

        crate::q_math::_VectorSubtract(
            (*npc).r.currentOrigin,
            (*enemy).r.currentOrigin,
            &mut dirEnemy2Me,
        );
        crate::q_math::VectorNormalize(&mut dirEnemy2Me);

        if (*enemy_client).ps.weaponTime != 0
            && (*enemy_client).ps.weaponstate == (WEAPON_FIRING) as i32
        {
            //enemy is attacking
            enemy_attacking = qtrue;
            evasionChance = 90;
        }

        if (*enemy_client).ps.fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
            //enemy is shooting lightning
            enemy_attacking = qtrue;
            shooting_lightning = qtrue;
            evasionChance = 50;
        }

        if (*enemy_client).ps.saberInFlight != qfalse
            && (*enemy_client).ps.saberEntityNum != ENTITYNUM_NONE
            && (*enemy_client).ps.saberEntityState != SES_RETURNING as c_int
        {
            enemy_attacking = qtrue;
            throwing_saber = qtrue;
        }

        if (*world).bg_state.rng.Q_irand(0, 100) < evasionChance {
            //check to see if he's coming at me
            let facingAmt: f32;
            if crate::q_math::VectorCompare(enemy_movedir, vec3_origin) != qfalse
                || shooting_lightning != qfalse
                || throwing_saber != qfalse
            {
                //he's not moving (or ranged attack), see if he's facing me
                let mut enemy_fwd: vec3_t = [0.0; 3];
                crate::q_math::AngleVectors(
                    (*enemy_client).ps.viewangles,
                    Some(&mut enemy_fwd),
                    None,
                    None,
                );
                facingAmt = crate::q_math::_DotProduct(enemy_fwd, dirEnemy2Me);
            } else {
                //he's moving
                facingAmt = crate::q_math::_DotProduct(enemy_movedir, dirEnemy2Me);
            }

            if (*world).bg_state.rng.flrand(0.25, 1.0) < facingAmt {
                //coming at/facing me!
                let mut whichDefense: c_int = 0;
                if (*client).ps.weaponTime != 0
                    || (*client).ps.saberInFlight != qfalse
                    || (*client).NPC_class == CLASS_BOBAFETT
                {
                    //I'm attacking or recovering, can only strafe/jump
                    if (*world).bg_state.rng.Q_irand(0, 10) < (*npc_info).stats.aggression {
                        return;
                    }
                    whichDefense = 100;
                } else {
                    if shooting_lightning != qfalse {
                        whichDefense = 100;
                    } else if throwing_saber != qfalse {
                        //he's thrown his saber!  See if it's coming at me
                        let saberDist: f32;
                        let mut saberDir2Me: vec3_t = [0.0; 3];
                        let mut saberMoveDir: vec3_t = [0.0; 3];
                        let saber = ge.add((*enemy_client).ps.saberEntityNum as usize);
                        crate::q_math::_VectorSubtract(
                            (*npc).r.currentOrigin,
                            (*saber).r.currentOrigin,
                            &mut saberDir2Me,
                        );
                        saberDist = crate::q_math::VectorNormalize(&mut saberDir2Me);
                        crate::q_math::_VectorCopy((*saber).s.pos.trDelta, &mut saberMoveDir);
                        crate::q_math::VectorNormalize(&mut saberMoveDir);
                        if (*world).bg_state.rng.Q_irand(0, 3) == 0 {
                            Jedi_Aggression(npc, 1);
                        }
                        if crate::q_math::_DotProduct(saberMoveDir, saberDir2Me) > 0.5 {
                            //it's heading towards me
                            if saberDist < 100.0 {
                                whichDefense = (*world).bg_state.rng.Q_irand(3, 6);
                            } else if saberDist < 200.0 {
                                whichDefense = (*world).bg_state.rng.Q_irand(0, 8);
                            }
                        }
                    }
                    if whichDefense != 0 {
                        //already chose one
                    } else if enemy_dist > 80.0 || enemy_attacking == qfalse {
                        //he's pretty far, or not swinging, just strafe
                        if crate::q_math::VectorCompare(enemy_movedir, vec3_origin) != qfalse {
                            return;
                        }
                        if (*world).bg_state.rng.Q_irand(0, 10) < (*npc_info).stats.aggression {
                            return;
                        }
                        whichDefense = 100;
                    } else {
                        //he's getting close and swinging at me
                        let mut fwd: vec3_t = [0.0; 3];
                        crate::q_math::AngleVectors(
                            (*client).ps.viewangles,
                            Some(&mut fwd),
                            None,
                            None,
                        );
                        if crate::q_math::_DotProduct(enemy_dir, fwd) < 0.5 {
                            whichDefense = (*world).bg_state.rng.Q_irand(5, 16);
                        } else if enemy_dist < 56.0 {
                            whichDefense =
                                (*world).bg_state.rng.Q_irand((*npc_info).stats.aggression, 12);
                        } else {
                            whichDefense = (*world).bg_state.rng.Q_irand(2, 16);
                        }
                    }
                }

                if whichDefense >= 4 && whichDefense <= 12 {
                    //would try to block
                    if (*client).ps.saberInFlight != qfalse {
                        whichDefense = 100;
                    }
                }

                match whichDefense {
                    0 | 1 | 2 | 3 => {
                        //use jedi force push?
                        if ((*npc_info).rank == RANK_ENSIGN as c_int
                            || (*npc_info).rank > RANK_LT_JG as c_int)
                            && crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse
                        {
                            crate::w_force::ForceThrow(ctx, npc, qfalse);
                        }
                    }
                    4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 => {
                        //try to parry the blow
                        Jedi_SaberBlock(ctx, 0, 0);
                    }
                    _ => {
                        //Evade!
                        if (*world).bg_state.rng.Q_irand(0, 5) == 0
                            || Jedi_Strafe(ctx, 300, 1000, 0, 1000, qfalse) == qfalse
                        {
                            //if couldn't strafe, try a different kind of evasion...
                            if shooting_lightning != qfalse
                                || throwing_saber != qfalse
                                || enemy_dist < 80.0
                            {
                                if shooting_lightning != qfalse
                                    || ((*world).bg_state.rng.Q_irand(0, 2) == 0
                                        && (*npc_info).stats.aggression < 4
                                        && crate::g_timer::TIMER_Done(
                                            ctx,
                                            npc,
                                            c"parryTime".as_ptr(),
                                        ) != qfalse)
                                {
                                    if ((*npc_info).rank == RANK_ENSIGN as c_int
                                        || (*npc_info).rank > RANK_LT_JG as c_int)
                                        && shooting_lightning == qfalse
                                        && (*world).bg_state.rng.Q_irand(0, 2) != 0
                                    {
                                        crate::w_force::ForceThrow(ctx, npc, qfalse);
                                    } else if ((*npc_info).rank == RANK_CREWMAN as c_int
                                        || (*npc_info).rank > RANK_LT_JG as c_int)
                                        && ((*npc_info).scriptFlags & SCF_NO_ACROBATICS) == 0
                                        && (*client).ps.fd.forceRageRecoveryTime
                                            < (*world).level.time
                                        && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                                        && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps)
                                            == qfalse
                                    {
                                        (*client).ps.fd.forceJumpCharge = 480.0;
                                        crate::g_timer::TIMER_Set(
                                            ctx,
                                            npc,
                                            c"jumpChaseDebounce".as_ptr(),
                                            (*world).bg_state.rng.Q_irand(2000, 5000),
                                        );
                                        if (*world).bg_state.rng.Q_irand(0, 2) != 0 {
                                            (*world).globals.ucmd.forwardmove = 127;
                                            (*client).ps.moveDir = [0.0, 0.0, 0.0];
                                        } else {
                                            (*world).globals.ucmd.forwardmove = -127;
                                            (*client).ps.moveDir = [0.0, 0.0, 0.0];
                                        }
                                        if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                                            (*client).ps.saberBlocked = BLOCKED_LOWER_RIGHT as c_int;
                                        } else {
                                            (*client).ps.saberBlocked = BLOCKED_LOWER_LEFT as c_int;
                                        }
                                    }
                                } else if enemy_attacking != qfalse {
                                    Jedi_SaberBlock(ctx, 0, 0);
                                }
                            }
                        } else {
                            //strafed
                            if d_jedi {
                                crate::g_main::Com_Printf(cstr("def strafe\n").as_ptr());
                            }
                            if ((*npc_info).scriptFlags & SCF_NO_ACROBATICS) == 0
                                && (*client).ps.fd.forceRageRecoveryTime < (*world).level.time
                                && ((*client).ps.fd.forcePowersActive & (1 << FP_RAGE)) == 0
                                && ((*npc_info).rank == RANK_CREWMAN as c_int
                                    || (*npc_info).rank > RANK_LT_JG as c_int)
                                && crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                                && (*world).bg_state.rng.Q_irand(0, 5) == 0
                            {
                                if (*client).NPC_class == CLASS_BOBAFETT {
                                    (*client).ps.fd.forceJumpCharge = 280.0;
                                } else {
                                    (*client).ps.fd.forceJumpCharge = 320.0;
                                }
                                crate::g_timer::TIMER_Set(
                                    ctx,
                                    npc,
                                    c"jumpChaseDebounce".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(2000, 5000),
                                );
                            }
                        }
                    }
                }

                //turn off slow walking no matter what
                crate::g_timer::TIMER_Set(ctx, npc, c"walking".as_ptr(), -(*world).level.time);
                crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), -(*world).level.time);
            }
        }
    }
}

/// Raven `Jedi_FindEnemyInCone`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3686-3761`
pub fn Jedi_FindEnemyInCone(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    fallback: *mut gentity_t,
    minDot: f32,
) -> *mut gentity_t {
    unsafe {
        let world = ctx.world;
        let ge = (*world).g_entities.as_mut_ptr();
        let self_client = (*self_).client as *mut gclient_t;

        let mut forward: vec3_t = [0.0; 3];
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];
        let mut dist: f32;
        let mut bestDist: f32 = Q3_INFINITE as f32;
        let mut enemy: *mut gentity_t = fallback;
        let mut check: *mut gentity_t;
        let mut entityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
        let mut e: c_int;
        let numListedEntities: c_int;
        let mut tr: trace_t = core::mem::zeroed();

        if (*self_).client.is_null() {
            return enemy;
        }

        crate::q_math::AngleVectors((*self_client).ps.viewangles, Some(&mut forward), None, None);

        e = 0;
        while e < 3 {
            mins[e as usize] = (*self_).r.currentOrigin[e as usize] - 1024.0;
            maxs[e as usize] = (*self_).r.currentOrigin[e as usize] + 1024.0;
            e += 1;
        }
        numListedEntities = crate::trap::EntitiesInBox(
            ctx.engine,
            mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                entityList.as_mut_ptr(),
                MAX_GENTITIES as c_int,
            ),
        );

        e = 0;
        while e < numListedEntities {
            check = ge.add(entityList[e as usize] as usize);
            let check_client = (*check).client as *mut gclient_t;
            if check == self_ {
                e += 1;
                continue;
            }
            if (*check).inuse == qfalse {
                e += 1;
                continue;
            }
            if (*check).client.is_null() {
                e += 1;
                continue;
            }
            if (*check_client).playerTeam != (*self_client).enemyTeam {
                e += 1;
                continue;
            }
            if (*check).health <= 0 {
                e += 1;
                continue;
            }

            if crate::trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &(*check).r.currentOrigin as *const vec3_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                ),
            ) == 0
            {
                e += 1;
                continue;
            }

            crate::q_math::_VectorSubtract(
                (*check).r.currentOrigin,
                (*self_).r.currentOrigin,
                &mut dir,
            );
            dist = crate::q_math::VectorNormalize(&mut dir);

            if crate::q_math::_DotProduct(dir, forward) < minDot {
                e += 1;
                continue;
            }

            crate::trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*self_).r.currentOrigin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &(*check).r.currentOrigin as *const vec3_t,
                    (*self_).s.number,
                    MASK_SHOT,
                ),
            );
            if tr.fraction < 1.0f32 && tr.entityNum != ((*check).s.number) as i16 {
                e += 1;
                continue;
            }

            if dist < bestDist {
                dist = bestDist;
                enemy = check;
            }
            let _ = dist;
            e += 1;
        }
        enemy
    }
}

/// Raven `Jedi_SetEnemyInfo`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3763-3796`
pub fn Jedi_SetEnemyInfo(
    ctx: GameContext<'_>,
    enemy_dest: &mut vec3_t,
    enemy_dir: &mut vec3_t,
    enemy_dist: *mut f32,
    enemy_movedir: &mut vec3_t,
    enemy_movespeed: *mut f32,
    prediction: c_int,
) {
    // enemy_dest/enemy_dir/enemy_movedir are written out-params (`&mut`); the
    // staged skeleton carried the stale by-value shape — updated to match.
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let ge = (*world).g_entities.as_mut_ptr();
        if npc.is_null() || (*npc).enemy.is_none() {
            //no valid enemy
            return;
        }
        let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
        let enemy_client = (*enemy).client as *mut gclient_t;
        let npc_client = (*npc).client as *mut gclient_t;
        if (*enemy).client.is_null() {
            *enemy_movedir = [0.0, 0.0, 0.0];
            *enemy_movespeed = 0.0;
            crate::q_math::_VectorCopy((*enemy).r.currentOrigin, enemy_dest);
            enemy_dest[2] += (*enemy).r.mins[2] + 24.0;
            let dest = *enemy_dest;
            crate::q_math::_VectorSubtract(dest, (*npc).r.currentOrigin, enemy_dir);
            *enemy_dist = crate::q_math::VectorNormalize(enemy_dir);
        } else {
            //see where enemy is headed
            crate::q_math::_VectorCopy((*enemy_client).ps.velocity, enemy_movedir);
            *enemy_movespeed = crate::q_math::VectorNormalize(enemy_movedir);
            //figure out where he'll be, say, 3 frames from now
            let mvd = *enemy_movedir;
            crate::q_math::_VectorMA(
                (*enemy).r.currentOrigin,
                *enemy_movespeed * 0.001 * prediction as f32,
                mvd,
                enemy_dest,
            );
            let dest = *enemy_dest;
            crate::q_math::_VectorSubtract(dest, (*npc).r.currentOrigin, enemy_dir);
            *enemy_dist = crate::q_math::VectorNormalize(enemy_dir)
                - ((*npc_client).saber[0].blade[0].lengthMax + (*npc).r.maxs[0] * 1.5 + 16.0);
        }
    }
}

/// Raven `Jedi_FaceEnemy`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3799-3874`
pub fn Jedi_FaceEnemy(
    ctx: GameContext<'_>,
    doPitch: qboolean,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        let mut enemy_eyes: vec3_t = [0.0; 3];
        let mut eyes: vec3_t = [0.0; 3];
        let mut angles: vec3_t = [0.0; 3];

        if npc.is_null() {
            return;
        }
        if (*npc).enemy.is_none() {
            return;
        }
        let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
        let enemy_client = (*enemy).client as *mut gclient_t;

        if (*client).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0
            && (*client).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
        {
            //don't update?
            (*npc_info).desiredPitch = (*client).ps.viewangles[PITCH as usize];
            (*npc_info).desiredYaw = (*client).ps.viewangles[YAW as usize];
            return;
        }
        crate::NPC_utils::CalcEntitySpot(ctx, npc, SPOT_HEAD, &mut eyes);
        crate::NPC_utils::CalcEntitySpot(ctx, enemy, SPOT_HEAD, &mut enemy_eyes);

        if (*client).NPC_class == CLASS_BOBAFETT
            && crate::g_timer::TIMER_Done(ctx, npc, c"flameTime".as_ptr()) != qfalse
            && (*npc).s.weapon != WP_NONE as c_int
            && (*npc).s.weapon != WP_DISRUPTOR as c_int
            && ((*npc).s.weapon != WP_ROCKET_LAUNCHER as c_int
                || ((*npc_info).scriptFlags & SCF_ALT_FIRE) == 0)
            && (*npc).s.weapon != WP_THERMAL as c_int
            && (*npc).s.weapon != WP_TRIP_MINE as c_int
            && (*npc).s.weapon != WP_DET_PACK as c_int
            && (*npc).s.weapon != WP_STUN_BATON as c_int
        {
            //boba leads his enemy
            if (*npc).health < ((*client).pers.maxHealth as f32 * 0.5f32) as c_int {
                //lead
                let missileSpeed = crate::g_weapon::WP_SpeedOfMissileForWeapon(
                    (*npc).s.weapon,
                    if (*npc_info).scriptFlags & SCF_ALT_FIRE != 0 { qtrue } else { qfalse },
                );
                if missileSpeed != 0.0 {
                    let mut eDist = crate::q_math::Distance(eyes, enemy_eyes);
                    eDist /= missileSpeed; //How many seconds it will take to get to the enemy
                    let ee = enemy_eyes;
                    crate::q_math::_VectorMA(
                        ee,
                        eDist * (*world).bg_state.rng.flrand(0.95f32, 1.25f32),
                        (*enemy_client).ps.velocity,
                        &mut enemy_eyes,
                    );
                }
            }
        }

        //Find the desired angles
        if (*client).ps.saberInFlight == qfalse
            && ((*client).ps.legsAnim == BOTH_A2_STABBACK1 as c_int
                || (*client).ps.legsAnim == BOTH_CROUCHATTACKBACK1 as c_int
                || (*client).ps.legsAnim == BOTH_ATTACK_BACK as c_int)
        {
            //point *away*
            crate::g_utils::GetAnglesForDirection(enemy_eyes, eyes, &mut angles);
        } else {
            //point towards him
            crate::g_utils::GetAnglesForDirection(eyes, enemy_eyes, &mut angles);
        }

        (*npc_info).desiredYaw = crate::q_math::AngleNormalize360(angles[YAW as usize]);

        if doPitch != qfalse {
            (*npc_info).desiredPitch = crate::q_math::AngleNormalize360(angles[PITCH as usize]);
            if (*client).ps.saberInFlight != qfalse {
                //tilt down a little
                (*npc_info).desiredPitch += 10.0;
            }
        }
    }
}

/// Raven `Jedi_DebounceDirectionChanges`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:3876-4005`
pub fn Jedi_DebounceDirectionChanges(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let client = (*npc).client as *mut gclient_t;
        //Time-debounce changes in forward/back dir
        if (*world).globals.ucmd.forwardmove > 0 {
            if crate::g_timer::TIMER_Done(ctx, npc, c"moveback".as_ptr()) == qfalse
                || crate::g_timer::TIMER_Done(ctx, npc, c"movenone".as_ptr()) == qfalse
            {
                (*world).globals.ucmd.forwardmove = 0;
                if (*world).globals.ucmd.rightmove > 0 {
                    (*world).globals.ucmd.rightmove = 127;
                } else if (*world).globals.ucmd.rightmove < 0 {
                    (*world).globals.ucmd.rightmove = -127;
                }
                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                crate::g_timer::TIMER_Set(ctx, npc, c"moveback".as_ptr(), -(*world).level.time);
                if crate::g_timer::TIMER_Done(ctx, npc, c"movenone".as_ptr()) != qfalse {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"movenone".as_ptr(),
                        (*world).bg_state.rng.Q_irand(1000, 2000),
                    );
                }
            } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveforward".as_ptr()) != qfalse {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"moveforward".as_ptr(),
                    (*world).bg_state.rng.Q_irand(500, 2000),
                );
            }
        } else if (*world).globals.ucmd.forwardmove < 0 {
            if crate::g_timer::TIMER_Done(ctx, npc, c"moveforward".as_ptr()) == qfalse
                || crate::g_timer::TIMER_Done(ctx, npc, c"movenone".as_ptr()) == qfalse
            {
                (*world).globals.ucmd.forwardmove = 0;
                if (*world).globals.ucmd.rightmove > 0 {
                    (*world).globals.ucmd.rightmove = 127;
                } else if (*world).globals.ucmd.rightmove < 0 {
                    (*world).globals.ucmd.rightmove = -127;
                }
                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                crate::g_timer::TIMER_Set(ctx, npc, c"moveforward".as_ptr(), -(*world).level.time);
                if crate::g_timer::TIMER_Done(ctx, npc, c"movenone".as_ptr()) != qfalse {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"movenone".as_ptr(),
                        (*world).bg_state.rng.Q_irand(1000, 2000),
                    );
                }
            } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveback".as_ptr()) != qfalse {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"moveback".as_ptr(),
                    (*world).bg_state.rng.Q_irand(250, 1000),
                );
            }
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveforward".as_ptr()) == qfalse {
            (*world).globals.ucmd.forwardmove = 127;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveback".as_ptr()) == qfalse {
            (*world).globals.ucmd.forwardmove = -127;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        }
        //Time-debounce changes in right/left dir
        if (*world).globals.ucmd.rightmove > 0 {
            if crate::g_timer::TIMER_Done(ctx, npc, c"moveleft".as_ptr()) == qfalse
                || crate::g_timer::TIMER_Done(ctx, npc, c"movecenter".as_ptr()) == qfalse
            {
                (*world).globals.ucmd.rightmove = 0;
                if (*world).globals.ucmd.forwardmove > 0 {
                    (*world).globals.ucmd.forwardmove = 127;
                } else if (*world).globals.ucmd.forwardmove < 0 {
                    (*world).globals.ucmd.forwardmove = -127;
                }
                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                crate::g_timer::TIMER_Set(ctx, npc, c"moveleft".as_ptr(), -(*world).level.time);
                if crate::g_timer::TIMER_Done(ctx, npc, c"movecenter".as_ptr()) != qfalse {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"movecenter".as_ptr(),
                        (*world).bg_state.rng.Q_irand(1000, 2000),
                    );
                }
            } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveright".as_ptr()) != qfalse {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"moveright".as_ptr(),
                    (*world).bg_state.rng.Q_irand(250, 1500),
                );
            }
        } else if (*world).globals.ucmd.rightmove < 0 {
            if crate::g_timer::TIMER_Done(ctx, npc, c"moveright".as_ptr()) == qfalse
                || crate::g_timer::TIMER_Done(ctx, npc, c"movecenter".as_ptr()) == qfalse
            {
                (*world).globals.ucmd.rightmove = 0;
                if (*world).globals.ucmd.forwardmove > 0 {
                    (*world).globals.ucmd.forwardmove = 127;
                } else if (*world).globals.ucmd.forwardmove < 0 {
                    (*world).globals.ucmd.forwardmove = -127;
                }
                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                crate::g_timer::TIMER_Set(ctx, npc, c"moveright".as_ptr(), -(*world).level.time);
                if crate::g_timer::TIMER_Done(ctx, npc, c"movecenter".as_ptr()) != qfalse {
                    crate::g_timer::TIMER_Set(
                        ctx,
                        npc,
                        c"movecenter".as_ptr(),
                        (*world).bg_state.rng.Q_irand(1000, 2000),
                    );
                }
            } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveleft".as_ptr()) != qfalse {
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"moveleft".as_ptr(),
                    (*world).bg_state.rng.Q_irand(250, 1500),
                );
            }
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveright".as_ptr()) == qfalse {
            (*world).globals.ucmd.rightmove = 127;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        } else if crate::g_timer::TIMER_Done(ctx, npc, c"moveleft".as_ptr()) == qfalse {
            (*world).globals.ucmd.rightmove = -127;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        }
    }
}

/// Raven `Jedi_TimersApply`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4007-4065`
pub fn Jedi_TimersApply(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let client = (*npc).client as *mut gclient_t;

        if (*world).globals.ucmd.rightmove == 0 {
            //only if not already strafing
            if crate::g_timer::TIMER_Done(ctx, npc, c"strafeLeft".as_ptr()) == qfalse {
                if (*npc_info).desiredYaw > (*client).ps.viewangles[YAW as usize] + 60.0 {
                    //we want to turn left, don't apply the strafing
                } else {
                    (*world).globals.ucmd.rightmove = -127;
                    (*client).ps.moveDir = [0.0, 0.0, 0.0];
                }
            } else if crate::g_timer::TIMER_Done(ctx, npc, c"strafeRight".as_ptr()) == qfalse {
                if (*npc_info).desiredYaw < (*client).ps.viewangles[YAW as usize] - 60.0 {
                    //we want to turn right, don't apply the strafing
                } else {
                    (*world).globals.ucmd.rightmove = 127;
                    (*client).ps.moveDir = [0.0, 0.0, 0.0];
                }
            }
        }

        Jedi_DebounceDirectionChanges(ctx);

        //use careful anim/slower movement if not already moving
        if (*world).globals.ucmd.forwardmove == 0
            && crate::g_timer::TIMER_Done(ctx, npc, c"walking".as_ptr()) == qfalse
        {
            (*world).globals.ucmd.buttons |= BUTTON_WALKING;
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"taunting".as_ptr()) == qfalse {
            (*world).globals.ucmd.buttons |= BUTTON_WALKING;
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"gripping".as_ptr()) == qfalse {
            (*world).globals.ucmd.buttons |= BUTTON_FORCEGRIP;
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"draining".as_ptr()) == qfalse {
            (*world).globals.ucmd.buttons |= BUTTON_FORCE_DRAIN;
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"holdLightning".as_ptr()) == qfalse {
            //hold down the lightning key
            (*world).globals.ucmd.buttons |= BUTTON_FORCE_LIGHTNING;
        }
    }
}

/// Raven `Jedi_CombatTimersUpdate`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4067-4273`
pub fn Jedi_CombatTimersUpdate(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let d_jedi = (*world).cvars.d_JediAI.integer != 0;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };

        if crate::g_timer::TIMER_Done(ctx, npc, c"roamTime".as_ptr()) != qfalse {
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"roamTime".as_ptr(),
                (*world).bg_state.rng.Q_irand(2000, 5000),
            );
            //okay, now mess with agression
            if (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
                Jedi_Aggression(npc, (*world).bg_state.rng.Q_irand(0, 3));
            } else if (*client).ps.fd.forceRageRecoveryTime > (*world).level.time {
                Jedi_Aggression(npc, (*world).bg_state.rng.Q_irand(0, -2));
            }
            if !enemy.is_null() && !(*enemy).client.is_null() {
                match (*enemy_client).ps.weapon {
                    w if w == WP_SABER as c_int => {
                        //If enemy has a lightsaber, always close in
                        if crate::bg_pmove::BG_SabersOff(&mut (*enemy_client).ps) != qfalse {
                            Jedi_Aggression(npc, 2);
                        } else {
                            Jedi_Aggression(npc, 1);
                        }
                    }
                    w if w == WP_BLASTER as c_int
                        || w == WP_BRYAR_PISTOL as c_int
                        || w == WP_DISRUPTOR as c_int
                        || w == WP_BOWCASTER as c_int
                        || w == WP_REPEATER as c_int
                        || w == WP_DEMP2 as c_int
                        || w == WP_FLECHETTE as c_int
                        || w == WP_ROCKET_LAUNCHER as c_int =>
                    {
                        if (*enemy).attackDebounceTime < (*world).level.time {
                            Jedi_Aggression(npc, 1);
                        }
                        if enemy_dist < 256 {
                            Jedi_Aggression(npc, 1);
                        }
                    }
                    _ => {}
                }
            }
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"noStrafe".as_ptr()) != qfalse
            && crate::g_timer::TIMER_Done(ctx, npc, c"strafeLeft".as_ptr()) != qfalse
            && crate::g_timer::TIMER_Done(ctx, npc, c"strafeRight".as_ptr()) != qfalse
        {
            if (*world).bg_state.rng.Q_irand(0, 4) == 0 {
                //start a strafe
                if Jedi_Strafe(ctx, 1000, 3000, 0, 4000, qtrue) != qfalse {
                    if d_jedi {
                        crate::g_main::Com_Printf(cstr("off strafe\n").as_ptr());
                    }
                }
            } else {
                //postpone any strafing for a while
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"noStrafe".as_ptr(),
                    (*world).bg_state.rng.Q_irand(1000, 3000),
                );
            }
        }

        if (*client).ps.saberEventFlags != 0 {
            //some kind of saber combat event is still pending
            let mut newFlags = (*client).ps.saberEventFlags;
            if (*client).ps.saberEventFlags & SEF_PARRIED as c_int != 0 {
                //parried
                crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), -1);
                if !enemy.is_null()
                    && crate::bg_panimate::PM_SaberInKnockaway((*enemy_client).ps.saberMove)
                        != qfalse
                {
                    //advance!
                    Jedi_Aggression(npc, 1);
                    Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel - 1);
                } else {
                    if (*world).bg_state.rng.Q_irand(0, 1) == 0 {
                        Jedi_Aggression(npc, -1);
                    }
                    if (*world).bg_state.rng.Q_irand(0, 1) == 0 {
                        Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel - 1);
                    }
                }
                newFlags &= !SEF_PARRIED as c_int;
            }
            if (*client).ps.weaponTime == 0 && ((*client).ps.saberEventFlags & SEF_HITENEMY as c_int) != 0 {
                //we hit our enemy last time we swung, drop our aggression
                if (*world).bg_state.rng.Q_irand(0, 1) == 0 {
                    Jedi_Aggression(npc, -1);
                    // PORT-NOTE(jediSpeechDebounceTime): `()` placeholder field indexed by team.
                    if (*world).bg_state.rng.Q_irand(0, 3) == 0
                        && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
                        && (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize]
                            < (*world).level.time
                        && (*npc).painDebounceTime < (*world).level.time - 1000
                    {
                        crate::NPC_sounds::G_AddVoiceEvent(
                            ctx,
                            npc,
                            (*world).bg_state.rng.Q_irand(
                                entity_event_t::EV_GLOAT1 as c_int,
                                entity_event_t::EV_GLOAT3 as c_int,
                            ),
                            3000,
                        );
                        (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 3000;
                        (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                            (*world).level.time + 3000;
                    }
                }
                if (*world).bg_state.rng.Q_irand(0, 2) == 0 {
                    Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel + 1);
                }
                newFlags &= !SEF_HITENEMY as c_int;
            }
            if (*client).ps.saberEventFlags & SEF_BLOCKED as c_int != 0 {
                //was blocked whilst attacking
                if crate::bg_saber::PM_SaberInBrokenParry((*client).ps.saberMove) != qfalse
                    || (*client).ps.saberBlocked == BLOCKED_PARRY_BROKEN as c_int
                {
                    if (*client).ps.saberInFlight != qfalse {
                        Jedi_Aggression(npc, -5);
                    } else {
                        Jedi_Aggression(npc, -2);
                    }
                    Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel + 1);
                } else {
                    if (*world).bg_state.rng.Q_irand(0, 2) == 0 {
                        Jedi_Aggression(npc, -1);
                    }
                    if (*world).bg_state.rng.Q_irand(0, 1) == 0 {
                        Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel + 1);
                    }
                }
                newFlags &= !SEF_BLOCKED as c_int;
            }
            if (*client).ps.saberEventFlags & SEF_DEFLECTED as c_int != 0 {
                //deflected a shot
                newFlags &= !SEF_DEFLECTED as c_int;
                if (*world).bg_state.rng.Q_irand(0, 3) == 0 {
                    Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel - 1);
                }
            }
            if (*client).ps.saberEventFlags & SEF_HITWALL as c_int != 0 {
                //hit a wall
                newFlags &= !SEF_HITWALL as c_int;
            }
            if (*client).ps.saberEventFlags & SEF_HITOBJECT as c_int != 0 {
                //hit some other damagable object
                if (*world).bg_state.rng.Q_irand(0, 3) == 0 {
                    Jedi_AdjustSaberAnimLevel(ctx, npc, (*client).ps.fd.saberAnimLevel - 1);
                }
                newFlags &= !SEF_HITOBJECT as c_int;
            }
            (*client).ps.saberEventFlags = newFlags;
        }
    }
}

/// Raven `Jedi_CombatIdle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4275-4337`
pub fn Jedi_CombatIdle(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let client = (*npc).client as *mut gclient_t;

        if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) == qfalse {
            return;
        }
        if (*client).ps.saberInFlight != qfalse {
            return;
        }
        if (*client).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
            || (*client).ps.fd.forceRageRecoveryTime > (*world).level.time
        {
            //never taunt while raging or recovering from rage
            return;
        }
        if enemy_dist >= 64 {
            let mut chance = 20;
            if (*client).NPC_class == CLASS_SHADOWTROOPER {
                chance = 10;
            }
            if (*world).bg_state.rng.Q_irand(2, chance) < (*npc_info).stats.aggression {
                if crate::g_timer::TIMER_Done(ctx, npc, c"chatter".as_ptr()) != qfalse
                    && (*client).ps.forceHandExtend == HANDEXTEND_NONE as c_int
                {
                    if enemy_dist > 200
                        && (*client).NPC_class != CLASS_BOBAFETT
                        && (*client).ps.saberHolstered == 0
                        && (*world).bg_state.rng.Q_irand(0, 5) == 0
                    {
                        //taunt even more, turn off the saber
                        crate::w_saber::WP_DeactivateSaber(ctx, npc, qfalse);
                        (*npc_info).stats.aggression = 3;
                        if (*client).playerTeam != NPCTEAM_PLAYER as c_int
                            && (*world).bg_state.rng.Q_irand(0, 1) == 0
                        {
                            (*client).ps.forceHandExtend = HANDEXTEND_JEDITAUNT as c_int;
                            (*client).ps.forceHandExtendTime = (*world).level.time + 5000;

                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"chatter".as_ptr(),
                                (*world).bg_state.rng.Q_irand(5000, 10000),
                            );
                            crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), 5500);
                        } else {
                            Jedi_BattleTaunt(ctx);
                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"taunting".as_ptr(),
                                (*world).bg_state.rng.Q_irand(5000, 10000),
                            );
                        }
                    } else if Jedi_BattleTaunt(ctx) != qfalse {
                        //FIXME: pick some anims
                    }
                }
            }
        }
    }
}

/// Raven `Jedi_AttackDecide`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4339-4467`
pub fn Jedi_AttackDecide(
    ctx: GameContext<'_>,
    enemy_dist: c_int,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };

        if !enemy.is_null()
            && !(*enemy).client.is_null()
            && (*enemy).s.weapon == WP_SABER as c_int
            && (*enemy_client).ps.saberLockTime > (*world).level.time
            && (*client).ps.saberLockTime < (*world).level.time
        {
            //enemy is in a saberLock and we are not
            return qfalse;
        }

        if (*client).ps.saberEventFlags & SEF_LOCK_WON as c_int != 0 {
            //we won a saber lock, press the advantage with an attack!
            let chance: c_int;
            if (*client).NPC_class == CLASS_DESANN
                || (*client).NPC_class == CLASS_LUKE
                || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) == 0
            {
                chance = 20;
            } else if (*client).NPC_class == CLASS_TAVION {
                chance = 10;
            } else if (*client).NPC_class == CLASS_REBORN
                && (*npc_info).rank == RANK_LT_JG as c_int
            {
                chance = 5;
            } else {
                chance = (*npc_info).rank;
            }
            if (*world).bg_state.rng.Q_irand(0, 30) < chance {
                (*client).ps.saberEventFlags &= !(SEF_LOCK_WON as c_int);
                crate::g_timer::TIMER_Set(
                    ctx,
                    npc,
                    c"noRetreat".as_ptr(),
                    (*world).bg_state.rng.Q_irand(500, 2000),
                );
                (*client).ps.weaponTime = 0;
                (*npc_info).shotTime = 0;
                (*npc).attackDebounceTime = 0;
                (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                crate::NPC_combat::WeaponThink(ctx, qtrue);
                return qtrue;
            }
        }

        if (*client).NPC_class == CLASS_TAVION
            || ((*client).NPC_class == CLASS_REBORN && (*npc_info).rank == RANK_LT_JG as c_int)
            || ((*client).NPC_class == CLASS_JEDI && (*npc_info).rank == RANK_COMMANDER as c_int)
        {
            if (crate::bg_panimate::PM_SaberInParry((*client).ps.saberMove) != qfalse
                || crate::bg_panimate::PM_SaberInKnockaway((*client).ps.saberMove) != qfalse)
                && (*client).ps.saberBlocked != BLOCKED_PARRY_BROKEN as c_int
            {
                //try to attack straight from a parry
                (*client).ps.weaponTime = 0;
                (*npc_info).shotTime = 0;
                (*npc).attackDebounceTime = 0;
                (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                Jedi_AdjustSaberAnimLevel(ctx, npc, FORCE_LEVEL_1);
                crate::NPC_combat::WeaponThink(ctx, qtrue);
                return qtrue;
            }
        }

        //try to hit them if we can
        if enemy_dist >= 64 {
            return qfalse;
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) == qfalse {
            return qfalse;
        }

        if ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0 {
            return qfalse;
        }

        if ((*world).globals.ucmd.buttons & BUTTON_ATTACK) == 0
            && ((*world).globals.ucmd.buttons & BUTTON_ALT_ATTACK) == 0
        {
            //not already attacking - Try to attack
            crate::NPC_combat::WeaponThink(ctx, qtrue);
        }

        if (*world).globals.ucmd.buttons & BUTTON_ATTACK != 0 {
            //attacking
            if (*world).globals.ucmd.rightmove == 0 {
                //not already strafing
                if (*world).bg_state.rng.Q_irand(0, 3) == 0 {
                    let mut right: vec3_t = [0.0; 3];
                    let mut dir2enemy: vec3_t = [0.0; 3];

                    crate::q_math::AngleVectors((*npc).r.currentAngles, None, Some(&mut right), None);
                    crate::q_math::_VectorSubtract(
                        (*enemy).r.currentOrigin,
                        (*npc).r.currentAngles,
                        &mut dir2enemy,
                    );
                    if crate::q_math::_DotProduct(right, dir2enemy) > 0.0 {
                        //he's to my right, strafe left
                        (*world).globals.ucmd.rightmove = -127;
                        (*client).ps.moveDir = [0.0, 0.0, 0.0];
                    } else {
                        //he's to my left, strafe right
                        (*world).globals.ucmd.rightmove = 127;
                        (*client).ps.moveDir = [0.0, 0.0, 0.0];
                    }
                }
            }
            return qtrue;
        }

        qfalse
    }
}

/// Raven `Jedi_Jump`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4473-4717`
pub fn Jedi_Jump(
    ctx: GameContext<'_>,
    dest: vec3_t,
    goalEntNum: c_int,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let client = (*npc).client as *mut gclient_t;

        if true {
            let mut targetDist: f32;
            let mut shotSpeed: f32 = 300.0;
            let mut travelTime: f32;
            let mut impactDist: f32;
            let mut bestImpactDist: f32 = Q3_INFINITE as f32;
            let mut targetDir: vec3_t = [0.0; 3];
            let mut shotVel: vec3_t = [0.0; 3];
            let mut failCase: vec3_t = [0.0; 3];
            let mut trace: trace_t = core::mem::zeroed();
            let mut tr: trajectory_t = core::mem::zeroed();
            let mut blocked: qboolean;
            let mut elapsedTime: c_int;
            let timeStep: c_int = 500;
            let mut hitCount: c_int = 0;
            let maxHits: c_int = 7;
            let mut lastPos: vec3_t = [0.0; 3];
            let mut testPos: vec3_t = [0.0; 3];
            let mut bottom: vec3_t = [0.0; 3];

            while hitCount < maxHits {
                crate::q_math::_VectorSubtract(dest, (*npc).r.currentOrigin, &mut targetDir);
                targetDist = crate::q_math::VectorNormalize(&mut targetDir);

                crate::q_math::_VectorScale(targetDir, shotSpeed, &mut shotVel);
                travelTime = targetDist / shotSpeed;
                shotVel[2] += travelTime * 0.5 * (*client).ps.gravity;

                if hitCount == 0 {
                    //save the first one as the worst case scenario
                    crate::q_math::_VectorCopy(shotVel, &mut failCase);
                }

                if true {
                    //do a rough trace of the path
                    blocked = qfalse;

                    crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut tr.trBase);
                    crate::q_math::_VectorCopy(shotVel, &mut tr.trDelta);
                    tr.trType = TR_GRAVITY;
                    tr.trTime = (*world).level.time;
                    travelTime *= 1000.0f32;
                    crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut lastPos);

                    elapsedTime = timeStep;
                    while (elapsedTime as f32) < travelTime.floor() + timeStep as f32 {
                        if (elapsedTime as f32) > travelTime {
                            //cap it
                            elapsedTime = travelTime.floor() as c_int;
                        }
                        crate::bg_misc::BG_EvaluateTrajectory(
                            &tr,
                            (*world).level.time + elapsedTime,
                            &mut testPos,
                        );
                        if testPos[2] < lastPos[2] {
                            //going down, ignore botclip
                            crate::trap::Trace(
                                ctx.engine,
                                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                    &mut trace as *mut trace_t,
                                    &lastPos as *const vec3_t,
                                    &(*npc).r.mins as *const vec3_t,
                                    &(*npc).r.maxs as *const vec3_t,
                                    &testPos as *const vec3_t,
                                    (*npc).s.number,
                                    (*npc).clipmask,
                                ),
                            );
                        } else {
                            //going up, check for botclip
                            crate::trap::Trace(
                                ctx.engine,
                                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                    &mut trace as *mut trace_t,
                                    &lastPos as *const vec3_t,
                                    &(*npc).r.mins as *const vec3_t,
                                    &(*npc).r.maxs as *const vec3_t,
                                    &testPos as *const vec3_t,
                                    (*npc).s.number,
                                    (*npc).clipmask | CONTENTS_BOTCLIP,
                                ),
                            );
                        }

                        if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
                            blocked = qtrue;
                            break;
                        }
                        if trace.fraction < 1.0f32 {
                            //hit something
                            if trace.entityNum == (goalEntNum) as i16 {
                                //hit the enemy, that's perfect!
                                break;
                            } else {
                                if trace.contents & CONTENTS_BOTCLIP != 0 {
                                    //hit a do-not-enter brush
                                    blocked = qtrue;
                                    break;
                                }
                                if trace.plane.normal[2] > 0.7
                                    && crate::q_math::DistanceSquared(trace.endpos, dest) < 4096.0
                                {
                                    //close enough!
                                    break;
                                } else {
                                    impactDist = crate::q_math::DistanceSquared(trace.endpos, dest);
                                    if impactDist < bestImpactDist {
                                        bestImpactDist = impactDist;
                                        crate::q_math::_VectorCopy(shotVel, &mut failCase);
                                    }
                                    blocked = qtrue;
                                    break;
                                }
                            }
                        }
                        if elapsedTime == travelTime.floor() as c_int {
                            //reached end, all clear
                            if trace.fraction >= 1.0f32 {
                                //hmm, make sure we'll land on the ground...
                                crate::q_math::_VectorCopy(trace.endpos, &mut bottom);
                                bottom[2] -= 128.0;
                                let te = trace.endpos;
                                crate::trap::Trace(
                                    ctx.engine,
                                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                        &mut trace as *mut trace_t,
                                        &te as *const vec3_t,
                                        &(*npc).r.mins as *const vec3_t,
                                        &(*npc).r.maxs as *const vec3_t,
                                        &bottom as *const vec3_t,
                                        (*npc).s.number,
                                        (*npc).clipmask,
                                    ),
                                );
                                if trace.fraction >= 1.0f32 {
                                    //would fall too far
                                    blocked = qtrue;
                                }
                            }
                            break;
                        } else {
                            //all clear, try next slice
                            crate::q_math::_VectorCopy(testPos, &mut lastPos);
                        }
                        elapsedTime += timeStep;
                    }
                    if blocked != qfalse {
                        //hit something, adjust speed (which will change arc)
                        hitCount += 1;
                        shotSpeed = 300.0 + ((hitCount - 2) as f32 * 100.0);
                        if hitCount >= 2 {
                            //skip 300 since that was the first value we tested
                            shotSpeed += 100.0;
                        }
                    } else {
                        //made it!
                        break;
                    }
                } else {
                    break;
                }
            }

            if hitCount >= maxHits {
                crate::q_math::_VectorCopy(failCase, &mut (*client).ps.velocity);
            }
            crate::q_math::_VectorCopy(shotVel, &mut (*client).ps.velocity);
        }
        qtrue
    }
}

/// Raven `Jedi_TryJump`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4719-4865`
pub fn Jedi_TryJump(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let goal_client = (*goal).client as *mut gclient_t;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };

        if ((*npc_info).scriptFlags & SCF_NO_ACROBATICS) != 0 {
            return qfalse;
        }
        if crate::g_timer::TIMER_Done(ctx, npc, c"jumpChaseDebounce".as_ptr()) != qfalse {
            if (*goal).client.is_null() || (*goal_client).ps.groundEntityNum != ENTITYNUM_NONE {
                if crate::bg_panimate::PM_InKnockDown(&mut (*client).ps) == qfalse
                    && crate::bg_panimate::BG_InRoll(&mut (*client).ps, (*client).ps.legsAnim)
                        == qfalse
                {
                    //enemy is on terra firma
                    let mut goal_diff: vec3_t = [0.0; 3];
                    let goal_z_diff: f32;
                    let goal_xy_dist: f32;
                    crate::q_math::_VectorSubtract(
                        (*goal).r.currentOrigin,
                        (*npc).r.currentOrigin,
                        &mut goal_diff,
                    );
                    goal_z_diff = goal_diff[2];
                    goal_diff[2] = 0.0;
                    goal_xy_dist = crate::q_math::VectorNormalize(&mut goal_diff);
                    if goal_xy_dist < 550.0 && goal_z_diff > -400.0 {
                        let mut debounce: qboolean = qfalse;
                        if (*npc).health < 150
                            && (((*npc).health < 30 && goal_z_diff < 0.0) || goal_z_diff < -128.0)
                        {
                            //don't jump, just walk off
                            debounce = qtrue;
                        } else if goal_z_diff < 32.0 && goal_xy_dist < 200.0 {
                            //what is their ideal jump height?
                            (*world).globals.ucmd.upmove = 127;
                            debounce = qtrue;
                        } else {
                            if goal_z_diff > 0.0 || goal_xy_dist > 128.0 {
                                //Fake a force-jump
                                let mut dest: vec3_t = [0.0; 3];
                                crate::q_math::_VectorCopy((*goal).r.currentOrigin, &mut dest);
                                if goal == enemy {
                                    let mut sideTry = 0;
                                    while sideTry < 10 {
                                        let mut trace: trace_t = core::mem::zeroed();
                                        let mut bottom: vec3_t = [0.0; 3];

                                        if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                                            dest[0] += (*enemy).r.maxs[0] * 1.25;
                                        } else {
                                            dest[0] += (*enemy).r.mins[0] * 1.25;
                                        }
                                        if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                                            dest[1] += (*enemy).r.maxs[1] * 1.25;
                                        } else {
                                            dest[1] += (*enemy).r.mins[1] * 1.25;
                                        }
                                        crate::q_math::_VectorCopy(dest, &mut bottom);
                                        bottom[2] -= 128.0;
                                        crate::trap::Trace(
                                            ctx.engine,
                                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                                &mut trace as *mut trace_t,
                                                &dest as *const vec3_t,
                                                &(*npc).r.mins as *const vec3_t,
                                                &(*npc).r.maxs as *const vec3_t,
                                                &bottom as *const vec3_t,
                                                (*goal).s.number,
                                                (*npc).clipmask,
                                            ),
                                        );
                                        if trace.fraction < 1.0f32 {
                                            //hit floor, okay to land here
                                            break;
                                        }
                                        sideTry += 1;
                                    }
                                    if sideTry >= 10 {
                                        //screw it, just jump right at him?
                                        crate::q_math::_VectorCopy(
                                            (*goal).r.currentOrigin,
                                            &mut dest,
                                        );
                                    }
                                }
                                if Jedi_Jump(ctx, dest, (*goal).s.number) != qfalse {
                                    {
                                        let jumpAnim: c_int;
                                        if (*client).NPC_class == CLASS_BOBAFETT
                                            || ((*npc_info).rank != RANK_CREWMAN as c_int
                                                && (*npc_info).rank <= RANK_LT_JG as c_int)
                                        {
                                            //can't do acrobatics
                                            jumpAnim = BOTH_FORCEJUMP1 as c_int;
                                        } else {
                                            jumpAnim = BOTH_FLIP_F as c_int;
                                        }
                                        crate::npc_c::NPC_SetAnim(
                                            npc,
                                            SETANIM_BOTH,
                                            jumpAnim,
                                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                                        );
                                    }

                                    (*client).ps.fd.forceJumpZStart = (*npc).r.currentOrigin[2];

                                    (*client).ps.weaponTime = (*client).ps.torsoTimer;
                                    (*client).ps.fd.forcePowersActive |= 1 << FP_LEVITATION;
                                    if (*client).NPC_class == CLASS_BOBAFETT {
                                        crate::g_utils::G_SoundOnEnt(
                                            ctx,
                                            npc,
                                            CHAN_ITEM as c_int,
                                            c"sound/boba/jeton.wav".as_ptr(),
                                        );
                                        (*client).jetPackTime = (*world).level.time
                                            + (*world).bg_state.rng.Q_irand(1000, 3000);
                                    } else {
                                        crate::g_utils::G_SoundOnEnt(
                                            ctx,
                                            npc,
                                            CHAN_BODY as c_int,
                                            c"sound/weapons/force/jump.wav".as_ptr(),
                                        );
                                    }

                                    crate::g_timer::TIMER_Set(
                                        ctx,
                                        npc,
                                        c"forceJumpChasing".as_ptr(),
                                        (*world).bg_state.rng.Q_irand(2000, 3000),
                                    );
                                    debounce = qtrue;
                                }
                            }
                        }
                        if debounce != qfalse {
                            crate::g_timer::TIMER_Set(
                                ctx,
                                npc,
                                c"jumpChaseDebounce".as_ptr(),
                                (*world).bg_state.rng.Q_irand(2000, 5000),
                            );
                            (*world).globals.ucmd.forwardmove = 127;
                            (*client).ps.moveDir = [0.0, 0.0, 0.0];
                            crate::g_timer::TIMER_Set(ctx, npc, c"duck".as_ptr(), -(*world).level.time);
                            return qtrue;
                        }
                    }
                }
            }
        }
        qfalse
    }
}

/// Raven `Jedi_Jumping`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4867-4914`
pub fn Jedi_Jumping(
    ctx: GameContext<'_>,
    goal: *mut gentity_t,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let client = (*npc).client as *mut gclient_t;
        if crate::g_timer::TIMER_Done(ctx, npc, c"forceJumpChasing".as_ptr()) == qfalse
            && !goal.is_null()
        {
            //force-jumping at the enemy
            if (*client).ps.groundEntityNum != ENTITYNUM_NONE {
                //landed
                crate::g_timer::TIMER_Set(ctx, npc, c"forceJumpChasing".as_ptr(), 0);
            } else {
                crate::NPC_utils::NPC_FaceEntity(ctx, goal, qtrue);
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `Jedi_CheckEnemyMovement`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:4917-5036`
pub fn Jedi_CheckEnemyMovement(
    ctx: GameContext<'_>,
    enemy_dist: f32,
) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        if (*npc).enemy.is_none() {
            return;
        }
        let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
        let enemy_client = (*enemy).client as *mut gclient_t;
        if (*enemy).client.is_null() {
            return;
        }

        if (*client).NPC_class != CLASS_TAVION
            && (*client).NPC_class != CLASS_DESANN
            && (*client).NPC_class != CLASS_LUKE
            && crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) != 0
        {
            let npc_id = ent_id(ge, npc);
            if (*enemy).enemy.is_some() && (*enemy).enemy == Some(npc_id) {
                //enemy is mad at *me*
                if (*enemy_client).ps.legsAnim == BOTH_JUMPFLIPSLASHDOWN1 as c_int
                    || (*enemy_client).ps.legsAnim == BOTH_JUMPFLIPSTABDOWN as c_int
                {
                    //enemy is flipping over me
                    if (*world).bg_state.rng.Q_irand(0, (*npc_info).rank) < RANK_LT as c_int {
                        //be nice and stand still for him...
                        (*world).globals.ucmd.forwardmove = 0;
                        (*world).globals.ucmd.rightmove = 0;
                        (*world).globals.ucmd.upmove = 0;
                        (*client).ps.moveDir = [0.0, 0.0, 0.0];
                        (*client).ps.fd.forceJumpCharge = 0.0;
                        crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), -1);
                        crate::g_timer::TIMER_Set(
                            ctx,
                            npc,
                            c"noStrafe".as_ptr(),
                            (*world).bg_state.rng.Q_irand(500, 1000),
                        );
                        crate::g_timer::TIMER_Set(
                            ctx,
                            npc,
                            c"movenone".as_ptr(),
                            (*world).bg_state.rng.Q_irand(500, 1000),
                        );
                        crate::g_timer::TIMER_Set(
                            ctx,
                            npc,
                            c"movecenter".as_ptr(),
                            (*world).bg_state.rng.Q_irand(500, 1000),
                        );
                    }
                } else if (*enemy_client).ps.legsAnim == BOTH_WALL_FLIP_BACK1 as c_int
                    || (*enemy_client).ps.legsAnim == BOTH_WALL_FLIP_RIGHT as c_int
                    || (*enemy_client).ps.legsAnim == BOTH_WALL_FLIP_LEFT as c_int
                    || (*enemy_client).ps.legsAnim == BOTH_WALL_RUN_LEFT_FLIP as c_int
                    || (*enemy_client).ps.legsAnim == BOTH_WALL_RUN_RIGHT_FLIP as c_int
                {
                    //he's flipping off a wall
                    if (*enemy_client).ps.groundEntityNum == ENTITYNUM_NONE {
                        //still in air
                        if enemy_dist < 256.0 {
                            if (*world).bg_state.rng.Q_irand(0, (*npc_info).rank) < RANK_LT as c_int {
                                let mut enemyFwd: vec3_t = [0.0; 3];
                                let mut dest: vec3_t = [0.0; 3];
                                let mut dir: vec3_t = [0.0; 3];

                                //stop current movement
                                (*world).globals.ucmd.forwardmove = 0;
                                (*world).globals.ucmd.rightmove = 0;
                                (*world).globals.ucmd.upmove = 0;
                                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                                (*client).ps.fd.forceJumpCharge = 0.0;
                                crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), -1);
                                crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), -1);
                                crate::g_timer::TIMER_Set(
                                    ctx,
                                    npc,
                                    c"noStrafe".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1000),
                                );
                                crate::g_timer::TIMER_Set(
                                    ctx,
                                    npc,
                                    c"noturn".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(250, 500)
                                        * (3 - (*world).cvars.g_spskill.integer),
                                );

                                crate::q_math::_VectorCopy(
                                    (*enemy_client).ps.velocity,
                                    &mut enemyFwd,
                                );
                                crate::q_math::VectorNormalize(&mut enemyFwd);
                                crate::q_math::_VectorMA(
                                    (*enemy).r.currentOrigin,
                                    -64.0,
                                    enemyFwd,
                                    &mut dest,
                                );
                                crate::q_math::_VectorSubtract(
                                    dest,
                                    (*npc).r.currentOrigin,
                                    &mut dir,
                                );
                                if crate::q_math::VectorNormalize(&mut dir) > 32.0 {
                                    crate::NPC_move::G_UcmdMoveForDir(
                                        npc,
                                        &mut (*world).globals.ucmd,
                                        dir,
                                    );
                                } else {
                                    crate::g_timer::TIMER_Set(
                                        ctx,
                                        npc,
                                        c"movenone".as_ptr(),
                                        (*world).bg_state.rng.Q_irand(500, 1000),
                                    );
                                    crate::g_timer::TIMER_Set(
                                        ctx,
                                        npc,
                                        c"movecenter".as_ptr(),
                                        (*world).bg_state.rng.Q_irand(500, 1000),
                                    );
                                }
                            }
                        }
                    }
                } else if (*enemy_client).ps.legsAnim == BOTH_A2_STABBACK1 as c_int {
                    //he's stabbing backwards
                    if enemy_dist < 256.0 && enemy_dist > 64.0 {
                        if crate::NPC_senses::InFront(
                            (*npc).r.currentOrigin,
                            (*enemy).r.currentOrigin,
                            (*enemy).r.currentAngles,
                            0.0f32,
                        ) == qfalse
                        {
                            //behind him
                            if (*world).bg_state.rng.Q_irand(0, (*npc_info).rank) == 0 {
                                let mut enemyFwd: vec3_t = [0.0; 3];
                                let mut dest: vec3_t = [0.0; 3];
                                let mut dir: vec3_t = [0.0; 3];

                                (*world).globals.ucmd.forwardmove = 0;
                                (*world).globals.ucmd.rightmove = 0;
                                (*world).globals.ucmd.upmove = 0;
                                (*client).ps.moveDir = [0.0, 0.0, 0.0];
                                (*client).ps.fd.forceJumpCharge = 0.0;
                                crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), -1);
                                crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), -1);
                                crate::g_timer::TIMER_Set(
                                    ctx,
                                    npc,
                                    c"noStrafe".as_ptr(),
                                    (*world).bg_state.rng.Q_irand(500, 1000),
                                );

                                crate::q_math::AngleVectors(
                                    (*enemy).r.currentAngles,
                                    Some(&mut enemyFwd),
                                    None,
                                    None,
                                );
                                crate::q_math::_VectorMA(
                                    (*enemy).r.currentOrigin,
                                    -32.0,
                                    enemyFwd,
                                    &mut dest,
                                );
                                crate::q_math::_VectorSubtract(
                                    dest,
                                    (*npc).r.currentOrigin,
                                    &mut dir,
                                );
                                if crate::q_math::VectorNormalize(&mut dir) > 64.0 {
                                    crate::NPC_move::G_UcmdMoveForDir(
                                        npc,
                                        &mut (*world).globals.ucmd,
                                        dir,
                                    );
                                } else {
                                    crate::g_timer::TIMER_Set(
                                        ctx,
                                        npc,
                                        c"movenone".as_ptr(),
                                        (*world).bg_state.rng.Q_irand(500, 1000),
                                    );
                                    crate::g_timer::TIMER_Set(
                                        ctx,
                                        npc,
                                        c"movecenter".as_ptr(),
                                        (*world).bg_state.rng.Q_irand(500, 1000),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Raven `Jedi_CheckJumps`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5038-5153`
pub fn Jedi_CheckJumps(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        let mut jumpVel: vec3_t = [0.0; 3];
        let mut trace: trace_t = core::mem::zeroed();
        let mut tr: trajectory_t = core::mem::zeroed();
        let mut lastPos: vec3_t = [0.0; 3];
        let mut testPos: vec3_t = [0.0; 3];
        let mut bottom: vec3_t = [0.0; 3];
        let mut elapsedTime: c_int;

        if ((*npc_info).scriptFlags & SCF_NO_ACROBATICS) != 0 {
            (*client).ps.fd.forceJumpCharge = 0.0;
            (*world).globals.ucmd.upmove = 0;
            return;
        }
        jumpVel = [0.0, 0.0, 0.0];

        if (*client).ps.fd.forceJumpCharge != 0.0 {
            crate::w_force::WP_GetVelocityForForceJump(
                ctx,
                npc,
                &mut jumpVel,
                &mut (*world).globals.ucmd,
            );
        } else if (*world).globals.ucmd.upmove > 0 {
            crate::q_math::_VectorCopy((*client).ps.velocity, &mut jumpVel);
            jumpVel[2] = JUMP_VELOCITY;
        } else {
            return;
        }

        if jumpVel[0] == 0.0 && jumpVel[1] == 0.0 {
            //we assume a jump straight up is safe
            return;
        }

        crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut tr.trBase);
        crate::q_math::_VectorCopy(jumpVel, &mut tr.trDelta);
        tr.trType = TR_GRAVITY;
        tr.trTime = (*world).level.time;
        crate::q_math::_VectorCopy((*npc).r.currentOrigin, &mut lastPos);

        trace.endpos = [0.0, 0.0, 0.0]; //shut the compiler up

        let unsafe_jump = 'check: {
            elapsedTime = 500;
            while elapsedTime <= 4000 {
                crate::bg_misc::BG_EvaluateTrajectory(
                    &tr,
                    (*world).level.time + elapsedTime,
                    &mut testPos,
                );
                if testPos[2] < lastPos[2] {
                    //going down, don't check for BOTCLIP
                    crate::trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &lastPos as *const vec3_t,
                            &(*npc).r.mins as *const vec3_t,
                            &(*npc).r.maxs as *const vec3_t,
                            &testPos as *const vec3_t,
                            (*npc).s.number,
                            (*npc).clipmask,
                        ),
                    );
                } else {
                    //going up, check for BOTCLIP
                    crate::trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace as *mut trace_t,
                            &lastPos as *const vec3_t,
                            &(*npc).r.mins as *const vec3_t,
                            &(*npc).r.maxs as *const vec3_t,
                            &testPos as *const vec3_t,
                            (*npc).s.number,
                            (*npc).clipmask | CONTENTS_BOTCLIP,
                        ),
                    );
                }
                if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 {
                    break 'check true;
                }
                if trace.fraction < 1.0f32 {
                    //hit something
                    if trace.contents & CONTENTS_BOTCLIP != 0 {
                        //hit a do-not-enter brush
                        break 'check true;
                    }
                    break;
                }
                crate::q_math::_VectorCopy(testPos, &mut lastPos);
                elapsedTime += 500;
            }
            //okay, reached end of jump, now trace down from here for a floor
            crate::q_math::_VectorCopy(trace.endpos, &mut bottom);
            if bottom[2] > (*npc).r.currentOrigin[2] {
                //only care about dist down from current height or lower
                bottom[2] = (*npc).r.currentOrigin[2];
            } else if (*npc).r.currentOrigin[2] - bottom[2] > 400.0 {
                //whoa, long drop, don't do it!
                break 'check true;
            }
            bottom[2] -= 128.0;
            let te = trace.endpos;
            crate::trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut trace as *mut trace_t,
                    &te as *const vec3_t,
                    &(*npc).r.mins as *const vec3_t,
                    &(*npc).r.maxs as *const vec3_t,
                    &bottom as *const vec3_t,
                    (*npc).s.number,
                    (*npc).clipmask,
                ),
            );
            if trace.allsolid != (qfalse) as u8 || trace.startsolid != (qfalse) as u8 || trace.fraction < 1.0f32 {
                //hit ground!
                if trace.entityNum < (ENTITYNUM_WORLD) as i16 {
                    //landed on an ent
                    let groundEnt = ge.add(trace.entityNum as usize);
                    if (*groundEnt).r.svFlags & SVF_GLASS_BRUSH != 0 {
                        //don't land on breakable glass!
                        break 'check true;
                    }
                }
                return;
            }
            true
        };
        if unsafe_jump {
            //probably no floor at end of jump, so don't jump
            (*client).ps.fd.forceJumpCharge = 0.0;
            (*world).globals.ucmd.upmove = 0;
        }
    }
}

/// Raven `Jedi_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5155-5344`
pub fn Jedi_Combat(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let enemy: *mut gentity_t = match (*npc).enemy {
            Some(id) => ge.add(id.0 as usize),
            None => core::ptr::null_mut(),
        };
        let enemy_client = if enemy.is_null() {
            core::ptr::null_mut()
        } else {
            (*enemy).client as *mut gclient_t
        };

        let mut enemy_dir: vec3_t = [0.0; 3];
        let mut enemy_movedir: vec3_t = [0.0; 3];
        let mut enemy_dest: vec3_t = [0.0; 3];
        let mut enemy_dist: f32 = 0.0;
        let mut enemy_movespeed: f32 = 0.0;
        let mut enemy_lost: qboolean = qfalse;

        //See where enemy will be 300 ms from now
        Jedi_SetEnemyInfo(
            ctx,
            &mut enemy_dest,
            &mut enemy_dir,
            &mut enemy_dist,
            &mut enemy_movedir,
            &mut enemy_movespeed,
            300,
        );

        if Jedi_Jumping(ctx, enemy) != qfalse {
            //I'm in the middle of a jump, so just see if I should attack
            Jedi_AttackDecide(ctx, enemy_dist as c_int);
            return;
        }

        if ((*client).ps.fd.forcePowersActive & (1 << FP_GRIP)) == 0
            || (*client).ps.fd.forcePowerLevel[FP_GRIP as usize] < FORCE_LEVEL_2
        {
            //not gripping
            if Jedi_ClearPathToSpot(ctx, enemy_dest, (*enemy).s.number) == qfalse {
                //hunt him down
                if (crate::NPC_utils::NPC_ClearLOS4(ctx, enemy) != qfalse
                    || (*npc_info).enemyLastSeenTime > (*world).level.time - 500)
                    && crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue) != qfalse
                {
                    if Jedi_TryJump(ctx, enemy) != qfalse {
                        return;
                    }
                }

                //Check for evasion
                if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse {
                    if (*client).ps.saberBlocked != BLOCKED_ATK_BOUNCE as c_int
                        && (*client).ps.saberBlocked != BLOCKED_PARRY_BROKEN as c_int
                    {
                        (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                    }
                }
                if Jedi_Hunt(ctx) != qfalse && ((*npc_info).aiFlags & NPCAI_BLOCKED) == 0 {
                    //can macro-navigate to him
                    // PORT-NOTE(jediSpeechDebounceTime): `()` placeholder field indexed by team.
                    if enemy_dist < 384.0
                        && (*world).bg_state.rng.Q_irand(0, 10) == 0
                        && (*npc_info).blockedSpeechDebounceTime < (*world).level.time
                        && (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize]
                            < (*world).level.time
                        && crate::NPC_utils::NPC_ClearLOS4(ctx, enemy) == qfalse
                    {
                        crate::NPC_sounds::G_AddVoiceEvent(
                            ctx,
                            npc,
                            (*world).bg_state.rng.Q_irand(
                                entity_event_t::EV_JLOST1 as c_int,
                                entity_event_t::EV_JLOST3 as c_int,
                            ),
                            3000,
                        );
                        (*npc_info).blockedSpeechDebounceTime = (*world).level.time + 3000;
                        (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                            (*world).level.time + 3000;
                    }

                    return;
                } else {
                    if (*npc_info).aiFlags & NPCAI_BLOCKED != 0 {
                        //try to jump to the blockedDest
                        let tempGoal = crate::g_utils::G_Spawn(ctx);
                        crate::g_utils::G_SetOrigin(tempGoal, (*npc_info).blockedDest);
                        crate::trap::LinkEntity(
                            ctx.engine,
                            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(tempGoal),
                        );
                        if Jedi_TryJump(ctx, tempGoal) != qfalse {
                            //going to jump to the dest
                            crate::g_utils::G_FreeEntity(ctx, tempGoal);
                            return;
                        }
                        crate::g_utils::G_FreeEntity(ctx, tempGoal);
                    }

                    enemy_lost = qtrue;
                }
            }
        }
        //else, we can see him or we can't track him at all

        //every few seconds, decide if we should we advance or retreat?
        Jedi_CombatTimersUpdate(ctx, enemy_dist as c_int);

        //maintain a distance from enemy appropriate for our aggression level
        Jedi_CombatDistance(ctx, enemy_dist as c_int);

        {
            //Update our seen enemy position
            if (*enemy).client.is_null()
                || ((*enemy_client).ps.groundEntityNum != ENTITYNUM_NONE
                    && (*client).ps.groundEntityNum != ENTITYNUM_NONE)
            {
                crate::q_math::_VectorCopy(
                    (*enemy).r.currentOrigin,
                    &mut (*npc_info).enemyLastSeenLocation,
                );
            }
            (*npc_info).enemyLastSeenTime = (*world).level.time;
        }

        //Turn to face the enemy
        if crate::g_timer::TIMER_Done(ctx, npc, c"noturn".as_ptr()) != qfalse {
            Jedi_FaceEnemy(ctx, qtrue);
        }
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

        //Check for evasion
        if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) != qfalse {
            if (*client).ps.saberBlocked != BLOCKED_ATK_BOUNCE as c_int
                && (*client).ps.saberBlocked != BLOCKED_PARRY_BROKEN as c_int
            {
                (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
            }
        }
        if (*enemy).s.weapon == WP_SABER as c_int {
            Jedi_EvasionSaber(ctx, enemy_movedir, enemy_dist, enemy_dir);
        }

        //apply strafing/walking timers, etc.
        Jedi_TimersApply(ctx);

        if (*client).ps.saberInFlight == qfalse
            && (((*client).ps.fd.forcePowersActive & (1 << FP_GRIP)) == 0
                || (*client).ps.fd.forcePowerLevel[FP_GRIP as usize] < FORCE_LEVEL_2)
        {
            //not throwing saber or using force grip
            if Jedi_AttackDecide(ctx, enemy_dist as c_int) == qfalse {
                //we're not attacking, decide what else to do
                Jedi_CombatIdle(ctx, enemy_dist as c_int);
            } else {
                //we are attacking; stop taunting
                crate::g_timer::TIMER_Set(ctx, npc, c"taunting".as_ptr(), -(*world).level.time);
            }
        }
        if (*client).NPC_class == CLASS_BOBAFETT {
            Boba_FireDecide(ctx);
        }

        //Check for certain enemy special moves
        Jedi_CheckEnemyMovement(ctx, enemy_dist);
        //Make sure that we don't jump off ledges over long drops
        Jedi_CheckJumps(ctx);
        //Just make sure we don't strafe into walls or off cliffs
        if NPC_MoveDirClear(
            ctx,
            (*world).globals.ucmd.forwardmove as c_int,
            (*world).globals.ucmd.rightmove as c_int,
            qtrue,
        ) == qfalse
        {
            //uh-oh, we are going to fall or hit something
            let mut info: navInfo_t = core::mem::zeroed();
            crate::NPC_move::NAV_GetLastMove(ctx, &mut info);
            if (info.flags & NIF_MACRO_NAV) == 0 {
                //micro-navigation told us to step off a ledge, try macronav for now
                crate::NPC_move::NPC_MoveToGoal(ctx, qfalse);
            }
            //reset the timers.
            crate::g_timer::TIMER_Set(ctx, npc, c"strafeLeft".as_ptr(), 0);
            crate::g_timer::TIMER_Set(ctx, npc, c"strafeRight".as_ptr(), 0);
        }
        let _ = enemy_lost;
    }
}

/// Raven `NPC_Jedi_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5358-5444`
pub fn NPC_Jedi_Pain(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
) {
    unsafe {
        let world = ctx.world;
        let client = (*self_).client as *mut gclient_t;
        let snpc = (*self_).NPC as *mut gNPC_t;
        let d_jedi = (*world).cvars.d_JediAI.integer != 0;
        let other = attacker;
        let mut point: vec3_t = [0.0; 3];

        crate::q_math::_VectorCopy((*world).globals.gPainPoint, &mut point);

        if (*other).s.weapon == WP_SABER as c_int {
            //back off
            crate::g_timer::TIMER_Set(ctx, self_, c"parryTime".as_ptr(), -1);
            if (*client).NPC_class == CLASS_DESANN
                || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
            {
                //less for Desann
                (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                    (*world).level.time + (3 - (*world).cvars.g_spskill.integer) * 50;
            } else if (*snpc).rank >= RANK_LT_JG as c_int {
                (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                    (*world).level.time + (3 - (*world).cvars.g_spskill.integer) * 100;
            } else {
                (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                    (*world).level.time + (3 - (*world).cvars.g_spskill.integer) * 200;
            }
            if (*world).bg_state.rng.Q_irand(0, 3) == 0 {
                //ouch... maybe switch up which saber power level we're using
                Jedi_AdjustSaberAnimLevel(
                    ctx,
                    self_,
                    (*world).bg_state.rng.Q_irand(FORCE_LEVEL_1, FORCE_LEVEL_3),
                );
            }
            if (*world).bg_state.rng.Q_irand(0, 1) == 0 {
                Jedi_Aggression(self_, -1);
            }
            if d_jedi {
                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "({}) PAIN: agg {}, no parry until {}\n",
                        (*world).level.time,
                        (*snpc).stats.aggression,
                        (*world).level.time + 500
                    ))
                    .as_ptr(),
                );
            }
            if d_jedi {
                let mut diff: vec3_t = [0.0; 3];
                let mut fwdangles: vec3_t = [0.0; 3];
                let mut right: vec3_t = [0.0; 3];
                let rightdot: f32;
                let zdiff: f32;

                crate::q_math::_VectorSubtract(point, (*client).renderInfo.eyePoint, &mut diff);
                diff[2] = 0.0;
                fwdangles[1] = (*client).ps.viewangles[1];
                crate::q_math::AngleVectors(fwdangles, None, Some(&mut right), None);
                rightdot = crate::q_math::_DotProduct(right, diff);
                zdiff = point[2] - (*client).renderInfo.eyePoint[2];

                crate::g_main::Com_Printf(
                    cstr(&format!(
                        "({}) saber hit at height {:.2}, zdiff: {:.2}, rightdot: {:.2}\n",
                        (*world).level.time,
                        point[2] - (*self_).r.absmin[2],
                        zdiff,
                        rightdot
                    ))
                    .as_ptr(),
                );
            }
        } else {
            //attack
            Jedi_Aggression(self_, 1);
        }

        (*snpc).enemyCheckDebounceTime = 0;

        crate::w_force::WP_ForcePowerStop(ctx, self_, FP_GRIP);

        crate::NPC_reactions::NPC_Pain(ctx, self_, attacker, damage);

        if damage == 0 && (*self_).health > 0 {
            //FIXME: better way to know I was pushed
            crate::NPC_sounds::G_AddVoiceEvent(
                ctx,
                self_,
                (*world).bg_state.rng.Q_irand(
                    entity_event_t::EV_PUSHED1 as c_int,
                    entity_event_t::EV_PUSHED3 as c_int,
                ),
                2000,
            );
        }

        //drop me from the ceiling if I'm on it
        if Jedi_WaitingAmbush(self_) != qfalse {
            (*client).noclip = qfalse;
        }
        if (*client).ps.legsAnim == BOTH_CEILING_CLING as c_int {
            crate::npc_c::NPC_SetAnim(
                self_,
                SETANIM_LEGS,
                BOTH_CEILING_DROP as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
        if (*client).ps.torsoAnim == BOTH_CEILING_CLING as c_int {
            crate::npc_c::NPC_SetAnim(
                self_,
                SETANIM_TORSO,
                BOTH_CEILING_DROP as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
    }
}

/// Raven `Jedi_CheckDanger`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5446-5463`
pub fn Jedi_CheckDanger(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let client = (*npc).client as *mut gclient_t;
        let alertEvent =
            crate::NPC_senses::NPC_CheckAlertEvents(ctx, qtrue, qtrue, -1, qfalse, AEL_MINOR as c_int);
        let ae = &(*world).level.alertEvents[alertEvent as usize];
        if ae.level as c_int >= AEL_DANGER as c_int {
            //run away!
            let owner = ae.owner;
            let owner_client = if owner.is_null() {
                core::ptr::null_mut()
            } else {
                (*owner).client as *mut gclient_t
            };
            if owner.is_null()
                || (*owner).client.is_null()
                || (owner != npc && (*owner_client).playerTeam != (*client).playerTeam)
            {
                //no owner
                return qfalse;
            }
            crate::NPC_combat::G_SetEnemy(ctx, npc, owner);
            (*npc_info).enemyLastSeenTime = (*world).level.time;
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"attackDelay".as_ptr(),
                (*world).bg_state.rng.Q_irand(500, 2500),
            );
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Jedi_CheckAmbushPlayer`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5465-5545`
pub fn Jedi_CheckAmbushPlayer(ctx: GameContext<'_>) -> qboolean {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let mut i = 0;
        let mut target_dist: f32;
        let zDiff: f32;

        while i < MAX_CLIENTS {
            let player = ge.add(i as usize);

            if player.is_null() || (*player).client.is_null() {
                i += 1;
                continue;
            }

            if crate::NPC_utils::NPC_ValidEnemy(ctx, player) == qfalse {
                i += 1;
                continue;
            }

            if (*client).ps.powerups[PW_CLOAKED as usize] != 0
                || crate::NPC_utils::NPC_SomeoneLookingAtMe(ctx, npc) == qfalse
            {
                //if I'm not cloaked and the player's crosshair is on me, I will wake up
                if crate::trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &(*player).r.currentOrigin as *const vec3_t,
                    &(*npc).r.currentOrigin as *const vec3_t,
                ),
            )
                    == 0
                {
                    //must be in same room
                    i += 1;
                    continue;
                } else {
                    if (*client).ps.powerups[PW_CLOAKED as usize] == 0 {
                        crate::NPC_utils::NPC_SetLookTarget(npc, 0, 0);
                    }
                }
                zDiff = (*npc).r.currentOrigin[2] - (*player).r.currentOrigin[2];
                if zDiff <= 0.0 || zDiff > 512.0 {
                    //never ambush if they're above me or way way below me
                    i += 1;
                    continue;
                }

                target_dist = crate::q_math::DistanceHorizontalSquared(
                    (*player).r.currentOrigin,
                    (*npc).r.currentOrigin,
                );
                if target_dist > 4096.0 {
                    //closer than 64 - always ambush
                    if target_dist > 147456.0 {
                        //> 384, not close enough to ambush
                        i += 1;
                        continue;
                    }
                    //Check FOV first
                    if (*client).ps.powerups[PW_CLOAKED as usize] != 0 {
                        if crate::NPC_senses::InFOV(ctx, player, npc, 30, 90) == qfalse {
                            i += 1;
                            continue;
                        }
                    } else {
                        if crate::NPC_senses::InFOV(ctx, player, npc, 45, 90) == qfalse {
                            i += 1;
                            continue;
                        }
                    }
                }

                if crate::NPC_utils::NPC_ClearLOS4(ctx, player) == qfalse {
                    i += 1;
                    continue;
                }
            }

            //Got him, return true;
            crate::NPC_combat::G_SetEnemy(ctx, npc, player);
            (*npc_info).enemyLastSeenTime = (*world).level.time;
            crate::g_timer::TIMER_Set(
                ctx,
                npc,
                c"attackDelay".as_ptr(),
                (*world).bg_state.rng.Q_irand(500, 2500),
            );
            return qtrue;
        }

        //Didn't get anyone.
        qfalse
    }
}

/// Raven `Jedi_Ambush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5547-5559`
pub fn Jedi_Ambush(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) {
    unsafe {
        let world = ctx.world;
        let client = (*self_).client as *mut gclient_t;
        (*client).noclip = qfalse;
        crate::npc_c::NPC_SetAnim(
            self_,
            SETANIM_BOTH,
            BOTH_CEILING_DROP as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        (*client).ps.weaponTime = (*client).ps.torsoTimer;
        if (*client).NPC_class != CLASS_BOBAFETT {
            crate::w_saber::WP_ActivateSaber(ctx, self_);
        }
        Jedi_Decloak(ctx, self_);
        crate::NPC_sounds::G_AddVoiceEvent(
            ctx,
            self_,
            (*world).bg_state.rng.Q_irand(
                entity_event_t::EV_ANGER1 as c_int,
                entity_event_t::EV_ANGER3 as c_int,
            ),
            1000,
        );
    }
}

/// Raven `Jedi_WaitingAmbush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5561-5568`
pub fn Jedi_WaitingAmbush(
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        let client = (*self_).client as *mut gclient_t;
        if ((*self_).spawnflags & JSF_AMBUSH) != 0 && (*client).noclip != qfalse {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Jedi_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5575-5728`
pub fn Jedi_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        (*client).ps.saberBlocked = BLOCKED_NONE as c_int;

        'finish: {
            if Jedi_WaitingAmbush(npc) != qfalse {
                //hiding on the ceiling
                crate::npc_c::NPC_SetAnim(
                    npc,
                    SETANIM_BOTH,
                    BOTH_CEILING_CLING as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                if (*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
                    //look for enemies
                    if Jedi_CheckAmbushPlayer(ctx) != qfalse || Jedi_CheckDanger(ctx) != qfalse {
                        //found him!
                        Jedi_Ambush(ctx, npc);
                        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                        return;
                    }
                }
            } else if (*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
                //look for enemies
                let mut best_enemy: *mut gentity_t = core::ptr::null_mut();
                let mut best_enemy_dist: f32 = Q3_INFINITE as f32;
                let mut i = 0;
                while i < ENTITYNUM_WORLD {
                    let enemy = ge.add(i as usize);
                    let enemy_c = (*enemy).client as *mut gclient_t;
                    let enemy_dist: f32;
                    if !enemy.is_null()
                        && !(*enemy).client.is_null()
                        && crate::NPC_utils::NPC_ValidEnemy(ctx, enemy) != qfalse
                        && (*enemy_c).playerTeam == (*client).enemyTeam
                    {
                        if crate::trap::InPVS(
                            ctx.engine,
                            mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                                &(*npc).r.currentOrigin as *const vec3_t,
                                &(*enemy).r.currentOrigin as *const vec3_t,
                            ),
                        ) != 0
                        {
                            //we could potentially see him
                            enemy_dist = crate::q_math::DistanceSquared(
                                (*npc).r.currentOrigin,
                                (*enemy).r.currentOrigin,
                            );
                            if (*enemy).s.eType == ET_PLAYER as c_int || enemy_dist < best_enemy_dist
                            {
                                if enemy_dist < (220.0 * 220.0)
                                    || ((*npc_info).investigateCount >= 3
                                        && (*client).ps.saberHolstered == 0)
                                {
                                    crate::NPC_combat::G_SetEnemy(ctx, npc, enemy);
                                    (*npc_info).stats.aggression = 3;
                                    break;
                                } else if (*enemy_c).ps.saberInFlight != qfalse
                                    && (*enemy_c).ps.saberHolstered == 0
                                {
                                    //threw his saber, see if heading toward me
                                    let saberDist: f32;
                                    let mut saberDir2Me: vec3_t = [0.0; 3];
                                    let mut saberMoveDir: vec3_t = [0.0; 3];
                                    let saber = ge.add((*enemy_c).ps.saberEntityNum as usize);
                                    crate::q_math::_VectorSubtract(
                                        (*npc).r.currentOrigin,
                                        (*saber).r.currentOrigin,
                                        &mut saberDir2Me,
                                    );
                                    saberDist = crate::q_math::VectorNormalize(&mut saberDir2Me);
                                    crate::q_math::_VectorCopy(
                                        (*saber).s.pos.trDelta,
                                        &mut saberMoveDir,
                                    );
                                    crate::q_math::VectorNormalize(&mut saberMoveDir);
                                    if crate::q_math::_DotProduct(saberMoveDir, saberDir2Me) > 0.5 {
                                        //it's heading towards me
                                        if saberDist < 200.0 {
                                            //incoming!
                                            crate::NPC_combat::G_SetEnemy(ctx, npc, enemy);
                                            (*npc_info).stats.aggression = 3;
                                            break;
                                        }
                                    }
                                }
                                best_enemy_dist = enemy_dist;
                                best_enemy = enemy;
                            }
                        }
                    }
                    i += 1;
                }
                if (*npc).enemy.is_none() {
                    //still not mad
                    if best_enemy.is_null() {
                        Jedi_AggressionErosion(ctx, -1);
                    } else {
                        //have one to consider
                        let best_c = (*best_enemy).client as *mut gclient_t;
                        if crate::NPC_utils::NPC_ClearLOS4(ctx, best_enemy) != qfalse {
                            //we have a clear LOS to him
                            if (*best_enemy).s.number != 0 {
                                //just attack
                                crate::NPC_combat::G_SetEnemy(ctx, npc, best_enemy);
                                (*npc_info).stats.aggression = 3;
                            } else if (*client).NPC_class != CLASS_BOBAFETT {
                                //the player, toy with him
                                if crate::g_timer::TIMER_Done(ctx, npc, c"watchTime".as_ptr())
                                    != qfalse
                                {
                                    if crate::g_timer::TIMER_Get(ctx, npc, c"watchTime".as_ptr())
                                        == -1
                                    {
                                        //ignore him for a couple seconds
                                        crate::g_timer::TIMER_Set(
                                            ctx,
                                            npc,
                                            c"watchTime".as_ptr(),
                                            (*world).bg_state.rng.Q_irand(3000, 5000),
                                        );
                                        break 'finish;
                                    } else {
                                        //start to notice him
                                        if (*npc_info).investigateCount == 0 {
                                            crate::NPC_sounds::G_AddVoiceEvent(
                                                ctx,
                                                npc,
                                                (*world).bg_state.rng.Q_irand(
                                                    entity_event_t::EV_JDETECTED1 as c_int,
                                                    entity_event_t::EV_JDETECTED3 as c_int,
                                                ),
                                                3000,
                                            );
                                        }
                                        (*npc_info).investigateCount += 1;
                                        crate::g_timer::TIMER_Set(
                                            ctx,
                                            npc,
                                            c"watchTime".as_ptr(),
                                            (*world).bg_state.rng.Q_irand(4000, 10000),
                                        );
                                    }
                                }
                                if best_enemy_dist < (440.0 * 440.0)
                                    || (*npc_info).investigateCount >= 2
                                {
                                    //stage three: keep facing him
                                    crate::NPC_utils::NPC_FaceEntity(ctx, best_enemy, qtrue);
                                    if best_enemy_dist < (330.0 * 330.0) {
                                        //stage four: turn on the saber
                                        if (*client).ps.saberInFlight == qfalse {
                                            crate::w_saber::WP_ActivateSaber(ctx, npc);
                                        }
                                    }
                                } else if best_enemy_dist < (550.0 * 550.0)
                                    || (*npc_info).investigateCount == 1
                                {
                                    //stage two: stop and face him every now and then
                                    if crate::g_timer::TIMER_Done(ctx, npc, c"watchTime".as_ptr())
                                        != qfalse
                                    {
                                        crate::NPC_utils::NPC_FaceEntity(ctx, best_enemy, qtrue);
                                    }
                                } else {
                                    //stage one: look at him.
                                    crate::NPC_utils::NPC_SetLookTarget(
                                        npc,
                                        (*best_enemy).s.number,
                                        0,
                                    );
                                }
                                let _ = best_c;
                            }
                        } else if crate::g_timer::TIMER_Done(ctx, npc, c"watchTime".as_ptr())
                            != qfalse
                        {
                            //haven't seen him in a bit, clear the lookTarget
                            crate::NPC_utils::NPC_ClearLookTarget(npc);
                        }
                    }
                }
            }
        }
        //finish:
        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*world).globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);

        if (*npc).enemy.is_some() {
            //just picked one up
            (*npc_info).enemyCheckDebounceTime =
                (*world).level.time + (*world).bg_state.rng.Q_irand(3000, 10000);
        }
    }
}

/// Raven `Jedi_CanPullBackSaber`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5730-5752`
pub fn Jedi_CanPullBackSaber(
    ctx: GameContext<'_>,
    self_: *mut gentity_t,
) -> qboolean {
    unsafe {
        let world = ctx.world;
        let client = (*self_).client as *mut gclient_t;
        if (*client).ps.saberBlocked == BLOCKED_PARRY_BROKEN as c_int
            && crate::g_timer::TIMER_Done(ctx, self_, c"parryTime".as_ptr()) == qfalse
        {
            return qfalse;
        }

        if (*client).NPC_class == CLASS_SHADOWTROOPER
            || (*client).NPC_class == CLASS_TAVION
            || (*client).NPC_class == CLASS_LUKE
            || (*client).NPC_class == CLASS_DESANN
            || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*self_).NPC_type) == 0
        {
            return qtrue;
        }

        if (*self_).painDebounceTime > (*world).level.time {
            return qfalse;
        }

        qtrue
    }
}

/// Raven `NPC_BSJedi_FollowLeader`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5758-5836`
pub fn NPC_BSJedi_FollowLeader(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;

        (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
        if (*npc).enemy.is_none() {
            Jedi_AggressionErosion(ctx, -1);
        }

        //did we drop our saber?  If so, go after it!
        if (*client).ps.saberInFlight != qfalse {
            //saber is not in hand
            if (*client).ps.saberEntityNum < ENTITYNUM_NONE && (*client).ps.saberEntityNum > 0 {
                let saber = ge.add((*client).ps.saberEntityNum as usize);
                if (*saber).s.pos.trType == TR_STATIONARY {
                    //fell to the ground, try to pick it up...
                    if Jedi_CanPullBackSaber(ctx, npc) != qfalse {
                        (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                        (*npc_info).goalEntity = Some(ent_id(ge, saber));
                        (*world).globals.ucmd.buttons |= BUTTON_ATTACK;
                        let enemy: *mut gentity_t = match (*npc).enemy {
                            Some(id) => ge.add(id.0 as usize),
                            None => core::ptr::null_mut(),
                        };
                        if !enemy.is_null() && (*enemy).health > 0 {
                            //get our saber back NOW!
                            if crate::NPC_move::NPC_MoveToGoal(ctx, qtrue) == qfalse {
                                //can't nav to it, try jumping to it
                                let goal: *mut gentity_t =
                                    ge.add((*npc_info).goalEntity.unwrap().0 as usize);
                                crate::NPC_utils::NPC_FaceEntity(ctx, goal, qtrue);
                                Jedi_TryJump(ctx, goal);
                            }
                            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                            return;
                        }
                    }
                }
            }
        }

        if (*npc_info).goalEntity.is_some() {
            let goal: *mut gentity_t = ge.add((*npc_info).goalEntity.unwrap().0 as usize);
            let mut trace: trace_t = core::mem::zeroed();

            if Jedi_Jumping(ctx, goal) != qfalse {
                //in mid-jump
                return;
            }

            if crate::g_nav::NAV_CheckAhead(
                ctx,
                npc,
                (*goal).r.currentOrigin,
                &mut trace,
                ((*npc).clipmask & !CONTENTS_BODY) | CONTENTS_BOTCLIP,
            ) == qfalse
            {
                //can't get straight to him
                if crate::NPC_utils::NPC_ClearLOS4(ctx, goal) != qfalse
                    && crate::NPC_utils::NPC_FaceEntity(ctx, goal, qtrue) != qfalse
                {
                    //no line of sight
                    if Jedi_TryJump(ctx, goal) != qfalse {
                        //started a jump
                        return;
                    }
                }
            }
            if (*npc_info).aiFlags & NPCAI_BLOCKED != 0 {
                //try to jump to the blockedDest
                if ((*npc_info).blockedDest[2] - (*npc).r.currentOrigin[2]).abs() > 64.0 {
                    let tempGoal = crate::g_utils::G_Spawn(ctx);
                    crate::g_utils::G_SetOrigin(tempGoal, (*npc_info).blockedDest);
                    crate::trap::LinkEntity(ctx.engine, mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(tempGoal));
                    crate::g_timer::TIMER_Set(ctx, npc, c"jumpChaseDebounce".as_ptr(), -1);
                    if Jedi_TryJump(ctx, tempGoal) != qfalse {
                        //going to jump to the dest
                        crate::g_utils::G_FreeEntity(ctx, tempGoal);
                        return;
                    }
                    crate::g_utils::G_FreeEntity(ctx, tempGoal);
                }
            }
        }
        //try normal movement
        crate::NPC_behavior::NPC_BSFollowLeader(ctx);
    }
}

/// Raven `Jedi_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:5845-6166`
pub fn Jedi_Attack(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let npc_id = ent_id(ge, npc);

        //Don't do anything if we're in a pain anim
        if (*npc).painDebounceTime > (*world).level.time {
            if (*world).bg_state.rng.Q_irand(0, 1) != 0 {
                Jedi_FaceEnemy(ctx, qtrue);
            }
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }

        if (*client).ps.saberLockTime > (*world).level.time {
            if (*client).ps.fd.forcePowerLevel[FP_PUSH as usize] > FORCE_LEVEL_2
                && (*client).ps.saberLockTime < (*world).level.time + 5000
                && (*world).bg_state.rng.Q_irand(0, 10) == 0
            {
                crate::w_force::ForceThrow(ctx, npc, qfalse);
            } else {
                let chance: f32;
                if (*client).NPC_class == CLASS_DESANN
                    || crate::q_shared::Q_stricmp(c"Yoda".as_ptr(), (*npc).NPC_type) == 0
                {
                    if (*world).cvars.g_spskill.integer != 0 {
                        chance = 4.0f32;
                    } else {
                        chance = 3.0f32;
                    }
                } else if (*client).NPC_class == CLASS_TAVION {
                    chance = 2.0f32 + (*world).cvars.g_spskill.value;
                } else {
                    let maxChance = (RANK_LT as c_int) as f32 / 2.0f32 + 3.0f32;
                    let mut ch;
                    if (*world).cvars.g_spskill.value == 0.0 {
                        ch = (*npc_info).rank as f32 / 2.0f32;
                    } else {
                        ch = (*npc_info).rank as f32 / 2.0f32 + 1.0f32;
                    }
                    if ch > maxChance {
                        ch = maxChance;
                    }
                    chance = ch;
                }
                if (*world).bg_state.rng.flrand(-4.0f32, chance) >= 0.0f32 {
                    (*world).globals.ucmd.buttons |= BUTTON_ATTACK;
                }
            }
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }
        //did we drop our saber?  If so, go after it!
        if (*client).ps.saberInFlight != qfalse {
            //saber is not in hand
            if (*client).ps.saberEntityNum == 0 && (*client).saberStoredIndex != 0 {
                if true {
                    if true {
                        (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                        let saber = ge.add((*client).saberStoredIndex as usize);
                        (*npc_info).goalEntity = Some(ent_id(ge, saber));
                        (*world).globals.ucmd.buttons |= BUTTON_ATTACK;
                        let enemy: *mut gentity_t = match (*npc).enemy {
                            Some(id) => ge.add(id.0 as usize),
                            None => core::ptr::null_mut(),
                        };
                        if !enemy.is_null() && (*enemy).health > 0 {
                            //get our saber back NOW!
                            let goal = ge.add((*npc_info).goalEntity.unwrap().0 as usize);
                            Jedi_Move(ctx, goal, qfalse);
                            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                            if (*enemy).s.weapon == WP_SABER as c_int {
                                //be sure to continue evasion
                                let mut enemy_dir: vec3_t = [0.0; 3];
                                let mut enemy_movedir: vec3_t = [0.0; 3];
                                let mut enemy_dest: vec3_t = [0.0; 3];
                                let mut enemy_dist: f32 = 0.0;
                                let mut enemy_movespeed: f32 = 0.0;
                                Jedi_SetEnemyInfo(
                                    ctx,
                                    &mut enemy_dest,
                                    &mut enemy_dir,
                                    &mut enemy_dist,
                                    &mut enemy_movedir,
                                    &mut enemy_movespeed,
                                    300,
                                );
                                Jedi_EvasionSaber(ctx, enemy_movedir, enemy_dist, enemy_dir);
                            }
                            return;
                        }
                    }
                }
            }
        }
        //see if our enemy was killed by us, gloat and turn off saber after cool down.
        if (*npc).enemy.is_some() {
            let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
            if (*enemy).health <= 0
                && (*enemy).enemy == Some(npc_id)
                && (*client).playerTeam != NPCTEAM_PLAYER as c_int
            {
                //my enemy is dead and I killed him
                (*npc_info).enemyCheckDebounceTime = 0;

                if (*client).NPC_class == CLASS_BOBAFETT {
                    if (*npc_info).walkDebounceTime < (*world).level.time
                        && (*npc_info).walkDebounceTime >= 0
                    {
                        crate::g_timer::TIMER_Set(ctx, npc, c"gloatTime".as_ptr(), 10000);
                        (*npc_info).walkDebounceTime = -1;
                    }
                    if crate::g_timer::TIMER_Done(ctx, npc, c"gloatTime".as_ptr()) == qfalse {
                        if crate::q_math::DistanceHorizontalSquared(
                            (*client).renderInfo.eyePoint,
                            (*enemy).r.currentOrigin,
                        ) > 4096.0
                            && ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) != 0
                        {
                            (*npc_info).goalEntity = (*npc).enemy;
                            Jedi_Move(ctx, enemy, qfalse);
                            (*world).globals.ucmd.buttons |= BUTTON_WALKING;
                        } else {
                            crate::g_timer::TIMER_Set(ctx, npc, c"gloatTime".as_ptr(), 0);
                        }
                    } else if (*npc_info).walkDebounceTime == -1 {
                        (*npc_info).walkDebounceTime = -2;
                        crate::NPC_sounds::G_AddVoiceEvent(
                            ctx,
                            npc,
                            (*world).bg_state.rng.Q_irand(
                                entity_event_t::EV_VICTORY1 as c_int,
                                entity_event_t::EV_VICTORY3 as c_int,
                            ),
                            3000,
                        );
                        (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                            (*world).level.time + 3000;
                        (*npc_info).desiredPitch = 0.0;
                        (*npc_info).goalEntity = None;
                    }
                    Jedi_FaceEnemy(ctx, qtrue);
                    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                } else {
                    if crate::g_timer::TIMER_Done(ctx, npc, c"parryTime".as_ptr()) == qfalse {
                        crate::g_timer::TIMER_Set(ctx, npc, c"parryTime".as_ptr(), -1);
                        (*client).ps.fd.forcePowerDebounce[FP_SABER_DEFENSE as usize] =
                            (*world).level.time + 500;
                    }
                    (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
                    if (*client).ps.saberHolstered == 0 && (*client).ps.saberInFlight != qfalse {
                        //saber is still on, count down erosion and keep facing the enemy
                        Jedi_AggressionErosion(ctx, -3);
                        if crate::bg_pmove::BG_SabersOff(&mut (*client).ps) != qfalse
                            && (*client).ps.saberInFlight == qfalse
                        {
                            //turned off saber (in hand), gloat
                            crate::NPC_sounds::G_AddVoiceEvent(
                                ctx,
                                npc,
                                (*world).bg_state.rng.Q_irand(
                                    entity_event_t::EV_VICTORY1 as c_int,
                                    entity_event_t::EV_VICTORY3 as c_int,
                                ),
                                3000,
                            );
                            (*world).globals.jediSpeechDebounceTime[(*client).playerTeam as usize] =
                                (*world).level.time + 3000;
                            (*npc_info).desiredPitch = 0.0;
                            (*npc_info).goalEntity = None;
                        }
                        crate::g_timer::TIMER_Set(ctx, npc, c"gloatTime".as_ptr(), 10000);
                    }
                    if (*client).ps.saberHolstered == 0
                        || (*client).ps.saberInFlight != qfalse
                        || crate::g_timer::TIMER_Done(ctx, npc, c"gloatTime".as_ptr()) == qfalse
                    {
                        //keep walking
                        if crate::q_math::DistanceHorizontalSquared(
                            (*client).renderInfo.eyePoint,
                            (*enemy).r.currentOrigin,
                        ) > 4096.0
                            && ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) != 0
                        {
                            (*npc_info).goalEntity = (*npc).enemy;
                            Jedi_Move(ctx, enemy, qfalse);
                            (*world).globals.ucmd.buttons |= BUTTON_WALKING;
                        } else {
                            //got there
                            if (*npc).health < (*client).pers.maxHealth
                                && ((*client).ps.fd.forcePowersKnown & (1 << FP_HEAL)) != 0
                                && ((*client).ps.fd.forcePowersActive & (1 << FP_HEAL)) == 0
                            {
                                crate::w_force::ForceHeal(ctx, npc);
                            }
                        }
                        Jedi_FaceEnemy(ctx, qtrue);
                        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                        return;
                    }
                }
            }
        }

        //If we don't have an enemy, just idle
        {
            let enemy: *mut gentity_t = match (*npc).enemy {
                Some(id) => ge.add(id.0 as usize),
                None => core::ptr::null_mut(),
            };
            if !enemy.is_null()
                && (*enemy).s.weapon == WP_TURRET as c_int
                && crate::q_shared::Q_stricmp(c"PAS".as_ptr(), (*enemy).classname) == 0
            {
                if (*enemy).count <= 0 {
                    //it's out of ammo
                    let activator: *mut gentity_t = match (*enemy).activator {
                        Some(id) => ge.add(id.0 as usize),
                        None => core::ptr::null_mut(),
                    };
                    if !activator.is_null()
                        && crate::NPC_utils::NPC_ValidEnemy(ctx, activator) != qfalse
                    {
                        let turretOwner = activator;
                        crate::NPC_combat::G_ClearEnemy(ctx, npc);
                        crate::NPC_combat::G_SetEnemy(ctx, npc, turretOwner);
                    } else {
                        crate::NPC_combat::G_ClearEnemy(ctx, npc);
                    }
                }
            }
        }
        crate::NPC_combat::NPC_CheckEnemy(ctx, qtrue, qtrue, qtrue);

        if (*npc).enemy.is_none() {
            (*client).ps.saberBlocked = BLOCKED_NONE as c_int;
            if (*npc_info).tempBehavior == BS_HUNT_AND_KILL {
                //lost him, go back to what we were doing before
                (*npc_info).tempBehavior = BS_DEFAULT;
                crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            }
            Jedi_Patrol(ctx);
            return;
        }

        //always face enemy if have one
        (*npc_info).combatMove = qtrue;

        //Track the player and kill them if possible
        Jedi_Combat(ctx);

        if ((*npc_info).scriptFlags & SCF_CHASE_ENEMIES) == 0
            || (((*client).ps.fd.forcePowersActive & (1 << FP_HEAL)) != 0
                && (*client).ps.fd.forcePowerLevel[FP_HEAL as usize] < FORCE_LEVEL_2)
        {
            (*world).globals.ucmd.forwardmove = 0;
            (*world).globals.ucmd.rightmove = 0;
            if (*world).globals.ucmd.upmove > 0 {
                (*world).globals.ucmd.upmove = 0;
            }
            (*client).ps.fd.forceJumpCharge = 0.0;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        }

        if (*client).ps.groundEntityNum == ENTITYNUM_NONE {
            //don't push while in air, throws off jumps!
            (*world).globals.ucmd.forwardmove = 0;
            (*world).globals.ucmd.rightmove = 0;
            (*client).ps.moveDir = [0.0, 0.0, 0.0];
        }

        if crate::g_timer::TIMER_Done(ctx, npc, c"duck".as_ptr()) == qfalse {
            (*world).globals.ucmd.upmove = -127;
        }

        if (*client).NPC_class != CLASS_BOBAFETT {
            if crate::bg_saber::PM_SaberInBrokenParry((*client).ps.saberMove) != qfalse
                || (*client).ps.saberBlocked == BLOCKED_PARRY_BROKEN as c_int
            {
                (*world).globals.ucmd.buttons &= !BUTTON_ATTACK;
            }
        }

        if ((*npc_info).scriptFlags & SCF_DONT_FIRE) != 0
            || (((*client).ps.fd.forcePowersActive & (1 << FP_HEAL)) != 0
                && (*client).ps.fd.forcePowerLevel[FP_HEAL as usize] < FORCE_LEVEL_3)
            || (((*client).ps.saberEventFlags & SEF_INWATER as c_int) != 0
                && (*client).ps.saberInFlight == qfalse)
        {
            (*world).globals.ucmd.buttons &= !(BUTTON_ATTACK | BUTTON_ALT_ATTACK);
        }

        if (*npc_info).scriptFlags & SCF_NO_ACROBATICS != 0 {
            (*world).globals.ucmd.upmove = 0;
            (*client).ps.fd.forceJumpCharge = 0.0;
        }

        if (*client).NPC_class != CLASS_BOBAFETT {
            Jedi_CheckDecreaseSaberAnimLevel(ctx);
        }

        if (*world).globals.ucmd.buttons & BUTTON_ATTACK != 0
            && (*client).playerTeam == NPCTEAM_ENEMY as c_int
        {
            if (*world).bg_state.rng.Q_irand(0, (*client).ps.fd.saberAnimLevel) > 0
                && (*world).bg_state.rng.Q_irand(0, (*client).pers.maxHealth + 10) > (*npc).health
                && (*world).bg_state.rng.Q_irand(0, 3) == 0
            {
                crate::NPC_sounds::G_AddVoiceEvent(
                    ctx,
                    npc,
                    (*world).bg_state.rng.Q_irand(
                        entity_event_t::EV_COMBAT1 as c_int,
                        entity_event_t::EV_COMBAT3 as c_int,
                    ),
                    1000,
                );
            }
        }

        if (*client).NPC_class != CLASS_BOBAFETT {
            if (*client).NPC_class == CLASS_TAVION
                || ((*world).cvars.g_spskill.integer != 0
                    && ((*client).NPC_class == CLASS_DESANN
                        || (*npc_info).rank
                            >= (*world)
                                .bg_state
                                .rng
                                .Q_irand(RANK_CREWMAN as c_int, RANK_CAPTAIN as c_int)))
            {
                //Tavion will kick in force speed if the player does...
                let enemy: *mut gentity_t = match (*npc).enemy {
                    Some(id) => ge.add(id.0 as usize),
                    None => core::ptr::null_mut(),
                };
                let enemy_client = if enemy.is_null() {
                    core::ptr::null_mut()
                } else {
                    (*enemy).client as *mut gclient_t
                };
                if !enemy.is_null()
                    && (*enemy).s.number == 0
                    && !(*enemy).client.is_null()
                    && ((*enemy_client).ps.fd.forcePowersActive & (1 << FP_SPEED)) != 0
                    && ((*client).ps.fd.forcePowersActive & (1 << FP_SPEED)) == 0
                {
                    // Raven's switch has case fall-through (no breaks on 0/1),
                    // so chance ends at 1 for skill 0, 1, and 2 (§20 quirk).
                    let mut chance = 0;
                    match (*world).cvars.g_spskill.integer {
                        0 => {
                            chance = 9;
                            chance = 3;
                            chance = 1;
                        }
                        1 => {
                            chance = 3;
                            chance = 1;
                        }
                        2 => {
                            chance = 1;
                        }
                        _ => {}
                    }
                    if (*world).bg_state.rng.Q_irand(0, chance) == 0 {
                        crate::w_force::ForceSpeed(ctx, npc, 0);
                    }
                }
            }
        }
    }
}

/// Raven `NPC_BSJedi_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Jedi.c:6170-6220`
pub fn NPC_BSJedi_Default(ctx: GameContext<'_>) {
    unsafe {
        let world = ctx.world;
        let npc = (*world).globals.NPC;
        let npc_info = (*world).globals.NPCInfo;
        let ge = (*world).g_entities.as_mut_ptr();
        let client = (*npc).client as *mut gclient_t;
        let npc_id = ent_id(ge, npc);

        Jedi_CheckCloak(ctx);
        if (*npc).enemy.is_none() {
            //don't have an enemy, look for one
            if (*client).NPC_class == CLASS_BOBAFETT {
                crate::NPC_AI_Stormtrooper::NPC_BSST_Patrol(ctx);
            } else {
                Jedi_Patrol(ctx);
            }
        } else {
            //have an enemy
            if Jedi_WaitingAmbush(npc) != qfalse {
                //we were still waiting to drop down
                Jedi_Ambush(ctx, npc);
            }
            let enemy: *mut gentity_t = ge.add((*npc).enemy.unwrap().0 as usize);
            if (*client).NPC_class == CLASS_BOBAFETT {
                if (*enemy).enemy != Some(npc_id)
                    && (*npc).health == (*client).pers.maxHealth
                    && crate::q_math::DistanceSquared(
                        (*npc).r.currentOrigin,
                        (*enemy).r.currentOrigin,
                    ) > (800.0 * 800.0)
                {
                    (*npc_info).scriptFlags |= SCF_ALT_FIRE;
                    Boba_ChangeWeapon(ctx, WP_DISRUPTOR as c_int);
                    crate::NPC_AI_Sniper::NPC_BSSniper_Default(ctx);
                    return;
                }
            }
            Jedi_Attack(ctx);
            //if we have multiple-jedi combat, keep checking for a better enemy
            if (((*world).globals.ucmd.buttons == 0 && (*client).ps.fd.forcePowersActive == 0)
                || ((*npc).enemy.is_some()
                    && (*ge.add((*npc).enemy.unwrap().0 as usize)).health <= 0))
                && (*npc_info).enemyCheckDebounceTime < (*world).level.time
            {
                //not doing anything, not using force powers and it's time to look again
                let sav_enemy = (*npc).enemy; //FIXME: what about NPC->lastEnemy?
                let newEnemy: *mut gentity_t;

                (*npc).enemy = None;
                newEnemy = crate::NPC_combat::NPC_CheckEnemy(
                    ctx,
                    if (*npc_info).confusionTime < (*world).level.time { qtrue } else { qfalse },
                    qfalse,
                    qfalse,
                );
                (*npc).enemy = sav_enemy;
                let sav_enemy_ptr: *mut gentity_t = match sav_enemy {
                    Some(id) => ge.add(id.0 as usize),
                    None => core::ptr::null_mut(),
                };
                if !newEnemy.is_null() && newEnemy != sav_enemy_ptr {
                    //picked up a new enemy!
                    (*npc).lastEnemy = (*npc).enemy;
                    crate::NPC_combat::G_SetEnemy(ctx, npc, newEnemy);
                }
                (*npc_info).enemyCheckDebounceTime =
                    (*world).level.time + (*world).bg_state.rng.Q_irand(1000, 3000);
            }
        }
    }
}
