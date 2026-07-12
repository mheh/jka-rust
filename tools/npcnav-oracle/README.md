# npcnav-oracle — fixture generator + differential-golden harness for CNavigator

Differential-golden harness for the engine-side nav-graph port
(`oracle/codemp/server/NPCNav/navigator.cpp` → the Rust `Navigator`/`Node`/
`Edge`/`PriorityQueue`, design doc `docs/subsystems/npcnav.md`). Follows the
GP2/`ibi-gen` §18 pattern: compile the **unmodified** oracle TU standalone
against stub headers, drive the real build path, commit the emitted bytes +
goldens. Settled by engine-fork-discovery **rulings 42/44/45** and NAV-D1/D2/D3.

## Usage

```sh
sh build.sh            # build, diff current output against fixtures/ + goldens/
sh build.sh --regen    # rebuild fixtures/*.nav and goldens/*.txt
```

The `.nav` fixtures and text goldens are committed, so the Rust parity tests
need no C++ toolchain; `build.sh` is only for regen / spot-check. Toolchain:
Homebrew **g++-16** (libstdc++). `oracle/` is never edited — `build.sh` copies
`navigator.cpp`/`navigator.h` into `build/` next to the stub headers so their
relative `#include`s resolve to the stubs (`stubs/game/`, `stubs/server/`).

## What each layout / fixture pins

Layouts are hand-authored, human-readable (`layouts/*.layout`, format documented
in `line3.layout`): `checksum`, `node <x y z flags radius>`, `connect <a b>`.
The dumper feeds each through the real `AddRawPoint → HardConnect →
CalculatePaths → Save` path and dumps the graph, the per-node **rank tables**,
and the full pure-graph query surface from the same in-memory run.

| fixture | shape | pins |
| --- | --- | --- |
| `line3` | 3-node chain | sanity: linear ranks, trivial paths |
| `diamond` | 4 nodes, two equal-cost 0→3 routes | minimal **equal-cost heap tie-order** witness (NAV-D2); non-default flags/radii |
| `star6` | degree-4 hub + 2 rim cross-links | varied `m_numEdges` per node, hub short-circuit ranks, flags/radii |
| `grid9` | 3×3 lattice | dense equal-cost collisions — the **strongest heap-sift gate** |

Goldens (`goldens/*.txt`) dump: `== graph ==` (nodes/edges), `== rank tables ==`
(each node's raw rank array in `curRank++` pop order — the direct heap-sift
gate), then `GetPathCost`/`GetBestNode`/`GetBestNodeAltRoute`/`Connected`/
`NodesAreNeighbors`/`GetProjectedNode` over all node pairs. The Rust port must
reproduce both the `.nav` bytes (via `Load`) and every golden line.

All edges are axis-aligned so every cost is an exact integer (100), with no
`sqrt` rounding — costs are unambiguous across C and Rust `f32`.

## The 4-byte-`long` shim (NAV-D1 / RULING 44) — mechanism + evidence

Raven's `.nav` format is Win32: every `long`/`unsigned long` in it is 4 bytes
(the `'JNV5'` NAV id and `'NODE'` id, `navigator.cpp:388,428,557-564,614,676`).
This host is LP64 (`long` == 8 bytes), which would double those fields' on-disk
width. The TU must compile **unmodified**, so the shim is a compile-time
arrangement (the `LittleShort=` precedent — flags/stub-headers are not source
edits), implemented in `stubs/game/q_shared.h`:

1. That stub — the **first** include of the TU — pulls in every system/STL
   header the TU needs (`<algorithm> <map> <vector> <list> <time.h> …`)
   **before** the shim is armed, so no libc/libstdc++ header is ever parsed with
   it active (their `long`-based typedefs — `size_t`, `ptrdiff_t`, `time_t` —
   stay 8-byte). Every later `#include` in the TU is then a guard no-op.
2. Its **last** line is `#define long int`, rewriting the bare `long` **keyword**
   tokens to `int` for exactly the code parsed afterward: `navigator.h` +
   `navigator.cpp`. The only bare-`long` tokens there are the six `.nav`-format
   sites plus the `GetLong` decl — all FS_Read/FS_Write'd fields — so each
   becomes the retail 4 bytes. (`main.cpp` `#undef long` immediately after the
   include, so its own code uses real 64-bit `long`.)

