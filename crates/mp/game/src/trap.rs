//! `mod trap` — the module-side outbound-call wrappers (SEAM-D13 call-site
//! conventions). One thin non-generic `fn` per outbound call `C`, each of the
//! frozen shape:
//!
//! ```ignore
//! pub fn X(engine: &Engine, args: <C as OutboundSysCall>::Args)
//!     -> <C as OutboundSysCall>::Output
//! {
//!     engine.execute(args)   // Execute<C>::execute for the selected backend
//! }
//! ```
//!
//! `Engine` is the `mp_engine_select` per-build alias, imported unchanged so this
//! module carries NO cfg and NO Cargo feature (SEAM-D10/D13). MP call sites read
//! `trap::X(engine, args)`.
//!
//! The per-call wrappers themselves are logic-port work (one per abi-traps row),
//! not frozen skeleton surface — this module fixes only the import + the shape.

#[allow(unused_imports)]
use mp_engine_select::Engine;

//TODO: Port trap::* outbound-call wrappers (one per abi-traps.md row)
// Source: docs/architecture/engine-seam.md § Call-site conventions (SEAM-D13)
