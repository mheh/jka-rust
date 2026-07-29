//! Port of `oracle/codemp/cgame/cg_main.c` — the module entry points, cvar/asset registration and config parsing. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_uint, CStr};
use core::ptr::null_mut;

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::public::rag_callback_bone_snap_t::ragCallbackBoneSnap_t;
use mp_abi::cgame::public::rag_callback_debug_box_t::ragCallbackDebugBox_t;
use mp_abi::cgame::public::rag_callback_debug_line_t::ragCallbackDebugLine_t;
use mp_abi::cgame::public::rag_callback_trace_line_t::ragCallbackTraceLine_t;
use mp_abi::cgame::shared_buffer::{
    autoMapInput_t, TCGCameraShake, TCGG2Mark, TCGGetBoltData, TCGImpactMark, TCGMiscEnt,
    TCGPointContents, TCGTrace, TCGVectorData,
};
use mp_bg::bg_channel::PmoveContext;
use mp_bg::bg_misc::{
    forcePowerSorted, selected_holdable_tag, BG_CycleForce, BG_CycleInven, BG_FindItemForPowerup,
    BG_FindItemForWeapon, BG_GetItemIndexByTag,
};
use mp_bg::bg_panimate::BG_ClearAnimsets;
use mp_bg::bg_saberLoad::WP_SaberLoadParms;
use mp_bg::bg_vehicleLoad::BG_VehicleLoadParms;
use mp_bg::public::bg_itemlist::{bg_itemlist, bg_numItems};
use mp_bg::public::configstring::{
    CS_AMBIENT_SET, CS_BSP_MODELS, CS_EFFECTS, CS_GAME_VERSION, CS_GLOBAL_AMBIENT_SET, CS_ICONS,
    CS_ITEMS, CS_LEVEL_START_TIME, CS_MODELS, CS_MUSIC, CS_PLAYERS, CS_SIEGE_OBJECTIVES,
    CS_SIEGE_STATE, CS_SIEGE_TIMEOVERRIDE, CS_SIEGE_WINTEAM, CS_SOUNDS, CS_TERRAINS,
};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::g_item::MAX_ITEM_MODELS;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use mp_bg::public::item_kind::ItemKind;
use mp_bg::public::item_type::IT_HOLDABLE;
use mp_bg::public::max_items::MAX_ITEMS;
use mp_bg::public::pers_enum::persEnum_t::PERS_ATTACKER;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_bg::public::spawn::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS};
use mp_bg::public::stat_index::statIndex_t::{STAT_CLIENTS_READY, STAT_HOLDABLE_ITEM};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::weapons::weapon_t::{WP_BRYAR_PISTOL, WP_NONE};
use mp_engine_select::Engine;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::common::mp::qcommon::usercmd_button::BUTTON_USE;
use mp_qshared::common::mp::qcommon::{usercmd_t, PMF_FOLLOW};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::cvar::{
    CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INTERNAL, CVAR_ROM, CVAR_SERVERINFO, CVAR_USERINFO,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::force_powers::{
    FP_HEAL, FP_LEVITATION, FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, NUM_FORCE_POWERS,
};
use mp_qshared::shared::limits::{
    ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS, MAX_CLIENTS_I32, MAX_FX, MAX_ICONS, MAX_MODELS,
    MAX_SOUNDS, MAX_STRING_CHARS, MAX_SUB_BSP, MAX_TOKEN_CHARS,
};
use mp_qshared::shared::q_color::S_COLOR_RED;
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorMA, _VectorSubtract, vec3_origin, AnglesToAxis, Distance, VectorLength,
    PITCH, ROLL, YAW,
};
use mp_qshared::shared::q_string::COM_Parse;
use mp_qshared::shared::sound_channel::CHAN_AUTO;
use mp_qshared::shared::{
    fileHandle_t, pc_token_t, qfalse, qhandle_t, qtrue, vec3_t, vec4_t, CIN_LOOP, FS_READ,
    MASK_PLAYERSOLID, MAX_CONFIGSTRINGS, MAX_GENTITIES, MAX_QPATH, MAX_TOKENLENGTH,
};
use mp_uishared::shared::cached_assets_t::NUM_CROSSHAIRS;
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::item_id::ItemId;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::{MenuSystem, MAX_MENUDEFFILE, MAX_MENUFILE};
use mp_uishared::shared::menudef::{
    CG_BLUE_NAME, CG_GAME_STATUS, CG_GAME_TYPE, CG_KILLER, CG_RED_NAME, FEEDER_BLUETEAM_LIST,
    FEEDER_REDTEAM_LIST, FEEDER_SCOREBOARD,
};
use mp_uishared::ui_shared::{
    Menu_New, Menu_Reset, Menu_SetFeederSelection, PC_Color_Parse, PC_Float_Parse, PC_Int_Parse,
    PC_String_Parse, String_Init, UI_CleanupGhoul2,
};
use native_string::{
    atof, atoi, buf_to_string, latin1_to_string, sscanf_f32s, string_to_latin1, strncpyz_string,
    Q_strcat, Q_stricmp,
};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_consolecmds::{CG_ConsoleCommand, CG_InitConsoleCommands};
use crate::cg_draw::{CG_Text_Paint, CG_Text_Width};
use crate::cg_effects::{CG_InitGlass, CG_TestLine};
use crate::cg_ents::{CG_CalcEntityLerpPositions, CG_ROFF_NotetrackCallback, ScaleModelAxis};
use crate::cg_info::{CG_LoadingClient, CG_LoadingItem, CG_LoadingString};
use crate::cg_light::CG_ClearLightStyles;
use crate::cg_localents::CG_InitLocalEntities;
use crate::cg_marks::{CG_ClearParticles, CG_ImpactMark, CG_InitMarkPolys};
use crate::cg_new_draw::{
    CG_EventHandling, CG_GameTypeString, CG_GetGameStatusText, CG_GetKillerText, CG_KeyEvent,
    CG_MouseEvent, CG_StatusHandle,
};
use crate::cg_players::{
    CG_AddGhoul2Mark, CG_CacheG2AnimInfo, CG_CleanJetpackGhoul2, CG_HandleAppendedSkin,
    CG_InitJetpackGhoul2, CG_NewClientInfo,
};
use crate::cg_predict::{CG_G2Trace, CG_PmoveClientPointerUpdate, CG_PointContents, CG_Trace};
use crate::cg_saga::{CG_InitSiegeMode, CG_ParseSiegeObjectiveStatus, CG_SetSiegeTimerCvar};
use crate::cg_scoreboard::{CG_GetClassCount, CG_GetTeamNonScoreCount};
use crate::cg_servercmds::{
    CG_KillCEntityG2, CG_ParseServerinfo, CG_PrecacheNPCSounds, CG_SetConfigValues,
    CG_ShaderStateChanged,
};
use crate::cg_view::{CG_DoCameraShake, CG_DrawActiveFrame};
use crate::cg_weapons::{
    CG_InitG2Weapons, CG_RegisterItemVisuals, CG_ShutDownG2Weapons, LAST_USEABLE_WEAPON,
};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::clientInfo_t;
use crate::local::footstep_t::footstep_t;
use crate::local::item_info_t::itemInfo_t;
use crate::local::trail_fn::TrailFn;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_main_state::CgMiscEnt;
use crate::world::{CgContext, CgState, CgWorld};

/// Raven `#define MAX_MISC_ENTS` — cgame's client-only "extra visual" registry
/// capacity. [`CG_MiscEnt`] (the `CG_MISC_ENT` vmcall) and
/// [`CG_CreateModelFromSpawnEnt`] are the two registrars that check it.
///
/// The oracle spells it twice under an `#ifdef _XBOX` — 500 on Xbox, 4000 on
/// the PC build this port ships. `CG_DrawMiscEnts`'s backing state is a growable
/// `Vec` rather than a fixed-size array (see
/// [`crate::world::cg_main_state::CgMiscEnt`]), so the cap only lives in those
/// two checks.
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
/// string-intern pool. The pool's own buffer is gone (see
/// [`CG_StrPool_Alloc`]); this is the budget the bump counter is checked
/// against.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3309`
pub const MAX_CGSTRPOOL_SIZE: usize = 32768;

/// Raven `#define DEFAULT_REDTEAM_NAME "Empire"` — the red team's display
/// name. `CG_OwnerDrawWidth`/`CG_NewClientInfo` use it in place of the
/// `cg_redTeamName` cvar, whose `cvarTable` row Raven commented out.
///
/// Source: `oracle/codemp/cgame/cg_local.h:87`
pub const DEFAULT_REDTEAM_NAME: &str = "Empire";

/// Raven `#define DEFAULT_BLUETEAM_NAME "Rebellion"` — [`DEFAULT_REDTEAM_NAME`]'s
/// twin.
///
/// Source: `oracle/codemp/cgame/cg_local.h:88`
pub const DEFAULT_BLUETEAM_NAME: &str = "Rebellion";

/// Raven `#define GAME_VERSION "basejka-1"` — `CG_Init`'s client/server build
/// check against the `CS_GAME_VERSION` configstring.
///
/// Source: `oracle/codemp/game/bg_public.h:20`
pub const GAME_VERSION: &str = "basejka-1";

/// Raven `char *HolocronIcons[]` (`oracle/codemp/cgame/holocronicons.h`,
/// `#include`d by `cg_main.c:34`) — indexed by `forcePowers_t`. cgame's own
/// copy of the same table `ui_main.rs` keeps (`holocronicons.h` compiles into
/// both hosts).
///
/// Source: `oracle/codemp/cgame/holocronicons.h:4-22`
const HOLOCRON_ICONS: [&str; NUM_FORCE_POWERS as usize] = [
    "gfx/mp/f_icon_lt_heal",       // FP_HEAL
    "gfx/mp/f_icon_levitation",    // FP_LEVITATION
    "gfx/mp/f_icon_speed",         // FP_SPEED
    "gfx/mp/f_icon_push",          // FP_PUSH
    "gfx/mp/f_icon_pull",          // FP_PULL
    "gfx/mp/f_icon_lt_telepathy",  // FP_TELEPATHY
    "gfx/mp/f_icon_dk_grip",       // FP_GRIP
    "gfx/mp/f_icon_dk_l1",         // FP_LIGHTNING
    "gfx/mp/f_icon_dk_rage",       // FP_RAGE
    "gfx/mp/f_icon_lt_protect",    // FP_PROTECT
    "gfx/mp/f_icon_lt_absorb",     // FP_ABSORB
    "gfx/mp/f_icon_lt_healother",  // FP_TEAM_HEAL
    "gfx/mp/f_icon_dk_forceother", // FP_TEAM_FORCE
    "gfx/mp/f_icon_dk_drain",      // FP_DRAIN
    "gfx/mp/f_icon_sight",         // FP_SEE
    "gfx/mp/f_icon_saber_attack",  // FP_SABER_OFFENSE
    "gfx/mp/f_icon_saber_defend",  // FP_SABER_DEFENSE
    "gfx/mp/f_icon_saber_throw",   // FP_SABERTHROW
];

// PORT-NOTE: `q_shared.h`'s font enum is anonymous, so per the anonymous-enum
// convention this is a `const`; file-local, the same copy `cg_draw.rs` and
// `mp_uishared::ui_shared` each keep.
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;

/// Raven `#define MAX_AMBIENT_SETS 256` — ambient soundsets are handed over in
/// configstrings, so this is how many `CS_AMBIENT_SET` rows exist. No canonical
/// `mp_qshared` home ported yet (`mp_game` keeps its own copy in `g_spawn.rs`,
/// a crate cgame does not depend on), so it lands here beside its one reader.
///
/// Source: `oracle/codemp/game/q_shared.h:2035`
const MAX_AMBIENT_SETS: c_int = 256;

/// Raven `#define MAX_TERRAINS 1` — rwwRMG's terrain-configstring budget. Raven
/// left the `32` it was cut down from in a trailing comment; the shipped value
/// is 1, so `CG_RegisterGraphics`'s terrain loop runs at most zero times.
///
/// Source: `oracle/codemp/game/q_shared.h:1988`
const MAX_TERRAINS: c_int = 1;

// PORT-NOTE: `cg_local.h`'s chunk-type enum is anonymous, so per the
// anonymous-enum convention these are `const`s. They index the first axis of
// `cgs.media.chunkModels`.
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_METAL1: usize = 0;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_METAL2: usize = 1;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK1: usize = 2;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK2: usize = 3;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK3: usize = 4;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_CRATE1: usize = 5;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_CRATE2: usize = 6;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_WHITE_METAL: usize = 7;

