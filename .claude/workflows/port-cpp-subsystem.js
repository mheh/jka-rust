export const meta = {
  name: 'port-cpp-subsystem',
  description: 'Idiomatically reimplement one Raven C++ subsystem (design-frozen skeleton, parallel porters, differential golden verification)',
  whenToUse: 'C++-track subsystems (porting-rules §F: ghoul2, FX, icarus, ROFF, terrain/RMG, containers). args = {subsystem, mpCrate, spCrate, mpDir, spDir, mpOracle: [paths], spOracle: [paths], designPath?, hard?: [class...], skipDocs?} — oracle lists are repo-relative .h/.cpp paths (empty list = mode not present); designPath names a pre-written, reviewed design doc and skips the Design phase. Exemplar everywhere: GP2 (crates/mp/engine/qcommon/src/gp2, crates/sp/qshared/src/common/sp/game/gp2, tools/gp2-oracle).',
  phases: [
    { title: 'Scout', detail: 'class roster, consumer-called API surface, MP/SP divergences' },
    { title: 'Design', detail: 'one high-effort design doc + adversarial review (skipped when designPath given)' },
    { title: 'Harness', detail: 'tools/<subsystem>-oracle: unmodified oracle TUs + stubs, fixtures, goldens, Rust parity tests' },
    { title: 'Skeleton', detail: 'frozen type defs + method signatures, todo!() bodies, cargo green' },
    { title: 'Port', detail: 'parallel per class, MP first, SP chained as diff' },
    { title: 'Verify', detail: 'parity vs goldens + cargo test + zero warnings, fixer rounds' },
    { title: 'Docs', detail: 'type-port-todo C++-track table' },
  ],
}

const A = (typeof args === 'string' ? JSON.parse(args) : args) || {}
for (const k of ['subsystem', 'mpCrate', 'spCrate', 'mpDir', 'spDir', 'mpOracle', 'spOracle']) {
  if (A[k] === undefined) throw new Error(`args.${k} required`)
}
const S = A.subsystem
const HARD = new Set(A.hard || [])
const EXEMPLAR = `EXEMPLAR (copy its shape, do not improvise a new one): the GP2 pilot —
Rust: crates/mp/engine/qcommon/src/gp2/ (MP), crates/sp/qshared/src/common/sp/game/gp2/ (SP);
harness: tools/gp2-oracle/ (run.sh, stubs/, main.cpp, fixtures/, golden/, README.md);
parity tests: crates/mp/engine/qcommon/tests/gp2_parity.rs and the sp_qshared twin.`

const RULES = `HOUSE RULES (binding — docs/porting-rules.md, esp. §F rules 17-21):
- NEVER edit anything under oracle/. Never commit; leave work for review.
- This is the C++ TRACK: idiomatic reimplementation, NOT byte-faithful. No
  #[repr(C)], no offset asserts. Behavior parity is what's checked — the
  differential goldens are the spec.
- The design doc is FROZEN. Fill todo!() bodies; do not change any pub
  signature, type shape, or file layout. If a signature is genuinely wrong,
  report it under problems in your structured result — do not improvise.
- Only touch your assigned files. mod.rs/lib.rs are pre-wired.
- Transcribe behavior from the cited oracle .cpp lines, control-flow-faithful
  first (§C10 allows reshaping only when observable behavior is identical).
  Preserve Raven's comments where they clarify behavior.
- Diverge ONLY where Raven is UB (buffer overrun, null deref, uninit read):
  pick the one defined behavior and note it in <=2 lines at the site (§F19).
- Preserve emergent per-mode quirks — SP is a duplicate, not a unification;
  port what the SP source says even when it looks like a bug (§F20).
- No static mut, no globals (§B3): Raven file-statics become struct fields or
  locals of the owning type.
- Doc comments in house style: /// Raven \`X\` — <one line>. + class-definition
  cite AND method-source cite (see any gp2 file).
- Inline #[cfg(test)] unit tests for tricky semantics you had to reason about.
  Do not call sibling classes whose bodies are still todo!() from your tests.
- Trust ONLY cargo (rust-analyzer is stale). End GREEN with ZERO new warnings.
- Tools and orchestration are BLACK BOXES: do not read .claude/workflows/.
${EXEMPLAR}`

