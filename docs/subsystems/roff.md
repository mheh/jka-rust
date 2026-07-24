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
    summary: "The singleton: owns the cache map + playback list + unique-ID counter; the five seam arms + cache/playback/cleanup/debug methods. Upward services (FS, entity, time, note-track VM_Call) reached via `&mut impl EngineHost` (ROFF-D2)."
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
    summary: "#[repr(C)] on-disk binary layouts for the two ROFF file formats; used only for bit-exact parsing (ROFF-D4). The `long` mVersion fields are the on-disk fixed 4-byte width, so `i32`, never `c_long` (ROFF-D4). Not a persisted type."
divergences:
  - id: ROFF-V1
    site: "oracle/codemp/qcommon/RoffSystem.cpp:101"
    note: "IsROFF's `!strcmp(hdr->mHeader, ROFF_STRING)` reads mHeader (char[4], no NUL) as a C-string that runs into mVersion's low byte; the compare accidentally passes valid files. Reproduce the 5-byte-with-trailing-version compare faithfully (§19); do NOT 'fix' to a 4-byte memcmp."
  - id: ROFF-V2
    site: "oracle/codemp/qcommon/RoffSystem.cpp:835-844,907"
    note: "Under the WinDed/DEDICATED macro set (ROFF-D3) the `if (roff_ent->mIsClient)` branch of ApplyROFF is EMPTY — its whole body is `#ifndef DEDICATED` (835-843) — so `ent` is only ever set in the server `else` (846-848) and the :907 `ent->next_roff_time` deref never sees a NULL. The empty client branch is a §20 zero-caller drop; the Raven UB never ports."
  - id: ROFF-V3
    site: "oracle/codemp/qcommon/RoffSystem.cpp:455-509"
    note: "Clean's real body is the `#else` branch: Unload everything, ignore `isClient`, never touch mROFFEntList. The `#if 0` per-client-selective variant is dead. Port the live branch only; `clean`'s `is_client` param is present (faithful signature) but unused there (§20)."
  - id: ROFF-V4
    site: "oracle/codemp/qcommon/RoffSystem.cpp:719-734,637-671"
    note: "PurgeEnt(char*) and ListEnts are entitySystem-stubbed (rjr/bjg): PurgeEnt-by-name always returns qfalse, ListEnts is empty. Port as faithful no-ops with §20 zero-live-caller notes."
  - id: ROFF-V5
    site: "oracle/codemp/qcommon/RoffSystem.cpp:41-53,228-236"
    note: "CROFF dtor scalar-deletes an array (`delete mNoteTrackIndexes[0]`) and the note tracks are one packed char blob with interior pointers. Rust owns the blob as one `Vec<u8>` (or `String`) plus offset indices; no manual free, layout-free (§F). Either representation is acceptable (free internal, §F/§A1); Golden A pins only the decoded note-track strings, not their in-memory form."
  - id: ROFF-V6
    site: "oracle/codemp/qcommon/RoffSystem.cpp:596-606"
    note: "Play sets `ent->r.mIsRoffing = qtrue` BEFORE the `if (ent == 0)` NULL check. Preserve the write-then-check order under faithful transcription."
  - id: ROFF-V7
    site: "oracle/codemp/qcommon/RoffSystem.cpp:344-353"
    note: "On InitROFF failure Raven does not return early; it falls through to `cROFF = (*mROFFList.find(0)).second` — an end()-iterator deref (NewID never mints 0, `.h:146`), oracle UB. Guard-and-return per §19 (ROFF-D5): Unload the roff, FS_FreeFile, `return 0` immediately — never reach the find(0) deref or the mUsedBy* write. Kept out of the shared Golden-A fixtures (valid fixtures never reach it)."
---

# CROFFSystem — ROFF Playback Design

