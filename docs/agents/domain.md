# Domain Docs

How the engineering skills consume this repo's domain documentation when they explore the codebase.

## Before exploring, read these

- `CLAUDE.md` and `docs/porting-rules.md` — the port rules and the project state.
- `docs/decisions.md` — the DEC ledger. Read the entries that touch the area you work in. Cite them, never re-litigate them.
- `docs/workspace-architecture.md` — the crate graph and the dependency tiers.
- `docs/doc-standards.md` — the template and the gates for design docs.

## No ADR scaffold

This repo does not use `CONTEXT.md` or `docs/adr/` (DEC-52). The DEC ledger in `docs/decisions.md` is the single decision store. A decision from any skill session resolves into a DEC entry there, in the ledger's existing format.

## Flag DEC conflicts

If your output contradicts a DEC entry, surface the conflict explicitly and cite the entry. Never override it silently. Only the user reopens a DEC.
