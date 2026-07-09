---
files:
  - path: crates/mp/engine/qcommon/src/roff/mod.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "(module)"
    summary: "Module doc + re-exports; ROFF_* consts (ROFF_VERSION=1, ROFF_NEW_VERSION=2, ROFF_STRING, ROFF_SAMPLE_RATE=10)."
  - path: crates/mp/engine/qcommon/src/roff/roff_system.rs
    crate: mp_engine_qcommon
    mode: mp
    class: CROFFSystem
    summary: "The singleton: owns the cache map + playback list + unique-ID counter; all cache/playback/cleanup/debug methods."
  - path: crates/mp/engine/qcommon/src/roff/croff.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "CROFFSystem::CROFF"
    summary: "One cached .rof file: id, path, decoded move/rotate list, frame time, lerp rate, note-track blob, client/server-used flags."
  - path: crates/mp/engine/qcommon/src/roff/sroff_entity.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "CROFFSystem::SROFFEntity"
    summary: "One playback-list entry: which entity is roffing, which roff, next-roff time, current frame, kill/signal/translate/client flags, start angles."
  - path: crates/mp/engine/qcommon/src/roff/header.rs
    crate: mp_engine_qcommon
    mode: mp
    class: "TROFFHeader/TROFFEntry/TROFF2Header/TROFF2Entry"
    summary: "#[repr(C)] on-disk binary layouts for the two ROFF file formats; used only for bit-exact parsing (ROFF-D3). Not a persisted type."
divergences:
  - id: ROFF-V1
    site: "oracle/codemp/qcommon/RoffSystem.cpp:101"
    note: "IsROFF's `!strcmp(hdr->mHeader, ROFF_STRING)` reads mHeader (char[4], no NUL) as a C-string that runs into mVersion's low byte; the compare accidentally passes valid files. Reproduce the 5-byte-with-trailing-version compare faithfully (§19); do NOT 'fix' to a 4-byte memcmp."
  - id: ROFF-V2
    site: "oracle/codemp/qcommon/RoffSystem.cpp:907"
    note: "ApplyROFF derefs `ent->next_roff_time` even on the client path where `ent` is left NULL (only set in the server branch) — Raven UB, avoided only because the client branch is `#ifndef DEDICATED`. Dedicated target never hits it (see ROFF-Q2)."
  - id: ROFF-V3
    site: "oracle/codemp/qcommon/RoffSystem.cpp:455-509"
    note: "Clean's real body is the `#else` branch: Unload everything, ignore `isClient`, never touch mROFFEntList. The `#if 0` per-client-selective variant is dead. Port the live branch only; `isClient` is unused there (§20)."
  - id: ROFF-V4
    site: "oracle/codemp/qcommon/RoffSystem.cpp:719-734,637-671"
    note: "PurgeEnt(char*) and ListEnts are entitySystem-stubbed (rjr/bjg): PurgeEnt-by-name always returns qfalse, ListEnts is empty. Port as faithful no-ops with §20 zero-live-caller notes."
  - id: ROFF-V5
    site: "oracle/codemp/qcommon/RoffSystem.cpp:41-53,228-236"
    note: "CROFF dtor scalar-deletes an array (`delete mNoteTrackIndexes[0]`) and the note tracks are one packed char blob with interior pointers. Rust owns the blob as one `Vec<u8>` (or `String`) plus offset indices; no manual free, layout-free (§F). Either representation is acceptable (free internal, §F/§A1); Golden A pins only the decoded note-track strings, not their in-memory form."
  - id: ROFF-V6
    site: "oracle/codemp/qcommon/RoffSystem.cpp:596-606"
    note: "Play sets `ent->r.mIsRoffing = qtrue` BEFORE the `if (ent == 0)` NULL check. Preserve the write-then-check order under faithful transcription."
---

# CROFFSystem — ROFF Playback Design
Status: DRAFT     Supersedes: none
Decision prefix: ROFF     Ledger deps: DEC-09 (oracle-differential parity)

## Standing context
Links only — never restate:
- `docs/porting-rules.md` — §F (C++-track idiomatic reimplementation), §B (state
  ownership), comment/source-cite rules.