> **As-built freeze record.** The signature blocks in this doc are the
> frozen shapes the port was transcribed INTO, quoted as of their freeze
> commit. The idiom-era campaigns (2026-07: #13/#20 `&str`/`String`/`bool`
> waves, DEC-35 typed model views, #19 ctx threading) have since evolved
> some of them — the live crate source is authoritative for CURRENT
> signatures; divergences from today's code are not errors in this record.

Status: FROZEN (user sign-off 2026-07-09)     Supersedes: none
Decision prefix: ROFF     Ledger deps: DEC-09 (oracle-differential parity)

## Standing context
Links only — never restate:
- `docs/porting-rules.md` — §F (C++-track idiomatic reimplementation), §B (state
  ownership), §19/§20 (UB / dead-surface notes), comment/source-cite rules.
- `docs/doc-standards.md` — this template + gates; rule 6 (C++-track roster).
- `docs/GOAL-engine.md` — pure-Rust dedicated `openjkded`; total scope, no stubs.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — port order/waves; §7 lists
  CROFFSystem among the §F design-first subsystems; Stage 0 (the game-host
  interface crate that defines `EngineHost`); appendix (WinDed Release macro set,
  `-DNDEBUG -DDEDICATED -DBOTLIB`, FINAL_BUILD undefined).
- `docs/handoffs/engine-fork-discovery.md` — settled forks + §F doc rulings:
  **RULING 11** (one `EngineHost` services trait), **RULING 12** (direct
  `Engine` fields), **RULING 14** (hand-authored, tool-compiled goldens; no
  retail blobs), **RULING 17** (§20 dead-surface drops), **RULING 22** (the
  `Cache` `InitROFF`-failure fallthrough → guard-and-return per §19, closing
  ROFF-Q4), and the **ROFF-Q3**
  evidence resolution (five roff syscall arms).
- `docs/workspace-architecture.md` — crate graph (qcommon tier; core defines
  `Engine`).
- Exemplar: GP2 (`crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`).

## Scope & non-goals
Decides the Rust shape, ownership, seam, and verification of **`CROFFSystem`**
(`oracle/codemp/qcommon/RoffSystem.{h,cpp}`): the .rof file cache, the per-entity
playback list, per-frame entity updates, and the five ROFF syscall arms plus the
outbound note-track vmcall.

Non-goals (punted, with pointers):
- **SP's game-side ROFF** (`oracle/code/game/g_roff.{h,cpp}`) is a *different*,
  game-DLL subsystem, not this qcommon one. Out of scope; not this doc.
- **The exact method roster of the `EngineHost` trait** (which FS/entity/time/
  VM_Call methods it exposes) is owned by the **Stage-0 interface-crate design**
  (`docs/plans/2026-07-08-mp-engine-build-out.md:250`, RULING 11), not here. This
  doc names the services ROFF consumes and takes `&mut impl EngineHost`; it does
  not define the trait.
- **Layout of the seam entity/trajectory types** (`sharedEntity_t`,
  `trajectory_t`, `trType_t`) is owned by the type-rosetta / server docs, not
  here; this doc only names the fields it touches.

## Raven ground truth
CITE OR OMIT. Paths are `oracle/`-relative.

**One global instance.** `CROFFSystem theROFFSystem;`
(`codemp/qcommon/RoffSystem.cpp:8`; `extern` decl `RoffSystem.h:183`). It holds
three members: `TROFFList mROFFList` (a `map<int, CROFF*>` — cached roffs,
`RoffSystem.h:45,48`), `int mID` (unique-ID generator, `:49`), and
`TROFFEntList mROFFEntList` (a `vector<SROFFEntity*>` — roffing entities,
`:46,51`). Ctor sets `mID = 0` and clears the ent list (`:161`); dtor calls
`Restart()` (`:162`).

**Cache path** (`Cache`, `RoffSystem.cpp:298-365`): `GetID` (linear filepath
scan of `mROFFList`, `:378-393`) short-circuits if already cached; else
`FS_ReadFile(file)`, falling back to `va("scripts/%s.rof", <stripped>)` on miss
(`:314-326`). `IsROFF` (`:96-122`) validates header string + version
(1 or 2) + positive count. A `NewID()` (`++mID`, `.h:146`; increments before
returning, so it never mints 0) mints the id, a new `CROFF` goes into
`mROFFList[id]`, and `InitROFF` decodes it. On `InitROFF` **failure the code does
not return early** (`:344-348`): it `Unload`s the roff (deletes it and erases
`mROFFList[id]`) and resets `id = 0`, then **falls through** to
`cROFF = (*mROFFList.find( id )).second` (`:353`). Because `NewID` never yields 0,
that `mROFFList.find(0)` returns `end()` and the deref is UB; the port
guards-and-returns per §19 (**ROFF-D5** / ROFF-V7). On success the roff's
`mUsedByClient`/`mUsedByServer` flag is set from `isClient` (`:354-361`) and `id`
is returned. Cache is also called *internally* from ICARUS
(`codemp/icarus/GameInterface.cpp:491,505`, both `qfalse`).

**File formats** (`RoffSystem.h:54-89`). Version 1: `TROFFHeader` = `char[4]`
mHeader, `long` mVersion, `float` mCount; entries `TROFFEntry` = origin[3] +
rotate[3] floats. Version 2: `TROFF2Header` adds `int mFrameRate` and
`int mNumNotes`; entries `TROFF2Entry` add `int mStartNote, mNumNotes`. These
on-disk structs were authored by the 32-bit-Windows retail exporter where `long`
is 4 bytes, so both `mVersion` fields occupy a fixed 4-byte slot in the file
(v1 header = 12 bytes, v2 header = 20 bytes); the Rust width follows from ROFF-D4
(bit-exact) — see there.
`InitROFF` (`:135-174`) handles v1 (defaults `mFrameTime = 1000/ROFF_SAMPLE_RATE`
= 100ms, `mLerp = ROFF_SAMPLE_RATE` = 10, no notes) and delegates to `InitROFF2`
(`:187-245`) when `mVersion == ROFF_NEW_VERSION`; v2 reads frame rate from file
(`mLerp = 1000/mFrameRate`) and copies the packed NUL-terminated note-track
strings into one blob with per-track pointers (`:214-237`). `FixBadAngles`
(`:258-285`, gated on `ROFF_AUTO_FIX_BAD_ANGLES`) wraps any rotate component
`> 180` or `< -180` by ∓360 in place — parity-visible, runs on every load.

**Playback** (`Play`, `:592-624`): resolves `SV_GentityNum(entID)`, sets
`ent->r.mIsRoffing = qtrue` (`:596-606`, before the NULL check — ROFF-V6),
allocates an `SROFFEntity`, seeds `mNextROFFTime = svs.time`, `mROFFFrame = 0`,
copies `ent->s.apos.trBase` into `mStartAngles`, and pushes onto `mROFFEntList`.
`UpdateEntities(isClient)` (`:746-808`) iterates `mROFFEntList` in insertion
order, skipping entries whose `mIsClient != isClient`, calls `ApplyROFF`; a false
return or missing roff sets `mKill`; a second pass erases killed entries.
`ApplyROFF` (`:820-911`): returns early if `svs.time < mNextROFFTime`; the
`if (mIsClient)` branch is empty under DEDICATED (ROFF-V2), the server `else`
(`:846-859`) reads/writes `ent->s.pos`, `ent->s.apos`, `ent->r.currentOrigin`,
`ent->r.currentAngles`; when the frame index reaches `mROFFEntries` it `SetLerp`s
both trajectories to `TR_STATIONARY`, clears `mIsRoffing`, and returns false
(done); otherwise it optionally rotates the origin offset by `mStartAngles`
(`AngleVectors`+`VectorScale`/`VectorMA`, `:873-883`), `SetLerp`s origin
(`TR_LINEAR`) and angles, fires any notes via `ProcessNote`, advances the frame,
sets `mNextROFFTime = svs.time + mFrameTime`, and writes `ent->next_roff_time`
(`:907`). `SetLerp` (`:1024-1039`) writes `trType/trTime/trBase` and
`trDelta = delta*rate` (or clears delta). `ClearLerp` (`:973-1011`) forces both
trajectories `TR_STATIONARY`.

**Note-track vmcall** (`ProcessNote`, `:927-961`): splits `note` on control chars
and, per non-empty token, calls out via
`VM_Call(gvm, GAME_ROFF_NOTETRACK_CALLBACK, entID, temp)` on the server (`:957`).
The client twin (`cgvm, CG_ROFF_NOTETRACK_CALLBACK`, `:951-952`) is
`#ifndef DEDICATED` — compiled out under WinDed (ROFF-D3, §20).

**Cleanup**: `Unload(id)` (`:407-441`) deletes one roff and erases it from the
map; `Clean(isClient)` (`:453-510`, live `#else` branch) Unloads *all*;
`Restart()` (`:66-83`) Unloads all and resets `mID = 0`; `PurgeEnt(int,isClient)`
(`:684-705`) `ClearLerp`s and erases the first matching (client,entID) ent.

**Iteration order is observable**: `mROFFList` is an ordered `map<int,...>`
(ID order) driving `List`/`GetID`; `mROFFEntList` is a `vector` walked in
insertion order by `UpdateEntities` (which fires notes/lerps and prints errors).
ROFF-D4 pins this via goldens.

### Method transcription
Public + key internal methods, their upward-service needs, and which golden pins
them. Internal-method signatures are free (§A1); the five seam arms freeze (see
`## Seam definition`). All `is_client` params arrive `false` under DEDICATED.

| Raven method | oracle | Rust | host services | golden |
|---|---|---|---|---|
| `Cache(char*,qboolean)` | `:298-365` | `cache` (seam) | FS_ReadFile | A |
| `IsROFF` / `InitROFF` / `InitROFF2` / `FixBadAngles` | `:96-122,135-285` | private parse helpers on `RoffSystem` | none | A |
| `GetID` / `NewID` / `List` | `:378-393`,`.h:146`,`:522-535,548-578` | private | none | A |
| `Unload(int)` / `Restart` | `:407-441,66-83` | private | none | A |
| `Clean(qboolean)` | `:453-510` | `clean` (seam) | none (Unload-all; ROFF-V3) | A |
| `Play(int,int,qboolean,qboolean)` | `:592-624` | `play` (seam) | SV_GentityNum, svs.time | B |
| `UpdateEntities(qboolean)` | `:746-808` | `update_entities` (seam) | (via ApplyROFF) | B |
| `ApplyROFF` | `:820-911` | private | SV_GentityNum, svs.time | B |
| `ProcessNote` | `:927-961` | private | VM_Call note-track | B |
| `SetLerp` / `ClearLerp` | `:973-1039` | private | SV_GentityNum (ClearLerp) | B |
| `PurgeEnt(int,qboolean)` | `:684-705` | `purge_ent` (seam) | SV_GentityNum (ClearLerp) | B |
| `PurgeEnt(char*)` / `ListEnts` | `:719-734,637-671` | faithful no-ops (ROFF-V4, §20) | none | — |

## State ownership
Mandatory table. The only Raven *global* the survey found is `theROFFSystem`;
its members are fields of that one struct. Rows below the rule are the external
services ROFF **reads/calls**, owned elsewhere and reached through the
`EngineHost` trait (RULING 11) — not threaded as individual params.

| Raven global/service | oracle cite | Rust owner (crate::Type.field) | constructed by | threaded via |
|---|---|---|---|---|
| `theROFFSystem` (`CROFFSystem`) | `RoffSystem.cpp:8`, `.h:161,183` | `roff::RoffSystem`, a **direct top-level `Engine.roff` field** — sibling of `cm: CollisionWorld` (`core/src/engine.rs:32`); RULING 12 (no Option/Box/nesting) | `Engine::default` initializes it in place via `RoffSystem::default()` (owns `Vec`/map; RULING 12 models lazy-init with Raven's own flags) | owned; its methods reach services via `&mut impl EngineHost` |
| `mROFFList` / `mID` / `mROFFEntList` | `RoffSystem.h:45-51` | fields of `roff::RoffSystem` (id-keyed map + `Vec` + `i32`; ROFF-D4) | in-struct, zeroed/empty | not a separate global |
| `CROFF` cache entries | `RoffSystem.h:94-118` | `roff::croff::Croff`, owned in `RoffSystem`'s map (ID→Croff) | `Cache` | by id/handle, never raw ptr (§B5) |
| `SROFFEntity` list entries | `RoffSystem.h:125-139` | `roff::sroff_entity::SroffEntity`, owned in `RoffSystem`'s `Vec` | `Play` | by index, never raw ptr |
| `FS_ReadFile` (read) | `RoffSystem.cpp:314-326` | qcommon filesystem (`Engine.fs`) | fs init | `EngineHost` FS service (RULING 11) |
| `svs.time` (read) | `server.h:211,232`; used `:612,828,904` | `mp_engine_server` (server spine) | server | `EngineHost` (RULING 11) |
| entity access `SV_GentityNum` | `server.h:349`; used `:594,848,994` | `mp_engine_server` / shared-memory gentity array | server | `EngineHost` shared-memory accessor (RULING 11) |
| game VM handle `gvm` (note-track) | `RoffSystem.cpp:957` | module registry (`Engine.common.modules`) | module load | `EngineHost` VM_Call service (RULING 11) |

**Crate host (ROFF-Q1 resolved).** `RoffSystem` stays defined in
`mp_engine_qcommon` (its Raven home). It names the `EngineHost` trait from the
Stage-0 interface crate (traits only, low tier), so qcommon takes a dependency on
that crate — it does **not** name `mp_engine_server` types directly, dissolving
the old dependency-inversion concern. `core` holds the `Engine.roff` field
(`core` already depends on qcommon: `core/src/engine.rs:4`). Cross-subsystem
reach (ICARUS → `roff.cache`) is served by RULING 11's split-borrow view struct.

## Seam definition
ROFF crosses **two** boundaries.

**(a) Inbound engine syscall arms** — `SV_GameSystemCalls` dispatches these
(no module-ABI layout; Rust→Rust inside the engine). The switch has exactly
**FIVE** roff arms (ROFF-D1; `codemp/server/sv_game.cpp:714-728`; enum
`codemp/game/g_public.h:241-245`), each dispatched with `is_client = false`:

| syscall | oracle callee | Rust method (frozen) |
|---|---|---|
| `G_ROFF_CLEAN` | `Clean(qfalse)` → `qboolean` | `fn clean(&mut self, is_client: bool) -> bool` |
| `G_ROFF_UPDATE_ENTITIES` | `UpdateEntities(qfalse)` → void | `fn update_entities(&mut self, is_client: bool, host: &mut impl EngineHost)` |
| `G_ROFF_CACHE` | `Cache((char*)VMA(1), qfalse)` → int | `fn cache(&mut self, file: &str, is_client: bool, host: &mut impl EngineHost) -> i32` |
| `G_ROFF_PLAY` | `Play(args[1],args[2],(qboolean)args[3],qfalse)` → qboolean | `fn play(&mut self, ent_id: i32, roff_id: i32, do_translation: bool, is_client: bool, host: &mut impl EngineHost) -> bool` |
| `G_ROFF_PURGE_ENT` | `PurgeEnt(args[1], qfalse)` → qboolean | `fn purge_ent(&mut self, ent_id: i32, is_client: bool, host: &mut impl EngineHost) -> bool` |

**Freeze scope.** All five method names, receiver mutability, value parameters,
`host: &mut impl EngineHost` (or its absence on `clean`, which needs no service),
and return types freeze now — ROFF-D2 (EngineHost) settled the former deferred
`ctx` slot, so nothing in this table is a hole. The `is_client: bool` parameter
mirrors Raven's `qboolean isClient` (present in the compiled WinDed signature,
ROFF-D3) and arrives `false` at every server call site; `clean` keeps it for
signature fidelity though its live body ignores it (ROFF-V3). `EngineHost`'s
method roster is defined by the Stage-0 interface crate, not here.

**(b) Outbound module vmcall** (ABI seam — engine→game `vmMain` dispatch, the
reverse of `docs/abi-traps.md`'s `trap_*` table; the `GAME_*` enum lives in
`g_public.h`, not the trap table): `GAME_ROFF_NOTETRACK_CALLBACK`
(`int entnum, char *notetrack`, `codemp/game/g_public.h:766`), issued from
`ProcessNote` via `VM_Call(gvm, …)` (`RoffSystem.cpp:957`) — routed through the
`EngineHost` VM_Call service. The client twin `CG_ROFF_NOTETRACK_CALLBACK`
(`codemp/cgame/cg_public.h:424`) and the client trajectory getters
(`CG_GET_ORIGIN_TRAJECTORY`/`CG_GET_ANGLE_TRAJECTORY`/`CG_GET_ORIGIN`/
`CG_GET_ANGLES`, `cg_public.h:418-422`; used `RoffSystem.cpp:835-843,981-989`)
are `#ifndef DEDICATED` — compiled out under WinDed (ROFF-D3), recorded as §20
zero-caller drops.

**Seam struct fields touched** (layout owned elsewhere): `sharedEntity_t`
`r.mIsRoffing` (`g_public.h:81`), `next_roff_time` (`g_public.h:714`),
`s.pos`/`s.apos` (trajectories), `r.currentOrigin`/`r.currentAngles`;
`trajectory_t` `trType/trTime/trBase/trDelta` (`q_shared.h:2653-2660`);
`trType_t` `TR_STATIONARY`/`TR_LINEAR` (`q_shared.h:2645-2652`).

## Decisions
Rendered from the settled session, in order.

- **ROFF-D1.** The syscall switch has exactly **five** ROFF arms — `G_ROFF_CLEAN`,
  `G_ROFF_UPDATE_ENTITIES`, `G_ROFF_CACHE`, `G_ROFF_PLAY`, `G_ROFF_PURGE_ENT`
  (`sv_game.cpp:714-728`) — so the seam table has five rows, one Rust method per
  arm. Because the count is oracle-evidenced (ROFF-Q3 resolution,
  `engine-fork-discovery.md:165-166`); every four-arm phrasing in earlier drafts
  is a defect. Rejected a collapsed façade: hides the per-arm parity points.
- **ROFF-D2.** ROFF's upward services (FS reads, entity access, `svs.time`, and
  the note-track `VM_Call`) are reached through the **one** `EngineHost` services
  trait (RULING 11), and `CROFFSystem`'s state is a **direct `Engine.roff`
  field** (RULING 12). Because the §F rule threads services as
  `(&mut RoffSystem, &mut impl EngineHost)` and attaches state as a plain
  Default-initialized field on `Engine` (like `cm`, `icarus`, `nav`, `g2`, `rmg`)
  — this resolves ROFF-Q1: `RoffSystem` stays in `mp_engine_qcommon` and names
  the trait from the Stage-0 interface crate, never `mp_engine_server` types.
  Rejected a bespoke `RoffSeam` carrier and a Rust global: RULING 11 mandates the
  single trait; §B3 forbids the global. Cross-subsystem calls (ICARUS →
  `roff.cache`) use RULING 11's split-borrow view struct.
