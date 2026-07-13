# Handoff — jka-rust live 30-bot server bug-hunt (2026-07-13)

**RESOLVED same day — do not pick up:** both open threads below closed,
pushed, and player-verified. Thread 1 (EV_SABER_HIT) was the ghoul2
identity-world-matrix debt (misc.rs gap note #6): collision positions came
back model-local, so effects spawned near the world origin — fixed by
threading the real matrix into `g2_trace_models` (`2312f204`). Thread 2
landed as a full row-identical regeneration of the three playerState tables
(`926cccb5`). A transient client "invalid entityState field count (141 vs
132)" during verification was stale-instance churn, not the new tables
(A/B-confirmed: old- and new-table servers both connect clean).

Fresh-agent pickup doc. Yesterday's session ran the first-ever bots-enabled live
boots of the Rust MP dedicated server and shook out a chain of engine/module
bugs the referee structurally cannot see. Two investigations are mid-flight.

## Where the repo stands

- Branch `master`, working tree clean. Safe-state Stage 1 is COMPLETE
  (waves W1A–W5 landed; plan §4 updated — `docs/plans/2026-07-12-safe-state-migration.md`).
  Next planned stage there: Stage 2a structural flip (user ruled "2 hours", see plan).
- **Seven bug-fix commits exist on master; PUSH IS HELD by user instruction**
  ("Don't push until this is resolved" — the open saber-hit issue). Only
  `2603f697` was pushed before the hold. Local-only: `ba1618fa`, `4138f7d0`,
  `32a1bd6c`, `fa2da4f3`, `855b73ef`, `d1130ae5`. Read their messages — they ARE
  the record of yesterday's fixes (LP64 word-width family, SV_Trace NULL guard,
  botlib BotClientCommand, powerup-log OOB, R_ModelMdxa GLM→GLA chain,
  ConvertedEntity widths, CM_ModelBounds real out-params).
- All seven are gate-verified: 346 workspace tests, six referee scenarios
  byte-identical, fmt clean.

## OPEN THREAD 1 — invisible EV_SABER_HIT effects (task #9)

Symptom: saber-hit blood/sparks render on OpenJK+C-module but NOT on our
server (damage itself lands). Bisect completed:

| engine | module | effects? |
|---|---|---|
| OpenJK | OpenJK C | yes |
| **ours** | **OpenJK C** | **yes** ← user-confirmed "saber hits registered" |
| ours | ours | no |

⇒ **our game module** emits differently. Ruled out: `entityStateFields` wire
table (verified identical to oracle, names+bits — see thread 2), engine
snapshot delivery (bisect), G_Dismember and the EV_SABER_HIT emission sites in
`w_saber.rs` (diffed verbatim vs `oracle/codemp/game/w_saber.c:4524/5996/6079/3642`).

