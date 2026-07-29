//! Port of `oracle/codemp/cgame/cg_draw.c` — the HUD and every other 2D overlay drawn each frame. Functions land via the C5
//! transcription waves.

// Raven's own spellings survive: `veh_damage_t` is a snake_case type name and
// `colorTable`/`vehDamageData` are camelCase consts.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::f64::consts::PI;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use native_string::{
    atoi, buf_to_string, cstr, string_to_latin1, Info_ValueForKey, Q_strcat, Q_strncpyz,
    Q_strncpyzBytes,
};

use mp_abi::ui::public::ui_menu_command_t::{
    UIMENU_CLOSEALL, UIMENU_SIEGEMESSAGE, UIMENU_SIEGEOBJECTIVES,
};
use mp_bg::bg_misc::{
    forcePowerSorted, selected_holdable_tag, BG_EvaluateTrajectory, BG_FindItemForPowerup,
    BG_GetItemIndexByTag, BG_GiveMeVectorFromMatrix, BG_HasYsalamiri, BG_IsItemSelectable,
    BG_ProperForceIndex,
};
use mp_bg::bg_panimate::BG_ParseAnimationFile;
use mp_bg::bg_saga::BG_SiegeFindClassByName;
use mp_bg::public::animation::animation_t;
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::configstring::{CS_LOCATIONS, CS_PLAYERS, SCORE_NOT_PRESENT, VOTE_TIME};
use mp_bg::public::duel_team::duelTeam_t;
use mp_bg::public::entity_flags::{EF_DEAD, EF_DOUBLE_AMMO, EF_NODRAW, EF_RADAROBJECT};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{
    GT_CTF, GT_CTY, GT_DUEL, GT_FFA, GT_HOLOCRON, GT_JEDIMASTER, GT_POWERDUEL, GT_SIEGE, GT_TEAM,
};
use mp_bg::public::holdable::HI_NUM_HOLDABLE;
use mp_bg::public::item_type::IT_HOLDABLE;
use mp_bg::public::pers_enum::persEnum_t::{PERS_SCORE, PERS_TEAM};
use mp_bg::public::pmtype::pmtype_t::{PM_INTERMISSION, PM_SPECTATOR};
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_CLOAKED, PW_NEUTRALFLAG, PW_NUM_POWERUPS, PW_REDFLAG,
};
use mp_bg::public::stat_index::statIndex_t::{
    STAT_ARMOR, STAT_HEALTH, STAT_HOLDABLE_ITEM, STAT_HOLDABLE_ITEMS, STAT_MAX_HEALTH,
};
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::public::weaponstate::weaponstate_t::WEAPON_CHARGING_ALT;
use mp_bg::saga::siege_class_flags_t::siegeClassFlags_t::CFL_STATVIEWER;
use mp_bg::saga::siege_team_t::SIEGE_ROUND_BEGIN_TIME;
use mp_bg::weapons::ammo_data::ammoData;
use mp_bg::weapons::weapon_data::weaponData;
use mp_bg::weapons::weapon_t::{WP_DISRUPTOR, WP_EMPLACED_GUN, WP_NONE, WP_SABER};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::qcommon::saber::saber_styles::saber_styles_t;
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::common::mp::qcommon::PMF_FOLLOW;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::force_powers::{
    FORCE_LEVEL_2, FORCE_LEVEL_3, FP_ABSORB, FP_HEAL, FP_LEVITATION, FP_PROTECT, FP_RAGE,
    FP_SABERTHROW, FP_SABER_DEFENSE, FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TELEPATHY,
    NUM_FORCE_POWERS,
};
use mp_qshared::shared::limits::{MAX_SAY_TEXT, SNAPFLAG_RATE_DELAYED};
use mp_qshared::shared::q_color::{colorWhite, g_color_table};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorSubtract, vec3_origin, AngleVectors, AnglesToAxis,
    Distance, VectorClear, VectorLength, VectorNormalize, VectorSet, PITCH, YAW,
};
use mp_qshared::shared::surface_flags::{
    CONTENTS_BODY, CONTENTS_FOG, CONTENTS_LAVA, CONTENTS_OPAQUE, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_WATER,
};
use mp_qshared::shared::{
    ct_table_t, mdxaBone_t, qfalse, qhandle_t, qtrue, vec3_t, vec4_t, Eorientations,
    BIGCHAR_HEIGHT, BIGCHAR_WIDTH, CHAN_ANNOUNCER, CHAN_AUTO, CHAN_LOCAL, ENTITYNUM_NONE,
    ENTITYNUM_WORLD, MAX_CLIENTS, MAX_CLIENTS_I32, MAX_QPATH, SCREEN_HEIGHT, SCREEN_WIDTH,
    SMALLCHAR_HEIGHT, TINYCHAR_HEIGHT, TINYCHAR_WIDTH,
};
use mp_uishared::shared::cached_assets_t::NUM_CROSSHAIRS;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_NORMAL, ITEM_TEXTSTYLE_OUTLINED,
    ITEM_TEXTSTYLE_OUTLINESHADOWED, ITEM_TEXTSTYLE_PULSE, ITEM_TEXTSTYLE_SHADOWED,
    ITEM_TEXTSTYLE_SHADOWEDMORE,
};
use mp_uishared::ui_shared::{Menu_FindItemByName, Menus_CloseByName, Menus_FindByName};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::cg_drawtools::{
    CG_ColorForHealth, CG_DrawBigString, CG_DrawNumField, CG_DrawPic, CG_DrawRect,
    CG_DrawRotatePic, CG_DrawRotatePic2, CG_DrawSmallString, CG_DrawStringExt, CG_DrawStrlen,
    CG_FadeColor, CG_FillRect, CG_GetColorForHealth, UI_DrawProportionalString,
    UI_DrawScaledProportionalString,
};
use crate::cg_main::{CG_ConfigString, CG_Error, CG_GetLocationString, CG_GetStringEdString};
use crate::cg_new_draw::{CG_OtherTeamHasFlag, CG_YourTeamHasFlag};
use crate::cg_players::{CG_IsMindTricked, CG_RadiusForCent};
use crate::cg_predict::{CG_G2Trace, CG_Trace};
use crate::cg_scoreboard::CG_DrawOldScoreboard;
use crate::cg_view::WAVE_FREQUENCY;
use crate::cg_weapons::{
    CG_CalcMuzzlePoint, CG_DrawIconBackground, CG_DrawWeaponSelect, CG_RegisterItemVisuals,
    WEAPON_SELECT_TIME,
};
use crate::local::cg_t::MAX_CHATBOX_ITEMS;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// PORT-NOTE: `q_shared.h`'s font enum is anonymous (`enum { FONT_NONE,
// FONT_SMALL=1, ... }`), so per the anonymous-enum convention these are
// `const`s; local, mirroring `mp_uishared::ui_shared`'s own file-local copies.
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_SMALL: c_int = 1;
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_LARGE: c_int = 3;
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_SMALL2: c_int = 4;

// PORT-NOTE: `tr_types.h`'s render flags have no `mp_qshared` home yet (the
// renderer crate keeps its own copy in `tr_public::ref_flags`, which cgame does
// not depend on), so the two `CG_Draw3DModel` needs land here.
/// Raven `RF_NOSHADOW` — don't add stencil shadows.
/// Source: `oracle/codemp/cgame/tr_types.h:25`
const RF_NOSHADOW: c_int = 0x00040;
/// Raven `RDF_NOWORLDMODEL` — used for player configuration screen.
/// Source: `oracle/codemp/cgame/tr_types.h:56`
const RDF_NOWORLDMODEL: c_int = 1;

// PORT-NOTE: `cg_public.h`'s usercmd ring-buffer depth. This is its first
// ported consumer in cgame, so it lands file-local like `FONT_SMALL` above.
/// Raven `#define CMD_BACKUP 64`.
/// Source: `oracle/codemp/cgame/cg_public.h:6`
const CMD_BACKUP: c_int = 64;

/// Raven `vec4_t bluehudtint` — the blue-team HUD tint `CG_DrawHUD` points
/// `hudTintColor` at. Read-only compiled-in data, so a `const` (§C8).
/// Source: `oracle/codemp/cgame/cg_draw.c:24`
const bluehudtint: vec4_t = [0.5, 0.5, 1.0, 1.0];

/// Raven `vec4_t redhudtint` — the red-team HUD tint.
/// Source: `oracle/codemp/cgame/cg_draw.c:25`
const redhudtint: vec4_t = [1.0, 0.5, 0.5, 1.0];

/// Raven `#define MAX_HUD_TICS 4` — tics per HUD bar (health/armor/force/ammo).
/// Source: `oracle/codemp/cgame/cg_draw.c:42`
pub const MAX_HUD_TICS: usize = 4;

/// Raven `const char *armorTicName[MAX_HUD_TICS]` — the armor bar's four menu
/// item names. Read-only compiled-in data, so a `const` (§C8).
/// Source: `oracle/codemp/cgame/cg_draw.c:43-49`
pub const armorTicName: [&str; MAX_HUD_TICS] =
    ["armor_tic1", "armor_tic2", "armor_tic3", "armor_tic4"];

/// Raven `const char *healthTicName[MAX_HUD_TICS]`.
/// Source: `oracle/codemp/cgame/cg_draw.c:51-57`
pub const healthTicName: [&str; MAX_HUD_TICS] =
    ["health_tic1", "health_tic2", "health_tic3", "health_tic4"];

/// Raven `const char *forceTicName[MAX_HUD_TICS]`.
/// Source: `oracle/codemp/cgame/cg_draw.c:59-65`
pub const forceTicName: [&str; MAX_HUD_TICS] =
    ["force_tic1", "force_tic2", "force_tic3", "force_tic4"];

/// Raven `const char *ammoTicName[MAX_HUD_TICS]`.
/// Source: `oracle/codemp/cgame/cg_draw.c:67-73`
pub const ammoTicName: [&str; MAX_HUD_TICS] = ["ammo_tic1", "ammo_tic2", "ammo_tic3", "ammo_tic4"];

/// Raven `char *showPowersName[]` — the `SP_INGAME` string keys for the force
/// select HUD, indexed by `forcePowers_t`.
///
/// Raven's table ends in a NULL sentinel that its one reader null-checks, so
/// the entries stay `Option<&str>` and the sentinel survives as `None`.
/// Source: `oracle/codemp/cgame/cg_draw.c:75-95`
pub const showPowersName: [Option<&str>; NUM_FORCE_POWERS as usize + 1] = [
    Some("HEAL2"),           //FP_HEAL
    Some("JUMP2"),           //FP_LEVITATION
    Some("SPEED2"),          //FP_SPEED
    Some("PUSH2"),           //FP_PUSH
    Some("PULL2"),           //FP_PULL
    Some("MINDTRICK2"),      //FP_TELEPTAHY
    Some("GRIP2"),           //FP_GRIP
    Some("LIGHTNING2"),      //FP_LIGHTNING
    Some("DARK_RAGE2"),      //FP_RAGE
    Some("PROTECT2"),        //FP_PROTECT
    Some("ABSORB2"),         //FP_ABSORB
    Some("TEAM_HEAL2"),      //FP_TEAM_HEAL
    Some("TEAM_REPLENISH2"), //FP_TEAM_FORCE
    Some("DRAIN2"),          //FP_DRAIN
    Some("SEEING2"),         //FP_SEE
    Some("SABER_OFFENSE2"),  //FP_SABER_OFFENSE
    Some("SABER_DEFENSE2"),  //FP_SABER_DEFENSE
    Some("SABER_THROW2"),    //FP_SABERTHROW
    None,
];

/// Raven `#define MAX_SHOWPOWERS NUM_FORCE_POWERS`.
/// Source: `oracle/codemp/cgame/cg_draw.c:1393`
pub const MAX_SHOWPOWERS: c_int = NUM_FORCE_POWERS;

/// Raven `#define MAX_VHUD_SHIELD_TICS 12`.
/// Source: `oracle/codemp/cgame/cg_draw.c:1868`
pub const MAX_VHUD_SHIELD_TICS: c_int = 12;

/// Raven `#define MAX_VHUD_SPEED_TICS 5`.
/// Source: `oracle/codemp/cgame/cg_draw.c:1869`
pub const MAX_VHUD_SPEED_TICS: c_int = 5;

/// Raven `#define MAX_VHUD_ARMOR_TICS 5`.
/// Source: `oracle/codemp/cgame/cg_draw.c:1870`
pub const MAX_VHUD_ARMOR_TICS: c_int = 5;

/// Raven `#define MAX_VHUD_AMMO_TICS 5`.
/// Source: `oracle/codemp/cgame/cg_draw.c:1871`
pub const MAX_VHUD_AMMO_TICS: c_int = 5;

/// Raven `#define FPS_FRAMES 16` — frames the fps counter averages over.
/// Source: `oracle/codemp/cgame/cg_draw.c:3070`
pub const FPS_FRAMES: usize = 16;

/// Raven `#define MAX_HEALTH_FOR_IFACE 100`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3116`
pub const MAX_HEALTH_FOR_IFACE: c_int = 100;

/// Raven `#define RADAR_RADIUS 60`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3169`
pub const RADAR_RADIUS: c_int = 60;

/// Raven `#define RADAR_X (580 - RADAR_RADIUS)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3170`
pub const RADAR_X: c_int = 580 - RADAR_RADIUS;

/// Raven `#define RADAR_CHAT_DURATION 6000`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3172`
pub const RADAR_CHAT_DURATION: c_int = 6000;

/// Raven `#define RADAR_MISSILE_RANGE 3000.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3175`
pub const RADAR_MISSILE_RANGE: f32 = 3000.0;

/// Raven `#define RADAR_ASTEROID_RANGE 10000.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3176`
pub const RADAR_ASTEROID_RANGE: f32 = 10000.0;

/// Raven `#define RADAR_MIN_ASTEROID_SURF_WARN_DIST 1200.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:3177`
pub const RADAR_MIN_ASTEROID_SURF_WARN_DIST: f32 = 1200.0;

/// Raven `#define LAG_SAMPLES 128` — the lagometer ring-buffer length; the
/// wrap is a mask, so it must stay a power of two.
/// Source: `oracle/codemp/cgame/cg_draw.c:4141`
pub const LAG_SAMPLES: usize = 128;

/// Raven `#define MAX_LAGOMETER_PING 900`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4244`
pub const MAX_LAGOMETER_PING: c_int = 900;

/// Raven `#define MAX_LAGOMETER_RANGE 300`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4245`
pub const MAX_LAGOMETER_RANGE: c_int = 300;

/// Raven `#define HEALTH_WIDTH 50.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4485`
pub const HEALTH_WIDTH: f32 = 50.0;

/// Raven `#define HEALTH_HEIGHT 5.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4486`
pub const HEALTH_HEIGHT: f32 = 5.0;

/// Raven `#define CGTIMERBAR_H 50.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4741`
pub const CGTIMERBAR_H: f32 = 50.0;

/// Raven `#define CGTIMERBAR_W 10.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4742`
pub const CGTIMERBAR_W: f32 = 10.0;

/// Raven `#define CGTIMERBAR_X (SCREEN_WIDTH-CGTIMERBAR_W-120.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4743`
pub const CGTIMERBAR_X: f32 = SCREEN_WIDTH as f32 - CGTIMERBAR_W - 120.0;

/// Raven `#define CGTIMERBAR_Y (SCREEN_HEIGHT-CGTIMERBAR_H-20.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4744`
pub const CGTIMERBAR_Y: f32 = SCREEN_HEIGHT as f32 - CGTIMERBAR_H - 20.0;

/// Raven `#define CRAZY_CROSSHAIR_MAX_ERROR_X (100.0f*640.0f/480.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4804`
pub const CRAZY_CROSSHAIR_MAX_ERROR_X: f32 = 100.0 * 640.0 / 480.0;

/// Raven `#define CRAZY_CROSSHAIR_MAX_ERROR_Y (100.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:4805`
pub const CRAZY_CROSSHAIR_MAX_ERROR_Y: f32 = 100.0;

/// Raven `#define MAX_XHAIR_DIST_ACCURACY 20000.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:6071`
pub const MAX_XHAIR_DIST_ACCURACY: f32 = 20000.0;

/// Raven `#define JPFUELBAR_H 100.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7146`
pub const JPFUELBAR_H: f32 = 100.0;

/// Raven `#define JPFUELBAR_W 20.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7147`
pub const JPFUELBAR_W: f32 = 20.0;

/// Raven `#define JPFUELBAR_X (SCREEN_WIDTH-JPFUELBAR_W-8.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7148`
pub const JPFUELBAR_X: f32 = SCREEN_WIDTH as f32 - JPFUELBAR_W - 8.0;

/// Raven `#define JPFUELBAR_Y 260.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7149`
pub const JPFUELBAR_Y: f32 = 260.0;

/// Raven `#define EWEBHEALTH_H 100.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7198`
pub const EWEBHEALTH_H: f32 = 100.0;

/// Raven `#define EWEBHEALTH_W 20.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7199`
pub const EWEBHEALTH_W: f32 = 20.0;

/// Raven `#define EWEBHEALTH_X (SCREEN_WIDTH-EWEBHEALTH_W-8.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7200`
pub const EWEBHEALTH_X: f32 = SCREEN_WIDTH as f32 - EWEBHEALTH_W - 8.0;

/// Raven `#define EWEBHEALTH_Y 290.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7201`
pub const EWEBHEALTH_Y: f32 = 290.0;

/// Raven `#define CLFUELBAR_H 100.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7261`
pub const CLFUELBAR_H: f32 = 100.0;

/// Raven `#define CLFUELBAR_W 20.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7262`
pub const CLFUELBAR_W: f32 = 20.0;

/// Raven `#define CLFUELBAR_X (SCREEN_WIDTH-CLFUELBAR_W-8.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7263`
pub const CLFUELBAR_X: f32 = SCREEN_WIDTH as f32 - CLFUELBAR_W - 8.0;

/// Raven `#define CLFUELBAR_Y 260.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7264`
pub const CLFUELBAR_Y: f32 = 260.0;

/// Raven `#define CHATBOX_CUTOFF_LEN 550`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7529`
pub const CHATBOX_CUTOFF_LEN: c_int = 550;

/// Raven `#define CHATBOX_FONT_HEIGHT 20`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7530`
pub const CHATBOX_FONT_HEIGHT: c_int = 20;

/// Raven `#define FALL_FADE_TIME 3000` — how long the black fall-to-death fade
/// takes to reach full opacity. A `q_shared.h` define with no crate-level home.
/// Source: `oracle/codemp/game/q_shared.h:2148`
const FALL_FADE_TIME: c_int = 3000;

// PORT-NOTE: three `cg_local.h` defines with no crate-level home — `cg_drawtools`
// already keeps its own file-local copies of the `NUM_FONT_*` pair (they are
// private there), so cg_draw.c's own users land here, same story as `FONT_SMALL`.
/// Raven `#define ICON_SIZE 48` — the item/powerup HUD icon edge.
/// Source: `oracle/codemp/cgame/cg_local.h:61`
const ICON_SIZE: f32 = 48.0;
/// Raven `#define NUM_FONT_SMALL 2` — number-field font style: small digits.
/// Source: `oracle/codemp/cgame/cg_local.h:71`
const NUM_FONT_SMALL: c_int = 2;
/// Raven `#define NUM_FONT_CHUNKY 3` — number-field font style: wide digits.
/// Source: `oracle/codemp/cgame/cg_local.h:72`
const NUM_FONT_CHUNKY: c_int = 3;
/// Raven `#define ITEM_BLOB_TIME 200` — how long the crosshair swells after a
/// pickup.
/// Source: `oracle/codemp/cgame/cg_local.h:46`
const ITEM_BLOB_TIME: f32 = 200.0;
/// Raven `#define TEAM_OVERLAY_MAXNAME_WIDTH 32`.
/// Source: `oracle/codemp/cgame/cg_local.h:76`
const TEAM_OVERLAY_MAXNAME_WIDTH: c_int = 32;
/// Raven `#define TEAM_OVERLAY_MAXLOCATION_WIDTH 64`.
/// Source: `oracle/codemp/cgame/cg_local.h:77`
const TEAM_OVERLAY_MAXLOCATION_WIDTH: c_int = 64;

// PORT-NOTE: `q_shared.h`'s location cap and the `UI_*` text flags have no
// crate-level home either — `cg_drawtools` keeps its own private `UI_*` copies,
// so cg_draw.c's own users land here, same story as `FONT_SMALL` above.
/// Raven `#define MAX_LOCATIONS 64`.
/// Source: `oracle/codemp/game/q_shared.h:1989`
const MAX_LOCATIONS: c_int = 64;
/// Source: `oracle/codemp/game/q_shared.h:489`
const UI_CENTER: c_int = 0x0000_0001;
/// Source: `oracle/codemp/game/q_shared.h:490`
const UI_RIGHT: c_int = 0x0000_0002;
/// Source: `oracle/codemp/game/q_shared.h:492`
const UI_SMALLFONT: c_int = 0x0000_0010;
/// Source: `oracle/codemp/game/q_shared.h:493`
const UI_BIGFONT: c_int = 0x0000_0020;
/// Source: `oracle/codemp/game/q_shared.h:495`
const UI_DROPSHADOW: c_int = 0x0000_0800;

// PORT-NOTE: `teams.h`'s NPC team constants live in `mp_game`, which cgame does
// not depend on; the one value `CG_DrawCrosshair` needs lands file-local.
/// Raven `NPCTEAM_PLAYER` — also `TEAM_BLUE`.
/// Source: `oracle/codemp/game/teams.h:4-14`
const NPCTEAM_PLAYER: c_int = 2;

// PORT-NOTE: `qfiles.h`'s two font-render bits, the same file-local pair
// `mp_ui`'s `ui_main` keeps — they are already a `mp_engine_qcommon` const
// (`qfiles/font_style.rs`) but cgame has no dependency on that crate.
/// Source: `oracle/codemp/qcommon/qfiles.h:570-571`
const STYLE_DROPSHADOW: u32 = 0x8000_0000;
/// Source: `oracle/codemp/qcommon/qfiles.h:570-571`
const STYLE_BLINK: u32 = 0x4000_0000;

// PORT-NOTE: `bg_vehicles.h`'s impact-damage surface bits. mp_bg carries the
// vehicle *types* but not these `#define`s, so the four pairs `vehDamageData`
// names land here, the same file-local-copy story as `FONT_SMALL` above.
/// Source: `oracle/codemp/game/bg_vehicles.h:432`
const SHIPSURF_DAMAGE_FRONT_LIGHT: i16 = 0;
/// Source: `oracle/codemp/game/bg_vehicles.h:433`
const SHIPSURF_DAMAGE_BACK_LIGHT: i16 = 1;
/// Source: `oracle/codemp/game/bg_vehicles.h:434`
const SHIPSURF_DAMAGE_RIGHT_LIGHT: i16 = 2;
/// Source: `oracle/codemp/game/bg_vehicles.h:435`
const SHIPSURF_DAMAGE_LEFT_LIGHT: i16 = 3;
/// Source: `oracle/codemp/game/bg_vehicles.h:436`
const SHIPSURF_DAMAGE_FRONT_HEAVY: i16 = 4;
/// Source: `oracle/codemp/game/bg_vehicles.h:437`
const SHIPSURF_DAMAGE_BACK_HEAVY: i16 = 5;
/// Source: `oracle/codemp/game/bg_vehicles.h:438`
const SHIPSURF_DAMAGE_RIGHT_HEAVY: i16 = 6;
/// Source: `oracle/codemp/game/bg_vehicles.h:439`
const SHIPSURF_DAMAGE_LEFT_HEAVY: i16 = 7;

/// Raven's `vehDamageData` index enum — which quarter of the vehicle-damage
/// HUD a `CG_DrawVehicleDamage` call is painting.
///
/// Anonymous `enum` in Raven, so plain `const`s.
/// Source: `oracle/codemp/cgame/cg_draw.c:2410-2416`
pub const VEH_DAMAGE_FRONT: usize = 0;
/// Source: `oracle/codemp/cgame/cg_draw.c:2410-2416`
pub const VEH_DAMAGE_BACK: usize = 1;
/// Source: `oracle/codemp/cgame/cg_draw.c:2410-2416`
pub const VEH_DAMAGE_LEFT: usize = 2;
/// Source: `oracle/codemp/cgame/cg_draw.c:2410-2416`
pub const VEH_DAMAGE_RIGHT: usize = 3;

/// Raven `veh_damage_t` — one vehicle-damage HUD zone: the menu item to tint
/// plus the two `brokenLimbs` bits that pick its colour.
///
/// Type definition source: `oracle/codemp/cgame/cg_draw.c:2418-2423`
#[derive(Debug, Clone, Copy)]
pub struct veh_damage_t {
    pub itemName: &'static str,
    pub heavyDamage: i16,
    pub lightDamage: i16,
}

/// Raven `veh_damage_t vehDamageData[4]` — read-only compiled-in data, so a
/// `const` beside its one reader (§C8).
/// Source: `oracle/codemp/cgame/cg_draw.c:2425-2431`
pub const vehDamageData: [veh_damage_t; 4] = [
    veh_damage_t {
        itemName: "vehicle_front",
        heavyDamage: SHIPSURF_DAMAGE_FRONT_HEAVY,
        lightDamage: SHIPSURF_DAMAGE_FRONT_LIGHT,
    },
    veh_damage_t {
        itemName: "vehicle_back",
        heavyDamage: SHIPSURF_DAMAGE_BACK_HEAVY,
        lightDamage: SHIPSURF_DAMAGE_BACK_LIGHT,
    },
    veh_damage_t {
        itemName: "vehicle_left",
        heavyDamage: SHIPSURF_DAMAGE_LEFT_HEAVY,
        lightDamage: SHIPSURF_DAMAGE_LEFT_LIGHT,
    },
    veh_damage_t {
        itemName: "vehicle_right",
        heavyDamage: SHIPSURF_DAMAGE_RIGHT_HEAVY,
        lightDamage: SHIPSURF_DAMAGE_RIGHT_LIGHT,
    },
];

// PORT-NOTE: `ColorIndex(COLOR_RED)`/`ColorIndex(COLOR_GREEN)` — Raven's
// `((c) - '0') & 7` over the `'1'`/`'2'` colour sigils, resolved once here
// because `mp_qshared` exports `g_color_table` but neither macro.
/// Source: `oracle/codemp/game/q_shared.h:1151,1158`
const COLOR_RED_INDEX: usize = 1;
/// Source: `oracle/codemp/game/q_shared.h:1152,1158`
const COLOR_GREEN_INDEX: usize = 2;
/// Source: `oracle/codemp/game/q_shared.h:1153,1158`
const COLOR_YELLOW_INDEX: usize = 3;
/// Source: `oracle/codemp/game/q_shared.h:1154,1158`
const COLOR_BLUE_INDEX: usize = 4;

/// Raven `vec4_t colorTable[CT_MAX]` — the named HUD palette every cgame `.c`
/// indexes with a `ct_table_t`. Never written, so a `const`, not state.
///
/// PORT-NOTE: Raven defines it in `cg_main.c`, but `cg_draw.c` is its first
/// consumer in this port and a wave may only write its own TU's files — later
/// waves import it from here rather than re-declaring it.
/// Source: `oracle/codemp/cgame/cg_main.c:27-109`
pub const colorTable: [vec4_t; ct_table_t::CT_MAX as usize] = [
    [0.0, 0.0, 0.0, 0.0],       // CT_NONE
    [0.0, 0.0, 0.0, 1.0],       // CT_BLACK
    [1.0, 0.0, 0.0, 1.0],       // CT_RED
    [0.0, 1.0, 0.0, 1.0],       // CT_GREEN
    [0.0, 0.0, 1.0, 1.0],       // CT_BLUE
    [1.0, 1.0, 0.0, 1.0],       // CT_YELLOW
    [1.0, 0.0, 1.0, 1.0],       // CT_MAGENTA
    [0.0, 1.0, 1.0, 1.0],       // CT_CYAN
    [1.0, 1.0, 1.0, 1.0],       // CT_WHITE
    [0.75, 0.75, 0.75, 1.0],    // CT_LTGREY
    [0.50, 0.50, 0.50, 1.0],    // CT_MDGREY
    [0.25, 0.25, 0.25, 1.0],    // CT_DKGREY
    [0.15, 0.15, 0.15, 1.0],    // CT_DKGREY2
    [0.810, 0.530, 0.0, 1.0],   // CT_VLTORANGE -- needs values
    [0.810, 0.530, 0.0, 1.0],   // CT_LTORANGE
    [0.610, 0.330, 0.0, 1.0],   // CT_DKORANGE
    [0.402, 0.265, 0.0, 1.0],   // CT_VDKORANGE
    [0.503, 0.375, 0.996, 1.0], // CT_VLTBLUE1
    [0.367, 0.261, 0.722, 1.0], // CT_LTBLUE1
    [0.199, 0.0, 0.398, 1.0],   // CT_DKBLUE1
    [0.160, 0.117, 0.324, 1.0], // CT_VDKBLUE1
    [0.300, 0.628, 0.816, 1.0], // CT_VLTBLUE2 -- needs values
    [0.300, 0.628, 0.816, 1.0], // CT_LTBLUE2
    [0.191, 0.289, 0.457, 1.0], // CT_DKBLUE2
    [0.125, 0.250, 0.324, 1.0], // CT_VDKBLUE2
    [0.796, 0.398, 0.199, 1.0], // CT_VLTBROWN1 -- needs values
    [0.796, 0.398, 0.199, 1.0], // CT_LTBROWN1
    [0.558, 0.207, 0.027, 1.0], // CT_DKBROWN1
    [0.328, 0.125, 0.035, 1.0], // CT_VDKBROWN1
    [0.996, 0.796, 0.398, 1.0], // CT_VLTGOLD1 -- needs values
    [0.996, 0.796, 0.398, 1.0], // CT_LTGOLD1
    [0.605, 0.441, 0.113, 1.0], // CT_DKGOLD1
    [0.386, 0.308, 0.148, 1.0], // CT_VDKGOLD1
    [0.648, 0.562, 0.784, 1.0], // CT_VLTPURPLE1 -- needs values
    [0.648, 0.562, 0.784, 1.0], // CT_LTPURPLE1
    [0.437, 0.335, 0.597, 1.0], // CT_DKPURPLE1
    [0.308, 0.269, 0.375, 1.0], // CT_VDKPURPLE1
    [0.816, 0.531, 0.710, 1.0], // CT_VLTPURPLE2 -- needs values
    [0.816, 0.531, 0.710, 1.0], // CT_LTPURPLE2
    [0.566, 0.269, 0.457, 1.0], // CT_DKPURPLE2
    [0.343, 0.226, 0.316, 1.0], // CT_VDKPURPLE2
    [0.929, 0.597, 0.929, 1.0], // CT_VLTPURPLE3
    [0.570, 0.371, 0.570, 1.0], // CT_LTPURPLE3
    [0.355, 0.199, 0.355, 1.0], // CT_DKPURPLE3
    [0.285, 0.136, 0.230, 1.0], // CT_VDKPURPLE3
    [0.953, 0.378, 0.250, 1.0], // CT_VLTRED1
    [0.953, 0.378, 0.250, 1.0], // CT_LTRED1
    [0.593, 0.121, 0.109, 1.0], // CT_DKRED1
    [0.429, 0.171, 0.113, 1.0], // CT_VDKRED1
    [0.25, 0.0, 0.0, 1.0],      // CT_VDKRED
    [0.70, 0.0, 0.0, 1.0],      // CT_DKRED
    [0.717, 0.902, 1.0, 1.0],   // CT_VLTAQUA
    [0.574, 0.722, 0.804, 1.0], // CT_LTAQUA
    [0.287, 0.361, 0.402, 1.0], // CT_DKAQUA
    [0.143, 0.180, 0.201, 1.0], // CT_VDKAQUA
    [0.871, 0.386, 0.375, 1.0], // CT_LTPINK
    [0.435, 0.193, 0.187, 1.0], // CT_DKPINK
    [0.0, 0.5, 0.5, 1.0],       // CT_LTCYAN
    [0.0, 0.25, 0.25, 1.0],     // CT_DKCYAN
    [0.179, 0.51, 0.92, 1.0],   // CT_LTBLUE3
    [0.199, 0.71, 0.92, 1.0],   // CT_BLUE3 (Raven's comment says CT_LTBLUE3 twice)
    [0.5, 0.05, 0.4, 1.0],      // CT_DKBLUE3
    [0.0, 0.613, 0.097, 1.0],   // CT_HUD_GREEN
    [0.835, 0.015, 0.015, 1.0], // CT_HUD_RED
    [0.567, 0.685, 1.0, 0.75],  // CT_ICON_BLUE
    [0.515, 0.406, 0.507, 1.0], // CT_NO_AMMO_RED
    [1.0, 0.658, 0.062, 1.0],   // CT_HUD_ORANGE
];

/// Raven `UI_ParseAnimationFile` — cgame's passthrough to bg's animation
/// parser, so the `ui_shared.c` copy compiled into cgame has a host to call.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:102-105`
pub fn UI_ParseAnimationFile(
    ctx: &mut CgContext,
    filename: &str,
    animset: *mut animation_t,
    isHumanoid: bool,
) -> c_int {
    let traps = CgBgTraps::new(ctx.engine);
    let mut callbacks = CgGameCallbacks::new(ctx.engine);
    let filename_c = cstr(filename);
    BG_ParseAnimationFile(
        &mut ctx.world.bg_state,
        &traps,
        &mut callbacks,
        filename_c.as_ptr(),
        animset,
        if isHumanoid { qtrue } else { qfalse },
    )
}

/// Raven `MenuFontToHandle`.
///
/// Raven: `FONT_LARGE` returns the medium handle, with the site's own
/// "fixme? Big fonr isn't registered...?" beside it.
/// Source: `oracle/codemp/cgame/cg_draw.c:107-119`
pub fn MenuFontToHandle(cgDC: &DisplayState, iMenuFont: c_int) -> qhandle_t {
    match iMenuFont {
        FONT_SMALL => cgDC.Assets.qhSmallFont,
        FONT_SMALL2 => cgDC.Assets.qhSmall2Font,
        FONT_MEDIUM => cgDC.Assets.qhMediumFont,
        FONT_LARGE => cgDC.Assets.qhMediumFont,
        _ => cgDC.Assets.qhMediumFont,
    }
}

/// Raven `CG_Draw3DModel` — renders one model into a screen rectangle, the
/// 3D-icon path behind `cg_draw3dIcons`.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:467-503`
#[allow(clippy::too_many_arguments)]
pub fn CG_Draw3DModel(
    ctx: &CgContext,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    model: qhandle_t,
    ghoul2: *mut c_void,
    g2radius: c_int,
    skin: qhandle_t,
    origin: vec3_t,
    angles: vec3_t,
) {
    if ctx.world.cvars.cg_draw3dIcons.integer == 0 || ctx.world.cvars.cg_drawIcons.integer == 0 {
        return;
    }

    // Raven's `memset( &refdef, 0, sizeof( refdef ) )` — `refdef_t` is scalars
    // and arrays with no padding, so the zeroed literal is the memset.
    let mut refdef = refdef_t {
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
    };

    let mut ent = refEntity_t::zeroed();
    AnglesToAxis(angles, ent.axis.as_mut_ptr());
    _VectorCopy(origin, &mut ent.origin);
    ent.hModel = model;
    ent.ghoul2 = ghoul2;
    ent.radius = g2radius as f32;
    ent.customSkin = skin;
    ent.renderfx = RF_NOSHADOW; // no stencil shadows

    refdef.rdflags = RDF_NOWORLDMODEL;

    // Raven: `AxisClear( refdef.viewaxis )` — the identity basis. `AxisClear`
    // takes a raw `*mut vec3_t` and needs `unsafe` to call on a `[vec3_t; 3]`,
    // which this wave may not write, so the three rows land inline instead.
    refdef.viewaxis[0] = [1.0, 0.0, 0.0];
    refdef.viewaxis[1] = [0.0, 1.0, 0.0];
    refdef.viewaxis[2] = [0.0, 0.0, 1.0];

    refdef.fov_x = 30.0;
    refdef.fov_y = 30.0;

    refdef.x = x as c_int;
    refdef.y = y as c_int;
    refdef.width = w as c_int;
    refdef.height = h as c_int;

    refdef.time = ctx.world.cg.time;

    trap::R_ClearScene(ctx.engine);
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
    trap::R_RenderScene(ctx.engine, &refdef);
}

/// Raven `DrawAmmo` — a dead stub: it computes a corner position and returns
/// without drawing anything.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:598-605`
pub fn DrawAmmo() {
    // Raven's `x = SCREEN_WIDTH-80; y = SCREEN_HEIGHT-80;` go nowhere — the
    // drawing body was cut and the locals are unused. Nothing observable here.
}

/// Raven `ForcePower_Valid` — can power `i` show on the force HUD? The four
/// always-on powers never do.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1395-1411`
pub fn ForcePower_Valid(world: &CgWorld, i: c_int) -> bool {
    if i == FP_LEVITATION || i == FP_SABER_OFFENSE || i == FP_SABER_DEFENSE || i == FP_SABERTHROW {
        return false;
    }

    // Raven derefs `cg.snap` unguarded; a null there is UB, so the port takes
    // the not-known answer instead (§19).
    // no snapshot yet means no known powers
    let Some(snap) = world.cg.snap_ref() else {
        return false;
    };
    (snap.ps.fd.forcePowersKnown & (1 << i)) != 0
}

