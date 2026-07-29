//! `UiWorld` — the one owned ui-module island (DEC-36 D1).

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_channel::{BgHost, BgState};
use mp_bg::bg_misc::MAX_POOL_SIZE_UI;
use mp_qshared::shared::qhandle_t;
use mp_qshared::shared::sfxHandle_t;

use crate::local::alias_info::AliasInfo;
use crate::local::game_type_info::GameTypeInfo;
use crate::local::map_info::MapInfo;
use crate::local::mod_info_t::ModInfo;
use crate::local::pending_server_status_t::PendingServerStatus;
use crate::local::player_species_info_t::PlayerSpeciesInfo;
use crate::local::server_status_info_t::ServerStatusInfo;
use crate::local::server_status_s::ServerStatus;
use crate::local::team_info::TeamInfo;
use crate::local::tier_info::TierInfo;

use super::ui_cvars::UiCvars;
use super::ui_force_state::UiForceState;
use super::ui_gameinfo_state::UiGameinfoState;
use super::ui_main_state::UiMainState;
use super::ui_saber_state::UiSaberState;
use super::ui_scratch::UiScratch;

/// Raven `#define MAX_FORCE_CONFIGS 128`.
///
/// Source: `oracle/codemp/ui/ui_local.h:103`
pub const MAX_FORCE_CONFIGS: usize = 128;

/// Raven `#define MAX_FOUNDPLAYER_SERVERS 16`.
///
/// Source: `oracle/codemp/ui/ui_local.h:580`
pub const MAX_FOUNDPLAYER_SERVERS: usize = 16;

/// Raven `#define MAX_SCROLLTEXT_LINES 64`.
///
/// Source: `oracle/codemp/ui/ui_local.h:597`
pub const MAX_SCROLLTEXT_LINES: usize = 64;

/// The ui module's one owned state island: Raven's `uiInfo_t uiInfo` spine
/// with every remaining file-scope global folded in (DEC-36 D1). It is a value
/// owned by the `vmMain` shell, not a global — the ABI entrypoints hold the
/// single instance and hand it inward inside a
/// [`UiContext`](super::ui_context::UiContext) (§B3/§B4).
///
/// Raven's `uiDC` member and `ui_shared.c`'s menu-framework globals are NOT
/// here: they are sibling fields of [`UiState`](super::ui_state::UiState), so
/// the ported fns can hold them beside a live `UiContext` (DEC-38 ruling 1).
///
/// The scoping census settled that ui has **zero** Class-A engine-retained
/// memory — no `trap` registers a pointer into module memory and every ui trap
/// is copy-semantics — so the whole island is Class C and lands idiomatic:
/// `char[N]`/`const char *` → `String`, `qboolean` → `bool`, grow-on-parse
/// arrays → `Vec`, pointer graphs → arenas plus handles.
///
/// PORT-NOTE (count fields): Raven paired sixteen `xxxCount`/`numXxx` ints with
/// the fixed arrays they filled (`aliasCount`, `teamCount`, `numGameTypes`,
/// `numJoinGameTypes`, `playerCount`, `myTeamCount`, `mapCount`, `tierCount`,
/// `modCount`, `demoCount`, `movieCount`, `scrolltextLineCount`,
/// `q3HeadCount`, `forceConfigCount`, `playerSpeciesCount`). Each array is a
/// `Vec` here and its count is `Vec::len()`, so the paired int does not
/// survive. Counts with no array behind them (`characterCount`,
/// `languageCount`, …) do. `numFoundPlayerServers` is the one exception and
/// survives as a field — see its doc.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:729-843`
pub struct UiWorld {
    pub newHighScoreTime: c_int,
    pub newBestTime: c_int,
    pub showPostGameTime: c_int,
    pub newHighScore: bool,
    pub demoAvailable: bool,
    pub soundHighScore: bool,

    pub characterCount: c_int,
    pub botIndex: c_int,
    // Raven declares `characterInfo characterList[MAX_HEADS]` here but has it
    // commented out (`ui_local.h:740`), so there is nothing to port.
    pub aliasList: Vec<AliasInfo>,

    pub teamList: Vec<TeamInfo>,

