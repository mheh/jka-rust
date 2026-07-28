//! `world` — the cgame module island: the owned [`CgState`] aggregate
//! ([`CgWorld`] plus the hoisted `menus`/`cgDC`), its per-source-file state
//! sub-structs, the DEC-46.3 effect pools, and the [`CgContext`] dispatch
//! receiver.
//!
//! One `Cg<Tu>State` sub-struct exists per `cg_*.c` that has mutable file-scope
//! globals, and every one is already wired as a [`CgWorld`] field, so a C5 wave
//! transcriber only ever opens its own TU's two files — the function file and
//! its state file. The eleven TUs with nothing to fold (`cg_consolecmds`,
//! `cg_drawtools`, `cg_event`, `cg_info`, `cg_newDraw`, `cg_playerstate`,
//! `cg_servercmds`, `cg_snapshot`, `cg_strap`, `cg_turret`, `cg_weaponinit`)
//! and the nine `fx_*.c` files get no state struct — their only file-scope
//! `static`s are read-only tables and return buffers, which land as `const`s
//! and owned returns beside the functions (§C7/§C8). `cg_localents.c` gets none
//! either: its three globals ARE the pool, and the pool is a `CgWorld` field.
//!
//! Source: `docs/decisions.md` DEC-46

pub mod cg_context;
pub mod cg_cvars;
pub mod cg_display_context;
pub mod cg_draw_state;
pub mod cg_effects_state;
pub mod cg_ents_state;
pub mod cg_light_state;
pub mod cg_main_state;
pub mod cg_marks_state;
pub mod cg_players_state;
pub mod cg_predict_state;
pub mod cg_saga_state;
pub mod cg_scoreboard_state;
pub mod cg_state;
pub mod cg_view_state;
pub mod cg_weapons_state;
pub mod cg_world;
pub mod effect_handle;
pub mod effect_pool;

pub use cg_context::CgContext;
pub use cg_cvars::CgCvars;
pub use cg_draw_state::CgDrawState;
pub use cg_effects_state::CgEffectsState;
pub use cg_ents_state::CgEntsState;
pub use cg_light_state::CgLightState;
pub use cg_main_state::CgMainState;
pub use cg_marks_state::CgMarksState;
pub use cg_players_state::CgPlayersState;
pub use cg_predict_state::CgPredictState;
pub use cg_saga_state::CgSagaState;
pub use cg_scoreboard_state::CgScoreboardState;
pub use cg_state::CgState;
pub use cg_view_state::CgViewState;
pub use cg_weapons_state::CgWeaponsState;
pub use cg_world::CgWorld;
pub use effect_handle::EffectHandle;
pub use effect_pool::EffectPool;
