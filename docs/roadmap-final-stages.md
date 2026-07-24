# Final stages — the end state past parity (recorded 2026-07-04)

Settled in discussion (session: pass-2/pass-3 design). This is the ordered
roadmap AFTER the logic port compiles green. Nothing here starts before the
referee gates pass; every stage below is a behavior-preserving refactor run
behind a green oracle diff (rule A2), or new capability layered beside it.

## Stage R — Referee (gates everything below)

Record real gameplay as input logs (usercmd streams + level.time + RNG seed);
replay through oracle DLL and Rust DLL under the same engine harness; byte-diff
every playerState/entityState per frame. First divergent field at first
divergent frame = named, bisectable bug. Saber combat parity = the four-link
determinism chain: same inputs, same RNG stream (fork-3 LCG, call-site order),
same FP regime (no fast-math/FMA contraction; parity defined against oracle
built under OUR FP regime — 2003 x87 is unmatchable and not the target), same
iteration order. An hour of recorded dueling replaying byte-identical ≈ proof.

> Status (2026-07-08): the in-repo mock-engine referee is live
> (`crates/jampgame/tests/referee.rs`, landed 6c34b9d8+) and is a local gate on
> the dev platform; its CI job was removed by user ruling (b91ee8b6) due to
> cross-host gcc-vs-LLVM float divergence, not a regression. The external
> engine-vs-engine variant described above is parked
> (`docs/plans/2026-07-07-rust-referee.md`).

## Stage W — Wiring to runnable (partly pre-parity)

Remaining Dispatch impls (~25), cdylib + dllEntry/vmMain C exports, spawns[]
dispatch live, ICARUS boot no-op, i686 cross-target with per-arch layout
asserts → boot under stock jampded (drop-in `jampgamex86.dll`). 64-bit build
of the same source targets our engine later (registers/SSE2/addr-space wins;
32-bit stays for stock interop).

> Status (2026-07-08): essentially done except the i686/ILP32 layout-assert
> pass — CI's 32-bit lanes are allowed-failure pending it
> (`.github/workflows/build.yml`).

## Stage 1 — Safe-state migration (post-parity, mechanical, behind green diff)

- Complete fork-4: remaining raw `gentity_t*` traffic → EntityId/arena;
  `GameContext.world` raw ptr → `&mut GameWorld`; unsafe retreats to the seam
  (rules B5/B11). The ruling-21 raw reborrow in GameCallbacksImpl dissolves.
- bg crate split (ruling 19 deferral ends): bg modules move mp_game → mp_bg,
  tier discipline becomes crate-enforced; cgame reuses BgState/BgTraps.

> Status (2026-07-24): FROZEN by DEC-31 (2026-07-16). The mechanical safe-state
> migration ran through sub-stage 0 (accessor seam) and the Stage-1 hub shards
> (each commit referee-byte-identical across six scenarios), then the idiom era
> superseded it — the #13 string, DEC-32 dedup, DEC-34 qsort, DEC-35/#17 ghoul2,
> and #19 ctx-threading campaigns landed instead (all merged to master). The bg
> crate split IS done: `mp_bg` now holds the `bg_*` function bodies. The
> remaining raw-pointer/entity-view retirement is deferred to the post-full-port
> great refactor. Original plan authority:
> `docs/plans/2026-07-12-safe-state-migration.md`. NOTE: the method-ization
> sketch in Stage 2 below is superseded on one point — the user ruled gameplay
> fns STAY free fns ("keep the free function"); see the plan doc §3.

## Stage 2 — API shape: Raven's vocabulary, Rust's grammar

- `GameContext` grows into `Game<'a>` (world+engine bundle); free fns
  `foo(ctx, ...)` become `impl Game` methods. Callers: `game.damage(...)`.
- Scoped views over the arena (§17 pattern): `EntityMut`/`ClientMut`/`NpcMut`/
  `VehicleMut` — typed accessors replace runtime discriminator checks
  (`Option`-returning `client()`, `as_npc()`).
- **Blast-radius stratification**: leaf mutations (single-entity field writes:
  set_anim, toggles, add_event) live on the views; orchestrators that can fan
  out (damage, use_targets, dispatch chains) stay on `Game` taking EntityIds —
  the type system documents which functions can re-enter the world.
- **Name discoverability is a hard requirement**: every renamed/method-ized
  Raven fn carries `#[doc(alias = "G_Damage")]` (etc.) so 20 years of
  community muscle memory — grep, rustdoc search, IDE lookup — still lands on
  the right item. Raven names stay primary wherever reasonable; the alias
  covers prefix-to-receiver moves (G_FreeEntity → ent.free()).
- §C7 completion: qboolean fields → bool where non-ABI, out-params → returns,
  char[N] → String/&str at non-seam boundaries.
