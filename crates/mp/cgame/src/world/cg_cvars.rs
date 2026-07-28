//! `CgCvars` — Raven's cgame file-scope `vmCvar_t` mirrors as one `CgWorld`
//! sub-struct.
//!
//! Every file-scope `vmCvar_t` cgame declares lands here (§B3: file-scope globals
//! become owned state). 124 are registered from Raven's `cvarTable[]`, whose
//! name/default/flags each field records; the remaining three (`cg_gun_frame`,
//! `cg_pmove_msec`, `g_showDuelHealths`) are declared and read but never
//! registered — Raven leaves them zeroed.
//!
//! PORT-NOTE: field order is Raven's declaration order, not `cvarTable[]` order —
//! `CG_RegisterCvars` walks the table, so the table order lives in that function
//! (§C8), not in this struct.
#![allow(non_snake_case)]

use mp_qshared::shared::vmCvar_t;

/// Raven's cgame cvar handles, one field per declared file-scope `vmCvar_t`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:702-873` (declarations)
/// Source: `oracle/codemp/cgame/cg_main.c:882-1053` (`cvarTable`)
#[derive(Clone, Default)]
pub struct CgCvars {
    /// `"cg_centertime"` — default `"3"`, `CVAR_CHEAT`.
    pub cg_centertime: vmCvar_t,
    /// `"cg_runpitch"` — default `"0.002"`, `CVAR_ARCHIVE`.
    pub cg_runpitch: vmCvar_t,
    /// `"cg_runroll"` — default `"0.005"`, `CVAR_ARCHIVE`.
    pub cg_runroll: vmCvar_t,
    /// `"cg_bobup"` — default `"0.005"`, `CVAR_ARCHIVE`.
    pub cg_bobup: vmCvar_t,
    /// `"cg_bobpitch"` — default `"0.002"`, `CVAR_ARCHIVE`.
    pub cg_bobpitch: vmCvar_t,
    /// `"cg_bobroll"` — default `"0.002"`, `CVAR_ARCHIVE`.
    pub cg_bobroll: vmCvar_t,

