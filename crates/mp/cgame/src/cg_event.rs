//! Port of `oracle/codemp/cgame/cg_event.c` — entity-event handling — obituaries, pickups, impacts. Functions land via the C5
//! transcription waves.
#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::public::weaponstate::weaponstate_t;
use mp_bg::public::{team_t, RANK_TIED_FLAG, TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::vehicles::vehicle_s::Vehicle_t;
use mp_qshared::common::mp::qcommon::entityState_t;
use mp_qshared::shared::{mdxaBone_t, vec3_t, Eorientations, CHAN_AUTO, MAX_CLIENTS_I32};

use crate::local::centity_s::centity_t;
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
pub fn CG_ReattachLimb(ctx: &mut CgContext, source: &mut centity_t) {
    let number = source.currentState.number;

    let torso_skin = if number >= MAX_CLIENTS_I32 {
        source.npcClient.as_deref().map(|ci| ci.torsoSkin)
    } else {
        Some(ctx.world.cgs.clientinfo[number as usize].torsoSkin)
    };

    // re-apply the skin
    if let Some(torso_skin) = torso_skin {
        if torso_skin > 0 {
            trap::G2API_SetSkin(ctx.engine, source.ghoul2, 0, torso_skin, torso_skin);
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
    todo!("CG_VehMuzzleFireFX muzzle FX loop — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_event.c:1394-1426")
}
