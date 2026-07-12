# Design-Doc Standards

The template contract for the logic-port design docs (`docs/architecture/`,
`docs/modules/`, `docs/subsystems/`, `docs/subsystems/cpp/`). Every doc is
written to be loaded standalone by a delegated agent that has never seen the
repo, alongside only `porting-rules.md`, `workspace-architecture.md`,
`decisions.md`, and oracle read access. Anything a porter would otherwise need
to "go figure out" is a doc defect.

Docs are authored through the interactive pipeline: survey dossier → decision
brief → **user-settled decisions** → delegated draft → adversarial review →
dry-run gate → user sign-off. Drafting agents make **no** design decisions; they
render settled ones.

## Required skeleton

Fixed H2 names, in order, so agents can `grep -A` into any doc blind:

```markdown
# <Area> Design
Status: DRAFT | REVIEWED | FROZEN     Supersedes: <doc or "none">
Decision prefix: <SV|FS|NET|CL|G|CG|UI|...>     Ledger deps: DEC-xx, DEC-yy

## Standing context
Links only — never restate: workspace-architecture.md (crate graph),
porting-rules.md (rules), decisions.md (settled choices), abi-traps.md
(trap signatures), sibling design docs this one builds on.

## Scope & non-goals
What this doc decides; what it explicitly punts, each punt with a pointer to
where it is (or will be) decided.

## Raven ground truth
How it actually works in C: init order, data flow, frame role, globals.
CITE OR OMIT: every behavioral claim carries `oracle/<path>:<line>`.
No "probably", no "presumably".

## State ownership
Mandatory table: Raven global | oracle cite | Rust owner (crate::Type.field) |
constructed by | threaded via. Must cover every global the survey found.

## Seam definition
What crosses ABI/crate boundaries: traps used (abi-traps.md rows), #[repr(C)]
types touched, and the pub Rust API this area exposes — EXACT signatures.
This section is what freezes; porters fill it without changing it.

## Decisions
Numbered records <PREFIX>-Dn: "We do X. Because Y. Rejected Z because W."
One decision per record, conclusion + ≤2-line rationale, alternatives as
one-line rejections. These come from the interactive sessions.

## Verification strategy
What oracle parity means HERE, per DEC-09: which TU harnesses, which live-peer
checks, which fixtures; which porting-rules clause governs (§E for native
track, §F for C++ track).

## Slice hooks
Which logic-port-plan slices touch this area, and what each needs frozen first.

## Open questions
MUST be empty at FROZEN. Each entry either resolves in review or escalates to
an interactive session (never self-resolved by an agent).
```

## Style rules (enforced by the review gate)

1. **Cite or omit** — uncited Raven claims are review defects.
2. **Decision-record voice** — conclusions with ≤2-line rationale (mirrors the
   porting-rules comment rule).
3. **Greppable IDs** — decisions `NET-D4`, open questions `NET-Q2`, ledger
   `DEC-07`. Cross-doc references use IDs, not prose.
4. **No duplication of standing docs** — if a paragraph could be "see
   porting-rules §B", it must be.
5. **Signatures are load-bearing** — `## Seam definition` spells exact Rust
   signatures; freeze semantics identical to `port-cpp-subsystem` design docs.
6. **C++-track docs** additionally carry the `files: [{path, crate, mode,
   class, summary}]` roster and `divergences` list so they drop into
   `port-cpp-subsystem`'s `designPath` unchanged.
7. **Status lifecycle** — DRAFT (author) → REVIEWED (adversarial gate passed) →
   FROZEN (dry-run gate passed + user sign-off). After FROZEN, changes are
   dated amendment notes appended to the affected section, never silent edits.
8. **Per-mode discipline** — state tables and seams are per-mode (DEC-04);
   an MP/SP pair is documented as MP + SP-diff, mirroring the porting style.

## Gates (run per doc, in order)

- **Gate 1 — mechanical checklist**: all sections present; every cite resolves;
  `## Open questions` empty or escalated; every decision has ID + rationale;
  state table covers the survey's global inventory; no standing-doc
  duplication; C++-track schema completeness.
- **Gate 2 — adversarial review** (fresh context): attack the decisions — find
  a cited Raven behavior the design cannot reproduce, a porting-rules clause
  violated, a fork a porter would hit that no decision covers, a seam signature
  that can't round-trip its trap. Revise until no confirmed findings; contested
  points escalate to the user, never self-resolve.
- **Gate 3 — dry-run** (fresh context): given this doc + standing docs + the
  sibling `docs/architecture/*.md` set + `docs/abi-traps.md` + oracle (what a
  real porter reads — amended 2026-07-03; a single-doc reading set registered
  legitimate cross-doc deferrals as holes), produce a scratch skeleton (file
  layout, pub signatures, owned-state struct) and a first-slice plan. Pass =
  zero unanswerable questions, seam matches the doc verbatim, no invented
  decisions. A cross-ref to a sibling doc is a hole only if the pointer is
  wrong or the target doesn't answer it. Any hole → revise → re-run.
