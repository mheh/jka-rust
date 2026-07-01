# Type-port scope & effort estimate

Derived from `docs/oracle-types.md` (mechanically extracted type index of the
Raven headers). Answers "how much is left, and how long?" for the full type
port. See also [[type-port-todo]] for live per-type status.

## Totals

`2,572` type definitions across both trees (headers only):
MP (`codemp/`) 1,334 · SP (`code/`) 1,238. Heavy MP↔SP overlap, but per the
"duplicate first, don't unify" rule each tree is ported independently (SP as a
fast diff against the MP port).

### By kind (both trees)

| Kind | Count |
|---|---:|
| struct | 1,152 |
| typedef (alias) | 638 |
| enum | 335 |
| class (C++) | 316 |
| fn-ptr typedef | 123 |
| union | 8 |

### By subsystem group (type counts)

| Group | Types | Port strategy |
|---|---:|---|
| engine (qcommon, client, server, botlib, icarus, RMG) | 743 | faithful port |
| renderer | 476 | faithful port (layout-critical: `refEntity_t`, `refdef_t`, shaders) |
| game (g_*, bg, q_shared) | 461 | faithful port — current focus |
| other (cgame, ff, Splines, win32/mac/unix, goblib, strings) | 436 | mixed — cgame/ff/Splines port; platform glue replaced |
| vendored (jpeg-6, png, zlib32, mp3code, smartheap) | 296 | **do not port** — swap for Rust crates |
| ui | 78 | faithful port |
| ghoul2 | 26 | faithful port |

The top-level `oracle/oracle/ui/` (menu *asset* files) has no C and is out of scope.

## Scope reduction

- **~296 vendored types** — not hand-ported. Replace with `jpeg-decoder`, `png`,
  `flate2`, `minimp3`, a Rust allocator (smartheap).
- **~316 C++ classes** (Ravl/Ratl/Ragl/Rufl containers + renderer/ghoul2 classes)
  — reimplemented idiomatically, **not** byte-faithfully. Separate workstream;
  many collapse to `Vec`/`HashMap`/slices.
- Leaves **~1,900 faithful C types** to port across both trees.

## Effort tiers (the ~1,900 faithful C types)

| Tier | ~Count | Per-type | Notes |
|---|---:|---|---|
| Trivial (aliases, small enums, fn-ptr sigs) | ~1,000 | 5–10 min | batchable, low-risk |
| Medium structs | ~700 | 15–30 min | field-by-field + `size_of` assert |
| Heavy layout-critical structs | ~200 | 30–90 min | `playerState_t`, `gentity_t`, `gclient_s`, `refEntity_t`, `snapshot_t` … byte-faithful `offset_of!` asserts — the ABI long pole |

## Time

- **By hand, one at a time, verified: ~500–700 engineer-hours** (~3–4 focused
  months). The count is *not* the cost driver — the ~200 layout-critical ABI
  structs are.
- **Agent-assisted batch porting** of the ~1,700 trivial/medium types (scout
  oracle → transcribe → assert → `cargo` verify in parallel) can compress
  wall-clock to roughly **4–8 weeks**. Bottleneck shifts to *verification*
  (cargo builds + offset asserts), not generation. The ~200 heavy structs are
  irreducible and can't be safely rushed regardless of parallelism.

Progress so far: ~40–50 types (the MP `g_local.h` data model). Low single-digit %.