/// Raven `CG_CheckTargetVehicle` — which vehicle the targeting HUD is on, and
/// how opaque to draw it. `Some((entity number, alpha))` is Raven's
/// `qtrue` + the two out-params.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1793-1866`
pub fn CG_CheckTargetVehicle(world: &mut CgWorld) -> Option<(usize, f32)> {
    let mut targetNum: c_int = ENTITYNUM_NONE;
    let mut targetVeh: Option<usize> = None;

    let mut alpha: f32 = 1.0;

    // FIXME (Raven): need to clear all of these when you die?
    if world.cg.predictedPlayerState.rocketLockIndex < ENTITYNUM_WORLD {
        targetNum = world.cg.predictedPlayerState.rocketLockIndex;
    } else if world.cg.crosshairVehNum < ENTITYNUM_WORLD
        && world.cg.time - world.cg.crosshairVehTime < 3000
    {
        // crosshair was on a vehicle in the last 3 seconds
        targetNum = world.cg.crosshairVehNum;
    } else if world.cg.crosshairClientNum < ENTITYNUM_WORLD {
        targetNum = world.cg.crosshairClientNum;
    }

    // real client. Raven's `targetNum < MAX_CLIENTS` also admits negatives and
    // then indexes `cg_entities[targetNum]` out of bounds; the port takes the
    // not-a-client path there instead (§19).
    if (0..MAX_CLIENTS_I32).contains(&targetNum)
        && world.entity(targetNum as usize).currentState.m_iVehicleNum >= MAX_CLIENTS_I32
    {
        // in a vehicle
        targetNum = world.entity(targetNum as usize).currentState.m_iVehicleNum;
    }

    if targetNum < ENTITYNUM_WORLD && targetNum >= MAX_CLIENTS_I32 {
        let cent = world.entity(targetNum as usize);
        // DEFERRED: Vehicle_t::m_pVehicleInfo->type == VH_FIGHTER — cgame owns
        // no `Vehicle_t` pool yet, so DEC-46.2's `Option<VehicleId>` can only
        // answer the presence half of Raven's test; the fighter-vs-anything
        // -else half is unavailable and this accepts every vehicle class.
        // Source: oracle/codemp/cgame/cg_draw.c:1831-1834
        if cent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
            && cent.m_pVehicle.is_some()
        {
            // it's a vehicle
            world.draw.cg_targVeh = targetNum;
            world.draw.cg_targVehLastTime = world.cg.time;
            alpha = 1.0;
        }
        // PORT-NOTE: Raven's `else { targetVeh = NULL; }` assigns the block's
        // own shadowing `centity_t *targetVeh`, never the outer one, so the
        // outer stays NULL on both arms and the fade block below is the only
        // thing that ever sets it. Preserved: nothing is assigned here.
    }

    if targetVeh.is_none()
        && world.draw.cg_targVehLastTime != 0
        && world.cg.time - world.draw.cg_targVehLastTime < 3000
    {
        targetVeh = Some(world.draw.cg_targVeh as usize);
        if world.cg.time - world.draw.cg_targVehLastTime < 1000 {
            // stay at full alpha for 1 sec after lose them from crosshair
            alpha = 1.0;
        } else {
            // fade out over 2 secs
            alpha = 1.0 - ((world.cg.time - world.draw.cg_targVehLastTime - 1000) as f32 / 2000.0);
        }
    }

    targetVeh.map(|n| (n, alpha))
}

/// Raven `CG_AddLagometerFrameInfo` — adds the current interpolate/extrapolate
/// bar for this frame.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4161-4167`
pub fn CG_AddLagometerFrameInfo(world: &mut CgWorld) {
    let offset = world.cg.time - world.cg.latestSnapshotTime;
    let lag = &mut world.draw.lagometer;
    lag.frameSamples[(lag.frameCount & (LAG_SAMPLES as c_int - 1)) as usize] = offset;
    lag.frameCount += 1;
}

/// Raven `CG_AddLagometerSnapshotInfo` — logs a received snapshot's ping and
/// flags. `None` is Raven's NULL, a dropped packet.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4179-4191`
/// `snap` is the `(ping, snapFlags)` pair — all this fn ever reads of Raven's
/// `snapshot_t *snap` — so callers borrowing the snapshot out of `cg` don't
/// have to copy the 139 KB struct to release the borrow.
pub fn CG_AddLagometerSnapshotInfo(world: &mut CgWorld, snap: Option<(c_int, c_int)>) {
    let lag = &mut world.draw.lagometer;
    let i = (lag.snapshotCount & (LAG_SAMPLES as c_int - 1)) as usize;

    // dropped packet
    let Some((ping, snapFlags)) = snap else {
        lag.snapshotSamples[i] = -1;
        lag.snapshotCount += 1;
        return;
    };

    // add this snapshot's info
    lag.snapshotSamples[i] = ping;
    lag.snapshotFlags[i] = snapFlags;
    lag.snapshotCount += 1;
}

/// Raven `CG_DrawSiegeMessage` — hands the siege briefing text to the ui module
/// and opens the matching menu.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4353-4368`
pub fn CG_DrawSiegeMessage(ctx: &CgContext, str: &str, objectiveScreen: c_int) {
    // Raven's `if (!( trap_Key_GetCatcher() & KEYCATCH_UI ))` guard is
    // commented out at the site, so the block below runs unconditionally.
    trap::OpenUIMenu(ctx.engine, UIMENU_CLOSEALL);
    trap::Cvar_Set(ctx.engine, "cg_siegeMessage", str);
    if objectiveScreen != 0 {
        trap::OpenUIMenu(ctx.engine, UIMENU_SIEGEOBJECTIVES);
    } else {
        trap::OpenUIMenu(ctx.engine, UIMENU_SIEGEMESSAGE);
    }
}

/// Raven `CG_CenterPrint` — latches the centerprint text and counts its lines
/// so the drawer can vertically center it.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4398-4415`
pub fn CG_CenterPrint(world: &mut CgWorld, str: &str, y: c_int, charWidth: c_int) {
    let bytes = string_to_latin1(str);
    let destsize = world.cg.centerPrint.len();
    Q_strncpyzBytes(&mut world.cg.centerPrint, &bytes, destsize);

    world.cg.centerPrintTime = world.cg.time;
    world.cg.centerPrintY = y;
    world.cg.centerPrintCharWidth = charWidth;

    // count the number of lines for centering
    let mut lines = 1;
    for &c in world.cg.centerPrint.iter() {
        if c == 0 {
            break;
        }
        if c == b'\n' as c_char {
            lines += 1;
        }
    }
    world.cg.centerPrintLines = lines;
}

/// Raven `CG_LerpCrosshairPos` — clamps how far the crosshair may travel in one
/// frame, then latches the new position.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4806-4845`
pub fn CG_LerpCrosshairPos(world: &mut CgWorld, x: &mut f32, y: &mut f32) {
    if world.draw.cg_crosshairPrevPosX != 0.0 {
        // blend from old pos
        let mut maxMove = 30.0 * (world.cg.frametime as f32 / 500.0) * 640.0 / 480.0;
        let xDiff = *x - world.draw.cg_crosshairPrevPosX;
        if xDiff.abs() > CRAZY_CROSSHAIR_MAX_ERROR_X {
            maxMove = CRAZY_CROSSHAIR_MAX_ERROR_X;
        }
        if xDiff > maxMove {
            *x = world.draw.cg_crosshairPrevPosX + maxMove;
        } else if xDiff < -maxMove {
            *x = world.draw.cg_crosshairPrevPosX - maxMove;
        }
    }
    world.draw.cg_crosshairPrevPosX = *x;

    if world.draw.cg_crosshairPrevPosY != 0.0 {
        // blend from old pos
        let mut maxMove = 30.0 * (world.cg.frametime as f32 / 500.0);
        let yDiff = *y - world.draw.cg_crosshairPrevPosY;
        // PORT-NOTE: Raven tests against the Y error but then clamps with the X
        // one (`CRAZY_CROSSHAIR_MAX_ERROR_X`). Kept as written — that mixed
        // pair is the shipped behavior.
        if yDiff.abs() > CRAZY_CROSSHAIR_MAX_ERROR_Y {
            maxMove = CRAZY_CROSSHAIR_MAX_ERROR_X;
        }
        if yDiff > maxMove {
            *y = world.draw.cg_crosshairPrevPosY + maxMove;
        } else if yDiff < -maxMove {
            *y = world.draw.cg_crosshairPrevPosY - maxMove;
        }
    }
    world.draw.cg_crosshairPrevPosY = *y;
}

/// Raven `CG_WorldCoordToScreenCoordFloat` — projects a world point into
/// virtual 640x480 screen space. `None` is Raven's qfalse, the point behind the
/// viewer, where the out-params stay untouched.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5270-5309`
pub fn CG_WorldCoordToScreenCoordFloat(world: &CgWorld, worldCoord: vec3_t) -> Option<(f32, f32)> {
    // Raven: did it this way because most draw functions expect virtual
    // 640x480 coords and adjust them for current resolution
    let xcenter: f32 = 640.0 / 2.0;
    let ycenter: f32 = 480.0 / 2.0;

    let mut vfwd: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut vup: vec3_t = [0.0; 3];
    AngleVectors(
        world.cg.refdef.viewangles,
        Some(&mut vfwd),
        Some(&mut vright),
        Some(&mut vup),
    );

    let mut local: vec3_t = [0.0; 3];
    _VectorSubtract(worldCoord, world.cg.refdef.vieworg, &mut local);

    let transformed: vec3_t = [
        _DotProduct(local, vright),
        _DotProduct(local, vup),
        _DotProduct(local, vfwd),
    ];

    // Make sure Z is not negative.
    if transformed[2] < 0.01 {
        return None;
    }

    let xzi = xcenter / transformed[2] * (96.0 / world.cg.refdef.fov_x);
    let yzi = ycenter / transformed[2] * (102.0 / world.cg.refdef.fov_y);

    Some((
        xcenter + xzi * transformed[0],
        ycenter - yzi * transformed[1],
    ))
}

/// Raven `CG_InFighter` — am I riding a fighter?
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5602-5616`
pub fn CG_InFighter(world: &CgWorld) -> bool {
    if world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        // I'm in a vehicle
        let vehCent = world.entity(world.cg.predictedPlayerState.m_iVehicleNum as usize);
        // DEFERRED: Vehicle_t::m_pVehicleInfo->type == VH_FIGHTER — cgame owns
        // no `Vehicle_t` pool yet, so only the presence half of Raven's test
        // survives and this answers true for any vehicle, fighter or not.
        // Source: oracle/codemp/cgame/cg_draw.c:5608-5610
        if vehCent.m_pVehicle.is_some() {
            // I'm in a fighter
            return true;
        }
    }
    false
}

/// Raven `CG_InATST` — am I riding an AT-ST (a walker)?
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5618-5632`
pub fn CG_InATST(world: &CgWorld) -> bool {
    if world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        // I'm in a vehicle
        let vehCent = world.entity(world.cg.predictedPlayerState.m_iVehicleNum as usize);
        // DEFERRED: Vehicle_t::m_pVehicleInfo->type == VH_WALKER — same missing
        // `Vehicle_t` pool as `CG_InFighter`; the walker-vs-anything-else half
        // of Raven's test is unavailable, so this answers true for any vehicle.
        // Source: oracle/codemp/cgame/cg_draw.c:5624-5626
        if vehCent.m_pVehicle.is_some() {
            // I'm in an atst
            return true;
        }
    }
    false
}

/// Raven `CG_IsDurationPower` — the powers whose HUD icon runs for as long as
/// the power is up, rather than blipping once.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5683-5697`
pub fn CG_IsDurationPower(power: c_int) -> bool {
    power == FP_HEAL
        || power == FP_SPEED
        || power == FP_TELEPATHY
        || power == FP_RAGE
        || power == FP_PROTECT
        || power == FP_ABSORB
        || power == FP_SEE
}

/// Raven `CG_CalcEWebMuzzlePoint` — the e-web's cannon-flash bolt, pulled back
/// into the bbox so the shot never starts inside geometry. The out-vectors are
/// left untouched when the bolt is missing, as in Raven.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6043-6064`
pub fn CG_CalcEWebMuzzlePoint(
    ctx: &CgContext,
    cent_num: usize,
    start: &mut vec3_t,
    d_f: &mut vec3_t,
    d_rt: &mut vec3_t,
    d_up: &mut vec3_t,
) {
    let cent = ctx.world.entity(cent_num);
    let bolt = trap::G2API_AddBolt(ctx.engine, cent.ghoul2, 0, "*cannonflash");

    debug_assert!(bolt != -1);

    if bolt != -1 {
        let mut boltMatrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };

        trap::G2API_GetBoltMatrix_NoRecNoRot(
            ctx.engine,
            cent.ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &cent.lerpAngles,
            &cent.lerpOrigin,
            ctx.world.cg.time,
            None,
            &cent.modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, start);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::NEGATIVE_X as c_int, d_f);

        // these things start the shot a little inside the bbox to assure not starting in something solid
        let origin = *start;
        let forward = *d_f;
        _VectorMA(origin, -16.0, forward, start);

        // I guess
        VectorClear(d_rt); // don't really need this, do we?
        VectorClear(d_up); // don't really need this, do we?
    }
}

/// Raven `CG_SanitizeString` — strips colour codes and control characters off a
/// name, truncating where the ui truncates.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6288-6326`
pub fn CG_SanitizeString(r#in: &str) -> String {
    // Latin-1 free text, so one char is one of Raven's bytes and the 128-byte
    // cutoff below counts the same things it does.
    let chars: Vec<char> = r#in.chars().collect();
    let mut out = String::new();

    let mut i = 0usize;
    while i < chars.len() {
        if i >= 128 - 1 {
            // the ui truncates the name here..
            break;
        }

        if chars[i] == '^' {
            // Raven reads `in[i+1]` past the last char, which is the NUL
            // terminator and so never a digit — the bounds test takes the same
            // "just skip the ^" arm.
            if i + 1 < chars.len() && chars[i + 1] >= '0' && chars[i + 1] <= '9' {
                // only skip it if there's a number after it for the color
                i += 2;
                continue;
            } else {
                // just skip the ^
                i += 1;
                continue;
            }
        }

        // Raven's `in[i] < 32` widens a signed `char`, so 0x80-0xFF read
        // negative and strip here too; match that with the signed cast.
        if (chars[i] as u8 as i8) < 32 {
            i += 1;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Raven `CG_DrawAmmoWarning` — the whole body sits inside `#if 0`, so this
/// draws nothing.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6888-6914`
pub fn CG_DrawAmmoWarning() {
    // Raven's "LOW AMMO WARNING" / "OUT OF AMMO" strings are compiled out.
}

/// Raven `CG_DrawTimedMenus` — closes the voice-chat menu 2.5 seconds after it
/// opened.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7073-7082`
pub fn CG_DrawTimedMenus(ctx: &mut CgContext, menus: &mut MenuSystem, ds: &DisplayState) {
    if ctx.world.cg.voiceTime != 0 {
        let t = ctx.world.cg.time - ctx.world.cg.voiceTime;
        if t > 2500 {
            // PORT-NOTE: `ctx` is the `DisplayContext` the menu framework calls
            // back through, on ui's `UiContext` pattern; `impl DisplayContext
            // for CgContext` lands with the C5 waves (`world/cg_context.rs`).
            Menus_CloseByName(menus, ds, ctx, "voiceMenu");
            trap::Cvar_Set(ctx.engine, "cl_conXOffset", "0");
            ctx.world.cg.voiceTime = 0;
        }
    }
}

/// Raven `CG_ChatBox_StrInsert` — splices `str` into `buffer` at `place`.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7534-7554`
pub fn CG_ChatBox_StrInsert(buffer: &mut String, place: usize, str: &str) {
    // Latin-1 free text again — char positions are Raven's byte positions, so
    // `place` indexes chars here.
    let mut chars: Vec<char> = buffer.chars().collect();
    // Raven's shift loop (`while (i >= place) ...`) never runs when `place` is
    // past the end, so the terminator at `buffer[len]` is never touched and the
    // insert lands past it - the visible string comes out unchanged. In-bounds
    // and defined, so match it rather than appending (§A2).
    if place > chars.len() {
        return;
    }
    chars.splice(place..place, str.chars());
    *buffer = chars.into_iter().collect();
    // PORT-NOTE: Raven also writes a stray NUL at `strlen+insLen+1`, one past
    // the real new end. The shift loop terminates the string correctly at
    // `strlen+insLen` first, so that byte is never read — nothing to port.
}

/// Raven `CG_DrawTourneyScoreboard` — an empty body in the shipped source.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:8518-8519`
pub fn CG_DrawTourneyScoreboard() {}

/// Raven `CG_Text_Width` — how wide `text` paints in the menu font.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:123-128`
pub fn CG_Text_Width(
    ctx: &CgContext,
    cgDC: &DisplayState,
    text: &str,
    scale: f32,
    iMenuFont: c_int,
) -> c_int {
    let iFontIndex = MenuFontToHandle(cgDC, iMenuFont);

    trap::R_Font_StrLenPixels(ctx.engine, text, iFontIndex, scale)
}

/// Raven `CG_Text_Height` — the font's line height; `text` is along for the
/// signature only, the engine never looks at it.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:130-135`
pub fn CG_Text_Height(
    ctx: &CgContext,
    cgDC: &DisplayState,
    _text: &str,
    scale: f32,
    iMenuFont: c_int,
) -> c_int {
    let iFontIndex = MenuFontToHandle(cgDC, iMenuFont);

    trap::R_Font_HeightPixels(ctx.engine, iFontIndex, scale)
}

/// Raven `CG_Text_Paint` — the menu framework's text slot; `adjust` is Raven's
/// dead per-char kerning argument.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:138-162`
#[allow(clippy::too_many_arguments)]
pub fn CG_Text_Paint(
    ctx: &CgContext,
    cgDC: &DisplayState,
    x: f32,
    y: f32,
    scale: f32,
    color: vec4_t,
    text: &str,
    _adjust: f32,
    limit: c_int,
    style: c_int,
    iMenuFont: c_int,
) {
    let iFontIndex = MenuFontToHandle(cgDC, iMenuFont);

    let iStyleOR: c_int = match style {
        ITEM_TEXTSTYLE_NORMAL => 0,                           // JK2 normal text
        ITEM_TEXTSTYLE_BLINK => STYLE_BLINK as c_int,         // JK2 fast blinking
        ITEM_TEXTSTYLE_PULSE => STYLE_BLINK as c_int,         // JK2 slow pulsing
        ITEM_TEXTSTYLE_SHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow ( need a color for this )
        ITEM_TEXTSTYLE_OUTLINED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow ( need a color for this )
        ITEM_TEXTSTYLE_OUTLINESHADOWED => STYLE_DROPSHADOW as c_int, // JK2 drop shadow ( need a color for this )
        ITEM_TEXTSTYLE_SHADOWEDMORE => STYLE_DROPSHADOW as c_int, // JK2 drop shadow ( need a color for this )
        // Raven's `switch` has no default and `iStyleOR` starts at 0.
        _ => 0,
    };

    trap::R_Font_DrawString(
        ctx.engine,
        x as c_int,                          // int ox
        y as c_int,                          // int oy
        text,                                // const char *text
        &color,                              // paletteRGBA_c c
        iStyleOR | iFontIndex,               // const int iFontHandle
        if limit == 0 { -1 } else { limit }, // iCharLimit (-1 = none)
        scale,                               // const float scale = 1.0f
    );
}

/// Raven `CG_DrawHead` — a client's head icon in a screen rect. `headAngles`
/// is a leftover of the 3D-head version and is never read.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:512-530`
pub fn CG_DrawHead(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    clientNum: c_int,
    _headAngles: vec3_t,
) {
    if clientNum >= MAX_CLIENTS_I32 {
        // npc?
        return;
    }

    let modelIcon = ctx.world.cgs.clientinfo[clientNum as usize].modelIcon;
    CG_DrawPic(ctx, x, y, w, h, modelIcon);

    // if they are deferred, draw a cross out
    if ctx.world.cgs.clientinfo[clientNum as usize].deferred != qfalse {
        let deferShader = ctx.world.cgs.media.deferShader;
        CG_DrawPic(ctx, x, y, w, h, deferShader);
    }
}

/// Raven `CG_DrawFlagModel` — the CTF flag as a spinning 3D model, or as a flat
/// item icon when 3D icons are off.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:539-591`
pub fn CG_DrawFlagModel(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    team: c_int,
    force2D: bool,
) {
    if !force2D && ctx.world.cvars.cg_draw3dIcons.integer != 0 {
        let mut angles: vec3_t = [0.0; 3];
        VectorClear(&mut angles);

        let cm = ctx.world.cgs.media.redFlagModel;

        // offset the origin y and z to center the flag
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        trap::R_ModelBounds(ctx.engine, cm, &mut mins, &mut maxs);

        let mut origin: vec3_t = [0.0; 3];
        origin[2] = (-0.5 * (mins[2] + maxs[2]) as f64) as f32;
        origin[1] = (0.5 * (mins[1] + maxs[1]) as f64) as f32;

        // calculate distance so the flag nearly fills the box
        // assume heads are taller than wide
        let len = (0.5 * (maxs[2] - mins[2]) as f64) as f32;
        origin[0] = (len as f64 / 0.268) as f32; // len / tan( fov/2 )

        angles[YAW] = (60.0 * (ctx.world.cg.time as f64 / 2000.0).sin()) as f32;

        let handle = if team == TEAM_RED {
            ctx.world.cgs.media.redFlagModel
        } else if team == TEAM_BLUE {
            ctx.world.cgs.media.blueFlagModel
        } else if team == TEAM_FREE {
            0 //cgs.media.neutralFlagModel;
        } else {
            return;
        };
        CG_Draw3DModel(ctx, x, y, w, h, handle, null_mut(), 0, 0, origin, angles);
    } else if ctx.world.cvars.cg_drawIcons.integer != 0 {
        let item = if team == TEAM_RED {
            BG_FindItemForPowerup(PW_REDFLAG)
        } else if team == TEAM_BLUE {
            BG_FindItemForPowerup(PW_BLUEFLAG)
        } else if team == TEAM_FREE {
            BG_FindItemForPowerup(PW_NEUTRALFLAG)
        } else {
            return;
        };
        if let Some(item) = item {
            // `ITEM_INDEX(item)` is the item's slot in `bg_itemlist`.
            let icon = ctx.world.cg_items[item.modelindex() as usize].icon;
            CG_DrawPic(ctx, x, y, w, h, icon);
        }
    }
}

/// Raven `CG_DrawSaberStyle` — lights the fast/medium/strong lamp on the saber
/// HUD for the style we are currently in.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:841-921`
pub fn CG_DrawSaberStyle(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    centNum: usize,
    menuHUD: Option<MenuId>,
) {
    if ctx.world.entity(centNum).currentState.weapon == 0 {
        // We don't have a weapon right now
        return;
    }

    if ctx.world.entity(centNum).currentState.weapon != WP_SABER {
        return;
    }

    // Can we find the menu?
    if menuHUD.is_none() {
        return;
    }

    // draw the current saber style in this window
    let itemName = match ctx.world.cg.predictedPlayerState.fd.saberDrawAnimLevel {
        // 1: FORCE_LEVEL_1, 5: FORCE_LEVEL_5 (Tavion)
        1 | 5 => "saberstyle_fast",
        // 2: FORCE_LEVEL_2, 6: SS_DUAL, 7: SS_STAFF
        2 | 6 | 7 => "saberstyle_medium",
        // 3: FORCE_LEVEL_3, 4: FORCE_LEVEL_4 (Desann)
        3 | 4 => "saberstyle_strong",
        // Raven's `switch` has no default arm — nothing is drawn.
        _ => return,
    };

    if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, itemName) {
        let tint = ctx.world.draw.hudTintColor;
        trap::R_SetColor(ctx.engine, tint.as_ref());

        let rect = menus.item(focusItem).window.rect;
        let background = menus.item(focusItem).window.background;
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }
}

/// Raven `CG_DrawVehicleShields` — the vehicle HUD's shield tic bar; returns
/// the shield fraction the caller uses to flash the rest of the HUD.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxShields` is
/// `veh->m_pVehicle->m_pVehicleInfo->shields`, and DEC-46.2's
/// `Option<VehicleId>` answers presence only ("ported code only tests
/// presence" until the pool lands, `local/vehicle_id.rs`). Both the tic loop
/// and the returned percentage hang off that number, so only the background
/// pic — everything Raven does before the read — is transcribed.
/// Source: `oracle/codemp/cgame/cg_draw.c:1895-1937`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1873-1938`
pub fn CG_DrawVehicleShields(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) -> f32 {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "armorbackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleShields
    // Source: oracle/codemp/cgame/cg_draw.c:1895-1937
    todo!("CG_DrawVehicleShields shield tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:1895-1937")
}

/// Raven `CG_DrawVehicleAmmo` — the single-weapon vehicle ammo tic bar.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxAmmo` is
/// `m_pVehicleInfo->weapon[0].ammoMax`; see [`CG_DrawVehicleShields`] for the
/// same blocker. The background pic is everything Raven does before the read.
/// Source: `oracle/codemp/cgame/cg_draw.c:1963-2009`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1942-2010`
pub fn CG_DrawVehicleAmmo(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "ammobackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleAmmo
    // Source: oracle/codemp/cgame/cg_draw.c:1963-2009
    todo!("CG_DrawVehicleAmmo ammo tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:1963-2009")
}

/// Raven `CG_DrawVehicleAmmoUpper` — the upper weapon's ammo tic bar.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxAmmo` is
/// `m_pVehicleInfo->weapon[0].ammoMax`; see [`CG_DrawVehicleShields`].
/// Source: `oracle/codemp/cgame/cg_draw.c:2034-2080`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2013-2081`
pub fn CG_DrawVehicleAmmoUpper(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "ammoupperbackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleAmmoUpper
    // Source: oracle/codemp/cgame/cg_draw.c:2034-2080
    todo!("CG_DrawVehicleAmmoUpper ammo tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2034-2080")
}

/// Raven `CG_DrawVehicleAmmoLower` — the lower weapon's ammo tic bar.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxAmmo` is
/// `m_pVehicleInfo->weapon[1].ammoMax`; see [`CG_DrawVehicleShields`].
/// Source: `oracle/codemp/cgame/cg_draw.c:2106-2152`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2084-2153`
pub fn CG_DrawVehicleAmmoLower(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "ammolowerbackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleAmmoLower
    // Source: oracle/codemp/cgame/cg_draw.c:2106-2152
    todo!("CG_DrawVehicleAmmoLower ammo tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2106-2152")
}

/// Raven `CG_DrawVehicleTurboRecharge` — the turbo bar filling back up.
///
/// DEFERRED: `Vehicle_t` referent pool — the whole `if (item)` body reads
/// `veh->m_pVehicle->m_iTurboTime` and `m_pVehicleInfo->turboRecharge`, which
/// DEC-46.2's presence-only id cannot supply; see [`CG_DrawVehicleShields`].
/// With no such item in the hud, Raven does nothing and so does this.
/// Source: `oracle/codemp/cgame/cg_draw.c:2165-2193`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2156-2194`
pub fn CG_DrawVehicleTurboRecharge(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if Menu_FindItemByName(menus, menuHUD, "turborecharge").is_some() {
        let _ = (ctx, vehNum);
        //TODO: Port CG_DrawVehicleTurboRecharge
        // Source: oracle/codemp/cgame/cg_draw.c:2165-2193
        todo!("CG_DrawVehicleTurboRecharge bar — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2165-2193")
    }
}

/// Raven `CG_DrawVehicleWeaponsLinked` — the "weapons linked" lamp, plus the
/// one-shot chirp when the link state flips.
///
/// DEFERRED: `Vehicle_t` referent pool — Raven's first arm is
/// `m_pVehicleInfo->weapon[0/1].linkable == 2` ("weapon is always linked"),
/// unanswerable through DEC-46.2's presence-only id. Taking the else arm means
/// no vehicle counts as always-linked and the networked
/// `predictedVehicleState.vehWeaponsLinked` bit is the only source until the
/// pool lands.
/// Source: `oracle/codemp/cgame/cg_draw.c:2200-2205`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2197-2249`
pub fn CG_DrawVehicleWeaponsLinked(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    _vehNum: usize,
) {
    let mut drawLink = false;

    //MP way:
    //must get sent over network
    if ctx.world.cg.predictedVehicleState.vehWeaponsLinked != qfalse {
        drawLink = true;
    }
    //NOTE: the SP way — cheating it off `veh->gent->m_pVehicle->weaponStatus[]`
    //— is commented out at the Raven site.

    if ctx.world.draw.cg_drawLink != drawLink {
        // state changed, play sound
        ctx.world.draw.cg_drawLink = drawLink;
        let sfx = trap::S_RegisterSound(ctx.engine, "sound/vehicles/common/linkweaps.wav");
        let clientNum = ctx.world.cg.predictedPlayerState.clientNum;
        trap::S_StartSound(ctx.engine, None, clientNum, CHAN_LOCAL, sfx);
    }

    if drawLink {
        if let Some(item) = Menu_FindItemByName(menus, menuHUD, "weaponslinked") {
            let rect = menus.item(item).window.rect;

            trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_CYAN as usize]));

            let whiteShader = ctx.world.cgs.media.whiteShader;
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, whiteShader);
        }
    }
}

/// Raven `CG_DrawVehicleSpeed` — the speed tic bar, flashing red in turbo.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxSpeed` is
/// `m_pVehicleInfo->speedMax` and the turbo flash reads `m_iTurboTime`; see
/// [`CG_DrawVehicleShields`]. The background pic is everything Raven does
/// before the read.
/// Source: `oracle/codemp/cgame/cg_draw.c:2272-2341`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2251-2342`
pub fn CG_DrawVehicleSpeed(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "speedbackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleSpeed
    // Source: oracle/codemp/cgame/cg_draw.c:2272-2341
    todo!("CG_DrawVehicleSpeed speed tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2272-2341")
}

/// Raven `CG_DrawVehicleArmor` — the vehicle hull (armor) tic bar.
///
/// DEFERRED: `Vehicle_t` referent pool — `maxArmor` is
/// `m_pVehicleInfo->armor`; see [`CG_DrawVehicleShields`]. Raven reads it
/// before painting the background, but the read makes no engine call, so
/// painting first and blocking after leaves the trap order untouched.
/// Source: `oracle/codemp/cgame/cg_draw.c:2352-2407`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2344-2408`
pub fn CG_DrawVehicleArmor(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    menuHUD: Option<MenuId>,
    vehNum: usize,
) {
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "shieldbackground") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let _ = vehNum;
    //TODO: Port CG_DrawVehicleArmor
    // Source: oracle/codemp/cgame/cg_draw.c:2352-2407
    todo!("CG_DrawVehicleArmor armor tics — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2352-2407")
}

/// Raven `CG_DrawVehicleDamage` — tints one quarter of the vehicle-damage
/// silhouette green/yellow/red/grey from the `brokenLimbs` bits.
///
/// DEFERRED: `Vehicle_t` referent pool — the silhouette handle itself is
/// `m_pVehicleInfo->iconFront/Back/Left/RightHandle`; see
/// [`CG_DrawVehicleShields`]. The colour pick and `trap_R_SetColor` are
/// everything Raven does before that switch.
/// Source: `oracle/codemp/cgame/cg_draw.c:2465-2489`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2434-2491`
pub fn CG_DrawVehicleDamage(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    vehNum: usize,
    brokenLimbs: c_int,
    menuHUD: Option<MenuId>,
    alpha: f32,
    index: usize,
) {
    if Menu_FindItemByName(menus, menuHUD, vehDamageData[index].itemName).is_some() {
        let heavy = 1 << vehDamageData[index].heavyDamage as c_int;
        let light = 1 << vehDamageData[index].lightDamage as c_int;

        let colorI = if brokenLimbs & heavy != 0 {
            if brokenLimbs & light != 0 {
                ct_table_t::CT_DKGREY
            } else {
                ct_table_t::CT_RED
            }
        } else if brokenLimbs & light != 0 {
            ct_table_t::CT_YELLOW
        } else {
            ct_table_t::CT_GREEN
        };

        let mut color = colorTable[colorI as usize];
        color[3] = alpha;
        trap::R_SetColor(ctx.engine, Some(&color));

        let _ = vehNum;
        //TODO: Port CG_DrawVehicleDamage
        // Source: oracle/codemp/cgame/cg_draw.c:2465-2489
        todo!("CG_DrawVehicleDamage silhouette handle — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2465-2489")
    }
}

