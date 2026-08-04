---
name: audit
description: Freeze an agent investigation or landing into a dated record under docs/audits/. Auto-runs after broad-scope landings, prompts the user when a record merely seems useful, and runs on demand as /audit <subject>.
---

# audit

An agent report dies with its session. This skill freezes the ones worth keeping into `docs/audits/`, where the next session, the next agent, and the DEC ledger can cite them.

## When to write one

Apply this decision rule after an agent lands work or returns an investigation:

- **Write the record without asking** when the work is broad scope: a workspace-wide mechanical conversion (a `c_char` sweep, a token-scheme change), a multi-crate feature, a change touching on the order of a hundred sites, a new subsystem, or any campaign that executes a DEC ruling.
- **Ask the user first, one question** when the scope is smaller but the record seems useful: non-obvious findings, a defect hunt with reusable evidence, rulings settled mid-lane.
- **No record, no prompt** for routine landings.
- **On demand always**: the user invokes `/audit <subject>` and the record is written from the material at hand.

## The file

- Path: `docs/audits/YYYY-MM-DD-<subject-slug>.md`. The date is the day the audited work happened, the slug names the subject the way a future grep would look for it.
- Content, in order:
  1. A header paragraph: what was audited or landed, who ran it (read-only audit, work lane, scout), the packet or commits under audit, and where follow-ups land.
  2. The agent report verbatim. The report is the record - do not summarize it away. Fix transport escaping (`&gt;`, `&amp;`) but change no content.
  3. A ruling section for every user decision the report triggered, with the date and the evidence cite.
  4. Follow-ups appended as they land, each under its own heading, marked as the frozen record if the living text moved elsewhere (a packet, the DEC ledger).
- The record is append-only after its first commit. Corrections append with a date, they do not rewrite.

## Linking

- The artifact the audit serves (a packet's Amendments section, a ticket, a DEC entry) carries one pointer line to the audit file. Never paste the report into two homes - the audit file is the frozen record, the packet or ledger holds the living text.

## Commit

- Subject: `audit: <subject> (<what it serves>)`. Use `--no-gpg-sign`. No trailer of any kind: no `Co-Authored-By`, no generated-with footer.
- The audit commits separately from the work it records, so the record's history reads clean.

## Style

House style applies: STE sentence form, unwrapped paragraphs, no em dashes in body text, one name per thing. The verbatim agent report keeps its original wording - only the framing text around it is yours to write.
