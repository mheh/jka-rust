# Handoff — Group A: round-4 args ready to launch (2026-07-03, post-session)

Repo: `/Users/milohehmsoth/Developer/Milo/jka-rust`, branch `crate-migration`,
all pushed. Working model unchanged: forks decided interactively with the user;
agents execute settled work only.

## State

Rounds 1–3 complete; ALL forks are now settled (29 total across four sessions).
The 2026-07-03 late-night session resolved the last five:

1. **SEAM-Q12 → GameContext.** Copyable
   `GameContext<'e> { world: *mut GameWorld, engine: &'e Engine }` in mp_game
   (Engine = mp_engine_select alias) supersedes WorldPtr; SEAM-D8 trait
   untouched; leaf call sites unchanged (`trap::X(engine, ...)`).
2. **Engine naming → keep both + disambiguate.** Canonical block added to
   workspace-architecture.md (2026-07-03); both docs get crate-qualifying notes.
3. **LOAD-Q10 → `load_module -> Option<SlotId>`,** caller fatals per
   `sv_game.cpp:1750-1752` (dated LOAD-D11 amendment; add the omitted
   caller-side ground truth).
4. **Recursive error → catch around recovery**; guard-set panic routes to
   `sys_error("recursive error after: {saved}")` (LIFE-D3 dated amendment).
5. **Gate-3 reading set amended** in doc-standards.md (2026-07-03): sibling
   `docs/architecture/*.md` + `docs/abi-traps.md` now in the probe set; a
   correct cross-doc pointer is no longer a hole.

Standing docs ALREADY patched: doc-standards.md (Gate-3 set),
workspace-architecture.md (Engine disambiguation block).

## FIRST ACTION on pick-up (morning)

Launch round 4 — expected to be the stamping round:

```
Workflow({ scriptPath: ".claude/workflows/author-design-doc.js",
           args: <docs/handoffs/group-a-regate4-args.json verbatim> })
```

Then: gate results → (any stragglers to the user) → sign-off → FROZEN ×4 →
delete docs/engine-plan.md → commit → compose B1–B5 batch (settled decisions in
the 2026-07-02 morning handoff §Wave-3a; B5 depends B1/B2/B4) → author the
slice-0 workflow.

## Process decision (user, 2026-07-03): incremental builds with reset points

Adopted direction for the code phase: validation builds are **cumulative, not
scratch**. Mechanism:

- A persistent **skeleton worktree/branch** seeded from the frozen docs' entire
  surface (crates, frozen signatures, `todo!()` bodies), `cargo check` green =
  the base build.
- Every phase/agent works on top of it; **each green state is a checkpoint
  commit** (tag per phase boundary). rustc becomes the dry-run: the
  SEAM-Q12-class contradictions round 3 found by prose probes fall out as
  compile errors instead.
- **Reset semantics:** step back = `git reset` to a checkpoint; deep reset =
  drop N checkpoints; restart entirely = re-branch from the seed commit.
  Nothing is unrecoverable, nothing is re-derived from scratch.
- Convergence: the validated skeleton IS slice 0's starting point, not
  throwaway gate output. Build this into the slice-0 `port-slice` workflow
  design (see CLAUDE.md "Port tooling" section). For the remaining doc round,
  gates stay as-is (stamping imminent; don't churn mechanics mid-round).

## Gotchas (carried)

- Full 4-doc batch ≈ 2h; the round-4 engine-seam entry is minimal (fast).
- zsh: quote `===`. rust-analyzer stale — trust cargo. Oracle never edited.
  AskUserQuestion previews: plain ASCII. Keep main loop lean — delegate, read
  summaries not full docs.
- Workflow resume is same-session-only; args files in docs/handoffs/ are the
  durable relaunch inputs. Findings files: group-a-regate{,2,3}-findings.json.