    pub gameTypes: Vec<GameTypeInfo>,
    pub joinGameTypes: Vec<GameTypeInfo>,

    pub redBlue: c_int,
    pub teamIndex: c_int,
    pub playerRefresh: c_int,
    pub playerIndex: c_int,
    pub playerNumber: c_int,
    pub teamLeader: bool,
    /// Raven `char playerNames[MAX_CLIENTS][MAX_NAME_LENGTH]` + `playerCount`.
    pub playerNames: Vec<String>,
    /// Raven `char teamNames[MAX_CLIENTS][MAX_NAME_LENGTH]` + `myTeamCount`;
    /// `teamClientNums` runs parallel to it.
    pub teamNames: Vec<String>,
    pub teamClientNums: Vec<c_int>,

    /// so we can vote-kick by index — parallel to `playerNames`.
    pub playerIndexes: Vec<c_int>,

    pub mapList: Vec<MapInfo>,

    pub tierList: Vec<TierInfo>,

    pub skillIndex: c_int,

    pub modList: Vec<ModInfo>,
    pub modIndex: c_int,

    pub demoList: Vec<String>,
    pub demoIndex: c_int,

    pub movieList: Vec<String>,
    pub movieIndex: c_int,
    pub previewMovie: c_int,

    /// Raven `char scrolltext[MAX_SCROLLTEXT_SIZE]` — the credits text as read
    /// from disk; `scrolltextLine` held pointers into it and now owns its
    /// lines.
    pub scrolltext: String,
    pub scrolltextLine: Vec<String>,

    pub serverStatus: ServerStatus,

    // for the showing the status of a server
    pub serverStatusAddress: String,
    pub serverStatusInfo: ServerStatusInfo,
    pub nextServerStatusRefresh: c_int,

    // to retrieve the status of server to find a player
    pub pendingServerStatus: PendingServerStatus,
    pub findPlayerName: String,
    pub foundPlayerServerAddresses: Vec<String>,
    pub foundPlayerServerNames: Vec<String>,
    pub currentFoundPlayerServer: c_int,

    /// Raven `int numFoundPlayerServers` — 1-based count over
    /// `foundPlayerServerAddresses`/`foundPlayerServerNames`.
    ///
    /// Restored despite the count-field-elimination convention above: `0` and
    /// `1` are distinct observable states no `Vec` length can represent, and
    /// the reserved trailing slot at `[count - 1]` is feeder-visible.
    /// Source: `oracle/codemp/ui/ui_local.h:807`
    pub numFoundPlayerServers: c_int,
    pub nextFindPlayerRefresh: c_int,

    pub currentCrosshair: c_int,
    pub startPostGameTime: c_int,
    pub newHighScoreSound: sfxHandle_t,

    /// Raven `char q3HeadNames[MAX_Q3PLAYERMODELS][64]` + `q3HeadCount`;
    /// `q3HeadIcons` runs parallel to it.
    pub q3HeadNames: Vec<String>,
    pub q3HeadIcons: Vec<qhandle_t>,
    pub q3SelectedHead: c_int,

    pub forceConfigSelected: c_int,
    /// Raven `char forceConfigNames[MAX_FORCE_CONFIGS][128]` +
    /// `forceConfigCount`; `forceConfigSide` runs parallel to it.
    pub forceConfigNames: Vec<String>,
    /// true if it's a light side config, false if dark side
    pub forceConfigSide: Vec<bool>,
    /// mark the index number dark configs start at
    pub forceConfigDarkIndexBegin: c_int,
    /// mark the index number light configs start at
    pub forceConfigLightIndexBegin: c_int,

    pub effectsColor: c_int,

    pub inGameLoad: bool,

    pub playerSpecies: Vec<PlayerSpeciesInfo>,
    pub playerSpeciesIndex: c_int,

    pub movesTitleIndex: i16,
    pub movesBaseAnim: String,
    pub moveAnimTime: c_int,

    pub languageCount: c_int,
    pub languageCountIndex: c_int,

    /// Raven's ~103 file-scope `vmCvar_t` handles.
    /// Source: `super::ui_cvars::UiCvars`
    pub cvars: UiCvars,

