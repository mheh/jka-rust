//! MP spawn-var limits from `bg_public.h`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `MAX_SPAWN_VARS`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:16`
pub const MAX_SPAWN_VARS: usize = 64;

/// Raven `MAX_SPAWN_VARS_CHARS`.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:17`
pub const MAX_SPAWN_VARS_CHARS: usize = 4096;
