# Newly-Live Code Review — 2026-07-08

Branch `skeleton` @ `de0bb6bd`. Scope: code activated by the recent un-stubbing
commits — GlobalUse dispatch (`95e14541`), index fns (`79be9e66`),
events/effects (`4f65a23e`), vehicle wiring (`3dbede1b`), batch-4 singles
(`09afce35`). This code had never executed against the oracle before; the
review compared the newly-reachable Rust against `oracle/oracle/codemp/game/`
prioritized by reachability in normal FFA/vehicle play.

Confidence key: **CONFIRMED** = both sources compared line-by-line.
**SUSPECT** = divergence is real but in-practice impact needs a deeper look.

## Summary table (worst-first)

| # | Severity | Confidence | Location | Finding |
|---|----------|-----------|----------|---------|
| 1 | Critical | CONFIRMED | `crates/mp/game/src/g_utils.rs:1771-1775, :1874` (TryUse) | Trace content-mask consts are all placeholder `0` — the use-key trace hits nothing, so the entire Siege heal / dispenser / use-a-target path is dead |
| 2 | High | CONFIRMED | `crates/mp/game/src/g_utils.rs:1761, :1787, :1953` (TryUse) | Local `GT_SIEGE = 4` (real value 7) and `g_gametype.value` instead of `.integer` — gametype gates fire in the wrong modes |
| 3 | High | CONFIRMED | `crates/mp/game/src/g_utils.rs:1765-1770` (TryUse) | Anim/flag consts are placeholder zeros (`BOTH_BUTTON_HOLD=0`, `SETANIM_TORSO=2` which is actually LEGS, `PMF_FOLLOW=0`, …) — wrong anim on wrong body part; spectator-follow guard never triggers |
| 4 | Medium | SUSPECT | `crates/mp/game/src/g_target.rs:159` (Use_Target_Delay) | Stores activator with non-null-safe `ent_id` — a NULL activator becomes a garbage `Some(EntityId)` that the delayed think later resolves |
| 5 | Low | SUSPECT | `crates/mp/game/src/NPC_reactions.rs:392-404` (NPC_ChoosePainAnimation) | Rust guards `pain_anim == -1` where C reads OOB (UB); Rust behavior is safer, exact-C irreproducible — divergence noted, do not "fix" |

Findings: **3 CONFIRMED bugs** (all in the still-stubbed `TryUse`),
**2 SUSPECT divergences**. Everything else reviewed came back clean.

---

## Findings in detail

### 1. TryUse trace content-mask is 0 — use-key heal/dispenser/objective path dead (CONFIRMED, Critical)

- **Rust:** `crates/mp/game/src/g_utils.rs:1771-1775` defines local placeholder
  consts `MASK_OPAQUE = CONTENTS_SOLID = CONTENTS_BODY = CONTENTS_ITEM =
  CONTENTS_CORPSE = 0`, used in the forward trace at `g_utils.rs:1874`.
- **C:** `oracle/oracle/codemp/game/g_utils.c:1694` traces with
  `MASK_OPAQUE|CONTENTS_SOLID|CONTENTS_BODY|CONTENTS_ITEM|CONTENTS_CORPSE`
  (all nonzero: `CONTENTS_SOLID=0x1`, `CONTENTS_BODY=0x100`,
  `CONTENTS_ITEM=0x100000`).
- **Consequence:** contentmask `0` collides with nothing, so
  `trace.fraction == 1.0` always and the `goto_tryJetPack` branch
  (`g_utils.rs:1878-1881`) is taken unconditionally. `TryHeal`
  (`g_utils.rs:1986`), `G_UseDispenserOn` (`:1927/:1930`), and the
  `ValidUseTarget`/`GlobalUse` branch (`:1952-1982`) are never reached via the
  use key. Healing a teammate, servicing teammates with health/ammo
  dispensers, and pressing Use on use-targets through `TryUse` are silently
  disabled. Commit `09afce35`'s `TryHeal`/dispenser fixes are unreachable
  through this path (only `emplaced_gun_use` → `TryHeal` at
  `g_weapon.rs:5706/5720` and the ground-toss dispenser path at
  `g_utils.rs:2011-2046` still work).

### 2. TryUse GT_SIEGE = 4 and `.value` instead of `.integer` (CONFIRMED, High)

- **Rust:** `crates/mp/game/src/g_utils.rs:1761` — `const GT_SIEGE: c_int = 4;`
  used at `:1787` and `:1953` via `g_gametype.value as c_int`.
- **C:** `bg_public.h:184-197` — `GT_SIEGE == 7`; C reads
  `g_gametype.integer` (`g_utils.c:1625, 1737`).
- **Consequence:** in real Siege (gametype 7) the "round not begun" gate at
  `g_utils.rs:1786-1789` never fires (players can Use before round start) and
  the friendly-objective FF gate at `:1953-1956` short-circuits; in
  POWERDUEL (gametype 4) the round gate wrongly fires. Commit `09afce35`
  fixed exactly these two bugs inside `TryHeal` but left them in its caller.
  Currently latent behind finding #1.

### 3. TryUse anim/flag consts are placeholder zeros (CONFIRMED, High)

