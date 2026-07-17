//! Port of `oracle/codemp/game/w_force.c` (jampgame force-power logic).
//!
//! Generated from `tools/closure-prototype/fnskel.py`; bodies filled per the
//! jampgame mega-pass (settled fork rulings,
//! `docs/handoffs/jampgame-fork-discovery.md`).
//!
//! SPINE (`docs/architecture/engine-seam.md`): logic fns that
//! reach `level`/cvars/`g_entities`/traps thread the `GameContext<'_>` receiver
//! (`.world: &mut GameWorld`, `.engine`) — the only ported-logic precedent
//! (`g_init_game`). Globals are `GameWorld` fields: `level` →
//! `ctx.world.level`, cvars → `ctx.world.cvars`, `g_entities[i]` →
//! `ctx.world.g_entities[i]`. Traps go through `trap::X(ctx.engine, …)`.
//! Cross-file callees are invoked with the packet's resolved raw-pointer
//! signatures verbatim (their own porters thread the spine).
//!
//! Raw `gentity_t*`/`gclient_t*`/`playerState_t*` chains are transcribed as
//! `unsafe` raw-pointer field access mirroring the C exactly (the fnskel
//! skeletons operate in raw-pointer space; `GameContext.world` is itself a raw
//! pointer). EntityId reshaping lands in the later integration pass.
//!
//! NOTE (integration-deferred): the packet does not enumerate the Raven
//! constant spellings (`EV_*`, `FP_*`, `FORCE_LEVEL_*`, `PDSOUND_*`, `CHAN_*`,
//! …) nor their owning enums; they are transcribed by their faithful Raven
//! names (the port preserves them) and their exact enum-qualification / module
//! path is resolved at integration (the mega-pass tree is not compiled per
//! porter — "Do NOT run cargo"). `forcePowerNeeded` is the bg-shared const
//! table (const tables stay const), referenced by its Raven name.
//!
//! Safe-state migration **Stage 1 → 2c**: entity-pointer params are `EntityId` /
//! `Option<EntityId>` handles (§B5) instead of raw `gentity_t*`; ctx-free leaf
//! helpers borrow `&gentity_t`/`&mut gentity_t`. Entity fields are read/written
//! through `ctx.world.entity(id)`/`entity_mut(id)` at the point of use.
//! `gclient_t`/vehicle-vtable derefs stay raw in tight `unsafe` blocks through a
//! copied pointer value (recipe 2b; NPC targets carry pool clients). Behavior is
//! byte-identical, referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use crate::npc::g_npc_t::gNPC_t;
use crate::prelude::*;
use mp_bg::local::force_power_needed::forcePowerNeeded;
use mp_bg::public::duel_team::duelTeam_t::DUELTEAM_LONE;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

/// Per-file `g_entities` base-pointer helper for `EntityId` arena resolution
/// (matches the `g_missile.rs`/`g_trigger.rs`/`NPC_combat.rs` precedent).
#[inline]
unsafe fn ent_base(ctx: &mut GameContext) -> *const gentity_t {
    ctx.world.g_entities.as_ptr()
}

// Raven force-mastery-level anonymous enum (bg_public.h) → int-wide consts per
// the enum-vs-alias rule (anonymous enum → `const`s). Only the two spellings the
// ported bodies name are surfaced.
// Source: `oracle/codemp/game/bg_public.h:383-392`
pub const FORCE_MASTERY_UNINITIATED: c_int = 0;
pub const FORCE_MASTERY_INITIATE: c_int = 1;
pub const FORCE_MASTERY_PADAWAN: c_int = 2;
pub const FORCE_MASTERY_JEDI: c_int = 3;
pub const FORCE_MASTERY_JEDI_GUARDIAN: c_int = 4;
pub const FORCE_MASTERY_JEDI_ADEPT: c_int = 5;
pub const FORCE_MASTERY_JEDI_KNIGHT: c_int = 6;
pub const FORCE_MASTERY_JEDI_MASTER: c_int = 7;
pub const NUM_FORCE_MASTERY_LEVELS: c_int = 8;
use crate::ai_main::{InFieldOfVision, OrgVisible};
use crate::g_cmds::Cmd_ToggleSaber_f;
use crate::g_combat::{G_Damage, TossClientWeapon};
use crate::g_missile::G_ReflectMissile;
use crate::g_team::OnSameTeam;
use crate::g_utils::{
    G_EffectIndex, G_EntitySound, G_MuteSound, G_PlayEffect, G_PlayEffectID, G_SetAnim, G_Sound,
    G_SoundAtLoc, G_SoundIndex, G_TempEntity, GlobalUse,
};
use crate::g_weapon::WP_FireGenericBlasterMissile;
use crate::level::spawn_flags::SPF_BUTTON_FPUSHABLE;
use crate::q_math::{
    vectoangles, AngleSubtract, AngleVectors, DirToByte, VectorLength, VectorNormalize,
};
use crate::trap;
use crate::w_saber::HasSetSaberOnly;
use crate::world::GameContext;
use crate::NPC_AI_Jedi::Jedi_Decloak;
use crate::NPC_senses::InFront;
use mp_bg::bg_misc::{BG_CanUseFPNow, BG_HasYsalamiri, BG_LegalizedForcePowers};
use mp_bg::bg_panimate::{
    BG_FullBodyTauntAnim, BG_InReboundHold, BG_InReboundJump, BG_SaberInSpecial,
};
use mp_bg::bg_pmove::BG_InKnockDown;
use mp_bg::bg_saber::BG_ForcePowerDrain;

// vec3 origin (`{0,0,0}`), the all-zero trace mins/maxs sentinel.
use crate::q_math::vec3_origin;

// Const/enum families transcribed by faithful Raven name (file header note).
use crate::entity::hit_location::*;
use crate::level::damage_flags::*;
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_CVAR_UPDATE::GCvarUpdateArgs;
use mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs;
use mp_abi::game::syscalls::G_GET_USERINFO::GGetUserinfoArgs;
use mp_abi::game::syscalls::G_SEND_SERVER_COMMAND::GSendServerCommandArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::public::anim_number::animNumber_t::*;
use mp_bg::public::effect_types::effectTypes_t::*;
use mp_bg::public::jump_velocity::JUMP_VELOCITY;
use mp_bg::public::weaponstate::weaponstate_t::*;
use mp_qshared::common::mp::qcommon::usercmd_button::*;

/// Raven `M_PI` (`<math.h>`), used by the seeker-drone orbit math.
const M_PI: f64 = std::f64::consts::PI;

// `PITCH`/`YAW`/`ROLL` (`crate::q_math`), `PMF_FOLLOW`/`PMF_STUCK_TO_WALL`
// (`mp_qshared::…::pm_flags`) and `SFL_TWO_HANDED` (`crate::saber::saber_flags`,
// the canonical `SFL_*` home) all resolve via the crate prelude glob; the
// shadowing local copies were removed by the placeholder-const sweep.

use mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs;
use mp_bg::public::entity_event::entity_event_t::{
    EV_FORCE_DRAINED, EV_PREDEFSOUND, EV_TEAM_POWER,
};

/// Raven `mindTrickTime` per force-mastery level (ms).
///
/// Source: `oracle/codemp/game/w_force.c:139-145`
pub const mindTrickTime: [c_int; 4] = [0 /*none*/, 5000, 10000, 15000];

/// Raven `G_PreDefSound` — spawn a predefined-sound temp entity at `org`.
///
/// Source: `oracle/codemp/game/w_force.c:40-49`
pub fn G_PreDefSound(ctx: &mut GameContext, org: vec3_t, pdSound: c_int) -> *mut gentity_t {
    let te_id = G_TempEntity(ctx, org, EV_PREDEFSOUND as c_int);
    let te = ctx.entity_mut(te_id) as *mut gentity_t;
    let e = ctx.world.entity_mut(te_id);
    e.s.eventParm = pdSound;
    e.s.origin = org; // VectorCopy(org, te->s.origin)
    te
}

/// Raven `WP_InitForcePowers`.
///
/// Source: `oracle/codemp/game/w_force.c:147-572`
pub fn WP_InitForcePowers(ctx: &mut GameContext, ent: Option<EntityId>) {
    unsafe {
        let mut maxRank = ctx.world.cvars.g_maxForceRank.integer;
        let mut warnClient = qfalse;
        let warnClientLimit = qfalse;
        let mut lastFPKnown: c_int = -1;
        let mut didEvent = qfalse;

        if maxRank == 0 {
            //if server has no max rank, default to max (50)
            maxRank = FORCE_MASTERY_JEDI_MASTER as c_int;
        } else if maxRank >= NUM_FORCE_MASTERY_LEVELS as c_int {
            //ack, prevent user from being dumb
            maxRank = FORCE_MASTERY_JEDI_MASTER as c_int;
            let val = format!("{}", maxRank);
            trap::Cvar_Set(
                ctx.engine,
                mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs::new(
                    std::ffi::CString::new("g_maxForceRank").unwrap(),
                    std::ffi::CString::new(val).unwrap(),
                ),
            );
        }

        // FLAG: gclient_t deref stays raw (`ent` can be an NPC carrying a pool
        // client, recipe 2b); read the pointer value via the safe entity borrow.
        let ent = match ent {
            Some(e) if !ctx.entity(e).client.is_null() => e,
            _ => return,
        };
        let cl = ctx.entity(ent).client;
        // `ent`'s eType/number/svFlags are invariant across this fn (only `cl`,
        // temp entities, and cvars/traps are mutated) — hoist once as Copy locals.
        let ent_etype = ctx.entity(ent).s.eType;
        let ent_number = ctx.entity(ent).s.number;
        let ent_svflags = ctx.entity(ent).r.svFlags;

        (*cl).ps.fd.saberAnimLevel = (*cl).sess.saberLevel;

        if (*cl).ps.fd.saberAnimLevel < FORCE_LEVEL_1 as c_int
            || (*cl).ps.fd.saberAnimLevel > FORCE_LEVEL_3 as c_int
        {
            (*cl).ps.fd.saberAnimLevel = FORCE_LEVEL_1 as c_int;
        }

        if ctx.world.speedLoopSound == 0 {
            //so that the client configstring is already modified with this when we need it
            let s = cstr("sound/weapons/force/speedloop.wav");
            ctx.world.speedLoopSound = G_SoundIndex(s.as_ptr());
        }
        if ctx.world.rageLoopSound == 0 {
            let s = cstr("sound/weapons/force/rageloop.wav");
            ctx.world.rageLoopSound = G_SoundIndex(s.as_ptr());
        }
        if ctx.world.absorbLoopSound == 0 {
            let s = cstr("sound/weapons/force/absorbloop.wav");
            ctx.world.absorbLoopSound = G_SoundIndex(s.as_ptr());
        }
        if ctx.world.protectLoopSound == 0 {
            let s = cstr("sound/weapons/force/protectloop.wav");
            ctx.world.protectLoopSound = G_SoundIndex(s.as_ptr());
        }
        if ctx.world.seeLoopSound == 0 {
            let s = cstr("sound/weapons/force/seeloop.wav");
            ctx.world.seeLoopSound = G_SoundIndex(s.as_ptr());
        }
        if ctx.world.ysalamiriLoopSound == 0 {
            let s = cstr("sound/player/nullifyloop.wav");
            ctx.world.ysalamiriLoopSound = G_SoundIndex(s.as_ptr());
        }

        if ent_etype == ET_NPC as c_int {
            //just stop here then.
            return;
        }

        for i in 0..NUM_FORCE_POWERS {
            (*cl).ps.fd.forcePowerLevel[i as usize] = 0;
            (*cl).ps.fd.forcePowersKnown &= !(1 << i);
        }

        (*cl).ps.fd.forcePowerSelected = -1;
        (*cl).ps.fd.forceSide = 0;

        let gametype = ctx.world.cvars.g_gametype.integer;

        if gametype == GT_SIEGE as c_int && (*cl).siegeClass != -1 {
            //Then use the powers for this class, and skip all this nonsense.
            for i in 0..NUM_FORCE_POWERS as usize {
                (*cl).ps.fd.forcePowerLevel[i] = (&ctx.world.bg_state.bgSiegeClasses)
                    [(*cl).siegeClass as usize]
                    .forcePowerLevels[i];
                if (*cl).ps.fd.forcePowerLevel[i] == 0 {
                    (*cl).ps.fd.forcePowersKnown &= !(1 << i);
                } else {
                    (*cl).ps.fd.forcePowersKnown |= 1 << i;
                }
            }

            if (*cl).sess.setForce == 0 {
                //bring up the class selection menu
                trap::SendServerCommand(
                    ctx.engine,
                    GSendServerCommandArgs::new(ent_number, cstr("scl")),
                );
            }
            (*cl).sess.setForce = qtrue;
            return;
        }

        let mut userinfo: [c_char; 1024] = [0; 1024];
        // Raven `char forcePowers[256]` — an IN/OUT C buffer: `BG_LegalizedForcePowers`
        // legalizes it in place below, and the parse loop that follows reads the
        // legalized contents back out of this same buffer, not a stale copy.
        // Source: `oracle/codemp/game/w_force.c:155` (`char forcePowers[256];`)
        let mut forcePowers: [c_char; 256] = [0; 256];

        if ent_etype == ET_NPC as c_int && ent_number >= MAX_CLIENTS as c_int {
            //rwwFIXMEFIXME: Temp
            write_cstr_field(&mut userinfo, "forcepowers\\7-1-333003000313003120");
        } else {
            trap::GetUserinfo(
                ctx.engine,
                mp_abi::game::syscalls::G_GET_USERINFO::GGetUserinfoArgs::new(
                    ent_number,
                    userinfo.as_mut_ptr(),
                    userinfo.len() as c_int,
                ),
            );
        }

        let userinfo_str = cstr_to_str(userinfo.as_ptr());
        let key = cstr("forcepowers");
        let val = Info_ValueForKey(
            &mut ctx.world.bg_state.qs,
            cstr(&userinfo_str).as_ptr(),
            key.as_ptr(),
        );
        write_cstr_field(&mut forcePowers, &cstr_to_str(val));

        if ent_svflags & SVF_BOT != 0
            && !(&ctx.world.globals.botstates)[ent_number as usize].is_null()
        {
            //if it's a bot just copy the info directly from its personality
            let bot_forceinfo = cstr_to_str(
                (*(&ctx.world.globals.botstates)[ent_number as usize])
                    .forceinfo
                    .as_ptr() as *const c_char,
            );
            write_cstr_field(&mut forcePowers, &bot_forceinfo);
        }

        //rww - parse through the string manually and eat out all the appropriate data
        let mut i: usize = 0;

        if ctx.world.cvars.g_forceBasedTeams.integer != 0 {
            if (*cl).sess.sessionTeam == TEAM_RED {
                warnClient = (BG_LegalizedForcePowers(
                    forcePowers.as_mut_ptr(),
                    maxRank,
                    HasSetSaberOnly(ctx),
                    FORCE_DARKSIDE as c_int,
                    gametype,
                    ctx.world.cvars.g_forcePowerDisable.integer,
                ) == 0) as qboolean;
            } else if (*cl).sess.sessionTeam == TEAM_BLUE {
                warnClient = (BG_LegalizedForcePowers(
                    forcePowers.as_mut_ptr(),
                    maxRank,
                    HasSetSaberOnly(ctx),
                    FORCE_LIGHTSIDE as c_int,
                    gametype,
                    ctx.world.cvars.g_forcePowerDisable.integer,
                ) == 0) as qboolean;
            } else {
                warnClient = (BG_LegalizedForcePowers(
                    forcePowers.as_mut_ptr(),
                    maxRank,
                    HasSetSaberOnly(ctx),
                    0,
                    gametype,
                    ctx.world.cvars.g_forcePowerDisable.integer,
                ) == 0) as qboolean;
            }
        } else {
            warnClient = (BG_LegalizedForcePowers(
                forcePowers.as_mut_ptr(),
                maxRank,
                HasSetSaberOnly(ctx),
                0,
                gametype,
                ctx.world.cvars.g_forcePowerDisable.integer,
            ) == 0) as qboolean;
        }

        // Read the buffer back out post-legalize (Raven re-reads `forcePowers[i]`
        // in the parse loop below — the same array `BG_LegalizedForcePowers` just
        // wrote into), not the pre-call string.
        let fp_bytes = cstr_to_str(forcePowers.as_ptr()).into_bytes();

        let mut i_r: usize;
        let mut readBuf: [u8; 256] = [0; 256];

        i_r = 0;
        while i < fp_bytes.len() && fp_bytes[i] != b'-' {
            readBuf[i_r] = fp_bytes[i];
            i_r += 1;
            i += 1;
        }
        readBuf[i_r] = 0;
        //THE RANK
        // Source: oracle/codemp/game/w_force.c:316 — plain `atoi(readBuf)`.
        (*cl).ps.fd.forceRank = atoi_str(&String::from_utf8_lossy(&readBuf[..i_r]));
        i += 1;

        i_r = 0;
        while i < fp_bytes.len() && fp_bytes[i] != b'-' {
            readBuf[i_r] = fp_bytes[i];
            i_r += 1;
            i += 1;
        }
        readBuf[i_r] = 0;
        //THE SIDE
        // Source: oracle/codemp/game/w_force.c:328 — plain `atoi(readBuf)`.
        (*cl).ps.fd.forceSide = atoi_str(&String::from_utf8_lossy(&readBuf[..i_r]));
        i += 1;

        let mut fp_bytes = fp_bytes;
        if gametype != GT_SIEGE as c_int
            && ent_svflags & SVF_BOT != 0
            && !(&ctx.world.globals.botstates)[ent_number as usize].is_null()
        {
            //hmm..I'm going to cheat here.
            let oldI = i;
            i_r = 0;
            while i < fp_bytes.len()
                && fp_bytes[i] != b'\n'
                && (i_r as c_int) < (NUM_FORCE_POWERS) as i32
            {
                if (*cl).ps.fd.forceSide == FORCE_LIGHTSIDE as c_int {
                    if i_r as c_int == FP_ABSORB {
                        fp_bytes[i] = b'3';
                    }
                    if (*(&ctx.world.globals.botstates)[ent_number as usize])
                        .settings
                        .skill
                        >= 4.0
                    {
                        //cheat and give them more stuff
                        if i_r as c_int == FP_HEAL {
                            fp_bytes[i] = b'3';
                        } else if i_r as c_int == FP_PROTECT {
                            fp_bytes[i] = b'3';
                        }
                    }
                } else if (*cl).ps.fd.forceSide == FORCE_DARKSIDE as c_int {
                    if (*(&ctx.world.globals.botstates)[ent_number as usize])
                        .settings
                        .skill
                        >= 4.0
                    {
                        if i_r as c_int == FP_GRIP {
                            fp_bytes[i] = b'3';
                        } else if i_r as c_int == FP_LIGHTNING {
                            fp_bytes[i] = b'3';
                        } else if i_r as c_int == FP_RAGE {
                            fp_bytes[i] = b'3';
                        } else if i_r as c_int == FP_DRAIN {
                            fp_bytes[i] = b'3';
                        }
                    }
                }

                if i_r as c_int == FP_PUSH {
                    fp_bytes[i] = b'3';
                } else if i_r as c_int == FP_PULL {
                    fp_bytes[i] = b'3';
                }

                i += 1;
                i_r += 1;
            }
            i = oldI;
        }

        i_r = 0;
        while i < fp_bytes.len()
            && fp_bytes[i] != b'\n'
            && (i_r as c_int) < (NUM_FORCE_POWERS) as i32
        {
            let ch = fp_bytes[i];
            // Oracle builds a 1-char `readBuf` (`readBuf[0]=forcePowers[i];
            // readBuf[1]=0;`) and calls `atoi(readBuf)` on it
            // (w_force.c:398-402) — over a single-char domain,
            // `to_digit(10).unwrap_or(0)` is exactly libc `atoi`, so this is
            // not re-flagged to `cstr_util::atoi`.
            let digit = (ch as char).to_digit(10).unwrap_or(0) as c_int;
            (*cl).ps.fd.forcePowerLevel[i_r] = digit;
            if (*cl).ps.fd.forcePowerLevel[i_r] != 0 {
                (*cl).ps.fd.forcePowersKnown |= 1 << i_r;
            } else {
                (*cl).ps.fd.forcePowersKnown &= !(1 << i_r);
            }
            i += 1;
            i_r += 1;
        }
        //THE POWERS

        if ent_etype != ET_NPC as c_int {
            if HasSetSaberOnly(ctx) != 0 {
                let te_id = G_TempEntity(ctx, vec3_origin, EV_SET_FREE_SABER as c_int);
                let e = ctx.world.entity_mut(te_id);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eventParm = 1;
            } else {
                let te_id = G_TempEntity(ctx, vec3_origin, EV_SET_FREE_SABER as c_int);
                let e = ctx.world.entity_mut(te_id);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eventParm = 0;
            }

            if ctx.world.cvars.g_forcePowerDisable.integer != 0 {
                let te_id = G_TempEntity(ctx, vec3_origin, EV_SET_FORCE_DISABLE as c_int);
                let e = ctx.world.entity_mut(te_id);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eventParm = 1;
            } else {
                let te_id = G_TempEntity(ctx, vec3_origin, EV_SET_FORCE_DISABLE as c_int);
                let e = ctx.world.entity_mut(te_id);
                e.r.svFlags |= SVF_BROADCAST;
                e.s.eventParm = 0;
            }
        }

        if ent_etype == ET_NPC as c_int {
            (*cl).sess.setForce = qtrue;
        } else if gametype == GT_SIEGE as c_int {
            if (*cl).sess.setForce == 0 {
                (*cl).sess.setForce = qtrue;
                //bring up the class selection menu
                trap::SendServerCommand(
                    ctx.engine,
                    GSendServerCommandArgs::new(ent_number, cstr("scl")),
                );
            }
        } else {
            if warnClient != 0 || (*cl).sess.setForce == 0 {
                //the client's rank is too high for the server and has been autocapped, so tell them
                if gametype != GT_HOLOCRON as c_int && gametype != GT_JEDIMASTER as c_int {
                    didEvent = qtrue;

                    if ent_svflags & SVF_BOT == 0 && ent_etype != ET_NPC as c_int {
                        if ctx.world.cvars.g_teamAutoJoin.integer == 0 {
                            //Make them a spectator so they can set their powerups up without being bothered.
                            (*cl).sess.sessionTeam = TEAM_SPECTATOR;
                            (*cl).sess.spectatorState =
                                crate::client::spectator_state::spectatorState_t::SPECTATOR_FREE;
                            (*cl).sess.spectatorClient = 0;

                            (*cl).pers.teamState.state =
                                crate::client::player_team_state::playerTeamStateState_t::TEAM_BEGIN;
                            trap::SendServerCommand(
                                ctx.engine,
                                GSendServerCommandArgs::new(ent_number, cstr("spc")),
                            ); // Fire up the profile menu
                        }
                    }

                    //Event isn't very reliable, I made it a string. This way I can send it to just one
                    //client also, as opposed to making a broadcast event.
                    let msg = format!("nfr {} {} {}", maxRank, 1, (*cl).sess.sessionTeam as c_int);
                    trap::SendServerCommand(
                        ctx.engine,
                        GSendServerCommandArgs::new(ent_number, cstr(&msg)),
                    );
                    //Arg1 is new max rank, arg2 is non-0 if force menu should be shown, arg3 is the current team
                }
                (*cl).sess.setForce = qtrue;
            }

            if didEvent == 0 {
                let msg = format!("nfr {} {} {}", maxRank, 0, (*cl).sess.sessionTeam as c_int);
                trap::SendServerCommand(
                    ctx.engine,
                    GSendServerCommandArgs::new(ent_number, cstr(&msg)),
                );
            }

            if warnClientLimit != 0 {
                //the server has one or more force powers disabled and the client is using them in his config
                //(kept commented in the oracle — no-op here too)
            }
        }

        i = 0;
        while (i as c_int) < (NUM_FORCE_POWERS) as i32 {
            if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0 && (*cl).ps.fd.forcePowerLevel[i] == 0 {
                //err..
                (*cl).ps.fd.forcePowersKnown &= !(1 << i);
            } else {
                if i as c_int != FP_LEVITATION
                    && i as c_int != FP_SABER_OFFENSE
                    && i as c_int != FP_SABER_DEFENSE
                    && i as c_int != FP_SABERTHROW
                {
                    lastFPKnown = i as c_int;
                }
            }
            i += 1;
        }

        if (*cl).ps.fd.forcePowersKnown & (*cl).sess.selectedFP != 0 {
            (*cl).ps.fd.forcePowerSelected = (*cl).sess.selectedFP;
        }

        // Raven shifts by forcePowerSelected while it can still be -1 (fresh client,
        // set to -1 above) — shift-by-negative UB; x86/ARM both mask the count (= 1<<31,
        // never a known power), so the masked shift is the one defined behavior (§19).
        if (*cl).ps.fd.forcePowersKnown & 1i32.wrapping_shl((*cl).ps.fd.forcePowerSelected as u32)
            == 0
        {
            if lastFPKnown != -1 {
                (*cl).ps.fd.forcePowerSelected = lastFPKnown;
            } else {
                (*cl).ps.fd.forcePowerSelected = 0;
            }
        }

        while (i as c_int) < (NUM_FORCE_POWERS) as i32 {
            (*cl).ps.fd.forcePowerBaseLevel[i] = (*cl).ps.fd.forcePowerLevel[i];
            i += 1;
        }
        (*cl).ps.fd.forceUsingAdded = 0;
    }
}

