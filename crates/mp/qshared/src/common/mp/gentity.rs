//! MP `gentity_t` game-private constants and typedefs from Raven
//! `codemp/game/g_local.h` / `g_public.h`.
//!
//! The `gentity_t` struct itself moved to `mp_game` (`crate::entity::gentity`)
//! per DEC-26 — the abi tier now carries entity pointers opaquely as
//! `gentity_s`. The constants and typedefs below stay here at the shared tier
//! because other crates (`mp_game`, the server, ICARUS, the nav) import them.
//!
//! Type declaration source: `oracle/codemp/game/g_local.h:16`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;
///`oracle/codemp/game/g_public.h:623-638`
pub const NUM_TIDS: usize = 10;
pub const NUM_BSETS: usize = 17;
pub const BSET_FIRST: usize = 0;

/// `oracle/codemp/game/g_public.h:673`
pub const MAX_FAILED_NODES: usize = 8;

/// `oracle/codemp/game/g_local.h:99-123`
pub const HL_MAX: usize = 23;

/// Raven MP `moverState_t`.
/// `oracle/codemp/game/g_local.h:89-94`
pub type moverState_t = c_int;

pub const MOVER_POS1: moverState_t = 0;
pub const MOVER_POS2: moverState_t = 1;
pub const MOVER_1TO2: moverState_t = 2;
pub const MOVER_2TO1: moverState_t = 3;

/// Raven MP `material_t`.
/// `oracle/codemp/game/q_shared.h:990`
pub type material_t = c_int;

/// Raven `material_e` variants
/// `oracle/codemp/game/q_shared.h:967-987`
pub const MAT_METAL: material_t = 0; // scorched blue-grey metal
pub const MAT_GLASS: material_t = 1; // not a real chunk type, just plays an effect with glass sprites
pub const MAT_ELECTRICAL: material_t = 2; // sparks only
pub const MAT_ELEC_METAL: material_t = 3; // sparks/electrical type metal
pub const MAT_DRK_STONE: material_t = 4; // brown
pub const MAT_LT_STONE: material_t = 5; // tan
pub const MAT_GLASS_METAL: material_t = 6; // glass sprites and METAl chunk
pub const MAT_METAL2: material_t = 7; // electrical metal type
pub const MAT_NONE: material_t = 8; // no chunks
pub const MAT_GREY_STONE: material_t = 9; // grey
pub const MAT_METAL3: material_t = 10; // METAL and METAL2 chunks
pub const MAT_CRATE1: material_t = 11; // yellow multi-colored crate chunks
pub const MAT_GRATE1: material_t = 12; // grate chunks
pub const MAT_ROPE: material_t = 13; // for yavin trial...no chunks, just wispy bits
pub const MAT_CRATE2: material_t = 14; // read multi-colored crate chunks
pub const MAT_WHITE_METAL: material_t = 15; // white angular chunks
pub const MAT_SNOWY_ROCK: material_t = 16; // gray & brown chunks
pub const NUM_MATERIALS: material_t = 17;
