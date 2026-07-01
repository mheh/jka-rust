export const meta = {
  name: 'port-wave',
  description: 'Port one wave of Raven C types (MP first, then SP as diff) with machine-verified layouts',
  whenToUse: 'Batch type-porting per docs/type-port-plan.md waves. args = {mpModule, spModule, mpCrate, spCrate, headers, onlyTypes?}',
  phases: [
    { title: 'Scout', detail: 'inventory + classify types per header' },
    { title: 'Skeleton', detail: 'pre-wire folders/lib.rs so porters never collide' },
    { title: 'Port MP', detail: 'parallel Sonnet batches, one folder each' },
    { title: 'Port SP', detail: 'SP as a diff against the landed MP port' },
    { title: 'Heavy', detail: 'layout-critical structs, serial, high effort' },
    { title: 'Verify', detail: 'closure.py badge sweep + cargo, fixer rounds' },
    { title: 'Docs', detail: 'update docs/type-port-todo.md' },
  ],
}

// args example (Wave 2):
// { mpModule: 'mp-bg', spModule: 'sp-bg', mpCrate: 'mp_bg', spCrate: 'sp_bg',
//   headers: ['game/bg_public.h', 'game/bg_vehicles.h'],
//   onlyTypes: ['team_t', ...] }   // optional smoke-run filter
const A = (typeof args === 'string' ? JSON.parse(args) : args) || {}
if (!A.mpModule || !A.spModule || !Array.isArray(A.headers) || !A.headers.length) {
  throw new Error('args must include mpModule, spModule, mpCrate, spCrate, headers[]')
}

const TOOL = 'tools/closure-prototype/.venv/bin/python tools/closure-prototype/closure.py'

const RULES = `HOUSE RULES (binding — docs/porting-rules.md):
- NEVER edit anything under oracle/oracle/.
- Enum-vs-alias fidelity: "typedef enum {...} X" -> #[repr(i32)] pub enum X;
  "typedef int X" + separate anon enum -> pub type X = c_int + pub const items.
  NEVER flatten a named enum to an int alias.
- One type per file; file/folder mirrors the owning Raven header's subsystem.
- Every item gets a doc comment + source ref, exactly this shape:
    /// Raven \`X\` — <one-line description>.
    ///
    /// Raven: <original Raven comment, if any>.
    /// Type definition source: \`oracle/oracle/codemp/game/bg_public.h:NNN-MMM\`
- ABI-crossing structs: #[repr(C)], exact Raven field names/order, plus
  size_of (always) and offset_of! asserts generated from the tool:
    ${TOOL} <module> <Type> --asserts
  Gate pointer-width-dependent asserts with #[cfg(target_pointer_width = "64")].
- Unported deps: //TODO: Port <RavenIdent> + // Source: <oracle path:line>.
  Pointer-only deps may stay opaque (*mut c_void or a fwd-declared opaque type).
- State the conclusion, not the derivation; no re-explaining C mechanics.
- rust-analyzer is stale: trust ONLY cargo check.`

// ---------------------------------------------------------------- Scout
phase('Scout')
const SCOUT_SCHEMA = {
  type: 'object',
  properties: {
    types: { type: 'array', items: { type: 'object', properties: {
      name: { type: 'string' },
      kind: { enum: ['enum', 'struct', 'union', 'alias', 'fnptr', 'const'] },
      tier: { enum: ['trivial', 'medium', 'heavy'] },
      cite: { type: 'string' },
      sizeB: { type: 'number' },
      status: { enum: ['unported', 'ported', 'skip'] },
      skipReason: { type: 'string' },
      folder: { type: 'string' },
      spDiverges: { type: 'boolean' },
      spCite: { type: 'string' },
    }, required: ['name', 'kind', 'tier', 'cite', 'status', 'folder', 'spDiverges'] } },
  },
  required: ['types'],
}

