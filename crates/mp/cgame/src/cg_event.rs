//! Port of `oracle/codemp/cgame/cg_event.c` — entity-event handling — obituaries, pickups, impacts. Functions land via the C5
//! transcription waves.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_uint};
use core::ptr::null_mut;

use mp_abi::ui::public::ui_menu_command_t::UIMENU_PLAYERCONFIG;

use mp_bg::bg_misc::{
    BG_CycleInven, BG_EvaluateTrajectory, BG_FindItemForHoldable, BG_GiveMeVectorFromMatrix,
};
use mp_bg::bg_panimate::BG_InKnockDownOnly;
use mp_bg::bg_saber::SFL2_NO_CLASH_FLARE;
use mp_bg::bg_saberLoad::WP_SaberBladeUseSecondBladeStyle;
use mp_bg::cstr_util::cstr_to_str;
use mp_bg::local::bg_customSiegeSoundNames;
use mp_bg::public::bg_itemlist::{bg_itemlist, bg_numItems};
use mp_bg::public::configstring::{CS_AMBIENT_SET, CS_EFFECTS, CS_PLAYERS, CS_SOUNDS};
use mp_bg::public::ctf_msg::ctfMsg_t;
use mp_bg::public::effect_types::effectTypes_t;
use mp_bg::public::entity_event::entity_event_t;
use mp_bg::public::entity_event::entity_event_t::EV_USE_ITEM0;
use mp_bg::public::entity_flags::{
    EF_ALT_FIRING, EF_DEAD, EF_JETPACK_ACTIVE, EF_PLAYER_EVENT, EF_SOUNDTRACKER,
};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_CTY, GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, GT_TEAM};
use mp_bg::public::gender::gender_t;
use mp_bg::public::global_team_sound::global_team_sound_t;
use mp_bg::public::holdable::{
    HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK, HI_MEDPAC,
    HI_MEDPAC_BIG, HI_NONE, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD,
};
use mp_bg::public::item_type::{IT_TEAM, IT_WEAPON};
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::pd_sounds::pdSounds_t;
use mp_bg::public::pers_enum::persEnum_t::{PERS_RANK, PERS_SCORE, PERS_TEAM};
use mp_bg::public::powerup::{PW_BATTLESUIT, PW_QUAD};
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t;
use mp_bg::public::viewheight::{DEFAULT_MAXS_2, DEFAULT_MINS_2};
use mp_bg::public::weaponstate::weaponstate_t;
use mp_bg::public::{team_t, RANK_TIED_FLAG, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::vehicles::vehicle_s::{Vehicle_t, MAX_VEHICLES, VEHICLE_BASE};
use mp_bg::weapons::weaponData;
use mp_bg::weapons::weapon_t::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_EMPLACED_GUN, WP_NONE, WP_NUM_WEAPONS, WP_REPEATER, WP_ROCKET_LAUNCHER,
    WP_SABER, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::ghoul2::bone_flags::{BONE_ANIM_BLEND, BONE_ANIM_OVERRIDE_FREEZE};
use mp_qshared::common::mp::qcommon::entityState_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS;
use mp_qshared::common::mp::qcommon::pm_flags::PMF_FOLLOW;
use mp_qshared::common::mp::qcommon::saber::saber_info::{saberInfo_t, MAX_SABERS};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::force_powers::{
    FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE,
};
use mp_qshared::shared::item_use_fail::itemUseFail_t;
use mp_qshared::shared::keycatch::KEYCATCH_UI;
use mp_qshared::shared::limits::MAX_VEH_WEAPONS;
use mp_qshared::shared::q_color::S_COLOR_WHITE;
use mp_qshared::shared::q_math::{
    _VectorMA, _VectorSubtract, vec3_origin, AngleVectors, ByteToDir, VectorClear, VectorLength,
    VectorNormalize, ROLL, YAW,
};
use mp_qshared::shared::surface_flags::{
    CONTENTS_SOLID, CONTENTS_TERRAIN, MASK_SOLID, MATERIAL_CANVAS, MATERIAL_CARPET, MATERIAL_DIRT,
    MATERIAL_FABRIC, MATERIAL_GRAVEL, MATERIAL_HOLLOWMETAL, MATERIAL_HOLLOWWOOD,
    MATERIAL_LONGGRASS, MATERIAL_MUD, MATERIAL_PLASTIC, MATERIAL_RUBBER, MATERIAL_SAND,
    MATERIAL_SHORTGRASS, MATERIAL_SNOW, MATERIAL_SOLIDMETAL, MATERIAL_SOLIDWOOD,
};
use mp_qshared::shared::trackchan::trackchan_t;
use mp_qshared::shared::{
    mdxaBone_t, qfalse, qtrue, vec3_t, Eorientations, BIGCHAR_WIDTH, CHAN_ANNOUNCER, CHAN_AUTO,
    CHAN_BODY, CHAN_LOCAL, CHAN_MENU1, CHAN_VOICE, CHAN_WEAPON, ENTITYNUM_NONE, ENTITYNUM_WORLD,
    GIANTCHAR_WIDTH, MASK_PLAYERSOLID, MAX_CLIENTS_I32, SCREEN_HEIGHT,
};
use mp_uishared::shared::display_state::DisplayState;
use native_string::{buf_to_string, string_to_latin1, Info_ValueForKey, Q_strncpyzBytes};

use crate::cg_draw::{
    showPowersName, CG_CenterPrint, CG_ChatBox_AddString, CG_InATST, CG_InFighter,
};
use crate::cg_effects::{
    CG_Chunks, CG_GlassShatter, CG_MiscModelExplosion, CG_ScorePlum, CG_TestLine,
};
use crate::cg_ents::{
    CG_Beam, CG_PlayDoorLoopSound, CG_PlayDoorSound, CG_S_AddRealLoopingSound,
    CG_S_StopLoopingSound, CG_SetEntitySoundPosition,
};
use crate::cg_main::{
    CG_ConfigString, CG_Error, CG_GetStringEdString, CG_Printf, CG_StartMusic, Com_Printf,
};
use crate::cg_players::{
    CG_AddGhoul2Mark, CG_CreateNPCClient, CG_CustomSound, CG_PlayerShieldHit, CG_ThereIsAMaster,
};
use crate::cg_predict::{CG_G2Trace, CG_Trace};
use crate::cg_saga::{CG_SiegeObjectiveCompleted, CG_SiegeRoundOver};
use crate::cg_view::{CGCam_SetMusicMult, CGCam_Shake, CG_AddBufferedSound};
use crate::cg_weaponinit::CG_RegisterWeapon;
use crate::cg_weapons::{
    CG_FireWeapon, CG_GetClientWeaponMuzzleBoltPoint, CG_MissileHitPlayer, CG_MissileHitWall,
    CG_OutOfAmmoChange, CG_VehicleWeaponImpact,
};
use crate::fx_bryarpistol::FX_ConcAltShot;
use crate::fx_demp2::FX_DEMP2_AltDetonate;
use crate::fx_disruptor::{
    FX_DisruptorAltMiss, FX_DisruptorAltShot, FX_DisruptorHitPlayer, FX_DisruptorHitWall,
    FX_DisruptorMainShot,
};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::{clientInfo_t, MAX_CUSTOM_SIEGE_SOUNDS};
use crate::local::footstep_t::footstep_t;
use crate::local::impact_sound_t::impactSound_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `DEBUGNAME(x)` — prints `x` (plus a trailing newline) when
/// `cg_debugEvents` is set.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1489`
#[allow(dead_code)]
fn DEBUGNAME(ctx: &mut CgContext, x: &str) {
    if ctx.world.cvars.cg_debugEvents.integer != 0 {
        // Raven's macro pastes the newline onto the literal and hands it
        // straight through as CG_Printf's format string (no substitution).
        CG_Printf(ctx, &format!("{x}\n"));
    }
}

/// Raven `EV_EVENT_BIT1`.
///
/// Source: `oracle/codemp/game/bg_public.h:728`
const EV_EVENT_BIT1: c_int = 0x00000100;

/// Raven `EV_EVENT_BIT2`.
///
/// Source: `oracle/codemp/game/bg_public.h:729`
const EV_EVENT_BIT2: c_int = 0x00000200;

/// Raven `EV_EVENT_BITS`.
///
/// Source: `oracle/codemp/game/bg_public.h:730`
const EV_EVENT_BITS: c_int = EV_EVENT_BIT1 | EV_EVENT_BIT2;

/// Raven `STEP_TIME` — how long the stair-climb view smoothing lasts, in msec.
/// (Same value as cg_view.rs's private copy of the `cg_local.h` define.)
///
/// Source: `oracle/codemp/cgame/cg_local.h:33`
const STEP_TIME: c_int = 200;

/// Raven `MAX_STEP_CHANGE` — the ceiling the accumulated step smoothing caps at.
///
/// Source: `oracle/codemp/cgame/cg_local.h:54`
const MAX_STEP_CHANGE: c_int = 32;

// The anonymous taunt-index enum from `cg_event.c:30-36` — plain int constants
// (Raven's `es->eventParm` is compared against them raw). Source:
// `oracle/codemp/cgame/cg_event.c:30-36`.
const TAUNT_TAUNT: c_int = 0;
const TAUNT_BOW: c_int = 1;
const TAUNT_MEDITATE: c_int = 2;
const TAUNT_FLOURISH: c_int = 3;
const TAUNT_GLOAT: c_int = 4;

/// Raven `cg_stringEdVoiceChatTable[MAX_CUSTOM_SIEGE_SOUNDS]` — string-package
/// `MENUS` reference names, index-parallel with `bg_customSiegeSoundNames`.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1429-1459`
static cg_stringEdVoiceChatTable: [Option<&str>; MAX_CUSTOM_SIEGE_SOUNDS] = [
    Some("VC_ATT"),           //"*att_attack",
    Some("VC_ATT_PRIMARY"),   //"*att_primary",
    Some("VC_ATT_SECONDARY"), //"*att_second",
    Some("VC_DEF_GUNS"),      //"*def_guns",
    Some("VC_DEF_POSITION"),  //"*def_position",
    Some("VC_DEF_PRIMARY"),   //"*def_primary",
    Some("VC_DEF_SECONDARY"), //"*def_second",
    Some("VC_REPLY_COMING"),  //"*reply_coming",
    Some("VC_REPLY_GO"),      //"*reply_go",
    Some("VC_REPLY_NO"),      //"*reply_no",
    Some("VC_REPLY_STAY"),    //"*reply_stay",
    Some("VC_REPLY_YES"),     //"*reply_yes",
    Some("VC_REQ_ASSIST"),    //"*req_assist",
    Some("VC_REQ_DEMO"),      //"*req_demo",
    Some("VC_REQ_HVY"),       //"*req_hvy",
    Some("VC_REQ_MEDIC"),     //"*req_medic",
    Some("VC_REQ_SUPPLY"),    //"*req_sup",
    Some("VC_REQ_TECH"),      //"*req_tech",
    Some("VC_SPOT_AIR"),      //"*spot_air",
    Some("VC_SPOT_DEF"),      //"*spot_defenses",
    Some("VC_SPOT_EMPLACED"), //"*spot_emplaced",
    Some("VC_SPOT_SNIPER"),   //"*spot_sniper",
    Some("VC_SPOT_TROOP"),    //"*spot_troops",
    Some("VC_TAC_COVER"),     //"*tac_cover",
    Some("VC_TAC_FALLBACK"),  //"*tac_fallback",
    Some("VC_TAC_FOLLOW"),    //"*tac_follow",
    Some("VC_TAC_HOLD"),      //"*tac_hold",
    Some("VC_TAC_SPLIT"),     //"*tac_split",
    Some("VC_TAC_TOGETHER"),  //"*tac_together",
    None,
];

/// Raven `CG_PlaceString` — builds the localized ordinal-rank string ("1st",
/// "Tied for 2nd", ...) for the scoreboard/reward stack.
///
/// The static `char str[64]` return buffer becomes an owned `String` — the
/// function always overwrites and returns it fresh, so there is nothing to
/// fold into `CgWorld` (no caller ever observes the buffer between calls). Its
/// 64-byte width is load-bearing, though: german ordinals overrun it and Raven
/// clips them, so the result caps at 63 Latin-1 bytes plus the NUL slot.
///
/// A missing string-package key comes back as the engine's `"??<key>"` marker,
/// not as an empty string (`oracle/codemp/client/cl_cgame.cpp:1670-1678`).
///
/// Source: `oracle/codemp/cgame/cg_event.c:45-94`
pub fn CG_PlaceString(ctx: &mut CgContext, mut rank: c_int) -> String {
    let s_st = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_ST", 10)
        .unwrap_or_else(|| "??MP_INGAME_NUMBER_ST".to_string());
    let s_nd = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_ND", 10)
        .unwrap_or_else(|| "??MP_INGAME_NUMBER_ND".to_string());
    let s_rd = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_RD", 10)
        .unwrap_or_else(|| "??MP_INGAME_NUMBER_RD".to_string());
    let s_th = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_TH", 10)
        .unwrap_or_else(|| "??MP_INGAME_NUMBER_TH".to_string());
    let mut s_tied_for = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_TIED_FOR", 64)
        .unwrap_or_else(|| "??MP_INGAME_TIED_FOR".to_string());
    // save worrying about translators adding spaces or not
    s_tied_for.push(' ');

    let t = if (rank & RANK_TIED_FLAG) != 0 {
        rank &= !RANK_TIED_FLAG;
        s_tied_for.as_str()
    } else {
        ""
    };

    let s = if rank == 1 {
        format!("1{s_st}") // draw in blue
    } else if rank == 2 {
        format!("2{s_nd}") // draw in red
    } else if rank == 3 {
        format!("3{s_rd}") // draw in yellow
    } else if rank == 11 {
        format!("11{s_th}")
    } else if rank == 12 {
        format!("12{s_th}")
    } else if rank == 13 {
        format!("13{s_th}")
    } else if rank % 10 == 1 {
        format!("{rank}{s_st}")
    } else if rank % 10 == 2 {
        format!("{rank}{s_nd}")
    } else if rank % 10 == 3 {
        format!("{rank}{s_rd}")
    } else {
        format!("{rank}{s_th}")
    };

    // Com_sprintf into `str[64]` - one Latin-1 char is one C byte, so 63 of
    // them plus the NUL is everything that survives
    format!("{t}{s}").chars().take(63).collect()
}

/// Raven `CG_ToggleBinoculars` — flips the local player's zoom mode and plays
/// the zoom start/end sound.
///
/// §F19: Raven derefs `cg.snap` unchecked; before the first snapshot there is
/// no zoom state to flip, so `cg_t::snap_mut`'s `None` takes the no-op.
/// Source: `oracle/codemp/cgame/cg_event.c:611-650`
pub fn CG_ToggleBinoculars(ctx: &mut CgContext, cent: &centity_t, forceZoom: c_int) {
    let number = cent.currentState.number;

    let Some(snap) = ctx.world.cg.snap_mut() else {
        return;
    };

    if number != snap.ps.clientNum {
        return;
    }

    if snap.ps.weaponstate != weaponstate_t::WEAPON_READY as c_int {
        // So we can't fool it and reactivate while switching to the saber or something.
        return;
    }

    /*
    if (cg.snap->ps.weapon == WP_SABER)
    { //No.
        return;
    }
    */

    if forceZoom != 0 {
        if forceZoom == 2 {
            snap.ps.zoomMode = 0;
        } else if forceZoom == 1 {
            snap.ps.zoomMode = 2;
        }
    }

    let zoomMode = snap.ps.zoomMode;
    let clientNum = snap.ps.clientNum;

    if zoomMode == 0 {
        trap::S_StartSound(
            ctx.engine,
            None,
            clientNum,
            CHAN_AUTO,
            ctx.world.cgs.media.zoomStart,
        );
    } else if zoomMode == 2 {
        trap::S_StartSound(
            ctx.engine,
            None,
            clientNum,
            CHAN_AUTO,
            ctx.world.cgs.media.zoomEnd,
        );
    }
}

/// Raven `CG_LocalTimingBar` — arms the generic countdown timer bar HUD
/// element (used for duel/round countdowns) with a start time, duration, and
/// a fixed yellow color.
///
/// The three `cg_draw.c` file statics it drives
/// (`oracle/codemp/cgame/cg_draw.c:4738-4740`) live on `CgDrawState`.
/// Source: `oracle/codemp/cgame/cg_event.c:656-665`
pub fn CG_LocalTimingBar(world: &mut CgWorld, startTime: c_int, duration: c_int) {
    world.draw.cg_genericTimerBar = startTime + duration;
    world.draw.cg_genericTimerDur = duration;

    world.draw.cg_genericTimerColor = [1.0, 1.0, 0.0, 1.0];
}

/// Raven `CG_ReattachLimb` — re-applies the torso skin on an entity's ghoul2
/// instance and clears its dismemberment/weapon-attach state.
///
/// Raven's limb-stub-capping block (commented out in the oracle) stays
/// commented out here — dead code, never compiled.
///
/// Source: `oracle/codemp/cgame/cg_event.c:868-943`
pub fn CG_ReattachLimb(ctx: &mut CgContext, sourceNum: usize) {
    let source = ctx.world.entity(sourceNum);
    let number = source.currentState.number;
    let ghoul2 = source.ghoul2;

    let torso_skin = if number >= MAX_CLIENTS_I32 {
        source.npcClient.as_deref().map(|ci| ci.torsoSkin)
    } else {
        Some(ctx.world.cgs.clientinfo[number as usize].torsoSkin)
    };

    // re-apply the skin
    if let Some(torso_skin) = torso_skin {
        if torso_skin > 0 {
            trap::G2API_SetSkin(ctx.engine, ghoul2, 0, torso_skin, torso_skin);
        }
    }

    // char *limbName;
    // char *stubCapName;
    // int i = G2_MODELPART_HEAD;
    //
    // //rww NOTE: Assumes G2_MODELPART_HEAD is first and G2_MODELPART_RLEG is last
    // while (i <= G2_MODELPART_RLEG)
    // {
    // 	if (source->torsoBolt & (1 << (i-10)))
    // 	{
    // 		switch (i)
    // 		{
    // 		case G2_MODELPART_HEAD:
    // 			limbName = "head";
    // 			stubCapName = "torso_cap_head";
    // 			break;
    // 		case G2_MODELPART_WAIST:
    // 			limbName = "torso";
    // 			stubCapName = "hips_cap_torso";
    // 			break;
    // 		case G2_MODELPART_LARM:
    // 			limbName = "l_arm";
    // 			stubCapName = "torso_cap_l_arm";
    // 			break;
    // 		case G2_MODELPART_RARM:
    // 			limbName = "r_arm";
    // 			stubCapName = "torso_cap_r_arm";
    // 			break;
    // 		case G2_MODELPART_RHAND:
    // 			limbName = "r_hand";
    // 			stubCapName = "r_arm_cap_r_hand";
    // 			break;
    // 		case G2_MODELPART_LLEG:
    // 			limbName = "l_leg";
    // 			stubCapName = "hips_cap_l_leg";
    // 			break;
    // 		case G2_MODELPART_RLEG:
    // 			limbName = "r_leg";
    // 			stubCapName = "hips_cap_r_leg";
    // 			break;
    // 		default:
    // 			source->torsoBolt = 0;
    // 			source->ghoul2weapon = NULL;
    // 			return;
    // 		}
    //
    // 		trap_G2API_SetSurfaceOnOff(source->ghoul2, limbName, 0);
    // 		trap_G2API_SetSurfaceOnOff(source->ghoul2, stubCapName, 0x00000100);
    // 	}
    // 	i++;
    // }

    let source = ctx.world.entity_mut(sourceNum);
    source.torsoBolt = 0;
    source.ghoul2weapon = null_mut();
}

/// Raven `CG_TeamName` — team enum to its uppercase HUD label.
///
/// Source: `oracle/codemp/cgame/cg_event.c:945-954`
pub fn CG_TeamName(team: team_t) -> &'static str {
    if team == TEAM_RED {
        "RED"
    } else if team == TEAM_BLUE {
        "BLUE"
    } else if team == TEAM_SPECTATOR {
        "SPECTATOR"
    } else {
        "FREE"
    }
}

/// Raven `CG_InClientBitflags` — tests bit `client` across `entityState_t`'s
/// four 16-bit-wide `trickedentindex*` chunks (client numbers 0-15, 16-31,
/// 32-47, 48-63 respectively).
///
/// Source: `oracle/codemp/cgame/cg_event.c:1170-1201`
pub fn CG_InClientBitflags(ent: &entityState_t, client: c_int) -> bool {
    let (checkIn, sub) = if client > 47 {
        (ent.trickedentindex4, 48)
    } else if client > 31 {
        (ent.trickedentindex3, 32)
    } else if client > 15 {
        (ent.trickedentindex2, 16)
    } else {
        (ent.trickedentindex, 0)
    };

    (checkIn & (1 << (client - sub))) != 0
}

