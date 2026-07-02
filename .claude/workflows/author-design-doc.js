export const meta = {
  name: 'author-design-doc',
  description: 'Draft logic-port design docs from user-settled decisions, then gate them (adversarial review + dry-run)',
  whenToUse: 'AFTER an interactive design session has settled a doc\'s decisions/outline (docs/doc-standards.md pipeline). args = {docs: [{path, title, prefix, scope, decisions[], outline, dossierPath?, oracleAnchors?[], model?, cppTrack?, depends?[]}], standingDocs?[], decisionsPath?}. Drafting agents make NO design decisions — contested points come back as needsSession. Docs end at Status: REVIEWED; FROZEN requires user sign-off outside the workflow. Output uncommitted.',
  phases: [
    { title: 'Draft', detail: 'render settled decisions into the doc-standards template' },
    { title: 'Review', detail: 'mechanical checklist + adversarial review, revise rounds' },
    { title: 'Dry-run', detail: 'fresh agent builds scratch skeleton + slice plan from the doc alone' },
  ],
}

const A = (typeof args === 'string' ? JSON.parse(args) : args) || {}
if (!Array.isArray(A.docs) || !A.docs.length) throw new Error('args.docs required')
for (const d of A.docs) {
  for (const k of ['path', 'title', 'prefix', 'scope', 'decisions', 'outline']) {
    if (d[k] === undefined) throw new Error(`docs entry ${d.path || '?'} missing ${k} — run the interactive session first`)
  }
}
const STANDING = A.standingDocs || [
  'docs/doc-standards.md', 'docs/porting-rules.md', 'docs/workspace-architecture.md',
]
const LEDGER = A.decisionsPath || 'docs/decisions.md'

const COMMON = `GROUND RULES:
- NEVER touch oracle/oracle/ or any source under crates/. Never commit.
- The doc template, style rules, and gates are docs/doc-standards.md — follow it
  exactly (fixed H2 skeleton, cite-or-omit, greppable IDs, no duplication of
  standing docs).
- Standing context docs: ${STANDING.join(', ')}; decision ledger: ${LEDGER}.
- You make NO design decisions. The settled decisions and outline you are given
  are the design. Anything they do not cover that you cannot resolve from cited
  oracle ground truth is reported back, never invented.`

const DRAFT_SCHEMA = {
  type: 'object',
  properties: {
    written: { type: 'boolean' },
    uncovered: { type: 'array', items: { type: 'string' } },
  },
  required: ['written', 'uncovered'],
}
const REVIEW_SCHEMA = {
  type: 'object',
  properties: {
    pass: { type: 'boolean' },
    fixedInPlace: { type: 'array', items: { type: 'string' } },
    defects: { type: 'array', items: { type: 'string' } },
    contested: { type: 'array', items: { type: 'string' } },
  },
  required: ['pass', 'fixedInPlace', 'defects', 'contested'],
}
const DRYRUN_SCHEMA = {
  type: 'object',
  properties: {
    pass: { type: 'boolean' },
    holes: { type: 'array', items: { type: 'string' } },
    inventedDecisions: { type: 'array', items: { type: 'string' } },
  },
  required: ['pass', 'holes', 'inventedDecisions'],
}

