//! `bg_channel` — cgame-tier implementors of the bg channel seam traits
//! (`mp_bg::bg_channel::{BgTraps, GameCallbacks}`). Mirrors
//! `mp_game::bg_channel` / `mp_ui::bg_channel`, one implementor struct per file
//! (porting-rules: one type per file).

pub mod cg_bg_traps;
pub mod cg_game_callbacks;

pub use cg_bg_traps::CgBgTraps;
pub use cg_game_callbacks::CgGameCallbacks;
