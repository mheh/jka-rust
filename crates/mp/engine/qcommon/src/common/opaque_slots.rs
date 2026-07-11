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