- `docs/doc-standards.md` — this template + gates; rule 6 (C++-track roster).
- `docs/GOAL-engine.md` — pure-Rust dedicated `openjkded`; total scope, no stubs.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — port order/waves; §7 lists
  CROFFSystem among the §F design-first subsystems.
- `docs/handoffs/engine-fork-discovery.md` — settled forks; **fork-2** (global
  placement → Engine sub-structs; no `static mut`), fork-3 (fn-scope statics).
- `docs/workspace-architecture.md` — crate graph (qcommon tier).
- Exemplar: GP2 (`crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`).

## Scope & non-goals
Decides the Rust shape, ownership, seam, and verification of **`CROFFSystem`**
(`oracle/codemp/qcommon/RoffSystem.{h,cpp}`): the .rof file cache, the per-entity
playback list, per-frame entity updates, and the five ROFF syscall arms plus the
outbound note-track vmcall.

Non-goals (punted, with pointers):
- **SP's game-side ROFF** (`oracle/code/game/g_roff.{h,cpp}`) is a *different*,
  game-DLL subsystem, not this qcommon one. Out of scope; not this doc.
- **The client (`#ifndef DEDICATED`) playback branches** — whether the `cgvm`
  paths are ported at all under the dedicated target — is unresolved (ROFF-Q2).
- **The exact crate host / seam-dependency-injection mechanism** for the upward
  server/vm dependencies is unresolved (ROFF-Q1).
- **Layout of the seam entity/trajectory types** (`sharedEntity_t`,
  `trajectory_t`, `trType_t`) is owned by the type-rosetta / server docs, not
  here; this doc only names the fields it touches.

## Raven ground truth
CITE OR OMIT. Paths are `oracle/`-relative.

**One global instance.** `CROFFSystem theROFFSystem;`
(`codemp/qcommon/RoffSystem.cpp:8`; `extern` decl `RoffSystem.h:183`). It holds
three members: `TROFFList mROFFList` (a `map<int, CROFF*>` — cached roffs),
`int mID` (unique-ID generator), and `TROFFEntList mROFFEntList`
(a `vector<SROFFEntity*>` — roffing entities) (`RoffSystem.h:45-51`). Ctor sets
`mID = 0` and clears the ent list; dtor calls `Restart()` (`RoffSystem.h:161-162`).

**Cache path** (`Cache`, `RoffSystem.cpp:298-365`): `GetID` (linear filepath
scan of `mROFFList`, `:378-393`) short-circuits if already cached; else
`FS_ReadFile(file)`, falling back to `va("scripts/%s.rof", <stripped>)` on miss
(`:314-326`). `IsROFF` (`:96-122`) validates header string + version
(1 or 2) + positive count. A `NewID()` (`++mID`, `.h:146`) mints the id, a new
`CROFF` goes into `mROFFList[id]`, and `InitROFF` decodes it; failure `Unload`s
and returns 0. Finally the roff's `mUsedByClient`/`mUsedByServer` flag is set from
`isClient` (`:353-361`). Cache is also called *internally* from ICARUS
(`codemp/icarus/GameInterface.cpp:491,505`, both `qfalse`).

**File formats** (`RoffSystem.h:54-89`). Version 1: `TROFFHeader` = `char[4]`
mHeader, `long` mVersion, `float` mCount; entries `TROFFEntry` = origin[3] +
rotate[3] floats. Version 2: `TROFF2Header` adds `int mFrameRate` and
`int mNumNotes`; entries `TROFF2Entry` add `int mStartNote, mNumNotes`.
`InitROFF` (`:135-174`) handles v1 (defaults `mFrameTime = 1000/ROFF_SAMPLE_RATE`
= 100ms, `mLerp = ROFF_SAMPLE_RATE` = 10, no notes) and delegates to `InitROFF2`
(`:187-245`) when `mVersion == ROFF_NEW_VERSION`; v2 reads frame rate from file
(`mLerp = 1000/mFrameRate`) and copies the packed NUL-terminated note-track
strings into one blob with per-track pointers (`:214-237`). `FixBadAngles`
(`:258-285`, gated on `ROFF_AUTO_FIX_BAD_ANGLES`) wraps any rotate component
`> 180` or `< -180` by ∓360 in place — parity-visible, runs on every load.

