//! Type-erased receiver slots for the above-tier engine state that qcommon
//! threads but never owns or dereferences.
//!
//! qcommon sits below `mp_engine_server`/`mp_engine_client`/`mp_engine_botlib`
//! in the crate graph, so it cannot name their real state structs (importing
//! them would cycle). The dispatch/registration seams
//! (`Cmd_ExecuteString`/`Cbuf_ExecuteText`, the `EngineHooks` table) still have
//! to *pass* those receivers through. Under the opaque-slot ruling (user,
//! 2026-07-12, option A) each such receiver crosses qcommon as a
//! `#[repr(transparent)]` type-erased pointer slot: qcommon is pass-through
//! only — it constructs nothing and dereferences nothing — and the owning
//! crate casts the slot back to its real `&mut State` at its own boundary (the
//! single documented `unsafe` cast pair per crate).

/// Type-erased slot for the `mp_engine_server` `Server` state; qcommon is
/// pass-through only — never dereferences it. Cast back to `&mut Server` at the
/// server-crate boundary.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct Server(*mut ());

impl Server {
    /// Wrap a raw server-state pointer into the slot (called at the owning
    /// crate's boundary from a live `&mut Server`).
    pub fn from_raw(p: *mut ()) -> Server {
        Server(p)
    }

    /// The raw pointer back out, for the owning crate's cast-back helper.
    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_client` `Client` state; qcommon is
/// pass-through only — never dereferences it.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct Client(*mut ());

impl Client {
    pub fn from_raw(p: *mut ()) -> Client {
        Client(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_botlib` `BotLib` state; qcommon is
/// pass-through only — never dereferences it.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct BotLib(*mut ());

impl BotLib {
    pub fn from_raw(p: *mut ()) -> BotLib {
        BotLib(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_rmg` `RmManager` state; qcommon is
/// pass-through only — never dereferences it. Cast back to the real
/// `mp_engine_rmg::rm_manager::RmManager` at the server-crate boundary. Re-exported
/// as `cm_load::RmManager`, the name the cm_load/server threading uses.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct RmManager(*mut ());

impl RmManager {
    pub fn from_raw(p: *mut ()) -> RmManager {
        RmManager(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_renderer` `RenderModels` state (the FROZEN
/// `tr-model.md` model registry, owned by `Engine.render_models`); qcommon is
/// pass-through only — never dereferences it. Cast back to the real
/// `mp_renderer::tr_model::render_models::RenderModels` at the server-crate
/// boundary. Re-exported as `cm_load::RenderModels`, the name the cm_load/server
/// threading uses.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A) — same treatment as the
/// sibling `RmManager`/`Ghoul2System` receivers, since qcommon sits below
/// `mp_renderer`/`mp_engine_server` in the crate graph and cannot name the real
/// state struct without cycling.
#[repr(transparent)]
pub struct RenderModels(*mut ());

impl RenderModels {
    pub fn from_raw(p: *mut ()) -> RenderModels {
        RenderModels(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_renderer` `RendererFrontend` carrier bundle
/// (the DEC-42.3 receivers the `RE_*` frontend takes, owned by `Engine.re`);
/// qcommon is pass-through only — never dereferences it. Cast back to the real
/// `mp_renderer::renderer_frontend::RendererFrontend` at the renderer-crate
/// boundary (`mp_renderer::hook_install::re_from_view`). NULL on dedicated,
/// where `Engine.re` is `None` and no path reads it.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A); DEC-55.2 / DEC-59.1 for
/// the client's direct `RE_*` reach.
#[repr(transparent)]
pub struct Renderer(*mut ());

impl Renderer {
    pub fn from_raw(p: *mut ()) -> Renderer {
        Renderer(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_client` `SoundSystem` state (the DEC-57
/// software mixer, owned by `Engine.snd`); qcommon is pass-through only — never
/// dereferences it. Cast back to `&mut SoundSystem` at the client-crate boundary
/// (`mp_engine_client::client_host::snd_from_view`). NULL on dedicated, where
/// `Engine.snd` is `None` and no path reads it.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct SoundSystem(*mut ());

impl SoundSystem {
    pub fn from_raw(p: *mut ()) -> SoundSystem {
        SoundSystem(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_ghoul2` `Ghoul2System` state; qcommon is
/// pass-through only — never dereferences it.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A).
#[repr(transparent)]
pub struct Ghoul2System(*mut ());

impl Ghoul2System {
    pub fn from_raw(p: *mut ()) -> Ghoul2System {
        Ghoul2System(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}

/// Type-erased slot for the `mp_engine_client` `FxSystem` state (DEC-61.2);
/// qcommon is pass-through only — never dereferences it. Cast back to the real
/// `mp_engine_client::fx::fx_system::FxSystem` at the client-crate boundary
/// (`mp_engine_client::client_host::fx_from_view`). NULL on dedicated, where
/// `Engine.fx` is `None` and no FX trap ever arrives.
///
/// Ruling: opaque-slot (user, 2026-07-12, option A); DEC-61.2 for the FX reach.
#[repr(transparent)]
pub struct FxSystem(*mut ());

impl FxSystem {
    pub fn from_raw(p: *mut ()) -> FxSystem {
        FxSystem(p)
    }

    pub fn as_raw(&mut self) -> *mut () {
        self.0
    }
}
