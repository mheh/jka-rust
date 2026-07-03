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

use mp_abi::game::syscalls::G_PRINT::GPrint;
use mp_abi::{Execute, OutboundSysCall};
use mp_engine_select::Engine;

/// Raven `trap_Printf` (`g_syscalls.c:27-29`): print message on the local
/// console (`G_PRINT`, `g_public.h:105`). The frozen SEAM-D13 wrapper shape.
pub fn Printf(
    engine: &Engine,
    args: <GPrint as OutboundSysCall>::Args,
) -> <GPrint as OutboundSysCall>::Output {
    // UFCS spelling of the frozen `engine.execute(args)` — the bare method
    // call cannot infer `C` from `Args` alone (mechanical; checkpoint-7 finding).
    <Engine as Execute<GPrint>>::execute(engine, args)
}

//TODO: Port trap::* outbound-call wrappers (the remaining abi-traps.md rows)
// Source: docs/architecture/engine-seam.md § Call-site conventions (SEAM-D13)
