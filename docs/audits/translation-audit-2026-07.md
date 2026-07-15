# Translation-bug audit — mp_game file-by-file (task #10 / GOAL item 5)

Working ledger of the file-by-file oracle comparison. Findings accumulate
here per wave; fixes land in referee-gated batches and graduate to
`referee-catches-2026-07.md` with their commit hashes. Lens list and method:
GOAL item 5.

Status: wave 1 of ~27 (live-in-FFA files first).

## Wave 1 (2026-07-15) — g_utils, g_items (audited clean-ish), g_active, g_missile (pending report)

### g_utils.rs + g_items.rs — 6 findings, none live-combat

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `g_utils.rs:440` G_Throw kvel[2] | `g_utils.c:333` | C folds `newDir[2]` into the f32 product chain, then `*1.5` in double; port restructured the association (newDir last, in f64) — different intermediate rounding on the Z knockback. Fix: match C's association exactly. | latent, live branch (~1-ULP) |
| 2 | `g_utils.rs:719` G_UseTargets2 | `g_utils.c:574` | `level.time * 0.001` is double in C; port is all-f32. Shader-remap timeOffset, masked by `%5.2f`. | rare-branch |
| 3 | `g_items.rs:202,205` adjustRespawnTime | `g_items.c:74,78` | `20.0 /` and `8.0 /` are double divides in C; port all-f32. Behind `g_adaptRespawn` (default 0), masked by int truncation. | rare-branch |
| 4 | `g_utils.rs:2578` ShortestLineSegBewteen2LineSegs | `g_public.h:9` | `Q3_INFINITE` is **16777216**, port used `f32::INFINITY`. Masked (map distances ≪ 2^24) but wrong constant. | latent (masked) |
| 5 | `g_items.rs:1004,1089,3358` | `g_items.c:628,690,3139` | C truncates the whole `level.time + K + random()*X` sum as one float; port truncates only the random part. Diverges once `level.time` exceeds f32 precision (~4.6 h uptime). | latent (very rare) |
| 6 | `g_utils.rs:2085` TryUse | `g_utils.c:1829-1836` | `Touch_Button` dispatch arm dropped (marked PORT-NOTE): USE on a touch-activated func_button does nothing. | rare-branch (documented) |

Re-verified clean under scrutiny: both crandom sites (d6ef7674 holds); pas_think
sin (oracle literals are float-suffixed); G_Throw kvel[0]/[1] macro expansion;
LaunchItem FL_BOUNCE_HALF clobber (faithful Raven bug); G_KillBox zero-vector
NULL convention (residual world-origin edge lives in g_combat, astronomically
rare). Full clean-lists in the wave-1 agent reports (session task outputs).

### g_active.rs + g_missile.rs — CLEAN (0 behavioral divergences)

Systematic top-to-bottom walk; every float-flattening candidate numerically
swept and cleared (P_DamageFeedback pitch-byte math, DoImpact `>= 0.2`
inclusive compare, fall-damage `delta*0.16` — all byte-identical to f64;
contrast the strict-`>` bounce compare that WAS a real bug, 435f7d57).
Eval-order, loop-increment, switch-completeness, NULL-sentinel, and
dead-quirk preservation all verified. Two §19 UB-hardening guards diverge
from Raven only where Raven would crash, both **unmarked** — add the ≤2-line
site notes per porting-rules §19:

- `g_missile.rs:353-359` G_ExplodeMissile: null-client guard on
  `accuracy_hits++` (oracle g_missile.c:245-252 derefs unconditionally).
- `g_missile.rs:984-987` G_MissileImpact: `think` null guard (oracle
  g_missile.c:677 calls unconditionally).

