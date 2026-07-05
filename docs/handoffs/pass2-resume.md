# Pass-2 resume card (stopped at usage wall, 2026-07-03)

Pass-2 porter workflow was stopped cleanly at 97% usage. Worktree
`.claude/worktrees/agent-a43cc53200d2fdf54` (branch `skeleton`) holds:

- `515df55` — pass-2 prep checkpoint (ctx retrofit + fork-10 backfill, GREEN)
- `5f52915` — partial porter output WIP (16 files touched, NOT green)

## To resume (same session)

Relaunch with cache — completed porters replay instantly:

    Workflow({
      scriptPath: "/Users/milohehmsoth/.claude/projects/-Users-milohehmsoth-Developer-Milo-jka-rust--claude-worktrees-agent-a43cc53200d2fdf54/755d92df-1a93-49db-90a9-a6358fd00be0/workflows/scripts/port-jampgame-pass2-wf_1817c88f-07b.js",
      resumeFromRunId: "wf_1817c88f-07b"
    })

## To resume (fresh session — no cache)

Same scriptPath, no resumeFromRunId. Porters are idempotent ("fill ONLY
todo!() bodies, skip already-implemented"), so a fresh run only does the
remainder. If the script file is gone, the design is: porters over
`tools/closure-prototype/out/pass2/manifest.json` packets (94, sharded),
then WIP commit -> triage -> parallel per-group fixers -> serial finisher
to green. Opus set: ai_main.c, w_saber.c, NPC_AI_Jedi.c, bg_pmove.c,
g_vehicles.c; haiku < 1000 loc_parked; effort low (opus medium).

## Known-expected outcome

~290 bg/boundary fns re-park with PORT-ESCALATION(bg-boundary) — the
fork-8a PmoveContext / vehicle-enum channel is designed but unbuilt; that
is pass 3, after a design session. See fork rulings 8-11 in
`jampgame-fork-discovery.md` and `tools/closure-prototype/out/pass2/ctx-free-boundary.json`.
