//! MP group-AI limit constants from `ai.h`.

/// Raven `MAX_FRAME_GROUPS`.
///
/// Source: `oracle/codemp/game/ai.h:85`
pub const MAX_FRAME_GROUPS: usize = 32;

/// Raven `MAX_GROUP_MEMBERS`.
///
/// Source: `oracle/codemp/game/ai.h:95`
pub const MAX_GROUP_MEMBERS: usize = 32;

/// Raven `NUM_SQUAD_STATES` — count from the anonymous `SQUAD_*` enum.
///
/// Source: `oracle/codemp/game/ai.h:19-29`
pub const NUM_SQUAD_STATES: usize = 7;
