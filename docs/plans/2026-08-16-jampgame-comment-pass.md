# The jampgame comment pass

User ask 2026-08-16: apply the `asd-ste100` and `house-style` skills to all jampgame code comments, so they keep their information but lose the odd formatting (column word wrap) and the confusing text.

## Relationship to DEC-39

DEC-39 (2026-07-26) ratified a post-parity two-pass sweep: pass 1 strips port-added prose down to Raven-verbatim comments plus cites, and pass 2 is an optional organic rewrite with its design deferred. This pass differs on one axis: it retains the information instead of deleting it. It is an information-preserving restyle, not a strip. The open rows below settle how the two rulings compose. DEC-39's protected classes carry over unchanged either way.

## Scope

Default: the `mp_game` crate only (`crates/mp/game`, 171 files, 166,805 lines, 25,314 comment lines). The sibling crates (`bg` 5,715 comment lines, `qshared` 2,833, `abi` 10,388) are shared with other modules and wait for their own passes.

## What changes

Only comment text. For every port-authored comment in scope:

- Column word wrap unwraps. One sentence per line, 150-column cap, and never a wrap width inferred from the surrounding file.
- Confusing or derivation-heavy prose rewrites into STE house style: state the conclusion, keep every fact, cut re-derivations of C mechanics the cited oracle lines already show.
- The house comment shape stays: doc-comment plus `Source:` cite on every item, in the format `docs/porting-rules.md` fixes.

## What never changes

- Code. Not one token. The gate below enforces this mechanically.
- Raven-verbatim comment text, including the `QUAKED`/spawnflag blocks, character for character (DEC-39 rule 1).
- `Source:` cites (badge and assert tooling greps them).
- The layout-assert blocks.
- `SAFETY:` comments keep their prefix and their invariant content. Their prose may restyle.

## The gate: `tools/comment-gate`

A small Rust binary, built first, before any worker runs. For each changed file it lexes the git-HEAD version and the worktree version with `proc-macro2` (comments and whitespace drop out of the token stream by construction) and requires the two streams identical. One command over the whole diff, exit nonzero on any drift. A worker whose diff fails the gate is rejected without review. This is the DEC-39 rule 3 gate, built now because this pass needs it first.

## Execution shape

The established fleet pattern: packets, blind workers, mechanical referee.

1. **Anchor.** A plain GitHub issue holds the campaign, and its packets live at `.claude/packets/<issue>/`.
2. **Waves.** The 171 files shard into waves of roughly 10-15 files per worker lane, grouped by subsystem prefix (`g_*`, `ai_*`, `bg_*`, `NPC_*`, ...). About 12-15 lanes total.
3. **Workers.** Fleet tier per the model policy. Every brief carries the two skill-invoke lines, the What-changes and What-never-changes lists above verbatim, and the standing constraints: Edit tool only, `oracle/` read-only, no pushes.
4. **Per-lane gates.** `tools/comment-gate` over the lane's diff, then `cargo build --workspace`. Tests and the referee add nothing per lane, because an identical token stream compiles to an identical binary.
5. **Belt-and-suspenders.** One lockstep-referee run and one `cargo test --workspace -- --test-threads=1` at campaign end (DEC-39 rule 3 wording).
6. **Review.** Lane-review per returning worker: the vet samples each lane's diff for information loss, Raven-verbatim drift, and format violations.

## Landing

Everything lands on this branch (`chore/dec-67-gate-mechanics`), so the PR becomes the whole docs pass: the DEC-67 amendment, this plan, the gate tool, and the comment waves. The deployment gate re-arms on every push, so the one approval and the one CI run come at the end, when the branch is complete.

## Open rows

1. **Scope** - `mp_game` only (default), or the full jampgame link set (`game` + `bg` + `qshared`).
2. **DEC-39 composition** - this pass becomes DEC-68 and supersedes the two-pass shape for jampgame: no later deletion pass, the information stays (default). Alternative: this is DEC-39 pass 2 pulled forward, and pass 1's strip still happens later.
3. **Worker tier** - sonnet medium for all lanes (default), or opus-4-8 for the dense AI/combat files.
4. **Campaign anchor** - a new plain GitHub issue (default), or hang the packets off an existing issue.
5. **Landing** - all waves on this one PR branch (default), or the plan merges first and the waves take their own PR.
