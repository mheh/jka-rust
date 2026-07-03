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

## Already covered — no decision (bless-the-rule appendix)

vec3_t out-params (§C7; VectorCopy ×1358), qboolean returns → bool ×652
(stored fields stay qboolean per §D12), va/Com_sprintf → owned String (×606),
goto ×94 → behavior-preserving rewrites (§C10; ai_main retry gotos may want
per-site care), naked-asm BoxOnPlaneSide → C fallback path (§19), 12-word
trap cap/gSharedBuffer (SEAM-D4), Com_Error→panic + EntityId + GameContext
(frozen Group A).
