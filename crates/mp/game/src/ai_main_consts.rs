//! MP `ai_main.h`/`ai_main.c` plain `#define` constants: `LEVELFLAG_*` bit
//! flags (`level.levelFlags` / `gLevelFlags` AI hint bits) and bot AI
//! range/distance/interval tunables.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//!
//! Source: `oracle/codemp/game/ai_main.h:40-79`,
//! `oracle/codemp/game/ai_main.c:43`

use core::ffi::c_int;

/// Raven `LEVELFLAG_NOPOINTPREDICTION` — don't take waypoint beyond current
/// into account when adjusting path view angles.
///
/// Source: `oracle/codemp/game/ai_main.h:40`
pub const LEVELFLAG_NOPOINTPREDICTION: c_int = 1;

/// Raven `LEVELFLAG_IGNOREINFALLBACK` — ignore enemies when in a fallback
/// navigation routine.
///
/// Source: `oracle/codemp/game/ai_main.h:41`
pub const LEVELFLAG_IGNOREINFALLBACK: c_int = 2;

/// Raven `LEVELFLAG_IMUSTNTRUNAWAY` — don't be scared.
///
/// Source: `oracle/codemp/game/ai_main.h:42`
pub const LEVELFLAG_IMUSTNTRUNAWAY: c_int = 4;

/// Raven `WP_KEEP_FLAG_DIST`.
///
/// Source: `oracle/codemp/game/ai_main.h:44`
pub const WP_KEEP_FLAG_DIST: c_int = 128;

/// Raven `MELEE_ATTACK_RANGE`.
///
/// Source: `oracle/codemp/game/ai_main.h:51`
pub const MELEE_ATTACK_RANGE: c_int = 256;

/// Raven `SABER_ATTACK_RANGE`.
///
/// Source: `oracle/codemp/game/ai_main.h:52`
pub const SABER_ATTACK_RANGE: c_int = 128;

/// Raven `BOT_WPTOUCH_DISTANCE`.
///
/// Source: `oracle/codemp/game/ai_main.h:56`
pub const BOT_WPTOUCH_DISTANCE: c_int = 32;

/// Raven `BOT_PLANT_DISTANCE` — plant if within this radius from the last
/// spotted enemy position.
///
/// Source: `oracle/codemp/game/ai_main.h:61`
pub const BOT_PLANT_DISTANCE: c_int = 256;

/// Raven `BOT_PLANT_INTERVAL` — only plant once per 15 seconds at max.
///
/// Source: `oracle/codemp/game/ai_main.h:62`
pub const BOT_PLANT_INTERVAL: c_int = 15000;

/// Raven `BOT_PLANT_BLOW_DISTANCE` — blow det packs if enemy is within this
/// radius and I am further away than the enemy.
///
/// Source: `oracle/codemp/game/ai_main.h:63`
pub const BOT_PLANT_BLOW_DISTANCE: c_int = 256;

/// Raven `BOT_MAX_WEAPON_CHASE_TIME`.
///
/// Source: `oracle/codemp/game/ai_main.h:66`
pub const BOT_MAX_WEAPON_CHASE_TIME: c_int = 15000;

/// Raven `BOT_MAX_WEAPON_GATHER_TIME`.
///
/// Source: `oracle/codemp/game/ai_main.h:65`
pub const BOT_MAX_WEAPON_GATHER_TIME: c_int = 1000;

/// Raven `BOT_MAX_WEAPON_CHASE_CTF`.
///
/// Source: `oracle/codemp/game/ai_main.h:68`
pub const BOT_MAX_WEAPON_CHASE_CTF: c_int = 5000;

/// Raven `BOT_MIN_SIEGE_GOAL_SHOOT`.
///
/// Source: `oracle/codemp/game/ai_main.h:70`
pub const BOT_MIN_SIEGE_GOAL_SHOOT: c_int = 1024;

/// Raven `BOT_MIN_SIEGE_GOAL_TRAVEL`.
///
/// Source: `oracle/codemp/game/ai_main.h:71`
pub const BOT_MIN_SIEGE_GOAL_TRAVEL: c_int = 128;

/// Raven `BASE_FLAGWAIT_DISTANCE`.
///
/// Source: `oracle/codemp/game/ai_main.h:74`
pub const BASE_FLAGWAIT_DISTANCE: c_int = 256;

/// Raven `BOT_FLAG_GET_DISTANCE`.
///
/// Source: `oracle/codemp/game/ai_main.h:77`
pub const BOT_FLAG_GET_DISTANCE: c_int = 256;

