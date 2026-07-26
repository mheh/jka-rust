//! `world` — the ui module island: the owned [`UiState`] aggregate ([`UiWorld`]
//! plus the hoisted `menus`/`uiDC`), its per-source-file state sub-structs, and
//! the [`UiContext`] dispatch receiver (DEC-36 D1/D4, DEC-38 ruling 1).

pub mod ui_context;
pub mod ui_cvars;
pub mod ui_force_state;
pub mod ui_gameinfo_state;
pub mod ui_main_state;
pub mod ui_saber_state;
pub mod ui_scratch;
pub mod ui_state;
pub mod ui_world;

pub use ui_context::UiContext;
pub use ui_cvars::UiCvars;
pub use ui_force_state::UiForceState;
pub use ui_gameinfo_state::UiGameinfoState;
pub use ui_main_state::UiMainState;
pub use ui_saber_state::UiSaberState;
pub use ui_scratch::UiScratch;
pub use ui_state::UiState;
pub use ui_world::UiWorld;