// ------------------------------------------------------------------ Scout
phase('Scout')
const SCOUT_SCHEMA = {
  type: 'object',
  properties: {
    classes: { type: 'array', items: { type: 'object', properties: {
      name: { type: 'string' },
      mpHeaderCite: { type: 'string' }, mpCppCite: { type: 'string' },
      spHeaderCite: { type: 'string' }, spCppCite: { type: 'string' },
      methodCount: { type: 'number' },
      spDiverges: { type: 'string' },
    }, required: ['name', 'methodCount'] } },
    consumerApi: { type: 'array', items: { type: 'string' } },
    statics: { type: 'array', items: { type: 'string' } },
    notes: { type: 'array', items: { type: 'string' } },
  },
  required: ['classes', 'consumerApi', 'statics', 'notes'],
}
const scout = await agent(
`Scout the Raven C++ subsystem "${S}" for an idiomatic Rust reimplementation
(repo root cwd; read-only — edit NOTHING).
Oracle sources: MP ${JSON.stringify(A.mpOracle)} SP ${JSON.stringify(A.spOracle)}.
1. Class roster: every class/struct these files define, with header cite
   (path:lines), .cpp method-block cite, method count.
2. MP-vs-SP: diff the twins; per class, one line on behavioral divergences
   (not formatting) — e.g. GP2's SP AddGroup never sets mParent.
3. Consumer API surface: grep the rest of oracle for call sites of
   these classes; list method names actually called by consumers (this drives
   the differential dump format and which API must exist).
4. File-statics / globals the .cpp files rely on (they must become owned state).
Return the structured result only.`,
  { label: `scout:${S}`, phase: 'Scout', model: 'sonnet', schema: SCOUT_SCHEMA })
if (!scout || !scout.classes.length) throw new Error('scout found no classes')
log(`scout: ${scout.classes.length} classes, ${scout.consumerApi.length} consumer-called methods`)

