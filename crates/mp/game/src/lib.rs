//! `mp_game` — MP server-game module (`g_*`), the game-side of the QVM boundary.
//!
//! The core `g_local.h` data model is ported (client/entity/level + AI/teams/npc
//! types), verified against oracle with size/offset asserts.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
// The port reaches world state through raw pointers (`(*ctx.world_raw()).…`).
// Container indexing on those paths implicitly autorefs through the deref, the exact pattern this deny-by-default lint flags.
// The refs are intentional: single-writer world, seam-confined unsafe.
// Silencing beats 130 noisy explicit-ref rewrites.
// Revisit when the safe-state migration lands.
#![deny(dangerous_implicit_autorefs)]

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

// jampgame function skeletons and generated boilerplate.
pub mod AnimalNPC;
pub mod FighterNPC;
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
pub mod ai_main_consts;
pub mod ai_util;
pub mod ai_wpnav;
pub mod anim_table;
pub mod bg_channel;
pub mod c_format;
pub mod com_boundary;
pub mod cstr_util;
pub mod ent_fn_enums;
pub mod ent_id;
pub mod g_ICARUScb;
pub mod g_active;
pub mod g_arenas;
pub mod g_bot;
pub mod g_client;
pub mod g_cmds;
pub mod g_combat;
pub mod g_exphysics;
pub mod g_icarus_set_type;
pub mod g_items;
pub mod g_local_consts;
pub mod g_log;
pub mod g_main;
pub mod g_mem;
pub mod g_misc;
pub mod g_missile;
pub mod g_mover;
pub mod g_nav;
pub mod g_nav_consts;
pub mod g_navnew;
pub mod g_object;
pub mod g_public_consts;
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
pub mod npc_c;
pub mod q_math;
pub mod q_shared;
pub mod q_shared_cvar_flags;
pub mod trap;
pub mod tri_coll_test;
pub mod veh_dispatch;
pub mod w_force;
pub mod w_saber;
pub mod world;

pub use world::{EntityId, GameContext, GameWorld};

// These qshared subsystems live in `mp_qshared`.
// Ported code refers to them as `crate::shared::…` and `crate::trajectory::…`, so the game crate is their logical home at those call sites.
// Re-exporting them under `crate::` resolves the paths without editing each call site.
pub use mp_qshared::common::mp::qcommon::taskID_t;
pub use mp_qshared::shared;
pub use mp_qshared::shared::trajectory;

// The export-command enum re-exports here so the jampgame shell names it through its existing two edges.
// SEAM-D10's exactly-two-edges shell property holds: the shell sees the seam through the logic crate.
pub use mp_abi::game::exports::MpGameExport;

// The per-command vmMain call types the shell's dispatch match names.
// The shell's arms need the C marker and Args types, so they route through the logic crate the same way.
pub mod vmcalls {
    pub use mp_abi::game::vmcalls::BOTAI_START_FRAME::BotAiStartFrame;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_BEGIN::GameClientBegin;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_COMMAND::GameClientCommand;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_CONNECT::GameClientConnect;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_DISCONNECT::GameClientDisconnect;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_THINK::GameClientThink;
    pub use mp_abi::game::vmcalls::GAME_CLIENT_USERINFO_CHANGED::GameClientUserinfoChanged;
    pub use mp_abi::game::vmcalls::GAME_CONSOLE_COMMAND::GameConsoleCommand;
    pub use mp_abi::game::vmcalls::GAME_GETITEMINDEXBYTAG::GameGetitemindexbytag;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_GETFLOAT::GameIcarusGetfloat;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_GETSETIDFORSTRING::GameIcarusGetsetidforstring;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_GETSTRING::GameIcarusGetstring;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_GETTAG::GameIcarusGettag;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_GETVECTOR::GameIcarusGetvector;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_KILL::GameIcarusKill;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_LERP2ANGLES::GameIcarusLerp2Angles;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_LERP2END::GameIcarusLerp2End;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_LERP2ORIGIN::GameIcarusLerp2Origin;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_LERP2POS::GameIcarusLerp2Pos;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_LERP2START::GameIcarusLerp2Start;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_PLAY::GameIcarusPlay;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_PLAYSOUND::GameIcarusPlaysound;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_REMOVE::GameIcarusRemove;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_SET::GameIcarusSet;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_SOUNDINDEX::GameIcarusSoundindex;
    pub use mp_abi::game::vmcalls::GAME_ICARUS_USE::GameIcarusUse;
    pub use mp_abi::game::vmcalls::GAME_INIT::GameInit;
    pub use mp_abi::game::vmcalls::GAME_NAV_CHECKNODEFAILEDFORENT::GameNavChecknodefailedforent;
    pub use mp_abi::game::vmcalls::GAME_NAV_CLEARLOS::GameNavClearlos;
    pub use mp_abi::game::vmcalls::GAME_NAV_CLEARPATHBETWEENPOINTS::GameNavClearpathbetweenpoints;
    pub use mp_abi::game::vmcalls::GAME_NAV_CLEARPATHTOPOINT::GameNavClearpathtopoint;
    pub use mp_abi::game::vmcalls::GAME_NAV_ENTISBREAKABLE::GameNavEntIsBreakable;
    pub use mp_abi::game::vmcalls::GAME_NAV_ENTISDOOR::GameNavEntIsDoor;
    pub use mp_abi::game::vmcalls::GAME_NAV_ENTISREMOVABLEUSABLE::GameNavEntIsRemovableUsable;
    pub use mp_abi::game::vmcalls::GAME_NAV_ENTISUNLOCKEDDOOR::GameNavEntIsUnlockedDoor;
    pub use mp_abi::game::vmcalls::GAME_NAV_FINDCOMBATPOINTWAYPOINTS::GameNavFindcombatpointwaypoints;
    pub use mp_abi::game::vmcalls::GAME_ROFF_NOTETRACK_CALLBACK::GameRoffNotetrackCallback;
    pub use mp_abi::game::vmcalls::GAME_RUN_FRAME::GameRunFrame;
    pub use mp_abi::game::vmcalls::GAME_SHUTDOWN::GameShutdown;
    pub use mp_abi::game::vmcalls::GAME_SPAWN_RMG_ENTITY::GameSpawnRmgEntity;
}
