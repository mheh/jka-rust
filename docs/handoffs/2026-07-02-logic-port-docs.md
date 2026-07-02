# Handoff — logic-port design-doc phase (2026-07-02, overnight pause)

Repo: `/Users/milohehmsoth/Developer/Milo/jka-rust`, branch `crate-migration`.
All commits local, NOT pushed. Working tree at handoff: this handoff dir +
`docs/architecture/engine-seam.md` (WIP draft, committed as WIP) + dossiers.

## Binding working model (do not deviate)

Per user direction (saved in auto-memory `interactive-design-delegated-execution`):
**design decisions are made interactively with the user** — Fable presents
recommendation vs. alternatives, the user decides; delegated agents (Opus/Sonnet)
execute only settled work (surveys, drafts to a decided outline, adversarial
review, dry-run gates). Never let an agent (or yourself) self-resolve a design
fork. Approved master plan:
`/Users/milohehmsoth/.claude/plans/before-we-run-the-reflective-fountain.md`.

## Where everything is written down (do not re-derive)

- `docs/decisions.md` — DEC-01..09 ledger + 2 dated amendments. Cite, never re-litigate.
- `docs/doc-standards.md` — doc template + the 3 gates (mechanical / adversarial / dry-run).
- `docs/architecture/two-island-model.md` — adopted STATE-D1 visualization.
- `.claude/workflows/author-design-doc.js` — the draft+gates workflow (drafters
  make NO design decisions; contested points return as `needsSession`).
- `docs/dossiers/*.md` — 9 survey dossiers (A1-A4, B1-B5), committed in the
  handoff commit for durability (previously gitignored scratch). They are the
  ground-truth inputs for drafting; every claim cited to oracle file:line.
- `docs/handoffs/group-a-batch-args.json` — the EXACT args for the Group A
  drafting batch (see next section).

## State: what is settled vs. drafted

**Settled interactively (all decisions final, recorded in the args file + here):**
- Phase 0: DEC-01..09 (renderer deferred/wgpu-later, winit, cpal+faithful mixer,
  strict per-mode w/ post-parity-unify option, module-loading scope incl. WASM
  first-class + i686-only retail hosting, full 1.01 wire compat, SP static-via-shim,
  panic+catch_unwind, TU-harnesses+live-peers verification).
- A1 engine-seam: SEAM-D1..D6 + slice-0 = NativeDll. Incl. the escalation-session
  resolutions: hand-written From/TryFrom enum conversions; RegisterSharedMemory is
  LIVE (engine handler `sv_game.cpp:940` → `sv.mSharedMemory`) — second
  registration in the SharedGameData family, slice 0 dispatches it.
- A2 state-ownership: STATE-D1..D4 (two-island model; GameWorld+raw seam with the
  user's MULTI-WORLD retrofit constraint; teardown-before-panic; structs+replaced
  allocator). Corrections folded in: files_pc "shadowing bug" is dead COMMENT text
  (no bug to model); MAX_FILE_HANDLES=64 on PC MP; FS_ReadFile live path is Z_Malloc.
- A3 lifecycle: LIFE-D1..D4 (winit owns loop w/ spin preserved; per-mode Com_*,
  2 bins via `dedicated` feature; typed panic payload, per-mode ERR enums;
  journaling in slice 0, Instant-backed sys_milliseconds, Drop for NET, polled console).
- A4 module-loading: LOAD-D1..D5 (shared loader mechanism/per-mode policy;
  drop+reload + **wasm in-place reset fast path (user choice)** with
  restart-equivalence parity test; WasmPtr<T> host views; one crate per module,
  cfg-gated entrypoints; no current-module global — VM_Free bug unreproducible).
