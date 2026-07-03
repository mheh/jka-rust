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
    /// Always present, NOT an `Option` (LIFE-Q7 resolution, round-6): liveness
    /// is `sv.state == SS_DEAD` — the direct dual of Raven's loader-zero-filled
    /// statics (`serverState_t state`, `SS_DEAD` = "no map loaded",
    /// `codemp/server/server.h:46-54`). (`mp_engine_server`.)
    pub sv: Server,
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

//TODO: Port ZeroValid for Engine
// Source: docs/handoffs/2026-07-03-skeleton-findings.md item 20 (whole-Engine
// zeroed path). Blocked, not forced (checkpoint-5 finding): ZeroValid's stated
// contract requires `#[repr(C)]` + all-zero-valid, but `Engine` is a plain Rust
// struct and `Common.time_base: std::time::Instant` has unspecified layout with
// no all-zero-validity guarantee. The impl lands once the contract question is
// settled in the doc round (relax the repr(C) wording, or exclude/late-init the
// Instant field).

impl Engine {
    /// Returns the one owned engine island (STATE-D5), heap-allocated: the
    /// WHOLE aggregate is built through the `ZeroValid`-bounded zeroed path
    /// (LIFE-Q7 resolution, round-6 — the direct dual of Raven's
    /// loader-zero-filled statics; stack construction rejected: `server_t`
    /// embeds 1024 `svEntity_t` by value, `server.h:53-88`).
    ///
    /// Mechanics (LIFE-Q9 generalized, round-7): zeroed bytes only cover the
    /// `ZeroValid` `#[repr(C)]` mass; EVERY non-`ZeroValid` field is written
    /// in place through `MaybeUninit` BEFORE `assume_init` —
    /// `common.time_base` (the `Instant` capture, LIFE-D4b),
    /// `common.modules` (a zeroed `Option<ModuleSlot>` is NOT guaranteed
    /// `None`), `cl = None`, `snd = None` (same non-guarantee). Non-zero
    /// *subsystem* init happens in `com_init` exactly where Raven does it.
    /// Runs first in `main()` (`let mut engine: Box<Engine> = Engine::new();`),
    /// before the warm-up `sys_milliseconds` read and `com_init`. Takes no
    /// command line (LIFE-D4b).
    ///
    /// Source: `docs/architecture/state-ownership.md` § `com_init` / `Engine::new`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Box<Engine> {
        todo!("Port Engine::new — whole-aggregate zeroed alloc + Instant base (LIFE-D4b)")
    }
}
