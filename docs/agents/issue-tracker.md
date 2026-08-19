# Issue tracker: GitHub

Issues for this repo live as GitHub issues on `mheh/jka-rust`. Use the `gh` CLI for all operations.

## Conventions

- **Create an issue**: `gh issue create --title "..." --body "..."`. Use a heredoc for a multi-line body.
- **Read an issue**: `gh issue view <number> --comments`, and also fetch the labels.
- **List issues**: `gh issue list --state open --json number,title,body,labels,comments` with `--label` and `--state` filters as needed.
- **Comment on an issue**: `gh issue comment <number> --body "..."`
- **Apply or remove labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Close**: `gh issue close <number> --comment "..."`

Infer the repo from `git remote -v`. The `gh` CLI does this automatically inside a clone.

## Pull requests as a triage surface

**PRs as a request surface: no.** _(Set to `yes` if this repo treats external PRs as feature requests; `/triage` reads this flag.)_

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Run `gh issue view <number> --comments`.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Destination / Notes / Decisions-so-far / Fog body. `gh issue create --label wayfinder:map`.
- **Child ticket**: an issue linked to the map as a GitHub sub-issue (`gh api` on the sub-issues endpoint). Where sub-issues are not enabled, add the child to a task list in the map body and put `Part of #<map>` at the top of the child body. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: GitHub's **native issue dependencies**. Add an edge with `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`, where `<blocker-db-id>` is the blocker's numeric **database id** (`gh api repos/<owner>/<repo>/issues/<n> --jq .id`, not the `#number` or `node_id`). GitHub reports `issue_dependencies_summary.blocked_by` (open blockers only). Where dependencies are not available, fall back to a `Blocked by: #<n>, #<n>` line at the top of the child body. A ticket is unblocked when every blocker is closed.
- **Frontier query**: list the map's open children (`gh issue list --state open`, scoped to the map's sub-issues), drop any with an open blocker or an assignee. First in map order wins.
- **Claim**: `gh issue edit <n> --add-assignee @me`. This is the session's first write.
- **Resolve**: `gh issue comment <n> --body "<answer>"`, then `gh issue close <n>`, then append a context pointer (gist + link) to the map's Decisions-so-far.

## Branch pattern (ruled 2026-08-01, amended by DEC-67 2026-08-16)

Work branches from `master` and is named by the issue that drives it:

- `wf/<ticket>-<slug>` for wayfinder ticket work, for example `wf/6-renderer-surface`.
- `issue/<n>-<slug>` for plain-issue work.
- `research/<name>` for research-agent output branches (the agent commits, the architect reviews and merges).
- `chore/<slug>` for work that no issue drives, for example a DEC entry or a doc touch-up.

Every change reaches `master` through a pull request (DEC-67). Push the branch, open the PR, and merge it on GitHub with a merge commit, only green (build + gates). Never squash, and never commit directly on master. Delete the branch after the merge. Long-lived phase branches (the old `ui-port` pattern) are retired.

The PR waits at the `ci-approval` deployment gate. The owner approves the deployment in the PR's checks box (Review deployments), the matrix runs, and the required `ci-green` check unlocks the merge. The repository admin can bypass the ruleset on any PR or push directly (the DEC-67 amendment records the full mechanics).

## Session state and handoffs (DEC-52 process pass, 2026-08-01)

In-progress state for claimed work goes into a comment on the claimed ticket: what is done, what is open, and the next step. A later session reads the ticket and continues. Do not write new files under `docs/handoffs/` - that directory is legacy and stays for history only. A deep investigation that outgrows a comment gets a doc under `docs/plans/` or `docs/audits/`, and the ticket links it.

## Parked items (DEC-52 process pass, 2026-08-01)

A parked idea has exactly two homes. An idea inside the active map's scope goes into the map's "Not yet specified" section (the fog). An idea outside the map's scope becomes a plain GitHub issue. Do not park work in memory files, handoff docs, or plan docs. A plan doc records settled design, never open work.

## Repo ruling (DEC-52)

The DEC ledger in `docs/decisions.md` stays the single decision store. A wayfinder ticket resolves INTO a DEC entry. The resolution comment and the map gist and link the entry. They never hold the canonical text. This inverts wayfinder's detail-in-ticket rule on purpose.