- **ROFF-D3.** Port exactly what the **WinDed Release macro set**
  (`-DNDEBUG -DDEDICATED -DBOTLIB`, FINAL_BUILD undefined; plan appendix) compiles;
  the `#ifndef DEDICATED` client-only branches (ApplyROFF `:835-843`, ClearLerp
  `:981-989`, ProcessNote `:951-952`, and the client syscall/getter twins) are
  §20 zero-caller drops with ≤2-line notes, mirroring RULING 17. Because DEDICATED
  is defined, those bodies do not exist in the ported TU. This resolves ROFF-Q2:
  the compiled `is_client` parameters and `mIsClient != isClient` comparisons are
  kept faithfully (they compile), the client branch bodies are dropped, and the
  ROFF-V2 NULL-ent deref is thereby unreachable. `is_client` arrives `true` only
  from `CL_CgameSystemCalls`' five `CG_ROFF_*` arms (`codemp/client/cl_cgame.cpp:1269-1282`,
  all `qtrue`) — but that file is **not in the WinDed link set** (`null/null_client.cpp`
  replaces it; `WinDed.vcproj` lists no `client\cl_cgame.cpp`), so `is_client == true`
  never enters the ported engine; every live caller (server `sv_game.cpp:714-728`,
  ICARUS `GameInterface.cpp:491,505`) passes `qfalse`. Rejected porting the client
  paths now (out of the DEDICATED build) and rejected deleting the `is_client`
  param (it compiles; §A2 keeps it).
