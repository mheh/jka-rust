//! MP `bg_public.h` misc `#define` constants and the `CS_*` configstring
//! index table.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/oracle/codemp/game/bg_public.h:37-90`

use core::ffi::c_int;

/// Raven `SCORE_NOT_PRESENT` — for the `CS_SCORES[12]` when only one player
/// is present.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:37`
pub const SCORE_NOT_PRESENT: c_int = -9999;

/// Raven `VOTE_TIME` — 30 seconds before vote times out.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:39`
pub const VOTE_TIME: c_int = 30000;

/// Raven `RANK_TIED_FLAG` — flag OR'd onto `PERS_RANK` when a player is tied.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:33`
pub const RANK_TIED_FLAG: c_int = 0x4000;

// Config strings are a general means of communicating variable length
// strings from the server to all connected clients.
//
// CS_SERVERINFO and CS_SYSTEMINFO are defined in q_shared.h.

/// Source: `oracle/oracle/codemp/game/bg_public.h:59`
pub const CS_MUSIC: c_int = 2;
/// Source: `oracle/oracle/codemp/game/bg_public.h:60`
pub const CS_MESSAGE: c_int = 3;
/// Source: `oracle/oracle/codemp/game/bg_public.h:61`
pub const CS_MOTD: c_int = 4;
/// Raven: server time when the match will be restarted.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:62`
pub const CS_WARMUP: c_int = 5;
/// Source: `oracle/oracle/codemp/game/bg_public.h:63`
pub const CS_SCORES1: c_int = 6;
/// Source: `oracle/oracle/codemp/game/bg_public.h:64`
pub const CS_SCORES2: c_int = 7;
/// Source: `oracle/oracle/codemp/game/bg_public.h:65`
pub const CS_VOTE_TIME: c_int = 8;
/// Source: `oracle/oracle/codemp/game/bg_public.h:66`
pub const CS_VOTE_STRING: c_int = 9;
/// Source: `oracle/oracle/codemp/game/bg_public.h:67`
pub const CS_VOTE_YES: c_int = 10;
/// Source: `oracle/oracle/codemp/game/bg_public.h:68`
pub const CS_VOTE_NO: c_int = 11;

/// Source: `oracle/oracle/codemp/game/bg_public.h:70`
pub const CS_TEAMVOTE_TIME: c_int = 12;
/// Source: `oracle/oracle/codemp/game/bg_public.h:71`
pub const CS_TEAMVOTE_STRING: c_int = 14;
/// Source: `oracle/oracle/codemp/game/bg_public.h:72`
pub const CS_TEAMVOTE_YES: c_int = 16;
/// Source: `oracle/oracle/codemp/game/bg_public.h:73`
pub const CS_TEAMVOTE_NO: c_int = 18;

/// Source: `oracle/oracle/codemp/game/bg_public.h:75`
pub const CS_GAME_VERSION: c_int = 20;
/// Raven: so the timer only shows the current level.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:76`
pub const CS_LEVEL_START_TIME: c_int = 21;
/// Raven: when 1, fraglimit/timelimit has been hit and intermission will
/// start in a second or two.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:77`
pub const CS_INTERMISSION: c_int = 22;
/// Raven: string indicating flag status in CTF.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:78`
pub const CS_FLAGSTATUS: c_int = 23;
/// Source: `oracle/oracle/codemp/game/bg_public.h:79`
pub const CS_SHADERSTATE: c_int = 24;
/// Source: `oracle/oracle/codemp/game/bg_public.h:80`
pub const CS_BOTINFO: c_int = 25;

/// Raven: string of 0's and 1's that tell which items are present.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:82`
pub const CS_ITEMS: c_int = 27;

/// Raven: current jedi master.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:84`
pub const CS_CLIENT_JEDIMASTER: c_int = 28;
/// Raven: current duel round winner - needed for printing at top of
/// scoreboard.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:85`
pub const CS_CLIENT_DUELWINNER: c_int = 29;
/// Raven: client numbers for both current duelists. Needed for a number of
/// client-side things.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:86`
pub const CS_CLIENT_DUELISTS: c_int = 30;
/// Raven: nmckenzie: DUEL_HEALTH. Hopefully adding this cs is safe and good?
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:87`
pub const CS_CLIENT_DUELHEALTHS: c_int = 31;
/// Source: `oracle/oracle/codemp/game/bg_public.h:88`
pub const CS_GLOBAL_AMBIENT_SET: c_int = 32;

/// Source: `oracle/oracle/codemp/game/bg_public.h:90`
pub const CS_AMBIENT_SET: c_int = 37;

// Raven computes the rest of the table from running offsets
// (`CS_X = CS_PREV + MAX_PREV`); MAX_AMBIENT_SETS=64, MAX_MODELS=512,
// MAX_SOUNDS=256, MAX_ICONS=64, MAX_CLIENTS=32, MAX_G2BONES=64,
// MAX_LOCATIONS=64, MAX_FX=64, MAX_LIGHT_STYLES=64, MAX_TERRAINS=1,
// MAX_SUB_BSP=32 (all from `q_shared.h`/`bg_public.h`/`cgs_t.rs`), folded
// into the literals below since `bg` cannot depend on the `game`/`cgame`
// crates that own several of those MAX_* consts.
//
// Source: `oracle/oracle/codemp/game/bg_public.h:92-120`
pub const CS_SIEGE_STATE: c_int = CS_AMBIENT_SET + 64;
pub const CS_SIEGE_OBJECTIVES: c_int = CS_SIEGE_STATE + 1;
pub const CS_SIEGE_TIMEOVERRIDE: c_int = CS_SIEGE_OBJECTIVES + 1;
pub const CS_SIEGE_WINTEAM: c_int = CS_SIEGE_TIMEOVERRIDE + 1;
pub const CS_SIEGE_ICONS: c_int = CS_SIEGE_WINTEAM + 1;

pub const CS_MODELS: c_int = CS_SIEGE_ICONS + 1;
/// Raven: skybox info.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:99`
pub const CS_SKYBOXORG: c_int = CS_MODELS + 512;
pub const CS_SOUNDS: c_int = CS_SKYBOXORG + 1;
pub const CS_ICONS: c_int = CS_SOUNDS + 256;
pub const CS_PLAYERS: c_int = CS_ICONS + 64;
pub const CS_G2BONES: c_int = CS_PLAYERS + 32;
pub const CS_LOCATIONS: c_int = CS_G2BONES + 64;
pub const CS_PARTICLES: c_int = CS_LOCATIONS + 64;
pub const CS_EFFECTS: c_int = CS_PARTICLES + 64;
/// Source: `oracle/oracle/codemp/game/bg_public.h:114`
pub const CS_LIGHT_STYLES: c_int = CS_EFFECTS + 64;

/// Source: `oracle/oracle/codemp/game/bg_public.h:117`
pub const CS_TERRAINS: c_int = CS_LIGHT_STYLES + (64 * 3);
pub const CS_BSP_MODELS: c_int = CS_TERRAINS + 1;

pub const CS_MAX: c_int = CS_BSP_MODELS + 32;

/// Raven `MAX_ICONS` — max registered icons you can have per map.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:2022`
pub const MAX_ICONS: c_int = 64;

/// Raven `MAX_G2BONES` (changed from `MAX_CHARSKINS`, value still equal).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:2030`
pub const MAX_G2BONES: c_int = 64;

/// Raven `MAX_SUB_BSP`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:2025`
pub const MAX_SUB_BSP: c_int = 32;
