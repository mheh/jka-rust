//! `mod gi` — the SP module-side outbound-call wrappers (SEAM-D2/D13 call-site
//! conventions). One thin fn per outbound call, the SP mirror of MP's
//! `mod trap`:
//!
//! ```ignore
//! pub fn X(engine: &game_import_t, args: …) -> … {
//!     // resolve the corresponding game_import_t fn pointer directly (SEAM-D2)
//! }
//! ```
//!
//! SP keeps its native table wire — `gi::X` resolves to the `game_import_t`
//! fn-pointer directly, NO word-encoding layer (SEAM-D2); call sites read
//! `gi::X(engine, args)`, uniform with MP's `trap::X(engine, args)`. `engine`
//! is the stored import table threaded inward from the jagame shell (the SP
//! engine handle, settled mapping 2026-07-03). SP needs no select crate: the
//! binding is always native (SEAM-D13).
//!
//! The per-call wrappers themselves are logic-port throughput, not frozen
//! skeleton surface — this module fixes only the import + the shape.

#[allow(unused_imports)]
use sp_abi::game::public::game_import_t::game_import_t;

//TODO: Port gi::* outbound-call wrappers (one per game_import_t member)
// Source: oracle/oracle/code/game/g_public.h:168-471