- **ROFF-D4.** `mROFFList`/`mROFFEntList` become owned Rust containers (id-keyed
  map + `Vec` + `i32`) with golden-pinned iteration order; the v1/v2 `.rof` parse
  (`IsROFF`/`InitROFF`/`InitROFF2`/`FixBadAngles`) is reproduced **bit-exact**
  with `#[repr(C)]` header/entry structs. Because the format is fixed 4-byte
  (32-bit-Windows origin; see Raven ground truth), the on-disk `long` fields
  (`TROFFHeader::mVersion`, `TROFF2Header::mVersion`, `RoffSystem.h:58,75`) are
  Rust **`i32`, never `c_long`** — `c_long` is 8 bytes under LP64 and shifts every
  following offset, silently breaking bit-exactness (the sibling SP g_roff port
  did exactly this: `crates/sp/game/src/roff/roff_hdr_s.rs:14` /
  `roff_hdr2_s.rs:16` use `c_long`, and their asserts encode the wrong 24-/32-byte
  LP64 layout instead of the on-disk 12-/20-byte one — a live parity break to not
  repeat). Goldens come from **committed
  hand-authored `.rof` fixtures** (mirror RULING 14: no retail blobs committed).
  Because the map's ID order and the vector's insertion order are behavior-visible
  (`List`, `UpdateEntities`, note firing) and the decoded cache is the golden
  surface (ROFF-V1). The concrete container type is a free internal choice (§A1)
  provided it reproduces ascending-ID iteration (e.g. `BTreeMap<i32, Croff>`).
  Rejected raw-pointer containers (§B5), a "cleaned-up" reader (§A2), and
  committing retail blobs (RULING 14).
