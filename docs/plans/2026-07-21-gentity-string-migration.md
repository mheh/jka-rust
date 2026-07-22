# gentity_t string-field migration — target design

Status: RATIFIED 2026-07-21 (user). Rulings: A — prefix slots stay
`*mut c_char` (drop-in engine compat; engines read AND write the slots), with
the G6 side-table ownership refinement queued; B — `rofftarget`/`roffname`
deleted behind F_IGNORE keys; C — Option-vs-String split per the §3 evidence
table. Inputs: two surface investigations (session scratchpad
`gentity-design-inputs.md`); census at HEAD `c25390fb`.

## 1. The constraint the investigation surfaced

Raven's ABI prefix extends **past** `s`/`r`. `g_local.h`'s "DO NOT MODIFY
ANYTHING ABOVE THIS, THE SERVER EXPECTS THE FIELDS IN THAT ORDER" covers
`taskID`, `parms`, `behaviorSet[17]`, `script_targetname`, `delayScriptTime`,
`fullName`, `targetname`, `classname`, `waypoint`… — mirrored byte-for-byte as
`sharedEntity_t` (`g_public.h:679-715`), offset-asserted LP64+ILP32 in
`shared_entity_t.rs`. The engine **reads and writes** these fields through that
layout: `ConvertedEntity` (sv_game.rs:308-346) VM-translates exactly `parms`,
`behaviorSet`, `script_targetname`, `fullName`, `targetname`, `classname`;
engine-tier ICARUS reads `classname`/`targetname`/`script_targetname`/
`behaviorSet` and even writes `script_targetname = targetname`
(game_interface/mod.rs:282); `hook_install.rs:113-135` reads `classname` for
SV_ShowNetEntity.

The module must stay loadable under stock/OpenJK engines (GOAL.md drop-in;
proven live by the OJKeng-RUSTgame lane). Foreign engines walk this prefix in
our module's memory. **Therefore the five prefix string fields + behaviorSet +
parms keep `*mut c_char` layout permanently** (as long as drop-in module
compat is a goal). This includes the two highest-pressure fields (`classname`
179 sites, `targetname` 105).

## 2. Scope split

- **PREFIX (stays pointer, ABI):** `classname`, `targetname`, `fullName`,
  `script_targetname`, `behaviorSet[17]`, `parms`. Their writes keep
  `G_NewString`; their reads keep the (safe-wrapped) seam decode.
- **TAIL (migrates to owned):** the other 24 fields — headlined by `NPC_type`
  (176 sites), `target` (66), `message` (39), `model` (~30), `target2` (23),
  `team` (~20), `soundSet` (11), `model2`/`NPC_targetname`/`paintarget` (8)…
- **DELETE:** `rofftarget` (0 reads anywhere) and `roffname` (0 reads; only
  the resolved `roffid` int is consumed). Their spawn-table entries become
  `F_IGNORE` so map keys still parse silently; the ICARUS `roffname` store
  drops. (Ruling B.)

## 3. Per-field owned types (tail)

Decided by the NULL-vs-empty evidence (bare `.is_null()` readers = the field
distinguishes Raven NULL from ""):

- **`Option<String>`** (bare-null readers exist; `None` = Raven NULL):
  `target`, `target2`, `team`, `message`, `NPC_type`, `model`, `closetarget`,
  `opentarget`, `paintarget`.
- **`String`** ("" ≡ absent everywhere; every guard pairs `!= 0`):
  `model2`, `NPC_targetname`, `NPC_target`, `target3`, `target4`, `target5`,
  `target6`, `targetShaderName`, `targetShaderNewName`, `goaltarget`,
  `idealclass`, `healingclass`, `healingsound`, `ownername`, `soundSet`.
- ICARUS `"NULL"` sentinel writers (Q3_SetTarget on `target`): sentinel maps
  to `None`. Spawn-key `""` maps to `Some("")` — preserving today's
  distinguishable states exactly.

Alternative considered: uniform `Option<String>` for all 24 (maximum fidelity,
worse ergonomics). Rejected because the `String` rows above have zero
null-distinguishing readers repo-wide — the collapse is proven safe. (Ruling C.)

## 4. Function signatures

### 4a. Spawn pipeline (folds in 2h Option A)
```rust
// level_locals_t
pub spawnVars: Vec<(String, String)>,          // replaces [[*mut c_char;2];64] + 4096-byte pool
// g_spawn.rs
pub fn G_SpawnString(ctx, key: &str, default: &str) -> (qboolean, String)   // was out *mut *mut c_char
pub fn G_SpawnInt/Float/Vector(...)            // shapes unchanged, internals on the String form
fn G_AddSpawnVarToken — DELETED               // Vec push replaces pool bump
pub fn G_ParseSpawnVars(ctx, inSubBSP) -> qboolean   // internals: engine token -> Vec
```

