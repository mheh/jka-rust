//! Port of `oracle/codemp/cgame/cg_main.c` — the module entry points, cvar/asset registration and config parsing. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;
use core::ptr::null_mut;

use mp_abi::cgame::shared_buffer::{TCGGetBoltData, TCGVectorData};
use mp_bg::bg_misc::{selected_holdable_tag, BG_CycleInven, BG_GetItemIndexByTag};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::g_item::MAX_ITEM_MODELS;
use mp_bg::public::gametype::GT_TEAM;
use mp_bg::public::item_type::IT_HOLDABLE;
use mp_bg::public::pers_enum::persEnum_t::PERS_ATTACKER;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::stat_index::statIndex_t::STAT_HOLDABLE_ITEM;
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED};
use mp_qshared::common::mp::qcommon::PMF_FOLLOW;
use mp_qshared::shared::cvar::{
    CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INTERNAL, CVAR_ROM, CVAR_SERVERINFO, CVAR_USERINFO,
};
use mp_qshared::shared::force_powers::{
    FP_HEAL, FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, NUM_FORCE_POWERS,
};
use mp_qshared::shared::limits::{MAX_CLIENTS_I32, MAX_STRING_CHARS, MAX_TOKEN_CHARS};
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_math::{_VectorCopy, _VectorSubtract, VectorLength, PITCH, ROLL};
use mp_qshared::shared::{
    fileHandle_t, pc_token_t, qfalse, qhandle_t, qtrue, vec3_t, CIN_LOOP, FS_READ, MAX_GENTITIES,
    MAX_TOKENLENGTH,
};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::{MenuSystem, MAX_MENUFILE};
use mp_uishared::shared::menudef::{FEEDER_BLUETEAM_LIST, FEEDER_REDTEAM_LIST, FEEDER_SCOREBOARD};
use mp_uishared::ui_shared::{
    Menu_SetFeederSelection, PC_Color_Parse, PC_Float_Parse, PC_Int_Parse, PC_String_Parse,
};
use native_string::{atof, atoi, buf_to_string, latin1_to_string, Q_stricmp};

use crate::local::client_info_t::clientInfo_t;
use crate::local::item_info_t::itemInfo_t;
use crate::trap;
use crate::world::{CgContext, CgWorld};

/// Raven `#define MAX_MISC_ENTS` — cgame's client-only "extra visual" registry
/// capacity (`CG_MISC_ENT` shared-buffer vmcall registers into it; `CG_AddMiscEnt`
/// / `CG_ClearMiscEnts` own that array and land in a later wave).
///
/// The oracle spells it twice under an `#ifdef _XBOX` — 500 on Xbox, 4000 on
/// the PC build this port ships. `CG_DrawMiscEnts`'s backing state is a growable
/// `Vec` rather than a fixed-size array (see
/// [`crate::world::cg_main_state::CgMiscEnt`]), so the value only matters to
/// whichever wave ports `CG_AddMiscEnt`'s cap check.
///
/// Source: `oracle/codemp/cgame/cg_main.c:137-141`
pub const MAX_MISC_ENTS: usize = 4000;

/// Raven `#define DEFAULT_MODEL "kyle"` — the userinfo `model` cvar's default.
///
/// Source: `oracle/codemp/cgame/cg_local.h:82`
pub const DEFAULT_MODEL: &str = "kyle";

/// Raven `#define DEFAULT_FORCEPOWERS` — the userinfo `forcepowers` cvar's
/// default (rank-side-powers, all powers at 0).
///
/// Source: `oracle/codemp/cgame/cg_local.h:84`
pub const DEFAULT_FORCEPOWERS: &str = "5-1-000000000000000000";

/// Raven `#define MAX_CGSTRPOOL_SIZE 32768` — capacity of `cg_main.c`'s
/// string-intern pool (`strPool`). No open fn in this wave reads or writes it.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3309`
pub const MAX_CGSTRPOOL_SIZE: usize = 32768;

// DEFERRED: CGFOFS — oracle/codemp/cgame/cg_main.c:3390
// `#define CGFOFS(x) ((int)&(((cgSpawnEnt_t *)0)->x))` computes a field offset into
// `cgSpawnEnt_t` for the spawn-var parse table; that struct and its consuming spawn
// functions aren't in this wave's call surface, so there's nothing to port yet.

/// Raven `CG_NoUseableForce` — true when the local player's known force powers
/// are all combat powers the crosshair-power-select HUD hides (saber
/// throw/offense/defense, levitation).
///
/// Source: `oracle/codemp/cgame/cg_main.c:159-179`
pub fn CG_NoUseableForce(world: &CgWorld) -> bool {
    let mut i = FP_HEAL;
    while i < NUM_FORCE_POWERS {
        if i != FP_SABERTHROW
            && i != FP_SABER_OFFENSE
            && i != FP_SABER_DEFENSE
            && i != FP_LEVITATION
        {
            // valid selectable power
            if world.cg.predictedPlayerState.fd.forcePowersKnown & (1 << i) != 0 {
                // we have it
                return false;
            }
        }
        i += 1;
    }

    // no useable force powers, I guess.
    true
}

/// Raven `C_GetLerpOrigin` — copies entity `data.mEntityNum`'s interpolated
/// origin into the `CG_GET_LERP_ORIGIN` payload.
///
/// Raven works entirely through `cg.sharedBuffer` cast to `TCGVectorData*`; per
/// DEC-46.6 the shared-buffer copy-out/decode lives at the vmMain dispatch call
/// boundary (a later wave), so this fn takes the already-decoded payload
/// directly and the caller is responsible for writing `data` back into the
/// buffer afterward.
///
/// Source: `oracle/codemp/cgame/cg_main.c:369-374`
pub fn C_GetLerpOrigin(world: &CgWorld, data: &mut TCGVectorData) {
    let ent = world.entity(data.mEntityNum as usize);
    _VectorCopy(ent.lerpOrigin, &mut data.mPoint);
}