- **ROFF-D5.** `Cache`'s `InitROFF`-failure path **guards and returns**: on
  failure it `Unload`s the roff, `FS_FreeFile`s the data, and `return`s 0
  immediately — it never executes the fall-through
  `cROFF = (*mROFFList.find( id )).second` (`RoffSystem.cpp:344-353`). Because
  Raven does *not* return early there and falls into `mROFFList.find(0)`, which is
  `end()` (`NewID` never mints 0, `.h:146`) — an end-iterator deref, oracle UB.
  Per porting-rules §19 the port picks one defined behavior and notes it at the
  site (ROFF-V7): return the failure without touching the map, matching the
  visible intent (`id = 0` already signals failure). Settles ROFF-Q4 (RULING 22,
  2026-07-09). Rejected faithfully modelling the `find(0)` miss as a no-op that
  still writes a `mUsedBy*` flag and returns 0: it either crashes or scribbles a
  flag through a garbage `CROFF*` — §19 forbids shipping the UB.

## Verification strategy
Per DEC-09 and porting-rules **§F (rules 18-20)** — this is a C++-track
subsystem, verified differentially against the unmodified oracle TU, goldens
committed so `cargo test` needs no C++ toolchain.

- **Harness**: `tools/roff-oracle/` compiles the unmodified
  `codemp/qcommon/RoffSystem.cpp` standalone under stub headers (mirroring
  `tools/gp2-oracle/`), compiled with the WinDed macro set (`-DDEDICATED`,
  ROFF-D3) so the oracle TU and the Rust port cover the same code, stubbing the
  seam behind a deterministic `EngineHost` impl (`FS_ReadFile`, `SV_GentityNum`,
  `svs.time`, `VM_Call`, `Com_Printf`) to capture behavior.