/// Raven `WP_SpawnInitForcePowers` — reset per-spawn force state.
///
/// Source: `oracle/codemp/game/w_force.c:574-691`
pub fn WP_SpawnInitForcePowers(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (`ent` can be an NPC pool client,
        // recipe 2b); read the pointer value via the safe entity borrow.
        let cl = ctx.entity(ent).client;

        (*cl).ps.saberAttackChainCount = 0;

        for i in 0..NUM_FORCE_POWERS as usize {
            if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 {
                WP_ForcePowerStop(ctx, ent, i as forcePowers_t);
            }
        }

        (*cl).ps.fd.forceDeactivateAll = 0;

        (*cl).ps.fd.forcePower = FORCE_POWER_MAX as c_int;
        (*cl).ps.fd.forcePowerMax = FORCE_POWER_MAX as c_int;
        (*cl).ps.fd.forcePowerRegenDebounceTime = 0;
        (*cl).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;
        (*cl).ps.fd.forceMindtrickTargetIndex = 0;
        (*cl).ps.fd.forceMindtrickTargetIndex2 = 0;
        (*cl).ps.fd.forceMindtrickTargetIndex3 = 0;
        (*cl).ps.fd.forceMindtrickTargetIndex4 = 0;

        (*cl).ps.holocronBits = 0;

        for i in 0..NUM_FORCE_POWERS as usize {
            (*cl).ps.holocronsCarried[i] = 0.0;
        }

        let gametype = ctx.world.cvars.g_gametype.integer;

        if gametype == GT_HOLOCRON as c_int {
            for i in 0..NUM_FORCE_POWERS as usize {
                (*cl).ps.fd.forcePowerLevel[i] = FORCE_LEVEL_0 as c_int;
            }

            if HasSetSaberOnly(ctx) != 0 {
                if (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 as c_int {
                    (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1 as c_int;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < FORCE_LEVEL_1 as c_int {
                    (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = FORCE_LEVEL_1 as c_int;
                }
            }
        }

        for i in 0..NUM_FORCE_POWERS as usize {
            (*cl).ps.fd.forcePowerDebounce[i] = 0;
            (*cl).ps.fd.forcePowerDuration[i] = 0;
        }

        (*cl).ps.fd.forcePowerRegenDebounceTime = 0;
        (*cl).ps.fd.forceJumpZStart = 0.0;
        (*cl).ps.fd.forceJumpCharge = 0.0;
        (*cl).ps.fd.forceJumpSound = 0;
        (*cl).ps.fd.forceGripDamageDebounceTime = 0;
        (*cl).ps.fd.forceGripBeingGripped = 0.0;
        (*cl).ps.fd.forceGripCripple = 0;
        (*cl).ps.fd.forceGripUseTime = 0;
        (*cl).ps.fd.forceGripSoundTime = 0.0;
        (*cl).ps.fd.forceGripStarted = 0.0;
        (*cl).ps.fd.forceHealTime = 0;
        (*cl).ps.fd.forceHealAmount = 0;
        (*cl).ps.fd.forceRageRecoveryTime = 0;
        (*cl).ps.fd.forceDrainEntNum = ENTITYNUM_NONE;
        (*cl).ps.fd.forceDrainTime = 0.0;

        for i in 0..NUM_FORCE_POWERS as usize {
            if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0 && (*cl).ps.fd.forcePowerLevel[i] == 0 {
                //make sure all known powers are cleared if we have level 0 in them
                (*cl).ps.fd.forcePowersKnown &= !(1 << i);
            }
        }

        if gametype == GT_SIEGE as c_int && (*cl).siegeClass != -1 {
            //Then use the powers for this class.
            for i in 0..NUM_FORCE_POWERS as usize {
                (*cl).ps.fd.forcePowerLevel[i] = (&ctx.world.bg_state.bgSiegeClasses)
                    [(*cl).siegeClass as usize]
                    .forcePowerLevels[i];
                if (*cl).ps.fd.forcePowerLevel[i] == 0 {
                    (*cl).ps.fd.forcePowersKnown &= !(1 << i);
                } else {
                    (*cl).ps.fd.forcePowersKnown |= 1 << i;
                }
            }
        }
    }
}

/// Raven `ForcePowerUsableOn` — can `attacker` use `forcePower` on `other`?
///
/// Source: `oracle/codemp/game/w_force.c:697-772`
pub fn ForcePowerUsableOn(
    ctx: &mut GameContext,
    attacker: Option<EntityId>,
    other: Option<EntityId>,
    forcePower: forcePowers_t,
) -> c_int {
    unsafe {
        let gametype = ctx.world.cvars.g_gametype.integer;
        let level_time = ctx.world.level.time;

        // FLAG: gclient_t derefs stay raw (`attacker`/`other` can be NPCs/vehicles
        // carrying pool clients, recipe 2b). `*_cl` is null when the handle is
        // `None` OR its `.client` is null — exactly Raven's paired guard. `other`
        // scalar fields are invariant here, hoisted once as Copy locals.
        let other_cl: *mut gclient_t = match other {
            Some(o) => ctx.entity(o).client,
            None => core::ptr::null_mut(),
        };
        let attacker_cl: *mut gclient_t = match attacker {
            Some(a) => ctx.entity(a).client,
            None => core::ptr::null_mut(),
        };
        let (other_number, other_etype, other_nclass) = match other {
            Some(o) => {
                let e = ctx.entity(o);
                (e.s.number, e.s.eType, e.s.NPC_class)
            }
            None => (0, 0, 0),
        };

        if !other_cl.is_null() && BG_HasYsalamiri(gametype, &mut (*other_cl).ps) != 0 {
            return 0;
        }

        if !attacker_cl.is_null()
            && BG_CanUseFPNow(gametype, &mut (*attacker_cl).ps, level_time, forcePower) == 0
        {
            return 0;
        }

        //Dueling fighters cannot use force powers on others, with the exception of force push when locked with each other
        if !attacker_cl.is_null() && (*attacker_cl).ps.duelInProgress != 0 {
            return 0;
        }

        if !other_cl.is_null() && (*other_cl).ps.duelInProgress != 0 {
            return 0;
        }

        if forcePower == FP_GRIP {
            if !other_cl.is_null() && (*other_cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
                //don't allow gripping to begin with if they are absorbing
                //play sound indicating that attack was absorbed
                if (*other_cl).forcePowerSoundDebounce < level_time {
                    let abSound =
                        G_PreDefSound(ctx, (*other_cl).ps.origin, PDSOUND_ABSORBHIT as c_int);
                    let ab_id = ctx.entity_id_of(abSound).unwrap();
                    ctx.world.entity_mut(ab_id).s.trickedentindex = other_number;
                    (*other_cl).forcePowerSoundDebounce = level_time + 400;
                }
                return 0;
            } else if !other_cl.is_null()
                && (*other_cl).ps.weapon == WP_SABER
                && BG_SaberInSpecial((*other_cl).ps.saberMove) != 0
            {
                //don't grip person while they are in a special or some really bad things can happen.
                return 0;
            }
        }

        if !other_cl.is_null() && (forcePower == FP_PUSH || forcePower == FP_PULL) {
            if BG_InKnockDown((*other_cl).ps.legsAnim) != 0 {
                return 0;
            }
        }

        if !other_cl.is_null()
            && other_etype == ET_NPC as c_int
            && other_nclass == CLASS_VEHICLE as c_int
        {
            //can't use the force on vehicles.. except lightning
            if forcePower == FP_LIGHTNING {
                return 1;
            } else {
                return 0;
            }
        }

        if !other_cl.is_null() && other_etype == ET_NPC as c_int && gametype == GT_SIEGE {
            //can't use powers at all on npc's normally in siege...
            return 0;
        }

        1
    }
}

/// Raven `WP_ForcePowerAvailable` — is there enough force pool for `forcePower`?
///
/// Source: `oracle/codemp/game/w_force.c:774-801`
pub fn WP_ForcePowerAvailable(
    ctx: &mut GameContext,
    self_: EntityId,
    forcePower: forcePowers_t,
    overrideAmt: c_int,
) -> qboolean {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let drain = if overrideAmt != 0 {
            overrideAmt
        } else {
            forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[forcePower as usize] as usize]
                [forcePower as usize]
        };

        if (*cl).ps.fd.forcePowersActive & (1 << forcePower) != 0 {
            //we're probably going to deactivate it..
            return qtrue;
        }
        if forcePower == FP_LEVITATION {
            return qtrue;
        }
        if drain == 0 {
            return qtrue;
        }
        if (forcePower == FP_DRAIN || forcePower == FP_LIGHTNING) && (*cl).ps.fd.forcePower >= 25 {
            //it's ok then, drain/lightning are actually duration
            return qtrue;
        }
        if (*cl).ps.fd.forcePower < drain {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `WP_ForcePowerInUse`.
///
/// Source: `oracle/codemp/game/w_force.c:803-811`
pub fn WP_ForcePowerInUse(self_: &gentity_t, forcePower: forcePowers_t) -> qboolean {
    // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
    let cl = self_.client;
    if unsafe { (*cl).ps.fd.forcePowersActive } & (1 << forcePower) != 0 {
        //already using this power
        return qtrue;
    }
    qfalse
}

/// Raven `WP_ForcePowerUsable` — full gate on activating `forcePower`.
///
/// Source: `oracle/codemp/game/w_force.c:813-938`
pub fn WP_ForcePowerUsable(
    ctx: &mut GameContext,
    self_: EntityId,
    forcePower: forcePowers_t,
) -> qboolean {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let self_health = ctx.entity(self_).health;
        let gametype = ctx.world.cvars.g_gametype.integer;
        let level_time = ctx.world.level.time;

        if BG_HasYsalamiri(gametype, &mut (*cl).ps) != 0 {
            return qfalse;
        }

        if self_health <= 0
            || (*cl).ps.stats[STAT_HEALTH as usize] <= 0
            || (*cl).ps.eFlags & EF_DEAD != 0
        {
            return qfalse;
        }

        if (*cl).ps.pm_flags & PMF_FOLLOW != 0 {
            //specs can't use powers through people
            return qfalse;
        }
        if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
            return qfalse;
        }
        if (*cl).tempSpectate >= level_time {
            return qfalse;
        }

        if BG_CanUseFPNow(gametype, &mut (*cl).ps, level_time, forcePower) == 0 {
            return qfalse;
        }

        if (*cl).ps.fd.forcePowersKnown & (1 << forcePower) == 0 {
            //don't know this power
            return qfalse;
        }

        if (*cl).ps.fd.forcePowersActive & (1 << forcePower) != 0 {
            //already using this power
            if forcePower != FP_LEVITATION {
                return qfalse;
            }
        }

        if forcePower == FP_LEVITATION && (*cl).fjDidJump != 0 {
            return qfalse;
        }

        if (*cl).ps.fd.forcePowerLevel[forcePower as usize] == 0 {
            return qfalse;
        }

        if ctx.world.cvars.g_debugMelee.integer != 0 {
            if (*cl).ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                //no offensive force powers when stuck to wall
                match forcePower {
                    FP_GRIP | FP_LIGHTNING | FP_DRAIN | FP_SABER_OFFENSE | FP_SABER_DEFENSE
                    | FP_SABERTHROW => return qfalse,
                    _ => {}
                }
            }
        }

        if (*cl).ps.saberHolstered == 0 {
            if (*cl).saber[0].saberFlags & SFL_TWO_HANDED != 0 {
                if ctx.world.cvars.g_saberRestrictForce.integer != 0 {
                    match forcePower {
                        FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                            return qfalse
                        }
                        _ => {}
                    }
                }
            }

            if (*cl).saber[0].saberFlags & SFL_TWO_HANDED != 0 || ((*cl).saber[0].model[0] != 0) {
                //this saber requires the use of two hands OR our other hand is using an active saber too
                if (*cl).saber[0].forceRestrictions & (1 << forcePower) != 0 {
                    //this power is verboten when using this saber
                    return qfalse;
                }
            }

            if (*cl).saber[0].model[0] != 0 {
                //both sabers on
                if ctx.world.cvars.g_saberRestrictForce.integer != 0 {
                    match forcePower {
                        FP_PUSH | FP_PULL | FP_TELEPATHY | FP_GRIP | FP_LIGHTNING | FP_DRAIN => {
                            return qfalse
                        }
                        _ => {}
                    }
                }
                if (*cl).saber[1].forceRestrictions & (1 << forcePower) != 0 {
                    //this power is verboten when using this saber
                    return qfalse;
                }
            }
        }
        WP_ForcePowerAvailable(ctx, self_, forcePower, 0)
        // OVERRIDEFIXME
    }
}

/// Raven `WP_AbsorbConversion` — absorb an incoming force attack, return the
/// remaining (post-absorb) power level, or `-1` when not absorbed.
///
/// Source: `oracle/codemp/game/w_force.c:940-997`
pub fn WP_AbsorbConversion(
    ctx: &mut GameContext,
    attacked: EntityId,
    atdAbsLevel: c_int,
    attacker: Option<EntityId>,
    atPower: c_int,
    atPowerLevel: c_int,
    atForceSpent: c_int,
) -> c_int {
    // `attacker` is unused in this body (Raven never reads it here).
    let _ = attacker;
    unsafe {
        let mut getLevel;
        let mut addTot;

        if atPower != FP_LIGHTNING
            && atPower != FP_DRAIN
            && atPower != FP_GRIP
            && atPower != FP_PUSH
            && atPower != FP_PULL
        {
            //Only these powers can be absorbed
            return -1;
        }

        if atdAbsLevel == 0 {
            //looks like attacker doesn't have any absorb power
            return -1;
        }

        // FLAG: gclient_t deref stays raw (`attacked` can be an NPC pool client,
        // recipe 2b).
        let atcl = ctx.entity(attacked).client;
        if (*atcl).ps.fd.forcePowersActive & (1 << FP_ABSORB) == 0 {
            //absorb is not active
            return -1;
        }

        //Subtract absorb power level from the offensive force power
        getLevel = atPowerLevel;
        getLevel -= atdAbsLevel;

        if getLevel < 0 {
            getLevel = 0;
        }

        //let the attacker absorb an amount of force used in this attack based on his level of absorb
        addTot = (atForceSpent / 3) * (*atcl).ps.fd.forcePowerLevel[FP_ABSORB as usize];

        if addTot < 1 && atForceSpent >= 1 {
            addTot = 1;
        }
        (*atcl).ps.fd.forcePower += addTot;
        if (*atcl).ps.fd.forcePower > 100 {
            (*atcl).ps.fd.forcePower = 100;
        }

        //play sound indicating that attack was absorbed
        let level_time = ctx.world.level.time;
        if (*atcl).forcePowerSoundDebounce < level_time {
            let abSound = G_PreDefSound(ctx, (*atcl).ps.origin, PDSOUND_ABSORBHIT as c_int);
            let ab_id = ctx.entity_id_of(abSound).unwrap();
            let attacked_number = ctx.entity(attacked).s.number;
            ctx.world.entity_mut(ab_id).s.trickedentindex = attacked_number;

            (*atcl).forcePowerSoundDebounce = level_time + 400;
        }

        getLevel
    }
}

/// Raven `WP_ForcePowerRegenerate` — regen the force pool on a regular interval.
///
/// Source: `oracle/codemp/game/w_force.c:999-1019`
pub fn WP_ForcePowerRegenerate(self_: &gentity_t, overrideAmt: c_int) {
    // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
    let cl = self_.client;
    unsafe {
        if cl.is_null() {
            return;
        }

        if overrideAmt != 0 {
            //custom regen amount
            (*cl).ps.fd.forcePower += overrideAmt;
        } else {
            //otherwise, just 1
            (*cl).ps.fd.forcePower += 1;
        }

        if (*cl).ps.fd.forcePower > (*cl).ps.fd.forcePowerMax {
            //cap it off at the max (default 100)
            (*cl).ps.fd.forcePower = (*cl).ps.fd.forcePowerMax;
        }
    }
}

/// Raven `WP_ForcePowerStart` — activate the given force power.
///
/// Source: `oracle/codemp/game/w_force.c:1021-1234`
pub fn WP_ForcePowerStart(
    ctx: &mut GameContext,
    self_: EntityId,
    forcePower: forcePowers_t,
    mut overrideAmt: c_int,
) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let mut duration: c_int = 0;
        let mut hearable = qfalse;
        let mut hearDist: f32 = 0.0;

        if WP_ForcePowerAvailable(ctx, self_, forcePower, overrideAmt) == 0 {
            return;
        }

        if BG_FullBodyTauntAnim((*cl).ps.legsAnim) != 0 {
            //stop taunt
            (*cl).ps.legsTimer = 0;
        }
        if BG_FullBodyTauntAnim((*cl).ps.torsoAnim) != 0 {
            //stop taunt
            (*cl).ps.torsoTimer = 0;
        }
        //hearable and hearDist are merely for the benefit of bots, and not related to if a sound is actually played.
        //If duration is set, the force power will assume to be timer-based.
        match forcePower {
            FP_HEAL => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_LEVITATION => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_SPEED => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_2 {
                    duration = 15000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                } else {
                    //shouldn't get here
                    // break;
                }
                if duration != 0 || (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize] >= FORCE_LEVEL_1
                {
                    if overrideAmt != 0 {
                        duration = overrideAmt;
                    }
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_PUSH => {
                hearable = qtrue;
                hearDist = 256.0;
            }
            FP_PULL => {
                hearable = qtrue;
                hearDist = 256.0;
            }
            FP_TELEPATHY => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
                    duration = 20000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
                    duration = 25000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_GRIP => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                (*cl).ps.powerups[PW_DISINT_4 as usize] = level_time + 60000;
            }
            FP_LIGHTNING => {
                hearable = qtrue;
                hearDist = 512.0;
                duration = overrideAmt;
                overrideAmt = 0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                (*cl).ps.activeForcePass = (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize];
            }
            FP_RAGE => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                    duration = 8000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                    duration = 14000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                    duration = 20000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_PROTECT => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = 20000;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_ABSORB => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = 20000;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_TEAM_HEAL => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_TEAM_FORCE => {
                hearable = qtrue;
                hearDist = 256.0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_DRAIN => {
                hearable = qtrue;
                hearDist = 256.0;
                duration = overrideAmt;
                overrideAmt = 0;
                (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
            }
            FP_SEE => {
                hearable = qtrue;
                hearDist = 256.0;
                if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_1 {
                    duration = 10000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_2 {
                    duration = 20000;
                } else if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] == FORCE_LEVEL_3 {
                    duration = 30000;
                }
                if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] >= FORCE_LEVEL_1 {
                    (*cl).ps.fd.forcePowersActive |= 1 << forcePower;
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            _ => {}
        }

        if duration != 0 {
            (*cl).ps.fd.forcePowerDuration[forcePower as usize] = level_time + duration;
        } else {
            (*cl).ps.fd.forcePowerDuration[forcePower as usize] = 0;
        }

        if hearable != 0 {
            (*cl).ps.otherSoundLen = hearDist;
            (*cl).ps.otherSoundTime = level_time + 100;
        }

        (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = 0;

        if forcePower == FP_SPEED && overrideAmt != 0 {
            BG_ForcePowerDrain(
                &mut (*cl).ps,
                forcePower,
                (overrideAmt as f32 * 0.025) as c_int,
            );
        } else if forcePower != FP_GRIP && forcePower != FP_DRAIN {
            //grip and drain drain as damage is done
            BG_ForcePowerDrain(&mut (*cl).ps, forcePower, overrideAmt);
        }
    }
}

/// Raven `ForceHeal`.
///
/// Source: `oracle/codemp/game/w_force.c:1236-1292`
pub fn ForceHeal(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_HEAL) == 0 {
            return;
        }

        if ctx.entity(self_).health >= (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_3 {
            ctx.entity_mut(self_).health += 25; //This was 50, but that angered the Balance God.
            if ctx.entity(self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                ctx.entity_mut(self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        } else if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2 {
            ctx.entity_mut(self_).health += 10;
            if ctx.entity(self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                ctx.entity_mut(self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        } else {
            ctx.entity_mut(self_).health += 5;
            if ctx.entity(self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                ctx.entity_mut(self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
            }
            BG_ForcePowerDrain(&mut (*cl).ps, FP_HEAL, 0);
        }
        //NOTE: Decided to make all levels instant.

        let snd = std::ffi::CString::new("sound/weapons/force/heal.wav").unwrap();
        G_Sound(ctx, Some(self_), CHAN_ITEM, G_SoundIndex(snd.as_ptr()));
    }
}

/// Raven `WP_AddToClientBitflags` — pack `entNum` into a temp-ent's tricked-index
/// bitfields.
///
/// Source: `oracle/codemp/game/w_force.c:1294-1317`
pub fn WP_AddToClientBitflags(ent: Option<&mut gentity_t>, entNum: c_int) {
    let ent = match ent {
        Some(e) => e,
        None => return,
    };

    if entNum > 47 {
        ent.s.trickedentindex4 |= 1 << (entNum - 48);
    } else if entNum > 31 {
        ent.s.trickedentindex3 |= 1 << (entNum - 32);
    } else if entNum > 15 {
        ent.s.trickedentindex2 |= 1 << (entNum - 16);
    } else {
        ent.s.trickedentindex |= 1 << entNum;
    }
}

/// Raven `ForceTeamHeal`.
///
/// Source: `oracle/codemp/game/w_force.c:1319-1422`
pub fn ForceTeamHeal(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        let mut radius: f32 = 256.0;
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let mut numpl: usize = 0;
        let mut pl: [usize; MAX_CLIENTS as usize] = [0; MAX_CLIENTS as usize];
        let healthadd: c_int;
        let mut te: *mut gentity_t = std::ptr::null_mut();

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_TEAM_HEAL) == 0 {
            return;
        }

        if (*cl).ps.fd.forcePowerDebounce[FP_TEAM_HEAL as usize] >= level_time {
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] == FORCE_LEVEL_2 as c_int {
            radius *= 1.5;
        }
        if (*cl).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] == FORCE_LEVEL_3 as c_int {
            radius *= 2.0;
        }

        for i in 0..MAX_CLIENTS as usize {
            let ent_id = EntityId(i as u32);
            let ent_cl = ctx.entity(ent_id).client;

            if !ent_cl.is_null()
                && self_ != ent_id
                && OnSameTeam(ctx, Some(self_), Some(ent_id)) != 0
                && (*ent_cl).ps.stats[STAT_HEALTH as usize]
                    < (*ent_cl).ps.stats[STAT_MAX_HEALTH as usize]
                && (*ent_cl).ps.stats[STAT_HEALTH as usize] > 0
                && ForcePowerUsableOn(ctx, Some(self_), Some(ent_id), FP_TEAM_HEAL) != 0
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*cl).ps.origin as *const vec3_t,
                        &(*ent_cl).ps.origin as *const vec3_t,
                    ),
                ) != 0
            {
                let a: vec3_t = [
                    (*cl).ps.origin[0] - (*ent_cl).ps.origin[0],
                    (*cl).ps.origin[1] - (*ent_cl).ps.origin[1],
                    (*cl).ps.origin[2] - (*ent_cl).ps.origin[2],
                ];

                if VectorLength(a) <= radius {
                    pl[numpl] = i;
                    numpl += 1;
                }
            }
        }

        if numpl < 1 {
            return;
        }

        if numpl == 1 {
            healthadd = 50;
        } else if numpl == 2 {
            healthadd = 33;
        } else {
            healthadd = 25;
        }

        (*cl).ps.fd.forcePowerDebounce[FP_TEAM_HEAL as usize] = level_time + 2000;

        for i in 0..numpl {
            let ent_id = EntityId(pl[i] as u32);
            let ocl = ctx.entity(ent_id).client;
            if (*ocl).ps.stats[STAT_HEALTH as usize] > 0 && ctx.entity(ent_id).health > 0 {
                (*ocl).ps.stats[STAT_HEALTH as usize] += healthadd;
                if (*ocl).ps.stats[STAT_HEALTH as usize] > (*ocl).ps.stats[STAT_MAX_HEALTH as usize]
                {
                    (*ocl).ps.stats[STAT_HEALTH as usize] =
                        (*ocl).ps.stats[STAT_MAX_HEALTH as usize];
                }

                ctx.entity_mut(ent_id).health = (*ocl).ps.stats[STAT_HEALTH as usize];

                //At this point we know we got one, so add him into the collective event client bitflag
                if te.is_null() {
                    let __teid17 = G_TempEntity(ctx, (*cl).ps.origin, EV_TEAM_POWER as c_int);
                    te = ctx.entity_mut(__teid17);
                    let te_id = ctx.entity_id_of(te).unwrap();
                    ctx.world.entity_mut(te_id).s.eventParm = 1; //eventParm 1 is heal, eventParm 2 is force regen

                    //since we had an extra check above, do the drain now because we got at least one guy
                    BG_ForcePowerDrain(
                        &mut (*cl).ps,
                        FP_TEAM_HEAL,
                        forcePowerNeeded
                            [(*cl).ps.fd.forcePowerLevel[FP_TEAM_HEAL as usize] as usize]
                            [FP_TEAM_HEAL as usize],
                    );
                }

                WP_AddToClientBitflags(te.as_mut(), pl[i] as c_int);
                //Now cramming it all into one event.. doing this many g_sound events at once was a Bad Thing.
            }
        }
    }
}

/// Raven `ForceTeamForceReplenish`.
///
/// Source: `oracle/codemp/game/w_force.c:1424-1521`
pub fn ForceTeamForceReplenish(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        let mut radius: f32 = 256.0;
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let mut numpl: usize = 0;
        let mut pl: [usize; MAX_CLIENTS as usize] = [0; MAX_CLIENTS as usize];
        let poweradd: c_int;
        let mut te: *mut gentity_t = std::ptr::null_mut();

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_TEAM_FORCE) == 0 {
            return;
        }

        if (*cl).ps.fd.forcePowerDebounce[FP_TEAM_FORCE as usize] >= level_time {
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] == FORCE_LEVEL_2 as c_int {
            radius *= 1.5;
        }
        if (*cl).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] == FORCE_LEVEL_3 as c_int {
            radius *= 2.0;
        }

        for i in 0..MAX_CLIENTS as usize {
            let ent_id = EntityId(i as u32);
            let ent_cl = ctx.entity(ent_id).client;

            if !ent_cl.is_null()
                && self_ != ent_id
                && OnSameTeam(ctx, Some(self_), Some(ent_id)) != 0
                && (*ent_cl).ps.fd.forcePower < 100
                && ForcePowerUsableOn(ctx, Some(self_), Some(ent_id), FP_TEAM_FORCE) != 0
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(
                        &(*cl).ps.origin as *const vec3_t,
                        &(*ent_cl).ps.origin as *const vec3_t,
                    ),
                ) != 0
            {
                let a: vec3_t = [
                    (*cl).ps.origin[0] - (*ent_cl).ps.origin[0],
                    (*cl).ps.origin[1] - (*ent_cl).ps.origin[1],
                    (*cl).ps.origin[2] - (*ent_cl).ps.origin[2],
                ];

                if VectorLength(a) <= radius {
                    pl[numpl] = i;
                    numpl += 1;
                }
            }
        }

        if numpl < 1 {
            return;
        }

        if numpl == 1 {
            poweradd = 50;
        } else if numpl == 2 {
            poweradd = 33;
        } else {
            poweradd = 25;
        }
        (*cl).ps.fd.forcePowerDebounce[FP_TEAM_FORCE as usize] = level_time + 2000;

        BG_ForcePowerDrain(
            &mut (*cl).ps,
            FP_TEAM_FORCE,
            forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[FP_TEAM_FORCE as usize] as usize]
                [FP_TEAM_FORCE as usize],
        );

        for i in 0..numpl {
            let ent_id = EntityId(pl[i] as u32);
            let ocl = ctx.entity(ent_id).client;
            (*ocl).ps.fd.forcePower += poweradd;
            if (*ocl).ps.fd.forcePower > 100 {
                (*ocl).ps.fd.forcePower = 100;
            }

            //At this point we know we got one, so add him into the collective event client bitflag
            if te.is_null() {
                let __teid18 = G_TempEntity(ctx, (*cl).ps.origin, EV_TEAM_POWER as c_int);
                te = ctx.entity_mut(__teid18);
                let te_id = ctx.entity_id_of(te).unwrap();
                ctx.world.entity_mut(te_id).s.eventParm = 2; //eventParm 1 is heal, eventParm 2 is force regen
            }

            WP_AddToClientBitflags(te.as_mut(), pl[i] as c_int);
            //Now cramming it all into one event.. doing this many g_sound events at once was a Bad Thing.
        }
    }
}