/// Raven `CG_DrawTeamBackground` — a translucent team-coloured wash behind a
/// HUD panel; the `trap_R_SetColor`/`teamStatusBar` pair either side of the
/// fill is commented out at the Raven site.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2776-2797`
pub fn CG_DrawTeamBackground(
    ctx: &mut CgContext,
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
    alpha: f32,
    team: c_int,
) {
    let mut hcolor: vec4_t = [0.0; 4];

    hcolor[3] = alpha;
    if team == TEAM_RED {
        hcolor[0] = 1.0;
        hcolor[1] = 0.2;
        hcolor[2] = 0.2;
    } else if team == TEAM_BLUE {
        hcolor[0] = 0.2;
        hcolor[1] = 0.2;
        hcolor[2] = 1.0;
    } else {
        return;
    }

    CG_FillRect(ctx, x as f32, y as f32, w as f32, h as f32, &hcolor);
    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_DrawRadar` — the vehicle/siege radar disc: every entity the
/// server flagged as a radar object, plus asteroid-impact and missile-lock
/// alarms. Returns the y below the disc so the HUD stacker can continue.
///
/// DEFERRED: `Vehicle_t` referent pool — the `ET_NPC` arm draws
/// `m_pVehicleInfo->radarIconHandle`, unavailable through DEC-46.2's
/// presence-only id, so vehicle blips are skipped entirely (Raven skips them
/// too when that handle is 0). Every other arm is complete.
/// Source: `oracle/codemp/cgame/cg_draw.c:3392-3454`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3179-3738`
pub fn CG_DrawRadar(ctx: &mut CgContext, y: f32) -> f32 {
    let xOffset: c_int = 0;

    // §F19: Raven's own `if (!cg.snap) return y;`.
    let (snapHealth, snapClientNum, snapPersTeam) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return y;
        };
        (
            snap.ps.stats[STAT_HEALTH as usize],
            snap.ps.clientNum,
            snap.ps.persistant[PERS_TEAM as usize],
        )
    };

    // Make sure the radar should be showing
    if snapHealth <= 0 {
        return y;
    }

    if (ctx.world.cg.predictedPlayerState.pm_flags & PMF_FOLLOW) != 0
        || ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR
    {
        return y;
    }

    let local_infoValid = ctx.world.cgs.clientinfo[snapClientNum as usize].infoValid;
    let local_team = ctx.world.cgs.clientinfo[snapClientNum as usize].team;
    if local_infoValid == qfalse {
        return y;
    }

    // Draw the radar background image
    let color: vec4_t = [1.0, 1.0, 1.0, 0.6];
    trap::R_SetColor(ctx.engine, Some(&color));
    let radarShader = ctx.world.cgs.media.radarShader;
    CG_DrawPic(
        ctx,
        (RADAR_X + xOffset) as f32,
        y,
        (RADAR_RADIUS * 2) as f32,
        (RADAR_RADIUS * 2) as f32,
        radarShader,
    );

    //Always green for your own team.
    let green = g_color_table[COLOR_GREEN_INDEX];
    // Raven's `VectorCopy` moves three components, then sets alpha itself.
    let teamColor: vec4_t = [green[0], green[1], green[2], 1.0];

    let mut arrow_w: f32;
    let mut arrow_h: f32;
    let mut arrowBaseScale: f32;
    let mut zScale: f32;

    // Draw all of the radar entities.  Draw them backwards so players are drawn last
    for i in (0..ctx.world.cg.radarEntityCount as c_int).rev() {
        let centNum = ctx.world.cg.radarEntities[i as usize] as usize;
        let es = ctx.world.entity(centNum).currentState;
        let cent_lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let cent_lerpAngles = ctx.world.entity(centNum).lerpAngles;
        let cent_vChatTime = ctx.world.entity(centNum).vChatTime;
        let cg_radarRange = ctx.world.draw.cg_radarRange;

        // Get the distances first
        let mut dirPlayer: vec3_t = [0.0; 3];
        _VectorSubtract(
            ctx.world.cg.predictedPlayerState.origin,
            cent_lerpOrigin,
            &mut dirPlayer,
        );
        dirPlayer[2] = 0.0;
        let mut distance = VectorNormalize(&mut dirPlayer);
        let mut actualDist = distance;

        if distance > cg_radarRange * 0.8 {
            if (es.eFlags & EF_RADAROBJECT) != 0
                //still want to draw the direction
                || (es.eType == entityType_t::ET_NPC as c_int //FIXME: draw last, with players...
                    && es.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && es.speed > 0.0)
            //always draw vehicles
            {
                distance = cg_radarRange * 0.8;
            } else {
                continue;
            }
        }

        distance /= cg_radarRange;
        distance *= RADAR_RADIUS as f32;

        let mut dirLook: vec3_t = [0.0; 3];
        AngleVectors(
            ctx.world.cg.predictedPlayerState.viewangles,
            Some(&mut dirLook),
            None,
            None,
        );

        dirLook[2] = 0.0;
        let anglePlayer = (dirPlayer[0] as f64).atan2(dirPlayer[1] as f64) as f32;
        VectorNormalize(&mut dirLook);
        let angleLook = (dirLook[0] as f64).atan2(dirLook[1] as f64) as f32;
        let angle = angleLook - anglePlayer;

        if es.eType == entityType_t::ET_NPC as c_int {
            //FIXME: draw last, with players...
            // DEFERRED: `m_pVehicleInfo->radarIconHandle` — see the fn doc.
            // Source: oracle/codemp/cgame/cg_draw.c:3392-3454
            continue;
        } else if es.eType == entityType_t::ET_MOVER as c_int {
            if es.speed != 0.0
                //the mover's size, actually
                && actualDist < (es.speed + RADAR_ASTEROID_RANGE)
                && ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
            {
                //a mover that's close to me and I'm in a vehicle
                let mut mayImpact = false;
                let mut surfaceDist = actualDist - es.speed;
                if surfaceDist < 0.0 {
                    surfaceDist = 0.0;
                }
                if surfaceDist < RADAR_MIN_ASTEROID_SURF_WARN_DIST {
                    //always warn!
                    mayImpact = true;
                } else {
                    //not close enough to always warn, yet, so check its direction
                    let timeStep: c_int = 500;
                    let mut predictTime: c_int = timeStep;
                    while predictTime < 5000 {
                        //asteroid dir, speed, size, + my dir & speed...
                        let mut asteroidPos: vec3_t = [0.0; 3];
                        BG_EvaluateTrajectory(
                            &es.pos,
                            ctx.world.cg.time + predictTime,
                            &mut asteroidPos,
                        );
                        //FIXME: I don't think it's calcing "myPos" correctly
                        let mut moveDir: vec3_t = [0.0; 3];
                        AngleVectors(
                            ctx.world.cg.predictedVehicleState.viewangles,
                            Some(&mut moveDir),
                            None,
                            None,
                        );
                        let mut myPos: vec3_t = [0.0; 3];
                        _VectorMA(
                            ctx.world.cg.predictedVehicleState.origin,
                            ctx.world.cg.predictedVehicleState.speed * predictTime as f32 / 1000.0,
                            moveDir,
                            &mut myPos,
                        );
                        let newDist = Distance(myPos, asteroidPos);
                        if (newDist - es.speed) <= RADAR_MIN_ASTEROID_SURF_WARN_DIST {
                            //200.0f )
                            //heading for an impact within the next 5 seconds
                            mayImpact = true;
                            break;
                        }
                        predictTime += timeStep;
                    }
                }
                if mayImpact {
                    //possible collision
                    let mut asteroidColor: vec4_t = [0.5, 0.5, 0.5, 1.0];
                    let mut asteroidScale = es.speed / 2000.0; //average asteroid radius?
                    if actualDist > RADAR_ASTEROID_RANGE {
                        actualDist = RADAR_ASTEROID_RANGE;
                    }
                    distance = (actualDist / RADAR_ASTEROID_RANGE) * RADAR_RADIUS as f32;

                    let x = RADAR_X as f32
                        + RADAR_RADIUS as f32
                        + (angle as f64).sin() as f32 * distance;
                    let ly = y + RADAR_RADIUS as f32 + (angle as f64).cos() as f32 * distance;

                    if asteroidScale > 3.0 {
                        asteroidScale = 3.0;
                    } else if asteroidScale < 0.2 {
                        asteroidScale = 0.2;
                    }
                    arrowBaseScale = 9.0 * asteroidScale;
                    if ctx.world.draw.impactSoundDebounceTime < ctx.world.cg.time {
                        if surfaceDist > RADAR_ASTEROID_RANGE * 0.66 {
                            ctx.world.draw.impactSoundDebounceTime = ctx.world.cg.time + 1000;
                        } else if surfaceDist > RADAR_ASTEROID_RANGE / 3.0 {
                            ctx.world.draw.impactSoundDebounceTime = ctx.world.cg.time + 400;
                        } else {
                            ctx.world.draw.impactSoundDebounceTime = ctx.world.cg.time + 100;
                        }
                        let mut soundOrg: vec3_t = [0.0; 3];
                        _VectorMA(
                            ctx.world.cg.refdef.vieworg,
                            -500.0 * (surfaceDist / RADAR_ASTEROID_RANGE),
                            dirPlayer,
                            &mut soundOrg,
                        );
                        let sfx = trap::S_RegisterSound(
                            ctx.engine,
                            "sound/vehicles/common/impactalarm.wav",
                        );
                        trap::S_StartSound(
                            ctx.engine,
                            Some(&soundOrg),
                            ENTITYNUM_WORLD,
                            CHAN_AUTO,
                            sfx,
                        );
                    }
                    //brighten it the closer it is
                    if surfaceDist > RADAR_ASTEROID_RANGE * 0.66 {
                        asteroidColor[0] = 0.7;
                        asteroidColor[1] = 0.7;
                        asteroidColor[2] = 0.7;
                    } else if surfaceDist > RADAR_ASTEROID_RANGE / 3.0 {
                        asteroidColor[0] = 0.85;
                        asteroidColor[1] = 0.85;
                        asteroidColor[2] = 0.85;
                    } else {
                        asteroidColor[0] = 1.0;
                        asteroidColor[1] = 1.0;
                        asteroidColor[2] = 1.0;
                    }
                    //alpha out the longer it's been since it was considered dangerous
                    if (ctx.world.cg.time - ctx.world.draw.impactSoundDebounceTime) > 100 {
                        asteroidColor[3] = ((ctx.world.cg.time
                            - ctx.world.draw.impactSoundDebounceTime)
                            - 100) as f32
                            / 900.0;
                    }

                    trap::R_SetColor(ctx.engine, Some(&asteroidColor));
                    let shader =
                        trap::R_RegisterShaderNoMip(ctx.engine, "gfx/menus/radar/asteroid");
                    CG_DrawPic(
                        ctx,
                        x - 4.0 + xOffset as f32,
                        ly - 4.0,
                        arrowBaseScale,
                        arrowBaseScale,
                        shader,
                    );
                }
            }
        } else if es.eType == entityType_t::ET_MISSILE as c_int {
            //cent->currentState.weapon == WP_ROCKET_LAUNCHER &&//a rocket
            if es.owner > MAX_CLIENTS_I32
                //belongs to an NPC
                && ctx.world.entity(es.owner as usize).currentState.NPC_class
                    == class_t::CLASS_VEHICLE as c_int
            {
                //a rocket belonging to an NPC, FIXME: only tracking rockets!
                let x =
                    RADAR_X as f32 + RADAR_RADIUS as f32 + (angle as f64).sin() as f32 * distance;
                let ly = y + RADAR_RADIUS as f32 + (angle as f64).cos() as f32 * distance;

                arrowBaseScale = 3.0;
                if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0 {
                    //I'm in a vehicle
                    //if it's targetting me, then play an alarm sound if I'm in a vehicle
                    if es.otherEntityNum == ctx.world.cg.predictedPlayerState.clientNum
                        || es.otherEntityNum == ctx.world.cg.predictedPlayerState.m_iVehicleNum
                    {
                        if ctx.world.draw.radarLockSoundDebounceTime < ctx.world.cg.time {
                            let alarmSound;
                            if actualDist > RADAR_MISSILE_RANGE * 0.66 {
                                ctx.world.draw.radarLockSoundDebounceTime =
                                    ctx.world.cg.time + 1000;
                                arrowBaseScale = 3.0;
                                alarmSound = trap::S_RegisterSound(
                                    ctx.engine,
                                    "sound/vehicles/common/lockalarm1.wav",
                                );
                            } else if actualDist > RADAR_MISSILE_RANGE / 3.0 {
                                ctx.world.draw.radarLockSoundDebounceTime = ctx.world.cg.time + 500;
                                arrowBaseScale = 6.0;
                                alarmSound = trap::S_RegisterSound(
                                    ctx.engine,
                                    "sound/vehicles/common/lockalarm2.wav",
                                );
                            } else {
                                ctx.world.draw.radarLockSoundDebounceTime = ctx.world.cg.time + 250;
                                arrowBaseScale = 9.0;
                                alarmSound = trap::S_RegisterSound(
                                    ctx.engine,
                                    "sound/vehicles/common/lockalarm3.wav",
                                );
                            }
                            if actualDist > RADAR_MISSILE_RANGE {
                                actualDist = RADAR_MISSILE_RANGE;
                            }
                            let mut soundOrg: vec3_t = [0.0; 3];
                            _VectorMA(
                                ctx.world.cg.refdef.vieworg,
                                -500.0 * (actualDist / RADAR_MISSILE_RANGE),
                                dirPlayer,
                                &mut soundOrg,
                            );
                            trap::S_StartSound(
                                ctx.engine,
                                Some(&soundOrg),
                                ENTITYNUM_WORLD,
                                CHAN_AUTO,
                                alarmSound,
                            );
                        }
                    }
                }
                zScale = 1.0;

                //we want to scale the thing up/down based on the relative Z (up/down) positioning
                let myZ = ctx.world.cg.predictedPlayerState.origin[2];
                if cent_lerpOrigin[2] > myZ {
                    //higher, scale up (between 16 and 24)
                    let mut dif = cent_lerpOrigin[2] - myZ;

                    //max out to 1.5x scale at 512 units above local player's height
                    dif /= 1024.0;
                    if dif > 0.5 {
                        dif = 0.5;
                    }
                    zScale += dif;
                } else if cent_lerpOrigin[2] < myZ {
                    //lower, scale down (between 16 and 8)
                    let mut dif = myZ - cent_lerpOrigin[2];

                    //half scale at 512 units below local player's height
                    dif /= 1024.0;
                    if dif > 0.5 {
                        dif = 0.5;
                    }
                    zScale -= dif;
                }

                arrowBaseScale *= zScale;

                let ownerVehNum = ctx
                    .world
                    .entity(es.owner as usize)
                    .currentState
                    .m_iVehicleNum;
                if es.owner >= MAX_CLIENTS_I32
                    //missile owned by an NPC
                    && ctx.world.entity(es.owner as usize).currentState.NPC_class
                        == class_t::CLASS_VEHICLE as c_int
                    //NPC is a vehicle
                    // §F19: a driverless vehicle has m_iVehicleNum 0 and Raven
                    // reads clientinfo[-1] (benign OOB garbage); the port
                    // requires a real driver instead of panicking on -1
                    && ownerVehNum > 0
                    && ownerVehNum <= MAX_CLIENTS_I32
                    //Vehicle has a player driver
                    && ctx.world.cgs.clientinfo[(ownerVehNum - 1) as usize].infoValid != qfalse
                //player driver is valid
                {
                    let cl_team = ctx.world.cgs.clientinfo[(ownerVehNum - 1) as usize].team;
                    if cl_team == local_team {
                        trap::R_SetColor(ctx.engine, Some(&teamColor));
                    } else {
                        trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
                    }
                } else {
                    trap::R_SetColor(ctx.engine, None);
                }
                let rocketIcon = ctx.world.cgs.media.mAutomapRocketIcon;
                CG_DrawPic(
                    ctx,
                    x - 4.0 + xOffset as f32,
                    ly - 4.0,
                    arrowBaseScale,
                    arrowBaseScale,
                    rocketIcon,
                );
            }
        } else if es.eType == entityType_t::ET_PLAYER as c_int {
            // not valid then dont draw it
            if ctx.world.cgs.clientinfo[es.number as usize].infoValid == qfalse {
                continue;
            }

            let mut color: vec4_t = teamColor;

            arrowBaseScale = 16.0;
            zScale = 1.0;

            // Pulse the radar icon after a voice message
            if cent_vChatTime + 2000 > ctx.world.cg.time {
                let f = (cent_vChatTime + 2000 - ctx.world.cg.time) as f32 / 3000.0;
                arrowBaseScale = 16.0 + 4.0 * f;
                color[0] = teamColor[0] + (1.0 - teamColor[0]) * f;
                color[1] = teamColor[1] + (1.0 - teamColor[1]) * f;
                color[2] = teamColor[2] + (1.0 - teamColor[2]) * f;
            }

            trap::R_SetColor(ctx.engine, Some(&color));

            //we want to scale the thing up/down based on the relative Z (up/down) positioning
            let myZ = ctx.world.cg.predictedPlayerState.origin[2];
            if cent_lerpOrigin[2] > myZ {
                //higher, scale up (between 16 and 32)
                let mut dif = cent_lerpOrigin[2] - myZ;

                //max out to 2x scale at 1024 units above local player's height
                dif /= 1024.0;
                if dif > 1.0 {
                    dif = 1.0;
                }
                zScale += dif;
            } else if cent_lerpOrigin[2] < myZ {
                //lower, scale down (between 16 and 8)
                let mut dif = myZ - cent_lerpOrigin[2];

                //half scale at 512 units below local player's height
                dif /= 1024.0;
                if dif > 0.5 {
                    dif = 0.5;
                }
                zScale -= dif;
            }

            arrowBaseScale *= zScale;

            arrow_w = arrowBaseScale * RADAR_RADIUS as f32 / 128.0;
            arrow_h = arrowBaseScale * RADAR_RADIUS as f32 / 128.0;

            // Raven leaves this pair uncast, so the whole sum evaluates in
            // double before it lands in the float argument.
            let px = (RADAR_X as f64
                + RADAR_RADIUS as f64
                + (angle as f64).sin() * distance as f64
                + xOffset as f64) as f32;
            let py =
                (y as f64 + RADAR_RADIUS as f64 + (angle as f64).cos() * distance as f64) as f32;
            let playerIcon = ctx.world.cgs.media.mAutomapPlayerIcon;
            CG_DrawRotatePic2(
                ctx,
                px,
                py,
                arrow_w,
                arrow_h,
                (360.0 - cent_lerpAngles[YAW]) + ctx.world.cg.predictedPlayerState.viewangles[YAW],
                playerIcon,
            );
        } else {
            let x = RADAR_X as f32 + RADAR_RADIUS as f32 + (angle as f64).sin() as f32 * distance;
            let ly = y + RADAR_RADIUS as f32 + (angle as f64).cos() as f32 * distance;

            arrowBaseScale = 9.0;
            let mut shader: qhandle_t = 0;
            zScale = 1.0;

            //we want to scale the thing up/down based on the relative Z (up/down) positioning
            let myZ = ctx.world.cg.predictedPlayerState.origin[2];
            if cent_lerpOrigin[2] > myZ {
                //higher, scale up (between 16 and 24)
                let mut dif = cent_lerpOrigin[2] - myZ;

                //max out to 1.5x scale at 512 units above local player's height
                dif /= 1024.0;
                if dif > 0.5 {
                    dif = 0.5;
                }
                zScale += dif;
            } else if cent_lerpOrigin[2] < myZ {
                //lower, scale down (between 16 and 8)
                let mut dif = myZ - cent_lerpOrigin[2];

                //half scale at 512 units below local player's height
                dif /= 1024.0;
                if dif > 0.5 {
                    dif = 0.5;
                }
                zScale -= dif;
            }

            arrowBaseScale *= zScale;

            // §F19: Raven's local `vec4_t color` is uninitialized stack, and
            // the two `VectorCopy` paths below fill only rgb — the port starts
            // it opaque, the alpha every sibling path here writes.
            let mut color: vec4_t = [0.0, 0.0, 0.0, 1.0];

            if es.brokenLimbs != 0 {
                //slightly misleading to use this value, but don't want to add more to entstate.
                //any ent with brokenLimbs non-0 and on radar is an objective ent.
                //brokenLimbs is literal team value.

                //we only want to draw it if the objective for it is not complete.
                //frame represents objective num.
                let objState = trap::Cvar_VariableStringBuffer(
                    ctx.engine,
                    &format!("team{}_objective{}", es.brokenLimbs, es.frame),
                    1024,
                );

                let complete = atoi(&objState);

                if complete == 0 {
                    // generic enemy index specifies a shader to use for the radar entity.
                    if es.genericenemyindex != 0 {
                        color = [1.0, 1.0, 1.0, 1.0];
                        shader = ctx.world.cgs.gameIcons[es.genericenemyindex as usize];
                    } else {
                        // The `cg.snap &&` half of Raven's test is already
                        // settled — this fn returned above without one.
                        if es.brokenLimbs == snapPersTeam {
                            let c = g_color_table[COLOR_RED_INDEX];
                            color[0] = c[0];
                            color[1] = c[1];
                            color[2] = c[2];
                        } else {
                            let c = g_color_table[COLOR_GREEN_INDEX];
                            color[0] = c[0];
                            color[1] = c[1];
                            color[2] = c[2];
                        }

                        shader = ctx.world.cgs.media.siegeItemShader;
                    }
                }
            } else {
                color = [1.0, 1.0, 1.0, 1.0];

                // generic enemy index specifies a shader to use for the radar entity.
                if es.genericenemyindex != 0 {
                    shader = ctx.world.cgs.gameIcons[es.genericenemyindex as usize];
                } else {
                    shader = ctx.world.cgs.media.siegeItemShader;
                }
            }

            if shader != 0 {
                // Pulse the alpha if time2 is set.  time2 gets set when the entity takes pain
                if (es.time2 != 0 && ctx.world.cg.time - es.time2 < 5000)
                    || (es.time2 as u32 == 0xFFFF_FFFF)
                {
                    if ((ctx.world.cg.time / 200) & 1) != 0 {
                        color[3] = 0.1 + 0.9 * (ctx.world.cg.time % 200) as f32 / 200.0;
                    } else {
                        color[3] = 1.0 - 0.9 * (ctx.world.cg.time % 200) as f32 / 200.0;
                    }
                }

                trap::R_SetColor(ctx.engine, Some(&color));
                CG_DrawPic(
                    ctx,
                    x - 4.0 + xOffset as f32,
                    ly - 4.0,
                    arrowBaseScale,
                    arrowBaseScale,
                    shader,
                );
            }
        }
    }

    arrowBaseScale = 16.0;

    arrow_w = arrowBaseScale * RADAR_RADIUS as f32 / 128.0;
    arrow_h = arrowBaseScale * RADAR_RADIUS as f32 / 128.0;

    trap::R_SetColor(ctx.engine, Some(&colorWhite));
    let playerIcon = ctx.world.cgs.media.mAutomapPlayerIcon;
    CG_DrawRotatePic2(
        ctx,
        (RADAR_X + RADAR_RADIUS + xOffset) as f32,
        y + RADAR_RADIUS as f32,
        arrow_w,
        arrow_h,
        0.0,
        playerIcon,
    );

    y + (RADAR_RADIUS * 2) as f32
}

/// Raven `CG_DrawSiegeMessageNonMenu` — centerprints a siege objective
/// message, resolving a leading `@` through the string package.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4370-4379`
pub fn CG_DrawSiegeMessageNonMenu(ctx: &mut CgContext, str: &str) {
    let resolved;
    let str = if let Some(key) = str.strip_prefix('@') {
        resolved = trap::SP_GetStringTextString(ctx.engine, key, 1024)
            .unwrap_or_else(|| format!("??{}", key));
        resolved.as_str()
    } else {
        str
    };

    CG_CenterPrint(
        ctx.world,
        str,
        // `SCREEN_HEIGHT * 0.20` is int-times-double in C, so it lands in the
        // int argument as 96.
        (SCREEN_HEIGHT as f64 * 0.20) as c_int,
        BIGCHAR_WIDTH,
    );
}

/// Raven `CG_WorldCoordToScreenCoord` — the integer twin of
/// [`CG_WorldCoordToScreenCoordFloat`].
///
/// Raven casts its two locals into the out-params even on the `qfalse` return,
/// where the float version never wrote them - reading uninitialized stack
/// (§F19). The port hands back `None` and writes nothing.
/// Source: `oracle/codemp/cgame/cg_draw.c:5311-5318`
pub fn CG_WorldCoordToScreenCoord(world: &CgWorld, worldCoord: vec3_t) -> Option<(c_int, c_int)> {
    let (xF, yF) = CG_WorldCoordToScreenCoordFloat(world, worldCoord)?;
    Some((xF as c_int, yF as c_int))
}

/// Raven `CG_DottedLine` — `numDots` evenly spaced squares from one point to
/// another, used by the vehicle targeting overlay.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5388-5411`
#[allow(clippy::too_many_arguments)]
pub fn CG_DottedLine(
    ctx: &mut CgContext,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    dotSize: f32,
    numDots: c_int,
    color: vec4_t,
    alpha: f32,
) {
    let mut colorRGBA: vec4_t = color;
    colorRGBA[3] = alpha;

    trap::R_SetColor(ctx.engine, Some(&colorRGBA));

    let xDiff = x2 - x1;
    let yDiff = y2 - y1;
    let xStep = xDiff / numDots as f32;
    let yStep = yDiff / numDots as f32;

    for dotNum in 0..numDots {
        let x = x1 + (xStep * dotNum as f32) - (dotSize * 0.5);
        let y = y1 + (yStep * dotNum as f32) - (dotSize * 0.5);

        let whiteShader = ctx.world.cgs.media.whiteShader;
        CG_DrawPic(ctx, x, y, dotSize, dotSize, whiteShader);
    }
}

/// Raven `CG_DrawHolocronIcons` — the holocron gametype's carried-power column
/// down the left edge.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5645-5681`
pub fn CG_DrawHolocronIcons(ctx: &mut CgContext) {
    let icon_size: c_int = 40;
    let mut i: usize = 0;
    let mut startx: c_int = 10;
    let mut starty: c_int = 10; //SCREEN_HEIGHT - icon_size*3;

    let endx = icon_size;
    let endy = icon_size;

    // §F19: Raven derefs `cg.snap` unguarded - with no snapshot there are no
    // holocron bits, so nothing is drawn.
    let (zoomMode, clientNum, holocronBits) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (snap.ps.zoomMode, snap.ps.clientNum, snap.ps.holocronBits)
    };

    if zoomMode != 0 {
        //don't display over zoom mask
        return;
    }

    if ctx.world.cgs.clientinfo[clientNum as usize].team == TEAM_SPECTATOR {
        return;
    }

    while (i as c_int) < NUM_FORCE_POWERS {
        if (holocronBits & (1 << forcePowerSorted[i])) != 0 {
            let icon = ctx.world.cgs.media.forcePowerIcons[forcePowerSorted[i] as usize];
            CG_DrawPic(
                ctx,
                startx as f32,
                starty as f32,
                endx as f32,
                endy as f32,
                icon,
            );
            starty += icon_size + 2; //+2 for spacing
            if (starty + icon_size) >= SCREEN_HEIGHT - 80 {
                starty = 10; //SCREEN_HEIGHT - icon_size*3;
                startx += icon_size + 2;
            }
        }

        i += 1;
    }
}

/// Raven `CG_DrawActivePowers` — the row of icons for the duration powers that
/// are running right now, plus the rage-recovery icon.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5700-5743`
pub fn CG_DrawActivePowers(ctx: &mut CgContext) {
    let icon_size: c_int = 40;
    let mut i: usize = 0;
    let mut startx: c_int = icon_size * 2 + 16;
    let mut starty: c_int = SCREEN_HEIGHT - icon_size * 2;

    let endx = icon_size;
    let endy = icon_size;

    // §F19: same unguarded `cg.snap` deref as `CG_DrawHolocronIcons`.
    let (zoomMode, clientNum, forcePowersActive, forceRageRecoveryTime) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (
            snap.ps.zoomMode,
            snap.ps.clientNum,
            snap.ps.fd.forcePowersActive,
            snap.ps.fd.forceRageRecoveryTime,
        )
    };

    if zoomMode != 0 {
        //don't display over zoom mask
        return;
    }

    if ctx.world.cgs.clientinfo[clientNum as usize].team == TEAM_SPECTATOR {
        return;
    }

    while (i as c_int) < NUM_FORCE_POWERS {
        if (forcePowersActive & (1 << forcePowerSorted[i])) != 0
            && CG_IsDurationPower(forcePowerSorted[i])
        {
            let icon = ctx.world.cgs.media.forcePowerIcons[forcePowerSorted[i] as usize];
            CG_DrawPic(
                ctx,
                startx as f32,
                starty as f32,
                endx as f32,
                endy as f32,
                icon,
            );
            startx += icon_size + 2; //+2 for spacing
            if (startx + icon_size) >= SCREEN_WIDTH - 80 {
                startx = icon_size * 2 + 16;
                starty += icon_size + 2;
            }
        }

        i += 1;
    }

    //additionally, draw an icon force force rage recovery
    if forceRageRecoveryTime > ctx.world.cg.time {
        let rageRecShader = ctx.world.cgs.media.rageRecShader;
        CG_DrawPic(
            ctx,
            startx as f32,
            starty as f32,
            endx as f32,
            endy as f32,
            rageRecShader,
        );
    }
}

/// Raven `CG_CalcVehicleMuzzlePoint` — where the crosshair trace starts when
/// you are flying/driving. `true` is Raven's `qtrue`, the turret-gunner case.
///
/// DEFERRED: `Vehicle_t` referent pool — both branches Raven can take need
/// `m_pVehicleInfo` (the `VH_WALKER` barrel offset, and the turret table the
/// muzzle averaging walks), which DEC-46.2's presence-only `Option<VehicleId>`
/// cannot supply, so the port always takes Raven's fall-through: the vehicle's
/// own origin and angles, `qfalse`. Restoring the turret branch also needs a
/// `CgContext` here, since `CG_CalcVehMuzzle` takes one.
/// Source: `oracle/codemp/cgame/cg_draw.c:5966-6035`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5963-6040`
pub fn CG_CalcVehicleMuzzlePoint(
    world: &CgWorld,
    entityNum: usize,
    start: &mut vec3_t,
    d_f: &mut vec3_t,
    d_rt: &mut vec3_t,
    d_up: &mut vec3_t,
) -> bool {
    let vehCent = world.entity(entityNum);

    _VectorCopy(vehCent.lerpOrigin, start);
    AngleVectors(vehCent.lerpAngles, Some(d_f), Some(d_rt), Some(d_up));

    false
}

/// Raven `CG_DrawFlagStatus` — the two little CTF flag icons up the left edge.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7084-7143`
pub fn CG_DrawFlagStatus(ctx: &mut CgContext) {
    let mut startDrawPos: c_int = 2;
    let ico_size: c_int = 32;

    // §F19: Raven's own `if (!cg.snap) return;`.
    let team = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        snap.ps.persistant[PERS_TEAM as usize]
    };

    if ctx.world.cgs.gametype != GT_CTF && ctx.world.cgs.gametype != GT_CTY {
        return;
    }

    let (myFlagTakenShader, theirFlagShader) = if ctx.world.cgs.gametype == GT_CTY {
        if team == TEAM_RED {
            (
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_rflag_x"),
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_bflag_ys"),
            )
        } else {
            (
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_bflag_x"),
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_rflag_ys"),
            )
        }
    } else if team == TEAM_RED {
        (
            trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_rflag_x"),
            trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_bflag"),
        )
    } else {
        (
            trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_bflag_x"),
            trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_rflag"),
        )
    };

    if CG_YourTeamHasFlag(ctx.world) {
        //CG_DrawPic( startDrawPos, 330, ico_size, ico_size, theirFlagShader );
        CG_DrawPic(
            ctx,
            2.0,
            (330 - startDrawPos) as f32,
            ico_size as f32,
            ico_size as f32,
            theirFlagShader,
        );
        startDrawPos += ico_size + 2;
    }

    if CG_OtherTeamHasFlag(ctx.world) {
        //CG_DrawPic( startDrawPos, 330, ico_size, ico_size, myFlagTakenShader );
        CG_DrawPic(
            ctx,
            2.0,
            (330 - startDrawPos) as f32,
            ico_size as f32,
            ico_size as f32,
            myFlagTakenShader,
        );
    }
}

/// Raven `CG_DrawSiegeHUDItem` — the little rotating 3D model of the siege
/// item you are carrying, in the top-left corner.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7483-7524`
pub fn CG_DrawSiegeHUDItem(ctx: &mut CgContext) {
    let centNum = ctx.world.draw.cgSiegeEntityRender as usize;
    let cent_ghoul2 = ctx.world.entity(centNum).ghoul2;

    let (g2, handle) = if !cent_ghoul2.is_null() {
        (cent_ghoul2, 0)
    } else {
        let modelindex = ctx.world.entity(centNum).currentState.modelindex;
        (null_mut(), ctx.world.cgs.gameModels[modelindex as usize])
    };

    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    if handle != 0 {
        trap::R_ModelBounds(ctx.engine, handle, &mut mins, &mut maxs);
    } else {
        VectorSet(&mut mins, -16.0, -16.0, -20.0);
        VectorSet(&mut maxs, 16.0, 16.0, 32.0);
    }

    let mut origin: vec3_t = [0.0; 3];
    origin[2] = (-0.5 * (mins[2] + maxs[2]) as f64) as f32;
    origin[1] = (0.5 * (mins[1] + maxs[1]) as f64) as f32;
    let len = (0.5 * (maxs[2] - mins[2]) as f64) as f32;
    origin[0] = (len as f64 / 0.268) as f32;

    let mut angles: vec3_t = [0.0; 3];
    VectorClear(&mut angles);
    angles[YAW] = ctx.world.cg.autoAngles[YAW];

    let g2radius = ctx.world.entity(centNum).currentState.g2radius;
    CG_Draw3DModel(
        ctx, 8.0, 8.0, 64.0, 64.0, handle, g2, g2radius, 0, origin, angles,
    );

    ctx.world.draw.cgSiegeEntityRender = 0; //reset for next frame
}

/// Raven `CG_ChatBox_ArrayInsert` — slides the chat draw-order array up one
/// slot and drops `item` in, recursing until it hits an empty slot.
///
/// Raven's `chatBoxItem_t **array` is a sort order over `cg.chatItems`, so the
/// slots are indices into that array here (§B5), not pointers.
/// Source: `oracle/codemp/cgame/cg_draw.c:7627-7640`
pub fn CG_ChatBox_ArrayInsert(
    ctx: &mut CgContext,
    array: &mut [Option<usize>],
    insPoint: usize,
    maxNum: usize,
    item: usize,
) {
    if let Some(occupant) = array[insPoint] {
        //recursively call, to move everything up to the top
        if insPoint + 1 >= maxNum {
            CG_Error(ctx, "CG_ChatBox_ArrayInsert: Exceeded array size");
            // Raven's CG_Error longjmps out; ours returns, so stop here
            return;
        }
        CG_ChatBox_ArrayInsert(ctx, array, insPoint + 1, maxNum, occupant);
    }

    //now that we have moved anything that would be in this slot up, insert what we want into the slot
    array[insPoint] = Some(item);
}

/// Raven `CG_DrawZoomMask` — the binocular and disruptor-scope overlays, each
/// with its own artwork, scrolling compass and charge readout.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:220-458`
pub fn CG_DrawZoomMask(ctx: &mut CgContext) {
    let mut color1: vec4_t = [0.0; 4];

    // int ammo = cg_entities[0].gent->client->ps.ammo[weaponData[cent->currentState.weapon].ammoIndex];

    // Check for Binocular specific zooming since we'll want to render different bits in each case
    if ctx.world.cg.predictedPlayerState.zoomMode == 2 {
        // zoom level
        let mut level = (80.0 - ctx.world.cg.predictedPlayerState.zoomFov) / 80.0;

        // ...so we'll clamp it
        if level < 0.0 {
            level = 0.0;
        } else if level > 1.0 {
            level = 1.0;
        }

        // Using a magic number to convert the zoom level to scale amount
        level *= 162.0;

        // draw blue tinted distortion mask, trying to make it as small as is necessary to fill in the viewable area
        trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
        let binocularStatic = ctx.world.cgs.media.binocularStatic;
        CG_DrawPic(ctx, 34.0, 48.0, 570.0, 362.0, binocularStatic);

        // Black out the area behind the numbers
        trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_BLACK as usize]));
        let whiteShader = ctx.world.cgs.media.whiteShader;
        CG_DrawPic(ctx, 212.0, 367.0, 200.0, 40.0, whiteShader);

        // Numbers should be kind of greenish
        color1[0] = 0.2;
        color1[1] = 0.4;
        color1[2] = 0.2;
        color1[3] = 0.3;
        trap::R_SetColor(ctx.engine, Some(&color1));

        // Draw scrolling numbers, use intervals 10 units apart--sorry, this section of code is just kind of hacked
        //	up with a bunch of magic numbers.....
        let yaw = ctx.world.cg.refdef.viewangles[YAW];
        let mut val: c_int = (((yaw + 180.0) / 10.0) as c_int) * 10;
        let off: f32 = (yaw + 180.0) - val as f32;

        let mut i: c_int = -10;
        while i < 30 {
            val -= 10;

            if val < 0 {
                val += 360;
            }

            // we only want to draw the very far left one some of the time, if it's too far to the left it will
            //	poke outside the mask.
            if (off > 3.0 && i == -10) || i > -10 {
                // draw the value, but add 200 just to bump the range up...arbitrary, so change it if you like
                CG_DrawNumField(
                    ctx,
                    ((155 + i * 10) as f32 + off * 10.0) as c_int,
                    374,
                    3,
                    val + 200,
                    24,
                    14,
                    NUM_FONT_CHUNKY,
                    true,
                );
                let whiteShader = ctx.world.cgs.media.whiteShader;
                CG_DrawPic(
                    ctx,
                    (245 + (i - 1) * 10) as f32 + off * 10.0,
                    376.0,
                    6.0,
                    6.0,
                    whiteShader,
                );
            }

            i += 10;
        }

        let binocularOverlay = ctx.world.cgs.media.binocularOverlay;
        CG_DrawPic(ctx, 212.0, 367.0, 200.0, 28.0, binocularOverlay);

        color1[0] = (((ctx.world.cg.time as f32 * 0.01) as f64).sin() * 0.5 + 0.5) as f32;
        color1[0] = color1[0] * color1[0];
        color1[1] = color1[0];
        color1[2] = color1[0];
        color1[3] = 1.0;

        trap::R_SetColor(ctx.engine, Some(&color1));

        let binocularCircle = ctx.world.cgs.media.binocularCircle;
        CG_DrawPic(ctx, 82.0, 94.0, 16.0, 16.0, binocularCircle);

        // Flickery color
        color1[0] = (0.7 + ctx.world.bg_state.rng.crandom() * 0.1) as f32;
        color1[1] = (0.8 + ctx.world.bg_state.rng.crandom() * 0.1) as f32;
        color1[2] = (0.7 + ctx.world.bg_state.rng.crandom() * 0.1) as f32;
        color1[3] = 1.0;
        trap::R_SetColor(ctx.engine, Some(&color1));

        let binocularMask = ctx.world.cgs.media.binocularMask;
        CG_DrawPic(ctx, 0.0, 0.0, 640.0, 480.0, binocularMask);

        let binocularArrow = ctx.world.cgs.media.binocularArrow;
        CG_DrawPic(ctx, 4.0, 282.0 - level, 16.0, 16.0, binocularArrow);

        // The top triangle bit randomly flips
        let binocularTri = ctx.world.cgs.media.binocularTri;
        if ctx.world.draw.flip {
            CG_DrawPic(ctx, 330.0, 60.0, -26.0, -30.0, binocularTri);
        } else {
            CG_DrawPic(ctx, 307.0, 40.0, 26.0, 30.0, binocularTri);
        }

        if ctx.world.bg_state.rng.random() > 0.98 && (ctx.world.cg.time & 1024) != 0 {
            ctx.world.draw.flip = !ctx.world.draw.flip;
        }
    } else if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
        // disruptor zoom mode
        let mut level = (50.0 - ctx.world.view.zoomFov) / 50.0; //(float)(80.0f - zoomFov) / 80.0f;

        // ...so we'll clamp it
        if level < 0.0 {
            level = 0.0;
        } else if level > 1.0 {
            level = 1.0;
        }

        // Using a magic number to convert the zoom level to a rotation amount that correlates more or less with the zoom artwork.
        level *= 103.0;

        // Draw target mask
        trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
        let disruptorMask = ctx.world.cgs.media.disruptorMask;
        CG_DrawPic(ctx, 0.0, 0.0, 640.0, 480.0, disruptorMask);

        // apparently 99.0f is the full zoom level
        if level >= 99.0 {
            // Fully zoomed, so make the rotating insert pulse
            color1[0] = 1.0;
            color1[1] = 1.0;
            color1[2] = 1.0;
            color1[3] = (0.7 + ((ctx.world.cg.time as f32 * 0.01) as f64).sin() * 0.3) as f32;

            trap::R_SetColor(ctx.engine, Some(&color1));
        }

        // Draw rotating insert
        let disruptorInsert = ctx.world.cgs.media.disruptorInsert;
        CG_DrawRotatePic2(ctx, 320.0, 240.0, 640.0, 480.0, -level, disruptorInsert);

        // Increase the light levels under the center of the target — Raven's
        // `CG_DrawPic( 198, 118, 246, 246, cgs.media.disruptorLight )` and the
        // base-five ammo readout below it are both commented out at the site.

        // §F19: Raven derefs `cg.snap` unguarded for the ammo arc; with no
        // snapshot there is no ammo to read, so only the arc is skipped — the
        // charge bar below runs either way.
        let snapAmmo = ctx
            .world
            .cg
            .snap_ref()
            .map(|snap| (snap.ps.eFlags, snap.ps.ammo));

        if let Some((eFlags, ammo)) = snapAmmo {
            let ammoIndex = weaponData[WP_DISRUPTOR as usize].ammoIndex as usize;
            let mut max = if (eFlags & EF_DOUBLE_AMMO) != 0 {
                ammo[ammoIndex] as f32 / (ammoData[ammoIndex].max as f32 * 2.0)
            } else {
                ammo[ammoIndex] as f32 / ammoData[ammoIndex].max as f32
            };
            if max > 1.0 {
                max = 1.0;
            }

            color1[0] = (1.0 - max) * 2.0;
            color1[1] = max * 1.5;
            color1[2] = 0.0;
            color1[3] = 1.0;

            // If we are low on health, make us flash
            if max < 0.15 && (ctx.world.cg.time & 512) != 0 {
                // Raven's `VectorClear` on a `vec4_t` only touches rgb, so the
                // alpha stays at the 1.0 set above.
                color1[0] = 0.0;
                color1[1] = 0.0;
                color1[2] = 0.0;
            }

            if color1[0] > 1.0 {
                color1[0] = 1.0;
            }

            if color1[1] > 1.0 {
                color1[1] = 1.0;
            }

            trap::R_SetColor(ctx.engine, Some(&color1));

            max *= 58.0;

            // going from 15 to 45 degrees, with 5 degree increments
            let mut fi: f32 = 18.5;
            while fi <= 18.5 + max {
                let cx = (320.0 + (((fi + 90.0) / 57.296) as f64).sin() * 190.0) as f32;
                let cy = (240.0 + (((fi + 90.0) / 57.296) as f64).cos() * 190.0) as f32;

                let disruptorInsertTick = ctx.world.cgs.media.disruptorInsertTick;
                CG_DrawRotatePic2(ctx, cx, cy, 12.0, 24.0, 90.0 - fi, disruptorInsertTick);

                fi += 3.0;
            }
        }

        if ctx.world.cg.predictedPlayerState.weaponstate == WEAPON_CHARGING_ALT as c_int {
            trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));

            // draw the charge level
            // bad hardcodedness 50 is disruptor charge unit and 30 is max charge units allowed.
            let mut max = (ctx.world.cg.time - ctx.world.cg.predictedPlayerState.weaponChargeTime)
                as f32
                / (50.0 * 30.0);

            if max > 1.0 {
                max = 1.0;
            }

            let disruptorChargeShader = ctx.world.cgs.media.disruptorChargeShader;
            trap::R_DrawStretchPic(
                ctx.engine,
                257.0,
                435.0,
                134.0 * max,
                34.0,
                0.0,
                0.0,
                max,
                1.0,
                disruptorChargeShader,
            );
        }
        // Raven's closing `trap_R_SetColor( colorTable[CT_WHITE] )` +
        // `CG_DrawPic( 0, 0, 640, 480, cgs.media.disruptorMask )` are commented out.
    }
}