- **The seam split** (design sketch 2026-07-14 — ratify before Stage-2
  execution): `gentity_t`/`gclient_t` cease to exist as single types. The
  LocateGameData contract is module-chosen stride + engine reads only the
  `sharedEntity_t` prefix (`s`+`r`) per entity slot and `playerState_t` at
  client slot offset 0 — so only `EntitySeam { s, r }` and `ps` stay
  `#[repr(C)]` in the registered arrays; every private field moves to
  parallel idiomatic arenas (`Vec<Entity>`, `Vec<Client>`): pointers →
  `EntityId`/`Option`, char* → String/enum, ghoul2 stays an opaque handle.
  No wire marshaling exists or is needed — the seam array IS the live
  storage the engine snapshots in place, in both directions (module writes
  `s`, engine writes `r.absmin`/`s.number`/`ps.ping`). Slot recycling keeps
  Raven's `inuse` semantics — NO generational indices; deliberate
  stale-slot reads are oracle-verified behavior. Drop-in compat (retail
  2003 jampded ILP32, OpenJK x86-64) is preserved by construction; the
  referee's digests cover exactly the seam memory, which is exactly the
  memory that never changes shape.

## Stage 3 — The single-writer platform (mailbox + snapshots)

Frame thread stays sole owner of GameWorld (compiler-enforced). Two channels:
inbound Command mailbox drained at frame boundary ("as if an admin typed it"),
outbound snapshot/event stream (GameWorld is an owned value — clone is cheap,
serialization happens on the sidecar thread). This unlocks, in rough order:
- **Deterministic replay/time travel**: input logs + sparse world keyframes
  (KB/s, not MB/s) → instant replay, kill-cams (camera paths as ROFF tracks —
  authored in Blender or synthesized from kill geometry; §F ROFF subsystem is
  the keyframe engine), server rewind, replayable crash dumps, theater mode.
- **Headless simulation**: no renderer/clock dependency → thousands of
  frames/sec for balance testing, fuzzing, soak tests; the referee harness IS
  this machine's first customer.
- **Sidecar adapters**: Discord bot (events out, slash commands → mailbox) and
  MCP server (world:// resources read-only + typed tools = mailbox commands;
  tiered per-identity permissions; audit log free — every mutation is a
  serialized command). Mailbox is a whitelist: no variant, no capability.
- **External NPC brains / director mode**: snapshot eyes + fn-ID goal
  injection via mailbox.

## Stage 4 — Performance (only after parity locks "correct")

Budget reality: sv_fps 20 = 50ms frames; modern hw idles the oracle at <1ms —
logic-tier Rust-vs-C delta is noise; owned-String allocs are the one watched
regression class. The real wins, in leverage order:
1. Ghoul2 with SIMD (glam-style) — THE historical server hog (saber-trace bone
   math); §F reimplementation makes it redesignable. FP discipline: batch
   independent bone transforms across lanes (bit-exact, same per-lane op
   order); never reassociate within a dot product.
2. Algorithmic: spatial hash + classname index for G_Find/radius scans —
   results sorted to slot order = behavior-identical, referee-verified.
3. Parallelism C-with-globals could never buy: compute-parallel /
   commit-sequential over read-heavy phases (trace batches, NPC perception)
   once ownership is safe (Stage 1).
4. Free wins: LTO, target-cpu=native, PGO vs a 2003 MSVC baseline.
Payoff ceiling: surplus → higher sv_fps / tick rate = feel, not benchmarks.

## Stage 5 — The mod story (why the structure was worth it)

A server mod = ONE crate depending on mp_qshared/mp_bg/
mp_abi; flat src/ mirroring the .c files; navigation experience matches the C
tree a modder already knows. Near-term: fork the game crate (as C mods fork
game/). Long-term: hook points on GameWorld/Game so mods depend-and-override
instead of forking 88 files. Safety sales pitch (vs C mods): EntityId kills
stale-slot bugs, String kills the userinfo overflow class, panics replace
silent corruption, owned world = unit-testable accounts/economy without
booting a server, typed enums kill int-confusion — convention moves out of
heads into the type system, making the codebase safe for contributors and
agents alike.

## Sequence summary

pass 3 green → W (boot) → R (referee) → 1 (safe state) → 2 (API shape) →
3 (platform) → 4 (perf) → 5 (mod ecosystem). cgame and ui logic ports slot in
after R proves the pipeline on jampgame — same tooling, same workflow, their
trap surfaces swapped in. R before cgame/ui: catch systematic porter error
patterns once, cheaply, before stamping them into two more modules.

> Update (2026-07-24): pass-3, W, and R are done; the MP game module and MP
> dedicated-server engine host live play. Stage 1 is frozen (see above). The
> active track is the client port — `ui` first, then `cgame` + renderer, toward
> a full `jamp` client (`docs/plans/2026-07-24-client-port/`). Threading (a
> prerequisite framing for Stage 3) is permanently out of scope for this repo;
> it lives in the fork.