// ----------------------------------------------------------------- Design
phase('Design')
const DESIGN_SCHEMA = {
  type: 'object',
  properties: {
    designPath: { type: 'string' },
    files: { type: 'array', items: { type: 'object', properties: {
      path: { type: 'string' }, crate: { type: 'string' },
      mode: { type: 'string', enum: ['mp', 'sp'] },
      class: { type: 'string' }, summary: { type: 'string' },
    }, required: ['path', 'crate', 'mode', 'class'] } },
    divergences: { type: 'array', items: { type: 'string' } },
  },
  required: ['designPath', 'files', 'divergences'],
}
let design
if (A.designPath) {
  design = await agent(
`Read the pre-written design doc ${A.designPath} for subsystem "${S}" and
return its file plan as structured data (path/crate/mode/class per Rust file,
plus the documented UB divergences). Read-only; edit nothing.`,
    { label: 'design:load', phase: 'Design', model: 'sonnet', schema: DESIGN_SCHEMA })
} else {
  design = await agent(
`Design the idiomatic Rust shape for Raven C++ subsystem "${S}" and write it to
docs/design/${S}-cpp-track.md (create dirs as needed). Repo root cwd.
Oracle: MP ${JSON.stringify(A.mpOracle)} SP ${JSON.stringify(A.spOracle)}.
Scout findings: ${JSON.stringify(scout)}
Targets: MP crate ${A.mpCrate} dir ${A.mpDir}; SP crate ${A.spCrate} dir ${A.spDir}.
Apply porting-rules §F17: closed virtual hierarchies -> enums; interface
classes -> arena + id newtype + copyable borrow wrapper (§B5) whenever
consumers walk parent/sibling/handle relations; intrusive lists, pools,
std:: members -> owned Vec/String/std collections; file statics -> owned
state. Design MP first, SP as documented diff (duplicate, don't unify).
The doc MUST pin, per Rust file: exact pub type definitions, every pub method
signature, which Raven methods map to each, dead API dropped (with the
zero-callers evidence), and every intended UB divergence. Porters will
transcribe INTO these signatures without changing them, so be complete.
${EXEMPLAR}
Return the structured file plan.`,
    { label: `design:${S}`, phase: 'Design', effort: 'high', schema: DESIGN_SCHEMA })
  if (!design) throw new Error('design agent failed')
  const review = await agent(
`Adversarially review the C++-track design doc ${design.designPath} for
subsystem "${S}" (repo root cwd). Check against docs/porting-rules.md §F and
§B, the oracle sources (MP ${JSON.stringify(A.mpOracle)} SP ${JSON.stringify(A.spOracle)}),
and the scout findings: ${JSON.stringify(scout)}.
Hunt specifically for: consumer-called API missing from the design (grep call
sites yourself); parent/sibling walks that need the arena pattern but got an
owned tree; SP divergences silently unified away; speculative API with no
callers; hidden global state surviving as a Rust global. FIX the doc in place
for anything confirmed (keep it consistent), and list what you changed.
Return JSON: {"changes": ["..."], "ok": true|false} via the structured result.`,
    { label: 'design:review', phase: 'Design', effort: 'high',
      schema: { type: 'object', properties: {
        changes: { type: 'array', items: { type: 'string' } }, ok: { type: 'boolean' } },
        required: ['changes', 'ok'] } })
  log(`design review: ${review ? review.changes.length : '?'} changes`)
}
if (!design || !design.files.length) throw new Error('no design file plan')
const mpFiles = design.files.filter(f => f.mode === 'mp')
const spFiles = design.files.filter(f => f.mode === 'sp')

// ---------------------------------------------------------------- Harness
// Built from the design doc so the Rust parity tests it writes target the
// designed API; the C++ side compiles the UNMODIFIED oracle TUs.
phase('Harness')
const HARNESS_SCHEMA = {
  type: 'object',
  properties: {
    dir: { type: 'string' }, goldensOk: { type: 'boolean' },
    fixtures: { type: 'number' },
    parityTests: { type: 'array', items: { type: 'string' } },
    gaps: { type: 'array', items: { type: 'string' } },
  },
  required: ['dir', 'goldensOk', 'fixtures', 'parityTests', 'gaps'],
}
const harness = await agent(
`Build the differential-oracle harness tools/${S}-oracle for subsystem "${S}"
(repo root cwd), copying the tools/gp2-oracle pattern file-for-file: run.sh
copies the UNMODIFIED oracle TUs (MP ${JSON.stringify(A.mpOracle)} SP ${JSON.stringify(A.spOracle)})
into build/ next to stub headers in stubs/ so relative #includes resolve to
the stubs (grow stubs until each TU compiles standalone; NEVER edit oracle/);
main.cpp dumps canonical behavior over fixtures/*.gp2-style inputs; goldens
are committed under golden/ so cargo test needs no C++ toolchain; README.md
records scope, dump format, and normalizations.
The dump must exercise the consumer-called API surface: ${JSON.stringify(scout.consumerApi)}.
Design doc (the Rust API your parity tests must call): ${design.designPath}.
Fixtures: cover normal shapes AND edge semantics (truncation, size limits,
per-mode divergences: ${JSON.stringify(design.divergences)}); keep Raven-UB
inputs OUT of shared fixtures or normalize them in the dumper with a comment
(§F19). If parts of the subsystem cannot run standalone (engine/FS/renderer
deps), stub the minimum, shrink scope honestly, and record every uncovered
area under gaps — no silent coverage claims.
Then write the Rust parity tests (tests/${S}_parity.rs in ${A.mpCrate}
and/or ${A.spCrate}, mirroring the dumper byte-for-byte like gp2_parity.rs).
They may not compile until the skeleton lands — that is expected; make the
C++ side build and regenerate goldens (sh tools/${S}-oracle/run.sh --regen),
then verify run.sh (diff mode) passes.
${RULES}
Return the structured result.`,
  { label: `harness:${S}`, phase: 'Harness', effort: 'high', schema: HARNESS_SCHEMA })