- Wave-3a (B1-B5) sessions COMPLETE, drafts NOT launched:
  - B1 cvar-cmd: faithful two-string latch; bitmask modifiedFlags; owned tokenizer
    scratch; rebuild-on-demand info-strings; MP >1023-byte Cbuf split bug preserved.
  - B2 filesystem: bundle + **case-insensitive VFS fallback** (deliberate divergence,
    2-line note); fixed handle arrays 64/16; Rust zip crate w/ byte-exact checksum
    math; no homepath; per-mode pack_t; live Z_Malloc behavior.
  - B3 collision: bundle + **epoch table NOW** (user chose redesign over faithful
    &mut checkcount — defensible because checkcount is internal dedup, never
    observable in trace results; TU goldens must prove trace-result equivalence);
    instance-shaped CollisionWorld; synthetic fixtures (no .bsp in repo).
  - B4 network: transcribe delta tables + goldens (traps documented: huffman seed =
    unmodified Q3:TA table at msg.cpp:2958, NOT the dead JK2 one; shipped
    playerStateFields = the _OPTIMIZED_VEHICLE_NETWORKING vehicle-split variant at
    msg.cpp:1410); std::net polled sleep (no mio); loopback as NetSource variant;
    SP is protocol-40 loopback-only (wire compat is MP-only).
  - B5 server: faithful client_t fixed array; svEntity/worldSector as indices in a
    per-world value; shared snapshot ring faithful; immediate configstring
    broadcast; per-mode server crates. Dossier gems: 45-step SV_SpawnServer script;
    NO `begin` command exists (first usercmd while PRIMED enters world);
    snapshot vis is a linear PVS scan, not a worldSector walk.

**Drafted:** `docs/architecture/engine-seam.md` exists as a good DRAFT that went
through review once (first run `wf_17ce58be-bd6` → NEEDS_SESSION; all its
escalations were resolved in session and are baked into the args file's A1
revision entry). The Group A batch run (`wf_b72dd8d0-82d`: A1 revise → A2 →
A3∥A4) was **stopped mid-A1-revision** for the overnight pause.

## FIRST ACTION tomorrow

Relaunch the Group A batch **fresh** (workflow resume is same-session-only; the
"cache" is the run's local journal, which a new session can't resume):

```
Workflow({
  scriptPath: ".claude/workflows/author-design-doc.js",
  args: <contents of docs/handoffs/group-a-batch-args.json>
})
```

The A1 entry is a revise-in-place pass over the existing draft, so no work is
lost. When the batch completes: report gate results to the user → user answers
any `needsSession` escalations → user signs off REVIEWED docs → flip Status to
FROZEN → commit → delete `docs/engine-plan.md` (superseded by A1) → then compose
and launch the B1-B5 batch (decisions above + dossiers; same workflow, B5
`depends` on B1/B2/B4 paths).

## Then (per the master plan)

1. B-batch drafts+gates → session for escalations → freeze.
2. Group M surveys (mp-game, mp-cgame, mp-ui, sp-game, sp-cgame-ui) → briefs →
   interactive sessions (M1/M4 full tier) → drafts.
3. B6-B10 (client, renderer-as-deferral-doc per DEC-01, sound, platform, botlib),
   B-cpp designPath docs (ghoul2 first), then C1 logic-port-plan LAST.
4. Set-level cross-doc consistency review, then Slice 0 execution is the ultimate
   doc test.

## Gotchas for the next session

- zsh: `echo ===` fails (`=cmd` expansion); quote it. Unquoted `$VAR` doesn't split.
- rust-analyzer stale — trust only cargo. Oracle (`oracle/oracle/`) is never edited.
- The author-design-doc workflow name may not be registry-visible immediately —
  invoke by `scriptPath`.
- AskUserQuestion ASCII diagrams: user couldn't see box-drawing chars earlier —
  use plain ASCII and/or write diagrams to repo files.
- Dossiers are now committed; if you regenerate any, keep them committed.
- GP2 pilot + port-cpp-subsystem workflow exist and are separate from this doc
  phase (see `docs/type-port-todo.md` "C++ track" section).

## Suggested skills

- **Workflow tool** with `.claude/workflows/author-design-doc.js` — the doc
  pipeline (first action above).
- **/handoff** — at the end of the next session, same drill.
- `/port-types`, `port-cpp-subsystem`, `port-wave` — the porting-phase machinery;
  not needed until docs freeze and slices begin.
