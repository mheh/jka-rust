# Handoff — Group A revision batch, halted at the 1-hour mark (2026-07-02)

Repo: `/Users/milohehmsoth/Developer/Milo/jka-rust`, branch `crate-migration`.
All commits local, NOT pushed. Working model unchanged: design forks are decided
interactively with the user; agents execute settled work only
(`docs/handoffs/2026-07-02-logic-port-docs.md` §"Binding working model").

## Where this sits in the plan

The Group A drafting batch (this morning's first action) completed and all four
docs came back NEEDS_SESSION. The escalation session was HELD and all 12
decisions are settled — they are recorded in
`docs/handoffs/group-a-revision-args.json` (the exact revision-batch args) and,
where ledger-level, as dated amendments in `docs/decisions.md` (DEC-02, DEC-07).
`docs/workspace-architecture.md` was updated with the settled crate topology
(engine/core facades, cdylib shells vs logic crates, abi-transport →
native/platform edge, MAX_GENTITIES tier note).

Session resolutions (summary; full text in the args file):
- SEAM-D10 thin shells (crates/jampgame etc. wrap transport-agnostic mp_* logic crates)
- STATE-D5 mp_engine_core facade defines Engine + com_* (LIFE-D2 amended to match)
- Static subsumes DEC-07 outbound word path (vmachine shim inbound-only)
- extern "C-unwind" at the seam (catch stays engine-side)
- STATE-D6 WorldCell shell static (2nd §D11 exemption; Mutex/RefCell rejected — real reentrancy)
- SEAM-D11 per-slot trampoline + Drop-guard *mut Engine stash (hosting real DLLs)
- Engine.snd: Option<SoundSystem>
- LOAD-D6 raw aliases move to native/platform; abi-transport re-exports
- LOAD-D1 amended: per-platform direct_first (Unix has NO bare-dlopen probe — #if 0'd)
- Sys_UnpackDLL in scope, deferred slice; slice 0 stubs
- ModuleRegistry per VM_Create slot semantics, home mp_engine_qcommon
- LIFE-D1 softened (one-frame input-deferral divergence noted); deferred opens:
  macOS filename infix, winit keymap table, STATE-Q2 §F subcrates

## State of the revision batch (run `wf_481e1cff-6c1`, HALTED)

**engine-seam.md — revision APPLIED, gates ~90% done.** Chain completed: revise
→ adversarial review (PASS, cite fixes applied) → dry-run#1 → patch#1 →
dry-run#2 → patch#2; halted during the third dry-run. The doc on disk carries
all session resolutions. Needs: one fresh dry-run gate to stamp it.

**state-ownership.md / lifecycle.md / module-loading.md — revisions NOT
STARTED** (they depend on engine-seam in the batch DAG). Their on-disk drafts
are still the first-batch versions; their revision entries in
`group-a-revision-args.json` are ready to run verbatim.

## New escalations from the engine-seam gates (need the user)

1. **SEAM-Q10 (new, genuine fork):** physical home of the per-module
   `type Engine` alias / `mod trap`, and the per-build mechanism binding it to
   CEngine vs Static vs wasm while logic crates stay transport-agnostic
   (SEAM-D10). No oracle ground truth, no in-repo precedent. Left in
   engine-seam.md ## Open questions.
2. **Decision-numbering confirmation:** the on-disk doc already used SEAM-D10
   for the C-unwind decision; the reviser honored the session brief
   (thin-shells=D10, trampoline=D11) and renumbered C-unwind to **SEAM-D12**,
   updating cross-refs. Confirm with the user that C-unwind=SEAM-D12 is fine.
3. (Unchanged) SEAM-Q7 — GetModuleAPI OpenJK-handshake contract stays open,
   deferred to module-loading.md (zero oracle occurrences).

## FIRST ACTION on pick-up

1. Quick session with the user: resolve SEAM-Q10 (recommendation prepared:
   present alias-in-shell-crate vs cfg-in-logic-crate options) and confirm the
   SEAM-D12 renumbering.
2. Relaunch the batch (fresh — workflow resume is same-session-only):

   ```
   Workflow({
     scriptPath: ".claude/workflows/author-design-doc.js",
     args: <group-a-revision-args.json, with the engine-seam entry REPLACED by a
            minimal revision entry: bake in the SEAM-Q10 resolution + confirm
            numbering, outline "apply only these amendments, then re-gate";
            other three entries verbatim>
   })
   ```

3. When the batch completes: gate results → user answers any new escalations →
   sign-off → FROZEN → commit → delete docs/engine-plan.md (superseded) →
   compose the B1–B5 batch (decisions already settled, see the morning handoff
   §Wave-3a; B5 `depends` on B1/B2/B4).

## Gotchas (carried forward + new)

- Timebox: a full 4-doc batch runs ~1.5–2h; the dominant cost is gate loops.
  A single-doc re-gate is ~15–25 min. Budget accordingly or run overnight.
- zsh: quote `===`; unquoted `$VAR` doesn't split. rust-analyzer stale — trust
  cargo only. Oracle never edited. AskUserQuestion: plain ASCII diagrams only.
- The user directive (saved in auto-memory `minimize-context-delegate-execution`):
  keep the main loop lean — delegate to Sonnet/Opus, read summaries not full docs.
- Standing docs were ALREADY updated for the settled topology; drafters/gates
  should see no workspace-architecture contradictions. decisions.md amendments
  for DEC-02/DEC-07 are in.
