#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_void;

use mp_bg::public::configstring::{MAX_FX, MAX_ICONS, MAX_MODELS, MAX_SOUNDS};
use mp_bg::public::gametype::gametype_t;
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::shared::limits::MAX_STRING_TOKENS;
use mp_qshared::shared::{
    fxHandle_t, gameState_t, qboolean, qhandle_t, sfxHandle_t, vec3_t, MAX_CLIENTS, MAX_QPATH,
};

use super::cg_effects_t::cgEffects_t;
use super::cg_media_t::cgMedia_t;
use super::client_info_t::clientInfo_t;

// `MAX_MODELS`/`MAX_SOUNDS`/`MAX_ICONS`/`MAX_FX` are the shared configstring
// limits (`q_shared.h:2020-2023`); imported from their canonical home in
// `mp_bg::public::configstring` (`c_int`, cast to `usize` at the array sites).

/// Raven `cgs_t` — the entire cgame state that persists across an active
/// connection, similar to `svs.clients[]`.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:1516-1609`
#[repr(C)]
pub struct cgs_t {
    /// gamestate from server
    pub gameState: gameState_t,
    /// rendering configuration
    pub glconfig: glconfig_t,
    /// derived from glconfig
    pub screenXScale: f32,
    pub screenYScale: f32,
    pub screenXBias: f32,

    /// reliable command stream counter
    pub serverCommandSequence: i32,
    /// the number of snapshots cgame has requested
    pub processedSnapshotNum: i32,

    /// detected on startup by checking sv_running
    pub localServer: qboolean,

    // parsed from serverinfo
    pub siegeTeamSwitch: i32,
    pub showDuelHealths: i32,
    pub gametype: gametype_t,
    pub debugMelee: i32,
    pub stepSlideFix: i32,
    pub noSpecMove: i32,
    pub dmflags: i32,
    pub teamflags: i32,
    pub fraglimit: i32,
    pub duel_fraglimit: i32,
    pub capturelimit: i32,
    pub timelimit: i32,
    pub maxclients: i32,
    pub needpass: qboolean,
    pub jediVmerc: qboolean,
    pub wDisable: i32,
    pub fDisable: i32,

    pub mapname: [c_char; MAX_QPATH],
    //	char			redTeam[MAX_QPATH];
    //	char			blueTeam[MAX_QPATH];
    pub voteTime: i32,
    pub voteYes: i32,
    pub voteNo: i32,
    /// beep whenever changed
    pub voteModified: qboolean,
    pub voteString: [c_char; MAX_STRING_TOKENS],

    pub teamVoteTime: [i32; 2],
    pub teamVoteYes: [i32; 2],
    pub teamVoteNo: [i32; 2],
    /// beep whenever changed
    pub teamVoteModified: [qboolean; 2],
    pub teamVoteString: [[c_char; MAX_STRING_TOKENS]; 2],

    pub levelStartTime: i32,

    /// from configstrings
    pub scores1: i32,
    pub scores2: i32,
    pub jediMaster: i32,
    pub duelWinner: i32,
    pub duelist1: i32,
    pub duelist2: i32,
    pub duelist3: i32,
    // nmckenzie: DUEL_HEALTH.  hmm.
    pub duelist1health: i32,
    pub duelist2health: i32,
    pub duelist3health: i32,

    /// flag status from configstrings
    pub redflag: i32,
    pub blueflag: i32,
    pub flagStatus: i32,

    pub newHud: qboolean,

    //
    // locally derived information from gamestate
    //
    pub gameModels: [qhandle_t; MAX_MODELS as usize],
    pub gameSounds: [sfxHandle_t; MAX_SOUNDS as usize],
    pub gameEffects: [fxHandle_t; MAX_FX as usize],
    pub gameIcons: [qhandle_t; MAX_ICONS as usize],

    pub numInlineModels: i32,
    pub inlineDrawModel: [qhandle_t; MAX_MODELS as usize],
    pub inlineModelMidpoints: [vec3_t; MAX_MODELS as usize],

    pub clientinfo: [clientInfo_t; MAX_CLIENTS],

    pub cursorX: i32,
    pub cursorY: i32,
    pub eventHandling: qboolean,
    pub mouseCaptured: qboolean,
    pub sizingHud: qboolean,
    pub capturedItem: *mut c_void,
    pub activeCursor: qhandle_t,