**Playback** (`Play`, `:592-624`): resolves `SV_GentityNum(entID)`, sets
`ent->r.mIsRoffing = qtrue`, allocates an `SROFFEntity`, seeds
`mNextROFFTime = svs.time`, `mROFFFrame = 0`, copies `ent->s.apos.trBase` into
`mStartAngles`, and pushes onto `mROFFEntList`. `UpdateEntities(isClient)`
(`:746-808`) iterates `mROFFEntList` in insertion order, skipping entries whose
`mIsClient != isClient`, calls `ApplyROFF`; a false return or missing roff sets
`mKill`; a second pass erases killed entries. `ApplyROFF` (`:820-911`): returns
early if `svs.time < mNextROFFTime`; on server, reads/writes `ent->s.pos`,
`ent->s.apos`, `ent->r.currentOrigin`, `ent->r.currentAngles`; when the frame
index reaches `mROFFEntries` it `SetLerp`s both trajectories to `TR_STATIONARY`,
clears `mIsRoffing`, and returns false (done); otherwise it optionally rotates the
origin offset by `mStartAngles` (`AngleVectors`+`VectorScale`/`VectorMA`,
`:873-883`), `SetLerp`s origin (`TR_LINEAR`) and angles, fires any notes via
`ProcessNote`, advances the frame, sets `mNextROFFTime = svs.time + mFrameTime`,
and writes `ent->next_roff_time`. `SetLerp` (`:1024-1039`) writes
`trType/trTime/trBase` and `trDelta = delta*rate` (or clears delta).
`ClearLerp` (`:973-1011`) forces both trajectories `TR_STATIONARY`.

**Note-track vmcall** (`ProcessNote`, `:927-961`): splits `note` on control chars
and, per non-empty token, calls out via `VM_Call(gvm, GAME_ROFF_NOTETRACK_CALLBACK,
entID, temp)` on the server (client uses `cgvm, CG_ROFF_NOTETRACK_CALLBACK`,
`#ifndef DEDICATED`).

**Cleanup**: `Unload(id)` (`:407-441`) deletes one roff and erases it from the
map; `Clean(isClient)` (`:453-510`, live `#else` branch) Unloads *all*;
`Restart()` (`:66-83`) Unloads all and resets `mID = 0`; `PurgeEnt(int,isClient)`
(`:684-705`) `ClearLerp`s and erases the first matching (client,entID) ent.

**Iteration order is observable**: `mROFFList` is an ordered `map<int,...>`
(ID order) driving `List`/`GetID`; `mROFFEntList` is a `vector` walked in
insertion order by `UpdateEntities` (which fires notes/lerps and prints errors).
ROFF-D2 pins this via goldens.

## State ownership
Mandatory table. The only Raven *global* the survey found is `theROFFSystem`;
its members are fields of that one struct. Rows below the rule are the external
globals ROFF **reads/calls** (owned elsewhere, threaded in — see ROFF-Q1).

| Raven global | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
|---|---|---|---|---|
| `theROFFSystem` (`CROFFSystem`) | `RoffSystem.cpp:8`, `.h:35,183` | `roff::RoffSystem` (crate per ROFF-Q1), a new **top-level** `Engine.roff` field — sibling of the qcommon-hosted `cm: CollisionWorld` (`core/src/engine.rs:32`), NOT nested in `Common` (ROFF-D1 / fork-2) | `Engine::new` writes it in place via `RoffSystem::default()` — non-`ZeroValid` (owns `Vec`/map), the LIFE-Q9 `MaybeUninit` pattern used for `common.modules` (`core/src/engine.rs:88`) | `&mut RoffSystem` reached from the syscall dispatch in `SV_GameSystemCalls` |
| `mROFFList` / `mID` / `mROFFEntList` | `RoffSystem.h:48-51` | fields of `roff::RoffSystem` (owned `Vec`/map; ROFF-D2) | in-struct, zeroed/empty | not a separate global |
| `CROFF` cache entries | `RoffSystem.h:94-118` | `roff::croff::Croff`, owned in `RoffSystem`'s map (ID→Croff), keyed by `id` | `Cache` | by id/handle, never raw ptr (§B5) |
| `SROFFEntity` list entries | `RoffSystem.h:125-139` | `roff::sroff_entity::SroffEntity`, owned in `RoffSystem`'s `Vec` | `Play` | by index, never raw ptr |
| `svs.time` (read) | `server.h:211,232`; used `RoffSystem.cpp:612,828,904` | `mp_engine_server` (`Engine.sv`, server spine) | server | passed as a param / via the server ctx (ROFF-Q1) |
| entity access `SV_GentityNum` | `server.h:349`; used `:594,848,994` | `mp_engine_server` | server | passed as a param / server ctx (ROFF-Q1) |
| game VM handle `gvm` | note-track callout `:957` | module registry (`Engine.common.modules`) | module load | passed as a param / vm dispatcher (ROFF-Q1) |