**Verified from the emitted bytes** (`build.sh` asserts it every run, aborts on
mismatch): the NAV id is 4 bytes, and byte offsets `[4..8]` hold the *checksum*,
not the zero high-half of an 8-byte `long`. Recomputed file size under the
4-byte formula equals the actual size exactly:

```
[diamond] navid=0x4A4E5635 (JNV5) word@4=222 (checksum) size=828 expect=828 -> OK (4-byte long)
```

`xxd fixtures/diamond.nav` — the id, checksum, numNodes, and first NODE id are
each 4 bytes and adjacent:

```
3556 4e4a  de00 0000  0400 0000  4544 4f4e
"5VNJ"     222(cksum)  4(nNodes)  "EDON"(NODE id)
```

Were `long` 8 bytes, `de000000` would be pushed to offset 8 and the NODE id
would land 4 bytes later — trivially visible. The Rust port pins these fields to
`i32`/`u32` (never `c_long`); the fixtures are retail-shaped so goldens, retail
pk3 `.nav` files, and the OpenJK referee all agree. A second independent witness:
each run reloads the just-written fixture through the oracle's own `Load` and
asserts the query surface is byte-identical (`Save/Load round-trip: OK`).

## Deterministic struct padding

`edge_t` is `{int ID; int cost; BYTE flags}` — 12 bytes with 3 trailing padding
bytes that `CNode::Save` writes raw. In the oracle those padding bytes are
uninitialised stack, which would make the fixtures non-reproducible. The build
pins them to zero with **`-ftrivial-auto-var-init=zero`** (a compile flag, not a
source edit), so the fixtures are byte-identical across runs and machines, and
the Rust port matches by zeroing its `repr(C)` padding. `failedEdges[32]` is
written from the file-scope `navigator` global (zero-initialised static
storage), so its 512-byte tail is deterministically zero.

Verified: `build.sh --regen` run twice produces byte-identical `.nav` + goldens.

## Heap tie-order ground truth (NAV-D2 / RULING 45)

`CalculatePath` floods each node's rank table via Raven's `CPriorityQueue`, which
is `std::push_heap`/`std::pop_heap` under `NodeTotalGreater` (a min-heap on
cost). `AddRank(…, curRank++)` records ranks in **pop order**, so the equal-cost
tie-break is baked into every rank table (see `diamond` node 0 → `ranks 0 1 2 3`;
`grid9` centre → `ranks 7 2 8 1 0 4 5 3 6`). Building with g++-16 makes the
committed ranks the **libstdc++** reference behavior automatically — the same
implementation the Rust port's hand-transcribed sift must match. Reference-only
cite (no GPL text in-repo): `/opt/homebrew/Cellar/gcc/16.1.0/include/c++/16/
bits/stl_heap.h`. Retail-MSVC tie-order divergence is accepted exactly as for
FP parity (RULING 26).

## Stub fidelity / harness stubs

Stub headers supply only what the TU references. Engine services are stubbed in
`main.cpp` and **do not touch the emitted bytes** — the generation path uses only
`FS_*` (backed by real files), `va`, `Cvar_Get` (`d_altRoutes`/`d_patched`
forced to 0, so the alt-route query surface is pure-graph), the vector math, and
a **clear `SV_Trace`** (fraction 1.0, so `HardConnect` cost == Euclidean distance
and edge flags == `EFLAG_NONE`, an open map). The trace/PVS/`gentity`/`VM_Call`
paths (`GetNearestNode`, failed-node/edge checks, callbacks) are compiled but not
executed — they belong to the 3c referee swap-in, not this golden surface.

## Deviations from the design doc

- **`GetBestNodeAltRoute` dumped with `d_altRoutes = 0` only.** With the cvar
  set, the alt-route path calls `RouteBlocked`, which needs live `SV_Trace` — a
  3c/referee dependency, not a pure-graph golden. At `d_altRoutes = 0` the doc's
  `!d_altRoutes->integer` short-circuit makes the query pure-graph, which is what
  is pinned here. The alt=1 branch verifies under the §3c referee.
- The trace/PVS-dependent query methods the doc lists as 3c (`GetNearestNode`,
  `GetEdgeCost`, `CheckFailedNodes`, `CheckFailedEdge`, `CheckBlockedEdges`,
  `GetBestPathBetweenEnts`) are intentionally **not** in these goldens; they need
  live engine+game state and verify under the referee (doc "Verification strategy
  → 3c").
