# jampgame pre-port fork discovery (2026-07-03) — ALL RULINGS SETTLED

Read-only investigation of the three broadest call trees in
`oracle/codemp/game/` (G_RunFrame 887 in-module fns; ClientThink_real
722; ClientSpawn chain 721; combined distinct reach 986 of ~2,000-2,500 —
~40-50% of the module and effectively all gameplay-visible code). Seam stops
clean at ≤77 distinct trap_* — no surprise seam surface. Heavy bg_ crossing:
mp_bg is a hard co-dependency, not portable separately. GAME_ICARUS_* reads
gSharedBuffer typed structs (SEAM-D4's known high-arity escape).

## Fork classes (blast-radius order) — user rulings recorded inline

1. **~240 file-scope globals' placement** (~157 vmCvar_t handles in g_main.c
   + ~80 mutable globals: gDoSlowMoDuel, teamgame, the g_saga.c siege bloc,
   remappedShaders, gEscaping, itemRegistered, …). Master table only owns
   level/g_entities/g_clients. RECOMMENDED: all become GameWorld fields —
   cvar handles in one GameCvars sub-struct, the rest grouped by owning .c
   file. **RULING: GameWorld fields, cvars grouped in GameCvars (user, 2026-07-03)**
2. **Entity fn-pointer dispatch** (think 191 assigns/91 targets, use 62/53,
   touch 45/29, die 26/18, pain 18/16, reached 7/4, blocked 6/2 — ~213
   distinct targets; g_local.h:285-291). Compared by address (yes — e.g.
   think == multi_trigger_run), stored across frames (nextthink ×266),
   NEVER serialized (MP has no savegame/functable — grep-confirmed).
   RECOMMENDED: per-field fn-ID enums (EntThink/EntTouch/… of named targets),
   central match dispatch, PartialEq replaces address compares.
   **RULING: Fn-ID enums per field + central match dispatch; PartialEq replaces address compares (user, 2026-07-03)**
3. **RNG: one file-static LCG** (`holdrand = 0x89abcdef`, q_math.c:1432;
   holdrand*214013+2531011 >>17; 1,014 Q_flrand/Q_irand + 85 crandom/random
   sites — all parity-visible). RECOMMENDED: reproduce the LCG bit-exactly as
   an owned, threaded Rng (bg-shared since bg calls it); seed via Rand_Init
   dual; never rand crate. **RULING: Faithful LCG, owned + threaded, bg-shared; Rand_Init dual; no rand crate (user, 2026-07-03)**
4. **Multi-entity simultaneous &mut + pointer identity** (ClientThink_real
   writes veh->client->ps while reading ent, g_active.c:1971-1988;
   m_pPilot == (bgEntity_t*)ent address compares; touch/use take up to 3
   entity refs; 970 direct g_entities[] indexes; 28 (bgEntity_t*)/127
   (gentity_t*) casts). STATE-D1 covers single-borrow only. RECOMMENDED:
   handlers take (world, self_id, other_id, …); split-borrow helpers for
   disjoint slots (short unsafe seam helpers where unavoidable); stored
   bgEntity_t*/pilot/parent become EntityId; identity compares become
   EntityId ==. **RULING: Multi-EntityId params + id-equality; split-borrow helpers, audited unsafe seam helpers where unavoidable; stored pointers become EntityId (user, 2026-07-03)**
5. **71 function-scope statics**, three kinds. RECOMMENDED rule: const tables
   → const; rotating scratch return buffers (va/vtos idiom, g_utils.c:627/653)
   → owned return values; genuine cross-frame state (lastbotthink_time,
   bops_initialized, itemRegistered) → GameWorld fields (fork 1).
   **RULING: BLESSED as stated (user, 2026-07-03)**
6. **ICARUS callback registration** (~100 Q3_* callbacks into
   interface_export_t, g_ICARUScb.c; GAME_ICARUS_* vmMain cases). RECOMMENDED:
   defer icarus internals (§F subsystem doc), port Q3_* bodies against a
   forward-declared interface stub. **RULING: BLESSED as stated (user, 2026-07-03)**
7. **Vehicle vtable** (~25-member fn table on vehicleInfo_t,
   bg_vehicles.h:291-359, closed set: Fighter/Walker/Speeder/Animal).
   RECOMMENDED: enum-over-vehicle-type dispatch in mp_bg (porting-rules
   §8/§17 closed-hierarchy rule). **RULING: BLESSED as stated (user, 2026-07-03)**

## Slice-0 needsSession riders (findings 27-29)

27. LOAD-Q12: sv_init_game_progs provisionally in mp_engine_core, driven from
    main() until the map-spawn trigger. Bless / re-place. **RULING: BLESSED (user, 2026-07-03)**
