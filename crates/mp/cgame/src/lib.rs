//! `mp_cgame` — the MP client-game module (`cgame`), Raven's
//! `oracle/codemp/cgame/`. One of the three loadable MP DLLs (`jampgame`,
//! `cgame`, `ui`); it renders the client's view of the world, predicts the
//! local player, and draws the HUD.
//!
//! The cgame track (task #46) runs the stage skeleton DEC-45 ratified,
//! mirroring the ui track's U0-U6:
//!
//! - **C0** cgpackets tooling, **C1** crate/abi reconciliation audit
//! - **C2** root-type sit-down — settled as DEC-46: `CgWorld` is the three-way
//!   `cg`/`cgs`/`entities` spine, `CEntity` is owned with resolution enums,
//!   effect pools are generation-counted slabs with an explicit LRU queue,
//!   fn-ptr tables become closed enums, and `cg.sharedBuffer` is a pinned
//!   `Box<[u8; 2048]>`
//! - **C3** the trap layer — [`trap`], this stage: all 215 `trap_*` wrappers of
//!   `oracle/codemp/cgame/cg_syscalls.c`, on ui's `trap.rs` pattern (DEC-45.3:
//!   the module track uses a syscall-shaped trap layer, forced by the OpenJK
//!   ABI; `EngineHostView` stays the in-engine convention)
//! - **C4** the `CgGameCallbacks` implementor, **C5** transcription waves,
//!   **C6** gates (live rounds under `openjk.app` + the demo referee)
//!
//! The cgame root types [`local`] and [`lights`] landed under C1/C2; [`world`]
//! is the C5 skeleton — [`world::CgWorld`], the effect pools, the cvar mirrors
//! and one state sub-struct per `.c` file with statics to fold. The per-TU
//! function modules below are empty until the waves fill them.

pub mod bg_channel;
pub mod cg_consolecmds;
pub mod cg_draw;
pub mod cg_drawtools;
pub mod cg_effects;
pub mod cg_ents;
pub mod cg_event;
pub mod cg_info;
pub mod cg_light;
pub mod cg_localents;
pub mod cg_main;
pub mod cg_marks;
pub mod cg_new_draw;
pub mod cg_players;
pub mod cg_playerstate;
pub mod cg_predict;
pub mod cg_saga;
pub mod cg_scoreboard;
pub mod cg_servercmds;
pub mod cg_snapshot;
pub mod cg_strap;
pub mod cg_turret;
pub mod cg_view;
pub mod cg_weaponinit;
pub mod cg_weapons;
pub mod fx_blaster;
pub mod fx_bowcaster;
pub mod fx_bryarpistol;
pub mod fx_demp2;
pub mod fx_disruptor;
pub mod fx_flechette;
pub mod fx_force;
pub mod fx_heavyrepeater;
pub mod fx_rocketlauncher;
pub mod lights;
pub mod local;
pub mod trap;
pub mod vehicle_npc;
pub mod world;