- **Fixture set (hand-authored, ROFF-D4/RULING 14)**: `.rof` binaries crafted
  directly to the on-disk layout — at least one version-1 and one version-2 with
  note tracks — plus a bad-angle case exercising `FixBadAngles`, and a
  `scripts/%s.rof` fallback-path case. Committed; **no retail blobs** (a retail
  corpus may run locally, uncommitted).
- **Golden A — parse/cache**: for each fixture, dump the resulting `CROFF`
  (`mROFFEntries`, `mFrameTime`, `mLerp`, `mNumNoteTracks`, every
  `mMoveRotateList` entry *after* `FixBadAngles`, and the note-track strings)
  plus the `List`/`GetID` ID ordering (ROFF-D4). Rust must reproduce byte-for-byte.
- **Golden B — playback trace**: drive `Play` + N×`UpdateEntities` against the
  stubbed `EngineHost`, recording per frame the `SetLerp` writes
  (`trType/trTime/trBase/trDelta`), the note-track vmcall emissions
  (`GAME_ROFF_NOTETRACK_CALLBACK` args), `next_roff_time`, and the kill/erase
  decisions and their order. This pins `ApplyROFF`, `ProcessNote`, and
  `UpdateEntities` ordering.
- **Live tie-in**: once the server spine lands (wave 25), the syscall arms run
  under the in-repo A/B referee; ROFF's arms fall out of the whole-syscall diff.