    /// `"cg_shadows"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_shadows: vmCvar_t,
    /// `"cg_renderToTextureFX"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_renderToTextureFX: vmCvar_t,
    /// `"cg_drawTimer"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_drawTimer: vmCvar_t,
    /// `"cg_drawFPS"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_drawFPS: vmCvar_t,
    /// `"cg_drawSnapshot"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_drawSnapshot: vmCvar_t,
    /// `"cg_draw3dIcons"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_draw3dIcons: vmCvar_t,
    /// `"cg_drawIcons"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawIcons: vmCvar_t,
    /// `"cg_drawAmmoWarning"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_drawAmmoWarning: vmCvar_t,
    /// `"cg_drawCrosshair"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawCrosshair: vmCvar_t,
    /// `"cg_drawCrosshairNames"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawCrosshairNames: vmCvar_t,
    /// `"cg_drawRadar"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawRadar: vmCvar_t,
    /// `"cg_drawVehLeadIndicator"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawVehLeadIndicator: vmCvar_t,
    /// `"cg_dynamicCrosshair"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_dynamicCrosshair: vmCvar_t,
    /// `"cg_dynamicCrosshairPrecision"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_dynamicCrosshairPrecision: vmCvar_t,
    /// `"cg_drawRewards"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawRewards: vmCvar_t,
    /// `"cg_drawScores"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawScores: vmCvar_t,
    /// `"cg_crosshairSize"` — default `"24"`, `CVAR_ARCHIVE`.
    pub cg_crosshairSize: vmCvar_t,
    /// `"cg_crosshairX"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_crosshairX: vmCvar_t,
    /// `"cg_crosshairY"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_crosshairY: vmCvar_t,
    /// `"cg_crosshairHealth"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_crosshairHealth: vmCvar_t,
    /// `"cg_draw2D"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_draw2D: vmCvar_t,
    /// `"cg_drawStatus"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawStatus: vmCvar_t,
    /// `"cg_animspeed"` — default `"1"`, `CVAR_CHEAT`.
    pub cg_animSpeed: vmCvar_t,
    /// `"cg_debuganim"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_debugAnim: vmCvar_t,
    /// `"cg_debugsaber"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_debugSaber: vmCvar_t,
    /// `"cg_debugposition"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_debugPosition: vmCvar_t,
    /// `"cg_debugevents"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_debugEvents: vmCvar_t,
    /// `"cg_errordecay"` — default `"100"`, `0`.
    pub cg_errorDecay: vmCvar_t,
    /// `"cg_nopredict"` — default `"0"`, `0`.
    pub cg_nopredict: vmCvar_t,
    /// `"cg_noplayeranims"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_noPlayerAnims: vmCvar_t,
    /// `"cg_showmiss"` — default `"0"`, `0`.
    pub cg_showmiss: vmCvar_t,
    /// `"cg_showVehMiss"` — default `"0"`, `0`.
    pub cg_showVehMiss: vmCvar_t,
    /// `"cg_footsteps"` — default `"3"`, `CVAR_ARCHIVE`.
    pub cg_footsteps: vmCvar_t,
    /// `"cg_marks"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_addMarks: vmCvar_t,
    /// `"cg_viewsize"` — default `"100"`, `CVAR_ARCHIVE`.
    pub cg_viewsize: vmCvar_t,
    /// `"cg_drawGun"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawGun: vmCvar_t,
    /// `cg_gun_frame` — declared and read, never registered in `cvarTable`.
    pub cg_gun_frame: vmCvar_t,
    /// `"cg_gunX"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_gun_x: vmCvar_t,
    /// `"cg_gunY"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_gun_y: vmCvar_t,
    /// `"cg_gunZ"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_gun_z: vmCvar_t,
    /// `"cg_autoswitch"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_autoswitch: vmCvar_t,
    /// `"cg_ignore"` — default `"0"`, `0`.
    pub cg_ignore: vmCvar_t,
    /// `"cg_simpleItems"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_simpleItems: vmCvar_t,
    /// `"cg_fov"` — default `"80"`, `CVAR_ARCHIVE`.
    pub cg_fov: vmCvar_t,
    /// `"cg_zoomfov"` — default `"40.0"`, `CVAR_ARCHIVE`.
    pub cg_zoomFov: vmCvar_t,

    /// `"cg_swingAngles"` — default `"1"`, `0`.
    pub cg_swingAngles: vmCvar_t,

    /// `"cg_oldPainSounds"` — default `"0"`, `0`.
    pub cg_oldPainSounds: vmCvar_t,

    /// `"broadsword"` — default `"0"`, `0`.
    pub cg_ragDoll: vmCvar_t,

    /// `"cg_jumpSounds"` — default `"0"`, `0`.
    pub cg_jumpSounds: vmCvar_t,

    /// `"r_autoMap"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_autoMap: vmCvar_t,
    /// `"r_autoMapX"` — default `"496"`, `CVAR_ARCHIVE`.
    pub cg_autoMapX: vmCvar_t,
    /// `"r_autoMapY"` — default `"32"`, `CVAR_ARCHIVE`.
    pub cg_autoMapY: vmCvar_t,
    /// `"r_autoMapW"` — default `"128"`, `CVAR_ARCHIVE`.
    pub cg_autoMapW: vmCvar_t,
    /// `"r_autoMapH"` — default `"128"`, `CVAR_ARCHIVE`.
    pub cg_autoMapH: vmCvar_t,

    /// `"bg_fighterAltControl"` — default `"0"`, `CVAR_SERVERINFO`.
    pub bg_fighterAltControl: vmCvar_t,

    /// `"cg_chatBox"` — default `"10000"`, `CVAR_ARCHIVE`.
    pub cg_chatBox: vmCvar_t,
    /// `"cg_chatBoxHeight"` — default `"350"`, `CVAR_ARCHIVE`.
    pub cg_chatBoxHeight: vmCvar_t,

