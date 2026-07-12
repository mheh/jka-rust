//! SP spawn-var limits from `g_local.h`.
//!
//! Note: unlike MP (which defines these in `bg_public.h`), SP keeps them in the
//! game-local header, and `MAX_SPAWN_VARS_CHARS` is half MP's.

/// Raven SP `MAX_SPAWN_VARS`.
///
/// Source: `oracle/code/game/g_local.h:143`
pub const MAX_SPAWN_VARS: usize = 64;

/// Raven SP `MAX_SPAWN_VARS_CHARS` — half of MP's 4096.
///
/// Source: `oracle/code/game/g_local.h:144`
pub const MAX_SPAWN_VARS_CHARS: usize = 2048;