/// Raven `CG_DrawHealth` — the four health tics plus the numeric readout, off
/// whichever HUD menu the caller found.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:614-695`
pub fn CG_DrawHealth(ctx: &mut CgContext, menus: &MenuSystem, menuHUD: Option<MenuId>) {
    // Can we find the menu?
    if menuHUD.is_none() {
        return;
    }

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there are no
    // stats to paint.
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    let stats = snap.ps.stats;

    // What's the health?
    let mut healthAmt = stats[STAT_HEALTH as usize];
    if healthAmt > stats[STAT_MAX_HEALTH as usize] {
        healthAmt = stats[STAT_MAX_HEALTH as usize];
    }

    let inc = (stats[STAT_MAX_HEALTH as usize] as f32 / MAX_HUD_TICS as f32) as c_int;
    let mut currValue = healthAmt;

    // Print the health tics, fading out the one which is partial health
    for i in (0..MAX_HUD_TICS).rev() {
        // This is bad
        let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, healthTicName[i]) else {
            continue;
        };

        // §F19: Raven derefs the `float *hudTintColor` global, which is NULL
        // until `CG_DrawHUD` points it somewhere; `trap_R_SetColor(NULL)` is
        // the renderer's white, so `None` reads as `colorTable[CT_WHITE]`.
        let mut calcColor = ctx
            .world
            .draw
            .hudTintColor
            .unwrap_or(colorTable[ct_table_t::CT_WHITE as usize]);

        if currValue <= 0 {
            // don't show tic
            break;
        } else if currValue < inc {
            // partial tic (alpha it out)
            let percent = currValue as f32 / inc as f32;
            calcColor[3] *= percent; // Fade it out
        }

        trap::R_SetColor(ctx.engine, Some(&calcColor));

        let rect = menus.item(focusItem).window.rect;
        let background = menus.item(focusItem).window.background;
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);

        currValue -= inc;
    }

    // Print the mueric amount
    if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "healthamount") {
        // Print health amount
        let foreColor = menus.item(focusItem).window.foreColor;
        trap::R_SetColor(ctx.engine, Some(&foreColor));

        let rect = menus.item(focusItem).window.rect;
        CG_DrawNumField(
            ctx,
            rect.x as c_int,
            rect.y as c_int,
            3,
            stats[STAT_HEALTH as usize],
            rect.w as c_int,
            rect.h as c_int,
            NUM_FONT_SMALL,
            false,
        );
    }
}

/// Raven `CG_DrawArmor` — the armor tics, the numeric readout, and the
/// low-armor flash latch.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:702-831`
pub fn CG_DrawArmor(ctx: &mut CgContext, menus: &MenuSystem, menuHUD: Option<MenuId>) {
    //ps = &cg.snap->ps;
    let stats = ctx.world.cg.predictedPlayerState.stats;

    // Can we find the menu?
    if menuHUD.is_none() {
        return;
    }

    let mut armor = stats[STAT_ARMOR as usize];
    let maxArmor = stats[STAT_MAX_HEALTH as usize];

    if armor > maxArmor {
        armor = maxArmor;
    }

    let mut currValue = armor;
    let inc = (maxArmor as f32 / MAX_HUD_TICS as f32) as c_int;

    // Raven's first `memcpy(calcColor, hudTintColor, …)` here is redone inside
    // the loop before anything reads it.
    for i in (0..MAX_HUD_TICS).rev() {
        // This is bad
        let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, armorTicName[i]) else {
            continue;
        };

        // §F19: see `CG_DrawHealth` — a NULL `hudTintColor` reads as white.
        let mut calcColor = ctx
            .world
            .draw
            .hudTintColor
            .unwrap_or(colorTable[ct_table_t::CT_WHITE as usize]);

        if currValue <= 0 {
            // don't show tic
            break;
        } else if currValue < inc {
            // partial tic (alpha it out)
            let percent = currValue as f32 / inc as f32;
            calcColor[3] *= percent;
        }

        trap::R_SetColor(ctx.engine, Some(&calcColor));

        let rect = menus.item(focusItem).window.rect;
        let background = menus.item(focusItem).window.background;

        // Raven's two arms draw the same pic; the top tic just sits out the
        // half of the flash cycle where `HUDArmorFlag` is off.
        if i == (MAX_HUD_TICS - 1) && currValue < inc {
            if ctx.world.cg.HUDArmorFlag != qfalse {
                CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
            }
        } else {
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
        }

        currValue -= inc;
    }

    if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "armoramount") {
        // Print armor amount
        let foreColor = menus.item(focusItem).window.foreColor;
        trap::R_SetColor(ctx.engine, Some(&foreColor));

        let rect = menus.item(focusItem).window.rect;
        CG_DrawNumField(
            ctx,
            rect.x as c_int,
            rect.y as c_int,
            3,
            armor,
            rect.w as c_int,
            rect.h as c_int,
            NUM_FONT_SMALL,
            false,
        );
    }

    // If armor is low, flash a graphic to warn the player
    if armor != 0 {
        // Is there armor? Draw the HUD Armor TIC
        let quarterArmor = stats[STAT_MAX_HEALTH as usize] as f32 / 4.0;

        // Make tic flash if armor is at 25% of full armor
        if (stats[STAT_ARMOR as usize] as f32) < quarterArmor {
            // Do whatever the flash timer says
            if ctx.world.cg.HUDTickFlashTime < ctx.world.cg.time as f32 {
                // Flip at the same time
                ctx.world.cg.HUDTickFlashTime = (ctx.world.cg.time + 400) as f32;
                if ctx.world.cg.HUDArmorFlag != qfalse {
                    ctx.world.cg.HUDArmorFlag = qfalse;
                } else {
                    ctx.world.cg.HUDArmorFlag = qtrue;
                }
            }
        } else {
            ctx.world.cg.HUDArmorFlag = qtrue;
        }
    } else {
        // No armor? Don't show it.
        ctx.world.cg.HUDArmorFlag = qfalse;
    }
}

/// Raven `CG_DrawForcePower` — the force tics and their numeric readout, with
/// the out-of-force flash the "no force" sound rides on.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1042-1162`
pub fn CG_DrawForcePower(ctx: &mut CgContext, menus: &MenuSystem, menuHUD: Option<MenuId>) {
    let maxForcePower: c_int = 100;
    let mut flash = false;

    // Can we find the menu?
    if menuHUD.is_none() {
        return;
    }

    // Make the hud flash by setting forceHUDTotalFlashTime above cg.time
    if ctx.world.cg.forceHUDTotalFlashTime > ctx.world.cg.time {
        flash = true;
        if ctx.world.cg.forceHUDNextFlashTime < ctx.world.cg.time {
            ctx.world.cg.forceHUDNextFlashTime = ctx.world.cg.time + 400;
            let noforceSound = ctx.world.cgs.media.noforceSound;
            trap::S_StartSound(ctx.engine, None, 0, CHAN_LOCAL, noforceSound);

            if ctx.world.cg.forceHUDActive != qfalse {
                ctx.world.cg.forceHUDActive = qfalse;
            } else {
                ctx.world.cg.forceHUDActive = qtrue;
            }
        }
    } else {
        // turn HUD back on if it had just finished flashing time.
        ctx.world.cg.forceHUDNextFlashTime = 0;
        ctx.world.cg.forceHUDActive = qtrue;
    }

    // Raven's `if (!cg.forceHUDActive) return;` is commented out at the site.

    let inc = maxForcePower as f32 / MAX_HUD_TICS as f32;

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // force amount to paint.
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    let forcePower = snap.ps.fd.forcePower;
    let mut value = forcePower as f32;

    for i in (0..MAX_HUD_TICS).rev() {
        let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, forceTicName[i]) else {
            continue;
        };

        // Raven's `memcpy(calcColor, hudTintColor, …)` is commented out here.
        let mut calcColor: vec4_t;

        if value <= 0.0 {
            // done
            break;
        } else if value < inc {
            // partial tic
            if flash {
                calcColor = colorTable[ct_table_t::CT_RED as usize];
            } else {
                calcColor = colorTable[ct_table_t::CT_WHITE as usize];
            }

            let percent = value / inc;
            calcColor[3] = percent;
        } else if flash {
            calcColor = colorTable[ct_table_t::CT_RED as usize];
        } else {
            calcColor = colorTable[ct_table_t::CT_WHITE as usize];
        }

        trap::R_SetColor(ctx.engine, Some(&calcColor));

        let rect = menus.item(focusItem).window.rect;
        let background = menus.item(focusItem).window.background;
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);

        value -= inc;
    }

    if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "forceamount") {
        // Print force amount
        let foreColor = menus.item(focusItem).window.foreColor;
        trap::R_SetColor(ctx.engine, Some(&foreColor));

        let rect = menus.item(focusItem).window.rect;
        CG_DrawNumField(
            ctx,
            rect.x as c_int,
            rect.y as c_int,
            3,
            forcePower,
            rect.w as c_int,
            rect.h as c_int,
            NUM_FONT_SMALL,
            false,
        );
    }
}

/// Raven `CG_DrawVehicleDamageHUD` — the ship silhouette panel: background,
/// frame, shield wash, then the four damage quarters.
///
/// DEFERRED: `Vehicle_t` referent pool — all three pics are
/// `veh->m_pVehicle->m_pVehicleInfo->dmgIndic*Handle`, which DEC-46.2's
/// presence-only `Option<VehicleId>` cannot supply; see
/// [`CG_DrawVehicleShields`]. Each block keeps Raven's item lookup (the work
/// before the read) and stops at the handle. The four `CG_DrawVehicleDamage`
/// tail calls are blocked on the same pool inside that fn.
/// Source: `oracle/codemp/cgame/cg_draw.c:2511,2548,2563`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2495-2583`
pub fn CG_DrawVehicleDamageHUD(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    vehNum: usize,
    brokenLimbs: c_int,
    percShields: f32,
    menuName: &str,
    alpha: f32,
) {
    let menuHUD = Menus_FindByName(menus, menuName);

    if menuHUD.is_none() {
        return;
    }

    if Menu_FindItemByName(menus, menuHUD, "background").is_some() {
        //TODO: Port CG_DrawVehicleDamageHUD vehicle arm
        // DEFERRED: Vehicle_t referent pool — `dmgIndicBackgroundHandle`
        // Source: oracle/codemp/cgame/cg_draw.c:2511-2541
        todo!("CG_DrawVehicleDamageHUD background pic — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2511-2541")
    }

    if Menu_FindItemByName(menus, menuHUD, "outer_frame").is_some() {
        //TODO: Port CG_DrawVehicleDamageHUD vehicle arm
        // DEFERRED: Vehicle_t referent pool — `dmgIndicFrameHandle`
        // Source: oracle/codemp/cgame/cg_draw.c:2548-2556
        todo!("CG_DrawVehicleDamageHUD frame pic — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2548-2556")
    }

    if Menu_FindItemByName(menus, menuHUD, "shields").is_some() {
        let _ = percShields;
        //TODO: Port CG_DrawVehicleDamageHUD vehicle arm
        // DEFERRED: Vehicle_t referent pool — `dmgIndicShieldHandle`
        // Source: oracle/codemp/cgame/cg_draw.c:2563-2573
        todo!("CG_DrawVehicleDamageHUD shield pic — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2563-2573")
    }

    //TODO (Raven): if we check nextState.brokenLimbs & prevState.brokenLimbs, we can tell when a damage flag has been added and flash that part of the ship
    //FIXME (Raven): when ship explodes, either stop drawing ship or draw all parts black
    CG_DrawVehicleDamage(
        ctx,
        menus,
        vehNum,
        brokenLimbs,
        menuHUD,
        alpha,
        VEH_DAMAGE_FRONT,
    );
    CG_DrawVehicleDamage(
        ctx,
        menus,
        vehNum,
        brokenLimbs,
        menuHUD,
        alpha,
        VEH_DAMAGE_BACK,
    );
    CG_DrawVehicleDamage(
        ctx,
        menus,
        vehNum,
        brokenLimbs,
        menuHUD,
        alpha,
        VEH_DAMAGE_LEFT,
    );
    CG_DrawVehicleDamage(
        ctx,
        menus,
        vehNum,
        brokenLimbs,
        menuHUD,
        alpha,
        VEH_DAMAGE_RIGHT,
    );
}

/// Raven `CG_DrawPickupItem` — the icon of the item you just picked up, fading
/// out over three seconds.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2752-2768`
pub fn CG_DrawPickupItem(ctx: &mut CgContext) {
    let value = ctx.world.cg.itemPickup;
    if value != 0 && ctx.world.cg_items[value as usize].icon != -1 {
        let itemPickupTime = ctx.world.cg.itemPickupTime;
        if let Some(fadeColor) = CG_FadeColor(ctx.world, itemPickupTime, 3000) {
            CG_RegisterItemVisuals(ctx, value);
            trap::R_SetColor(ctx.engine, Some(&fadeColor));
            let icon = ctx.world.cg_items[value as usize].icon;
            CG_DrawPic(ctx, 573.0, 320.0, ICON_SIZE, ICON_SIZE, icon);
            trap::R_SetColor(ctx.engine, None);
        }
    }
}

/// Raven `CG_DrawMiniScoreboard` — the one-line red/blue score above the
/// upper-right HUD stack. Returns the y the next element starts at.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2813-2859`
pub fn CG_DrawMiniScoreboard(ctx: &mut CgContext, ds: &DisplayState, mut y: f32) -> f32 {
    // Raven's `#ifdef _XBOX` shifts this by -40; the PC build keeps 0.
    let xOffset: c_int = 0;

    if ctx.world.cvars.cg_drawScores.integer == 0 {
        return y;
    }

    if ctx.world.cgs.gametype == GT_SIEGE {
        //don't bother with this in siege
        return y;
    }

    if ctx.world.cgs.gametype >= GT_TEAM {
        // §F19: Raven's `strcpy(temp, va("%s: ", …))` is unbounded into a
        // 64-byte buffer; the port's `Q_strncpyz` truncates there instead.
        let mut temp = [0 as c_char; MAX_QPATH];
        let red = CG_GetStringEdString(ctx, "MP_INGAME", "RED");
        Q_strncpyz(&mut temp, &format!("{red}: "), MAX_QPATH);

        let scores1 = ctx.world.cgs.scores1;
        let scores2 = ctx.world.cgs.scores2;

        if scores1 == SCORE_NOT_PRESENT {
            Q_strcat(&mut temp, MAX_QPATH, "-");
        } else {
            Q_strcat(&mut temp, MAX_QPATH, &format!("{scores1}"));
        }

        let blue = CG_GetStringEdString(ctx, "MP_INGAME", "BLUE");
        Q_strcat(&mut temp, MAX_QPATH, &format!(" {blue}: "));

        if scores2 == SCORE_NOT_PRESENT {
            Q_strcat(&mut temp, MAX_QPATH, "-");
        } else {
            Q_strcat(&mut temp, MAX_QPATH, &format!("{scores2}"));
        }

        let temp = buf_to_string(&temp.map(|c| c as u8));

        let w = CG_Text_Width(ctx, ds, &temp, 0.7, FONT_MEDIUM);
        CG_Text_Paint(
            ctx,
            ds,
            (630 - w + xOffset) as f32,
            y,
            0.7,
            colorWhite,
            &temp,
            0.0,
            0,
            ITEM_TEXTSTYLE_SHADOWEDMORE,
            FONT_MEDIUM,
        );
        y += 15.0;
    }
    //rww - no longer doing this. Since the attacker now shows who is first, we print the score there.
    // (the non-team "1st:/2nd:" block is commented out at the Raven site.)

    y
}

/// Raven `CG_DrawHealthBarRough` — the chunky three-rect health bar: creme-y
/// filling, used-up-ness, hard crispy shell.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3117-3130`
#[allow(clippy::too_many_arguments)]
pub fn CG_DrawHealthBarRough(
    ctx: &mut CgContext,
    x: f32,
    y: f32,
    width: c_int,
    height: c_int,
    ratio: f32,
    color1: &vec4_t,
    color2: &vec4_t,
) {
    let mut color3: vec4_t = [1.0, 0.0, 0.0, 0.7];

    let midpoint = width as f32 * ratio - 1.0;
    let remainder = width as f32 - midpoint;
    color3[0] = color1[0] * 0.5;

    debug_assert!(height % 4 == 0, "this won't line up otherwise.");
    // creme-y filling.
    CG_DrawRect(
        ctx,
        x + 1.0,
        y + (height / 2 - 1) as f32,
        midpoint,
        1.0,
        (height / 4 + 1) as f32,
        color1,
    );
    // used-up-ness.
    CG_DrawRect(
        ctx,
        x + midpoint,
        y + (height / 2 - 1) as f32,
        remainder,
        1.0,
        (height / 4 + 1) as f32,
        &color3,
    );
    // hard crispy shell
    CG_DrawRect(ctx, x, y, width as f32, height as f32, 1.0, color2);
}

/// Raven `CG_DrawCenterString` — the centerprint text, one line per `\n`,
/// vertically centered on `cg.centerPrintY`.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4423-4473`
pub fn CG_DrawCenterString(ctx: &mut CgContext, ds: &DisplayState) {
    let scale: f32 = 1.0; //0.5

    if ctx.world.cg.centerPrintTime == 0 {
        return;
    }

    let centerPrintTime = ctx.world.cg.centerPrintTime;
    let totalMsec = (1000.0 * ctx.world.cvars.cg_centertime.value) as c_int;
    let Some(color) = CG_FadeColor(ctx.world, centerPrintTime, totalMsec) else {
        return;
    };

    trap::R_SetColor(ctx.engine, Some(&color));

    let start = buf_to_string(&ctx.world.cg.centerPrint.map(|c| c as u8));

    let mut y = ctx.world.cg.centerPrintY - ctx.world.cg.centerPrintLines * BIGCHAR_HEIGHT / 2;

    for line in start.split('\n') {
        // Raven's `linebuffer` takes the first 50 characters of the line and
        // then skips ahead to the newline, so a longer line loses its tail.
        let linebuffer: String = line.chars().take(50).collect();

        let w = CG_Text_Width(ctx, ds, &linebuffer, scale, FONT_MEDIUM);
        let h = CG_Text_Height(ctx, ds, &linebuffer, scale, FONT_MEDIUM);
        let x = (SCREEN_WIDTH - w) / 2;
        CG_Text_Paint(
            ctx,
            ds,
            x as f32,
            (y + h) as f32,
            scale,
            color,
            &linebuffer,
            0.0,
            0,
            ITEM_TEXTSTYLE_SHADOWEDMORE,
            FONT_MEDIUM,
        );
        y += h + 6;
    }

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_DrawSiegeInfo` — the health and ammo bars a siege class with the
/// stat-viewer flag sees under a teammate's crosshair.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4489-4629`
pub fn CG_DrawSiegeInfo(
    ctx: &mut CgContext,
    centNum: usize,
    chX: f32,
    chY: f32,
    chW: f32,
    chH: f32,
) {
    let es = ctx.world.entity(centNum).currentState;
    let number = es.number as usize;

    debug_assert!(number < MAX_CLIENTS);

    let se = ctx.world.saga.cg_siegeExtendedData[number];

    if se.lastUpdated > ctx.world.cg.time {
        //strange, shouldn't happen
        return;
    }

    if (ctx.world.cg.time - se.lastUpdated) > 10000 {
        //if you haven't received a status update on this guy in 10 seconds, forget about it
        return;
    }

    if es.eFlags & EF_DEAD != 0 {
        //he's dead, don't display info on him
        return;
    }

    if es.weapon != se.weapon {
        //data is invalidated until it syncs back again
        return;
    }

    if ctx.world.cgs.clientinfo[number].team
        != ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize]
    {
        //not on the same team
        return;
    }

    let index = ctx.world.cg.predictedPlayerState.clientNum + CS_PLAYERS;
    let configstring = CG_ConfigString(ctx, index);
    let v = Info_ValueForKey(&configstring, "siegeclass");

    if v.is_empty() {
        //don't have siege class in info?
        return;
    }

    let siegeClass = BG_SiegeFindClassByName(&v, &ctx.world.bg_state);

    if siegeClass.is_null() {
        //invalid
        return;
    }

    // `BG_SiegeFindClassByName` hands back a raw pointer into `bgSiegeClasses`
    // and module code never derefs one (§D11), so the same first-match lookup
    // names the owning slot for the flags read.
    let classflags = {
        let n = ctx.world.bg_state.bgNumSiegeClasses as usize;
        match ctx.world.bg_state.bgSiegeClasses[..n]
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(&v))
        {
            Some(c) => c.classflags,
            None => return,
        }
    };

    if classflags & (1 << CFL_STATVIEWER as c_int) == 0 {
        //doesn't really have the ability to see others' stats
        return;
    }

    let mut x = chX + ((chW / 2.0) - (HEALTH_WIDTH / 2.0));
    let mut y = (chY + chH) + 8.0;
    let mut percent = (se.health as f32 / se.maxhealth as f32) * HEALTH_WIDTH;

    //color of the bar
    let mut aColor: vec4_t = [0.0, 1.0, 0.0, 0.4];

    //color of the border — Raven fills it here and never reads it
    let mut _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing health"
    let mut cColor: vec4_t = [0.5, 0.5, 0.5, 0.4];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        HEALTH_WIDTH,
        HEALTH_HEIGHT,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + percent,
        y + 1.0,
        HEALTH_WIDTH - percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &cColor,
    );

    //now draw his ammo
    // `es.weapon` comes off the network snapshot unclamped; an out-of-range
    // value reads past `weaponData`/`ammoData` in Raven (UB) where the port's
    // fixed-size array indexing panics instead (§F19).
    let wd = &weaponData[es.weapon as usize];
    let mut ammoMax = ammoData[wd.ammoIndex as usize].max;
    if es.eFlags & EF_DOUBLE_AMMO != 0 {
        ammoMax *= 2;
    }

    x = chX + ((chW / 2.0) - (HEALTH_WIDTH / 2.0));
    y = (chY + chH) + HEALTH_HEIGHT + 10.0;

    if wd.energyPerShot == 0 && wd.altEnergyPerShot == 0 {
        //a weapon that takes no ammo, so show full
        percent = HEALTH_WIDTH;
    } else {
        percent = (se.ammo as f32 / ammoMax as f32) * HEALTH_WIDTH;
    }

    //color of the bar
    aColor[0] = 1.0;
    aColor[1] = 1.0;
    aColor[2] = 0.0;
    aColor[3] = 0.4;

    //color of the border
    _bColor[0] = 0.0;
    _bColor[1] = 0.0;
    _bColor[2] = 0.0;
    _bColor[3] = 0.3;

    //color of greyed out "missing health"
    cColor[0] = 0.5;
    cColor[1] = 0.5;
    cColor[2] = 0.5;
    cColor[3] = 0.4;

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        HEALTH_WIDTH,
        HEALTH_HEIGHT,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + percent,
        y + 1.0,
        HEALTH_WIDTH - percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &cColor,
    );
}

/// Raven `CG_DrawHealthBar` — the little bar under the crosshair over a
/// damageable entity, team-coloured where the entity has an owner.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4632-4688`
pub fn CG_DrawHealthBar(
    ctx: &mut CgContext,
    centNum: usize,
    chX: f32,
    chY: f32,
    chW: f32,
    chH: f32,
) {
    let es = ctx.world.entity(centNum).currentState;

    let x = chX + ((chW / 2.0) - (HEALTH_WIDTH / 2.0));
    let y = (chY + chH) + 8.0;
    let percent = (es.health as f32 / es.maxhealth as f32) * HEALTH_WIDTH;
    if percent <= 0.0 {
        return;
    }

    //color of the bar
    let aColor: vec4_t = if es.teamowner == 0 || ctx.world.cgs.gametype < GT_TEAM {
        //not owned by a team or teamplay
        [1.0, 1.0, 0.0, 0.4]
    } else if es.teamowner == ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] {
        //owned by my team
        [0.0, 1.0, 0.0, 0.4]
    } else {
        //hostile
        [1.0, 0.0, 0.0, 0.4]
    };

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing health"
    let cColor: vec4_t = [0.5, 0.5, 0.5, 0.4];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        HEALTH_WIDTH,
        HEALTH_HEIGHT,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + percent,
        y + 1.0,
        HEALTH_WIDTH - percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &cColor,
    );
}

/// Raven `CG_DrawHaqrBar` — the hacking progress bar under the crosshair, with
/// the hacker icon above it.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4691-4735`
pub fn CG_DrawHaqrBar(ctx: &mut CgContext, chX: f32, chY: f32, chW: f32, chH: f32) {
    let x = chX + ((chW / 2.0) - (HEALTH_WIDTH / 2.0));
    let y = (chY + chH) + 8.0;
    let percent = ((ctx.world.cg.predictedPlayerState.hackingTime as f32
        - ctx.world.cg.time as f32)
        / ctx.world.cg.predictedPlayerState.hackingBaseTime as f32)
        * HEALTH_WIDTH;

    if percent > HEALTH_WIDTH || percent < 1.0 {
        return;
    }

    //color of the bar
    let aColor: vec4_t = [1.0, 1.0, 0.0, 0.4];

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out done area
    let cColor: vec4_t = [0.5, 0.5, 0.5, 0.1];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        HEALTH_WIDTH,
        HEALTH_HEIGHT,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + percent,
        y + 1.0,
        HEALTH_WIDTH - percent - 1.0,
        HEALTH_HEIGHT - 1.0,
        &cColor,
    );

    //draw the hacker icon
    let hackerIconShader = ctx.world.cgs.media.hackerIconShader;
    CG_DrawPic(
        ctx,
        x,
        y - HEALTH_WIDTH,
        HEALTH_WIDTH,
        HEALTH_WIDTH,
        hackerIconShader,
    );
}

/// Raven `CG_DrawGenericTimerBar` — the scripted countdown bar on the right
/// edge, in whatever colour the server asked for.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4745-4790`
pub fn CG_DrawGenericTimerBar(ctx: &mut CgContext) {
    let x = CGTIMERBAR_X;
    let y = CGTIMERBAR_Y;
    let mut percent = ((ctx.world.draw.cg_genericTimerBar - ctx.world.cg.time) as f32
        / ctx.world.draw.cg_genericTimerDur as f32)
        * CGTIMERBAR_H;

    if percent > CGTIMERBAR_H {
        return;
    }

    if percent < 0.1 {
        percent = 0.1;
    }

    //color of the bar
    let aColor: vec4_t = ctx.world.draw.cg_genericTimerColor;

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing fuel"
    let cColor: vec4_t = [0.5, 0.5, 0.5, 0.1];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        CGTIMERBAR_W,
        CGTIMERBAR_H,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0 + (CGTIMERBAR_H - percent),
        CGTIMERBAR_W - 2.0,
        CGTIMERBAR_H - 1.0 - (CGTIMERBAR_H - percent),
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        CGTIMERBAR_W - 2.0,
        CGTIMERBAR_H - percent,
        &cColor,
    );
}

/// Raven `CG_DrawRocketLocking` — the eight wedges closing around a
/// rocket-lock target, plus the tick and lock sounds.
///
/// DEFERRED: `Vehicle_t` referent pool — when you are flying, Raven retimes
/// the lock off `m_pVehicleInfo->weapon[0/1].ID`'s `g_vehWeaponInfo` entry.
/// DEC-46.2's presence-only `Option<VehicleId>` cannot name that weapon, so
/// `vehWeapon` stays NULL and the hard-coded interval stands — the same arm
/// Raven takes for an out-of-range weapon ID.
/// Source: `oracle/codemp/cgame/cg_draw.c:5772-5807`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5747-5960`
pub fn CG_DrawRocketLocking(ctx: &mut CgContext, lockEntNum: usize, _lockTime: c_int) {
    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // lock to draw.
    let (rocketLockTime, snapClientNum, rocketLockIndex, m_iVehicleNum) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (
            snap.ps.rocketLockTime,
            snap.ps.clientNum,
            snap.ps.rocketLockIndex,
            snap.ps.m_iVehicleNum,
        )
    };

    let lockTimeInterval: f32 = (if ctx.world.cgs.gametype == GT_SIEGE {
        2400.0
    } else {
        1200.0
    }) / 16.0;
    //FIXME (Raven): if in a vehicle, use the vehicle's lockOnTime...
    let mut dif: c_int = ((ctx.world.cg.time as f32 - rocketLockTime) / lockTimeInterval) as c_int;

    if rocketLockTime == 0.0 {
        return;
    }

    if ctx.world.cgs.clientinfo[snapClientNum as usize].team == TEAM_SPECTATOR {
        return;
    }

    // The `if (cg.snap->ps.m_iVehicleNum)` / `if (veh->m_pVehicle)` pair here
    // exists only to reach the deferred `m_pVehicleInfo` weapon read (see the
    // fn doc), so nothing is transcribed for it.

    //We can't check to see in pmove if players are on the same team, so we resort
    //to just not drawing the lock if a teammate is the locked on ent
    if rocketLockIndex >= 0 && rocketLockIndex < ENTITYNUM_NONE {
        let ci_team = if rocketLockIndex < MAX_CLIENTS_I32 {
            Some(ctx.world.cgs.clientinfo[rocketLockIndex as usize].team)
        } else {
            ctx.world
                .entity(rocketLockIndex as usize)
                .npcClient
                .as_deref()
                .map(|ci| ci.team)
        };

        if let Some(ci_team) = ci_team {
            let myTeam = ctx.world.cgs.clientinfo[snapClientNum as usize].team;

            if ci_team == myTeam {
                if ctx.world.cgs.gametype >= GT_TEAM {
                    return;
                }
            } else if ctx.world.cgs.gametype >= GT_TEAM {
                let hitEnt = ctx.world.entity(rocketLockIndex as usize).currentState;
                if hitEnt.eType == entityType_t::ET_NPC as c_int
                    && hitEnt.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && hitEnt.owner < ENTITYNUM_WORLD
                {
                    //this is a vehicle, if it has a pilot and that pilot is on my team, then...
                    let pilot_team = if hitEnt.owner < MAX_CLIENTS_I32 {
                        Some(ctx.world.cgs.clientinfo[hitEnt.owner as usize].team)
                    } else {
                        ctx.world
                            .entity(hitEnt.owner as usize)
                            .npcClient
                            .as_deref()
                            .map(|ci| ci.team)
                    };
                    if pilot_team == Some(myTeam) {
                        return;
                    }
                }
            }
        }
    }

    if rocketLockTime != -1.0 {
        ctx.world.draw.lastvalidlockdif = dif;
    } else {
        dif = ctx.world.draw.lastvalidlockdif;
    }

    // Raven's `if (!cent)` can never fire — `cent` is `&cg_entities[lockEntNum]`.
    let org = ctx.world.entity(lockEntNum).lerpOrigin;

    if let Some((cx, mut cy)) = CG_WorldCoordToScreenCoord(ctx.world, org) {
        // we care about distance from enemy to eye, so this is good enough
        let mut sz = Distance(org, ctx.world.cg.refdef.vieworg) / 1024.0;

        if sz > 1.0 {
            sz = 1.0;
        } else if sz < 0.0 {
            sz = 0.0;
        }

        sz = (1.0 - sz) * (1.0 - sz) * 32.0 + 6.0;

        if m_iVehicleNum != 0 {
            sz *= 2.0;
        }

        cy = (cy as f32 + sz * 0.5) as c_int;

        if dif < 0 {
            ctx.world.draw.oldDif = 0;
            return;
        } else if dif > 8 {
            dif = 8;
        }

        // do sounds
        if ctx.world.draw.oldDif != dif {
            let sample = if dif == 8 {
                if m_iVehicleNum != 0 {
                    "sound/vehicles/weapons/common/lock.wav"
                } else {
                    "sound/weapons/rocket/lock.wav"
                }
            } else if m_iVehicleNum != 0 {
                "sound/vehicles/weapons/common/tick.wav"
            } else {
                "sound/weapons/rocket/tick.wav"
            };
            let sfx = trap::S_RegisterSound(ctx.engine, sample);
            trap::S_StartSound(ctx.engine, Some(&org), 0, CHAN_AUTO, sfx);
        }

        ctx.world.draw.oldDif = dif;

        let mut color: vec4_t = [0.0, 0.0, 0.0, 0.0];
        for i in 0..dif {
            color[0] = 1.0;
            color[1] = 0.0;
            color[2] = 0.0;
            color[3] = 0.1 * i as f32 + 0.2;

            trap::R_SetColor(ctx.engine, Some(&color));

            // our slices are offset by about 45 degrees.
            let wedge = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/2d/wedge");
            CG_DrawRotatePic(
                ctx,
                cx as f32 - sz,
                cy as f32 - sz,
                sz,
                sz,
                i as f32 * 45.0,
                wedge,
            );
        }

        // we are locked and loaded baby
        if dif == 8 {
            let pulse = (((ctx.world.cg.time as f32 * 0.05) as f64).sin() * 0.5 + 0.5) as f32;
            color[0] = pulse;
            color[1] = pulse;
            color[2] = pulse;
            color[3] = 1.0; // this art is additive, so the alpha value does nothing

            trap::R_SetColor(ctx.engine, Some(&color));

            let lock = trap::R_RegisterShaderNoMip(ctx.engine, "gfx/2d/lock");
            CG_DrawPic(
                ctx,
                cx as f32 - sz,
                cy as f32 - sz * 2.0,
                sz * 2.0,
                sz * 2.0,
                lock,
            );
        }
    }
}

/// Raven `CG_DrawFollow` — the "following <name>" banner while spectating;
/// `false` is Raven's qfalse, meaning we are not following anyone.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6815-6853`
pub fn CG_DrawFollow(ctx: &mut CgContext, ds: &DisplayState) -> bool {
    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot we are not
    // following anyone yet.
    let (pm_flags, snapClientNum) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return false;
        };
        (snap.ps.pm_flags, snap.ps.clientNum)
    };

    if (pm_flags & PMF_FOLLOW) == 0 {
        return false;
    }

    //	s = "following";
    let s = if ctx.world.cgs.gametype == GT_POWERDUEL {
        let duelTeam = ctx.world.cgs.clientinfo[snapClientNum as usize].duelTeam;

        if duelTeam == duelTeam_t::DUELTEAM_LONE as c_int {
            CG_GetStringEdString(ctx, "MP_INGAME", "FOLLOWINGLONE")
        } else if duelTeam == duelTeam_t::DUELTEAM_DOUBLE as c_int {
            CG_GetStringEdString(ctx, "MP_INGAME", "FOLLOWINGDOUBLE")
        } else {
            CG_GetStringEdString(ctx, "MP_INGAME", "FOLLOWING")
        }
    } else {
        CG_GetStringEdString(ctx, "MP_INGAME", "FOLLOWING")
    };

    let w = CG_Text_Width(ctx, ds, &s, 1.0, FONT_MEDIUM);
    CG_Text_Paint(
        ctx,
        ds,
        (320 - w / 2) as f32,
        60.0,
        1.0,
        colorWhite,
        &s,
        0.0,
        0,
        0,
        FONT_MEDIUM,
    );

    let s = buf_to_string(
        &ctx.world.cgs.clientinfo[snapClientNum as usize]
            .name
            .map(|c| c as u8),
    );
    let w = CG_Text_Width(ctx, ds, &s, 2.0, FONT_MEDIUM);
    CG_Text_Paint(
        ctx,
        ds,
        (320 - w / 2) as f32,
        80.0,
        2.0,
        colorWhite,
        &s,
        0.0,
        0,
        0,
        FONT_MEDIUM,
    );

    true
}

/// Raven `CG_DrawJetpackFuel` — the jetpack fuel meter, drawn only while the
/// tank is not full.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7150-7195`
pub fn CG_DrawJetpackFuel(ctx: &mut CgContext) {
    let x = JPFUELBAR_X;
    let y = JPFUELBAR_Y;

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // fuel level to draw.
    let Some(snap) = ctx.world.cg.snap_ref() else {
        return;
    };
    let mut percent = (snap.ps.jetpackFuel as f32 / 100.0) * JPFUELBAR_H;

    if percent > JPFUELBAR_H {
        return;
    }

    if percent < 0.1 {
        percent = 0.1;
    }

    //color of the bar
    let aColor: vec4_t = [0.5, 0.0, 0.0, 0.8];

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing fuel"
    let cColor: vec4_t = [0.5, 0.5, 0.5, 0.1];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        JPFUELBAR_W,
        JPFUELBAR_H,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0 + (JPFUELBAR_H - percent),
        JPFUELBAR_W - 1.0,
        JPFUELBAR_H - 1.0 - (JPFUELBAR_H - percent),
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        JPFUELBAR_W - 1.0,
        JPFUELBAR_H - percent,
        &cColor,
    );
}

