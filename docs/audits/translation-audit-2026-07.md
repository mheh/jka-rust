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

### g_active.rs + g_missile.rs

Report pending (agent in flight at wave-1 close); findings to be appended.