/// Raven `#define NUM_CHUNK_MODELS 4` — how many `.md3` variants each chunk
/// type registers.
///
/// Source: `oracle/codemp/cgame/cg_local.h:1061`
const NUM_CHUNK_MODELS: usize = 4;

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
        .unwrap_or_else(|| format!("??{key}"))
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
pub fn CG_Asset_Parse(ctx: &mut CgContext, ds: &mut DisplayState, handle: c_int) -> bool {
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
                || !PC_Int_Parse(ctx, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhMediumFont = ctx.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // smallFont
        if Q_stricmp(&tokenStr, "smallFont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(ctx, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhSmallFont = ctx.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // smallFont
        if Q_stricmp(&tokenStr, "small2Font") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(ctx, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhSmall2Font = ctx.RegisterFont(&pc_token_str(&token));
            continue;
        }

        // font
        if Q_stricmp(&tokenStr, "bigfont") == 0 {
            let mut pointSize = 0;
            if !trap::PC_ReadToken(ctx.engine, handle, &mut token)
                || !PC_Int_Parse(ctx, handle, &mut pointSize)
            {
                return false;
            }
            ds.Assets.qhBigFont = ctx.RegisterFont(&pc_token_str(&token));
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
            if !PC_String_Parse(ctx, handle, &mut ds.Assets.cursorStr) {
                return false;
            }
            let cursorStr = ds.Assets.cursorStr.clone();
            ds.Assets.cursor = trap::R_RegisterShaderNoMip(ctx.engine, &cursorStr);
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeClamp") == 0 {
            if !PC_Float_Parse(ctx, handle, &mut ds.Assets.fadeClamp) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeCycle") == 0 {
            if !PC_Int_Parse(ctx, handle, &mut ds.Assets.fadeCycle) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "fadeAmount") == 0 {
            if !PC_Float_Parse(ctx, handle, &mut ds.Assets.fadeAmount) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowX") == 0 {
            if !PC_Float_Parse(ctx, handle, &mut ds.Assets.shadowX) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowY") == 0 {
            if !PC_Float_Parse(ctx, handle, &mut ds.Assets.shadowY) {
                return false;
            }
            continue;
        }

        if Q_stricmp(&tokenStr, "shadowColor") == 0 {
            if !PC_Color_Parse(ctx, handle, &mut ds.Assets.shadowColor) {
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
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    p: Option<MenuId>,
) {
    // §F19: on the null-snap path Raven reads through a null `ps`. -1 matches no
    // `scores[i].client`, so `cg.selectedScore` keeps its previous value and the
    // rest of the fn runs exactly as Raven's.
    let clientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    let mut red = 0;
    let mut blue = 0;
    for i in 0..ctx.world.cg.numScores as usize {
        if ctx.world.cg.scores[i].team == TEAM_RED {
            red += 1;
        } else if ctx.world.cg.scores[i].team == TEAM_BLUE {
            blue += 1;
        }
        if clientNum == ctx.world.cg.scores[i].client {
            ctx.world.cg.selectedScore = i as c_int;
        }
    }

    if p.is_none() {
        // just interested in setting the selected score
        return;
    }

    if ctx.world.cgs.gametype >= GT_TEAM {
        let mut feeder = FEEDER_REDTEAM_LIST;
        let mut i = red;
        if ctx.world.cg.scores[ctx.world.cg.selectedScore as usize].team == TEAM_BLUE {
            feeder = FEEDER_BLUETEAM_LIST;
            i = blue;
        }
        Menu_SetFeederSelection(menus, ds, ctx, p, feeder, i, None);
    } else {
        let selectedScore = ctx.world.cg.selectedScore;
        Menu_SetFeederSelection(menus, ds, ctx, p, FEEDER_SCOREBOARD, selectedScore, None);
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
/// The `_XBOX` `widescreen` save/restore around the memset is not in the MP PC
/// build.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3254-3263`
pub fn CG_Init_CG(world: &mut CgWorld) {
    // Raven: memset( &cg, 0, sizeof(cg));
    world.cg.zero_in_place();
}

/// Raven `CG_Init_CGents` — wipes the entity array between map loads.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3274-3278`
pub fn CG_Init_CGents(world: &mut CgWorld) {
    // Raven: memset(&cg_entities, 0, sizeof(cg_entities));
    for cent in world.entities.iter_mut() {
        *cent = centity_t::zeroed();
    }
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
        .unwrap_or_else(|| format!("??{}", &loc[1..]))
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

/// Raven `C_PointContents` — the `CG_POINT_CONTENTS` vmcall body.
///
/// Same DEC-46.6 shape as [`C_GetLerpOrigin`]: Raven casts `cg.sharedBuffer` to
/// `TCGPointContents *` right here, the port takes the already-decoded payload
/// from the vmMain dispatch boundary.
///
/// Source: `oracle/codemp/cgame/cg_main.c:362-367`
pub fn C_PointContents(ctx: &mut CgContext, data: &TCGPointContents) -> c_int {
    CG_PointContents(ctx, &data.mPoint, data.mPassEntityNum)
}

/// Raven `Com_Error` — the qcommon-shaped fatal-error entry point cgame
/// supplies for the code it shares with the engine. `level` is dropped, exactly
/// as [`CG_Error`] drops it.
///
/// Raven's `va_list`/`vsprintf` staging into `char text[1024]` collapses to the
/// caller's already-formatted string (dictionary: `va()`/`Com_sprintf` →
/// `format!`).
///
/// Source: `oracle/codemp/cgame/cg_main.c:1234-1243`
pub fn Com_Error(ctx: &mut CgContext, _level: c_int, error: &str) {
    CG_Error(ctx, error);
}

/// Raven `Com_Printf` — the qcommon-shaped console print cgame supplies; same
/// `vsprintf`-collapses-to-`format!` shape as [`Com_Error`].
///
/// Source: `oracle/codemp/cgame/cg_main.c:1245-1254`
pub fn Com_Printf(ctx: &mut CgContext, msg: &str) {
    CG_Printf(ctx, msg);
}

/// Raven `CG_RegisterItemSounds` — registers item `itemNum`'s pickup sound plus
/// everything named in its two space-separated precache strings.
///
/// The `sounds` string registers every token as a sound; the `precaches` string
/// registers only the `.efx` tokens (the rest are models/shaders somebody else
/// precaches).
///
/// Source: `oracle/codemp/cgame/cg_main.c:1289-1354`
pub fn CG_RegisterItemSounds(ctx: &mut CgContext, itemNum: c_int) {
    let item = &bg_itemlist[itemNum as usize];

    if let Some(pickup_sound) = item.pickup_sound {
        trap::S_RegisterSound(ctx.engine, pickup_sound);
    }

    // parse the space seperated precache string for other media
    let s = item.sounds;
    if s.is_empty() {
        return;
    }

    // Raven walks a `char *` cursor rather than splitting, and the cursor walk
    // is load-bearing: a doubled space yields a zero-length token, which is the
    // `len < 5` error below. A trailing space does NOT (the `while (*s)` test
    // sees the terminator first).
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }

        let len = i - start;
        if len >= MAX_QPATH || len < 5 {
            let msg = format!("PrecacheItem: {} has bad precache string", item.classname);
            CG_Error(ctx, &msg);
            return;
        }
        let data = &s[start..i];
        if i < bytes.len() {
            i += 1;
        }

        trap::S_RegisterSound(ctx.engine, data);
    }

    // parse the space seperated precache string for other media
    let s = item.precaches;
    if s.is_empty() {
        return;
    }

    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }

        let len = i - start;
        if len >= MAX_QPATH || len < 5 {
            let msg = format!("PrecacheItem: {} has bad precache string", item.classname);
            CG_Error(ctx, &msg);
            return;
        }
        let data = &s[start..i];
        if i < bytes.len() {
            i += 1;
        }

        if data.ends_with("efx") {
            trap::FX_RegisterEffect(ctx.engine, data);
        }
    }
}

/// Raven `CG_RegisterEffects` — the glass mini-system plus the footstep,
/// landing and splash material effects.
///
/// Raven's `CS_EFFECTS` configstring loop above these is commented out in the
/// oracle: "the above was redundant as it's being done in CG_RegisterSounds".
///
/// Source: `oracle/codemp/cgame/cg_main.c:1867-1905`
pub fn CG_RegisterEffects(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // Set up the glass effects mini-system.
    CG_InitGlass(ctx.world);

    //footstep effects
    ctx.world.cgs.effects.footstepMud = trap::FX_RegisterEffect(engine, "materials/mud");
    ctx.world.cgs.effects.footstepSand = trap::FX_RegisterEffect(engine, "materials/sand");
    ctx.world.cgs.effects.footstepSnow = trap::FX_RegisterEffect(engine, "materials/snow");
    ctx.world.cgs.effects.footstepGravel = trap::FX_RegisterEffect(engine, "materials/gravel");
    //landing effects
    ctx.world.cgs.effects.landingMud = trap::FX_RegisterEffect(engine, "materials/mud_large");
    ctx.world.cgs.effects.landingSand = trap::FX_RegisterEffect(engine, "materials/sand_large");
    ctx.world.cgs.effects.landingDirt = trap::FX_RegisterEffect(engine, "materials/dirt_large");
    ctx.world.cgs.effects.landingSnow = trap::FX_RegisterEffect(engine, "materials/snow_large");
    ctx.world.cgs.effects.landingGravel = trap::FX_RegisterEffect(engine, "materials/gravel_large");
    //splashes
    ctx.world.cgs.effects.waterSplash = trap::FX_RegisterEffect(engine, "env/water_impact");
    ctx.world.cgs.effects.lavaSplash = trap::FX_RegisterEffect(engine, "env/lava_splash");
    ctx.world.cgs.effects.acidSplash = trap::FX_RegisterEffect(engine, "env/acid_splash");
}

/// Raven `CG_SiegeCountCvars` — publishes the siege team/class head counts into
/// the `ui_tm*` cvars the menus read.
///
/// Raven's own note on the shader handles: "This is because the only way we can
/// match up classes is by the gfx handle."
///
/// Source: `oracle/codemp/cgame/cg_main.c:2442-2472`
pub fn CG_SiegeCountCvars(ctx: &mut CgContext) {
    let engine = ctx.engine;
    let mut classGfx: [qhandle_t; 6] = [0; 6];

    let tm1 = CG_GetTeamNonScoreCount(ctx.world, TEAM_RED);
    trap::Cvar_Set(engine, "ui_tm1_cnt", &format!("{tm1}"));
    let tm2 = CG_GetTeamNonScoreCount(ctx.world, TEAM_BLUE);
    trap::Cvar_Set(engine, "ui_tm2_cnt", &format!("{tm2}"));
    let tm3 = CG_GetTeamNonScoreCount(ctx.world, TEAM_SPECTATOR);
    trap::Cvar_Set(engine, "ui_tm3_cnt", &format!("{tm3}"));

    // This is because the only way we can match up classes is by the gfx handle.
    classGfx[0] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_infantry");
    classGfx[1] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_heavy_weapons");
    classGfx[2] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_demolitionist");
    classGfx[3] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_vanguard");
    classGfx[4] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_support");
    classGfx[5] = trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_jedi_general");

    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[0]);
    trap::Cvar_Set(engine, "ui_tm1_c0_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[1]);
    trap::Cvar_Set(engine, "ui_tm1_c1_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[2]);
    trap::Cvar_Set(engine, "ui_tm1_c2_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[3]);
    trap::Cvar_Set(engine, "ui_tm1_c3_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[4]);
    trap::Cvar_Set(engine, "ui_tm1_c4_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_RED, classGfx[5]);
    trap::Cvar_Set(engine, "ui_tm1_c5_cnt", &format!("{c}"));

    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[0]);
    trap::Cvar_Set(engine, "ui_tm2_c0_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[1]);
    trap::Cvar_Set(engine, "ui_tm2_c1_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[2]);
    trap::Cvar_Set(engine, "ui_tm2_c2_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[3]);
    trap::Cvar_Set(engine, "ui_tm2_c3_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[4]);
    trap::Cvar_Set(engine, "ui_tm2_c4_cnt", &format!("{c}"));
    let c = CG_GetClassCount(ctx.world, TEAM_BLUE, classGfx[5]);
    trap::Cvar_Set(engine, "ui_tm2_c5_cnt", &format!("{c}"));
}

/// Raven `CG_ConfigString` — the configstring at `index`, read out of the
/// gamestate blob the engine handed us.
///
/// Raven returned a `char *` into `cgs.gameState.stringData`; the port decodes
/// an owned `String` at the read, so nothing aliases a buffer the next
/// `CG_SetConfigValues` can move.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2535-2540`
pub fn CG_ConfigString(ctx: &mut CgContext, index: c_int) -> String {
    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        let msg = format!("CG_ConfigString: bad index: {index}");
        CG_Error(ctx, &msg);
        // Raven's `CG_Error` never comes back; the port's trap wrapper does, so
        // the bad-index path answers the empty configstring.
        return String::new();
    }

    let offset = ctx.world.cgs.gameState.stringOffsets[index as usize] as usize;
    let tail = &ctx.world.cgs.gameState.stringData[offset..];
    let end = tail.iter().position(|&c| c == 0).unwrap_or(tail.len());
    latin1_to_string(&tail[..end].iter().map(|&c| c as u8).collect::<Vec<u8>>())
}

/// Raven `CG_ParseMenu` — loads one menu script and hands each `menudef` block
/// to the shared framework.
///
/// A menu file that won't open falls back to `ui/testhud.menu`; if that won't
/// open either there is nothing to parse.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2753-2797`
pub fn CG_ParseMenu(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &mut DisplayState,
    menuFile: &str,
) {
    let mut token = zero_pc_token();

    let mut handle = trap::PC_LoadSource(ctx.engine, menuFile);
    if handle == 0 {
        handle = trap::PC_LoadSource(ctx.engine, "ui/testhud.menu");
    }
    if handle == 0 {
        return;
    }

    loop {
        if !trap::PC_ReadToken(ctx.engine, handle, &mut token) {
            break;
        }

        // Raven's "Missing {" and "Too many menus!" guards are both commented
        // out in the oracle.

        if token.string[0] == b'}' as c_char {
            break;
        }

        let tokenStr = pc_token_str(&token);

        if Q_stricmp(&tokenStr, "assetGlobalDef") == 0 {
            if CG_Asset_Parse(ctx, ds, handle) {
                continue;
            } else {
                break;
            }
        }

        if Q_stricmp(&tokenStr, "menudef") == 0 {
            // start a new menu
            Menu_New(menus, ds, ctx, handle);
        }
    }
    trap::PC_FreeSource(ctx.engine, handle);
}

/// Raven `CG_FeederItemText` — one scoreboard-feeder cell: the text for
/// `column` of row `index`, plus up to three row shader handles.
///
/// PORT-NOTE: same `float feederID` handling as [`CG_FeederCount`] — Raven
/// compares the float against the int `FEEDER_*` defines, the port casts once
/// and compares ints.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2909-2994`
pub fn CG_FeederItemText(
    world: &CgWorld,
    feederID: f32,
    index: c_int,
    column: c_int,
    handle1: &mut qhandle_t,
    handle2: &mut qhandle_t,
    handle3: &mut qhandle_t,
) -> String {
    let feeder = feederID as c_int;
    let mut team = -1;

    *handle1 = -1;
    *handle2 = -1;
    *handle3 = -1;

    if feeder == FEEDER_REDTEAM_LIST {
        team = TEAM_RED;
    } else if feeder == FEEDER_BLUETEAM_LIST {
        team = TEAM_BLUE;
    }

    // Raven's `info &&` can't fail - `CG_InfoFromScoreIndex` always answers with
    // a `cgs.clientinfo` row.
    let (info, scoreIndex) = CG_InfoFromScoreIndex(world, index, team);
    let sp = &world.cg.scores[scoreIndex as usize];

    if info.infoValid != qfalse {
        match column {
            0 => {
                // §F19: each `BG_FindItemForPowerup` miss is a null deref in
                // Raven; the port leaves `handle1` at -1 (the "no row icon"
                // value it was just set to).
                if info.powerups & (1 << PW_NEUTRALFLAG) != 0 {
                    if let Some(item) = BG_FindItemForPowerup(PW_NEUTRALFLAG) {
                        *handle1 = world.cg_items[item.modelindex() as usize].icon;
                    }
                } else if info.powerups & (1 << PW_REDFLAG) != 0 {
                    if let Some(item) = BG_FindItemForPowerup(PW_REDFLAG) {
                        *handle1 = world.cg_items[item.modelindex() as usize].icon;
                    }
                } else if info.powerups & (1 << PW_BLUEFLAG) != 0 {
                    if let Some(item) = BG_FindItemForPowerup(PW_BLUEFLAG) {
                        *handle1 = world.cg_items[item.modelindex() as usize].icon;
                    }
                } else {
                    // Raven's bot-skill / handicap fallbacks are commented out.
                }
            }

            1 => {
                if team == -1 {
                    return String::new();
                }
                *handle1 = CG_StatusHandle(world, info.teamTask);
            }

            2 => {
                // §F19: Raven reads through `cg.snap` unguarded. With no
                // snapshot nobody is flagged ready, which is this row's
                // not-ready answer.
                let ready = world.cg.snap_ref().is_some_and(|snap| {
                    snap.ps.stats[STAT_CLIENTS_READY as usize] & (1 << sp.client) != 0
                });
                if ready {
                    return "Ready".to_string();
                }
                if team == -1 {
                    if world.cgs.gametype == GT_DUEL || world.cgs.gametype == GT_POWERDUEL {
                        return format!("{}/{}", info.wins, info.losses);
                    } else if info.infoValid != qfalse && info.team == TEAM_SPECTATOR {
                        return "Spectator".to_string();
                    } else {
                        return String::new();
                    }
                } else if info.teamLeader != qfalse {
                    return "Leader".to_string();
                }
            }

            3 => {
                return buf_to_string(&info.name.iter().map(|&c| c as u8).collect::<Vec<u8>>());
            }

            4 => {
                return format!("{}", info.score);
            }

            5 => {
                return format!("{:4}", sp.time);
            }

            6 => {
                if sp.ping == -1 {
                    return "connecting".to_string();
                }
                return format!("{:4}", sp.ping);
            }

            _ => {}
        }
    }

    String::new()
}

/// Raven `CG_CreateWeatherZoneFromSpawnEnt` — a `misc_weather_zone` brush hands
/// the renderer's weather system its bounds.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3445-3449`
pub fn CG_CreateWeatherZoneFromSpawnEnt(ctx: &mut CgContext, ent: &mut CgSpawnEnt) {
    CG_CreateBrushEntData(ctx, ent);
    trap::WE_AddWeatherZone(ctx.engine, &ent.mins, &ent.maxs);
}

/// Raven `CG_AddSpawnVarToken` — takes one parsed spawn-var token off the
/// map's `MAX_SPAWN_VARS_CHARS` budget.
///
/// Raven bump-copied the token into `cg_spawnVarChars[]` and returned a pointer
/// into it; the port returns the owned copy and only keeps the budget counter,
/// because the budget overrun is an observable `CG_Error` (the same fold
/// `G_AddSpawnVarToken` got — `crates/mp/game/src/g_spawn.rs:828-831`).
///
/// Source: `oracle/codemp/cgame/cg_main.c:3515-3531`
pub fn CG_AddSpawnVarToken(ctx: &mut CgContext, string: &str) -> String {
    let l = string.len() as c_int;
    if ctx.world.main.cg_numSpawnVarChars + l + 1 > MAX_SPAWN_VARS_CHARS as c_int {
        // Raven's `CG_Error` never comes back; the port's trap wrapper does, so
        // the over-budget token is still handed back below.
        CG_Error(ctx, "CG_AddSpawnVarToken: MAX_SPAWN_VARS");
    }

    ctx.world.main.cg_numSpawnVarChars += l + 1;

    string.to_string()
}

/// Raven `CG_NextForcePower_f` — the `+forcenext` command: cycle the selected
/// force power forward, unless USE is held (then it's the inventory that
/// cycles).
///
/// Source: `oracle/codemp/cgame/cg_main.c:4023-4063`
pub fn CG_NextForcePower_f(ctx: &mut CgContext) {
    if ctx.world.cg.snap.is_null() {
        return;
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        return;
    }

    let current = trap::GetCurrentCmdNumber(ctx.engine);
    let mut cmd = usercmd_t::default();
    trap::GetUserCmd(ctx.engine, current, &mut cmd);
    if cmd.buttons & BUTTON_USE != 0 || CG_NoUseableForce(ctx.world) {
        CG_NextInventory_f(ctx.world);
        return;
    }

    // §F19: `cg.snap` is non-null here, so every `None` arm below is Raven's
    // null deref and can only answer by doing nothing.
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    if snap.ps.pm_flags & PMF_FOLLOW != 0 {
        return;
    }

    // Raven's first `BG_CycleForce(&cg.snap->ps, 1)` here is commented out.
    if ctx.world.cg.forceSelect != -1 {
        let forceSelect = ctx.world.cg.forceSelect;
        let Some(snap) = ctx.world.cg.snap_mut() else {
            return;
        };
        snap.ps.fd.forcePowerSelected = forceSelect;
    }

    let Some(snap) = ctx.world.cg.snap_mut() else {
        return;
    };
    BG_CycleForce(&mut snap.ps, 1);

    let selected = match ctx.world.cg.snap_ref() {
        Some(snap) if snap.ps.fd.forcePowersKnown & (1 << snap.ps.fd.forcePowerSelected) != 0 => {
            Some(snap.ps.fd.forcePowerSelected)
        }
        _ => None,
    };
    if let Some(selected) = selected {
        ctx.world.cg.forceSelect = selected;
        ctx.world.cg.forceSelectTime = ctx.world.cg.time as f32;
    }
}

/// Raven `CG_PrevForcePower_f` — [`CG_NextForcePower_f`] backwards; Raven
/// duplicated the body verbatim except for the cycle direction and the
/// inventory command it falls back to.
///
/// Source: `oracle/codemp/cgame/cg_main.c:4070-4110`
pub fn CG_PrevForcePower_f(ctx: &mut CgContext) {
    if ctx.world.cg.snap.is_null() {
        return;
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        return;
    }

    let current = trap::GetCurrentCmdNumber(ctx.engine);
    let mut cmd = usercmd_t::default();
    trap::GetUserCmd(ctx.engine, current, &mut cmd);
    if cmd.buttons & BUTTON_USE != 0 || CG_NoUseableForce(ctx.world) {
        CG_PrevInventory_f(ctx.world);
        return;
    }

    // §F19: same non-null `cg.snap` as [`CG_NextForcePower_f`].
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    if snap.ps.pm_flags & PMF_FOLLOW != 0 {
        return;
    }

    // Raven's first `BG_CycleForce(&cg.snap->ps, -1)` here is commented out.
    if ctx.world.cg.forceSelect != -1 {
        let forceSelect = ctx.world.cg.forceSelect;
        let Some(snap) = ctx.world.cg.snap_mut() else {
            return;
        };
        snap.ps.fd.forcePowerSelected = forceSelect;
    }

    let Some(snap) = ctx.world.cg.snap_mut() else {
        return;
    };
    BG_CycleForce(&mut snap.ps, -1);

    let selected = match ctx.world.cg.snap_ref() {
        Some(snap) if snap.ps.fd.forcePowersKnown & (1 << snap.ps.fd.forcePowerSelected) != 0 => {
            Some(snap.ps.fd.forcePowerSelected)
        }
        _ => None,
    };
    if let Some(selected) = selected {
        ctx.world.cg.forceSelect = selected;
        ctx.world.cg.forceSelectTime = ctx.world.cg.time as f32;
    }
}

/// The zero fill Raven gets from `memset(&cg.refdef, 0, sizeof(cg.refdef))`.
/// `refdef_t` is a `#[repr(C)]` seam type with no `Default`, so the fill is
/// spelled out once.
fn zeroed_refdef() -> refdef_t {
    refdef_t {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        fov_x: 0.0,
        fov_y: 0.0,
        vieworg: [0.0; 3],
        viewangles: [0.0; 3],
        viewaxis: [[0.0; 3]; 3],
        viewContents: 0,
        time: 0,
        rdflags: 0,
        areamask: [0; MAX_MAP_AREA_BYTES],
        text: [[0; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
    }
}

/// The zero fill Raven gets from `memset( cg_weapons, 0, sizeof( cg_weapons ) )`.
/// `weaponInfo_t` is plain handles plus two trail-fn slots and has no `Default`,
/// so the fill is spelled out once.
fn zeroed_weapon_info() -> weaponInfo_t {
    weaponInfo_t {
        registered: qfalse,
        item: None,

        handsModel: 0,
        weaponModel: 0,
        viewModel: 0,
        barrelModel: 0,
        flashModel: 0,

        weaponMidpoint: [0.0; 3],

        flashDlight: 0.0,
        flashDlightColor: [0.0; 3],

        weaponIcon: 0,
        ammoIcon: 0,

        ammoModel: 0,

        flashSound: [0; 4],
        firingSound: 0,
        chargeSound: 0,
        muzzleEffect: 0,
        missileModel: 0,
        missileSound: 0,
        missileTrailFunc: TrailFn::None,
        missileDlight: 0.0,
        missileDlightColor: [0.0; 3],
        missileRenderfx: 0,
        missileHitSound: 0,

        altFlashSound: [0; 4],
        altFiringSound: 0,
        altChargeSound: 0,
        altMuzzleEffect: 0,
        altMissileModel: 0,
        altMissileSound: 0,
        altMissileTrailFunc: TrailFn::None,
        altMissileDlight: 0.0,
        altMissileDlightColor: [0.0; 3],
        altMissileRenderfx: 0,
        altMissileHitSound: 0,

        selectSound: 0,

        readySound: 0,
        trailRadius: 0.0,
        wiTrailTime: 0.0,
    }
}

/// Raven `CG_MiscEnt` — the `CG_MISC_ENT` vmcall body: registers one
/// client-only decorative model and its cull radius into the misc-ent registry
/// [`CG_DrawMiscEnts`] walks.
///
/// Same DEC-46.6 shape as [`C_GetLerpOrigin`]: Raven casts `cg.sharedBuffer` to
/// `TCGMiscEnt *` right here, the port takes the already-decoded payload from
/// the vmMain dispatch boundary.
///
/// Source: `oracle/codemp/cgame/cg_main.c:582-622`
pub fn CG_MiscEnt(ctx: &mut CgContext, data: &TCGMiscEnt) {
    if ctx.world.main.miscEnts.len() >= MAX_MISC_ENTS {
        return;
    }

    // `RefEnt = &MiscEnts[NumMiscEnts++]` — the slot is claimed before the model
    // registers, so the error path below leaves it claimed, exactly as Raven.
    ctx.world.main.miscEnts.push(CgMiscEnt {
        ent: refEntity_t::zeroed(),
        radius: 0.0,
        zOffset: 0.0,
    });
    let slot = ctx.world.main.miscEnts.len() - 1;

    let mModel = buf_to_string(&data.mModel.map(|c| c as u8));
    let modelIndex = trap::R_RegisterModel(ctx.engine, &mModel);
    if modelIndex == 0 {
        Com_Error(
            ctx,
            errorParm_t::ERR_DROP as c_int,
            "client_model has invalid model definition",
        );
        return;
    }

    ctx.world.main.miscEnts[slot].zOffset = 0.0;

    // `memset(RefEnt, 0, sizeof(refEntity_t))` and the fill that follows it; the
    // finished record goes back into its slot at the bottom.
    let mut RefEnt = refEntity_t::zeroed();
    RefEnt.reType = refEntityType_t::RT_MODEL;
    RefEnt.hModel = modelIndex;
    RefEnt.frame = 0;

    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    trap::R_ModelBounds(ctx.engine, modelIndex, &mut mins, &mut maxs);
    _VectorCopy(data.mScale, &mut RefEnt.modelScale);
    _VectorCopy(data.mOrigin, &mut RefEnt.origin);

    // Raven `VectorScaleVector(mins, data->mScale, mins)` - per-axis scale,
    // no ported home for the macro yet.
    mins[0] *= data.mScale[0];
    mins[1] *= data.mScale[1];
    mins[2] *= data.mScale[2];
    maxs[0] *= data.mScale[0];
    maxs[1] *= data.mScale[1];
    maxs[2] *= data.mScale[2];
    ctx.world.main.miscEnts[slot].radius = Distance(mins, maxs);

    AnglesToAxis(data.mAngles, RefEnt.axis.as_mut_ptr());
    ScaleModelAxis(&mut RefEnt);

    ctx.world.main.miscEnts[slot].ent = RefEnt;
}

/// Raven `CG_AS_Register` — hands the ambient-sound system every soundset the
/// server listed in configstrings, then parses them.
///
/// Raven's `#if 0` arm (the game-side `as_preCacheMap` walk, "that is evil")
/// stays out.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1356-1392`
pub fn CG_AS_Register(ctx: &mut CgContext) {
    // CG_LoadingString( "ambient sound sets" );

    // Load the ambient sets
    trap::AS_AddPrecacheEntry(ctx.engine, "#clear");

    for i in 1..MAX_AMBIENT_SETS {
        let soundName = CG_ConfigString(ctx, CS_AMBIENT_SET + i);
        if soundName.is_empty() {
            break;
        }

        trap::AS_AddPrecacheEntry(ctx.engine, &soundName);
    }
    let soundName = CG_ConfigString(ctx, CS_GLOBAL_AMBIENT_SET);
    if !soundName.is_empty() && Q_stricmp(&soundName, "default") != 0 {
        // global soundset
        trap::AS_AddPrecacheEntry(ctx.engine, &soundName);
    }

    trap::AS_ParseSets(ctx.engine);
}

/// Raven `static char *sb_nums[11]` — the full-size HUD number shaders.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1926-1938`
const sb_nums: [&str; 11] = [
    "gfx/2d/numbers/zero",
    "gfx/2d/numbers/one",
    "gfx/2d/numbers/two",
    "gfx/2d/numbers/three",
    "gfx/2d/numbers/four",
    "gfx/2d/numbers/five",
    "gfx/2d/numbers/six",
    "gfx/2d/numbers/seven",
    "gfx/2d/numbers/eight",
    "gfx/2d/numbers/nine",
    "gfx/2d/numbers/minus",
];

/// Raven `static char *sb_t_nums[11]` — the small HUD number shaders.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1940-1952`
const sb_t_nums: [&str; 11] = [
    "gfx/2d/numbers/t_zero",
    "gfx/2d/numbers/t_one",
    "gfx/2d/numbers/t_two",
    "gfx/2d/numbers/t_three",
    "gfx/2d/numbers/t_four",
    "gfx/2d/numbers/t_five",
    "gfx/2d/numbers/t_six",
    "gfx/2d/numbers/t_seven",
    "gfx/2d/numbers/t_eight",
    "gfx/2d/numbers/t_nine",
    "gfx/2d/numbers/t_minus",
];

/// Raven `static char *sb_c_nums[11]` — the chunky HUD number shaders. The
/// last row is Raven's own `"gfx/2d/numbers/t_minus", //?????`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1954-1966`
const sb_c_nums: [&str; 11] = [
    "gfx/2d/numbers/c_zero",
    "gfx/2d/numbers/c_one",
    "gfx/2d/numbers/c_two",
    "gfx/2d/numbers/c_three",
    "gfx/2d/numbers/c_four",
    "gfx/2d/numbers/c_five",
    "gfx/2d/numbers/c_six",
    "gfx/2d/numbers/c_seven",
    "gfx/2d/numbers/c_eight",
    "gfx/2d/numbers/c_nine",
    "gfx/2d/numbers/t_minus", //?????
];

/// Raven `CG_RegisterGraphics` — the map load's asset pass: world map, HUD
/// shaders, effects, chunk models, item visuals, inline/sub-BSP models and
/// terrain, stepping `cg.loadLCARSStage` as it goes.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1919-2426`
pub fn CG_RegisterGraphics(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // clear any references to old media
    ctx.world.cg.refdef = zeroed_refdef();
    trap::R_ClearScene(engine);

    let mapname = buf_to_string(&ctx.world.cgs.mapname.map(|c| c as u8));
    CG_LoadingString(ctx, &mapname);

    // #ifndef _XBOX
    trap::R_LoadWorldMap(engine, &mapname);
    // #endif

    // precache status bar pics
    // CG_LoadingString( "game media" );

    for i in 0..11 {
        ctx.world.cgs.media.numberShaders[i] = trap::R_RegisterShader(engine, sb_nums[i]);
    }

    ctx.world.cg.loadLCARSStage = 3;

    for i in 0..11 {
        ctx.world.cgs.media.numberShaders[i] = trap::R_RegisterShaderNoMip(engine, sb_nums[i]);
        ctx.world.cgs.media.smallnumberShaders[i] =
            trap::R_RegisterShaderNoMip(engine, sb_t_nums[i]);
        ctx.world.cgs.media.chunkyNumberShaders[i] =
            trap::R_RegisterShaderNoMip(engine, sb_c_nums[i]);
    }

    trap::R_RegisterShaderNoMip(engine, "gfx/mp/pduel_icon_lone");
    trap::R_RegisterShaderNoMip(engine, "gfx/mp/pduel_icon_double");

    ctx.world.cgs.media.balloonShader = trap::R_RegisterShader(engine, "gfx/mp/chat_icon");
    ctx.world.cgs.media.vchatShader = trap::R_RegisterShader(engine, "gfx/mp/vchat_icon");

    ctx.world.cgs.media.deferShader = trap::R_RegisterShaderNoMip(engine, "gfx/2d/defer.tga");

    ctx.world.cgs.media.radarShader =
        trap::R_RegisterShaderNoMip(engine, "gfx/menus/radar/radar.png");
    ctx.world.cgs.media.siegeItemShader =
        trap::R_RegisterShaderNoMip(engine, "gfx/menus/radar/goalitem");
    ctx.world.cgs.media.mAutomapPlayerIcon =
        trap::R_RegisterShader(engine, "gfx/menus/radar/arrow_w");
    ctx.world.cgs.media.mAutomapRocketIcon =
        trap::R_RegisterShader(engine, "gfx/menus/radar/rocket");

    ctx.world.cgs.media.wireframeAutomapFrame_left =
        trap::R_RegisterShader(engine, "gfx/mp_automap/mpauto_frame_left");
    ctx.world.cgs.media.wireframeAutomapFrame_right =
        trap::R_RegisterShader(engine, "gfx/mp_automap/mpauto_frame_right");
    ctx.world.cgs.media.wireframeAutomapFrame_top =
        trap::R_RegisterShader(engine, "gfx/mp_automap/mpauto_frame_top");
    ctx.world.cgs.media.wireframeAutomapFrame_bottom =
        trap::R_RegisterShader(engine, "gfx/mp_automap/mpauto_frame_bottom");

    ctx.world.cgs.media.lagometerShader = trap::R_RegisterShaderNoMip(engine, "gfx/2d/lag");
    ctx.world.cgs.media.connectionShader = trap::R_RegisterShaderNoMip(engine, "gfx/2d/net");

    trap::FX_InitSystem(engine, &mut ctx.world.cg.refdef);
    CG_RegisterEffects(ctx);

    ctx.world.cgs.media.boltShader = trap::R_RegisterShader(engine, "gfx/misc/blueLine");

    ctx.world.cgs.effects.turretShotEffect = trap::FX_RegisterEffect(engine, "turret/shot");
    ctx.world.cgs.effects.mEmplacedDeadSmoke =
        trap::FX_RegisterEffect(engine, "emplaced/dead_smoke.efx");
    ctx.world.cgs.effects.mEmplacedExplode =
        trap::FX_RegisterEffect(engine, "emplaced/explode.efx");
    ctx.world.cgs.effects.mTurretExplode = trap::FX_RegisterEffect(engine, "turret/explode.efx");
    ctx.world.cgs.effects.mSparkExplosion =
        trap::FX_RegisterEffect(engine, "sparks/spark_explosion.efx");
    ctx.world.cgs.effects.mTripmineExplosion =
        trap::FX_RegisterEffect(engine, "tripMine/explosion.efx");
    ctx.world.cgs.effects.mDetpackExplosion =
        trap::FX_RegisterEffect(engine, "detpack/explosion.efx");
    ctx.world.cgs.effects.mFlechetteAltBlow =
        trap::FX_RegisterEffect(engine, "flechette/alt_blow.efx");
    ctx.world.cgs.effects.mStunBatonFleshImpact =
        trap::FX_RegisterEffect(engine, "stunBaton/flesh_impact.efx");
    ctx.world.cgs.effects.mAltDetonate = trap::FX_RegisterEffect(engine, "demp2/altDetonate.efx");
    ctx.world.cgs.effects.mSparksExplodeNoSound =
        trap::FX_RegisterEffect(engine, "sparks/spark_exp_nosnd");
    ctx.world.cgs.effects.mTripMineLaster = trap::FX_RegisterEffect(engine, "tripMine/laser.efx");
    ctx.world.cgs.effects.mEmplacedMuzzleFlash =
        trap::FX_RegisterEffect(engine, "effects/emplaced/muzzle_flash");
    ctx.world.cgs.effects.mConcussionAltRing =
        trap::FX_RegisterEffect(engine, "concussion/alt_ring");

    ctx.world.cgs.effects.mHyperspaceStars =
        trap::FX_RegisterEffect(engine, "ships/hyperspace_stars");
    ctx.world.cgs.effects.mBlackSmoke = trap::FX_RegisterEffect(engine, "volumetric/black_smoke");
    ctx.world.cgs.effects.mShipDestDestroyed =
        trap::FX_RegisterEffect(engine, "effects/ships/dest_destroyed.efx");
    ctx.world.cgs.effects.mShipDestBurning =
        trap::FX_RegisterEffect(engine, "effects/ships/dest_burning.efx");
    ctx.world.cgs.effects.mBobaJet = trap::FX_RegisterEffect(engine, "effects/boba/jet.efx");

    ctx.world.cgs.effects.itemCone = trap::FX_RegisterEffect(engine, "mp/itemcone.efx");
    ctx.world.cgs.effects.mTurretMuzzleFlash =
        trap::FX_RegisterEffect(engine, "effects/turret/muzzle_flash.efx");
    ctx.world.cgs.effects.mSparks = trap::FX_RegisterEffect(engine, "sparks/spark_nosnd.efx"); //sparks/spark.efx
    ctx.world.cgs.effects.mSaberCut = trap::FX_RegisterEffect(engine, "saber/saber_cut.efx");
    ctx.world.cgs.effects.mSaberBlock = trap::FX_RegisterEffect(engine, "saber/saber_block.efx");
    ctx.world.cgs.effects.mSaberBloodSparks =
        trap::FX_RegisterEffect(engine, "saber/blood_sparks_mp.efx");
    ctx.world.cgs.effects.mSaberBloodSparksSmall =
        trap::FX_RegisterEffect(engine, "saber/blood_sparks_25_mp.efx");
    ctx.world.cgs.effects.mSaberBloodSparksMid =
        trap::FX_RegisterEffect(engine, "saber/blood_sparks_50_mp.efx");
    ctx.world.cgs.effects.mSpawn = trap::FX_RegisterEffect(engine, "mp/spawn.efx");
    ctx.world.cgs.effects.mJediSpawn = trap::FX_RegisterEffect(engine, "mp/jedispawn.efx");
    ctx.world.cgs.effects.mBlasterDeflect = trap::FX_RegisterEffect(engine, "blaster/deflect.efx");
    ctx.world.cgs.effects.mBlasterSmoke = trap::FX_RegisterEffect(engine, "blaster/smoke_bolton");
    ctx.world.cgs.effects.mForceConfustionOld =
        trap::FX_RegisterEffect(engine, "force/confusion_old.efx");

    ctx.world.cgs.effects.forceLightning =
        trap::FX_RegisterEffect(engine, "effects/force/lightning.efx");
    ctx.world.cgs.effects.forceLightningWide =
        trap::FX_RegisterEffect(engine, "effects/force/lightningwide.efx");
    ctx.world.cgs.effects.forceDrain = trap::FX_RegisterEffect(engine, "effects/mp/drain.efx");
    ctx.world.cgs.effects.forceDrainWide =
        trap::FX_RegisterEffect(engine, "effects/mp/drainwide.efx");
    ctx.world.cgs.effects.forceDrained = trap::FX_RegisterEffect(engine, "effects/mp/drainhit.efx");

    ctx.world.cgs.effects.mDisruptorDeathSmoke =
        trap::FX_RegisterEffect(engine, "disruptor/death_smoke");

    for i in 0..NUM_CROSSHAIRS {
        let name = format!("gfx/2d/crosshair{}", (b'a' + i as u8) as char);
        ctx.world.cgs.media.crosshairShader[i] = trap::R_RegisterShaderNoMip(engine, &name);
    }

    ctx.world.cg.loadLCARSStage = 4;

    ctx.world.cgs.media.backTileShader = trap::R_RegisterShader(engine, "gfx/2d/backtile");

    // precache the fpls skin
    // trap_R_RegisterSkin("models/players/kyle/model_fpls2.skin");

    ctx.world.cgs.media.itemRespawningPlaceholder =
        trap::R_RegisterShader(engine, "powerups/placeholder");
    ctx.world.cgs.media.itemRespawningRezOut = trap::R_RegisterShader(engine, "powerups/rezout");

    ctx.world.cgs.media.playerShieldDamage =
        trap::R_RegisterShader(engine, "gfx/misc/personalshield");
    ctx.world.cgs.media.protectShader = trap::R_RegisterShader(engine, "gfx/misc/forceprotect");
    ctx.world.cgs.media.forceSightBubble = trap::R_RegisterShader(engine, "gfx/misc/sightbubble");
    ctx.world.cgs.media.forceShell = trap::R_RegisterShader(engine, "powerups/forceshell");
    ctx.world.cgs.media.sightShell = trap::R_RegisterShader(engine, "powerups/sightshell");

    ctx.world.cgs.media.itemHoloModel =
        trap::R_RegisterModel(engine, "models/map_objects/mp/holo.md3");

    // DEFERRED: forceHolocronModels[] — oracle/codemp/cgame/cg_ents.c:632-651
    // Raven's `if (cgs.gametype == GT_HOLOCRON || cg_buildScript.integer)` block
    // walks that 18-entry model table and registers each row. The table is
    // `cg_ents.c` file scope (`cg_main.c:1909` only declares it `extern`), so its
    // Rust home is `cg_ents.rs` — a file this wave may not open, and nothing in
    // the tree declares it yet. The models still register lazily when `CG_Item`
    // draws a holocron; only the up-front precache is missing.

    if ctx.world.cgs.gametype == GT_CTF
        || ctx.world.cgs.gametype == GT_CTY
        || ctx.world.cvars.cg_buildScript.integer != 0
    {
        if ctx.world.cvars.cg_buildScript.integer != 0 {
            trap::R_RegisterModel(engine, "models/flags/r_flag.md3");
            trap::R_RegisterModel(engine, "models/flags/b_flag.md3");
            trap::R_RegisterModel(engine, "models/flags/r_flag_ysal.md3");
            trap::R_RegisterModel(engine, "models/flags/b_flag_ysal.md3");
        }

        if ctx.world.cgs.gametype == GT_CTF {
            ctx.world.cgs.media.redFlagModel =
                trap::R_RegisterModel(engine, "models/flags/r_flag.md3");
            ctx.world.cgs.media.blueFlagModel =
                trap::R_RegisterModel(engine, "models/flags/b_flag.md3");
        } else {
            ctx.world.cgs.media.redFlagModel =
                trap::R_RegisterModel(engine, "models/flags/r_flag_ysal.md3");
            ctx.world.cgs.media.blueFlagModel =
                trap::R_RegisterModel(engine, "models/flags/b_flag_ysal.md3");
        }

        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_rflag_x");
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_bflag_x");

        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_rflag_ys");
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_bflag_ys");

        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_rflag");
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mpi_bflag");

        trap::R_RegisterShaderNoMip(engine, "gfx/2d/net.tga");

        ctx.world.cgs.media.flagPoleModel =
            trap::R_RegisterModel(engine, "models/flag2/flagpole.md3");
        ctx.world.cgs.media.flagFlapModel =
            trap::R_RegisterModel(engine, "models/flag2/flagflap3.md3");

        ctx.world.cgs.media.redFlagBaseModel =
            trap::R_RegisterModel(engine, "models/mapobjects/flagbase/red_base.md3");
        ctx.world.cgs.media.blueFlagBaseModel =
            trap::R_RegisterModel(engine, "models/mapobjects/flagbase/blue_base.md3");
        ctx.world.cgs.media.neutralFlagBaseModel =
            trap::R_RegisterModel(engine, "models/mapobjects/flagbase/ntrl_base.md3");
    }

    if ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0 {
        ctx.world.cgs.media.teamRedShader = trap::R_RegisterShader(engine, "sprites/team_red");
        ctx.world.cgs.media.teamBlueShader = trap::R_RegisterShader(engine, "sprites/team_blue");
        // cgs.media.redQuadShader = trap_R_RegisterShader("powerups/blueflag" );
        ctx.world.cgs.media.teamStatusBar = trap::R_RegisterShader(engine, "gfx/2d/colorbar.tga");
    } else if ctx.world.cgs.gametype == GT_JEDIMASTER {
        ctx.world.cgs.media.teamRedShader = trap::R_RegisterShader(engine, "sprites/team_red");
    }

    if ctx.world.cgs.gametype == GT_POWERDUEL || ctx.world.cvars.cg_buildScript.integer != 0 {
        // trap_R_RegisterShader("gfx/mp/pduel_gameicon_ally")
        ctx.world.cgs.media.powerDuelAllyShader =
            trap::R_RegisterShader(engine, "gfx/mp/pduel_icon_double");
    }

    ctx.world.cgs.media.heartShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/selectedhealth.tga");

    ctx.world.cgs.media.ysaliredShader = trap::R_RegisterShader(engine, "powerups/ysaliredshell");
    ctx.world.cgs.media.ysaliblueShader = trap::R_RegisterShader(engine, "powerups/ysaliblueshell");
    ctx.world.cgs.media.ysalimariShader = trap::R_RegisterShader(engine, "powerups/ysalimarishell");
    ctx.world.cgs.media.boonShader = trap::R_RegisterShader(engine, "powerups/boonshell");
    ctx.world.cgs.media.endarkenmentShader =
        trap::R_RegisterShader(engine, "powerups/endarkenmentshell");
    ctx.world.cgs.media.enlightenmentShader =
        trap::R_RegisterShader(engine, "powerups/enlightenmentshell");
    ctx.world.cgs.media.invulnerabilityShader =
        trap::R_RegisterShader(engine, "powerups/invulnerabilityshell");

    // Raven's six `#ifdef JK2AWARDS` medal shaders are compiled out of the MP
    // build (`cg_main.c:2163-2170`).

    // Binocular interface
    ctx.world.cgs.media.binocularCircle = trap::R_RegisterShader(engine, "gfx/2d/binCircle");
    ctx.world.cgs.media.binocularMask = trap::R_RegisterShader(engine, "gfx/2d/binMask");
    ctx.world.cgs.media.binocularArrow = trap::R_RegisterShader(engine, "gfx/2d/binSideArrow");
    ctx.world.cgs.media.binocularTri = trap::R_RegisterShader(engine, "gfx/2d/binTopTri");
    ctx.world.cgs.media.binocularStatic = trap::R_RegisterShader(engine, "gfx/2d/binocularWindow");
    ctx.world.cgs.media.binocularOverlay =
        trap::R_RegisterShader(engine, "gfx/2d/binocularNumOverlay");

    ctx.world.cg.loadLCARSStage = 5;

    // Chunk models
    // FIXME: jfm:? bother to conditionally load these if an ent has this material type?
    for i in 0..NUM_CHUNK_MODELS {
        ctx.world.cgs.media.chunkModels[CHUNK_METAL2][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/metal/metal1_{}.md3", i + 1)); //_ /switched\ _
        ctx.world.cgs.media.chunkModels[CHUNK_METAL1][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/metal/metal2_{}.md3", i + 1)); //  \switched/
        ctx.world.cgs.media.chunkModels[CHUNK_ROCK1][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/rock/rock1_{}.md3", i + 1));
        ctx.world.cgs.media.chunkModels[CHUNK_ROCK2][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/rock/rock2_{}.md3", i + 1));
        ctx.world.cgs.media.chunkModels[CHUNK_ROCK3][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/rock/rock3_{}.md3", i + 1));
        ctx.world.cgs.media.chunkModels[CHUNK_CRATE1][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/crate/crate1_{}.md3", i + 1));
        ctx.world.cgs.media.chunkModels[CHUNK_CRATE2][i] =
            trap::R_RegisterModel(engine, &format!("models/chunks/crate/crate2_{}.md3", i + 1));
        ctx.world.cgs.media.chunkModels[CHUNK_WHITE_METAL][i] = trap::R_RegisterModel(
            engine,
            &format!("models/chunks/metal/wmetal1_{}.md3", i + 1),
        );
    }

    ctx.world.cgs.media.chunkSound =
        trap::S_RegisterSound(engine, "sound/weapons/explosions/glasslcar");
    ctx.world.cgs.media.grateSound = trap::S_RegisterSound(engine, "sound/effects/grate_destroy");
    ctx.world.cgs.media.rockBreakSound = trap::S_RegisterSound(engine, "sound/effects/wall_smash");
    ctx.world.cgs.media.rockBounceSound[0] =
        trap::S_RegisterSound(engine, "sound/effects/stone_bounce");
    ctx.world.cgs.media.rockBounceSound[1] =
        trap::S_RegisterSound(engine, "sound/effects/stone_bounce2");
    ctx.world.cgs.media.metalBounceSound[0] =
        trap::S_RegisterSound(engine, "sound/effects/metal_bounce");
    ctx.world.cgs.media.metalBounceSound[1] =
        trap::S_RegisterSound(engine, "sound/effects/metal_bounce2");
    ctx.world.cgs.media.glassChunkSound =
        trap::S_RegisterSound(engine, "sound/weapons/explosions/glassbreak1");
    ctx.world.cgs.media.crateBreakSound[0] =
        trap::S_RegisterSound(engine, "sound/weapons/explosions/crateBust1");
    ctx.world.cgs.media.crateBreakSound[1] =
        trap::S_RegisterSound(engine, "sound/weapons/explosions/crateBust2");

    // Ghoul2 Insert Start
    CG_InitItems(ctx.world);
    // Ghoul2 Insert End

    for w in ctx.world.cg_weapons.iter_mut() {
        *w = zeroed_weapon_info();
    }

    // only register the items that the server says we need
    let items = CG_ConfigString(ctx, CS_ITEMS);
    let itemBytes = items.as_bytes();

    for i in 1..bg_numItems {
        // §F19: Raven reads `char items[MAX_ITEMS+1]` past the copied string's
        // terminator when the configstring is short; a byte that isn't there is
        // "not requested" here.
        if itemBytes.get(i as usize) == Some(&b'1') || ctx.world.cvars.cg_buildScript.integer != 0 {
            CG_LoadingItem(ctx, i);
            CG_RegisterItemVisuals(ctx, i);
        }
    }

    ctx.world.cg.loadLCARSStage = 6;

    ctx.world.cgs.media.glassShardShader = trap::R_RegisterShader(engine, "gfx/misc/test_crackle");

    // doing one shader just makes it look like a shell.  By using two shaders with different bulge offsets and different texture scales, it has a much more chaotic look
    ctx.world.cgs.media.electricBodyShader = trap::R_RegisterShader(engine, "gfx/misc/electric");
    ctx.world.cgs.media.electricBody2Shader =
        trap::R_RegisterShader(engine, "gfx/misc/fullbodyelectric2");

    ctx.world.cgs.media.fsrMarkShader = trap::R_RegisterShader(engine, "footstep_r");
    ctx.world.cgs.media.fslMarkShader = trap::R_RegisterShader(engine, "footstep_l");
    ctx.world.cgs.media.fshrMarkShader = trap::R_RegisterShader(engine, "footstep_heavy_r");
    ctx.world.cgs.media.fshlMarkShader = trap::R_RegisterShader(engine, "footstep_heavy_l");

    ctx.world.cgs.media.refractionShader = trap::R_RegisterShader(engine, "effects/refraction");

    ctx.world.cgs.media.cloakedShader = trap::R_RegisterShader(engine, "gfx/effects/cloakedShader");

    // wall marks
    ctx.world.cgs.media.shadowMarkShader = trap::R_RegisterShader(engine, "markShadow");
    ctx.world.cgs.media.wakeMarkShader = trap::R_RegisterShader(engine, "wake");

    ctx.world.cgs.media.viewPainShader = trap::R_RegisterShader(engine, "gfx/misc/borgeyeflare");
    ctx.world.cgs.media.viewPainShader_Shields =
        trap::R_RegisterShader(engine, "gfx/mp/dmgshader_shields");
    ctx.world.cgs.media.viewPainShader_ShieldsAndHealth =
        trap::R_RegisterShader(engine, "gfx/mp/dmgshader_shieldsandhealth");

    // register the inline models
    ctx.world.cgs.numInlineModels = trap::CM_NumInlineModels(engine);
    let mut breakPoint = ctx.world.cgs.numInlineModels;
    for i in 1..ctx.world.cgs.numInlineModels {
        let name = format!("*{i}");
        ctx.world.cgs.inlineDrawModel[i as usize] = trap::R_RegisterModel(engine, &name);
        if ctx.world.cgs.inlineDrawModel[i as usize] == 0 {
            breakPoint = i;
            break;
        }

        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        trap::R_ModelBounds(
            engine,
            ctx.world.cgs.inlineDrawModel[i as usize],
            &mut mins,
            &mut maxs,
        );
        for j in 0..3 {
            ctx.world.cgs.inlineModelMidpoints[i as usize][j] =
                (mins[j] as f64 + 0.5 * (maxs[j] as f64 - mins[j] as f64)) as f32;
        }
    }

    ctx.world.cg.loadLCARSStage = 7;

    // register all the server specified models
    for i in 1..MAX_MODELS {
        let cModelName = CG_ConfigString(ctx, CS_MODELS + i);
        if cModelName.is_empty() {
            break;
        }

        let mut modelName = cModelName;
        if modelName.contains(".glm") || modelName.starts_with('$') {
            // Check to see if it has a custom skin attached.
            CG_HandleAppendedSkin(ctx, &mut modelName);
            CG_CacheG2AnimInfo(ctx, &modelName);
        }

        if !modelName.starts_with('$') && !modelName.starts_with('@') {
            // don't register vehicle names and saber names as models.
            ctx.world.cgs.gameModels[i as usize] = trap::R_RegisterModel(engine, &modelName);
        } else {
            // FIXME: register here so that stuff gets precached!!!
            ctx.world.cgs.gameModels[i as usize] = 0;
        }
    }
    ctx.world.cg.loadLCARSStage = 8;
    // Ghoul2 Insert Start

    // CG_LoadingString( "BSP instances" );

    for i in 1..MAX_SUB_BSP {
        let bspName = CG_ConfigString(ctx, CS_BSP_MODELS + i);
        if bspName.is_empty() {
            break;
        }

        trap::CM_LoadMap(engine, &bspName, true);
        ctx.world.cgs.inlineDrawModel[breakPoint as usize] =
            trap::R_RegisterModel(engine, &bspName);
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        trap::R_ModelBounds(
            engine,
            ctx.world.cgs.inlineDrawModel[breakPoint as usize],
            &mut mins,
            &mut maxs,
        );
        for j in 0..3 {
            ctx.world.cgs.inlineModelMidpoints[breakPoint as usize][j] =
                (mins[j] as f64 + 0.5 * (maxs[j] as f64 - mins[j] as f64)) as f32;
        }
        breakPoint += 1;
        for sub in 1..MAX_MODELS {
            let temp = format!("*{i}-{sub}");
            ctx.world.cgs.inlineDrawModel[breakPoint as usize] =
                trap::R_RegisterModel(engine, &temp);
            if ctx.world.cgs.inlineDrawModel[breakPoint as usize] == 0 {
                break;
            }
            let mut mins: vec3_t = [0.0; 3];
            let mut maxs: vec3_t = [0.0; 3];
            trap::R_ModelBounds(
                engine,
                ctx.world.cgs.inlineDrawModel[breakPoint as usize],
                &mut mins,
                &mut maxs,
            );
            for j in 0..3 {
                ctx.world.cgs.inlineModelMidpoints[breakPoint as usize][j] =
                    (mins[j] as f64 + 0.5 * (maxs[j] as f64 - mins[j] as f64)) as f32;
            }
            breakPoint += 1;
        }
    }

    // CG_LoadingString( "Creating terrain" );
    for i in 1..MAX_TERRAINS {
        let terrainInfo = CG_ConfigString(ctx, CS_TERRAINS + i);
        if terrainInfo.is_empty() {
            break;
        }

        let terrainID = trap::CM_RegisterTerrain(engine, &terrainInfo);

        trap::RMG_Init(engine, terrainID, &terrainInfo);

        // Send off the terrainInfo to the renderer
        trap::RE_InitRendererTerrain(engine, &terrainInfo);
    }

    // Raven's `CS_CHARSKINS` skin loop is commented out; rww replaced it with
    // CS_G2BONES - a custom skin is now a `*` suffix on the indexed model name,
    // used for NPCs only.

    // CG_LoadingString("weapons");

    CG_InitG2Weapons(ctx);

    // Ghoul2 Insert End
    ctx.world.cg.loadLCARSStage = 9;

    // new stuff
    ctx.world.cgs.media.patrolShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/patrol.tga");
    ctx.world.cgs.media.assaultShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/assault.tga");
    ctx.world.cgs.media.campShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/camp.tga");
    ctx.world.cgs.media.followShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/follow.tga");
    ctx.world.cgs.media.defendShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/defend.tga");
    ctx.world.cgs.media.teamLeaderShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/team_leader.tga");
    ctx.world.cgs.media.retrieveShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/retrieve.tga");
    ctx.world.cgs.media.escortShader =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/escort.tga");
    ctx.world.cgs.media.cursor = trap::R_RegisterShaderNoMip(engine, "menu/art/3_cursor2");
    ctx.world.cgs.media.sizeCursor =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/sizecursor.tga");
    ctx.world.cgs.media.selectCursor =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/selectcursor.tga");
    ctx.world.cgs.media.flagShaders[0] =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/flag_in_base.tga");
    ctx.world.cgs.media.flagShaders[1] =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/flag_capture.tga");
    ctx.world.cgs.media.flagShaders[2] =
        trap::R_RegisterShaderNoMip(engine, "ui/assets/statusbar/flag_missing.tga");

    ctx.world.cgs.media.halfShieldModel =
        trap::R_RegisterModel(engine, "models/weaphits/testboom.md3");
    ctx.world.cgs.media.halfShieldShader = trap::R_RegisterShader(engine, "halfShieldShell");

    trap::FX_RegisterEffect(engine, "force/force_touch");

    CG_ClearParticles(ctx.world);

    // Raven's `MAX_PARTICLES_AREAS` `CG_NewParticleArea` loop is commented out.
}

/// Raven `CG_BuildSpectatorString` — rebuilds the scrolling spectator list and
/// latches its width for a re-measure when the text changed.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2480-2497`
pub fn CG_BuildSpectatorString(ctx: &mut CgContext) {
    ctx.world.cg.spectatorList[0] = 0;

    // Count up the number of players per team and per class
    CG_SiegeCountCvars(ctx);

    for i in 0..MAX_CLIENTS {
        if ctx.world.cgs.clientinfo[i].infoValid != qfalse
            && ctx.world.cgs.clientinfo[i].team == TEAM_SPECTATOR
        {
            let name = buf_to_string(&ctx.world.cgs.clientinfo[i].name.map(|c| c as u8));
            Q_strcat(
                &mut ctx.world.cg.spectatorList,
                MAX_STRING_CHARS,
                &format!("{name}     "),
            );
        }
    }
    let i = ctx
        .world
        .cg
        .spectatorList
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(MAX_STRING_CHARS) as c_int;
    if i != ctx.world.cg.spectatorLen {
        ctx.world.cg.spectatorLen = i;
        ctx.world.cg.spectatorWidth = -1.0;
    }
}

/// Raven `CG_StartMusic` — starts the map's background track from the
/// `CS_MUSIC` configstring's intro/loop pair.
///
/// `bForceStart` is inverted at the trap: Raven passes it as
/// `bReturnWithoutStarting`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2550-2560`
pub fn CG_StartMusic(ctx: &mut CgContext, bForceStart: bool) {
    // start the background music
    let s = CG_ConfigString(ctx, CS_MUSIC);
    let (tok1, s) = COM_Parse(&s, true);
    let parm1 = strncpyz_string(&string_to_latin1(&tok1), MAX_QPATH);
    let (tok2, _s) = COM_Parse(s, true);
    let parm2 = strncpyz_string(&string_to_latin1(&tok2), MAX_QPATH);

    trap::S_StartBackgroundTrack(ctx.engine, &parm1, &parm2, !bForceStart);
}

/// Raven `CG_Load_Menu` — consumes one `{ menufile menufile … }` block out of
/// the hud-menu list and parses each named file. `p` is the caller's cursor,
/// advanced in place.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2800-2826`
pub fn CG_Load_Menu(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &mut DisplayState,
    p: &mut &str,
) -> bool {
    // Raven's `COM_ParseExt(p, qtrue)`; the ported `COM_Parse` already carries
    // the `allowLineBreaks` flag.
    let (token, rest) = COM_Parse(*p, true);
    *p = rest;

    if !token.starts_with('{') {
        return false;
    }

    loop {
        let (token, rest) = COM_Parse(*p, true);
        *p = rest;

        if Q_stricmp(&token, "}") == 0 {
            return true;
        }

        if token.is_empty() {
            return false;
        }

        CG_ParseMenu(ctx, menus, ds, &token);
    }
    // Raven's trailing `return qfalse;` is unreachable past the `while (1)`.
}

/// Raven `CG_Text_PaintWithCursor` — the menu framework's edit-field text slot.
/// Raven never draws the cursor here, it just forwards to
/// [`CG_Text_Paint`](crate::cg_draw::CG_Text_Paint) with `adjust` zeroed.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3027-3029`
#[allow(clippy::too_many_arguments)]
pub fn CG_Text_PaintWithCursor(
    ctx: &CgContext,
    cgDC: &DisplayState,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    _cursorPos: c_int,
    _cursor: u8,
    limit: c_int,
    style: c_int,
    iMenuFont: c_int,
) {
    CG_Text_Paint(
        ctx, cgDC, x, y, scale, color, text, 0.0, limit, style, iMenuFont,
    );
}

/// Raven `CG_OwnerDrawWidth` — how wide the five text ownerdraws paint, so the
/// menu framework can right-align them. Anything else measures 0.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3031-3051`
pub fn CG_OwnerDrawWidth(
    ctx: &mut CgContext,
    _menus: &mut MenuSystem,
    ds: &DisplayState,
    ownerDraw: c_int,
    scale: f32,
) -> c_int {
    match ownerDraw {
        CG_GAME_TYPE => {
            let s = CG_GameTypeString(ctx.world);
            CG_Text_Width(ctx, ds, s, scale, FONT_MEDIUM)
        }
        CG_GAME_STATUS => {
            let s = CG_GetGameStatusText(ctx);
            CG_Text_Width(ctx, ds, &s, scale, FONT_MEDIUM)
        }
        CG_KILLER => {
            let s = CG_GetKillerText(ctx);
            CG_Text_Width(ctx, ds, &s, scale, FONT_MEDIUM)
        }
        // cg_redTeamName.string
        CG_RED_NAME => CG_Text_Width(ctx, ds, DEFAULT_REDTEAM_NAME, scale, FONT_MEDIUM),
        // cg_blueTeamName.string
        CG_BLUE_NAME => CG_Text_Width(ctx, ds, DEFAULT_BLUETEAM_NAME, scale, FONT_MEDIUM),

        _ => 0,
    }
}

/// Raven `CG_StrPool_Alloc` — hands out `size` zeroed bytes off the 32K
/// ent-parsing pool.
///
/// The bump arena is gone: the pool's one consumer (`CG_NewString`) builds an
/// owned string, so this returns the owned zeroed buffer and only the budget
/// counter survives (the same fold [`CG_AddSpawnVarToken`] got). The overrun
/// `Com_Error` is kept because it is observable; Raven's never comes back and
/// the port's does, so an over-budget request is still served below.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3313-3329`
pub fn CG_StrPool_Alloc(ctx: &mut CgContext, size: c_int) -> Vec<u8> {
    if ctx.world.main.cg_strPoolSize + size >= MAX_CGSTRPOOL_SIZE as c_int {
        Com_Error(
            ctx,
            errorParm_t::ERR_DROP as c_int,
            "You exceeded the cgame string pool size. Bad programmer!\n",
        );
    }

    ctx.world.main.cg_strPoolSize += size;

    // memset it for them, just to be nice.
    // §F19: a negative `size` is a negative `memset` length in Raven; here it
    // clamps to an empty buffer.
    vec![0u8; size.max(0) as usize]
}

/// Raven `CG_CreateModelFromSpawnEnt` — a `misc_model_static` becomes one
/// misc-ent draw record, scaled, z-offset and yawed into place.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3452-3508`
pub fn CG_CreateModelFromSpawnEnt(ctx: &mut CgContext, ent: &mut CgSpawnEnt) {
    if ctx.world.main.miscEnts.len() >= MAX_MISC_ENTS {
        let msg = format!("Too many misc_model_static's on level, ask a programmer to raise the limit (currently {MAX_MISC_ENTS}), or take some out.");
        Com_Error(ctx, errorParm_t::ERR_DROP as c_int, &msg);
        return;
    }

    // Raven's `!ent || !ent->model` null tests are gone with the reference and
    // the owned `String`; the empty-model test is the one that survives.
    if ent.model.is_empty() {
        Com_Error(
            ctx,
            errorParm_t::ERR_DROP as c_int,
            "misc_model_static with no model.",
        );
        return;
    }

    // `RefEnt = &MiscEnts[NumMiscEnts++]` — same claim-then-fill as [`CG_MiscEnt`].
    ctx.world.main.miscEnts.push(CgMiscEnt {
        ent: refEntity_t::zeroed(),
        radius: 0.0,
        zOffset: 0.0,
    });
    let slot = ctx.world.main.miscEnts.len() - 1;

    let modelIndex = trap::R_RegisterModel(ctx.engine, &ent.model);
    if modelIndex == 0 {
        let msg = format!("misc_model_static failed to load model '{}'", ent.model);
        Com_Error(ctx, errorParm_t::ERR_DROP as c_int, &msg);
        return;
    }

    let mut RefEnt = refEntity_t::zeroed();
    RefEnt.reType = refEntityType_t::RT_MODEL;
    RefEnt.hModel = modelIndex;
    RefEnt.frame = 0;

    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    trap::R_ModelBounds(ctx.engine, modelIndex, &mut mins, &mut maxs);
    _VectorCopy(ent.scale, &mut RefEnt.modelScale);
    if ent.fScale != 0.0 {
        // use same scale on each axis then
        RefEnt.modelScale[0] = ent.fScale;
        RefEnt.modelScale[1] = ent.fScale;
        RefEnt.modelScale[2] = ent.fScale;
    }
    _VectorCopy(ent.origin, &mut RefEnt.origin);
    _VectorCopy(ent.origin, &mut RefEnt.lightingOrigin);

    // Raven `VectorScaleVector(mins, ent->scale, mins)` - the per-axis `scale`,
    // not the `fScale` override applied above.
    mins[0] *= ent.scale[0];
    mins[1] *= ent.scale[1];
    mins[2] *= ent.scale[2];
    maxs[0] *= ent.scale[0];
    maxs[1] *= ent.scale[1];
    maxs[2] *= ent.scale[2];
    ctx.world.main.miscEnts[slot].radius = Distance(mins, maxs);
    ctx.world.main.miscEnts[slot].zOffset = ent.zoffset;

    if ent.angle != 0.0 {
        // only yaw supplied...
        ent.angles[YAW] = ent.angle;
    }

    AnglesToAxis(ent.angles, RefEnt.axis.as_mut_ptr());
    ScaleModelAxis(&mut RefEnt);

    ctx.world.main.miscEnts[slot].ent = RefEnt;
}

/// Raven `CG_ParseSpawnVars` — pulls one map entity's key/value pairs out of the
/// engine's entity-token stream into `cg_spawnVars`. `false` is the end of the
/// spawn string.
///
/// Raven's `CG_Error` never comes back; the port's trap wrapper does, so every
/// error path below answers `false` — the same "no more entities" the missing
/// opening brace already answers.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3541-3593`
pub fn CG_ParseSpawnVars(ctx: &mut CgContext) -> bool {
    ctx.world.main.cg_spawnVars.clear();
    ctx.world.main.cg_numSpawnVarChars = 0;

    // parse the opening brace
    let Some(com_token) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
        // end of spawn string
        return false;
    };
    if !com_token.starts_with('{') {
        let msg = format!("CG_ParseSpawnVars: found {com_token} when expecting {{");
        CG_Error(ctx, &msg);
        return false;
    }

    // go through all the key / value pairs
    loop {
        // parse key
        let Some(keyname) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
            CG_Error(ctx, "CG_ParseSpawnVars: EOF without closing brace");
            return false;
        };

        if keyname.starts_with('}') {
            break;
        }

        // parse value
        let Some(com_token) = trap::GetEntityToken(ctx.engine, MAX_TOKEN_CHARS) else {
            // this happens on mike's test level, I don't know why. Fixme?
            // CG_Error( "CG_ParseSpawnVars: EOF without closing brace" );
            break;
        };

        if com_token.starts_with('}') {
            CG_Error(ctx, "CG_ParseSpawnVars: closing brace without data");
            return false;
        }
        if ctx.world.main.cg_spawnVars.len() == MAX_SPAWN_VARS {
            CG_Error(ctx, "CG_ParseSpawnVars: MAX_SPAWN_VARS");
            return false;
        }
        let key = CG_AddSpawnVarToken(ctx, &keyname);
        let value = CG_AddSpawnVarToken(ctx, &com_token);
        ctx.world.main.cg_spawnVars.push([key, value]);
    }

    true
}

/// Raven `CG_DestroyAllGhoul2` — the map-teardown sweep: every entity's ghoul2
/// instances and npc client info, the weapon instances, every item's g2 models
/// and the global jetpack instance.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3951-3984`
pub fn CG_DestroyAllGhoul2(ctx: &mut CgContext) {
    // Com_Printf("... CGameside GHOUL2 Cleanup\n");
    // free all dynamically allocated npc client info structs and ghoul2 instances
    for i in 0..MAX_GENTITIES {
        CG_KillCEntityG2(ctx, i);
    }

    // Clean the weapon instances
    CG_ShutDownG2Weapons(ctx);

    let engine = ctx.engine;
    // and now for items
    for i in 0..MAX_ITEMS {
        for j in 0..MAX_ITEM_MODELS {
            let g2 = ctx.world.cg_items[i].g2Models[j];
            if !g2.is_null() && trap::G2_HaveWeGhoul2Models(engine, g2) {
                trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.cg_items[i].g2Models[j]);
                ctx.world.cg_items[i].g2Models[j] = null_mut();
            }
        }
    }

    // Clean the global jetpack instance
    CG_CleanJetpackGhoul2(ctx);
}

/// Raven `C_Trace` — the `CG_TRACE` vmcall body.
///
/// Same DEC-46.6 shape as [`C_PointContents`]: Raven casts `cg.sharedBuffer` to
/// `TCGTrace *` right here, the port takes the already-decoded payload from the
/// vmMain dispatch boundary and writes the result back into `mResult`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:408-413`
pub fn C_Trace(ctx: &mut CgContext, td: &mut TCGTrace) {
    CG_Trace(
        ctx,
        &mut td.mResult,
        &td.mStart,
        &td.mMins,
        &td.mMaxs,
        &td.mEnd,
        td.mSkipNumber,
        td.mMask,
    );
}

/// Raven `C_G2Trace` — [`C_Trace`]'s twin on the `CG_G2TRACE` vmcall, so the
/// sweep also probes ghoul2 sub-models.
///
/// Source: `oracle/codemp/cgame/cg_main.c:415-420`
pub fn C_G2Trace(ctx: &mut CgContext, td: &mut TCGTrace) {
    CG_G2Trace(
        ctx,
        &mut td.mResult,
        &td.mStart,
        &td.mMins,
        &td.mMaxs,
        &td.mEnd,
        td.mSkipNumber,
        td.mMask,
    );
}

/// Raven `C_G2Mark` — the `CG_G2MARK` vmcall body: fire a 64-unit probe along
/// `dir` and project a gore decal onto whoever it hits.
///
/// Source: `oracle/codemp/cgame/cg_main.c:422-443`
pub fn C_G2Mark(ctx: &mut CgContext, td: &TCGG2Mark) {
    let mut tr = trace_t::zeroed();
    let mut end: vec3_t = [0.0; 3];

    _VectorMA(td.start, 64.0, td.dir, &mut end);
    // Raven passes NULL mins/maxs; `CM_BoxTrace` substitutes `vec3_origin` for a
    // NULL box (`oracle/codemp/qcommon/cm_trace.cpp:1603-1606`) and
    // `CG_ClipMoveToEntities` only forwards them, so the zero vector is the same
    // trace.
    CG_G2Trace(
        ctx,
        &mut tr,
        &td.start,
        &vec3_origin,
        &vec3_origin,
        &end,
        ENTITYNUM_NONE,
        MASK_PLAYERSOLID,
    );

    if (tr.entityNum as c_int) < ENTITYNUM_WORLD
        && !ctx.world.entities[tr.entityNum as usize].ghoul2.is_null()
    {
        //hit someone with a ghoul2 instance, let's project the decal on them then.
        let cent = tr.entityNum as usize;

        //CG_TestLine(tr.endpos, end, 2000, 0x0000ff, 1);

        let lerpOrigin = ctx.world.entities[cent].lerpOrigin;
        let entangle = ctx.world.entities[cent].lerpAngles[YAW];
        let ghoul2 = ctx.world.entities[cent].ghoul2;
        // `modelScale` goes in by pointer in Raven and `CG_AddGhoul2Mark` really
        // does stomp it (its argument-swapped `VectorCopy`), so copy it back out.
        let mut modelScale = ctx.world.entities[cent].modelScale;
        let lifeTime = ctx.world.bg_state.rng.Q_irand(2000, 4000);

        CG_AddGhoul2Mark(
            ctx,
            td.shader,
            td.size,
            &tr.endpos,
            &end,
            tr.entityNum as c_int,
            &lerpOrigin,
            entangle,
            ghoul2,
            &mut modelScale,
            lifeTime,
        );
        ctx.world.entities[cent].modelScale = modelScale;
        //I'm making fx system decals have a very short lifetime.
    }
}

/// Raven `CG_RegisterSounds` — the map-load sound/effect precache: the fixed
/// list Raven spells out inline, then everything the server asked for through
/// `CS_SOUNDS`/`CS_EFFECTS`/`CS_ICONS`.
///
/// Most of the calls throw their handle away — registering is the point, the
/// engine hands the same handle back when the sound is played by name.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1456-1857`
pub fn CG_RegisterSounds(ctx: &mut CgContext) {
    let engine = ctx.engine;

    CG_AS_Register(ctx);

    // CG_LoadingString( "sounds" );

    trap::S_RegisterSound(engine, "sound/weapons/melee/punch1.mp3");
    trap::S_RegisterSound(engine, "sound/weapons/melee/punch2.mp3");
    trap::S_RegisterSound(engine, "sound/weapons/melee/punch3.mp3");
    trap::S_RegisterSound(engine, "sound/weapons/melee/punch4.mp3");
    trap::S_RegisterSound(engine, "sound/movers/objects/saber_slam");

    trap::S_RegisterSound(engine, "sound/player/bodyfall_human1.wav");
    trap::S_RegisterSound(engine, "sound/player/bodyfall_human2.wav");
    trap::S_RegisterSound(engine, "sound/player/bodyfall_human3.wav");

    //test effects
    trap::FX_RegisterEffect(engine, "effects/mp/test_sparks.efx");
    trap::FX_RegisterEffect(engine, "effects/mp/test_wall_impact.efx");

    ctx.world.cgs.media.oneMinuteSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM004");
    ctx.world.cgs.media.fiveMinuteSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM005");
    ctx.world.cgs.media.oneFragSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM001");
    ctx.world.cgs.media.twoFragSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM002");
    ctx.world.cgs.media.threeFragSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM003");
    ctx.world.cgs.media.count3Sound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM035");
    ctx.world.cgs.media.count2Sound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM036");
    ctx.world.cgs.media.count1Sound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM037");
    ctx.world.cgs.media.countFightSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM038");

    ctx.world.cgs.media.hackerIconShader =
        trap::R_RegisterShaderNoMip(engine, "gfx/mp/c_icon_tech");

    ctx.world.cgs.media.redSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/red_glow");
    ctx.world.cgs.media.redSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/red_line");
    ctx.world.cgs.media.orangeSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/orange_glow");
    ctx.world.cgs.media.orangeSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/orange_line");
    ctx.world.cgs.media.yellowSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/yellow_glow");
    ctx.world.cgs.media.yellowSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/yellow_line");
    ctx.world.cgs.media.greenSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/green_glow");
    ctx.world.cgs.media.greenSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/green_line");
    ctx.world.cgs.media.blueSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/blue_glow");
    ctx.world.cgs.media.blueSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/blue_line");
    ctx.world.cgs.media.purpleSaberGlowShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/purple_glow");
    ctx.world.cgs.media.purpleSaberCoreShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/purple_line");
    ctx.world.cgs.media.saberBlurShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/saberBlur");
    ctx.world.cgs.media.swordTrailShader =
        trap::R_RegisterShader(engine, "gfx/effects/sabers/swordTrail");

    ctx.world.cgs.media.forceCoronaShader =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/force_swirl");

    ctx.world.cgs.media.yellowDroppedSaberShader =
        trap::R_RegisterShader(engine, "gfx/effects/yellow_glow");

    ctx.world.cgs.media.rivetMarkShader = trap::R_RegisterShader(engine, "gfx/damage/rivetmark");

    trap::R_RegisterShader(engine, "gfx/effects/saberFlare");

    trap::R_RegisterShader(engine, "powerups/ysalimarishell");

    trap::R_RegisterShader(engine, "gfx/effects/forcePush");

    trap::R_RegisterShader(engine, "gfx/misc/red_dmgshield");
    trap::R_RegisterShader(engine, "gfx/misc/red_portashield");
    trap::R_RegisterShader(engine, "gfx/misc/blue_dmgshield");
    trap::R_RegisterShader(engine, "gfx/misc/blue_portashield");

    trap::R_RegisterShader(engine, "models/map_objects/imp_mine/turret_chair_dmg.tga");

    for i in 1..9 {
        trap::S_RegisterSound(engine, &format!("sound/weapons/saber/saberhup{i}.wav"));
    }

    for i in 1..10 {
        trap::S_RegisterSound(engine, &format!("sound/weapons/saber/saberblock{i}.wav"));
    }

    for i in 1..4 {
        trap::S_RegisterSound(engine, &format!("sound/weapons/saber/bounce{i}.wav"));
    }

    trap::S_RegisterSound(engine, "sound/weapons/saber/enemy_saber_on.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/enemy_saber_off.wav");

    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhum1.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberon.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberoffquick.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhitwall1");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhitwall2");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhitwall3");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhit.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhit1.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhit2.wav");
    trap::S_RegisterSound(engine, "sound/weapons/saber/saberhit3.wav");

    trap::S_RegisterSound(engine, "sound/weapons/saber/saber_catch.wav");

    ctx.world.cgs.media.teamHealSound =
        trap::S_RegisterSound(engine, "sound/weapons/force/teamheal.wav");
    ctx.world.cgs.media.teamRegenSound =
        trap::S_RegisterSound(engine, "sound/weapons/force/teamforce.wav");

    trap::S_RegisterSound(engine, "sound/weapons/force/heal.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/speed.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/see.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/rage.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/lightning");
    trap::S_RegisterSound(engine, "sound/weapons/force/lightninghit1");
    trap::S_RegisterSound(engine, "sound/weapons/force/lightninghit2");
    trap::S_RegisterSound(engine, "sound/weapons/force/lightninghit3");
    trap::S_RegisterSound(engine, "sound/weapons/force/drain.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/jumpbuild.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/distract.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/distractstop.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/pull.wav");
    trap::S_RegisterSound(engine, "sound/weapons/force/push.wav");

    for i in 1..3 {
        trap::S_RegisterSound(engine, &format!("sound/weapons/thermal/bounce{i}.wav"));
    }

    trap::S_RegisterSound(engine, "sound/movers/switches/switch2.wav");
    trap::S_RegisterSound(engine, "sound/movers/switches/switch3.wav");
    trap::S_RegisterSound(engine, "sound/ambience/spark5.wav");
    trap::S_RegisterSound(engine, "sound/chars/turret/ping.wav");
    trap::S_RegisterSound(engine, "sound/chars/turret/startup.wav");
    trap::S_RegisterSound(engine, "sound/chars/turret/shutdown.wav");
    trap::S_RegisterSound(engine, "sound/chars/turret/move.wav");
    trap::S_RegisterSound(engine, "sound/player/pickuphealth.wav");
    trap::S_RegisterSound(engine, "sound/player/pickupshield.wav");

    trap::S_RegisterSound(engine, "sound/effects/glassbreak1.wav");

    trap::S_RegisterSound(engine, "sound/weapons/rocket/tick.wav");
    trap::S_RegisterSound(engine, "sound/weapons/rocket/lock.wav");

    trap::S_RegisterSound(engine, "sound/weapons/force/speedloop.wav");

    trap::S_RegisterSound(engine, "sound/weapons/force/protecthit.mp3"); //PDSOUND_PROTECTHIT
    trap::S_RegisterSound(engine, "sound/weapons/force/protect.mp3"); //PDSOUND_PROTECT
    trap::S_RegisterSound(engine, "sound/weapons/force/absorbhit.mp3"); //PDSOUND_ABSORBHIT
    trap::S_RegisterSound(engine, "sound/weapons/force/absorb.mp3"); //PDSOUND_ABSORB
    trap::S_RegisterSound(engine, "sound/weapons/force/jump.mp3"); //PDSOUND_FORCEJUMP
    trap::S_RegisterSound(engine, "sound/weapons/force/grip.mp3"); //PDSOUND_FORCEGRIP

    if ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0 {
        // #ifdef JK2AWARDS: cgs.media.captureAwardSound — not defined in the MP
        // build, so the retail module never registers it.

        ctx.world.cgs.media.redLeadsSound =
            trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM046");
        ctx.world.cgs.media.blueLeadsSound =
            trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM045");
        ctx.world.cgs.media.teamsTiedSound =
            trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM032");

        ctx.world.cgs.media.redScoredSound =
            trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM044");
        ctx.world.cgs.media.blueScoredSound =
            trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM043");

        if ctx.world.cgs.gametype == GT_CTF || ctx.world.cvars.cg_buildScript.integer != 0 {
            ctx.world.cgs.media.redFlagReturnedSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM042");
            ctx.world.cgs.media.blueFlagReturnedSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM041");
            ctx.world.cgs.media.redTookFlagSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM040");
            ctx.world.cgs.media.blueTookFlagSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM039");
        }
        if ctx.world.cgs.gametype == GT_CTY
        /*|| cg_buildScript.integer*/
        {
            ctx.world.cgs.media.redYsalReturnedSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM050");
            ctx.world.cgs.media.blueYsalReturnedSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM049");
            ctx.world.cgs.media.redTookYsalSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM048");
            ctx.world.cgs.media.blueTookYsalSound =
                trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM047");
        }
    }

    ctx.world.cgs.media.drainSound =
        trap::S_RegisterSound(engine, "sound/weapons/force/drained.mp3");

    ctx.world.cgs.media.happyMusic = trap::S_RegisterSound(engine, "music/goodsmall.mp3");
    ctx.world.cgs.media.dramaticFailure = trap::S_RegisterSound(engine, "music/badsmall.mp3");

    //PRECACHE ALL MUSIC HERE (don't need to precache normally because it's streamed off the disk)
    if ctx.world.cvars.cg_buildScript.integer != 0 {
        trap::S_StartBackgroundTrack(engine, "music/mp/duel.mp3", "music/mp/duel.mp3", false);
    }

    ctx.world.cg.loadLCARSStage = 1;

    ctx.world.cgs.media.selectSound = trap::S_RegisterSound(engine, "sound/weapons/change.wav");

    ctx.world.cgs.media.teleInSound = trap::S_RegisterSound(engine, "sound/player/telein.wav");
    ctx.world.cgs.media.teleOutSound = trap::S_RegisterSound(engine, "sound/player/teleout.wav");
    ctx.world.cgs.media.respawnSound = trap::S_RegisterSound(engine, "sound/items/respawn1.wav");

    trap::S_RegisterSound(engine, "sound/movers/objects/objectHit.wav");

    ctx.world.cgs.media.talkSound = trap::S_RegisterSound(engine, "sound/player/talk.wav");
    ctx.world.cgs.media.landSound = trap::S_RegisterSound(engine, "sound/player/land1.wav");
    ctx.world.cgs.media.fallSound = trap::S_RegisterSound(engine, "sound/player/fallsplat.wav");

    ctx.world.cgs.media.crackleSound =
        trap::S_RegisterSound(engine, "sound/effects/energy_crackle.wav");
    // #ifdef JK2AWARDS: impressiveSound/excellentSound/deniedSound/
    // humiliationSound/defendSound — not defined in the MP build.

    /*
    cgs.media.takenLeadSound = trap_S_RegisterSound( "sound/chars/protocol/misc/40MOM051");
    cgs.media.tiedLeadSound = trap_S_RegisterSound( "sound/chars/protocol/misc/40MOM032");
    cgs.media.lostLeadSound = trap_S_RegisterSound( "sound/chars/protocol/misc/40MOM052");
    */

    ctx.world.cgs.media.rollSound = trap::S_RegisterSound(engine, "sound/player/roll1.wav");

    ctx.world.cgs.media.noforceSound = trap::S_RegisterSound(engine, "sound/weapons/force/noforce");

    ctx.world.cgs.media.watrInSound = trap::S_RegisterSound(engine, "sound/player/watr_in.wav");
    ctx.world.cgs.media.watrOutSound = trap::S_RegisterSound(engine, "sound/player/watr_out.wav");
    ctx.world.cgs.media.watrUnSound = trap::S_RegisterSound(engine, "sound/player/watr_un.wav");

    ctx.world.cgs.media.explosionModel =
        trap::R_RegisterModel(engine, "models/map_objects/mp/sphere.md3");
    ctx.world.cgs.media.surfaceExplosionShader = trap::R_RegisterShader(engine, "surfaceExplosion");

    ctx.world.cgs.media.disruptorShader = trap::R_RegisterShader(engine, "gfx/effects/burn");

    if ctx.world.cvars.cg_buildScript.integer != 0 {
        trap::R_RegisterShader(engine, "gfx/effects/turretflashdie");
    }

    ctx.world.cgs.media.solidWhite = trap::R_RegisterShader(engine, "gfx/effects/solidWhite_cull");

    trap::R_RegisterShader(engine, "gfx/misc/mp_light_enlight_disable");
    trap::R_RegisterShader(engine, "gfx/misc/mp_dark_enlight_disable");

    trap::R_RegisterModel(engine, "models/map_objects/mp/sphere.md3");
    trap::R_RegisterModel(engine, "models/items/remote.md3");

    ctx.world.cgs.media.holocronPickup = trap::S_RegisterSound(engine, "sound/player/holocron.wav");

    // Zoom
    ctx.world.cgs.media.zoomStart = trap::S_RegisterSound(engine, "sound/interface/zoomstart.wav");
    ctx.world.cgs.media.zoomLoop = trap::S_RegisterSound(engine, "sound/interface/zoomloop.wav");
    ctx.world.cgs.media.zoomEnd = trap::S_RegisterSound(engine, "sound/interface/zoomend.wav");

    for i in 0..4 {
        // Raven builds each name with `Com_sprintf` into a `char[MAX_QPATH]`;
        // every one of these is far shorter than the cap.
        let footsteps = &mut ctx.world.cgs.media.footsteps;

        let name = format!("sound/player/footsteps/stone_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_STONEWALK as usize][i] =
            trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/stone_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_STONERUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/metal_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_METALWALK as usize][i] =
            trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/metal_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_METALRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/pipe_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_PIPEWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/pipe_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_PIPERUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/water_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SPLASH as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/water_walk{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_WADE as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/water_wade_0{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SWIM as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/snow_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SNOWWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/snow_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SNOWRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/sand_walk{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SANDWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/sand_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_SANDRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/grass_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_GRASSWALK as usize][i] =
            trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/grass_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_GRASSRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/dirt_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_DIRTWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/dirt_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_DIRTRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/mud_walk{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_MUDWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/mud_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_MUDRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/gravel_walk{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_GRAVELWALK as usize][i] =
            trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/gravel_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_GRAVELRUN as usize][i] =
            trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/rug_step{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_RUGWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/rug_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_RUGRUN as usize][i] = trap::S_RegisterSound(engine, &name);

        let name = format!("sound/player/footsteps/wood_walk{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_WOODWALK as usize][i] = trap::S_RegisterSound(engine, &name);
        let name = format!("sound/player/footsteps/wood_run{}.wav", i + 1);
        footsteps[footstep_t::FOOTSTEP_WOODRUN as usize][i] = trap::S_RegisterSound(engine, &name);
    }

    // only register the items that the server says we need
    let items = CG_ConfigString(ctx, CS_ITEMS);
    let itemBytes = items.as_bytes();

    for i in 1..bg_numItems {
        // §F19: Raven reads `char items[MAX_ITEMS+1]` past the copied string's
        // terminator when the configstring is short; a byte that isn't there is
        // "not requested" here (same read [`CG_RegisterGraphics`] takes).
        if itemBytes.get(i as usize) == Some(&b'1') || ctx.world.cvars.cg_buildScript.integer != 0 {
            CG_RegisterItemSounds(ctx, i);
        }
    }

    for i in 1..MAX_SOUNDS {
        let soundName = CG_ConfigString(ctx, CS_SOUNDS + i);
        if soundName.is_empty() {
            break;
        }
        if soundName.as_bytes()[0] == b'*' {
            if soundName.as_bytes().get(1) == Some(&b'$') {
                //an NPC soundset
                CG_PrecacheNPCSounds(ctx, &soundName);
            }
            continue; // custom sound
        }
        ctx.world.cgs.gameSounds[i as usize] = trap::S_RegisterSound(engine, &soundName);
    }

    for i in 1..MAX_FX {
        let soundName = CG_ConfigString(ctx, CS_EFFECTS + i);
        if soundName.is_empty() {
            break;
        }

        if soundName.as_bytes()[0] == b'*' {
            //it's a special global weather effect
            CG_ParseWeatherEffect(ctx, &soundName);
            ctx.world.cgs.gameEffects[i as usize] = 0;
        } else {
            ctx.world.cgs.gameEffects[i as usize] = trap::FX_RegisterEffect(engine, &soundName);
        }
    }

    // register all the server specified icons
    for i in 1..MAX_ICONS {
        let iconName = CG_ConfigString(ctx, CS_ICONS + i);
        if iconName.is_empty() {
            break;
        }

        ctx.world.cgs.gameIcons[i as usize] = trap::R_RegisterShaderNoMip(engine, &iconName);
    }

    let soundName = CG_ConfigString(ctx, CS_SIEGE_STATE);

    if !soundName.is_empty() {
        CG_ParseSiegeState(ctx.world, &soundName);
    }

    let soundName = CG_ConfigString(ctx, CS_SIEGE_WINTEAM);

    if !soundName.is_empty() {
        ctx.world.scoreboard.cg_siegeWinTeam = atoi(&soundName);
    }

    if ctx.world.cgs.gametype == GT_SIEGE {
        let objectives = CG_ConfigString(ctx, CS_SIEGE_OBJECTIVES);
        CG_ParseSiegeObjectiveStatus(ctx, &objectives);
        let timeOverride = CG_ConfigString(ctx, CS_SIEGE_TIMEOVERRIDE);
        ctx.world.draw.cg_beatingSiegeTime = atoi(&timeOverride);
        if ctx.world.draw.cg_beatingSiegeTime != 0 {
            let msec = ctx.world.draw.cg_beatingSiegeTime;
            CG_SetSiegeTimerCvar(ctx, msec);
        }
    }

    ctx.world.cg.loadLCARSStage = 2;

    // FIXME: only needed with item
    ctx.world.cgs.media.deploySeeker =
        trap::S_RegisterSound(engine, "sound/chars/seeker/misc/hiss");
    ctx.world.cgs.media.medkitSound = trap::S_RegisterSound(engine, "sound/items/use_bacta.wav");

    ctx.world.cgs.media.winnerSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM006");
    ctx.world.cgs.media.loserSound =
        trap::S_RegisterSound(engine, "sound/chars/protocol/misc/40MOM010");
}

/// Raven `CG_LoadMenus` — reads the hud-menu list file and hands every
/// `loadmenu { … }` block to [`CG_Load_Menu`].
///
/// A missing list file falls back to `ui/jahud.txt`; if that is missing too
/// Raven prints and reads on with the handle still 0, which is kept below.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3076-3137`
pub fn CG_LoadMenus(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &mut DisplayState,
    menuFile: &str,
) {
    // Raven's `static char buf[MAX_MENUDEFFILE]` is read-then-parse scratch that
    // never outlives the call, so it stays a local (the cap survives as the
    // length guard below).
    let mut f: fileHandle_t = 0;
    let mut len = trap::FS_FOpenFile(ctx.engine, menuFile, &mut f, FS_READ);

    if f == 0 {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file not found: {}, using default\n",
                S_COLOR_RED.to_str().unwrap(),
                menuFile
            ),
        );

        len = trap::FS_FOpenFile(ctx.engine, "ui/jahud.txt", &mut f, FS_READ);
        if f == 0 {
            // Raven hands `menuFile` to a `va()` format with no conversion in
            // it, so the name never reaches the output - kept as it prints.
            trap::Print(
                ctx.engine,
                &format!(
                    "{}default menu file not found: ui/hud.txt, unable to continue!\n",
                    S_COLOR_RED.to_str().unwrap()
                ),
            );
        }
    }

    if len >= MAX_MENUDEFFILE as c_int {
        trap::Print(
            ctx.engine,
            &format!(
                "{}menu file too large: {} is {}, max allowed is {}",
                S_COLOR_RED.to_str().unwrap(),
                menuFile,
                len,
                MAX_MENUDEFFILE
            ),
        );
        trap::FS_FCloseFile(ctx.engine, f);
        return;
    }

    // §F19: a failed second open leaves `len` at the trap's error value and
    // Raven still reads with it and writes `buf[len]`; a negative length reads
    // nothing here. `buf[len] = 0` is the slice end.
    let mut buf = vec![0u8; len.max(0) as usize];
    trap::FS_Read(ctx.engine, &mut buf, f);
    trap::FS_FCloseFile(ctx.engine, f);

    let text = latin1_to_string(&buf);
    let mut p: &str = &text;

    loop {
        // Raven's `COM_ParseExt(&p, qtrue)`; the ported `COM_Parse` already
        // carries the `allowLineBreaks` flag.
        let (token, rest) = COM_Parse(p, true);
        p = rest;
        if token.is_empty() || token.starts_with('}') {
            break;
        }

        if Q_stricmp(&token, "}") == 0 {
            break;
        }

        if Q_stricmp(&token, "loadmenu") == 0 {
            if CG_Load_Menu(ctx, menus, ds, &mut p) {
                continue;
            } else {
                break;
            }
        }
    }

    //Com_Printf("UI menu load time = %d milli seconds\n", cgi_Milliseconds() - start);
}

/// Raven `CG_NewString` — interns a spawn-var value in the cgame string pool,
/// turning `\n` into a real linefeed on the way.
///
/// A backslash in front of anything else eats the escaped character and leaves
/// the backslash - Raven's quirk, kept. The pool buffer is gone (see
/// [`CG_StrPool_Alloc`]), so this hands back the owned string.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3344-3370`
pub fn CG_NewString(ctx: &mut CgContext, string: &str) -> String {
    let mut src = string_to_latin1(string);
    // `l = strlen(string) + 1` - the copy loop walks the terminator too.
    src.push(0);
    let l = src.len();

    let mut newb = CG_StrPool_Alloc(ctx, l as c_int);

    let mut new_p = 0;

    // turn \n into a real linefeed
    let mut i = 0;
    while i < l {
        if src[i] == b'\\' && i < l - 1 {
            i += 1;
            if src[i] == b'n' {
                newb[new_p] = b'\n';
            } else {
                newb[new_p] = b'\\';
            }
            new_p += 1;
        } else {
            newb[new_p] = src[i];
            new_p += 1;
        }
        i += 1;
    }

    // Raven returns the pool pointer and the reader stops at the NUL the loop
    // copied (or at the zeroed tail when a trailing backslash ate it).
    let end = newb.iter().position(|&c| c == 0).unwrap_or(newb.len());
    latin1_to_string(&newb[..end])
}

/// Raven `cg_spawnFields[]` + `BG_ParseField`, collapsed to one key dispatch
/// over [`CgSpawnEnt`].
///
/// The table's rows are `CGFOFS` byte offsets into a `cgSpawnEnt_t`, and the
/// port's record is idiomatic (`char *` → `String`, no `#[repr(C)]`), so there
/// is no offset to hand `BG_ParseField`. The 13 rows land as the arms below in
/// table order, each decoding exactly what its `fieldtype_t` decodes; a key
/// that matches nothing is dropped, same as falling off the table scan.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3392-3408`;
/// `oracle/codemp/game/bg_misc.c:358-423`
fn CG_ParseSpawnField(ctx: &mut CgContext, ent: &mut CgSpawnEnt, key: &str, value: &str) {
    if Q_stricmp(key, "classname") == 0 {
        ent.classname = CG_NewString(ctx, value);
    } else if Q_stricmp(key, "origin") == 0 {
        spawn_vector(value, &mut ent.origin);
    } else if Q_stricmp(key, "angles") == 0 {
        spawn_vector(value, &mut ent.angles);
    } else if Q_stricmp(key, "angle") == 0 {
        ent.angle = atof(value) as f32;
    } else if Q_stricmp(key, "modelscale") == 0 {
        ent.fScale = atof(value) as f32;
    } else if Q_stricmp(key, "modelscale_vec") == 0 {
        spawn_vector(value, &mut ent.scale);
    } else if Q_stricmp(key, "model") == 0 {
        ent.model = CG_NewString(ctx, value);
    } else if Q_stricmp(key, "mins") == 0 {
        spawn_vector(value, &mut ent.mins);
    } else if Q_stricmp(key, "maxs") == 0 {
        spawn_vector(value, &mut ent.maxs);
    } else if Q_stricmp(key, "zoffset") == 0 {
        ent.zoffset = atof(value) as f32;
    } else if Q_stricmp(key, "onlyfoghere") == 0 {
        ent.onlyFogHere = atoi(value);
    } else if Q_stricmp(key, "fogstart") == 0 {
        ent.fogstart = atof(value) as f32;
    } else if Q_stricmp(key, "radarrange") == 0 {
        ent.radarrange = atof(value) as f32;
    }
}

/// `BG_ParseField`'s `F_VECTOR` case.
///
/// `sscanf(value, "%f %f %f", …)` has no count check, so an unmatched component
/// keeps the 0.0 seed (§F19, the same read `BG_ParseField` itself took).
///
/// Source: `oracle/codemp/game/bg_misc.c:378-384`
fn spawn_vector(value: &str, out: &mut vec3_t) {
    *out = [0.0; 3];
    sscanf_f32s(value, out);
}

/// Raven `CG_SpawnCGameEntFromVars` — turns one parsed map entity into whatever
/// cgame keeps of it: worldspawn fog/radar overrides, static models, sky-portal
/// points and weather zones. Everything else on the map is the game's problem.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3608-3654`
pub fn CG_SpawnCGameEntFromVars(ctx: &mut CgContext) {
    let mut ent = CgSpawnEnt::default();

    for i in 0..ctx.world.main.cg_spawnVars.len() {
        //shove all this stuff into our data structure used specifically for getting spawn info
        let [key, value] = ctx.world.main.cg_spawnVars[i].clone();
        CG_ParseSpawnField(ctx, &mut ent, &key, &value);
    }

    // Raven's `ent.classname && ent.classname[0]` — the owned `String` folds the
    // null test and the empty test into one.
    if !ent.classname.is_empty() {
        //we'll just stricmp this bastard, since there aren't all that many cgame-only things, and they all have special handling
        if Q_stricmp(&ent.classname, "worldspawn") == 0 {
            //I'd like some info off this guy
            if ent.fogstart != 0.0 {
                //linear fog method
                ctx.world.view.cg_linearFogOverride = ent.fogstart;
            }
            //get radarRange off of worldspawn
            if ent.radarrange != 0.0 {
                //linear fog method
                ctx.world.draw.cg_radarRange = ent.radarrange;
            }
        } else if Q_stricmp(&ent.classname, "misc_model_static") == 0 {
            //we've got us a static model
            CG_CreateModelFromSpawnEnt(ctx, &mut ent);
        } else if Q_stricmp(&ent.classname, "misc_skyportal_orient") == 0 {
            //a sky portal orientation point
            CG_CreateSkyOriFromSpawnEnt(ctx.world, &ent);
        } else if Q_stricmp(&ent.classname, "misc_skyportal") == 0 {
            //might as well parse this thing cgame side for the extra info I want out of it
            CG_CreateSkyPortalFromSpawnEnt(ctx.world, &ent);
        } else if Q_stricmp(&ent.classname, "misc_weather_zone") == 0 {
            //might as well parse this thing cgame side for the extra info I want out of it
            CG_CreateWeatherZoneFromSpawnEnt(ctx, &mut ent);
        }
    }

    //reset the string pool for the next entity, if there is one
    CG_StrPool_Reset(ctx.world);
}

/// Raven `CG_Shutdown` — the `CG_SHUTDOWN` vmMain arm: drop every ghoul2
/// instance, tear the FX/ROFF systems down and put the weather back.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3993-4016`
pub fn CG_Shutdown(ctx: &mut CgContext, menus: &mut MenuSystem) {
    BG_ClearAnimsets(); //free all dynamic allocations made through the engine

    CG_DestroyAllGhoul2(ctx);

    //	Com_Printf("... FX System Cleanup\n");
    trap::FX_FreeSystem(ctx.engine);
    trap::ROFF_Clean(ctx.engine);

    if ctx.world.main.cgWeatherOverride != 0 {
        trap::R_WeatherContentsOverride(ctx.engine, 0); //rwwRMG - reset it engine-side
    }

    //reset weather
    trap::R_WorldEffectCommand(ctx.engine, "die");

    // ctx is the DisplayContext (DEC-47.1, the DEC-38 shape applied to cgame).
    UI_CleanupGhoul2(menus, ctx);
    //If there was any ghoul2 stuff in our side of the shared ui code, then remove it now.

    // some mods may need to do cleanup work here,
    // like closing files or archiving session data
}

/// Raven `CG_DebugBoxLines` — draws the 12 edges of an axis-aligned box as
/// debug lines, all in the same blue.
///
/// Source: `oracle/codemp/cgame/cg_main.c:445-505`
pub fn CG_DebugBoxLines(world: &mut CgWorld, mins: vec3_t, maxs: vec3_t, duration: c_int) {
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut vert: vec3_t = [0.0; 3];

    let x = maxs[0] - mins[0];
    let y = maxs[1] - mins[1];

    start[2] = maxs[2];
    vert[2] = mins[2];

    vert[0] = mins[0];
    vert[1] = mins[1];
    start[0] = vert[0];
    start[1] = vert[1];
    CG_TestLine(world, &start, &vert, duration, 0x00000ff, 1);

    vert[0] = mins[0];
    vert[1] = maxs[1];
    start[0] = vert[0];
    start[1] = vert[1];
    CG_TestLine(world, &start, &vert, duration, 0x00000ff, 1);

    vert[0] = maxs[0];
    vert[1] = mins[1];
    start[0] = vert[0];
    start[1] = vert[1];
    CG_TestLine(world, &start, &vert, duration, 0x00000ff, 1);

    vert[0] = maxs[0];
    vert[1] = maxs[1];
    start[0] = vert[0];
    start[1] = vert[1];
    CG_TestLine(world, &start, &vert, duration, 0x00000ff, 1);

    // top of box
    _VectorCopy(maxs, &mut start);
    _VectorCopy(maxs, &mut end);
    start[0] -= x;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    end[0] = start[0];
    end[1] -= y;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    start[1] = end[1];
    start[0] += x;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    CG_TestLine(world, &start, &maxs, duration, 0x00000ff, 1);
    // bottom of box
    _VectorCopy(mins, &mut start);
    _VectorCopy(mins, &mut end);
    start[0] += x;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    end[0] = start[0];
    end[1] += y;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    start[1] = end[1];
    start[0] -= x;
    CG_TestLine(world, &start, &end, duration, 0x00000ff, 1);
    CG_TestLine(world, &start, &mins, duration, 0x00000ff, 1);
}

/// Raven `C_ImpactMark` — the `CG_IMPACT_MARK` vmMain arm: decode the shared
/// buffer and forward straight into [`CG_ImpactMark`], always permanent
/// (`alphaFade` true, `temporary` false — the two flags Raven hardcodes at the
/// call site, absent from the wire payload).
///
/// Source: `oracle/codemp/cgame/cg_main.c:570-580`
pub fn C_ImpactMark(ctx: &mut CgContext, data: &TCGImpactMark) {
    CG_ImpactMark(
        ctx,
        data.mHandle,
        data.mPoint,
        data.mAngle,
        data.mRotation,
        data.mRed,
        data.mGreen,
        data.mBlue,
        data.mAlphaStart,
        true,
        data.mSizeStart,
        false,
    );
}

/// Raven `CG_LoadHudMenu` — assembles the `cgDC` display vtable and loads the
/// hud menu set named by `cg_hudFiles` (`ui/jahud.txt` when unset).
///
/// PORT-NOTE: the `cgDC.<slot> = &Xxx` assignment block (`cg_main.c:3149-3205`)
/// and the `Init_Display(&cgDC)` call right after it are dropped — DEC-36 D3
/// replaces the vtable with the `DisplayContext` trait `impl DisplayContext
/// for CgContext` already provides
/// (`crates/mp/cgame/src/world/cg_display_context.rs`), threaded per-call
/// instead of stored on a file-scope `DC` pointer; there is no `cgDC` field
/// left for either to assign, matching `Init_Display`'s own DEFERRED note
/// (`crates/mp/uishared/src/ui_shared.rs:421-428`) and its `_UI_Init` twin
/// (`crates/mp/ui/src/ui_main.rs:13299-13309`).
///
/// Source: `oracle/codemp/cgame/cg_main.c:3145-3219`
pub fn CG_LoadHudMenu(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &mut DisplayState) {
    Menu_Reset(menus);

    let hudSet = buf_to_string(&ctx.world.cvars.cg_hudFiles.string.map(|c| c as u8));
    let hudSet = if hudSet.is_empty() {
        "ui/jahud.txt".to_string()
    } else {
        hudSet
    };

    CG_LoadMenus(ctx, menus, ds, &hudSet);
}

/// Raven `CG_SpawnCGameOnlyEnts` — parses the BSP entity string a second time
/// cgame-side, looking for the client-only spawn classes `CG_SpawnCGameEntFromVars`
/// cares about (sky portals, weather zones, cgame-only static models, ...).
///
/// PORT-NOTE: Raven's reset call `trap_GetEntityToken(NULL, -1)` passes a NULL
/// buffer/`-1` length to rewind the engine's parse cursor with no token copy;
/// the ported `trap::GetEntityToken` wrapper always allocates and passes a
/// real buffer, so there is no zero-copy NULL-pointer shape to call through -
/// a zero-length buffer is the closest reachable stand-in and the return value
/// is discarded either way (Raven's own comment: "make sure it is reset").
///
/// Source: `oracle/codemp/cgame/cg_main.c:3664-3682`
pub fn CG_SpawnCGameOnlyEnts(ctx: &mut CgContext) {
    //make sure it is reset
    trap::GetEntityToken(ctx.engine, 0);

    if !CG_ParseSpawnVars(ctx) {
        //first one is gonna be the world spawn
        CG_Error(ctx, "no entities for cgame parse");
        return;
    } else {
        //parse the world spawn info we want
        CG_SpawnCGameEntFromVars(ctx);
    }

    //now run through the whole list, and look for things we care about cgame-side
    while CG_ParseSpawnVars(ctx) {
        CG_SpawnCGameEntFromVars(ctx);
    }
}

/// Raven `#define RAG_CALLBACK_DEBUGBOX 1` — [`CG_RagCallback`]'s ragdoll
/// debug-box case selector.
/// Source: `oracle/codemp/cgame/cg_public.h:541`
const RAG_CALLBACK_DEBUGBOX: c_int = 1;

/// Raven `#define RAG_CALLBACK_DEBUGLINE 2`.
/// Source: `oracle/codemp/cgame/cg_public.h:550`
const RAG_CALLBACK_DEBUGLINE: c_int = 2;

/// Raven `#define RAG_CALLBACK_BONESNAP 3`.
/// Source: `oracle/codemp/cgame/cg_public.h:560`
const RAG_CALLBACK_BONESNAP: c_int = 3;

/// Raven `#define RAG_CALLBACK_BONEIMPACT 4`.
/// Source: `oracle/codemp/cgame/cg_public.h:571`
const RAG_CALLBACK_BONEIMPACT: c_int = 4;

/// Raven `#define RAG_CALLBACK_BONEINSOLID 5`.
/// Source: `oracle/codemp/cgame/cg_public.h:576`
const RAG_CALLBACK_BONEINSOLID: c_int = 5;

/// Raven `#define RAG_CALLBACK_TRACELINE 6`.
/// Source: `oracle/codemp/cgame/cg_public.h:582`
const RAG_CALLBACK_TRACELINE: c_int = 6;

/// Raven `CG_RagCallback` — the `CG_RAG_CALLBACK` vmcall body ghoul2's ragdoll
/// solver invokes for debug draws and ragdoll-triggered gameplay events (bone
/// snap sound, trace queries for the solver).
///
/// `callType` is the only vmMain argument (see `CgRagCallbackArgs`); the
/// per-case payload rides `cg.sharedBuffer` itself, cast to a different struct
/// per `callType` - unlike the single-shape `C_*` vmcalls this module already
/// ports, the shape here is only known once we're inside the switch, so the
/// decode happens at this fn (this call *is* the DEC-46.6 boundary for this
/// vmcall) rather than at a shared dispatch site.
///
/// `RAG_CALLBACK_BONESNAP` falls through into `RAG_CALLBACK_BONEIMPACT` in
/// Raven (no `break`); the impact arm is empty so the fallthrough has no
/// observable effect beyond what the snap arm already does.
/// `RAG_CALLBACK_BONEINSOLID`'s body is `#if 0`'d out in Raven - dead code,
/// so the port does nothing there too.
///
/// Source: `oracle/codemp/cgame/cg_main.c:508-568`
pub fn CG_RagCallback(ctx: &mut CgContext, callType: c_int) -> c_int {
    match callType {
        RAG_CALLBACK_DEBUGBOX => {
            // SAFETY: the engine writes a `ragCallbackDebugBox_t` into
            // `cg.sharedBuffer` before invoking this callback with
            // callType == RAG_CALLBACK_DEBUGBOX; `read_unaligned` copies it
            // out without forming a reference into the byte buffer (whose
            // alignment is 1).
            let callData = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const ragCallbackDebugBox_t).read_unaligned()
            };
            let (mins, maxs, duration) = (callData.mins, callData.maxs, callData.duration);

            CG_DebugBoxLines(ctx.world, mins, maxs, duration);
        }
        RAG_CALLBACK_DEBUGLINE => {
            // SAFETY: same shared-buffer contract as the debug-box arm above,
            // typed `ragCallbackDebugLine_t` for this callType; unaligned copy,
            // no reference formed.
            let callData = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const ragCallbackDebugLine_t).read_unaligned()
            };
            let (start, end, time, color, radius) = (
                callData.start,
                callData.end,
                callData.time,
                callData.color as c_uint,
                callData.radius,
            );

            CG_TestLine(ctx.world, &start, &end, time, color, radius);
        }
        RAG_CALLBACK_BONESNAP => {
            // SAFETY: same shared-buffer contract, typed `ragCallbackBoneSnap_t`
            // for this callType; unaligned copy, no reference formed.
            let callData = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const ragCallbackBoneSnap_t).read_unaligned()
            };
            let entNum = callData.entNum;
            let lerpOrigin = ctx.world.entity(entNum as usize).lerpOrigin;
            let roll = ctx.world.bg_state.rng.Q_irand(1, 3);
            let sample = format!("sound/player/bodyfall_human{roll}.wav");
            let snapSound = trap::S_RegisterSound(ctx.engine, &sample);

            trap::S_StartSound(ctx.engine, Some(&lerpOrigin), entNum, CHAN_AUTO, snapSound);
            // falls through to RAG_CALLBACK_BONEIMPACT in Raven, which is a
            // no-op arm - nothing further to do
        }
        RAG_CALLBACK_BONEIMPACT => {}
        RAG_CALLBACK_BONEINSOLID => {
            // Raven's body here is `#if 0`'d out - dead code, no-op.
        }
        RAG_CALLBACK_TRACELINE => {
            // SAFETY: same shared-buffer contract, typed
            // `ragCallbackTraceLine_t` for this callType; unaligned copy, no
            // reference formed.
            let callData = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const ragCallbackTraceLine_t).read_unaligned()
            };
            let (start, end, mins, maxs, ignore, mask) = (
                callData.start,
                callData.end,
                callData.mins,
                callData.maxs,
                callData.ignore,
                callData.mask,
            );
            let mut tr = callData.tr;

            CG_Trace(ctx, &mut tr, &start, &mins, &maxs, &end, ignore, mask);

            // Raven traces straight into `callData->tr` and the engine reads
            // the result back out of shared memory (G2_bones.cpp:2690-2701) -
            // write the filled trace back where it looks.
            // SAFETY: same buffer contract; `addr_of_mut!` projects the field
            // without a reference, `write_unaligned` tolerates the byte
            // buffer's alignment.
            unsafe {
                let base = ctx.world.shared_buffer.as_mut_ptr() as *mut ragCallbackTraceLine_t;
                core::ptr::addr_of_mut!((*base).tr).write_unaligned(tr);
            }
        }
        _ => {
            Com_Error(
                ctx,
                errorParm_t::ERR_DROP as c_int,
                "Invalid callType in CG_RagCallback",
            );
            return 0;
        }
    }

    0
}

/// Raven `CG_ForceModelChange` — reloads every present client's ghoul2 model
/// off its `CS_PLAYERS` configstring, forcing a rebuild.
///
/// Raven stashed `oldGhoul2` from `cgs.clientinfo[i].ghoul2Model` but never
/// read it back before `CG_NewClientInfo` overwrote the slot - dead store,
/// dropped.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1135-1150`
pub fn CG_ForceModelChange(ctx: &mut CgContext) {
    for i in 0..MAX_CLIENTS {
        let clientInfo = CG_ConfigString(ctx, CS_PLAYERS + i as c_int);
        if clientInfo.is_empty() {
            continue;
        }

        CG_NewClientInfo(ctx, i as c_int, true);
    }
}

/// Raven `CG_RegisterClients` — loads the local client first, then every
/// other present client, then rebuilds the spectator-follow string.
///
/// Source: `oracle/codemp/cgame/cg_main.c:2505-2526`
pub fn CG_RegisterClients(ctx: &mut CgContext) {
    let clientNum = ctx.world.cg.clientNum;

    CG_LoadingClient(ctx, clientNum);
    CG_NewClientInfo(ctx, clientNum, false);

    for i in 0..MAX_CLIENTS_I32 {
        if clientNum == i {
            continue;
        }

        let clientInfo = CG_ConfigString(ctx, CS_PLAYERS + i);
        if clientInfo.is_empty() {
            continue;
        }
        CG_LoadingClient(ctx, i);
        CG_NewClientInfo(ctx, i, false);
    }

    CG_BuildSpectatorString(ctx);
}

/// Raven `CG_UpdateCvars` — refreshes every registered cvar's local mirror
/// from the engine, then handles the two side effects that fire when a
/// tracked cvar's `modificationCount` moved since the last call.
///
/// PORT-NOTE: `cvarTable`'s literal row order (`cg_main.c:882-1053`) is not in
/// this packet; `CG_RegisterCvars` already walks `CgCvars`'s declaration order
/// in its place (its own PORT-NOTE, `cg_main.c:1062-1112`) with the same
/// reasoning — each `trap_Cvar_Update` call is independent of the others, so
/// the reorder is behaviorally inert. This walk matches that one call-for-call.
///
/// Source: `oracle/codemp/cgame/cg_main.c:1157-1187`
pub fn CG_UpdateCvars(ctx: &mut CgContext) {
    let engine = ctx.engine;

    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_centertime);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_runpitch);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_runroll);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_bobup);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_bobpitch);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_bobroll);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_shadows);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_renderToTextureFX);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawTimer);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawFPS);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawSnapshot);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_draw3dIcons);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawIcons);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawAmmoWarning);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawCrosshair);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawCrosshairNames);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawRadar);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawVehLeadIndicator);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_dynamicCrosshair);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_dynamicCrosshairPrecision);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawRewards);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawScores);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_crosshairSize);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_crosshairX);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_crosshairY);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_crosshairHealth);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_draw2D);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawStatus);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_animSpeed);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_debugAnim);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_debugSaber);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_debugPosition);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_debugEvents);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_errorDecay);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_nopredict);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_noPlayerAnims);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_showmiss);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_showVehMiss);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_footsteps);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_addMarks);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_viewsize);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawGun);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_gun_x);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_gun_y);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_gun_z);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoswitch);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_ignore);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_simpleItems);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_fov);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_zoomFov);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_swingAngles);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_oldPainSounds);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_ragDoll);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_jumpSounds);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoMap);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoMapX);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoMapY);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoMapW);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_autoMapH);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.bg_fighterAltControl);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_chatBox);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_chatBoxHeight);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_saberModelTraceEffect);
    trap::Cvar_Update(
        engine,
        &mut ctx.world.cvars.cg_saberClientVisualCompensation,
    );
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_g2TraceLod);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_fpls);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_ghoul2Marks);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_optvehtrace);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_saberDynamicMarks);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_saberDynamicMarkTime);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_saberContact);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_saberTrail);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_duelHeadAngles);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_speedTrail);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_auraShell);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_repeaterOrb);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_animBlend);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_dismember);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonSpecialCam);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPerson);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonRange);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonAngle);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonPitchOffset);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonVertOffset);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonCameraDamp);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonTargetDamp);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonAlpha);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_thirdPersonHorzOffset);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_stereoSeparation);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_lagometer);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawEnemyInfo);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_synchronousClients);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_stats);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_buildScript);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_forceModel);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_paused);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_blood);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_predictItems);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_deferPlayers);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawTeamOverlay);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_teamOverlayUserinfo);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_drawFriend);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_teamChatsOnly);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_hudFiles);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_scorePlum);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_smoothClients);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.pmove_fixed);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.pmove_msec);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_cameraMode);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_cameraOrbit);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_cameraOrbitDelay);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_timescaleFadeEnd);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_timescaleFadeSpeed);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_timescale);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_noTaunt);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_noProjectileTrail);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_debugBB);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_currentSelectedPlayer);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_currentSelectedPlayerName);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_recordSPDemo);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_recordSPDemoName);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_showVehBounds);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.ui_myteam);
    trap::Cvar_Update(engine, &mut ctx.world.cvars.cg_snapshotTimeout);

    // check for modications here

    // If team overlay is on, ask for updates from the server.  If its off,
    // let the server know so we don't receive it
    if ctx.world.main.drawTeamOverlayModificationCount
        != ctx.world.cvars.cg_drawTeamOverlay.modificationCount
    {
        ctx.world.main.drawTeamOverlayModificationCount =
            ctx.world.cvars.cg_drawTeamOverlay.modificationCount;

        if ctx.world.cvars.cg_drawTeamOverlay.integer > 0 {
            trap::Cvar_Set(engine, "teamoverlay", "1");
        } else {
            trap::Cvar_Set(engine, "teamoverlay", "0");
        }
        // FIXME E3 HACK
        trap::Cvar_Set(engine, "teamoverlay", "1");
    }

    // if force model changed
    if ctx.world.main.forceModelModificationCount != ctx.world.cvars.cg_forceModel.modificationCount
    {
        ctx.world.main.forceModelModificationCount =
            ctx.world.cvars.cg_forceModel.modificationCount;
        CG_ForceModelChange(ctx);
    }
}