## Seam definition
ROFF crosses **two** boundaries.

**(a) Inbound engine syscall arms** — `SV_GameSystemCalls` dispatches these
(no module-ABI layout, they are Rust→Rust calls inside the engine). Oracle arms
(`codemp/server/sv_game.cpp:714-728`; enum `codemp/game/g_public.h:241-245`):

| syscall | oracle callee | Rust method (proposed) |
|---|---|---|
| `G_ROFF_CLEAN` | `Clean(qfalse)` → `qboolean` | `fn clean(&mut self) -> bool` |
| `G_ROFF_UPDATE_ENTITIES` | `UpdateEntities(qfalse)` → void | `fn update_entities(&mut self, ctx: &mut RoffSeam) ` |
| `G_ROFF_CACHE` | `Cache((char*)VMA(1), qfalse)` → int | `fn cache(&mut self, file: &str, ctx: &RoffSeam) -> i32` |
| `G_ROFF_PLAY` | `Play(args[1],args[2],args[3],qfalse)` → qboolean | `fn play(&mut self, ent_id: i32, roff_id: i32, do_translation: bool, ctx: &mut RoffSeam) -> bool` |
| `G_ROFF_PURGE_ENT` | `PurgeEnt(args[1], qfalse)` → qboolean | `fn purge_ent(&mut self, ent_id: i32, ctx: &mut RoffSeam) -> bool` |

All five pass `isClient = qfalse` from the server path. `RoffSeam` is the
carrier for the threaded upward deps (`svs.time`, `SV_GentityNum` →
`&mut sharedEntity_t`, `gvm` note-track dispatch). Its exact split is
porter-fillable against the landed server API (waves 20/25) **once ROFF-Q1 is
settled** — it is NOT a module-ABI type, so its shape is free (§F).

**Freeze scope (what is / is not frozen).** The five method names, receiver
mutability, and value parameters (`file: &str`, `ent_id: i32`, `roff_id: i32`,
`do_translation: bool`, and the return types) freeze now. The **single deferred
element** is the `ctx: &RoffSeam` / `&mut RoffSeam` parameter *type*:
`RoffSeam`'s field list / trait shape is defined by ROFF-Q1 (a porter must not
invent it), so at DRAFT the `ctx` type is the one non-frozen slot. In a
first-slice skeleton that deferred slot is transcribed as the standard
unported-work marker — `//TODO: Port RoffSeam` + `// Source:` → ROFF-Q1
(porting-rules §14) — **not** an invented struct/trait; the frozen names,
receiver mutability, and value params are written around it, so the seam arms
compile-shaped while the seam type stays a marked hole. `cache` is the
narrowest arm — its sole upward dep is `FS_ReadFile`/`COM_StripExtension`
(`RoffSystem.cpp:314-326`), i.e. the qcommon filesystem, not
`svs.time`/`SV_GentityNum`/`gvm`; whether it takes the same `RoffSeam`, a
narrower filesystem handle, or no ctx, and how ICARUS's direct call supplies it,
is ROFF-Q3.

