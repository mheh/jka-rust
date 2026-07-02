//! Multiplayer common code shared within MP modules, but not yet proven common with SP.

pub mod bg;
pub mod botlib;
pub mod cgame;
pub mod game;
pub mod gentity;
pub mod qcommon;
pub mod trace_t;
pub mod ui;

pub use gentity::gentity_t;
