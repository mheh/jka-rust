//! Single-player common code shared within SP modules, but not yet proven common with MP.

pub mod bg;
pub mod cgame;
pub mod game;
pub mod gentity;
pub mod qcommon;
pub mod trace_t;
pub mod ui;

pub use gentity::gentity_t;
