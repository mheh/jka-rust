//! `bg_channel` — ui-tier implementors of the bg channel seam traits
//! (`mp_bg::bg_channel::{BgTraps, GameCallbacks}`). Mirrors
//! `mp_game::bg_channel`'s `game_impl` shape, one implementor struct per file
//! (porting-rules: one type per file).

pub mod ui_bg_traps;
pub mod ui_game_callbacks;

pub use ui_bg_traps::UiBgTraps;
pub use ui_game_callbacks::UiGameCallbacks;
