#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::common::sp::ff::ff_handle_t::ffHandle_t;
use sp_qshared::shared::{qhandle_t, sfxHandle_t};

use super::footstep_t::footstep_t;

/// Raven `cgMedia_t` — all of the model, shader, and sound references that are
/// loaded at gamestate time.
///
/// Type definition source: `oracle/oracle/code/cgame/cg_media.h:96-306`
#[repr(C)]
pub struct cgMedia_t {
    pub charsetShader: qhandle_t,
    pub whiteShader: qhandle_t,

    pub crosshairShader: [qhandle_t; 9],
    pub backTileShader: qhandle_t,
    // Raven: `//\tqhandle_t\tnoammoShader;` — commented out in the oracle.

    pub numberShaders: [qhandle_t; 11],
    pub smallnumberShaders: [qhandle_t; 11],
    pub chunkyNumberShaders: [qhandle_t; 11],

    pub loadTick: qhandle_t,
    pub loadTickCap: qhandle_t,

    // HUD artwork
    pub currentBackground: c_int,
    pub weaponbox: qhandle_t,
    pub weaponIconBackground: qhandle_t,
    pub forceIconBackground: qhandle_t,
    pub inventoryIconBackground: qhandle_t,
    pub turretComputerOverlayShader: qhandle_t,
    pub turretCrossHairShader: qhandle_t,

    // Chunks
    // `[NUM_CHUNK_TYPES][4]` — NUM_CHUNK_TYPES is the 8-entry anonymous enum
    // terminator at oracle/oracle/code/cgame/cg_media.h:36-89 (not separately
    // ported; used here only as this array bound).
    pub chunkModels: [[qhandle_t; 4]; 8],
    pub chunkSound: sfxHandle_t,
    pub grateSound: sfxHandle_t,
    pub rockBreakSound: sfxHandle_t,
    pub rockBounceSound: [sfxHandle_t; 2],
    pub metalBounceSound: [sfxHandle_t; 2],
    pub glassChunkSound: sfxHandle_t,
    pub crateBreakSound: [sfxHandle_t; 2],

    // Saber shaders
    //-----------------------------
    pub forceCoronaShader: qhandle_t,
    pub saberBlurShader: qhandle_t,
    pub swordTrailShader: qhandle_t,
    /// glow
    pub yellowDroppedSaberShader: qhandle_t,

    pub redSaberGlowShader: qhandle_t,
    pub redSaberCoreShader: qhandle_t,
    pub orangeSaberGlowShader: qhandle_t,
    pub orangeSaberCoreShader: qhandle_t,
    pub yellowSaberGlowShader: qhandle_t,
    pub yellowSaberCoreShader: qhandle_t,
    pub greenSaberGlowShader: qhandle_t,
    pub greenSaberCoreShader: qhandle_t,
    pub blueSaberGlowShader: qhandle_t,
    pub blueSaberCoreShader: qhandle_t,
    pub purpleSaberGlowShader: qhandle_t,
    pub purpleSaberCoreShader: qhandle_t,

    pub explosionModel: qhandle_t,
    pub surfaceExplosionShader: qhandle_t,

    pub halfShieldModel: qhandle_t,

    pub solidWhiteShader: qhandle_t,
    pub electricBodyShader: qhandle_t,
    pub electricBody2Shader: qhandle_t,
    pub refractShader: qhandle_t,
    pub boltShader: qhandle_t,

    // Disruptor zoom graphics
    pub disruptorMask: qhandle_t,
    pub disruptorInsert: qhandle_t,
    pub disruptorLight: qhandle_t,
    pub disruptorInsertTick: qhandle_t,

    // Binocular graphics
    pub binocularCircle: qhandle_t,
    pub binocularMask: qhandle_t,
    pub binocularArrow: qhandle_t,
    pub binocularTri: qhandle_t,
    pub binocularStatic: qhandle_t,
    pub binocularOverlay: qhandle_t,

    // LA Goggles graphics
    pub laGogglesStatic: qhandle_t,
    pub laGogglesMask: qhandle_t,
    pub laGogglesSideBit: qhandle_t,
    pub laGogglesBracket: qhandle_t,
    pub laGogglesArrow: qhandle_t,

    // wall mark shaders
    pub scavMarkShader: qhandle_t,
    pub rivetMarkShader: qhandle_t,

    pub shadowMarkShader: qhandle_t,
    pub wakeMarkShader: qhandle_t,
    pub fsrMarkShader: qhandle_t,
    pub fslMarkShader: qhandle_t,
    pub fshrMarkShader: qhandle_t,
    pub fshlMarkShader: qhandle_t,

