# Type-port plan — crate-by-crate

The execution plan for porting **all faithful C types** across the whole crate
graph, before game logic. Companion to:
[[type-port-scope]] (how much / how long), [[type-port-todo]] (live per-type
status), [[workspace-architecture]] (crate graph & tiers),
[[porting-rules]] (how to port each type).

## Ordering principle: bottom-up along the dependency graph

Types are ported **bottom-up** through the dependency edges in
`docs/workspace-architecture.md`. A type in a lower tier is a compile-time
dependency of every crate above it, so porting upward means:

- no wave forward-references an unported type;
- each wave `size_of`/`offset_of!`-asserts against types the wave below froze;
- a green `cargo build` at each step validates layout parity incrementally.

**Within each crate: MP first, then SP as a fast diff** (the "duplicate,
don't unify" rule — SP and MP `q_shared.h` differ by ~3,466 lines).

## Two separate workstreams (NOT in the per-type faithful counts)

Per [[type-port-scope]], these are tracked apart from the byte-faithful port:

- **C++ classes (~316)** — Ravl/Ratl/Ragl/Rufl containers, Splines, goblib, and
  renderer/ghoul2/rmg classes. **Reimplemented idiomatically, not
  byte-faithfully**; many collapse to `Vec`/`HashMap`/slices. Home: mostly
  `native/containers`, plus per-subsystem idiomatic types.
- **Vendored (~296)** — jpeg-6, png, zlib32, mp3code, smartheap. **Not ported at
  all** — swapped for Rust crates (`jpeg-decoder`, `png`, `flate2`, `minimp3`, a
  Rust allocator). Also skip eax (client, 77+77 types — vendored SDK).

## Per-crate scope (mechanically bucketed from `oracle-types.md`)

Counts are faithful-C types by owning header; C++/vendored excluded per above.

| Wave | Crate(s) | MP | SP | Status | Layout risk |
|---|---|---:|---:|---|---|
| 0 | `native/math` | q_math | q_math | started (aliases) | low |
| 0 | `native/types` | scalar/handle | scalar/handle | started | low |
| 0 | `native/containers` | C++ track | C++ track | stub | idiomatic |
| 0 | `native/platform` | 25 (replace) | 38 (replace) | started | n/a — replaced |
| 1 | `mp/qshared` · `sp/qshared` | 88 | 65 | 42 / 36 files | **HIGH — ABI long pole** |
| 2 | `mp/bg` · `sp/bg` | 60 | 17 | 4 / 1 | medium |
| 3 | `mp/uishared` · `sp/uishared` | 17 | 17 | **done** (+ tr_types.h into qshared) | low |
| 4 | `*/game` | 71 (+27\*) | 78 (+55\*) | **done** (module types; logic TBD) | high (gentity/gclient — done both trees) |
| 4 | `*/cgame` | 58 | 47 | **done** | medium |
| 4 | `*/ui` | 35 | 9 | **done** | low |
| 5 | `*/engine/qcommon` | 134 | 105 | stub | high (`qfiles.h`, netchan) |
| 5 | `*/engine/botlib` | 43 | ~ | stub | medium |
| 5 | `*/engine/ghoul2` | 20 | 6 | stub | medium |
| 5 | `*/engine/icarus` | 58 | 14 | stub | medium |
| 5 | `*/engine/rmg` | 30 | 31 | stub | mostly C++ track |
| 5 | `*/engine/server` | 14 | 8 | stub | medium |
| 6 | `*/engine/client` | 85 | 67 | stub | medium (skip eax 77+77) |
| 7 | `*/renderer` | 239 | 237 | stub | **HIGH** (`refEntity_t`, `refdef_t`, shaders, mdx) |

\* `game/*` headers (`g_public.h`, `g_shared.h`, …) not yet split across
`qshared`/`bg`/`game`; classified per-file when Wave 4 starts.

## Waves

**Wave 0 — `native/*` foundation.** Finish `native/math` (q_math — the one true
cross-mode port: vec/matrix/axis/angles types; identical MP/SP) and
`native/types` (scalar/handle primitives). Stand `native/containers` up as
idiomatic Rust (C++ track). `native/platform` is *replacement*, not porting.
Unblocks everything above.

**Wave 1 — `qshared` (Tier 0, the ABI long pole).** The ~200 heavy
layout-critical structs concentrate here: `playerState_t`, `entityState_t`,
`trajectory_t`, `trace_t`, `saber*` (done). Byte-faithful `offset_of!` asserts;
**do not batch these** — irreducible cost driver. Finish MP q_shared.h, then SP
diff.

**Wave 2 — `bg`.** Depends only on qshared. `bg_public.h` + vehicles/pmove/saga.
Mostly batchable medium structs.

**Wave 3 — `uishared`.** Small. Depends on qshared+bg.

**Wave 4 — modules (`game`/`cgame`/`ui`).** Mutually independent (crate graph
enforces cgame ⊄ game) → parallelizable once bg+uishared land. MP `game` data
model done; remaining is `g_public.h`/`g_shared.h` classification + cgame/ui.

**Wave 5 — engine core.** `qcommon` first (underpins the rest), then
botlib/ghoul2/icarus/rmg/server. rmg is mostly the C++ track.

**Wave 6 — `client`.** Skip the 154 eax vendor types.

**Wave 7 — `renderer`.** Largest bucket. `tr_local.h`, `mdx_format.h`,
`refEntity_t`/`refdef_t`/shaders are layout-critical; glext/qgl are GL bindings
(mechanical or replaceable).

## Per-crate batching workflow (every wave)

1. **Scout** the crate's oracle header(s) → type list by kind vs `oracle-types.md`.
2. **Split** trivial (aliases/small enums/fn-ptr sigs — batch several per commit)
   vs heavy layout-critical structs (one per commit, `offset_of!` asserts).
3. **Enum-vs-alias fidelity** (the `spectatorState_t`/`alertEvent*` trap):
   `typedef enum` → `#[repr(i32)] enum`; `typedef int` + anon enum →
   `type = c_int` + consts. Never flatten a named enum.
4. **One type per file**, folder mirrors the owning Raven header's subsystem.
5. `cargo build` per commit (rust-analyzer is stale — [[verify-with-cargo-build]]).
6. Update `docs/type-port-todo.md` status.

## Delegation

Batchable trivial/medium types fan out to sub-agents (scout oracle → transcribe →
assert → `cargo` verify). The ~200 heavy layout-critical structs are done
carefully, one per commit — not rushed regardless of parallelism.