/// Raven `CG_Init` — the module's one-time load/reload entry point: resets
/// bg/pmove/menu state, registers cvars/console commands, pulls the engine's
/// glconfig/gamestate/map, validates the client/server build against
/// `GAME_VERSION`, then registers every sound/graphic/client asset and spawns
/// the local-only entity set.
///
/// PORT-NOTE: `item` (Raven `static gitem_t *item`) is assigned then read back
/// inline within the same loop iteration and never observed across calls —
/// unlike `forceModelModificationCount`/`drawTeamOverlayModificationCount`
/// (`CG_UpdateCvars`), it does not fold into `CgMainState`; a local binding is
/// behaviorally identical.
///
/// Source: `oracle/codemp/cgame/cg_main.c:3704-3933`
pub fn CG_Init(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &mut DisplayState,
    serverMessageNum: c_int,
    serverCommandSequence: c_int,
    clientNum: c_int,
) {
    let engine = ctx.engine;

    // clear it out
    {
        let traps = CgBgTraps::new(engine, ctx.world_raw());
        let mut callbacks = CgGameCallbacks::new(engine, ctx.world_raw());
        let mut pmctx = PmoveContext::new(&mut ctx.world.bg_state, &traps, &mut callbacks);
        pmctx.BG_InitAnimsets();
    }

    trap::CG_RegisterSharedMemory(engine, &mut ctx.world.shared_buffer);

    // Load external vehicle data
    {
        let traps = CgBgTraps::new(engine, ctx.world_raw());
        BG_VehicleLoadParms(&mut ctx.world.bg_state, &traps);
    }

    // clear everything
    /*
    Ghoul2 Insert Start
    */

    // memset( cg_entities, 0, sizeof( cg_entities ) );
    CG_Init_CGents(ctx.world);
    // this is a No-No now we have stl vector classes in here.
    // memset( &cg, 0, sizeof( cg ) );
    CG_Init_CG(ctx.world);
    CG_InitItems(ctx.world);

    // create the global jetpack instance
    CG_InitJetpackGhoul2(ctx);

    CG_PmoveClientPointerUpdate(ctx.world);

    /*
    Ghoul2 Insert End
    */

    // Load sabers.cfg data
    {
        let traps = CgBgTraps::new(engine, ctx.world_raw());
        WP_SaberLoadParms(&mut ctx.world.bg_state, &traps);
    }

    // this is kinda dumb as well, but I need to pre-load some fonts in order to have the text available
    //	to say I'm loading the assets.... which includes loading the fonts. So I'll set these up as reasonable
    //	defaults, then let the menu asset parser (which actually specifies the ingame fonts) load over them
    //	if desired during parse.  Dunno how legal it is to store in these cgDC things, but it causes no harm
    //	and even if/when they get overwritten they'll be legalised by the menu asset parser :-)
    // CG_LoadFonts();
    ds.Assets.qhSmallFont = trap::R_RegisterFont(engine, "ocr_a");
    ds.Assets.qhMediumFont = trap::R_RegisterFont(engine, "ergoec");
    ds.Assets.qhBigFont = ds.Assets.qhMediumFont;

    // SAFETY: `cgs_t` is POD (fixed arrays/ints/handles, no owned heap fields)
    // - same zero-fill shape `centity_t`/`clientInfo_t` already use elsewhere
    // in this crate. Raven: `memset( &cgs, 0, sizeof( cgs ) )`.
    ctx.world.cgs = unsafe { core::mem::zeroed() };
    // Raven: `memset( cg_weapons, 0, sizeof(cg_weapons) )`.
    for w in ctx.world.cg_weapons.iter_mut() {
        *w = zeroed_weapon_info();
    }

    ctx.world.cg.clientNum = clientNum;

    ctx.world.cgs.processedSnapshotNum = serverMessageNum;
    ctx.world.cgs.serverCommandSequence = serverCommandSequence;

    ctx.world.cg.loadLCARSStage = 0;

    ctx.world.cg.itemSelect = -1;
    ctx.world.cg.forceSelect = -1;

    // load a few needed things before we do any screen updates
    ctx.world.cgs.media.charsetShader = trap::R_RegisterShaderNoMip(engine, "gfx/2d/charsgrid_med");
    ctx.world.cgs.media.whiteShader = trap::R_RegisterShader(engine, "white");

    ctx.world.cgs.media.loadBarLED = trap::R_RegisterShaderNoMip(engine, "gfx/hud/load_tick");
    ctx.world.cgs.media.loadBarLEDCap =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/load_tick_cap");
    ctx.world.cgs.media.loadBarLEDSurround =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/mp_levelload");

    // Force HUD set up
    ctx.world.cg.forceHUDActive = qtrue;
    ctx.world.cg.forceHUDTotalFlashTime = 0;
    ctx.world.cg.forceHUDNextFlashTime = 0;

    let mut i = WP_NONE + 1;
    while i <= LAST_USEABLE_WEAPON {
        let item = BG_FindItemForWeapon(i);
        let icon = item.item().icon;

        if let Some(icon) = icon.filter(|s| !s.is_empty()) {
            ctx.world.cgs.media.weaponIcons[i as usize] = trap::R_RegisterShaderNoMip(engine, icon);
            ctx.world.cgs.media.weaponIcons_NA[i as usize] =
                trap::R_RegisterShaderNoMip(engine, &format!("{icon}_na"));
        } else {
            // make sure it is zero'd (default shader)
            ctx.world.cgs.media.weaponIcons[i as usize] = 0;
            ctx.world.cgs.media.weaponIcons_NA[i as usize] = 0;
        }
        i += 1;
    }
    let buf = trap::Cvar_VariableStringBuffer(engine, "com_buildscript", 64);
    if atoi(&buf) != 0 {
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/w_icon_saberstaff");
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/w_icon_duallightsaber");
    }

    // HUD artwork for cycling inventory,weapons and force powers
    ctx.world.cgs.media.weaponIconBackground =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/background");
    ctx.world.cgs.media.forceIconBackground =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/background_f");
    ctx.world.cgs.media.inventoryIconBackground =
        trap::R_RegisterShaderNoMip(engine, "gfx/hud/background_i");

    //rww - precache holdable item icons here
    // Raven resets its loop index (`i = 0`) here to walk `bg_itemlist` with the
    // same variable the weapon-icon loop above used; the port gives this loop
    // its own iterator instead of reusing a shared index (§C10 - shape is free).
    for it in bg_itemlist.iter().take(bg_numItems as usize) {
        if let ItemKind::Holdable(giTag) = it.kind {
            if let Some(icon) = it.icon {
                ctx.world.cgs.media.invenIcons[giTag as usize] =
                    trap::R_RegisterShaderNoMip(engine, icon);
            } else {
                ctx.world.cgs.media.invenIcons[giTag as usize] = 0;
            }
        }
    }

    //rww - precache force power icons here
    for (i, path) in HOLOCRON_ICONS.iter().enumerate() {
        ctx.world.cgs.media.forcePowerIcons[i] = trap::R_RegisterShaderNoMip(engine, path);
    }
    ctx.world.cgs.media.rageRecShader =
        trap::R_RegisterShaderNoMip(engine, "gfx/mp/f_icon_ragerec");

    //body decal shaders -rww
    ctx.world.cgs.media.bdecal_bodyburn1 =
        trap::R_RegisterShader(engine, "gfx/damage/bodyburnmark1");
    ctx.world.cgs.media.bdecal_saberglow =
        trap::R_RegisterShader(engine, "gfx/damage/saberglowmark");
    ctx.world.cgs.media.bdecal_burn1 =
        trap::R_RegisterShader(engine, "gfx/damage/bodybigburnmark1");
    ctx.world.cgs.media.mSaberDamageGlow =
        trap::R_RegisterShader(engine, "gfx/effects/saberDamageGlow");

    CG_RegisterCvars(ctx);

    CG_InitConsoleCommands(ctx);

    ctx.world.cg.weaponSelect = WP_BRYAR_PISTOL;

    ctx.world.cgs.redflag = -1; // For compatibily, default to unset for
    ctx.world.cgs.blueflag = -1;
    ctx.world.cgs.flagStatus = -1;
    // old servers

    // get the rendering configuration from the client system
    trap::GetGlconfig(engine, &mut ctx.world.cgs.glconfig);
    ctx.world.cgs.screenXScale = ctx.world.cgs.glconfig.vidWidth as f32 / 640.0;
    ctx.world.cgs.screenYScale = ctx.world.cgs.glconfig.vidHeight as f32 / 480.0;

    // get the gamestate from the client system
    trap::GetGameState(engine, &mut ctx.world.cgs.gameState);

    CG_TransitionPermanent(ctx); //rwwRMG - added

    // check version
    let s = CG_ConfigString(ctx, CS_GAME_VERSION);
    if s != GAME_VERSION {
        CG_Error(
            ctx,
            &format!("Client/Server game mismatch: {GAME_VERSION}/{s}"),
        );
        return;
    }

    let s = CG_ConfigString(ctx, CS_LEVEL_START_TIME);
    ctx.world.cgs.levelStartTime = atoi(&s);

    CG_ParseServerinfo(ctx);

    // load the new map
    // CG_LoadingString( "collision map" );

    let mapname = buf_to_string(&ctx.world.cgs.mapname.map(|c| c as u8));
    trap::CM_LoadMap(engine, &mapname, false);

    String_Init(menus, ctx);

    ctx.world.cg.loading = qtrue; // force players to load instead of defer

    //make sure saber data is loaded before this! (so we can precache the appropriate hilts)
    CG_InitSiegeMode(ctx);

    CG_RegisterSounds(ctx);

    // CG_LoadingString( "graphics" );

    CG_RegisterGraphics(ctx);

    // CG_LoadingString( "clients" );

    CG_RegisterClients(ctx); // if low on memory, some clients will be deferred

    CG_AssetCache(ctx, ds);
    CG_LoadHudMenu(ctx, menus, ds); // load new hud stuff

    ctx.world.cg.loading = qfalse; // future players will be deferred

    CG_InitLocalEntities(ctx.world);

    CG_InitMarkPolys(ctx.world);

    // remove the last loading update
    ctx.world.cg.infoScreenText[0] = 0;

    // Make sure we have update values (scores)
    CG_SetConfigValues(ctx);

    CG_StartMusic(ctx, false);

    // CG_LoadingString( "Clearing light styles" );
    CG_ClearLightStyles(ctx);

    // CG_LoadingString( "Creating automap data" );
    //init automap
    trap::R_InitWireframeAutomap(engine);

    CG_LoadingString(ctx, "");

    CG_ShaderStateChanged(ctx);

    trap::S_ClearLoopingSounds(engine);

    ctx.world.cg.distanceCull = trap::R_GetDistanceCull(engine);

    //now get all the cgame only cents
    CG_SpawnCGameOnlyEnts(ctx);
}