/// Raven `C_GetLerpData` — only used by the FX system to pass to getboltmat.
/// Fills the `CG_GET_LERP_DATA` payload with entity `data.mEntityNum`'s
/// interpolated origin/scale/angles, zeroing pitch/roll for players and (most)
/// vehicles so bolt-relative FX don't inherit body lean.
///
/// Same DEC-46.6 shape as [`C_GetLerpOrigin`]: the payload is already decoded
/// by the caller, not read from `cg.sharedBuffer` here.
///
/// Source: `oracle/codemp/cgame/cg_main.c:376-406`
pub fn C_GetLerpData(world: &CgWorld, data: &mut TCGGetBoltData) {
    let ent = world.entity(data.mEntityNum as usize);
    _VectorCopy(ent.lerpOrigin, &mut data.mOrigin);
    _VectorCopy(ent.modelScale, &mut data.mScale);
    _VectorCopy(ent.lerpAngles, &mut data.mAngles);

    if ent.currentState.eType == entityType_t::ET_PLAYER as c_int {
        // normal player
        data.mAngles[PITCH] = 0.0;
        data.mAngles[ROLL] = 0.0;
    } else if ent.currentState.eType == entityType_t::ET_NPC as c_int {
        // an NPC
        match ent.m_pVehicle {
            None => {
                // for vehicles, we may or may not want to 0 out pitch and roll
                data.mAngles[PITCH] = 0.0;
                data.mAngles[ROLL] = 0.0;
            }
            Some(_veh) => {
                // DEFERRED: Vehicle_t / m_pVehicleInfo->type — oracle/codemp/cgame/cg_players.c:7014-7042
                // `VehicleId` doesn't resolve to its owning `Vehicle_t`/`vehicleInfo_t` yet
                // (DEC-46.2: "until then ported code only tests presence"), so the
                // VH_SPEEDER-vs-VH_FIGHTER split below can't run. Defaulting to the "not a
                // fighter" outcome (zero both) — 1 of Raven's 3 vehicle branches (a speeder
                // keeps its roll, a fighter keeps both) — until the vehicle referent pool lands.
                data.mAngles[PITCH] = 0.0;
                data.mAngles[ROLL] = 0.0;
            }
        }
    }
}

/// Raven `CG_DrawMiscEnts` — culls and draws the client-only "extra visual"
/// entities registered through `CG_MISC_ENT` (torches, decorative props).
///
/// Source: `oracle/codemp/cgame/cg_main.c:624-657`
pub fn CG_DrawMiscEnts(ctx: &mut CgContext) {
    // `cg.refdef`/`cg.distanceCull` are copied out because the PVS trap wants
    // `cg.snap->areamask` by `&mut` - both live on `cg_t`.
    let vieworg = ctx.world.cg.refdef.vieworg;
    let distanceCull = ctx.world.cg.distanceCull;

    for i in 0..ctx.world.main.miscEnts.len() {
        let mut cullOrigin = ctx.world.main.miscEnts[i].ent.origin;
        cullOrigin[2] += 1.0;

        let zOff = ctx.world.main.miscEnts[i].zOffset;
        if zOff != 0.0 {
            cullOrigin[2] += zOff;
        }

        // Raven's own `cg.snap &&` guard - no snapshot, nothing is in the PVS
        let inPVS = match ctx.world.cg.snap_mut() {
            Some(snap) => trap::R_inPVS(ctx.engine, &vieworg, &cullOrigin, &mut snap.areamask),
            None => false,
        };

        if inPVS {
            let mut difference: vec3_t = [0.0; 3];
            _VectorSubtract(
                ctx.world.main.miscEnts[i].ent.origin,
                vieworg,
                &mut difference,
            );
            if VectorLength(difference) - ctx.world.main.miscEnts[i].radius <= distanceCull {
                trap::R_AddRefEntityToScene(ctx.engine, &ctx.world.main.miscEnts[i].ent);
            }
        }
    }
}