    pub damageBlendBlobShader: qhandle_t,

    // fonts...
    //
    pub qhFontSmall: qhandle_t,
    pub qhFontMedium: qhandle_t,

    // special effects models / etc.
    pub personalShieldShader: qhandle_t,
    pub cloakedShader: qhandle_t,

    // Interface media
    pub ammoslider: qhandle_t,
    pub emplacedHealthBarShader: qhandle_t,

    pub dataPadFrame: qhandle_t,
    pub DPForcePowerOverlay: qhandle_t,

    pub bdecal_burnmark1: qhandle_t,
    pub bdecal_saberglowmark: qhandle_t,

    pub messageLitOn: qhandle_t,
    pub messageLitOff: qhandle_t,
    pub messageObjCircle: qhandle_t,

    pub batteryChargeShader: qhandle_t,
    pub useableHint: qhandle_t,

    pub levelLoad: qhandle_t,

    // new stuff for Jedi Academy
    // force power icons
    // Raven: `//\tqhandle_t\tforcePowerIcons[NUM_FORCE_POWERS];` — commented
    // out in the oracle.
    pub rageRecShader: qhandle_t,
    pub playerShieldDamage: qhandle_t,
    pub forceSightBubble: qhandle_t,
    pub forceShell: qhandle_t,
    pub sightShell: qhandle_t,
    pub drainShader: qhandle_t,

    // sounds
    pub disintegrateSound: sfxHandle_t,
    pub disintegrate2Sound: sfxHandle_t,
    pub disintegrate3Sound: sfxHandle_t,

    pub grenadeBounce1: sfxHandle_t,
    pub grenadeBounce2: sfxHandle_t,

    pub flechetteStickSound: sfxHandle_t,
    pub detPackStickSound: sfxHandle_t,
    pub tripMineStickSound: sfxHandle_t,

    pub selectSound: sfxHandle_t,
    pub selectSound2: sfxHandle_t,
    pub overchargeSlowSound: sfxHandle_t,
    pub overchargeFastSound: sfxHandle_t,
    pub overchargeLoopSound: sfxHandle_t,
    pub overchargeEndSound: sfxHandle_t,

    // Raven: `//\tsfxHandle_t\tuseNothingSound;` — commented out in the oracle.
    pub footsteps: [[sfxHandle_t; 4]; footstep_t::FOOTSTEP_TOTAL as usize],

    // Raven: `//\tsfxHandle_t talkSound;` — commented out in the oracle.
    pub noAmmoSound: sfxHandle_t,

    pub landSound: sfxHandle_t,
    pub rollSound: sfxHandle_t,
    pub messageLitSound: sfxHandle_t,

    pub batteryChargeSound: sfxHandle_t,

    pub watrInSound: sfxHandle_t,
    pub watrOutSound: sfxHandle_t,
    pub watrUnSound: sfxHandle_t,

    pub lavaInSound: sfxHandle_t,
    pub lavaOutSound: sfxHandle_t,
    pub lavaUnSound: sfxHandle_t,

    pub noforceSound: sfxHandle_t,

    // Zoom
    pub zoomStart: sfxHandle_t,
    pub zoomLoop: sfxHandle_t,
    pub zoomEnd: sfxHandle_t,
    pub disruptorZoomLoop: sfxHandle_t,

    // new stuff for Jedi Academy
    pub drainSound: sfxHandle_t,

    // Raven: `#ifdef _IMMERSION` — force-feedback registration; layout
    // reflects the `_IMMERSION`-enabled build the packet's offsets were
    // captured against.
    pub grenadeBounce1Force: ffHandle_t,
    pub grenadeBounce2Force: ffHandle_t,

    pub selectForce: ffHandle_t,

    pub footstepForces: [[ffHandle_t; 4]; footstep_t::FOOTSTEP_TOTAL as usize],

    pub noAmmoForce: ffHandle_t,

    pub landForce: ffHandle_t,
    pub messageLitForce: ffHandle_t,

    pub watrInForce: ffHandle_t,
    pub watrOutForce: ffHandle_t,
    pub watrUnForce: ffHandle_t,

    pub zoomStartForce: ffHandle_t,
    pub zoomLoopForce: ffHandle_t,
    pub zoomEndForce: ffHandle_t,
    pub disruptorZoomLoopForce: ffHandle_t,
}