/// Raven `CG_CalcVehMuzzle` — resolves a vehicle muzzle's world position and
/// direction from its ghoul2 bolt for this frame, memoized per-frame via
/// `m_iMuzzleTime`.
///
/// Raven's `assert(pVeh)` is vacuous once the parameter is a Rust reference.
///
/// DEFERRED: the `VH_ANIMAL`/`VH_WALKER`/`VH_SPEEDER` pitch/roll zeroing reads
/// `pVeh->m_pVehicleInfo->type`; `Vehicle_t.m_pVehicleInfo` is still a raw
/// `*mut vehicleInfo_t` (no safe accessor exists yet) and this wave forbids
/// `unsafe` outside `trap.rs`, so that branch never fires — `vehAngles` is
/// always the entity's raw `lerpAngles` here.
/// Source: `oracle/codemp/cgame/cg_event.c:1365-1376`
///
/// Source: `oracle/codemp/cgame/cg_event.c:1350-1381`
pub fn CG_CalcVehMuzzle(
    ctx: &mut CgContext,
    pVeh: &mut Vehicle_t,
    ent: &centity_t,
    muzzleNum: c_int,
) {
    let muzzle_idx = muzzleNum as usize;

    if pVeh.m_iMuzzleTime[muzzle_idx] == ctx.world.cg.time {
        // already done for this frame, don't need to do it again
        return;
    }
    // Uh... how about we set this, hunh...?  :)
    pVeh.m_iMuzzleTime[muzzle_idx] = ctx.world.cg.time;

    let veh_angles: vec3_t = ent.lerpAngles;
    if !pVeh.m_pVehicleInfo.is_null() {
        // DEFERRED: vehicleInfo_t.type read — see the fn doc above.
    }

    let mut bolt_matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    trap::G2API_GetBoltMatrix_NoRecNoRot(
        ctx.engine,
        ent.ghoul2,
        0,
        pVeh.m_iMuzzleTag[muzzle_idx],
        &mut bolt_matrix,
        &veh_angles,
        &ent.lerpOrigin,
        ctx.world.cg.time,
        None,
        &ent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(
        &bolt_matrix as *const mdxaBone_t,
        Eorientations::ORIGIN as c_int,
        &mut pVeh.m_vMuzzlePos[muzzle_idx],
    );
    BG_GiveMeVectorFromMatrix(
        &bolt_matrix as *const mdxaBone_t,
        Eorientations::NEGATIVE_Y as c_int,
        &mut pVeh.m_vMuzzleDir[muzzle_idx],
    );
}

/// Raven `CG_VehMuzzleFireFX` — plays the bolted muzzle-flash effect for every
/// vehicle muzzle the broadcaster's `trickedentindex` bits report as fired
/// this frame.
///
/// ESCALATION: blocked past the presence guard — `centity_t.m_pVehicle` is
/// DEC-46.2's `Option<VehicleId>`, which carries only the vehicle cent's
/// entity number ("ported code only tests presence… until [the Vehicle_t
/// referent pool] lands"). The muzzle loop needs the actual `Vehicle_t` data
/// (`m_iMuzzleTag`, `m_pVehicleInfo`, the turret/weapon tables) that only a
/// `VehicleId -> &Vehicle_t` accessor can supply, and that pool hasn't landed
/// yet. This blocks every future cgame fn that walks a `centity_t`'s vehicle
/// through `m_pVehicle` the same way.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1384-1427`
pub fn CG_VehMuzzleFireFX(ctx: &mut CgContext, veh: &centity_t, broadcaster: &entityState_t) {
    if veh.m_pVehicle.is_none() || veh.ghoul2.is_null() {
        return;
    }

    let _ = (ctx, broadcaster);
    // DEFERRED: muzzle-fire FX loop needs Vehicle_t field access
    // (m_iMuzzleTag, m_pVehicleInfo, weapMuzzle/turret tables) — see the fn
    // doc above.
    //TODO: Port CG_VehMuzzleFireFX
    // Source: oracle/codemp/cgame/cg_event.c:1394-1426
    todo!("CG_VehMuzzleFireFX muzzle FX loop — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_event.c:1394-1426")
}

/// Raven `CG_UseItem` — dispatches on the holdable item just used (server-sent
/// `EV_USE_ITEM0 + n` event), plays the item's sound, and cycles the local
/// player's holdable selection off it.
///
/// §F19: Raven derefs `cg.snap` unchecked in both the "print a message"
/// lookup and the trailing cycle-inventory check; before the first snapshot
/// there's no local player to match against, so `cg_t::snap_ref`'s `None`
/// takes the no-op in both spots.
///
/// Source: `oracle/codemp/cgame/cg_event.c:672-743`
pub fn CG_UseItem(ctx: &mut CgContext, cent: &centity_t) {
    let es = &cent.currentState;

    let mut itemNum = (es.event & !EV_EVENT_BITS) - EV_USE_ITEM0 as c_int;
    if itemNum < 0 || itemNum > HI_NUM_HOLDABLE {
        itemNum = 0;
    }

    // print a message if the local player
    if ctx
        .world
        .cg
        .snap_ref()
        .is_some_and(|snap| es.number == snap.ps.clientNum)
        && itemNum != HI_NONE
    {
        // Raven's lookup result (`item`) is assigned and never read again
        // (cg_event.c:690) — the call itself is what matters, since
        // `BG_FindItemForHoldable` panics on an unknown holdable.
        let _item = BG_FindItemForHoldable(itemNum);
    }

    match itemNum {
        HI_BINOCULARS => CG_ToggleBinoculars(ctx, cent, es.eventParm),

        HI_SEEKER => {
            trap::S_StartSound(
                ctx.engine,
                None,
                es.number,
                CHAN_AUTO,
                ctx.world.cgs.media.deploySeeker,
            );
        }

        HI_SHIELD | HI_SENTRY_GUN => {}

        HI_MEDPAC | HI_MEDPAC_BIG => {
            let clientNum = es.clientNum;
            if clientNum >= 0 && clientNum < MAX_CLIENTS_I32 {
                let time = ctx.world.cg.time;
                ctx.world.cgs.clientinfo[clientNum as usize].medkitUsageTime = time;
            }
            //Different sound for big bacta?
            trap::S_StartSound(
                ctx.engine,
                None,
                es.number,
                CHAN_AUTO,
                ctx.world.cgs.media.medkitSound,
            );
        }

        HI_JETPACK => {} //Do something?

        HI_HEALTHDISP => {
            //CG_LocalTimingBar(cg.time, TOSS_DEBOUNCE_TIME);
        }

        HI_AMMODISP => {
            //CG_LocalTimingBar(cg.time, TOSS_DEBOUNCE_TIME);
        }

        HI_EWEB => {}

        HI_CLOAK => {} //Do something?

        // HI_NONE and any out-of-range fallthrough both land here.
        _ => {
            //trap_S_StartSound (NULL, es->number, CHAN_BODY, cgs.media.useNothingSound );
        }
    }

    let should_cycle = ctx.world.cg.snap_ref().is_some_and(|snap| {
        snap.ps.clientNum == cent.currentState.number
            && itemNum != HI_BINOCULARS
            && itemNum != HI_JETPACK
            && itemNum != HI_HEALTHDISP
            && itemNum != HI_AMMODISP
            && itemNum != HI_CLOAK
            && itemNum != HI_EWEB
    });

    if should_cycle {
        //if not using binoculars/jetpack/dispensers/cloak, we just used that item up, so switch
        if let Some(snap) = ctx.world.cg.snap_mut() {
            BG_CycleInven(&mut snap.ps, 1);
        }
        ctx.world.cg.itemSelect = -1; //update the client-side selection display
    }
}

/// Raven `CG_GetStringForVoiceSound` — maps a custom siege voice-order sound
/// name (`bg_customSiegeSoundNames`) to its localized `MENUS` string-package
/// text, falling back to `"voice chat"` on a miss.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1464-1479`
pub fn CG_GetStringForVoiceSound(ctx: &mut CgContext, s: &str) -> String {
    let mut i = 0;
    while i < MAX_CUSTOM_SIEGE_SOUNDS {
        if let Some(name) = bg_customSiegeSoundNames[i] {
            if name.to_str().unwrap_or("").eq_ignore_ascii_case(s) {
                //get the matching reference name
                // Raven asserts in debug and passes the NULL through in the
                // shipped build; "" is the defined stand-in for that (§F19)
                debug_assert!(
                    cg_stringEdVoiceChatTable[i].is_some(),
                    "cg_stringEdVoiceChatTable entry missing for a populated bg_customSiegeSoundNames slot"
                );
                let refName = cg_stringEdVoiceChatTable[i].unwrap_or("");
                return CG_GetStringEdString(ctx, "MENUS", refName);
            }
        }
        i += 1;
    }

    "voice chat".to_string()
}

/// Raven `CG_Obituary` — turns an `EV_OBITUARY` entity event into the console
/// death message (plus the centerprint when the local player got the kill).
///
/// Raven's `goto clientkilled` is the fall-through when the kill is *not*
/// client-on-client, so the port hangs the same block off the negated test.
/// The four `char[32]` name buffers become owned `String`s clipped to Raven's
/// `sizeof(buf) - 2` (29 Latin-1 chars) so the `^7` `strcat` still fits.
///
/// §F19: Raven derefs `cg.snap` unchecked; with no snapshot yet there is no
/// local client, so both "is this about me?" tests take the no arm.
///
/// Source: `oracle/codemp/cgame/cg_event.c:103-607`
pub fn CG_Obituary(ctx: &mut CgContext, ent: &entityState_t) {
    let mut message: Option<&str>;
    let mut vehMessage = false;

    let target = ent.otherEntityNum;
    let attacker = ent.otherEntityNum2;
    let r#mod = ent.eventParm;

    if target < 0 || target >= MAX_CLIENTS_I32 {
        CG_Error(ctx, "CG_Obituary: target out of range");
        return;
    }
    // Raven's `ci = &cgs.clientinfo[target]` is only ever read for its gender.
    let gender = ctx.world.cgs.clientinfo[target as usize].gender;

    let attackerInfo = if attacker < 0 || attacker >= MAX_CLIENTS_I32 {
        //attacker = ENTITYNUM_WORLD;
        None
    } else {
        Some(CG_ConfigString(ctx, CS_PLAYERS + attacker))
    };

    let targetInfo = CG_ConfigString(ctx, CS_PLAYERS + target);
    // PORT-NOTE: Raven's `if (!targetInfo) return;` guards a pointer
    // `CG_ConfigString` never hands back as NULL; the owned `String` keeps the
    // guard just as dead, so it is not transcribed.

    let mut targetName: String = Info_ValueForKey(&targetInfo, "n")
        .chars()
        .take(29)
        .collect();
    targetName.push_str(S_COLOR_WHITE.to_str().unwrap());

    // check for target in a vehicle
    let mut targetVehName = String::new();
    if ent.lookTarget > VEHICLE_BASE && ent.lookTarget < MAX_VEHICLES as c_int {
        let name = ctx.world.bg_state.g_vehicleInfo[ent.lookTarget as usize].name;
        if !name.is_null() {
            // SAFETY: `.name` points into `bg_state`'s own `VehicleParms` parse
            // buffer - the same read `bg_vehicleLoad.rs:809` makes.
            targetVehName = unsafe { cstr_to_str(name) }.chars().take(29).collect();
        }
    }

    // check for attacker in a vehicle
    // DEFERRED: `Vehicle_t` referent pool — Raven copies
    // `attVehCent->m_pVehicle->m_pVehicleInfo->name` out of
    // `cg_entities[ent->brokenLimbs]`, and DEC-46.2's `Option<VehicleId>`
    // answers presence only until the pool lands (`local/vehicle_id.rs`), so
    // there is nothing to read the name from and this stays empty.
    // Source: oracle/codemp/cgame/cg_event.c:147-158
    let attackerVehName = String::new();

    //check for specific vehicle weapon
    let mut attackerVehWeapName = String::new();
    if ent.weapon > 0 {
        // §F19: Raven indexes `g_vehWeaponInfo[MAX_VEH_WEAPONS]` with
        // `ent->weapon-1`, which runs past the table for the top `weapon_t`
        // values; out of range reads as "no name" here.
        if (ent.weapon as usize) <= MAX_VEH_WEAPONS {
            let name = ctx.world.bg_state.g_vehWeaponInfo[(ent.weapon - 1) as usize].name;
            if !name.is_null() {
                // SAFETY: `.name` points into `bg_state`'s own `VehWeaponParms`
                // parse buffer - the same read `bg_vehicleLoad.rs:305` makes.
                attackerVehWeapName = unsafe { cstr_to_str(name) }.chars().take(29).collect();
            }
        }
    }

    // check for single client messages

    if ent.saberInFlight != qfalse {
        //asteroid->vehicle collision
        message = match ctx.world.bg_state.rng.Q_irand(0, 2) {
            1 => Some("DIED_ASTEROID2"),
            2 => Some("DIED_ASTEROID3"),
            // Raven's `default:` falls into `case 0:`
            _ => Some("DIED_ASTEROID1"),
        };
        vehMessage = true;
    } else {
        message = match r#mod {
            m if m == meansOfDeath_t::MOD_VEHICLE as c_int
                || m == meansOfDeath_t::MOD_SUICIDE as c_int
                || m == meansOfDeath_t::MOD_FALLING as c_int
                || m == meansOfDeath_t::MOD_COLLISION as c_int
                || m == meansOfDeath_t::MOD_VEH_EXPLOSION as c_int
                || m == meansOfDeath_t::MOD_CRUSH as c_int
                || m == meansOfDeath_t::MOD_WATER as c_int
                || m == meansOfDeath_t::MOD_SLIME as c_int
                || m == meansOfDeath_t::MOD_LAVA as c_int
                || m == meansOfDeath_t::MOD_TRIGGER_HURT as c_int =>
            {
                Some("DIED_GENERIC")
            }

            m if m == meansOfDeath_t::MOD_TARGET_LASER as c_int => {
                vehMessage = true;
                Some("DIED_TURBOLASER")
            }

            _ => None,
        };
    }

    // Attacker killed themselves.  Ridicule them for it.
    if attacker == target {
        vehMessage = false;
        message = Some(match r#mod {
            m if m == meansOfDeath_t::MOD_BRYAR_PISTOL as c_int
                || m == meansOfDeath_t::MOD_BRYAR_PISTOL_ALT as c_int
                || m == meansOfDeath_t::MOD_BLASTER as c_int
                || m == meansOfDeath_t::MOD_TURBLAST as c_int
                || m == meansOfDeath_t::MOD_DISRUPTOR as c_int
                || m == meansOfDeath_t::MOD_DISRUPTOR_SPLASH as c_int
                || m == meansOfDeath_t::MOD_DISRUPTOR_SNIPER as c_int
                || m == meansOfDeath_t::MOD_BOWCASTER as c_int
                || m == meansOfDeath_t::MOD_REPEATER as c_int
                || m == meansOfDeath_t::MOD_REPEATER_ALT as c_int
                || m == meansOfDeath_t::MOD_FLECHETTE as c_int =>
            {
                match gender {
                    gender_t::GENDER_FEMALE => "SUICIDE_SHOT_FEMALE",
                    gender_t::GENDER_NEUTER => "SUICIDE_SHOT_GENDERLESS",
                    _ => "SUICIDE_SHOT_MALE",
                }
            }

            m if m == meansOfDeath_t::MOD_REPEATER_ALT_SPLASH as c_int
                || m == meansOfDeath_t::MOD_FLECHETTE_ALT_SPLASH as c_int
                || m == meansOfDeath_t::MOD_ROCKET as c_int
                || m == meansOfDeath_t::MOD_ROCKET_SPLASH as c_int
                || m == meansOfDeath_t::MOD_ROCKET_HOMING as c_int
                || m == meansOfDeath_t::MOD_ROCKET_HOMING_SPLASH as c_int
                || m == meansOfDeath_t::MOD_THERMAL as c_int
                || m == meansOfDeath_t::MOD_THERMAL_SPLASH as c_int
                || m == meansOfDeath_t::MOD_TRIP_MINE_SPLASH as c_int
                || m == meansOfDeath_t::MOD_TIMED_MINE_SPLASH as c_int
                || m == meansOfDeath_t::MOD_DET_PACK_SPLASH as c_int
                || m == meansOfDeath_t::MOD_VEHICLE as c_int
                || m == meansOfDeath_t::MOD_CONC as c_int
                || m == meansOfDeath_t::MOD_CONC_ALT as c_int =>
            {
                match gender {
                    gender_t::GENDER_FEMALE => "SUICIDE_EXPLOSIVES_FEMALE",
                    gender_t::GENDER_NEUTER => "SUICIDE_EXPLOSIVES_GENDERLESS",
                    _ => "SUICIDE_EXPLOSIVES_MALE",
                }
            }

            m if m == meansOfDeath_t::MOD_DEMP2 as c_int => match gender {
                gender_t::GENDER_FEMALE => "SUICIDE_ELECTROCUTED_FEMALE",
                gender_t::GENDER_NEUTER => "SUICIDE_ELECTROCUTED_GENDERLESS",
                _ => "SUICIDE_ELECTROCUTED_MALE",
            },

            m if m == meansOfDeath_t::MOD_FALLING as c_int => match gender {
                gender_t::GENDER_FEMALE => "SUICIDE_FALLDEATH_FEMALE",
                gender_t::GENDER_NEUTER => "SUICIDE_FALLDEATH_GENDERLESS",
                _ => "SUICIDE_FALLDEATH_MALE",
            },

            _ => match gender {
                gender_t::GENDER_FEMALE => "SUICIDE_GENERICDEATH_FEMALE",
                gender_t::GENDER_NEUTER => "SUICIDE_GENERICDEATH_GENDERLESS",
                _ => "SUICIDE_GENERICDEATH_MALE",
            },
        });
    }

    // Raven's `goto clientkilled` skips this block for a client-on-client kill.
    if !(target != attacker && target < MAX_CLIENTS_I32 && attacker < MAX_CLIENTS_I32) {
        if let Some(mut msg) = message {
            // PORT-NOTE: Raven's `!message[0]` arm is unreachable - every path
            // that leaves `message` non-NULL sets a non-empty literal. Kept.
            if msg.is_empty() {
                vehMessage = false;
                msg = match gender {
                    gender_t::GENDER_FEMALE => "SUICIDE_GENERICDEATH_FEMALE",
                    gender_t::GENDER_NEUTER => "SUICIDE_GENERICDEATH_GENDERLESS",
                    _ => "SUICIDE_GENERICDEATH_MALE",
                };
            }
            let text = if vehMessage {
                CG_GetStringEdString(ctx, "MP_INGAMEVEH", msg)
            } else {
                CG_GetStringEdString(ctx, "MP_INGAME", msg)
            };

            CG_Printf(ctx, &format!("{targetName} {text}\n"));
            return;
        }
    }

    // clientkilled:

    // check for kill messages from the current clientNum
    let snapInfo = ctx.world.cg.snap_ref().map(|snap| {
        (
            snap.ps.clientNum,
            snap.ps.isJediMaster,
            snap.ps.persistant[PERS_RANK as usize],
            snap.ps.persistant[PERS_SCORE as usize],
        )
    });

    if let Some((snapClientNum, snapIsJediMaster, persRank, persScore)) = snapInfo {
        if attacker == snapClientNum {
            let s;

            if ctx.world.cgs.gametype < GT_TEAM
                && ctx.world.cgs.gametype != GT_DUEL
                && ctx.world.cgs.gametype != GT_POWERDUEL
            {
                if ctx.world.cgs.gametype == GT_JEDIMASTER
                    && attacker < MAX_CLIENTS_I32
                    && ent.isJediMaster == qfalse
                    && snapIsJediMaster == qfalse
                    && CG_ThereIsAMaster(ctx.world)
                {
                    let part1 =
                        trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_KILLED_MESSAGE", 512)
                            .unwrap_or_else(|| "??MP_INGAME_KILLED_MESSAGE".to_string());
                    let part2 =
                        trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_JMKILLED_NOTJM", 512)
                            .unwrap_or_else(|| "??MP_INGAME_JMKILLED_NOTJM".to_string());
                    s = format!("{part1} {targetName}\n{part2}\n");
                } else if ctx.world.cgs.gametype == GT_JEDIMASTER
                    && attacker < MAX_CLIENTS_I32
                    && ent.isJediMaster == qfalse
                    && snapIsJediMaster == qfalse
                {
                    //no JM, saber must be out
                    let part1 =
                        trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_KILLED_MESSAGE", 512)
                            .unwrap_or_else(|| "??MP_INGAME_KILLED_MESSAGE".to_string());
                    /*
                    kmsg1 = "for 0 points.\nGo for the saber!";
                    strcpy(part2, kmsg1);

                    s = va("%s %s %s\n", part1, targetName, part2);
                    */
                    s = format!("{part1} {targetName}\n");
                } else if ctx.world.cgs.gametype == GT_POWERDUEL {
                    // PORT-NOTE: unreachable - the enclosing test already ruled
                    // GT_POWERDUEL out. Raven's dead arm, kept.
                    s = String::new();
                } else {
                    let sPlaceWith =
                        trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_PLACE_WITH", 256)
                            .unwrap_or_else(|| "??MP_INGAME_PLACE_WITH".to_string());
                    let sKilledStr =
                        trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_KILLED_MESSAGE", 256)
                            .unwrap_or_else(|| "??MP_INGAME_KILLED_MESSAGE".to_string());

                    let place = CG_PlaceString(ctx, persRank + 1);
                    s = format!("{sKilledStr} {targetName}.\n{place} {sPlaceWith} {persScore}.");
                }
            } else {
                let sKilledStr =
                    trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_KILLED_MESSAGE", 256)
                        .unwrap_or_else(|| "??MP_INGAME_KILLED_MESSAGE".to_string());
                s = format!("{sKilledStr} {targetName}");
            }

            //if (!(cg_singlePlayerActive.integer && cg_cameraOrbit.integer)) {
            CG_CenterPrint(
                ctx.world,
                &s,
                (SCREEN_HEIGHT as f64 * 0.30) as c_int,
                BIGCHAR_WIDTH,
            );
            //}
            // print the text message as well
        }
    }

    // check for double client messages
    let attackerName = match &attackerInfo {
        None => {
            //attacker = ENTITYNUM_WORLD;
            "noname".to_string()
        }
        Some(info) => {
            let mut name: String = Info_ValueForKey(info, "n").chars().take(29).collect();
            name.push_str(S_COLOR_WHITE.to_str().unwrap());
            // check for kill messages about the current clientNum
            if let Some((snapClientNum, ..)) = snapInfo {
                if target == snapClientNum {
                    let destsize = ctx.world.cg.killerName.len();
                    let bytes = string_to_latin1(&name);
                    Q_strncpyzBytes(&mut ctx.world.cg.killerName, &bytes, destsize);
                }
            }
            name
        }
    };

    if attacker != ENTITYNUM_WORLD {
        message = match r#mod {
            m if m == meansOfDeath_t::MOD_STUN_BATON as c_int => Some("KILLED_STUN"),

            m if m == meansOfDeath_t::MOD_MELEE as c_int => Some("KILLED_MELEE"),

            m if m == meansOfDeath_t::MOD_SABER as c_int => Some("KILLED_SABER"),

            m if m == meansOfDeath_t::MOD_BRYAR_PISTOL as c_int
                || m == meansOfDeath_t::MOD_BRYAR_PISTOL_ALT as c_int =>
            {
                Some("KILLED_BRYAR")
            }

            m if m == meansOfDeath_t::MOD_BLASTER as c_int => Some("KILLED_BLASTER"),

            m if m == meansOfDeath_t::MOD_TURBLAST as c_int => Some("KILLED_BLASTER"),

            m if m == meansOfDeath_t::MOD_DISRUPTOR as c_int
                || m == meansOfDeath_t::MOD_DISRUPTOR_SPLASH as c_int =>
            {
                Some("KILLED_DISRUPTOR")
            }

            m if m == meansOfDeath_t::MOD_DISRUPTOR_SNIPER as c_int => {
                Some("KILLED_DISRUPTORSNIPE")
            }

            m if m == meansOfDeath_t::MOD_BOWCASTER as c_int => Some("KILLED_BOWCASTER"),

            m if m == meansOfDeath_t::MOD_REPEATER as c_int => Some("KILLED_REPEATER"),

            m if m == meansOfDeath_t::MOD_REPEATER_ALT as c_int
                || m == meansOfDeath_t::MOD_REPEATER_ALT_SPLASH as c_int =>
            {
                Some("KILLED_REPEATERALT")
            }

            m if m == meansOfDeath_t::MOD_DEMP2 as c_int
                || m == meansOfDeath_t::MOD_DEMP2_ALT as c_int =>
            {
                Some("KILLED_DEMP2")
            }

            m if m == meansOfDeath_t::MOD_FLECHETTE as c_int => Some("KILLED_FLECHETTE"),

            m if m == meansOfDeath_t::MOD_FLECHETTE_ALT_SPLASH as c_int => {
                Some("KILLED_FLECHETTE_MINE")
            }

            m if m == meansOfDeath_t::MOD_ROCKET as c_int
                || m == meansOfDeath_t::MOD_ROCKET_SPLASH as c_int =>
            {
                Some("KILLED_ROCKET")
            }

            m if m == meansOfDeath_t::MOD_ROCKET_HOMING as c_int
                || m == meansOfDeath_t::MOD_ROCKET_HOMING_SPLASH as c_int =>
            {
                Some("KILLED_ROCKET_HOMING")
            }

            m if m == meansOfDeath_t::MOD_THERMAL as c_int
                || m == meansOfDeath_t::MOD_THERMAL_SPLASH as c_int =>
            {
                Some("KILLED_THERMAL")
            }

            m if m == meansOfDeath_t::MOD_TRIP_MINE_SPLASH as c_int => Some("KILLED_TRIPMINE"),

            m if m == meansOfDeath_t::MOD_TIMED_MINE_SPLASH as c_int => {
                Some("KILLED_TRIPMINE_TIMED")
            }

            m if m == meansOfDeath_t::MOD_DET_PACK_SPLASH as c_int => Some("KILLED_DETPACK"),

            m if m == meansOfDeath_t::MOD_VEHICLE as c_int => {
                vehMessage = true;
                match ent.generic1 {
                    //primary blasters
                    WP_BLASTER => match ctx.world.bg_state.rng.Q_irand(0, 2) {
                        2 => Some("KILLED_VEH_BLASTER3"),
                        1 => Some("KILLED_VEH_BLASTER2"),
                        _ => Some("KILLED_VEH_BLASTER1"),
                    },

                    //missile
                    WP_ROCKET_LAUNCHER => {
                        if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                            Some("KILLED_VEH_MISSILE2")
                        } else {
                            Some("KILLED_VEH_MISSILE1")
                        }
                    }

                    //bomb
                    WP_THERMAL => Some("KILLED_VEH_BOMB"),

                    //ion cannon
                    WP_DEMP2 => Some("KILLED_VEH_ION"),

                    //turret
                    WP_TURRET => Some("KILLED_VEH_TURRET"),

                    _ => {
                        vehMessage = false;
                        Some("KILLED_GENERIC")
                    }
                }
            }

            m if m == meansOfDeath_t::MOD_CONC as c_int
                || m == meansOfDeath_t::MOD_CONC_ALT as c_int =>
            {
                Some("KILLED_GENERIC")
            }

            m if m == meansOfDeath_t::MOD_FORCE_DARK as c_int => Some("KILLED_DARKFORCE"),

            m if m == meansOfDeath_t::MOD_SENTRY as c_int => Some("KILLED_SENTRY"),

            m if m == meansOfDeath_t::MOD_TELEFRAG as c_int => Some("KILLED_TELEFRAG"),

            m if m == meansOfDeath_t::MOD_CRUSH as c_int => Some("KILLED_GENERIC"), //"KILLED_FORCETOSS"

            m if m == meansOfDeath_t::MOD_FALLING as c_int => Some("KILLED_FORCETOSS"),

            m if m == meansOfDeath_t::MOD_COLLISION as c_int
                || m == meansOfDeath_t::MOD_VEH_EXPLOSION as c_int =>
            {
                let msg = match ctx.world.bg_state.rng.Q_irand(0, 2) {
                    1 => "KILLED_VEH_COLLISION2",
                    2 => "KILLED_VEH_COLLISION3",
                    // Raven's `default:` falls into `case 0:`
                    _ => "KILLED_VEH_COLLISION1",
                };
                vehMessage = true;
                Some(msg)
            }

            m if m == meansOfDeath_t::MOD_TRIGGER_HURT as c_int => Some("KILLED_GENERIC"), //"KILLED_FORCETOSS"

            m if m == meansOfDeath_t::MOD_TARGET_LASER as c_int => {
                let msg = if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                    "KILLED_TURRET1"
                } else {
                    "KILLED_TURRET2"
                };
                vehMessage = true;
                Some(msg)
            }

            _ => Some("KILLED_GENERIC"),
        };

        if let Some(msg) = message {
            let text = if vehMessage {
                CG_GetStringEdString(ctx, "MP_INGAMEVEH", msg)
            } else {
                CG_GetStringEdString(ctx, "MP_INGAME", msg)
            };

            CG_Printf(ctx, &format!("{targetName} "));
            if !targetVehName.is_empty() {
                CG_Printf(ctx, &format!("({targetVehName}) "));
            }
            if r#mod == meansOfDeath_t::MOD_TARGET_LASER as c_int {
                //no attacker name, just a turbolaser or other kind of turret...
                CG_Printf(ctx, &text);
            } else {
                CG_Printf(ctx, &format!("{text} {attackerName}"));

                if !attackerVehName.is_empty() && !attackerVehWeapName.is_empty() {
                    CG_Printf(ctx, &format!(" ({attackerVehName} {attackerVehWeapName})"));
                } else if !attackerVehName.is_empty() {
                    CG_Printf(ctx, &format!(" ({attackerVehName})"));
                } else if !attackerVehWeapName.is_empty() {
                    CG_Printf(ctx, &format!(" ({attackerVehWeapName})"));
                }
            }
            CG_Printf(ctx, "\n");
            return;
        }
    }

    // we don't know what it was
    let died = CG_GetStringEdString(ctx, "MP_INGAME", "DIED_GENERIC");
    CG_Printf(ctx, &format!("{targetName} {died}\n"));
}

