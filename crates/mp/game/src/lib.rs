//! `mp_game` — MP server-game module (`g_*`), the game-side of the QVM boundary.
//!
//! The core `g_local.h` data model is ported (client/entity/level + AI/teams/npc
//! types), verified against oracle with size/offset asserts. //TODO: Port the
//! gameplay logic (g_*.c functions).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod botai;
pub mod client;
pub mod entity;
pub mod g_init_game;
pub mod g_shutdown_game;
pub mod level;
pub mod npc;
pub mod prelude;
pub mod saber;
pub mod say;
pub mod teams;

// --- jampgame function skeletons + generated boilerplate (mega-pass) ---
pub mod AnimalNPC;
pub mod FighterNPC;
pub mod npc_c;
pub mod NPC_AI_Atst;
pub mod NPC_AI_Default;
pub mod NPC_AI_Droid;
pub mod NPC_AI_GalakMech;
pub mod NPC_AI_Grenadier;
pub mod NPC_AI_Howler;
pub mod NPC_AI_ImperialProbe;
pub mod NPC_AI_Interrogator;
pub mod NPC_AI_Jedi;
pub mod NPC_AI_Mark1;
pub mod NPC_AI_Mark2;
pub mod NPC_AI_MineMonster;
pub mod NPC_AI_Rancor;
pub mod NPC_AI_Remote;
pub mod NPC_AI_Seeker;
pub mod NPC_AI_Sentry;
pub mod NPC_AI_Sniper;
pub mod NPC_AI_Stormtrooper;
pub mod NPC_AI_Utils;
pub mod NPC_AI_Wampa;
pub mod NPC_behavior;
pub mod NPC_combat;
pub mod NPC_goal;
pub mod NPC_misc;
pub mod NPC_move;
pub mod NPC_reactions;
pub mod NPC_senses;
pub mod NPC_sounds;
pub mod NPC_spawn;
pub mod NPC_stats;
pub mod NPC_utils;
pub mod SpeederNPC;
pub mod WalkerNPC;
pub mod ai_main;
pub mod ai_util;
pub mod ai_wpnav;
pub mod bg_channel;
pub mod bg_g2_utils;
pub mod bg_lib;
pub mod bg_misc;
pub mod bg_panimate;
pub mod bg_pmove;
pub mod bg_saber;
pub mod bg_saberLoad;
pub mod bg_saga;
pub mod bg_slidemove;
pub mod bg_vehicleLoad;
pub mod ent_fn_enums;
pub mod g_ICARUScb;
pub mod g_active;
pub mod g_arenas;
pub mod g_bot;
pub mod g_client;
pub mod g_cmds;
pub mod g_combat;
pub mod g_exphysics;
pub mod g_items;
pub mod g_log;
pub mod g_main;
pub mod g_mem;
pub mod g_misc;
pub mod g_missile;
pub mod g_mover;
pub mod g_nav;
pub mod g_nav_consts;
pub mod g_public_consts;
pub mod g_navnew;
pub mod g_object;
pub mod g_saga;
pub mod g_session;
pub mod g_spawn;
pub mod g_strap;
pub mod g_svcmds;
pub mod g_target;
pub mod g_team;
pub mod g_timer;
pub mod g_trigger;
pub mod g_turret;
pub mod g_turret_G2;
pub mod g_utils;
pub mod g_vehicleTurret;
pub mod g_vehicles;
pub mod g_weapon;
pub mod game_cvars;
pub mod game_globals;
pub mod q_math;
pub mod q_shared;
pub mod tri_coll_test;
pub mod w_force;
pub mod w_saber;
pub mod trap;
pub mod world;

pub use world::{EntityId, GameContext, GameWorld};

// Pass-3 prep C1 (agenda B6 prelude/re-export fix): crate-root re-exports of the
// qshared subsystems that pass-2 porter bodies spell as `crate::shared::…` /
// `crate::trajectory::…` (the module lives in `mp_qshared`, but the game tier is
// its logical home in those transcriptions). Re-homing under `crate::` resolves
// the absolute-path references without touching each call site.
pub use mp_qshared::shared;
pub use mp_qshared::shared::trajectory;

// The export-command enum, re-exported so the jampgame shell names it through
// its existing two edges (round-7 item 25; SEAM-D10's exactly-two-edges shell
// property stays intact — the shell sees the seam through the logic crate).
pub use mp_abi::game::exports::MpGameExport;

// The per-command vmMain call types the shell's dispatch match names, seen
// through the logic crate on the same item-25 principle (checkpoint-7 finding:
// mechanical extension — the shell's arms need the C marker/Args types).
pub mod vmcalls {
    pub use mp_abi::game::vmcalls::GAME_INIT::GameInit;
    pub use mp_abi::game::vmcalls::GAME_SHUTDOWN::GameShutdown;
}