/// Raven `ForceGrip`.
///
/// Source: `oracle/codemp/game/w_force.c:1523-1594`
pub fn ForceGrip(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); read pointer value via the
        // safe entity borrow. `self_`'s number is invariant here — hoist as a local.
        let cl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if (*cl).ps.fd.forceGripUseTime > level_time {
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_GRIP) == 0 {
            return;
        }

        let mut tfrom: vec3_t = (*cl).ps.origin;
        tfrom[2] += (*cl).ps.viewheight as f32;
        let mut fwd: vec3_t = [0.0; 3];
        AngleVectors((*cl).ps.viewangles, Some(&mut fwd), None, None);
        let tto: vec3_t = [
            tfrom[0] + fwd[0] * MAX_GRIP_DISTANCE as f32,
            tfrom[1] + fwd[1] * MAX_GRIP_DISTANCE as f32,
            tfrom[2] + fwd[2] * MAX_GRIP_DISTANCE as f32,
        ];

        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &tfrom as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &tto as *const vec3_t,
                self_number,
                MASK_PLAYERSOLID,
            ),
        );

        if tr.fraction != 1.0
            && tr.entityNum != (ENTITYNUM_NONE) as i16
            && !ctx.world.g_entities[tr.entityNum as usize].client.is_null()
            && (*(ctx.world.g_entities[tr.entityNum as usize].client))
                .ps
                .fd
                .forceGripCripple
                == 0
            && (*(ctx.world.g_entities[tr.entityNum as usize].client))
                .ps
                .fd
                .forceGripBeingGripped
                < level_time as f32
            && ForcePowerUsableOn(
                ctx,
                Some(self_),
                Some(EntityId((tr.entityNum) as u32)),
                FP_GRIP,
            ) != 0
            && (ctx.world.cvars.g_friendlyFire.integer != 0
                || OnSameTeam(ctx, Some(self_), EntityId::from_num(tr.entityNum as c_int)) == 0)
        //don't grip someone who's still crippled
        {
            let target_id = EntityId(tr.entityNum as u32);
            // FLAG: gclient_t deref stays raw (grip target may be an NPC pool
            // client, recipe 2b).
            let tcl = ctx.entity(target_id).client;

            if ctx.entity(target_id).s.number < MAX_CLIENTS as c_int && (*tcl).ps.m_iVehicleNum != 0
            {
                //a player on a vehicle
                let veh_id = EntityId((*tcl).ps.m_iVehicleNum as u32);
                if ctx.entity(veh_id).inuse != qfalse
                    && !ctx.entity(veh_id).client.is_null()
                    && !ctx.entity(veh_id).m_pVehicle.is_null()
                {
                    // FLAG: Vehicle_t (`m_pVehicle`/`m_pVehicleInfo`) is a C++ seam
                    // type with no accessor; derefs stay raw.
                    let pVeh = ctx.entity(veh_id).m_pVehicle;
                    if (*(*pVeh).m_pVehicleInfo).r#type == vehicleType_t::VH_SPEEDER
                        || (*(*pVeh).m_pVehicleInfo).r#type == vehicleType_t::VH_ANIMAL
                    {
                        //push the guy off
                        // FLAG: `veh_dispatch::eject` is a seam fn taking a raw
                        // `bgEntity_t*` alongside `ctx`; feed a copied arena raw
                        // pointer (can't restructure without touching that file).
                        let target_raw =
                            ctx.world.entity_mut(target_id) as *mut gentity_t as *mut bgEntity_t;
                        crate::veh_dispatch::eject(ctx, pVeh, target_raw, qfalse);
                    }
                }
            }
            (*cl).ps.fd.forceGripEntityNum = (tr.entityNum) as i32;
            (*tcl).ps.fd.forceGripStarted = level_time as f32;
            (*cl).ps.fd.forceGripDamageDebounceTime = 0;

            (*cl).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD as c_int;
            (*cl).ps.forceHandExtendTime = level_time + 5000;
        } else {
            (*cl).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;
            return;
        }
    }
}

/// Raven `ForceSpeed`.
///
/// Source: `oracle/codemp/game/w_force.c:1596-1629`
pub fn ForceSpeed(ctx: &mut GameContext, self_: EntityId, forceDuration: c_int) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_SPEED);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_SPEED) == 0 {
            return;
        }

        if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
            && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
        {
            //holding Siege item
            if ctx.world.g_entities[(*cl).holdingObjectiveItem as usize].genericValue15 != 0 {
                //disables force powers
                return;
            }
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_SPEED, forceDuration);
        let snd = std::ffi::CString::new("sound/weapons/force/speed.wav").unwrap();
        G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(snd.as_ptr()));
        let loop_sound = ctx.world.speedLoopSound;
        G_Sound(ctx, Some(self_), TRACK_CHANNEL_2 as c_int, loop_sound);
    }
}

/// Raven `ForceSeeing`.
///
/// Source: `oracle/codemp/game/w_force.c:1631-1656`
pub fn ForceSeeing(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_SEE) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_SEE);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_SEE) == 0 {
            return;
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_SEE, 0);

        let snd = std::ffi::CString::new("sound/weapons/force/see.wav").unwrap();
        G_Sound(ctx, Some(self_), CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
        let loop_sound = ctx.world.seeLoopSound;
        G_Sound(ctx, Some(self_), TRACK_CHANNEL_5 as c_int, loop_sound);
    }
}

/// Raven `ForceProtect`.
///
/// Source: `oracle/codemp/game/w_force.c:1658-1692`
pub fn ForceProtect(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_PROTECT) == 0 {
            return;
        }

        // Make sure to turn off Force Rage and Force Absorb.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_PROTECT, 0);
        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_PROTECT as c_int);
        let loop_sound = ctx.world.protectLoopSound;
        G_Sound(ctx, Some(self_), TRACK_CHANNEL_3 as c_int, loop_sound);
    }
}

/// Raven `ForceAbsorb`.
///
/// Source: `oracle/codemp/game/w_force.c:1694-1728`
pub fn ForceAbsorb(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_ABSORB) == 0 {
            return;
        }

        // Make sure to turn off Force Rage and Force Protection.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_ABSORB, 0);
        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_ABSORB as c_int);
        let loop_sound = ctx.world.absorbLoopSound;
        G_Sound(ctx, Some(self_), TRACK_CHANNEL_3 as c_int, loop_sound);
    }
}

/// Raven `ForceRage`.
///
/// Source: `oracle/codemp/game/w_force.c:1730-1775`
pub fn ForceRage(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_RAGE) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_RAGE);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_RAGE) == 0 {
            return;
        }

        if (*cl).ps.fd.forceRageRecoveryTime >= level_time {
            return;
        }

        if ctx.entity(self_).health < 10 {
            return;
        }

        // Make sure to turn off Force Protection and Force Absorb.
        if (*cl).ps.fd.forcePowersActive & (1 << FP_PROTECT) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_PROTECT);
        }
        if (*cl).ps.fd.forcePowersActive & (1 << FP_ABSORB) != 0 {
            WP_ForcePowerStop(ctx, self_, FP_ABSORB);
        }

        (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

        WP_ForcePowerStart(ctx, self_, FP_RAGE, 0);

        let snd = std::ffi::CString::new("sound/weapons/force/rage.wav").unwrap();
        G_Sound(
            ctx,
            Some(self_),
            TRACK_CHANNEL_4 as c_int,
            G_SoundIndex(snd.as_ptr()),
        );
        let loop_sound = ctx.world.rageLoopSound;
        G_Sound(ctx, Some(self_), TRACK_CHANNEL_3 as c_int, loop_sound);
    }
}

/// Raven `ForceLightning`.
///
/// Source: `oracle/codemp/game/w_force.c:1777-1810`
pub fn ForceLightning(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }
        if (*cl).ps.fd.forcePower < 25 || WP_ForcePowerUsable(ctx, self_, FP_LIGHTNING) == 0 {
            return;
        }
        if (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] > level_time {
            //stops it while using it and also after using it, up to 3 second delay
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        //Shoot lightning from hand
        //using grip anim now, to extend the burst time
        (*cl).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD as c_int;
        (*cl).ps.forceHandExtendTime = level_time + 20000;

        let snd = std::ffi::CString::new("sound/weapons/force/lightning").unwrap();
        G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(snd.as_ptr()));

        WP_ForcePowerStart(ctx, self_, FP_LIGHTNING, 500);
    }
}

/// Raven `ForceLightningDamage` — apply a lightning tick to `traceEnt`.
///
/// Source: `oracle/codemp/game/w_force.c:1812-1900`
pub fn ForceLightningDamage(
    ctx: &mut GameContext,
    self_: EntityId,
    traceEnt: Option<EntityId>,
    mut dir: vec3_t,
    impactPoint: vec3_t,
) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let scl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        (*scl).dangerTime = level_time;
        (*scl).ps.eFlags &= !EF_INVULNERABLE;
        (*scl).invulnerableTimer = 0;

        // Raven: `if (traceEnt && traceEnt->takedamage)` — the rest of the body is
        // the last statement, so a non-taking / null target returns here.
        let te = match traceEnt {
            Some(t) if ctx.entity(t).takedamage != 0 => t,
            _ => return,
        };

        if ctx.entity(te).client.is_null() && ctx.entity(te).s.eType == ET_NPC as c_int {
            //g2animent
            if ctx.entity(te).s.genericenemyindex < level_time {
                ctx.entity_mut(te).s.genericenemyindex = level_time + 2000;
            }
        }
        if !ctx.entity(te).client.is_null() {
            //an enemy or object
            let tcl = ctx.entity(te).client;
            if (*tcl).noLightningTime >= level_time {
                //give them power and don't hurt them.
                (*tcl).ps.fd.forcePower += 1;
                if (*tcl).ps.fd.forcePower > 100 {
                    (*tcl).ps.fd.forcePower = 100;
                }
                return;
            }
            if ForcePowerUsableOn(ctx, Some(self_), Some(te), FP_LIGHTNING) != 0 {
                let mut dmg = ctx.world.bg_state.rng.Q_irand(1, 2); //ctx.world.bg_state.rng.Q_irand( 1, 3 );

                let mut modPowerLevel = -1;

                if !tcl.is_null() {
                    modPowerLevel = WP_AbsorbConversion(
                        ctx,
                        te,
                        (*tcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                        Some(self_),
                        FP_LIGHTNING,
                        (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize],
                        1,
                    );
                }

                if modPowerLevel != -1 {
                    if modPowerLevel == 0 {
                        dmg = 0;
                        (*tcl).noLightningTime = level_time + 400;
                    } else if modPowerLevel == 1 {
                        dmg = 1;
                        (*tcl).noLightningTime = level_time + 300;
                    } else if modPowerLevel == 2 {
                        dmg = 1;
                        (*tcl).noLightningTime = level_time + 100;
                    }
                }

                if (*scl).ps.weapon == WP_MELEE
                    && (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_2
                {
                    //2-handed lightning
                    //jackin' 'em up, Palpatine-style
                    dmg *= 2;
                }

                if dmg != 0 {
                    //rww - Shields can now absorb lightning too.
                    G_Damage(
                        ctx,
                        Some(te),
                        Some(self_),
                        Some(self_),
                        Some(&mut dir),
                        impactPoint,
                        dmg,
                        0,
                        MOD_FORCE_DARK as c_int,
                    );
                }
                if !tcl.is_null() {
                    if ctx.world.bg_state.rng.Q_irand(0, 2) == 0 {
                        let snd = std::ffi::CString::new(format!(
                            "sound/weapons/force/lightninghit{}",
                            ctx.world.bg_state.rng.Q_irand(1, 3)
                        ))
                        .unwrap();
                        G_Sound(ctx, Some(te), CHAN_BODY, G_SoundIndex(snd.as_ptr()));
                    }

                    if (*tcl).ps.electrifyTime < (level_time + 400) {
                        //only update every 400ms to reduce bandwidth usage (as it is passing a 32-bit time value)
                        (*tcl).ps.electrifyTime = level_time + 800;
                    }
                    if (*tcl).ps.powerups[PW_CLOAKED as usize] != 0 {
                        //disable cloak temporarily
                        Jedi_Decloak(ctx, Some(te));
                        (*tcl).cloakToggleTime =
                            level_time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
                    }
                }
            }
        }
    }
}

/// Raven `ForceShootLightning`.
///
/// Source: `oracle/codemp/game/w_force.c:1902-2020`
pub fn ForceShootLightning(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); `self_`'s number is
        // invariant here — hoist as a local.
        let scl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;

        if ctx.entity(self_).health <= 0 {
            return;
        }
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors((*scl).ps.viewangles, Some(&mut forward), None, None);
        VectorNormalize(&mut forward);

        let mut tr: trace_t = core::mem::zeroed();

        if (*scl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_2 {
            //arc
            let radius: f32 = FORCE_LIGHTNING_RADIUS as f32;
            let center: vec3_t = (*scl).ps.origin;
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            for i in 0..3 {
                mins[i] = center[i] - radius;
                maxs[i] = center[i] + radius;
            }
            let mut iEntityList = [0i32; MAX_GENTITIES as usize];
            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    iEntityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let te_id = EntityId(iEntityList[e as usize] as u32);

                if te_id == self_ {
                    continue;
                }
                // `traceEnt`'s fields are invariant until the terminal
                // `ForceLightningDamage` call — hoist the reads as Copy locals.
                let te_ownerNum = ctx.entity(te_id).r.ownerNum;
                let te_weapon = ctx.entity(te_id).s.weapon;
                let te_inuse = ctx.entity(te_id).inuse;
                let te_takedamage = ctx.entity(te_id).takedamage;
                let te_health = ctx.entity(te_id).health;
                let te_absmin = ctx.entity(te_id).r.absmin;
                let te_absmax = ctx.entity(te_id).r.absmax;
                let te_bmodel = ctx.entity(te_id).r.bmodel;
                let te_number = ctx.entity(te_id).s.number;

                if te_ownerNum == self_number && te_weapon != WP_THERMAL
                //can push your own thermals
                {
                    continue;
                }
                if te_inuse == 0 {
                    continue;
                }
                if te_takedamage == 0 {
                    continue;
                }
                if te_health <= 0 {
                    //no torturing corpses
                    continue;
                }
                if ctx.world.cvars.g_friendlyFire.integer == 0
                    && OnSameTeam(ctx, Some(self_), Some(te_id)) != 0
                {
                    continue;
                }

                // find the distance from the edge of the bounding box
                let mut v: vec3_t = [0.0; 3];
                for i in 0..3 {
                    if center[i] < te_absmin[i] {
                        v[i] = te_absmin[i] - center[i];
                    } else if center[i] > te_absmax[i] {
                        v[i] = center[i] - te_absmax[i];
                    } else {
                        v[i] = 0.0;
                    }
                }

                let size: vec3_t = [
                    te_absmax[0] - te_absmin[0],
                    te_absmax[1] - te_absmin[1],
                    te_absmax[2] - te_absmin[2],
                ];
                let ent_org: vec3_t = [
                    te_absmin[0] + 0.5 * size[0],
                    te_absmin[1] + 0.5 * size[1],
                    te_absmin[2] + 0.5 * size[2],
                ];

                //see if they're in front of me / within the forward cone
                let mut dir: vec3_t = [
                    ent_org[0] - center[0],
                    ent_org[1] - center[1],
                    ent_org[2] - center[2],
                ];
                VectorNormalize(&mut dir);
                let dot = dir[0] * forward[0] + dir[1] * forward[1] + dir[2] * forward[2];
                if dot < 0.5 {
                    continue;
                }

                //must be close enough
                let dist = VectorLength(v);
                if dist >= radius {
                    continue;
                }

                //in PVS?
                if te_bmodel == 0
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &ent_org as *const vec3_t,
                            &(*scl).ps.origin as *const vec3_t,
                        ),
                    ) == 0
                {
                    //must be in PVS
                    continue;
                }

                //Now check and see if we can actually hit it
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*scl).ps.origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &ent_org as *const vec3_t,
                        self_number,
                        MASK_SHOT,
                    ),
                );
                if tr.fraction < 1.0 && tr.entityNum != (te_number) as i16 {
                    //must have clear LOS
                    continue;
                }

                // ok, we are within the radius, add us to the incoming list
                ForceLightningDamage(ctx, self_, Some(te_id), dir, ent_org);
            }
        } else {
            //trace-line
            let end: vec3_t = [
                (*scl).ps.origin[0] + 2048.0 * forward[0],
                (*scl).ps.origin[1] + 2048.0 * forward[1],
                (*scl).ps.origin[2] + 2048.0 * forward[2],
            ];

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*scl).ps.origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    self_number,
                    MASK_SHOT,
                ),
            );
            if tr.entityNum == (ENTITYNUM_NONE) as i16
                || tr.fraction == 1.0
                || tr.allsolid != 0
                || tr.startsolid != 0
            {
                return;
            }

            let te_id = EntityId(tr.entityNum as u32);
            ForceLightningDamage(ctx, self_, Some(te_id), forward, tr.endpos);
        }
    }
}

/// Raven `ForceDrain`.
///
/// Source: `oracle/codemp/game/w_force.c:2022-2056`
pub fn ForceDrain(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if (*cl).ps.fd.forcePower < 25 || WP_ForcePowerUsable(ctx, self_, FP_DRAIN) == 0 {
            return;
        }
        if (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] > level_time {
            //stops it while using it and also after using it, up to 3 second delay
            return;
        }

        (*cl).ps.forceHandExtend = HANDEXTEND_FORCE_HOLD as c_int;
        (*cl).ps.forceHandExtendTime = level_time + 20000;

        let snd = std::ffi::CString::new("sound/weapons/force/drain.wav").unwrap();
        G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(snd.as_ptr()));

        WP_ForcePowerStart(ctx, self_, FP_DRAIN, 500);
    }
}

/// Raven `ForceDrainDamage`.
///
/// Source: `oracle/codemp/game/w_force.c:2058-2182`
pub fn ForceDrainDamage(
    ctx: &mut GameContext,
    self_: EntityId,
    traceEnt: Option<EntityId>,
    dir: vec3_t,
    impactPoint: vec3_t,
) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let scl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        (*scl).dangerTime = level_time;
        (*scl).ps.eFlags &= !EF_INVULNERABLE;
        (*scl).invulnerableTimer = 0;

        // Raven: `if (traceEnt && traceEnt->takedamage)` — the rest of the body is
        // the last statement, so a non-taking / null target returns here.
        let te = match traceEnt {
            Some(t) if ctx.entity(t).takedamage != 0 => t,
            _ => return,
        };
        let tcl = ctx.entity(te).client;
        if !tcl.is_null()
            && (OnSameTeam(ctx, Some(self_), Some(te)) == 0
                || ctx.world.cvars.g_friendlyFire.integer != 0)
            && (*scl).ps.fd.forceDrainTime < (level_time) as f32
            && (*tcl).ps.fd.forcePower != 0
        {
            //an enemy or object
            if ctx.entity(te).client.is_null() && ctx.entity(te).s.eType == ET_NPC as c_int {
                //g2animent
                if ctx.entity(te).s.genericenemyindex < level_time {
                    ctx.entity_mut(te).s.genericenemyindex = level_time + 2000;
                }
            }
            if ForcePowerUsableOn(ctx, Some(self_), Some(te), FP_DRAIN) != 0 {
                let mut modPowerLevel = -1;
                let mut dmg = 0; //ctx.world.bg_state.rng.Q_irand( 1, 3 );
                if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_1 {
                    dmg = 2; //because it's one-shot
                } else if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_2 {
                    dmg = 3;
                } else if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] == FORCE_LEVEL_3 {
                    dmg = 4;
                }

                if !tcl.is_null() {
                    modPowerLevel = WP_AbsorbConversion(
                        ctx,
                        te,
                        (*tcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                        Some(self_),
                        FP_DRAIN,
                        (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize],
                        1,
                    );
                }

                if modPowerLevel != -1 {
                    if modPowerLevel == 0 {
                        dmg = 0;
                    } else if modPowerLevel == 1 {
                        dmg = 1;
                    } else if modPowerLevel == 2 {
                        dmg = 2;
                    }
                }
                //G_Damage( traceEnt, self, self, dir, impactPoint, dmg, 0, MOD_FORCE_DARK );

                if dmg != 0 {
                    (*tcl).ps.fd.forcePower -= dmg;
                }
                if (*tcl).ps.fd.forcePower < 0 {
                    (*tcl).ps.fd.forcePower = 0;
                }

                if (*scl).ps.stats[STAT_HEALTH as usize] < (*scl).ps.stats[STAT_MAX_HEALTH as usize]
                    && ctx.entity(self_).health > 0
                    && (*scl).ps.stats[STAT_HEALTH as usize] > 0
                {
                    ctx.entity_mut(self_).health += dmg;
                    if ctx.entity(self_).health > (*scl).ps.stats[STAT_MAX_HEALTH as usize] {
                        ctx.entity_mut(self_).health = (*scl).ps.stats[STAT_MAX_HEALTH as usize];
                    }
                    (*scl).ps.stats[STAT_HEALTH as usize] = ctx.entity(self_).health;
                }

                //don't let the client being drained get force power back right away
                (*tcl).ps.fd.forcePowerRegenDebounceTime = level_time + 800;

                if (*tcl).forcePowerSoundDebounce < level_time {
                    let tent_id = G_TempEntity(ctx, impactPoint, EV_FORCE_DRAINED as c_int);
                    let dir_byte = DirToByte(dir);
                    let te_number = ctx.entity(te).s.number;
                    let e = ctx.world.entity_mut(tent_id);
                    e.s.eventParm = dir_byte;
                    e.s.owner = te_number;

                    (*tcl).forcePowerSoundDebounce = level_time + 400;
                }
            }
        }
    }
}

/// Raven `ForceShootDrain`.
///
/// Source: `oracle/codemp/game/w_force.c:2184-2315`
pub fn ForceShootDrain(ctx: &mut GameContext, self_: EntityId) -> c_int {
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); `self_`'s number is
        // invariant here — hoist as a local.
        let scl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let level_time = ctx.world.level.time;
        let mut gotOneOrMore = 0;

        if ctx.entity(self_).health <= 0 {
            return 0;
        }
        let mut forward: vec3_t = [0.0; 3];
        AngleVectors((*scl).ps.viewangles, Some(&mut forward), None, None);
        VectorNormalize(&mut forward);

        let mut tr: trace_t = core::mem::zeroed();

        if (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_2 {
            //arc
            let radius: f32 = MAX_DRAIN_DISTANCE as f32;
            let center: vec3_t = (*scl).ps.origin;
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            for i in 0..3 {
                mins[i] = center[i] - radius;
                maxs[i] = center[i] + radius;
            }
            let mut iEntityList = [0i32; MAX_GENTITIES as usize];
            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    iEntityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let te_id = EntityId(iEntityList[e as usize] as u32);

                if te_id == self_ {
                    continue;
                }
                if ctx.entity(te_id).inuse == 0 {
                    continue;
                }
                if ctx.entity(te_id).takedamage == 0 {
                    continue;
                }
                if ctx.entity(te_id).health <= 0 {
                    //no torturing corpses
                    continue;
                }
                // FLAG: gclient_t deref stays raw (target may be an NPC pool client,
                // recipe 2b).
                let tcl = ctx.entity(te_id).client;
                if tcl.is_null() {
                    continue;
                }
                if (*tcl).ps.fd.forcePower == 0 {
                    continue;
                }
                if OnSameTeam(ctx, Some(self_), Some(te_id)) != 0
                    && ctx.world.cvars.g_friendlyFire.integer == 0
                {
                    continue;
                }

                // `traceEnt`'s box/model fields are invariant until the terminal
                // `ForceDrainDamage` call — hoist the reads as Copy locals.
                let te_absmin = ctx.entity(te_id).r.absmin;
                let te_absmax = ctx.entity(te_id).r.absmax;
                let te_bmodel = ctx.entity(te_id).r.bmodel;
                let te_number = ctx.entity(te_id).s.number;

                // find the distance from the edge of the bounding box
                let mut v: vec3_t = [0.0; 3];
                for i in 0..3 {
                    if center[i] < te_absmin[i] {
                        v[i] = te_absmin[i] - center[i];
                    } else if center[i] > te_absmax[i] {
                        v[i] = center[i] - te_absmax[i];
                    } else {
                        v[i] = 0.0;
                    }
                }

                let size: vec3_t = [
                    te_absmax[0] - te_absmin[0],
                    te_absmax[1] - te_absmin[1],
                    te_absmax[2] - te_absmin[2],
                ];
                let ent_org: vec3_t = [
                    te_absmin[0] + 0.5 * size[0],
                    te_absmin[1] + 0.5 * size[1],
                    te_absmin[2] + 0.5 * size[2],
                ];

                let mut dir: vec3_t = [
                    ent_org[0] - center[0],
                    ent_org[1] - center[1],
                    ent_org[2] - center[2],
                ];
                VectorNormalize(&mut dir);
                let dot = dir[0] * forward[0] + dir[1] * forward[1] + dir[2] * forward[2];
                if dot < 0.5 {
                    continue;
                }

                let dist = VectorLength(v);
                if dist >= radius {
                    continue;
                }

                if te_bmodel == 0
                    && trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &ent_org as *const vec3_t,
                            &(*scl).ps.origin as *const vec3_t,
                        ),
                    ) == 0
                {
                    continue;
                }

                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &(*scl).ps.origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &ent_org as *const vec3_t,
                        self_number,
                        MASK_SHOT,
                    ),
                );
                if tr.fraction < 1.0 && tr.entityNum != (te_number) as i16 {
                    continue;
                }

                ForceDrainDamage(ctx, self_, Some(te_id), dir, ent_org);
                gotOneOrMore = 1;
            }
        } else {
            //trace-line
            let end: vec3_t = [
                (*scl).ps.origin[0] + 2048.0 * forward[0],
                (*scl).ps.origin[1] + 2048.0 * forward[1],
                (*scl).ps.origin[2] + 2048.0 * forward[2],
            ];

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*scl).ps.origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &end as *const vec3_t,
                    self_number,
                    MASK_SHOT,
                ),
            );
            if tr.entityNum == (ENTITYNUM_NONE) as i16
                || tr.fraction == 1.0
                || tr.allsolid != 0
                || tr.startsolid != 0
                || ctx.world.g_entities[tr.entityNum as usize].client.is_null()
                || ctx.world.g_entities[tr.entityNum as usize].inuse == 0
            {
                return 0;
            }

            let te_id = EntityId(tr.entityNum as u32);
            ForceDrainDamage(ctx, self_, Some(te_id), forward, tr.endpos);
            gotOneOrMore = 1;
        }

        (*scl).ps.activeForcePass = (*scl).ps.fd.forcePowerLevel[FP_DRAIN as usize] + FORCE_LEVEL_3;

        //used to be 1, but this did, too, anger the God of Balance.
        BG_ForcePowerDrain(&mut (*scl).ps, FP_DRAIN, 5);

        (*scl).ps.fd.forcePowerRegenDebounceTime = level_time + 500;

        gotOneOrMore
    }
}

