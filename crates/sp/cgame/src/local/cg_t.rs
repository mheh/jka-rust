#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_abi::cgame::public::snapshot_s::snapshot_t;
use sp_qshared::common::sp::qcommon::player_state::playerState_t;
use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::common::sp::renderer::refdef_t::refdef_t;
use sp_qshared::shared::{qboolean, vec3_t, MAX_QPATH, MAX_STRING_CHARS};

use super::overrides_t::overrides_t;

/// Raven `MAX_PRINTTEXT`.
///
/// Source: `oracle/code/cgame/cg_local.h:64`
pub const MAX_PRINTTEXT: usize = 128;

/// Raven `MAX_CAPTIONTEXT`.
///
/// Raven: we don't need 64 now since we don't use this for scroll text, and I
/// needed to change a hardwired 128 to 256, so...
/// Source: `oracle/code/cgame/cg_local.h:65`
pub const MAX_CAPTIONTEXT: usize = 32;

/// Raven `cg_t` — cgame per-frame local state, the "everything" struct.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:297-503`
#[repr(C)]
pub struct cg_t {
    /// incremented each frame
    pub clientFrame: i32,

    /// taking a level menu screenshot
    pub levelShot: qboolean,

    // there are only one or two snapshot_t that are relevent at a time
    /// the number of snapshots the client system has received
    pub latestSnapshotNum: i32,
    /// the time from latestSnapshotNum, so we don't need to read the snapshot yet
    pub latestSnapshotTime: i32,
    /// the number of snapshots cgame has requested
    pub processedSnapshotNum: i32,
    /// cg.snap->serverTime <= cg.time
    pub snap: *mut snapshot_t,
    /// cg.nextSnap->serverTime > cg.time, or NULL
    pub nextSnap: *mut snapshot_t,

    /// (float)( cg.time - cg.frame->serverTime ) / (cg.nextFrame->serverTime - cg.frame->serverTime)
    pub frameInterpolation: f32,

    pub thisFrameTeleport: qboolean,
    pub nextFrameTeleport: qboolean,

    /// cg.time - cg.oldTime
    pub frametime: i32,

    /// this is the time value that the client
    /// is rendering at.
    pub time: i32,
    /// time at last frame, used for missile trails and prediction checking
    pub oldTime: i32,

    /// 5 min, 1 min, overtime
    pub timelimitWarnings: i32,

    /// during deaths, chasecams, etc
    pub renderingThirdPerson: qboolean,

    // prediction state
    /// true if prediction has hit a trigger_teleport
    pub hyperspace: qboolean,
    pub predicted_player_state: playerState_t,
    /// clear until the first call to CG_PredictPlayerState
    pub validPPS: qboolean,
    pub predictedErrorTime: i32,
    pub predictedError: vec3_t,

    /// for stair up smoothing
    pub stepChange: f32,
    pub stepTime: i32,

    /// for duck viewheight smoothing
    pub duckChange: f32,
    pub duckTime: i32,

    /// for landing hard
    pub landChange: f32,
    pub landTime: i32,

    // input state sent to server
    pub weaponSelect: i32,
    pub saberAnimLevelPending: i32,

    // auto rotating items
    pub autoAngles: vec3_t,
    pub autoAxis: [vec3_t; 3],
    pub autoAnglesFast: vec3_t,
    pub autoAxisFast: [vec3_t; 3],

    // view rendering
    pub refdef: refdef_t,
    /// will be converted to refdef.viewaxis
    pub refdefViewAngles: vec3_t,

    // zoom key
    /// 0 - not zoomed, 1 - binoculars, 2 - disruptor weapon
    pub zoomMode: i32,
    /// -1, 1
    pub zoomDir: i32,
    pub zoomTime: i32,
    pub zoomLocked: qboolean,

    // gonk use
    pub batteryChargeTime: i32,

    // FIXME:
    pub forceCrosshairStartTime: i32,
    pub forceCrosshairEndTime: i32,

    // information screen text during loading
    pub infoScreenText: [c_char; MAX_STRING_CHARS],

    // centerprinting
    pub centerPrintTime: i32,
    pub centerPrintY: i32,
    pub centerPrint: [c_char; 1024],
    pub centerPrintLines: i32,