/// Raven `CG_ItemPickup` — latches the pickup HUD blend, runs the
/// `cg_autoswitch` "is this a better gun?" rule, and prints the pickup line.
///
/// §F19: Raven derefs `cg.snap` for the weapon compare behind its own NULL
/// test; the port reads the two `ps` fields it needs through `snap_ref` and
/// skips the whole autoswitch block when there is no snapshot, same as Raven.
///
/// Source: `oracle/codemp/cgame/cg_event.c:753-832`
pub fn CG_ItemPickup(ctx: &mut CgContext, itemNum: c_int) {
    let time = ctx.world.cg.time;
    ctx.world.cg.itemPickup = itemNum;
    ctx.world.cg.itemPickupTime = time;
    ctx.world.cg.itemPickupBlendTime = time;

    let item = &bg_itemlist[itemNum as usize];

    // see if it should be the grabbed weapon
    let snapPs = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.weapon, snap.ps.emplacedIndex));

    if let Some((psWeapon, psEmplacedIndex)) = snapPs {
        if item.giType() == IT_WEAPON {
            // 0 == no switching
            // 1 == automatically switch to best SAFE weapon
            // 2 == automatically switch to best weapon, safe or otherwise
            // 3 == if not saber, automatically switch to best weapon, safe or otherwise

            let cg_autoswitch = ctx.world.cvars.cg_autoswitch.integer;
            if cg_autoswitch == 0 {
                // don't switch
            } else if cg_autoswitch == 1 {
                //only autoselect if not explosive ("safe")
                if item.giTag() != WP_TRIP_MINE
                    && item.giTag() != WP_DET_PACK
                    && item.giTag() != WP_THERMAL
                    && item.giTag() != WP_ROCKET_LAUNCHER
                    && item.giTag() > psWeapon
                    && psWeapon != WP_SABER
                {
                    if psEmplacedIndex == 0 {
                        ctx.world.cg.weaponSelectTime = time;
                    }
                    ctx.world.cg.weaponSelect = item.giTag();
                }
            } else if cg_autoswitch == 2 {
                //autoselect if better
                if item.giTag() > psWeapon && psWeapon != WP_SABER {
                    if psEmplacedIndex == 0 {
                        ctx.world.cg.weaponSelectTime = time;
                    }
                    ctx.world.cg.weaponSelect = item.giTag();
                }
            }
            /*
            else if ( cg_autoswitch.integer == 3)
            { //autoselect if better and not using the saber as a weapon
                if (bg_itemlist[itemNum].giTag > cg.snap->ps.weapon &&
                    cg.snap->ps.weapon != WP_SABER)
                {
                    if (!cg.snap->ps.emplacedIndex)
                    {
                        cg.weaponSelectTime = cg.time;
                    }
                    cg.weaponSelect = bg_itemlist[itemNum].giTag;
                }
            }
            */
            //No longer required - just not switching ever if using saber
        }
    }

    //rww - print pickup messages
    if !item.classname.is_empty()
        && (item.giType() != IT_TEAM || (item.giTag() != PW_REDFLAG && item.giTag() != PW_BLUEFLAG))
    {
        //don't print messages for flags, they have their own pickup event broadcasts
        let upperKey = item.classname.to_ascii_uppercase();
        let key = format!("SP_INGAME_{upperKey}");
        let text = trap::SP_GetStringTextString(ctx.engine, &key, 1024);
        let pickupLine = CG_GetStringEdString(ctx, "MP_INGAME", "PICKUPLINE");

        if let Some(text) = text {
            Com_Printf(ctx, &format!("{pickupLine} {text}\n"));
        } else {
            Com_Printf(ctx, &format!("{pickupLine} {}\n", item.classname));
        }
    }
}

/// Raven `CG_PrintCTFMessage` — prints one CTF flag-event line, splicing the
/// team name into the localized string's `%s` when it has one.
///
/// Raven's `clientInfo_t *ci` becomes the `cgs.clientinfo` slot index (NULL ->
/// `None`) so the fn can take `&mut CgContext` without holding a borrow into
/// the array across the print; only `ci->name` is ever read.
///
/// Source: `oracle/codemp/cgame/cg_event.c:956-1042`
pub fn CG_PrintCTFMessage(
    ctx: &mut CgContext,
    ci: Option<usize>,
    teamName: Option<&str>,
    ctfMessage: c_int,
) {
    let refName = match ctfMessage {
        m if m == ctfMsg_t::CTFMESSAGE_FRAGGED_FLAG_CARRIER as c_int => "FRAGGED_FLAG_CARRIER",
        m if m == ctfMsg_t::CTFMESSAGE_FLAG_RETURNED as c_int => "FLAG_RETURNED",
        m if m == ctfMsg_t::CTFMESSAGE_PLAYER_RETURNED_FLAG as c_int => "PLAYER_RETURNED_FLAG",
        m if m == ctfMsg_t::CTFMESSAGE_PLAYER_CAPTURED_FLAG as c_int => "PLAYER_CAPTURED_FLAG",
        m if m == ctfMsg_t::CTFMESSAGE_PLAYER_GOT_FLAG as c_int => "PLAYER_GOT_FLAG",
        _ => return,
    };

    let psStringEDString = CG_GetStringEdString(ctx, "MP_INGAME", refName);

    if psStringEDString.is_empty() {
        return;
    }

    let ciName = ci.map(|n| {
        buf_to_string(
            &ctx.world.cgs.clientinfo[n]
                .name
                .iter()
                .map(|&c| c as u8)
                .collect::<Vec<u8>>(),
        )
    });

    if let Some(teamName) = teamName {
        if !teamName.is_empty() && psStringEDString.contains("%s") {
            let mut printMsg = String::new();

            if let Some(name) = &ciName {
                printMsg = format!("{name} ");
            }

            let src: Vec<char> = psStringEDString.chars().collect();
            let mut i = 0;
            while i < src.len() && i < 512 {
                if src[i] == '%' && src.get(i + 1) == Some(&'s') {
                    printMsg.push_str(teamName);

                    i += 1;
                } else {
                    printMsg.push(src[i]);
                }

                i += 1;
            }

            Com_Printf(ctx, &format!("{printMsg}\n"));
            return;
        }
    }

    let printMsg = match &ciName {
        Some(name) => format!("{name} {psStringEDString}"),
        None => psStringEDString,
    };
    // Com_sprintf into `printMsg[1024]` - one Latin-1 char is one C byte
    let printMsg: String = printMsg.chars().take(1023).collect();

    Com_Printf(ctx, &format!("{printMsg}\n"));
}

/// Raven `CG_PainEvent` — plays a health-banded pain grunt for an entity
/// (throttled to two a second) and flips its programmatic twitch direction.
///
/// Source: `oracle/codemp/cgame/cg_event.c:842-865`
pub fn CG_PainEvent(ctx: &mut CgContext, centNum: usize, health: c_int) {
    let time = ctx.world.cg.time;
    let painTime = ctx.world.entity(centNum).pe.painTime;

    // don't do more than two pain sounds a second
    if time - painTime < 500 {
        return;
    }

    let snd = if health < 25 {
        "*pain25.wav"
    } else if health < 50 {
        "*pain50.wav"
    } else if health < 75 {
        "*pain75.wav"
    } else {
        "*pain100.wav"
    };

    let number = ctx.world.entity(centNum).currentState.number;
    let custom = CG_CustomSound(ctx, number, snd);
    trap::S_StartSound(ctx.engine, None, number, CHAN_VOICE, custom);

    // save pain time for programitic twitch animation
    let cent = ctx.world.entity_mut(centNum);
    cent.pe.painTime = time;
    cent.pe.painDirection ^= 1;
}

/// Raven `CG_GetCTFMessageEvent` — resolves the flag-event's client/team
/// indices into a `clientinfo_t` slot and a team label, then hands off to
/// [`CG_PrintCTFMessage`].
///
/// §F19: Raven's `clIndex < MAX_CLIENTS` guards only the upper bound; a
/// negative `trickedentindex` would index `cgs.clientinfo` out of bounds (UB)
/// in C. The port adds a `clIndex >= 0` check and treats a miss as "no
/// client", matching the fn's own `if (!ci) return;` early-out.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1044-1067`
pub fn CG_GetCTFMessageEvent(ctx: &mut CgContext, es: &entityState_t) {
    let clIndex = es.trickedentindex;
    let teamIndex = es.trickedentindex2;

    let ci = if clIndex >= 0 && clIndex < MAX_CLIENTS_I32 {
        Some(clIndex as usize)
    } else {
        None
    };

    let teamName = if teamIndex < 50 {
        Some(CG_TeamName(teamIndex))
    } else {
        None
    };

    let Some(ci) = ci else {
        return;
    };

    CG_PrintCTFMessage(ctx, Some(ci), teamName, es.eventParm);
}

/// Raven `DoFall` — picks the landing sound (corpse crack, knockdown thud, or
/// footfall) from the fall delta and, for the local player, smooths the
/// screen's landing-Z bob.
///
/// The `_XBOX`-gated rumble tail (`FF_XboxDamage`) never compiled into the MP
/// build and is dropped, not transcribed.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1078-1168`
pub fn DoFall(ctx: &mut CgContext, centNum: usize, es: &entityState_t, clientNum: c_int) {
    let delta = es.eventParm;

    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    let currentNumber = ctx.world.entity(centNum).currentState.number;

    if eFlags & EF_DEAD != 0 {
        //corpses crack into the ground ^_^
        if delta > 25 {
            let sfx = ctx.world.cgs.media.fallSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        } else {
            let sfx = trap::S_RegisterSound(ctx.engine, "sound/movers/objects/objectHit.wav");
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }
    } else if BG_InKnockDownOnly(es.legsAnim) != qfalse {
        if delta > 14 {
            let sfx = ctx.world.cgs.media.fallSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        } else {
            let sfx = trap::S_RegisterSound(ctx.engine, "sound/movers/objects/objectHit.wav");
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }
    } else if delta > 50 {
        let sfx = ctx.world.cgs.media.fallSound;
        trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        let custom = CG_CustomSound(ctx, currentNumber, "*land1.wav");
        trap::S_StartSound(ctx.engine, None, currentNumber, CHAN_VOICE, custom);
        let time = ctx.world.cg.time;
        // don't play a pain sound right after this
        ctx.world.entity_mut(centNum).pe.painTime = time;
    } else if delta > 44 {
        let sfx = ctx.world.cgs.media.fallSound;
        trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        let custom = CG_CustomSound(ctx, currentNumber, "*land1.wav");
        trap::S_StartSound(ctx.engine, None, currentNumber, CHAN_VOICE, custom);
        let time = ctx.world.cg.time;
        // don't play a pain sound right after this
        ctx.world.entity_mut(centNum).pe.painTime = time;
    } else {
        let sfx = ctx.world.cgs.media.landSound;
        trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
    }

    if clientNum == ctx.world.cg.predictedPlayerState.clientNum {
        // smooth landing z changes
        ctx.world.cg.landChange = -delta as f32;
        if ctx.world.cg.landChange > 32.0 {
            ctx.world.cg.landChange = 32.0;
        }
        if ctx.world.cg.landChange < -32.0 {
            ctx.world.cg.landChange = -32.0;
        }
        ctx.world.cg.landTime = ctx.world.cg.time;
    }
}

/// Raven `CG_TryPlayCustomSound` — plays a per-model custom sound at
/// `origin`, silently doing nothing when the entity has no override for it.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1206-1216`
pub fn CG_TryPlayCustomSound(
    ctx: &mut CgContext,
    origin: Option<&vec3_t>,
    entityNum: c_int,
    channel: c_int,
    soundName: &str,
) {
    let cSound = CG_CustomSound(ctx, entityNum, soundName);

    if cSound <= 0 {
        return;
    }

    trap::S_StartSound(ctx.engine, origin, entityNum, channel, cSound);
}