const _: () = assert!(core::mem::size_of::<cgMedia_t>() == 1640);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, charsetShader) == 0);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, whiteShader) == 4);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, crosshairShader) == 8);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, backTileShader) == 44);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, numberShaders) == 48);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, smallnumberShaders) == 92);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkyNumberShaders) == 136);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loadTick) == 180);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loadTickCap) == 184);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, currentBackground) == 188);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, weaponbox) == 192);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, weaponIconBackground) == 196);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceIconBackground) == 200);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, inventoryIconBackground) == 204);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, turretComputerOverlayShader) == 208);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, turretCrossHairShader) == 212);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkModels) == 216);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkSound) == 344);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grateSound) == 348);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rockBreakSound) == 352);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rockBounceSound) == 356);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, metalBounceSound) == 364);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, glassChunkSound) == 372);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, crateBreakSound) == 376);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceCoronaShader) == 384);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, saberBlurShader) == 388);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, swordTrailShader) == 392);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowDroppedSaberShader) == 396);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redSaberGlowShader) == 400);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redSaberCoreShader) == 404);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, orangeSaberGlowShader) == 408);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, orangeSaberCoreShader) == 412);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowSaberGlowShader) == 416);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowSaberCoreShader) == 420);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, greenSaberGlowShader) == 424);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, greenSaberCoreShader) == 428);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueSaberGlowShader) == 432);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueSaberCoreShader) == 436);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, purpleSaberGlowShader) == 440);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, purpleSaberCoreShader) == 444);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, explosionModel) == 448);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, surfaceExplosionShader) == 452);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, halfShieldModel) == 456);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, solidWhiteShader) == 460);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, electricBodyShader) == 464);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, electricBody2Shader) == 468);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, refractShader) == 472);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, boltShader) == 476);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorMask) == 480);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorInsert) == 484);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorLight) == 488);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorInsertTick) == 492);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularCircle) == 496);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularMask) == 500);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularArrow) == 504);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularTri) == 508);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularStatic) == 512);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularOverlay) == 516);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, laGogglesStatic) == 520);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, laGogglesMask) == 524);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, laGogglesSideBit) == 528);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, laGogglesBracket) == 532);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, laGogglesArrow) == 536);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, scavMarkShader) == 540);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rivetMarkShader) == 544);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, shadowMarkShader) == 548);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wakeMarkShader) == 552);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fsrMarkShader) == 556);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fslMarkShader) == 560);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fshrMarkShader) == 564);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fshlMarkShader) == 568);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, damageBlendBlobShader) == 572);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, qhFontSmall) == 576);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, qhFontMedium) == 580);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, personalShieldShader) == 584);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, cloakedShader) == 588);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, ammoslider) == 592);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, emplacedHealthBarShader) == 596);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, dataPadFrame) == 600);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, DPForcePowerOverlay) == 604);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bdecal_burnmark1) == 608);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bdecal_saberglowmark) == 612);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, messageLitOn) == 616);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, messageLitOff) == 620);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, messageObjCircle) == 624);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, batteryChargeShader) == 628);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, useableHint) == 632);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, levelLoad) == 636);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rageRecShader) == 640);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, playerShieldDamage) == 644);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceSightBubble) == 648);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceShell) == 652);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, sightShell) == 656);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, drainShader) == 660);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disintegrateSound) == 664);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disintegrate2Sound) == 668);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disintegrate3Sound) == 672);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce1) == 676);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce2) == 680);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, flechetteStickSound) == 684);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, detPackStickSound) == 688);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, tripMineStickSound) == 692);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, selectSound) == 696);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, selectSound2) == 700);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, overchargeSlowSound) == 704);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, overchargeFastSound) == 708);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, overchargeLoopSound) == 712);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, overchargeEndSound) == 716);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, footsteps) == 720);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, noAmmoSound) == 1120);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, landSound) == 1124);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rollSound) == 1128);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, messageLitSound) == 1132);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, batteryChargeSound) == 1136);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrInSound) == 1140);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrOutSound) == 1144);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrUnSound) == 1148);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lavaInSound) == 1152);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lavaOutSound) == 1156);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lavaUnSound) == 1160);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, noforceSound) == 1164);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomStart) == 1168);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomLoop) == 1172);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomEnd) == 1176);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorZoomLoop) == 1180);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, drainSound) == 1184);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce1Force) == 1188);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce2Force) == 1192);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, selectForce) == 1196);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, footstepForces) == 1200);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, noAmmoForce) == 1600);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, landForce) == 1604);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, messageLitForce) == 1608);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrInForce) == 1612);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrOutForce) == 1616);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrUnForce) == 1620);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomStartForce) == 1624);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomLoopForce) == 1628);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomEndForce) == 1632);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorZoomLoopForce) == 1636);