const scouted = await parallel(A.headers.map(h => () => agent(
`Scout oracle header codemp/${h} for the jka-rust type port (repo root cwd).
Goal: a complete inventory of the types THIS header owns, ready for porting
into crates/${A.mpModule.replace('-', '/')}/src/.

1. List candidate types from docs/oracle-types.md (grep the "${h}" rows) and
   by reading the header itself: oracle/oracle/codemp/${h}.
2. For each type, check current status with the verified badge:
     ${TOOL} ${A.mpModule} <TypeName>
   ☑ -> status "ported". ☐/◐ -> "unported". Vendored/C++-class/platform-glue
   types -> "skip" with skipReason (see docs/type-port-plan.md exclusions).
3. Classify tier: trivial = alias/small enum/fn-ptr sig/const; heavy =
   layout-critical struct that crosses the ABI or is >200B or pointer-bearing
   with many fields; medium = everything else.
4. Pick the target folder inside crates/${A.mpModule.replace('-', '/')}/src/
   consistent with the existing layout there (look at it first).
5. Check SP divergence: does oracle/oracle/code/game/<equiv header> define it,
   and does the definition differ (fields/values/size)? Set spDiverges and
   spCite (SP file:line) accordingly. Absent in SP -> spDiverges true,
   spCite "" and note it in skipReason-style suffix of cite? No — keep spCite "".
6. cite = "oracle/oracle/codemp/${h}:START-END".
Return ONLY the structured result.`,
  { label: `scout:${h.split('/').pop()}`, phase: 'Scout', model: 'sonnet', schema: SCOUT_SCHEMA }
)))

let types = scouted.filter(Boolean).flatMap(s => s.types)
  .filter(t => t.status === 'unported')
if (Array.isArray(A.onlyTypes) && A.onlyTypes.length) {
  types = types.filter(t => A.onlyTypes.includes(t.name))
}
const skipped = scouted.filter(Boolean).flatMap(s => s.types).filter(t => t.status === 'skip')
log(`scouted: ${types.length} to port (${skipped.length} skipped, filter=${A.onlyTypes ? A.onlyTypes.length : 'none'})`)
if (!types.length) return { ported: [], note: 'nothing unported after scout/filter', skipped }

// ------------------------------------------------------------ Batching (deterministic)
const heavies = types.filter(t => t.tier === 'heavy')
const light = types.filter(t => t.tier !== 'heavy')
const byFolder = {}
for (const t of light) (byFolder[t.folder] = byFolder[t.folder] || []).push(t)
const batches = []
for (const folder of Object.keys(byFolder)) {
  const list = byFolder[folder]
  for (let i = 0; i < list.length; i += 8) batches.push({ folder, types: list.slice(i, i + 8) })
}
// one agent per folder at a time would still collide if two batches share a
// folder — merge same-folder batches into a serial chain per folder instead.
const perFolder = Object.keys(byFolder).map(folder => ({
  folder, chunks: batches.filter(b => b.folder === folder).map(b => b.types),
}))
log(`batching: ${light.length} light types across ${perFolder.length} folders, ${heavies.length} heavy serial`)

// ---------------------------------------------------------------- Skeleton
phase('Skeleton')
const allFolders = [...new Set(types.map(t => t.folder))]
await agent(
`Prepare crate skeletons so parallel porters never touch shared files
(repo root cwd). ${RULES}

In crates/${A.mpModule.replace('-', '/')}/src/ AND crates/${A.spModule.replace('-', '/')}/src/:
for each of these folders: ${JSON.stringify(allFolders)}
- ensure the folder exists with a mod.rs (create empty mod.rs if new),
- ensure it is registered up the module tree to lib.rs (pub mod ...),
- do NOT create any type files.
Finish with cargo check -p ${A.mpCrate} and cargo check -p ${A.spCrate} GREEN.
If a folder already exists and is wired, leave it alone.`,
  { label: 'skeleton', phase: 'Skeleton', model: 'sonnet' }
)

// ---------------------------------------------------------------- Port helpers
const PORT_SCHEMA = {
  type: 'object',
  properties: {
    ported: { type: 'array', items: { type: 'object', properties: {
      name: { type: 'string' }, file: { type: 'string' } }, required: ['name', 'file'] } },
    deferred: { type: 'array', items: { type: 'object', properties: {
      subject: { type: 'string' }, source: { type: 'string' } }, required: ['subject', 'source'] } },
    problems: { type: 'array', items: { type: 'string' } },
  },
  required: ['ported', 'deferred', 'problems'],
}