/// Raven `ForceJumpCharge`.
///
/// Source: `oracle/codemp/game/w_force.c:2317-2375`
pub fn ForceJumpCharge(ctx: &mut GameContext, self_: EntityId, ucmd: *mut usercmd_t) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let forceJumpChargeInterval: f32 =
            forceJumpStrength[0] / (FORCE_JUMP_CHARGE_TIME as f32 / FRAMETIME as f32);

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.fd.forceJumpCharge == 0.0 && (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
            return;
        }

        if (*cl).ps.fd.forcePower
            < forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                [FP_LEVITATION as usize]
        {
            G_MuteSound(
                ctx,
                (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                CHAN_VOICE,
            );
            return;
        }

        if (*cl).ps.fd.forceJumpCharge == 0.0 {
            (*cl).ps.fd.forceJumpAddTime = 0;
        }

        if (*cl).ps.fd.forceJumpAddTime >= level_time {
            return;
        }

        //need to play sound
        if (*cl).ps.fd.forceJumpCharge == 0.0 {
            let s = cstr("sound/weapons/force/jumpbuild.wav");
            G_Sound(
                ctx,
                Some(self_),
                TRACK_CHANNEL_1 as c_int,
                G_SoundIndex(s.as_ptr()),
            );
        }

        //Increment
        if (*cl).ps.fd.forceJumpAddTime < level_time {
            (*cl).ps.fd.forceJumpCharge += forceJumpChargeInterval * 50.0;
            (*cl).ps.fd.forceJumpAddTime = level_time + 500;
        }

        //clamp to max strength for current level
        if (*cl).ps.fd.forceJumpCharge
            > forceJumpStrength[(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
        {
            (*cl).ps.fd.forceJumpCharge =
                forceJumpStrength[(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize];
            G_MuteSound(
                ctx,
                (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                CHAN_VOICE,
            );
        }

        //clamp to max available force power
        if (*cl).ps.fd.forceJumpCharge
            / forceJumpChargeInterval
            / (FORCE_JUMP_CHARGE_TIME as f32 / FRAMETIME as f32)
            * (forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                [FP_LEVITATION as usize] as f32)
            > (*cl).ps.fd.forcePower as f32
        {
            //can't use more than you have
            G_MuteSound(
                ctx,
                (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                CHAN_VOICE,
            );
            (*cl).ps.fd.forceJumpCharge = (*cl).ps.fd.forcePower as f32 * forceJumpChargeInterval
                / (FORCE_JUMP_CHARGE_TIME as f32 / FRAMETIME as f32);
        }
    }
}

/// Raven `WP_GetVelocityForForceJump`.
///
/// Source: `oracle/codemp/game/w_force.c:2377-2460`
// `jumpVel` is a written-through out-param (`VectorMA(... jumpVel)`); the
// out-param reshape turns the by-value `vec3_t` into `&mut vec3_t`.
pub fn WP_GetVelocityForForceJump(
    ctx: &mut GameContext,
    self_: EntityId,
    jumpVel: &mut vec3_t,
    ucmd: *mut usercmd_t,
) -> c_int {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        // `ucmd` is a raw `*mut usercmd_t` seam param — derefs stay raw.
        let cl = ctx.entity(self_).client;

        let mut pushFwd: f32 = 0.0;
        let mut pushRt: f32 = 0.0;
        let mut view: vec3_t = (*cl).ps.viewangles;
        view[0] = 0.0;
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(view, Some(&mut forward), Some(&mut right), None);

        if (*ucmd).forwardmove != 0 && (*ucmd).rightmove != 0 {
            if (*ucmd).forwardmove > 0 {
                pushFwd = 50.0;
            } else {
                pushFwd = -50.0;
            }
            if (*ucmd).rightmove > 0 {
                pushRt = 50.0;
            } else {
                pushRt = -50.0;
            }
        } else if (*ucmd).forwardmove != 0 || (*ucmd).rightmove != 0 {
            if (*ucmd).forwardmove > 0 {
                pushFwd = 100.0;
            } else if (*ucmd).forwardmove < 0 {
                pushFwd = -100.0;
            } else if (*ucmd).rightmove > 0 {
                pushRt = 100.0;
            } else if (*ucmd).rightmove < 0 {
                pushRt = -100.0;
            }
        }

        G_MuteSound(
            ctx,
            (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
            CHAN_VOICE,
        );

        G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);

        if (*cl).ps.fd.forceJumpCharge < JUMP_VELOCITY + 40.0 {
            //give him at least a tiny boost from just a tap
            (*cl).ps.fd.forceJumpCharge = JUMP_VELOCITY + 400.0;
        }

        if (*cl).ps.velocity[2] < -30.0 {
            //so that we can get a good boost when force jumping in a fall
            (*cl).ps.velocity[2] = -30.0;
        }

        for i in 0..3 {
            jumpVel[i] = (*cl).ps.velocity[i] + pushFwd * forward[i];
        }
        for i in 0..3 {
            jumpVel[i] = (*cl).ps.velocity[i] + pushRt * right[i];
        }
        jumpVel[2] += (*cl).ps.fd.forceJumpCharge;
        if pushFwd > 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_FORWARD
        } else if pushFwd < 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_BACKWARD
        } else if pushRt > 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_RIGHT
        } else if pushRt < 0.0 && (*cl).ps.fd.forceJumpCharge > 200.0 {
            FJ_LEFT
        } else {
            FJ_UP
        }
    }
}

/// Raven `ForceJump`.
///
/// Source: `oracle/codemp/game/w_force.c:2462-2500`
pub fn ForceJump(ctx: &mut GameContext, self_: EntityId, ucmd: *mut usercmd_t) {
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        if (*cl).ps.fd.forcePowerDuration[FP_LEVITATION as usize] > level_time {
            return;
        }
        if WP_ForcePowerUsable(ctx, self_, FP_LEVITATION) == 0 {
            return;
        }
        if ctx.entity(self_).s.groundEntityNum == ENTITYNUM_NONE {
            return;
        }
        if ctx.entity(self_).health <= 0 {
            return;
        }

        (*cl).fjDidJump = qtrue;

        let forceJumpChargeInterval: f32 = forceJumpStrength
            [(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
            / (FORCE_JUMP_CHARGE_TIME as f32 / FRAMETIME as f32);

        let mut jumpVel: vec3_t = [0.0; 3];
        WP_GetVelocityForForceJump(ctx, self_, &mut jumpVel, ucmd);

        //FIXME: sound effect
        (*cl).ps.fd.forceJumpZStart = (*cl).ps.origin[2]; //remember this for when we land
        (*cl).ps.velocity = jumpVel;
        //wasn't allowing them to attack when jumping, but that was annoying
        //self->client->ps.weaponTime = self->client->ps.torsoAnimTimer;

        WP_ForcePowerStart(
            ctx,
            self_,
            FP_LEVITATION,
            ((*cl).ps.fd.forceJumpCharge
                / forceJumpChargeInterval
                / (FORCE_JUMP_CHARGE_TIME as f32 / FRAMETIME as f32)
                * (forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] as usize]
                    [FP_LEVITATION as usize] as f32)) as c_int,
        );
        //self->client->ps.fd.forcePowerDuration[FP_LEVITATION] = level.time + self->client->ps.weaponTime;
        (*cl).ps.fd.forceJumpCharge = 0.0;
        (*cl).ps.forceJumpFlip = qtrue;
    }
}

/// Raven `WP_AddAsMindtricked` — pack `entNum` into a forcedata mindtrick mask.
///
/// Source: `oracle/codemp/game/w_force.c:2502-2525`
pub fn WP_AddAsMindtricked(fd: *mut forcedata_t, entNum: c_int) {
    unsafe {
        if fd.is_null() {
            return;
        }

        if entNum > 47 {
            (*fd).forceMindtrickTargetIndex4 |= 1 << (entNum - 48);
        } else if entNum > 31 {
            (*fd).forceMindtrickTargetIndex3 |= 1 << (entNum - 32);
        } else if entNum > 15 {
            (*fd).forceMindtrickTargetIndex2 |= 1 << (entNum - 16);
        } else {
            (*fd).forceMindtrickTargetIndex |= 1 << entNum;
        }
    }
}

/// Raven `ForceTelepathyCheckDirectNPCTarget`.
///
/// Source: `oracle/codemp/game/w_force.c:2527-2721`
pub fn ForceTelepathyCheckDirectNPCTarget(
    ctx: &mut GameContext,
    self_: EntityId,
    tr: *mut trace_t,
    tookPower: *mut qboolean,
) -> qboolean {
    unsafe {
        // FLAG: gclient_t / gNPC_t derefs stay raw (recipe 2b/2c); read pointer
        // values via the safe entity borrow. `tr`/`tookPower` are raw seam params.
        let cl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let self_npc = ctx.entity(self_).NPC;
        let mut targetLive = qfalse;
        let mut mindTrickDone = qfalse;
        let radius: f32 = (MAX_TRICK_DISTANCE) as f32;

        //Check for a direct usage on NPCs first
        let mut tfrom: vec3_t = (*cl).ps.origin;
        tfrom[2] += (*cl).ps.viewheight as f32;
        let mut fwd: vec3_t = [0.0; 3];
        AngleVectors((*cl).ps.viewangles, Some(&mut fwd), None, None);
        let tto: vec3_t = [
            tfrom[0] + fwd[0] * radius / 2.0,
            tfrom[1] + fwd[1] * radius / 2.0,
            tfrom[2] + fwd[2] * radius / 2.0,
        ];

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                tr,
                &tfrom as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &tto as *const vec3_t,
                self_number,
                MASK_PLAYERSOLID,
            ),
        );

        if (*tr).entityNum == (ENTITYNUM_NONE) as i16
            || (*tr).fraction == 1.0
            || (*tr).allsolid != 0
            || (*tr).startsolid != 0
        {
            return qfalse;
        }

        let te_id = EntityId((*tr).entityNum as u32);
        // FLAG: gclient_t (`tcl`) / gNPC_t (`npc`) stay raw (recipe 2b/2c).
        let tcl = ctx.entity(te_id).client;
        let npc = ctx.entity(te_id).NPC;

        if !npc.is_null() && (*npc).scriptFlags & SCF_NO_FORCE != 0 {
            return qfalse;
        }

        if !tcl.is_null() {
            match (*tcl).NPC_class {
                CLASS_GALAKMECH | CLASS_ATST | CLASS_PROBE | CLASS_GONK | CLASS_R2D2
                | CLASS_R5D2 | CLASS_MARK1 | CLASS_MARK2 | CLASS_MOUSE | CLASS_SEEKER
                | CLASS_REMOTE | CLASS_PROTOCOL | CLASS_BOBAFETT | CLASS_RANCOR => {}
                _ => {
                    targetLive = qtrue;
                }
            }
        }

        if ctx.entity(te_id).s.number < MAX_CLIENTS as c_int {
            //a regular client
            return qfalse;
        }

        if targetLive != 0 && !npc.is_null() {
            //hit an organic non-player
            let mut over_ride: c_int = 0;

            if G_ActivateBehavior(ctx, Some(te_id), (BSET_MINDTRICK) as i32) != 0 {
                //activated a script on him
                //FIXME: do the visual sparkles effect on their heads, still?
                WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
            } else if (self_npc != std::ptr::null_mut() && (*tcl).playerTeam != (*cl).playerTeam)
                || (self_npc == std::ptr::null_mut()
                    && (*tcl).playerTeam != (*cl).sess.sessionTeam as c_int)
            {
                //an enemy
                if (*npc).scriptFlags & SCF_NO_MIND_TRICK != 0 {
                    // no-op, matches the empty Raven arm
                } else if ctx.entity(te_id).s.weapon != WP_SABER as c_int
                    && (*tcl).NPC_class != CLASS_REBORN
                {
                    //haha!  Jedi aren't easily confused!
                    if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] > FORCE_LEVEL_2 as c_int {
                        //turn them to our side
                        //if mind trick 3 and aiming at an enemy need more force power
                        if ctx.entity(te_id).s.weapon != WP_NONE as c_int {
                            //don't charm people who aren't capable of fighting... like ugnaughts and droids
                            let (newPlayerTeam, newEnemyTeam);

                            if ctx.entity(te_id).enemy.is_some() {
                                G_ClearEnemy(ctx, te_id);
                            }
                            if !npc.is_null() {
                                //traceEnt->NPC->tempBehavior = BS_FOLLOW_LEADER;
                                (*tcl).leader = Some(self_);
                            }
                            //FIXME: maybe pick an enemy right here?
                            if self_npc != std::ptr::null_mut() {
                                //NPC
                                newPlayerTeam = (*cl).playerTeam;
                                newEnemyTeam = (*cl).enemyTeam;
                            } else {
                                //client/bot
                                if (*cl).sess.sessionTeam == TEAM_BLUE {
                                    //rebel
                                    newPlayerTeam = NPCTEAM_PLAYER;
                                    newEnemyTeam = NPCTEAM_ENEMY;
                                } else if (*cl).sess.sessionTeam == TEAM_RED {
                                    //imperial
                                    newPlayerTeam = NPCTEAM_ENEMY;
                                    newEnemyTeam = NPCTEAM_PLAYER;
                                } else {
                                    //neutral - wan't attack anyone
                                    newPlayerTeam = NPCTEAM_NEUTRAL;
                                    newEnemyTeam = NPCTEAM_NEUTRAL;
                                }
                            }
                            //store these for retrieval later
                            let te_teamowner = ctx.entity(te_id).s.teamowner;
                            ctx.entity_mut(te_id).genericValue1 = (*tcl).playerTeam;
                            ctx.entity_mut(te_id).genericValue2 = (*tcl).enemyTeam;
                            ctx.entity_mut(te_id).genericValue3 = te_teamowner;
                            //set the new values
                            (*tcl).playerTeam = newPlayerTeam;
                            (*tcl).enemyTeam = newEnemyTeam;
                            ctx.entity_mut(te_id).s.teamowner = newPlayerTeam;
                            //FIXME: need a *charmed* timer on this...?  Or do TEAM_PLAYERS assume that "confusion" means they should switch to team_enemy when done?
                            (*npc).charmedTime = ctx.world.level.time
                                + mindTrickTime
                                    [(*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] as usize];
                        }
                    } else {
                        //just confuse them
                        //somehow confuse them?  Set don't fire to true for a while?  Drop their aggression?  Maybe just take their enemy away and don't let them pick one up for a while unless shot?
                        (*npc).confusionTime = ctx.world.level.time
                            + mindTrickTime
                                [(*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] as usize]; //confused for about 10 seconds
                        crate::NPC_sounds::NPC_PlayConfusionSound(ctx, te_id);
                        if ctx.entity(te_id).enemy.is_some() {
                            G_ClearEnemy(ctx, te_id);
                        }
                    }
                } else {
                    crate::NPC_AI_Jedi::NPC_Jedi_PlayConfusionSound(ctx, te_id);
                }
                WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, over_ride);
            } else if (*tcl).playerTeam == (*cl).playerTeam {
                //an ally
                //maybe just have him look at you?  Respond?  Take your enemy?
                if (*tcl).ps.pm_type < PM_DEAD as c_int
                    && !npc.is_null()
                    && (*npc).scriptFlags & SCF_NO_RESPONSE == 0
                {
                    crate::NPC_reactions::NPC_UseResponse(ctx, te_id, Some(self_), qfalse);
                    WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 1);
                }
            } //NOTE: no effect on TEAM_NEUTRAL?
            let mut eyeDir: vec3_t = [0.0; 3];
            AngleVectors((*tcl).renderInfo.eyeAngles, Some(&mut eyeDir), None, None);
            VectorNormalize(&mut eyeDir);
            G_PlayEffectID(
                G_EffectIndex(cstr("force/force_touch").as_ptr()),
                (*tcl).renderInfo.eyePoint,
                eyeDir,
            );

            //make sure this plays and that you cannot press fire for about 1 second after this
            //FIXME: BOTH_FORCEMINDTRICK or BOTH_FORCEDISTRACT
            //NPC_SetAnim( self, SETANIM_TORSO, BOTH_MINDTRICK1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_RESTART|SETANIM_FLAG_HOLD );
            //FIXME: build-up or delay this until in proper part of anim
            mindTrickDone = qtrue;
        } else {
            if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] > FORCE_LEVEL_1 as c_int
                && (*tr).fraction * 2048.0 > 64.0
            {
                //don't create a diversion less than 64 from you of if at power level 1
                //use distraction anim instead
                G_PlayEffectID(
                    G_EffectIndex(cstr("force/force_touch").as_ptr()),
                    (*tr).endpos,
                    (*tr).plane.normal,
                );
                //FIXME: these events don't seem to always be picked up...?
                AddSoundEvent(ctx, Some(self_), (*tr).endpos, 512.0, AEL_SUSPICIOUS, qtrue); //, qtrue );
                AddSightEvent(ctx, Some(self_), (*tr).endpos, 512.0, AEL_SUSPICIOUS, 50.0);
                WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
                *tookPower = qtrue;
            }
            //NPC_SetAnim( self, SETANIM_TORSO, BOTH_MINDTRICK2, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_RESTART|SETANIM_FLAG_HOLD );
        }
        //self->client->ps.saberMove = self->client->ps.saberBounceMove = LS_READY;//don't finish whatever saber anim you may have been in
        (*cl).ps.saberBlocked = BLOCKED_NONE as c_int;
        (*cl).ps.weaponTime = 1000;
        qtrue
    }
}

/// Raven `ForceTelepathy`.
///
/// Source: `oracle/codemp/game/w_force.c:2723-2893`
pub fn ForceTelepathy(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        let mut tr: trace_t = core::mem::zeroed();
        let mut visionArc: f32 = 0.0;
        let mut radius: f32 = MAX_TRICK_DISTANCE as f32;
        let mut tookPower: qboolean = qfalse;

        if ctx.entity(self_).health <= 0 {
            return;
        }

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if (*cl).ps.powerups[PW_REDFLAG as usize] != 0
            || (*cl).ps.powerups[PW_BLUEFLAG as usize] != 0
        {
            //can't mindtrick while carrying the flag
            return;
        }

        if (*cl).ps.forceAllowDeactivateTime < level_time
            && (*cl).ps.fd.forcePowersActive & (1 << FP_TELEPATHY) != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
            return;
        }

        if WP_ForcePowerUsable(ctx, self_, FP_TELEPATHY) == 0 {
            return;
        }

        if ForceTelepathyCheckDirectNPCTarget(ctx, self_, &mut tr, &mut tookPower) != 0 {
            //hit an NPC directly
            (*cl).ps.forceAllowDeactivateTime = level_time + 1500;
            let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
            G_Sound(ctx, Some(self_), CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
            (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
            (*cl).ps.forceHandExtendTime = level_time + 1000;
            return;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_2 {
            visionArc = 180.0;
        } else if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_3 {
            visionArc = 360.0;
            radius = MAX_TRICK_DISTANCE as f32 * 2.0;
        }

        let fwdangles: vec3_t = (*cl).ps.viewangles;
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(fwdangles, Some(&mut forward), Some(&mut right), None);
        let center: vec3_t = (*cl).ps.origin;

        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        for i in 0..3 {
            mins[i] = center[i] - radius;
            maxs[i] = center[i] + radius;
        }

        if (*cl).ps.fd.forcePowerLevel[FP_TELEPATHY as usize] == FORCE_LEVEL_1 {
            let ent_id2 = EntityId(tr.entityNum as u32);
            // FLAG: gclient_t deref stays raw (recipe 2b).
            let e_cl = ctx.entity(ent_id2).client;
            if tr.fraction != 1.0
                && tr.entityNum != (ENTITYNUM_NONE) as i16
                && ctx.entity(ent_id2).inuse != 0
                && !e_cl.is_null()
                && (*e_cl).pers.connected != 0
                && (*e_cl).sess.sessionTeam != TEAM_SPECTATOR
            {
                WP_AddAsMindtricked(&mut (*cl).ps.fd, (tr.entityNum) as i32);
                if tookPower == 0 {
                    WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
                }

                let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
                G_Sound(ctx, Some(self_), CHAN_AUTO, G_SoundIndex(snd.as_ptr()));

                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                (*cl).ps.forceHandExtendTime = level_time + 1000;
            }
        } else {
            //level 2 & 3
            let mut entityList = [0i32; MAX_GENTITIES as usize];
            let mut gotatleastone: qboolean = qfalse;

            let numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    entityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            for e in 0..numListedEntities {
                let ent_id2 = EntityId(entityList[e as usize] as u32);

                {
                    // FLAG: gclient_t deref stays raw (recipe 2b).
                    let e_cl = ctx.entity(ent_id2).client;
                    let mut thispush_org: vec3_t;
                    if !e_cl.is_null() {
                        thispush_org = (*e_cl).ps.origin;
                    } else {
                        thispush_org = ctx.entity(ent_id2).s.pos.trBase;
                    }
                    let mut tto: vec3_t = (*cl).ps.origin;
                    tto[2] += (*cl).ps.viewheight as f32;
                    let mut a: vec3_t = [
                        thispush_org[0] - tto[0],
                        thispush_org[1] - tto[1],
                        thispush_org[2] - tto[2],
                    ];
                    let a_in = a;
                    vectoangles(a_in, &mut a);

                    if e_cl.is_null() {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if InFieldOfVision((*cl).ps.viewangles, visionArc, a) == 0 {
                        //only bother with arc rules if the victim is a client
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if ForcePowerUsableOn(ctx, Some(self_), Some(ent_id2), FP_TELEPATHY) == 0
                    {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    } else if OnSameTeam(ctx, Some(self_), Some(ent_id2)) != 0 {
                        entityList[e as usize] = ENTITYNUM_NONE;
                    }
                }
                let ent_id2 = EntityId(entityList[e as usize] as u32);
                let e_cl = ctx.entity(ent_id2).client;
                if ent_id2 != self_ && !e_cl.is_null() {
                    gotatleastone = qtrue;
                    let en = ctx.entity(ent_id2).s.number;
                    WP_AddAsMindtricked(&mut (*cl).ps.fd, en);
                }
            }

            if gotatleastone != 0 {
                (*cl).ps.forceAllowDeactivateTime = level_time + 1500;

                if tookPower == 0 {
                    WP_ForcePowerStart(ctx, self_, FP_TELEPATHY, 0);
                }

                let snd = std::ffi::CString::new("sound/weapons/force/distract.wav").unwrap();
                G_Sound(ctx, Some(self_), CHAN_AUTO, G_SoundIndex(snd.as_ptr()));

                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                (*cl).ps.forceHandExtendTime = level_time + 1000;
            }
        }
    }
}

/// Raven `GEntity_UseFunc`.
///
/// Source: `oracle/codemp/game/w_force.c:2895-2898`
pub fn GEntity_UseFunc(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    GlobalUse(ctx, Some(self_), other, activator);
}

/// Raven `CanCounterThrow`.
///
/// Source: `oracle/codemp/game/w_force.c:2900-2968`
pub fn CanCounterThrow(
    ctx: &mut GameContext,
    self_: EntityId,
    thrower: Option<EntityId>,
    pull: qboolean,
) -> qboolean {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); `thrower_cl` is null when the
        // handle is `None` OR its `.client` is null — exactly Raven's paired guard.
        let cl = ctx.entity(self_).client;
        let thrower_cl: *mut gclient_t = match thrower {
            Some(t) => ctx.entity(t).client,
            None => core::ptr::null_mut(),
        };
        let level_time = ctx.world.level.time;
        let powerUse: forcePowers_t;

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            return 0;
        }

        if (*cl).ps.weaponTime > 0 {
            return 0;
        }

        if ctx.entity(self_).health <= 0 {
            return 0;
        }

        if (*cl).ps.powerups[PW_DISINT_4 as usize] > level_time {
            return 0;
        }

        if (*cl).ps.weaponstate == WEAPON_CHARGING as c_int
            || (*cl).ps.weaponstate == WEAPON_CHARGING_ALT as c_int
        {
            //don't autodefend when charging a weapon
            return 0;
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE && pull != 0 && !thrower_cl.is_null() {
            //in siege, pull will affect people if they are not facing you, so they can't run away so much
            let tcl = thrower_cl;
            let mut d: vec3_t = [
                (*tcl).ps.origin[0] - (*cl).ps.origin[0],
                (*tcl).ps.origin[1] - (*cl).ps.origin[1],
                (*tcl).ps.origin[2] - (*cl).ps.origin[2],
            ];
            let d_in = d;
            vectoangles(d_in, &mut d);

            let a = AngleSubtract(d[YAW], (*cl).ps.viewangles[YAW]);

            if a > 60.0 || a < -60.0 {
                //if facing more than 60 degrees away they cannot defend
                return 0;
            }
        }

        if pull != 0 {
            powerUse = FP_PULL;
        } else {
            powerUse = FP_PUSH;
        }

        if WP_ForcePowerUsable(ctx, self_, powerUse) == 0 {
            return 0;
        }

        if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
            //you cannot counter a push/pull if you're in the air
            return 0;
        }

        1
    }
}

