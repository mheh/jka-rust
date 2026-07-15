# Goal

Build Rust implementations of Jedi Academy's modules and engine that speak
the same engine/module ABI as Raven's shipped binaries — drop-in
replacements, verified against the oracle.

## MP game module (`jampgame`): ABI target ACHIEVED, parity campaign closing

The original checklist (ABI vocabulary, `dllEntry`/`vmMain` contract,
`PASSFLOAT`, layout asserts, engine-named artifacts, smoke + golden differential
suites, per-file oracle audit) is complete — history in git (this file,
pre-2026-07-14) and `docs/audits/`. The three items that were open at the last
revision are now also done:

- **Shared memory / globals modeled**: `LocateGameData` seam live
  (engine island DEC-23; boots and hosts live players since 2026-07-12).
- **Hot-swap proven**: the module runs under OpenJK and our own `jampded`
  equivalent through init → map load → frame loop → client connect →
  shutdown (live bot + human sessions, 2026-07-07 → 2026-07-14).
- **Referee replay gate**: superseded by the stronger **lockstep referee**
  (`docs/plans/2026-07-13-engine-lockstep-referee.md`): Raven's compiled
  module and ours run side by side on live servers, comparing entity/player
  state + the syscall stream per frame, byte for byte. Soak record:
  11,985 frames / 0 divergences; a 23-minute live human session re-judged
  offline to zero module state divergences.

## Active campaign — referee close-out, then the unsafe finish

The centralized step list (task list mirrors this; keep both current).
Ordered; F-items float unordered.

1. **Kill the last known module divergence — item-toss velocity.**
   `ent115.pos.trDelta+8` at live-session frame 14282: a death-dropped item
   (`ET_ITEM`, `TR_GRAVITY`) spawned with a different toss-velocity z.
   Suspects: the `LaunchItem`/`TossClientItems` path (`oracle/codemp/game/
   g_items.c`, `g_combat.c` player_die drops) — RNG draws or velocity math;
   plausibly another instance of an F-item class bug. Method: probe inputs +
   `holdrand` both sides (see `tools/referee-oracle/build.sh` G6DBG blocks +
   in-tree Rust taps), soak or short play session via
   `tools/lockstep-referee/run.sh`, diff dump streams.
2. **Explain the replica-connect syscall blip.** Three live-session frames
   (737 / 5883 / 5964) diverged on syscall COUNT only, state digests equal —
   the follower's synthetic human-connect (`ref_inject_connect`,
   `crates/mp/engine/server/src/sv_referee.rs`) makes extra syscalls vs the
   real network connect path. Either align it call-for-call or teach the
   follower to expect it at injected connects.
3. **Probe formalization.** Keep the referee diagnostics permanently (user
   ruling 2026-07-14). Rename env `G6DBG` → `REF_PROBES`; tags become
   descriptive (`SAB_TRACE`/`SAB_CD`/`MUZZLE`/`LOOK_*`/`BONE_OVR`/
   `DEATH_ANIM`/…) — our tags and the oracle build.sh tags must stay
   byte-identical pairs. One `probe!` macro, env flag cached in a
   `LazyLock<bool>`. Commit the currently-uncommitted Rust-side taps
   (`w_saber.rs`, `bg_pmove.rs`, `g_combat.rs`, `g_weapon.rs`,
   `sv_game.rs`, `rng.rs` accessor); update the build.sh tag strings to
   match.
4. **G8 land.** Plan-doc statuses (lockstep plan G7/G8), refresh the stale
   README Status block (dated 2026-07-11), lift the push hold
   (29+ held commits).
5. **Safe-state Stage 4** — overlay/shared-buffer casts behind typed seam
   adapters (findings F5/F6) + the unsafe-retiring slice of porting-rules
   §C7. Referee-gated shards. Plan:
   `docs/plans/2026-07-12-safe-state-migration.md` (Stages 0–3 DONE).
6. **Safe-state Stage 5** — bg crate split (ends the ruling-19 deferral;
   mp_bg holds no fn bodies today — a split, not a dedup).
7. **Ratify the seam split, then dissolve the tail.** The design sketch is
   in `docs/roadmap-final-stages.md` Stage 2 (2026-07-14): `gentity_t`/
   `gclient_t` → `#[repr(C)] EntitySeam {s, r}`/`ps` registered arrays +
   parallel idiomatic arenas; module-chosen `LocateGameData` stride keeps
   2003/OpenJK drop-in compat. Interactive sit-down to ratify, then shards
   retire the 27 marked Stage-2b irreducibles and the entity/gclient deref
   regime ("2c territory").

Floating (unordered, run when convenient — each instance found prevents a
future referee interruption):

- **F1 — Class-bug sweep: unsuffixed C double literals flattened to f32.**
  Two confirmed kills: `f51f89e9` (BG_G2ClientNeckAngles 0.4/0.6/0.1),
  `435f7d57` (G_BounceMissile VectorScale 0.65). Method: grep oracle
  `game/*.c` for unsuffixed FP literals inside float expressions, map to
  Rust sites, fix as `(x as f64 * lit) as f32`. Reference idiom:
  `crates/mp/game/src/bg_misc.rs` trajectory code (already correct).
- **F2 — Class-bug sweep: dropped nullable-vec3 guards.** One batch ruling
  ("by-value vec3_t can never be null") erased real NULL semantics; four
  restored in `956101f7` (zero-vector sentinel). Method: grep
  `PORT-NOTE(point-null)` / `PORT-NOTE(dir/point-null)` and audit every
  other nullable-pointer-param PORT-NOTE crate-wide against the oracle's
  `if (!param)` sites.

Key context for all of the above: memory file `lockstep-referee-2026-07-14`
(hunt method, V-record decode offsets, import-number decode, stale-module
trap: build `-p jampgame`/`-p mp_app`, never `-p mp_game` alone).

## Related ABI track: SP `GetGameAPI` (future)

SP game uses a different transport: `GetGameAPI(game_import_t*) ->
game_export_t*` function tables, not `dllEntry`/`vmMain`. The tables are
ported as `#[repr(C)]` twins (`crates/sp/abi/src/game/public/`), the
`jagame` shell exports the symbol, and the remaining work (type foundation
for table fields, opaque-type policy for Raven C++ classes, function-pointer
conventions, field manifests, layout verification) is inventoried in git
history of this file and the marker inventory. SP inherits the MP patterns
as a diff once the MP campaign closes; keep the table ABI separate from the
`vmMain` enum transport.