function portPrompt(mode, module, crate, folder, list, mpFiles) {
  const spDiff = mode === 'SP' ? `
This is the SP pass: port from the SP oracle (oracle/oracle/code/), using the
already-landed MP port as the diff baseline — read the MP file listed per type
and adapt to SP's actual definition (spCite). SP frequently diverges (different
fields/sizes/enum values); port what SP says, never copy MP blindly.` : ''
  return `Port these Raven types into the jka-rust ${mode} tree (repo root cwd).
${RULES}
${spDiff}
Target: crates/${module.replace('-', '/')}/src/${folder}/ (crate ${crate}).
Types (name | tier | oracle cite${mode === 'SP' ? ' | SP cite | MP baseline file' : ''}):
${list.map(t => `- ${t.name} | ${t.tier} | ${t.cite}${mode === 'SP' ? ` | ${t.spCite || 'CHECK SP ORACLE'} | ${(mpFiles && mpFiles[t.name]) || 'n/a'}` : ''}`).join('\n')}

Per type:
1. Read the oracle definition at the cite. For structs, get ground truth:
     ${TOOL} ${module} <Type> --layout   and   --asserts
2. One file per type in the target folder, house doc comment + source ref,
   asserts pasted for #[repr(C)] structs. Register in the folder's mod.rs.
   Touch ONLY files inside your target folder (mod.rs included) — lib.rs and
   parent modules are already wired.
3. Unported by-value deps: if trivial, port them too (same rules, same folder
   unless the header says otherwise); if not portable now, use the //TODO: Port
   marker pattern and record it under "deferred".
4. End state: cargo check -p ${crate} GREEN. Verify each struct's badge:
     ${TOOL} ${module} <Type>   must show ☑ for every struct you ported.
Return ONLY the structured result; final message is data, not prose.`
}

// ---------------------------------------------------------------- Port MP
phase('Port MP')
const mpResults = await parallel(perFolder.map(({ folder, chunks }) => async () => {
  const out = []
  for (const chunk of chunks) { // serial per folder (mod.rs safety), folders in parallel
    const r = await agent(
      portPrompt('MP', A.mpModule, A.mpCrate, folder, chunk, null),
      { label: `mp:${folder}`, phase: 'Port MP', model: 'sonnet', schema: PORT_SCHEMA })
    if (r) out.push(r)
  }
  return out
}))
const mpPorted = mpResults.filter(Boolean).flat().flatMap(r => r.ported)
const mpFiles = Object.fromEntries(mpPorted.map(p => [p.name, p.file]))
log(`MP: ${mpPorted.length} light types ported`)

// ---------------------------------------------------------------- Port SP
phase('Port SP')
const spTargets = light.filter(t => t.spCite || t.spDiverges !== undefined)
const spByFolder = {}
for (const t of spTargets) (spByFolder[t.folder] = spByFolder[t.folder] || []).push(t)
const spResults = await parallel(Object.entries(spByFolder).map(([folder, list]) => async () => {
  const out = []
  for (let i = 0; i < list.length; i += 8) {
    const r = await agent(
      portPrompt('SP', A.spModule, A.spCrate, folder, list.slice(i, i + 8), mpFiles),
      { label: `sp:${folder}`, phase: 'Port SP', model: 'sonnet', schema: PORT_SCHEMA })
    if (r) out.push(r)
  }
  return out
}))
const spPorted = spResults.filter(Boolean).flat().flatMap(r => r.ported)
log(`SP: ${spPorted.length} light types ported`)

