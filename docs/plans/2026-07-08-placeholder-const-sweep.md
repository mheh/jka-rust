# Task #18 — repo-wide sweep for unmarked placeholder consts

Status: phases 1–3 COMPLETE 2026-07-08 (deliverables in tools/closure-prototype/out/constsweep/;
buckets: WRONG-VALUE 66, SHADOWING 415, HOUSE-NAMED 222, CLEAN 1072, SOURCE-CONFLICT 0).
Phase 4 (dispatch) awaits user go-ahead.

## The bug class

Skeleton-phase porting left function-local `const NAME: c_int = <guessed literal>;` stand-ins and
enum-looking bare literals with NO marker comment. They compile, pass marker triage, and have now
evaded three manual passes (marker triage, an Opus fix agent working in the same function, and the
batch-4 author). Confirmed live bugs of this class, all in g_utils.rs (fixed 72c25697/8e0cf1aa):
GT_SIEGE=4 (real 7), SETANIM_TORSO=2 (=LEGS; real 1), CLASS_VEHICLE=0 (=CLASS_NONE),
STAT_MAX_HEALTH=1 (real 8; slot 1 is STAT_HOLDABLE_ITEM), HI_JETPACK=2 (real 7), PMF_FOLLOW=0
(real 4096), all-zero trace content masks, `.value as c_int` where oracle reads `.integer`.
Known secondary signature: a comment falsely claiming "not yet ported / no canonical exists"
while the canonical sits in the prelude.

## Process

Phase 1 — enumerate (script, exhaustive). Extract every `const NAME: <int type> = <literal>`
(function-local AND module-level) across crates/mp → file, line, enclosing fn, value. Also grep
lying-comment signatures ("not yet ported", "no canonical", "placeholder", "stand-in") near consts.

Phase 2 — ground truth (script, both sides).
- Oracle: libclang dump of NAME→value for every enum member + #define in oracle/oracle/codemp
  headers (q_shared.h, bg_public.h, bg_weapons.h, anims.h, teams.h, g_local.h, ...) reusing
  tools/closure-prototype machinery (same parse that extracts layouts).
- Workspace: extract every `pub const NAME: <int> = <value>` from crates/mp (prelude, mp_bg,
  mp_qshared, ...) → NAME→(value, path).

Phase 3 — mechanical join (script). Bucket every local const:
- WRONG-VALUE: name matches oracle, value differs → live bug worklist (headline).
- SHADOWING: value correct but canonical workspace def exists → import-instead worklist.
- HOUSE-NAMED: name in neither table (e.g. LAST_USEABLE_WEAPON) → judgment worklist.
- CLEAN: name+value match oracle and no canonical to prefer (or intentionally-local idiom like
  qtrue/qfalse mirrors, EV_EVENT_BITS).
Output: tools/closure-prototype/out/constsweep/ (script + json tables + worklist.md with counts).
NOTE: out/ is gitignored — durable on disk, not in git; commit the script itself later if kept.

=== STOP POINT (end of this session) ===

Phase 4 — judgment wave (parallel Opus agents, disjoint file sets; NOT YET RUN). Each agent:
- fixes its WRONG-VALUE items with canonical imports AND re-reviews surrounding logic vs oracle
  line-by-line (TryUse lesson: a wrong const marks a half-ported region);
- adjudicates HOUSE-NAMED items at the oracle usage site;
- re-checks lying comments;
- targeted bare-literal patterns (judgment-only part): `stats[<n>]`, `gametype == <n>`,
  class/team/holdable comparisons, SETANIM args, content masks — verify the oracle's line names a
  constant the Rust should too.
Agents edit, do NOT commit. Standard per-agent clause applies (memory:
agent-prompts-check-placeholder-consts).

Phase 5 — integrate (one agent): workspace build, BOTH rig corpora (run-ab.sh, artifacts now at
~/Developer/jka/seam-test/referee/artifacts/), commits in logical chunks, no co-author trailers,
findings appended to docs/audits/.

## Scope decisions (user-approved 2026-07-08)
- crates/mp only (sp unshipped; jagame known-todo).
- Integer identity-constants only; float literals and array sizes out of scope.