async function authorDoc(d) {
  const label = d.path.split('/').pop().replace(/\.md$/, '')
  const cppNote = d.cppTrack ? `
This is a C++-track designPath doc: it must additionally carry the
files: [{path, crate, mode, class, summary}] roster and the divergences list so
.claude/workflows/port-cpp-subsystem.js can consume it unchanged (see
docs/subsystems/cpp/ requirements in doc-standards rule 6).` : ''

  // ------------------------------------------------------------------ Draft
  const draft = await agent(
`Write the design doc ${d.path} ("${d.title}", decision prefix ${d.prefix}).
Scope: ${d.scope}
${COMMON}${cppNote}
Inputs (the settled design — render it, don't reinterpret it):
- Settled decisions (become the ## Decisions records, in order):
${d.decisions.map((x, i) => `  ${d.prefix}-D${i + 1}: ${x}`).join('\n')}
- Outline / section guidance from the design session:
${d.outline}
${d.dossierPath ? `- Survey dossier (ground truth with cites — verify any cite you reuse): ${d.dossierPath}` : '- No dossier: gather ground truth from the oracle yourself, citing every claim.'}
${d.oracleAnchors && d.oracleAnchors.length ? `- Key oracle anchors: ${d.oracleAnchors.join('; ')}` : ''}
Set the header Status: DRAFT. Every Raven claim cited. Anything the inputs
don't settle and oracle ground truth can't resolve goes in ## Open questions
AND your structured "uncovered" list — do not decide it.
Return the structured result only.`,
    { label: `draft:${label}`, phase: 'Draft', model: d.model || 'opus', effort: 'high', schema: DRAFT_SCHEMA })
  if (!draft || !draft.written) return { path: d.path, status: 'FAILED', contested: [], holes: ['draft agent failed'] }

  // ----------------------------------------------------------------- Review
  let review = null
  const contested = [...(draft.uncovered || [])]
  for (let round = 0; round < 3; round++) {
    review = await agent(
`Adversarially review the design doc ${d.path} (Gate 1 + Gate 2 of
docs/doc-standards.md — run BOTH).
${COMMON}
Gate 1 (mechanical): all template sections present and in order; every
oracle cite RESOLVES and says what's claimed (spot-verify by reading the cited
lines); ## Open questions empty or explicitly escalated; every decision has an
ID + <=2-line rationale; state table covers the globals${d.dossierPath ? ` in ${d.dossierPath}` : ' the oracle shows'};
no duplication of standing docs.${d.cppTrack ? ' C++-track roster/divergences present and schema-complete.' : ''}
Gate 2 (adversarial): find a cited Raven behavior the design cannot reproduce;
a porting-rules clause violated; a fork a future porter would hit that no
decision covers; a seam signature that can't round-trip its cited trap;
a conflict with ${LEDGER} or the standing docs.
FIX directly in the doc: mechanical defects, wrong/imprecise cites, template
violations. DO NOT change any design decision — a finding that would alter a
decision goes in "contested" verbatim. Genuine defects you cannot fix without
deciding something go in "defects".
Return the structured result; pass = no remaining defects.`,
      { label: `review:${label}#${round + 1}`, phase: 'Review', model: 'opus', effort: 'high', schema: REVIEW_SCHEMA })
    if (!review) break
    contested.push(...review.contested)
    if (review.pass || !review.defects.length) break
    await agent(
`Revise the design doc ${d.path} to clear these review defects, without making
any new design decision (the settled decisions are listed inside the doc's
## Decisions — they are immutable here).
${COMMON}
Defects:
${review.defects.map(f => `- ${f}`).join('\n')}
Return the structured result.`,
      { label: `revise:${label}#${round + 1}`, phase: 'Review', model: d.model || 'opus', schema: DRAFT_SCHEMA })
  }

  // ---------------------------------------------------------------- Dry-run
  let dry = null
  for (let round = 0; round < 2; round++) {
    dry = await agent(
`Dry-run comprehensiveness probe (Gate 3 of docs/doc-standards.md) for
${d.path}. You are a porter with NO prior context: read ONLY ${d.path},
${STANDING.join(', ')}, ${LEDGER}, and the oracle (read-only).
Produce IN YOUR SCRATCH ANALYSIS ONLY (write no files, change nothing):
(a) the Rust skeleton this doc implies — file layout, pub signatures, owned
state struct(s); (b) a step plan for the first slice touching this area
(## Slice hooks). Then judge: could you execute without asking a human
anything? Report every question you could not self-answer from the doc+oracle
("holes") and every point where you had to INVENT a decision the doc should
have made ("inventedDecisions"). pass = both lists empty. Be adversarially
honest — a soft pass here costs far more later.
Return the structured result only.`,
      { label: `dryrun:${label}#${round + 1}`, phase: 'Dry-run', model: 'sonnet', effort: 'high', schema: DRYRUN_SCHEMA })
    if (!dry || dry.pass) break
    await agent(
`Patch the design doc ${d.path} to close these dry-run holes WITHOUT making new
design decisions: holes answerable from cited oracle ground truth or the
settled decisions get answered in place; holes that genuinely require a new
decision get listed under ## Open questions (they will go back to a design
session). ${COMMON}
Holes:
${[...dry.holes, ...dry.inventedDecisions].map(h => `- ${h}`).join('\n')}
Return the structured result.`,
      { label: `patch:${label}#${round + 1}`, phase: 'Dry-run', model: d.model || 'opus', schema: DRAFT_SCHEMA })
  }

  const clean = (review ? review.pass || !review.defects.length : false) && dry && dry.pass && !contested.length
  if (clean) {
    await agent(
`In ${d.path}, set the header line "Status: DRAFT" to "Status: REVIEWED".
Change nothing else. Return {"written": true, "uncovered": []}.`,
      { label: `stamp:${label}`, phase: 'Dry-run', model: 'haiku', schema: DRAFT_SCHEMA })
  }
  return {
    path: d.path,
    status: clean ? 'REVIEWED' : 'NEEDS_SESSION',
    contested: [...new Set(contested)],
    holes: dry && !dry.pass ? [...dry.holes, ...dry.inventedDecisions] : [],
    defects: review && review.defects.length ? review.defects : [],
  }
}

// Dependency levels: docs whose depends[] are all satisfied run concurrently.
const byPath = new Map(A.docs.map(d => [d.path, d]))
const done = new Set()
const results = []
let remaining = [...A.docs]
while (remaining.length) {
  const ready = remaining.filter(d => (d.depends || []).every(p => done.has(p) || !byPath.has(p)))
  if (!ready.length) throw new Error(`dependency cycle among: ${remaining.map(d => d.path).join(', ')}`)
  log(`authoring ${ready.length} doc(s): ${ready.map(d => d.path.split('/').pop()).join(', ')}`)
  const batch = await parallel(ready.map(d => () => authorDoc(d)))
  for (const r of batch.filter(Boolean)) { results.push(r); done.add(r.path) }
  for (const d of ready) done.add(d.path)
  remaining = remaining.filter(d => !done.has(d.path))
}

return {
  reviewed: results.filter(r => r.status === 'REVIEWED').map(r => r.path),
  needsSession: results.filter(r => r.status !== 'REVIEWED'),
}