    /// `ui_force.c`'s file-scope globals.
    /// Source: `super::ui_force_state::UiForceState`
    pub force: UiForceState,

    /// `ui_saber.c`'s file-scope statics.
    /// Source: `super::ui_saber_state::UiSaberState`
    pub saber: UiSaberState,

    /// `ui_gameinfo.c`'s file-scope globals.
    /// Source: `super::ui_gameinfo_state::UiGameinfoState`
    pub gameinfo: UiGameinfoState,

    /// `ui_main.c`'s mutable file-scope globals.
    /// Source: `super::ui_main_state::UiMainState`
    pub main: UiMainState,

    /// ui-tier function-local persistent scratch.
    /// Source: `super::ui_scratch::UiScratch`
    pub scratch: UiScratch,

    /// The ui module's own bg-tier state — Raven compiled the bg files into
    /// the ui link unit (`WE_ARE_IN_THE_UI`), giving ui its own copies of the
    /// bg globals (rand state, siege class tables, parse scratch, `BG_Alloc`
    /// pool at the 512000 ui arm). DEC-36 addendum 11 (D5's second-implementor
    /// story).
    /// Source: `oracle/codemp/game/bg_misc.c:3311-3316`
    pub bg_state: BgState,
}

impl Default for UiWorld {
    /// Raven's `uiInfo_t uiInfo` is a zeroed file-scope struct that `_UI_Init`
    /// fills; this is the same starting point, with owned fields empty and the
    /// per-file sub-structs at their own Raven initializers.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:875`
    fn default() -> Self {
        UiWorld {
            bg_state: BgState::with_pool_size(MAX_POOL_SIZE_UI, BgHost::Ui),
            newHighScoreTime: 0,
            newBestTime: 0,
            showPostGameTime: 0,
            newHighScore: false,
            demoAvailable: false,
            soundHighScore: false,
            characterCount: 0,
            botIndex: 0,
            aliasList: Vec::new(),
            teamList: Vec::new(),
            gameTypes: Vec::new(),
            joinGameTypes: Vec::new(),
            redBlue: 0,
            teamIndex: 0,
            playerRefresh: 0,
            playerIndex: 0,
            playerNumber: 0,
            teamLeader: false,
            playerNames: Vec::new(),
            teamNames: Vec::new(),
            teamClientNums: Vec::new(),
            playerIndexes: Vec::new(),
            mapList: Vec::new(),
            tierList: Vec::new(),
            skillIndex: 0,
            modList: Vec::new(),
            modIndex: 0,
            demoList: Vec::new(),
            demoIndex: 0,
            movieList: Vec::new(),
            movieIndex: 0,
            previewMovie: 0,
            scrolltext: String::new(),
            scrolltextLine: Vec::new(),
            serverStatus: ServerStatus::default(),
            serverStatusAddress: String::new(),
            serverStatusInfo: ServerStatusInfo::default(),
            nextServerStatusRefresh: 0,
            pendingServerStatus: PendingServerStatus::default(),
            findPlayerName: String::new(),
            foundPlayerServerAddresses: Vec::new(),
            foundPlayerServerNames: Vec::new(),
            currentFoundPlayerServer: 0,
            numFoundPlayerServers: 0,
            nextFindPlayerRefresh: 0,
            currentCrosshair: 0,
            startPostGameTime: 0,
            newHighScoreSound: 0,
            q3HeadNames: Vec::new(),
            q3HeadIcons: Vec::new(),
            q3SelectedHead: 0,
            forceConfigSelected: 0,
            forceConfigNames: Vec::new(),
            forceConfigSide: Vec::new(),
            forceConfigDarkIndexBegin: 0,
            forceConfigLightIndexBegin: 0,
            effectsColor: 0,
            inGameLoad: false,
            playerSpecies: Vec::new(),
            playerSpeciesIndex: 0,
            movesTitleIndex: 0,
            movesBaseAnim: String::new(),
            moveAnimTime: 0,
            languageCount: 0,
            languageCountIndex: 0,
            cvars: UiCvars::default(),
            force: UiForceState::default(),
            saber: UiSaberState::default(),
            gameinfo: UiGameinfoState::default(),
            main: UiMainState::default(),
            scratch: UiScratch::default(),
        }
    }
}
