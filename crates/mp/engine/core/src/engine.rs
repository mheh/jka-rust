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
    /// ICARUS scripting subsystem — a plain, `Default`-initialized field per
    /// ICARUS-D3 (rulings 12/27; STATE-Q2 CLOSED,
    /// `docs/architecture/state-ownership.md:1860-1876`): the fork-2 owner of
    /// every ICARUS file-scope global, reached through the ICARUS-D2
    /// `EngineHostView`/`icarus_call` split-borrow. Not `Option`/`Box`-wrapped;
    /// "is ICARUS initialized?" is Raven's own `iICARUS != NULL` NULL-flag
    /// (`icarus.instance.is_some()`).
    pub icarus: mp_engine_icarus::Icarus,
    /// Server-side Ghoul2 (bones/bolts/ragdoll/gore + the Ghoul2InfoArray
    /// arena) — plain `Default` field per rulings 12/29 (`mp_engine_ghoul2`).
    pub g2: mp_engine_ghoul2::ghoul2_system::Ghoul2System,
    /// RMG mission manager — `land: Option<TerrainHandle>` mirrors Raven's
    /// null-initialized `mLandScape` (rulings 12/28; `mp_engine_rmg`).
    pub rmg: mp_engine_rmg::rm_manager::RmManager,
    /// Headless model registry/cache (`tr.models` + CachedModels) — reached by
    /// ghoul2 only through `EngineHost::model_*` (rulings 52/53; `mp_renderer`).
    pub render_models: mp_renderer::tr_model::render_models::RenderModels,
    /// Engine-side nav graph (`CNavigator` twin) — plain `Default` field per
    /// rulings 12/30 (`mp_engine_server::npcnav`).
    pub nav: mp_engine_server::npcnav::navigator::Navigator,
    /// ROFF cache + per-entity playback list — plain `Default` field per
    /// rulings 12 (`mp_engine_qcommon::roff`).
    pub roff: mp_engine_qcommon::roff::RoffSystem,
    // botlib engine-side state becomes a direct field here (STATE-Q2 CLOSED —
    // engine-fork-discovery rulings 12/13/43); it lands with the botlib
    // integration waves, reached via the EngineHostView split-borrow
    // constructors (ruling 43).
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
        use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};
        use std::ptr::addr_of_mut;

        use mp_engine_qcommon::vm::ModuleRegistry;

        let layout = Layout::new::<Engine>();
        // SAFETY: the zeroed bytes cover only the ZeroValid-audited #[repr(C)]
        // mass; EVERY non-ZeroValid field is written in place below before the
        // Box is exposed (the MaybeUninit pattern, LIFE-Q9) — raw `.write()`s,
        // no drops of uninit memory, single-threaded.
        unsafe {
            let p = alloc_zeroed(layout) as *mut Engine;
            if p.is_null() {
                handle_alloc_error(layout);
            }
            // The Instant timer base (LIFE-D4b) — captured here, first in main().
            addr_of_mut!((*p).common.time_base).write(std::time::Instant::now());
            // The empty ModuleRegistry (step-30 VM_Init's default-shaped build);
            // a zeroed Option<ModuleSlot> is NOT guaranteed None.
            addr_of_mut!((*p).common.modules).write(ModuleRegistry::default());
            // Option<Client>/Option<SoundSystem>: same niche non-guarantee.
            addr_of_mut!((*p).cl).write(None);
            addr_of_mut!((*p).snd).write(None);
            // Icarus holds Box slot-arrays, HashMaps, and a fn-item table — NONE
            // all-zero-valid — so it is written in place through its hand-written
            // Default before the Box is exposed (rulings 12/27; the modules /
            // time_base non-ZeroValid precedent).
            addr_of_mut!((*p).icarus).write(Default::default());
            // The five §F aggregates hold Vecs/BTreeMaps/Strings — none
            // all-zero-valid — so each is written in place through its Default
            // (rulings 12/28/29/30/53; same precedent as `icarus` above).
            addr_of_mut!((*p).g2).write(Default::default());
            addr_of_mut!((*p).rmg).write(Default::default());
            addr_of_mut!((*p).render_models).write(Default::default());
            addr_of_mut!((*p).nav).write(Default::default());
            addr_of_mut!((*p).roff).write(Default::default());
            // cm.shaderTextTable: Vec-backed CmHashTable placeholder, not zero-valid.
            addr_of_mut!((*p).cm.shaderTextTable).write(Default::default());
            // cm.cmShaderTable: Vec-backed CmHashTable, not zero-valid.
            addr_of_mut!((*p).cm.cmShaderTable).write(Default::default());
            // cm.svInfoParms: real name-pointer/flag lookup table, not zero.
            addr_of_mut!((*p).cm.svInfoParms).write(CollisionWorld::init_svInfoParms());
            // cm.svMaterialNames: real C-string pointer table, not zero.
            addr_of_mut!((*p).cm.svMaterialNames).write(CollisionWorld::init_svMaterialNames());
            // Common.stringed (ruling 50/55): BTreeMap-backed store, written
            // through its Default (= Raven's Clear(SE_FALSE)) per the ruling-55
            // construction story.
            addr_of_mut!((*p).common.stringed).write(Default::default());
            Box::from_raw(p)
        }
    }
}
