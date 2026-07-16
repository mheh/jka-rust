# Safe-state migration — stage map (SETTLED 2026-07-12; Stage 1 executing)

Decomposition of roadmap **Stage 1 — Safe-state migration**
(`docs/roadmap-final-stages.md`). All design decisions below were settled
interactively with the user on 2026-07-12 (§3); execution began the same day.
Nothing here changes seam layout (§D12), the oracle, or the referee's
comparison strength.

Progress lives in §4 (execution log). The referee (six scenarios incl. three
real-map runs over `mp/duel1`, per-frame playerState/entityState byte-diff +
syscall digest) gates every commit.

---

## 1. The problem shape

`mp_game` (183 files, ~203k LOC) carries ~2,723 unsafe-bearing lines smeared
through gameplay logic — the §D11 violation ("unsafe confined to the seam")
this migration exists to fix. Dominant pattern families:

| # | family | ~sites |
|---|---|---|
| F1 | world reached through raw ptr: `(*ctx.world).field` | ~5,464 |
| F2 | raw `*mut gentity_t`/`*mut gclient_t` threaded as params | ~1,593 params / 76 files |
| F3 | entity access through pointer deref `(*ent).field` (autoref trigger) | ~1,115 |
| F4 | arena index math `g_entities[idx]` | ~737 |
| F5 | `c_void` overlay casts + ICARUS `gSharedBuffer` reads | ~50 casts / ~20 files |
| F6 | CStr/char-array handling at non-seam boundaries | broad |
| F7 | function-scope `static mut` scratch (§20) | ~14 sites / 6 files |

**F1 is the linchpin and F2 blocks it**: entity pointer params alias *into*
`ctx.world.g_entities`, so `GameContext.world` cannot become `&mut GameWorld`
until entity traffic is by-index. That dependency forces the stage order.

The `dangerous_implicit_autorefs` crate-wide allow (`mp_game/src/lib.rs`) is a
downstream casualty of F1: once the world is reached through an accessor
returning a real `&mut`, there is no raw-deref place to autoref through.

## 2. Stage map

Every stage ends: `cargo build --workspace` green, **referee green (all six
scenarios)**, 346 workspace tests, fmt, i686 ILP32 lane. A stage that cannot
be independently referee-verified is wrong-sized and must be re-cut.

- **Stage 0 — Accessor seam** (infrastructure). `GameWorld::entity/entity_mut/
  client/client_mut`, `GameContext::world()/entity()/entity_mut()/entity_id_of()`
  (the ONE sanctioned transitional unsafe deref, deleted in Stage 2),
  `EntityId::from_num`/`to_num` seam helpers; `g_object.rs` converted end-to-end
  as the pilot.
- **Stage 1 — Entities by index (F2).** Every fn taking ctx converts its
  `*mut gentity_t`/`*mut gclient_t` params to `EntityId`/`Option<EntityId>`;
  ctx-free leaf helpers take `&mut gentity_t`/`&gentity_t` borrows. Sharded by
  subsystem, one referee-verified commit per shard. Returns stay raw (later
  pass); deref-saturated bodies may convert at the signature only, re-deriving
  raw pointers at the top with an in-code `// STAGE-1: … (Stage-2 debt)` mark.
- **Stage 2 — World-borrow flip (F1+F3; the headline).** Re-cut into 2a/2b
  (user ruling 2026-07-12, "do #2 in 2 hours"):
  - **2a — structural flip (fast, ~2h):** delete
    `#![allow(dangerous_implicit_autorefs)]` (deny it, fix the ~130 fire
    sites with explicit `&raw`/`addr_of!` forms, compiler-guided);
    `GameContext.world` flips to `&'a mut GameWorld` with
    `ctx: &mut GameContext` threading (implicit reborrow keeps most call
    sites unchanged; `f(ctx, ctx.m(x))` sites need compiler-guided
    arg-hoisting); the transitional `world()` accessor dies; `(*ctx.world).X`
    text stays valid (deref of `&mut`) and its now-unused `unsafe` strips
    mechanically. The ruling-21 `GameCallbacksImpl` raw reborrow dissolves.
    Key enablers: raw pointers don't hold borrows, so the Stage-1 re-derive
    bodies keep compiling unchanged.
  - **2b — body sweep (rolling):** dissolve the Stage-1 signature-only
    bodies file-by-file (multi-entity borrow restructuring per the pilot's
    re-acquire pattern) — the "unsafe retreats to the seam" bulk, landed as
    independent referee-gated commits after 2a.