/// Raven `CG_DrawEWebHealth` — the health meter of the e-web you are manning,
/// shuffled left past whichever fuel bars are already up.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7202-7258`
pub fn CG_DrawEWebHealth(ctx: &mut CgContext) {
    let mut x = EWEBHEALTH_X;
    let y = EWEBHEALTH_Y;
    let emplacedIndex = ctx.world.cg.predictedPlayerState.emplacedIndex as usize;
    let eweb = ctx.world.entity(emplacedIndex).currentState;
    let mut percent = (eweb.health as f32 / eweb.maxhealth as f32) * EWEBHEALTH_H;

    if percent > EWEBHEALTH_H {
        return;
    }

    if percent < 0.1 {
        percent = 0.1;
    }

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot we skip the
    // meter entirely rather than read fuel out of nothing.
    let (jetpackFuel, cloakFuel) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (snap.ps.jetpackFuel, snap.ps.cloakFuel)
    };

    //kind of hacky, need to pass a coordinate in here
    if jetpackFuel < 100 {
        x -= JPFUELBAR_W + 8.0;
    }
    if cloakFuel < 100 {
        x -= JPFUELBAR_W + 8.0;
    }

    //color of the bar
    let aColor: vec4_t = [0.5, 0.0, 0.0, 0.8];

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing fuel"
    let cColor: vec4_t = [0.5, 0.5, 0.5, 0.1];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        EWEBHEALTH_W,
        EWEBHEALTH_H,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much health there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0 + (EWEBHEALTH_H - percent),
        EWEBHEALTH_W - 1.0,
        EWEBHEALTH_H - 1.0 - (EWEBHEALTH_H - percent),
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        EWEBHEALTH_W - 1.0,
        EWEBHEALTH_H - percent,
        &cColor,
    );
}

/// Raven `CG_DrawCloakFuel` — the cloak fuel meter, shifted left when the
/// jetpack bar is already sharing that corner.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7265-7315`
pub fn CG_DrawCloakFuel(ctx: &mut CgContext) {
    let mut x = CLFUELBAR_X;
    let y = CLFUELBAR_Y;

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // fuel level to draw.
    let (jetpackFuel, cloakFuel) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (snap.ps.jetpackFuel, snap.ps.cloakFuel)
    };

    let mut percent = (cloakFuel as f32 / 100.0) * CLFUELBAR_H;

    if percent > CLFUELBAR_H {
        return;
    }

    if jetpackFuel < 100 {
        //if drawing jetpack fuel bar too, then move this over...?
        x -= JPFUELBAR_W + 8.0;
    }

    if percent < 0.1 {
        percent = 0.1;
    }

    //color of the bar
    let aColor: vec4_t = [0.0, 0.0, 0.6, 0.8];

    //color of the border — Raven fills it here and never reads it
    let _bColor: vec4_t = [0.0, 0.0, 0.0, 0.3];

    //color of greyed out "missing fuel"
    let cColor: vec4_t = [0.1, 0.1, 0.3, 0.1];

    //draw the background (black)
    CG_DrawRect(
        ctx,
        x,
        y,
        CLFUELBAR_W,
        CLFUELBAR_H,
        1.0,
        &colorTable[ct_table_t::CT_BLACK as usize],
    );

    //now draw the part to show how much fuel there is in the color specified
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0 + (CLFUELBAR_H - percent),
        CLFUELBAR_W - 1.0,
        CLFUELBAR_H - 1.0 - (CLFUELBAR_H - percent),
        &aColor,
    );

    //then draw the other part greyed out
    CG_FillRect(
        ctx,
        x + 1.0,
        y + 1.0,
        CLFUELBAR_W - 1.0,
        CLFUELBAR_H - percent,
        &cColor,
    );
}

/// Raven `CG_ChatBox_AddString` — latches one chat line into the ring buffer,
/// hard-wrapping it at the chatbox cutoff width first.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7557-7624`
pub fn CG_ChatBox_AddString(ctx: &mut CgContext, ds: &DisplayState, chatStr: &str) {
    if ctx.world.cvars.cg_chatBox.integer <= 0 {
        //don't bother then.
        return;
    }

    // §F19: Raven truncates at `> sizeof(chat->string)`, so an exactly
    // 150-character line still `strcpy`s 151 bytes over the slot's end; the
    // port's bounded copy stops at 149 characters plus the terminator. The
    // `\n` inserts below also grow a `String` where Raven's StrInsert overruns
    // its fixed 150-byte buffer on a heavily-wrapped line, so long lines keep
    // a different visible tail.
    // PORT-NOTE: Raven truncates the *caller's* buffer in place — every caller
    // passes a scratch buffer it does not re-read, so the port truncates its
    // own copy instead.
    let mut chatChars: Vec<char> = chatStr.chars().collect();
    if chatChars.len() > MAX_SAY_TEXT {
        chatChars.truncate(MAX_SAY_TEXT - 1);
    }
    let mut string: String = chatChars.into_iter().collect();

    let mut lines: c_int = 1;

    let mut chatLen = CG_Text_Width(ctx, ds, &string, 1.0, FONT_SMALL) as f32;
    if chatLen > CHATBOX_CUTOFF_LEN as f32 {
        //we have to break it into segments...
        let mut i: usize = 0;
        let mut lastLinePt: usize = 0;

        chatLen = 0.0;
        loop {
            let chars: Vec<char> = string.chars().collect();
            if i >= chars.len() {
                break;
            }

            let s: String = chars[i].to_string();
            chatLen += CG_Text_Width(ctx, ds, &s, 0.65, FONT_SMALL) as f32;

            if chatLen >= CHATBOX_CUTOFF_LEN as f32 {
                let mut j = i;
                while j > 0 && j > lastLinePt {
                    if chars[j] == ' ' {
                        break;
                    }
                    j -= 1;
                }
                if chars[j] == ' ' {
                    i = j;
                }

                lines += 1;
                CG_ChatBox_StrInsert(&mut string, i, "\n");
                i += 1;
                chatLen = 0.0;
                lastLinePt = i + 1;
            }
            i += 1;
        }
    }

    let time = ctx.world.cg.time + ctx.world.cvars.cg_chatBox.integer;
    let slot = ctx.world.cg.chatItemActive as usize;
    let bytes = string_to_latin1(&string);
    {
        let chat = &mut ctx.world.cg.chatItems[slot];
        // Raven's `memset(chat, 0, sizeof(chatBoxItem_t))`
        chat.string.fill(0);
        chat.time = 0;
        chat.lines = 0;

        let destsize = chat.string.len();
        Q_strncpyzBytes(&mut chat.string, &bytes, destsize);
        chat.time = time;
        chat.lines = lines;
    }

    ctx.world.cg.chatItemActive += 1;
    if ctx.world.cg.chatItemActive >= MAX_CHATBOX_ITEMS as c_int {
        ctx.world.cg.chatItemActive = 0;
    }
}

/// Raven `CG_ChatBox_DrawStrings` — paints the live chat lines bottom-up,
/// oldest at the top of the stack.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7643-7699`
pub fn CG_ChatBox_DrawStrings(ctx: &mut CgContext, ds: &DisplayState) {
    let mut drawThese: [Option<usize>; MAX_CHATBOX_ITEMS] = [None; MAX_CHATBOX_ITEMS];
    let mut numToDraw: usize = 0;
    let mut linesToDraw: c_int = 0;
    let x: c_int = 30;
    let mut y: c_int = if ctx.world.cg.scoreBoardShowing != qfalse {
        475
    } else {
        ctx.world.cvars.cg_chatBoxHeight.integer
    };
    let fontScale: f32 = 0.65;

    if ctx.world.cvars.cg_chatBox.integer == 0 {
        return;
    }

    for i in 0..MAX_CHATBOX_ITEMS {
        if ctx.world.cg.chatItems[i].time >= ctx.world.cg.time {
            let mut check = numToDraw as isize;
            let mut insertionPoint = numToDraw;

            while check >= 0 {
                //insert here
                if let Some(slot) = drawThese[check as usize] {
                    if ctx.world.cg.chatItems[i].time < ctx.world.cg.chatItems[slot].time {
                        insertionPoint = check as usize;
                    }
                }
                check -= 1;
            }
            CG_ChatBox_ArrayInsert(ctx, &mut drawThese, insertionPoint, MAX_CHATBOX_ITEMS, i);
            numToDraw += 1;
            linesToDraw += ctx.world.cg.chatItems[i].lines;
        }
    }

    if numToDraw == 0 {
        //nothing, then, just get out of here now.
        return;
    }

    //move initial point up so we draw bottom-up (visually)
    y = (y as f32 - (CHATBOX_FONT_HEIGHT as f32 * fontScale) * linesToDraw as f32) as c_int;

    //we have the items we want to draw, just quickly loop through them now
    for slot in drawThese.iter().take(numToDraw) {
        // Raven indexes `drawThese[i]` straight through; every slot below
        // `numToDraw` was filled by the insert above.
        let Some(slot) = *slot else {
            continue;
        };
        let string = buf_to_string(&ctx.world.cg.chatItems[slot].string.map(|c| c as u8));
        let lines = ctx.world.cg.chatItems[slot].lines;

        CG_Text_Paint(
            ctx,
            ds,
            x as f32,
            y as f32,
            fontScale,
            colorWhite,
            &string,
            0.0,
            0,
            ITEM_TEXTSTYLE_OUTLINED,
            FONT_SMALL,
        );
        y = (y as f32 + (CHATBOX_FONT_HEIGHT as f32 * fontScale) * lines as f32) as c_int;
    }
}

/// Raven `CG_DrawAmmo` — the ammo readout and its four tics, or the "--"
/// infinite marker for a weapon that costs nothing to fire.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:928-1035`
pub fn CG_DrawAmmo(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    ds: &DisplayState,
    centNum: usize,
    menuHUD: Option<MenuId>,
) {
    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // ammo count to paint.
    let ammo = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        snap.ps.ammo
    };

    // Can we find the menu?
    if menuHUD.is_none() {
        return;
    }

    let weapon = ctx.world.entity(centNum).currentState.weapon;
    if weapon == 0 {
        // We don't have a weapon right now
        return;
    }

    let ammoIndex = weaponData[weapon as usize].ammoIndex as usize;
    let mut value = ammo[ammoIndex] as f32;
    if value < 0.0 {
        // No ammo
        return;
    }

    // Raven's first `Menu_FindItemByName(menuHUD, "ammoamount")` here is redone
    // inside both arms below before anything reads it (host-side, no engine
    // call - dropped); the `R_SetColor` beside it is a live syscall and stays.
    let tint = ctx.world.draw.hudTintColor;
    trap::R_SetColor(ctx.engine, tint.as_ref());
    let mut inc: f32 = 0.0;

    if weaponData[weapon as usize].energyPerShot == 0
        && weaponData[weapon as usize].altEnergyPerShot == 0
    {
        //just draw "infinite"
        inc = 8.0 / MAX_HUD_TICS as f32;
        value = 8.0;

        let focusItem = Menu_FindItemByName(menus, menuHUD, "ammoinfinite");
        let tint = ctx.world.draw.hudTintColor;
        trap::R_SetColor(ctx.engine, tint.as_ref());
        if let Some(focusItem) = focusItem {
            let rect = menus.item(focusItem).window.rect;
            let foreColor = menus.item(focusItem).window.foreColor;
            // Raven passes `NUM_FONT_SMALL` where the fn wants a `UI_*` style
            // mask; the value doubles as `UI_RIGHT`, so that is how it draws.
            UI_DrawProportionalString(
                ctx,
                ds,
                rect.x as c_int,
                rect.y as c_int,
                "--",
                NUM_FONT_SMALL,
                foreColor,
            );
        }
    } else {
        let focusItem = Menu_FindItemByName(menus, menuHUD, "ammoamount");
        let tint = ctx.world.draw.hudTintColor;
        trap::R_SetColor(ctx.engine, tint.as_ref());
        if let Some(focusItem) = focusItem {
            if ctx.world.entity(centNum).currentState.eFlags & EF_DOUBLE_AMMO != 0 {
                inc = (ammoData[ammoIndex].max as f32 * 2.0) / MAX_HUD_TICS as f32;
            } else {
                inc = ammoData[ammoIndex].max as f32 / MAX_HUD_TICS as f32;
            }
            value = ammo[ammoIndex] as f32;

            let rect = menus.item(focusItem).window.rect;
            CG_DrawNumField(
                ctx,
                rect.x as c_int,
                rect.y as c_int,
                3,
                value as c_int,
                rect.w as c_int,
                rect.h as c_int,
                NUM_FONT_SMALL,
                false,
            );
        }
    }

    // Draw tics
    for i in (0..MAX_HUD_TICS).rev() {
        let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, ammoTicName[i]) else {
            continue;
        };

        // §F19: see `CG_DrawHealth` — a NULL `hudTintColor` reads as white.
        let mut calcColor = ctx
            .world
            .draw
            .hudTintColor
            .unwrap_or(colorTable[ct_table_t::CT_WHITE as usize]);

        if value <= 0.0 {
            // done
            break;
        } else if value < inc {
            // partial tic
            let percent = value / inc;
            calcColor[3] = percent;
        }

        trap::R_SetColor(ctx.engine, Some(&calcColor));

        let rect = menus.item(focusItem).window.rect;
        let background = menus.item(focusItem).window.background;
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);

        value -= inc;
    }
}

/// Raven `CG_DrawForceSelect` — the force-power carousel that pops up for a
/// second and a half after you cycle powers.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1421-1578`
pub fn CG_DrawForceSelect(ctx: &mut CgContext, ds: &DisplayState) {
    // Raven's `#ifdef _XBOX` nudges the whole carousel up by 50; the PC build
    // keeps 0.
    let yOffset: c_int = 0;

    // Raven's `x2`/`y2` are zeroed here and never read again — dropped.

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there are no
    // powers to show.
    let (health, forcePowerSelected, forcePowersKnown) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (
            snap.ps.stats[STAT_HEALTH as usize],
            snap.ps.fd.forcePowerSelected,
            snap.ps.fd.forcePowersKnown,
        )
    };

    // don't display if dead
    if health <= 0 {
        return;
    }

    if (ctx.world.cg.forceSelectTime + WEAPON_SELECT_TIME as f32) < ctx.world.cg.time as f32 {
        // Time is up for the HUD to display
        ctx.world.cg.forceSelect = forcePowerSelected;
        return;
    }

    if forcePowersKnown == 0 {
        return;
    }

    // count the number of powers owned
    let mut count: c_int = 0;
    for i in 0..NUM_FORCE_POWERS {
        if ForcePower_Valid(ctx.world, i) {
            count += 1;
        }
    }

    if count == 0 {
        // If no force powers, don't display
        return;
    }

    let sideMax: c_int = 3; // Max number of icons on the side

    // Calculate how many icons will appear to either side of the center one
    let holdCount = count - 1; // -1 for the center icon
    let (sideLeftIconCnt, sideRightIconCnt) = if holdCount == 0 {
        // No icons to either side
        (0, 0)
    } else if count > (2 * sideMax) {
        // Go to the max on each side
        (sideMax, sideMax)
    } else {
        // Less than max, so do the calc
        let left = holdCount / 2;
        (left, holdCount - left)
    };

    let smallIconSize: c_int = 30;
    let bigIconSize: c_int = 60;
    let pad: c_int = 12;

    let x: c_int = 320;
    let y: c_int = 425;

    // Raven measures a `length` for the background here and never draws one —
    // dropped.

    let forceSelect = ctx.world.cg.forceSelect;

    let mut i = BG_ProperForceIndex(forceSelect) - 1;
    if i < 0 {
        i = MAX_SHOWPOWERS;
    }

    trap::R_SetColor(ctx.engine, None);
    // Work backwards from current icon
    let mut holdX = x - ((bigIconSize / 2) + pad + smallIconSize);
    let mut iconCnt: c_int = 1;
    while iconCnt < (sideLeftIconCnt + 1) {
        if i < 0 {
            i = MAX_SHOWPOWERS;
        }

        // §F19: Raven wraps to `MAX_SHOWPOWERS`, one slot past
        // `forcePowerSorted`'s end, and reads whatever follows the array; the
        // port treats that slot as no power at all.
        let Some(&power) = forcePowerSorted.get(i as usize) else {
            i -= 1;
            continue;
        };

        if !ForcePower_Valid(ctx.world, power) {
            // Does he have this power?
            i -= 1;
            continue;
        }

        iconCnt += 1; // Good icon

        let icon = ctx.world.cgs.media.forcePowerIcons[power as usize];
        if icon != 0 {
            CG_DrawPic(
                ctx,
                holdX as f32,
                (y + yOffset) as f32,
                smallIconSize as f32,
                smallIconSize as f32,
                icon,
            );
            holdX -= smallIconSize + pad;
        }
        i -= 1;
    }

    if ForcePower_Valid(ctx.world, forceSelect) {
        // Current Center Icon
        let icon = ctx.world.cgs.media.forcePowerIcons[forceSelect as usize];
        if icon != 0 {
            //only cache the icon for display
            CG_DrawPic(
                ctx,
                (x - (bigIconSize / 2)) as f32,
                ((y - ((bigIconSize - smallIconSize) / 2)) + yOffset) as f32,
                bigIconSize as f32,
                bigIconSize as f32,
                icon,
            );
        }
    }

    let mut i = BG_ProperForceIndex(forceSelect) + 1;
    if i > MAX_SHOWPOWERS {
        i = 0;
    }

    // Work forwards from current icon
    let mut holdX = x + (bigIconSize / 2) + pad;
    let mut iconCnt: c_int = 1;
    while iconCnt < (sideRightIconCnt + 1) {
        if i > MAX_SHOWPOWERS {
            i = 0;
        }

        // §F19: same one-past-the-end slot as the backwards loop above.
        let Some(&power) = forcePowerSorted.get(i as usize) else {
            i += 1;
            continue;
        };

        if !ForcePower_Valid(ctx.world, power) {
            // Does he have this power?
            i += 1;
            continue;
        }

        iconCnt += 1; // Good icon

        let icon = ctx.world.cgs.media.forcePowerIcons[power as usize];
        if icon != 0 {
            //only cache the icon for display
            CG_DrawPic(
                ctx,
                holdX as f32,
                (y + yOffset) as f32,
                smallIconSize as f32,
                smallIconSize as f32,
                icon,
            );
            holdX += smallIconSize + pad;
        }
        i += 1;
    }

    // §F19: Raven indexes `showPowersName` with `cg.forceSelect` unchecked, and
    // that is -1 before the first cycle; the port reads no name there.
    if let Some(name) = showPowersName.get(forceSelect as usize).copied().flatten() {
        let s = CG_GetStringEdString(ctx, "SP_INGAME", name);
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            y + 30 + yOffset,
            &s,
            UI_CENTER | UI_SMALLFONT,
            colorTable[ct_table_t::CT_ICON_BLUE as usize],
        );
    }
}

/// Raven `CG_DrawInvenSelect` — the holdable-item carousel, same pop-up rules
/// as the force one.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1585-1789`
pub fn CG_DrawInvenSelect(ctx: &mut CgContext, ds: &DisplayState) {
    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is no
    // inventory to show.
    let (health, holdableItem, holdableItems) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (
            snap.ps.stats[STAT_HEALTH as usize],
            snap.ps.stats[STAT_HOLDABLE_ITEM as usize],
            snap.ps.stats[STAT_HOLDABLE_ITEMS as usize],
        )
    };

    // don't display if dead
    if health <= 0 {
        return;
    }

    if (ctx.world.cg.invenSelectTime + WEAPON_SELECT_TIME as f32) < ctx.world.cg.time as f32 {
        // Time is up for the HUD to display
        return;
    }

    if holdableItem == 0 || holdableItems == 0 {
        return;
    }

    if ctx.world.cg.itemSelect == -1 {
        let tag = match ctx.world.cg.snap_ref() {
            Some(snap) => selected_holdable_tag(&snap.ps),
            None => return,
        };
        ctx.world.cg.itemSelect = tag;
    }

    //const int bits = cg.snap->ps.stats[ STAT_ITEMS ];

    // count the number of items owned
    let mut count: c_int = 0;
    for i in 0..HI_NUM_HOLDABLE {
        //CG_InventorySelectable(i) && inv_icons[i]
        if holdableItems & (1 << i) != 0 {
            count += 1;
        }
    }

    if count == 0 {
        let y2: c_int = 0; //err?
        UI_DrawProportionalString(
            ctx,
            ds,
            320,
            y2 + 22,
            "EMPTY INVENTORY",
            UI_CENTER | UI_SMALLFONT,
            colorTable[ct_table_t::CT_ICON_BLUE as usize],
        );
        return;
    }

    let sideMax: c_int = 3; // Max number of icons on the side

    // Calculate how many icons will appear to either side of the center one
    let holdCount = count - 1; // -1 for the center icon
    let (sideLeftIconCnt, sideRightIconCnt) = if holdCount == 0 {
        // No icons to either side
        (0, 0)
    } else if count > (2 * sideMax) {
        // Go to the max on each side
        (sideMax, sideMax)
    } else {
        // Less than max, so do the calc
        let left = holdCount / 2;
        (left, holdCount - left)
    };

    let itemSelect = ctx.world.cg.itemSelect;

    let mut i = itemSelect - 1;
    if i < 0 {
        i = HI_NUM_HOLDABLE - 1;
    }

    let smallIconSize: c_int = 40;
    let bigIconSize: c_int = 80;
    let pad: c_int = 16;

    let x: c_int = 320;
    let y: c_int = 410;

    // Raven's `height` and `addX` are written in all three blocks below and
    // never read — the `CG_DrawNumField` calls that used them are commented out
    // at every site, so the dead stores are dropped.

    // Left side ICONS
    // Work backwards from current icon
    let mut holdX = x - ((bigIconSize / 2) + pad + smallIconSize);
    let mut iconCnt: c_int = 0;
    while iconCnt < sideLeftIconCnt {
        if i < 0 {
            i = HI_NUM_HOLDABLE - 1;
        }

        if (holdableItems & (1 << i)) == 0 || i == itemSelect {
            i -= 1;
            continue;
        }

        iconCnt += 1; // Good icon

        if BG_IsItemSelectable(&mut ctx.world.cg.predictedPlayerState, i) == qfalse {
            i -= 1;
            continue;
        }

        let icon = ctx.world.cgs.media.invenIcons[i as usize];
        if icon != 0 {
            trap::R_SetColor(ctx.engine, None);
            CG_DrawPic(
                ctx,
                holdX as f32,
                (y + 10) as f32,
                smallIconSize as f32,
                smallIconSize as f32,
                icon,
            );

            trap::R_SetColor(
                ctx.engine,
                Some(&colorTable[ct_table_t::CT_ICON_BLUE as usize]),
            );

            holdX -= smallIconSize + pad;
        }
        i -= 1;
    }

    // Current Center Icon
    let centerIcon = ctx.world.cgs.media.invenIcons[itemSelect as usize];
    if centerIcon != 0
        && BG_IsItemSelectable(&mut ctx.world.cg.predictedPlayerState, itemSelect) != qfalse
    {
        trap::R_SetColor(ctx.engine, None);
        CG_DrawPic(
            ctx,
            (x - (bigIconSize / 2)) as f32,
            ((y - ((bigIconSize - smallIconSize) / 2)) + 10) as f32,
            bigIconSize as f32,
            bigIconSize as f32,
            centerIcon,
        );
        trap::R_SetColor(
            ctx.engine,
            Some(&colorTable[ct_table_t::CT_ICON_BLUE as usize]),
        );

        let itemNdex = BG_GetItemIndexByTag(itemSelect, IT_HOLDABLE);
        // Raven null-checks `bg_itemlist[itemNdex].classname`; the ported
        // `GItem::classname` is a plain `&'static str`, so the arm always runs.
        let classname = bg_itemlist[itemNdex as usize].classname;
        let textColor: vec4_t = [0.312, 0.75, 0.621, 1.0];

        // Raven's `Q_strupr` is the C-locale `toupper`, i.e. ASCII only.
        let upperKey = classname.to_ascii_uppercase();
        let text = trap::SP_GetStringTextString(ctx.engine, &format!("SP_INGAME_{upperKey}"), 1024);

        match text {
            Some(text) => UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y + 45,
                &text,
                UI_CENTER | UI_SMALLFONT,
                textColor,
            ),
            None => UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y + 45,
                classname,
                UI_CENTER | UI_SMALLFONT,
                textColor,
            ),
        }
    }

    let mut i = itemSelect + 1;
    if i > HI_NUM_HOLDABLE - 1 {
        i = 0;
    }

    // Right side ICONS
    // Work forwards from current icon
    let mut holdX = x + (bigIconSize / 2) + pad;
    let mut iconCnt: c_int = 0;
    while iconCnt < sideRightIconCnt {
        if i > HI_NUM_HOLDABLE - 1 {
            i = 0;
        }

        if (holdableItems & (1 << i)) == 0 || i == itemSelect {
            i += 1;
            continue;
        }

        iconCnt += 1; // Good icon

        if BG_IsItemSelectable(&mut ctx.world.cg.predictedPlayerState, i) == qfalse {
            i += 1;
            continue;
        }

        let icon = ctx.world.cgs.media.invenIcons[i as usize];
        if icon != 0 {
            trap::R_SetColor(ctx.engine, None);
            CG_DrawPic(
                ctx,
                holdX as f32,
                (y + 10) as f32,
                smallIconSize as f32,
                smallIconSize as f32,
                icon,
            );

            trap::R_SetColor(
                ctx.engine,
                Some(&colorTable[ct_table_t::CT_ICON_BLUE as usize]),
            );

            holdX += smallIconSize + pad;
        }
        i += 1;
    }
}

/// Raven `CG_DrawVehicleHud` — the whole swoop/fighter HUD panel. `true` is
/// Raven's qtrue: draw the ordinary player HUD on top of this.
///
/// Raven takes a `cent` it never reads; the panel is built from
/// `cg.predictedPlayerState.m_iVehicleNum` instead.
///
/// DEFERRED: `Vehicle_t` referent pool — the ammo-bar pick and the `hideRider`
/// damage panel both read `veh->m_pVehicle->m_pVehicleInfo`, which DEC-46.2's
/// presence-only `Option<VehicleId>` cannot supply; see
/// [`CG_DrawVehicleShields`]. Every call Raven makes before that read is
/// transcribed.
/// Source: `oracle/codemp/cgame/cg_draw.c:2665-2687`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2585-2691`
pub fn CG_DrawVehicleHud(ctx: &mut CgContext, menus: &MenuSystem, _centNum: usize) -> bool {
    let menuHUD = Menus_FindByName(menus, "swoopvehiclehud");
    if menuHUD.is_none() {
        return true; // Draw player HUD
    }

    // Raven's `!ps` can never fire — `ps` is `&cg.predictedPlayerState`.
    let m_iVehicleNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
    if m_iVehicleNum == 0 {
        return true; // Draw player HUD
    }

    // Raven's `if (!veh)` can never fire either — `veh` is `&cg_entities[…]`.
    let vehNum = m_iVehicleNum as usize;

    CG_DrawVehicleTurboRecharge(ctx, menus, menuHUD, vehNum);
    CG_DrawVehicleWeaponsLinked(ctx, menus, menuHUD, vehNum);

    // Draw frame
    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "leftframe") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "rightframe") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    CG_DrawVehicleArmor(ctx, menus, menuHUD, vehNum);

    // Raven's "get animal hud for speed" menu swap either side of the speed bar
    // is commented out at the site.
    CG_DrawVehicleSpeed(ctx, menus, menuHUD, vehNum);

    let _shieldPerc = CG_DrawVehicleShields(ctx, menus, menuHUD, vehNum);

    //TODO: Port CG_DrawVehicleHud vehicle arm
    // DEFERRED: Vehicle_t referent pool — `m_pVehicleInfo->weapon[0/1].ID` and
    // `->hideRider`; see the fn doc.
    // Source: oracle/codemp/cgame/cg_draw.c:2665-2687
    todo!("CG_DrawVehicleHud weapon/hideRider arms — blocked on the Vehicle_t referent pool, oracle/codemp/cgame/cg_draw.c:2665-2687")
}

/// Raven `CG_DrawDuelistHealth` — one duelist's health bar on the spectator
/// interface; the bar reddens as it empties.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3132-3159`
pub fn CG_DrawDuelistHealth(ctx: &mut CgContext, x: f32, y: f32, w: f32, h: f32, duelist: c_int) {
    let mut duelHealthColor: vec4_t = [1.0, 0.0, 0.0, 0.7];
    let mut healthSrc: f32 = 0.0;

    if duelist == 1 {
        healthSrc = ctx.world.cgs.duelist1health as f32;
    } else if duelist == 2 {
        healthSrc = ctx.world.cgs.duelist2health as f32;
    }

    let mut ratio = healthSrc / MAX_HEALTH_FOR_IFACE as f32;
    if ratio > 1.0 {
        ratio = 1.0;
    }
    if ratio < 0.0 {
        ratio = 0.0;
    }
    duelHealthColor[0] = (ratio * 0.2) + 0.5;

    // new art for this?  I'm not crazy about how this looks.
    CG_DrawHealthBarRough(
        ctx,
        x,
        y,
        w as c_int,
        h as c_int,
        ratio,
        &duelHealthColor,
        &colorTable[ct_table_t::CT_WHITE as usize],
    );
}

/// Raven `CG_DrawTeamOverlay` — the team roster strip: name, location, health,
/// armor, weapon icon and powerups per teammate. Returns the y the next HUD
/// element starts at.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3779-3949`
pub fn CG_DrawTeamOverlay(
    ctx: &mut CgContext,
    ds: &DisplayState,
    mut y: f32,
    right: bool,
    upper: bool,
) -> f32 {
    // Raven's `#ifdef _XBOX` shifts the strip by -40; the PC build keeps 0.
    let xOffset: c_int = 0;

    if ctx.world.cvars.cg_drawTeamOverlay.integer == 0 {
        return y;
    }

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot we are on no
    // team yet.
    let myTeam = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.ps.persistant[PERS_TEAM as usize],
        None => return y,
    };

    if myTeam != TEAM_RED && myTeam != TEAM_BLUE {
        return y; // Not on any team
    }

    let mut plyrs: c_int = 0;

    // max player name width
    let mut pwidth: c_int = 0;
    let numSorted = ctx.world.draw.numSortedTeamPlayers;
    let count = if numSorted > 8 { 8 } else { numSorted };
    for i in 0..count as usize {
        let ciNum = ctx.world.draw.sortedTeamPlayers[i] as usize;
        let ci = &ctx.world.cgs.clientinfo[ciNum];
        if ci.infoValid != qfalse && ci.team == myTeam {
            plyrs += 1;
            let name = buf_to_string(&ci.name.map(|c| c as u8));
            let len = CG_DrawStrlen(&name);
            if len > pwidth {
                pwidth = len;
            }
        }
    }

    if plyrs == 0 {
        return y;
    }

    if pwidth > TEAM_OVERLAY_MAXNAME_WIDTH {
        pwidth = TEAM_OVERLAY_MAXNAME_WIDTH;
    }

    // max location name width
    let mut lwidth: c_int = 0;
    for i in 1..MAX_LOCATIONS {
        let cs = CG_ConfigString(ctx, CS_LOCATIONS + i);
        let p = CG_GetLocationString(ctx, &cs);
        if !p.is_empty() {
            let len = CG_DrawStrlen(&p);
            if len > lwidth {
                lwidth = len;
            }
        }
    }

    if lwidth > TEAM_OVERLAY_MAXLOCATION_WIDTH {
        lwidth = TEAM_OVERLAY_MAXLOCATION_WIDTH;
    }

    let w = (pwidth + lwidth + 4 + 7) * TINYCHAR_WIDTH;

    let x = if right { 640 - w } else { 0 };

    let h = plyrs * TINYCHAR_HEIGHT;

    // Raven's `ret_y` is an int, so the fractional part of `y` is dropped here.
    let ret_y: c_int;
    if upper {
        ret_y = (y + h as f32) as c_int;
    } else {
        y -= h as f32;
        ret_y = y as c_int;
    }

    let mut hcolor: vec4_t = [0.0; 4];
    if myTeam == TEAM_RED {
        hcolor[0] = 1.0;
        hcolor[1] = 0.0;
        hcolor[2] = 0.0;
        hcolor[3] = 0.33;
    } else {
        // if ( cg.snap->ps.persistant[PERS_TEAM] == TEAM_BLUE )
        hcolor[0] = 0.0;
        hcolor[1] = 0.0;
        hcolor[2] = 1.0;
        hcolor[3] = 0.33;
    }
    trap::R_SetColor(ctx.engine, Some(&hcolor));
    let teamStatusBar = ctx.world.cgs.media.teamStatusBar;
    CG_DrawPic(
        ctx,
        (x + xOffset) as f32,
        y,
        w as f32,
        h as f32,
        teamStatusBar,
    );
    trap::R_SetColor(ctx.engine, None);

    for i in 0..count as usize {
        let ciNum = ctx.world.draw.sortedTeamPlayers[i] as usize;
        let infoValid = ctx.world.cgs.clientinfo[ciNum].infoValid;
        let team = ctx.world.cgs.clientinfo[ciNum].team;
        if infoValid == qfalse || team != myTeam {
            continue;
        }

        hcolor[0] = 1.0;
        hcolor[1] = 1.0;
        hcolor[2] = 1.0;
        hcolor[3] = 1.0;

        let mut xx = x + TINYCHAR_WIDTH;

        let name = buf_to_string(&ctx.world.cgs.clientinfo[ciNum].name.map(|c| c as u8));
        CG_DrawStringExt(
            ctx,
            ds,
            xx + xOffset,
            y as c_int,
            &name,
            &hcolor,
            false,
            false,
            TINYCHAR_WIDTH,
            TINYCHAR_HEIGHT,
            TEAM_OVERLAY_MAXNAME_WIDTH,
        );

        if lwidth != 0 {
            let location = ctx.world.cgs.clientinfo[ciNum].location;
            let cs = CG_ConfigString(ctx, CS_LOCATIONS + location);
            let mut p = CG_GetLocationString(ctx, &cs);
            if p.is_empty() {
                p = "unknown".to_string();
            }
            // Raven clamps a `len` here for a centering calc that is commented
            // out below it, so the clamp goes nowhere — dropped.

            xx = x + TINYCHAR_WIDTH * 2 + TINYCHAR_WIDTH * pwidth;
            CG_DrawStringExt(
                ctx,
                ds,
                xx + xOffset,
                y as c_int,
                &p,
                &hcolor,
                false,
                false,
                TINYCHAR_WIDTH,
                TINYCHAR_HEIGHT,
                TEAM_OVERLAY_MAXLOCATION_WIDTH,
            );
        }

        let health = ctx.world.cgs.clientinfo[ciNum].health;
        let armor = ctx.world.cgs.clientinfo[ciNum].armor;
        hcolor = CG_GetColorForHealth(health, armor);

        let st = format!("{health:3} {armor:3}");

        xx = x + TINYCHAR_WIDTH * 3 + TINYCHAR_WIDTH * pwidth + TINYCHAR_WIDTH * lwidth;

        CG_DrawStringExt(
            ctx,
            ds,
            xx + xOffset,
            y as c_int,
            &st,
            &hcolor,
            false,
            false,
            TINYCHAR_WIDTH,
            TINYCHAR_HEIGHT,
            0,
        );

        // draw weapon icon
        xx += TINYCHAR_WIDTH * 3;

        let curWeapon = ctx.world.cgs.clientinfo[ciNum].curWeapon;
        let weaponIcon = ctx.world.cg_weapons[curWeapon as usize].weaponIcon;
        if weaponIcon != 0 {
            CG_DrawPic(
                ctx,
                (xx + xOffset) as f32,
                y,
                TINYCHAR_WIDTH as f32,
                TINYCHAR_HEIGHT as f32,
                weaponIcon,
            );
        } else {
            let deferShader = ctx.world.cgs.media.deferShader;
            CG_DrawPic(
                ctx,
                (xx + xOffset) as f32,
                y,
                TINYCHAR_WIDTH as f32,
                TINYCHAR_HEIGHT as f32,
                deferShader,
            );
        }

        // Draw powerup icons
        xx = if right { x } else { x + w - TINYCHAR_WIDTH };

        let powerups = ctx.world.cgs.clientinfo[ciNum].powerups;
        for j in 0..=PW_NUM_POWERUPS {
            if powerups & (1 << j) == 0 {
                continue;
            }

            let Some(item) = BG_FindItemForPowerup(j) else {
                continue;
            };

            // §F19: Raven hands `item->icon` straight to the trap; the ported
            // icon is optional, and an item without one registers the empty
            // name rather than dereferencing NULL.
            let icon = item.item().icon.unwrap_or("");
            let shader = trap::R_RegisterShader(ctx.engine, icon);
            CG_DrawPic(
                ctx,
                (xx + xOffset) as f32,
                y,
                TINYCHAR_WIDTH as f32,
                TINYCHAR_HEIGHT as f32,
                shader,
            );
            if right {
                xx -= TINYCHAR_WIDTH;
            } else {
                xx += TINYCHAR_WIDTH;
            }
        }

        y += TINYCHAR_HEIGHT as f32;
    }

    ret_y as f32
}

/// Raven `CG_DrawPowerupIcons` — the powerup column down the right edge, each
/// icon over its remaining seconds.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3952-4011`
pub fn CG_DrawPowerupIcons(ctx: &mut CgContext, ds: &DisplayState, mut y: c_int) {
    let ico_size: c_int = 64;
    // Raven's `#ifdef _XBOX` shifts the column by -40; the PC build keeps 0.
    let xOffset: c_int = 0;

    let powerups = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        snap.ps.powerups
    };

    y += 16;

    for j in 0..=PW_NUM_POWERUPS {
        // §F19: Raven's `j <= PW_NUM_POWERUPS` runs one slot past `ps.powerups`
        // (16 entries) and reads whatever follows it; the port skips that slot.
        let Some(&pwTime) = powerups.get(j as usize) else {
            continue;
        };

        if pwTime <= ctx.world.cg.time {
            continue;
        }

        let secondsleft = (pwTime - ctx.world.cg.time) / 1000;

        let Some(item) = BG_FindItemForPowerup(j) else {
            continue;
        };

        let icoShader = if ctx.world.cgs.gametype == GT_CTY && (j == PW_REDFLAG || j == PW_BLUEFLAG)
        {
            if j == PW_REDFLAG {
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_rflag_ys")
            } else {
                trap::R_RegisterShaderNoMip(ctx.engine, "gfx/hud/mpi_bflag_ys")
            }
        } else {
            // §F19: see `CG_DrawTeamOverlay` — an item with no icon registers
            // the empty name rather than dereferencing NULL.
            let icon = item.item().icon.unwrap_or("");
            trap::R_RegisterShader(ctx.engine, icon)
        };

        CG_DrawPic(
            ctx,
            ((640.0 - (ico_size as f64 * 1.1)) + xOffset as f64) as f32,
            y as f32,
            ico_size as f32,
            ico_size as f32,
            icoShader,
        );

        y += ico_size;

        if j != PW_REDFLAG && j != PW_BLUEFLAG && secondsleft < 999 {
            UI_DrawProportionalString(
                ctx,
                ds,
                ((640.0 - (ico_size as f64 * 1.1)) + (ico_size / 2) as f64 + xOffset as f64)
                    as c_int,
                y - 8,
                &format!("{secondsleft}"),
                UI_CENTER | UI_BIGFONT | UI_DROPSHADOW,
                colorTable[ct_table_t::CT_WHITE as usize],
            );
        }

        y += ico_size / 3;
    }
}