**(b) Outbound module vmcall** (ABI seam — engine→game `vmMain` dispatch, the
reverse of `docs/abi-traps.md`'s `trap_*` table; the `GAME_*` enum lives in
`g_public.h`, not the trap table):
`GAME_ROFF_NOTETRACK_CALLBACK` (`int entnum, char *notetrack`,
`codemp/game/g_public.h:766`), issued from `ProcessNote` via `VM_Call(gvm, …)`
(`RoffSystem.cpp:957`). The client twin `CG_ROFF_NOTETRACK_CALLBACK`
(`codemp/cgame/cg_public.h:424`) and the client trajectory getters
(`CG_GET_ORIGIN_TRAJECTORY`/`CG_GET_ANGLE_TRAJECTORY`/`CG_GET_ORIGIN`/
`CG_GET_ANGLES`, `cg_public.h:418-422`) are `#ifndef DEDICATED` — scope per
ROFF-Q2.

**Seam struct fields touched** (layout owned elsewhere): `sharedEntity_t`
`r.mIsRoffing` (`g_public.h:81`), `next_roff_time` (`g_public.h:714`),
`s.pos`/`s.apos` (trajectories), `r.currentOrigin`/`r.currentAngles`;
`trajectory_t` `trType/trTime/trBase/trDelta` (`q_shared.h:2653-2660`);
`trType_t` `TR_STATIONARY`/`TR_LINEAR` (`q_shared.h:2645-2652`).

## Decisions
Rendered from the settled session, in order.

- **ROFF-D1.** We host `CROFFSystem` as a single owned `roff::RoffSystem` value
  held as a field on the `Engine` aggregate (not a `static`/`static mut`).
  Because fork-2 rules every file-scope global becomes a field on its owning
  subsystem struct under `Engine` (`engine-fork-discovery.md:29`), and §B6 gives
  the one true singleton one owned instance. Rejected a Rust global: violates §B3.
  The field is **top-level** on `Engine` (`Engine.roff`), following the
  `cm: CollisionWorld` precedent — a qcommon subsystem singleton held directly on
  `Engine`, not folded into `Common` (whose fields are the cvar/cmd/fs
  subsystem); `core/src/engine.rs:32`. Because it owns non-`ZeroValid`
  containers (`Vec`/map), `Engine::new` writes it in place via
  `RoffSystem::default()` (the LIFE-Q9 `MaybeUninit` pattern, as `common.modules`),
  not in the zeroed mass. Which crate *defines* `RoffSystem` (qcommon vs a higher
  crate) is ROFF-Q1; the top-level field placement holds either way.
- **ROFF-D2.** `mFiles`/`mEntList` (`TROFFList` `map`, `TROFFEntList` `vector`)
  become owned Rust containers (`Vec` + id-keyed map) per §17. Because the map's
  ID order and the vector's insertion order are behavior-visible (`List`,
  `UpdateEntities`, note firing); those orders are pinned by goldens (ROFF-D3
  fixtures). Rejected raw-pointer containers: §B5. The **concrete** container
  type is a free internal choice (§A1 — internals are free) provided it
  reproduces the ascending-ID iteration `List`/`GetID` rely on (e.g.
  `BTreeMap<i32, Croff>`, or any structure walked in ID order); the exact type is
  not frozen because the ROFF-D3 goldens pin the observable order, not the type.
- **ROFF-D3.** The ROFF binary parse (`IsROFF`/`InitROFF`/`InitROFF2`/
  `FixBadAngles`) is reproduced bit-exact for **both** version 1 and version 2,
  with `#[repr(C)]` header/entry structs matching on-disk layout. Because the
  decoded cache is the golden surface; goldens come from retail `.rof` fixtures.
  Rejected a "cleaned-up" reader: §A2 (faithful first) + ROFF-V1.
- **ROFF-D4.** The seam is 1:1 with the syscall-switch callees — one Rust method
  per arm, faithful params, no merging/renaming of the public surface. Because
  `SV_GameSystemCalls` and the ICARUS `Cache` caller must bind unchanged.
  Rejected a collapsed façade: hides the per-arm parity points. The syscall
  switch has exactly **five** ROFF arms (`Clean`, `UpdateEntities`, `Cache`,
  `Play`, `PurgeEnt`; `sv_game.cpp:714-728`) — the five-row seam table above; the
  count is settled. The only count-adjacent open point is whether ICARUS's direct
  `Cache` call is inside the frozen seam (ROFF-Q3).

