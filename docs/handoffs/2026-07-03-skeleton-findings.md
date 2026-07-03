# Skeleton seed findings + resolutions (2026-07-03)

The DEC-10 base build (branch `skeleton`, seed commit `497fff4`, parent
`c36babe` = crate-migration HEAD) surfaced six findings. Two were forks, both
settled with the user this session; four are mechanical. ALL of these become
dated amendments to the Group-A docs **after** the round-4 workflow stamps
(don't churn the running round). The skeleton applies them immediately per
DEC-10 (checkpoint 2, in progress via the seeder agent).

## Settled forks (user, 2026-07-03)

1. **SEAM-D11 trampoline → C shim via `cc`.** Stable Rust cannot *define* a
   C-variadic function (`c_variadic` is nightly-only), and the fixed-arity
   workaround breaks for foreign variadic callers on arm64 macOS (stack-passed
   va-args). Resolution: a small committed `.c` file defines
   `game_syscall_trampoline(intptr_t cmd, ...)`, unpacks the va_list into an
   `intptr_t` array mirroring the oracle's own `VM_DllSyscall` unpacking, and
   forwards to a Rust `extern "C-unwind"` fn taking `*const isize`. Built by a
   `cc` build script in the owning crate. Exact drop-in ABI, stable toolchain;
   accepted cost: build-time C compiler dependency for that crate.

2. **SEAM-D11 layering → inject at load, Raven-style.** `EngineSlot` +
   trampoline are specced into `mp_engine_qcommon`, which cannot name
   `mp_engine_core::Engine` or `mp_engine_server::sv_game_system_calls` (both
   uphill). Resolution: qcommon keeps `EngineSlot` but stores **injected**
   state — an opaque ctx pointer + the syscall fn pointer passed in at
   module-load time — mirroring Raven, where `VM_Create` *receives* the
   `systemCalls` pointer as an argument (`codemp/qcommon/vm.cpp`) rather than
   naming the server. `load_module` gains the injected-systemCalls parameter.
   No crate-graph change.

## Settled fork from the round-4 gate (user, 2026-07-03)

**SP access discipline → settle the mapping now** (round-4 state-ownership
escalation: the doc claimed "SP jagame"/"SP mirror" on vmMain-shaped
mechanisms, but SP uses `GetGameAPI` — `g_main.cpp:875` returns a
`game_export_t` of direct fn pointers; no command dispatch). Settled mapping:

- SP jagame has **no vmMain, no command decode, no `Dispatch<C>` routing** —
  the `game_export_t` fn pointers are the entry surface.
- World lifetime: **`ge->Init` writes the SP WORLD cell; `ge->Shutdown` takes
  it** — the direct analog of MP's GAME_INIT-write / GAME_SHUTDOWN-take.
- **Each export derives its own `*mut GameWorld`** from the SP WORLD cell in
  its prologue and constructs the SP `GameContext` mirror (in `sp/game`)
  itself — per-export construction replaces MP's once-per-vmMain construction.
- SP module-side engine handle = the stored `game_import_t` the engine passes
  into `GetGameAPI` (`g_public.h`) — the SP mirror of mp_engine_select's
  transport alias. If the precise SP alias name/crate isn't derivable from
  existing decisions, drafters escalate rather than invent.

## Mechanical (fold into the same post-stamp amendment pass)

3. `CEngine` needs `unsafe impl Send/Sync` for SEAM-D10's
   `static ENGINE: OnceLock<CEngine>` (set-once/single-thread; sound). Docs
   never state this compile-forced detail — add 1 line at SEAM-D10.
4. ~~`GameContext` has private fields … add `GameContext::new` to the frozen
   block.~~ **SUPERSEDED by the round-5 fork-1 ruling (pub fields + struct
   literal, no `::new`) — do NOT act on this item's original text.** (Its
   stale wording caused the round-6 STATE-D8 flip-flop.)
5. `MAX_GENTITIES` now also in `mp_qshared` (GameWorld needs it); the
   `mp_engine_server` copy + its size-asserts still stand. The doc's
   "server re-imports it" dedupe is a deferred mechanical sweep.
