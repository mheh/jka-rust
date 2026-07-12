//! Ghoul2 performance-analysis feature guard.
//!
//! Sourced from `game/q_shared.h` (not `G2.h`, unlike this module's other
//! consts) but colocated here since its only consumers are ghoul2/renderer
//! allocation counters (`tr_ghoul2.cpp`).
//!
//! Source: `oracle/codemp/game/q_shared.h:45-46`

/// Raven `G2_PERFORMANCE_ANALYSIS` — guards the ghoul2 allocation-counter
/// instrumentation (`g_Ghoul2Allocations`/`g_G2ServerAlloc`/`g_G2ClientAlloc`).
/// Ported as `bool` since Raven never gives it a value, only tests it with
/// `#ifdef`; defined whenever `FINAL_BUILD` is undefined, matching this
/// project's referee-build convention (`FINAL_BUILD`/`Q3_VM` both undefined,
/// see `crates/mp/game/src/g_utils.rs`).
///
/// Source: `oracle/codemp/game/q_shared.h:45-46`
pub const G2_PERFORMANCE_ANALYSIS: bool = true;