/// Raven `CG_RegisterCvars` — registers every cgame cvar mirror plus the
/// read-only UI/server transfer cvars, and snapshots whether the local
/// machine is also running the server.
///
/// PORT-NOTE: Raven walks a `cvarTable[]` built in a separate declaration order
/// (`cg_main.c:882-1053`) from the field order `CgCvars` documents
/// (`cg_main.c:702-873`); that table's literal row order isn't in this
/// packet, so the registrations below walk `CgCvars`'s declaration order
/// instead — every row still registers with its correct name/default/flags,
/// only the wall-clock registration order (behaviorally inert; each
/// `trap_Cvar_Register` call is independent) differs from Raven's.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1062-1112`
pub fn CG_RegisterCvars(ctx: &mut CgContext) {
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_centertime),
        "cg_centertime",
        "3",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_runpitch),
        "cg_runpitch",
        "0.002",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_runroll),
        "cg_runroll",
        "0.005",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_bobup),
        "cg_bobup",
        "0.005",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_bobpitch),
        "cg_bobpitch",
        "0.002",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_bobroll),
        "cg_bobroll",
        "0.002",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_shadows),
        "cg_shadows",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_renderToTextureFX),
        "cg_renderToTextureFX",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawTimer),
        "cg_drawTimer",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawFPS),
        "cg_drawFPS",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawSnapshot),
        "cg_drawSnapshot",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_draw3dIcons),
        "cg_draw3dIcons",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawIcons),
        "cg_drawIcons",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawAmmoWarning),
        "cg_drawAmmoWarning",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawCrosshair),
        "cg_drawCrosshair",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawCrosshairNames),
        "cg_drawCrosshairNames",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawRadar),
        "cg_drawRadar",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawVehLeadIndicator),
        "cg_drawVehLeadIndicator",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_dynamicCrosshair),
        "cg_dynamicCrosshair",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_dynamicCrosshairPrecision),
        "cg_dynamicCrosshairPrecision",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawRewards),
        "cg_drawRewards",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawScores),
        "cg_drawScores",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_crosshairSize),
        "cg_crosshairSize",
        "24",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_crosshairX),
        "cg_crosshairX",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_crosshairY),
        "cg_crosshairY",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_crosshairHealth),
        "cg_crosshairHealth",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_draw2D),
        "cg_draw2D",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawStatus),
        "cg_drawStatus",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_animSpeed),
        "cg_animspeed",
        "1",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_debugAnim),
        "cg_debuganim",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_debugSaber),
        "cg_debugsaber",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_debugPosition),
        "cg_debugposition",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_debugEvents),
        "cg_debugevents",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_errorDecay),
        "cg_errordecay",
        "100",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_nopredict),
        "cg_nopredict",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_noPlayerAnims),
        "cg_noplayeranims",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_showmiss),
        "cg_showmiss",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_showVehMiss),
        "cg_showVehMiss",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_footsteps),
        "cg_footsteps",
        "3",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_addMarks),
        "cg_marks",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_viewsize),
        "cg_viewsize",
        "100",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawGun),
        "cg_drawGun",
        "1",
        CVAR_ARCHIVE,
    );
    // `cg_gun_frame` — declared and read, never registered in `cvarTable` (CgCvars doc).
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_gun_x),
        "cg_gunX",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_gun_y),
        "cg_gunY",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_gun_z),
        "cg_gunZ",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoswitch),
        "cg_autoswitch",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_ignore),
        "cg_ignore",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_simpleItems),
        "cg_simpleItems",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_fov),
        "cg_fov",
        "80",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_zoomFov),
        "cg_zoomfov",
        "40.0",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_swingAngles),
        "cg_swingAngles",
        "1",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_oldPainSounds),
        "cg_oldPainSounds",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_ragDoll),
        "broadsword",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_jumpSounds),
        "cg_jumpSounds",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoMap),
        "r_autoMap",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoMapX),
        "r_autoMapX",
        "496",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoMapY),
        "r_autoMapY",
        "32",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoMapW),
        "r_autoMapW",
        "128",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_autoMapH),
        "r_autoMapH",
        "128",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.bg_fighterAltControl),
        "bg_fighterAltControl",
        "0",
        CVAR_SERVERINFO,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_chatBox),
        "cg_chatBox",
        "10000",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_chatBoxHeight),
        "cg_chatBoxHeight",
        "350",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberModelTraceEffect),
        "cg_saberModelTraceEffect",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberClientVisualCompensation),
        "cg_saberClientVisualCompensation",
        "1",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_g2TraceLod),
        "cg_g2TraceLod",
        "2",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_fpls),
        "cg_fpls",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_ghoul2Marks),
        "cg_ghoul2Marks",
        "16",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_optvehtrace),
        "com_optvehtrace",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberDynamicMarks),
        "cg_saberDynamicMarks",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberDynamicMarkTime),
        "cg_saberDynamicMarkTime",
        "60000",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberContact),
        "cg_saberContact",
        "1",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_saberTrail),
        "cg_saberTrail",
        "1",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_duelHeadAngles),
        "cg_duelHeadAngles",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_speedTrail),
        "cg_speedTrail",
        "1",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_auraShell),
        "cg_auraShell",
        "1",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_repeaterOrb),
        "cg_repeaterOrb",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_animBlend),
        "cg_animBlend",
        "1",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_dismember),
        "cg_dismember",
        "0",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonSpecialCam),
        "cg_thirdPersonSpecialCam",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPerson),
        "cg_thirdPerson",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonRange),
        "cg_thirdPersonRange",
        "80",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonAngle),
        "cg_thirdPersonAngle",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonPitchOffset),
        "cg_thirdPersonPitchOffset",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonVertOffset),
        "cg_thirdPersonVertOffset",
        "16",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonCameraDamp),
        "cg_thirdPersonCameraDamp",
        "0.3",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonTargetDamp),
        "cg_thirdPersonTargetDamp",
        "0.5",
        CVAR_CHEAT,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonAlpha),
        "cg_thirdPersonAlpha",
        "1.0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_thirdPersonHorzOffset),
        "cg_thirdPersonHorzOffset",
        "0",
        CVAR_CHEAT,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_stereoSeparation),
        "cg_stereoSeparation",
        "0.4",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_lagometer),
        "cg_lagometer",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawEnemyInfo),
        "cg_drawEnemyInfo",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_synchronousClients),
        "g_synchronousClients",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_stats),
        "cg_stats",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_buildScript),
        "com_buildScript",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_forceModel),
        "cg_forceModel",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_paused),
        "cl_paused",
        "0",
        CVAR_ROM,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_blood),
        "com_blood",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_predictItems),
        "cg_predictItems",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_deferPlayers),
        "cg_deferPlayers",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawTeamOverlay),
        "cg_drawTeamOverlay",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_teamOverlayUserinfo),
        "teamoverlay",
        "0",
        CVAR_ROM | CVAR_USERINFO,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_drawFriend),
        "cg_drawFriend",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_teamChatsOnly),
        "cg_teamChatsOnly",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_hudFiles),
        "cg_hudFiles",
        "ui/jahud.txt",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_scorePlum),
        "cg_scorePlums",
        "1",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_smoothClients),
        "cg_smoothClients",
        "1",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.pmove_fixed),
        "pmove_fixed",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.pmove_msec),
        "pmove_msec",
        "8",
        0,
    );

    // `g_showDuelHealths`/`cg_pmove_msec` — declared and read, never registered in `cvarTable`.
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_cameraMode),
        "com_cameraMode",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_cameraOrbit),
        "cg_cameraOrbit",
        "0",
        CVAR_CHEAT,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_cameraOrbitDelay),
        "cg_cameraOrbitDelay",
        "50",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_timescaleFadeEnd),
        "cg_timescaleFadeEnd",
        "1",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_timescaleFadeSpeed),
        "cg_timescaleFadeSpeed",
        "0",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_timescale),
        "timescale",
        "1",
        0,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_noTaunt),
        "cg_noTaunt",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_noProjectileTrail),
        "cg_noProjectileTrail",
        "0",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_debugBB),
        "debugBB",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_currentSelectedPlayer),
        "cg_currentSelectedPlayer",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_currentSelectedPlayerName),
        "cg_currentSelectedPlayerName",
        "",
        CVAR_ARCHIVE,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_recordSPDemo),
        "ui_recordSPDemo",
        "0",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_recordSPDemoName),
        "ui_recordSPDemoName",
        "",
        CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_showVehBounds),
        "cg_showVehBounds",
        "0",
        0,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.ui_myteam),
        "ui_myteam",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    trap::Cvar_Register(
        ctx.engine,
        Some(&mut ctx.world.cvars.cg_snapshotTimeout),
        "cg_snapshotTimeout",
        "10",
        CVAR_ARCHIVE,
    );

    // see if we are also running the server on this machine
    let var = trap::Cvar_VariableStringBuffer(ctx.engine, "sv_running", MAX_TOKEN_CHARS);
    ctx.world.cgs.localServer = atoi(&var);

    ctx.world.main.forceModelModificationCount = ctx.world.cvars.cg_forceModel.modificationCount;

    trap::Cvar_Register(
        ctx.engine,
        None,
        "model",
        DEFAULT_MODEL,
        CVAR_USERINFO | CVAR_ARCHIVE,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "forcepowers",
        DEFAULT_FORCEPOWERS,
        CVAR_USERINFO | CVAR_ARCHIVE,
    );

    // Cvars uses for transferring data between client and server
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_gametype",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_fraglimit",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_capturelimit",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_duellimit",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_timelimit",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_maxclients",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_dmflags",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_mapname",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_hostname",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_needpass",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_about_botminplayers",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm3_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c0_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c1_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c2_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c3_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c4_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm1_c5_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );

    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c0_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c1_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c2_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c3_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c4_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
    trap::Cvar_Register(
        ctx.engine,
        None,
        "ui_tm2_c5_cnt",
        "0",
        CVAR_ROM | CVAR_INTERNAL,
    );
}

