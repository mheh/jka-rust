# Host-seam migration — worker spec (W2/W3)

> **Status: COMPLETED — worker spec for the executed host-seam restructure.**

Contract types are already landed (commit `seam(w1)`), and are READ-ONLY for
workers: `common/engine_host_view.rs` (the `EngineHostView` bundle + live
`EngineHost` impl), `common/engine_hooks.rs` (view-typed hook fields),
`cmd/cmd_function_t.rs` (`pub type CmdFunction = fn(&mut EngineHostView)`).
The tree is deliberately RED until every group lands (blind-parallel model);
do NOT run cargo and do NOT edit files outside your assigned group — other
groups' errors are not yours to fix.

## The view

```rust
use crate::common::engine_host_view::EngineHostView;   // (qcommon)
use mp_engine_qcommon::common::engine_host_view::EngineHostView; // (server)

pub struct EngineHostView<'a> {
    pub common: &'a mut Common,
    pub cm: &'a mut CollisionWorld,
    pub sv: opaque_slots::Server,       // type-erased slots, by value
    pub cl: opaque_slots::Client,
    pub bot: opaque_slots::BotLib,
    pub rm: opaque_slots::RenderModels,
    pub rmg: opaque_slots::RmManager,
    pub g2: opaque_slots::Ghoul2System,
}
impl EngineHost for EngineHostView<'_> { … }
```

## Migration rules (deterministic — apply blind, no cross-file inspection)

1. **Signature collapse.** Any fn taking a `host: &mut dyn EngineHost` (or
   `&mut impl EngineHost`) param **plus** any of the receiver params
   (`common: &mut Common`, `cm: &mut CollisionWorld`, and/or the opaque slots
   `sv`/`cl`/`bot`/`rm`/`rmg`/`g2`): replace that whole receiver+host set with
   `view: &mut EngineHostView` as the FIRST param. Keep all other params in
   their existing order. Doc comments/`Source:` cites stay.
2. **Body rewrite.** `common` → `view.common`, `cm` → `view.cm`,
   `host.method(…)` → `view.method(…)` (the view IS the host; keep the
   `EngineHost` trait imported at file top for method resolution),
   slot params (`sv`, `rm`, …) → `view.sv`, `view.rm`, ….
3. **Call sites.** Callee migrated by rule 1 (it took host + receivers —
   derivable from today's signature, no inspection needed) → pass `view`.
   Callee narrow (takes only `common`/`cm`/a slot, NO host) → pass
   `view.common` / `view.cm` / `&mut view.sv` field reborrows, unchanged
   otherwise. Callee took ONLY `host` and no receivers (§F-internal generic
   fns) → pass `view` (it satisfies `&mut impl EngineHost`).
4. **Hook calls.** `(common.hooks.X.expect("…"))(recv-args…)` →
   `let f = view.common.hooks.X.expect("…"); f(view, value-args…)` — hooks
   now take the view first, then only the VALUE args (netadr_t, msec, …);
   the old receiver args disappear.
5. **Hook-target fns** (server: `SV_Init`, `SV_Shutdown`, `SV_GameCommand`,
   `SV_ShutdownGameProgs`): give them EXACTLY the hook field's signature
   (`fn(&mut EngineHostView)`, `fn(&mut EngineHostView, &str)`, …) — they are
   installed directly (`hook_install.rs` already references them by path).
6. **Cmd handlers** conform to `CmdFunction = fn(&mut EngineHostView)`.
   Registration forwarders that today thread the 7-receiver list collapse to
   one view param; handler bodies that need the real `Server`/`Ghoul2System`
   cast the view's slot at the top (rule 7).
7. **Slot casts (server crate only).** To reach the real state behind a slot:
   ```rust
   // SAFETY: view-constructor slot, single-threaded, no other live cast of
   // this slot for the borrow's duration.
   let sv = unsafe { &mut *(view.sv.as_raw() as *mut Server) };
   ```
   Per-slot rule: while such a cast borrow is live you may use `view` freely
   (its `common`/`cm` and OTHER slots), but nothing you call may cast the
   SAME slot again. Existing fns that already take the real `&mut Server`
   as a param KEEP it (pass it down as today); only the host param collapses
   into the view. In-server calls to `host.sv_time()`/`host.gentity(…)`
   REPLACE with the direct read (`sv.svs.time` / `SV_GentityNum(sv, …)`) when
   a real `&mut Server` is already in scope — never call an sv-touching view
   method while holding the real borrow.
8. **§F-internal generic code keeps its signatures** (fns taking
   `&mut impl EngineHost` + their OWN state struct, driven by §18 golden
   tests): stringed `package.rs`/`interface.rs` internals, `roff_system.rs`
   internals, npcnav, renderer tr_model. Only their qcommon-facing WRAPPERS
   migrate. For state living inside `Common` (only `common.stringed`), the
   wrapper uses take/put-back:
   ```rust
   let mut pkg = std::mem::take(&mut view.common.stringed);
   se_init(&mut pkg, &mut *view);
   view.common.stringed = pkg;
   ```
   (Document the error-path caveat once per file: a com_error panic inside
   leaves the field defaulted; SE load failures are init-fatal in Raven too.)
   State OUTSIDE the view (e.g. `RoffSystem`) stays a sidecar param:
   `fn ROFF_UpdateEntities(roff: &mut RoffSystem, view: &mut EngineHostView, …)`.
9. **Sidecar receivers.** Real `&mut BotLib`/`&mut Icarus`/`&mut Navigator`/
   `&mut RoffSystem` params (state not borrowable through the view) stay as
   explicit params AFTER the view.
10. **Tests.** In-crate tests calling migrated fns build a view over their
    test state:
    ```rust
    let mut view = EngineHostView {
        common: &mut common, cm: &mut cm,
        sv: opaque_slots::Server::from_raw(core::ptr::null_mut()),
        cl: opaque_slots::Client::from_raw(core::ptr::null_mut()),
        bot: opaque_slots::BotLib::from_raw(core::ptr::null_mut()),
        rm: opaque_slots::RenderModels::from_raw(core::ptr::null_mut()),
        rmg: opaque_slots::RmManager::from_raw(core::ptr::null_mut()),
        g2: opaque_slots::Ghoul2System::from_raw(core::ptr::null_mut()),
    };
    ```
    Null slots are fine while no cast path runs. Tests that used `MockHost`
    to drive §F-generic fns are UNCHANGED.
11. **Style.** Imports at file top only; no fn-body `use`; no inline
    fully-qualified paths in expressions (test modules + layout-assert blocks
    exempt); preserve doc comments and Raven cites; rustfmt formatting by
    hand (do not run tools).
12. If a case genuinely fits no rule, leave the code untouched and REPORT it
    (file:line + why) instead of inventing a shape.