- **Stage 3 — `static mut` scratch → world-owned scratch (F7)**,
  preserving buffer-rotation index semantics (va/vtos rings). The DEC-19
  qshared twin gets the mirrored engine-side home.
  EXECUTION AMENDMENT (2026-07-13): the `q_shared.c` statics (COM parse
  session/token, va + Info_ValueForKey rings) home in `BgState.qs`
  (`QSharedScratch`) — NOT a game-only struct — because bg's saber/vehicle
  loaders parse through them with `&mut BgState` and no `GameContext`;
  `BgState` lives inside `GameWorld`, so the world stays a complete value.
  Game-only scratch (GalakMech impact pos, w_saber faces, g_utils
  tv/vtos/shader-config rings) homes in `GameWorld.scratch`. va/tv have
  ZERO callers (idiomatic format! replaced them at every site) — their
  fns keep faithful bodies over the new home, no call-site impact.
- **Stage 4 — Overlay/shared-buffer casts behind typed seam adapters (F5, F6
  unsafe-bearing subset).** One typed adapter per `T_G_ICARUS_*` overlay.
- **Stage 5 — bg crate split** (ruling-19 deferral ends). NOT a dedup: mp_bg
  is types-only today; this introduces its first fn bodies. Needs its own
  short design session before execution (game-side-type dependency question).

## 3. Design decisions — SETTLED by the user, 2026-07-12

1. **`GameContext.world` final shape: `&mut ctx` threading.**
   `GameContext<'a> { world: &'a mut GameWorld, engine: &'a Engine }`, fns take
   `ctx: &mut GameContext`. Realized in Stage 2.
2. **Entity NULL: `Option<EntityId>` now** (over sentinel-first). Every
   Stage-1 shard carries the null-shape conversion; shards cut smaller.
   Stored entity fields on `#[repr(C)]` seam structs keep raw layout (§D12).
3. **Stage-1 depth: index at ctx boundaries.** Ctx-taking fns take ids;
   ctx-free leaf helpers may take `&mut gentity_t` borrows.
4. **Static-mut scratch home: `GameWorld.scratch` sub-struct** (Stage 3).
5. **bg-split is not a dedup** — mp_bg holds zero fn bodies today; Stage 5
   gets its own design session.
