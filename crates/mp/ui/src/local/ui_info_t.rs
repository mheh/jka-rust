#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::limits::MAX_NAME_LENGTH;
use mp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, MAX_CLIENTS, MAX_STRING_CHARS};
use mp_uishared::shared::display_context_def_t::displayContextDef_t;

use super::alias_info::aliasInfo;
use super::game_type_info::gameTypeInfo;
use super::map_info::mapInfo;
use super::mod_info_t::modInfo_t;
use super::pending_server_status_t::pendingServerStatus_t;
use super::player_species_info_t::playerSpeciesInfo_t;
use super::server_status_info_t::serverStatusInfo_t;
use super::server_status_s::serverStatus_t;
use super::team_info::teamInfo;
use super::tier_info::tierInfo;

/// `MAX_ALIASES`.
///
/// Source: `oracle/codemp/ui/ui_local.h:563`
const MAX_ALIASES: usize = 64;

/// `MAX_TEAMS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:565`
const MAX_TEAMS: usize = 64;

/// `MAX_GAMETYPES`.
///
/// Source: `oracle/codemp/ui/ui_local.h:566`
const MAX_GAMETYPES: usize = 16;

/// `MAX_MAPS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:567`
const MAX_MAPS: usize = 128;

/// `MAX_TIERS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:589`
const MAX_TIERS: usize = 16;

/// `MAX_MODS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:590`
const MAX_MODS: usize = 64;

/// `MAX_DEMOS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:591`
const MAX_DEMOS: usize = 256;

/// `MAX_MOVIES`.
///
/// Source: `oracle/codemp/ui/ui_local.h:592`
const MAX_MOVIES: usize = 256;

/// `MAX_SCROLLTEXT_SIZE`.
///
/// Source: `oracle/codemp/ui/ui_local.h:596`
const MAX_SCROLLTEXT_SIZE: usize = 4096;

/// `MAX_SCROLLTEXT_LINES`.
///
/// Source: `oracle/codemp/ui/ui_local.h:597`
const MAX_SCROLLTEXT_LINES: usize = 64;

/// `MAX_ADDRESSLENGTH`.
///
/// Source: `oracle/codemp/ui/ui_local.h:571`
const MAX_ADDRESSLENGTH: usize = 64;

/// `MAX_FOUNDPLAYER_SERVERS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:580`
const MAX_FOUNDPLAYER_SERVERS: usize = 16;

/// `MAX_Q3PLAYERMODELS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:593` (`MAX_Q3PLAYERMODELS`)
const MAX_Q3PLAYERMODELS: usize = 256;

/// `MAX_FORCE_CONFIGS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:103`
const MAX_FORCE_CONFIGS: usize = 128;

/// `MAX_PLAYERMODELS`.
///
/// Source: `oracle/codemp/ui/ui_local.h:594`
const MAX_PLAYERMODELS: usize = 32;

/// Raven `uiInfo_t` — the UI module's top-level runtime state (menu display
/// context plus every cached list the menus draw from: teams, gametypes,
/// maps, tiers, mods, demos, movies, server browser/status, force configs,
/// player species, etc).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:729-841`
#[repr(C)]
pub struct uiInfo_t {
	pub uiDC: displayContextDef_t,
	pub newHighScoreTime: i32,
	pub newBestTime: i32,
	pub showPostGameTime: i32,
	pub newHighScore: qboolean,
	pub demoAvailable: qboolean,
	pub soundHighScore: qboolean,

	pub characterCount: i32,
	pub botIndex: i32,
	// Raven declares `characterInfo characterList[MAX_HEADS]` here but has it
	// commented out (`ui_local.h:740`) — the field is genuinely absent from the
	// struct, so there is nothing to port.

	pub aliasCount: i32,
	pub aliasList: [aliasInfo; MAX_ALIASES],

	pub teamCount: i32,
	pub teamList: [teamInfo; MAX_TEAMS],

	pub numGameTypes: i32,
	pub gameTypes: [gameTypeInfo; MAX_GAMETYPES],

	pub numJoinGameTypes: i32,
	pub joinGameTypes: [gameTypeInfo; MAX_GAMETYPES],

	pub redBlue: i32,
	pub playerCount: i32,
	pub myTeamCount: i32,
	pub teamIndex: i32,
	pub playerRefresh: i32,
	pub playerIndex: i32,
	pub playerNumber: i32,
	pub teamLeader: qboolean,
	pub playerNames: [[c_char; MAX_NAME_LENGTH]; MAX_CLIENTS],
	pub teamNames: [[c_char; MAX_NAME_LENGTH]; MAX_CLIENTS],
	pub teamClientNums: [i32; MAX_CLIENTS],

	// so we can vote-kick by index
	pub playerIndexes: [i32; MAX_CLIENTS],

	pub mapCount: i32,
	pub mapList: [mapInfo; MAX_MAPS],

	pub tierCount: i32,
	pub tierList: [tierInfo; MAX_TIERS],

	pub skillIndex: i32,

	pub modList: [modInfo_t; MAX_MODS],
	pub modCount: i32,
	pub modIndex: i32,

	pub demoList: [*const c_char; MAX_DEMOS],
	pub demoCount: i32,
	pub demoIndex: i32,

	pub movieList: [*const c_char; MAX_MOVIES],
	pub movieCount: i32,
	pub movieIndex: i32,
	pub previewMovie: i32,