### 4b. Spawn-field table (the F_LSTRING split)
`BG_field_t`/`FIELDS` stays for POD types (F_INT/F_FLOAT/F_VECTOR/F_ANGLEHACK/
F_PARM/F_IGNORE, offset-written as today). String entries split:
```rust
// bg_field.rs — new entry arm consumed by BG_ParseField
F_LSTRING          // prefix fields only (5 + 15 behaviorSet keys): offset write
                   //   of callbacks.new_string(...) — unchanged mechanism
F_STRING_OWNED(set: fn(&mut gentity_t, &str))   // tail fields: typed setter,
                   //   no offset, no alloc — setter does field = val.into() /
                   //   Some(val.into())
```
`BG_ParseField` keeps its signature; the F_STRING_OWNED arm calls the setter
with the decoded value. (bg tier holds no gentity knowledge beyond the fn
pointer type it already stores.)

### 4c. G_Find family (FOFS string walk retired)
```rust
// g_utils.rs
pub enum EntFindField { Targetname, Classname, Target, Target2, ScriptTargetname }
pub fn G_Find(ctx, from: Option<EntityId>, field: EntFindField, match_: &str) -> *mut gentity_t
```
Internal per-variant read: prefix variants (Targetname/Classname/
ScriptTargetname) decode the pointer field via the existing seam helper; tail
variants (Target/Target2) borrow the Option<String>. `fieldofs: c_int` param,
every `offset_of!` at the 70 call sites, and `FOFS_targetname` die.
G_PickTarget/G_UseTargets2/ICARUS Q3_* callers mechanical.

### 4d. Lifecycle
```rust
impl gentity_t {
    fn take_owned_strings(&mut self)   // drops all 24 owned fields (mem::take)
    fn seat_owned_strings(p: *mut Self) // ptr::write fresh defaults post-zero
}
```
- `G_FreeEntity`: `take_owned_strings()` → `write_bytes(ed,0,1)` (unchanged
  Raven zero) → `seat_owned_strings` → `classname = "freed"` (prefix,
  unchanged). Mirrors the landed ClientSpawn dance.
- Arena seating: `unsafe impl ZeroValid for gentity_t` removed; the two
  `zeroed_box` arena constructors get a `zeroed_entities()` twin of
  `zeroed_clients()` (alloc_zeroed + per-slot `seat_owned_strings`).
- NPC template-copy + trigger-clone pointer-aliasing sites (~18) become
  `.clone()` (worker B verified zero pointer-identity compares exist).
- `G_FindTeams` targetname transfer: prefix field, unchanged.

### 4e. Removals / retentions
- REMOVED: `G_AddSpawnVarToken`, `spawnVarChars` pool, `FOFS_*` constants,
  `fieldofs` offset walking, `ZeroValid for gentity_t`, `rofftarget`+`roffname`
  fields, ~150+ `cstr_to_str` sites (tail-field reads + locals holding them).
- RETAINED (prefix-only, reduced): `G_NewString` + the 256KB `G_Alloc` pool
  (consumers after migration: prefix strings, behaviorSet, ICARUS `parms_t`).
  `GameCallbacks::new_string` stays (prefix F_LSTRING arm).

## 5. Batching (each referee-gated; live boot at end)

- **G1** — spawn pipeline: spawnVars Vec, G_SpawnString reshape (49+ sites),
  F_STRING_OWNED table arm, tail-field struct flip for the LOW-pressure String
  rows (≤4 sites each: target5/6, shader pair, goaltarget, idealclass,
  ownername, healing pair, NPC_target) + lifecycle scaffolding
  (take/seat_owned_strings, zeroed_entities, G_FreeEntity dance) + rofftarget/
  roffname deletion.
- **G2** — `NPC_type` + `NPC_targetname` + NPC template-copy clone dance
  (NPC_spawn.rs is 80% of the writes).
- **G3** — `target`/`target2`/`team`/`message` + G_Find enum reshape (70 call
  sites) + G_UseTargets/PickTarget/ICARUS Q3_ callers.
- **G4** — `model`/`model2`/`soundSet`/`target3`/`target4` + remaining locals.
- **G5** — sweep: dead helpers, census re-run, live boot validation
  (mandatory: NPC spawn, siege, vote, train/door paths).
- **G6** (follow-on, separately gated) — prefix side-table ownership: the five
  prefix slots + behaviorSet keep their `*mut c_char` layout, but allocations
  move from the G_Alloc pool to a per-entity `CString` ledger on GameWorld;
  writes go setter → ledger stores the CString → slot gets `.as_ptr()`.
  READS STAY SLOT-BASED (engine ICARUS writes slots — e.g.
  `script_targetname = targetname` — so the slot is the truth; the ledger is
  an ownership record, never a read path). Deletes G_NewString + the string
  half of G_Alloc; the pool then serves only ICARUS `parms_t`. Discipline:
  setter keeps the old CString alive until the slot is rewritten; entity free
  clears slots before dropping ledger entries; 'static literal writes
  (c"noclass"/"freed") bypass the ledger.

## 6. Open rulings

A. Prefix stays pointer (drop-in engine compat) — the design's foundation.
B. `rofftarget`/`roffname` field deletion with F_IGNORE spawn keys.
C. The Option-vs-String split as tabled in §3 (vs uniform Option).
