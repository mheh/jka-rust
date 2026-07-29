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

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cgs_t>() == 229576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameState) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, glconfig) == 22808);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, screenXScale) == 22904);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, screenYScale) == 22908);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, screenXBias) == 22912);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, serverCommandSequence) == 22916);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, processedSnapshotNum) == 22920);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, localServer) == 22924);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, siegeTeamSwitch) == 22928);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, showDuelHealths) == 22932);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gametype) == 22936);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, debugMelee) == 22940);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, stepSlideFix) == 22944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, noSpecMove) == 22948);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, dmflags) == 22952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamflags) == 22956);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, fraglimit) == 22960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duel_fraglimit) == 22964);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, capturelimit) == 22968);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, timelimit) == 22972);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, maxclients) == 22976);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, needpass) == 22980);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, jediVmerc) == 22984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, wDisable) == 22988);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, fDisable) == 22992);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, mapname) == 22996);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, voteTime) == 23060);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, voteYes) == 23064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, voteNo) == 23068);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, voteModified) == 23072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, voteString) == 23076);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamVoteTime) == 24100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamVoteYes) == 24108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamVoteNo) == 24116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamVoteModified) == 24124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, teamVoteString) == 24132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, levelStartTime) == 26180);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, scores1) == 26184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, scores2) == 26188);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, jediMaster) == 26192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelWinner) == 26196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist1) == 26200);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist2) == 26204);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist3) == 26208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist1health) == 26212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist2health) == 26216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, duelist3health) == 26220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, redflag) == 26224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, blueflag) == 26228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, flagStatus) == 26232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, newHud) == 26236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameModels) == 26240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameSounds) == 28288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameEffects) == 29312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, gameIcons) == 29568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, numInlineModels) == 29824);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, inlineDrawModel) == 29828);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, inlineModelMidpoints) == 31876);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, clientinfo) == 38024);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, cursorX) == 227464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, cursorY) == 227468);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, eventHandling) == 227472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, mouseCaptured) == 227476);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, sizingHud) == 227480);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, capturedItem) == 227488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, activeCursor) == 227496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, media) == 227500);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(cgs_t, effects) == 229216);