- **UB (§19/§20)**: ROFF-V1/V6 quirks are reproduced, not normalized; the
  client-branch NULL-ent deref (ROFF-V2) never ports (compiled out under
  DEDICATED, ROFF-D3), so it is absent from the shared fixtures by construction.
  The `Cache` `InitROFF`-failure fallthrough (`map::find(0)` → `end()` deref,
  oracle UB) is guard-and-returned per §19 (**ROFF-D5** / ROFF-V7) and kept out of
  the shared Golden-A fixtures (all valid, none reach it).

## Slice hooks
From `docs/plans/2026-07-08-mp-engine-build-out.md`:
- **Stage 0 (interface crate)** must define `EngineHost` (RULING 11) before the
  five seam arms' bodies compile — the FS/entity/time/VM_Call methods ROFF calls
  live there. The parse path (Golden A) needs only the FS method.
- **ICARUS (wave 12)** calls `theROFFSystem.Cache` (`GameInterface.cpp:491,505`)
  — needs `cache` + the parse path frozen before ICARUS integrates. ICARUS reaches
  `Engine.roff` through the RULING 11 split-borrow view struct and supplies its own
  `&mut impl EngineHost`; the parse/cache path (Golden A) is self-contained.
- **`SV_GameSystemCalls` (wave 20)** dispatches the five arms (ROFF-D1) with
  `is_client = false` — needs the seam frozen.
