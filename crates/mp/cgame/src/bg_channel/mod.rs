//! `bg_channel` — cgame-tier implementors of the bg channel seam traits
//! (`mp_bg::bg_channel::{BgTraps, GameCallbacks}`). Mirrors
//! `mp_game::bg_channel` / `mp_ui::bg_channel`, one implementor struct per file
//! (porting-rules: one type per file). Stage C4 lands the `GameCallbacks`
//! implementor; the `BgTraps` one follows with the transcription waves.

pub mod cg_game_callbacks;

pub use cg_game_callbacks::CgGameCallbacks;