    // Scrolling text, caption text and LCARS text use this
    pub printText: [[c_char; 128]; MAX_PRINTTEXT],
    pub printTextY: i32,

    /// bosted for taiwanese squealy radio static speech in kejim post
    pub captionText: [[c_char; 256]; MAX_CAPTIONTEXT],
    pub captionTextY: i32,

    /// Number of lines being printed
    pub scrollTextLines: i32,
    pub scrollTextTime: i32,

    pub captionNextTextTime: i32,
    pub captionTextCurrentLine: i32,
    pub captionTextTime: i32,
    pub captionLetterTime: i32,

    // For flashing health armor counter
    pub oldhealth: i32,
    pub oldHealthTime: i32,
    pub oldarmor: i32,
    pub oldArmorTime: i32,
    pub oldammo: i32,
    pub oldAmmoTime: i32,

    /// 1 = low, 2 = empty
    pub lowAmmoWarning: i32,

    // crosshair client ID
    /// who you're looking at
    pub crosshairClientNum: i32,
    /// last time you looked at them
    pub crosshairClientTime: i32,

    // powerup active flashing
    pub powerupActive: i32,
    pub powerupTime: i32,

    //==========================
    pub creditsStart: i32,

    pub itemPickup: i32,
    pub itemPickupTime: i32,
    /// the pulse around the crosshair is timed seperately
    pub itemPickupBlendTime: i32,

    /// How far into opening sequence the icon HUD is
    pub iconHUDPercent: f32,
    /// How long the Icon HUD has been active
    pub iconSelectTime: i32,
    pub iconHUDActive: qboolean,

    /// Current inventory item chosen on Data Pad
    pub DataPadInventorySelect: i32,
    /// Current weapon item chosen on Data Pad
    pub DataPadWeaponSelect: i32,
    /// Current force power chosen on Data Pad
    pub DataPadforcepowerSelect: i32,

    /// Flag to show of message lite is active
    pub messageLitActive: qboolean,

    pub weaponSelectTime: i32,
    pub weaponAnimation: i32,
    pub weaponAnimationTime: i32,

    /// Current inventory item chosen
    pub inventorySelect: i32,
    pub inventorySelectTime: i32,

    /// Current force power chosen
    pub forcepowerSelect: i32,
    pub forcepowerSelectTime: i32,

    // blend blobs
    pub damageTime: f32,
    pub damageX: f32,
    pub damageY: f32,
    pub damageValue: f32,

    // status bar head
    pub headYaw: f32,
    pub headEndPitch: f32,
    pub headEndYaw: f32,
    pub headEndTime: i32,
    pub headStartPitch: f32,
    pub headStartYaw: f32,
    pub headStartTime: i32,

    pub loadLCARSStage: i32,

    pub missionInfoFlashTime: i32,
    pub missionStatusShow: qboolean,
    pub missionStatusDeadTime: i32,

    pub forceHUDTotalFlashTime: i32,
    pub forceHUDNextFlashTime: i32,
    /// Flag to show force hud is off/on
    pub forceHUDActive: qboolean,

    /// qtrue if opened
    pub missionFailedScreen: qboolean,

    pub weaponPickupTextTime: i32,

    pub VHUDFlashTime: i32,
    pub VHUDTurboFlag: qboolean,
    pub HUDTickFlashTime: i32,
    pub HUDArmorFlag: qboolean,
    pub HUDHealthFlag: qboolean,

    // view movement
    pub v_dmg_time: f32,
    pub v_dmg_pitch: f32,
    pub v_dmg_roll: f32,

    /// when interrogator gets you, wonky time controls "drugged" camera view.
    pub wonkyTime: i32,

    /// weapon kicks
    pub kick_angles: vec3_t,
    /// when the kick happened, so it gets reduced over time
    pub kick_time: i32,

    // temp working variables for player view
    pub bobfracsin: f32,
    pub bobcycle: i32,
    pub xyspeed: f32,

    // development tool
    pub testModelName: [c_char; MAX_QPATH],
    // Ghoul2 Insert Start
    pub testModel: i32,
    /// had to be moved so we wouldn't wipe these out with the memset - these have STL in them and shouldn't be cleared that way
    pub activeSnapshots: [snapshot_t; 2],
    pub testModelEntity: refEntity_t,
    // Ghoul2 Insert End
    /// for overriding certain third-person camera properties
    pub overrides: overrides_t,
}

