//! MP `bg_local.h` — pmove-internal shared types not exposed via `bg_public.h`.
//!
//! //TODO: Port module mp_bg::local — subsystem dir only; porters add flat
//! `<type>.rs` files here as types are ported.
//! Source: `oracle/oracle/codemp/game/bg_local.h`
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod bg_custom_siege_sound_names;
pub mod bg_toggleable_surfaces;
pub mod eventnames;
pub mod force_levels;
pub mod force_power_needed;
pub mod pml_t;

pub use bg_custom_siege_sound_names::bg_customSiegeSoundNames;
pub use bg_toggleable_surfaces::{bgToggleableSurfaceDebris, bgToggleableSurfaces};
pub use eventnames::eventnames;
pub use force_levels::{forceJumpHeight, forceJumpStrength, forceSpeedLevels};
pub use force_power_needed::forcePowerNeeded;
