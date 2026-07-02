#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::qcommon::playerState_t;
use mp_qshared::shared::{qboolean, qhandle_t, vec3_t, MAX_CLIENTS, MAX_QPATH, MAX_STRING_CHARS};

use super::chat_box_item_s::chatBoxItem_t;
use super::score_t::score_t;
use super::skulltrail_t::skulltrail_t;

/// Raven `MAX_REWARDSTACK`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_local.h:736`
pub const MAX_REWARDSTACK: usize = 10;

/// Raven `MAX_SOUNDBUFFER`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_local.h:737`
pub const MAX_SOUNDBUFFER: usize = 20;

/// Raven `MAX_PREDICTED_EVENTS`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_local.h:744`
pub const MAX_PREDICTED_EVENTS: usize = 16;

/// Raven `MAX_CHATBOX_ITEMS`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_local.h:747`
pub const MAX_CHATBOX_ITEMS: usize = 5;

/// Raven `MAX_NAME_LENGTH`.
///
/// Raven: max length of a client name.
/// Source: `oracle/oracle/codemp/game/q_shared.h:400`
pub const MAX_NAME_LENGTH: usize = 32;

/// Raven `MAX_CG_SHARED_BUFFER_SIZE`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:593`
pub const MAX_CG_SHARED_BUFFER_SIZE: usize = 2048;

/// Raven `cg_t` — cgame per-frame local state, the "everything" struct.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:755-1014`
#[repr(C)]
pub struct cg_t {
	/// incremented each frame
	pub clientFrame: i32,

	pub clientNum: i32,

	pub demoPlayback: qboolean,
	/// taking a level menu screenshot
	pub levelShot: qboolean,
	pub deferredPlayerLoading: i32,
	/// don't defer players at initial startup
	pub loading: qboolean,
	/// don't play voice rewards, because game will end shortly
	pub intermissionStarted: qboolean,

	// there are only one or two snapshot_t that are relevent at a time
	/// the number of snapshots the client system has received
	pub latestSnapshotNum: i32,
	/// the time from latestSnapshotNum, so we don't need to read the snapshot yet
	pub latestSnapshotTime: i32,

	/// cg.snap->serverTime <= cg.time
	pub snap: *mut snapshot_t,
	/// cg.nextSnap->serverTime > cg.time, or NULL
	pub nextSnap: *mut snapshot_t,

	/// (float)( cg.time - cg.frame->serverTime ) / (cg.nextFrame->serverTime - cg.frame->serverTime)
	pub frameInterpolation: f32,

	pub mMapChange: qboolean,

	pub thisFrameTeleport: qboolean,
	pub nextFrameTeleport: qboolean,

	/// cg.time - cg.oldTime
	pub frametime: i32,

	/// this is the time value that the client
	/// is rendering at.
	pub time: i32,
	/// time at last frame, used for missile trails and prediction checking
	pub oldTime: i32,

	/// either cg.snap->time or cg.nextSnap->time
	pub physicsTime: i32,

	/// 5 min, 1 min, overtime
	pub timelimitWarnings: i32,
	pub fraglimitWarnings: i32,

	/// set on a map restart to set back the weapon
	pub mapRestart: qboolean,

	/// rwwRMG - added
	pub mInRMG: qboolean,
	/// rwwRMG - added
	pub mRMGWeather: qboolean,

	/// during deaths, chasecams, etc
	pub renderingThirdPerson: qboolean,

	// prediction state
	/// true if prediction has hit a trigger_teleport
	pub hyperspace: qboolean,
	pub predictedPlayerState: playerState_t,
	pub predictedVehicleState: playerState_t,

	// rww - I removed centity_t predictedPlayerEntity and made it use cg_entities[clnum] directly.

	/// clear until the first call to CG_PredictPlayerState
	pub validPPS: qboolean,
	pub predictedErrorTime: i32,
	pub predictedError: vec3_t,

	pub eventSequence: i32,
	pub predictableEvents: [i32; MAX_PREDICTED_EVENTS],

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

	pub forceSelect: i32,
	pub itemSelect: i32,

	// auto rotating items
	pub autoAngles: vec3_t,
	pub autoAxis: [vec3_t; 3],
	pub autoAnglesFast: vec3_t,
	pub autoAxisFast: [vec3_t; 3],

	// view rendering
	pub refdef: refdef_t,

	// zoom key
	pub zoomed: qboolean,
	pub zoomTime: i32,
	pub zoomSensitivity: f32,

	// information screen text during loading
	pub infoScreenText: [c_char; MAX_STRING_CHARS],