/// Raven `BOT_SABER_THROW_RANGE`.
///
/// Source: `oracle/codemp/game/ai_main.h:79`
pub const BOT_SABER_THROW_RANGE: c_int = 800;

/// Raven `BOT_THINK_TIME` — bot think interval (0, i.e. re-evaluated every
/// server frame).
///
/// Source: `oracle/codemp/game/ai_main.c:43`
pub const BOT_THINK_TIME: c_int = 0;

/// Raven `MAX_CHICKENWUSS_TIME` — wait 10 secs between checking which
/// run-away path to take.
///
/// Source: `oracle/codemp/game/ai_main.h:53`
pub const MAX_CHICKENWUSS_TIME: c_int = 10000;

/// Raven `BOT_RUN_HEALTH`.
///
/// Source: `oracle/codemp/game/ai_main.h:55`
pub const BOT_RUN_HEALTH: c_int = 40;

/// Raven `ENEMY_FORGET_MS` — if our enemy isn't visible within this many ms
/// (aprx 10sec) then "forget" about him and treat him like every other
/// threat, but still look for more immediate threats while main enemy is not
/// visible.
///
/// Source: `oracle/codemp/game/ai_main.h:57`
pub const ENEMY_FORGET_MS: c_int = 10000;

/// Raven `BASE_GUARD_DISTANCE` — guarding the flag.
///
/// Source: `oracle/codemp/game/ai_main.h:73`
pub const BASE_GUARD_DISTANCE: c_int = 256;

/// Raven `BASE_GETENEMYFLAG_DISTANCE` — waiting around to get the enemy's
/// flag.
///
/// Source: `oracle/codemp/game/ai_main.h:75`
pub const BASE_GETENEMYFLAG_DISTANCE: c_int = 256;

/// Raven botlib.h print-level `#define`s (not an enum), so §C8 makes them
/// `const`s directly.
///
/// Source: `oracle/codemp/game/botlib.h:39-43`
pub const PRT_MESSAGE: c_int = 1;
/// Raven `PRT_WARNING`. Source: `oracle/codemp/game/botlib.h:40`
pub const PRT_WARNING: c_int = 2;
/// Raven `PRT_ERROR`. Source: `oracle/codemp/game/botlib.h:41`
pub const PRT_ERROR: c_int = 3;
/// Raven `PRT_FATAL`. Source: `oracle/codemp/game/botlib.h:42`
pub const PRT_FATAL: c_int = 4;
/// Raven `PRT_EXIT`. Source: `oracle/codemp/game/botlib.h:43`
pub const PRT_EXIT: c_int = 5;

/// Raven `STRAFEAROUND_RIGHT`/`STRAFEAROUND_LEFT` — file-local `#define`s in
/// `ai_main.c` under `BOT_STRAFE_AVOIDANCE`, no ported home in a header.
///
/// Source: `oracle/codemp/game/ai_main.c:1551-1552`
pub const STRAFEAROUND_RIGHT: c_int = 1;
/// Raven `STRAFEAROUND_LEFT`. Source: `oracle/codemp/game/ai_main.c:1552`
pub const STRAFEAROUND_LEFT: c_int = 2;

/// Raven `ctfStateDescriptions` — CTF state description strings, indexed by
/// `ctfState_t`.
///
/// Source: `oracle/codemp/game/ai_main.c:106-113`
pub static ctfStateDescriptions: [&core::ffi::CStr; 6] = [
    c"I'm not occupied",
    c"I'm attacking the enemy's base",
    c"I'm defending our base",
    c"I'm getting our flag back",
    c"I'm escorting our flag carrier",
    c"I've got the enemy's flag",
];

/// Raven `siegeStateDescriptions` — siege state description strings.
///
/// Source: `oracle/codemp/game/ai_main.c:115-119`
pub static siegeStateDescriptions: [&core::ffi::CStr; 3] = [
    c"I'm not occupied",
    c"I'm attemtping to complete the current objective",
    c"I'm preventing the enemy from completing their objective",
];

/// Raven `teamplayStateDescriptions` — teamplay state description strings.
///
/// Source: `oracle/codemp/game/ai_main.c:121-126`
pub static teamplayStateDescriptions: [&core::ffi::CStr; 4] = [
    c"I'm not occupied",
    c"I'm following my squad commander",
    c"I'm assisting my commanding",
    c"I'm attempting to regroup and form a new squad",
];