28. Trampoline→slot arming: GAME_SLOT cell + arm_game_slot at load (the
    injected-slot amendment never specified the channel). Bless + doc line.
    **RULING: BLESSED + doc line rides next amendment pass (user, 2026-07-03)**
29. ServerGame concrete shape (engine-seam forward-decl): pin. Options: type
    alias to Server vs wrapper struct bundling what the syscall arms need.
    **RULING: Wrapper struct: pub struct ServerGame<'a> bundling the &mut borrows the syscall arms need; grows per arm (user, 2026-07-03)**

## Post-mega-pass rulings (2026-07-03, pass-1 escalation triage — accepted-for-now, user)

Pass 1 (wf_55d03832-2c3): 88 files, 717 fns ported, 1,875 parked, 619
escalations (clustered in `docs/handoffs/jampgame-escalations.json`, removed
2026-07-08; see git history). ~95% of
parks trace to the generated signatures not carrying already-settled rulings.

8. **Context threading in generated signatures** (~450 escalations:
   raw-ptr-skeleton-no-world-handle, ambient-state, ai-context, seam-threading,
   pmove-working-state, …). fnskel emitted faithful C signatures with no
   world/engine channel; B3/B4 forbid ambient reach, so every stateful fn
   parked. **RULING: every fn in the transitive stateful closure gains
   `ctx: GameContext` as first param, by value (the `g_init_game` shape;
   SEAM-Q12 Copy struct, world via STATE-D6 leaf reborrows, traps via
   `trap::X(ctx.engine, …)`). Needs-ctx set computed mechanically by tooling:
   a fn needs ctx iff it touches traps/globals/cvars/file-statics or calls a
   needs-ctx fn. Cross-file resolved signatures regenerate to match.
   (accepted-for-now, user, 2026-07-03)**
   - 8a. bg-tier fns cannot see `GameContext` (bg < game). **RULING: bg_pmove's
     file-static working set (pm, pml, pm_entSelf/pm_entVeh, pm_flying,
     gPMDoSlowFall, pm_cancelOutZoom) becomes a bg-owned `PmoveContext`
     constructed per Pmove call and threaded through; game-tier builds it; no
     game type leaks below game. Same pattern for other bg working sets.
     (accepted-for-now, user, 2026-07-03)**
   - 8b. AI/bot/saber file statics (botstates, floattime, w_saber damage
     arrays, IP filters, …) → GameWorld fields per fork 1, reached via ctx.
     **RULING: BLESSED (accepted-for-now, user, 2026-07-03)**