    /// media
    pub media: cgMedia_t,

    /// effects
    pub effects: cgEffects_t,
}

impl cgs_t {
    /// §19: a server bmodel `modelindex` can exceed `MAX_MODELS` on huge maps
    /// (live Lugormod map with 590 inline models, 2026-07-29) - Raven reads
    /// adjacent-memory garbage there; out-of-range answers the zero midpoint.
    pub fn inline_model_midpoint(&self, modelindex: usize) -> vec3_t {
        *self
            .inlineModelMidpoints
            .get(modelindex)
            .unwrap_or(&[0.0; 3])
    }

    /// §19: same out-of-range family as ``inline_model_midpoint`` - Raven
    /// hands the renderer a garbage handle; out-of-range answers handle 0.
    pub fn inline_draw_model(&self, modelindex: usize) -> qhandle_t {
        *self.inlineDrawModel.get(modelindex).unwrap_or(&0)
    }
}

// `gameState` sits at the front of the struct, so its offset holds on both pointer widths.
const _: () = assert!(core::mem::offset_of!(cgs_t, gameState) == 0);
// `glconfig` holds four `const char *`, so its alignment and every offset from `glconfig` onward change with the pointer width.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<cgs_t>() == 229576);
    assert!(core::mem::offset_of!(cgs_t, glconfig) == 22808);
    assert!(core::mem::offset_of!(cgs_t, screenXScale) == 22904);
    assert!(core::mem::offset_of!(cgs_t, screenYScale) == 22908);
    assert!(core::mem::offset_of!(cgs_t, screenXBias) == 22912);
    assert!(core::mem::offset_of!(cgs_t, serverCommandSequence) == 22916);
    assert!(core::mem::offset_of!(cgs_t, processedSnapshotNum) == 22920);
    assert!(core::mem::offset_of!(cgs_t, localServer) == 22924);
    assert!(core::mem::offset_of!(cgs_t, siegeTeamSwitch) == 22928);
    assert!(core::mem::offset_of!(cgs_t, showDuelHealths) == 22932);
    assert!(core::mem::offset_of!(cgs_t, gametype) == 22936);
    assert!(core::mem::offset_of!(cgs_t, debugMelee) == 22940);
    assert!(core::mem::offset_of!(cgs_t, stepSlideFix) == 22944);
    assert!(core::mem::offset_of!(cgs_t, noSpecMove) == 22948);
    assert!(core::mem::offset_of!(cgs_t, dmflags) == 22952);
    assert!(core::mem::offset_of!(cgs_t, teamflags) == 22956);
    assert!(core::mem::offset_of!(cgs_t, fraglimit) == 22960);
    assert!(core::mem::offset_of!(cgs_t, duel_fraglimit) == 22964);
    assert!(core::mem::offset_of!(cgs_t, capturelimit) == 22968);
    assert!(core::mem::offset_of!(cgs_t, timelimit) == 22972);
    assert!(core::mem::offset_of!(cgs_t, maxclients) == 22976);
    assert!(core::mem::offset_of!(cgs_t, needpass) == 22980);
    assert!(core::mem::offset_of!(cgs_t, jediVmerc) == 22984);
    assert!(core::mem::offset_of!(cgs_t, wDisable) == 22988);
    assert!(core::mem::offset_of!(cgs_t, fDisable) == 22992);
    assert!(core::mem::offset_of!(cgs_t, mapname) == 22996);
    assert!(core::mem::offset_of!(cgs_t, voteTime) == 23060);
    assert!(core::mem::offset_of!(cgs_t, voteYes) == 23064);
    assert!(core::mem::offset_of!(cgs_t, voteNo) == 23068);
    assert!(core::mem::offset_of!(cgs_t, voteModified) == 23072);
    assert!(core::mem::offset_of!(cgs_t, voteString) == 23076);
    assert!(core::mem::offset_of!(cgs_t, teamVoteTime) == 24100);
    assert!(core::mem::offset_of!(cgs_t, teamVoteYes) == 24108);
    assert!(core::mem::offset_of!(cgs_t, teamVoteNo) == 24116);
    assert!(core::mem::offset_of!(cgs_t, teamVoteModified) == 24124);
    assert!(core::mem::offset_of!(cgs_t, teamVoteString) == 24132);
    assert!(core::mem::offset_of!(cgs_t, levelStartTime) == 26180);
    assert!(core::mem::offset_of!(cgs_t, scores1) == 26184);
    assert!(core::mem::offset_of!(cgs_t, scores2) == 26188);
    assert!(core::mem::offset_of!(cgs_t, jediMaster) == 26192);
    assert!(core::mem::offset_of!(cgs_t, duelWinner) == 26196);
    assert!(core::mem::offset_of!(cgs_t, duelist1) == 26200);
    assert!(core::mem::offset_of!(cgs_t, duelist2) == 26204);
    assert!(core::mem::offset_of!(cgs_t, duelist3) == 26208);
    assert!(core::mem::offset_of!(cgs_t, duelist1health) == 26212);
    assert!(core::mem::offset_of!(cgs_t, duelist2health) == 26216);
    assert!(core::mem::offset_of!(cgs_t, duelist3health) == 26220);
    assert!(core::mem::offset_of!(cgs_t, redflag) == 26224);
    assert!(core::mem::offset_of!(cgs_t, blueflag) == 26228);
    assert!(core::mem::offset_of!(cgs_t, flagStatus) == 26232);
    assert!(core::mem::offset_of!(cgs_t, newHud) == 26236);
    assert!(core::mem::offset_of!(cgs_t, gameModels) == 26240);
    assert!(core::mem::offset_of!(cgs_t, gameSounds) == 28288);
    assert!(core::mem::offset_of!(cgs_t, gameEffects) == 29312);
    assert!(core::mem::offset_of!(cgs_t, gameIcons) == 29568);
    assert!(core::mem::offset_of!(cgs_t, numInlineModels) == 29824);
    assert!(core::mem::offset_of!(cgs_t, inlineDrawModel) == 29828);
    assert!(core::mem::offset_of!(cgs_t, inlineModelMidpoints) == 31876);
    assert!(core::mem::offset_of!(cgs_t, clientinfo) == 38024);
    assert!(core::mem::offset_of!(cgs_t, cursorX) == 227464);
    assert!(core::mem::offset_of!(cgs_t, cursorY) == 227468);
    assert!(core::mem::offset_of!(cgs_t, eventHandling) == 227472);
    assert!(core::mem::offset_of!(cgs_t, mouseCaptured) == 227476);
    assert!(core::mem::offset_of!(cgs_t, sizingHud) == 227480);
    assert!(core::mem::offset_of!(cgs_t, capturedItem) == 227488);
    assert!(core::mem::offset_of!(cgs_t, activeCursor) == 227496);
    assert!(core::mem::offset_of!(cgs_t, media) == 227500);
    assert!(core::mem::offset_of!(cgs_t, effects) == 229216);
};
// ILP32 twin: clang i386 ground truth, where msvc and linux-gnu agree.
// These numbers are the retail 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<cgs_t>() == 229024);
    assert!(core::mem::offset_of!(cgs_t, glconfig) == 22804);
    assert!(core::mem::offset_of!(cgs_t, screenXScale) == 22880);
    assert!(core::mem::offset_of!(cgs_t, screenYScale) == 22884);
    assert!(core::mem::offset_of!(cgs_t, screenXBias) == 22888);
    assert!(core::mem::offset_of!(cgs_t, serverCommandSequence) == 22892);
    assert!(core::mem::offset_of!(cgs_t, processedSnapshotNum) == 22896);
    assert!(core::mem::offset_of!(cgs_t, localServer) == 22900);
    assert!(core::mem::offset_of!(cgs_t, siegeTeamSwitch) == 22904);
    assert!(core::mem::offset_of!(cgs_t, showDuelHealths) == 22908);
    assert!(core::mem::offset_of!(cgs_t, gametype) == 22912);
    assert!(core::mem::offset_of!(cgs_t, debugMelee) == 22916);
    assert!(core::mem::offset_of!(cgs_t, stepSlideFix) == 22920);
    assert!(core::mem::offset_of!(cgs_t, noSpecMove) == 22924);
    assert!(core::mem::offset_of!(cgs_t, dmflags) == 22928);
    assert!(core::mem::offset_of!(cgs_t, teamflags) == 22932);
    assert!(core::mem::offset_of!(cgs_t, fraglimit) == 22936);
    assert!(core::mem::offset_of!(cgs_t, duel_fraglimit) == 22940);
    assert!(core::mem::offset_of!(cgs_t, capturelimit) == 22944);
    assert!(core::mem::offset_of!(cgs_t, timelimit) == 22948);
    assert!(core::mem::offset_of!(cgs_t, maxclients) == 22952);
    assert!(core::mem::offset_of!(cgs_t, needpass) == 22956);
    assert!(core::mem::offset_of!(cgs_t, jediVmerc) == 22960);
    assert!(core::mem::offset_of!(cgs_t, wDisable) == 22964);
    assert!(core::mem::offset_of!(cgs_t, fDisable) == 22968);
    assert!(core::mem::offset_of!(cgs_t, mapname) == 22972);
    assert!(core::mem::offset_of!(cgs_t, voteTime) == 23036);
    assert!(core::mem::offset_of!(cgs_t, voteYes) == 23040);
    assert!(core::mem::offset_of!(cgs_t, voteNo) == 23044);
    assert!(core::mem::offset_of!(cgs_t, voteModified) == 23048);
    assert!(core::mem::offset_of!(cgs_t, voteString) == 23052);
    assert!(core::mem::offset_of!(cgs_t, teamVoteTime) == 24076);
    assert!(core::mem::offset_of!(cgs_t, teamVoteYes) == 24084);
    assert!(core::mem::offset_of!(cgs_t, teamVoteNo) == 24092);
    assert!(core::mem::offset_of!(cgs_t, teamVoteModified) == 24100);
    assert!(core::mem::offset_of!(cgs_t, teamVoteString) == 24108);
    assert!(core::mem::offset_of!(cgs_t, levelStartTime) == 26156);
    assert!(core::mem::offset_of!(cgs_t, scores1) == 26160);
    assert!(core::mem::offset_of!(cgs_t, scores2) == 26164);
    assert!(core::mem::offset_of!(cgs_t, jediMaster) == 26168);
    assert!(core::mem::offset_of!(cgs_t, duelWinner) == 26172);
    assert!(core::mem::offset_of!(cgs_t, duelist1) == 26176);
    assert!(core::mem::offset_of!(cgs_t, duelist2) == 26180);
    assert!(core::mem::offset_of!(cgs_t, duelist3) == 26184);
    assert!(core::mem::offset_of!(cgs_t, duelist1health) == 26188);
    assert!(core::mem::offset_of!(cgs_t, duelist2health) == 26192);
    assert!(core::mem::offset_of!(cgs_t, duelist3health) == 26196);
    assert!(core::mem::offset_of!(cgs_t, redflag) == 26200);
    assert!(core::mem::offset_of!(cgs_t, blueflag) == 26204);
    assert!(core::mem::offset_of!(cgs_t, flagStatus) == 26208);
    assert!(core::mem::offset_of!(cgs_t, newHud) == 26212);
    assert!(core::mem::offset_of!(cgs_t, gameModels) == 26216);
    assert!(core::mem::offset_of!(cgs_t, gameSounds) == 28264);
    assert!(core::mem::offset_of!(cgs_t, gameEffects) == 29288);
    assert!(core::mem::offset_of!(cgs_t, gameIcons) == 29544);
    assert!(core::mem::offset_of!(cgs_t, numInlineModels) == 29800);
    assert!(core::mem::offset_of!(cgs_t, inlineDrawModel) == 29804);
    assert!(core::mem::offset_of!(cgs_t, inlineModelMidpoints) == 31852);
    assert!(core::mem::offset_of!(cgs_t, clientinfo) == 37996);
    assert!(core::mem::offset_of!(cgs_t, cursorX) == 226924);
    assert!(core::mem::offset_of!(cgs_t, cursorY) == 226928);
    assert!(core::mem::offset_of!(cgs_t, eventHandling) == 226932);
    assert!(core::mem::offset_of!(cgs_t, mouseCaptured) == 226936);
    assert!(core::mem::offset_of!(cgs_t, sizingHud) == 226940);
    assert!(core::mem::offset_of!(cgs_t, capturedItem) == 226944);
    assert!(core::mem::offset_of!(cgs_t, activeCursor) == 226948);
    assert!(core::mem::offset_of!(cgs_t, media) == 226952);
    assert!(core::mem::offset_of!(cgs_t, effects) == 228668);
};