- **Rust:** `crates/mp/game/src/g_utils.rs:1765-1770` —
  `BOTH_BUTTON_HOLD=0`, `BOTH_CONSOLE1=0`, `SETANIM_TORSO=2`, `PMF_FOLLOW=0`,
  `HANDEXTEND_DRAGGING=0`.
- **C:** `SETANIM_TORSO=1` (`bg_public.h:498` — 2 is `SETANIM_LEGS`),
  `PMF_FOLLOW=4096` (`bg_public.h:415`), `BOTH_BUTTON_HOLD`/`BOTH_CONSOLE1`
  and `HANDEXTEND_DRAGGING` are nonzero enums.
- **Consequence:** even with finding #1 fixed, the Use animation would be set
  to anim index 0 on the legs instead of `BOTH_BUTTON_HOLD` on the torso, the
  torsoTimer-extend comparisons check against anim 0, and the `PMF_FOLLOW`
  spectator guard at `g_utils.rs:1799` never triggers. Directly undoes in the
  caller the `SETANIM_TORSO 2→1` correction `09afce35` made inside `TryHeal`.

### 4. Use_Target_Delay stores activator with non-null-safe `ent_id` (SUSPECT, Medium)

- **Rust:** `crates/mp/game/src/g_target.rs:159` —
  `(*ent).activator = Some(ent_id(base, activator))` — the non-null-aware
  helper. A NULL `activator` produces a wild `offset_from` delta cast to
  `u32`, stored as `Some(garbage)`. `Think_Target_Delay`
  (`g_target.rs:136`) later `resolve()`s it into a bogus entity slot.
- **C:** `oracle/oracle/codemp/game/g_target.c:90` — stores the raw pointer;
  NULL stays NULL and `G_UseTargets(ent, NULL)` is tolerated.
- **Consequence:** a `target_delay` fired with NULL activator (think-driven or
  script-fired chains) targets a garbage entity on the delayed think — wrong
  entity fired or crash. Fine with a normal player activator, so invisible in
  casual FFA. Every other reviewed handler uses the null-safe `ent_id_opt`
  (e.g. `Use_BinaryMover` `g_mover.rs:1142-1143`, `target_counter_use`
  `g_target.rs:892`); this is the lone outlier. SUSPECT only because a NULL
  activator reaching this path in shipped maps is unverified.

### 5. NPC_ChoosePainAnimation guards pain_anim == -1 where C has UB (SUSPECT, Low)

- **Rust:** `crates/mp/game/src/NPC_reactions.rs:392-404` — computes anim
  length only `if pain_anim >= 0`, else 0 (no debounce).
- **C:** `oracle/oracle/codemp/game/NPC_reactions.c:351` — indexes
  `anims[pain_anim]` unconditionally; the knockdown/roll/flip/spin and
  force-grip branches can leave `pain_anim == -1` → out-of-bounds read →
  garbage, build-dependent `painDebounceTime`.
- **Consequence:** when an NPC takes pain while knocked down/rolling/gripped,
  Rust applies no pain debounce where C applies an unpredictable one. Rust is
  the safer behavior and the exact C behavior is irreproducible UB — noted
  for the record, not a fix candidate.

---

## Clusters that came back CLEAN

### Cluster 1 — Use-handler dispatch targets (11/12 clean, CONFIRMED)

Dispatch plumbing is faithful end to end: `dispatch_use`
(`ent_fn_enums.rs:280-350`) passes `(self_, other, activator)` in-order to
every handler; `GlobalUse` (`g_utils.rs:664`) mirrors C `g_utils.c:552-564`
including the FL_INACTIVE and null-use guards; `G_UseTargets2`
(`g_utils.rs:689`) calls `GlobalUse(t, ent, activator)` exactly as
`g_utils.c:589`; the `t_use` cheat path (`g_cmds.rs:4881`) passes
`(targ, ent, ent)` as C does. No systemic self/other/activator swap exists.

Verified line-for-line faithful: `Use_BinaryMover` (g_mover.rs:1107),
`Use_BinaryMover_Go` (:929, all four moverState branches incl. nonlinear-stop
math), `func_usable_use` (:3467), `Use_Multi` (g_trigger.rs:486),
`Use_target_push` (:1278), `Use_Target_Print` (g_target.rs:220),
`Use_Target_Speaker` (:352 — incl. the spawnflag-8 sound-on-*activator*
subtlety), `target_relay_use` (:685), `target_counter_use` (:861),
`target_activate_use` (:1179), `target_deactivate_use` (:1194). Only
divergence: finding #4.

### Cluster 2 — G_PlayEffect / G_PlayEffectID call sites (clean, CONFIRMED)

Core impls (`g_utils.rs:1348` / `:1369`) are faithful to
`g_utils.c:1250-1284` — param order, `s.angles`/`s.origin`/`s.eventParm`
assignment, and the `G_PlayEffectID` zero-angles → `angles[1] = 1` default
all match.

24 call sites compared, all CONFIRMED clean — no fxID/origin/angles
transpositions, no muzzle-vs-origin swaps:

- `g_weapon.rs`: 8/8 clean, including `g_weapon.rs:3989` (stun-baton hit)
  which correctly passes `tr.endpos` + plane normal, not the muzzle point.
- `g_combat.rs`: 16/16 clean, including the DeathFX chain
  (`g_combat.rs:1474-1646` vs `g_combat.c:1899-1976`) — `g_combat.rs:1512`
  correctly *preserves* C's one site that uses `currentOrigin` instead of
  `effectPos`; and the R2/R5 droid head-throw effects
  (`g_combat.rs:2112/:2130`) with the `up` vector from `AngleVectors`.
- `g_misc.rs`: zero call sites in both languages (nothing to check).
- Two C call sites verified as dead code and correctly not ported:
  `g_weapon.c:3221/:3227` (disabled preprocessor block) and `g_combat.c:387`
  (block comment).

### Cluster 3 — Vehicle board/eject (clean, CONFIRMED)

- Dispatch shims (`veh_dispatch.rs`): `board`/`eject` override routing matches
  the C function-pointer setup exactly — Fighter overrides Board+Eject
  (FighterNPC.c:1958-1959), Walker overrides Board only (WalkerNPC.c:557-558,
  Eject commented out in C), Speeder/Animal generic. Arg order and qboolean
  sense correct in `board`, `eject`, `eject_all`, `validate_board`,
  `set_pilot`, `ghost`, `un_ghost`.
- Implementations (`g_vehicles.rs`): `Board` (:411 vs g_vehicles.c:630-872),
  `VEH_TryEject` ejectDir switch (:595-619 vs C:890-921 — all six directions
  use the correct AngleVectors slot and sign), `Eject` (:2438 vs C:1019-1374,
  incl. the `goto getItOutOfMe` modeling, passenger-to-pilot promotion, slot
  compaction), `EjectAll` (:722 vs C:1377-1448), `ValidateBoard`, `SetPilot`,
  `Ghost`/`UnGhost`, and the Update boarding-maintenance/death-kick paths
  (:1078-1225 vs C:2016-2103) are all faithful. No stubs, no inverted flags,
  no dropped state updates.
- Two faithful-to-C quirks (do **not** fix): the passenger death-kick
  double-decrements `m_iNumPassengers` exactly as C does
  (g_vehicles.rs:2210-2217 + :2604 vs g_vehicles.c:2094-2095 + :1233); and
  `VEH_TryEject` zero-inits `vVehLeaveDir` where C reads uninitialized stack
  for the BOTTOM/default case — harmless.

### Cluster 4 — Batch-4 singles bodies (clean; problems are in the caller)

The three functions `09afce35` rewrote are byte-faithful:
`TryHeal` body (`g_utils.rs:1657-1755` vs `g_utils.c:1546-1602`),
`G_UseDispenserOn` (`g_utils.rs:1555-1601` vs `g_utils.c:1474-1505`),
`G_CanUseDispOn` (`g_utils.rs:1606-1652` vs `g_utils.c:1508-1544`),
`NPC_Pain` (`NPC_reactions.rs:417-599` vs `NPC_reactions.c:363-529`), and the
anim-length lookup chain in `NPC_ChoosePainAnimation`
(`NPC_reactions.rs:398-406` vs `NPC_reactions.c:351-353` — correct table
split between skeleton-specific `numFrames` and humanoid `frameLerp`, no
ms-vs-frames confusion). The infidelity lives entirely in the unchanged
caller `TryUse` (findings #1-#3).

### Cluster 5 — G_AddEvent core + call sites (clean, CONFIRMED)

Core impl (`g_utils.rs:1308-1343` vs `g_utils.c:1221-1243`): zero-event
guard, `eventTime = level.time` for both branches, and the MP
`EV_EVENT_BITS`/`EV_EVENT_BIT1` top-bit rotation replicated exactly for both
the `ps.externalEvent` (client) and `s.event` (non-client) branches. (Note:
MP does not use the SP-style `eventSequence << 8` packing; the Rust matches
the MP oracle.)

Twelve call sites sampled across `g_missile.rs` (:292, :536-545, :595-604,
:1015-1024), `g_weapon.rs` (:2292, :5090, :5165, :5202, :5614), `g_combat.rs`
(:533, :656, :2701-2703), `g_active.rs` (:196, :2641-2718 private-duel
block), `g_client.rs` (:568, :1212) — all arg-for-arg faithful, incl.
`DirToByte` eventParms and correct ent-vs-other targeting
(`EV_BECOME_JEDIMASTER` on `other`, `EV_PAIN` on `player`). The apparent
EV_USE_ITEM0 count gap (13 vs 15) is C dead code inside a block comment
(`g_active.c:3346/3350`); Rust correctly omits it. No findings.

---

## Method note

Five parallel line-by-line comparison passes against
`oracle/oracle/codemp/game/`, one per cluster, plus an independent
cross-check of the dispatch chain (`GlobalUse` → `dispatch_use`,
`G_UseTargets`/`G_UseTargets2`, the `t_use` cheat path) and the
`G_PlayEffect`/`G_PlayEffectID` core impls. No files other than this audit
were touched.