- **Server spine (wave 25)** backs the `EngineHost` entity/time/VM_Call methods
  (`SV_GentityNum`, `svs.time`, trajectory types, `gvm` note-track dispatch) —
  full Golden-B playback parity gates on it. Parse-only (Golden A) is verifiable
  earlier (self-contained TU + FS stub).
- **First-slice skeleton boundary (dry-run note).** A porter produces the whole
  slice with **no open points**: `mod.rs` re-exports + `ROFF_*` consts;
  `header.rs`'s four `#[repr(C)]` on-disk structs (`mVersion` = `i32`, ROFF-D4);
  the whole parse/cache Golden-A path (`IsROFF`/`InitROFF`/`InitROFF2`/
  `FixBadAngles`, `Croff`/`SroffEntity` state, the id-keyed container per ROFF-D4,
  the `Cache` `InitROFF`-failure guard-and-return per ROFF-D5); and the five
  `roff_system.rs` seam methods with the frozen signatures above (taking
  `&mut impl EngineHost`). The `#ifndef DEDICATED` client branches are §20 drops
  (ROFF-D3), not stubs. The only external dependency is the `EngineHost` trait
  (Stage 0), a cross-doc pointer, not a hole.

## Open questions
Empty — all four questions are settled below (the gate may advance Status to
REVIEWED).

- **ROFF-Q1** (crate host / seam carrier) → **ROFF-D2** (RULING 11 `EngineHost` +
  RULING 12 direct `Engine.roff` field; `RoffSystem` stays in `mp_engine_qcommon`).
- **ROFF-Q2** (DEDICATED client branches / `is_client` param) → **ROFF-D3** (port
  the WinDed macro set; client-only branches are §20 drops; the compiled
  `is_client` params are kept).
- **ROFF-Q3** (five vs four seam arms; ICARUS `Cache` edge) → **ROFF-D1** (five
  arms, oracle-evidenced) + **ROFF-D2** (`cache` takes the same
  `&mut impl EngineHost`; ICARUS supplies its own via the view struct).
- **ROFF-Q4** (Cache `InitROFF`-failure fallthrough — UB path) → **ROFF-D5**
  (guard-and-return per §19: Unload, FS_FreeFile, `return 0`, never the `find(0)`
  deref; site note ROFF-V7; RULING 22, 2026-07-09).