/// Raven `CG_G2MarkEvent` — projects a ghoul2 scorch/burn decal at a
/// projectile's impact point, re-tracing to the surface first when the
/// server flagged a radius-explosion source.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1218-1348`
pub fn CG_G2MarkEvent(ctx: &mut CgContext, es: &entityState_t) {
    // es->origin should be the hit location of the projectile,
    // whereas es->origin2 is the predicted position of the
    // projectile. (based on the trajectory upon impact) -rww
    let ownerNum = es.otherEntityNum as usize;

    if ctx.world.entity(ownerNum).ghoul2.is_null() {
        //can't do anything then...
        return;
    }

    // es->eventParm being non-0 means to do a special trace check
    // first. This will give us an impact right at the surface to
    // project the mark on. Typically this is used for radius
    // explosions and such, where the source position could be
    // way outside of model space.
    let startPoint: vec3_t = if es.eventParm != 0 {
        let mut tr = trace_t::zeroed();
        let mut ignore = ENTITYNUM_NONE;

        CG_G2Trace(
            ctx,
            &mut tr,
            &es.origin,
            &vec3_origin,
            &vec3_origin,
            &es.origin2,
            ignore,
            MASK_PLAYERSOLID,
        );

        if tr.entityNum as c_int != es.otherEntityNum {
            //try again if we hit an ent but not the one we wanted.
            if (tr.entityNum as c_int) < ENTITYNUM_WORLD {
                ignore = tr.entityNum as c_int;
                CG_G2Trace(
                    ctx,
                    &mut tr,
                    &es.origin,
                    &vec3_origin,
                    &vec3_origin,
                    &es.origin2,
                    ignore,
                    MASK_PLAYERSOLID,
                );
                if tr.entityNum as c_int != es.otherEntityNum {
                    //try extending the trace a bit.. or not
                    //didn't manage to collide with the desired person. No mark will be placed then.
                    return;
                }
            }
        }

        //otherwise we now have a valid starting point.
        tr.endpos
    } else {
        es.origin
    };

    let mut size: f32 = 0.0;
    let mut shader: c_int = 0;

    if es.eFlags & EF_JETPACK_ACTIVE != 0 {
        // a vehicle weapon, make it a larger size mark
        //OR base this on the size of the thing you hit?
        // §F19: Raven indexes `g_vehWeaponInfo[otherEntityNum2]` unchecked; an
        // out-of-range index reads as the zeroed entry here rather than OOB
        // memory.
        let vw = ctx
            .world
            .bg_state
            .g_vehWeaponInfo
            .get(es.otherEntityNum2 as usize);
        let (markSize, markShader) =
            vw.map_or((0.0, 0), |w| (w.fG2MarkSize, w.iG2MarkShaderHandle));

        if markSize != 0.0 {
            size = ctx.world.bg_state.rng.flrand(0.6, 1.4) * markSize;
        } else {
            size = ctx.world.bg_state.rng.flrand(32.0, 72.0);
        }
        //specify mark shader in vehWeapon file
        if markShader != 0 {
            //have one we want to use instead of defaults
            shader = markShader;
        }
    }

    match es.weapon {
        WP_BRYAR_PISTOL | WP_CONCUSSION | WP_BRYAR_OLD | WP_BLASTER | WP_DISRUPTOR
        | WP_BOWCASTER | WP_REPEATER | WP_TURRET => {
            if size == 0.0 {
                size = 4.0;
            }
            if shader == 0 {
                shader = ctx.world.cgs.media.bdecal_bodyburn1;
            }

            let owner = ctx.world.entity(ownerNum);
            let ownerGhoul2 = owner.ghoul2;
            let ownerLerpOrigin = owner.lerpOrigin;
            let ownerLerpYaw = owner.lerpAngles[YAW];
            let mut ownerScale = owner.modelScale;
            let lifeTime = ctx.world.bg_state.rng.Q_irand(10000, 20000);

            CG_AddGhoul2Mark(
                ctx,
                shader,
                size,
                &startPoint,
                &es.origin2,
                es.owner,
                &ownerLerpOrigin,
                ownerLerpYaw,
                ownerGhoul2,
                &mut ownerScale,
                lifeTime,
            );
            // the callee mutates the caller's scale vector in place (see the
            // PORT-NOTE on `CG_AddGhoul2Mark`'s swapped-argument bug) - write
            // the (possibly stomped) result back same as Raven's pointer would.
            ctx.world.entity_mut(ownerNum).modelScale = ownerScale;
        }

        WP_ROCKET_LAUNCHER | WP_THERMAL => {
            if size == 0.0 {
                size = 24.0;
            }
            if shader == 0 {
                shader = ctx.world.cgs.media.bdecal_burn1;
            }

            let owner = ctx.world.entity(ownerNum);
            let ownerGhoul2 = owner.ghoul2;
            let ownerLerpOrigin = owner.lerpOrigin;
            let ownerLerpYaw = owner.lerpAngles[YAW];
            let mut ownerScale = owner.modelScale;
            let lifeTime = ctx.world.bg_state.rng.Q_irand(10000, 20000);

            CG_AddGhoul2Mark(
                ctx,
                shader,
                size,
                &startPoint,
                &es.origin2,
                es.owner,
                &ownerLerpOrigin,
                ownerLerpYaw,
                ownerGhoul2,
                &mut ownerScale,
                lifeTime,
            );
            ctx.world.entity_mut(ownerNum).modelScale = ownerScale;
        }

        //Issues with small scale?
        _ => {}
    }
}

/// Resolves the `clientInfo_t` an `EV_SABER_*` event should read custom
/// blade fx/sounds from: the entity's own `npcClient` for an NPC, else the
/// `cgs.clientinfo` slot. `None` when there is no valid slot.
///
/// §F19: Raven only bounds-checks the upper `< MAX_CLIENTS`; the negative check
/// keeps a server-supplied index from reaching past `cgs.clientinfo`.
fn eventSaberClient(world: &CgWorld, entNum: c_int, isNpc: bool) -> Option<&clientInfo_t> {
    if isNpc {
        world.entity(entNum as usize).npcClient.as_deref()
    } else if entNum >= 0 && entNum < MAX_CLIENTS_I32 {
        Some(&world.cgs.clientinfo[entNum as usize])
    } else {
        None
    }
}