/// Raven `CG_DrawCrosshair` — the crosshair itself, tinted by whatever it sits
/// over, plus the health/siege/hack bars that hang under it. `worldPoint` is
/// Raven's optional world anchor (NULL falls back to the `cg_crosshairX/Y`
/// cvars).
///
/// DEFERRED: `Vehicle_t` referent pool — a ship's own crosshair art is
/// `vehCent->m_pVehicle->m_pVehicleInfo->crosshairShaderHandle`, unreachable
/// through DEC-46.2's presence-only `Option<VehicleId>` (see
/// [`CG_DrawVehicleShields`]), so `hShader` stays 0 and the default
/// `cgs.media.crosshairShader` is drawn — Raven's arm for a vehicle with no
/// crosshair of its own. The doubled size still applies.
/// Source: `oracle/codemp/cgame/cg_draw.c:5163-5170`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4848-5268`
pub fn CG_DrawCrosshair(ctx: &mut CgContext, worldPoint: Option<vec3_t>, chEntValid: c_int) {
    let mut hShader: qhandle_t = 0;
    let mut corona = false;
    let mut ecolor: vec4_t = [0.0, 0.0, 0.0, 0.0];
    let mut crossEnt: Option<usize> = None;

    if let Some(worldPoint) = worldPoint {
        _VectorCopy(worldPoint, &mut ctx.world.draw.cg_crosshairPos);
    }

    if ctx.world.cvars.cg_drawCrosshair.integer == 0 {
        return;
    }

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot there is nothing
    // to aim at.
    let (fallingToDeath, snapClientNum, duelInProgress, duelIndex) = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        (
            snap.ps.fallingToDeath,
            snap.ps.clientNum as usize,
            snap.ps.duelInProgress,
            snap.ps.duelIndex,
        )
    };

    if fallingToDeath != 0 {
        return;
    }

    if ctx.world.cg.predictedPlayerState.zoomMode != 0 {
        //not while scoped
        return;
    }

    // Raven's "in vehicle, rocket lock-on replaces crosshair" early-out is
    // commented out at the site.

    if ctx.world.cvars.cg_crosshairHealth.integer != 0 {
        let hcolor = CG_ColorForHealth(ctx.world);
        trap::R_SetColor(ctx.engine, Some(&hcolor));
    } else {
        //set color based on what kind of ent is under crosshair
        let chNum = ctx.world.cg.crosshairClientNum;
        if chNum >= ENTITYNUM_WORLD {
            trap::R_SetColor(ctx.engine, None);
        } else if chEntValid != 0 {
            // Raven reads `cg_entities[…]` only behind the `chEntValid` guard,
            // so the entity load stays inside it here too.
            let es = ctx.world.entity(chNum as usize).currentState;
            //rwwFIXMEFIXME: Write this a different way, it's getting a bit too sloppy looking
            let looking = es.number < MAX_CLIENTS_I32
                || es.eType == entityType_t::ET_NPC as c_int
                || es.shouldtarget != qfalse
                //always show ents with health data under crosshair
                || es.health != 0
                || (es.eType == entityType_t::ET_MOVER as c_int
                    && es.bolt1 != 0
                    && ctx.world.cg.predictedPlayerState.weapon == WP_SABER)
                || (es.eType == entityType_t::ET_MOVER as c_int && es.teamowner != 0);

            if looking {
                crossEnt = Some(chNum as usize);

                let gametype = ctx.world.cgs.gametype;
                let myPersTeam = ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize];
                let myClientTeam = ctx.world.cgs.clientinfo[snapClientNum].team;

                if es.powerups & (1 << PW_CLOAKED) != 0 {
                    //don't show up for cloaked guys
                    ecolor[0] = 1.0; //R
                    ecolor[1] = 1.0; //G
                    ecolor[2] = 1.0; //B
                } else if es.number < MAX_CLIENTS_I32 {
                    if gametype >= GT_TEAM
                        && ctx.world.cgs.clientinfo[es.number as usize].team == myClientTeam
                    {
                        //Allies are green
                        ecolor[0] = 0.0; //R
                        ecolor[1] = 1.0; //G
                        ecolor[2] = 0.0; //B
                    } else if gametype == GT_POWERDUEL
                        && ctx.world.cgs.clientinfo[es.number as usize].duelTeam
                            == ctx.world.cgs.clientinfo[snapClientNum].duelTeam
                    {
                        //on the same duel team in powerduel, so he's a friend
                        ecolor[0] = 0.0; //R
                        ecolor[1] = 1.0; //G
                        ecolor[2] = 0.0; //B
                    } else {
                        //Enemies are red
                        ecolor[0] = 1.0; //R
                        ecolor[1] = 0.0; //G
                        ecolor[2] = 0.0; //B
                    }

                    if duelInProgress != qfalse {
                        if es.number != duelIndex {
                            //grey out crosshair for everyone but your foe if you're in a duel
                            ecolor[0] = 0.4;
                            ecolor[1] = 0.4;
                            ecolor[2] = 0.4;
                        }
                    } else if es.bolt1 != 0 {
                        //this fellow is in a duel. We just checked if we were in a duel above, so
                        //this means we aren't and he is. Which of course means our crosshair greys out over him.
                        ecolor[0] = 0.4;
                        ecolor[1] = 0.4;
                        ecolor[2] = 0.4;
                    }
                } else if es.shouldtarget != qfalse || es.eType == entityType_t::ET_NPC as c_int {
                    //VectorCopy( crossEnt->startRGBA, ecolor ); — commented out at the site
                    if ecolor[0] == 0.0 && ecolor[1] == 0.0 && ecolor[2] == 0.0 {
                        // We really don't want black, so set it to yellow
                        ecolor[0] = 1.0; //R
                        ecolor[1] = 0.8; //G
                        ecolor[2] = 0.3; //B
                    }

                    if es.eType == entityType_t::ET_NPC as c_int {
                        let plTeam = if gametype == GT_SIEGE {
                            myPersTeam
                        } else {
                            NPCTEAM_PLAYER
                        };

                        if es.powerups & (1 << PW_CLOAKED) != 0 {
                            ecolor[0] = 1.0; //R
                            ecolor[1] = 1.0; //G
                            ecolor[2] = 1.0; //B
                        } else if es.teamowner == 0 {
                            //not on a team
                            // Raven re-tests `!teamowner` here; the arm above
                            // already guarantees it, so the `||` never matters.
                            if es.teamowner == 0 || es.NPC_class == class_t::CLASS_VEHICLE as c_int
                            {
                                //neutral
                                if es.owner < MAX_CLIENTS_I32 {
                                    //base color on who is pilotting this thing
                                    let ci_team = ctx.world.cgs.clientinfo[es.owner as usize].team;

                                    if gametype >= GT_TEAM && ci_team == myPersTeam {
                                        //friendly
                                        ecolor[0] = 0.0; //R
                                        ecolor[1] = 1.0; //G
                                        ecolor[2] = 0.0; //B
                                    } else {
                                        //hostile
                                        ecolor[0] = 1.0; //R
                                        ecolor[1] = 0.0; //G
                                        ecolor[2] = 0.0; //B
                                    }
                                } else {
                                    //unmanned
                                    ecolor[0] = 1.0; //R
                                    ecolor[1] = 1.0; //G
                                    ecolor[2] = 0.0; //B
                                }
                            } else {
                                ecolor[0] = 1.0; //R
                                ecolor[1] = 0.0; //G
                                ecolor[2] = 0.0; //B
                            }
                        } else if es.teamowner != plTeam {
                            // on enemy team
                            ecolor[0] = 1.0; //R
                            ecolor[1] = 0.0; //G
                            ecolor[2] = 0.0; //B
                        } else {
                            //a friend
                            ecolor[0] = 0.0; //R
                            ecolor[1] = 1.0; //G
                            ecolor[2] = 0.0; //B
                        }
                    } else if es.teamowner == TEAM_RED || es.teamowner == TEAM_BLUE {
                        if gametype < GT_TEAM {
                            //not teamplay, just neutral then
                            ecolor[0] = 1.0; //R
                            ecolor[1] = 1.0; //G
                            ecolor[2] = 0.0; //B
                        } else if es.teamowner != myClientTeam {
                            //on the enemy team
                            ecolor[0] = 1.0; //R
                            ecolor[1] = 0.0; //G
                            ecolor[2] = 0.0; //B
                        } else {
                            //on my team
                            ecolor[0] = 0.0; //R
                            ecolor[1] = 1.0; //G
                            ecolor[2] = 0.0; //B
                        }
                    } else if es.owner == snapClientNum as c_int
                        || (gametype >= GT_TEAM && es.teamowner == myClientTeam)
                    {
                        ecolor[0] = 0.0; //R
                        ecolor[1] = 1.0; //G
                        ecolor[2] = 0.0; //B
                    } else if es.teamowner == 16
                        || (gametype >= GT_TEAM
                            && es.teamowner != 0
                            && es.teamowner != myClientTeam)
                    {
                        ecolor[0] = 1.0; //R
                        ecolor[1] = 0.0; //G
                        ecolor[2] = 0.0; //B
                    }
                } else if es.eType == entityType_t::ET_MOVER as c_int
                    && es.bolt1 != 0
                    && ctx.world.cg.predictedPlayerState.weapon == WP_SABER
                {
                    //can push/pull this mover. Only show it if we're using the saber.
                    ecolor[0] = 0.2;
                    ecolor[1] = 0.5;
                    ecolor[2] = 1.0;

                    corona = true;
                } else if es.eType == entityType_t::ET_MOVER as c_int && es.teamowner != 0 {
                    //a team owns this - if it's my team green, if not red, if not teamplay then yellow
                    if gametype < GT_TEAM {
                        ecolor[0] = 1.0; //R
                        ecolor[1] = 1.0; //G
                        ecolor[2] = 0.0; //B
                    } else if myPersTeam != es.teamowner {
                        //not my team
                        ecolor[0] = 1.0; //R
                        ecolor[1] = 0.0; //G
                        ecolor[2] = 0.0; //B
                    } else {
                        //my team
                        ecolor[0] = 0.0; //R
                        ecolor[1] = 1.0; //G
                        ecolor[2] = 0.0; //B
                    }
                } else if es.health != 0 {
                    if es.teamowner == 0 || gametype < GT_TEAM {
                        //not owned by a team or teamplay
                        ecolor[0] = 1.0;
                        ecolor[1] = 1.0;
                        ecolor[2] = 0.0;
                    } else if es.teamowner == myPersTeam {
                        //owned by my team
                        ecolor[0] = 0.0;
                        ecolor[1] = 1.0;
                        ecolor[2] = 0.0;
                    } else {
                        //hostile
                        ecolor[0] = 1.0;
                        ecolor[1] = 0.0;
                        ecolor[2] = 0.0;
                    }
                }

                ecolor[3] = 1.0;

                trap::R_SetColor(ctx.engine, Some(&ecolor));
            }
        }
    }

    let mut w;
    let mut h;
    if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        //I'm in a vehicle — the custom crosshair handle is deferred (fn doc),
        //so `hShader` stays 0 here.
        //bigger by default
        w = ctx.world.cvars.cg_crosshairSize.value * 2.0;
        h = w;
    } else {
        w = ctx.world.cvars.cg_crosshairSize.value;
        h = w;
    }

    // pulse the size of the crosshair when picking up items
    let f = (ctx.world.cg.time - ctx.world.cg.itemPickupBlendTime) as f32;
    if f > 0.0 && f < ITEM_BLOB_TIME {
        let f = f / ITEM_BLOB_TIME;
        w *= 1.0 + f;
        h *= 1.0 + f;
    }

    let (x, y) = match worldPoint {
        Some(wp) if VectorLength(wp) != 0.0 => {
            //CG_LerpCrosshairPos( &x, &y ); — commented out at the site
            match CG_WorldCoordToScreenCoordFloat(ctx.world, wp) {
                Some((sx, sy)) => (sx - 320.0, sy - 240.0),
                //off screen, don't draw it
                None => return,
            }
        }
        _ => (
            ctx.world.cvars.cg_crosshairX.integer as f32,
            ctx.world.cvars.cg_crosshairY.integer as f32,
        ),
    };

    if hShader == 0 {
        // §F19: a negative `cg_drawCrosshair` gives C a negative `%` and Raven
        // reads a garbage handle before the array (drawing nothing useful);
        // the port draws nothing for negatives - a cvar the player can set,
        // so a panic here would be a user-typed crash.
        let pick = ctx.world.cvars.cg_drawCrosshair.integer % NUM_CROSSHAIRS as c_int;
        if pick < 0 {
            return;
        }
        hShader = ctx.world.cgs.media.crosshairShader[pick as usize];
    }

    let chX = ((x + ctx.world.cg.refdef.x as f32) as f64 + 0.5 * (640.0f32 - w) as f64) as f32;
    let mut chY = ((y + ctx.world.cg.refdef.y as f32) as f64 + 0.5 * (480.0f32 - h) as f64) as f32;
    trap::R_DrawStretchPic(ctx.engine, chX, chY, w, h, 0.0, 0.0, 1.0, 1.0, hShader);

    //draw a health bar directly under the crosshair if we're looking at something
    //that takes damage
    if let Some(centNum) = crossEnt {
        let es = ctx.world.entity(centNum).currentState;
        if es.maxhealth != 0 {
            CG_DrawHealthBar(ctx, centNum, chX, chY, w, h);
            chY += HEALTH_HEIGHT * 2.0;
        } else if es.number < MAX_CLIENTS_I32 {
            if ctx.world.cgs.gametype == GT_SIEGE {
                CG_DrawSiegeInfo(ctx, centNum, chX, chY, w, h);
                chY += HEALTH_HEIGHT * 4.0;
            }
            if ctx.world.cg.crosshairVehNum != 0
                && ctx.world.cg.time == ctx.world.cg.crosshairVehTime
            {
                //it was in the crosshair this frame
                let vehNum = ctx.world.cg.crosshairVehNum as usize;
                let hisVeh = ctx.world.entity(vehNum);
                let drawIt = hisVeh.currentState.eType == entityType_t::ET_NPC as c_int
                    && hisVeh.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && hisVeh.currentState.maxhealth != 0
                    && hisVeh.m_pVehicle.is_some();

                if drawIt {
                    //draw the health for this vehicle
                    CG_DrawHealthBar(ctx, vehNum, chX, chY, w, h);
                    chY += HEALTH_HEIGHT * 2.0;
                }
            }
        }
    }

    if ctx.world.cg.predictedPlayerState.hackingTime != 0 {
        //hacking something
        CG_DrawHaqrBar(ctx, chX, chY, w, h);
    }

    if ctx.world.draw.cg_genericTimerBar > ctx.world.cg.time {
        //draw generic timing bar, can be used for whatever
        CG_DrawGenericTimerBar(ctx);
    }

    if corona {
        // drawing extra bits
        ecolor[3] = 0.5;
        // don't draw full color
        let v = ((1.0 - ecolor[3]) as f64
            * (((ctx.world.cg.time as f32 * 0.001) as f64).sin() * 0.08 + 0.35))
            as f32;
        ecolor[0] = v;
        ecolor[1] = v;
        ecolor[2] = v;
        ecolor[3] = 1.0;

        trap::R_SetColor(ctx.engine, Some(&ecolor));

        w *= 2.0;
        h *= 2.0;

        let forceCoronaShader = ctx.world.cgs.media.forceCoronaShader;
        trap::R_DrawStretchPic(
            ctx.engine,
            ((x + ctx.world.cg.refdef.x as f32) as f64 + 0.5 * (640.0f32 - w) as f64) as f32,
            ((y + ctx.world.cg.refdef.y as f32) as f64 + 0.5 * (480.0f32 - h) as f64) as f32,
            w,
            h,
            0.0,
            0.0,
            1.0,
            1.0,
            forceCoronaShader,
        );
    }
}

/// Raven `CG_SaberClashFlare` — the 150ms white flare where two sabers just
/// met, skipped when the clash is behind you or out of line of sight.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5327-5386`
pub fn CG_SaberClashFlare(ctx: &mut CgContext) {
    let maxTime: c_int = 150;

    let t = ctx.world.cg.time - ctx.world.draw.cg_saberFlashTime;

    if t <= 0 || t >= maxTime {
        return;
    }

    // Don't do clashes for things that are behind us
    let flashPos = ctx.world.draw.cg_saberFlashPos;
    let mut dif: vec3_t = [0.0; 3];
    _VectorSubtract(flashPos, ctx.world.cg.refdef.vieworg, &mut dif);

    if _DotProduct(dif, ctx.world.cg.refdef.viewaxis[0]) < 0.2 {
        return;
    }

    // Raven's NULL mins/maxs are the collision model's own `vec3_origin`
    // ("allow NULL to be passed in for 0,0,0", `qcommon/cm_trace.cpp:1602`).
    let mut tr = trace_t::zeroed();
    let vieworg = ctx.world.cg.refdef.vieworg;
    CG_Trace(
        ctx,
        &mut tr,
        &vieworg,
        &vec3_origin,
        &vec3_origin,
        &flashPos,
        -1,
        CONTENTS_SOLID,
    );

    if tr.fraction < 1.0 {
        return;
    }

    let len = VectorNormalize(&mut dif);

    // clamp to a known range — Raven's 800-unit clamp is commented out here
    if len > 1200.0 {
        return;
    }

    let mut v = (1.0 - (t as f32 / maxTime as f32)) * ((1.0 - (len / 800.0)) * 2.0 + 0.35);
    if v < 0.001 {
        v = 0.001;
    }

    // §F19: Raven ignores this `qboolean` and paints at the uninitialized `x`/`y`
    // when the clash is off screen; the port draws nothing there.
    let Some((x, y)) = CG_WorldCoordToScreenCoord(ctx.world, flashPos) else {
        return;
    };

    // §F19: Raven hands `trap_R_SetColor` a `vec3_t`, so the alpha it reads is
    // whatever follows the array on the stack; the port sends a 1.0 alpha.
    let color: vec4_t = [0.8, 0.8, 0.8, 1.0];
    trap::R_SetColor(ctx.engine, Some(&color));

    let flare = trap::R_RegisterShader(ctx.engine, "gfx/effects/saberFlare");
    CG_DrawPic(
        ctx,
        x as f32 - (v * 300.0),
        y as f32 - (v * 300.0),
        v * 600.0,
        v * 600.0,
        flare,
    );
}

/// Raven `CG_BracketEntity` — the four corner brackets a fighter's HUD paints
/// around a distant ship, green for a friend and red for a foe.
///
/// DEFERRED: `Vehicle_t` referent pool — the lead indicator needs my ship's
/// `m_pVehicle->m_pVehicleInfo->weapon[0].ID` to look the projectile speed up
/// in `g_vehWeaponInfo`, and DEC-46.2's presence-only `Option<VehicleId>`
/// cannot name that weapon; see [`CG_DrawVehicleShields`]. The arm Raven takes
/// for a ship with no valid primary weapon draws nothing, which is what happens
/// here — everything before that read is pure, so nothing is transcribed for it.
/// Source: `oracle/codemp/cgame/cg_draw.c:5545-5599`
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5413-5600`
pub fn CG_BracketEntity(ctx: &mut CgContext, centNum: usize, radius: f32) {
    let mut isEnemy = false;

    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
    let es = ctx.world.entity(centNum).currentState;

    let mut dif: vec3_t = [0.0; 3];
    _VectorSubtract(lerpOrigin, ctx.world.cg.refdef.vieworg, &mut dif);
    let len = VectorNormalize(&mut dif);

    let rocketLockIndex = ctx.world.cg.snap_ref().map(|snap| snap.ps.rocketLockIndex);
    if ctx.world.cg.crosshairClientNum != es.clientNum && rocketLockIndex != Some(es.clientNum) {
        //if they're the entity you're locking onto or under your crosshair, always draw bracket
        //Hmm... for now, if they're closer than 2000, don't bracket?
        if len < 2000.0 {
            return;
        }

        let mut tr = trace_t::zeroed();
        let vieworg = ctx.world.cg.refdef.vieworg;
        CG_Trace(
            ctx,
            &mut tr,
            &vieworg,
            &vec3_origin,
            &vec3_origin,
            &lerpOrigin,
            -1,
            CONTENTS_OPAQUE,
        );

        //don't bracket if can't see them
        if tr.fraction < 1.0 {
            return;
        }
    }

    let Some((mut x, mut y)) = CG_WorldCoordToScreenCoordFloat(ctx.world, lerpOrigin) else {
        //off-screen, don't draw it
        return;
    };

    //just to see if it's centered — the debug `CG_DrawPic` is commented out

    // §F19: Raven derefs `cg.snap` for the local client's team; with no
    // snapshot there is nobody to compare against.
    let snapClientNum = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.ps.clientNum as usize,
        None => return,
    };
    let localTeam = ctx.world.cgs.clientinfo[snapClientNum].team;

    if es.m_iVehicleNum != 0
        && ctx.world.cgs.clientinfo[(es.m_iVehicleNum - 1) as usize].infoValid != qfalse
    {
        //vehicle has a driver
        if ctx.world.cgs.gametype < GT_TEAM {
            //ffa?
            isEnemy = true;
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
        } else if ctx.world.cgs.clientinfo[(es.m_iVehicleNum - 1) as usize].team == localTeam {
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_GREEN_INDEX]));
        } else {
            isEnemy = true;
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
        }
    } else if es.teamowner != 0 {
        if ctx.world.cgs.gametype < GT_TEAM {
            //ffa?
            isEnemy = true;
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
        } else if es.teamowner != ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] {
            // on enemy team
            isEnemy = true;
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
        } else {
            //a friend
            trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_GREEN_INDEX]));
        }
    } else {
        //FIXME: if we want to ever bracket anything besides vehicles (like siege objectives we want to blow up), we should handle the coloring here
        trap::R_SetColor(ctx.engine, None);
    }

    let mut size = if len <= 1.0 {
        //super-close, max out at 400 times radius (which is HUGE)
        radius * 400.0
    } else {
        //scale by dist
        radius * (400.0 / len)
    };

    if size < 1.0 {
        size = 1.0;
    }

    //length scales with dist
    let mut lineLength = size * 0.1;
    if lineLength < 0.5 {
        //always visible
        lineLength = 0.5;
    }
    //always visible width
    let lineWidth: f32 = 1.0;

    x -= size * 0.5;
    y -= size * 0.5;

    // Raven's on-screen guard around this block is commented out, so the
    // brackets always draw.
    let whiteShader = ctx.world.cgs.media.whiteShader;
    //upper left corner
    //horz
    CG_DrawPic(ctx, x, y, lineLength, lineWidth, whiteShader);
    //vert
    CG_DrawPic(ctx, x, y, lineWidth, lineLength, whiteShader);
    //upper right corner
    //horz
    CG_DrawPic(
        ctx,
        x + size - lineLength,
        y,
        lineLength,
        lineWidth,
        whiteShader,
    );
    //vert
    CG_DrawPic(
        ctx,
        x + size - lineWidth,
        y,
        lineWidth,
        lineLength,
        whiteShader,
    );
    //lower left corner
    //horz
    CG_DrawPic(
        ctx,
        x,
        y + size - lineWidth,
        lineLength,
        lineWidth,
        whiteShader,
    );
    //vert
    CG_DrawPic(
        ctx,
        x,
        y + size - lineLength,
        lineWidth,
        lineLength,
        whiteShader,
    );
    //lower right corner
    //horz
    CG_DrawPic(
        ctx,
        x + size - lineLength,
        y + size - lineWidth,
        lineLength,
        lineWidth,
        whiteShader,
    );
    //vert
    CG_DrawPic(
        ctx,
        x + size - lineWidth,
        y + size - lineLength,
        lineWidth,
        lineLength,
        whiteShader,
    );

    //Lead Indicator...
    // The `cg_drawVehLeadIndicator` / `isEnemy` / `CLASS_VEHICLE` /
    // `VectorCompare` chain exists only to reach the deferred vehicle-weapon
    // read (see the fn doc); every test in it is pure, so nothing is
    // transcribed for it.
    let _ = isEnemy;
}

/// Raven `CG_DrawSiegeTimer` — the round clock on the siege HUD.
///
/// Raven: this function is pretty much totally placeholder
/// ("rwwFIXMEFIXME: Make someone make assets and use them.").
/// Source: `oracle/codemp/cgame/cg_draw.c:7356-7422`
pub fn CG_DrawSiegeTimer(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    ds: &DisplayState,
    timeRemaining: c_int,
    isMyTeam: bool,
) {
    let menuHUD = Menus_FindByName(menus, "mp_timer");
    if menuHUD.is_none() {
        return;
    }

    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "frame") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let mut minutes: c_int = 0;
    let mut seconds: c_int = timeRemaining;

    while seconds >= 60 {
        minutes += 1;
        seconds -= 60;
    }

    let timeStr = format!("{minutes}:{seconds:02}");

    let fColor = if isMyTeam {
        ct_table_t::CT_HUD_RED
    } else {
        ct_table_t::CT_HUD_GREEN
    };

    // Raven's `trap_Cvar_Set("ui_siegeTimer", …)` and the fixed-position
    // `UI_DrawProportionalString` above this are commented out at the site.

    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "timer") {
        let rect = menus.item(item).window.rect;
        UI_DrawProportionalString(
            ctx,
            ds,
            rect.x as c_int,
            rect.y as c_int,
            &timeStr,
            UI_SMALLFONT | UI_DROPSHADOW,
            colorTable[fColor as usize],
        );
    }
}

/// Raven `CG_DrawSiegeDeathTimer` — the respawn-wave countdown, drawn in the
/// same `mp_timer` frame as [`CG_DrawSiegeTimer`].
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7424-7479`
pub fn CG_DrawSiegeDeathTimer(
    ctx: &mut CgContext,
    menus: &MenuSystem,
    ds: &DisplayState,
    timeRemaining: c_int,
) {
    let menuHUD = Menus_FindByName(menus, "mp_timer");
    if menuHUD.is_none() {
        return;
    }

    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "frame") {
        let foreColor = menus.item(item).window.foreColor;
        let rect = menus.item(item).window.rect;
        let background = menus.item(item).window.background;

        trap::R_SetColor(ctx.engine, Some(&foreColor));
        CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, background);
    }

    let mut minutes: c_int = 0;
    let mut seconds: c_int = timeRemaining;

    while seconds >= 60 {
        minutes += 1;
        seconds -= 60;
    }

    let timeStr = if seconds < 10 {
        format!("{minutes}:0{seconds}")
    } else {
        format!("{minutes}:{seconds}")
    };

    if let Some(item) = Menu_FindItemByName(menus, menuHUD, "deathtimer") {
        let rect = menus.item(item).window.rect;
        let foreColor = menus.item(item).window.foreColor;
        UI_DrawProportionalString(
            ctx,
            ds,
            rect.x as c_int,
            rect.y as c_int,
            &timeStr,
            UI_SMALLFONT | UI_DROPSHADOW,
            foreColor,
        );
    }
}