const _: () = assert!(core::mem::size_of::<cg_t>() == 321248);
const _: () = assert!(core::mem::offset_of!(cg_t, clientFrame) == 0);
const _: () = assert!(core::mem::offset_of!(cg_t, levelShot) == 4);
const _: () = assert!(core::mem::offset_of!(cg_t, latestSnapshotNum) == 8);
const _: () = assert!(core::mem::offset_of!(cg_t, latestSnapshotTime) == 12);
const _: () = assert!(core::mem::offset_of!(cg_t, processedSnapshotNum) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, snap) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, nextSnap) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, frameInterpolation) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, thisFrameTeleport) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, nextFrameTeleport) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, frametime) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, time) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldTime) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, timelimitWarnings) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, renderingThirdPerson) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, hyperspace) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predicted_player_state) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, validPPS) == 5072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedErrorTime) == 5076);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedError) == 5080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, stepChange) == 5092);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, stepTime) == 5096);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, duckChange) == 5100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, duckTime) == 5104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, landChange) == 5108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, landTime) == 5112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponSelect) == 5116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, saberAnimLevelPending) == 5120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAngles) == 5124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAxis) == 5136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAnglesFast) == 5172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAxisFast) == 5184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, refdef) == 5220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, refdefViewAngles) == 5336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomMode) == 5348);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomDir) == 5352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomTime) == 5356);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomLocked) == 5360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, batteryChargeTime) == 5364);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceCrosshairStartTime) == 5368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceCrosshairEndTime) == 5372);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, infoScreenText) == 5376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintTime) == 6400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintY) == 6404);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrint) == 6408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintLines) == 7432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, printText) == 7436);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, printTextY) == 23820);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionText) == 23824);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionTextY) == 32016);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scrollTextLines) == 32020);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scrollTextTime) == 32024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionNextTextTime) == 32028);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionTextCurrentLine) == 32032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionTextTime) == 32036);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, captionLetterTime) == 32040);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldhealth) == 32044);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldHealthTime) == 32048);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldarmor) == 32052);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldArmorTime) == 32056);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldammo) == 32060);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldAmmoTime) == 32064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, lowAmmoWarning) == 32068);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairClientNum) == 32072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairClientTime) == 32076);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, powerupActive) == 32080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, powerupTime) == 32084);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, creditsStart) == 32088);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickup) == 32092);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickupTime) == 32096);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickupBlendTime) == 32100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconHUDPercent) == 32104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconSelectTime) == 32108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconHUDActive) == 32112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, DataPadInventorySelect) == 32116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, DataPadWeaponSelect) == 32120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, DataPadforcepowerSelect) == 32124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, messageLitActive) == 32128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponSelectTime) == 32132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponAnimation) == 32136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponAnimationTime) == 32140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, inventorySelect) == 32144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, inventorySelectTime) == 32148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forcepowerSelect) == 32152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forcepowerSelectTime) == 32156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageTime) == 32160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageX) == 32164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageY) == 32168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageValue) == 32172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headYaw) == 32176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndPitch) == 32180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndYaw) == 32184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndTime) == 32188);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartPitch) == 32192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartYaw) == 32196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartTime) == 32200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, loadLCARSStage) == 32204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, missionInfoFlashTime) == 32208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, missionStatusShow) == 32212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, missionStatusDeadTime) == 32216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDTotalFlashTime) == 32220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDNextFlashTime) == 32224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDActive) == 32228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, missionFailedScreen) == 32232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponPickupTextTime) == 32236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, VHUDFlashTime) == 32240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, VHUDTurboFlag) == 32244);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDTickFlashTime) == 32248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDArmorFlag) == 32252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDHealthFlag) == 32256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_time) == 32260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_pitch) == 32264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_roll) == 32268);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, wonkyTime) == 32272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, kick_angles) == 32276);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, kick_time) == 32288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bobfracsin) == 32292);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bobcycle) == 32296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, xyspeed) == 32300);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModelName) == 32304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModel) == 32368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, activeSnapshots) == 32376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModelEntity) == 321032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, overrides) == 321208);
