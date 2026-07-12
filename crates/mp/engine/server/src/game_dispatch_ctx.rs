//! `GameDispatchCtx` — the game-slot dispatch note (user ruling 2026-07-12):
//! the engine-side generalization of Raven's `currentVM` stash.
//!
//! Raven's `SV_GameSystemCalls` reaches everything through C file-scope
//! globals, and its return path knows which VM called back via the
//! `currentVM` global `VM_Call` stashes before entering the module
//! (`vm.cpp:377`). Our world is threaded, not ambient (§B3), so the one
//! function C fed from globals gets them written down instead: this struct —
//! built ONCE at boot by `mp_engine_core::install_engine_hooks` from the boxed
//! `Engine`'s field addresses (stable for the process lifetime) and armed as
//! the game slot's `ctx` — is everything `game_system_calls_shim` needs to
//! rebuild the `EngineHostView` + sidecars when a module syscall arrives.
//!
//! Nothing here crosses the ABI: the module sees only Raven's three-pointer
//! contract (`dllEntry`/`vmMain` out, the variadic syscall in); this note is
//! private engine bookkeeping behind the trampoline (SEAM-D11).
//!
//! Reentrancy contract: a syscall arrives only while the engine caller is
//! suspended inside `VM_Call` (single-threaded, Raven's synchronous trap
//! model), so the caller's borrows of these same objects are dormant for the
//! whole dispatch — the DEC-23 slot-cast discipline at the module seam.

use mp_engine_icarus::Icarus;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::roff::RoffSystem;

use crate::npcnav::navigator::Navigator;

/// The note's fields: typed pointers where the shim dereferences directly,
/// erased `*mut ()` where the view only wraps them back into its opaque slots.
#[allow(clippy::missing_safety_doc)]
pub struct GameDispatchCtx {
    /// `Engine.common` — dereferenced into the view.
    pub common: *mut Common,
    /// `Engine.cm` — dereferenced into the view.
    pub cm: *mut CollisionWorld,
    /// `Engine.sv` (erased; becomes the view's `sv` slot).
    pub sv: *mut (),
    /// `Engine.cl` (erased; NULL on dedicated — the null-build client hooks
    /// never cast it).
    pub cl: *mut (),
    /// `Engine.bot` (erased; becomes the view's `bot` slot).
    pub bot: *mut (),
    /// `Engine.render_models` (erased; becomes the view's `rm` slot).
    pub rm: *mut (),
    /// `Engine.rmg` (erased; becomes the view's `rmg` slot).
    pub rmg: *mut (),
    /// `Engine.g2` (erased; becomes the view's `g2` slot).
    pub g2: *mut (),
    /// `Engine.icarus` — dispatcher sidecar (ruling 9 shape).
    pub icarus: *mut Icarus,
    /// `Engine.nav` — dispatcher sidecar.
    pub nav: *mut Navigator,
    /// `Engine.roff` — dispatcher sidecar.
    pub roff: *mut RoffSystem,
}