6. Old `OutboundSysCallExecutor`/`InboundVmCallExecutor`/`message.rs`
   placeholders kept alongside the new `Execute<C>`/`Dispatch<C>` traits
   (extend-don't-rewrite). SEAM-D5's "retires message.rs" removal is a
   mechanical follow-up commit on `skeleton`.

## Checkpoint-2 additions (same amendment pass)

Applied on `skeleton` as `5ee02ed` (both resolutions) + `f70aa59` (finding-6
removal); green before each commit.

7. **`EngineSlotGuard`/`enter` superseded.** Round-3 SEAM-D11's per-call
   `Cell` + guard shape is replaced by load-time injection:
   `EngineSlot { ctx: *mut c_void, syscall: SlotSyscall }` with
   `SlotSyscall = extern "C-unwind" fn(*mut c_void, *const isize) -> isize`
   (Raven's `systemCalls` widened with the opaque ctx; `vm.cpp:471-472,506`).
   The amendment must record the guard's supersession explicitly.
8. **`restart` does NOT re-take the injection.** Raven's native `VM_Restart`
   saves `systemCall` off the freed `vm_t` and reuses it (`vm.cpp:399-409`);
   the skeleton mirrors that by reusing the stored `EngineSlot`, so the frozen
   LOAD-D12b signature stands unchanged. Worth one doc line stating the
   reuse-stored-injection reading.
9. `load_module` widened:
   `(…, syscall: RawSyscall, system_calls: SlotSyscall, ctx: *mut c_void)
   -> Option<SlotId>` — the two fn-pointer params are Raven duals
   (`Sys_LoadDll`'s trampoline arg vs `VM_Create`'s `systemCalls`).
   Trampoline shim: `vm/game_syscall_trampoline.c` mirrors `VM_DllSyscall`'s
   16-word unpack (`vm.cpp:363-377`); `cc` build dep confined to
   `mp_engine_qcommon`.

## Checkpoint-3 findings (SP mirror seed, commit 31a89db — for the NEXT doc pass)

Checkpoint 3 seeded the SP surface per the settled GetGameAPI mapping; green
(`cargo check --workspace` + all four cdylibs link; `nm` confirms jagame
exports exactly `_GetGameAPI`, jampgame `_dllEntry`/`_vmMain`/`_GetModuleAPI`).
Round 5 was already in flight, so these land in the NEXT amendment pass (or as
straggler items at sign-off). No user forks — all mechanical or tracked-open:

10. **`game_export_t` fn-pointer fields flipped to `extern "C-unwind"`**
    (compile-forced: the table can't hold C-unwind shell fns otherwise). A
    partial, non-silent application of the SEAM-D12 sweep; `game_import_t`
    untouched (sweep still pending). Doc round records the partial application.
11. **jagame's dep edges exceed workspace-architecture's shell table**: needs
    `sp_abi` + `sp_qshared` in addition to `abi-transport` + `sp/game` (the
    GetGameAPI table types + member-signature types live there). Amend the dep
    table. (Alternative — sp_game re-exports — would be a NEW decision; not
    taken.)
12. **No SP transport-alias name minted** (the flagged non-derivable):
    `GameContext.engine` and `mod gi` bind `game_import_t` directly per
    SEAM-D2; whether an alias name/crate should exist stays with the doc round.
13. **`OnceLock<game_import_t>` isn't Sync** (raw-pointer member
    `VoiceVolume: *mut c_int`) — jagame shell needed the same compile-forced
    cell wrapper (`EngineCell`, local unsafe impls) as `CEngine` (finding-3
    family). Add to the SEAM-D10 amendment line.
14. **Retired `abi_transport::entrypoints::{qvm, sp_game}` stub modules**
    (their `#[no_mangle]` symbols collide at cdylib link with live shell
    exports; LOAD-D4/SEAM-D10 already mandate per-shell exports). cgame/ui got
    the stub bodies verbatim in their own lib.rs (still `extern "C"`; SEAM-D12
    sweep untouched) pending their live shape.
15. **SP engine-side signatures are placeholders, not frozen** — no Group-A
    doc freezes `sv_init_game_progs` / the `ge` handle placement (marked
    in-source, cites sv_game.cpp:478/:403/:669-691). Needs a doc home when the
    SP engine island gets its pass. SP divergences preserved in the seed: no
    clients box in SP GameWorld (no g_clients, MAX_CLIENTS=1); NO
    load_module/EngineSlot/trampoline dual (SP has no VM_Create; direct
    GetGameAPI attach per DEC-07).

## Round-5 gate results — FIVE FORKS AWAIT THE USER (2026-07-03, end of session)

Round 5 (`wf_0f629c7a-b09`) returned zero stamps; all four docs NEEDS_SESSION.
The widened full-sibling reading set caught real cross-doc contradictions —
the prose analog of what rustc catches on the skeleton.

> **ALL FIVE RESOLVED (user, 2026-07-03, "all recommended"):** 1 = pub fields
> + struct literal (STATE-D8 stands; `::new` dropped from doc + skeleton);
> 2 = `&mut Common` in mp_engine_qcommon (lifecycle.md amended); 3 = drop the
> `!systemCalls` disjunct as structurally unreachable (guard =
> `name.is_empty()` only); 4 = Engine::new() reuses zeroed_box for sv/cm,
> mp_engine_core -> native_platform edge sanctioned; 5 = unsafe marker trait
> `ZeroValid` (hand-rolled bytemuck-Zeroable style), `zeroed_box<T: ZeroValid>`
> stays safe, per-type `unsafe impl` next to the layout asserts. Round 6
> composed from these + items 10-15 + the pinnings below.

1. **GameContext construction surface** (engine-seam amended text vs
   state-ownership STATE-D8 FROZEN): private fields + `pub fn new()` (what the
   skeleton has) vs `pub` fields + struct literal (STATE-D8's text, WorldPtr
   precedent). Visibility was never actually settled in the SEAM-Q12 session.
   Recommendation: pub fields + struct literal (no invariant to protect on a
   Copy struct of raw pointers); skeleton then drops `::new`.
2. **com_printf receiver** (STATE-D11 vs lifecycle.md frozen block):
   `com_printf(&mut Common, …)` in mp_engine_qcommon vs `&mut Engine` in
   mp_engine_core. Recommendation: &mut Common/qcommon (Raven's common.cpp
   tier; narrowest owner; avoids uphill callers); amend lifecycle.md.
3. **LOAD-D11 bad-parms guard**: Raven's `!systemCalls` disjunct has a
   non-nullable Rust dual (`SlotSyscall`), so the frozen guard tests the WRONG
   parameter (`syscall.is_null()`, which is Sys_LoadDll's trampoline, not
   VM_Create's systemCalls). Options: (a) drop disjunct as structurally
   unreachable like `!module` (recommended), (b) keep null check relabeled as
   defensive-Rust (speculative divergence), (c) Option<SlotSyscall> ceremony.
4. **Engine::new() construction of sv/cm** (lifecycle hole): server_t embeds
   1024 svEntity_t by value — same stack-overflow class STATE-D9 solved with
   zeroed_box for GameWorld — but no doc sanctions reusing zeroed_box on the
   engine island (would add mp_engine_core -> native_platform edge).
   Recommendation: reuse it (mechanics recorded in lifecycle/state-ownership).
5. **zeroed_box form (STATE-Q10)**: frozen as a SAFE fn with an unchecked
   all-zero-valid precondition — unsound as written (zeroed_box::<String>()
   compiles). Options: unsafe marker trait ZeroValid, bytemuck-style
   (recommended: per-type unsafe impl next to the layout asserts, call sites
   stay safe), unsafe fn, or leave-with-comment (rejects the confine-unsafe
   rule).

Low-stakes pinnings to fold into round 6 as MECHANICAL (defaults consistent
with settled decisions/skeleton; user can override): vmMain `dispatch` =
inline exhaustive match mirroring outbound sv_game_system_calls; trap wrappers
live at crates/mp/game/src/trap.rs (mirrors g_syscalls.c); pre-decode
bootstrap comparison spelled `command == MpGameExport::GameInit as c_int`;
STATE-Q8 closes by citing the round-4 keep-both+disambiguate/no-renames
decision. STATE-Q9 (SP alias name) stays sanctioned-open, owner: SP slice.
Checkpoint-3 findings 10-15 above also ride in round 6.

Round-6 relaunch: same workflow, args pattern = round 5's
(docs/handoffs/group-a-round5-args.json) + the five resolutions + items 10-15
+ the pinnings; keep the full-sibling standingDocs list and the
sanctioned-open gate policy. Skeleton follow-up after resolutions: adjust
GameContext visibility per fork 1; zeroed_box shape per forks 4/5.

## Checkpoint-4 findings (commit b6c50e3 — for the next amendment pass / sign-off stragglers)

Checkpoint 4 applied all five round-5 resolutions code-side; green + all four
cdylibs link. Two small doc items surfaced (round 6 was already in flight):

16. **`sp_game -> native/platform` Cargo edge added** (orphan rule: SP
    level_locals_t's ZeroValid impl lives in sp_game). The MP dual
    (mp_game -> native/platform) was already sanctioned in STATE-D9 text;
    workspace-architecture's dep table needs BOTH rows.
17. **`unsafe impl<T: ZeroValid, const N: usize> ZeroValid for [T; N]`** added
    in native_platform::mem — the GameWorld entity boxes are arrays, and
    array types have no owning file for a colocated per-type impl, so the
    resolution's rule can't cover them without this blanket. STATE-D9's
    ZeroValid amendment should record the array rule explicitly.

## Round-6 gate results + resolutions (2026-07-03, late session)

Round 6 (`wf_de9f1127-bd2`): zero stamps. Two failure classes: (a) an args
bug — round-5 args had told state-ownership "private + new()" before fork 1
was identified, so round 6's "STATE-D8 stands as written (pub fields)" was
self-contradictory and the drafter kept private+new, opposite the user ruling;
(b) pipeline staleness — per-doc gates read siblings mid-revision (the
"ZeroValid missing" hole was engine-seam's gate reading pre-round-6
state-ownership). **Process fix adopted: one reconciliation agent edits all
four docs, THEN a gate-only pass reads final text.**

Mechanical reconciliations (settled by existing rulings/oracle, no forks):
STATE-D8 → pub fields (user ruling reapplied); STATE-D6 vmMain block → the
SEAM-D9/D10 `(command: AbiCommand, arg0..arg11: AbiWord) -> AbiWord` shape
(oracle g_main.c:515 is the all-`int` 32-bit original; LOAD-D12e's widened
word is the settled 64-bit dual — record the tie-break); ComError → pub fields
in BOTH docs (lifecycle's own cross-crate `com_error_recover` requires it);
Dispatch-impl colocation pinned (all thin `impl Dispatch<C> for GameContext`
adapters in world/game_context.rs; per-command logic stays one-fn-per-file);
`WorldCell::new()` spelled (trivial const fn).

**Three forks RESOLVED (user, 2026-07-03, all recommended):**

18. **Unix NDEBUG in-loader fatal → reproduce faithfully.** sys_load_dll's
    missing-export arm gains a `cfg(not(debug_assertions))` receiverless
    `com_error(ERR_FATAL, …)` dual of unix_main.c:431-436; debug arm keeps
    print+None. Porting-rules §20 (preserve per-mode quirks). Option contract
    otherwise unchanged; Slice-0 debug path untouched.
19. **STATE-Q11 → world/ folder wins.** crates/{mp,sp}/game/src/world/
    {mod,game_world,game_context,entity_id}.rs per the doc; skeleton
    checkpoint 5 moves the flat files and seeds entity_id.rs (EntityId(u32)).
20. **LIFE-Q7 → drop Option; whole-Engine zeroed.** Frozen field becomes
    `sv: Server` (no Option) with liveness = `sv.state == SS_DEAD`, the direct
    dual of Raven's loader-zero-filled statics; `Engine::new() -> Box<Engine>`
    allocates the WHOLE aggregate through the ZeroValid zeroed path; non-zero
    init happens in com_init exactly where Raven does it. Dissolves the
    per-field zeroed_box and stack-ordering questions entirely; STATE-D13's
    per-member mechanics and lifecycle's call-site-only narrowing are
    superseded. jampded main sketch: `let mut engine: Box<Engine> =
    Engine::new();`.

## Checkpoint-5 findings (commit 47cbb17 — forwarded to the reconciliation pass)

21. **Literal `unsafe impl ZeroValid for Engine` is UNSOUND** — Common's
    `time_base: std::time::Instant` has unspecified layout and no all-zero
    validity; the trait contract (repr(C) + zero-valid) can't cover the
    aggregate. Item-20 mechanics must be worded as: boxed zeroed allocation
    with explicit in-place init of the non-zero-valid fields before exposure
    (MaybeUninit pattern) — NOT a ZeroValid impl on Engine. Also record:
    soundness, not rustc, is the gate for ZeroValid impls (a bare marker impl
    always compiles).
22. **Mixed presence idioms in Engine**: `sv: Server` (always-present,
    state-gated per item 20) now coexists with `cl`/`snd: Option<…>` — the
    docs should state the rule (server mirrors Raven's zero-filled statics;
    client-side stays Option until its own pass, owner: client slice).
23. **Item-18's release fatal needs a MECHANISM line**: native/platform
    (tier −1) cannot name mp_engine_qcommon::com_error (uphill). The CONTRACT
    is settled (in-loader ERR_FATAL, unix_main.c:431-436); the mechanism
    (injected fatal hook à la checkpoint-2's systemCalls injection vs
    platform-local sys-fatal vs three-state return with the fatal in
    qcommon's load_module) is tracked OPEN, owner: slice-0 wiring — not
    user-decided yet.

## Round-7 gate results + resolutions (2026-07-03)

Round 7 (`wf_b4257bcb-385`, gate-only after the reconciliation pass):
**engine-seam.md and module-loading.md STAMPED REVIEWED.** state-ownership +
lifecycle returned final compile-wiring holes. Three forks RESOLVED (user,
2026-07-03, all recommended):

24. **STATE-Q12 → mp_abi re-exports the seam traits.** mp_abi (already
    dependent on abi-transport; the seam crate by definition) re-exports
    `Dispatch`/`InboundVmCall`/`Execute`/`OutboundSysCall`; module logic
    crates keep their frozen dep sets, no new edges.
25. **Shell visibility → mp_game re-exports `MpGameExport`** (crate-root
    `pub use`); SEAM-D10's frozen exactly-two-edges shell property stays
    intact — the shell sees the seam through the logic crate it wraps.
26. **LIFE-Q8 → boot-success stubs.** Slice-0 com_init steps 3/5/7/12
    (Cvar_Init, Cbuf_Init, Cmd_Init, FS_InitFilesystem) are
    deliberately-callable no-ops with the mandated `//TODO: Port <subject>` +
    justification markers; DEC-09.2 boot-transcript diffing activates when
    B1/B2 land.

Mechanical (no forks): LIFE-Q9 generalized — the Engine::new MaybeUninit
in-place-init list = EVERY non-ZeroValid field (`time_base`, `modules` —
zeroed `Option<ModuleSlot>`/`Option<Client>`/`Option<SoundSystem>` are NOT
guaranteed None — so `modules`, `cl: None`, `snd: None` are written
explicitly); `Engine` pinned to engine.rs in mp_engine_core (one-type-per-
file); `sys_error` pinned to lifecycle.rs; `WorldCell` field gains
`pub(crate)` (LOAD-D12f precedent) and lives in jampgame's world_cell.rs per
the frozen per-file placement.

## GROUP A COMPLETE — all four docs REVIEWED (round 8, wf_2926f5cc-533)

Round 8 gate-only pass: state-ownership.md + lifecycle.md STAMPED with zero
escalations. Combined with round 7's stamps, **all four Group-A architecture
docs are Status: REVIEWED** (verified on disk). Awaiting: user sign-off →
FROZEN ×4 → delete docs/engine-plan.md (superseded by engine-seam.md) →
commit → compose B1-B5 batch → author the slice-0 port-slice workflow (builds
on the `skeleton` branch, 7 green checkpoints, seed 497fff4 → 4887d0c).
On freeze, also record the parked post-parity seam-inversion DEC (below).

## Parked until Group A settles (user, 2026-07-03)

**Post-parity seam inversion — record as a new DEC after this round stamps,
not before.** Direction discussed and endorsed in principle: once oracle
parity is proven per subsystem, the generic transport layer
(`Execute<C>`/`Dispatch<C>`, typed calls) becomes the true call path wherever
both endpoints are ours; the Raven ABI shapes solidify into a frozen,
layout-asserted compatibility shell engaged only at foreign-endpoint edges
(original DLLs, real engines, wire peers), with dual-path CI so the shell
never rots. User's standing priority during the port: **possibility over
pursuit** — keep the inversion possible, change nothing toward it now; syntax
and semantics of Raven behavior are what matter most during this process.

## Also carried from the seed report

- SP mirror surface: initially deferred (Group-A froze MP only), then
  **UN-DEFERRED by the user (2026-07-03)** once the SP access discipline was
  settled — seeding it early validates the engine logic is sound for SP
  continued work. Skeleton checkpoint 3 seeds it per the settled GetGameAPI
  mapping above.
- Per-call `trap::*` wrappers, `Dispatch<C>` impls, dispatcher match arms are
  logic-port throughput, left as `//TODO: Port` module markers.
- `extern "C"`→`"C-unwind"` sweep of qvm/sp_game stubs = SEAM-D12 follow-up
  slice, untouched.