/// Raven `CG_CrosshairPlayer` — the clientNum under the crosshair, or `-1` when
/// the crosshair-player timeout has lapsed or the stored slot is out of range.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1189-1200`
pub fn CG_CrosshairPlayer(world: &CgWorld) -> c_int {
    if world.cg.time > world.cg.crosshairClientTime + 1000 {
        return -1;
    }

    if world.cg.crosshairClientNum >= MAX_CLIENTS_I32 {
        return -1;
    }

    world.cg.crosshairClientNum
}

/// Raven `CG_LastAttacker` — the clientNum of whoever last damaged the local
/// player, or `-1` when nothing has.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1202-1207`
pub fn CG_LastAttacker(world: &CgWorld) -> c_int {
    if world.cg.attackerTime == 0 {
        return -1;
    }

    // §F19: Raven derefs `cg.snap` unguarded - with no snapshot we answer `-1`,
    // this fn's own "nobody attacked us" value.
    let Some(snap) = world.cg.snap_ref() else {
        return -1;
    };
    snap.ps.persistant[PERS_ATTACKER as usize]
}

/// Raven `CG_Printf` — Raven's `...`/`vsprintf` formatting collapses to
/// `format!` (dictionary: `va()`/`Com_sprintf` -> `format!`); the caller
/// already hands a fully formatted `msg` string.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1209-1218`
pub fn CG_Printf(ctx: &mut CgContext, msg: &str) {
    trap::Print(ctx.engine, msg);
}

/// Raven `CG_Error` — same `vsprintf`-collapses-to-`format!` shape as
/// [`CG_Printf`].
///
/// Source: `oracle/codemp/cgame/cg_main.c:1220-1229`
pub fn CG_Error(ctx: &mut CgContext, msg: &str) {
    trap::Error(ctx.engine, msg);
}

/// Raven `CG_Argv` — the console command argv slot at `arg`.
///
/// Raven read into a function-scope `static char buffer[MAX_STRING_CHARS]`;
/// the port returns the decoded `String` directly (each call fully
/// repopulates the buffer before use, so there is no cross-call state to
/// preserve).
///
/// Source: `oracle/codemp/cgame/cg_main.c:1263-1269`
pub fn CG_Argv(ctx: &mut CgContext, arg: c_int) -> String {
    trap::Argv(ctx.engine, arg, MAX_STRING_CHARS)
}

/// Raven `BG_GetTime` — the bg-tier's read of the client's current render
/// time, so bg code (which cgame links a second copy of) can read `cg.time`
/// without depending on `cg_t` directly.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1276-1279`
pub fn BG_GetTime(world: &CgWorld) -> c_int {
    world.cg.time
}

/// Raven `CG_ParseWeatherEffect` — forwards a `*`-prefixed weather command
/// string to the renderer, skipping the leading `*`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1395-1400`
pub fn CG_ParseWeatherEffect(ctx: &mut CgContext, s: &str) {
    // pass the '*'
    trap::R_WorldEffectCommand(ctx.engine, &s[1..]);
}

/// Raven `CG_ParseSiegeState` — parses the `roundState|roundTime` siege
/// configstring into `cgSiegeRoundState`/`cgSiegeRoundTime`, latching
/// `cgSiegeRoundBeganTime` when the round enters state 0 (pre-round) or 2
/// (in-progress).
///
/// PORT-NOTE: Raven copies through a fixed `char b[1024]` scratch buffer with
/// no length check (`b[j] = str[i]` unbounded); the port slices `s` directly
/// instead of replaying that buffer, which sidesteps the overflow rather than
/// reproducing it (§F19 spirit: pick the one defined behavior on a Raven UB
/// path). Raven's `prevState`/`if (cgSiegeRoundState != prevState)` guard
/// around the "it changed" block is commented out in the oracle (dead code
/// kept for reference), so this always re-runs unconditionally, matching.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1403-1443`
pub fn CG_ParseSiegeState(world: &mut CgWorld, s: &str) {
    match s.split_once('|') {
        Some((before, after)) => {
            world.saga.cgSiegeRoundState = atoi(before);

            // it changed
            world.saga.cgSiegeRoundTime = atoi(after);
            if world.saga.cgSiegeRoundState == 0 || world.saga.cgSiegeRoundState == 2 {
                world.draw.cgSiegeRoundBeganTime = world.saga.cgSiegeRoundTime;
            }
        }
        None => {
            world.saga.cgSiegeRoundState = atoi(s);
            world.saga.cgSiegeRoundTime = world.cg.time;
        }
    }
}