	pub scrolltext: [c_char; MAX_SCROLLTEXT_SIZE],
	pub scrolltextLine: [*const c_char; MAX_SCROLLTEXT_LINES],
	pub scrolltextLineCount: i32,

	pub serverStatus: serverStatus_t,

	// for the showing the status of a server
	pub serverStatusAddress: [c_char; MAX_ADDRESSLENGTH],
	pub serverStatusInfo: serverStatusInfo_t,
	pub nextServerStatusRefresh: i32,

	// to retrieve the status of server to find a player
	pub pendingServerStatus: pendingServerStatus_t,
	pub findPlayerName: [c_char; MAX_STRING_CHARS],
	pub foundPlayerServerAddresses: [[c_char; MAX_ADDRESSLENGTH]; MAX_FOUNDPLAYER_SERVERS],
	pub foundPlayerServerNames: [[c_char; MAX_ADDRESSLENGTH]; MAX_FOUNDPLAYER_SERVERS],
	pub currentFoundPlayerServer: i32,
	pub numFoundPlayerServers: i32,
	pub nextFindPlayerRefresh: i32,

	pub currentCrosshair: i32,
	pub startPostGameTime: i32,
	pub newHighScoreSound: sfxHandle_t,

	pub q3HeadCount: i32,
	pub q3HeadNames: [[c_char; 64]; MAX_Q3PLAYERMODELS],
	pub q3HeadIcons: [qhandle_t; MAX_Q3PLAYERMODELS],
	pub q3SelectedHead: i32,

	pub forceConfigCount: i32,
	pub forceConfigSelected: i32,
	pub forceConfigNames: [[c_char; 128]; MAX_FORCE_CONFIGS],
	// true if it's a light side config, false if dark side
	pub forceConfigSide: [qboolean; MAX_FORCE_CONFIGS],
	// mark the index number dark configs start at
	pub forceConfigDarkIndexBegin: i32,
	// mark the index number light configs start at
	pub forceConfigLightIndexBegin: i32,

	pub effectsColor: i32,

	pub inGameLoad: qboolean,

	pub playerSpeciesCount: i32,
	pub playerSpecies: [playerSpeciesInfo_t; MAX_PLAYERMODELS],
	pub playerSpeciesIndex: i32,

	pub movesTitleIndex: i16,
	pub movesBaseAnim: *mut c_char,
	pub moveAnimTime: i32,

	pub languageCount: i32,
	pub languageCountIndex: i32,
}

const _: () = assert!(core::mem::size_of::<uiInfo_t>() == 342384);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, uiDC) == 0);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, newHighScoreTime) == 872);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, newBestTime) == 876);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, showPostGameTime) == 880);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, newHighScore) == 884);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, demoAvailable) == 888);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, soundHighScore) == 892);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, characterCount) == 896);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, botIndex) == 900);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, aliasCount) == 904);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, aliasList) == 912);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamCount) == 2448);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamList) == 2456);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, numGameTypes) == 8600);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, gameTypes) == 8608);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, numJoinGameTypes) == 8864);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, joinGameTypes) == 8872);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, redBlue) == 9128);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerCount) == 9132);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, myTeamCount) == 9136);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamIndex) == 9140);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerRefresh) == 9144);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerIndex) == 9148);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerNumber) == 9152);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamLeader) == 9156);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerNames) == 9160);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamNames) == 10184);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, teamClientNums) == 11208);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerIndexes) == 11336);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, mapCount) == 11464);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, mapList) == 11472);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, tierCount) == 26832);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, tierList) == 26840);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, skillIndex) == 27736);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modList) == 27744);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modCount) == 28768);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, modIndex) == 28772);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, demoList) == 28776);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, demoCount) == 30824);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, demoIndex) == 30828);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movieList) == 30832);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movieCount) == 32880);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movieIndex) == 32884);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, previewMovie) == 32888);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, scrolltext) == 32892);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, scrolltextLine) == 36992);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, scrolltextLineCount) == 37504);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, serverStatus) == 37508);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, serverStatusAddress) == 48992);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, serverStatusInfo) == 49056);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, nextServerStatusRefresh) == 54344);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, pendingServerStatus) == 54348);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, findPlayerName) == 56592);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, foundPlayerServerAddresses) == 57616);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, foundPlayerServerNames) == 58640);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, currentFoundPlayerServer) == 59664);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, numFoundPlayerServers) == 59668);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, nextFindPlayerRefresh) == 59672);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, currentCrosshair) == 59676);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, startPostGameTime) == 59680);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, newHighScoreSound) == 59684);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, q3HeadCount) == 59688);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, q3HeadNames) == 59692);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, q3HeadIcons) == 76076);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, q3SelectedHead) == 77100);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigCount) == 77104);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigSelected) == 77108);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigNames) == 77112);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigSide) == 93496);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigDarkIndexBegin) == 94008);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, forceConfigLightIndexBegin) == 94012);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, effectsColor) == 94016);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, inGameLoad) == 94020);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpeciesCount) == 94024);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpecies) == 94028);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, playerSpeciesIndex) == 342348);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movesTitleIndex) == 342352);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, movesBaseAnim) == 342360);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, moveAnimTime) == 342368);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, languageCount) == 342372);
const _: () = assert!(core::mem::offset_of!(uiInfo_t, languageCountIndex) == 342376);
