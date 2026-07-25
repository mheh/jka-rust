//! Raven force-mastery-level constants.

use core::ffi::c_int;

// Raven's force-mastery anonymous enum → int-wide consts per the enum-vs-alias
// rule (anonymous enum → `const`s). Canonical home per DEC-32 (was privately
// duplicated in `mp_game::w_force` and `mp_ui::world::ui_force_state`).
/// Raven `FORCE_MASTERY_UNINITIATED`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_UNINITIATED: c_int = 0;
/// Raven `FORCE_MASTERY_INITIATE`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_INITIATE: c_int = 1;
/// Raven `FORCE_MASTERY_PADAWAN`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_PADAWAN: c_int = 2;
/// Raven `FORCE_MASTERY_JEDI`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_JEDI: c_int = 3;
/// Raven `FORCE_MASTERY_JEDI_GUARDIAN`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_JEDI_GUARDIAN: c_int = 4;
/// Raven `FORCE_MASTERY_JEDI_ADEPT`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_JEDI_ADEPT: c_int = 5;
/// Raven `FORCE_MASTERY_JEDI_KNIGHT`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_JEDI_KNIGHT: c_int = 6;
/// Raven `FORCE_MASTERY_JEDI_MASTER`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const FORCE_MASTERY_JEDI_MASTER: c_int = 7;
/// Raven `NUM_FORCE_MASTERY_LEVELS`.
///
/// Source: `oracle/codemp/game/bg_public.h:383-393`
pub const NUM_FORCE_MASTERY_LEVELS: c_int = 8;