/// Raven `CG_EntityEvent` — the client-side event dispatcher: turns one entity's
/// `entityState_t.event` into its sound / effect / HUD / ghoul2 consequence.
///
/// `ds` threads in for the one chat-box call (EV_VOICECMD_SOUND); `position` is
/// the caller's resolved event origin.
///
/// Raven's top-level `ci` (`cgs.clientinfo[clientNum]` / `cent->npcClient`) is a
/// dead store — every arm that needs client info rebinds its own local — so only
/// the ET_NPC `npcClient` allocation side-effect and `clientNum = es->number`
/// survive from that setup.
///
/// §F19: many arms deref `cg.snap` unchecked; before the first snapshot the
/// `snap_ref` `None` arm takes the no-op.
///
/// Source: `oracle/codemp/cgame/cg_event.c:1491-3691`
pub fn CG_EntityEvent(ctx: &mut CgContext, ds: &DisplayState, centNum: usize, position: &vec3_t) {
    let es = ctx.world.entity(centNum).currentState;
    let event = es.event & !EV_EVENT_BITS;

    if ctx.world.cvars.cg_debugEvents.integer != 0 {
        CG_Printf(ctx, &format!("ent:{:3}  event:{:3} ", es.number, event));
    }

    if event == 0 {
        DEBUGNAME(ctx, "ZEROEVENT");
        return;
    }

    let mut clientNum = es.clientNum;
    if clientNum < 0 || clientNum >= MAX_CLIENTS_I32 {
        clientNum = 0;
    }

    if es.eType == entityType_t::ET_NPC as c_int {
        clientNum = es.number;

        if ctx.world.entity(centNum).npcClient.is_none() {
            // allocate memory for it; CG_CreateNPCClient hands back a zeroed
            // clientInfo_t (ghoul2Model already NULL). Raven's alloc-failure
            // `assert(0); return;` can't fire — `Box` never returns null.
            ctx.world.entity_mut(centNum).npcClient = Some(CG_CreateNPCClient());
        }
        // Raven's `ci = cent->npcClient; assert(ci)` is a dead store here.
    }
    // else: Raven's `ci = &cgs.clientinfo[clientNum]` is a pure dead store; ci
    // is never read below (each arm rebinds its own), so it is dropped.

    let ppsClientNum = ctx.world.cg.predictedPlayerState.clientNum;

    match event {
        //
        // movement generated events
        //
        v if v == entity_event_t::EV_CLIENTJOIN as c_int => {
            DEBUGNAME(ctx, "EV_CLIENTJOIN");

            //Slight hack to force a local reinit of client entity on join.
            //cl_ent is `&cg_entities[es->eventParm]` — always a valid ref, so
            //Raven's `if (cl_ent)` is always true.
            let cl_ent = ctx.world.entity_mut(es.eventParm as usize);
            //cl_ent->torsoBolt = 0;
            cl_ent.bolt1 = 0;
            cl_ent.bolt2 = 0;
            cl_ent.bolt3 = 0;
            cl_ent.bolt4 = 0;
            cl_ent.bodyHeight = 0.0; //SABER_LENGTH_MAX;
                                     //cl_ent->saberExtendTime = 0;
            cl_ent.boltInfo = 0;
            cl_ent.frame_minus1_refreshed = 0;
            cl_ent.frame_minus2_refreshed = 0;
            cl_ent.frame_hold_time = 0;
            cl_ent.frame_hold_refreshed = 0;
            cl_ent.trickAlpha = 0;
            cl_ent.trickAlphaTime = 0;
            cl_ent.ghoul2weapon = null_mut();
            cl_ent.weapon = WP_NONE;
            cl_ent.teamPowerEffectTime = 0;
            cl_ent.teamPowerType = 0;
            cl_ent.numLoopingSounds = 0;
            //cl_ent->localAnimIndex = 0;
        }

        v if v == entity_event_t::EV_FOOTSTEP as c_int => {
            DEBUGNAME(ctx, "EV_FOOTSTEP");
            if ctx.world.cvars.cg_footsteps.integer != 0 {
                let soundType = match es.eventParm {
                    m if m == MATERIAL_MUD => footstep_t::FOOTSTEP_MUDWALK,
                    m if m == MATERIAL_DIRT => footstep_t::FOOTSTEP_DIRTWALK,
                    m if m == MATERIAL_SAND => footstep_t::FOOTSTEP_SANDWALK,
                    m if m == MATERIAL_SNOW => footstep_t::FOOTSTEP_SNOWWALK,
                    m if m == MATERIAL_SHORTGRASS || m == MATERIAL_LONGGRASS => {
                        footstep_t::FOOTSTEP_GRASSWALK
                    }
                    m if m == MATERIAL_SOLIDMETAL => footstep_t::FOOTSTEP_METALWALK,
                    m if m == MATERIAL_HOLLOWMETAL => footstep_t::FOOTSTEP_PIPEWALK,
                    m if m == MATERIAL_GRAVEL => footstep_t::FOOTSTEP_GRAVELWALK,
                    m if m == MATERIAL_CARPET
                        || m == MATERIAL_FABRIC
                        || m == MATERIAL_CANVAS
                        || m == MATERIAL_RUBBER
                        || m == MATERIAL_PLASTIC =>
                    {
                        footstep_t::FOOTSTEP_RUGWALK
                    }
                    m if m == MATERIAL_SOLIDWOOD || m == MATERIAL_HOLLOWWOOD => {
                        footstep_t::FOOTSTEP_WOODWALK
                    }
                    _ => footstep_t::FOOTSTEP_STONEWALK,
                };

                let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
                let sfx = ctx.world.cgs.media.footsteps[soundType as usize][idx];
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, sfx);
            }
        }

        v if v == entity_event_t::EV_FOOTSTEP_METAL as c_int => {
            DEBUGNAME(ctx, "EV_FOOTSTEP_METAL");
            if ctx.world.cvars.cg_footsteps.integer != 0 {
                let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
                let sfx =
                    ctx.world.cgs.media.footsteps[footstep_t::FOOTSTEP_METALWALK as usize][idx];
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, sfx);
            }
        }

        v if v == entity_event_t::EV_FOOTSPLASH as c_int => {
            DEBUGNAME(ctx, "EV_FOOTSPLASH");
            if ctx.world.cvars.cg_footsteps.integer != 0 {
                let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
                let sfx = ctx.world.cgs.media.footsteps[footstep_t::FOOTSTEP_SPLASH as usize][idx];
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, sfx);
            }
        }

        v if v == entity_event_t::EV_FOOTWADE as c_int => {
            DEBUGNAME(ctx, "EV_FOOTWADE");
            if ctx.world.cvars.cg_footsteps.integer != 0 {
                let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
                let sfx = ctx.world.cgs.media.footsteps[footstep_t::FOOTSTEP_SPLASH as usize][idx];
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, sfx);
            }
        }

        v if v == entity_event_t::EV_SWIM as c_int => {
            DEBUGNAME(ctx, "EV_SWIM");
            if ctx.world.cvars.cg_footsteps.integer != 0 {
                let idx = (ctx.world.bg_state.rng.rand() & 3) as usize;
                let sfx = ctx.world.cgs.media.footsteps[footstep_t::FOOTSTEP_SPLASH as usize][idx];
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, sfx);
            }
        }

        v if v == entity_event_t::EV_FALL as c_int => {
            DEBUGNAME(ctx, "EV_FALL");
            let skip = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.number == s.ps.clientNum && s.ps.fallingToDeath != 0);
            if !skip {
                DoFall(ctx, centNum, &es, clientNum);
            }
        }

        v if v == entity_event_t::EV_STEP_4 as c_int
            || v == entity_event_t::EV_STEP_8 as c_int
            || v == entity_event_t::EV_STEP_12 as c_int
            || v == entity_event_t::EV_STEP_16 as c_int =>
        {
            // smooth out step up transitions
            DEBUGNAME(ctx, "EV_STEP");
            'step: {
                if clientNum != ctx.world.cg.predictedPlayerState.clientNum {
                    break 'step;
                }
                // if we are interpolating, we don't need to smooth steps
                let interp = ctx.world.cg.demoPlayback != qfalse
                    || ctx
                        .world
                        .cg
                        .snap_ref()
                        .is_some_and(|s| (s.ps.pm_flags & PMF_FOLLOW) != 0)
                    || ctx.world.cvars.cg_nopredict.integer != 0
                    || ctx.world.cvars.cg_synchronousClients.integer != 0;
                if interp {
                    break 'step;
                }
                // check for stepping up before a previous step is completed
                let delta = ctx.world.cg.time - ctx.world.cg.stepTime;
                let oldStep = if delta < STEP_TIME {
                    ctx.world.cg.stepChange * (STEP_TIME - delta) as f32 / STEP_TIME as f32
                } else {
                    0.0
                };

                // add this amount
                let step = 4 * (event - entity_event_t::EV_STEP_4 as c_int + 1);
                ctx.world.cg.stepChange = oldStep + step as f32;
                if ctx.world.cg.stepChange > MAX_STEP_CHANGE as f32 {
                    ctx.world.cg.stepChange = MAX_STEP_CHANGE as f32;
                }
                ctx.world.cg.stepTime = ctx.world.cg.time;
            }
        }

        v if v == entity_event_t::EV_JUMP_PAD as c_int => {
            DEBUGNAME(ctx, "EV_JUMP_PAD");
        }

        v if v == entity_event_t::EV_GHOUL2_MARK as c_int => {
            DEBUGNAME(ctx, "EV_GHOUL2_MARK");
            if ctx.world.cvars.cg_ghoul2Marks.integer != 0 {
                //Can we put a burn mark on him?
                CG_G2MarkEvent(ctx, &es);
            }
        }

        v if v == entity_event_t::EV_GLOBAL_DUEL as c_int => {
            DEBUGNAME(ctx, "EV_GLOBAL_DUEL");
            //used for beginning of power duels
            if es.otherEntityNum == ppsClientNum
                || es.otherEntityNum2 == ppsClientNum
                || es.groundEntityNum == ppsClientNum
            {
                let s = CG_GetStringEdString(ctx, "MP_SVGAME", "BEGIN_DUEL");
                CG_CenterPrint(ctx.world, &s, 120, GIANTCHAR_WIDTH * 2);
                let sfx = ctx.world.cgs.media.countFightSound;
                trap::S_StartLocalSound(ctx.engine, sfx, CHAN_ANNOUNCER);
            }
        }

        v if v == entity_event_t::EV_PRIVATE_DUEL as c_int => {
            DEBUGNAME(ctx, "EV_PRIVATE_DUEL");
            'duel: {
                let is_me = ctx
                    .world
                    .cg
                    .snap_ref()
                    .is_some_and(|s| s.ps.clientNum == es.number);
                if !is_me {
                    break 'duel;
                }

                if es.eventParm != 0 {
                    //starting the duel
                    if es.eventParm == 2 {
                        let s = CG_GetStringEdString(ctx, "MP_SVGAME", "BEGIN_DUEL");
                        CG_CenterPrint(ctx.world, &s, 120, GIANTCHAR_WIDTH * 2);
                        let sfx = ctx.world.cgs.media.countFightSound;
                        trap::S_StartLocalSound(ctx.engine, sfx, CHAN_ANNOUNCER);
                    } else {
                        trap::S_StartBackgroundTrack(
                            ctx.engine,
                            "music/mp/duel.mp3",
                            "music/mp/duel.mp3",
                            false,
                        );
                    }
                } else {
                    //ending the duel
                    CG_StartMusic(ctx, true);
                }
            }
        }

        v if v == entity_event_t::EV_JUMP as c_int => {
            DEBUGNAME(ctx, "EV_JUMP");
            if ctx.world.cvars.cg_jumpSounds.integer != 0 {
                let custom = CG_CustomSound(ctx, es.number, "*jump1.wav");
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_VOICE, custom);
            }
        }

        v if v == entity_event_t::EV_ROLL as c_int => {
            DEBUGNAME(ctx, "EV_ROLL");
            'roll: {
                let dead_fall = ctx
                    .world
                    .cg
                    .snap_ref()
                    .is_some_and(|s| es.number == s.ps.clientNum && s.ps.fallingToDeath != 0);
                if dead_fall {
                    break 'roll;
                }
                if es.eventParm != 0 {
                    //fall-roll-in-one event
                    DoFall(ctx, centNum, &es, clientNum);
                }

                let custom = CG_CustomSound(ctx, es.number, "*jump1.wav");
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_VOICE, custom);
                let rollSound = ctx.world.cgs.media.rollSound;
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_BODY, rollSound);
                //FIXME: need some sort of body impact on ground sound and maybe kick up some dust?
            }
        }

        v if v == entity_event_t::EV_TAUNT as c_int => {
            DEBUGNAME(ctx, "EV_TAUNT");
            let mut soundIndex = 0;
            if ctx.world.cgs.gametype != GT_DUEL
                && ctx.world.cgs.gametype != GT_POWERDUEL
                && es.eventParm == TAUNT_TAUNT
            {
                //normal taunt
                soundIndex = CG_CustomSound(ctx, es.number, "*taunt.wav");
            } else {
                match es.eventParm {
                    TAUNT_BOW => {
                        //soundIndex = CG_CustomSound( es->number, va("*respect%d.wav", Q_irand(1,3)) );
                    }
                    TAUNT_MEDITATE => {
                        //soundIndex = CG_CustomSound( es->number, va("*meditate%d.wav", Q_irand(1,3)) );
                    }
                    TAUNT_FLOURISH => {
                        if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                            soundIndex =
                                CG_CustomSound(ctx, es.number, &format!("*deflect{r}.wav"));
                            if soundIndex == 0 {
                                let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                                soundIndex =
                                    CG_CustomSound(ctx, es.number, &format!("*gloat{r}.wav"));
                                if soundIndex == 0 {
                                    let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                                    soundIndex =
                                        CG_CustomSound(ctx, es.number, &format!("*anger{r}.wav"));
                                }
                            }
                        } else {
                            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                            soundIndex = CG_CustomSound(ctx, es.number, &format!("*gloat{r}.wav"));
                            if soundIndex == 0 {
                                let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                                soundIndex =
                                    CG_CustomSound(ctx, es.number, &format!("*deflect{r}.wav"));
                                if soundIndex == 0 {
                                    let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                                    soundIndex =
                                        CG_CustomSound(ctx, es.number, &format!("*anger{r}.wav"));
                                }
                            }
                        }
                    }
                    TAUNT_GLOAT => {
                        let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                        soundIndex = CG_CustomSound(ctx, es.number, &format!("*victory{r}.wav"));
                    }
                    // TAUNT_TAUNT and any other value both land here.
                    _ => {
                        if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                            soundIndex = CG_CustomSound(ctx, es.number, &format!("*anger{r}.wav"));
                        } else {
                            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                            soundIndex = CG_CustomSound(ctx, es.number, &format!("*taunt{r}.wav"));
                            if soundIndex == 0 {
                                let r = ctx.world.bg_state.rng.Q_irand(1, 3);
                                soundIndex =
                                    CG_CustomSound(ctx, es.number, &format!("*anger{r}.wav"));
                            }
                        }
                    }
                }
            }
            if soundIndex == 0 {
                soundIndex = CG_CustomSound(ctx, es.number, "*taunt.wav");
            }
            if soundIndex != 0 {
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_VOICE, soundIndex);
            }
        }

        //Begin NPC sounds
        v if v == entity_event_t::EV_ANGER1 as c_int
            || v == entity_event_t::EV_ANGER2 as c_int
            || v == entity_event_t::EV_ANGER3 as c_int =>
        {
            //Say when acquire an enemy when didn't have one before
            DEBUGNAME(ctx, "EV_ANGERx");
            let n = event - entity_event_t::EV_ANGER1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*anger{n}.wav"));
        }

        v if v == entity_event_t::EV_VICTORY1 as c_int
            || v == entity_event_t::EV_VICTORY2 as c_int
            || v == entity_event_t::EV_VICTORY3 as c_int =>
        {
            //Say when killed an enemy
            DEBUGNAME(ctx, "EV_VICTORYx");
            let n = event - entity_event_t::EV_VICTORY1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*victory{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_CONFUSE1 as c_int
            || v == entity_event_t::EV_CONFUSE2 as c_int
            || v == entity_event_t::EV_CONFUSE3 as c_int =>
        {
            //Say when confused
            DEBUGNAME(ctx, "EV_CONFUSEDx");
            let n = event - entity_event_t::EV_CONFUSE1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*confuse{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_PUSHED1 as c_int
            || v == entity_event_t::EV_PUSHED2 as c_int
            || v == entity_event_t::EV_PUSHED3 as c_int =>
        {
            //Say when pushed
            DEBUGNAME(ctx, "EV_PUSHEDx");
            let n = event - entity_event_t::EV_PUSHED1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*pushed{n}.wav"));
        }

        v if v == entity_event_t::EV_CHOKE1 as c_int
            || v == entity_event_t::EV_CHOKE2 as c_int
            || v == entity_event_t::EV_CHOKE3 as c_int =>
        {
            //Say when choking
            DEBUGNAME(ctx, "EV_CHOKEx");
            let n = event - entity_event_t::EV_CHOKE1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*choke{n}.wav"));
        }

        v if v == entity_event_t::EV_FFWARN as c_int => {
            //Warn ally to stop shooting you
            DEBUGNAME(ctx, "EV_FFWARN");
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, "*ffwarn.wav");
        }

        v if v == entity_event_t::EV_FFTURN as c_int => {
            //Turn on ally after being shot by them
            DEBUGNAME(ctx, "EV_FFTURN");
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, "*ffturn.wav");
        }

        //extra sounds for ST
        v if v == entity_event_t::EV_CHASE1 as c_int
            || v == entity_event_t::EV_CHASE2 as c_int
            || v == entity_event_t::EV_CHASE3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_CHASEx");
            let n = event - entity_event_t::EV_CHASE1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*chase{n}.wav"));
        }

        v if v == entity_event_t::EV_COVER1 as c_int
            || v == entity_event_t::EV_COVER2 as c_int
            || v == entity_event_t::EV_COVER3 as c_int
            || v == entity_event_t::EV_COVER4 as c_int
            || v == entity_event_t::EV_COVER5 as c_int =>
        {
            DEBUGNAME(ctx, "EV_COVERx");
            let n = event - entity_event_t::EV_COVER1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*cover{n}.wav"));
        }

        v if v == entity_event_t::EV_DETECTED1 as c_int
            || v == entity_event_t::EV_DETECTED2 as c_int
            || v == entity_event_t::EV_DETECTED3 as c_int
            || v == entity_event_t::EV_DETECTED4 as c_int
            || v == entity_event_t::EV_DETECTED5 as c_int =>
        {
            DEBUGNAME(ctx, "EV_DETECTEDx");
            let n = event - entity_event_t::EV_DETECTED1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*detected{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_GIVEUP1 as c_int
            || v == entity_event_t::EV_GIVEUP2 as c_int
            || v == entity_event_t::EV_GIVEUP3 as c_int
            || v == entity_event_t::EV_GIVEUP4 as c_int =>
        {
            DEBUGNAME(ctx, "EV_GIVEUPx");
            let n = event - entity_event_t::EV_GIVEUP1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*giveup{n}.wav"));
        }

        v if v == entity_event_t::EV_LOOK1 as c_int || v == entity_event_t::EV_LOOK2 as c_int => {
            DEBUGNAME(ctx, "EV_LOOKx");
            let n = event - entity_event_t::EV_LOOK1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*look{n}.wav"));
        }

        v if v == entity_event_t::EV_LOST1 as c_int => {
            DEBUGNAME(ctx, "EV_LOST1");
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, "*lost1.wav");
        }

        v if v == entity_event_t::EV_OUTFLANK1 as c_int
            || v == entity_event_t::EV_OUTFLANK2 as c_int =>
        {
            DEBUGNAME(ctx, "EV_OUTFLANKx");
            let n = event - entity_event_t::EV_OUTFLANK1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*outflank{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_ESCAPING1 as c_int
            || v == entity_event_t::EV_ESCAPING2 as c_int
            || v == entity_event_t::EV_ESCAPING3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_ESCAPINGx");
            let n = event - entity_event_t::EV_ESCAPING1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*escaping{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_SIGHT1 as c_int
            || v == entity_event_t::EV_SIGHT2 as c_int
            || v == entity_event_t::EV_SIGHT3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_SIGHTx");
            let n = event - entity_event_t::EV_SIGHT1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*sight{n}.wav"));
        }

        v if v == entity_event_t::EV_SOUND1 as c_int
            || v == entity_event_t::EV_SOUND2 as c_int
            || v == entity_event_t::EV_SOUND3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_SOUNDx");
            let n = event - entity_event_t::EV_SOUND1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*sound{n}.wav"));
        }

        v if v == entity_event_t::EV_SUSPICIOUS1 as c_int
            || v == entity_event_t::EV_SUSPICIOUS2 as c_int
            || v == entity_event_t::EV_SUSPICIOUS3 as c_int
            || v == entity_event_t::EV_SUSPICIOUS4 as c_int
            || v == entity_event_t::EV_SUSPICIOUS5 as c_int =>
        {
            DEBUGNAME(ctx, "EV_SUSPICIOUSx");
            let n = event - entity_event_t::EV_SUSPICIOUS1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*suspicious{n}.wav"),
            );
        }

        //extra sounds for Jedi
        v if v == entity_event_t::EV_COMBAT1 as c_int
            || v == entity_event_t::EV_COMBAT2 as c_int
            || v == entity_event_t::EV_COMBAT3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_COMBATx");
            let n = event - entity_event_t::EV_COMBAT1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*combat{n}.wav"));
        }

        v if v == entity_event_t::EV_JDETECTED1 as c_int
            || v == entity_event_t::EV_JDETECTED2 as c_int
            || v == entity_event_t::EV_JDETECTED3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_JDETECTEDx");
            let n = event - entity_event_t::EV_JDETECTED1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*jdetected{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_TAUNT1 as c_int
            || v == entity_event_t::EV_TAUNT2 as c_int
            || v == entity_event_t::EV_TAUNT3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_TAUNTx");
            let n = event - entity_event_t::EV_TAUNT1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*taunt{n}.wav"));
        }

        v if v == entity_event_t::EV_JCHASE1 as c_int
            || v == entity_event_t::EV_JCHASE2 as c_int
            || v == entity_event_t::EV_JCHASE3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_JCHASEx");
            let n = event - entity_event_t::EV_JCHASE1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*jchase{n}.wav"));
        }

        v if v == entity_event_t::EV_JLOST1 as c_int
            || v == entity_event_t::EV_JLOST2 as c_int
            || v == entity_event_t::EV_JLOST3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_JLOSTx");
            let n = event - entity_event_t::EV_JLOST1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*jlost{n}.wav"));
        }

        v if v == entity_event_t::EV_DEFLECT1 as c_int
            || v == entity_event_t::EV_DEFLECT2 as c_int
            || v == entity_event_t::EV_DEFLECT3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_DEFLECTx");
            let n = event - entity_event_t::EV_DEFLECT1 as c_int + 1;
            CG_TryPlayCustomSound(
                ctx,
                None,
                es.number,
                CHAN_VOICE,
                &format!("*deflect{n}.wav"),
            );
        }

        v if v == entity_event_t::EV_GLOAT1 as c_int
            || v == entity_event_t::EV_GLOAT2 as c_int
            || v == entity_event_t::EV_GLOAT3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_GLOATx");
            let n = event - entity_event_t::EV_GLOAT1 as c_int + 1;
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, &format!("*gloat{n}.wav"));
        }

        v if v == entity_event_t::EV_PUSHFAIL as c_int => {
            DEBUGNAME(ctx, "EV_PUSHFAIL");
            CG_TryPlayCustomSound(ctx, None, es.number, CHAN_VOICE, "*pushfail.wav");
        }
        //End NPC sounds
        v if v == entity_event_t::EV_SIEGESPEC as c_int => {
            DEBUGNAME(ctx, "EV_SIEGESPEC");
            if es.owner == ppsClientNum {
                ctx.world.draw.cg_siegeDeathTime = es.time;
            }
        }

        v if v == entity_event_t::EV_WATER_TOUCH as c_int => {
            DEBUGNAME(ctx, "EV_WATER_TOUCH");
            let sfx = ctx.world.cgs.media.watrInSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }

        v if v == entity_event_t::EV_WATER_LEAVE as c_int => {
            DEBUGNAME(ctx, "EV_WATER_LEAVE");
            let sfx = ctx.world.cgs.media.watrOutSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }

        v if v == entity_event_t::EV_WATER_UNDER as c_int => {
            DEBUGNAME(ctx, "EV_WATER_UNDER");
            let sfx = ctx.world.cgs.media.watrUnSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }

        v if v == entity_event_t::EV_WATER_CLEAR as c_int => {
            DEBUGNAME(ctx, "EV_WATER_CLEAR");
            let custom = CG_CustomSound(ctx, es.number, "*gasp.wav");
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, custom);
        }

        v if v == entity_event_t::EV_ITEM_PICKUP as c_int => {
            DEBUGNAME(ctx, "EV_ITEM_PICKUP");
            'pickup: {
                // player predicted
                let mut index = ctx
                    .world
                    .entity(es.eventParm as usize)
                    .currentState
                    .modelindex;

                if index < 1
                    && ctx
                        .world
                        .entity(es.eventParm as usize)
                        .currentState
                        .isJediMaster
                        != qfalse
                {
                    //a holocron most likely
                    index = ctx
                        .world
                        .entity(es.eventParm as usize)
                        .currentState
                        .trickedentindex4;
                    let holocronPickup = ctx.world.cgs.media.holocronPickup;
                    trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, holocronPickup);

                    let is_me = ctx
                        .world
                        .cg
                        .snap_ref()
                        .is_some_and(|s| es.number == s.ps.clientNum);
                    // §F19: Raven indexes `showPowersName[index]` unchecked with
                    // a server-supplied holocron index; out of range reads as
                    // "no name" (skip the print) here.
                    let powerName = showPowersName.get(index as usize).copied().flatten();
                    if is_me {
                        if let Some(powerName) = powerName {
                            let strText = CG_GetStringEdString(ctx, "MP_INGAME", "PICKUPLINE");
                            //Com_Printf("%s %s\n", strText, showPowersName[index]);
                            let powerText = CG_GetStringEdString(ctx, "SP_INGAME", powerName);
                            CG_CenterPrint(
                                ctx.world,
                                &format!("{strText} {powerText}\n"),
                                (SCREEN_HEIGHT as f64 * 0.30) as c_int,
                                BIGCHAR_WIDTH,
                            );
                        }
                    }

                    //Show the player their force selection bar in case picking the holocron up changed the current selection
                    let sel = ctx.world.cg.snap_ref().map(|s| {
                        (
                            s.ps.clientNum,
                            s.ps.fd.forcePowerSelected,
                            s.ps.fd.forcePowersActive,
                        )
                    });
                    if let Some((snapClientNum, forcePowerSelected, forcePowersActive)) = sel {
                        if index != FP_SABER_OFFENSE
                            && index != FP_SABER_DEFENSE
                            && index != FP_SABERTHROW
                            && index != FP_LEVITATION
                            && es.number == snapClientNum
                            && (index == forcePowerSelected
                                || (forcePowersActive & (1 << forcePowerSelected)) == 0)
                        {
                            let mut newindex = false;
                            if ctx.world.cg.forceSelect != index {
                                ctx.world.cg.forceSelect = index;
                                newindex = true;
                            }

                            if es.number == snapClientNum && newindex {
                                // `cg.forceSelectTime` is `f32` here; Raven's
                                // `int cg.time` widens into it (and the compare).
                                let time = ctx.world.cg.time as f32;
                                if ctx.world.cg.forceSelectTime < time {
                                    ctx.world.cg.forceSelectTime = time;
                                }
                            }
                        }
                    }

                    break 'pickup;
                }

                if ctx.world.entity(es.eventParm as usize).weapon >= ctx.world.cg.time {
                    //rww - an unfortunately necessary hack to prevent double item pickups
                    break 'pickup;
                }

                //Hopefully even if this entity is somehow removed and replaced with, say, another
                //item, this time will have expired by the time that item needs to be picked up.
                let time = ctx.world.cg.time;
                ctx.world.entity_mut(es.eventParm as usize).weapon = time + 500;

                if index < 1 || index >= bg_numItems {
                    break 'pickup;
                }
                let item = &bg_itemlist[index as usize];

                if
                /*item->giType != IT_POWERUP && */
                item.giType() != IT_TEAM {
                    if let Some(pickup_sound) = item.pickup_sound {
                        if !pickup_sound.is_empty() {
                            let sfx = trap::S_RegisterSound(ctx.engine, pickup_sound);
                            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
                        }
                    }
                }

                // show icon and name on status bar
                let is_me = ctx
                    .world
                    .cg
                    .snap_ref()
                    .is_some_and(|s| es.number == s.ps.clientNum);
                if is_me {
                    CG_ItemPickup(ctx, index);
                }
            }
        }

        v if v == entity_event_t::EV_GLOBAL_ITEM_PICKUP as c_int => {
            DEBUGNAME(ctx, "EV_GLOBAL_ITEM_PICKUP");
            'gpickup: {
                let index = es.eventParm; // player predicted

                if index < 1 || index >= bg_numItems {
                    break 'gpickup;
                }
                let item = &bg_itemlist[index as usize];
                // powerup pickups are global
                if let Some(pickup_sound) = item.pickup_sound {
                    if !pickup_sound.is_empty() {
                        if let Some(snapClientNum) = ctx.world.cg.snap_ref().map(|s| s.ps.clientNum)
                        {
                            let sfx = trap::S_RegisterSound(ctx.engine, pickup_sound);
                            trap::S_StartSound(ctx.engine, None, snapClientNum, CHAN_AUTO, sfx);
                        }
                    }
                }

                // show icon and name on status bar
                let is_me = ctx
                    .world
                    .cg
                    .snap_ref()
                    .is_some_and(|s| es.number == s.ps.clientNum);
                if is_me {
                    CG_ItemPickup(ctx, index);
                }
            }
        }

        v if v == entity_event_t::EV_VEH_FIRE as c_int => {
            DEBUGNAME(ctx, "EV_VEH_FIRE");
            // veh = &cg_entities[es->owner]; take it out to hand CG_VehMuzzleFireFX
            // a &centity_t while ctx stays borrowed, then put it back (cg_ents.rs
            // CG_General precedent).
            // NOTE: CG_VehMuzzleFireFX's body is a cited todo!() behind the
            // m_pVehicle presence guard - inert until the Vehicle_t referent
            // pool lands (nothing assigns m_pVehicle yet), live the moment it
            // does. Resolve the todo with that pool ruling.
            let owner = es.owner as usize;
            let veh = core::mem::replace(ctx.world.entity_mut(owner), centity_t::zeroed());
            CG_VehMuzzleFireFX(ctx, &veh, &es);
            *ctx.world.entity_mut(owner) = veh;
        }

        //
        // weapon events
        //
        v if v == entity_event_t::EV_NOAMMO as c_int => {
            DEBUGNAME(ctx, "EV_NOAMMO");
            //		trap_S_StartSound (NULL, es->number, CHAN_AUTO, cgs.media.noAmmoSound );
            'noammo: {
                let snapClientNum = ctx.world.cg.snap_ref().map(|s| s.ps.clientNum);
                let Some(snapClientNum) = snapClientNum else {
                    break 'noammo;
                };
                if es.number != snapClientNum {
                    break 'noammo;
                }

                let snapWeapon = ctx.world.cg.snap_ref().map(|s| s.ps.weapon).unwrap_or(0);

                if CG_InFighter(ctx.world) || CG_InATST(ctx.world) || snapWeapon == WP_NONE {
                    //just letting us know our vehicle is out of ammo
                    //FIXME: flash something on HUD or give some message so we know we have no ammo
                    // DEFERRED: Vehicle_t referent pool — Raven picks the
                    // vehicle weapon's custom `soundNoAmmo` off
                    // `localCent->m_pVehicle->m_pVehicleInfo->weapon[eventParm]`;
                    // DEC-46.2's `Option<VehicleId>` answers presence only, so we
                    // fall back to the default "no ammo" sound until the pool
                    // lands (cg_draw.rs vehicle-HUD precedent).
                    // Source: oracle/codemp/cgame/cg_event.c:2170-2179
                    let noAmmoSound = ctx.world.cgs.media.noAmmoSound;
                    trap::S_StartSound(ctx.engine, None, snapClientNum, CHAN_AUTO, noAmmoSound);

                    //flash the HUD so they associate the sound with the visual indicator that they don't have enough ammo
                    let time = ctx.world.cg.time;
                    if ctx.world.draw.cg_vehicleAmmoWarningTime < time
                        || ctx.world.draw.cg_vehicleAmmoWarning != es.eventParm
                    {
                        //if there's already one going, don't interrupt it (unless they tried to fire another weapon that's out of ammo)
                        ctx.world.draw.cg_vehicleAmmoWarning = es.eventParm;
                        ctx.world.draw.cg_vehicleAmmoWarningTime = time + 500;
                    }
                } else if snapWeapon == WP_SABER {
                    let time = ctx.world.cg.time;
                    ctx.world.cg.forceHUDTotalFlashTime = time + 1000;
                } else {
                    let mut weap = 0;

                    if es.eventParm != 0 && es.eventParm < WP_NUM_WEAPONS {
                        if let Some(s) = ctx.world.cg.snap_mut() {
                            s.ps.stats[statIndex_t::STAT_WEAPONS as usize] &= !(1 << es.eventParm);
                            weap = s.ps.weapon;
                        }
                    } else if es.eventParm != 0 {
                        weap = es.eventParm - WP_NUM_WEAPONS;
                    }
                    CG_OutOfAmmoChange(ctx, weap);
                }
            }
        }

        v if v == entity_event_t::EV_CHANGE_WEAPON as c_int => {
            DEBUGNAME(ctx, "EV_CHANGE_WEAPON");
            let weapon = es.eventParm;

            debug_assert!(weapon >= 0 && (weapon as usize) < MAX_WEAPONS);
            // §F19: Raven's assert compiles out in retail and the OOB index
            // reads garbage; eventParm is server-supplied, so skip instead.
            if weapon < 0 || weapon as usize >= MAX_WEAPONS {
                return;
            }

            let selectSound = ctx.world.cg_weapons[weapon as usize].selectSound;

            if selectSound != 0 {
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, selectSound);
            } else if weapon != WP_SABER {
                //not sure what SP is doing for this but I don't want a select sound for saber (it has the saber-turn-on)
                let sfx = ctx.world.cgs.media.selectSound;
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
            }
        }

        v if v == entity_event_t::EV_FIRE_WEAPON as c_int => {
            DEBUGNAME(ctx, "EV_FIRE_WEAPON");
            'fire: {
                if es.number >= MAX_CLIENTS_I32 && es.eType != entityType_t::ET_NPC as c_int {
                    //special case for turret firing
                    if ctx.world.cg_weapons[WP_TURRET as usize].registered == qfalse {
                        CG_RegisterWeapon(ctx, WP_TURRET);
                    }

                    let ghoul2 = ctx.world.entity(centNum).ghoul2;
                    if ghoul2.is_null() {
                        break 'fire;
                    }

                    if ctx.world.entity(centNum).bolt1 == 0 {
                        let b = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flash01");
                        ctx.world.entity_mut(centNum).bolt1 = b;
                    }
                    if ctx.world.entity(centNum).bolt2 == 0 {
                        let b = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flash02");
                        ctx.world.entity_mut(centNum).bolt2 = b;
                    }
                    let time = ctx.world.cg.time;
                    trap::G2API_SetBoneAnim(
                        ctx.engine,
                        ghoul2,
                        0,
                        "Bone02",
                        1,
                        4,
                        BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
                        1.0,
                        time,
                        -1.0,
                        300,
                    );

                    let boltIndex = if es.eventParm != 0 {
                        ctx.world.entity(centNum).bolt2
                    } else {
                        ctx.world.entity(centNum).bolt1
                    };
                    let angles = es.angles;
                    let origin = es.origin;
                    let modelScale = ctx.world.entity(centNum).modelScale;
                    let mut matrix = mdxaBone_t {
                        matrix: [[0.0; 4]; 3],
                    };
                    trap::G2API_GetBoltMatrix(
                        ctx.engine,
                        ghoul2,
                        0,
                        boltIndex,
                        &mut matrix,
                        &angles,
                        &origin,
                        time,
                        Some(&mut ctx.world.cgs.gameModels[0]),
                        &modelScale,
                    );

                    let gunpoint: vec3_t = [
                        matrix.matrix[0][3],
                        matrix.matrix[1][3],
                        matrix.matrix[2][3],
                    ];
                    let gunangle: vec3_t = [
                        -matrix.matrix[0][0],
                        -matrix.matrix[1][0],
                        -matrix.matrix[2][0],
                    ];

                    let eID = ctx.world.cgs.effects.mEmplacedMuzzleFlash;
                    trap::FX_PlayEffectID(ctx.engine, eID, &gunpoint, &gunangle, -1, -1);
                } else if es.weapon != WP_EMPLACED_GUN || es.eType == entityType_t::ET_NPC as c_int
                {
                    if es.eType == entityType_t::ET_NPC as c_int
                        && es.NPC_class == class_t::CLASS_VEHICLE as c_int
                        && ctx.world.entity(centNum).m_pVehicle.is_some()
                    {
                        //vehicles do nothing for clientside weapon fire events.. at least for now.
                        break 'fire;
                    }
                    CG_FireWeapon(ctx, centNum, false);
                }
            }
        }

        v if v == entity_event_t::EV_ALT_FIRE as c_int => {
            DEBUGNAME(ctx, "EV_ALT_FIRE");
            'alt: {
                if es.weapon == WP_EMPLACED_GUN {
                    //don't do anything for emplaced stuff
                    break 'alt;
                }

                if es.eType == entityType_t::ET_NPC as c_int
                    && es.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && ctx.world.entity(centNum).m_pVehicle.is_some()
                {
                    //vehicles do nothing for clientside weapon fire events.. at least for now.
                    break 'alt;
                }

                CG_FireWeapon(ctx, centNum, true);

                //if you just exploded your detpacks and you have no ammo left for them, autoswitch
                let detEmpty = ctx.world.cg.snap_ref().is_some_and(|s| {
                    s.ps.clientNum == es.number
                        && s.ps.weapon == WP_DET_PACK
                        && s.ps.ammo[weaponData[WP_DET_PACK as usize].ammoIndex as usize] == 0
                });
                if detEmpty {
                    CG_OutOfAmmoChange(ctx, WP_DET_PACK);
                }
            }
        }

        v if v == entity_event_t::EV_SABER_ATTACK as c_int => {
            DEBUGNAME(ctx, "EV_SABER_ATTACK");
            let r = ctx.world.bg_state.rng.Q_irand(1, 8);
            let mut swingSound =
                trap::S_RegisterSound(ctx.engine, &format!("sound/weapons/saber/saberhup{r}.wav"));

            let idx = es.number;
            let isNpc =
                ctx.world.entity(idx as usize).currentState.eType == entityType_t::ET_NPC as c_int;
            // custom swing sound: only when the client is valid and saber[0] has one
            let has_swing = eventSaberClient(ctx.world, idx, isNpc)
                .is_some_and(|c| c.infoValid != qfalse && c.saber[0].swingSound[0] != 0);
            if has_swing {
                let sr = ctx.world.bg_state.rng.Q_irand(0, 2) as usize;
                swingSound =
                    eventSaberClient(ctx.world, idx, isNpc).unwrap().saber[0].swingSound[sr];
            }
            trap::S_StartSound(
                ctx.engine,
                Some(&es.pos.trBase),
                es.number,
                CHAN_WEAPON,
                swingSound,
            );
        }

        v if v == entity_event_t::EV_SABER_HIT as c_int => {
            DEBUGNAME(ctx, "EV_SABER_HIT");
            let mut hitPersonFxID = ctx.world.cgs.effects.mSaberBloodSparks;
            let mut hitPersonSmallFxID = ctx.world.cgs.effects.mSaberBloodSparksSmall;
            let mut hitPersonMidFxID = ctx.world.cgs.effects.mSaberBloodSparksMid;
            let mut hitOtherFxID = ctx.world.cgs.effects.mSaberCut;
            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
            let mut hitSound =
                trap::S_RegisterSound(ctx.engine, &format!("sound/weapons/saber/saberhit{r}.wav"));

            if es.otherEntityNum2 >= 0 && es.otherEntityNum2 < ENTITYNUM_NONE {
                //we have a specific person who is causing this effect, see if we should override it with any custom saber effects/sounds
                let idx = es.otherEntityNum2;
                let isNpc = ctx.world.entity(idx as usize).currentState.eType
                    == entityType_t::ET_NPC as c_int;
                let valid =
                    eventSaberClient(ctx.world, idx, isNpc).is_some_and(|c| c.infoValid != qfalse);
                if valid {
                    let saberNum = es.weapon;
                    let bladeNum = es.legsAnim;
                    // §F19: `es->weapon` is a server-supplied saber index Raven
                    // reads unchecked; out of `[MAX_SABERS]` range keeps the
                    // default fx/sounds here.
                    if saberNum >= 0 && (saberNum as usize) < MAX_SABERS {
                        let sNum = saberNum as usize;
                        let saberPtr: &mut saberInfo_t = if isNpc {
                            &mut ctx
                                .world
                                .entity_mut(idx as usize)
                                .npcClient
                                .as_deref_mut()
                                .unwrap()
                                .saber[sNum]
                        } else {
                            &mut ctx.world.cgs.clientinfo[idx as usize].saber[sNum]
                        };
                        let useSecond =
                            WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;

                        // capture the handles in one borrow (incl. saber[0]'s
                        // hitOtherEffect for Raven's index-0 quirk below)
                        let (hitPE, hitPE2, hitOE, hitOE2, hitS, hitS2, saber0HitOther) = {
                            let c = eventSaberClient(ctx.world, idx, isNpc).unwrap();
                            let s = &c.saber[sNum];
                            (
                                s.hitPersonEffect,
                                s.hitPersonEffect2,
                                s.hitOtherEffect,
                                s.hitOtherEffect2,
                                s.hitSound,
                                s.hit2Sound,
                                c.saber[0].hitOtherEffect,
                            )
                        };

                        if useSecond {
                            //use second blade style values
                            if hitPE2 != 0 {
                                //custom hit person effect
                                hitPersonFxID = hitPE2;
                                hitPersonSmallFxID = hitPE2;
                                hitPersonMidFxID = hitPE2;
                            }
                            if hitOE2 != 0 {
                                //custom hit other effect
                                hitOtherFxID = hitOE2;
                            }
                            if hitS2[0] != 0 {
                                //custom hit sound
                                hitSound = hitS2[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
                            }
                        } else {
                            //use first blade style values
                            if hitPE != 0 {
                                //custom hit person effect
                                hitPersonFxID = hitPE;
                                hitPersonSmallFxID = hitPE;
                                hitPersonMidFxID = hitPE;
                            }
                            if hitOE != 0 {
                                //custom hit other effect
                                //Raven reads saber[0] here (not saberNum) - kept.
                                hitOtherFxID = saber0HitOther;
                            }
                            if hitS[0] != 0 {
                                //custom hit sound
                                hitSound = hitS[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
                            }
                        }
                    }
                }
            }

            if es.eventParm == 16 {
                //Make lots of sparks, something special happened
                let mut fxDir = es.angles;
                if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                    fxDir[1] = 1.0;
                }
                trap::S_StartSound(ctx.engine, Some(&es.origin), es.number, CHAN_AUTO, hitSound);
                for _ in 0..6 {
                    trap::FX_PlayEffectID(ctx.engine, hitPersonFxID, &es.origin, &fxDir, -1, -1);
                }
            } else if es.eventParm != 0 {
                //hit a person
                let mut fxDir = es.angles;
                if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                    fxDir[1] = 1.0;
                }
                trap::S_StartSound(ctx.engine, Some(&es.origin), es.number, CHAN_AUTO, hitSound);
                if es.eventParm == 3 {
                    // moderate or big hits.
                    trap::FX_PlayEffectID(
                        ctx.engine,
                        hitPersonSmallFxID,
                        &es.origin,
                        &fxDir,
                        -1,
                        -1,
                    );
                } else if es.eventParm == 2 {
                    // this is for really big hits.
                    trap::FX_PlayEffectID(ctx.engine, hitPersonMidFxID, &es.origin, &fxDir, -1, -1);
                } else {
                    // this should really just be done in the effect itself, no?
                    for _ in 0..3 {
                        trap::FX_PlayEffectID(
                            ctx.engine,
                            hitPersonFxID,
                            &es.origin,
                            &fxDir,
                            -1,
                            -1,
                        );
                    }
                }
            } else {
                //hit something else
                let mut fxDir = es.angles;
                if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                    fxDir[1] = 1.0;
                }
                //old jk2mp method
                /*
                trap_S_StartSound(es->origin, es->number, CHAN_AUTO, trap_S_RegisterSound("sound/weapons/saber/saberhit.wav"));
                trap_FX_PlayEffectID( trap_FX_RegisterEffect("saber/spark.efx"), es->origin, fxDir, -1, -1 );
                */
                trap::FX_PlayEffectID(ctx.engine, hitOtherFxID, &es.origin, &fxDir, -1, -1);
            }

            //rww - store the hit indecies + time so we can do between-frame visual tricks
            if es.otherEntityNum != ENTITYNUM_NONE && es.otherEntityNum2 != ENTITYNUM_NONE {
                let time = ctx.world.cg.time;
                let saberOwner = ctx.world.entity_mut(es.otherEntityNum2 as usize);
                saberOwner.serverSaberHitIndex = es.otherEntityNum;
                saberOwner.serverSaberHitTime = time;
                saberOwner.serverSaberFleshImpact = if es.eventParm != 0 { qtrue } else { qfalse };
            }
        }

        v if v == entity_event_t::EV_SABER_BLOCK as c_int => {
            DEBUGNAME(ctx, "EV_SABER_BLOCK");
            if es.eventParm != 0 {
                //saber block
                let mut blockFXID = ctx.world.cgs.effects.mSaberBlock;
                let r = ctx.world.bg_state.rng.Q_irand(1, 9);
                let mut blockSound = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/weapons/saber/saberblock{r}.wav"),
                );
                let mut noFlare = false;

                if es.otherEntityNum2 >= 0 && es.otherEntityNum2 < ENTITYNUM_NONE {
                    //we have a specific person causing this, maybe override with custom saber effects/sounds
                    let idx = es.otherEntityNum2;
                    let isNpc = ctx.world.entity(idx as usize).currentState.eType
                        == entityType_t::ET_NPC as c_int;
                    let valid = eventSaberClient(ctx.world, idx, isNpc)
                        .is_some_and(|c| c.infoValid != qfalse);
                    if valid {
                        let saberNum = es.weapon;
                        let bladeNum = es.legsAnim;
                        // §F19: server-supplied saber index, unchecked in Raven.
                        if saberNum >= 0 && (saberNum as usize) < MAX_SABERS {
                            let sNum = saberNum as usize;
                            let saberPtr: &mut saberInfo_t = if isNpc {
                                &mut ctx
                                    .world
                                    .entity_mut(idx as usize)
                                    .npcClient
                                    .as_deref_mut()
                                    .unwrap()
                                    .saber[sNum]
                            } else {
                                &mut ctx.world.cgs.clientinfo[idx as usize].saber[sNum]
                            };
                            let useSecond =
                                WP_SaberBladeUseSecondBladeStyle(saberPtr, bladeNum) != qfalse;

                            let (blockE, blockE2, blockS, blockS2, saberFlags2) = {
                                let c = eventSaberClient(ctx.world, idx, isNpc).unwrap();
                                let s = &c.saber[sNum];
                                (
                                    s.blockEffect,
                                    s.blockEffect2,
                                    s.blockSound,
                                    s.block2Sound,
                                    s.saberFlags2,
                                )
                            };

                            if useSecond {
                                //use second blade style values
                                if blockE2 != 0 {
                                    blockFXID = blockE2;
                                }
                                if blockS2[0] != 0 {
                                    blockSound =
                                        blockS2[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
                                }
                            } else {
                                if blockE != 0 {
                                    blockFXID = blockE;
                                }
                                if blockS[0] != 0 {
                                    blockSound =
                                        blockS[ctx.world.bg_state.rng.Q_irand(0, 2) as usize];
                                }
                            }
                            if (saberFlags2 & SFL2_NO_CLASH_FLARE) != 0 {
                                noFlare = true;
                            }
                        }
                    }
                }

                let mut cullPass = false;
                if ctx.world.cg.mInRMG != qfalse {
                    let vieworg = ctx.world.cg.refdef.vieworg;
                    let mut vecSub = [0.0; 3];
                    _VectorSubtract(vieworg, es.origin, &mut vecSub);

                    if VectorLength(vecSub) < 5000.0 {
                        let mut tr = trace_t::zeroed();
                        CG_Trace(
                            ctx,
                            &mut tr,
                            &vieworg,
                            &vec3_origin,
                            &vec3_origin,
                            &es.origin,
                            ENTITYNUM_NONE,
                            CONTENTS_TERRAIN | CONTENTS_SOLID,
                        );

                        if tr.fraction == 1.0 || (tr.entityNum as c_int) < MAX_CLIENTS_I32 {
                            cullPass = true;
                        }
                    }
                } else {
                    cullPass = true;
                }

                if cullPass {
                    let mut fxDir = es.angles;
                    if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                        fxDir[1] = 1.0;
                    }
                    trap::S_StartSound(
                        ctx.engine,
                        Some(&es.origin),
                        es.number,
                        CHAN_AUTO,
                        blockSound,
                    );
                    trap::FX_PlayEffectID(ctx.engine, blockFXID, &es.origin, &fxDir, -1, -1);

                    if !noFlare {
                        ctx.world.draw.cg_saberFlashTime = ctx.world.cg.time - 50;
                        ctx.world.draw.cg_saberFlashPos = es.origin;
                    }
                }
            } else {
                //projectile block
                let mut fxDir = es.angles;
                if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                    fxDir[1] = 1.0;
                }
                let fx = ctx.world.cgs.effects.mBlasterDeflect;
                trap::FX_PlayEffectID(ctx.engine, fx, &es.origin, &fxDir, -1, -1);
            }
        }

        v if v == entity_event_t::EV_SABER_CLASHFLARE as c_int => {
            DEBUGNAME(ctx, "EV_SABER_CLASHFLARE");
            let mut cullPass = false;

            if ctx.world.cg.mInRMG != qfalse {
                let vieworg = ctx.world.cg.refdef.vieworg;
                let mut vecSub = [0.0; 3];
                _VectorSubtract(vieworg, es.origin, &mut vecSub);

                if VectorLength(vecSub) < 5000.0 {
                    let mut tr = trace_t::zeroed();
                    CG_Trace(
                        ctx,
                        &mut tr,
                        &vieworg,
                        &vec3_origin,
                        &vec3_origin,
                        &es.origin,
                        ENTITYNUM_NONE,
                        CONTENTS_TERRAIN | CONTENTS_SOLID,
                    );

                    if tr.fraction == 1.0 || (tr.entityNum as c_int) < MAX_CLIENTS_I32 {
                        cullPass = true;
                    }
                }
            } else {
                cullPass = true;
            }

            if cullPass {
                ctx.world.draw.cg_saberFlashTime = ctx.world.cg.time - 50;
                ctx.world.draw.cg_saberFlashPos = es.origin;
            }
            let r = ctx.world.bg_state.rng.Q_irand(1, 3);
            let sfx =
                trap::S_RegisterSound(ctx.engine, &format!("sound/weapons/saber/saberhitwall{r}"));
            trap::S_StartSound(ctx.engine, Some(&es.origin), -1, CHAN_WEAPON, sfx);
        }

        v if v == entity_event_t::EV_SABER_UNHOLSTER as c_int => {
            DEBUGNAME(ctx, "EV_SABER_UNHOLSTER");
            let isNpc = es.eType == entityType_t::ET_NPC as c_int;
            // capture the two soundOn handles from the resolved client, if any
            let sounds = if isNpc {
                ctx.world
                    .entity(es.number as usize)
                    .npcClient
                    .as_deref()
                    .map(|ci| (ci.saber[0].soundOn, ci.saber[1].soundOn))
            } else if es.number < MAX_CLIENTS_I32 && es.number >= 0 {
                let ci = &ctx.world.cgs.clientinfo[es.number as usize];
                Some((ci.saber[0].soundOn, ci.saber[1].soundOn))
            } else {
                None
            };

            if let Some((soundOn0, soundOn1)) = sounds {
                if soundOn0 != 0 {
                    trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, soundOn0);
                }
                if soundOn1 != 0 {
                    trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, soundOn1);
                }
            }
        }

        v if v == entity_event_t::EV_BECOME_JEDIMASTER as c_int => {
            DEBUGNAME(ctx, "EV_SABER_UNHOLSTER");
            let playerMins: vec3_t = [-15.0, -15.0, (DEFAULT_MINS_2 + 8) as f32];
            let playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];
            let mut ang = [0.0; 3];

            VectorClear(&mut ang);
            ang[ROLL] = 1.0;

            let mut dpos = *position;
            dpos[2] -= 4096.0;

            let mut tr = trace_t::zeroed();
            CG_Trace(
                ctx,
                &mut tr,
                position,
                &playerMins,
                &playerMaxs,
                &dpos,
                es.number,
                MASK_SOLID,
            );
            let pos = tr.endpos;

            if tr.fraction != 1.0 {
                let mJediSpawn = ctx.world.cgs.effects.mJediSpawn;
                trap::FX_PlayEffectID(ctx.engine, mJediSpawn, &pos, &ang, -1, -1);

                let sfx = trap::S_RegisterSound(ctx.engine, "sound/weapons/saber/saberon.wav");
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);

                let is_me = ctx
                    .world
                    .cg
                    .snap_ref()
                    .is_some_and(|s| s.ps.clientNum == es.number);
                if is_me {
                    let happyMusic = ctx.world.cgs.media.happyMusic;
                    trap::S_StartLocalSound(ctx.engine, happyMusic, CHAN_LOCAL);
                    CGCam_SetMusicMult(ctx.world, 0.3, 5000);
                }
            }
        }

        v if v == entity_event_t::EV_DISRUPTOR_MAIN_SHOT as c_int => {
            DEBUGNAME(ctx, "EV_DISRUPTOR_MAIN_SHOT");
            let is_local = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.eventParm == s.ps.clientNum);
            if !is_local || ctx.world.cg.renderingThirdPerson != qfalse {
                //h4q3ry
                let mut to = ctx.world.entity(centNum).currentState.origin2;
                CG_GetClientWeaponMuzzleBoltPoint(ctx, es.eventParm, &mut to);
                ctx.world.entity_mut(centNum).currentState.origin2 = to;
            } else {
                let fp = ctx.world.cg.lastFPFlashPoint;
                if fp[0] != 0.0 || fp[1] != 0.0 || fp[2] != 0.0 {
                    //get the position of the muzzle flash for the first person weapon model from the last frame
                    ctx.world.entity_mut(centNum).currentState.origin2 = fp;
                }
            }
            let origin2 = ctx.world.entity(centNum).currentState.origin2;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            FX_DisruptorMainShot(ctx, &origin2, &lerpOrigin);
        }

        v if v == entity_event_t::EV_DISRUPTOR_SNIPER_SHOT as c_int => {
            DEBUGNAME(ctx, "EV_DISRUPTOR_SNIPER_SHOT");
            let is_local = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.eventParm == s.ps.clientNum);
            if !is_local || ctx.world.cg.renderingThirdPerson != qfalse {
                //h4q3ry
                let mut to = ctx.world.entity(centNum).currentState.origin2;
                CG_GetClientWeaponMuzzleBoltPoint(ctx, es.eventParm, &mut to);
                ctx.world.entity_mut(centNum).currentState.origin2 = to;
            } else {
                let fp = ctx.world.cg.lastFPFlashPoint;
                if fp[0] != 0.0 || fp[1] != 0.0 || fp[2] != 0.0 {
                    //get the position of the muzzle flash for the first person weapon model from the last frame
                    ctx.world.entity_mut(centNum).currentState.origin2 = fp;
                }
            }
            let origin2 = ctx.world.entity(centNum).currentState.origin2;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let shouldtarget = ctx.world.entity(centNum).currentState.shouldtarget;
            FX_DisruptorAltShot(ctx, &origin2, &lerpOrigin, shouldtarget != qfalse);
        }

        v if v == entity_event_t::EV_DISRUPTOR_SNIPER_MISS as c_int => {
            DEBUGNAME(ctx, "EV_DISRUPTOR_SNIPER_MISS");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            if es.weapon != 0 {
                //primary
                FX_DisruptorHitWall(ctx, &lerpOrigin, &dir);
            } else {
                //secondary
                FX_DisruptorAltMiss(ctx, &lerpOrigin, &dir);
            }
        }

        v if v == entity_event_t::EV_DISRUPTOR_HIT as c_int => {
            DEBUGNAME(ctx, "EV_DISRUPTOR_HIT");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            if es.weapon != 0 {
                //client
                FX_DisruptorHitPlayer(ctx, &lerpOrigin, &dir, true);
            } else {
                //non-client
                FX_DisruptorHitWall(ctx, &lerpOrigin, &dir);
            }
        }

        v if v == entity_event_t::EV_DISRUPTOR_ZOOMSOUND as c_int => {
            DEBUGNAME(ctx, "EV_DISRUPTOR_ZOOMSOUND");
            let snap = ctx
                .world
                .cg
                .snap_ref()
                .map(|s| (s.ps.clientNum, s.ps.zoomMode));
            if let Some((snapClientNum, zoomMode)) = snap {
                if es.number == snapClientNum {
                    if zoomMode != 0 {
                        let sfx = trap::S_RegisterSound(
                            ctx.engine,
                            "sound/weapons/disruptor/zoomstart.wav",
                        );
                        trap::S_StartLocalSound(ctx.engine, sfx, CHAN_AUTO);
                    } else {
                        let sfx = trap::S_RegisterSound(
                            ctx.engine,
                            "sound/weapons/disruptor/zoomend.wav",
                        );
                        trap::S_StartLocalSound(ctx.engine, sfx, CHAN_AUTO);
                    }
                }
            }
        }

        v if v == entity_event_t::EV_PREDEFSOUND as c_int => {
            DEBUGNAME(ctx, "EV_PREDEFSOUND");
            let mut sID = -1;

            match es.eventParm {
                m if m == pdSounds_t::PDSOUND_PROTECTHIT as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/protecthit.mp3");
                }
                m if m == pdSounds_t::PDSOUND_PROTECT as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/protect.mp3");
                }
                m if m == pdSounds_t::PDSOUND_ABSORBHIT as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/absorbhit.mp3");
                    if es.trickedentindex >= 0 && es.trickedentindex < MAX_CLIENTS_I32 {
                        let clnum = es.trickedentindex as usize;
                        let time = ctx.world.cg.time;
                        let cl = ctx.world.entity_mut(clnum);
                        cl.teamPowerEffectTime = time + 1000;
                        cl.teamPowerType = 3;
                    }
                }
                m if m == pdSounds_t::PDSOUND_ABSORB as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/absorb.mp3");
                }
                m if m == pdSounds_t::PDSOUND_FORCEJUMP as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/jump.mp3");
                }
                m if m == pdSounds_t::PDSOUND_FORCEGRIP as c_int => {
                    sID = trap::S_RegisterSound(ctx.engine, "sound/weapons/force/grip.mp3");
                }
                _ => {}
            }

            // Raven's `if (sID != 1)` (note: not -1) is kept verbatim - the
            // registered handle is never 1, so this always fires.
            if sID != 1 {
                trap::S_StartSound(ctx.engine, Some(&es.origin), es.number, CHAN_AUTO, sID);
            }
        }

        v if v == entity_event_t::EV_TEAM_POWER as c_int => {
            DEBUGNAME(ctx, "EV_TEAM_POWER");
            let mut clnum = 0;
            while clnum < MAX_CLIENTS_I32 {
                if CG_InClientBitflags(&es, clnum) {
                    let time = ctx.world.cg.time;
                    if es.eventParm == 1 {
                        //eventParm 1 is heal
                        let sfx = ctx.world.cgs.media.teamHealSound;
                        trap::S_StartSound(ctx.engine, None, clnum, CHAN_AUTO, sfx);
                        let cl = ctx.world.entity_mut(clnum as usize);
                        cl.teamPowerEffectTime = time + 1000;
                        cl.teamPowerType = 1;
                    } else {
                        //eventParm 2 is force regen
                        let sfx = ctx.world.cgs.media.teamRegenSound;
                        trap::S_StartSound(ctx.engine, None, clnum, CHAN_AUTO, sfx);
                        let cl = ctx.world.entity_mut(clnum as usize);
                        cl.teamPowerEffectTime = time + 1000;
                        cl.teamPowerType = 0;
                    }
                }
                clnum += 1;
            }
        }

        v if v == entity_event_t::EV_SCREENSHAKE as c_int => {
            DEBUGNAME(ctx, "EV_SCREENSHAKE");
            if es.modelindex == 0
                || ctx.world.cg.predictedPlayerState.clientNum == es.modelindex - 1
            {
                CGCam_Shake(ctx.world, es.angles[0], es.time);
            }
        }

        v if v == entity_event_t::EV_LOCALTIMER as c_int => {
            DEBUGNAME(ctx, "EV_LOCALTIMER");
            if es.owner == ppsClientNum {
                CG_LocalTimingBar(ctx.world, es.time, es.time2);
            }
        }

        v if v == entity_event_t::EV_USE_ITEM0 as c_int
            || v == entity_event_t::EV_USE_ITEM1 as c_int
            || v == entity_event_t::EV_USE_ITEM2 as c_int
            || v == entity_event_t::EV_USE_ITEM3 as c_int
            || v == entity_event_t::EV_USE_ITEM4 as c_int
            || v == entity_event_t::EV_USE_ITEM5 as c_int
            || v == entity_event_t::EV_USE_ITEM6 as c_int
            || v == entity_event_t::EV_USE_ITEM7 as c_int
            || v == entity_event_t::EV_USE_ITEM8 as c_int
            || v == entity_event_t::EV_USE_ITEM9 as c_int
            || v == entity_event_t::EV_USE_ITEM10 as c_int
            || v == entity_event_t::EV_USE_ITEM11 as c_int
            || v == entity_event_t::EV_USE_ITEM12 as c_int
            || v == entity_event_t::EV_USE_ITEM13 as c_int
            || v == entity_event_t::EV_USE_ITEM14 as c_int =>
        {
            // Raven has one DEBUGNAME per case (EV_USE_ITEM0..14); the dispatch
            // body is identical `CG_UseItem(cent)`.
            DEBUGNAME(ctx, "EV_USE_ITEM");
            let cent = core::mem::replace(ctx.world.entity_mut(centNum), centity_t::zeroed());
            CG_UseItem(ctx, &cent);
            *ctx.world.entity_mut(centNum) = cent;
        }

        v if v == entity_event_t::EV_ITEMUSEFAIL as c_int => {
            DEBUGNAME(ctx, "EV_ITEMUSEFAIL");
            let is_me = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| s.ps.clientNum == es.number);
            if is_me {
                let refName = match es.eventParm {
                    m if m == itemUseFail_t::SENTRY_NOROOM as c_int => Some("SENTRY_NOROOM"),
                    m if m == itemUseFail_t::SENTRY_ALREADYPLACED as c_int => {
                        Some("SENTRY_ALREADYPLACED")
                    }
                    m if m == itemUseFail_t::SHIELD_NOROOM as c_int => Some("SHIELD_NOROOM"),
                    m if m == itemUseFail_t::SEEKER_ALREADYDEPLOYED as c_int => {
                        Some("SEEKER_ALREADYDEPLOYED")
                    }
                    _ => None,
                };

                if let Some(refName) = refName {
                    let psStringEDRef = CG_GetStringEdString(ctx, "MP_INGAME", refName);
                    Com_Printf(ctx, &format!("{psStringEDRef}\n"));
                }
            }
        }

        //=================================================================
        //
        // other events
        //
        v if v == entity_event_t::EV_PLAYER_TELEPORT_IN as c_int => {
            DEBUGNAME(ctx, "EV_PLAYER_TELEPORT_IN");
            let playerMins: vec3_t = [-15.0, -15.0, (DEFAULT_MINS_2 + 8) as f32];
            let playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];
            let mut ang = [0.0; 3];

            VectorClear(&mut ang);
            ang[ROLL] = 1.0;

            let mut dpos = *position;
            dpos[2] -= 4096.0;

            let mut tr = trace_t::zeroed();
            CG_Trace(
                ctx,
                &mut tr,
                position,
                &playerMins,
                &playerMaxs,
                &dpos,
                es.number,
                MASK_SOLID,
            );
            let pos = tr.endpos;

            let teleInSound = ctx.world.cgs.media.teleInSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, teleInSound);

            if tr.fraction != 1.0 {
                let mSpawn = ctx.world.cgs.effects.mSpawn;
                trap::FX_PlayEffectID(ctx.engine, mSpawn, &pos, &ang, -1, -1);
            }
        }

        v if v == entity_event_t::EV_PLAYER_TELEPORT_OUT as c_int => {
            DEBUGNAME(ctx, "EV_PLAYER_TELEPORT_OUT");
            let playerMins: vec3_t = [-15.0, -15.0, (DEFAULT_MINS_2 + 8) as f32];
            let playerMaxs: vec3_t = [15.0, 15.0, DEFAULT_MAXS_2 as f32];
            let mut ang = [0.0; 3];

            VectorClear(&mut ang);
            ang[ROLL] = 1.0;

            let mut dpos = *position;
            dpos[2] -= 4096.0;

            let mut tr = trace_t::zeroed();
            CG_Trace(
                ctx,
                &mut tr,
                position,
                &playerMins,
                &playerMaxs,
                &dpos,
                es.number,
                MASK_SOLID,
            );
            let pos = tr.endpos;

            let teleOutSound = ctx.world.cgs.media.teleOutSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, teleOutSound);

            if tr.fraction != 1.0 {
                let mSpawn = ctx.world.cgs.effects.mSpawn;
                trap::FX_PlayEffectID(ctx.engine, mSpawn, &pos, &ang, -1, -1);
            }
        }

        v if v == entity_event_t::EV_ITEM_POP as c_int => {
            DEBUGNAME(ctx, "EV_ITEM_POP");
            let sfx = ctx.world.cgs.media.respawnSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }

        v if v == entity_event_t::EV_ITEM_RESPAWN as c_int => {
            DEBUGNAME(ctx, "EV_ITEM_RESPAWN");
            let time = ctx.world.cg.time;
            ctx.world.entity_mut(centNum).miscTime = time; // scale up from this
            let sfx = ctx.world.cgs.media.respawnSound;
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
        }

        v if v == entity_event_t::EV_GRENADE_BOUNCE as c_int => {
            DEBUGNAME(ctx, "EV_GRENADE_BOUNCE");
            //Do something here?
        }

        v if v == entity_event_t::EV_SCOREPLUM as c_int => {
            DEBUGNAME(ctx, "EV_SCOREPLUM");
            let otherEntityNum = ctx.world.entity(centNum).currentState.otherEntityNum;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let time = ctx.world.entity(centNum).currentState.time;
            CG_ScorePlum(ctx.world, otherEntityNum, &lerpOrigin, time);
        }

        v if v == entity_event_t::EV_CTFMESSAGE as c_int => {
            DEBUGNAME(ctx, "EV_CTFMESSAGE");
            CG_GetCTFMessageEvent(ctx, &es);
        }

        v if v == entity_event_t::EV_BODYFADE as c_int => {
            'body: {
                if es.eType != entityType_t::ET_BODY as c_int {
                    debug_assert!(false, "EV_BODYFADE event from a non-corpse");
                    break 'body;
                }

                let ghoul2 = ctx.world.entity(centNum).ghoul2;
                if !ghoul2.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, ghoul2) {
                    //turn the inside of the face off, to avoid showing the mouth when we start alpha fading the corpse
                    trap::G2API_SetSurfaceOnOff(
                        ctx.engine,
                        ghoul2,
                        "head_eyes_mouth",
                        0x0000_0002, /*G2SURFACEFLAG_OFF*/
                    );
                }

                let time = ctx.world.cg.time;
                ctx.world.entity_mut(centNum).bodyFadeTime = time + 60000;
            }
        }

        //
        // siege gameplay events
        //
        v if v == entity_event_t::EV_SIEGE_ROUNDOVER as c_int => {
            DEBUGNAME(ctx, "EV_SIEGE_ROUNDOVER");
            let weap = ctx.world.entity(centNum).currentState.weapon;
            CG_SiegeRoundOver(ctx, weap as usize, es.eventParm);
        }

        v if v == entity_event_t::EV_SIEGE_OBJECTIVECOMPLETE as c_int => {
            DEBUGNAME(ctx, "EV_SIEGE_OBJECTIVECOMPLETE");
            let weap = ctx.world.entity(centNum).currentState.weapon;
            let trickedentindex = ctx.world.entity(centNum).currentState.trickedentindex;
            CG_SiegeObjectiveCompleted(ctx, weap as usize, es.eventParm, trickedentindex);
        }

        v if v == entity_event_t::EV_DESTROY_GHOUL2_INSTANCE as c_int => {
            DEBUGNAME(ctx, "EV_DESTROY_GHOUL2_INSTANCE");
            let idx = es.eventParm as usize;
            let ghoul2 = ctx.world.entity(idx).ghoul2;
            if !ghoul2.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, ghoul2) {
                if es.eventParm < MAX_CLIENTS_I32 {
                    //You try to do very bad thing!
                    // (_DEBUG-only warning omitted - not compiled in retail)
                } else {
                    // CleanGhoul2Models nulls the handle through the pointer;
                    // read-modify-write the entity's `ghoul2` field to match.
                    let mut g2 = ctx.world.entity(idx).ghoul2;
                    trap::G2API_CleanGhoul2Models(ctx.engine, &mut g2);
                    ctx.world.entity_mut(idx).ghoul2 = g2;
                }
            }
        }

        v if v == entity_event_t::EV_DESTROY_WEAPON_MODEL as c_int => {
            DEBUGNAME(ctx, "EV_DESTROY_WEAPON_MODEL");
            let idx = es.eventParm as usize;
            let ghoul2 = ctx.world.entity(idx).ghoul2;
            if !ghoul2.is_null()
                && trap::G2_HaveWeGhoul2Models(ctx.engine, ghoul2)
                && trap::G2API_HasGhoul2ModelOnIndex(ctx.engine, ghoul2, 1)
            {
                trap::G2API_RemoveGhoul2Model(ctx.engine, ghoul2, 1);
                ctx.world.entity_mut(idx).ghoul2 = ghoul2;
            }
        }

        v if v == entity_event_t::EV_GIVE_NEW_RANK as c_int => {
            DEBUGNAME(ctx, "EV_GIVE_NEW_RANK");
            let is_me = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.trickedentindex == s.ps.clientNum);
            if is_me {
                trap::Cvar_Set(ctx.engine, "ui_rankChange", &format!("{}", es.eventParm));
                trap::Cvar_Set(ctx.engine, "ui_myteam", &format!("{}", es.bolt2));

                if (trap::Key_GetCatcher(ctx.engine) & KEYCATCH_UI) == 0 && es.bolt1 == 0 {
                    trap::OpenUIMenu(ctx.engine, UIMENU_PLAYERCONFIG);
                }
            }
        }

        v if v == entity_event_t::EV_SET_FREE_SABER as c_int => {
            DEBUGNAME(ctx, "EV_SET_FREE_SABER");
            trap::Cvar_Set(ctx.engine, "ui_freeSaber", &format!("{}", es.eventParm));
        }

        v if v == entity_event_t::EV_SET_FORCE_DISABLE as c_int => {
            DEBUGNAME(ctx, "EV_SET_FORCE_DISABLE");
            trap::Cvar_Set(
                ctx.engine,
                "ui_forcePowerDisable",
                &format!("{}", es.eventParm),
            );
        }

        //
        // missile impacts
        //
        v if v == entity_event_t::EV_CONC_ALT_IMPACT as c_int => {
            DEBUGNAME(ctx, "EV_CONC_ALT_IMPACT");
            // VectorNormalize mutates es->angles in place (a pointer into the
            // entity in Raven); mirror that by writing the normalized angles
            // back to the entity's currentState.
            let mut angles = es.angles;
            let shotDist = VectorNormalize(&mut angles);
            ctx.world.entity_mut(centNum).currentState.angles = angles;

            let mut spot = [0.0; 3];
            let mut dist = 0.0f32;
            while dist < shotDist {
                //one effect would be.. a whole lot better
                _VectorMA(es.origin2, dist, angles, &mut spot);
                let ring = ctx.world.cgs.effects.mConcussionAltRing;
                trap::FX_PlayEffectID(ctx.engine, ring, &spot, &es.angles2, -1, -1);
                dist += 64.0;
            }

            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            CG_MissileHitWall(
                ctx,
                WP_CONCUSSION,
                es.owner,
                position,
                &dir,
                impactSound_t::IMPACTSOUND_DEFAULT,
                true,
                0,
            );

            FX_ConcAltShot(ctx, &es.origin2, &spot);

            //steal the bezier effect from the disruptor
            FX_DisruptorAltMiss(ctx, position, &dir);
        }

        v if v == entity_event_t::EV_MISSILE_STICK as c_int => {
            DEBUGNAME(ctx, "EV_MISSILE_STICK");
            //		trap_S_StartSound (NULL, es->number, CHAN_AUTO, cgs.media.missileStick );
        }

        v if v == entity_event_t::EV_MISSILE_HIT as c_int => {
            DEBUGNAME(ctx, "EV_MISSILE_HIT");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            if es.emplacedOwner != 0 {
                //hack: this is an index to a custom effect to use
                let fx = ctx.world.cgs.gameEffects[es.emplacedOwner as usize];
                trap::FX_PlayEffectID(ctx.engine, fx, position, &dir, -1, -1);
            } else {
                let vwi = {
                    let cent =
                        core::mem::replace(ctx.world.entity_mut(centNum), centity_t::zeroed());
                    let r = CG_VehicleWeaponImpact(ctx, &cent);
                    *ctx.world.entity_mut(centNum) = cent;
                    r
                };
                if vwi {
                    //a vehicle missile that uses an overridden impact effect...
                } else if (es.eFlags & EF_ALT_FIRING) != 0 {
                    CG_MissileHitPlayer(ctx, es.weapon, position, &dir, es.otherEntityNum, true);
                } else {
                    CG_MissileHitPlayer(ctx, es.weapon, position, &dir, es.otherEntityNum, false);
                }
            }

            if ctx.world.cvars.cg_ghoul2Marks.integer != 0 && es.trickedentindex != 0 {
                //flag to place a ghoul2 mark
                CG_G2MarkEvent(ctx, &es);
            }
        }

        v if v == entity_event_t::EV_MISSILE_MISS as c_int => {
            DEBUGNAME(ctx, "EV_MISSILE_MISS");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            if es.emplacedOwner != 0 {
                //hack: this is an index to a custom effect to use
                let fx = ctx.world.cgs.gameEffects[es.emplacedOwner as usize];
                trap::FX_PlayEffectID(ctx.engine, fx, position, &dir, -1, -1);
            } else {
                let vwi = {
                    let cent =
                        core::mem::replace(ctx.world.entity_mut(centNum), centity_t::zeroed());
                    let r = CG_VehicleWeaponImpact(ctx, &cent);
                    *ctx.world.entity_mut(centNum) = cent;
                    r
                };
                if vwi {
                    //a vehicle missile that used an overridden impact effect...
                } else if (es.eFlags & EF_ALT_FIRING) != 0 {
                    CG_MissileHitWall(
                        ctx,
                        es.weapon,
                        0,
                        position,
                        &dir,
                        impactSound_t::IMPACTSOUND_DEFAULT,
                        true,
                        es.generic1,
                    );
                } else {
                    CG_MissileHitWall(
                        ctx,
                        es.weapon,
                        0,
                        position,
                        &dir,
                        impactSound_t::IMPACTSOUND_DEFAULT,
                        false,
                        0,
                    );
                }
            }

            if ctx.world.cvars.cg_ghoul2Marks.integer != 0 && es.trickedentindex != 0 {
                //flag to place a ghoul2 mark
                CG_G2MarkEvent(ctx, &es);
            }
        }

        v if v == entity_event_t::EV_MISSILE_MISS_METAL as c_int => {
            DEBUGNAME(ctx, "EV_MISSILE_MISS_METAL");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            if es.emplacedOwner != 0 {
                //hack: this is an index to a custom effect to use
                let fx = ctx.world.cgs.gameEffects[es.emplacedOwner as usize];
                trap::FX_PlayEffectID(ctx.engine, fx, position, &dir, -1, -1);
            } else {
                let vwi = {
                    let cent =
                        core::mem::replace(ctx.world.entity_mut(centNum), centity_t::zeroed());
                    let r = CG_VehicleWeaponImpact(ctx, &cent);
                    *ctx.world.entity_mut(centNum) = cent;
                    r
                };
                if vwi {
                    //a vehicle missile that used an overridden impact effect...
                } else if (es.eFlags & EF_ALT_FIRING) != 0 {
                    CG_MissileHitWall(
                        ctx,
                        es.weapon,
                        0,
                        position,
                        &dir,
                        impactSound_t::IMPACTSOUND_METAL,
                        true,
                        es.generic1,
                    );
                } else {
                    CG_MissileHitWall(
                        ctx,
                        es.weapon,
                        0,
                        position,
                        &dir,
                        impactSound_t::IMPACTSOUND_METAL,
                        false,
                        0,
                    );
                }
            }
        }

        v if v == entity_event_t::EV_PLAY_EFFECT as c_int => {
            DEBUGNAME(ctx, "EV_PLAY_EFFECT");
            let eID = match es.eventParm {
                //it isn't a hack, it's ingenuity!
                m if m == effectTypes_t::EFFECT_SMOKE as c_int => {
                    ctx.world.cgs.effects.mEmplacedDeadSmoke
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION as c_int => {
                    ctx.world.cgs.effects.mEmplacedExplode
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_PAS as c_int => {
                    ctx.world.cgs.effects.mTurretExplode
                }
                m if m == effectTypes_t::EFFECT_SPARK_EXPLOSION as c_int => {
                    ctx.world.cgs.effects.mSparkExplosion
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_TRIPMINE as c_int => {
                    ctx.world.cgs.effects.mTripmineExplosion
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_DETPACK as c_int => {
                    ctx.world.cgs.effects.mDetpackExplosion
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_FLECHETTE as c_int => {
                    ctx.world.cgs.effects.mFlechetteAltBlow
                }
                m if m == effectTypes_t::EFFECT_STUNHIT as c_int => {
                    ctx.world.cgs.effects.mStunBatonFleshImpact
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_DEMP2ALT as c_int => {
                    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
                    FX_DEMP2_AltDetonate(ctx, &lerpOrigin, es.weapon as f32);
                    ctx.world.cgs.effects.mAltDetonate
                }
                m if m == effectTypes_t::EFFECT_EXPLOSION_TURRET as c_int => {
                    ctx.world.cgs.effects.mTurretExplode
                }
                m if m == effectTypes_t::EFFECT_SPARKS as c_int => {
                    ctx.world.cgs.effects.mSparksExplodeNoSound
                }
                m if m == effectTypes_t::EFFECT_WATER_SPLASH as c_int => {
                    ctx.world.cgs.effects.waterSplash
                }
                m if m == effectTypes_t::EFFECT_ACID_SPLASH as c_int => {
                    ctx.world.cgs.effects.acidSplash
                }
                m if m == effectTypes_t::EFFECT_LAVA_SPLASH as c_int => {
                    ctx.world.cgs.effects.lavaSplash
                }
                m if m == effectTypes_t::EFFECT_LANDING_MUD as c_int => {
                    ctx.world.cgs.effects.landingMud
                }
                m if m == effectTypes_t::EFFECT_LANDING_SAND as c_int => {
                    ctx.world.cgs.effects.landingSand
                }
                m if m == effectTypes_t::EFFECT_LANDING_DIRT as c_int => {
                    ctx.world.cgs.effects.landingDirt
                }
                m if m == effectTypes_t::EFFECT_LANDING_SNOW as c_int => {
                    ctx.world.cgs.effects.landingSnow
                }
                m if m == effectTypes_t::EFFECT_LANDING_GRAVEL as c_int => {
                    ctx.world.cgs.effects.landingGravel
                }
                _ => -1,
            };

            if eID != -1 {
                let mut fxDir = es.angles;
                if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                    fxDir[1] = 1.0;
                }
                trap::FX_PlayEffectID(ctx.engine, eID, &es.origin, &fxDir, -1, -1);
            }
        }

        v if v == entity_event_t::EV_PLAY_EFFECT_ID as c_int
            || v == entity_event_t::EV_PLAY_PORTAL_EFFECT_ID as c_int =>
        {
            DEBUGNAME(ctx, "EV_PLAY_EFFECT_ID");
            //This effect should only be played inside sky portals.
            let portalEffect = event == entity_event_t::EV_PLAY_PORTAL_EFFECT_ID as c_int;

            let mut fxDir = [0.0; 3];
            AngleVectors(es.angles, Some(&mut fxDir), None, None);

            if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
                fxDir[1] = 1.0;
            }

            let mut efxIndex = 0;
            if ctx.world.cgs.gameEffects[es.eventParm as usize] != 0 {
                efxIndex = ctx.world.cgs.gameEffects[es.eventParm as usize];
            } else {
                let s = CG_ConfigString(ctx, CS_EFFECTS + es.eventParm);
                if !s.is_empty() {
                    efxIndex = trap::FX_RegisterEffect(ctx.engine, &s);
                }
            }

            if efxIndex != 0 {
                if portalEffect {
                    trap::FX_PlayPortalEffectID(ctx.engine, efxIndex, position, &fxDir, -1, -1);
                } else {
                    trap::FX_PlayEffectID(ctx.engine, efxIndex, position, &fxDir, -1, -1);
                }
            }
        }

        v if v == entity_event_t::EV_PLAYDOORSOUND as c_int => {
            CG_PlayDoorSound(ctx, centNum, es.eventParm);
        }

        v if v == entity_event_t::EV_PLAYDOORLOOPSOUND as c_int => {
            CG_PlayDoorLoopSound(ctx, centNum);
        }

        v if v == entity_event_t::EV_BMODEL_SOUND as c_int => {
            DEBUGNAME(ctx, "EV_BMODEL_SOUND");
            'bmodel: {
                let soundSet = CG_ConfigString(ctx, CS_AMBIENT_SET + es.soundSetIndex);

                if soundSet.is_empty() {
                    break 'bmodel;
                }

                let sfx = trap::AS_GetBModelSound(ctx.engine, &soundSet, es.eventParm);

                if sfx == -1 {
                    break 'bmodel;
                }

                trap::S_StartSound(ctx.engine, None, es.number, CHAN_AUTO, sfx);
            }
        }

        v if v == entity_event_t::EV_MUTE_SOUND as c_int => {
            DEBUGNAME(ctx, "EV_MUTE_SOUND");
            let idx = es.trickedentindex2 as usize;
            if (ctx.world.entity(idx).currentState.eFlags & EF_SOUNDTRACKER) != 0 {
                ctx.world.entity_mut(idx).currentState.eFlags -= EF_SOUNDTRACKER;
            }
            trap::S_MuteSound(ctx.engine, es.trickedentindex2, es.trickedentindex);
            CG_S_StopLoopingSound(ctx.world, es.trickedentindex2 as usize, -1);
        }

        v if v == entity_event_t::EV_VOICECMD_SOUND as c_int => {
            DEBUGNAME(ctx, "EV_VOICECMD_SOUND");
            'voice: {
                if es.groundEntityNum >= MAX_CLIENTS_I32 {
                    //don't ever use this unless it is being used on a real client
                    break 'voice;
                }

                let mut sfx = ctx.world.cgs.gameSounds[es.eventParm as usize];
                let gnum = es.groundEntityNum as usize;

                let sndStr = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                let descr = CG_GetStringForVoiceSound(ctx, &sndStr);

                if sfx == 0 {
                    let s = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                    sfx = CG_CustomSound(ctx, es.groundEntityNum, &s);
                }

                if sfx != 0 {
                    let ciTeam = ctx.world.cgs.clientinfo[gnum].team;
                    let myTeam = ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize];

                    if es.groundEntityNum != ctx.world.cg.predictedPlayerState.clientNum {
                        //play on the head as well to simulate hearing in radio and in world
                        if ciTeam == myTeam {
                            //don't hear it if this person is on the other team, but they can still
                            //hear it in the world spot.
                            if let Some(snapClientNum) =
                                ctx.world.cg.snap_ref().map(|s| s.ps.clientNum)
                            {
                                trap::S_StartSound(
                                    ctx.engine,
                                    None,
                                    snapClientNum,
                                    CHAN_MENU1,
                                    sfx,
                                );
                            }
                        }
                    }
                    if ciTeam == myTeam {
                        //add to the chat box
                        let ciName = buf_to_string(
                            &ctx.world.cgs.clientinfo[gnum]
                                .name
                                .iter()
                                .map(|&c| c as u8)
                                .collect::<Vec<u8>>(),
                        );
                        let vchatstr = format!("<{ciName}: {descr}>\n");
                        CG_Printf(ctx, &vchatstr);
                        CG_ChatBox_AddString(ctx, ds, &vchatstr);
                    }

                    //and play in world for everyone
                    trap::S_StartSound(ctx.engine, None, es.groundEntityNum, CHAN_VOICE, sfx);
                    let time = ctx.world.cg.time;
                    ctx.world.entity_mut(gnum).vChatTime = time + 1000;
                }
            }
        }

        v if v == entity_event_t::EV_GENERAL_SOUND as c_int => {
            DEBUGNAME(ctx, "EV_GENERAL_SOUND");
            if es.saberEntityNum == trackchan_t::TRACK_CHANNEL_2 as c_int
                || es.saberEntityNum == trackchan_t::TRACK_CHANNEL_3 as c_int
                || es.saberEntityNum == trackchan_t::TRACK_CHANNEL_5 as c_int
            {
                //channels 2 and 3 are for speed and rage, 5 for sight
                let gs = ctx.world.cgs.gameSounds[es.eventParm as usize];
                if gs != 0 {
                    CG_S_AddRealLoopingSound(
                        ctx.world,
                        es.number as usize,
                        es.pos.trBase,
                        vec3_origin,
                        gs,
                    );
                }
            } else {
                let gs = ctx.world.cgs.gameSounds[es.eventParm as usize];
                if gs != 0 {
                    trap::S_StartSound(ctx.engine, None, es.number, es.saberEntityNum, gs);
                } else {
                    let s = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                    let custom = CG_CustomSound(ctx, es.number, &s);
                    trap::S_StartSound(ctx.engine, None, es.number, es.saberEntityNum, custom);
                }
            }
        }

        v if v == entity_event_t::EV_GLOBAL_SOUND as c_int => {
            // play from the player's head so it never diminishes
            DEBUGNAME(ctx, "EV_GLOBAL_SOUND");
            let snapClientNum = ctx.world.cg.snap_ref().map(|s| s.ps.clientNum);
            let gs = ctx.world.cgs.gameSounds[es.eventParm as usize];
            if let Some(snapClientNum) = snapClientNum {
                if gs != 0 {
                    trap::S_StartSound(ctx.engine, None, snapClientNum, CHAN_MENU1, gs);
                } else {
                    let s = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                    let custom = CG_CustomSound(ctx, es.number, &s);
                    trap::S_StartSound(ctx.engine, None, snapClientNum, CHAN_MENU1, custom);
                }
            }
        }

        v if v == entity_event_t::EV_GLOBAL_TEAM_SOUND as c_int => {
            // play from the player's head so it never diminishes
            DEBUGNAME(ctx, "EV_GLOBAL_TEAM_SOUND");
            match es.eventParm {
                m if m == global_team_sound_t::GTS_RED_CAPTURE as c_int => {
                    // CTF: red team captured the blue flag, 1FCTF: red team captured the neutral flag
                    //CG_AddBufferedSound( cgs.media.redScoredSound );
                }
                m if m == global_team_sound_t::GTS_BLUE_CAPTURE as c_int => {
                    // CTF: blue team captured the red flag, 1FCTF: blue team captured the neutral flag
                    //CG_AddBufferedSound( cgs.media.blueScoredSound );
                }
                m if m == global_team_sound_t::GTS_RED_RETURN as c_int => {
                    // CTF: blue flag returned, 1FCTF: never used
                    if ctx.world.cgs.gametype == GT_CTY {
                        let sfx = ctx.world.cgs.media.blueYsalReturnedSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    } else {
                        let sfx = ctx.world.cgs.media.blueFlagReturnedSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    }
                }
                m if m == global_team_sound_t::GTS_BLUE_RETURN as c_int => {
                    // CTF red flag returned, 1FCTF: neutral flag returned
                    if ctx.world.cgs.gametype == GT_CTY {
                        let sfx = ctx.world.cgs.media.redYsalReturnedSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    } else {
                        let sfx = ctx.world.cgs.media.redFlagReturnedSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    }
                }
                m if m == global_team_sound_t::GTS_RED_TAKEN as c_int => {
                    // CTF: red team took blue flag, 1FCTF: blue team took the neutral flag
                    // if this player picked up the flag then a sound is played in CG_CheckLocalSounds
                    if ctx.world.cgs.gametype == GT_CTY {
                        let sfx = ctx.world.cgs.media.redTookYsalSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    } else {
                        let sfx = ctx.world.cgs.media.redTookFlagSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    }
                }
                m if m == global_team_sound_t::GTS_BLUE_TAKEN as c_int => {
                    // CTF: blue team took the red flag, 1FCTF red team took the neutral flag
                    // if this player picked up the flag then a sound is played in CG_CheckLocalSounds
                    if ctx.world.cgs.gametype == GT_CTY {
                        let sfx = ctx.world.cgs.media.blueTookYsalSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    } else {
                        let sfx = ctx.world.cgs.media.blueTookFlagSound;
                        CG_AddBufferedSound(ctx.world, sfx);
                    }
                }
                m if m == global_team_sound_t::GTS_REDTEAM_SCORED as c_int => {
                    let sfx = ctx.world.cgs.media.redScoredSound;
                    CG_AddBufferedSound(ctx.world, sfx);
                }
                m if m == global_team_sound_t::GTS_BLUETEAM_SCORED as c_int => {
                    let sfx = ctx.world.cgs.media.blueScoredSound;
                    CG_AddBufferedSound(ctx.world, sfx);
                }
                m if m == global_team_sound_t::GTS_REDTEAM_TOOK_LEAD as c_int => {
                    let sfx = ctx.world.cgs.media.redLeadsSound;
                    CG_AddBufferedSound(ctx.world, sfx);
                }
                m if m == global_team_sound_t::GTS_BLUETEAM_TOOK_LEAD as c_int => {
                    let sfx = ctx.world.cgs.media.blueLeadsSound;
                    CG_AddBufferedSound(ctx.world, sfx);
                }
                m if m == global_team_sound_t::GTS_TEAMS_ARE_TIED as c_int => {
                    let sfx = ctx.world.cgs.media.teamsTiedSound;
                    CG_AddBufferedSound(ctx.world, sfx);
                }
                _ => {}
            }
        }

        v if v == entity_event_t::EV_ENTITY_SOUND as c_int => {
            DEBUGNAME(ctx, "EV_ENTITY_SOUND");
            //somewhat of a hack - weapon is the caller entity's index, trickedentindex is the proper sound channel
            let gs = ctx.world.cgs.gameSounds[es.eventParm as usize];
            if gs != 0 {
                trap::S_StartSound(ctx.engine, None, es.clientNum, es.trickedentindex, gs);
            } else {
                let s = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                let custom = CG_CustomSound(ctx, es.clientNum, &s);
                trap::S_StartSound(ctx.engine, None, es.clientNum, es.trickedentindex, custom);
            }
        }

        v if v == entity_event_t::EV_PLAY_ROFF as c_int => {
            DEBUGNAME(ctx, "EV_PLAY_ROFF");
            trap::ROFF_Play(ctx.engine, es.weapon, es.eventParm, es.trickedentindex != 0);
        }

        v if v == entity_event_t::EV_GLASS_SHATTER as c_int => {
            DEBUGNAME(ctx, "EV_GLASS_SHATTER");
            CG_GlassShatter(
                ctx,
                es.genericenemyindex as usize,
                &es.origin,
                &es.angles,
                es.trickedentindex as f32,
                es.pos.trTime,
            );
        }

        v if v == entity_event_t::EV_DEBRIS as c_int => {
            DEBUGNAME(ctx, "EV_DEBRIS");
            CG_Chunks(
                ctx,
                es.owner,
                &es.origin,
                &es.angles,
                &es.origin2,
                &es.angles2,
                es.speed,
                es.eventParm,
                es.trickedentindex,
                es.modelindex,
                es.apos.trBase[0],
            );
        }

        v if v == entity_event_t::EV_MISC_MODEL_EXP as c_int => {
            DEBUGNAME(ctx, "EV_MISC_MODEL_EXP");
            CG_MiscModelExplosion(ctx, es.origin2, es.angles2, es.time, es.eventParm);
        }

        v if v == entity_event_t::EV_PAIN as c_int => {
            // local player sounds are triggered in CG_CheckLocalSounds,
            // so ignore events on the player
            DEBUGNAME(ctx, "EV_PAIN");
            let is_local = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.number == s.ps.clientNum);
            if ctx.world.cvars.cg_oldPainSounds.integer == 0 || !is_local {
                CG_PainEvent(ctx, centNum, es.eventParm);
            }
        }

        v if v == entity_event_t::EV_DEATH1 as c_int
            || v == entity_event_t::EV_DEATH2 as c_int
            || v == entity_event_t::EV_DEATH3 as c_int =>
        {
            DEBUGNAME(ctx, "EV_DEATHx");
            let n = event - entity_event_t::EV_DEATH1 as c_int + 1;
            let custom = CG_CustomSound(ctx, es.number, &format!("*death{n}.wav"));
            trap::S_StartSound(ctx.engine, None, es.number, CHAN_VOICE, custom);
            let is_me = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.number == s.ps.clientNum);
            if es.eventParm != 0 && is_me {
                let dramaticFailure = ctx.world.cgs.media.dramaticFailure;
                trap::S_StartLocalSound(ctx.engine, dramaticFailure, CHAN_LOCAL);
                CGCam_SetMusicMult(ctx.world, 0.3, 5000);
            }
        }

        v if v == entity_event_t::EV_OBITUARY as c_int => {
            DEBUGNAME(ctx, "EV_OBITUARY");
            CG_Obituary(ctx, &es);
        }

        //
        // powerup events
        //
        v if v == entity_event_t::EV_POWERUP_QUAD as c_int => {
            DEBUGNAME(ctx, "EV_POWERUP_QUAD");
            let is_me = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.number == s.ps.clientNum);
            if is_me {
                ctx.world.cg.powerupActive = PW_QUAD;
                ctx.world.cg.powerupTime = ctx.world.cg.time;
            }
            //trap_S_StartSound (NULL, es->number, CHAN_ITEM, cgs.media.quadSound );
        }

        v if v == entity_event_t::EV_POWERUP_BATTLESUIT as c_int => {
            DEBUGNAME(ctx, "EV_POWERUP_BATTLESUIT");
            let is_me = ctx
                .world
                .cg
                .snap_ref()
                .is_some_and(|s| es.number == s.ps.clientNum);
            if is_me {
                ctx.world.cg.powerupActive = PW_BATTLESUIT;
                ctx.world.cg.powerupTime = ctx.world.cg.time;
            }
            //trap_S_StartSound (NULL, es->number, CHAN_ITEM, cgs.media.protectSound );
        }

        v if v == entity_event_t::EV_FORCE_DRAINED as c_int => {
            DEBUGNAME(ctx, "EV_FORCE_DRAINED");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            //FX_ForceDrained(position, dir);
            let drainSound = ctx.world.cgs.media.drainSound;
            trap::S_StartSound(ctx.engine, None, es.owner, CHAN_AUTO, drainSound);
            let time = ctx.world.cg.time;
            let cl = ctx.world.entity_mut(es.owner as usize);
            cl.teamPowerEffectTime = time + 1000;
            cl.teamPowerType = 2;
        }

        v if v == entity_event_t::EV_GIB_PLAYER as c_int => {
            DEBUGNAME(ctx, "EV_GIB_PLAYER");
            //trap_S_StartSound( NULL, es->number, CHAN_BODY, cgs.media.gibSound );
            //CG_GibPlayer( cent->lerpOrigin );
        }

        v if v == entity_event_t::EV_STARTLOOPINGSOUND as c_int => {
            DEBUGNAME(ctx, "EV_STARTLOOPINGSOUND");
            let isnd = if ctx.world.cgs.gameSounds[es.eventParm as usize] != 0 {
                ctx.world.cgs.gameSounds[es.eventParm as usize]
            } else {
                let s = CG_ConfigString(ctx, CS_SOUNDS + es.eventParm);
                CG_CustomSound(ctx, es.number, &s)
            };

            CG_S_AddRealLoopingSound(
                ctx.world,
                es.number as usize,
                es.pos.trBase,
                vec3_origin,
                isnd,
            );
            ctx.world.entity_mut(centNum).currentState.loopSound = isnd;
        }

        v if v == entity_event_t::EV_STOPLOOPINGSOUND as c_int => {
            DEBUGNAME(ctx, "EV_STOPLOOPINGSOUND");
            CG_S_StopLoopingSound(ctx.world, es.number as usize, -1);
            ctx.world.entity_mut(centNum).currentState.loopSound = 0;
        }

        v if v == entity_event_t::EV_WEAPON_CHARGE as c_int => {
            DEBUGNAME(ctx, "EV_WEAPON_CHARGE");
            debug_assert!(es.eventParm > WP_NONE && es.eventParm < WP_NUM_WEAPONS);
            let chargeSound = ctx.world.cg_weapons[es.eventParm as usize].chargeSound;
            if chargeSound != 0 {
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_WEAPON, chargeSound);
            } else if es.eventParm == WP_DISRUPTOR {
                let sfx = ctx.world.cgs.media.disruptorZoomLoop;
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_WEAPON, sfx);
            }
        }

        v if v == entity_event_t::EV_WEAPON_CHARGE_ALT as c_int => {
            DEBUGNAME(ctx, "EV_WEAPON_CHARGE_ALT");
            debug_assert!(es.eventParm > WP_NONE && es.eventParm < WP_NUM_WEAPONS);
            let altChargeSound = ctx.world.cg_weapons[es.eventParm as usize].altChargeSound;
            if altChargeSound != 0 {
                trap::S_StartSound(ctx.engine, None, es.number, CHAN_WEAPON, altChargeSound);
            }
        }

        v if v == entity_event_t::EV_SHIELD_HIT as c_int => {
            DEBUGNAME(ctx, "EV_SHIELD_HIT");
            let mut dir = [0.0; 3];
            ByteToDir(es.eventParm, &mut dir);
            CG_PlayerShieldHit(ctx.world, es.otherEntityNum, &mut dir, es.time2);
        }

        v if v == entity_event_t::EV_DEBUG_LINE as c_int => {
            DEBUGNAME(ctx, "EV_DEBUG_LINE");
            CG_Beam(ctx, centNum);
        }

        v if v == entity_event_t::EV_TESTLINE as c_int => {
            DEBUGNAME(ctx, "EV_TESTLINE");
            CG_TestLine(
                ctx.world,
                &es.origin,
                &es.origin2,
                es.time2,
                es.weapon as c_uint,
                1,
            );
        }

        _ => {
            DEBUGNAME(ctx, "UNKNOWN");
            CG_Error(ctx, &format!("Unknown event: {event}"));
        }
    }
}

