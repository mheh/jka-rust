//! `UiCvars` — Raven's file-scope `vmCvar_t` mirrors as one `UiWorld`
//! sub-struct.
//!
//! Every file-scope `vmCvar_t` ui declares lands here (§B3: file-scope globals
//! become owned state). 99 are registered from Raven's `cvarTable[]`, whose
//! name/default/flags each field records; the remaining four
//! (`ui_arenasFile`, `ui_teamName`, `ui_hudFiles`, `ui_serverFilterType`) are
//! declared and read but never registered — Raven leaves them zeroed.
//!
//! PORT-NOTE: the `#ifdef _XBOX` cvars (`ui_opti*`, `ui_hide?callout`) are not
//! part of the PC build and are dropped (porting-rules §20).
#![allow(non_snake_case)]

use mp_qshared::shared::cvar::vmCvar_t;

/// Raven's ui cvar handles, one field per declared file-scope `vmCvar_t`.
///
/// Source: `oracle/codemp/ui/ui_main.c:11280-11396` (declarations)
/// Source: `oracle/codemp/ui/ui_main.c:11399-11532` (`cvarTable`)
/// Source: `oracle/codemp/ui/ui_force.c:26` (`ui_freeSaber`, `ui_forcePowerDisable`)
#[derive(Clone, Default)]
pub struct UiCvars {
    /// `"ui_debug"` — default `"0"`, `CVAR_TEMP | CVAR_INTERNAL`.
    pub ui_debug: vmCvar_t,
    /// `"ui_initialized"` — default `"0"`, `CVAR_TEMP | CVAR_INTERNAL`.
    pub ui_initialized: vmCvar_t,
    /// `"ui_char_color_red"` — default `"255"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_color_red: vmCvar_t,
    /// `"ui_char_color_green"` — default `"255"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_color_green: vmCvar_t,
    /// `"ui_char_color_blue"` — default `"255"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_color_blue: vmCvar_t,
    /// `"ui_PrecacheModels"` — default `"0"`, `CVAR_ARCHIVE`.
    pub ui_PrecacheModels: vmCvar_t,
    /// `"ui_char_anim"` — default `"BOTH_WALK1"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_anim: vmCvar_t,
    /// `"ui_rankChange"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_rankChange: vmCvar_t,
    /// `"ui_ffa_fraglimit"` — default `"20"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_ffa_fraglimit: vmCvar_t,
    /// `"ui_ffa_timelimit"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_ffa_timelimit: vmCvar_t,
    /// `"ui_selectedModelIndex"` — default `"16"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_selectedModelIndex: vmCvar_t,
    /// `"ui_char_model"` — default `"jedi_tf"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_model: vmCvar_t,
    /// `"ui_char_skin_head"` — default `"head_a1"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_skin_head: vmCvar_t,
    /// `"ui_char_skin_torso"` — default `"torso_a1"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_skin_torso: vmCvar_t,
    /// `"ui_char_skin_legs"` — default `"lower_a1"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_char_skin_legs: vmCvar_t,
    /// `"ui_saber_type"` — default `"single"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_saber_type: vmCvar_t,
    /// `"ui_saber"` — default `"single_1"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_saber: vmCvar_t,
    /// `"ui_saber2"` — default `"none"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_saber2: vmCvar_t,
    /// `"ui_saber_color"` — default `"yellow"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_saber_color: vmCvar_t,
    /// `"ui_saber2_color"` — default `"yellow"`, `CVAR_ROM | CVAR_INTERNAL`.
    pub ui_saber2_color: vmCvar_t,
    /// `"ui_team_fraglimit"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_team_fraglimit: vmCvar_t,
    /// `"ui_team_timelimit"` — default `"20"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_team_timelimit: vmCvar_t,
    /// `"ui_team_friendly"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_team_friendly: vmCvar_t,
    /// `"ui_ctf_capturelimit"` — default `"8"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_ctf_capturelimit: vmCvar_t,
    /// `"ui_ctf_timelimit"` — default `"30"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_ctf_timelimit: vmCvar_t,
    /// `"ui_ctf_friendly"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_ctf_friendly: vmCvar_t,
    /// `ui_arenasFile` — declared and read, never registered in `cvarTable`.
    pub ui_arenasFile: vmCvar_t,
    /// `"g_botsFile"` — default `""`, `CVAR_INIT | CVAR_ROM`.
    pub ui_botsFile: vmCvar_t,
    /// `"g_spSkill"` — default `"2"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_spSkill: vmCvar_t,
    /// `"ui_browserMaster"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_browserMaster: vmCvar_t,
    /// `"ui_browserGameType"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_browserGameType: vmCvar_t,
    /// `"ui_browserSortKey"` — default `"4"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_browserSortKey: vmCvar_t,
    /// `"ui_browserShowFull"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_browserShowFull: vmCvar_t,
    /// `"ui_browserShowEmpty"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_browserShowEmpty: vmCvar_t,
    /// `"cg_drawCrosshair"` — default `"1"`, `CVAR_ARCHIVE`.
    pub ui_drawCrosshair: vmCvar_t,
    /// `"cg_drawCrosshairNames"` — default `"1"`, `CVAR_ARCHIVE`.
    pub ui_drawCrosshairNames: vmCvar_t,
    /// `"cg_marks"` — default `"1"`, `CVAR_ARCHIVE`.
    pub ui_marks: vmCvar_t,
    /// `"ui_redteam"` — default `"Empire"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam: vmCvar_t,
    /// `"ui_redteam1"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam1: vmCvar_t,
    /// `"ui_redteam2"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam2: vmCvar_t,
    /// `"ui_redteam3"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam3: vmCvar_t,
    /// `"ui_redteam4"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam4: vmCvar_t,
    /// `"ui_redteam5"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam5: vmCvar_t,
    /// `"ui_redteam6"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam6: vmCvar_t,
    /// `"ui_redteam7"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam7: vmCvar_t,
    /// `"ui_redteam8"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_redteam8: vmCvar_t,
    /// `"ui_blueteam"` — default `"Rebellion"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam: vmCvar_t,
    /// `"ui_blueteam1"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam1: vmCvar_t,
    /// `"ui_blueteam2"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam2: vmCvar_t,
    /// `"ui_blueteam3"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam3: vmCvar_t,
    /// `"ui_blueteam4"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam4: vmCvar_t,
    /// `"ui_blueteam5"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam5: vmCvar_t,
    /// `"ui_blueteam6"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam6: vmCvar_t,
    /// `"ui_blueteam7"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam7: vmCvar_t,
    /// `"ui_blueteam8"` — default `"1"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_blueteam8: vmCvar_t,
    /// `ui_teamName` — declared and read, never registered in `cvarTable`.
    pub ui_teamName: vmCvar_t,
    /// `"ui_dedicated"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_dedicated: vmCvar_t,
    /// `"ui_gametype"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_gameType: vmCvar_t,
    /// `"ui_netGametype"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_netGameType: vmCvar_t,
    /// `"ui_actualNetGametype"` — default `"3"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_actualNetGameType: vmCvar_t,
    /// `"ui_joinGametype"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_joinGameType: vmCvar_t,
    /// `"ui_netSource"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_netSource: vmCvar_t,
    /// `ui_serverFilterType` — declared and read, never registered in `cvarTable`.
    pub ui_serverFilterType: vmCvar_t,
    /// `"ui_opponentName"` — default `"Rebellion"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_opponentName: vmCvar_t,
    /// `"ui_menuFilesMP"` — default `"ui/jampmenus.txt"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_menuFiles: vmCvar_t,
    /// `"ui_currentMap"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_currentMap: vmCvar_t,
    /// `"ui_currentNetMap"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_currentNetMap: vmCvar_t,
    /// `"ui_mapIndex"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_mapIndex: vmCvar_t,
    /// `"ui_currentOpponent"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_currentOpponent: vmCvar_t,
    /// `"cg_selectedPlayer"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_selectedPlayer: vmCvar_t,
    /// `"cg_selectedPlayerName"` — default `""`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_selectedPlayerName: vmCvar_t,
    /// `"ui_lastServerRefresh_0"` — default `""`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_lastServerRefresh_0: vmCvar_t,
    /// `"ui_lastServerRefresh_1"` — default `""`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_lastServerRefresh_1: vmCvar_t,
    /// `"ui_lastServerRefresh_2"` — default `""`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_lastServerRefresh_2: vmCvar_t,
    /// `"ui_lastServerRefresh_3"` — default `""`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_lastServerRefresh_3: vmCvar_t,
    /// `"ui_singlePlayerActive"` — default `"0"`, `CVAR_INTERNAL`.
    pub ui_singlePlayerActive: vmCvar_t,
    /// `"ui_scoreAccuracy"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreAccuracy: vmCvar_t,
    /// `"ui_scoreImpressives"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreImpressives: vmCvar_t,
    /// `"ui_scoreExcellents"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreExcellents: vmCvar_t,
    /// `"ui_scoreCaptures"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreCaptures: vmCvar_t,
    /// `"ui_scoreDefends"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreDefends: vmCvar_t,
    /// `"ui_scoreAssists"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreAssists: vmCvar_t,
    /// `"ui_scoreGauntlets"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreGauntlets: vmCvar_t,
    /// `"ui_scoreScore"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreScore: vmCvar_t,
    /// `"ui_scorePerfect"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scorePerfect: vmCvar_t,
    /// `"ui_scoreTeam"` — default `"0 to 0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreTeam: vmCvar_t,
    /// `"ui_scoreBase"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreBase: vmCvar_t,
    /// `"ui_scoreTimeBonus"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreTimeBonus: vmCvar_t,
    /// `"ui_scoreSkillBonus"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreSkillBonus: vmCvar_t,
    /// `"ui_scoreShutoutBonus"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreShutoutBonus: vmCvar_t,
    /// `"ui_scoreTime"` — default `"00:00"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_scoreTime: vmCvar_t,
    /// `"ui_captureLimit"` — default `"5"`, `CVAR_INTERNAL`.
    pub ui_captureLimit: vmCvar_t,
    /// `"ui_fragLimit"` — default `"10"`, `CVAR_INTERNAL`.
    pub ui_fragLimit: vmCvar_t,
    /// `"ui_findPlayer"` — default `"Kyle"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_findPlayer: vmCvar_t,
    /// `ui_hudFiles` — declared and read, never registered in `cvarTable`.
    pub ui_hudFiles: vmCvar_t,
    /// `"ui_recordSPDemo"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_recordSPDemo: vmCvar_t,
    /// `"capturelimit"` — default `"0"`, `CVAR_SERVERINFO | CVAR_ARCHIVE | CVAR_NORESTART`.
    pub ui_realCaptureLimit: vmCvar_t,
    /// `"g_warmup"` — default `"20"`, `CVAR_ARCHIVE`.
    pub ui_realWarmUp: vmCvar_t,
    /// `"ui_serverStatusTimeOut"` — default `"7000"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_serverStatusTimeOut: vmCvar_t,
    /// `"se_language"` — default `"english"`, `CVAR_ARCHIVE | CVAR_NORESTART`.
    pub se_language: vmCvar_t,
    /// `"ui_bypassMainMenuLoad"` — default `"0"`, `CVAR_INTERNAL`.
    pub ui_bypassMainMenuLoad: vmCvar_t,
    /// `"ui_freeSaber"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_freeSaber: vmCvar_t,
    /// `"ui_forcePowerDisable"` — default `"0"`, `CVAR_ARCHIVE | CVAR_INTERNAL`.
    pub ui_forcePowerDisable: vmCvar_t,
}