/// Raven `vmMain` — the module's one ABI dispatch shell, `mp_ui`'s `vmMain`
/// shape (DEC-38 ruling 1 revised): the shell owns the one [`CgState`],
/// splits it into its three disjoint borrows, builds a [`CgContext`] over the
/// world half, and routes the command to the matching `CG_*`/`C_*` handler.
///
/// No `MpCgameExport::try_from` exists yet (unlike `mp_ui`'s `MpUiExport`,
/// `mp_abi::cgame::exports` carries no `TryFrom<i32>` impl and this wave may
/// not touch `mp_abi`), so the dispatch is a chain of `c if c == X as c_int`
/// guards over the plain `c_int` wire value instead of a `match` on the enum
/// itself; the `_` arm reproduces Raven's `default: CG_Error(...); break;`
/// falling through to the trailing `return -1`.
///
/// Several arms (`CG_GET_ORIGIN`/`CG_GET_ANGLES`/`CG_GET_ORIGIN_TRAJECTORY`/
/// `CG_GET_ANGLE_TRAJECTORY`/`CG_GET_GHOUL2`/`CG_GET_MODEL_LIST`) hand the
/// engine a raw address (Raven's `(float *)arg1` / `(int)&x` casts) - this
/// `vmMain` boundary is the ABI seam itself (porting-rules §D11). The slots
/// and return are `isize` (the platform layer's `AbiWord`), Raven's `int`
/// being pointer-width only because retail is ILP32 - so the address arms are
/// sound on both the i686 module builds and the LP64 dev builds. Each carries
/// a `SAFETY` note at the site.
///
/// All 32 arms dispatch live under DEC-47.1 (ruled 2026-07-28: the DEC-38
/// shape applies to cgame — no fn takes a separate `dc: &mut dyn
/// DisplayContext`, `ctx` is the carrier). The wave-era two-param shape could
/// never be dispatched from here: `CgContext` is the sole `DisplayContext`
/// implementor in `mp_cgame` and every `DisplayContext` method takes
/// `&mut self`, so `ctx` AND `dc` from the one owned `CgState` would need two
/// live `&mut` paths into the same `CgWorld` — the E0499 pattern DEC-38
/// ruling 1's revision proved unbuildable for `mp_ui`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:190-359`
#[allow(clippy::too_many_arguments)]
pub fn vmMain(
    state: &mut CgState,
    engine: &Engine,
    command: isize,
    arg0: isize,
    arg1: isize,
    arg2: isize,
    _arg3: isize,
    _arg4: isize,
    _arg5: isize,
    _arg6: isize,
    _arg7: isize,
    _arg8: isize,
    _arg9: isize,
    _arg10: isize,
    _arg11: isize,
) -> isize {
    let CgState {
        world,
        menus,
        cgDC: ds,
    } = state;
    let mut ctx = CgContext {
        world: &mut **world,
        engine,
    };

    match command {
        c if c == MpCgameExport::CG_INIT as isize => {
            CG_Init(
                &mut ctx,
                menus,
                ds,
                arg0 as c_int,
                arg1 as c_int,
                arg2 as c_int,
            );
            0
        }

        c if c == MpCgameExport::CG_SHUTDOWN as isize => {
            CG_Shutdown(&mut ctx, menus);
            0
        }

        c if c == MpCgameExport::CG_CONSOLE_COMMAND as isize => {
            // PORT-NOTE: menuScoreboard is cg_draw's cached scoreboard handle
            // with no CgWorld home yet (the ownerdraw tail owns it); None
            // reproduces Raven's NULL until the scoreboard menu first caches.
            CG_ConsoleCommand(&mut ctx, menus, ds, None) as isize
        }

        c if c == MpCgameExport::CG_DRAW_ACTIVE_FRAME as isize => {
            let demoPlayback = if arg2 != 0 { qtrue } else { qfalse };
            CG_DrawActiveFrame(
                &mut ctx,
                arg0 as c_int,
                arg1 as c_int,
                demoPlayback,
                menus,
                ds,
            );
            0
        }

        c if c == MpCgameExport::CG_CROSSHAIR_PLAYER as isize => {
            CG_CrosshairPlayer(ctx.world) as isize
        }

        c if c == MpCgameExport::CG_LAST_ATTACKER as isize => CG_LastAttacker(ctx.world) as isize,

        c if c == MpCgameExport::CG_KEY_EVENT as isize => {
            CG_KeyEvent(&mut ctx, menus, ds, arg0 as c_int, arg1 != 0);
            0
        }

        c if c == MpCgameExport::CG_MOUSE_EVENT as isize => {
            ds.cursorx = ctx.world.cgs.cursorX;
            ds.cursory = ctx.world.cgs.cursorY;
            CG_MouseEvent(&mut ctx, menus, ds, arg0 as c_int, arg1 as c_int);
            0
        }

        c if c == MpCgameExport::CG_EVENT_HANDLING as isize => {
            CG_EventHandling(&mut ctx, menus, ds, arg0 as c_int);
            0
        }

        c if c == MpCgameExport::CG_POINT_CONTENTS as isize => {
            // SAFETY: the engine writes a `TCGPointContents` into
            // `cg.sharedBuffer` before invoking `CG_POINT_CONTENTS`;
            // `read_unaligned` copies it out without forming a reference into
            // the byte buffer (alignment 1).
            let data = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const TCGPointContents).read_unaligned()
            };
            C_PointContents(&mut ctx, &data) as isize
        }

        c if c == MpCgameExport::CG_GET_LERP_ORIGIN as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGVectorData`.
            let mut data = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const TCGVectorData).read_unaligned()
            };
            C_GetLerpOrigin(ctx.world, &mut data);
            // SAFETY: same buffer contract; the engine reads `mPoint` back out
            // of `cg.sharedBuffer` after this call returns.
            unsafe {
                (ctx.world.shared_buffer.as_mut_ptr() as *mut TCGVectorData).write_unaligned(data);
            }
            0
        }

        c if c == MpCgameExport::CG_GET_LERP_DATA as isize => {
            // SAFETY: same shared-buffer contract, typed `TCGGetBoltData`.
            let mut data = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const TCGGetBoltData).read_unaligned()
            };
            C_GetLerpData(ctx.world, &mut data);
            // SAFETY: same buffer contract; the engine reads the filled
            // origin/scale/angles back out of `cg.sharedBuffer` after this
            // call returns.
            unsafe {
                (ctx.world.shared_buffer.as_mut_ptr() as *mut TCGGetBoltData).write_unaligned(data);
            }
            0
        }

        c if c == MpCgameExport::CG_GET_GHOUL2 as isize => {
            //NOTE: This is used by the effect bolting which is actually not used at all.
            //I'm fairly sure if you try to use it with vm's it will just give you total
            //garbage. In other words, use at your own risk.
            //
            // SAFETY: this vmMain boundary IS the ABI seam (§D11); the engine
            // treats the return as a native pointer-width address. The raw
            // entity index is engine-supplied (ROFF's mEntID and friends) - a
            // bad one panics via entity(), the crate's posture, where Raven
            // read garbage; same at every CG_GET_*/lerp arm below.
            ctx.world.entity(arg0 as usize).ghoul2 as isize
        }

        c if c == MpCgameExport::CG_GET_MODEL_LIST as isize => {
            // SAFETY: same ABI-seam address convention as `CG_GET_GHOUL2`.
            // The engine keeps this address across calls (RoffSystem), which
            // is fine: `CgState.world` is a Box that lives for the module's
            // whole life, so the array never moves.
            ctx.world.cgs.gameModels.as_ptr() as isize
        }

        c if c == MpCgameExport::CG_CALC_LERP_POSITIONS as isize => {
            CG_CalcEntityLerpPositions(&mut ctx, arg0 as usize);
            0
        }

        c if c == MpCgameExport::CG_TRACE as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGTrace`.
            let mut td =
                unsafe { (ctx.world.shared_buffer.as_ptr() as *const TCGTrace).read_unaligned() };
            C_Trace(&mut ctx, &mut td);
            // SAFETY: same buffer contract; the engine reads `mResult` back
            // out of `cg.sharedBuffer` after this call returns
            // (G2_bones.cpp precedent - see `CG_RagCallback`'s
            // `RAG_CALLBACK_TRACELINE` arm above).
            unsafe {
                (ctx.world.shared_buffer.as_mut_ptr() as *mut TCGTrace).write_unaligned(td);
            }
            0
        }

        c if c == MpCgameExport::CG_GET_SORTED_FORCE_POWER as isize => {
            // §F19: `arg0` is an engine-supplied index into the 18-entry
            // `forcePowerSorted` table; Raven read whatever followed the
            // array on an out-of-range index. No engine caller of this export
            // exists in-tree (§20 dead surface, arm kept for the dispatch
            // contract) - neutral "no power" (`-1`) instead of indexing OOB.
            forcePowerSorted.get(arg0 as usize).copied().unwrap_or(-1) as isize
        }

        c if c == MpCgameExport::CG_G2TRACE as isize => {
            // SAFETY: same shared-buffer contract as `CG_TRACE`.
            let mut td =
                unsafe { (ctx.world.shared_buffer.as_ptr() as *const TCGTrace).read_unaligned() };
            C_G2Trace(&mut ctx, &mut td);
            // SAFETY: same buffer contract as `CG_TRACE`'s write-back.
            unsafe {
                (ctx.world.shared_buffer.as_mut_ptr() as *mut TCGTrace).write_unaligned(td);
            }
            0
        }

        c if c == MpCgameExport::CG_G2MARK as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGG2Mark`.
            let data =
                unsafe { (ctx.world.shared_buffer.as_ptr() as *const TCGG2Mark).read_unaligned() };
            C_G2Mark(&mut ctx, &data);
            0
        }

        c if c == MpCgameExport::CG_RAG_CALLBACK as isize => {
            CG_RagCallback(&mut ctx, arg0 as c_int) as isize
        }

        c if c == MpCgameExport::CG_INCOMING_CONSOLE_COMMAND as isize => {
            // Raven's `#if 0` filter block (cg_main.c:264-279) never compiled
            // in retail - not transcribed. The live body is just this.
            1
        }

        c if c == MpCgameExport::CG_GET_USEABLE_FORCE as isize => {
            CG_NoUseableForce(ctx.world) as isize
        }

        c if c == MpCgameExport::CG_GET_ORIGIN as isize => {
            let origin = ctx.world.entity(arg0 as usize).currentState.pos.trBase;
            // SAFETY: this vmMain boundary IS the ABI seam (§D11); `arg1` is
            // the native pointer-width out-param address the engine handed
            // this call to receive 3 floats into.
            unsafe {
                (arg1 as *mut vec3_t).write_unaligned(origin);
            }
            0
        }

        c if c == MpCgameExport::CG_GET_ANGLES as isize => {
            let angles = ctx.world.entity(arg0 as usize).currentState.apos.trBase;
            // SAFETY: same ABI-seam out-param contract as `CG_GET_ORIGIN`.
            unsafe {
                (arg1 as *mut vec3_t).write_unaligned(angles);
            }
            0
        }

        c if c == MpCgameExport::CG_GET_ORIGIN_TRAJECTORY as isize => {
            // SAFETY: same ABI-seam address convention as `CG_GET_GHOUL2`.
            // The engine WRITES through this (RoffSystem's SetLerp stores
            // trType/trTime/trBase/trDelta), so the address must carry mut
            // provenance and stays valid for the module's life (Box'd world).
            &mut ctx.world.entity_mut(arg0 as usize).nextState.pos as *mut _ as isize
        }

        c if c == MpCgameExport::CG_GET_ANGLE_TRAJECTORY as isize => {
            // SAFETY: same ABI-seam mutable-address contract as
            // `CG_GET_ORIGIN_TRAJECTORY`.
            &mut ctx.world.entity_mut(arg0 as usize).nextState.apos as *mut _ as isize
        }

        c if c == MpCgameExport::CG_ROFF_NOTETRACK_CALLBACK as isize => {
            // SAFETY: `arg1` is the native address of a NUL-terminated C
            // string the engine handed this call; borrow-only read, no
            // ownership transfer.
            let notetrack =
                latin1_to_string(unsafe { CStr::from_ptr(arg1 as *const c_char) }.to_bytes());
            CG_ROFF_NotetrackCallback(&mut ctx, arg0 as usize, &notetrack);
            0
        }

        c if c == MpCgameExport::CG_IMPACT_MARK as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGImpactMark`.
            let data = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const TCGImpactMark).read_unaligned()
            };
            C_ImpactMark(&mut ctx, &data);
            0
        }

        c if c == MpCgameExport::CG_MAP_CHANGE as isize => {
            // this trap may be called more than once for a given map change,
            // as the server is going to attempt to send out multiple
            // broadcasts in hopes that the client will receive one of them
            ctx.world.cg.mMapChange = qtrue;
            0
        }

        c if c == MpCgameExport::CG_AUTOMAP_INPUT as isize => {
            //special input during automap mode -rww
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `autoMapInput_t`.
            let autoInput = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const autoMapInput_t).read_unaligned()
            };
            ctx.world.view.cg_autoMapInput = autoInput;

            if arg0 == 0 {
                //if this is non-0, it's actually a one-frame mouse event
                ctx.world.view.cg_autoMapInputTime = ctx.world.cg.time + 1000;
            } else {
                if ctx.world.view.cg_autoMapInput.yaw != 0.0 {
                    ctx.world.view.cg_autoMapAngle[YAW] += ctx.world.view.cg_autoMapInput.yaw;
                }

                if ctx.world.view.cg_autoMapInput.pitch != 0.0 {
                    ctx.world.view.cg_autoMapAngle[PITCH] += ctx.world.view.cg_autoMapInput.pitch;
                }
                ctx.world.view.cg_autoMapInput.yaw = 0.0;
                ctx.world.view.cg_autoMapInput.pitch = 0.0;
            }
            0
        }

        c if c == MpCgameExport::CG_MISC_ENT as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGMiscEnt`.
            let data =
                unsafe { (ctx.world.shared_buffer.as_ptr() as *const TCGMiscEnt).read_unaligned() };
            CG_MiscEnt(&mut ctx, &data);
            0
        }

        c if c == MpCgameExport::CG_FX_CAMERASHAKE as isize => {
            // SAFETY: same shared-buffer contract as `CG_POINT_CONTENTS`,
            // typed `TCGCameraShake`.
            let data = unsafe {
                (ctx.world.shared_buffer.as_ptr() as *const TCGCameraShake).read_unaligned()
            };
            CG_DoCameraShake(
                ctx.world,
                data.mOrigin,
                data.mIntensity,
                data.mRadius,
                data.mTime,
            );
            0
        }

        _ => {
            CG_Error(&mut ctx, &format!("vmMain: unknown command {command}"));
            return -1;
        }
    }
}