/// Raven `CG_GetStringEdString` — looks up `refSection_refName` in the
/// StringEd table.
///
/// Raven alternates between two static buffers (`text[2][1024]`) so a nested
/// call doesn't clobber the caller's still-live result; returning an owned
/// `String` per call makes that aliasing workaround unnecessary.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2429-2437`
pub fn CG_GetStringEdString(ctx: &mut CgContext, refSection: &str, refName: &str) -> String {
    let key = format!("{refSection}_{refName}");
    trap::SP_GetStringTextString(ctx.engine, &key, MAX_STRING_CHARS)
}

/// Raven `CG_GetMenuBuffer` — reads a menu script file into memory.
///
/// Raven read into a function-scope `static char buf[MAX_MENUFILE]` and
/// returned `NULL` on failure; the port reads into a local buffer sized to
/// the file length and returns `None` on the same two failure paths (not
/// found, too large).
///
/// Source: `oracle/codemp/cgame/cg_main.c:2563-2584`
pub fn CG_GetMenuBuffer(ctx: &mut CgContext, filename: &str) -> Option<String> {
    let mut f: fileHandle_t = 0;
    let len = trap::FS_FOpenFile(ctx.engine, filename, &mut f, FS_READ);
    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file not found: {}, using default\n",
                S_COLOR_RED.to_str().unwrap(),
                filename
            ),
        );
        return None;
    }

    if len >= MAX_MENUFILE as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                filename,
                len,
                MAX_MENUFILE
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return None;
    }

    let mut buf = vec![0u8; len as usize];
    trap::FS_Read(ctx.engine, &mut buf, f);
    trap::FS_FCloseFile(ctx.engine, f);
    // Raven hands back a `char *` closed with `buf[len] = 0`, so an embedded NUL
    // ends the menu text early - the decode stops on the same byte.
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(latin1_to_string(&buf[..end]))
}

/// `CG_Asset_Parse`'s `pc_token_t token;` stack local. Raven leaves it
/// uninitialized and lets `trap_PC_ReadToken` fill it; the port hands the trap a
/// zeroed one.
fn zero_pc_token() -> pc_token_t {
    pc_token_t {
        type_: 0,
        subtype: 0,
        intvalue: 0,
        floatvalue: 0.0,
        string: [0; MAX_TOKENLENGTH],
    }
}

/// Decode a `pc_token_t.string` fixed buffer into an owned `String` — the port's
/// `pc_token_t.string` is a byte buffer, not a Rust string, so the token
/// comparisons Raven spells against `token.string` go through here.
fn pc_token_str(token: &pc_token_t) -> String {
    buf_to_string(&token.string.iter().map(|&c| c as u8).collect::<Vec<u8>>())
}