## Verification strategy
Per DEC-09 and porting-rules **§F (rules 18-20)** — this is a C++-track
subsystem, verified differentially against the unmodified oracle TU, goldens
committed so `cargo test` needs no C++ toolchain.

- **Harness**: `tools/roff-oracle/` compiles the unmodified
  `codemp/qcommon/RoffSystem.cpp` standalone under stub headers (mirroring
  `tools/gp2-oracle/`), stubbing the seam (`FS_ReadFile`, `SV_GentityNum`,
  `svs.time`, `VM_Call`, `Com_Printf`) to capture behavior.
- **Fixture set**: retail `.rof` files covering both formats — at least one
  version-1 and one version-2 with note tracks — plus a bad-angle case
  exercising `FixBadAngles`, and a `scripts/%s.rof` fallback-path case.
- **Golden A — parse/cache**: for each fixture, dump the resulting `CROFF`
  (`mROFFEntries`, `mFrameTime`, `mLerp`, `mNumNoteTracks`, every
  `mMoveRotateList` entry *after* `FixBadAngles`, and the note-track strings)
  plus the `List`/`GetID` ID ordering (ROFF-D2). Rust must reproduce byte-for-byte.
- **Golden B — playback trace**: drive `Play` + N×`UpdateEntities` against the
  stubbed seam, recording per frame the `SetLerp` writes
  (`trType/trTime/trBase/trDelta`), the note-track vmcall emissions
  (`GAME_ROFF_NOTETRACK_CALLBACK` args), `next_roff_time`, and the kill/erase
  decisions and their order. This pins `ApplyROFF`, `ProcessNote`, and
  `UpdateEntities` ordering.
- **Live tie-in**: once the server spine lands (wave 25), the syscall arms run
  under the in-repo A/B referee; ROFF's arms fall out of the whole-syscall diff.
- **UB (§19)**: ROFF-V1/V2/V6 quirks are reproduced, not normalized; the
  NULL-ent client deref (ROFF-V2) is kept out of shared fixtures (dedicated
  target, ROFF-Q2).

## Slice hooks
From `docs/plans/2026-07-08-mp-engine-build-out.md`:
- **ICARUS (wave 12)** calls `theROFFSystem.Cache` (`GameInterface.cpp:491,505`)
  — needs `cache` + the parse path frozen before ICARUS integrates. ICARUS calls
  `Cache` **directly** (not via `SV_GameSystemCalls`) with just a filename; the
  parse/cache path (Golden A) is self-contained and portable now, but how ICARUS
  supplies `cache`'s context arg is ROFF-Q3.
- **`SV_GameSystemCalls` (wave 20)** dispatches the five arms — needs the full
  seam (ROFF-D4) frozen; `RoffSeam`'s server-side deps resolve here.
- **Server spine (wave 25)** provides `SV_GentityNum`, `svs.time`, trajectory
  types, and the `gvm` note-track dispatch — full Golden-B playback parity gates
  on it. Parse-only (Golden A) can be verified earlier (self-contained TU).
- **Seam-body sequencing (dry-run note).** The five seam methods' *full bodies* —
  the parts touching `svs.time`/`SV_GentityNum`/`gvm` — are blocked until
  `RoffSeam` is defined (ROFF-Q1) and the server spine lands (wave 25); Golden A
  (parse/cache) is transcribable and testable before then. This is expected
  sequencing, not a doc gap.
- **First-slice skeleton boundary (dry-run note).** A porter can produce the
  slice **now** without touching any open question: `mod.rs` re-exports + the
  `ROFF_*` consts; `header.rs`'s four `#[repr(C)]` on-disk structs; the whole
  parse/cache Golden-A path (`IsROFF`/`InitROFF`/`InitROFF2`/`FixBadAngles`,
  `Croff`/`SroffEntity` state, the id-keyed container per ROFF-D2); and the five
  `roff_system.rs` seam method *shapes* — frozen names/receiver/value-params of
  the ROFF-D4 table. What the skeleton carries as a **marked** hole, not an
  invented answer: (1) each seam arm's `ctx` type → `//TODO: Port RoffSeam` →
  ROFF-Q1 (per the Seam § "Freeze scope" note); (2) the internal
  (non-seam) methods' `is_client` parameter and the `#ifndef DEDICATED` client
  arms → `//TODO: Port` → ROFF-Q2 — so those method *bodies*/signatures are
  stubbed pending the fork, not shaped speculatively; (3) `cache`'s ICARUS-side
  call convention → ROFF-Q3. The skeleton is complete and compiles-shaped for
  the resolved surface; the three markers are the only deferrals, each pointing
  at its open question, none requiring a porter decision.

