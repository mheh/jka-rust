//! Port of `oracle/codemp/cgame/cg_draw.c` — the HUD and every other 2D overlay drawn each frame. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use native_string::{string_to_latin1, Q_strncpyzBytes};

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_abi::ui::public::ui_menu_command_t::{
    UIMENU_CLOSEALL, UIMENU_SIEGEMESSAGE, UIMENU_SIEGEOBJECTIVES,
};
use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::public::animation::animation_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::shared::force_powers::{
    FP_ABSORB, FP_HEAL, FP_LEVITATION, FP_PROTECT, FP_RAGE, FP_SABERTHROW, FP_SABER_DEFENSE,
    FP_SABER_OFFENSE, FP_SEE, FP_SPEED, FP_TELEPATHY, NUM_FORCE_POWERS,
};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorSubtract, AngleVectors, AnglesToAxis, VectorClear,
};
use mp_qshared::shared::{
    mdxaBone_t, qhandle_t, vec3_t, Eorientations, ENTITYNUM_NONE, ENTITYNUM_WORLD, MAX_CLIENTS_I32,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use mp_uishared::ui_shared::Menus_CloseByName;

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
pub fn CG_AddLagometerSnapshotInfo(world: &mut CgWorld, snap: Option<&snapshot_t>) {
    let lag = &mut world.draw.lagometer;
    let i = (lag.snapshotCount & (LAG_SAMPLES as c_int - 1)) as usize;

    // dropped packet
    let snap = match snap {
        Some(s) => s,
        None => {
            lag.snapshotSamples[i] = -1;
            lag.snapshotCount += 1;
            return;
        }
    };

    // add this snapshot's info
    lag.snapshotSamples[i] = snap.ping;
    lag.snapshotFlags[i] = snap.snapFlags;
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