    /// `"cg_saberModelTraceEffect"` — default `"0"`, `0`.
    pub cg_saberModelTraceEffect: vmCvar_t,

    /// `"cg_saberClientVisualCompensation"` — default `"1"`, `0`.
    pub cg_saberClientVisualCompensation: vmCvar_t,

    /// `"cg_g2TraceLod"` — default `"2"`, `0`.
    pub cg_g2TraceLod: vmCvar_t,

    /// `"cg_fpls"` — default `"0"`, `0`.
    pub cg_fpls: vmCvar_t,

    /// `"cg_ghoul2Marks"` — default `"16"`, `0`.
    pub cg_ghoul2Marks: vmCvar_t,

    /// `"com_optvehtrace"` — default `"0"`, `0`.
    pub cg_optvehtrace: vmCvar_t,

    /// `"cg_saberDynamicMarks"` — default `"0"`, `0`.
    pub cg_saberDynamicMarks: vmCvar_t,
    /// `"cg_saberDynamicMarkTime"` — default `"60000"`, `0`.
    pub cg_saberDynamicMarkTime: vmCvar_t,

    /// `"cg_saberContact"` — default `"1"`, `0`.
    pub cg_saberContact: vmCvar_t,
    /// `"cg_saberTrail"` — default `"1"`, `0`.
    pub cg_saberTrail: vmCvar_t,

    /// `"cg_duelHeadAngles"` — default `"0"`, `0`.
    pub cg_duelHeadAngles: vmCvar_t,

    /// `"cg_speedTrail"` — default `"1"`, `0`.
    pub cg_speedTrail: vmCvar_t,
    /// `"cg_auraShell"` — default `"1"`, `0`.
    pub cg_auraShell: vmCvar_t,

    /// `"cg_repeaterOrb"` — default `"0"`, `0`.
    pub cg_repeaterOrb: vmCvar_t,

    /// `"cg_animBlend"` — default `"1"`, `0`.
    pub cg_animBlend: vmCvar_t,

    /// `"cg_dismember"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_dismember: vmCvar_t,

    /// `"cg_thirdPersonSpecialCam"` — default `"0"`, `0`.
    pub cg_thirdPersonSpecialCam: vmCvar_t,

    /// `"cg_thirdPerson"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_thirdPerson: vmCvar_t,
    /// `"cg_thirdPersonRange"` — default `"80"`, `CVAR_CHEAT`.
    pub cg_thirdPersonRange: vmCvar_t,
    /// `"cg_thirdPersonAngle"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_thirdPersonAngle: vmCvar_t,
    /// `"cg_thirdPersonPitchOffset"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_thirdPersonPitchOffset: vmCvar_t,
    /// `"cg_thirdPersonVertOffset"` — default `"16"`, `CVAR_CHEAT`.
    pub cg_thirdPersonVertOffset: vmCvar_t,
    /// `"cg_thirdPersonCameraDamp"` — default `"0.3"`, `0`.
    pub cg_thirdPersonCameraDamp: vmCvar_t,
    /// `"cg_thirdPersonTargetDamp"` — default `"0.5"`, `CVAR_CHEAT`.
    pub cg_thirdPersonTargetDamp: vmCvar_t,

    /// `"cg_thirdPersonAlpha"` — default `"1.0"`, `CVAR_CHEAT`.
    pub cg_thirdPersonAlpha: vmCvar_t,
    /// `"cg_thirdPersonHorzOffset"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_thirdPersonHorzOffset: vmCvar_t,