**Next step, already started:** diff `G_TempEntity` (`crates/mp/game/src/g_utils.rs`)
against `oracle/codemp/game/g_utils.c:1054-1078`. Key oracle details: `s.eType =
ET_EVENTS + event`, `eventTime = level.time`, `freeAfterEvent = qtrue`,
SnapVector'd origin via `G_SetOrigin` (s.origin deliberately NOT set),
`trap_LinkEntity`. Check especially: eType arithmetic, eventTime/freeAfterEvent
handling in our event-free sweep (G_RunFrame's freeAfterEvent pass), and
whether `G_SetOrigin` sets `pos.trBase` correctly. If G_TempEntity is clean,
next suspects: the `eventParm` set at the personal-hit sites, and the
freeAfterEvent lifetime (freed before first snapshot ⇒ never transmitted —
compare our G_RunFrame event-sweep order vs oracle `g_main.c`).
Note the referee CANNOT arbitrate this: its scenarios never land a saber hit;
consider adding a scenario that does once fixed.

## OPEN THREAD 2 — playerState netfield tables missing rows (task #8)

`crates/mp/engine/qcommon/src/msg.rs`: `build_player_state_fields` has 137 rows
(oracle 152), pilot 140 (152), veh 69 (80) — the missing rows are vehicle
fields (`vehOrientation[…]`, etc.) dropped mid-table, so **every later field's
wire index is shifted** vs retail clients. Corruption fires whenever a
post-shift field changes (e.g. `legsFlip`). `build_entity_state_fields` is
verified identical (132 rows, names AND bits) — don't touch it.

Fix approach (validated): regenerate the three builders row-for-row from
`oracle/codemp/qcommon/msg.cpp` (`playerStateFields`/`pilotPlayerStateFields`/
`vehPlayerStateFields`; PSF rows). Style: existing `nf("name", offset_of!(...)
 + i * 4, bits)`; enum-indexed rows use e.g. `offset_of!(playerState_t,
fd.forcePowerLevel) + FP_LEVITATION as usize * 4` (see msg.rs:653-661);
`GENTITYNUM_BITS` stays symbolic. A generation script existed in the session
scratchpad (gone) — trivial to rewrite: extract `\{\s*PSF\(([^)]+)\)\s*,\s*
([^},]+?)\s*\}` rows in order, emit nf lines, handle `[N]` numeric and
`[FP_*]` enum indices. Verify with a name+bits re-diff after, then full gates.

## Live-hunt rig (how to reproduce/test)

- Rust server basepath: `~/Developer/jka/rust-server` (pk3 symlinks + our
  cdylib copied to `base/jampgamei386.so` — the engine's faithful load name).
  Launch: `./target/debug/mp_app +set dedicated 1 +set fs_basepath
  /Users/milohehmsoth/Developer/jka/rust-server +set sv_maxclients 32 +set
  bot_enable 1 +set g_gametype 0 +set rconpassword <pick-one> +map t3_hevil
  +set bot_minplayers 30` (port auto-picks 29071 while a `taystjk` app holds
  29070; or force `+set net_port`). Run under
  `lldb --batch -o run -o "thread backtrace" -o quit --` to trap crashes.
- Bisect basepath (our engine + OpenJK C module): `~/Developer/jka/oracle-server`
  (currently holds `jampgamearm64-STOCK-C.dylib` copied as `jampgamei386.so`).
- OpenJK reference: `~/Developer/jka/seam-test/openjkded.arm64 +set fs_basepath
  ~/Developer/jka/openjk-ref +set vm_game 0 +set sv_maxclients 16 …` (port 29074).
- The user plays and reports; announce server readiness audibly via `say` (their
  standing preference), and give them the rcon password for `entitylist`.
- Raven's ORACLE dylib (`tools/referee-oracle/build/liboraclejampgame.dylib`)
  CANNOT host NPC/ICARUS maps on 64-bit (its `vmMain` takes C `int` args —
  pointer-carrying nav callbacks truncate inside it). Use the OpenJK C module
  for live behavior comparisons, but remember it has OpenJK extensions.

## Settled findings (don't re-litigate)

- Missing MD3 props (`misc_model_breakable`, ammo racks, `misc_camera`,
  `NPC_Player`, `target_secret/autosave`) on SP maps = **retail MP behavior**
  (oracle module prints the same drops; OpenJK added `SP_misc_model_breakable`
  etc. as extensions — `nm` on their dylib proves it). Supporting them in ours
  is a DEC-ledger decision the user has NOT made. (Their OpenJK modelscale bug
  is not our problem.)
- Bot code has no executable stubs; botlib import leaves `Trace`/`EntityTrace`/
  `BSPModelMinsMaxsOrigin` as documented `None` deferrals (Q3-AAS-only, no live
  caller; `BSPModelMinsMaxsOrigin` is trivially wireable now).
- `g_dismember` defaults 0 both sides — the "gibs" in question are the
  EV_SABER_HIT effect, not limbs.
- Verification blind spots found live, worth internalizing: (a) referee
  scenarios have no bots/NPCs/sabers-connecting, (b) referee's mock engine
  never exercises network delta-encoding, (c) engine-island bugs shared by
  both A/B arms are invisible to the referee by construction.

## Open tasks (task list)

- #7 push the batch once the user confirms resolution (saber-hit fix is the gate)
- #8 playerState netfield tables (thread 2)
- #9 EV_SABER_HIT (thread 1)
- After both: run full gates, get user confirmation, push, then update the
  session memory notes.

## Suggested skills

- `/diagnose` — for thread 1 if the G_TempEntity diff comes up clean
  (disciplined reproduce → instrument loop; instrument our module's event
  spawn with a temporary eprintln and watch a live bot fight).
- `/code-review` — over the netfield-table regeneration diff before commit
  (mechanical but wire-critical).
- `/verify` — before pushing the batch: boot the server, saber-hit an NPC,
  see the effect (the whole point).
