#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::holdable::HI_NUM_HOLDABLE;
use mp_bg::weapons::weapon_t::WP_NUM_WEAPONS;
use mp_qshared::shared::{qhandle_t, sfxHandle_t, NUM_FORCE_POWERS};
use mp_uishared::shared::cached_assets_t::NUM_CROSSHAIRS;

use super::footstep_t::footstep_t;

/// Raven `cgMedia_t` — all of the model, shader, and sound references that are
/// loaded at gamestate time.
///
/// Raven: Other media that can be tied to clients, weapons, or items are
/// stored in the clientInfo_t, itemInfo_t, weaponInfo_t, and powerupInfo_t.
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1067-1380`
#[repr(C)]
pub struct cgMedia_t {
    pub charsetShader: qhandle_t,
    pub whiteShader: qhandle_t,

    pub loadBarLED: qhandle_t,
    pub loadBarLEDCap: qhandle_t,
    pub loadBarLEDSurround: qhandle_t,

    pub bryarFrontFlash: qhandle_t,
    pub greenFrontFlash: qhandle_t,
    pub lightningFlash: qhandle_t,

    pub itemHoloModel: qhandle_t,
    pub redFlagModel: qhandle_t,
    pub blueFlagModel: qhandle_t,

    pub flagPoleModel: qhandle_t,
    pub flagFlapModel: qhandle_t,

    pub redFlagBaseModel: qhandle_t,
    pub blueFlagBaseModel: qhandle_t,
    pub neutralFlagBaseModel: qhandle_t,

    pub teamStatusBar: qhandle_t,

    pub deferShader: qhandle_t,

    pub radarShader: qhandle_t,
    pub siegeItemShader: qhandle_t,
    pub mAutomapPlayerIcon: qhandle_t,
    pub mAutomapRocketIcon: qhandle_t,

    pub wireframeAutomapFrame_left: qhandle_t,
    pub wireframeAutomapFrame_right: qhandle_t,
    pub wireframeAutomapFrame_top: qhandle_t,
    pub wireframeAutomapFrame_bottom: qhandle_t,

    // Chunks
    // `[NUM_CHUNK_TYPES][4]` — NUM_CHUNK_TYPES is the 8-entry anonymous enum
    // terminator at oracle/codemp/cgame/cg_local.h:1049-1059 (not
    // separately ported; used here only as this array bound).
    pub chunkModels: [[qhandle_t; 4]; 8],
    pub chunkSound: sfxHandle_t,
    pub grateSound: sfxHandle_t,
    pub rockBreakSound: sfxHandle_t,
    pub rockBounceSound: [sfxHandle_t; 2],
    pub metalBounceSound: [sfxHandle_t; 2],
    pub glassChunkSound: sfxHandle_t,
    pub crateBreakSound: [sfxHandle_t; 2],

    pub hackerIconShader: qhandle_t,

    // Saber shaders
    //-----------------------------
    pub forceCoronaShader: qhandle_t,

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
    pub saberBlurShader: qhandle_t,
    pub swordTrailShader: qhandle_t,

    pub yellowDroppedSaberShader: qhandle_t,

    pub rivetMarkShader: qhandle_t,

    pub teamRedShader: qhandle_t,
    pub teamBlueShader: qhandle_t,

    pub powerDuelAllyShader: qhandle_t,

    pub balloonShader: qhandle_t,
    pub vchatShader: qhandle_t,
    pub connectionShader: qhandle_t,

    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
    pub lagometerShader: qhandle_t,
    pub backTileShader: qhandle_t,

    pub numberShaders: [qhandle_t; 11],
    pub smallnumberShaders: [qhandle_t; 11],
    pub chunkyNumberShaders: [qhandle_t; 11],

    pub electricBodyShader: qhandle_t,
    pub electricBody2Shader: qhandle_t,

    pub fsrMarkShader: qhandle_t,
    pub fslMarkShader: qhandle_t,
    pub fshrMarkShader: qhandle_t,
    pub fshlMarkShader: qhandle_t,

    pub refractionShader: qhandle_t,

    pub cloakedShader: qhandle_t,

    pub boltShader: qhandle_t,

    pub shadowMarkShader: qhandle_t,

    // glass shard shader
    pub glassShardShader: qhandle_t,

    // wall mark shaders
    pub wakeMarkShader: qhandle_t,

    // Pain view shader
    pub viewPainShader: qhandle_t,
    pub viewPainShader_Shields: qhandle_t,
    pub viewPainShader_ShieldsAndHealth: qhandle_t,

    pub itemRespawningPlaceholder: qhandle_t,
    pub itemRespawningRezOut: qhandle_t,

    pub playerShieldDamage: qhandle_t,
    pub protectShader: qhandle_t,
    pub forceSightBubble: qhandle_t,
    pub forceShell: qhandle_t,
    pub sightShell: qhandle_t,

    // Disruptor zoom graphics
    pub disruptorMask: qhandle_t,
    pub disruptorInsert: qhandle_t,
    pub disruptorLight: qhandle_t,
    pub disruptorInsertTick: qhandle_t,
    pub disruptorChargeShader: qhandle_t,

    // Binocular graphics
    pub binocularCircle: qhandle_t,
    pub binocularMask: qhandle_t,
    pub binocularArrow: qhandle_t,
    pub binocularTri: qhandle_t,
    pub binocularStatic: qhandle_t,
    pub binocularOverlay: qhandle_t,

    // weapon effect models
    pub lightningExplosionModel: qhandle_t,

    // explosion assets
    pub explosionModel: qhandle_t,
    pub surfaceExplosionShader: qhandle_t,

    pub disruptorShader: qhandle_t,

    pub solidWhite: qhandle_t,

    pub heartShader: qhandle_t,

    // All the player shells
    pub ysaliredShader: qhandle_t,
    pub ysaliblueShader: qhandle_t,
    pub ysalimariShader: qhandle_t,
    pub boonShader: qhandle_t,
    pub endarkenmentShader: qhandle_t,
    pub enlightenmentShader: qhandle_t,
    pub invulnerabilityShader: qhandle_t,

    // sounds
    pub selectSound: sfxHandle_t,
    pub footsteps: [[sfxHandle_t; 4]; footstep_t::FOOTSTEP_TOTAL as usize],

    pub winnerSound: sfxHandle_t,
    pub loserSound: sfxHandle_t,

    pub crackleSound: sfxHandle_t,

    pub grenadeBounce1: sfxHandle_t,
    pub grenadeBounce2: sfxHandle_t,

    pub teamHealSound: sfxHandle_t,
    pub teamRegenSound: sfxHandle_t,

    pub teleInSound: sfxHandle_t,
    pub teleOutSound: sfxHandle_t,
    pub respawnSound: sfxHandle_t,
    pub talkSound: sfxHandle_t,
    pub landSound: sfxHandle_t,
    pub fallSound: sfxHandle_t,

    pub oneMinuteSound: sfxHandle_t,
    pub fiveMinuteSound: sfxHandle_t,

    pub threeFragSound: sfxHandle_t,
    pub twoFragSound: sfxHandle_t,
    pub oneFragSound: sfxHandle_t,

    pub rollSound: sfxHandle_t,

    pub watrInSound: sfxHandle_t,
    pub watrOutSound: sfxHandle_t,
    pub watrUnSound: sfxHandle_t,

    pub noforceSound: sfxHandle_t,

    pub deploySeeker: sfxHandle_t,
    pub medkitSound: sfxHandle_t,

    // teamplay sounds
    pub redScoredSound: sfxHandle_t,
    pub blueScoredSound: sfxHandle_t,
    pub redLeadsSound: sfxHandle_t,
    pub blueLeadsSound: sfxHandle_t,
    pub teamsTiedSound: sfxHandle_t,

    pub redFlagReturnedSound: sfxHandle_t,
    pub blueFlagReturnedSound: sfxHandle_t,
    pub redTookFlagSound: sfxHandle_t,
    pub blueTookFlagSound: sfxHandle_t,

    pub redYsalReturnedSound: sfxHandle_t,
    pub blueYsalReturnedSound: sfxHandle_t,
    pub redTookYsalSound: sfxHandle_t,
    pub blueTookYsalSound: sfxHandle_t,

    pub drainSound: sfxHandle_t,

    // music blips
    pub happyMusic: sfxHandle_t,
    pub dramaticFailure: sfxHandle_t,

    // tournament sounds
    pub count3Sound: sfxHandle_t,
    pub count2Sound: sfxHandle_t,
    pub count1Sound: sfxHandle_t,
    pub countFightSound: sfxHandle_t,

    // new stuff
    pub patrolShader: qhandle_t,
    pub assaultShader: qhandle_t,
    pub campShader: qhandle_t,
    pub followShader: qhandle_t,
    pub defendShader: qhandle_t,
    pub teamLeaderShader: qhandle_t,
    pub retrieveShader: qhandle_t,
    pub escortShader: qhandle_t,
    pub flagShaders: [qhandle_t; 3],

    pub halfShieldModel: qhandle_t,
    pub halfShieldShader: qhandle_t,

    pub demp2Shell: qhandle_t,
    pub demp2ShellShader: qhandle_t,

    pub cursor: qhandle_t,
    pub selectCursor: qhandle_t,
    pub sizeCursor: qhandle_t,

    // weapon icons
    pub weaponIcons: [qhandle_t; WP_NUM_WEAPONS as usize],
    pub weaponIcons_NA: [qhandle_t; WP_NUM_WEAPONS as usize],

    // holdable inventory item icons
    pub invenIcons: [qhandle_t; HI_NUM_HOLDABLE as usize],

    // force power icons
    pub forcePowerIcons: [qhandle_t; NUM_FORCE_POWERS as usize],

    pub rageRecShader: qhandle_t,

    // other HUD parts
    pub currentBackground: c_int,
    pub weaponIconBackground: qhandle_t,
    pub forceIconBackground: qhandle_t,
    pub inventoryIconBackground: qhandle_t,

    pub holocronPickup: sfxHandle_t,

    // Zoom
    pub zoomStart: sfxHandle_t,
    pub zoomLoop: sfxHandle_t,
    pub zoomEnd: sfxHandle_t,
    pub disruptorZoomLoop: sfxHandle_t,

    pub bdecal_bodyburn1: qhandle_t,
    pub bdecal_saberglow: qhandle_t,
    pub bdecal_burn1: qhandle_t,
    pub mSaberDamageGlow: qhandle_t,

    // For vehicles only now
    pub noAmmoSound: sfxHandle_t,
}

const _: () = assert!(core::mem::size_of::<cgMedia_t>() == 1716);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, charsetShader) == 0);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, whiteShader) == 4);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loadBarLED) == 8);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loadBarLEDCap) == 12);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loadBarLEDSurround) == 16);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bryarFrontFlash) == 20);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, greenFrontFlash) == 24);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lightningFlash) == 28);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, itemHoloModel) == 32);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redFlagModel) == 36);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueFlagModel) == 40);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, flagPoleModel) == 44);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, flagFlapModel) == 48);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redFlagBaseModel) == 52);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueFlagBaseModel) == 56);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, neutralFlagBaseModel) == 60);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamStatusBar) == 64);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, deferShader) == 68);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, radarShader) == 72);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, siegeItemShader) == 76);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, mAutomapPlayerIcon) == 80);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, mAutomapRocketIcon) == 84);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wireframeAutomapFrame_left) == 88);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wireframeAutomapFrame_right) == 92);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wireframeAutomapFrame_top) == 96);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wireframeAutomapFrame_bottom) == 100);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkModels) == 104);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkSound) == 232);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grateSound) == 236);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rockBreakSound) == 240);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rockBounceSound) == 244);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, metalBounceSound) == 252);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, glassChunkSound) == 260);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, crateBreakSound) == 264);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, hackerIconShader) == 272);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceCoronaShader) == 276);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redSaberGlowShader) == 280);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redSaberCoreShader) == 284);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, orangeSaberGlowShader) == 288);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, orangeSaberCoreShader) == 292);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowSaberGlowShader) == 296);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowSaberCoreShader) == 300);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, greenSaberGlowShader) == 304);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, greenSaberCoreShader) == 308);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueSaberGlowShader) == 312);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueSaberCoreShader) == 316);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, purpleSaberGlowShader) == 320);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, purpleSaberCoreShader) == 324);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, saberBlurShader) == 328);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, swordTrailShader) == 332);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, yellowDroppedSaberShader) == 336);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rivetMarkShader) == 340);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamRedShader) == 344);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamBlueShader) == 348);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, powerDuelAllyShader) == 352);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, balloonShader) == 356);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, vchatShader) == 360);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, connectionShader) == 364);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, crosshairShader) == 368);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lagometerShader) == 404);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, backTileShader) == 408);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, numberShaders) == 412);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, smallnumberShaders) == 456);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, chunkyNumberShaders) == 500);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, electricBodyShader) == 544);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, electricBody2Shader) == 548);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fsrMarkShader) == 552);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fslMarkShader) == 556);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fshrMarkShader) == 560);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fshlMarkShader) == 564);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, refractionShader) == 568);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, cloakedShader) == 572);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, boltShader) == 576);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, shadowMarkShader) == 580);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, glassShardShader) == 584);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, wakeMarkShader) == 588);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, viewPainShader) == 592);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, viewPainShader_Shields) == 596);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, viewPainShader_ShieldsAndHealth) == 600);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, itemRespawningPlaceholder) == 604);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, itemRespawningRezOut) == 608);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, playerShieldDamage) == 612);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, protectShader) == 616);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceSightBubble) == 620);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceShell) == 624);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, sightShell) == 628);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorMask) == 632);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorInsert) == 636);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorLight) == 640);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorInsertTick) == 644);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorChargeShader) == 648);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularCircle) == 652);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularMask) == 656);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularArrow) == 660);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularTri) == 664);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularStatic) == 668);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, binocularOverlay) == 672);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, lightningExplosionModel) == 676);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, explosionModel) == 680);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, surfaceExplosionShader) == 684);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorShader) == 688);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, solidWhite) == 692);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, heartShader) == 696);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, ysaliredShader) == 700);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, ysaliblueShader) == 704);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, ysalimariShader) == 708);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, boonShader) == 712);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, endarkenmentShader) == 716);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, enlightenmentShader) == 720);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, invulnerabilityShader) == 724);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, selectSound) == 728);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, footsteps) == 732);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, winnerSound) == 1132);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, loserSound) == 1136);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, crackleSound) == 1140);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce1) == 1144);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, grenadeBounce2) == 1148);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamHealSound) == 1152);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamRegenSound) == 1156);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teleInSound) == 1160);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teleOutSound) == 1164);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, respawnSound) == 1168);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, talkSound) == 1172);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, landSound) == 1176);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fallSound) == 1180);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, oneMinuteSound) == 1184);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, fiveMinuteSound) == 1188);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, threeFragSound) == 1192);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, twoFragSound) == 1196);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, oneFragSound) == 1200);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rollSound) == 1204);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrInSound) == 1208);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrOutSound) == 1212);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, watrUnSound) == 1216);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, noforceSound) == 1220);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, deploySeeker) == 1224);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, medkitSound) == 1228);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redScoredSound) == 1232);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueScoredSound) == 1236);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redLeadsSound) == 1240);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueLeadsSound) == 1244);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamsTiedSound) == 1248);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redFlagReturnedSound) == 1252);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueFlagReturnedSound) == 1256);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redTookFlagSound) == 1260);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueTookFlagSound) == 1264);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redYsalReturnedSound) == 1268);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueYsalReturnedSound) == 1272);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, redTookYsalSound) == 1276);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, blueTookYsalSound) == 1280);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, drainSound) == 1284);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, happyMusic) == 1288);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, dramaticFailure) == 1292);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, count3Sound) == 1296);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, count2Sound) == 1300);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, count1Sound) == 1304);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, countFightSound) == 1308);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, patrolShader) == 1312);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, assaultShader) == 1316);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, campShader) == 1320);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, followShader) == 1324);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, defendShader) == 1328);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, teamLeaderShader) == 1332);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, retrieveShader) == 1336);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, escortShader) == 1340);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, flagShaders) == 1344);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, halfShieldModel) == 1356);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, halfShieldShader) == 1360);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, demp2Shell) == 1364);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, demp2ShellShader) == 1368);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, cursor) == 1372);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, selectCursor) == 1376);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, sizeCursor) == 1380);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, weaponIcons) == 1384);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, weaponIcons_NA) == 1460);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, invenIcons) == 1536);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forcePowerIcons) == 1584);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, rageRecShader) == 1656);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, currentBackground) == 1660);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, weaponIconBackground) == 1664);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, forceIconBackground) == 1668);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, inventoryIconBackground) == 1672);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, holocronPickup) == 1676);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomStart) == 1680);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomLoop) == 1684);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, zoomEnd) == 1688);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, disruptorZoomLoop) == 1692);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bdecal_bodyburn1) == 1696);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bdecal_saberglow) == 1700);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, bdecal_burn1) == 1704);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, mSaberDamageGlow) == 1708);
const _: () = assert!(core::mem::offset_of!(cgMedia_t, noAmmoSound) == 1712);