if (!harness || !harness.goldensOk) throw new Error('harness failed to produce goldens')
log(`harness: ${harness.fixtures} fixtures, gaps: ${harness.gaps.length}`)

// --------------------------------------------------------------- Skeleton
// The C++-track analog of ABI-sized placeholders: every designed pub type
// and pub signature exists and compiles, bodies are todo!("Port <Class>::<m>
// — <cite>"). After this, porters never collide and cannot drift the shape.
phase('Skeleton')
await agent(
`Create the frozen skeleton for subsystem "${S}" exactly as the design doc
${design.designPath} pins it (repo root cwd). For every planned file:
full pub type definitions, every pub method with its exact designed signature
and a todo!("Port <RavenClass>::<method> — <oracle cite>") body (loud stubs,
porting-rules markers section), house doc comments with cites, mod.rs/lib.rs
fully wired (match sibling mod.rs style). Files:
${design.files.map(f => `- [${f.mode}] ${f.path} (${f.crate}) <- ${f.class}`).join('\n')}
${RULES}
End with cargo check -p ${A.mpCrate} -p ${A.spCrate} GREEN, zero new warnings
(unused-variable warnings from todo!() bodies must be silenced by using _
parameter names ONLY where the design doc says so — otherwise keep designed
names and add nothing; todo!() bodies do not warn).`,
  { label: `skeleton:${S}`, phase: 'Skeleton', model: 'sonnet' })

// ------------------------------------------------------------------- Port
// One porter per class file. MP porters run in parallel; each SP twin chains
// after its MP counterpart (SP transcribes as a diff against the MP port).
phase('Port')
const PORT_SCHEMA = {
  type: 'object',
  properties: {
    ported: { type: 'array', items: { type: 'string' } },
    divergenceNotes: { type: 'array', items: { type: 'string' } },
    problems: { type: 'array', items: { type: 'string' } },
  },
  required: ['ported', 'divergenceNotes', 'problems'],
}
const portPrompt = (f, mpTwin) => `Port Raven class ${f.class} of subsystem
"${S}" into ${f.path} (crate ${f.crate}) by filling the skeleton's todo!()
bodies. Mode: ${f.mode.toUpperCase()}. Oracle sources: ${JSON.stringify(f.mode === 'mp' ? A.mpOracle : A.spOracle)}.
Design doc (frozen): ${design.designPath}.${mpTwin ? `
SP pass: ${mpTwin} holds the finished MP twin — use it as the baseline and
port the SP DIFF from the SP oracle source (divergences are real; keep them).` : ''}
${RULES}
End state: cargo test -p ${f.crate} --lib GREEN (zero new warnings), your
inline tests pass. Return the structured result only.`
const spByClass = new Map(spFiles.map(f => [f.class, f]))
const spOnly = spFiles.filter(f => !mpFiles.some(m => m.class === f.class))
const portResults = []
await parallel([
  ...mpFiles.map(f => async () => {
    const eff = HARD.has(f.class) ? 'high' : undefined
    const r = await agent(portPrompt(f, null),
      { label: `mp:${f.class}`, phase: 'Port', model: 'sonnet', effort: eff, schema: PORT_SCHEMA })
    if (r) portResults.push({ ...r, file: f.path })
    const twin = spByClass.get(f.class)
    if (twin) {
      const r2 = await agent(portPrompt(twin, f.path),
        { label: `sp:${twin.class}`, phase: 'Port', model: 'sonnet', effort: eff, schema: PORT_SCHEMA })
      if (r2) portResults.push({ ...r2, file: twin.path })
    }
  }),
  ...spOnly.map(f => () => agent(portPrompt(f, null),
    { label: `sp:${f.class}`, phase: 'Port', model: 'sonnet',
      effort: HARD.has(f.class) ? 'high' : undefined, schema: PORT_SCHEMA })
    .then(r => { if (r) portResults.push({ ...r, file: f.path }) })),
])
const problems = portResults.flatMap(r => r.problems)
log(`port done: ${portResults.length} files, ${problems.length} reported problems`)