	// scoreboard
	pub scoresRequestTime: i32,
	pub numScores: i32,
	pub selectedScore: i32,
	pub teamScores: [i32; 2],
	pub scores: [score_t; MAX_CLIENTS],
	pub showScores: qboolean,
	pub scoreBoardShowing: qboolean,
	pub scoreFadeTime: i32,
	pub killerName: [c_char; MAX_NAME_LENGTH],
	/// list of names
	pub spectatorList: [c_char; MAX_STRING_CHARS],
	/// length of list
	pub spectatorLen: i32,
	/// width in device units
	pub spectatorWidth: f32,
	/// next time to offset
	pub spectatorTime: i32,
	/// current paint x
	pub spectatorPaintX: i32,
	/// current paint x
	pub spectatorPaintX2: i32,
	/// current offset from start
	pub spectatorOffset: i32,
	/// current offset from start
	pub spectatorPaintLen: i32,

	// skull trails
	pub skulltrails: [skulltrail_t; MAX_CLIENTS],

	// centerprinting
	pub centerPrintTime: i32,
	pub centerPrintCharWidth: i32,
	pub centerPrintY: i32,
	pub centerPrint: [c_char; 1024],
	pub centerPrintLines: i32,

	/// 1 = low, 2 = empty
	pub lowAmmoWarning: i32,

	// kill timers for carnage reward
	pub lastKillTime: i32,

	// crosshair client ID
	pub crosshairClientNum: i32,
	pub crosshairClientTime: i32,

	pub crosshairVehNum: i32,
	pub crosshairVehTime: i32,

	// powerup active flashing
	pub powerupActive: i32,
	pub powerupTime: i32,

	// attacking player
	pub attackerTime: i32,
	pub voiceTime: i32,

	// reward medals
	pub rewardStack: i32,
	pub rewardTime: i32,
	pub rewardCount: [i32; MAX_REWARDSTACK],
	pub rewardShader: [qhandle_t; MAX_REWARDSTACK],
	pub rewardSound: [qhandle_t; MAX_REWARDSTACK],

	// sound buffer mainly for announcer sounds
	pub soundBufferIn: i32,
	pub soundBufferOut: i32,
	pub soundTime: i32,
	pub soundBuffer: [qhandle_t; MAX_SOUNDBUFFER],

	// for voice chat buffer
	pub voiceChatTime: i32,
	pub voiceChatBufferIn: i32,
	pub voiceChatBufferOut: i32,

	// warmup countdown
	pub warmup: i32,
	pub warmupCount: i32,

	pub itemPickup: i32,
	pub itemPickupTime: i32,
	/// the pulse around the crosshair is timed seperately
	pub itemPickupBlendTime: i32,

	pub weaponSelectTime: i32,
	pub weaponAnimation: i32,
	pub weaponAnimationTime: i32,

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

	// view movement
	pub v_dmg_time: f32,
	pub v_dmg_pitch: f32,
	pub v_dmg_roll: f32,

	/// weapon kicks
	pub kick_angles: vec3_t,
	pub kick_time: i32,
	pub kick_origin: vec3_t,

	// temp working variables for player view
	pub bobfracsin: f32,
	pub bobcycle: i32,
	pub xyspeed: f32,
	pub nextOrbitTime: i32,

	// qboolean cameraMode; // if rendering from a loaded camera
	pub loadLCARSStage: i32,

	pub forceHUDTotalFlashTime: i32,
	pub forceHUDNextFlashTime: i32,
	/// Flag to show force hud is off/on
	pub forceHUDActive: qboolean,

	// development tool
	pub testModelEntity: refEntity_t,
	pub testModelName: [c_char; MAX_QPATH],
	pub testGun: qboolean,

	pub VHUDFlashTime: i32,
	pub VHUDTurboFlag: qboolean,

	// HUD stuff
	pub HUDTickFlashTime: f32,
	pub HUDArmorFlag: qboolean,
	pub HUDHealthFlag: qboolean,
	pub iconHUDActive: qboolean,
	pub iconHUDPercent: f32,
	pub iconSelectTime: f32,
	pub invenSelectTime: f32,
	pub forceSelectTime: f32,

	pub lastFPFlashPoint: vec3_t,

	// Ghoul2 Insert Start
	pub testModel: i32,
	/// had to be moved so we wouldn't wipe these out with the memset - these have STL in them and shouldn't be cleared that way
	pub activeSnapshots: [snapshot_t; 2],
	// Ghoul2 Insert End

	pub sharedBuffer: [c_char; MAX_CG_SHARED_BUFFER_SIZE],

	pub radarEntityCount: i16,
	pub radarEntities: [i16; MAX_CLIENTS + 16],

	pub bracketedEntityCount: i16,
	pub bracketedEntities: [i16; MAX_CLIENTS + 16],

	pub distanceCull: f32,

	pub chatItems: [chatBoxItem_t; MAX_CHATBOX_ITEMS],
	pub chatItemActive: i32,
}