/// Raven `G_InGetUpAnim`.
///
/// Source: `oracle/codemp/game/w_force.c:2970-3023`
pub fn G_InGetUpAnim(ps: *mut playerState_t) -> qboolean {
    unsafe {
        let legs = (*ps).legsAnim;
        if legs == BOTH_GETUP1 as c_int
            || legs == BOTH_GETUP2 as c_int
            || legs == BOTH_GETUP3 as c_int
            || legs == BOTH_GETUP4 as c_int
            || legs == BOTH_GETUP5 as c_int
            || legs == BOTH_FORCE_GETUP_F1 as c_int
            || legs == BOTH_FORCE_GETUP_F2 as c_int
            || legs == BOTH_FORCE_GETUP_B1 as c_int
            || legs == BOTH_FORCE_GETUP_B2 as c_int
            || legs == BOTH_FORCE_GETUP_B3 as c_int
            || legs == BOTH_FORCE_GETUP_B4 as c_int
            || legs == BOTH_FORCE_GETUP_B5 as c_int
            || legs == BOTH_GETUP_BROLL_B as c_int
            || legs == BOTH_GETUP_BROLL_F as c_int
            || legs == BOTH_GETUP_BROLL_L as c_int
            || legs == BOTH_GETUP_BROLL_R as c_int
            || legs == BOTH_GETUP_FROLL_B as c_int
            || legs == BOTH_GETUP_FROLL_F as c_int
            || legs == BOTH_GETUP_FROLL_L as c_int
            || legs == BOTH_GETUP_FROLL_R as c_int
        {
            return qtrue;
        }

        let torso = (*ps).torsoAnim;
        if torso == BOTH_GETUP1 as c_int
            || torso == BOTH_GETUP2 as c_int
            || torso == BOTH_GETUP3 as c_int
            || torso == BOTH_GETUP4 as c_int
            || torso == BOTH_GETUP5 as c_int
            || torso == BOTH_FORCE_GETUP_F1 as c_int
            || torso == BOTH_FORCE_GETUP_F2 as c_int
            || torso == BOTH_FORCE_GETUP_B1 as c_int
            || torso == BOTH_FORCE_GETUP_B2 as c_int
            || torso == BOTH_FORCE_GETUP_B3 as c_int
            || torso == BOTH_FORCE_GETUP_B4 as c_int
            || torso == BOTH_FORCE_GETUP_B5 as c_int
            || torso == BOTH_GETUP_BROLL_B as c_int
            || torso == BOTH_GETUP_BROLL_F as c_int
            || torso == BOTH_GETUP_BROLL_L as c_int
            || torso == BOTH_GETUP_BROLL_R as c_int
            || torso == BOTH_GETUP_FROLL_B as c_int
            || torso == BOTH_GETUP_FROLL_F as c_int
            || torso == BOTH_GETUP_FROLL_L as c_int
            || torso == BOTH_GETUP_FROLL_R as c_int
        {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `G_LetGoOfWall`.
///
/// Source: `oracle/codemp/game/w_force.c:3025-3042`
pub fn G_LetGoOfWall(ctx: &mut GameContext, ent: Option<EntityId>) {
    // FLAG: gclient_t deref stays raw (`ent` can be an NPC pool client, recipe 2b);
    // read the pointer value via the safe entity borrow.
    let cl = match ent {
        Some(e) if !ctx.entity(e).client.is_null() => ctx.entity(e).client,
        _ => return,
    };
    unsafe {
        (*cl).ps.pm_flags &= !PMF_STUCK_TO_WALL;
        if BG_InReboundJump((*cl).ps.legsAnim) != 0 || BG_InReboundHold((*cl).ps.legsAnim) != 0 {
            (*cl).ps.legsTimer = 0;
        }
        if BG_InReboundJump((*cl).ps.torsoAnim) != 0 || BG_InReboundHold((*cl).ps.torsoAnim) != 0 {
            (*cl).ps.torsoTimer = 0;
        }
    }
}

/// Raven `ForceThrow`.
///
/// Source: `oracle/codemp/game/w_force.c:3054-3820`
pub fn ForceThrow(ctx: &mut GameContext, self_: EntityId, pull: qboolean) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b; `self_`/targets can be NPC
        // pool clients); read the pointer value via the safe entity borrow. Entity
        // fields are read through `ctx.world.entity(id)` at the point of use;
        // `push_list` holds `EntityId` handles rather than raw `gentity_t*`.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let mut entityList: [c_int; MAX_GENTITIES as usize] = [0; MAX_GENTITIES as usize];
        let mut push_list: [EntityId; MAX_GENTITIES as usize] =
            [EntityId(0); MAX_GENTITIES as usize];
        let mut numListedEntities: c_int;
        let radius: f32 = 1024.0; //since it's view-based now. //350;
        let powerLevel: c_int;
        let mut visionArc: f32 = 0.0;
        let mut pushPower: c_int;
        let mut fwdangles: vec3_t = [0.0; 3];
        let mut tr: trace_t = core::mem::zeroed();
        let mut ent_count: usize = 0;

        if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int
            && ((*cl).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN as c_int
                || G_InGetUpAnim(&mut (*cl).ps) == 0)
        {
            return;
        }

        if ctx.world.cvars.g_useWhileThrowing.integer == 0 && (*cl).ps.saberInFlight != 0 {
            return;
        }

        if (*cl).ps.weaponTime > 0 {
            return;
        }

        if ctx.entity(self_).health <= 0 {
            return;
        }
        if (*cl).ps.powerups[PW_DISINT_4 as usize] > level_time {
            return;
        }
        let powerUse: forcePowers_t = if pull != 0 { FP_PULL } else { FP_PUSH };

        if WP_ForcePowerUsable(ctx, self_, powerUse) == 0 {
            return;
        }

        if pull == 0 && (*cl).ps.saberLockTime > level_time && (*cl).ps.saberLockFrame != 0 {
            let s = cstr("sound/weapons/force/push.wav");
            G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(s.as_ptr()));
            (*cl).ps.powerups[PW_DISINT_4 as usize] = level_time + 1500;

            (*cl).ps.saberLockHits += (*cl).ps.fd.forcePowerLevel[FP_PUSH as usize] * 2;

            WP_ForcePowerStart(ctx, self_, FP_PUSH, 0);
            return;
        }

        WP_ForcePowerStart(ctx, self_, powerUse, 0);

        //make sure this plays and that you cannot press fire for about 1 second after this
        if pull != 0 {
            let s = cstr("sound/weapons/force/pull.wav");
            G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(s.as_ptr()));
            if (*cl).ps.forceHandExtend == HANDEXTEND_NONE as c_int {
                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPULL as c_int;
                if ctx.world.cvars.g_gametype.integer == GT_SIEGE as c_int
                    && (*cl).ps.weapon == WP_SABER as c_int
                {
                    //hold less so can attack right after a pull
                    (*cl).ps.forceHandExtendTime = level_time + 200;
                } else {
                    (*cl).ps.forceHandExtendTime = level_time + 400;
                }
            }
            (*cl).ps.powerups[PW_DISINT_4 as usize] = (*cl).ps.forceHandExtendTime + 200;
            (*cl).ps.powerups[PW_PULL as usize] = (*cl).ps.powerups[PW_DISINT_4 as usize];
        } else {
            let s = cstr("sound/weapons/force/push.wav");
            G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(s.as_ptr()));
            if (*cl).ps.forceHandExtend == HANDEXTEND_NONE as c_int {
                (*cl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                (*cl).ps.forceHandExtendTime = level_time + 1000;
            } else if (*cl).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                && G_InGetUpAnim(&mut (*cl).ps) != 0
            {
                if (*cl).ps.forceDodgeAnim > 4 {
                    (*cl).ps.forceDodgeAnim -= 8;
                }
                (*cl).ps.forceDodgeAnim += 8; //special case, play push on upper torso, but keep playing current knockdown anim on legs
            }
            (*cl).ps.powerups[PW_DISINT_4 as usize] = level_time + 1100;
            (*cl).ps.powerups[PW_PULL as usize] = 0;
        }

        fwdangles = (*cl).ps.viewangles;
        let mut forward: vec3_t = [0.0; 3];
        let mut right: vec3_t = [0.0; 3];
        AngleVectors(fwdangles, Some(&mut forward), Some(&mut right), None);
        let center: vec3_t = (*cl).ps.origin;

        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        for i in 0..3 {
            mins[i] = center[i] - radius;
            maxs[i] = center[i] + radius;
        }

        if pull != 0 {
            powerLevel = (*cl).ps.fd.forcePowerLevel[FP_PULL as usize];
            pushPower = 256 * (*cl).ps.fd.forcePowerLevel[FP_PULL as usize];
        } else {
            powerLevel = (*cl).ps.fd.forcePowerLevel[FP_PUSH as usize];
            pushPower = 256 * (*cl).ps.fd.forcePowerLevel[FP_PUSH as usize];
        }

        if powerLevel == 0 {
            //Shouldn't have made it here..
            return;
        }

        if powerLevel == FORCE_LEVEL_2 as c_int {
            visionArc = 60.0;
        } else if powerLevel == FORCE_LEVEL_3 as c_int {
            visionArc = 180.0;
        }

        if powerLevel == FORCE_LEVEL_1 as c_int {
            //can only push/pull targeted things at level 1
            let mut tfrom: vec3_t = (*cl).ps.origin;
            tfrom[2] += (*cl).ps.viewheight as f32;
            let mut fwd: vec3_t = [0.0; 3];
            AngleVectors((*cl).ps.viewangles, Some(&mut fwd), None, None);
            let tto: vec3_t = [
                tfrom[0] + fwd[0] * radius / 2.0,
                tfrom[1] + fwd[1] * radius / 2.0,
                tfrom[2] + fwd[2] * radius / 2.0,
            ];

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &tfrom as *const vec3_t,
                    std::ptr::null(),
                    std::ptr::null(),
                    &tto as *const vec3_t,
                    ctx.entity(self_).s.number,
                    MASK_PLAYERSOLID,
                ),
            );

            if tr.fraction != 1.0 && tr.entityNum != (ENTITYNUM_NONE) as i16 {
                let hit_id = EntityId(tr.entityNum as u32);
                let hcl = ctx.world.entity(hit_id).client;
                if hcl.is_null() && ctx.world.entity(hit_id).s.eType == ET_NPC as c_int {
                    //g2animent
                    if ctx.world.entity(hit_id).s.genericenemyindex < level_time {
                        ctx.world.entity_mut(hit_id).s.genericenemyindex = level_time + 2000;
                    }
                }

                numListedEntities = 0;
                entityList[numListedEntities as usize] = (tr.entityNum) as i32;

                if pull != 0 {
                    if ForcePowerUsableOn(ctx, Some(self_), Some(hit_id), FP_PULL) == 0 {
                        return;
                    }
                } else {
                    if ForcePowerUsableOn(ctx, Some(self_), Some(hit_id), FP_PUSH) == 0 {
                        return;
                    }
                }
                numListedEntities += 1;
            } else {
                //didn't get anything, so just
                return;
            }
        } else {
            numListedEntities = trap::EntitiesInBox(
                ctx.engine,
                GEntitiesInBoxArgs::new(
                    &mins as *const vec3_t,
                    &maxs as *const vec3_t,
                    entityList.as_mut_ptr(),
                    MAX_GENTITIES as c_int,
                ),
            );

            let mut e: usize = 0;
            while (e as c_int) < numListedEntities {
                let ent_id = EntityId(entityList[e] as u32);
                let ecl = ctx.world.entity(ent_id).client;

                if ecl.is_null() && ctx.world.entity(ent_id).s.eType == ET_NPC as c_int {
                    //g2animent
                    if ctx.world.entity(ent_id).s.genericenemyindex < level_time {
                        ctx.world.entity_mut(ent_id).s.genericenemyindex = level_time + 2000;
                    }
                }

                let thispush_org: vec3_t = if !ecl.is_null() {
                    (*ecl).ps.origin
                } else {
                    ctx.world.entity(ent_id).s.pos.trBase
                };

                //not in the arc, don't consider it
                let mut tto2: vec3_t = (*cl).ps.origin;
                tto2[2] += (*cl).ps.viewheight as f32;
                let mut a: vec3_t = [
                    thispush_org[0] - tto2[0],
                    thispush_org[1] - tto2[1],
                    thispush_org[2] - tto2[2],
                ];
                vectoangles(a, &mut a);

                if !ecl.is_null()
                    && InFieldOfVision((*cl).ps.viewangles, visionArc, a) == 0
                    && ForcePowerUsableOn(ctx, Some(self_), Some(ent_id), powerUse) != 0
                {
                    //only bother with arc rules if the victim is a client
                    entityList[e] = ENTITYNUM_NONE;
                } else if !ecl.is_null() {
                    if pull != 0 {
                        if ForcePowerUsableOn(ctx, Some(self_), Some(ent_id), FP_PULL) == 0 {
                            entityList[e] = ENTITYNUM_NONE;
                        }
                    } else {
                        if ForcePowerUsableOn(ctx, Some(self_), Some(ent_id), FP_PUSH) == 0 {
                            entityList[e] = ENTITYNUM_NONE;
                        }
                    }
                }
                e += 1;
            }
        }

        for e in 0..(numListedEntities as usize) {
            let ent_id = if entityList[e] != ENTITYNUM_NONE
                && entityList[e] >= 0
                && entityList[e] < MAX_GENTITIES as c_int
            {
                Some(EntityId(entityList[e] as u32))
            } else {
                None
            };

            let ent_id = match ent_id {
                Some(id) => id,
                None => continue,
            };
            if ent_id == self_ {
                continue;
            }
            // FLAG: gclient_t deref stays raw (recipe 2b; `ent_id` can be an NPC
            // pool client).
            let ecl = ctx.world.entity(ent_id).client;
            if !ecl.is_null() && OnSameTeam(ctx, Some(ent_id), Some(self_)) != 0 {
                continue;
            }
            if ctx.world.entity(ent_id).inuse == 0 {
                continue;
            }
            if ctx.world.entity(ent_id).s.eType != ET_MISSILE as c_int {
                if ctx.world.entity(ent_id).s.eType != ET_ITEM as c_int {
                    //FIXME: need pushable objects
                    let classname = cstr_to_str(ctx.world.entity(ent_id).classname);
                    if classname.eq_ignore_ascii_case("func_button") {
                        //we might push it
                        if pull != 0
                            || ctx.world.entity(ent_id).spawnflags & SPF_BUTTON_FPUSHABLE == 0
                        {
                            //not force-pushable, never pullable
                            continue;
                        }
                    } else {
                        if ctx.world.entity(ent_id).s.eFlags & EF_NODRAW != 0 {
                            continue;
                        }
                        if ecl.is_null() {
                            if !classname.eq_ignore_ascii_case("lightsaber") {
                                //not a lightsaber
                                if !classname.eq_ignore_ascii_case("func_door")
                                    || ctx.world.entity(ent_id).spawnflags & 2 == 0
                                //not a force-usable door
                                {
                                    if !classname.eq_ignore_ascii_case("func_static")
                                        || (ctx.world.entity(ent_id).spawnflags & 1 == 0
                                            && ctx.world.entity(ent_id).spawnflags & 2 == 0)
                                    //not a force-usable func_static
                                    {
                                        if !classname.eq_ignore_ascii_case("limb") {
                                            //not a limb
                                            continue;
                                        }
                                    }
                                } else if ctx.world.entity(ent_id).moverState != MOVER_POS1 as c_int
                                    && ctx.world.entity(ent_id).moverState != MOVER_POS2 as c_int
                                {
                                    //not at rest
                                    continue;
                                }
                            }
                        } else if (*ecl).NPC_class == CLASS_GALAKMECH
                            || (*ecl).NPC_class == CLASS_ATST
                            || (*ecl).NPC_class == CLASS_RANCOR
                        {
                            //can't push ATST or Galak or Rancor
                            continue;
                        }
                    }
                }
            } else {
                if ctx.world.entity(ent_id).s.pos.trType == TR_STATIONARY
                    && ctx.world.entity(ent_id).s.eFlags & EF_MISSILE_STICK != 0
                {
                    //can't force-push/pull stuck missiles (detpacks, tripmines)
                    continue;
                }
                if ctx.world.entity(ent_id).s.pos.trType == TR_STATIONARY
                    && ctx.world.entity(ent_id).s.weapon != WP_THERMAL as c_int
                {
                    //only thermal detonators can be pushed once stopped
                    continue;
                }
            }

            //this is all to see if we need to start a saber attack, if it's in flight, this doesn't matter
            // find the distance from the edge of the bounding box
            let mut v: vec3_t = [0.0; 3];
            for i in 0..3 {
                if center[i] < ctx.world.entity(ent_id).r.absmin[i] {
                    v[i] = ctx.world.entity(ent_id).r.absmin[i] - center[i];
                } else if center[i] > ctx.world.entity(ent_id).r.absmax[i] {
                    v[i] = center[i] - ctx.world.entity(ent_id).r.absmax[i];
                } else {
                    v[i] = 0.0;
                }
            }

            let size: vec3_t = [
                ctx.world.entity(ent_id).r.absmax[0] - ctx.world.entity(ent_id).r.absmin[0],
                ctx.world.entity(ent_id).r.absmax[1] - ctx.world.entity(ent_id).r.absmin[1],
                ctx.world.entity(ent_id).r.absmax[2] - ctx.world.entity(ent_id).r.absmin[2],
            ];
            let ent_org: vec3_t = [
                ctx.world.entity(ent_id).r.absmin[0] + 0.5 * size[0],
                ctx.world.entity(ent_id).r.absmin[1] + 0.5 * size[1],
                ctx.world.entity(ent_id).r.absmin[2] + 0.5 * size[2],
            ];

            let mut dir: vec3_t = [
                ent_org[0] - center[0],
                ent_org[1] - center[1],
                ent_org[2] - center[2],
            ];
            VectorNormalize(&mut dir);
            let dot1 = dir[0] * forward[0] + dir[1] * forward[1] + dir[2] * forward[2];
            if dot1 < 0.6 {
                continue;
            }

            let dist = VectorLength(v);

            //Now check and see if we can actually deflect it
            //method1
            //if within a certain range, deflect it
            if dist >= radius {
                continue;
            }

            //in PVS?
            if ctx.world.entity(ent_id).r.bmodel == 0
                && trap::InPVS(
                    ctx.engine,
                    GInPvsArgs::new(&ent_org as *const vec3_t, &(*cl).ps.origin as *const vec3_t),
                ) == 0
            {
                //must be in PVS
                continue;
            }

            //really should have a clear LOS to this thing...
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &(*cl).ps.origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &vec3_origin as *const vec3_t,
                    &ent_org as *const vec3_t,
                    ctx.entity(self_).s.number,
                    MASK_SHOT,
                ),
            );
            if tr.fraction < 1.0 && tr.entityNum != (ctx.world.entity(ent_id).s.number) as i16 {
                //must have clear LOS
                //try from eyes too before you give up
                let mut eyePoint: vec3_t = (*cl).ps.origin;
                eyePoint[2] += (*cl).ps.viewheight as f32;
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &eyePoint as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &vec3_origin as *const vec3_t,
                        &ent_org as *const vec3_t,
                        ctx.entity(self_).s.number,
                        MASK_SHOT,
                    ),
                );

                if tr.fraction < 1.0 && tr.entityNum != (ctx.world.entity(ent_id).s.number) as i16 {
                    continue;
                }
            }

            // ok, we are within the radius, add us to the incoming list
            push_list[ent_count] = ent_id;
            ent_count += 1;
        }

        if ent_count != 0 {
            //method1:
            for x in 0..ent_count {
                // FLAG: gclient_t deref stays raw (recipe 2b; target can be an NPC
                // pool client); read the pointer value via the safe entity borrow.
                let pcl = ctx.world.entity(push_list[x]).client;
                let mut modPowerLevel = powerLevel;

                if !pcl.is_null() {
                    modPowerLevel = WP_AbsorbConversion(
                        ctx,
                        push_list[x],
                        (*pcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
                        Some(self_),
                        powerUse,
                        powerLevel,
                        forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[powerUse as usize] as usize]
                            [powerUse as usize],
                    );
                    if modPowerLevel == -1 {
                        modPowerLevel = powerLevel;
                    }
                }

                pushPower = 256 * modPowerLevel;

                let thispush_org: vec3_t = if !pcl.is_null() {
                    (*pcl).ps.origin
                } else {
                    ctx.world.entity(push_list[x]).s.origin
                };

                if !pcl.is_null() {
                    //FIXME: make enemy jedi able to hunker down and resist this?
                    let mut otherPushPower = (*pcl).ps.fd.forcePowerLevel[powerUse as usize];
                    let mut canPullWeapon = qtrue;
                    let mut dirLen: f32 = 0.0;

                    if ctx.world.cvars.g_debugMelee.integer != 0 {
                        if (*pcl).ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                            //no resistance if stuck to wall
                            //push/pull them off the wall
                            otherPushPower = 0;
                            G_LetGoOfWall(ctx, Some(push_list[x]));
                        }
                    }

                    let knockback: f32 = if pull != 0 { 0.0 } else { 200.0 };
                    let _ = knockback;

                    // Raven `int pushPowerMod`: each compound step evaluates in
                    // double then truncates back to int before the next use.
                    let mut pushPowerMod: c_int = pushPower;

                    if (*pcl).pers.cmd.forwardmove != 0 || (*pcl).pers.cmd.rightmove != 0 {
                        //if you are moving, you get one less level of defense
                        otherPushPower -= 1;

                        if otherPushPower < 0 {
                            otherPushPower = 0;
                        }
                    }

                    if otherPushPower != 0
                        && CanCounterThrow(ctx, push_list[x], Some(self_), pull) != 0
                    {
                        if pull != 0 {
                            let s = cstr("sound/weapons/force/pull.wav");
                            G_Sound(ctx, Some(push_list[x]), CHAN_BODY, G_SoundIndex(s.as_ptr()));
                            (*pcl).ps.forceHandExtend = HANDEXTEND_FORCEPULL as c_int;
                            (*pcl).ps.forceHandExtendTime = level_time + 400;
                        } else {
                            let s = cstr("sound/weapons/force/push.wav");
                            G_Sound(ctx, Some(push_list[x]), CHAN_BODY, G_SoundIndex(s.as_ptr()));
                            (*pcl).ps.forceHandExtend = HANDEXTEND_FORCEPUSH as c_int;
                            (*pcl).ps.forceHandExtendTime = level_time + 1000;
                        }
                        (*pcl).ps.powerups[PW_DISINT_4 as usize] =
                            (*pcl).ps.forceHandExtendTime + 200;

                        if pull != 0 {
                            (*pcl).ps.powerups[PW_PULL as usize] =
                                (*pcl).ps.powerups[PW_DISINT_4 as usize];
                        } else {
                            (*pcl).ps.powerups[PW_PULL as usize] = 0;
                        }

                        //Make a counter-throw effect

                        if otherPushPower >= modPowerLevel {
                            pushPowerMod = 0;
                            canPullWeapon = qfalse;
                        } else {
                            let powerDif = modPowerLevel - otherPushPower;

                            if powerDif >= 3 {
                                pushPowerMod =
                                    (pushPowerMod as f64 - pushPowerMod as f64 * 0.2) as c_int;
                            } else if powerDif == 2 {
                                pushPowerMod =
                                    (pushPowerMod as f64 - pushPowerMod as f64 * 0.4) as c_int;
                            } else if powerDif == 1 {
                                pushPowerMod =
                                    (pushPowerMod as f64 - pushPowerMod as f64 * 0.8) as c_int;
                            }

                            if pushPowerMod < 0 {
                                pushPowerMod = 0;
                            }
                        }
                    }

                    //shove them
                    let pushDir: vec3_t;
                    if pull != 0 {
                        pushDir = [
                            (*cl).ps.origin[0] - thispush_org[0],
                            (*cl).ps.origin[1] - thispush_org[1],
                            (*cl).ps.origin[2] - thispush_org[2],
                        ];

                        if VectorLength(pushDir) <= 256.0 {
                            let mut randfact: c_int = 0;

                            if modPowerLevel == FORCE_LEVEL_1 as c_int {
                                randfact = 3;
                            } else if modPowerLevel == FORCE_LEVEL_2 as c_int {
                                randfact = 7;
                            } else if modPowerLevel == FORCE_LEVEL_3 as c_int {
                                randfact = 10;
                            }

                            if OnSameTeam(ctx, Some(self_), Some(push_list[x])) == 0
                                && ctx.world.bg_state.rng.Q_irand(1, 10) <= randfact
                                && canPullWeapon != 0
                            {
                                let mut uorg: vec3_t = (*cl).ps.origin;
                                uorg[2] += 64.0;

                                let mut vecnorm: vec3_t = [
                                    uorg[0] - thispush_org[0],
                                    uorg[1] - thispush_org[1],
                                    uorg[2] - thispush_org[2],
                                ];
                                VectorNormalize(&mut vecnorm);

                                TossClientWeapon(ctx, push_list[x], vecnorm, 500.0);
                            }
                        }
                    } else {
                        pushDir = [
                            thispush_org[0] - (*cl).ps.origin[0],
                            thispush_org[1] - (*cl).ps.origin[1],
                            thispush_org[2] - (*cl).ps.origin[2],
                        ];
                    }
                    let mut pushDir = pushDir;

                    if (modPowerLevel > otherPushPower || (*pcl).ps.m_iVehicleNum != 0)
                        && !pcl.is_null()
                    {
                        if modPowerLevel == FORCE_LEVEL_3 as c_int
                            && (*pcl).ps.forceHandExtend != HANDEXTEND_KNOCKDOWN as c_int
                        {
                            dirLen = VectorLength(pushDir);

                            if BG_KnockDownable(&mut (*pcl).ps) != 0
                                && dirLen <= (64.0 * ((modPowerLevel - otherPushPower) - 1) as f32)
                            {
                                //can only do a knockdown if fairly close
                                (*pcl).ps.forceHandExtend = HANDEXTEND_KNOCKDOWN as c_int;
                                (*pcl).ps.forceHandExtendTime = level_time + 700;
                                (*pcl).ps.forceDodgeAnim = 0; //this toggles between 1 and 0, when it's 1 we should play the get up anim
                                (*pcl).ps.quickerGetup = qtrue;
                            } else if ctx.world.entity(push_list[x]).s.number < MAX_CLIENTS as c_int
                                && (*pcl).ps.m_iVehicleNum != 0
                                && dirLen <= 128.0
                            {
                                //a player on a vehicle
                                let veh_id = EntityId((*pcl).ps.m_iVehicleNum as u32);
                                if ctx.world.entity(veh_id).inuse != qfalse
                                    && !ctx.world.entity(veh_id).client.is_null()
                                    && !ctx.world.entity(veh_id).m_pVehicle.is_null()
                                {
                                    // FLAG: vehicle-vtable seam — `m_pVehicle`/
                                    // `m_pVehicleInfo` derefs stay raw; `eject` takes
                                    // a raw `bgEntity_t*` (no EntityId overload,
                                    // one-file rule), materialized tight at the call.
                                    let pVeh = ctx.world.entity(veh_id).m_pVehicle;
                                    if (*(*pVeh).m_pVehicleInfo).r#type == vehicleType_t::VH_SPEEDER
                                        || (*(*pVeh).m_pVehicleInfo).r#type
                                            == vehicleType_t::VH_ANIMAL
                                    {
                                        //push the guy off
                                        let pEnt = ctx.world.entity_mut(push_list[x])
                                            as *mut gentity_t
                                            as *mut bgEntity_t;
                                        crate::veh_dispatch::eject(ctx, pVeh, pEnt, qfalse);
                                    }
                                }
                            }
                        }
                    }

                    if dirLen == 0.0 {
                        dirLen = VectorLength(pushDir);
                    }

                    VectorNormalize(&mut pushDir);

                    //escape a force grip if we're in one
                    if (*cl).ps.fd.forceGripBeingGripped > level_time as f32 {
                        //force the enemy to stop gripping me if I managed to push him
                        if (*pcl).ps.fd.forceGripEntityNum == ctx.entity(self_).s.number {
                            if modPowerLevel >= (*pcl).ps.fd.forcePowerLevel[FP_GRIP as usize] {
                                //only break the grip if our push/pull level is >= their grip level
                                WP_ForcePowerStop(ctx, push_list[x], FP_GRIP);
                                (*cl).ps.fd.forceGripBeingGripped = 0.0;
                                (*pcl).ps.fd.forceGripUseTime = level_time + 1000;
                                //since we just broke out of it..
                            }
                        }
                    }

                    (*pcl).ps.otherKiller = ctx.entity(self_).s.number;
                    (*pcl).ps.otherKillerTime = level_time + 5000;
                    (*pcl).ps.otherKillerDebounceTime = level_time + 100;
                    (*pcl).otherKillerMOD = MOD_UNKNOWN as c_int;
                    (*pcl).otherKillerVehWeapon = 0;
                    (*pcl).otherKillerWeaponType = WP_NONE as c_int;

                    pushPowerMod = (pushPowerMod as f64 - dirLen as f64 * 0.7) as c_int;
                    if pushPowerMod < 16 {
                        pushPowerMod = 16;
                    }

                    //fullbody push effect
                    (*pcl).pushEffectTime = level_time + 600;

                    (*pcl).ps.velocity[0] = pushDir[0] * pushPowerMod as f32;
                    (*pcl).ps.velocity[1] = pushDir[1] * pushPowerMod as f32;

                    if (*pcl).ps.velocity[2] as c_int == 0 {
                        //if not going anywhere vertically, boost them up a bit
                        (*pcl).ps.velocity[2] = pushDir[2] * pushPowerMod as f32;

                        if (*pcl).ps.velocity[2] < 128.0 {
                            (*pcl).ps.velocity[2] = 128.0;
                        }
                    } else {
                        (*pcl).ps.velocity[2] = pushDir[2] * pushPowerMod as f32;
                    }
                } else if ctx.world.entity(push_list[x]).s.eType == ET_MISSILE as c_int
                    && ctx.world.entity(push_list[x]).s.pos.trType != TR_STATIONARY
                    && (ctx.world.entity(push_list[x]).s.pos.trType != TR_INTERPOLATE
                        || ctx.world.entity(push_list[x]).s.weapon != WP_THERMAL as c_int)
                //rolling and stationary thermal detonators are dealt with below
                {
                    if pull != 0 {
                        //deflect rather than reflect?
                    } else {
                        G_ReflectMissile(ctx, self_, push_list[x], forward);
                    }
                } else if cstr_to_str(ctx.world.entity(push_list[x]).classname)
                    .eq_ignore_ascii_case("func_static")
                {
                    //force-usable func_static
                    if pull == 0 && ctx.world.entity(push_list[x]).spawnflags & 1 != 0 {
                        GEntity_UseFunc(ctx, push_list[x], Some(self_), Some(self_));
                    } else if pull != 0 && ctx.world.entity(push_list[x]).spawnflags & 2 != 0 {
                        GEntity_UseFunc(ctx, push_list[x], Some(self_), Some(self_));
                    }
                } else if cstr_to_str(ctx.world.entity(push_list[x]).classname)
                    .eq_ignore_ascii_case("func_door")
                    && ctx.world.entity(push_list[x]).spawnflags & 2 != 0
                {
                    //push/pull the door
                    let mut trFrom: vec3_t = (*cl).ps.origin;
                    trFrom[2] += (*cl).ps.viewheight as f32;

                    let mut fwd2: vec3_t = [0.0; 3];
                    AngleVectors((*cl).ps.viewangles, Some(&mut fwd2), None, None);
                    VectorNormalize(&mut fwd2);
                    let end: vec3_t = [
                        trFrom[0] + radius * fwd2[0],
                        trFrom[1] + radius * fwd2[1],
                        trFrom[2] + radius * fwd2[2],
                    ];
                    trap::Trace(
                        ctx.engine,
                        GTraceArgs::new(
                            &mut tr as *mut trace_t,
                            &trFrom as *const vec3_t,
                            &vec3_origin as *const vec3_t,
                            &vec3_origin as *const vec3_t,
                            &end as *const vec3_t,
                            ctx.entity(self_).s.number,
                            MASK_SHOT,
                        ),
                    );
                    if tr.entityNum != (ctx.world.entity(push_list[x]).s.number) as i16
                        || tr.fraction == 1.0
                        || tr.allsolid != 0
                        || tr.startsolid != 0
                    {
                        //must be pointing right at it
                        continue;
                    }

                    let mut center2: vec3_t;
                    let mut pos1: vec3_t;
                    let mut pos2: vec3_t;
                    if VectorCompare(vec3_origin, ctx.world.entity(push_list[x]).s.origin) != 0 {
                        //does not have an origin brush, so pos1 & pos2 are relative to world origin, need to calc center
                        let size: vec3_t = [
                            ctx.world.entity(push_list[x]).r.absmax[0]
                                - ctx.world.entity(push_list[x]).r.absmin[0],
                            ctx.world.entity(push_list[x]).r.absmax[1]
                                - ctx.world.entity(push_list[x]).r.absmin[1],
                            ctx.world.entity(push_list[x]).r.absmax[2]
                                - ctx.world.entity(push_list[x]).r.absmin[2],
                        ];
                        center2 = [
                            ctx.world.entity(push_list[x]).r.absmin[0] + 0.5 * size[0],
                            ctx.world.entity(push_list[x]).r.absmin[1] + 0.5 * size[1],
                            ctx.world.entity(push_list[x]).r.absmin[2] + 0.5 * size[2],
                        ];
                        if ctx.world.entity(push_list[x]).spawnflags & 1 != 0
                            && ctx.world.entity(push_list[x]).moverState == MOVER_POS1 as c_int
                        {
                            //if at pos1 and started open, make sure we get the center where it *started* because we're going to add back in the relative values pos1 and pos2
                            center2 = [
                                center2[0] - ctx.world.entity(push_list[x]).pos1[0],
                                center2[1] - ctx.world.entity(push_list[x]).pos1[1],
                                center2[2] - ctx.world.entity(push_list[x]).pos1[2],
                            ];
                        } else if ctx.world.entity(push_list[x]).spawnflags & 1 == 0
                            && ctx.world.entity(push_list[x]).moverState == MOVER_POS2 as c_int
                        {
                            //if at pos2, make sure we get the center where it *started* because we're going to add back in the relative values pos1 and pos2
                            center2 = [
                                center2[0] - ctx.world.entity(push_list[x]).pos2[0],
                                center2[1] - ctx.world.entity(push_list[x]).pos2[1],
                                center2[2] - ctx.world.entity(push_list[x]).pos2[2],
                            ];
                        }
                        pos1 = [
                            center2[0] + ctx.world.entity(push_list[x]).pos1[0],
                            center2[1] + ctx.world.entity(push_list[x]).pos1[1],
                            center2[2] + ctx.world.entity(push_list[x]).pos1[2],
                        ];
                        pos2 = [
                            center2[0] + ctx.world.entity(push_list[x]).pos2[0],
                            center2[1] + ctx.world.entity(push_list[x]).pos2[1],
                            center2[2] + ctx.world.entity(push_list[x]).pos2[2],
                        ];
                    } else {
                        //actually has an origin, pos1 and pos2 are absolute
                        center2 = ctx.world.entity(push_list[x]).r.currentOrigin;
                        pos1 = ctx.world.entity(push_list[x]).pos1;
                        pos2 = ctx.world.entity(push_list[x]).pos2;
                    }

                    if Distance(pos1, trFrom) < Distance(pos2, trFrom) {
                        //pos1 is closer
                        if ctx.world.entity(push_list[x]).moverState == MOVER_POS1 as c_int {
                            //at the closest pos
                            if pull != 0 {
                                //trying to pull, but already at closest point, so screw it
                                continue;
                            }
                        } else if ctx.world.entity(push_list[x]).moverState == MOVER_POS2 as c_int {
                            //at farthest pos
                            if pull == 0 {
                                //trying to push, but already at farthest point, so screw it
                                continue;
                            }
                        }
                    } else {
                        //pos2 is closer
                        if ctx.world.entity(push_list[x]).moverState == MOVER_POS1 as c_int {
                            //at the farthest pos
                            if pull == 0 {
                                //trying to push, but already at farthest point, so screw it
                                continue;
                            }
                        } else if ctx.world.entity(push_list[x]).moverState == MOVER_POS2 as c_int {
                            //at closest pos
                            if pull != 0 {
                                //trying to pull, but already at closest point, so screw it
                                continue;
                            }
                        }
                    }
                    GEntity_UseFunc(ctx, push_list[x], Some(self_), Some(self_));
                } else if cstr_to_str(ctx.world.entity(push_list[x]).classname)
                    .eq_ignore_ascii_case("func_button")
                {
                    //pretend you pushed it
                    Touch_Button(ctx, push_list[x], Some(self_), std::ptr::null_mut());
                    continue;
                }
            }
        }

        //attempt to break any leftover grips
        //if we're still in a current grip that wasn't broken by the push, it will still remain
        (*cl).dangerTime = level_time;
        (*cl).ps.eFlags &= !EF_INVULNERABLE;
        (*cl).invulnerableTimer = 0;

        if (*cl).ps.fd.forceGripBeingGripped > level_time as f32 {
            (*cl).ps.fd.forceGripBeingGripped = 0.0;
        }
    }
}

