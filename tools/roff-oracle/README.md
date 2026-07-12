# roff-oracle — differential golden harness for the CROFFSystem port

Verifies the `mp_engine_qcommon` `roff` port (`crates/mp/engine/qcommon/src/roff/`,
design `docs/subsystems/roff.md`, FROZEN) against the **unmodified** Raven
`codemp/qcommon/RoffSystem.cpp`, exactly like `tools/gp2-oracle` /
`tools/trmodel-oracle` / `tools/icarus-oracle` (porting-rules §18). The oracle
`.cpp`/`.h` are copied into `build/` and compiled standalone against stub headers;
canonical dumps are stored under `goldens/` and committed, so the Rust parity
tests need **no** C++ toolchain — only `build.sh` does, to (re)generate or check.

`oracle/` is never edited.

## Usage

```sh
sh build.sh          # build + run both dumpers, diff against goldens/
sh build.sh --regen  # regenerate fixtures/* and goldens/*
```

Toolchain: Homebrew `g++-16` (override with `CXX=`). Flags mirror the sibling
harnesses (`-fsigned-char -ffp-contract=off -fno-fast-math`) plus the **WinDed
DEDICATED Release macro set** the port models (ROFF-D3): **`-DDEDICATED`** (every
`#ifndef DEDICATED` client branch — the `cgvm` getters/note-track twins, the empty
`ApplyROFF` client arm — compiles OUT, so `is_client == true` never enters the
ported code, ROFF-V2/V3) and **`-DNDEBUG`** (the `#ifdef _DEBUG` `Com_Printf`
lines compile OUT — the shipped Release build). Each dumper is run twice and the
two outputs are required byte-identical (determinism guard).

## Goldens (each pins a § Verification-strategy unit)

| Golden | Pins | Doc unit |
| --- | --- | --- |
| `goldens/cache.txt` | **Golden A (parse/cache).** `Cache`→`GetID`/`NewID`/`IsROFF`/`InitROFF`/`InitROFF2`/`FixBadAngles` over every fixture: the returned id, the ascending-`mROFFList` ID ordering (ROFF-D4), and per cached roff `mROFFEntries`/`mFrameTime`/`mLerp`/`mNumNoteTracks`, every `mMoveRotateList` entry **after** `FixBadAngles` (raw IEEE-754 bits), and the decoded note-track strings. Also the two reject paths (bad version, bad count → id 0) and re-cache idempotency. | "Golden A — parse/cache" |
| `goldens/play.txt` | **Golden B (playback trace).** `Play` + N×`UpdateEntities` recording, per frame, the `SetLerp` writes (`trType`/`trTime`/`trBase`/`trDelta` on `s.pos` & `s.apos`, raw bits), `r.mIsRoffing`, `next_roff_time`, the note-track `VM_Call` emissions (`GAME_ROFF_NOTETRACK_CALLBACK` args), and the kill/erase decisions via `mROFFEntList` size. Five scenarios: non-translated v1, translated v1 (the `AngleVectors` path, `mTranslated`), v2 note firing, the roff-not-found error+`ClearLerp` path, and `PurgeEnt` success/miss. | "Golden B — playback trace" |

The Rust parity tests (in `mp_engine_qcommon`) read `fixtures/*` + `goldens/*`
from here and must reproduce every golden exactly, injecting FS/entity/time/VM_Call
behaviour via a deterministic `EngineHost` impl mirroring `host.cpp`.

## Fixtures — `roffgen.cpp` (RULING 14 / ROFF-D4: hand-authored, no retail data)

`roffgen` emits minimal-but-valid `.rof` byte images in the **true ship on-disk
layout** (fixed 4-byte `mVersion`, v1 header = 12 bytes, v2 header = 20 bytes;
ROFF-D4) — the same bytes the Rust `#[repr(C)]` header structs parse. Every field
and offset is spelled out with its `RoffSystem.h` cite. No retail blobs (a retail
`.rof` corpus may run locally, uncommitted).

- **`v1_basic.rof`** — v1, 3 entries; drives non-translated + translated playback.
- **`v1_badangle.rof`** — v1, rotate components outside ±180 (270, -200, 181, -181)
  to exercise `FixBadAngles` (>180 → −360; < −180 → +360).
