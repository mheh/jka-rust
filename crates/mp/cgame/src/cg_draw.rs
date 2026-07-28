//! Port of `oracle/codemp/cgame/cg_draw.c` — the HUD and every other 2D overlay drawn each frame. Functions land via the C5
//! transcription waves.

// Raven's own spellings survive: `veh_damage_t` is a snake_case type name and
// `colorTable`/`vehDamageData` are camelCase consts.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use native_string::{atoi, string_to_latin1, Q_strncpyzBytes};

use mp_abi::ui::public::ui_menu_command_t::{
    UIMENU_CLOSEALL, UIMENU_SIEGEMESSAGE, UIMENU_SIEGEOBJECTIVES,
};
use mp_bg::bg_misc::{
    forcePowerSorted, BG_EvaluateTrajectory, BG_FindItemForPowerup, BG_GiveMeVectorFromMatrix,
};
use mp_bg::public::animation::animation_t;
use mp_bg::public::entity_flags::EF_RADAROBJECT;
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_CTF, GT_CTY};
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_NEUTRALFLAG, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t::STAT_HEALTH;
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED, TEAM_SPECTATOR};
use mp_bg::weapons::weapon_t::WP_SABER;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::qcommon::PMF_FOLLOW;
use mp_qshared::shared::force_powers::{
    FP_ABSORB, FP_HEAL, FP_LEVITATION, FP_PROTECT, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE,
    FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TELEPATHY, NUM_FORCE_POWERS,
};
use mp_qshared::shared::q_color::{colorWhite, g_color_table};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorSubtract, AngleVectors, AnglesToAxis, Distance,
    VectorClear, VectorNormalize, VectorSet, YAW,
};
use mp_qshared::shared::{
    ct_table_t, mdxaBone_t, qfalse, qhandle_t, vec3_t, vec4_t, Eorientations, BIGCHAR_WIDTH,
    CHAN_AUTO, CHAN_LOCAL, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS_I32, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_id::MenuId;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::shared::menudef::{
    ITEM_TEXTSTYLE_BLINK, ITEM_TEXTSTYLE_NORMAL, ITEM_TEXTSTYLE_OUTLINED,
    ITEM_TEXTSTYLE_OUTLINESHADOWED, ITEM_TEXTSTYLE_PULSE, ITEM_TEXTSTYLE_SHADOWED,
    ITEM_TEXTSTYLE_SHADOWEDMORE,
};
use mp_uishared::ui_shared::{Menu_FindItemByName, Menus_CloseByName};

use crate::cg_drawtools::{CG_DrawPic, CG_DrawRotatePic2, CG_FillRect};
use crate::cg_main::CG_Error;
use crate::cg_new_draw::{CG_OtherTeamHasFlag, CG_YourTeamHasFlag};
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

/// Raven `#define MAX_HUD_TICS 4` — tics per HUD bar (health/armor/force/ammo).
/// Source: `oracle/codemp/cgame/cg_draw.c:42`
pub const MAX_HUD_TICS: usize = 4;

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

/// Raven `#define JPFUELBAR_W 20.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7147`
pub const JPFUELBAR_W: f32 = 20.0;

/// Raven `#define JPFUELBAR_X (SCREEN_WIDTH-JPFUELBAR_W-8.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7148`
pub const JPFUELBAR_X: f32 = SCREEN_WIDTH as f32 - JPFUELBAR_W - 8.0;

/// Raven `#define JPFUELBAR_Y 260.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7149`
pub const JPFUELBAR_Y: f32 = 260.0;

/// Raven `#define EWEBHEALTH_W 20.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7199`
pub const EWEBHEALTH_W: f32 = 20.0;

/// Raven `#define EWEBHEALTH_X (SCREEN_WIDTH-EWEBHEALTH_W-8.0f)`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7200`
pub const EWEBHEALTH_X: f32 = SCREEN_WIDTH as f32 - EWEBHEALTH_W - 8.0;

/// Raven `#define EWEBHEALTH_Y 290.0f`.
/// Source: `oracle/codemp/cgame/cg_draw.c:7201`
pub const EWEBHEALTH_Y: f32 = 290.0;

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
    // DEFERRED: CgBgTraps — cgame has no `mp_bg::bg_channel::BgTraps`
    // implementor yet (`crates/mp/cgame/src/bg_channel/mod.rs`: "the `BgTraps`
    // one follows with the transcription waves"), and `BG_ParseAnimationFile`
    // takes one, so the single call this fn is cannot be made.
    // Source: oracle/codemp/cgame/cg_draw.c:104
    let _ = (ctx, filename, animset, isHumanoid);
    //TODO: Port Port
    // Source: oracle/codemp/cgame/cg_draw.c:102-105
    todo!("Port UI_ParseAnimationFile — oracle/codemp/cgame/cg_draw.c:102-105")
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
        resolved = trap::SP_GetStringTextString(ctx.engine, key, 1024);
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