## Open questions
MUST be empty at FROZEN. None self-resolved.

- **ROFF-Q1 — crate host, `RoffSeam` shape & roster crate.** The oracle file is
  qcommon, but ROFF *calls up* into the server (`SV_GentityNum`, `svs.time`,
  `RoffSystem.cpp:594,612,828,...`) and the module VM (`gvm`, `:957`). Tree
  ground truth: `mp_engine_server` depends on `mp_engine_qcommon`
  (`server/Cargo.toml:11`), and `mp_engine_qcommon` has **no** engine-crate deps
  (`qcommon/Cargo.toml`) — so qcommon cannot name server/vm types directly. Three
  linked, unresolved sub-questions: (1) **host crate** — move `RoffSystem` up to
  `mp_engine_core` (which defines `Engine` and can name `Server`/`gvm`) vs. keep
  it in `mp_engine_qcommon` and inject the seam; (2) **`RoffSeam` shape** — its
  field list / trait (the carrier for `svs.time`, the `SV_GentityNum → &mut
  sharedEntity_t` accessor, and the `gvm` note-track dispatcher) is defined here,
  and every seam method's `ctx` parameter type depends on it (a porter must not
  invent it — see Seam § "Freeze scope"); (3) the file roster's
  `crate: mp_engine_qcommon` entries are therefore **provisional** — they encode
  sub-question (1)'s second branch, and if (1) picks `mp_engine_core` the roster
  crate flips. The top-level `Engine.roff` field placement (ROFF-D1) holds under
  either branch. Neither the ROFF decisions nor oracle ground truth settles this.
  Needs session.
- **ROFF-Q2 — DEDICATED client branches & the `is_client` internal param.** Two
  linked points. (a) The `#ifndef DEDICATED` `cgvm` playback paths in
  `ApplyROFF`/`ClearLerp`/`ProcessNote`
  (`RoffSystem.cpp:833-843,981-989,951-953`) and the client syscall arms
  (`cl_cgame.cpp:1269-1282`) do not exist on the dedicated target
  (`docs/GOAL-engine.md`): port them (for a later non-dedicated half) vs. exclude
  them now — and where the NULL-ent deref (ROFF-V2, `:907`) falls out. (b)
  Consequently, whether the **internal** (non-seam) methods
  `Play`/`Clean`/`UpdateEntities`/`PurgeEnt`/`ApplyROFF`/`ClearLerp`/`ProcessNote`
  — all of which take/branch on `isClient` in Raven — keep an `is_client: bool`
  parameter (so a later client half can call them) or hard-code the server-only
  path now. This changes every internal method signature in `roff_system.rs`, not
  just the five frozen seam arms (which already pass `isClient = qfalse`). Needs
  session.
- **ROFF-Q3 — is the ICARUS `Cache` edge inside the frozen seam?** The count is
  settled: the syscall switch has exactly **five** ROFF arms (`Clean`,
  `UpdateEntities`, `Cache`, `Play`, `PurgeEnt`; `sv_game.cpp:714-728`) — the
  five-row seam table (ROFF-D4). The open point: `Cache` has an additional
  in-engine caller in ICARUS (`GameInterface.cpp:491,505`) that invokes
  `theROFFSystem.Cache((char*)sVal1, qfalse)` **directly** with just a filename
  and no context. Since `Cache`'s sole upward dep is `FS_ReadFile`
  (`RoffSystem.cpp:314-326`), not the server/vm seam, `cache`'s context parameter
  and how ICARUS supplies it (same `RoffSeam` vs. a narrower filesystem handle
  vs. no ctx) is unresolved and depends on ROFF-Q1's `RoffSeam` shape. Needs
  session.
</content>
</invoke>