// ---------------------------------------------------------------- Heavy (serial)
phase('Heavy')
const heavyPorted = []
for (const t of heavies) {
  for (const [mode, module, crate] of [['MP', A.mpModule, A.mpCrate], ['SP', A.spModule, A.spCrate]]) {
    if (mode === 'SP' && !t.spCite && !t.spDiverges) continue
    const r = await agent(
      portPrompt(mode, module, crate, t.folder, [t], mode === 'SP' ? mpFiles : null) +
      `\nThis is a HEAVY layout-critical struct: full offset_of! asserts for every
field boundary the tool reports; do not rush; if any by-value dep is itself
heavy, STOP and record it under deferred instead of porting it inline.`,
      { label: `heavy-${mode.toLowerCase()}:${t.name}`, phase: 'Heavy',
        model: 'sonnet', effort: 'high', schema: PORT_SCHEMA })
    if (r) heavyPorted.push(...r.ported.map(p => ({ ...p, mode })))
    if (r && mode === 'MP') r.ported.forEach(p => { mpFiles[p.name] = p.file })
  }
}
log(`heavy: ${heavyPorted.length} ports done`)

// ---------------------------------------------------------------- Verify
phase('Verify')
const claimed = {
  [A.mpModule]: [...new Set([...mpPorted.map(p => p.name), ...heavyPorted.filter(h => h.mode === 'MP').map(h => h.name)])],
  [A.spModule]: [...new Set([...spPorted.map(p => p.name), ...heavyPorted.filter(h => h.mode === 'SP').map(h => h.name)])],
}
const VERIFY_SCHEMA = {
  type: 'object',
  properties: {
    cargoGreen: { type: 'boolean' },
    failures: { type: 'array', items: { type: 'object', properties: {
      module: { type: 'string' }, name: { type: 'string' }, detail: { type: 'string' } },
      required: ['module', 'name', 'detail'] } },
  },
  required: ['cargoGreen', 'failures'],
}

let verdict = null
for (let round = 0; round < 3; round++) {
  verdict = await agent(
`Machine-verify the wave port (repo root cwd).
1. cargo check --workspace — report failures verbatim.
2. For every type below, run the verified badge and require ☑ (a documented
   //TODO deferral in the file is acceptable ONLY for non-struct aliases):
${Object.entries(claimed).map(([m, names]) => names.map(n => `   ${TOOL} ${m} ${n}`).join('\n')).join('\n')}
Badge meanings: ☑ verified; "NO SIZE ASSERT"/"SIZE MISMATCH"/☐ are FAILURES for
structs. Return the structured result only.`,
    { label: `verify#${round + 1}`, phase: 'Verify', model: 'sonnet', schema: VERIFY_SCHEMA })
  if (!verdict || (verdict.cargoGreen && verdict.failures.length === 0)) break
  if (round === 2) break
  log(`verify round ${round + 1}: ${verdict.failures.length} failures — fixing`)
  await agent(
`Fix these verification failures in the jka-rust wave port (repo root cwd).
${RULES}
Failures:
${verdict.failures.map(f => `- [${f.module}] ${f.name}: ${f.detail}`).join('\n')}
${verdict.cargoGreen ? '' : 'cargo check --workspace is also failing — fix compile errors first.'}
Use ${TOOL} <module> <Type> --layout / --asserts for ground truth. If a
mismatch traces to a genuine oracle divergence you cannot resolve, remove the
bad assert, add a //TODO: Port marker, and note it. End with cargo check
--workspace GREEN.`,
    { label: `fix#${round + 1}`, phase: 'Verify', model: 'sonnet' })
}

// ---------------------------------------------------------------- Docs
phase('Docs')
const docs = await agent(
`Update ${A.todoDoc || 'docs/type-port-todo.md'} for the wave that just landed
(repo root cwd). Match the existing table format exactly (☐/◐/☑ status marks,
oracle cites, divergence notes for SP). Newly ported:
MP (${A.mpModule}): ${claimed[A.mpModule].join(', ') || '(none)'}
SP (${A.spModule}): ${claimed[A.spModule].join(', ') || '(none)'}
Deferred/skipped from scout: ${skipped.map(s => s.name).join(', ') || '(none)'}
Do not commit. Only edit that one doc.`,
  { label: 'docs', phase: 'Docs', model: 'sonnet' })

return {
  scouted: types.length,
  mpPorted: claimed[A.mpModule],
  spPorted: claimed[A.spModule],
  heavyCount: heavies.length,
  skipped: skipped.map(s => ({ name: s.name, reason: s.skipReason })),
  verify: verdict,
  docs,
}