- **`v2_notes.rof`** — v2, `mFrameRate=50` (→ `mFrameTime=50`, `mLerp=1000/50=20`),
  one note track fired by entry 0 (`mStartNote=0`, `mNumNotes=1`).
- **`scripts/fallbackcase.rof`** — valid v1 placed under `scripts/` so `Cache`'s
  `FS_ReadFile` miss → `va("scripts/%s.rof", …)` fallback path fires.
- **`bad_version.rof`** — header "ROFF" ok but version 99 → `IsROFF` version reject.
- **`bad_count.rof`** — v1 `mCount=0` → `IsROFF` count reject.

## Reproduced UB / quirks (§19/§20)

- **ROFF-V1** (`RoffSystem.cpp:101`): `IsROFF`'s `!strcmp(hdr->mHeader, ROFF_STRING)`
  reads `mHeader` (`char[4]`, no NUL) as a C-string running into `mVersion`'s low
  byte; the nonzero version byte is what makes valid files *pass*. Reproduced
  faithfully (the LP64 shim below copies the ship version bytes into the header pad
  so the `strcmp` behaves identically) — **not** "fixed" to a 4-byte memcmp.
- **ROFF-V2/V3** (client branches): compiled out under `-DDEDICATED`, so the empty
  client `ApplyROFF` arm and its NULL-ent deref never exist in the ported TU;
  `Clean`'s live body is the `#else` (Unload-all, `is_client` ignored). Absent from
  the fixtures by construction.
- **ROFF-V6** (`Play` sets `r.mIsRoffing = qtrue` *before* the `ent == 0` check):
  faithful; the mock `SV_GentityNum` never returns 0 for the valid ids used.
- **ROFF-D5 / ROFF-V7** (the `Cache` `InitROFF`-failure `map::find(0)` end-iterator
  deref): the valid fixtures never reach it; it is a Rust guard-and-return, kept out
  of the shared goldens per §19.

## Normalizations / host model

`host.cpp` is a deterministic `EngineHost` stand-in: a fixture-backed FS
(`FS_ReadFile` under `fixtures/`), a mock gentity array (`SV_GentityNum`), a
controllable `svs.time`, a note-track `VM_Call` log, and console capture
(`Com_Printf` → stdout, part of the golden). `AngleVectors` and `COM_StripExtension`
are copied faithfully from `q_math.c` / `q_shared.c` so the translated-playback
golden is bit-exact.

### The LP64 header shim (ROFF-D4) — read this

The oracle spells `TROFFHeader::mVersion` / `TROFF2Header::mVersion` as **`long`**.
On the shipped 32-bit WinDed target `long` is 4 bytes and the parse matches the
on-disk format. **On this LP64 host `long` is 8 bytes**, so the unmodified oracle
would read `mVersion` (and every following field) at the wrong offset and reject
every valid fixture — and no 32-bit toolchain is available here (`g++-16 -m32` is
unsupported on arm64; macOS i386 no longer links). Because `oracle/` must stay
unedited, `host.cpp`'s `FS_ReadFile` contains a **documented shim** that re-lays the
committed ship-format (4-byte-header) fixture into the host's `long`-width struct
layout before handing it to the oracle. This is guarded by `sizeof(long) == 4`, so
on a real ILP32 build it is a **no-op**. The parsed *values* (version, count,
frameRate, entries, notes, playback lerps) are identical to the ship parse, so the
goldens are ship-faithful and the Rust port (which reads the raw 4-byte fixtures
with `i32` headers, ROFF-D4) reproduces them.

**Coverage boundary (no silent claims):** what the shim leaves *uncovered here* is
the exact 4-byte-`long` header **byte-offset arithmetic** (the 12-byte v1 / 20-byte
v2 header layout) — a pure ROFF-D4 layout concern. That is covered instead by (a)
the Rust header structs' own `size_of`/`offset_of!` static-asserts and (b) the
project's 32-bit CI lane, where `sizeof(long)==4` makes the shim vanish and the
oracle parses the committed fixtures byte-for-byte. Everything else — version
dispatch, count, `FixBadAngles`, the note-track decode, ID ordering, and all of
playback — is fully differential here.