6. **`GameContext` `Copy`: dropped** (falls out of #1).

**Method-ization: REJECTED** (user ruling, supersedes the roadmap Stage-2
sketch on this point): gameplay logic stays free fns taking ctx permanently.
Accessors on GameWorld/GameContext remain methods; the ruling is about ported
Raven gameplay functions.

**Tail execution ruling (user, 2026-07-12):** after the hub shards, Stage-1
runs 2-wide — two worktree-isolated agents per wave with mutually-exclusive
file lists (cross-bridges into the partner's files are reported, not edited,
and applied at integration); agents run thin gates (check/fmt/tests + the
three fast referee scenarios); integration is serial and human-reviewed
("diff the merges"), with the FULL six-scenario referee + tests + i686 run
before each commit. One commit per shard keeps the chain referee-bisectable.

## 4. Execution log

- **Stage 0** — DONE, commit `19ed8ba2` (accessor seam + `g_object.rs` pilot).
- **Stage 1** — hub shards DONE serially, one commit each, every one
  referee-byte-identical across all six scenarios:
  S1 g_missile+g_target `f196f512` · S2 g_trigger+g_spawn `59a8e08b` ·
  S3 g_items `c50c0121` · S4 g_mover `50115025` · S5 g_active `611633cb` ·
  S6 g_combat (G_Damage hub, ~150 sites/33 files) `b7ec40f1` ·
  S7 g_weapon (80 fns) `3b065975` · S8 g_utils+g_main (~858 sites/51 files)
  `41f7c1e5` · S9 g_cmds `0a78e4e7` · S10 g_client `fb74d84e`.
  Tail DONE (2-wide waves, every shard referee-byte-identical):
  W1A w_force+w_saber `dc9070fd` ∥ W1B g_team+g_saga+g_log+g_timer
  `00759c2e`; W2A g_misc+g_ICARUScb `be5f00ac` ∥ W2B NPC_combat/utils/
  senses/goal/move/reactions/behavior `5f7b01bc`; W3A NPC_spawn+npc_c+
  stats+sounds `5810762f` ∥ W3B g_nav+g_navnew+ai_wpnav+ai_main
  `26e13763`; W4A NPC_AI_Jedi+Stormtrooper `0c9cc20b` ∥ W4B
  turrets+g_vehicles+FighterNPC `485b24f5`; W5 small NPC_AI_* +
  Stage-1-end cleanup `36786079` (stale STAGING headers swept, final
  param sweep: only seam/arena-base/double-pointer raws remain by
  design). **Stage 1 is complete.**
- **Stage 2a — DONE** (`aafdb713`): world borrow flipped (&mut GameWorld,
  non-Copy ctx, &mut self dispatch, ~2,000 sigs threaded), autoref lint
  DENIED (278 explicit forms), world() accessor deleted; deref bodies ride
  the `world_raw()` bridge (~4,680 uses, 60 files) pending 2b. Gates: 346
  tests + six referee scenarios byte-identical + i686.
- **Stage 2b wave cut** (by bridge-use weight, 2-wide worktree agents,
  mutually-exclusive files, referee-gated commit per shard):
  V1 w_saber ∥ g_weapon+g_mover · V2 g_combat+g_client ∥ g_saga+g_active ·
  V3 g_items+w_force+g_misc ∥ NPC_AI_GalakMech+NPC_AI_Stormtrooper+
  NPC_reactions+NPC_combat · V4 g_nav+g_trigger+g_ICARUScb+ai_main ∥
  g_main+NPC_utils+g_cmds+g_utils · V5–V6 the remaining ~40 small files,
  split by weight at cut time. Worker pattern: dissolve `world_raw()`/
  `__hN` temps into real borrows per the g_object.rs pilot re-acquire
  pattern; behavior/evaluation order preserved; referee arbitrates.
- **Stage 2b — DONE** (V1A `0d3d1065` … V6B, 12 shard commits, every one
  346-tests + six-referee-scenarios byte-identical): 4,680 raw world reaches
  dissolved to 27 marked irreducibles (bg-seam GameCallbacksImpl raw fields,
  &mut-world out-params aliasing ctx at their own calls, raw-ABI callees —
  each a Stage-2-API-shape question, not debt); all ~1,000 hoist temps
  renamed in place (RNG/syscall order preserved by construction); ~230 dead
  unsafe blocks retired. Unsafe now genuinely retreats to the seam + the
  entity/client deref regime (2c/gclient territory, out of scope).
- **Stage 4 scope (inventoried 2026-07-15).** The F5 estimate (~50 casts /
  ~20 files) covered only the ICARUS family. The dominant F5 sub-family is the
  tier-blocked `c_void` fields on `gentity_t` (`client`, `NPC`, `m_pVehicle` —
  `mp_qshared` sits below the tiers that own the real types): ~2,440 cast
  sites / ~70 files. Shard cut:
  - **4A** ICARUS `gSharedBuffer` → typed `SharedBuffer` adapter (17 overlays,
    `world/game_context.rs` + registration).
  - **4B** `g_timer.rs` void*-handle retirement (self-contained, 8 sites) +
    `g_svcmds.rs` `StringToFilter` param typing and IP byte↔`c_uint`
    reinterprets (`from_ne_bytes`).
  - **4C** allocator-result typing: `G_Alloc`/`BG_Alloc` call-site casts and
    the `.client`/`.NPC` pool-origin sites (`g_utils.rs:559`,
    `NPC_spawn.rs:1039`).
  - **4D** `client`/`NPC`/`m_pVehicle` field re-typing (~2,440 sites): ONE
    design decision (typed accessors vs. tier move), overlaps the gclient
    deref-regime ruling — **requires the task-#7 interactive sit-down**;
    held.
  - **4F** CStr/strcpy family: local-buffer half (cvar `.string`, spawnVars,
    userinfo, saga config — safe wrapper) executes now; entity-field half
    (classname/model/bone-name reads) aliases the deref regime and lands
    with 4D.
  - Excluded as seam-by-design: qsort comparator `void*` interface (ported
    qsort's generic contract; Rust sorts would change tie order), the
    `g_active.rs` `gentity_t`→`bgEntity_t` base-pointer overlay (D12 layout
    contract), `ghoul2` identity casts feeding G2API traps, botlib trap-arg
    packing in `ai_main.rs`.
- **Stage 4 — DONE except 4D (2026-07-15)**, four referee-gated commits, each
  six-scenario byte-identical + workspace tests + fmt; i686 lane green at
  stage end:
  - 4A `b39f16b7` — ICARUS `SharedBuffer` typed adapter (17 accessors,
    align(8) + per-payload const-asserts); all GameIcarus* arms safe.
    Adversarial gate-2 confirmed the copy-in/write-back out-param arms and
    that the reentrancy/strcpy-overlap hazards are oracle behaviors
    preserved verbatim.
  - 4B `bc20e8dc` — g_timer `gtimer_t*` signatures restored (void* was
    port-introduced; oracle g_timer.c:79/106/187); g_svcmds
    `from_ne_bytes`/`to_ne_bytes` reinterprets + `StringToFilter` param
    typing (raw ptr kept per the STAGE-2b aliasing constraint).
  - 4C `a26be7ca` — `gClPtrs` pool restored to Raven's `gclient_t*`
    elements (`gNPCPtrs` was already typed).
  - 4F `e3b6f3e7` — safe `cstr_from_chars` helper; 28 `CStr::from_ptr`
    sites over provably Rust-owned arrays converted; entity-field/param
    sites left for 4D.
  - **4D remains** (client/NPC/m_pVehicle re-typing, ~2,440 sites + the
    entity-field CStr half) — one design decision, folded into the task-#7
    seam-ratification sit-down.
- **Stage 3 — DONE**: zero `static mut` in mp_game; q_shared parse/format
  state in `BgState.qs` per the execution amendment, game-only scratch in
  `GameWorld.scratch`; ~224 call sites threaded across both tiers; botlib's
  va/FmtArg realigned to the DEC-19 qshared twin. Gates green.
- **Stage-1 precedents established** (binding on later shards): returns stay
  raw; signature-only re-derives for saturated bodies with in-code Stage-2
  debt marks; unused handler `other`/`activator` params are `Option<EntityId>`;
  deref-or-crash nullable params take Option + resolve() preserving Raven's
  exact crash surface; `Some(EntityId(i))` direct at index-built caller args
  (no `from_num` sentinel filtering); gclient-only callers bridge via
  `client.offset_from(level.clients)`.
- **Known debt:** unsafe counts in signature-only files rise transiently
  (+~50 `ent_id::resolve` wrappers crate-wide) — Stage 2's body sweep retires
  them together with the verbatim raw bodies.

## 5. Explicitly OUT of scope

- The ABI seam layout (§D12) — `#[repr(C)]`/`offset_of!`-asserted types never
  change; the engine↔module LocateGameData raw aliasing stays raw forever.
- The oracle (`oracle/**`) — never edited.
- Referee comparison strength — this plan consumes the referee as a gate.
- Engine-island unsafe (~1,848 lines) — governed by DEC-13…DEC-23.
- Roadmap Stage-2 API shape (EntityMut views, qboolean→bool, …) — follows
  safe-state; only the unsafe-retiring subset of §C7 rides along in Stage 4.
- SP mirror — inherits the pattern later as a diff.
