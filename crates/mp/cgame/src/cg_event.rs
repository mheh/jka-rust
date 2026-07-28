//! Port of `oracle/codemp/cgame/cg_event.c` — entity-event handling — obituaries, pickups, impacts. Functions land via the C5
//! transcription waves.
#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_bg::bg_misc::{BG_CycleInven, BG_FindItemForHoldable, BG_GiveMeVectorFromMatrix};
use mp_bg::local::bg_customSiegeSoundNames;
use mp_bg::public::entity_event::entity_event_t::EV_USE_ITEM0;
use mp_bg::public::holdable::{
    HI_AMMODISP, HI_BINOCULARS, HI_CLOAK, HI_EWEB, HI_HEALTHDISP, HI_JETPACK, HI_MEDPAC,
    HI_MEDPAC_BIG, HI_NONE, HI_NUM_HOLDABLE, HI_SEEKER, HI_SENTRY_GUN, HI_SHIELD,
};
use mp_bg::public::weaponstate::weaponstate_t;
use mp_bg::public::{team_t, RANK_TIED_FLAG, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::vehicles::vehicle_s::Vehicle_t;
use mp_qshared::common::mp::qcommon::entityState_t;
use mp_qshared::shared::{mdxaBone_t, vec3_t, Eorientations, CHAN_AUTO, MAX_CLIENTS_I32};

use crate::cg_main::CG_GetStringEdString;
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
        // DEFERRED: CG_Printf (oracle/codemp/cgame/cg_main.c:1209) isn't
        // ported yet — it's cg_main.c's varargs wrapper over trap_Print.
        // Raven's macro hands the caller's string straight through as the
        // format string (no substitution), so calling trap_Print directly is
        // the faithful stand-in until CG_Printf lands.
        trap::Print(ctx.engine, &format!("{x}\n"));
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
    let s_st = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_ST", 10);
    let s_nd = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_ND", 10);
    let s_rd = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_RD", 10);
    let s_th = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_NUMBER_TH", 10);
    let mut s_tied_for = trap::SP_GetStringTextString(ctx.engine, "MP_INGAME_TIED_FOR", 64);
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