9. **vec3 out-param mechanical shape** (~32 escalations). §C7 was blessed but
   the generator emitted by-value `[f32;3]` (Copy — writes can't propagate).
   **RULING: NULL-able multi-outs (AngleVectors) → `Option<&mut [f32;3]>`;
   non-nullable outs → `&mut [f32;3]`; mutate+scalar-return (VectorNormalize)
   → `&mut` + scalar; single-out pure helpers → return value.
   (accepted-for-now, user, 2026-07-03)**
10. **Const/enum backfill wave before pass 2** (~30 escalations):
    animNumber_t (#[repr(i32)] enum per fidelity rule), BUTTON_*,
    WEAPON_CHARGING/_ALT, saberFace_t, SABERLOCK_*, ai_main.h const families
    (BWEAPONRANGE_*/TEAMPLAYSTATE_*/SIEGESTATE_*/CTFSTATE_*),
    MIN_LANDING_SLOPE, JUMP_VELOCITY, RAND_MAX (0x7fff beside the fork-3 LCG),
    Q_IsColorString as a q_shared helper.
    **RULING: BLESSED as listed (accepted-for-now, user, 2026-07-03)**
11. **Riders**: raw C fn-ptr params (localTrace) → typed Rust fn params;
    #ifdef-only debug fns (G_DebugBoxLines, DEBUG_SABER_BOX) → dropped with
    module-doc note per §20; runtime anim tables (bgAllAnims,
    bgHumanoidAnimations, WeaponReadyAnim/WeaponAttackAnim/…) → bg-owned state
    threaded per 8a. **RULING: BLESSED (accepted-for-now, user, 2026-07-03)**

## Pass-3 design session rulings (2026-07-04, user — supersede the
## accepted-for-now placeholders in 8a where they conflict)

12. **bg state = PmoveContext (per-call) + BgState (session).** PmoveContext
    built per Pmove call holds the pm/pml working set + &mut BgState; BgState
    owned by GameWorld (cgame later owns its own), holding bgAllAnims, saber
    parse buffers, vehicle info arrays, item list, bg pool.
    **RULING: BLESSED (user, 2026-07-04)**
13. **bg→engine = BgTraps trait** declared in the bg tier covering the full
    surface (trace, pointcontents, fs_*, g2api_* straps, fx_*, snap_vector);
    game implements over trap.rs; PmoveContext/BgState carry &dyn BgTraps.
    **RULING: BLESSED (user, 2026-07-04)**
14. **bg entity access stays the faithful baseEnt/entSize overlay**
    (unsafe seam helpers, PM_BGEntForNum shape); EntityId cleanup post-parity.
    **RULING: BLESSED (user, 2026-07-04)**
15. **RNG (fork-3 LCG) lives in BgState**; game reaches it via
    world.bg_state.rng. **RULING: BLESSED (user, 2026-07-04)**
16. **bg→game upcalls = GameCallbacks trait** (16 methods: damage, add_event,
    play_effect, alloc, …) declared in bg with bg-visible types; game
    implements; PmoveContext carries &mut dyn GameCallbacks.
    **RULING: BLESSED (user, 2026-07-04)**
17. **Fork-4 EntityId applies NOW (pass 3)**: the 38 stored gentity_t*
    struct fields become EntityId before porters run; porters write id
    compares natively. **RULING: BLESSED (user, 2026-07-04)**
18. **va/printf: mechanical format! mapping table**; Com_sprintf into
    char[N] → write bytes+NUL. **Diverge at g_target.c:800** (owned String;
    ≤2-line site note per §19; excluded from shared fixtures).
    **RULING: BLESSED (user, 2026-07-04)**
19. **bg crate split deferred to post-parity** — bg modules stay in mp_game
    for pass 3; tier discipline enforced by the trait boundary.
    **RULING: BLESSED (user, 2026-07-04)**
20. Pass-2 remainder (32 unfinished porters + never-run integration) is
    **folded into pass 3** — no pass-2 resume; files fill once against final
    signatures. (Recommendation accepted with this session.)
21. **Ruling-12 amendment (hand-slice finding, user 2026-07-04):** bg_state
    stays a GameWorld field; the bg_state/GameCallbacks aliasing conflict at
    Pmove call sites is resolved by **blessing the audited raw reborrow** —
    GameCallbacksImpl holds `*mut GameWorld` and reborrows per call (STATE-D8
    precedent, unsafe confined to the one named impl). Safe-borrow cleanup
    rides the post-parity EntityId migration. **RULING: BLESSED**
    Slice riders (mechanical, fold into prep C): `MAX_CLIENTS` needs a c_int
    dual (hundreds of `< MAX_CLIENTS` compares); pml_t + PmoveContext-adjacent
    types join the game prelude; RNG LCG is bit-exact for 32-bit unsigned
    long only — document + assert at the boot target; pmove_t's trace/
    pointcontents fn-ptr fields stay for layout but bg logic uses BgTraps
    (two-channel situation is intended).
22. **Fork-4 execution shape (stated 2026-07-04; count corrected at flip
    time: 37 fields — gNPC_t has 12 gentity_t* fields, not 13; gclient_t's
    formationGoal stays a pointer):** the stored fields
    become `Option<EntityId>` (Raven NULL → None; entity 0 is valid so no
    sentinel niche); conversion is compiler-driven (change field types, fix
    every resulting type error before porters launch); gentity_t/gclient_t
    private-tail layout is free (engine learns stride via
    trap_LocateGameData, only s/r/ps prefixes are fixed) — regenerate
    tail asserts.

## Landing-pass rulings (2026-07-04, user)

27. **`gentity_t.m_pVehicle`.** DISPOSITION (user, 2026-07-04): deferred to the
    abi-seam refactor — keep `*mut c_void` + ruling-14 overlay casts (same
    treatment as gclient client field); lands when the seam refactor
    restructures the tier boundary.
    (Note: this branch's ledger predates the pass-3 landing-pass block; entry
    added here to record the disposition where the residual data-table port
    commits.)

29. **`g_weapon.c` fire-time scratch vec3s.** RULING: BLESSED (user, 2026-07-05):
    the file-statics `forward`, `vright`, `up`, `muzzle` (`g_weapon.c:13-14`) —
    fire-time scratch `vec3_t`s shared across `WP_Fire*`/`CalcMuzzlePoint` —
    become `GameGlobals` fields per ruling 24's standard rule (file-scope
    mutable → `GameWorld` globals). `WP_Fire*` fns access them via
    `(*ctx.world).globals.<name>`; functions that declare *locals* of the same
    name keep the locals (only the unresolved bare-static references move).
30. **ctx-less `strap_*` bg-boundary wrappers.** RULING: BLESSED (user,
    2026-07-05): the `strap_G2API_*` family (`g_strap.rs`) are ctx-less
    bg-boundary wrappers — called from bg logic (`bg_pmove.rs`) without a
    `GameContext` — so they reach the engine through a seam-scoped
    `OnceCell`/`OnceLock<...>` engine-handle cell in the strap/seam module,
    initialized exactly once by the ABI entrypoint that owns the engine
    (`g_init_game`, GAME_INIT). §D11 seam confinement applies (this mirrors
    Raven's global syscall pointer). All ctx-taking code continues to use
    `ctx.engine`; the cell is ONLY for the ctx-less boundary fn-ptrs.

31. **Vehicle-dispatch ctx threading.** RULING: BLESSED (user, 2026-07-05):
    every `veh_dispatch` virtual (`Board`/`Eject`/`ValidateBoard`/`SetPilot`/
    `Ghost`/`UnGhost`/`Inhabited`/`Animate{Vehicle,Riders}`/`Process{Move,Orient}
    Commands`/…) and its per-class impls (`{Speeder,Walker,Animal,Fighter}NPC`,
    `g_vehicles`) take `ctx: GameContext<'_>` as their FIRST param, threaded from
    the `veh_dispatch` entry fns; the `GameCallbacks::board_vehicle` seam rebuilds
    `ctx` from its held `world`/`engine` (mirrors `try_grapple`). The removed
    Raven fn-ptr vtable slots make the dispatch our own construct, so the slot
    signatures were free to change. This cleared the parked bg-channel debt now
    that world/bg_state are reachable: `BG_AnimLength` calls in `AnimalNPC::
    AnimateVehicle`, `SpeederNPC::AnimateRiders`, and `g_vehicles::UpdateRider`
    (was a `0`/`100` placeholder); and the `.offset((owner-number))` pointer-
    arithmetic arena workarounds in `WalkerNPC`/`AnimalNPC` `ProcessOrientCommands`
    + `AnimalNPC::AnimateRiders` became direct `(*ctx.world).g_entities[...]`
    indexing.

**BG_MySaber arena-base param.** RULING: BLESSED (user, 2026-07-05): the bg
saber lookup takes an explicit arena base — `BG_MySaber(clientNum, saberNum,
ents: *mut gentity_t)` (`bg_saber.rs:437`) — because it reads `g_entities` under
`#ifdef QAGAME` and bg has no ambient world. Pmove-tier callers pass
`pm->baseEnt` (the overlay base, ruling-14 cast); game-tier callers pass
`g_entities`. Where the caller already holds a `PmoveContext` receiver, the
`PmoveContext::BG_MySaber` method form (`bg_saber.rs:2828`) is preferred over the
free fn (it reaches the base off `self.pm`). `BG_MySaber` null-checks `ents`, so
a null base degrades to a missing-saber lookup rather than a deref — the guard
`BG_SetAnimFinal` relies on when game-tier callers build a pm-null context.

**bg_fighterAltControl BgState cvar mirror.** RULING: BLESSED (user, 2026-07-05):
bg movement code that reads a game-tier cvar (`bg_pmove.c`'s
`bg_fighterAltControl`) reads a mirror field on `BgState`
(`bg.bg_fighterAltControl`), not the cvar directly (bg cannot reach the cvar
table). The game tier owns the `vmCvar_t` (`game_cvars.rs:238`) and writes the
mirror wherever it registers/refreshes the cvar (`g_main.rs:198,268`); bg reads
`bg.bg_fighterAltControl` (`bg_pmove.rs:7855`). This is the standard pattern for
any future bg-visible cvar: game-tier `vmCvar_t` + a `BgState` mirror updated at
register/update time.

## Referee-era rulings (post-integration)

**holdrand width = `c_ulong` (platform-faithful).** RULING (user, 2026-07-09):
`Rng::holdrand` is `core::ffi::c_ulong`, matching Raven's
`static unsigned long holdrand` (`oracle/codemp/game/q_math.c:1432`) at
whatever width the target compiles it — 32-bit on the retail i686 ship
(retail parity kept), 64-bit on LP64 referee/native builds (referee parity
gained). Reverses the 2026-07 u32 normalization (rng.rs assert + the
jampgame-oracle `unsigned int` rewrite): the referee A/B oracle proved the
64-bit stream is ground truth on LP64 — t2_wedge `Q_irand` NPC-type picks
diverged under the u32 model even though the low 32 bits of the stream
agreed. `tools/jampgame-oracle/run.sh` now extracts the LCG at native width;
the qmath golden's rng section is host-width by construction.

## Already covered — no decision (bless-the-rule appendix)

vec3_t out-params (§C7; VectorCopy ×1358), qboolean returns → bool ×652
(stored fields stay qboolean per §D12), va/Com_sprintf → owned String (×606),
goto ×94 → behavior-preserving rewrites (§C10; ai_main retry gotos may want
per-site care), naked-asm BoxOnPlaneSide → C fallback path (§19), 12-word
trap cap/gSharedBuffer (SEAM-D4), Com_Error→panic + EntityId + GameContext
(frozen Group A).