// ----------------------------------------------------------------- Verify
phase('Verify')
const VERIFY_SCHEMA = {
  type: 'object',
  properties: {
    cargoGreen: { type: 'boolean' },
    parityGreen: { type: 'boolean' },
    failures: { type: 'array', items: { type: 'object', properties: {
      area: { type: 'string' }, detail: { type: 'string' } }, required: ['area', 'detail'] } },
  },
  required: ['cargoGreen', 'parityGreen', 'failures'],
}
let verdict = null
for (let round = 0; round < 3; round++) {
  verdict = await agent(
`Machine-verify the "${S}" C++-track port (repo root cwd, read/run only —
fix nothing yourself).
1. cargo build --workspace: must be green with ZERO new warnings.
2. cargo test -p ${A.mpCrate} -p ${A.spCrate}: lib tests AND the ${S}_parity
   golden tests must pass.
3. sh tools/${S}-oracle/run.sh (diff mode): C++ oracle output still matches
   the committed goldens.
4. grep the new files for leftover todo!() bodies and for pub signatures that
   drifted from the design doc ${design.designPath}.
Porter-reported problems to adjudicate: ${JSON.stringify(problems)}
Report every failure verbatim in the structured result.`,
    { label: `verify#${round + 1}`, phase: 'Verify', model: 'sonnet', schema: VERIFY_SCHEMA })
  if (!verdict || (verdict.cargoGreen && verdict.parityGreen && !verdict.failures.length) || round === 2) break
  log(`verify round ${round + 1}: ${verdict.failures.length} failures — fixing`)
  await agent(
`Fix these "${S}" C++-track verification failures (repo root cwd).
${RULES}
Design doc: ${design.designPath}. Parity goldens are GROUND TRUTH: when Rust
output diverges from a golden, the Rust port is wrong (re-read the cited
oracle lines) — NEVER regenerate or edit goldens/fixtures to make a diff pass.
A genuine design-doc error may be corrected, but update the doc AND every
affected file coherently, and say so.
Failures:
${verdict.failures.map(f => `- [${f.area}] ${f.detail}`).join('\n')}
End with cargo build --workspace green (zero new warnings) and both parity
tests passing.`,
    { label: `fix#${round + 1}`, phase: 'Verify', effort: 'high' })
}

// ------------------------------------------------------------------- Docs
if (!A.skipDocs) {
  phase('Docs')
  await agent(
`Update docs/type-port-todo.md's "C++ track — idiomatic reimplementations"
table for subsystem "${S}" (repo root cwd): add/complete its row in the GP2
row's exact style (classes, MP/SP module paths, harness dir, verification
note incl. fixture count and any harness gaps: ${JSON.stringify(harness.gaps)}).
If this subsystem appears in a wave deferral table above, add a "-> done,
C++ track" pointer there. Only edit that one doc. Do not commit.`,
    { label: 'docs', phase: 'Docs', model: 'sonnet' })
}

return {
  subsystem: S,
  designPath: design.designPath,
  harness: { dir: harness.dir, fixtures: harness.fixtures, gaps: harness.gaps },
  files: design.files.map(f => f.path),
  problems,
  verify: verdict,
}
