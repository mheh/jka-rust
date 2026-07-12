# Host-seam restructure: `EngineHostView` replaces receiver-list + dyn-host threading

Status: PLANNED (user ruling 2026-07-11: "restructure the seam first" — chosen over
the raw-pointer LiveHost, which was rejected for its `&mut`-noalias aliasing risk).
Prerequisite for the boot/lifecycle wiring (phase 2, § below).

## Problem

The engine island has no live `EngineHost` implementation (only `MockHost`).
The frozen seam threads `&mut Common` *alongside* `&mut dyn EngineHost`
(e.g. `SE_Init(common, host)`, `Cvar_Get(common, cm, rm, host, …)`), yet host
methods (`cvar_register`, `fs_read_file`, `print`, `flrand`, `vm_call`) must
reach state inside that same `Common`. No safe-Rust view can coexist with the
caller's `&mut Common`; a raw-pointer host would violate the `noalias` contract
on `&mut` parameters (real miscompile risk, not pedantry).

## Design (ruling 43 amended)

One concrete bundle in `mp_engine_qcommon` (`common/engine_host_view.rs`):

```rust
pub struct EngineHostView<'a> {
    pub common: &'a mut Common,
    pub cm: &'a mut CollisionWorld,
    pub sv: cmd_pc::Server,              // existing opaque type-erased slots,
    pub rm: cm_load::RenderModels,       // held by value (newtype *mut ())
    pub rmg: cm_load::RmManager,
    pub g2: opaque_slots::Ghoul2System,
}
impl EngineHost for EngineHostView<'_> { … }
```

- A function that consumes host services takes `view: &mut EngineHostView<'_>`
  as its single world parameter — `view.common` for state, `view` forwarded to
  callees, `view.print(…)` for host methods. One `&mut` path; sound.
- The trait impl's methods route to the real, view-migrated functions by
  passing `self` (`cvar_register` → `Cvar_Get(self, …)`), so recursion through
  the seam has no aliasing.
- Methods needing `Server`/`RenderModels` (`sv_time`, `gentity`,
  `shared_memory`, `sv_shownet_entity_classname`, `model_mdxm`, `model_mdxa`,
  `skin_surfaces`) go through **new accessor fields on `Common.hooks`**
  (the 2026-07-12 hook-table ruling, extended): qcommon declares
  `Option<fn(&mut <slot>, …) -> …>` fields; `mp_engine_server`/the renderer
  crate provide the casting adapter fns; `mp_engine_core` installs them at
  boot. Slot casts stay in the crates that can name the real types.
- `sys_init`/`sys_quit`/`sys_error`/`sys_show_console` → `native_platform`
  directly. `error` → the receiverless `com_error`. `is_lan_address` → the
  ported `Sys_IsLANAddress` twin (worker locates its home; net-state detail).
- **Construction**: `mp_engine_core` gets the ruling-43 split constructor —
  a fn taking `&mut Engine` and returning the view (field split-borrows +
  `server_slot`/`rm_slot`/`rmg_slot`/`ghoul2_slot`), with the sidecars
  returned alongside as needed.

### What the view does NOT carry

`BotLib`, `RoffSystem`, `Icarus`, `Navigator` — no `EngineHost` method needs
them; they remain explicit sidecar parameters where a function needs them
(`SV_Init(view, bot)` shape). No aliasing: they are disjoint `Engine` fields.

### §F crates are untouched

ghoul2 (122 host-taking fns), icarus (115), rmg, stringed/roff/npcnav
*internals* keep `&mut dyn EngineHost` / `&mut impl EngineHost` — they now
receive the live view where MockHost stood in tests. All §18 golden harnesses
and `MockHost` stay byte-for-byte as they are.

### The stringed exception (subsystem state inside Common)

`Common.stringed` is the one §F state living *inside* the view's `common`.
Its entry wrappers (`SE_Init`, `SE_GetString`, …) migrate to the view and use
the **take/put-back pattern**:

```rust
let mut pkg = std::mem::take(&mut view.common.stringed);
se_init(&mut pkg, view);          // se_* signatures unchanged (harness-tested)
view.common.stringed = pkg;
```

`se_*` never reaches stringed through host methods (cvar/fs/print only), so
this is sound. Caveat (documented at the site): a `com_error` panic inside
leaves `Common.stringed` defaulted — acceptable; SE load failures are
init-fatal in Raven too.

### Slot-cast discipline (unchanged, restated)

Never hold a slot-cast `&mut` (e.g. `server_from_slot(&mut view.sv)`) across a
call that takes the view. Cast, use, drop — the existing opaque-slot rule.

## Migration rules (mechanical)

1. Signature rewrite: any fn taking `host: &mut dyn EngineHost` **plus** any of
   `common`/`cm`/`sv`/`rm`/`rmg`/`g2` → replace that receiver set with
   `view: &mut EngineHostView<'_>`. (~178 fns qcommon, ~94 server.)
2. Body rewrite: `common` → `view.common`, `cm` → `view.cm`, `host.m(…)` →
   `view.m(…)`, slot params → `view.sv` etc.
3. Call-site rewrite: migrated callee → pass `view` (or build a view if at a
   boundary); unmigrated narrow callee → pass `view.common`/`view.cm` field
   reborrows. (~630 sites.)
4. `EngineHooks` fn-pointer fields and the `CmdFunction` receiver chain retype
   to `(&mut EngineHostView, …)` — amends the pinned 2026-07-12 receiver order
   (user-ratified with this plan).
5. In-crate tests constructing receiver lists build a view over their test
   state (null slots are fine while no cast path runs).
6. No fn-body `use`, no inline crate paths, imports at file top (standing
   style rules); rustfmt; zero markers.

## Work breakdown (each round: green build + 343 tests + local commit)

- **W1 — seam kernel** (single reviewed worker): view struct + trait impl +
  new hook accessor fields + server/renderer adapter fns + core split
  constructor + core installer fn. Nothing consumes it yet; workspace stays
  green.
- **W2 — qcommon migration**: rules 1–3 over the 9 host-calling files and the
  threading-only files (cvar/files/cmd/net_chan/cm_load/z_memman…), fix-round
  loop driven by cargo errors.
- **W3 — server migration** + hooks/CmdFunction retype + their call sites.
- **W4 — sweep**: renderer (6) + rmg (1) edges; grep-gate: zero
  `host: &mut dyn EngineHost` left in C-track qcommon/server signatures
  (§F internals exempt); `cargo check --workspace` 0 errors 0 warnings;
  full test suite; fmt check.
- **W5 — docs**: decisions.md ruling entry (this restructure + the raw-pointer
  rejection rationale); state-ownership.md ruling-43 amendment; lifecycle.md
  cross-ref touch-ups; the 2026-07-12 receiver/hook ruling amendments.

Push policy: local commits per round; single batched push when the phase is
green end-to-end (master triggers CI publish).

## Phase 2 (follow-on, separate plan-of-record: the original task)

Boot/lifecycle wiring on the new seam: real 42-step `com_init_body`,
`com_frame_body`, `com_error_recover`, `com_shutdown`, transcribe
`SV_Frame`/`SV_CalcPings`/`SV_CheckTimeouts`/`SV_CheckCvars`/`SV_PacketEvent`
(`sv_main.cpp:594-940`), `NET_Init`/`NET_OpenIP`/`NET_Sleep` (unix twin),
hook + accessor installation in `main()`, and the dedicated OS loop
(`Sleep(5)`/`com_frame`).
