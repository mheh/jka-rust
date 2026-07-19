# Dedup campaign — one home per duplicated implementation (2026-07-18)

Status: RATIFIED (user sign-off 2026-07-18, four contested points settled — see
DEC-32). Cluster numbers below cite
`docs/audits/duplicate-inventory-2026-07-18.md` (the merged output of the
whole-workspace read sweep; per-cluster sites and evidence live there).

Principle: fix behavioral divergence first, then give each duplicated
implementation exactly one home at the lowest crate tier all its users can
reach (the `native_math`/`rng` precedent). Oracle-inherited duplication
(inventory category 5, ~30 clusters) is explicitly out of scope — faithful per
porting-rules §A2/§20.

## Settled decisions (DEC-32)

1. **`crates/native/string` is created** — home for the C-string runtime
   family: `c_str*`, `c_atoi`, ptr→String helpers, the
   `Com_Filter`/`Com_FilterPath`/`Com_StringContains` glob family
   (`native_platform` re-exports from it), `VALIDSTRING`, GP2 tokenizer.
2. **`c_atoi` uses strtol-style clamp semantics** (saturate to
   `i32::MAX`/`MIN`) — matches retail win32 msvcrt `atoi`; the icarus
   0-on-overflow and stringed/mock wrap-truncate tails converge on it.
3. **All 14 byte-identical SP/MP twin clusters hoist into `native`** —
   per-side re-exports, layout asserts retained at the re-export sites.
4. **The `QSharedScratch`-threaded string shape wins now** — `mp_game`'s
   threaded impls move into `mp_qshared`; the `static mut` `q_string`/
   `COM_Parse` copies are retired (closes the §B3 violation inside qshared).

## Phases (each commit: one logical cluster, `cargo build` + referee green;
pushes held to session end)

### Phase 0 — correctness fixes (inventory §1, ranked)
- 1.1 delete the 8 plain-`f32::sqrt` `VectorLength`/`VectorNormalize` locals
  (sv_world, g_client, ghoul2 ×5, cm_patch) → canonical f64-promoting q_math.
- 1.2 delete ghoul2 `misc.rs`/`ragdoll.rs` f32-trig locals → canonical
  `vectoangles`/`AngleVectors`/`AnglesToAxis`.
- 1.3 `be_aas_sample_fns.rs` imports its own `ON_EPSILON = 0.0` (bspq3 0.005
  import deleted).
- 1.4 `common_fns.rs` lossy-UTF8 locals deleted → byte-exact
  `mp_qshared::q_string`.
- 1.5 `BG_AnimLength` method delegates to the guarded free fn (§19 note).
- 1.6 `c_atoi` canonical lands in `native/string` with clamp semantics; the
  three sites become call sites.

### Phase 1 — `crates/native/string` (decision 1)
Create crate; migrate the family listed above; callers across
mp_game/mp_qshared/mp_bg/engine crates swap via the established re-export
style.

### Phase 2 — math call sites onto `native_math`
Clusters 14–22: cm_patch toolkit, botlib/aas/sv_world locals, bg_pmove's 18
inline dot-products, gore/misc `v3_*`, npcnav/ragdoll vec libs, g_utils
closures, `vector_compare`, inline `VectorMA`, `AngleDifference` inlines —
delete-and-import. `Create_Matrix`: canonical `pub(crate)` in ghoul2
`misc.rs`, f64-correct body, other three copies deleted.

### Phase 3 — ghoul2 crate-local `mdxa_layout` module
Clusters 23–27: mdxa offset helpers (7 files), `NUM_SURFACES_OFFSET`,
`read_i32/f32/u32` raw readers → one `pub(crate)` module;
`register_server_model` family canonical in `api_models.rs`; `ANIM_NAME`
reader/trim loops → file-local helpers.

### Phase 4 — mp_qshared string unification (decision 4)
Clusters 3, 4, 7, 8, 9, 10, 12, 13: `q_format.rs` sole formatter
(`c_format.rs` deleted, re-exported); ~25-fn overlap + `COM_Parse` dual
consolidated onto threaded shape; `COM_DefaultExtension`/`COM_StripExtension`
locals (stringed/icarus/roff) deleted; renderer `write_qpath`/`read_qpath`,
`qstricmp_eq`, `copy_cname` become call sites.

### Phase 5 — point fixes
Clusters 30–36, 39–48 per the ratified table: `ent_ptr` ×4 →
`ent_id::resolve`; `BG_GiveMeVectorFromMatrix` partials → bg_misc canonical;
`setType_t` hoists to mp_qshared; build.rs pair → new `crates/build-support`;
`MAX_ZPATH`/`MAX_OSPATH` single consts; saber consts canonical in mp_bg;
`MAX_SKINS`/`MAX_MOD_KNOWN` from tr_globals; `BG_GetTime` wins over invented
`level_time`; `GetFlagStr` shadow consts deleted; `sv_from_view` one
`pub(crate)` helper (inline seam casts stay — safe-state frozen);
game_globals wrapper macro; Huff delegation; IP-format helper; npcnav cursor
util; `MSG_Init` extract; `sys_error` lifecycle-canonical (view delegates if
graph allows); sv_referee uses view's `cvar_string`.
Leave faithful: `NotWithinRange` (37), nav-distance consts (38).

### Phase 6 — SP/MP twin hoists (decision 3; inventory §3)
`qboolean` double-def inside native fixed first (native_math depends on
native_types); then `animNumber_t`, `animEventType_t`/`footstepType_t`,
`vmCvar_t`, `symmetry_t`/`ERMDir`, MP3/sound vendor types
(`DECODE_FUNCTION` stays per-side), small cgame types,
`console_t`/`kbutton_t`/`field_t`, `modInfo_t`/`playerSpeciesInfo_t`,
`saying_t`, `FILE` stub, 7 identical uishared types, `byte`/`clipHandle_t`
redeclarations → imports; `CMiniHeap` → `native/containers`; GP2 tokenizer →
`native/string`. Leave: `mapVert_t`/`drawVert_t`.

### Phase 7 — tests + hygiene (inventory §4, §6.6)
Dev-only `crates/testkit` (`oracle_root`, `fixture_host`/`seed_dir`,
`compare`, `pi`/`pf` parsers, gp2 dump helpers); pmove/pmove_saber ~700-line
harness + `TestTraps`/`TestCallbacks` → `mp/game/tests/common/`; referee test
imports `sv_referee_fields`; `snap_vector` mocks call
`bg_misc::snap_vector`; rename the 18 `Cg*`-named structs in
`crates/mp/abi/src/ui/syscalls/` to `Ui*`.

## Out of scope
Inventory category 5 (oracle-inherited, ~30 clusters) and category 6
conventions (ABI one-file-per-token boilerplate, trap-shim pairs, vtable
dispatch, trait/mock triads) — no action.
