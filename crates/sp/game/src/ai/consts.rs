//! SP group-AI limit constants from `ai.h` (identical values to MP).

/// Raven SP `MAX_FRAME_GROUPS`.
///
/// Source: `oracle/code/game/ai.h:94`
pub const MAX_FRAME_GROUPS: usize = 32;

/// Raven SP `MAX_GROUP_MEMBERS`.
///
/// Source: `oracle/code/game/ai.h:104`
pub const MAX_GROUP_MEMBERS: usize = 32;

/// Raven SP `NUM_SQUAD_STATES` — count from the anonymous `SQUAD_*` enum.
///
/// Source: `oracle/code/game/ai.h:18-28`
pub const NUM_SQUAD_STATES: usize = 7;