/// Raven `WP_ForcePowerStop`.
///
/// Source: `oracle/codemp/game/w_force.c:3822-3946`
pub fn WP_ForcePowerStop(ctx: &mut GameContext, self_: EntityId, forcePower: forcePowers_t) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let wasActive = (*cl).ps.fd.forcePowersActive;

        (*cl).ps.fd.forcePowersActive &= !(1 << forcePower);

        match forcePower {
            FP_HEAL => {
                (*cl).ps.fd.forceHealAmount = 0;
                (*cl).ps.fd.forceHealTime = 0;
            }
            FP_LEVITATION => {}
            FP_SPEED => {
                if wasActive & (1 << FP_SPEED) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_2 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_PUSH => {}
            FP_PULL => {}
            FP_TELEPATHY => {
                if wasActive & (1 << FP_TELEPATHY) != 0 {
                    let snd =
                        std::ffi::CString::new("sound/weapons/force/distractstop.wav").unwrap();
                    G_Sound(ctx, Some(self_), CHAN_AUTO, G_SoundIndex(snd.as_ptr()));
                }
                (*cl).ps.fd.forceMindtrickTargetIndex = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex2 = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex3 = 0;
                (*cl).ps.fd.forceMindtrickTargetIndex4 = 0;
            }
            FP_SEE => {
                if wasActive & (1 << FP_SEE) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_5 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_GRIP => {
                (*cl).ps.fd.forceGripUseTime = level_time + 3000;
                let grip_id = EntityId((*cl).ps.fd.forceGripEntityNum as u32);
                // FLAG: gclient_t deref stays raw (recipe 2b).
                let gcl = ctx.entity(grip_id).client;
                if (*cl).ps.fd.forcePowerLevel[FP_GRIP as usize] > FORCE_LEVEL_1
                    && !gcl.is_null()
                    && ctx.entity(grip_id).health > 0
                    && ctx.entity(grip_id).inuse != 0
                    && (level_time as f32 - (*gcl).ps.fd.forceGripStarted) > 500.0
                {
                    //if we had our throat crushed in for more than half a second, gasp for air when we're let go
                    if wasActive & (1 << FP_GRIP) != 0 {
                        let snd = std::ffi::CString::new("*gasp.wav").unwrap();
                        G_EntitySound(ctx, grip_id, CHAN_VOICE, G_SoundIndex(snd.as_ptr()));
                    }
                }

                if !gcl.is_null() && ctx.entity(grip_id).inuse != 0 {
                    (*gcl).ps.forceGripChangeMovetype = PM_NORMAL as c_int;
                }

                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0;
                }

                (*cl).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;

                (*cl).ps.powerups[PW_DISINT_4 as usize] = 0;
            }
            FP_LIGHTNING => {
                if (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] < FORCE_LEVEL_2 {
                    //don't do it again for 3 seconds, minimum...
                    (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] = level_time + 3000;
                } else {
                    (*cl).ps.fd.forcePowerDebounce[FP_LIGHTNING as usize] = level_time + 1500;
                }
                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0; //reset hand position
                }

                (*cl).ps.activeForcePass = 0;
            }
            FP_RAGE => {
                (*cl).ps.fd.forceRageRecoveryTime = level_time + 10000;
                if wasActive & (1 << FP_RAGE) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_ABSORB => {
                if wasActive & (1 << FP_ABSORB) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_PROTECT => {
                if wasActive & (1 << FP_PROTECT) != 0 {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_3 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                }
            }
            FP_DRAIN => {
                if (*cl).ps.fd.forcePowerLevel[FP_DRAIN as usize] < FORCE_LEVEL_2 {
                    //don't do it again for 3 seconds, minimum...
                    (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] = level_time + 3000;
                } else {
                    (*cl).ps.fd.forcePowerDebounce[FP_DRAIN as usize] = level_time + 1500;
                }

                if (*cl).ps.forceHandExtend == HANDEXTEND_FORCE_HOLD as c_int {
                    (*cl).ps.forceHandExtendTime = 0; //reset hand position
                }

                (*cl).ps.activeForcePass = 0;
            }
            _ => {}
        }
    }
}

/// Raven `DoGripAction`.
///
/// Source: `oracle/codemp/game/w_force.c:3948-4162`
pub fn DoGripAction(ctx: &mut GameContext, self_: EntityId, forcePower: forcePowers_t) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow. `self_`'s number is invariant — hoist as a local.
        let cl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let level_time = ctx.world.level.time;

        (*cl).dangerTime = level_time;
        (*cl).ps.eFlags &= !EF_INVULNERABLE;
        (*cl).invulnerableTimer = 0;

        let grip_id = EntityId((*cl).ps.fd.forceGripEntityNum as u32);
        let gcl = ctx.entity(grip_id).client;

        if gcl.is_null()
            || ctx.entity(grip_id).inuse == 0
            || ctx.entity(grip_id).health < 1
            || ForcePowerUsableOn(ctx, Some(self_), Some(grip_id), FP_GRIP) == 0
        {
            WP_ForcePowerStop(ctx, self_, forcePower);
            (*cl).ps.fd.forceGripEntityNum = ENTITYNUM_NONE;

            if !gcl.is_null() && ctx.entity(grip_id).inuse != 0 {
                (*gcl).ps.forceGripChangeMovetype = PM_NORMAL as c_int;
            }
            return;
        }

        let a: vec3_t = [
            (*gcl).ps.origin[0] - (*cl).ps.origin[0],
            (*gcl).ps.origin[1] - (*cl).ps.origin[1],
            (*gcl).ps.origin[2] - (*cl).ps.origin[2],
        ];

        let mut tr: trace_t = core::mem::zeroed();
        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*cl).ps.origin as *const vec3_t,
                std::ptr::null(),
                std::ptr::null(),
                &(*gcl).ps.origin as *const vec3_t,
                self_number,
                MASK_PLAYERSOLID,
            ),
        );

        let mut gripLevel = WP_AbsorbConversion(
            ctx,
            grip_id,
            (*gcl).ps.fd.forcePowerLevel[FP_ABSORB as usize],
            Some(self_),
            FP_GRIP,
            (*cl).ps.fd.forcePowerLevel[FP_GRIP as usize],
            forcePowerNeeded[(*cl).ps.fd.forcePowerLevel[FP_GRIP as usize] as usize]
                [FP_GRIP as usize],
        );

        if gripLevel == -1 {
            gripLevel = (*cl).ps.fd.forcePowerLevel[FP_GRIP as usize];
        }

        if gripLevel == 0 {
            WP_ForcePowerStop(ctx, self_, forcePower);
            return;
        }

        if VectorLength(a) > (MAX_GRIP_DISTANCE) as f32 {
            WP_ForcePowerStop(ctx, self_, forcePower);
            return;
        }

        if InFront((*gcl).ps.origin, (*cl).ps.origin, (*cl).ps.viewangles, 0.9) == 0
            && gripLevel < FORCE_LEVEL_3 as c_int
        {
            WP_ForcePowerStop(ctx, self_, forcePower);
            return;
        }

        if tr.fraction != 1.0 && tr.entityNum != (ctx.entity(grip_id).s.number) as i16 {
            WP_ForcePowerStop(ctx, self_, forcePower);
            return;
        }

        if (*cl).ps.fd.forcePowerDebounce[FP_GRIP as usize] < level_time {
            //2 damage per second while choking, resulting in 10 damage total (not including The Squeeze<tm>)
            (*cl).ps.fd.forcePowerDebounce[FP_GRIP as usize] = level_time + 1000;
            G_Damage(
                ctx,
                Some(grip_id),
                Some(self_),
                Some(self_),
                None,
                [0.0; 3],
                2,
                DAMAGE_NO_ARMOR,
                MOD_FORCE_DARK as c_int,
            );
        }

        //make sure the guy being gripped has his jetpack off.
        Jetpack_Off(ctx.entity_mut(grip_id));

        if gripLevel == FORCE_LEVEL_1 as c_int {
            (*gcl).ps.fd.forceGripBeingGripped = (level_time + 1000) as f32;

            if (level_time - (*gcl).ps.fd.forceGripStarted as c_int) > 5000 {
                WP_ForcePowerStop(ctx, self_, forcePower);
            }
            return;
        }

        if gripLevel == FORCE_LEVEL_2 as c_int {
            (*gcl).ps.fd.forceGripBeingGripped = (level_time + 1000) as f32;

            if (*gcl).ps.forceGripMoveInterval < level_time {
                (*gcl).ps.velocity[2] = 30.0;

                (*gcl).ps.forceGripMoveInterval = level_time + 300; //only update velocity every 300ms, so as to avoid heavy bandwidth usage
            }

            (*gcl).ps.otherKiller = self_number;
            (*gcl).ps.otherKillerTime = level_time + 5000;
            (*gcl).ps.otherKillerDebounceTime = level_time + 100;
            (*gcl).otherKillerMOD = MOD_UNKNOWN as c_int;
            (*gcl).otherKillerVehWeapon = 0;
            (*gcl).otherKillerWeaponType = WP_NONE as c_int;

            (*gcl).ps.forceGripChangeMovetype = PM_FLOAT as c_int;

            if (level_time - (*gcl).ps.fd.forceGripStarted as c_int) > 3000
                && (*cl).ps.fd.forceGripDamageDebounceTime == 0
            {
                //if we managed to lift him into the air for 2 seconds, give him a crack
                (*cl).ps.fd.forceGripDamageDebounceTime = 1;
                G_Damage(
                    ctx,
                    Some(grip_id),
                    Some(self_),
                    Some(self_),
                    None,
                    [0.0; 3],
                    20,
                    DAMAGE_NO_ARMOR,
                    MOD_FORCE_DARK as c_int,
                );

                //Must play custom sounds on the actual entity. Don't use G_Sound (it creates a temp entity for the sound)
                let snd = format!("*choke{}.wav", ctx.world.bg_state.rng.Q_irand(1, 3));
                G_EntitySound(ctx, grip_id, CHAN_VOICE, G_SoundIndex(cstr(&snd).as_ptr()));

                (*gcl).ps.forceHandExtend = HANDEXTEND_CHOKE as c_int;
                (*gcl).ps.forceHandExtendTime = level_time + 2000;

                if (*gcl).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                    //choking, so don't let him keep gripping himself
                    WP_ForcePowerStop(ctx, grip_id, FP_GRIP);
                }
            } else if (level_time - (*gcl).ps.fd.forceGripStarted as c_int) > 4000 {
                WP_ForcePowerStop(ctx, self_, forcePower);
            }
            return;
        }

        if gripLevel == FORCE_LEVEL_3 as c_int {
            (*gcl).ps.fd.forceGripBeingGripped = (level_time + 1000) as f32;

            (*gcl).ps.otherKiller = self_number;
            (*gcl).ps.otherKillerTime = level_time + 5000;
            (*gcl).ps.otherKillerDebounceTime = level_time + 100;
            (*gcl).otherKillerMOD = MOD_UNKNOWN as c_int;
            (*gcl).otherKillerVehWeapon = 0;
            (*gcl).otherKillerWeaponType = WP_NONE as c_int;

            (*gcl).ps.forceGripChangeMovetype = PM_FLOAT as c_int;

            if (*gcl).ps.forceGripMoveInterval < level_time {
                let start_o: vec3_t = (*gcl).ps.origin;
                let mut fwd: vec3_t = [0.0; 3];
                AngleVectors((*cl).ps.viewangles, Some(&mut fwd), None, None);
                let mut fwd_o: vec3_t = [
                    (*cl).ps.origin[0] + fwd[0] * 128.0,
                    (*cl).ps.origin[1] + fwd[1] * 128.0,
                    (*cl).ps.origin[2] + fwd[2] * 128.0,
                ];
                fwd_o[2] += 16.0;
                let mut nvel: vec3_t = [
                    fwd_o[0] - start_o[0],
                    fwd_o[1] - start_o[1],
                    fwd_o[2] - start_o[2],
                ];

                let nvLen = VectorLength(nvel);

                if nvLen < 16.0 {
                    //within x units of desired spot
                    VectorNormalize(&mut nvel);
                    (*gcl).ps.velocity[0] = nvel[0] * 8.0;
                    (*gcl).ps.velocity[1] = nvel[1] * 8.0;
                    (*gcl).ps.velocity[2] = nvel[2] * 8.0;
                } else if nvLen < 64.0 {
                    VectorNormalize(&mut nvel);
                    (*gcl).ps.velocity[0] = nvel[0] * 128.0;
                    (*gcl).ps.velocity[1] = nvel[1] * 128.0;
                    (*gcl).ps.velocity[2] = nvel[2] * 128.0;
                } else if nvLen < 128.0 {
                    VectorNormalize(&mut nvel);
                    (*gcl).ps.velocity[0] = nvel[0] * 256.0;
                    (*gcl).ps.velocity[1] = nvel[1] * 256.0;
                    (*gcl).ps.velocity[2] = nvel[2] * 256.0;
                } else if nvLen < 200.0 {
                    VectorNormalize(&mut nvel);
                    (*gcl).ps.velocity[0] = nvel[0] * 512.0;
                    (*gcl).ps.velocity[1] = nvel[1] * 512.0;
                    (*gcl).ps.velocity[2] = nvel[2] * 512.0;
                } else {
                    VectorNormalize(&mut nvel);
                    (*gcl).ps.velocity[0] = nvel[0] * 700.0;
                    (*gcl).ps.velocity[1] = nvel[1] * 700.0;
                    (*gcl).ps.velocity[2] = nvel[2] * 700.0;
                }

                (*gcl).ps.forceGripMoveInterval = level_time + 300; //only update velocity every 300ms, so as to avoid heavy bandwidth usage
            }

            if (level_time - (*gcl).ps.fd.forceGripStarted as c_int) > 3000
                && (*cl).ps.fd.forceGripDamageDebounceTime == 0
            {
                //if we managed to lift him into the air for 2 seconds, give him a crack
                (*cl).ps.fd.forceGripDamageDebounceTime = 1;
                G_Damage(
                    ctx,
                    Some(grip_id),
                    Some(self_),
                    Some(self_),
                    None,
                    [0.0; 3],
                    40,
                    DAMAGE_NO_ARMOR,
                    MOD_FORCE_DARK as c_int,
                );

                //Must play custom sounds on the actual entity. Don't use G_Sound (it creates a temp entity for the sound)
                let snd = format!("*choke{}.wav", ctx.world.bg_state.rng.Q_irand(1, 3));
                G_EntitySound(ctx, grip_id, CHAN_VOICE, G_SoundIndex(cstr(&snd).as_ptr()));

                (*gcl).ps.forceHandExtend = HANDEXTEND_CHOKE as c_int;
                (*gcl).ps.forceHandExtendTime = level_time + 2000;

                if (*gcl).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                    //choking, so don't let him keep gripping himself
                    WP_ForcePowerStop(ctx, grip_id, FP_GRIP);
                }
            } else if (level_time - (*gcl).ps.fd.forceGripStarted as c_int) > 4000 {
                WP_ForcePowerStop(ctx, self_, forcePower);
            }
            return;
        }
    }
}

/// Raven `G_IsMindTricked` — is `client` in one of `fd`'s mindtrick masks?
///
/// Source: `oracle/codemp/game/w_force.c:4164-4206`
pub fn G_IsMindTricked(fd: *mut forcedata_t, client: c_int) -> qboolean {
    unsafe {
        let checkIn;
        let mut sub = 0;

        if fd.is_null() {
            return qfalse;
        }

        let trickIndex1 = (*fd).forceMindtrickTargetIndex;
        let trickIndex2 = (*fd).forceMindtrickTargetIndex2;
        let trickIndex3 = (*fd).forceMindtrickTargetIndex3;
        let trickIndex4 = (*fd).forceMindtrickTargetIndex4;

        if client > 47 {
            checkIn = trickIndex4;
            sub = 48;
        } else if client > 31 {
            checkIn = trickIndex3;
            sub = 32;
        } else if client > 15 {
            checkIn = trickIndex2;
            sub = 16;
        } else {
            checkIn = trickIndex1;
        }

        if checkIn & (1 << (client - sub)) != 0 {
            return qtrue;
        }

        qfalse
    }
}