/// Raven `CG_Asset_Parse` — parses the `assetGlobalDef` block at the head of a
/// menu file into `cgDC.Assets`.
///
/// The font arms parse a point size Raven then throws away (its
/// `registerFont(name, pointSize, …)` call is commented out) — the read still
/// has to happen, it consumes the token.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2592-2751`
#[allow(clippy::too_many_lines)]
pub fn CG_Asset_Parse(
    ctx: &mut CgContext,
    ds: &mut DisplayState,
    dc: &mut dyn DisplayContext,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();

    if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
        return false;
    }
    if Q_stricmp(&pc_token_str(&token), "{") != 0 {
        return false;
    }

    loop {
        if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
            return false;
        }

        let tokenStr = pc_token_str(&token);

        if Q_stricmp(&tokenStr, "}") == 0 {
            return true;
        }

        // font
        if Q_stricmp(&tokenStr, "font") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhMediumFont = dc.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // smallFont
        if Q_stricmp(&tokenStr, "smallFont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhSmallFont = dc.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // smallFont
        if Q_stricmp(&tokenStr, "small2Font") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhSmall2Font = dc.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // font
        if Q_stricmp(&tokenStr, "bigfont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(dc, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhBigFont = dc.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // gradientbar
        if Q_stricmp(&tokenStr, "gradientbar") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ds.Assets.gradientBar = trap::R_RegisterShaderNoMip(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // enterMenuSound
        if Q_stricmp(&tokenStr, "menuEnterSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ds.Assets.menuEnterSound = trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // exitMenuSound
        if Q_stricmp(&tokenStr, "menuExitSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ds.Assets.menuExitSound = trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // itemFocusSound
        if Q_stricmp(&tokenStr, "itemFocusSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ds.Assets.itemFocusSound = trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        // menuBuzzSound
        if Q_stricmp(&tokenStr, "menuBuzzSound") == 0 {
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
                return false;
            }
            ds.Assets.menuBuzzSound = trap::S_RegisterSound(ctx.engine, &pc_token_str(&token));
            continue;
        }

        if Q_stricmp(&tokenStr, "cursor") == 0 {
            if !PC_String_Parse(dc, handle, &mut ds.Assets.cursorStr) {
                return false;
            }
            let cursorStr = ds.Assets.cursorStr.clone();
            ds.Assets.cursor = trap::R_RegisterShaderNoMip(ctx.engine, &cursorStr);
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeClamp") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ds.Assets.fadeClamp) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeCycle") == 0 {
            if !PC_Int_Parse(dc, handle, &mut ds.Assets.fadeCycle) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeAmount") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ds.Assets.fadeAmount) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowX") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ds.Assets.shadowX) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowY") == 0 {
            if !PC_Float_Parse(dc, handle, &mut ds.Assets.shadowY) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowColor") == 0 {
            if !PC_Color_Parse(dc, handle, &mut ds.Assets.shadowColor) {
                return false;
            }
            ds.Assets.shadowFadeClamp = ds.Assets.shadowColor[3];
            continue;
        }
    }
    // Raven: "bk001204 - why not?" — the `while(1)` never falls through, so the
    // trailing `return qfalse` is unreachable and has no Rust counterpart.
}

/// Raven `CG_OwnerDrawHandleKey` — cgame's ownerdraw key handler: it eats
/// nothing, ever. (ui's is the one with real handlers.)
///
/// Source: `oracle/codemp/cgame/cg_main.c:2829-2831`
pub fn CG_OwnerDrawHandleKey(
    _ownerDraw: c_int,
    _flags: c_int,
    _special: &mut f32,
    _key: c_int,
) -> bool {
    false
}

/// Raven `CG_FeederCount` — how many rows the scoreboard feeder `feederID` has.
///
/// PORT-NOTE: Raven compares the `float feederID` against the int `FEEDER_*`
/// defines; the port casts once to `c_int` and compares ints (ui's
/// `UI_FeederCount` idiom — the framework only ever passes exact integers).
///
/// Source: `oracle/codemp/cgame/cg_main.c:2834-2853`
pub fn CG_FeederCount(world: &CgWorld, feederID: f32) -> c_int {
    let feeder = feederID as c_int;
    let mut count = 0;

    if feeder == FEEDER_REDTEAM_LIST {
        for i in 0..world.cg.numScores as usize {
            if world.cg.scores[i].team == TEAM_RED {
                count += 1;
            }
        }
    } else if feeder == FEEDER_BLUETEAM_LIST {
        for i in 0..world.cg.numScores as usize {
            if world.cg.scores[i].team == TEAM_BLUE {
                count += 1;
            }
        }
    } else if feeder == FEEDER_SCOREBOARD {
        return world.cg.numScores;
    }

    count
}

/// Raven `CG_SetScoreSelection` — parks `cg.selectedScore` on the local
/// player's row and moves `menu`'s feeder cursor there. `None` is Raven's NULL
/// menu: set the score, draw nothing.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2856-2888`
pub fn CG_SetScoreSelection(
    world: &mut CgWorld,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    p: Option<MenuId>,
) {
    // §F19: on the null-snap path Raven reads through a null `ps`. -1 matches no
    // `scores[i].client`, so `cg.selectedScore` keeps its previous value and the
    // rest of the fn runs exactly as Raven's.
    let clientNum = world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    let mut red = 0;
    let mut blue = 0;
    for i in 0..world.cg.numScores as usize {
        if world.cg.scores[i].team == TEAM_RED {
            red += 1;
        } else if world.cg.scores[i].team == TEAM_BLUE {
            blue += 1;
        }
        if clientNum == world.cg.scores[i].client {
            world.cg.selectedScore = i as c_int;
        }
    }

    if p.is_none() {
        // just interested in setting the selected score
        return;
    }

    if world.cgs.gametype >= GT_TEAM {
        let mut feeder = FEEDER_REDTEAM_LIST;
        let mut i = red;
        if world.cg.scores[world.cg.selectedScore as usize].team == TEAM_BLUE {
            feeder = FEEDER_BLUETEAM_LIST;
            i = blue;
        }
        Menu_SetFeederSelection(menus, ds, dc, p, feeder, i, None);
    } else {
        Menu_SetFeederSelection(
            menus,
            ds,
            dc,
            p,
            FEEDER_SCOREBOARD,
            world.cg.selectedScore,
            None,
        );
    }
}

/// Raven `CG_InfoFromScoreIndex` — the client info behind row `index` of a
/// team feeder, plus the `cg.scores` row it came from (Raven's `*scoreIndex`
/// out-param).
///
/// Source: `oracle/codemp/cgame/cg_main.c:2891-2907`
pub fn CG_InfoFromScoreIndex(world: &CgWorld, index: c_int, team: c_int) -> (&clientInfo_t, c_int) {
    if world.cgs.gametype >= GT_TEAM {
        let mut count = 0;
        for i in 0..world.cg.numScores {
            if world.cg.scores[i as usize].team == team {
                if count == index {
                    let client = world.cg.scores[i as usize].client as usize;
                    return (&world.cgs.clientinfo[client], i);
                }
                count += 1;
            }
        }
    }

    let client = world.cg.scores[index as usize].client as usize;
    (&world.cgs.clientinfo[client], index)
}

/// Raven `CG_FeederItemImage` — cgame's feeders are text-only, so every row
/// image is the null handle.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2996-2998`
pub fn CG_FeederItemImage(_feederID: f32, _index: c_int) -> qhandle_t {
    0
}

/// Raven `CG_FeederSelection` — the user clicked row `index` of a scoreboard
/// feeder; remember which `cg.scores` row that is.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3000-3018`
pub fn CG_FeederSelection(
    world: &mut CgWorld,
    feederID: f32,
    index: c_int,
    _item: Option<ItemId>,
) -> bool {
    if world.cgs.gametype >= GT_TEAM {
        let team = if feederID as c_int == FEEDER_REDTEAM_LIST {
            TEAM_RED
        } else {
            TEAM_BLUE
        };
        let mut count = 0;
        for i in 0..world.cg.numScores as usize {
            if world.cg.scores[i].team == team {
                if index == count {
                    world.cg.selectedScore = i as c_int;
                }
                count += 1;
            }
        }
    } else {
        world.cg.selectedScore = index;
    }

    true
}

/// Raven `CG_Cvar_Get` — the menu framework's cvar-as-float read.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3020-3025`
pub fn CG_Cvar_Get(ctx: &mut CgContext, cvar: &str) -> f32 {
    let buff = trap::Cvar_VariableStringBuffer(ctx.engine, cvar, 128);
    atof(&buff) as f32
}

/// Raven `CG_PlayCinematic` — the framework's cinematic slot; cgame always
/// loops.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3053-3055`
pub fn CG_PlayCinematic(ctx: &mut CgContext, name: &str, x: f32, y: f32, w: f32, h: f32) -> c_int {
    trap::CIN_PlayCinematic(
        ctx.engine, name, x as c_int, y as c_int, w as c_int, h as c_int, CIN_LOOP,
    )
}

/// Raven `CG_StopCinematic`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3057-3059`
pub fn CG_StopCinematic(ctx: &mut CgContext, handle: c_int) {
    trap::CIN_StopCinematic(ctx.engine, handle);
}

/// Raven `CG_DrawCinematic`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3061-3064`
pub fn CG_DrawCinematic(ctx: &mut CgContext, handle: c_int, x: f32, y: f32, w: f32, h: f32) {
    trap::CIN_SetExtents(
        ctx.engine, handle, x as c_int, y as c_int, w as c_int, h as c_int,
    );
    trap::CIN_DrawCinematic(ctx.engine, handle);
}

/// Raven `CG_RunCinematicFrame`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3066-3068`
pub fn CG_RunCinematicFrame(ctx: &mut CgContext, handle: c_int) {
    trap::CIN_RunCinematic(ctx.engine, handle);
}

/// Raven `CG_AssetCache` — registers the menu framework's own art (scrollbars,
/// sliders, the fx swatches) into `cgDC.Assets`.
///
/// The `ASSET_*`/`ART_FX_*` shader names are `ui_shared.h` defines; they inline
/// here the way `AssetCache` inlined them on the ui side.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3221-3244`,
/// `oracle/codemp/ui/ui_shared.h:79-98`
pub fn CG_AssetCache(ctx: &mut CgContext, ds: &mut DisplayState) {
    ds.Assets.gradientBar = trap::R_RegisterShaderNoMip(ctx.engine, "ui/assets/gradientbar2.tga");
    ds.Assets.fxBasePic = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_base");
    ds.Assets.fxPic[0] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_red");
    ds.Assets.fxPic[1] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_yel");
    ds.Assets.fxPic[2] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_grn");
    ds.Assets.fxPic[3] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_teal");
    ds.Assets.fxPic[4] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_blue");
    ds.Assets.fxPic[5] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_cyan");
    ds.Assets.fxPic[6] = trap::R_RegisterShaderNoMip(ctx.engine, "menu/art/fx_white");
    ds.Assets.scrollBar = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar.tga");
    ds.Assets.scrollBarArrowDown =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_dwn_a.tga");
    ds.Assets.scrollBarArrowUp =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_up_a.tga");
    ds.Assets.scrollBarArrowLeft =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_left.tga");
    ds.Assets.scrollBarArrowRight =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_arrow_right.tga");
    ds.Assets.scrollBarThumb =
        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/scrollbar_thumb.tga");
    ds.Assets.sliderBar = trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/slider");
    ds.Assets.sliderThumb = trap::R_RegisterShaderNoMip(ctx.engine, "menu/new/sliderthumb");
}

/// Raven `CG_Init_CG` — wipes `cg` between map loads.
///
/// ESCALATION: blocked on a safe zero-fill for `cg_t`. Raven's body is one
/// `memset( &cg, 0, sizeof(cg) )`; `cg_t` is the ~295 KB `#[repr(C)]` C1 port
/// with no `zeroed()`/`Default`, and the only zero-fill in the crate is
/// `CgWorld::new_boxed`'s `unsafe write_bytes` — which lives in `cg_world.rs`, a
/// file a wave transcriber may not touch, and this wave may not write `unsafe`
/// either. A by-value literal is out too: a 295 KB `cg_t` on the stack is the
/// guard-page overflow `CgWorld::new_boxed` documents. Needs `cg_t::zeroed()`
/// (in-place, boxed) before this fn can be filled in.
///
/// The `_XBOX` `widescreen` save/restore around the memset is not in the MP PC
/// build.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3254-3263`
pub fn CG_Init_CG(world: &mut CgWorld) {
    let _ = world;
    todo!("CG_Init_CG — blocked on a safe cg_t zero-fill, oracle/codemp/cgame/cg_main.c:3254-3263")
}

/// Raven `CG_Init_CGents` — wipes the entity array between map loads.
///
/// ESCALATION: blocked on a safe zero-fill for `centity_t`, the same gap
/// [`CG_Init_CG`] hit — `memset(&cg_entities, 0, sizeof(cg_entities))` needs
/// `centity_t::zeroed()` (`local/centity_s.rs`, outside this wave's two files)
/// or an `unsafe` fill this wave may not write.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3274-3278`
pub fn CG_Init_CGents(world: &mut CgWorld) {
    let _ = world;
    todo!(
        "CG_Init_CGents — blocked on a safe centity_t zero-fill, oracle/codemp/cgame/cg_main.c:3274-3278"
    )
}

/// Raven `CG_InitItems` — drops every item's registered models/icons.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3281-3284`
pub fn CG_InitItems(world: &mut CgWorld) {
    // `itemInfo_t` is small enough to spell the all-zero value out, so the
    // memset lands without an `unsafe` fill.
    for item in world.cg_items.iter_mut() {
        *item = itemInfo_t {
            registered: qfalse,
            models: [0; MAX_ITEM_MODELS],
            icon: 0,
            g2Models: [null_mut(); MAX_ITEM_MODELS],
            radius: [0.0; MAX_ITEM_MODELS],
        };
    }
}

/// Raven `CG_TransitionPermanent` — asks the engine for every entity's default
/// state and latches the ones it has (the RMG's permanent entities) into
/// `cg_permanents`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3286-3304`
pub fn CG_TransitionPermanent(ctx: &mut CgContext) {
    ctx.world.main.cg_permanents.clear();

    for i in 0..MAX_GENTITIES {
        if trap::GetDefaultState(
            ctx.engine,
            i as c_int,
            &mut ctx.world.entities[i].currentState,
        ) {
            let cent = &mut ctx.world.entities[i];
            cent.nextState = cent.currentState;
            _VectorCopy(cent.currentState.origin, &mut cent.lerpOrigin);
            _VectorCopy(cent.currentState.angles, &mut cent.lerpAngles);
            cent.currentValid = qtrue;

            ctx.world.main.cg_permanents.push(i);
        }
    }
}

/// Raven `CG_StrPool_Reset` — rewinds the string-intern bump pointer, which is
/// how the pool is freed.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3331-3334`
pub fn CG_StrPool_Reset(world: &mut CgWorld) {
    world.main.cg_strPoolSize = 0;
}

/// Raven `cgSpawnEnt_t` — the scratch record `CG_ParseEntityFromSpawnVars`
/// parses one map entity into before handing it to the `CG_Create*FromSpawnEnt`
/// consumers below. File-local to `cg_main.c` and never crosses the seam, so it
/// takes the idiomatic shape (`char *` → `String`); Raven's `BG_field_t
/// cg_spawnFields[]` offset table (`CGFOFS`) is the parser's concern, not this
/// wave's.
///
/// Type definition source: `oracle/codemp/cgame/cg_main.c:3373-3388`
#[derive(Clone, Default)]
pub struct CgSpawnEnt {
    pub classname: String,
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub angle: f32,
    pub scale: vec3_t,
    pub fScale: f32,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub model: String,
    pub zoffset: f32,
    pub onlyFogHere: c_int,
    pub fogstart: f32,
    pub radarrange: f32,
}

/// Raven `CG_CreateSkyPortalFromSpawnEnt` — a `misc_skyportal` with
/// `onlyFogHere` set moves all global fog inside the portal.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3418-3424`
pub fn CG_CreateSkyPortalFromSpawnEnt(world: &mut CgWorld, ent: &CgSpawnEnt) {
    if ent.onlyFogHere != 0 {
        // only globally fog INSIDE the sky portal
        world.main.cg_noFogOutsidePortal = true;
    }
}

/// Raven `CG_CreateSkyOriFromSpawnEnt` — latches the sky portal's parallax
/// origin and scale for `CG_DrawSkyBoxPortal`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3432-3437`
pub fn CG_CreateSkyOriFromSpawnEnt(world: &mut CgWorld, ent: &CgSpawnEnt) {
    world.main.cg_skyOri = true;
    _VectorCopy(ent.origin, &mut world.main.cg_skyOriPos);
    world.main.cg_skyOriScale = ent.fScale;
}

/// Raven `CG_CreateBrushEntData` — fills a spawned brush model's bounds from
/// the registered model.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3440-3443`
pub fn CG_CreateBrushEntData(ctx: &mut CgContext, ent: &mut CgSpawnEnt) {
    let model = trap::R_RegisterModel(ctx.engine, &ent.model);
    trap::R_ModelBounds(ctx.engine, model, &mut ent.mins, &mut ent.maxs);
}

/// Raven `CG_GetLocationString` — resolves a `@`-prefixed location reference
/// through the string package; anything else is already the display string.
///
/// PORT-NOTE: Raven answered out of a function-scope `static char text[1024]`;
/// the port returns an owned `String` per call, so there is no buffer left to
/// hand a caller the previous lookup's text.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3936-3947`
pub fn CG_GetLocationString(ctx: &mut CgContext, loc: &str) -> String {
    if !loc.starts_with('@') {
        // just a raw string
        return loc.to_string();
    }

    trap::SP_GetStringTextString(ctx.engine, &loc[1..], 1024)
}

/// Raven `CG_NextInventory_f` — cycles the held-item selection forward.
///
/// Source: `oracle/codemp/cgame/cg_main.c:4112-4140`
pub fn CG_NextInventory_f(world: &mut CgWorld) {
    CG_CycleInventory(world, 1);
}

/// Raven `CG_PrevInventory_f` — cycles the held-item selection backward.
///
/// Source: `oracle/codemp/cgame/cg_main.c:4142-4170`
pub fn CG_PrevInventory_f(world: &mut CgWorld) {
    CG_CycleInventory(world, -1);
}

/// The body Raven duplicated verbatim into `CG_NextInventory_f` and
/// `CG_PrevInventory_f` — the only difference is the `BG_CycleInven` direction.
///
/// Source: `oracle/codemp/cgame/cg_main.c:4112-4170`
fn CG_CycleInventory(world: &mut CgWorld, direction: c_int) {
    if world.cg.snap.is_null() {
        return;
    }

    // §F19: `cg.snap` is non-null here, so the `None` arms below are Raven's
    // null deref and can only answer by doing nothing.
    let Some(snap) = world.cg.snap_ref() else {
        return;
    };
    if snap.ps.pm_flags & PMF_FOLLOW != 0 {
        return;
    }

    if world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        return;
    }

    if world.cg.itemSelect != -1 {
        let itemSelect = world.cg.itemSelect;
        let Some(snap) = world.cg.snap_mut() else {
            return;
        };
        snap.ps.stats[STAT_HOLDABLE_ITEM as usize] = BG_GetItemIndexByTag(itemSelect, IT_HOLDABLE);
    }

    let Some(snap) = world.cg.snap_mut() else {
        return;
    };
    BG_CycleInven(&mut snap.ps, direction);

    // `selected_holdable_tag` IS Raven's
    // `bg_itemlist[ps->stats[STAT_HOLDABLE_ITEM]].giTag` read.
    let tag = match world.cg.snap_ref() {
        Some(snap) if snap.ps.stats[STAT_HOLDABLE_ITEM as usize] != 0 => {
            Some(selected_holdable_tag(&snap.ps))
        }
        _ => None,
    };
    if let Some(tag) = tag {
        world.cg.itemSelect = tag;
        world.cg.invenSelectTime = world.cg.time as f32;
    }
}