    /// `"cg_stereoSeparation"` — default `"0.4"`, `CVAR_ARCHIVE`.
    pub cg_stereoSeparation: vmCvar_t,
    /// `"cg_lagometer"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_lagometer: vmCvar_t,
    /// `"cg_drawEnemyInfo"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawEnemyInfo: vmCvar_t,
    /// `"g_synchronousClients"` — default `"0"`, `0`.
    pub cg_synchronousClients: vmCvar_t,
    /// `"cg_stats"` — default `"0"`, `0`.
    pub cg_stats: vmCvar_t,
    /// `"com_buildScript"` — default `"0"`, `0`.
    pub cg_buildScript: vmCvar_t,
    /// `"cg_forceModel"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_forceModel: vmCvar_t,
    /// `"cl_paused"` — default `"0"`, `CVAR_ROM`.
    pub cg_paused: vmCvar_t,
    /// `"com_blood"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_blood: vmCvar_t,
    /// `"cg_predictItems"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_predictItems: vmCvar_t,
    /// `"cg_deferPlayers"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_deferPlayers: vmCvar_t,
    /// `"cg_drawTeamOverlay"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_drawTeamOverlay: vmCvar_t,
    /// `"teamoverlay"` — default `"0"`, `CVAR_ROM | CVAR_USERINFO`.
    pub cg_teamOverlayUserinfo: vmCvar_t,
    /// `"cg_drawFriend"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_drawFriend: vmCvar_t,
    /// `"cg_teamChatsOnly"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_teamChatsOnly: vmCvar_t,
    /// `"cg_hudFiles"` — default `"ui/jahud.txt"`, `CVAR_ARCHIVE`.
    pub cg_hudFiles: vmCvar_t,
    /// `"cg_scorePlums"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_scorePlum: vmCvar_t,
    /// `"cg_smoothClients"` — default `"1"`, `CVAR_ARCHIVE`.
    pub cg_smoothClients: vmCvar_t,

    /// `"pmove_fixed"` — default `"0"`, `0`.
    pub pmove_fixed: vmCvar_t,

    /// `"pmove_msec"` — default `"8"`, `0`.
    pub pmove_msec: vmCvar_t,

    /// `g_showDuelHealths` — declared and read, never registered in `cvarTable`.
    pub g_showDuelHealths: vmCvar_t,

    /// `cg_pmove_msec` — declared and read, never registered in `cvarTable`.
    pub cg_pmove_msec: vmCvar_t,
    /// `"com_cameraMode"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_cameraMode: vmCvar_t,
    /// `"cg_cameraOrbit"` — default `"0"`, `CVAR_CHEAT`.
    pub cg_cameraOrbit: vmCvar_t,
    /// `"cg_cameraOrbitDelay"` — default `"50"`, `CVAR_ARCHIVE`.
    pub cg_cameraOrbitDelay: vmCvar_t,
    /// `"cg_timescaleFadeEnd"` — default `"1"`, `0`.
    pub cg_timescaleFadeEnd: vmCvar_t,
    /// `"cg_timescaleFadeSpeed"` — default `"0"`, `0`.
    pub cg_timescaleFadeSpeed: vmCvar_t,
    /// `"timescale"` — default `"1"`, `0`.
    pub cg_timescale: vmCvar_t,
    /// `"cg_noTaunt"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_noTaunt: vmCvar_t,
    /// `"cg_noProjectileTrail"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_noProjectileTrail: vmCvar_t,

    /// `"debugBB"` — default `"0"`, `0`.
    pub cg_debugBB: vmCvar_t,

    /// `"cg_currentSelectedPlayer"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_currentSelectedPlayer: vmCvar_t,
    /// `"cg_currentSelectedPlayerName"` — default `""`, `CVAR_ARCHIVE`.
    pub cg_currentSelectedPlayerName: vmCvar_t,

    /// `"ui_recordSPDemo"` — default `"0"`, `CVAR_ARCHIVE`.
    pub cg_recordSPDemo: vmCvar_t,
    /// `"ui_recordSPDemoName"` — default `""`, `CVAR_ARCHIVE`.
    pub cg_recordSPDemoName: vmCvar_t,
    /// `"cg_showVehBounds"` — default `"0"`, `0`.
    pub cg_showVehBounds: vmCvar_t,

    /// `"ui_myteam"` — default `"0"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_myteam: vmCvar_t,

    /// `"cg_snapshotTimeout"` — default `"10"`, `CVAR_ARCHIVE`.
    pub cg_snapshotTimeout: vmCvar_t,
}
