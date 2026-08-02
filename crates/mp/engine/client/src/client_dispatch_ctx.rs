//! `ClientDispatchCtx` — the cgame/ui dispatch note (DEC-55.1), the client
//! twin of `mp_engine_server`'s `GameDispatchCtx`.
//!
//! Raven's `CL_CgameSystemCalls`/`CL_UISystemCalls` reach everything through C
//! file-scope globals, and the return path knows which VM called back through
//! the `currentVM` global `VM_Call` stashes (`vm.cpp:377`). Our world is
//! threaded, not ambient (§B3), so the globals those two functions fed on get
//! written down instead: this struct — built ONCE at boot by
//! `mp_engine_core::install_engine_hooks` from the boxed `Engine`'s field
//! addresses (stable for the process lifetime) — is everything
//! `cgame_system_calls_shim` and `ui_system_calls_shim` need to rebuild the
//! dispatcher receivers when a module syscall arrives.
//!
//! ONE note serves both slots. cgame and ui read the same islands and differ
//! only in which dispatcher the armed shim enters, so the cgame and ui slots
//! arm the same pointer with two different shims.
//!
//! Every field is a typed pointer, not the game note's erased `*mut ()`: this
//! crate can name each island (`mp_engine_server` cannot name `Client`, which
//! is why the game note erases). The two late-seated islands are pointers to
//! their `Option` FIELD rather than to a payload, so the note stays correct
//! whichever boot step seats them — `Engine.cl` and `Engine.re` are `None` when
//! `install_engine_hooks` runs.
//!
//! Nothing here crosses the ABI: each module sees only Raven's three-pointer
//! contract (`dllEntry`/`vmMain` out, the variadic syscall in); this note is
//! private engine bookkeeping behind the trampolines (SEAM-D11).
//!
//! Reentrancy contract: a syscall arrives only while the engine caller is
//! suspended inside `VM_Call` (single-threaded, Raven's synchronous trap
//! model), so the caller's borrows of these same objects are dormant for the
//! whole dispatch — the DEC-23 slot-cast discipline at the module seam.

use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::roff::RoffSystem;
use mp_engine_rmg::rm_manager::RmManager;
use mp_renderer::renderer_frontend::RendererFrontend;
use mp_renderer::tr_model::render_models::RenderModels;

use crate::client_host::Client;
use crate::fx::fx_system::FxSystem;

/// The addresses the two client shims dereference on every module trap.
#[allow(clippy::missing_safety_doc)]
pub struct ClientDispatchCtx {
    /// `Engine.common`.
    pub common: *mut Common,
    /// `Engine.cm` — the collision world.
    pub cm: *mut CollisionWorld,
    /// `Engine.sv` (erased: this crate's `Server` receiver is
    /// `mp_engine_server`'s, and the dispatchers take it back through the
    /// view's slot rather than by name).
    pub sv: *mut (),
    /// `&mut Engine.cl` — the `Option` field, so the seating step may run after
    /// the arming. A trap that arrives while it is `None` is a boot-order bug,
    /// and the shim reports it rather than reading a fabricated world.
    pub cl: *mut Option<Client>,
    /// `Engine.bot` (erased; the client threads it only through the view).
    pub bot: *mut (),
    /// `Engine.render_models` — the one model registry the server and the
    /// client share.
    pub rm: *mut RenderModels,
    /// `&mut Engine.re` — the renderer frontend's carrier bundle, an `Option`
    /// field for the same seating reason as `cl` (DEC-59.1).
    pub re: *mut Option<RendererFrontend>,
    /// `Engine.rmg` — the RMG mission manager.
    pub rmg: *mut RmManager,
    /// `Engine.g2` — the ghoul2 system the G2 trap block reaches.
    pub g2: *mut Ghoul2System,
    /// `Engine.roff` — Raven's one `theROFFSystem`, which the five `CG_ROFF_*`
    /// arms drive the same way the server's `G_ROFF_*` arms do.
    pub roff: *mut RoffSystem,
    /// `&mut Engine.fx` — the FX system's `Option` field. The first `CG_FX_*`
    /// trap seats it, the way Raven's file-scope FX globals came alive on
    /// `FX_Init` (DEC-61.2).
    pub fx: *mut Option<FxSystem>,
}
