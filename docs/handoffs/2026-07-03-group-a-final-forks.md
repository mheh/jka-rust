# Handoff — Group A round-3 gates done; 4 final forks await the user (2026-07-03)

Repo: `/Users/milohehmsoth/Developer/Milo/jka-rust`, branch `crate-migration`,
all commits pushed through `3d291db`. Working model unchanged: forks are decided
interactively with the user; agents execute settled work only.

## Where this sits

Three revision rounds are complete. Convergence is real: round 3 returned
engine-seam and lifecycle with **one finding each**; every remaining item is a
named, concrete fork. Full finding texts:
`docs/handoffs/group-a-regate3-findings.json`. All prior session resolutions
(rounds 1–3: 24 total) landed without contest and are on disk in the four docs.

## The 4 forks (questions were presented 2026-07-03 ~01:20, user AFK)

1. **SEAM-Q12 — engine handle inside Dispatch impls.** Handlers must call
   `trap::X(engine, ...)` mid-logic in oracle syscall order, but
   `dispatch(&self, args)` + the round-3 `WorldPtr` receiver can't reach the
   shell's `ENGINE`. *Recommended:* supersede WorldPtr with copyable
   `GameContext<'e> { world: *mut GameWorld, engine: &'e Engine }` (Engine =
   mp_engine_select alias) in mp_game; vmMain builds it per entry; SEAM-D8
   trait untouched. (Alternatives: engine param on the trait — ripples through
   cross-mode abi-transport; shell-side trap calls — breaks syscall ordering.)
2. **Engine naming collision.** `mp_engine_core::Engine` (aggregate) vs
   `mp_engine_select::Engine` (transport alias), zero cross-refs. *Recommended:*
   keep both + mandatory disambiguation notes in both docs +
   workspace-architecture (opposite islands, never co-scoped); alternatives are
   renaming either (amends frozen signatures).
3. **LOAD-Q10 — load_module not-found.** Gate found the oracle answer the doc
   missed: `VM_Create` returns NULL, the CALLER fatals
   (`sv_game.cpp:1750-1752`). *Recommended:* dated LOAD-D11 amendment —
   `load_module -> Option<SlotId>`, server caller does
   `com_error(ErrFatal, "VM_Create on game failed")`. (Alternative: fatal
   inside the registry — contradicts oracle geometry.)
4. **LIFE-D3 recursive-error gap.** Nested com_error during recovery escapes
   the consumed catch: no `recursive error after: %s` banner, abort instead of
   exit(1); doc claims equivalence it lacks. *Recommended:* wrap
   com_error_recover in its own catch_unwind; guard-set panic routes to
   `sys_error("recursive error after: {saved}")` — exact Raven banner + exit
   path. (Alternative: explicit divergence note.)

## Process question to also put to the user

Most of state-ownership's 9 "holes" are Gate-3 reading-set artifacts: the
dry-run probe reads only the target doc + doc-standards/porting-rules/
workspace-architecture/decisions, so cross-refs to sibling Group-A docs,
`abi-traps.md` rows, and lifecycle's ErrorLevel look like holes. *Proposed:*
amend doc-standards.md's Gate-3 reading set to include the sibling
`docs/architecture/*.md` set + `docs/abi-traps.md` (a porter reads the full doc
set in reality). Also mechanical: sweep stale "(pending)" annotations — the
sibling docs all exist now.

## Mechanical items for the next batch (no session needed, cite findings file)

- `EngineSlotCell` vs `EngineSlot` name drift between module-loading.md and
  engine-seam.md's frozen struct — unify (engine-seam froze `EngineSlot`).
- LOAD-Q1's macOS `suffix: &'static str` can't represent "unset" —
  `Option<&'static str>` (or empty sentinel + note), per the finding.
- STATE-Q2 stays open (owner: §F subsystem docs) — carried, not a blocker.

## FIRST ACTION on pick-up

1. Re-ask the 4 forks + the reading-set process question (AskUserQuestion;
   recommendations above).
2. Compose round-4 batch args (pattern: `group-a-regate3-args.json`; engine-seam
   entry = minimal amendment for SEAM-Q12 + naming note; state-ownership =
   GameContext supersedes WorldPtr + naming note; module-loading = LOAD-Q10
   amendment + EngineSlot/suffix mechanicals; lifecycle = recursive-error fix.
   If the reading-set amendment is approved, patch doc-standards.md FIRST and
   include it in standingDocs).
3. This should be the stamping round — expect REVIEWED across the board, then:
   user sign-off → FROZEN → delete docs/engine-plan.md → commit → compose B1–B5
   (settled decisions in the 2026-07-02 morning handoff §Wave-3a; B5 depends
   B1/B2/B4) → author the slice-0 `port-slice` workflow (see CLAUDE.md "Port
   tooling" section; goal: slice 0 in hours via skeleton → parallel fill →
   machine verify → batched escalations).

## Gotchas (carried)

- Full 4-doc batch ≈ 2h; single-doc re-gate ≈ 15–25 min. zsh: quote `===`.
  rust-analyzer stale — trust cargo. Oracle never edited. AskUserQuestion:
  plain-ASCII previews only. Keep main loop lean — delegate, read summaries.
- Workflow resume is same-session-only; args files in docs/handoffs/ are the
  durable relaunch inputs.