Wave-1 fix batch landed: `b0bee6e4` (findings 1-5 fixed + the two §19
markers; #6 TryUse stays documented-parked).

## Wave 2 (2026-07-15) — bg_pmove, w_saber, g_combat+g_client, bg_saber+g_cmds

### bg_pmove.rs — 5 findings (all f32-vs-double threshold class), core path CLEAN

Full-coverage walk (lines 127-10377, no gaps); prior fixes (lastHeadAngles,
neck-angle f64s) re-verified intact. Every finding is a libm-double or
bare-double-literal C expression evaluated in f32 (Q_fabs-for-fabs accounts
for 3 of 5); none in the non-vehicle live-combat hot path:

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `bg_pmove.rs:384-385` PM_SetVehicleAngles | `bg_pmove.c:526` | bare-double `0.5` keeps the pitch adds in f64; sibling branch's `0.5f` asymmetry is real in the oracle | latent (vehicle-in-water) |
| 2 | `bg_pmove.rs:415` PM_SetVehicleAngles | `bg_pmove.c:563` | `speed *= sin(...)` product is double in C; port narrows sin first | latent (vehicle banking) |
| 3 | `bg_pmove.rs:549-550,640-642` PM_HoverTrace | `bg_pmove.c:757,835` | `fabs+fabs > 100` sums in double; f32 sum can flip at the boundary — gated arm calls Q_irand → **RNG-stream desync** | latent, PRIORITY (RNG-coupled) |
| 4 | `bg_pmove.rs:2790` PM_CheckJump | `bg_pmove.c:1988` | `fabs(dotR) > fabs(dotF)*1.5` double product vs f32 — flip-anim axis tie-break | latent (anim, crosses wire) |
| 5 | `bg_pmove.rs:4225` PM_CrashLandEffect | `bg_pmove.c:3691` | `fabs(v)/10` double divide vs f32 at the `>= 30.0` dust-effect gate | latent/cosmetic |
| — | `bg_pmove.rs:9907-9955` PM_VehicleViewAngles | `bg_pmove.c:9646-9709` | oracle reads clampMin/Max uninitialized (UB); zero-init pick needs its §19 site note | doc-only |

### g_combat.rs + g_client.rs — CLEAN (damage/death spine and client lifecycle byte-faithful)

No live-combat or rare-branch bugs. Prior fixes (four point-null guards)
re-verified. One systemic observation promoted to a sweep item (below);
one stale PORT-NOTE (G_DamageFromKiller claims a zero-vector dir, code
correctly passes None) — doc-drift fix rides the next batch. g_client:
zero findings across ~45 fns (field-copy completeness, IPstring
truncation, spawn-sort loops, all NULL-guard polarities verified).

### w_saber.rs — 1 LIVE-COMBAT bug (fixed same-day) + 2 latents; 82 fns otherwise faithful

| # | Site | Oracle | Finding | Severity |
|---|---|---|---|---|
| 1 | `w_saber.rs:6699` WP_SaberStartMissileBlockCheck | `w_saber.c:5561` | **One-token enumerator flip**: oracle gates on `!= ET_NPC` ("player doesn't auto-activate"); port wrote `!= ET_PLAYER` — inverted holstered-saber missile-block behavior between players and NPCs. Behind `BG_SabersOff && !CLASS_BOBAFETT`, which is why duel1 scenarios never hit it. FIXED same-day (referee suite green incl. real-map combat). | live-combat |
| 2 | `w_saber.rs:11661-11675` WP_SaberBlockNonRandom | `w_saber.c:9150-9173` | `rightdot > 0.3/0.1` double compares in f32 — F3 class, measure-zero. | latent (→ F3 sweep) |
| 3 | `w_saber.rs:658-664` SetSaberBoxSize | `w_saber.c:450-461` | The one `g_saberDebugPrint` diagnostic (of 15) dropped marker-less. Diagnostic-only; needs a marker or the print. | latent (doc) |

Prior fixes re-verified (saberEnd write-back, lastHeadAngles, ghoul2 bolt
consumption); CheckSaberDamage's full double-vs-float literal set verified;
RNG-consuming short-circuit chains verified operand-by-operand.

### bg_saber.rs + g_cmds.rs — bg_saber CLEAN in full; g_cmds priority paths clean

One cross-cutting latent: say/tell/vote paths round-trip netnames/chat
through lossy UTF-8 (`cstr_to_str`) — non-ASCII bytes (0x80-0xFF, common in
player names) become U+FFFD, diverging wire bytes/logs and team-vote name
matching vs C's raw-byte copies. Referee-blind (bots are ASCII). Fix shape:
raw-byte handling through the say/tell/vote formatting. Also: g_cmds has an
explicitly-listed NOT-audited tail (~38 lower-priority fns — team/duel/
follow/give handlers) for a wave-3 pass. bg_saber: every fn faithful, incl.
preserved saber1/saber2 copy-paste oracle bug and the vote `msg[1]` typo.

### NEW SWEEP ITEM — F3: unsuffixed-double THRESHOLD COMPARES in f32

`float dot; if (dot > 0.3)` promotes to double in C (`0.3` is a double
literal); the port compares in f32 wherever libm/multiplication didn't
force f64. Sites: G_GetHitQuad (0.3×4), G_GetHitLocation (0.1/.800/.400/
.333/.666/0.4), and port-wide instances of the same convention.
Measure-zero boundary flips (dismember quadrant, hit-location modifier),
but `435f7d57` already ruled this class a bug when it promoted
G_BounceMissile's `> 0.7`. Method: grep oracle for unsuffixed FP literals
in comparisons against float lvalues, promote per `(x as f64) > lit`.