/// Raven `CG_Draw2DScreenTints` — the full-screen washes: rage, rage recovery,
/// absorb, protect and ysalamiri each ramp up while their power is on and fade
/// out after, and lava/slime/water tint on top of all of it.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:7701-8116`
pub fn CG_Draw2DScreenTints(ctx: &mut CgContext) {
    let mut hcolor: vec4_t = [0.0; 4];

    // §F19: Raven derefs `cg.snap` unguarded; with no snapshot no power tint
    // applies, but the view-contents wash below still does.
    let snapState = ctx.world.cg.snap_ref().map(|snap| {
        (
            snap.ps.clientNum as usize,
            snap.ps.fd.forcePowersActive,
            snap.ps.fd.forceRageRecoveryTime,
            snap.ps.rocketLockIndex,
            snap.ps.rocketLockTime,
        )
    });

    if let Some((
        snapClientNum,
        forcePowersActive,
        forceRageRecoveryTime,
        rocketLockIndex,
        rocketLockTime,
    )) = snapState
    {
        if ctx.world.cgs.clientinfo[snapClientNum].team != TEAM_SPECTATOR {
            if forcePowersActive & (1 << FP_RAGE) != 0 {
                if ctx.world.draw.cgRageTime == 0 {
                    ctx.world.draw.cgRageTime = ctx.world.cg.time;
                }

                let mut rageTime = (ctx.world.cg.time - ctx.world.draw.cgRageTime) as f32;

                rageTime /= 9000.0;

                if rageTime < 0.0 {
                    rageTime = 0.0;
                }
                if rageTime > 0.15 {
                    rageTime = 0.15;
                }

                hcolor[3] = rageTime;
                hcolor[0] = 0.7;
                hcolor[1] = 0.0;
                hcolor[2] = 0.0;

                if ctx.world.cg.renderingThirdPerson == qfalse {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                }

                ctx.world.draw.cgRageFadeTime = 0;
                ctx.world.draw.cgRageFadeVal = 0.0;
            } else if ctx.world.draw.cgRageTime != 0 {
                if ctx.world.draw.cgRageFadeTime == 0 {
                    ctx.world.draw.cgRageFadeTime = ctx.world.cg.time;
                    ctx.world.draw.cgRageFadeVal = 0.15;
                }

                let mut rageTime = ctx.world.draw.cgRageFadeVal;

                // C promotes the f32 lhs to double, subtracts, and narrows once
                ctx.world.draw.cgRageFadeVal = (ctx.world.draw.cgRageFadeVal as f64
                    - (ctx.world.cg.time - ctx.world.draw.cgRageFadeTime) as f64 * 0.000005)
                    as f32;

                if rageTime < 0.0 {
                    rageTime = 0.0;
                }
                if rageTime > 0.15 {
                    rageTime = 0.15;
                }

                if forceRageRecoveryTime > ctx.world.cg.time {
                    let mut checkRageRecTime = rageTime;

                    if checkRageRecTime < 0.15 {
                        checkRageRecTime = 0.15;
                    }

                    hcolor[3] = checkRageRecTime;
                    hcolor[0] = rageTime * 4.0;
                    if hcolor[0] < 0.2 {
                        hcolor[0] = 0.2;
                    }
                    hcolor[1] = 0.2;
                    hcolor[2] = 0.2;
                } else {
                    hcolor[3] = rageTime;
                    hcolor[0] = 0.7;
                    hcolor[1] = 0.0;
                    hcolor[2] = 0.0;
                }

                if ctx.world.cg.renderingThirdPerson == qfalse && rageTime != 0.0 {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                } else {
                    if forceRageRecoveryTime > ctx.world.cg.time {
                        hcolor[3] = 0.15;
                        hcolor[0] = 0.2;
                        hcolor[1] = 0.2;
                        hcolor[2] = 0.2;
                        CG_DrawRect(
                            ctx,
                            0.0,
                            0.0,
                            SCREEN_WIDTH as f32,
                            SCREEN_HEIGHT as f32,
                            (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                            &hcolor,
                        );
                    }
                    ctx.world.draw.cgRageTime = 0;
                }
            } else if forceRageRecoveryTime > ctx.world.cg.time {
                if ctx.world.draw.cgRageRecTime == 0 {
                    ctx.world.draw.cgRageRecTime = ctx.world.cg.time;
                }

                let mut rageRecTime = (ctx.world.cg.time - ctx.world.draw.cgRageRecTime) as f32;

                rageRecTime /= 9000.0;

                if rageRecTime < 0.15 {
                    //0
                    rageRecTime = 0.15; //0
                }
                if rageRecTime > 0.15 {
                    rageRecTime = 0.15;
                }

                hcolor[3] = rageRecTime;
                hcolor[0] = 0.2;
                hcolor[1] = 0.2;
                hcolor[2] = 0.2;

                if ctx.world.cg.renderingThirdPerson == qfalse {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                }

                ctx.world.draw.cgRageRecFadeTime = 0;
                ctx.world.draw.cgRageRecFadeVal = 0.0;
            } else if ctx.world.draw.cgRageRecTime != 0 {
                if ctx.world.draw.cgRageRecFadeTime == 0 {
                    ctx.world.draw.cgRageRecFadeTime = ctx.world.cg.time;
                    ctx.world.draw.cgRageRecFadeVal = 0.15;
                }

                let mut rageRecTime = ctx.world.draw.cgRageRecFadeVal;

                // C promotes the f32 lhs to double, subtracts, and narrows once
                ctx.world.draw.cgRageRecFadeVal = (ctx.world.draw.cgRageRecFadeVal as f64
                    - (ctx.world.cg.time - ctx.world.draw.cgRageRecFadeTime) as f64 * 0.000005)
                    as f32;

                if rageRecTime < 0.0 {
                    rageRecTime = 0.0;
                }
                if rageRecTime > 0.15 {
                    rageRecTime = 0.15;
                }

                hcolor[3] = rageRecTime;
                hcolor[0] = 0.2;
                hcolor[1] = 0.2;
                hcolor[2] = 0.2;

                if ctx.world.cg.renderingThirdPerson == qfalse && rageRecTime != 0.0 {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                } else {
                    ctx.world.draw.cgRageRecTime = 0;
                }
            }

            if forcePowersActive & (1 << FP_ABSORB) != 0 {
                if ctx.world.draw.cgAbsorbTime == 0 {
                    ctx.world.draw.cgAbsorbTime = ctx.world.cg.time;
                }

                let mut absorbTime = (ctx.world.cg.time - ctx.world.draw.cgAbsorbTime) as f32;

                absorbTime /= 9000.0;

                if absorbTime < 0.0 {
                    absorbTime = 0.0;
                }
                if absorbTime > 0.15 {
                    absorbTime = 0.15;
                }

                hcolor[3] = absorbTime / 2.0;
                hcolor[0] = 0.0;
                hcolor[1] = 0.0;
                hcolor[2] = 0.7;

                if ctx.world.cg.renderingThirdPerson == qfalse {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                }

                ctx.world.draw.cgAbsorbFadeTime = 0;
                ctx.world.draw.cgAbsorbFadeVal = 0.0;
            } else if ctx.world.draw.cgAbsorbTime != 0 {
                if ctx.world.draw.cgAbsorbFadeTime == 0 {
                    ctx.world.draw.cgAbsorbFadeTime = ctx.world.cg.time;
                    ctx.world.draw.cgAbsorbFadeVal = 0.15;
                }

                let mut absorbTime = ctx.world.draw.cgAbsorbFadeVal;

                // C promotes the f32 lhs to double, subtracts, and narrows once
                ctx.world.draw.cgAbsorbFadeVal = (ctx.world.draw.cgAbsorbFadeVal as f64
                    - (ctx.world.cg.time - ctx.world.draw.cgAbsorbFadeTime) as f64 * 0.000005)
                    as f32;

                if absorbTime < 0.0 {
                    absorbTime = 0.0;
                }
                if absorbTime > 0.15 {
                    absorbTime = 0.15;
                }

                hcolor[3] = absorbTime / 2.0;
                hcolor[0] = 0.0;
                hcolor[1] = 0.0;
                hcolor[2] = 0.7;

                if ctx.world.cg.renderingThirdPerson == qfalse && absorbTime != 0.0 {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                } else {
                    ctx.world.draw.cgAbsorbTime = 0;
                }
            }

            if forcePowersActive & (1 << FP_PROTECT) != 0 {
                if ctx.world.draw.cgProtectTime == 0 {
                    ctx.world.draw.cgProtectTime = ctx.world.cg.time;
                }

                let mut protectTime = (ctx.world.cg.time - ctx.world.draw.cgProtectTime) as f32;

                protectTime /= 9000.0;

                if protectTime < 0.0 {
                    protectTime = 0.0;
                }
                if protectTime > 0.15 {
                    protectTime = 0.15;
                }

                hcolor[3] = protectTime / 2.0;
                hcolor[0] = 0.0;
                hcolor[1] = 0.7;
                hcolor[2] = 0.0;

                if ctx.world.cg.renderingThirdPerson == qfalse {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                }

                ctx.world.draw.cgProtectFadeTime = 0;
                ctx.world.draw.cgProtectFadeVal = 0.0;
            } else if ctx.world.draw.cgProtectTime != 0 {
                if ctx.world.draw.cgProtectFadeTime == 0 {
                    ctx.world.draw.cgProtectFadeTime = ctx.world.cg.time;
                    ctx.world.draw.cgProtectFadeVal = 0.15;
                }

                let mut protectTime = ctx.world.draw.cgProtectFadeVal;

                // C promotes the f32 lhs to double, subtracts, and narrows once
                ctx.world.draw.cgProtectFadeVal = (ctx.world.draw.cgProtectFadeVal as f64
                    - (ctx.world.cg.time - ctx.world.draw.cgProtectFadeTime) as f64 * 0.000005)
                    as f32;

                if protectTime < 0.0 {
                    protectTime = 0.0;
                }
                if protectTime > 0.15 {
                    protectTime = 0.15;
                }

                hcolor[3] = protectTime / 2.0;
                hcolor[0] = 0.0;
                hcolor[1] = 0.7;
                hcolor[2] = 0.0;

                if ctx.world.cg.renderingThirdPerson == qfalse && protectTime != 0.0 {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                } else {
                    ctx.world.draw.cgProtectTime = 0;
                }
            }

            if rocketLockIndex != ENTITYNUM_NONE
                && (ctx.world.cg.time as f32 - rocketLockTime) > 0.0
            {
                CG_DrawRocketLocking(ctx, rocketLockIndex as usize, rocketLockTime as c_int);
            }

            let gametype = ctx.world.cgs.gametype;
            let hasYsalamiri = ctx
                .world
                .cg
                .snap_mut()
                .is_some_and(|snap| BG_HasYsalamiri(gametype, &mut snap.ps) != qfalse);

            if hasYsalamiri {
                if ctx.world.draw.cgYsalTime == 0 {
                    ctx.world.draw.cgYsalTime = ctx.world.cg.time;
                }

                let mut ysalTime = (ctx.world.cg.time - ctx.world.draw.cgYsalTime) as f32;

                ysalTime /= 9000.0;

                if ysalTime < 0.0 {
                    ysalTime = 0.0;
                }
                if ysalTime > 0.15 {
                    ysalTime = 0.15;
                }

                hcolor[3] = ysalTime / 2.0;
                hcolor[0] = 0.7;
                hcolor[1] = 0.7;
                hcolor[2] = 0.0;

                if ctx.world.cg.renderingThirdPerson == qfalse {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                }

                ctx.world.draw.cgYsalFadeTime = 0;
                ctx.world.draw.cgYsalFadeVal = 0.0;
            } else if ctx.world.draw.cgYsalTime != 0 {
                if ctx.world.draw.cgYsalFadeTime == 0 {
                    ctx.world.draw.cgYsalFadeTime = ctx.world.cg.time;
                    ctx.world.draw.cgYsalFadeVal = 0.15;
                }

                let mut ysalTime = ctx.world.draw.cgYsalFadeVal;

                // C promotes the f32 lhs to double, subtracts, and narrows once
                ctx.world.draw.cgYsalFadeVal = (ctx.world.draw.cgYsalFadeVal as f64
                    - (ctx.world.cg.time - ctx.world.draw.cgYsalFadeTime) as f64 * 0.000005)
                    as f32;

                if ysalTime < 0.0 {
                    ysalTime = 0.0;
                }
                if ysalTime > 0.15 {
                    ysalTime = 0.15;
                }

                hcolor[3] = ysalTime / 2.0;
                hcolor[0] = 0.7;
                hcolor[1] = 0.7;
                hcolor[2] = 0.0;

                if ctx.world.cg.renderingThirdPerson == qfalse && ysalTime != 0.0 {
                    CG_DrawRect(
                        ctx,
                        0.0,
                        0.0,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
                        &hcolor,
                    );
                } else {
                    ctx.world.draw.cgYsalTime = 0;
                }
            }
        }
    }

    if ctx.world.cg.refdef.viewContents & CONTENTS_LAVA != 0 {
        //tint screen red
        let phase = (ctx.world.cg.time as f64 / 1000.0 * WAVE_FREQUENCY * PI * 2.0) as f32;
        hcolor[3] = (0.5 + (0.15 * (phase as f64).sin())) as f32;
        hcolor[0] = 0.7;
        hcolor[1] = 0.0;
        hcolor[2] = 0.0;

        CG_DrawRect(
            ctx,
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
            &hcolor,
        );
    } else if ctx.world.cg.refdef.viewContents & CONTENTS_SLIME != 0 {
        //tint screen green
        let phase = (ctx.world.cg.time as f64 / 1000.0 * WAVE_FREQUENCY * PI * 2.0) as f32;
        hcolor[3] = (0.4 + (0.1 * (phase as f64).sin())) as f32;
        hcolor[0] = 0.0;
        hcolor[1] = 0.7;
        hcolor[2] = 0.0;

        CG_DrawRect(
            ctx,
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
            &hcolor,
        );
    } else if ctx.world.cg.refdef.viewContents & CONTENTS_WATER != 0 {
        //tint screen light blue -- FIXME: don't do this if CONTENTS_FOG? (in case someone *does* make a water shader with fog in it?)
        let phase = (ctx.world.cg.time as f64 / 1000.0 * WAVE_FREQUENCY * PI * 2.0) as f32;
        hcolor[3] = (0.3 + (0.05 * (phase as f64).sin())) as f32;
        hcolor[0] = 0.0;
        hcolor[1] = 0.2;
        hcolor[2] = 0.8;

        CG_DrawRect(
            ctx,
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
            &hcolor,
        );
    }
}

/// Raven `CG_DrawHUD` — the whole left/right HUD frame: team tint, scanline and
/// frame pics, armor/health, the score readout, force power, and ammo or saber
/// style. The `cg_hudFiles` debug path draws the plain numeric HUD instead.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:1169-1391`
pub fn CG_DrawHUD(ctx: &mut CgContext, menus: &MenuSystem, ds: &DisplayState, centNum: usize) {
    if ctx.world.cvars.cg_hudFiles.integer != 0 {
        let x: c_int = 0;
        let y: c_int = SCREEN_HEIGHT - 80;
        let mut weapX: c_int = x;

        // §F19: Raven derefs `cg.snap` unguarded here; no snapshot = no stats
        // to paint.
        let (health, armor, snapWeapon, saberLevel, forcePower, ammo) =
            match ctx.world.cg.snap_ref() {
                Some(snap) => (
                    snap.ps.stats[STAT_HEALTH as usize],
                    snap.ps.stats[STAT_ARMOR as usize],
                    snap.ps.weapon,
                    snap.ps.fd.saberDrawAnimLevel,
                    snap.ps.fd.forcePower,
                    snap.ps.ammo,
                ),
                None => return,
            };

        UI_DrawProportionalString(
            ctx,
            ds,
            x + 16,
            y + 40,
            &format!("{health}"),
            UI_SMALLFONT | UI_DROPSHADOW,
            colorTable[ct_table_t::CT_HUD_RED as usize],
        );

        UI_DrawProportionalString(
            ctx,
            ds,
            x + 18 + 14,
            y + 40 + 14,
            &format!("{armor}"),
            UI_SMALLFONT | UI_DROPSHADOW,
            colorTable[ct_table_t::CT_HUD_GREEN as usize],
        );

        let ammoString: String;
        if snapWeapon == WP_SABER {
            if saberLevel == saber_styles_t::SS_DUAL as c_int {
                ammoString = "AKIMBO".to_string();
                weapX += 16;
            } else if saberLevel == saber_styles_t::SS_STAFF as c_int {
                ammoString = "STAFF".to_string();
                weapX += 16;
            } else if saberLevel == FORCE_LEVEL_3 {
                ammoString = "STRONG".to_string();
                weapX += 16;
            } else if saberLevel == FORCE_LEVEL_2 {
                ammoString = "MEDIUM".to_string();
                weapX += 16;
            } else {
                ammoString = "FAST".to_string();
            }
        } else {
            // §F19: `currentState.weapon` and its `ammoIndex` are server-supplied;
            // a bad index reads garbage in Raven, so the port reads 0 rather than
            // panic.
            let weapon = ctx.world.entity(centNum).currentState.weapon;
            let ammoCount = weaponData
                .get(usize::try_from(weapon).unwrap_or(usize::MAX))
                .and_then(|wd| ammo.get(usize::try_from(wd.ammoIndex).unwrap_or(usize::MAX)))
                .copied()
                .unwrap_or(0);
            ammoString = format!("{ammoCount}");
        }

        UI_DrawProportionalString(
            ctx,
            ds,
            SCREEN_WIDTH - (weapX + 16 + 32),
            y + 40,
            &ammoString,
            UI_SMALLFONT | UI_DROPSHADOW,
            colorTable[ct_table_t::CT_HUD_ORANGE as usize],
        );

        UI_DrawProportionalString(
            ctx,
            ds,
            SCREEN_WIDTH - (x + 18 + 14 + 32),
            y + 40 + 14,
            &format!("{forcePower}"),
            UI_SMALLFONT | UI_DROPSHADOW,
            colorTable[ct_table_t::CT_ICON_BLUE as usize],
        );

        return;
    }

    // §F19: Raven derefs `cg.snap` unguarded below (team tint + score); no
    // snapshot = no HUD stats.
    let (persTeam, persScore) = match ctx.world.cg.snap_ref() {
        Some(snap) => (
            snap.ps.persistant[PERS_TEAM as usize],
            snap.ps.persistant[PERS_SCORE as usize],
        ),
        None => return,
    };

    let gametype = ctx.world.cgs.gametype;
    if gametype >= GT_TEAM && gametype != GT_SIEGE {
        // tint the hud items based on team
        if persTeam == TEAM_RED {
            ctx.world.draw.hudTintColor = Some(redhudtint);
        } else if persTeam == TEAM_BLUE {
            ctx.world.draw.hudTintColor = Some(bluehudtint);
        } else {
            // If we're not on a team for whatever reason, leave things as they are.
            ctx.world.draw.hudTintColor = Some(colorTable[ct_table_t::CT_WHITE as usize]);
        }
    } else {
        // tint the hud items white (don't tint)
        ctx.world.draw.hudTintColor = Some(colorTable[ct_table_t::CT_WHITE as usize]);
    }
    let tint = ctx
        .world
        .draw
        .hudTintColor
        .unwrap_or(colorTable[ct_table_t::CT_WHITE as usize]);

    // Draw the left HUD
    let menuHUD = Menus_FindByName(menus, "lefthud");
    if menuHUD.is_some() {
        // Print scanline
        if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "scanline") {
            trap::R_SetColor(ctx.engine, Some(&tint));
            let rect = menus.item(focusItem).window.rect;
            let bg = menus.item(focusItem).window.background;
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, bg);
        }

        // Print frame
        if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "frame") {
            trap::R_SetColor(ctx.engine, Some(&tint));
            let rect = menus.item(focusItem).window.rect;
            let bg = menus.item(focusItem).window.background;
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, bg);
        }

        if ctx.world.cg.predictedPlayerState.pm_type != PM_SPECTATOR as c_int {
            CG_DrawArmor(ctx, menus, menuHUD);
            CG_DrawHealth(ctx, menus, menuHUD);
        }
    }
    // Raven's else-branch is a commented-out CG_Error; nothing to do.

    let scoreStr: String;
    if gametype == GT_DUEL {
        // A duel that requires more than one kill to knock the current enemy
        // back to the queue: show current kills out of how many needed.
        scoreStr = format!(
            "{}: {}/{}",
            CG_GetStringEdString(ctx, "MP_INGAME", "SCORE"),
            persScore,
            ctx.world.cgs.fraglimit
        );
    }
    // Raven's `else if (0 && cgs.gametype < GT_TEAM)` score-bias block is dead
    // (the `0 &&` short-circuits) and is dropped.
    else {
        // Don't draw a bias.
        scoreStr = format!(
            "{}: {}",
            CG_GetStringEdString(ctx, "MP_INGAME", "SCORE"),
            persScore
        );
    }

    let menuHUD = Menus_FindByName(menus, "righthud");
    if menuHUD.is_some() {
        if gametype != GT_POWERDUEL {
            if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "score_line") {
                let rect = menus.item(focusItem).window.rect;
                let foreColor = menus.item(focusItem).window.foreColor;
                UI_DrawScaledProportionalString(
                    ctx,
                    ds,
                    rect.x as c_int,
                    rect.y as c_int,
                    &scoreStr,
                    UI_RIGHT | UI_DROPSHADOW,
                    foreColor,
                    0.7,
                );
            }
        }

        // Print scanline
        if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "scanline") {
            trap::R_SetColor(ctx.engine, Some(&tint));
            let rect = menus.item(focusItem).window.rect;
            let bg = menus.item(focusItem).window.background;
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, bg);
        }

        if let Some(focusItem) = Menu_FindItemByName(menus, menuHUD, "frame") {
            trap::R_SetColor(ctx.engine, Some(&tint));
            let rect = menus.item(focusItem).window.rect;
            let bg = menus.item(focusItem).window.background;
            CG_DrawPic(ctx, rect.x, rect.y, rect.w, rect.h, bg);
        }

        CG_DrawForcePower(ctx, menus, menuHUD);

        // Draw ammo tics or saber style
        if ctx.world.entity(centNum).currentState.weapon == WP_SABER {
            CG_DrawSaberStyle(ctx, menus, centNum, menuHUD);
        } else {
            CG_DrawAmmo(ctx, menus, ds, centNum, menuHUD);
        }
    }
    // Raven's else-branch is a commented-out CG_Error; nothing to do.
}

/// Raven `CG_DrawEnemyInfo` — the top-right enemy/leader panel: their model
/// icon, name, the mode title, their score, and (in duel) a health bar.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2866-3040`
pub fn CG_DrawEnemyInfo(ctx: &mut CgContext, ds: &DisplayState, mut y: f32) -> f32 {
    let xOffset: c_int = 0;

    // §F19: Raven's `if (!cg.snap) return y;` — the port pulls the snapshot
    // scalars it needs here and takes the same early-out with no snapshot.
    let (duelInProgress, duelIndex, snapClientNum) = match ctx.world.cg.snap_ref() {
        Some(snap) => (snap.ps.duelInProgress, snap.ps.duelIndex, snap.ps.clientNum),
        None => return y,
    };

    if ctx.world.cvars.cg_drawEnemyInfo.integer == 0 {
        return y;
    }

    if ctx.world.cg.predictedPlayerState.stats[STAT_HEALTH as usize] <= 0 {
        return y;
    }

    if ctx.world.cgs.gametype == GT_POWERDUEL {
        // just get out of here then
        return y;
    }

    let title: String;
    let clientNum: c_int;
    if ctx.world.cgs.gametype == GT_JEDIMASTER {
        let jm = ctx.world.cgs.jediMaster;
        if jm < 0 {
            let t = CG_GetStringEdString(ctx, "MP_INGAME", "GET_SABER");
            let size = ICON_SIZE * 1.25;
            y += 5.0;
            let icon = ctx.world.cgs.media.weaponIcons[WP_SABER as usize];
            CG_DrawPic(
                ctx,
                640.0 - size - 12.0 + xOffset as f32,
                y,
                size,
                size,
                icon,
            );
            y += size;
            let w = CG_Text_Width(ctx, ds, &t, 0.7, FONT_MEDIUM);
            CG_Text_Paint(
                ctx,
                ds,
                630.0 - w as f32 + xOffset as f32,
                y,
                0.7,
                colorWhite,
                &t,
                0.0,
                0,
                0,
                FONT_MEDIUM,
            );
            return y + BIGCHAR_HEIGHT as f32 + 2.0;
        }
        title = CG_GetStringEdString(ctx, "MP_INGAME", "MASTERY7");
        clientNum = jm;
    } else if duelInProgress != qfalse {
        title = CG_GetStringEdString(ctx, "MP_INGAME", "DUELING");
        clientNum = duelIndex;
    } else if ctx.world.cgs.gametype == GT_DUEL
        && ctx.world.cgs.clientinfo[snapClientNum as usize].team != TEAM_SPECTATOR
    {
        title = CG_GetStringEdString(ctx, "MP_INGAME", "DUELING");
        if snapClientNum == ctx.world.cgs.duelist1 {
            clientNum = ctx.world.cgs.duelist2;
        } else if snapClientNum == ctx.world.cgs.duelist2 {
            clientNum = ctx.world.cgs.duelist1;
        } else if snapClientNum == ctx.world.cgs.duelist3 {
            clientNum = ctx.world.cgs.duelist1;
        } else {
            return y;
        }
    } else {
        // As of current, we don't want to draw the attacker. Instead, draw
        // whoever is in first place.
        if ctx.world.cgs.duelWinner < 0 || ctx.world.cgs.duelWinner >= MAX_CLIENTS_I32 {
            return y;
        }
        title = format!(
            "{}: {}",
            CG_GetStringEdString(ctx, "MP_INGAME", "LEADER"),
            ctx.world.cgs.scores1
        );
        clientNum = ctx.world.cgs.duelWinner;
    }

    // §F19: Raven's `!(&cgs.clientinfo[clientNum])` is always false (address of
    // an array element), so the port keeps only the `>= MAX_CLIENTS` half and
    // adds a negative guard so a bad `clientNum` skips instead of panicking.
    if !(0..MAX_CLIENTS_I32).contains(&clientNum) {
        return y;
    }

    let size = ICON_SIZE * 1.25;
    y += 5.0;

    let modelIcon = ctx.world.cgs.clientinfo[clientNum as usize].modelIcon;
    if modelIcon != 0 {
        CG_DrawPic(
            ctx,
            640.0 - size - 5.0 + xOffset as f32,
            y,
            size,
            size,
            modelIcon,
        );
    }

    y += size;

    let name = buf_to_string(
        &ctx.world.cgs.clientinfo[clientNum as usize]
            .name
            .map(|c| c as u8),
    );
    let w = CG_Text_Width(ctx, ds, &name, 1.0, FONT_SMALL2);
    CG_Text_Paint(
        ctx,
        ds,
        630.0 - w as f32 + xOffset as f32,
        y,
        1.0,
        colorWhite,
        &name,
        0.0,
        0,
        0,
        FONT_SMALL2,
    );

    y += 15.0;
    let w = CG_Text_Width(ctx, ds, &title, 1.0, FONT_SMALL2);
    CG_Text_Paint(
        ctx,
        ds,
        630.0 - w as f32 + xOffset as f32,
        y,
        1.0,
        colorWhite,
        &title,
        0.0,
        0,
        0,
        FONT_SMALL2,
    );

    if (ctx.world.cgs.gametype == GT_DUEL || ctx.world.cgs.gametype == GT_POWERDUEL)
        && ctx.world.cgs.clientinfo[snapClientNum as usize].team != TEAM_SPECTATOR
    {
        // also print their score
        y += 15.0;
        let text = format!(
            "{}/{}",
            ctx.world.cgs.clientinfo[clientNum as usize].score, ctx.world.cgs.fraglimit
        );
        let w = CG_Text_Width(ctx, ds, &text, 0.7, FONT_MEDIUM);
        CG_Text_Paint(
            ctx,
            ds,
            630.0 - w as f32 + xOffset as f32,
            y,
            0.7,
            colorWhite,
            &text,
            0.0,
            0,
            0,
            FONT_MEDIUM,
        );
    }

    // nmckenzie: DUEL_HEALTH - fixme - need checks and such here. And this is
    // coded to duelist 1 right now, which is wrongly.
    if ctx.world.cgs.showDuelHealths >= 2 {
        y += 15.0;
        if ctx.world.cgs.duelist1 == clientNum {
            CG_DrawDuelistHealth(ctx, 640.0 - size - 5.0 + xOffset as f32, y, 64.0, 8.0, 1);
        } else if ctx.world.cgs.duelist2 == clientNum {
            CG_DrawDuelistHealth(ctx, 640.0 - size - 5.0 + xOffset as f32, y, 64.0, 8.0, 2);
        }
    }

    y + BIGCHAR_HEIGHT as f32 + 2.0
}

/// Raven `CG_DrawSnapshot` — the timing debug line (server time, snapshot
/// number, command sequence) in the top-right.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3047-3063`
pub fn CG_DrawSnapshot(ctx: &mut CgContext, ds: &DisplayState, y: f32) -> f32 {
    let xOffset: c_int = 0;

    // §F19: Raven derefs `cg.snap->serverTime` unguarded; no snapshot = nothing
    // to time.
    let s = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return y;
        };
        format!(
            "time:{} snap:{} cmd:{}",
            snap.serverTime, ctx.world.cg.latestSnapshotNum, ctx.world.cgs.serverCommandSequence
        )
    };
    let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;

    CG_DrawBigString(ctx, ds, 635 - w + xOffset, (y + 2.0) as c_int, &s, 1.0);

    y + BIGCHAR_HEIGHT as f32 + 4.0
}

/// Raven `CG_DrawFPS` — the smoothed frames-per-second counter, sampled off
/// `trap_Milliseconds` (not server time) at most every 50ms.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3071-3113`
pub fn CG_DrawFPS(ctx: &mut CgContext, ds: &DisplayState, y: f32) -> f32 {
    let xOffset: c_int = 0;

    // don't use serverTime, because that will be drifting to correct for
    // internet lag changes, timescales, timedemos, etc
    let t = trap::Milliseconds(ctx.engine);
    let frameTime = (t - ctx.world.draw.fpsPrevious) as u16;
    ctx.world.draw.fpsPrevious = t;
    if t - ctx.world.draw.fpsLastupdate > 50 {
        // don't sample faster than this
        ctx.world.draw.fpsLastupdate = t;
        let idx = ctx.world.draw.fpsIndex as usize % FPS_FRAMES;
        ctx.world.draw.fpsPreviousTimes[idx] = frameTime;
        ctx.world.draw.fpsIndex = ctx.world.draw.fpsIndex.wrapping_add(1);
    }

    // average multiple frames together to smooth changes out a bit
    let mut total: c_int = 0;
    for i in 0..FPS_FRAMES {
        total += ctx.world.draw.fpsPreviousTimes[i] as c_int;
    }
    if total == 0 {
        total = 1;
    }
    let fps = 1000 * FPS_FRAMES as c_int / total;

    let s = format!("{fps}fps");
    let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;

    CG_DrawBigString(ctx, ds, 635 - w + xOffset, (y + 2.0) as c_int, &s, 1.0);

    y + BIGCHAR_HEIGHT as f32 + 4.0
}

/// Raven `CG_DrawTimer` — the mm:ss match clock in the top-right.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:3745-3770`
pub fn CG_DrawTimer(ctx: &mut CgContext, ds: &DisplayState, y: f32) -> f32 {
    let xOffset: c_int = 0;

    let msec = ctx.world.cg.time - ctx.world.cgs.levelStartTime;

    let mut seconds = msec / 1000;
    let mins = seconds / 60;
    seconds -= mins * 60;
    let tens = seconds / 10;
    seconds -= tens * 10;

    let s = format!("{mins}:{tens}{seconds}");
    let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;

    CG_DrawBigString(ctx, ds, 635 - w + xOffset, (y + 2.0) as c_int, &s, 1.0);

    y + BIGCHAR_HEIGHT as f32 + 4.0
}

/// Raven `CG_DrawDisconnect` — the "connection interrupted" text and blinking
/// phone-jack icon when we run past our command buffers, plus the map-change
/// notice.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4200-4241`
pub fn CG_DrawDisconnect(ctx: &mut CgContext, ds: &DisplayState) {
    if ctx.world.cg.mMapChange != qfalse {
        let s = CG_GetStringEdString(ctx, "MP_INGAME", "SERVER_CHANGING_MAPS");
        let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;
        CG_DrawBigString(ctx, ds, 320 - w / 2, 100, &s, 1.0);

        let s = CG_GetStringEdString(ctx, "MP_INGAME", "PLEASE_WAIT");
        let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;
        CG_DrawBigString(ctx, ds, 320 - w / 2, 200, &s, 1.0);
        return;
    }

    // draw the phone jack if we are completely past our buffers
    let cmdNum = trap::GetCurrentCmdNumber(ctx.engine) - CMD_BACKUP + 1;
    let mut cmd = usercmd_t::default();
    trap::GetUserCmd(ctx.engine, cmdNum, &mut cmd);

    // §F19: Raven derefs `cg.snap->ps.commandTime` unguarded; no snapshot = no
    // jack.
    let commandTime = {
        let Some(snap) = ctx.world.cg.snap_ref() else {
            return;
        };
        snap.ps.commandTime
    };
    if cmd.serverTime <= commandTime || cmd.serverTime > ctx.world.cg.time {
        // special check for map_restart
        return;
    }

    // also add text in center of screen
    let s = CG_GetStringEdString(ctx, "MP_INGAME", "CONNECTION_INTERRUPTED");
    let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;
    CG_DrawBigString(ctx, ds, 320 - w / 2, 100, &s, 1.0);

    // blink the icon
    if (ctx.world.cg.time >> 9) & 1 != 0 {
        return;
    }

    let x: f32 = 640.0 - 48.0;
    let y: f32 = 480.0 - 48.0;

    let shader = trap::R_RegisterShader(ctx.engine, "gfx/2d/net.tga");
    CG_DrawPic(ctx, x, y, 48.0, 48.0, shader);
}

/// Raven `CG_DrawBracketedEntities` — draws the targeting bracket around every
/// entity the server flagged as bracketed this frame.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:5634-5642`
pub fn CG_DrawBracketedEntities(ctx: &mut CgContext) {
    for i in 0..ctx.world.cg.bracketedEntityCount as usize {
        let n = ctx.world.cg.bracketedEntities[i] as usize;
        let radius = CG_RadiusForCent(ctx.world.entity(n));
        CG_BracketEntity(ctx, n, radius);
    }
}

/// Raven `CG_ScanForCrosshairEntity` — the per-frame trace out of the muzzle
/// (or camera) that decides which entity the crosshair is over, picks the
/// pilot's name over a vehicle, and drives the crosshair-name fade timer.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6072-6286`
pub fn CG_ScanForCrosshairEntity(ctx: &mut CgContext) {
    let mut trace = trace_t::zeroed();
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];

    let mut ignore = ctx.world.cg.predictedPlayerState.clientNum;

    // §F19: Raven derefs `cg.snap` unguarded at several points below (the muzzle
    // client index, the mind-trick and team reads, the final crosshair latch);
    // with no snapshot there is nothing under the crosshair to scan.
    let (snapWeapon, snapEmplacedIndex, snapClientNum, snapPersTeam) = match ctx.world.cg.snap_ref()
    {
        Some(snap) => (
            snap.ps.weapon,
            snap.ps.emplacedIndex,
            snap.ps.clientNum,
            snap.ps.persistant[PERS_TEAM as usize],
        ),
        None => return,
    };

    if ctx.world.cvars.cg_dynamicCrosshair.integer != 0 {
        let mut d_f: vec3_t = [0.0; 3];
        let mut d_rt: vec3_t = [0.0; 3];
        let mut d_up: vec3_t = [0.0; 3];

        // For now we still want to draw the crosshair in relation to the player's
        // world coordinates even if we have a melee weapon/no weapon.
        let m_iVehicleNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
        let eFlags = ctx.world.cg.predictedPlayerState.eFlags;
        if m_iVehicleNum != 0 && (eFlags & EF_NODRAW) != 0 {
            // we're *inside* a vehicle - do the vehicle's crosshair instead
            ignore = m_iVehicleNum;
            let _gunner = CG_CalcVehicleMuzzlePoint(
                ctx.world,
                m_iVehicleNum as usize,
                &mut start,
                &mut d_f,
                &mut d_rt,
                &mut d_up,
            );
            // DEFERRED: the huge-map fighter auto-aim (bVehCheckTraceFromCamPos)
            // gates on `veh->m_pVehicle->m_pVehicleInfo->type == VH_FIGHTER` and
            // its extra camera-origin trace reads `m_pVehicleInfo->length`;
            // DEC-46.2's presence-only `Option<VehicleId>` supplies neither, so
            // the feature stays off (normal crosshair trace) until the Vehicle_t
            // referent pool lands.
            //TODO: Port CG_ScanForCrosshairEntity vehicle arm
            // Source: oracle/codemp/cgame/cg_draw.c:6116-6216
        } else if snapWeapon == WP_EMPLACED_GUN
            && snapEmplacedIndex != 0
            && !ctx
                .world
                .entity(snapEmplacedIndex as usize)
                .ghoul2
                .is_null()
            && ctx
                .world
                .entity(snapEmplacedIndex as usize)
                .currentState
                .weapon
                == WP_NONE
        {
            // locked into our e-web, calc the muzzle from it
            CG_CalcEWebMuzzlePoint(
                ctx,
                snapEmplacedIndex as usize,
                &mut start,
                &mut d_f,
                &mut d_rt,
                &mut d_up,
            );
        } else {
            // pitchConstraint's first `VectorCopy(cg.refdef.viewangles, …)` in the
            // emplaced arm is immediately overwritten by the if/else below, so
            // only the conditional value survives.
            let mut pitchConstraint: vec3_t;
            if snapWeapon == WP_EMPLACED_GUN && snapEmplacedIndex != 0 {
                ignore = snapEmplacedIndex;

                pitchConstraint = if ctx.world.cg.renderingThirdPerson != qfalse {
                    ctx.world.cg.predictedPlayerState.viewangles
                } else {
                    ctx.world.cg.refdef.viewangles
                };

                if pitchConstraint[PITCH] > 40.0 {
                    pitchConstraint[PITCH] = 40.0;
                }
            } else {
                pitchConstraint = if ctx.world.cg.renderingThirdPerson != qfalse {
                    ctx.world.cg.predictedPlayerState.viewangles
                } else {
                    ctx.world.cg.refdef.viewangles
                };
            }
            AngleVectors(
                pitchConstraint,
                Some(&mut d_f),
                Some(&mut d_rt),
                Some(&mut d_up),
            );
            CG_CalcMuzzlePoint(ctx.world, snapClientNum, &mut start);
        }

        _VectorMA(start, ctx.world.cg.distanceCull, d_f, &mut end);
    } else {
        _VectorCopy(ctx.world.cg.refdef.vieworg, &mut start);
        let axis0 = ctx.world.cg.refdef.viewaxis[0];
        _VectorMA(start, 131072.0, axis0, &mut end);
    }

    if ctx.world.cvars.cg_dynamicCrosshair.integer != 0
        && ctx.world.cvars.cg_dynamicCrosshairPrecision.integer != 0
    {
        // then do a trace with ghoul2 models in mind
        CG_G2Trace(
            ctx,
            &mut trace,
            &start,
            &vec3_origin,
            &vec3_origin,
            &end,
            ignore,
            CONTENTS_SOLID | CONTENTS_BODY,
        );
    } else {
        CG_Trace(
            ctx,
            &mut trace,
            &start,
            &vec3_origin,
            &vec3_origin,
            &end,
            ignore,
            CONTENTS_SOLID | CONTENTS_BODY,
        );
    }

    if (trace.entityNum as c_int) < MAX_CLIENTS_I32 {
        let cs = ctx.world.entity(trace.entityNum as usize).currentState;
        if CG_IsMindTricked(
            ctx.world,
            cs.trickedentindex,
            cs.trickedentindex2,
            cs.trickedentindex3,
            cs.trickedentindex4,
            snapClientNum,
        ) {
            if ctx.world.cg.crosshairClientNum == trace.entityNum as c_int {
                ctx.world.cg.crosshairClientNum = ENTITYNUM_NONE;
                ctx.world.cg.crosshairClientTime = 0;
            }

            CG_DrawCrosshair(ctx, Some(trace.endpos), 0);

            // this entity is mind-tricking the current client, so don't render it
            return;
        }
    }

    if snapPersTeam != TEAM_SPECTATOR {
        if (trace.entityNum as c_int) < ENTITYNUM_WORLD {
            ctx.world.cg.crosshairClientNum = trace.entityNum as c_int;
            ctx.world.cg.crosshairClientTime = ctx.world.cg.time;

            if ctx.world.cg.crosshairClientNum < ENTITYNUM_WORLD {
                let ccn = ctx.world.cg.crosshairClientNum;
                let veh = ctx.world.entity(ccn as usize).currentState;

                if veh.eType == entityType_t::ET_NPC as c_int
                    && veh.NPC_class == class_t::CLASS_VEHICLE as c_int
                    && veh.owner < MAX_CLIENTS_I32
                {
                    // draw the name of the pilot then
                    ctx.world.cg.crosshairClientNum = veh.owner;
                    ctx.world.cg.crosshairVehNum = veh.number;
                    ctx.world.cg.crosshairVehTime = ctx.world.cg.time;
                }
            }

            CG_DrawCrosshair(ctx, Some(trace.endpos), 1);
        } else {
            CG_DrawCrosshair(ctx, Some(trace.endpos), 0);
        }
    }

    // if the player is in fog, don't show it
    let content = trap::CM_PointContents(ctx.engine, &trace.endpos, 0);
    if content & CONTENTS_FOG != 0 {
        return;
    }

    // update the fade timer
    ctx.world.cg.crosshairClientNum = trace.entityNum as c_int;
    ctx.world.cg.crosshairClientTime = ctx.world.cg.time;
}

/// Raven `CG_DrawSpectator` — the spectator banner: the "SPECTATOR" line, the
/// duel matchup with model icons/scores/health, and the join hint.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6470-6545`
pub fn CG_DrawSpectator(ctx: &mut CgContext, ds: &DisplayState) {
    let mut s = CG_GetStringEdString(ctx, "MP_INGAME", "SPECTATOR");

    let gametype = ctx.world.cgs.gametype;
    if (gametype == GT_DUEL || gametype == GT_POWERDUEL)
        && ctx.world.cgs.duelist1 != -1
        && ctx.world.cgs.duelist2 != -1
    {
        let size: c_int = 64;
        let d1 = ctx.world.cgs.duelist1 as usize;
        let d2 = ctx.world.cgs.duelist2 as usize;
        let name1 = buf_to_string(&ctx.world.cgs.clientinfo[d1].name.map(|c| c as u8));
        let name2 = buf_to_string(&ctx.world.cgs.clientinfo[d2].name.map(|c| c as u8));

        let text = if gametype == GT_POWERDUEL && ctx.world.cgs.duelist3 != -1 {
            let d3 = ctx.world.cgs.duelist3 as usize;
            let name3 = buf_to_string(&ctx.world.cgs.clientinfo[d3].name.map(|c| c as u8));
            let versus = CG_GetStringEdString(ctx, "MP_INGAME", "SPECHUD_VERSUS");
            let and = CG_GetStringEdString(ctx, "MP_INGAME", "AND");
            format!("{name1}^7 {versus} {name2}^7 {and} {name3}")
        } else {
            let versus = CG_GetStringEdString(ctx, "MP_INGAME", "SPECHUD_VERSUS");
            format!("{name1}^7 {versus} {name2}")
        };
        let w = CG_Text_Width(ctx, ds, &text, 1.0, 3);
        CG_Text_Paint(
            ctx,
            ds,
            (320 - w / 2) as f32,
            420.0,
            1.0,
            colorWhite,
            &text,
            0.0,
            0,
            0,
            3,
        );

        trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
        let icon1 = ctx.world.cgs.clientinfo[d1].modelIcon;
        if icon1 != 0 {
            CG_DrawPic(
                ctx,
                10.0,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5),
                size as f32,
                size as f32,
                icon1,
            );
        }
        let icon2 = ctx.world.cgs.clientinfo[d2].modelIcon;
        if icon2 != 0 {
            CG_DrawPic(
                ctx,
                SCREEN_WIDTH as f32 - size as f32 - 10.0,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5),
                size as f32,
                size as f32,
                icon2,
            );
        }

        // nmckenzie: DUEL_HEALTH
        if gametype == GT_DUEL && ctx.world.cgs.showDuelHealths >= 1 {
            // draw the healths on the two guys - how does this interact with
            // power duel, though?
            CG_DrawDuelistHealth(
                ctx,
                10.0,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5) - 12.0,
                64.0,
                8.0,
                1,
            );
            CG_DrawDuelistHealth(
                ctx,
                SCREEN_WIDTH as f32 - size as f32 - 10.0,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5) - 12.0,
                64.0,
                8.0,
                2,
            );
        }

        if gametype != GT_POWERDUEL {
            let t1 = format!(
                "{}/{}",
                ctx.world.cgs.clientinfo[d1].score, ctx.world.cgs.fraglimit
            );
            let w = CG_Text_Width(ctx, ds, &t1, 1.0, 2);
            CG_Text_Paint(
                ctx,
                ds,
                (42 - w / 2) as f32,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5) + 64.0,
                1.0,
                colorWhite,
                &t1,
                0.0,
                0,
                0,
                2,
            );

            let t2 = format!(
                "{}/{}",
                ctx.world.cgs.clientinfo[d2].score, ctx.world.cgs.fraglimit
            );
            let w = CG_Text_Width(ctx, ds, &t2, 1.0, 2);
            CG_Text_Paint(
                ctx,
                ds,
                (SCREEN_WIDTH - size + 22 - w / 2) as f32,
                SCREEN_HEIGHT as f32 - (size as f32 * 1.5) + 64.0,
                1.0,
                colorWhite,
                &t2,
                0.0,
                0,
                0,
                2,
            );
        }

        if gametype == GT_POWERDUEL && ctx.world.cgs.duelist3 != -1 {
            let d3 = ctx.world.cgs.duelist3 as usize;
            let icon3 = ctx.world.cgs.clientinfo[d3].modelIcon;
            if icon3 != 0 {
                CG_DrawPic(
                    ctx,
                    SCREEN_WIDTH as f32 - size as f32 - 10.0,
                    SCREEN_HEIGHT as f32 - (size as f32 * 2.8),
                    size as f32,
                    size as f32,
                    icon3,
                );
            }
        }
    } else {
        let w = CG_Text_Width(ctx, ds, &s, 1.0, 3);
        CG_Text_Paint(
            ctx,
            ds,
            (320 - w / 2) as f32,
            420.0,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            0,
            3,
        );
    }

    if gametype == GT_DUEL || gametype == GT_POWERDUEL {
        s = CG_GetStringEdString(ctx, "MP_INGAME", "WAITING_TO_PLAY");
        let w = CG_Text_Width(ctx, ds, &s, 1.0, 3);
        CG_Text_Paint(
            ctx,
            ds,
            (320 - w / 2) as f32,
            440.0,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            0,
            3,
        );
    } else {
        s = CG_GetStringEdString(ctx, "MP_INGAME", "SPEC_CHOOSEJOIN");
        let w = CG_Text_Width(ctx, ds, &s, 1.0, 3);
        CG_Text_Paint(
            ctx,
            ds,
            (320 - w / 2) as f32,
            440.0,
            1.0,
            colorWhite,
            &s,
            0.0,
            0,
            0,
            3,
        );
    }
}

