//! Port of `oracle/codemp/cgame/cg_event.c` — entity-event handling — obituaries, pickups, impacts. Functions land via the C5
//! transcription waves.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_bg::bg_misc::{BG_CycleInven, BG_FindItemForHoldable, BG_GiveMeVectorFromMatrix};
use mp_bg::bg_panimate::BG_InKnockDownOnly;
use mp_bg::cstr_util::cstr_to_str;
use mp_bg::local::bg_customSiegeSoundNames;
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::configstring::CS_PLAYERS;
use mp_bg::public::ctf_msg::ctfMsg_t;
use mp_bg::public::entity_event::entity_event_t::EV_USE_ITEM0;
use mp_bg::public::entity_flags::{EF_DEAD, EF_JETPACK_ACTIVE};
use mp_bg::public::gametype::{GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, GT_TEAM};
use mp_bg::public::gender::gender_t;
use mp_bg::public::holdable::{
    HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK, HI_MEDPAC,
    HI_MEDPAC_BIG, HI_NONE, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD,
};
use mp_bg::public::item_type::{IT_TEAM, IT_WEAPON};
use mp_bg::public::means_of_death::meansOfDeath_t;
use mp_bg::public::pers_enum::persEnum_t::{PERS_RANK, PERS_SCORE};
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_REDFLAG};
use mp_bg::public::weaponstate::weaponstate_t;
use mp_bg::public::{team_t, RANK_TIED_FLAG, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::vehicles::vehicle_s::{Vehicle_t, MAX_VEHICLES, VEHICLE_BASE};
use mp_bg::weapons::weapon_t::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use mp_qshared::common::mp::qcommon::entityState_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::limits::MAX_VEH_WEAPONS;
use mp_qshared::shared::q_color::S_COLOR_WHITE;
use mp_qshared::shared::q_math::{vec3_origin, YAW};
use mp_qshared::shared::{
    mdxaBone_t, qfalse, vec3_t, Eorientations, BIGCHAR_WIDTH, CHAN_AUTO, CHAN_VOICE,
    ENTITYNUM_NONE, ENTITYNUM_WORLD, MASK_PLAYERSOLID, MAX_CLIENTS_I32, SCREEN_HEIGHT,
};
use native_string::{buf_to_string, string_to_latin1, Info_ValueForKey, Q_strncpyzBytes};

use crate::cg_draw::CG_CenterPrint;
use crate::cg_main::{CG_ConfigString, CG_Error, CG_GetStringEdString, CG_Printf, Com_Printf};
use crate::cg_players::{CG_AddGhoul2Mark, CG_CustomSound, CG_ThereIsAMaster};
use crate::cg_predict::CG_G2Trace;
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::MAX_CUSTOM_SIEGE_SOUNDS;
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
