//! `mp_engine_core` — the per-mode engine-island facade: the aggregate `Engine`
//! (STATE-D5) plus the `com_*` lifecycle surface (LIFE-D2). The one crate that
//! depends on all engine subcrates, so it can name Server/Client/etc. as fields
//! and `com_frame` can call `SV_Frame`/`CL_Frame`.

#![allow(non_camel_case_types, non_snake_case)]

pub mod engine;
pub mod game_version_consts;
pub mod lifecycle;
pub mod sv_init_game_progs;

pub use engine::Engine;
pub use lifecycle::{com_frame, com_init, com_shutdown, sys_error, sys_milliseconds};
pub use sv_init_game_progs::sv_init_game_progs;
