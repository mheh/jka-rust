//! `UiMainState` — `ui_main.c`'s mutable file-scope globals as one `UiWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::connstate::connstate_t;

/// Raven `#define MAX_SABER_HILTS 64`.
///
/// Source: `oracle/codemp/ui/ui_main.c:6144`
pub const MAX_SABER_HILTS: usize = 64;

/// `ui_main.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned `UiWorld` state).
///
/// PORT-NOTE (not folded in here):
/// * The file's read-only tables — `forcepowerDesc`, `datapadMoveTitleData`,
///   `datapadMoveTitleBaseAnims`, `datapadMoveData`, `serverFilters`,
///   `skillLevels`, `teamArenaGameTypes`, `netnames`, `handicapValues`,
///   `gamecodetoui`/`uitogamecode`, `serverStatusCvars`, `cvarTable` and the
///   `numX` sizeof-counts beside them — are compiled-in data, not state; they
///   land as `const`s beside the functions that read them (§C8).
/// * The hand-maintained animation fork — `UIPAFtextLoaded`, `UIPAFtext`,
///   `uiHumanoidAnimations`, `bgAllAnims`, `uiNumAllAnims` and
///   `UI_ParseAnimationFile` — is dropped: DEC-36 D5 rules that ui reuses
///   `mp_bg`'s animation module instead of Raven's manually synced copy
///   (`ui_main.c:633-643` admits the duplication).
///
/// Source: `oracle/codemp/ui/ui_main.c:628-631,872-894,1236-1241,1437,5408,6146-6147,9594,11036-11037`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMainState {
    /// Raven `char holdSPString[MAX_STRING_CHARS]`.
    /// Source: `oracle/codemp/ui/ui_main.c:872`
    pub holdSPString: String,
    /// Raven `char holdSPString2[MAX_STRING_CHARS]`.
    /// Source: `oracle/codemp/ui/ui_main.c:873`
    pub holdSPString2: String,

    /// Raven `int uiSkinColor` — initialized to `TEAM_FREE`.
    /// Source: `oracle/codemp/ui/ui_main.c:893`
    pub uiSkinColor: c_int,
    /// Raven `int uiHoldSkinColor` — stores the skin color so that in non-team
    /// games, the player screen remembers the team you chose, in case you're
    /// coming back from the force powers screen.
    /// Source: `oracle/codemp/ui/ui_main.c:894`
    pub uiHoldSkinColor: c_int,

    /// Raven `int frameCount`.
    /// Source: `oracle/codemp/ui/ui_main.c:1236`
    pub frameCount: c_int,
    /// Raven `int startTime`.
    /// Source: `oracle/codemp/ui/ui_main.c:1237`
    pub startTime: c_int,

    /// Raven `char parsedFPMessage[1024]`.
    /// Source: `oracle/codemp/ui/ui_main.c:1241`
    pub parsedFPMessage: String,

    /// Raven `char *defaultMenu` — the fallback menu text, NULL until the
    /// default menu file is read.
    /// Source: `oracle/codemp/ui/ui_main.c:1437`
    pub defaultMenu: Option<String>,

    /// Raven `int gUISelectedMap`.
    /// Source: `oracle/codemp/ui/ui_main.c:5408`
    pub gUISelectedMap: c_int,

    /// Raven `char *saberSingleHiltInfo[MAX_SABER_HILTS]`.
    /// Source: `oracle/codemp/ui/ui_main.c:6146`
    pub saberSingleHiltInfo: Vec<String>,
    /// Raven `char *saberStaffHiltInfo[MAX_SABER_HILTS]`.
    /// Source: `oracle/codemp/ui/ui_main.c:6147`
    pub saberStaffHiltInfo: Vec<String>,

    /// Raven `siegeClassDesc_t g_UIClassDescriptions[MAX_SIEGE_CLASSES]` — one
    /// `char desc[4096]` per siege class; the owned `Vec<String>` is indexed by
    /// the same class index.
    /// Source: `oracle/codemp/ui/ui_main.c:628`
    pub g_UIClassDescriptions: Vec<String>,

    /// Raven `siegeTeam_t *siegeTeam1` — a pointer into `mp_bg`'s parsed siege
    /// team table, so the port carries the table index instead (§B5).
    /// Source: `oracle/codemp/ui/ui_main.c:629`
    pub siegeTeam1: Option<usize>,
    /// Raven `siegeTeam_t *siegeTeam2`.
    /// Source: `oracle/codemp/ui/ui_main.c:630`
    pub siegeTeam2: Option<usize>,
    /// Raven `int g_UIGloballySelectedSiegeClass`.
    /// Source: `oracle/codemp/ui/ui_main.c:631`
    pub g_UIGloballySelectedSiegeClass: c_int,
    /// Raven `int g_siegedFeederForcedSet`.
    /// Source: `oracle/codemp/ui/ui_main.c:9594`
    pub g_siegedFeederForcedSet: c_int,

    /// Raven `static connstate_t lastConnState` — the connect screen's
    /// previous state, used to time the download/loading text.
    /// Source: `oracle/codemp/ui/ui_main.c:11036`
    pub lastConnState: connstate_t,
    /// Raven `static char lastLoadingText[MAX_INFO_VALUE]`.
    /// Source: `oracle/codemp/ui/ui_main.c:11037`
    pub lastLoadingText: String,
}

impl Default for UiMainState {
    /// Raven's static initializers: `uiSkinColor`/`uiHoldSkinColor` start at
    /// `TEAM_FREE` (0), `g_UIGloballySelectedSiegeClass` at -1, everything else
    /// zeroed.
    ///
    /// Source: `oracle/codemp/ui/ui_main.c:631,893-894`
    fn default() -> Self {
        UiMainState {
            holdSPString: String::new(),
            holdSPString2: String::new(),
            uiSkinColor: 0,
            uiHoldSkinColor: 0,
            frameCount: 0,
            startTime: 0,
            parsedFPMessage: String::new(),
            defaultMenu: None,
            gUISelectedMap: 0,
            saberSingleHiltInfo: Vec::new(),
            saberStaffHiltInfo: Vec::new(),
            g_UIClassDescriptions: Vec::new(),
            siegeTeam1: None,
            siegeTeam2: None,
            g_UIGloballySelectedSiegeClass: -1,
            g_siegedFeederForcedSet: 0,
            lastConnState: connstate_t::CA_UNINITIALIZED,
            lastLoadingText: String::new(),
        }
    }
}