/// Raven `RemoveTrickedEnt` — clear `client` from `fd`'s mindtrick masks.
///
/// Source: `oracle/codemp/game/w_force.c:4208-4231`
fn RemoveTrickedEnt(fd: *mut forcedata_t, client: c_int) {
    unsafe {
        if fd.is_null() {
            return;
        }

        if client > 47 {
            (*fd).forceMindtrickTargetIndex4 &= !(1 << (client - 48));
        } else if client > 31 {
            (*fd).forceMindtrickTargetIndex3 &= !(1 << (client - 32));
        } else if client > 15 {
            (*fd).forceMindtrickTargetIndex2 &= !(1 << (client - 16));
        } else {
            (*fd).forceMindtrickTargetIndex &= !(1 << client);
        }
    }
}

/// Raven `WP_UpdateMindtrickEnts`.
///
/// Source: `oracle/codemp/game/w_force.c:4236-4280`
fn WP_UpdateMindtrickEnts(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); read pointer values via the
        // safe entity borrow.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let g_time_since = ctx.world.globals.g_TimeSinceLastFrame;
        let gametype = ctx.world.cvars.g_gametype.integer;

        let mut i: c_int = 0;
        while i < MAX_CLIENTS as c_int {
            if G_IsMindTricked(&mut (*cl).ps.fd, i) != 0 {
                let ent_id2 = EntityId(i as u32);
                let ecl = ctx.entity(ent_id2).client;

                if ecl.is_null()
                    || ctx.entity(ent_id2).inuse == 0
                    || ctx.entity(ent_id2).health < 1
                    || ((*ecl).ps.fd.forcePowersActive & (1 << FP_SEE)) != 0
                {
                    RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                } else if (level_time - (*cl).dangerTime) < g_time_since * 4 {
                    //Untrick this entity if the tricker (self) fires while in his fov
                    let en = ctx.entity(ent_id2).s.number;
                    if trap::InPVS(
                        ctx.engine,
                        GInPvsArgs::new(
                            &(*ecl).ps.origin as *const vec3_t,
                            &(*cl).ps.origin as *const vec3_t,
                        ),
                    ) != 0
                        && OrgVisible(ctx, (*ecl).ps.origin, (*cl).ps.origin, en) != 0
                    {
                        RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                    }
                } else if BG_HasYsalamiri(gametype, &mut (*ecl).ps) != 0 {
                    RemoveTrickedEnt(&mut (*cl).ps.fd, i);
                }
            }

            i += 1;
        }

        if (*cl).ps.fd.forceMindtrickTargetIndex == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex2 == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex3 == 0
            && (*cl).ps.fd.forceMindtrickTargetIndex4 == 0
        {
            //everyone who we had tricked is no longer tricked, so stop the power
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
        } else if (*cl).ps.powerups[PW_REDFLAG as usize] != 0
            || (*cl).ps.powerups[PW_BLUEFLAG as usize] != 0
        {
            WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
        }
    }
}

/// Raven `WP_ForcePowerRun`.
///
/// Source: `oracle/codemp/game/w_force.c:4282-4506`
fn WP_ForcePowerRun(
    ctx: &mut GameContext,
    self_: EntityId,
    forcePower: forcePowers_t,
    cmd: *mut usercmd_t,
) {
    // Raven declares `extern usercmd_t ucmd;` here but never references it.
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); `cmd` is a raw seam param.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;

        match forcePower {
            FP_HEAL => {
                if (*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                    && ((*cl).ps.velocity[0] != 0.0
                        || (*cl).ps.velocity[1] != 0.0
                        || (*cl).ps.velocity[2] != 0.0)
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if ctx.entity(self_).health < 1 || (*cl).ps.stats[STAT_HEALTH as usize] < 1 {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forceHealTime > level_time {
                    return;
                }
                if ctx.entity(self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                    //we might start out over max_health and we don't want force heal taking us down
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }
                (*cl).ps.fd.forceHealTime = level_time + 1000;
                ctx.entity_mut(self_).health += 1;
                (*cl).ps.fd.forceHealAmount += 1;

                if ctx.entity(self_).health > (*cl).ps.stats[STAT_MAX_HEALTH as usize] {
                    ctx.entity_mut(self_).health = (*cl).ps.stats[STAT_MAX_HEALTH as usize];
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }

                if ((*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_1
                    && (*cl).ps.fd.forceHealAmount >= 25)
                    || ((*cl).ps.fd.forcePowerLevel[FP_HEAL as usize] == FORCE_LEVEL_2
                        && (*cl).ps.fd.forceHealAmount >= 33)
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }
            }
            FP_SPEED => {
                //This is handled in PM_WalkMove and PM_StepSlideMove
                if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
                    && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
                {
                    if ctx.world.g_entities[(*cl).holdingObjectiveItem as usize].genericValue15 != 0
                    {
                        //disables force powers
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }
                }
            }
            FP_GRIP => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    WP_ForcePowerStop(ctx, self_, FP_GRIP);
                    return;
                }

                if (*cl).ps.fd.forcePowerDebounce[FP_PULL as usize] < level_time {
                    //Using the debounce value reserved for pull for this because pull doesn't need it.
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    (*cl).ps.fd.forcePowerDebounce[FP_PULL as usize] = level_time + 100;
                }

                if (*cl).ps.fd.forcePower < 1 {
                    WP_ForcePowerStop(ctx, self_, FP_GRIP);
                    return;
                }

                DoGripAction(ctx, self_, forcePower);
            }
            FP_LEVITATION => {
                if (*cl).ps.groundEntityNum != ENTITYNUM_NONE
                    && (*cl).ps.fd.forceJumpZStart == (0) as f32
                {
                    //done with jump
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }
            }
            FP_RAGE => {
                if ctx.entity(self_).health < 1 {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }
                if (*cl).ps.forceRageDrainTime < level_time {
                    let mut addTime = 400;

                    ctx.entity_mut(self_).health -= 2;

                    if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_1 {
                        addTime = 150;
                    } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_2 {
                        addTime = 300;
                    } else if (*cl).ps.fd.forcePowerLevel[FP_RAGE as usize] == FORCE_LEVEL_3 {
                        addTime = 450;
                    }
                    (*cl).ps.forceRageDrainTime = level_time + addTime;
                }

                if ctx.entity(self_).health < 1 {
                    ctx.entity_mut(self_).health = 1;
                    WP_ForcePowerStop(ctx, self_, forcePower);
                }

                (*cl).ps.stats[STAT_HEALTH as usize] = ctx.entity(self_).health;
            }
            FP_DRAIN => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forcePowerLevel[FP_DRAIN as usize] > FORCE_LEVEL_1 {
                    //higher than level 1
                    if ((*cmd).buttons & BUTTON_FORCE_DRAIN) != 0
                        || (((*cmd).buttons & BUTTON_FORCEPOWER) != 0
                            && (*cl).ps.fd.forcePowerSelected == FP_DRAIN)
                    {
                        //holding it keeps it going
                        (*cl).ps.fd.forcePowerDuration[FP_DRAIN as usize] = level_time + 500;
                    }
                }
                // OVERRIDEFIXME
                if WP_ForcePowerAvailable(ctx, self_, forcePower, 0) == 0
                    || (*cl).ps.fd.forcePowerDuration[FP_DRAIN as usize] < level_time
                    || (*cl).ps.fd.forcePower < 25
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                } else {
                    ForceShootDrain(ctx, self_);
                }
            }
            FP_LIGHTNING => {
                if (*cl).ps.forceHandExtend != HANDEXTEND_FORCE_HOLD as c_int {
                    //once hand starts to go in in animation, lightning should stop
                    WP_ForcePowerStop(ctx, self_, forcePower);
                    return;
                }

                if (*cl).ps.fd.forcePowerLevel[FP_LIGHTNING as usize] > FORCE_LEVEL_1 {
                    //higher than level 1
                    if ((*cmd).buttons & BUTTON_FORCE_LIGHTNING) != 0
                        || (((*cmd).buttons & BUTTON_FORCEPOWER) != 0
                            && (*cl).ps.fd.forcePowerSelected == FP_LIGHTNING)
                    {
                        //holding it keeps it going
                        (*cl).ps.fd.forcePowerDuration[FP_LIGHTNING as usize] = level_time + 500;
                    }
                }
                // OVERRIDEFIXME
                if WP_ForcePowerAvailable(ctx, self_, forcePower, 0) == 0
                    || (*cl).ps.fd.forcePowerDuration[FP_LIGHTNING as usize] < level_time
                    || (*cl).ps.fd.forcePower < 25
                {
                    WP_ForcePowerStop(ctx, self_, forcePower);
                } else {
                    ForceShootLightning(ctx, self_);
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 0);
                }
            }
            FP_TELEPATHY => {
                if (*cl).holdingObjectiveItem >= MAX_CLIENTS as c_int
                    && (*cl).holdingObjectiveItem < ENTITYNUM_WORLD
                    && ctx.world.g_entities[(*cl).holdingObjectiveItem as usize].genericValue15 != 0
                {
                    //if force hindered can't mindtrick whilst carrying a siege item
                    WP_ForcePowerStop(ctx, self_, FP_TELEPATHY);
                } else {
                    WP_UpdateMindtrickEnts(ctx, self_);
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            FP_PROTECT => {
                if (*cl).ps.fd.forcePowerDebounce[forcePower as usize] < level_time {
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    if (*cl).ps.fd.forcePower < 1 {
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }

                    (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = level_time + 300;
                }
            }
            FP_ABSORB => {
                if (*cl).ps.fd.forcePowerDebounce[forcePower as usize] < level_time {
                    BG_ForcePowerDrain(&mut (*cl).ps, forcePower, 1);
                    if (*cl).ps.fd.forcePower < 1 {
                        WP_ForcePowerStop(ctx, self_, forcePower);
                    }

                    (*cl).ps.fd.forcePowerDebounce[forcePower as usize] = level_time + 600;
                }
            }
            _ => {}
        }
    }
}

