//! `Engine` — the one owned engine-island instance (per mode), STATE-D5.

use mp_engine_client::{Client, SoundSystem};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::Common;
use mp_engine_server::Server;

/// The engine-island aggregate. DEFINED here (the one crate that depends on all
/// engine subcrates, so it can name Server/Client/etc. as fields). One value,
/// INSTANTIATED by `Engine::new` called from the thin `mp/app` bin shell
/// (STATE-D5), threaded `&mut` DOWN call chains (STATE-D1). No field is ever a
/// `static`.
///
/// NOTE (disambiguation, 2026-07-03): this `mp_engine_core::Engine` is a
/// **different type** from `mp_engine_select::Engine` (the module-side transport
/// alias). Opposite islands, never co-scoped (STATE-Q8 / workspace-architecture
/// canonical disambiguation block).
///
/// Source: `docs/architecture/state-ownership.md` § `Engine` (STATE-D5).
pub struct Engine {
    /// cvars, cmd, cbuf, fs, net, module registry (`mp_engine_qcommon`).
    pub common: Common,
    /// `Some` when a server is running (`mp_engine_server`).
    pub sv: Option<Server>,
    /// `Some` on client builds; `None` on dedicated (`mp_engine_client`).
    pub cl: Option<Client>,
    /// `cmg` + SubBSP, instance-shaped value — NOT an `Option`
    /// (`mp_engine_qcommon`).
    pub cm: CollisionWorld,
    /// Client only; `None` on dedicated (`mp_engine_client`).
    pub snd: Option<SoundSystem>,
    // botlib/ghoul2/icarus/rmg engine-side state is NOT yet a field here — those
    // four §F subcrates were outside the A2 survey; attachment point is STATE-Q2.
}

impl Engine {
    /// Returns the one owned `Engine` value (STATE-D5). Captures the
    /// `std::time::Instant` timer base into `Engine.common` (LIFE-D4b); runs
    /// first in `main()`, before the warm-up `sys_milliseconds` read and
    /// `com_init`. Takes no command line (LIFE-D4b).
    ///
    /// `sv`/`cm` are constructed via the `ZeroValid`-bounded
    /// `native_platform::zeroed_box` path (round-5 resolution, STATE-D9
    /// analogy; stack `Default` rejected — `server_t` embeds 1024 `svEntity_t`
    /// by value, `server.h:53-88`). Their `unsafe impl ZeroValid` lines land
    /// with their real field sets, beside their layout asserts.
    ///
    /// Source: `docs/architecture/state-ownership.md` § `com_init` / `Engine::new`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Engine {
        todo!("Port Engine::new — capture Instant base into Common (LIFE-D4b)")
    }
}