/// Raven `CG_CheckEvents` - fires the entity's event (event-only entity or a
/// riding `event` change) exactly once per event, then evaluates its position
/// at the current snapshot time before handing off to `CG_EntityEvent`.
///
/// Source: `oracle/codemp/cgame/cg_event.c:3700-3730`
pub fn CG_CheckEvents(ctx: &mut CgContext, ds: &DisplayState, centNum: usize) {
    // check for event-only entities
    {
        let cent = ctx.world.entity_mut(centNum);
        if cent.currentState.eType > entityType_t::ET_EVENTS as c_int {
            if cent.previousEvent != 0 {
                return; // already fired
            }
            // if this is a player event set the entity number of the client entity number
            if cent.currentState.eFlags & EF_PLAYER_EVENT != 0 {
                cent.currentState.number = cent.currentState.otherEntityNum;
            }

            cent.previousEvent = 1;

            cent.currentState.event = cent.currentState.eType - entityType_t::ET_EVENTS as c_int;
        } else {
            // check for events riding with another entity
            if cent.currentState.event == cent.previousEvent {
                return;
            }
            cent.previousEvent = cent.currentState.event;
            if (cent.currentState.event & !EV_EVENT_BITS) == 0 {
                return;
            }
        }
    }

    // calculate the position at exactly the frame time
    let serverTime = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.serverTime,
        // §F19: no snapshot to evaluate the trajectory against yet - nothing
        // to fire the event's position off of, so skip this frame.
        None => return,
    };
    let cent = ctx.world.entity_mut(centNum);
    BG_EvaluateTrajectory(&cent.currentState.pos, serverTime, &mut cent.lerpOrigin);

    CG_SetEntitySoundPosition(ctx, centNum);

    let position = ctx.world.entity(centNum).lerpOrigin;
    CG_EntityEvent(ctx, ds, centNum, &position);
}
