# jampgame pre-port fork discovery (2026-07-03) — ALL RULINGS SETTLED

Read-only investigation of the three broadest call trees in
`oracle/oracle/codemp/game/` (G_RunFrame 887 in-module fns; ClientThink_real
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
escalations (clustered in `docs/handoffs/jampgame-escalations.json`). ~95% of
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
22. **Fork-4 execution shape (stated 2026-07-04):** the 38 stored fields
    become `Option<EntityId>` (Raven NULL → None; entity 0 is valid so no
    sentinel niche); conversion is compiler-driven (change field types, fix
    every resulting type error before porters launch); gentity_t/gclient_t
    private-tail layout is free (engine learns stride via
    trap_LocateGameData, only s/r/ps prefixes are fixed) — regenerate
    tail asserts.

## Already covered — no decision (bless-the-rule appendix)

vec3_t out-params (§C7; VectorCopy ×1358), qboolean returns → bool ×652
(stored fields stay qboolean per §D12), va/Com_sprintf → owned String (×606),
goto ×94 → behavior-preserving rewrites (§C10; ai_main retry gotos may want
per-site care), naked-asm BoxOnPlaneSide → C fallback path (§19), 12-word
trap cap/gSharedBuffer (SEAM-D4), Com_Error→panic + EntityId + GameContext
(frozen Group A).