const _: () = assert!(core::mem::size_of::<cg_t>() == 295424);
const _: () = assert!(core::mem::offset_of!(cg_t, clientFrame) == 0);
const _: () = assert!(core::mem::offset_of!(cg_t, clientNum) == 4);
const _: () = assert!(core::mem::offset_of!(cg_t, demoPlayback) == 8);
const _: () = assert!(core::mem::offset_of!(cg_t, levelShot) == 12);
const _: () = assert!(core::mem::offset_of!(cg_t, deferredPlayerLoading) == 16);
const _: () = assert!(core::mem::offset_of!(cg_t, loading) == 20);
const _: () = assert!(core::mem::offset_of!(cg_t, intermissionStarted) == 24);
const _: () = assert!(core::mem::offset_of!(cg_t, latestSnapshotNum) == 28);
const _: () = assert!(core::mem::offset_of!(cg_t, latestSnapshotTime) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, snap) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, nextSnap) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, frameInterpolation) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, mMapChange) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, thisFrameTeleport) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, nextFrameTeleport) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, frametime) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, time) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, oldTime) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, physicsTime) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, timelimitWarnings) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, fraglimitWarnings) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, mapRestart) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, mInRMG) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, mRMGWeather) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, renderingThirdPerson) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, hyperspace) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedPlayerState) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedVehicleState) == 1668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, validPPS) == 3220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedErrorTime) == 3224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictedError) == 3228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, eventSequence) == 3240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, predictableEvents) == 3244);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, stepChange) == 3308);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, stepTime) == 3312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, duckChange) == 3316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, duckTime) == 3320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, landChange) == 3324);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, landTime) == 3328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponSelect) == 3332);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceSelect) == 3336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemSelect) == 3340);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAngles) == 3344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAxis) == 3356);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAnglesFast) == 3392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, autoAxisFast) == 3404);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, refdef) == 3440);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomed) == 3824);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomTime) == 3828);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, zoomSensitivity) == 3832);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, infoScreenText) == 3836);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scoresRequestTime) == 4860);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, numScores) == 4864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, selectedScore) == 4868);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, teamScores) == 4872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scores) == 4880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, showScores) == 6800);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scoreBoardShowing) == 6804);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, scoreFadeTime) == 6808);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, killerName) == 6812);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorList) == 6844);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorLen) == 7868);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorWidth) == 7872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorTime) == 7876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorPaintX) == 7880);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorPaintX2) == 7884);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorOffset) == 7888);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, spectatorPaintLen) == 7892);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, skulltrails) == 7896);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintTime) == 11864);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintCharWidth) == 11868);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintY) == 11872);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrint) == 11876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, centerPrintLines) == 12900);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, lowAmmoWarning) == 12904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, lastKillTime) == 12908);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairClientNum) == 12912);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairClientTime) == 12916);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairVehNum) == 12920);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, crosshairVehTime) == 12924);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, powerupActive) == 12928);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, powerupTime) == 12932);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, attackerTime) == 12936);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, voiceTime) == 12940);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, rewardStack) == 12944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, rewardTime) == 12948);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, rewardCount) == 12952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, rewardShader) == 12992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, rewardSound) == 13032);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, soundBufferIn) == 13072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, soundBufferOut) == 13076);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, soundTime) == 13080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, soundBuffer) == 13084);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, voiceChatTime) == 13164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, voiceChatBufferIn) == 13168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, voiceChatBufferOut) == 13172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, warmup) == 13176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, warmupCount) == 13180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickup) == 13184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickupTime) == 13188);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, itemPickupBlendTime) == 13192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponSelectTime) == 13196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponAnimation) == 13200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, weaponAnimationTime) == 13204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageTime) == 13208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageX) == 13212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageY) == 13216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, damageValue) == 13220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headYaw) == 13224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndPitch) == 13228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndYaw) == 13232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headEndTime) == 13236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartPitch) == 13240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartYaw) == 13244);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, headStartTime) == 13248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_time) == 13252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_pitch) == 13256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, v_dmg_roll) == 13260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, kick_angles) == 13264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, kick_time) == 13276);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, kick_origin) == 13280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bobfracsin) == 13292);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bobcycle) == 13296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, xyspeed) == 13300);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, nextOrbitTime) == 13304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, loadLCARSStage) == 13308);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDTotalFlashTime) == 13312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDNextFlashTime) == 13316);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceHUDActive) == 13320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModelEntity) == 13328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModelName) == 13544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testGun) == 13608);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, VHUDFlashTime) == 13612);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, VHUDTurboFlag) == 13616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDTickFlashTime) == 13620);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDArmorFlag) == 13624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, HUDHealthFlag) == 13628);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconHUDActive) == 13632);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconHUDPercent) == 13636);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, iconSelectTime) == 13640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, invenSelectTime) == 13644);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, forceSelectTime) == 13648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, lastFPFlashPoint) == 13652);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, testModel) == 13664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, activeSnapshots) == 13668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, sharedBuffer) == 292372);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, radarEntityCount) == 294420);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, radarEntities) == 294422);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bracketedEntityCount) == 294518);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, bracketedEntities) == 294520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, distanceCull) == 294616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, chatItems) == 294620);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cg_t, chatItemActive) == 295420);