/// Raven `WP_DoSpecificPower`.
///
/// Source: `oracle/codemp/game/w_force.c:4508-4671`
pub fn WP_DoSpecificPower(
    ctx: &mut GameContext,
    self_: EntityId,
    ucmd: *mut usercmd_t,
    forcepower: forcePowers_t,
) -> c_int {
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); `ucmd` is a raw seam param.
        let cl = ctx.entity(self_).client;

        let mut powerSucceeded = 1;

        // OVERRIDEFIXME
        if WP_ForcePowerAvailable(ctx, self_, forcepower, 0) == 0 {
            return 0;
        }

        match forcepower {
            FP_HEAL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceHeal(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_LEVITATION => {
                //if leave the ground by some other means, cancel the force jump
                if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
                    (*cl).ps.fd.forceJumpCharge = 0.0;
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                } else {
                    //still on ground, so jump
                    ForceJump(ctx, self_, ucmd);
                }
            }
            FP_SPEED => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceSpeed(ctx, self_, 0);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_GRIP => {
                if (*cl).ps.fd.forceGripEntityNum == ENTITYNUM_NONE {
                    ForceGrip(ctx, self_);
                }

                if (*cl).ps.fd.forceGripEntityNum != ENTITYNUM_NONE {
                    if (*cl).ps.fd.forcePowersActive & (1 << FP_GRIP) == 0 {
                        WP_ForcePowerStart(ctx, self_, FP_GRIP, 0);
                        BG_ForcePowerDrain(&mut (*cl).ps, FP_GRIP, GRIP_DRAIN_AMOUNT);
                    }
                } else {
                    powerSucceeded = 0;
                }
            }
            FP_LIGHTNING => {
                ForceLightning(ctx, self_);
            }
            FP_PUSH => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if !((*cl).ps.fd.forceButtonNeedRelease != 0
                    && (ctx.entity(self_).r.svFlags & SVF_BOT) == 0)
                {
                    ForceThrow(ctx, self_, qfalse);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_PULL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceThrow(ctx, self_, qtrue);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TELEPATHY => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTelepathy(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_RAGE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceRage(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_PROTECT => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceProtect(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_ABSORB => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceAbsorb(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TEAM_HEAL => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTeamHeal(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_TEAM_FORCE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceTeamForceReplenish(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_DRAIN => {
                ForceDrain(ctx, self_);
            }
            FP_SEE => {
                powerSucceeded = 0; //always 0 for nonhold powers
                if (*cl).ps.fd.forceButtonNeedRelease == 0 {
                    ForceSeeing(ctx, self_);
                    (*cl).ps.fd.forceButtonNeedRelease = 1;
                }
            }
            FP_SABER_OFFENSE => {}
            FP_SABER_DEFENSE => {}
            FP_SABERTHROW => {}
            _ => {}
        }

        powerSucceeded
    }
}

/// Raven `FindGenericEnemyIndex`.
///
/// Source: `oracle/codemp/game/w_force.c:4673-4709`
pub fn FindGenericEnemyIndex(ctx: &mut GameContext, self_: EntityId) {
    //Find another client that would be considered a threat.
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); `self_`'s number is
        // invariant — hoist as a local.
        let scl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let mut besten: Option<EntityId> = None;
        let mut blen: f32 = 99999999.0;

        let mut i: c_int = 0;
        while i < MAX_CLIENTS as c_int {
            let ent_id2 = EntityId(i as u32);
            let ecl = ctx.entity(ent_id2).client;

            if !ecl.is_null()
                && ctx.entity(ent_id2).s.number != self_number
                && ctx.entity(ent_id2).health > 0
                && OnSameTeam(ctx, Some(self_), Some(ent_id2)) == 0
                && (*ecl).ps.pm_type != PM_INTERMISSION as c_int
                && (*ecl).ps.pm_type != PM_SPECTATOR as c_int
            {
                let a: vec3_t = [
                    (*ecl).ps.origin[0] - (*scl).ps.origin[0],
                    (*ecl).ps.origin[1] - (*scl).ps.origin[1],
                    (*ecl).ps.origin[2] - (*scl).ps.origin[2],
                ];
                let tlen = VectorLength(a);

                if tlen < blen
                    && InFront(
                        (*ecl).ps.origin,
                        (*scl).ps.origin,
                        (*scl).ps.viewangles,
                        0.8,
                    ) != 0
                    && OrgVisible(ctx, (*scl).ps.origin, (*ecl).ps.origin, self_number) != 0
                {
                    blen = tlen;
                    besten = Some(ent_id2);
                }
            }

            i += 1;
        }

        let besten = match besten {
            Some(b) => b,
            None => return,
        };

        (*scl).ps.genericEnemyIndex = ctx.entity(besten).s.number;
    }
}

/// Raven `SeekerDroneUpdate`.
///
/// Source: `oracle/codemp/game/w_force.c:4711-4868`
pub fn SeekerDroneUpdate(ctx: &mut GameContext, self_: EntityId) {
    unsafe {
        // FLAG: gclient_t derefs stay raw (recipe 2b); `self_`'s number is
        // invariant — hoist as a local.
        let cl = ctx.entity(self_).client;
        let self_number = ctx.entity(self_).s.number;
        let level_time = ctx.world.level.time;

        if (*cl).ps.eFlags & EF_SEEKERDRONE == 0 {
            (*cl).ps.genericEnemyIndex = -1;
            return;
        }

        if ctx.entity(self_).health < 1 {
            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            // Raven `float angle`: the orbit angle narrows to f32, then re-widens
            // for libm's `double cos`/`sin`.
            let angle = (((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0) as f32;
            let dir: vec3_t = [
                ((angle as f64).cos() * 20.0) as f32,
                ((angle as f64).sin() * 20.0) as f32,
                ((angle as f64).cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            let mut a: vec3_t = [0.0; 3];
            a[ROLL] = 0.0;
            a[YAW] = 0.0;
            a[PITCH] = 1.0;

            G_PlayEffect(EFFECT_SPARK_EXPLOSION as c_int, org, a);

            (*cl).ps.eFlags -= EF_SEEKERDRONE;
            (*cl).ps.genericEnemyIndex = -1;

            return;
        }

        if (*cl).ps.droneExistTime >= (level_time) as f32
            && (*cl).ps.droneExistTime < (level_time + 5000) as f32
        {
            (*cl).ps.genericEnemyIndex = (1024.0 + (*cl).ps.droneExistTime) as c_int;
            if (*cl).ps.droneFireTime < (level_time) as f32 {
                let snd = std::ffi::CString::new("sound/weapons/laser_trap/warning.wav").unwrap();
                G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(snd.as_ptr()));
                (*cl).ps.droneFireTime = (level_time + 100) as f32;
            }
            return;
        } else if (*cl).ps.droneExistTime < (level_time) as f32 {
            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            let mut prefig = ((*cl).ps.droneExistTime - level_time as f32) / 80.0;

            if prefig > 55.0 {
                prefig = 55.0;
            } else if prefig < 1.0 {
                prefig = 1.0;
            }

            elevated[2] -= 55.0 - prefig;

            // Raven `float angle`: the orbit angle narrows to f32, then re-widens
            // for libm's `double cos`/`sin`.
            let angle = (((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0) as f32;
            let dir: vec3_t = [
                ((angle as f64).cos() * 20.0) as f32,
                ((angle as f64).sin() * 20.0) as f32,
                ((angle as f64).cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            let mut a: vec3_t = [0.0; 3];
            a[ROLL] = 0.0;
            a[YAW] = 0.0;
            a[PITCH] = 1.0;

            G_PlayEffect(EFFECT_SPARK_EXPLOSION as c_int, org, a);

            (*cl).ps.eFlags -= EF_SEEKERDRONE;
            (*cl).ps.genericEnemyIndex = -1;

            return;
        }

        if (*cl).ps.genericEnemyIndex == -1 {
            (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
        }

        if (*cl).ps.genericEnemyIndex != ENTITYNUM_NONE && (*cl).ps.genericEnemyIndex != -1 {
            let en_id = EntityId((*cl).ps.genericEnemyIndex as u32);

            if ctx.entity(en_id).client.is_null() {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if ctx.entity(en_id).s.number == self_number {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if ctx.entity(en_id).health < 1 {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else if OnSameTeam(ctx, Some(self_), Some(en_id)) != 0 {
                (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
            } else {
                // FLAG: gclient_t deref stays raw (recipe 2b).
                let ecl = ctx.entity(en_id).client;
                if InFront((*ecl).ps.origin, (*cl).ps.origin, (*cl).ps.viewangles, 0.8) == 0 {
                    (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
                } else if OrgVisible(ctx, (*cl).ps.origin, (*ecl).ps.origin, self_number) == 0 {
                    (*cl).ps.genericEnemyIndex = ENTITYNUM_NONE;
                }
            }
        }

        if (*cl).ps.genericEnemyIndex == ENTITYNUM_NONE || (*cl).ps.genericEnemyIndex == -1 {
            FindGenericEnemyIndex(ctx, self_);
        }

        if (*cl).ps.genericEnemyIndex != ENTITYNUM_NONE && (*cl).ps.genericEnemyIndex != -1 {
            let en_id = EntityId((*cl).ps.genericEnemyIndex as u32);

            let mut elevated: vec3_t = (*cl).ps.origin;
            elevated[2] += 40.0;

            // Raven `float angle`: the orbit angle narrows to f32, then re-widens
            // for libm's `double cos`/`sin`.
            let angle = (((level_time / 12) & 255) as f64 * (M_PI * 2.0) / 255.0) as f32;
            let dir: vec3_t = [
                ((angle as f64).cos() * 20.0) as f32,
                ((angle as f64).sin() * 20.0) as f32,
                ((angle as f64).cos() * 5.0) as f32,
            ];
            let org: vec3_t = [
                elevated[0] + dir[0],
                elevated[1] + dir[1],
                elevated[2] + dir[2],
            ];

            //org is now where the thing should be client-side because it uses the same time-based offset
            if (*cl).ps.droneFireTime < (level_time) as f32 {
                // FLAG: gclient_t deref stays raw (recipe 2b).
                let ecl = ctx.entity(en_id).client;
                let mut tr: trace_t = core::mem::zeroed();
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &org as *const vec3_t,
                        core::ptr::null(),
                        core::ptr::null(),
                        &(*ecl).ps.origin as *const vec3_t,
                        -1,
                        MASK_SOLID,
                    ),
                );

                if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
                    let mut endir: vec3_t = [
                        (*ecl).ps.origin[0] - org[0],
                        (*ecl).ps.origin[1] - org[1],
                        (*ecl).ps.origin[2] - org[2],
                    ];
                    VectorNormalize(&mut endir);

                    WP_FireGenericBlasterMissile(
                        ctx,
                        self_,
                        org,
                        endir,
                        0,
                        15,
                        2000,
                        MOD_BLASTER as c_int,
                    );
                    let snd = std::ffi::CString::new("sound/weapons/bryar/fire.wav").unwrap();
                    G_SoundAtLoc(ctx, org, CHAN_WEAPON, G_SoundIndex(snd.as_ptr()));

                    (*cl).ps.droneFireTime =
                        (level_time + ctx.world.bg_state.rng.Q_irand(400, 700)) as f32;
                }
            }
        }
    }
}

/// Raven `HolocronUpdate`.
///
/// Source: `oracle/codemp/game/w_force.c:4870-4956`
pub fn HolocronUpdate(ctx: &mut GameContext, self_: EntityId) {
    //keep holocron status updated in holocron mode
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;

        let mut noHRank = 0;

        if noHRank < FORCE_LEVEL_0 {
            noHRank = FORCE_LEVEL_0;
        }
        if noHRank > FORCE_LEVEL_3 {
            noHRank = FORCE_LEVEL_3;
        }

        trap::Cvar_Update(
            ctx.engine,
            GCvarUpdateArgs::new(&mut ctx.world.cvars.g_MaxHolocronCarry as *mut vmCvar_t),
        );

        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if (*cl).ps.holocronsCarried[i as usize] != (0) as f32 {
                //carrying it, make sure we have the power
                (*cl).ps.holocronBits |= 1 << i;
                (*cl).ps.fd.forcePowersKnown |= 1 << i;
                (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;
            } else {
                //otherwise, make sure the power is cleared from us
                (*cl).ps.fd.forcePowerLevel[i as usize] = 0;
                if (*cl).ps.holocronBits & (1 << i) != 0 {
                    (*cl).ps.holocronBits -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0
                    && i != (FP_LEVITATION) as usize
                    && i != (FP_SABER_OFFENSE) as usize
                {
                    (*cl).ps.fd.forcePowersKnown -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0
                    && i != (FP_LEVITATION) as usize
                    && i != (FP_SABER_OFFENSE) as usize
                {
                    WP_ForcePowerStop(ctx, self_, (i) as i32);
                }

                if i == (FP_LEVITATION) as usize {
                    if noHRank >= FORCE_LEVEL_1 {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = noHRank;
                    } else {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                    }
                } else if i == (FP_SABER_OFFENSE) as usize {
                    (*cl).ps.fd.forcePowersKnown |= 1 << i;

                    if noHRank >= FORCE_LEVEL_1 {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = noHRank;
                    } else {
                        (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                    }
                } else {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
                }
            }

            i += 1;
        }

        if HasSetSaberOnly(ctx) != 0 {
            //if saberonly, we get these powers no matter what (still need the holocrons for level 3)
            if (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] < FORCE_LEVEL_1 {
                (*cl).ps.fd.forcePowerLevel[FP_SABER_OFFENSE as usize] = FORCE_LEVEL_1;
            }
            if (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] < FORCE_LEVEL_1 {
                (*cl).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize] = FORCE_LEVEL_1;
            }
        }
    }
}

/// Raven `JediMasterUpdate`.
///
/// Source: `oracle/codemp/game/w_force.c:4958-5011`
pub fn JediMasterUpdate(ctx: &mut GameContext, self_: EntityId) {
    //keep jedi master status updated for JM gametype
    unsafe {
        // FLAG: gclient_t deref stays raw (pool clients on NPC entities, recipe 2b).
        let cl = ctx.entity(self_).client;

        trap::Cvar_Update(
            ctx.engine,
            GCvarUpdateArgs::new(&mut ctx.world.cvars.g_MaxHolocronCarry as *mut vmCvar_t),
        );

        let mut i = 0;
        while i < NUM_FORCE_POWERS {
            if (*cl).ps.isJediMaster != 0 {
                (*cl).ps.fd.forcePowersKnown |= 1 << i;
                (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_3;

                if i == (FP_TEAM_HEAL) as usize
                    || i == (FP_TEAM_FORCE) as usize
                    || i == (FP_DRAIN) as usize
                    || i == (FP_ABSORB) as usize
                {
                    //team powers are useless in JM, absorb is too, drain relatively useless
                    (*cl).ps.fd.forcePowersKnown &= !(1 << i);
                    (*cl).ps.fd.forcePowerLevel[i as usize] = 0;
                }

                if i == (FP_TELEPATHY) as usize {
                    //level 3 mindtrick lets the JM hide too much
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_2;
                }
            } else {
                if (*cl).ps.fd.forcePowersKnown & (1 << i) != 0 && i != (FP_LEVITATION) as usize {
                    (*cl).ps.fd.forcePowersKnown -= 1 << i;
                }

                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 && i != (FP_LEVITATION) as usize {
                    WP_ForcePowerStop(ctx, self_, (i) as i32);
                }

                if i == (FP_LEVITATION) as usize {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_1;
                } else {
                    (*cl).ps.fd.forcePowerLevel[i as usize] = FORCE_LEVEL_0;
                }
            }

            i += 1;
        }
    }
}

/// Raven `WP_HasForcePowers` — does `ps` know any non-trivial force power?
///
/// Source: `oracle/codemp/game/w_force.c:5013-5034`
pub fn WP_HasForcePowers(ps: *const playerState_t) -> qboolean {
    unsafe {
        if !ps.is_null() {
            let mut i = 0;
            while i < NUM_FORCE_POWERS {
                if i == (FP_LEVITATION) as usize {
                    if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_1 {
                        return qtrue;
                    }
                } else if (*ps).fd.forcePowerLevel[i as usize] > FORCE_LEVEL_0 {
                    return qtrue;
                }
                i += 1;
            }
        }
        qfalse
    }
}

/// Raven `G_SpecialRollGetup`.
///
/// Source: `oracle/codemp/game/w_force.c:5037-5092`
pub fn G_SpecialRollGetup(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    unsafe {
        // FLAG: gclient_t deref stays raw (recipe 2b); `cmd` points into the raw
        // client's `pers.cmd`.
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let mut rolled: qboolean = qfalse;
        let cmd = &mut (*cl).pers.cmd as *mut usercmd_t;

        if (*cl).pers.cmd.rightmove > 0 && (*cl).pers.cmd.forwardmove == 0 {
            G_SetAnim(
                ctx,
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_R as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove < 0 && (*cl).pers.cmd.forwardmove == 0 {
            G_SetAnim(
                ctx,
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_L as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove == 0 && (*cl).pers.cmd.forwardmove > 0 {
            G_SetAnim(
                ctx,
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_F as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.rightmove == 0 && (*cl).pers.cmd.forwardmove < 0 {
            G_SetAnim(
                ctx,
                self_,
                cmd,
                SETANIM_BOTH,
                BOTH_GETUP_BROLL_B as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                0,
            );
            rolled = qtrue;
        } else if (*cl).pers.cmd.upmove != 0 {
            G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);
            (*cl).ps.forceDodgeAnim = 2;
            (*cl).ps.forceHandExtendTime = level_time + 500;
        }

        if rolled != 0 {
            let snd = std::ffi::CString::new("*jump1.wav").unwrap();
            G_EntitySound(ctx, self_, CHAN_VOICE, G_SoundIndex(snd.as_ptr()));
        }

        rolled
    }
}

/// Raven `WP_ForcePowersUpdate`.
///
/// Source: `oracle/codemp/game/w_force.c:5094-5671`
pub fn WP_ForcePowersUpdate(ctx: &mut GameContext, self_: Option<EntityId>, ucmd: *mut usercmd_t) {
    unsafe {
        let mut usingForce = qfalse;
        let mut prepower: c_int = 0;

        // FLAG: gclient_t deref stays raw (recipe 2b); `ucmd` is a raw seam param.
        // `self_` null OR null-client returns, matching Raven's paired guard.
        let self_ = match self_ {
            Some(s) if !ctx.entity(s).client.is_null() => s,
            _ => return,
        };
        let cl = ctx.entity(self_).client;
        let self_svflags = ctx.entity(self_).r.svFlags;
        let level_time = ctx.world.level.time;
        let gametype = ctx.world.cvars.g_gametype.integer;

        if (*cl).ps.pm_flags & PMF_FOLLOW != 0 {
            //not a "real" game client, it's a spectator following someone
            return;
        }
        if (*cl).sess.sessionTeam == TEAM_SPECTATOR {
            return;
        }

        //The stance in relation to power level is no longer applicable with the crazy new akimbo/staff stances.
        if (*cl).ps.fd.saberAnimLevel == 0 {
            (*cl).ps.fd.saberAnimLevel = FORCE_LEVEL_1 as c_int;
        }

        if gametype != GT_SIEGE as c_int {
            if (*cl).ps.fd.forcePowersKnown & (1 << FP_LEVITATION) == 0 {
                (*cl).ps.fd.forcePowersKnown |= 1 << FP_LEVITATION;
            }

            if (*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] < FORCE_LEVEL_1 as c_int {
                (*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize] = FORCE_LEVEL_1 as c_int;
            }
        }

        if (*cl).ps.fd.forcePowerSelected < 0 {
            //bad
            (*cl).ps.fd.forcePowerSelected = 0;
        }

        if ((*cl).sess.selectedFP != (*cl).ps.fd.forcePowerSelected
            || (*cl).sess.saberLevel != (*cl).ps.fd.saberAnimLevel)
            && self_svflags & SVF_BOT == 0
        {
            if (*cl).sess.updateUITime < level_time {
                //a bit hackish, but we don't want the client to flood with userinfo updates if they rapidly cycle
                //through their force powers or saber attack levels
                (*cl).sess.selectedFP = (*cl).ps.fd.forcePowerSelected;
                (*cl).sess.saberLevel = (*cl).ps.fd.saberAnimLevel;
            }
        }

        if ctx.world.globals.g_LastFrameTime == 0 {
            ctx.world.globals.g_LastFrameTime = level_time;
        }

        if (*cl).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int {
            (*cl).ps.zoomFov = 0.0;
            (*cl).ps.zoomMode = 0;
            (*cl).ps.zoomLocked = qfalse;
            (*cl).ps.zoomTime = 0;
        }

        if (*cl).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
            && (*cl).ps.forceHandExtendTime >= level_time
        {
            (*cl).ps.saberMove = 0;
            (*cl).ps.saberBlocking = 0;
            (*cl).ps.saberBlocked = 0;
            (*cl).ps.weaponTime = 0;
            (*cl).ps.weaponstate = WEAPON_READY as c_int;
        } else if (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int
            && (*cl).ps.forceHandExtendTime < level_time
        {
            if (*cl).ps.forceHandExtend == HANDEXTEND_KNOCKDOWN as c_int
                && (*cl).ps.forceDodgeAnim == 0
            {
                if ctx.entity(self_).health < 1 || (*cl).ps.eFlags & EF_DEAD != 0 {
                    (*cl).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                } else if G_SpecialRollGetup(ctx, self_) != 0 {
                    (*cl).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                } else {
                    //hmm.. ok.. no more getting up on your own, you've gotta push something, unless..
                    if (level_time - (*cl).ps.forceHandExtendTime) > 4000 {
                        //4 seconds elapsed, I guess they're too dumb to push something to get up!
                        if (*cl).pers.cmd.upmove != 0
                            && (*cl).ps.fd.forcePowerLevel[FP_LEVITATION as usize]
                                > FORCE_LEVEL_1 as c_int
                        {
                            //force getup
                            G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);
                            (*cl).ps.forceDodgeAnim = 2;
                            (*cl).ps.forceHandExtendTime = level_time + 500;
                        //self->client->ps.velocity[2] = 400;
                        } else if (*cl).ps.quickerGetup != 0 {
                            G_EntitySound(
                                ctx,
                                self_,
                                CHAN_VOICE,
                                G_SoundIndex(cstr("*jump1.wav").as_ptr()),
                            );
                            (*cl).ps.forceDodgeAnim = 3;
                            (*cl).ps.forceHandExtendTime = level_time + 500;
                            (*cl).ps.velocity[2] = 300.0;
                        } else {
                            (*cl).ps.forceDodgeAnim = 1;
                            (*cl).ps.forceHandExtendTime = level_time + 1000;
                        }
                    }
                }
                (*cl).ps.quickerGetup = qfalse;
            } else if (*cl).ps.forceHandExtend == HANDEXTEND_POSTTHROWN as c_int {
                if ctx.entity(self_).health < 1 || (*cl).ps.eFlags & EF_DEAD != 0 {
                    (*cl).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                } else if (*cl).ps.groundEntityNum != ENTITYNUM_NONE && (*cl).ps.forceDodgeAnim == 0
                {
                    (*cl).ps.forceDodgeAnim = 1;
                    (*cl).ps.forceHandExtendTime = level_time + 1000;
                    G_EntitySound(
                        ctx,
                        self_,
                        CHAN_VOICE,
                        G_SoundIndex(cstr("*jump1.wav").as_ptr()),
                    );
                    (*cl).ps.velocity[2] = 100.0;
                } else if (*cl).ps.forceDodgeAnim == 0 {
                    (*cl).ps.forceHandExtendTime = level_time + 100;
                } else {
                    (*cl).ps.forceHandExtend = HANDEXTEND_WEAPONREADY as c_int;
                }
            } else {
                (*cl).ps.forceHandExtend = HANDEXTEND_WEAPONREADY as c_int;
            }
        }

        if gametype == GT_HOLOCRON as c_int {
            HolocronUpdate(ctx, self_);
        }
        if gametype == GT_JEDIMASTER as c_int {
            JediMasterUpdate(ctx, self_);
        }

        SeekerDroneUpdate(ctx, self_);

        if (*cl).ps.powerups[PW_FORCE_BOON as usize] != 0 {
            prepower = (*cl).ps.fd.forcePower;
        }

        if BG_HasYsalamiri(gametype, &mut (*cl).ps) != 0
            || (*cl).ps.fd.forceDeactivateAll != 0
            || (*cl).tempSpectate >= level_time
        {
            //has ysalamiri.. or we want to forcefully stop all his active powers
            for i in 0..NUM_FORCE_POWERS as usize {
                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 && i != FP_LEVITATION as usize {
                    WP_ForcePowerStop(ctx, self_, i as forcePowers_t);
                }
            }

            if (*cl).tempSpectate >= level_time {
                (*cl).ps.fd.forcePower = 100;
                (*cl).ps.fd.forceRageRecoveryTime = 0;
            }

            (*cl).ps.fd.forceDeactivateAll = 0;

            if (*cl).ps.fd.forceJumpCharge != 0.0 {
                G_MuteSound(
                    ctx,
                    (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                    CHAN_VOICE,
                );
                (*cl).ps.fd.forceJumpCharge = 0.0;
            }
        } else {
            //otherwise just do a check through them all to see if they need to be stopped for any reason.
            for i in 0..NUM_FORCE_POWERS as usize {
                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0
                    && i != FP_LEVITATION as usize
                    && BG_CanUseFPNow(gametype, &mut (*cl).ps, level_time, i as forcePowers_t) == 0
                {
                    WP_ForcePowerStop(ctx, self_, i as forcePowers_t);
                }
            }
        }

        if (*cl).ps.powerups[PW_FORCE_ENLIGHTENED_LIGHT as usize] != 0
            || (*cl).ps.powerups[PW_FORCE_ENLIGHTENED_DARK as usize] != 0
        {
            //enlightenment
            if (*cl).ps.fd.forceUsingAdded == 0 {
                for i in 0..NUM_FORCE_POWERS as usize {
                    (*cl).ps.fd.forcePowerBaseLevel[i] = (*cl).ps.fd.forcePowerLevel[i];

                    if forcePowerDarkLight[i] == 0
                        || (*cl).ps.fd.forceSide == forcePowerDarkLight[i]
                    {
                        (*cl).ps.fd.forcePowerLevel[i] = FORCE_LEVEL_3 as c_int;
                        (*cl).ps.fd.forcePowersKnown |= 1 << i;
                    }
                }

                (*cl).ps.fd.forceUsingAdded = 1;
            }
        } else if (*cl).ps.fd.forceUsingAdded != 0 {
            //we don't have enlightenment but we're still using enlightened powers, so clear them back to how they should be.
            for i in 0..NUM_FORCE_POWERS as usize {
                (*cl).ps.fd.forcePowerLevel[i] = (*cl).ps.fd.forcePowerBaseLevel[i];
                if (*cl).ps.fd.forcePowerLevel[i] == 0 {
                    if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 {
                        WP_ForcePowerStop(ctx, self_, i as forcePowers_t);
                    }
                    (*cl).ps.fd.forcePowersKnown &= !(1 << i);
                }
            }

            (*cl).ps.fd.forceUsingAdded = 0;
        }

        if (*cl).ps.fd.forcePowersActive & (1 << FP_TELEPATHY) == 0 {
            //clear the mindtrick index values
            (*cl).ps.fd.forceMindtrickTargetIndex = 0;
            (*cl).ps.fd.forceMindtrickTargetIndex2 = 0;
            (*cl).ps.fd.forceMindtrickTargetIndex3 = 0;
            (*cl).ps.fd.forceMindtrickTargetIndex4 = 0;
        }

        if ctx.entity(self_).health < 1 {
            (*cl).ps.fd.forceGripBeingGripped = 0.0;
        }

        if (*cl).ps.fd.forceGripBeingGripped > level_time as f32 {
            (*cl).ps.fd.forceGripCripple = 1;

            //keep the saber off during this period
            if (*cl).ps.weapon == WP_SABER as c_int && (*cl).ps.saberHolstered == 0 {
                Cmd_ToggleSaber_f(ctx, self_);
            }
        } else {
            (*cl).ps.fd.forceGripCripple = 0;
        }

        if (*cl).ps.fd.forceJumpSound != 0 {
            G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEJUMP as c_int);
            (*cl).ps.fd.forceJumpSound = 0;
        }

        if (*cl).ps.fd.forceGripCripple != 0 {
            if ((*cl).ps.fd.forceGripSoundTime as c_int) < level_time {
                G_PreDefSound(ctx, (*cl).ps.origin, PDSOUND_FORCEGRIP as c_int);
                (*cl).ps.fd.forceGripSoundTime = (level_time + 1000) as f32;
            }
        }

        if (*cl).ps.fd.forcePowersActive & (1 << FP_SPEED) != 0 {
            (*cl).ps.powerups[PW_SPEED as usize] = level_time + 100;
        }

        let mut dead = false;
        if ctx.entity(self_).health <= 0 {
            //if dead, deactivate any active force powers
            for i in 0..NUM_FORCE_POWERS as usize {
                if (*cl).ps.fd.forcePowerDuration[i] != 0
                    || (*cl).ps.fd.forcePowersActive & (1 << i) != 0
                {
                    WP_ForcePowerStop(ctx, self_, i as forcePowers_t);
                    (*cl).ps.fd.forcePowerDuration[i] = 0;
                }
            }
            dead = true;
        }

        if !dead {
            if (*cl).ps.groundEntityNum != ENTITYNUM_NONE {
                (*cl).fjDidJump = qfalse;
            }

            if (*cl).ps.fd.forceJumpCharge != 0.0
                && (*cl).ps.groundEntityNum == ENTITYNUM_NONE
                && (*cl).fjDidJump != 0
            {
                //this was for the "charge" jump method... I guess
                if (*ucmd).upmove < 10
                    && ((*ucmd).buttons & BUTTON_FORCEPOWER == 0
                        || (*cl).ps.fd.forcePowerSelected != FP_LEVITATION)
                {
                    G_MuteSound(
                        ctx,
                        (*cl).ps.fd.killSoundEntIndex[(TRACK_CHANNEL_1 as c_int - 50) as usize],
                        CHAN_VOICE,
                    );
                    (*cl).ps.fd.forceJumpCharge = 0.0;
                }
            }
            // The two `else if` charge-jump arms at w_force.c:5444-5454 are
            // `#ifndef METROID_JUMP` — dead in retail (defined at :7), not ported.

            if (*cl).ps.pm_flags & PMF_JUMP_HELD == 0 && (*cl).ps.fd.forceJumpCharge != 0.0 {
                if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                    || (*cl).ps.fd.forcePowerSelected != FP_LEVITATION
                {
                    if WP_DoSpecificPower(ctx, self_, ucmd, FP_LEVITATION) != 0 {
                        usingForce = qtrue;
                    }
                }
            }

            if (*ucmd).buttons & BUTTON_FORCEGRIP != 0 {
                //grip is one of the powers with its own button.. if it's held, call the specific grip power function.
                if WP_DoSpecificPower(ctx, self_, ucmd, FP_GRIP) != 0 {
                    usingForce = qtrue;
                } else {
                    //don't let recharge even if the grip misses if the player still has the button down
                    usingForce = qtrue;
                }
            } else {
                //see if we're using it generically.. if not, stop.
                if (*cl).ps.fd.forcePowersActive & (1 << FP_GRIP) != 0 {
                    if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                        || (*cl).ps.fd.forcePowerSelected != FP_GRIP
                    {
                        WP_ForcePowerStop(ctx, self_, FP_GRIP);
                    }
                }
            }

            if (*ucmd).buttons & BUTTON_FORCE_LIGHTNING != 0 {
                //lightning
                WP_DoSpecificPower(ctx, self_, ucmd, FP_LIGHTNING);
                usingForce = qtrue;
            } else {
                //see if we're using it generically.. if not, stop.
                if (*cl).ps.fd.forcePowersActive & (1 << FP_LIGHTNING) != 0 {
                    if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                        || (*cl).ps.fd.forcePowerSelected != FP_LIGHTNING
                    {
                        WP_ForcePowerStop(ctx, self_, FP_LIGHTNING);
                    }
                }
            }

            if (*ucmd).buttons & BUTTON_FORCE_DRAIN != 0 {
                //drain
                WP_DoSpecificPower(ctx, self_, ucmd, FP_DRAIN);
                usingForce = qtrue;
            } else {
                //see if we're using it generically.. if not, stop.
                if (*cl).ps.fd.forcePowersActive & (1 << FP_DRAIN) != 0 {
                    if (*ucmd).buttons & BUTTON_FORCEPOWER == 0
                        || (*cl).ps.fd.forcePowerSelected != FP_DRAIN
                    {
                        WP_ForcePowerStop(ctx, self_, FP_DRAIN);
                    }
                }
            }

            if (*ucmd).buttons & BUTTON_FORCEPOWER != 0
                && BG_CanUseFPNow(
                    gametype,
                    &mut (*cl).ps,
                    level_time,
                    (*cl).ps.fd.forcePowerSelected,
                ) != 0
            {
                if (*cl).ps.fd.forcePowerSelected == FP_LEVITATION {
                    ForceJumpCharge(ctx, self_, ucmd);
                    usingForce = qtrue;
                } else if WP_DoSpecificPower(ctx, self_, ucmd, (*cl).ps.fd.forcePowerSelected) != 0
                {
                    usingForce = qtrue;
                } else if (*cl).ps.fd.forcePowerSelected == FP_GRIP {
                    usingForce = qtrue;
                }
            } else {
                (*cl).ps.fd.forceButtonNeedRelease = 0;
            }

            for i in 0..NUM_FORCE_POWERS as usize {
                if (*cl).ps.fd.forcePowerDuration[i] != 0 {
                    if (*cl).ps.fd.forcePowerDuration[i] < level_time {
                        if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 {
                            //turn it off
                            WP_ForcePowerStop(ctx, self_, i as forcePowers_t);
                        }
                        (*cl).ps.fd.forcePowerDuration[i] = 0;
                    }
                }
                if (*cl).ps.fd.forcePowersActive & (1 << i) != 0 {
                    usingForce = qtrue;
                    WP_ForcePowerRun(ctx, self_, i as forcePowers_t, ucmd);
                }
            }
            if (*cl).ps.saberInFlight != 0 && (*cl).ps.saberEntityNum != 0 {
                //don't regen force power while throwing saber
                if (*cl).ps.saberEntityNum < ENTITYNUM_NONE && (*cl).ps.saberEntityNum > 0 {
                    //player is 0
                    if ctx.world.g_entities[(*cl).ps.saberEntityNum as usize]
                        .s
                        .pos
                        .trType
                        == TR_LINEAR
                    {
                        //fell to the ground and we're trying to pull it back
                        usingForce = qtrue;
                    }
                }
            }
            if (*cl).ps.fd.forcePowersActive == 0
                || (*cl).ps.fd.forcePowersActive == (1 << FP_DRAIN)
            {
                //when not using the force, regenerate at 1 point per half second
                if (*cl).ps.saberInFlight == 0
                    && (*cl).ps.fd.forcePowerRegenDebounceTime < level_time
                    && ((*cl).ps.weapon != WP_SABER as c_int
                        || BG_SaberInSpecial((*cl).ps.saberMove) == 0)
                {
                    if gametype != GT_HOLOCRON as c_int
                        || ctx.world.cvars.g_MaxHolocronCarry.value != 0.0
                    {
                        if (*cl).ps.powerups[PW_FORCE_BOON as usize] != 0 {
                            WP_ForcePowerRegenerate(ctx.entity(self_), 6);
                        } else if (*cl).ps.isJediMaster != 0 && gametype == GT_JEDIMASTER as c_int {
                            WP_ForcePowerRegenerate(ctx.entity(self_), 4); //jedi master regenerates 4 times as fast
                        } else {
                            WP_ForcePowerRegenerate(ctx.entity(self_), 0);
                        }
                    } else {
                        //regenerate based on the number of holocrons carried
                        let mut holoregen: c_int = 0;
                        for holo in 0..NUM_FORCE_POWERS as usize {
                            if (*cl).ps.holocronsCarried[holo] != 0.0 {
                                holoregen += 1;
                            }
                        }

                        WP_ForcePowerRegenerate(ctx.entity(self_), holoregen);
                    }

                    if gametype == GT_SIEGE as c_int {
                        if (*cl).holdingObjectiveItem != 0
                            && ctx.world.g_entities[(*cl).holdingObjectiveItem as usize].inuse != 0
                            && ctx.world.g_entities[(*cl).holdingObjectiveItem as usize]
                                .genericValue15
                                != 0
                        {
                            //1 point per 7 seconds.. super slow
                            (*cl).ps.fd.forcePowerRegenDebounceTime = level_time + 7000;
                        } else if (*cl).siegeClass != -1
                            && (&ctx.world.bg_state.bgSiegeClasses)[(*cl).siegeClass as usize]
                                .classflags
                                & (1 << CFL_FASTFORCEREGEN as c_int)
                                != 0
                        {
                            //if this is siege and our player class has the fast force regen ability, then recharge with 1/5th the usual delay
                            // Raven's `0.2` is a double literal, so the multiply runs in f64.
                            (*cl).ps.fd.forcePowerRegenDebounceTime = level_time
                                + (ctx.world.cvars.g_forceRegenTime.integer as f64 * 0.2) as c_int;
                        } else {
                            (*cl).ps.fd.forcePowerRegenDebounceTime =
                                level_time + ctx.world.cvars.g_forceRegenTime.integer;
                        }
                    } else {
                        if gametype == GT_POWERDUEL as c_int
                            && (*cl).sess.duelTeam == DUELTEAM_LONE as c_int
                        {
                            if ctx.world.cvars.g_duel_fraglimit.integer != 0 {
                                // Raven's `0.6`/`.3` are double literals, so the whole
                                // multiply runs in f64; `(float)wins`/`(float)fraglimit`
                                // narrow to f32 before promoting into that f64 divide.
                                (*cl).ps.fd.forcePowerRegenDebounceTime = level_time
                                    + (ctx.world.cvars.g_forceRegenTime.integer as f64
                                        * (0.6
                                            + (0.3 * (*cl).sess.wins as f32 as f64
                                                / ctx.world.cvars.g_duel_fraglimit.integer as f32
                                                    as f64)))
                                        as c_int;
                            } else {
                                // Raven's `0.7` is a double literal, so the multiply runs in f64.
                                (*cl).ps.fd.forcePowerRegenDebounceTime = level_time
                                    + (ctx.world.cvars.g_forceRegenTime.integer as f64 * 0.7)
                                        as c_int;
                            }
                        } else {
                            (*cl).ps.fd.forcePowerRegenDebounceTime =
                                level_time + ctx.world.cvars.g_forceRegenTime.integer;
                        }
                    }
                }
            }
        }

        // powersetcheck:
        if prepower != 0 && (*cl).ps.fd.forcePower < prepower {
            let mut dif = (prepower - (*cl).ps.fd.forcePower) / 2;
            if dif < 1 {
                dif = 1;
            }

            (*cl).ps.fd.forcePower = prepower - dif;
        }
        let _ = usingForce;
    }
}

/// Raven `Jedi_DodgeEvasion`.
///
/// Source: `oracle/codemp/game/w_force.c:5673-5801`
pub fn Jedi_DodgeEvasion(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    shooter: Option<EntityId>,
    tr: *mut trace_t,
    hitLoc: c_int,
) -> qboolean {
    // `shooter` is unused in this body (Raven never reads it here).
    let _ = shooter;
    unsafe {
        let mut dodgeAnim: c_int = -1;

        // Raven: return unless `self_` non-null, has a client, and is alive.
        // FLAG: gclient_t deref stays raw (recipe 2b).
        let self_ = match self_ {
            Some(s) if !ctx.entity(s).client.is_null() && ctx.entity(s).health > 0 => s,
            _ => return qfalse,
        };
        let cl = ctx.entity(self_).client;
        let level_time = ctx.world.level.time;
        let g_forceDodge = ctx.world.cvars.g_forceDodge.integer;

        if g_forceDodge == 0 {
            return qfalse;
        }

        if g_forceDodge != 2 {
            if (*cl).ps.fd.forcePowersActive & (1 << FP_SEE) == 0 {
                return qfalse;
            }
        }

        if (*cl).ps.groundEntityNum == ENTITYNUM_NONE {
            //can't dodge in mid-air
            return qfalse;
        }

        if (*cl).ps.weaponTime > 0 || (*cl).ps.forceHandExtend != HANDEXTEND_NONE as c_int {
            //in some effect that stops me from moving on my own
            return qfalse;
        }

        if g_forceDodge == 2 {
            if (*cl).ps.fd.forcePowersActive != 0 {
                //for now just don't let us dodge if we're using a force power at all
                return qfalse;
            }
        }

        if g_forceDodge == 2 {
            if WP_ForcePowerUsable(ctx, self_, FP_SPEED) == 0 {
                //make sure we have it and have enough force power
                return qfalse;
            }
        }

        if g_forceDodge == 2 {
            if ctx.world.bg_state.rng.Q_irand(1, 7) > (*cl).ps.fd.forcePowerLevel[FP_SPEED as usize]
            {
                //more likely to fail on lower force speed level
                return qfalse;
            }
        } else {
            //We now dodge all the time, but only on level 3
            if (*cl).ps.fd.forcePowerLevel[FP_SEE as usize] < FORCE_LEVEL_3 {
                //more likely to fail on lower force sight level
                return qfalse;
            }
        }

        match hitLoc {
            HL_NONE => return qfalse,
            HL_FOOT_RT | HL_FOOT_LT | HL_LEG_RT | HL_LEG_LT => return qfalse,
            HL_BACK_RT => dodgeAnim = BOTH_DODGE_FL as c_int,
            HL_CHEST_RT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_BACK_LT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_CHEST_LT => dodgeAnim = BOTH_DODGE_FR as c_int,
            HL_BACK | HL_CHEST | HL_WAIST => dodgeAnim = BOTH_DODGE_FL as c_int,
            HL_ARM_RT | HL_HAND_RT => dodgeAnim = BOTH_DODGE_L as c_int,
            HL_ARM_LT | HL_HAND_LT => dodgeAnim = BOTH_DODGE_R as c_int,
            HL_HEAD => dodgeAnim = BOTH_DODGE_FL as c_int,
            _ => return qfalse,
        }

        if dodgeAnim != -1 {
            //Our own happy way of forcing an anim:
            (*cl).ps.forceHandExtend = HANDEXTEND_DODGE as c_int;
            (*cl).ps.forceDodgeAnim = dodgeAnim;
            (*cl).ps.forceHandExtendTime = level_time + 300;

            (*cl).ps.powerups[PW_SPEEDBURST as usize] = level_time + 100;

            if g_forceDodge == 2 {
                ForceSpeed(ctx, self_, 500);
            } else {
                let snd = std::ffi::CString::new("sound/weapons/force/speed.wav").unwrap();
                G_Sound(ctx, Some(self_), CHAN_BODY, G_SoundIndex(snd.as_ptr()));
            }
            return qtrue;
        }
        qfalse
    }
}