/// Raven `CG_DrawVote` — the callvote status line: the command, the yes/no
/// tally, and the countdown, plus the "press ESC" hint.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6552-6652`
pub fn CG_DrawVote(ctx: &mut CgContext, ds: &DisplayState) {
    if ctx.world.cgs.voteTime == 0 {
        return;
    }

    // play a talk beep whenever it is modified
    if ctx.world.cgs.voteModified != qfalse {
        ctx.world.cgs.voteModified = qfalse;
        // Raven's `trap_S_StartLocalSound( cgs.media.talkSound, … )` is commented
        // out at the site.
    }

    let mut sec = (VOTE_TIME - (ctx.world.cg.time - ctx.world.cgs.voteTime)) / 1000;
    if sec < 0 {
        sec = 0;
    }

    let voteString = buf_to_string(&ctx.world.cgs.voteString.map(|c| c as u8));

    // §F19: Raven's `sCmd` is an uninitialized stack buffer printed even when no
    // vote-string prefix matches; the port starts it empty (the one defined
    // reading of that garbage).
    let mut sCmd = String::new();
    let mut sParm: Option<String> = None;

    if voteString.starts_with("map_restart") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_RESTART_MAP", 100)
            .unwrap_or_else(|| "??MENUS_RESTART_MAP".to_string());
    } else if voteString.starts_with("vstr nextmap") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_NEXT_MAP", 100)
            .unwrap_or_else(|| "??MENUS_NEXT_MAP".to_string());
    } else if voteString.starts_with("g_doWarmup") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_WARMUP", 100)
            .unwrap_or_else(|| "??MENUS_WARMUP".to_string());
    } else if voteString.starts_with("g_gametype") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_GAME_TYPE", 100)
            .unwrap_or_else(|| "??MENUS_GAME_TYPE".to_string());
        let parm = voteString.get(11..).unwrap_or("");
        if parm.eq_ignore_ascii_case("Free For All") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "FREE_FOR_ALL"));
        } else if parm.eq_ignore_ascii_case("Duel") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "DUEL"));
        } else if parm.eq_ignore_ascii_case("Holocron FFA") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "HOLOCRON_FFA"));
        } else if parm.eq_ignore_ascii_case("Power Duel") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "POWERDUEL"));
        } else if parm.eq_ignore_ascii_case("Team FFA") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "TEAM_FFA"));
        } else if parm.eq_ignore_ascii_case("Siege") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "SIEGE"));
        } else if parm.eq_ignore_ascii_case("Capture the Flag") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_FLAG"));
        } else if parm.eq_ignore_ascii_case("Capture the Ysalamiri") {
            sParm = Some(CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_YSALIMARI"));
        }
    } else if voteString.starts_with("map") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_NEW_MAP", 100)
            .unwrap_or_else(|| "??MENUS_NEW_MAP".to_string());
        sParm = Some(voteString.get(4..).unwrap_or("").to_string());
    } else if voteString.starts_with("kick") {
        sCmd = trap::SP_GetStringTextString(ctx.engine, "MENUS_KICK_PLAYER", 100)
            .unwrap_or_else(|| "??MENUS_KICK_PLAYER".to_string());
        sParm = Some(voteString.get(5..).unwrap_or("").to_string());
    }

    let sVote = trap::SP_GetStringTextString(ctx.engine, "MENUS_VOTE", 20)
        .unwrap_or_else(|| "??MENUS_VOTE".to_string());
    let sYes = trap::SP_GetStringTextString(ctx.engine, "MENUS_YES", 20)
        .unwrap_or_else(|| "??MENUS_YES".to_string());
    let sNo = trap::SP_GetStringTextString(ctx.engine, "MENUS_NO", 20)
        .unwrap_or_else(|| "??MENUS_NO".to_string());

    let voteYes = ctx.world.cgs.voteYes;
    let voteNo = ctx.world.cgs.voteNo;
    let s = match &sParm {
        Some(sp) if !sp.is_empty() => {
            format!("{sVote}({sec}):<{sCmd} {sp}> {sYes}:{voteYes} {sNo}:{voteNo}")
        }
        _ => format!("{sVote}({sec}):<{sCmd}> {sYes}:{voteYes} {sNo}:{voteNo}"),
    };
    CG_DrawSmallString(ctx, ds, 4, 58, &s, 1.0);

    let s = CG_GetStringEdString(ctx, "MP_INGAME", "OR_PRESS_ESC_THEN_CLICK_VOTE");
    CG_DrawSmallString(ctx, ds, 4, 58 + SMALLCHAR_HEIGHT + 2, &s, 1.0);
}

/// Raven `CG_DrawTeamVote` — the team callvote status line for the local
/// player's team (red uses slot 0, blue slot 1), with a "make X leader" gloss.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6659-6725`
pub fn CG_DrawTeamVote(ctx: &mut CgContext, ds: &DisplayState) {
    let cs_offset = if ctx.world.cgs.clientinfo[0].team == TEAM_RED {
        0usize
    } else if ctx.world.cgs.clientinfo[0].team == TEAM_BLUE {
        1usize
    } else {
        return;
    };

    if ctx.world.cgs.teamVoteTime[cs_offset] == 0 {
        return;
    }

    // play a talk beep whenever it is modified
    if ctx.world.cgs.teamVoteModified[cs_offset] != qfalse {
        ctx.world.cgs.teamVoteModified[cs_offset] = qfalse;
        // Raven's `trap_S_StartLocalSound(…)` is commented out at the site.
    }

    let mut sec = (VOTE_TIME - (ctx.world.cg.time - ctx.world.cgs.teamVoteTime[cs_offset])) / 1000;
    if sec < 0 {
        sec = 0;
    }

    let tvs = buf_to_string(&ctx.world.cgs.teamVoteString[cs_offset].map(|c| c as u8));
    let voteYes = ctx.world.cgs.teamVoteYes[cs_offset];
    let voteNo = ctx.world.cgs.teamVoteNo[cs_offset];

    let s = if tvs.contains("leader") {
        // walk to the first space; the rest is the target client index
        let bytes = tvs.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }

        if i < bytes.len() && bytes[i] == b' ' {
            let voteIndex = atoi(&tvs[i + 1..]);
            // §F19: `voteIndex` is server-supplied; a bad index reads garbage in
            // Raven, so an out-of-range one paints an empty name here.
            let name = if (0..MAX_CLIENTS_I32).contains(&voteIndex) {
                buf_to_string(
                    &ctx.world.cgs.clientinfo[voteIndex as usize]
                        .name
                        .map(|c| c as u8),
                )
            } else {
                String::new()
            };
            format!("TEAMVOTE({sec}):(Make {name} the new team leader) yes:{voteYes} no:{voteNo}")
        } else {
            format!("TEAMVOTE({sec}):{tvs} yes:{voteYes} no:{voteNo}")
        }
    } else {
        format!("TEAMVOTE({sec}):{tvs} yes:{voteYes} no:{voteNo}")
    };

    CG_DrawSmallString(ctx, ds, 4, 90, &s, 1.0);
}

/// Raven `CG_DrawWarmup` — the pre-match banner: the mode name (or the duel
/// matchup), the "starts in" countdown, and the announcer count-down beeps.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6923-7065`
pub fn CG_DrawWarmup(ctx: &mut CgContext, ds: &DisplayState) {
    let mut sec = ctx.world.cg.warmup;
    if sec == 0 {
        return;
    }

    if sec < 0 {
        let s = CG_GetStringEdString(ctx, "MP_INGAME", "WAITING_FOR_PLAYERS");
        let w = CG_DrawStrlen(&s) * BIGCHAR_WIDTH;
        CG_DrawBigString(ctx, ds, 320 - w / 2, 24, &s, 1.0);
        ctx.world.cg.warmupCount = 0;
        return;
    }

    let gametype = ctx.world.cgs.gametype;
    if gametype == GT_DUEL || gametype == GT_POWERDUEL {
        // find the two (or three) active players
        let mut ci1: Option<usize> = None;
        let mut ci2: Option<usize> = None;
        let mut ci3: Option<usize> = None;

        if gametype == GT_POWERDUEL {
            if ctx.world.cgs.duelist1 != -1 {
                ci1 = Some(ctx.world.cgs.duelist1 as usize);
            }
            if ctx.world.cgs.duelist2 != -1 {
                ci2 = Some(ctx.world.cgs.duelist2 as usize);
            }
            if ctx.world.cgs.duelist3 != -1 {
                ci3 = Some(ctx.world.cgs.duelist3 as usize);
            }
        } else {
            for i in 0..ctx.world.cgs.maxclients as usize {
                if ctx.world.cgs.clientinfo[i].infoValid != qfalse
                    && ctx.world.cgs.clientinfo[i].team == TEAM_FREE
                {
                    if ci1.is_none() {
                        ci1 = Some(i);
                    } else {
                        ci2 = Some(i);
                    }
                }
            }
        }

        if let (Some(c1), Some(c2)) = (ci1, ci2) {
            let name1 = buf_to_string(&ctx.world.cgs.clientinfo[c1].name.map(|c| c as u8));
            let name2 = buf_to_string(&ctx.world.cgs.clientinfo[c2].name.map(|c| c as u8));
            let s = if let Some(c3) = ci3 {
                let name3 = buf_to_string(&ctx.world.cgs.clientinfo[c3].name.map(|c| c as u8));
                format!("{name1} vs {name2} and {name3}")
            } else {
                format!("{name1} vs {name2}")
            };
            let w = CG_Text_Width(ctx, ds, &s, 0.6, FONT_MEDIUM);
            CG_Text_Paint(
                ctx,
                ds,
                (320 - w / 2) as f32,
                60.0,
                0.6,
                colorWhite,
                &s,
                0.0,
                0,
                ITEM_TEXTSTYLE_SHADOWEDMORE,
                FONT_MEDIUM,
            );
        }
    } else {
        let s = if gametype == GT_FFA {
            CG_GetStringEdString(ctx, "MENUS", "FREE_FOR_ALL")
        } else if gametype == GT_HOLOCRON {
            CG_GetStringEdString(ctx, "MENUS", "HOLOCRON_FFA")
        } else if gametype == GT_JEDIMASTER {
            CG_GetStringEdString(ctx, "MENUS", "POWERDUEL")
        } else if gametype == GT_TEAM {
            CG_GetStringEdString(ctx, "MENUS", "TEAM_FFA")
        } else if gametype == GT_SIEGE {
            CG_GetStringEdString(ctx, "MENUS", "SIEGE")
        } else if gametype == GT_CTF {
            CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_FLAG")
        } else if gametype == GT_CTY {
            CG_GetStringEdString(ctx, "MENUS", "CAPTURE_THE_YSALIMARI")
        } else {
            String::new()
        };
        let w = CG_Text_Width(ctx, ds, &s, 1.5, FONT_MEDIUM);
        CG_Text_Paint(
            ctx,
            ds,
            (320 - w / 2) as f32,
            90.0,
            1.5,
            colorWhite,
            &s,
            0.0,
            0,
            ITEM_TEXTSTYLE_SHADOWEDMORE,
            FONT_MEDIUM,
        );
    }

    sec = (sec - ctx.world.cg.time) / 1000;
    if sec < 0 {
        ctx.world.cg.warmup = 0;
        sec = 0;
    }
    let s = format!(
        "{}: {}",
        CG_GetStringEdString(ctx, "MP_INGAME", "STARTS_IN"),
        sec + 1
    );
    if sec != ctx.world.cg.warmupCount {
        ctx.world.cg.warmupCount = sec;

        if ctx.world.cgs.gametype != GT_SIEGE {
            match sec {
                0 => {
                    let snd = ctx.world.cgs.media.count1Sound;
                    trap::S_StartLocalSound(ctx.engine, snd, CHAN_ANNOUNCER);
                }
                1 => {
                    let snd = ctx.world.cgs.media.count2Sound;
                    trap::S_StartLocalSound(ctx.engine, snd, CHAN_ANNOUNCER);
                }
                2 => {
                    let snd = ctx.world.cgs.media.count3Sound;
                    trap::S_StartLocalSound(ctx.engine, snd, CHAN_ANNOUNCER);
                }
                _ => {}
            }
        }
    }

    // Raven's `cw` here is a pure dead store (set in the switch, never read),
    // so only `scale` survives.
    let scale = match ctx.world.cg.warmupCount {
        0 => 1.25f32,
        1 => 1.15,
        2 => 1.05,
        _ => 0.9,
    };

    let w = CG_Text_Width(ctx, ds, &s, scale, FONT_MEDIUM);
    CG_Text_Paint(
        ctx,
        ds,
        (320 - w / 2) as f32,
        125.0,
        scale,
        colorWhite,
        &s,
        0.0,
        0,
        ITEM_TEXTSTYLE_SHADOWEDMORE,
        FONT_MEDIUM,
    );
}

/// Raven `CG_DrawStats` — draws the vehicle HUD if we're piloting, then the
/// player HUD. Most of the body is commented out in Raven; only the HUD dispatch
/// survives.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:2699-2745`
pub fn CG_DrawStats(ctx: &mut CgContext, menus: &MenuSystem, ds: &DisplayState) {
    let mut drawHUD = true;

    // Raven: `cent = &cg_entities[cg.snap->ps.clientNum]` then treats it as
    // non-null, so the whole body runs.
    let centNum = match ctx.world.cg.snap_ref() {
        Some(snap) => snap.ps.clientNum as usize,
        // §F19: `cg.snap` is server-supplied and Raven derefs it unguarded; with
        // no snapshot yet there's nothing to draw.
        None => return,
    };

    // ps = &cg.predictedPlayerState
    if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0 {
        // In a vehicle???
        drawHUD = CG_DrawVehicleHud(ctx, menus, centNum);
    }

    if drawHUD {
        CG_DrawHUD(ctx, menus, ds, centNum);
    }
}

/// Raven `CG_DrawUpperRight` — the stacked upper-right readouts (team overlay,
/// snapshot/fps/timer, radar, enemy info, mini scoreboard, powerup icons).
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4020-4056`
pub fn CG_DrawUpperRight(ctx: &mut CgContext, ds: &DisplayState) {
    // Raven's `#ifdef _XBOX` starts `y` at 50; the PC build starts at 0.
    let mut y: f32 = 0.0;

    trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));

    if ctx.world.cgs.gametype >= GT_TEAM && ctx.world.cvars.cg_drawTeamOverlay.integer == 1 {
        y = CG_DrawTeamOverlay(ctx, ds, y, true, true);
    }
    if ctx.world.cvars.cg_drawSnapshot.integer != 0 {
        y = CG_DrawSnapshot(ctx, ds, y);
    }

    if ctx.world.cvars.cg_drawFPS.integer != 0 {
        y = CG_DrawFPS(ctx, ds, y);
    }
    if ctx.world.cvars.cg_drawTimer.integer != 0 {
        y = CG_DrawTimer(ctx, ds, y);
    }

    if (ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0)
        && ctx.world.cvars.cg_drawRadar.integer != 0
    {
        //draw Radar in Siege mode or when in a vehicle of any kind
        y = CG_DrawRadar(ctx, y);
    }

    y = CG_DrawEnemyInfo(ctx, ds, y);

    y = CG_DrawMiniScoreboard(ctx, ds, y);

    CG_DrawPowerupIcons(ctx, ds, y as c_int);
}

/// Raven `CG_DrawLagometer` — the two-row interpolate/snapshot graph in the
/// lower-right, or the disconnect icon in its place.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:4252-4351`
pub fn CG_DrawLagometer(ctx: &mut CgContext, ds: &DisplayState) {
    if ctx.world.cvars.cg_lagometer.integer == 0 || ctx.world.cgs.localServer != qfalse {
        CG_DrawDisconnect(ctx, ds);
        return;
    }

    //
    // draw the graph
    //
    let x: c_int = 640 - 48;
    let y: c_int = 480 - 144;

    let lagometerShader = ctx.world.cgs.media.lagometerShader;
    let whiteShader = ctx.world.cgs.media.whiteShader;

    trap::R_SetColor(ctx.engine, None);
    CG_DrawPic(ctx, x as f32, y as f32, 48.0, 48.0, lagometerShader);

    let ax: f32 = x as f32;
    let ay: f32 = y as f32;
    let aw: f32 = 48.0;
    let ah: f32 = 48.0;

    let mut color: c_int = -1;
    let mut range: f32 = ah / 3.0;
    let mid: f32 = ay + range;

    let mut vscale: f32 = range / MAX_LAGOMETER_RANGE as f32;

    // draw the frame interpoalte / extrapolate graph
    let frameCount = ctx.world.draw.lagometer.frameCount;
    for a in 0..(aw as c_int) {
        let i = ((frameCount - 1 - a) & (LAG_SAMPLES as c_int - 1)) as usize;
        let mut v = ctx.world.draw.lagometer.frameSamples[i] as f32;
        v *= vscale;
        if v > 0.0 {
            if color != 1 {
                color = 1;
                trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_YELLOW_INDEX]));
            }
            if v > range {
                v = range;
            }
            trap::R_DrawStretchPic(
                ctx.engine,
                ax + aw - a as f32,
                mid - v,
                1.0,
                v,
                0.0,
                0.0,
                0.0,
                0.0,
                whiteShader,
            );
        } else if v < 0.0 {
            if color != 2 {
                color = 2;
                trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_BLUE_INDEX]));
            }
            v = -v;
            if v > range {
                v = range;
            }
            trap::R_DrawStretchPic(
                ctx.engine,
                ax + aw - a as f32,
                mid,
                1.0,
                v,
                0.0,
                0.0,
                0.0,
                0.0,
                whiteShader,
            );
        }
    }

    // draw the snapshot latency / drop graph
    range = ah / 2.0;
    vscale = range / MAX_LAGOMETER_PING as f32;

    let snapshotCount = ctx.world.draw.lagometer.snapshotCount;
    for a in 0..(aw as c_int) {
        let i = ((snapshotCount - 1 - a) & (LAG_SAMPLES as c_int - 1)) as usize;
        let mut v = ctx.world.draw.lagometer.snapshotSamples[i] as f32;
        if v > 0.0 {
            if ctx.world.draw.lagometer.snapshotFlags[i] & SNAPFLAG_RATE_DELAYED != 0 {
                if color != 5 {
                    color = 5; // YELLOW for rate delay
                    trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_YELLOW_INDEX]));
                }
            } else if color != 3 {
                color = 3;
                trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_GREEN_INDEX]));
            }
            v *= vscale;
            if v > range {
                v = range;
            }
            trap::R_DrawStretchPic(
                ctx.engine,
                ax + aw - a as f32,
                ay + ah - v,
                1.0,
                v,
                0.0,
                0.0,
                0.0,
                0.0,
                whiteShader,
            );
        } else if v < 0.0 {
            if color != 4 {
                color = 4; // RED for dropped snapshots
                trap::R_SetColor(ctx.engine, Some(&g_color_table[COLOR_RED_INDEX]));
            }
            trap::R_DrawStretchPic(
                ctx.engine,
                ax + aw - a as f32,
                ay + ah - range,
                1.0,
                range,
                0.0,
                0.0,
                0.0,
                0.0,
                whiteShader,
            );
        }
    }

    trap::R_SetColor(ctx.engine, None);

    if ctx.world.cvars.cg_nopredict.integer != 0
        || ctx.world.cvars.cg_synchronousClients.integer != 0
    {
        CG_DrawBigString(ctx, ds, ax as c_int, ay as c_int, "snc", 1.0);
    }

    CG_DrawDisconnect(ctx, ds);
}

/// Raven `CG_DrawCrosshairNames` — the name of whoever the crosshair is on,
/// tinted by team/duel relationship and faded on the crosshair timer.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6333-6460`
pub fn CG_DrawCrosshairNames(ctx: &mut CgContext, ds: &DisplayState) {
    if ctx.world.cvars.cg_drawCrosshair.integer == 0 {
        return;
    }

    // scan the known entities to see if the crosshair is sighted on one
    CG_ScanForCrosshairEntity(ctx);

    if ctx.world.cvars.cg_drawCrosshairNames.integer == 0 {
        return;
    }
    //rww - still do the trace, our dynamic crosshair depends on it

    let mut isVeh = false;
    if ctx.world.cg.crosshairClientNum < ENTITYNUM_WORLD {
        // copy the vehicle's state out so the borrow ends before we write cg
        let ves = &ctx
            .world
            .entity(ctx.world.cg.crosshairClientNum as usize)
            .currentState;
        let (eType, npcClass, owner, number) = (ves.eType, ves.NPC_class, ves.owner, ves.number);

        if eType == entityType_t::ET_NPC as c_int
            && npcClass == class_t::CLASS_VEHICLE as c_int
            && owner < MAX_CLIENTS_I32
        {
            //draw the name of the pilot then
            ctx.world.cg.crosshairClientNum = owner;
            ctx.world.cg.crosshairVehNum = number;
            ctx.world.cg.crosshairVehTime = ctx.world.cg.time;
            isVeh = true; //so we know we're drawing the pilot's name
        }
    }

    if ctx.world.cg.crosshairClientNum >= MAX_CLIENTS_I32 {
        return;
    }

    let cn = ctx.world.cg.crosshairClientNum as usize;

    if ctx.world.entity(cn).currentState.powerups & (1 << PW_CLOAKED) != 0 {
        return;
    }

    // draw the name of the player being looked at
    let crosshairClientTime = ctx.world.cg.crosshairClientTime;
    let Some(color) = CG_FadeColor(ctx.world, crosshairClientTime, 1000) else {
        trap::R_SetColor(ctx.engine, None);
        return;
    };

    // Raven derefs `cg.snap` unguarded below (duel state); a null there is UB.
    let (snapPsClientNum, snapPsDuelInProgress, snapPsDuelIndex) = match ctx.world.cg.snap_ref() {
        Some(snap) => (
            snap.ps.clientNum as usize,
            snap.ps.duelInProgress,
            snap.ps.duelIndex,
        ),
        // §F19: no snapshot yet means nothing to draw.
        None => return,
    };
    let predClientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;

    let name = buf_to_string(&ctx.world.cgs.clientinfo[cn].name.map(|c| c as u8));

    let mut baseColor: usize;
    if ctx.world.cgs.gametype >= GT_TEAM {
        // Raven's `if (1)`: instead of team-based we orient by which team we're
        // on. The `else` arm (TEAM_RED/TEAM_BLUE coloring) is dead behind it.
        if ctx.world.cgs.clientinfo[cn].team
            == ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize]
        {
            baseColor = ct_table_t::CT_GREEN as usize;
        } else {
            baseColor = ct_table_t::CT_RED as usize;
        }
    } else if ctx.world.cgs.gametype == GT_POWERDUEL
        && ctx.world.cgs.clientinfo[snapPsClientNum].team != TEAM_SPECTATOR
        && ctx.world.cgs.clientinfo[cn].duelTeam == ctx.world.cgs.clientinfo[predClientNum].duelTeam
    {
        //on the same duel team in powerduel, so he's a friend
        baseColor = ct_table_t::CT_GREEN as usize;
    } else {
        baseColor = ct_table_t::CT_RED as usize; //just make it red in nonteam modes since everyone is hostile and crosshair will be red on them too
    }

    if snapPsDuelInProgress != qfalse {
        if cn as c_int != snapPsDuelIndex {
            //grey out crosshair for everyone but your foe if you're in a duel
            baseColor = ct_table_t::CT_BLACK as usize;
        }
    } else if ctx.world.entity(cn).currentState.bolt1 != 0 {
        //this fellow is in a duel. We just checked if we were in a duel above, so
        //this means we aren't and he is. Which of course means our crosshair greys out over him.
        baseColor = ct_table_t::CT_BLACK as usize;
    }

    let mut tcolor: vec4_t = [0.0; 4];
    tcolor[0] = colorTable[baseColor][0];
    tcolor[1] = colorTable[baseColor][1];
    tcolor[2] = colorTable[baseColor][2];
    tcolor[3] = color[3] * 0.5;

    let sanitized = CG_SanitizeString(&name);

    if isVeh {
        let str = format!("{sanitized} (pilot)");
        UI_DrawProportionalString(ctx, ds, 320, 170, &str, UI_CENTER, tcolor);
    } else {
        UI_DrawProportionalString(ctx, ds, 320, 170, &sanitized, UI_CENTER, tcolor);
    }

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_DrawScoreboard` — the new (menu-driven) scoreboard is compiled out
/// (`#if 0`), so this just forwards to the old scoreboard.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6727-6793`
pub fn CG_DrawScoreboard(ctx: &mut CgContext, ds: &DisplayState) -> bool {
    CG_DrawOldScoreboard(ctx, ds)
}

/// Raven `CG_DrawIntermission`.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:6800-6808`
pub fn CG_DrawIntermission(ctx: &mut CgContext, ds: &DisplayState) {
    // int key;
    // if (cg_singlePlayer.integer) {
    // 	CG_DrawCenterString();
    // 	return;
    // }
    ctx.world.cg.scoreFadeTime = ctx.world.cg.time;
    let scoreBoardShowing = CG_DrawScoreboard(ctx, ds);
    ctx.world.cg.scoreBoardShowing = if scoreBoardShowing { qtrue } else { qfalse };
}

/// Raven `CG_Draw2D` — the whole 2D pass: screen tints, rocket/holocron/power
/// overlays, fuel and e-web bars, the zoom mask, spectator vs. live status
/// (crosshair, selection wheels, stats), the fall-to-death fade, votes,
/// lagometer, follow/warmup, siege round + death timers, scoreboard and chat.
///
/// Source: `oracle/codemp/cgame/cg_draw.c:8118-8515`
pub fn CG_Draw2D(ctx: &mut CgContext, menus: &MenuSystem, ds: &DisplayState) {
    let inTime = ctx.world.cg.invenSelectTime + WEAPON_SELECT_TIME as f32;
    let wpTime = ctx.world.cg.weaponSelectTime as f32 + WEAPON_SELECT_TIME as f32;

    // if we are taking a levelshot for the menu, don't draw anything
    if ctx.world.cg.levelShot != qfalse {
        return;
    }

    // Raven derefs `cg.snap` unguarded through this whole body; §F19 — with no
    // snapshot yet there's no HUD to draw, so bail rather than fault on a
    // server-supplied null.
    let ps = match ctx.world.cg.snap_ref() {
        Some(snap) => &snap.ps,
        None => return,
    };
    let clientNum = ps.clientNum;
    let pm_type = ps.pm_type;
    let rocketLockIndex = ps.rocketLockIndex;
    let rocketLockTime = ps.rocketLockTime;
    let holocronBits = ps.holocronBits;
    let forcePowersActive = ps.fd.forcePowersActive;
    let forceRageRecoveryTime = ps.fd.forceRageRecoveryTime;
    let jetpackFuel = ps.jetpackFuel;
    let cloakFuel = ps.cloakFuel;
    let persTeam = ps.persistant[PERS_TEAM as usize];
    let health = ps.stats[STAT_HEALTH as usize];
    let fallingToDeath = ps.fallingToDeath;
    let psOrigin = ps.origin;

    if ctx.world.cgs.clientinfo[clientNum as usize].team == TEAM_SPECTATOR {
        ctx.world.draw.cgRageTime = 0;
        ctx.world.draw.cgRageFadeTime = 0;
        ctx.world.draw.cgRageFadeVal = 0.0;

        ctx.world.draw.cgRageRecTime = 0;
        ctx.world.draw.cgRageRecFadeTime = 0;
        ctx.world.draw.cgRageRecFadeVal = 0.0;

        ctx.world.draw.cgAbsorbTime = 0;
        ctx.world.draw.cgAbsorbFadeTime = 0;
        ctx.world.draw.cgAbsorbFadeVal = 0.0;

        ctx.world.draw.cgProtectTime = 0;
        ctx.world.draw.cgProtectFadeTime = 0;
        ctx.world.draw.cgProtectFadeVal = 0.0;

        ctx.world.draw.cgYsalTime = 0;
        ctx.world.draw.cgYsalFadeTime = 0;
        ctx.world.draw.cgYsalFadeVal = 0.0;
    }

    if ctx.world.cvars.cg_draw2D.integer == 0 {
        return;
    }

    if pm_type == PM_INTERMISSION as c_int {
        CG_DrawIntermission(ctx, ds);
        CG_ChatBox_DrawStrings(ctx, ds);
        return;
    }

    CG_Draw2DScreenTints(ctx);

    if rocketLockIndex != ENTITYNUM_NONE && (ctx.world.cg.time as f32 - rocketLockTime) > 0.0 {
        CG_DrawRocketLocking(ctx, rocketLockIndex as usize, rocketLockTime as c_int);
    }

    if holocronBits != 0 {
        CG_DrawHolocronIcons(ctx);
    }
    if forcePowersActive != 0 || forceRageRecoveryTime > ctx.world.cg.time {
        CG_DrawActivePowers(ctx);
    }

    if jetpackFuel < 100 {
        //draw it as long as it isn't full
        CG_DrawJetpackFuel(ctx);
    }
    if cloakFuel < 100 {
        //draw it as long as it isn't full
        CG_DrawCloakFuel(ctx);
    }
    if ctx.world.cg.predictedPlayerState.emplacedIndex > 0 {
        let emplacedIndex = ctx.world.cg.predictedPlayerState.emplacedIndex as usize;

        if ctx.world.entity(emplacedIndex).currentState.weapon == WP_NONE as c_int {
            //using an e-web, draw its health
            CG_DrawEWebHealth(ctx);
        }
    }

    // Draw this before the text so that any text won't get clipped off
    CG_DrawZoomMask(ctx);

    if persTeam == TEAM_SPECTATOR {
        CG_DrawSpectator(ctx, ds);
        CG_DrawCrosshair(ctx, None, 0);
        CG_DrawCrosshairNames(ctx, ds);
        CG_SaberClashFlare(ctx);
    } else {
        // don't draw any status if dead or the scoreboard is being explicitly shown
        if ctx.world.cg.showScores == qfalse && health > 0 {
            // Raven's `if (0)` Menu_PaintAll/CG_DrawTimedMenus block ("Reenable
            // if stats are drawn with menu system again") never runs in the
            // shipped build - dropped as unreachable, cg_marks.c precedent.

            CG_DrawAmmoWarning();

            CG_DrawCrosshairNames(ctx, ds);

            if ctx.world.cvars.cg_drawStatus.integer != 0 {
                CG_DrawIconBackground(ctx.world);
            }

            let bestTime: f32;
            let mut drawSelect: c_int;
            if inTime > wpTime {
                drawSelect = 1;
                bestTime = ctx.world.cg.invenSelectTime;
            } else {
                //only draw the most recent since they're drawn in the same place
                drawSelect = 2;
                bestTime = ctx.world.cg.weaponSelectTime as f32;
            }

            if ctx.world.cg.forceSelectTime > bestTime {
                drawSelect = 3;
            }

            match drawSelect {
                1 => CG_DrawInvenSelect(ctx, ds),
                2 => CG_DrawWeaponSelect(ctx, ds),
                3 => CG_DrawForceSelect(ctx, ds),
                _ => {}
            }

            if ctx.world.cvars.cg_drawStatus.integer != 0 {
                //Powerups now done with upperright stuff
                CG_DrawFlagStatus(ctx);
            }

            CG_SaberClashFlare(ctx);

            if ctx.world.cvars.cg_drawStatus.integer != 0 {
                CG_DrawStats(ctx, menus, ds);
            }

            CG_DrawPickupItem(ctx);
        }
    }

    if fallingToDeath != 0 {
        let mut fallTime = (ctx.world.cg.time - fallingToDeath) as f32;

        fallTime /= (FALL_FADE_TIME / 2) as f32;

        if fallTime < 0.0 {
            fallTime = 0.0;
        }
        if fallTime > 1.0 {
            fallTime = 1.0;
        }

        let hcolor: vec4_t = [0.0, 0.0, 0.0, fallTime];

        CG_DrawRect(
            ctx,
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            (SCREEN_WIDTH * SCREEN_HEIGHT) as f32,
            &hcolor,
        );

        if !ctx.world.draw.gCGHasFallVector {
            ctx.world.draw.gCGFallVector = psOrigin;
            ctx.world.draw.gCGHasFallVector = true;
        }
    } else if ctx.world.draw.gCGHasFallVector {
        ctx.world.draw.gCGHasFallVector = false;
        ctx.world.draw.gCGFallVector = [0.0; 3];
    }

    CG_DrawVote(ctx, ds);
    CG_DrawTeamVote(ctx, ds);

    CG_DrawLagometer(ctx, ds);

    if ctx.world.cvars.cg_paused.integer == 0 {
        CG_DrawBracketedEntities(ctx);
        CG_DrawUpperRight(ctx, ds);
    }

    if !CG_DrawFollow(ctx, ds) {
        CG_DrawWarmup(ctx, ds);
    }

    if ctx.world.saga.cgSiegeRoundState != 0 {
        //cgSiegeRoundBeganTime = 0;

        match ctx.world.saga.cgSiegeRoundState {
            1 => {
                let s = CG_GetStringEdString(ctx, "MP_INGAME", "WAITING_FOR_PLAYERS");
                CG_CenterPrint(
                    ctx.world,
                    &s,
                    (SCREEN_HEIGHT as f64 * 0.30) as c_int,
                    BIGCHAR_WIDTH,
                );
            }

            2 => {
                let mut rTime =
                    SIEGE_ROUND_BEGIN_TIME - (ctx.world.cg.time - ctx.world.saga.cgSiegeRoundTime);

                if rTime < 0 {
                    rTime = 0;
                }
                if rTime > SIEGE_ROUND_BEGIN_TIME {
                    rTime = SIEGE_ROUND_BEGIN_TIME;
                }

                rTime /= 1000;

                rTime += 1;

                if rTime < 1 {
                    rTime = 1;
                }

                if rTime <= 3 && rTime != ctx.world.draw.cgSiegeRoundCountTime {
                    ctx.world.draw.cgSiegeRoundCountTime = rTime;

                    match rTime {
                        1 => trap::S_StartLocalSound(
                            ctx.engine,
                            ctx.world.cgs.media.count1Sound,
                            CHAN_ANNOUNCER,
                        ),
                        2 => trap::S_StartLocalSound(
                            ctx.engine,
                            ctx.world.cgs.media.count2Sound,
                            CHAN_ANNOUNCER,
                        ),
                        3 => trap::S_StartLocalSound(
                            ctx.engine,
                            ctx.world.cgs.media.count3Sound,
                            CHAN_ANNOUNCER,
                        ),
                        _ => {}
                    }
                }

                let ed = CG_GetStringEdString(ctx, "MP_INGAME", "ROUNDBEGINSIN");
                let pStr = format!("{} {}...", ed, rTime);
                CG_CenterPrint(
                    ctx.world,
                    &pStr,
                    (SCREEN_HEIGHT as f64 * 0.30) as c_int,
                    BIGCHAR_WIDTH,
                );
                //same
            }

            _ => {}
        }

        ctx.world.draw.cgSiegeEntityRender = 0;
    } else if ctx.world.saga.cgSiegeRoundTime != 0 {
        CG_CenterPrint(
            ctx.world,
            "",
            (SCREEN_HEIGHT as f64 * 0.30) as c_int,
            BIGCHAR_WIDTH,
        );
        ctx.world.saga.cgSiegeRoundTime = 0;

        //cgSiegeRoundBeganTime = cg.time;
        ctx.world.draw.cgSiegeEntityRender = 0;
    } else if ctx.world.draw.cgSiegeRoundBeganTime != 0 {
        //Draw how much time is left in the round based on local info.
        let mut timedTeam = TEAM_FREE;
        let mut timedValue = 0;

        if ctx.world.draw.cgSiegeEntityRender != 0 {
            //render the objective item model since this client has it
            CG_DrawSiegeHUDItem(ctx);
        }

        if ctx.world.saga.team1Timed != 0 {
            timedTeam = TEAM_RED; //team 1
            if ctx.world.draw.cg_beatingSiegeTime != 0 {
                timedValue = ctx.world.draw.cg_beatingSiegeTime;
            } else {
                timedValue = ctx.world.saga.team1Timed;
            }
        } else if ctx.world.saga.team2Timed != 0 {
            timedTeam = TEAM_BLUE; //team 2
            if ctx.world.draw.cg_beatingSiegeTime != 0 {
                timedValue = ctx.world.draw.cg_beatingSiegeTime;
            } else {
                timedValue = ctx.world.saga.team2Timed;
            }
        }

        if timedTeam != TEAM_FREE {
            //one of the teams has a timer
            let mut timeRemaining;
            let mut isMyTeam = false;

            if ctx.world.cgs.siegeTeamSwitch != 0 && ctx.world.draw.cg_beatingSiegeTime == 0 {
                //in switchy mode but not beating a time, so count up.
                timeRemaining = ctx.world.cg.time - ctx.world.draw.cgSiegeRoundBeganTime;
                if timeRemaining < 0 {
                    timeRemaining = 0;
                }
            } else {
                timeRemaining =
                    (ctx.world.draw.cgSiegeRoundBeganTime + timedValue) - ctx.world.cg.time;
            }

            if timeRemaining > timedValue {
                timeRemaining = timedValue;
            } else if timeRemaining < 0 {
                timeRemaining = 0;
            }

            if timeRemaining != 0 {
                timeRemaining /= 1000;
            }

            if ctx.world.cg.predictedPlayerState.persistant[PERS_TEAM as usize] == timedTeam {
                //the team that's timed is the one this client is on
                isMyTeam = true;
            }

            CG_DrawSiegeTimer(ctx, menus, ds, timeRemaining, isMyTeam);
        }
    } else {
        ctx.world.draw.cgSiegeEntityRender = 0;
    }

    if ctx.world.draw.cg_siegeDeathTime != 0 {
        let mut timeRemaining = ctx.world.draw.cg_siegeDeathTime - ctx.world.cg.time;

        if timeRemaining < 0 {
            timeRemaining = 0;
            ctx.world.draw.cg_siegeDeathTime = 0;
        }

        if timeRemaining != 0 {
            timeRemaining /= 1000;
        }

        CG_DrawSiegeDeathTimer(ctx, menus, ds, timeRemaining);
    }

    // don't draw center string if scoreboard is up
    let scoreBoardShowing = CG_DrawScoreboard(ctx, ds);
    ctx.world.cg.scoreBoardShowing = if scoreBoardShowing { qtrue } else { qfalse };
    if ctx.world.cg.scoreBoardShowing == qfalse {
        CG_DrawCenterString(ctx, ds);
    }

    // always draw chat
    CG_ChatBox_DrawStrings(ctx, ds);
}
