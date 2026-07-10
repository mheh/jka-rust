export const meta = {
  name: 'port-f-lane',
  description: 'Generic §F subsystem lane: doc-driven skeleton (per-class, completeness-gated) -> parallel porters -> golden verify',
  phases: [
    { title: 'Load', detail: 'extract the FROZEN doc roster' },
    { title: 'Harness', detail: 'build tools/<s>-oracle + goldens (only when missing)' },
    { title: 'Skeleton', detail: 'shared aggregate + parallel per-class skeletons + completeness gate' },
    { title: 'Port', detail: 'parallel per-class porters + parity-test writer' },
    { title: 'Verify', detail: 'lane-scoped cargo + goldens, bounded fix rounds' },
  ],
}

const A = (typeof args === 'string' ? JSON.parse(args) : args) || {}
// args: { subsystem, doc, crates: [pkg...], needHarness?: bool, harnessDir?: str, hardClasses?: [name...] }
for (const k of ['subsystem', 'doc', 'crates']) if (A[k] === undefined) throw new Error(`args.${k} required`)
const S = A.subsystem
const REPO = '/Users/milohehmsoth/Developer/Milo/jka-rust'
const DOC = A.doc
const CRATES = A.crates
const HARD = new Set(A.hardClasses || [])
const CHECK = CRATES.map(c => `-p ${c}`).join(' ')

const STYLE = `HOUSE RULES (binding): NEVER edit anything under oracle/. NEVER git commit. NEVER add a co-author trailer. Doc-comment + Source cite on every item. Preserve Raven comments that clarify behavior. Trust ONLY cargo (rust-analyzer is stale). Do not read .claude/workflows/. NEVER run cargo build/check/test --workspace — other subsystem lanes are mid-port in this tree; scope every cargo command to ${CHECK}.`
const F_RULES = `C++ TRACK RULES (porting-rules §F 17-21): idiomatic reimplementation against the FROZEN doc ${DOC} — its pinned signatures/shapes are LAW; a genuinely wrong signature is REPORTED under problems (with the oracle cite proving it), never improvised around. Divergences only at the doc's enumerated §19 sites, <=2-line note each. No static mut; state lives on the doc's owner structs. Naming: bare Hungarian C prefix drops (ruling 40); ABI-frozen names keep exact Raven names. Only touch your assigned files. NO todo!()/TODO/FIXME may remain in YOUR files when you finish.`

phase('Load')
const LOAD_SCHEMA = { type: 'object', properties: {
  units: { type: 'array', items: { type: 'object', properties: {
    files: { type: 'array', items: { type: 'string' } }, class: { type: 'string' },
    crate: { type: 'string' }, fns: { type: 'number' } }, required: ['files', 'class', 'crate', 'fns'] } },
  sharedFiles: { type: 'array', items: { type: 'string' } },
  harnessDir: { type: 'string' },
}, required: ['units', 'sharedFiles'] }
const plan = await agent(
`Read the FROZEN design doc ${DOC} (repo root ${REPO}; read-only). From its Files roster, return the port plan:
- units: one entry per CLASS-BEARING roster file group (path(s) incl. small helper-type files that belong with the class, the Raven class name, the owning crate package name, and the doc's approximate method/fn count for that class — read the roster summaries and the Raven-ground-truth/scope sections for the counts; be accurate, the skeleton completeness gate uses them).
- sharedFiles: lib.rs / mod.rs / aggregate-struct files the roster pins that are NOT per-class port units (module decls, the fork-2 aggregate owner struct, Engine-attachment work named in Slice hooks).
- harnessDir: the tools/<x>-oracle dir the Verification strategy names (empty string if none exists yet).
No prose.`,
  { label: `load:${S}`, phase: 'Load', model: 'sonnet', schema: LOAD_SCHEMA })
if (!plan || !plan.units.length) throw new Error('roster load failed')
log(`${S}: ${plan.units.length} port units, ${plan.units.reduce((a, u) => a + u.fns, 0)} fns`)

const HDIR = A.harnessDir || plan.harnessDir || `tools/${S}-oracle`
if (A.needHarness) {
  phase('Harness')
  const h = await agent(
`Build the §18 differential-oracle harness ${HDIR} for subsystem "${S}" (repo root ${REPO}). Copy the established pattern (tools/icarus-oracle, tools/trmodel-oracle, tools/gp2-oracle: build.sh/run.sh copying UNMODIFIED oracle TUs beside stub headers; dumper main; hand-authored fixtures — NO retail assets committed; committed goldens; README with scope/format/normalizations; g++-16, -fsigned-char -ffp-contract=off -fno-fast-math; run-twice byte-identical). The doc ${DOC} § Verification strategy pins the units/dump formats/fixture plan — follow it exactly. Reuse existing fixture generators where the doc says so (tools/trmodel-oracle's modelgen .glm/.gla generator is available). If part of the surface cannot run standalone, stub minimally and record every uncovered area — no silent coverage claims.
${STYLE}
Return JSON {goldensOk: bool, fixtures: n, gaps: [...]}.`,
    { label: `harness:${S}`, phase: 'Harness', model: 'opus', effort: 'high', schema: { type: 'object', properties: { goldensOk: { type: 'boolean' }, fixtures: { type: 'number' }, gaps: { type: 'array', items: { type: 'string' } } }, required: ['goldensOk', 'gaps'] } })
  if (!h || !h.goldensOk) throw new Error(`harness failed for ${S}: ` + JSON.stringify(h && h.gaps))
  log(`harness: ${h.fixtures} fixtures, ${h.gaps.length} gaps`)
}

phase('Skeleton')
const SKEL_SCHEMA = { type: 'object', properties: { green: { type: 'boolean' }, problems: { type: 'array', items: { type: 'string' } } }, required: ['green', 'problems'] }
const shared = await agent(
`SHARED SKELETON for §F subsystem "${S}" (repo root ${REPO}, branch skeleton). From the FROZEN doc ${DOC}: create/reshape ONLY the shared roster files — ${plan.sharedFiles.join(', ')} — the aggregate owner struct with its pinned fields + Default/construction story, every mod decl the roster requires, Cargo.toml deps (mp_host_interface path dep authorized by ruling 56c; any doc-named crate edges), Engine-attachment (the mp_engine_core::Engine field the doc's Slice hooks name): SKIP IT — several lanes run concurrently and the orchestrator wires all Engine fields at merge time; just record the required field name/type in your problems return. Do NOT touch crates/mp/engine/core at all. Do NOT create the per-class files (parallel agents own those) — but your mod decls may reference them; that is fine, they land within minutes.
${F_RULES}
${STYLE}
todo!() bodies are permitted as uncommitted intermediate state only. END: your files parse (rustfmt --edition 2021 --emit stdout <file> >/dev/null). cargo will not be green until per-class skeletons land — do not fight that; just ensure YOUR files are correct.
Return JSON {green: true, problems: [...]}.`,
  { label: `skel-shared:${S}`, phase: 'Skeleton', model: 'opus', effort: 'high', schema: SKEL_SCHEMA })
await parallel(plan.units.map(u => () => agent(
`PER-CLASS SKELETON for Raven class ${u.class} of §F subsystem "${S}" (repo root ${REPO}, branch skeleton). Create/reshape ONLY these files: ${u.files.join(', ')} per the FROZEN doc ${DOC} (its roster row for your class + State ownership + Seam definition + Decisions pin every type shape and method signature). EVERY method the doc says ports for this class (~${u.fns} fns; read the doc AND the oracle class definition it cites to enumerate them ALL — private helpers included; §20-dropped methods get module-doc notes, NOT stubs) gets its exact pinned signature with a todo!("Port ${u.class}::<method> — <oracle cite>") body. COMPLETENESS IS THE POINT: a later machine gate counts your stubs against the doc; a missing method blocks every porter that calls it.
${F_RULES}
${STYLE}
END: your files parse via rustfmt (rustfmt --edition 2021 --emit stdout <file> >/dev/null); do NOT run cargo (siblings are landing in parallel).
Return JSON {green: true, problems: ["<any doc/oracle mismatch found while enumerating>"]}.`,
  { label: `skel:${u.class}`, phase: 'Skeleton', model: 'sonnet', schema: SKEL_SCHEMA })))

// Completeness gate — the calibration lesson. One repair round allowed.
let gate = null
for (let round = 1; round <= 2; round++) {
  gate = await agent(
`SKELETON COMPLETENESS GATE for §F subsystem "${S}" (repo root ${REPO}; fix NOTHING). For each unit below, count the fn items (stub or real) in its files and compare against the doc ${DOC}'s method inventory for that class (read the doc AND the cited oracle header to enumerate expected methods; §20-dropped ones are expected ABSENT):
${plan.units.map(u => `- ${u.class}: ${u.files.join(', ')} (doc ~${u.fns} fns)`).join('\n')}
Also: cargo check ${CHECK} 2>&1 — must compile (todo!() bodies are fine at this stage).
Return JSON {ok: bool, gaps: [{class, missing: ["<Class::method> — <cite>"], detail}], compileErrors: n}.`,
    { label: `gate#${round}:${S}`, phase: 'Skeleton', model: 'sonnet', schema: { type: 'object', properties: { ok: { type: 'boolean' }, gaps: { type: 'array', items: { type: 'object', properties: { class: { type: 'string' }, missing: { type: 'array', items: { type: 'string' } }, detail: { type: 'string' } }, required: ['class'] } }, compileErrors: { type: 'number' } }, required: ['ok', 'gaps'] } })
  if (!gate || gate.ok || round === 2) break
  log(`gate round ${round}: ${gate.gaps.length} gaps — repairing`)
  await agent(
`SKELETON REPAIR for §F subsystem "${S}" (repo root ${REPO}). The completeness gate found these missing method stubs / compile errors — materialize every one with its exact doc-pinned signature and todo!() body (and fix skeleton-level compile errors):
${JSON.stringify(gate.gaps).slice(0, 6000)}
${F_RULES}
${STYLE}
END: cargo check ${CHECK} green. Return JSON {green: true, problems: [...]}.`,
    { label: `skel-repair:${S}`, phase: 'Skeleton', model: 'opus', effort: 'high', schema: SKEL_SCHEMA })
}
if (!gate || (!gate.ok && (gate.gaps || []).length)) log(`WARNING: gate not fully clean after repair — porting anyway, verify owns the residue`)

phase('Port')
const PORT_SCHEMA = { type: 'object', properties: {
  ported: { type: 'array', items: { type: 'string' } },
  divergenceNotes: { type: 'array', items: { type: 'string' } },
  problems: { type: 'array', items: { type: 'string' } } }, required: ['ported', 'divergenceNotes', 'problems'] }
const portResults = []
const porterJobs = plan.units.map(u => () => agent(
`Port Raven class ${u.class} (§F subsystem "${S}") by filling the skeleton's todo!() bodies in YOUR FILES ONLY: ${u.files.join(', ')}. Repo root ${REPO}, branch skeleton.
FROZEN doc: ${DOC} (roster row, State ownership, Seam definition, Decisions, Divergences). Transcribe from the oracle sources the doc cites, control-flow-faithful. The EngineHost seam is crates/mp/host-interface/src/engine_host.rs. Every todo!() in your files gets a real body; §19 divergences exactly as the doc states. Sibling classes exist as complete gated skeletons — call them per their real signatures; a genuinely missing sibling method is a problems report, never an invention. Inline #[cfg(test)] tests for tricky semantics (do not call siblings whose bodies may still be todo!()).
${F_RULES}
${STYLE}
END: rustfmt parse gate on your files, then cargo check ${CHECK} and fix errors IN YOUR FILES (siblings' todo bodies type-check; their in-flight edits may transiently break a check — retry once, then report).
Return JSON {ported: [...], divergenceNotes: [...], problems: [...]}.`,
  { label: `port:${u.class}`, phase: 'Port', model: (HARD.has(u.class) || u.fns > 25) ? 'opus' : 'sonnet', effort: (HARD.has(u.class) || u.fns > 40) ? 'high' : undefined, schema: PORT_SCHEMA })
  .then(r => { if (r) portResults.push({ ...r, unit: u.class }) }))
porterJobs.push(() => agent(
`Write the golden PARITY TESTS for §F subsystem "${S}": a tests/<s>_parity.rs in the primary crate ${CRATES[0]}. Repo root ${REPO}, branch skeleton.
GROUND TRUTH: ${HDIR}/ (read its README, dumpers, fixtures/, goldens/). Tests must reproduce each committed golden BYTE-FOR-BYTE through the ported Rust API, mirroring the dumper formats exactly (exemplars: crates/mp/engine/icarus/tests/icarus_parity.rs, crates/mp/engine/qcommon/tests/gp2_parity.rs). Use the fixture-backed MockHost (crates/mp/host-interface/src/mock.rs) for host-taking surfaces; extend MockHost minimally in its own style only if a fixture knob is genuinely missing (report it). The skeleton API is FROZEN — code against it; bodies are landing in parallel, so your tests only need to COMPILE now (cargo test ${CHECK.split(' ').slice(0, 2).join(' ')} --no-run). Never weaken assertions; never touch goldens/fixtures.
${STYLE}
Return JSON {ported: ["<test fn>"], divergenceNotes: [], problems: [...]}.`,
  { label: `parity:${S}`, phase: 'Port', model: 'opus', effort: 'high', schema: PORT_SCHEMA })
  .then(r => { if (r) portResults.push({ ...r, unit: 'parity-tests' }) }))
await parallel(porterJobs)
const problems = portResults.flatMap(r => (r.problems || []).map(p => `[${r.unit}] ${p}`))
log(`port done: ${portResults.length} units, ${problems.length} problems`)

phase('Verify')
const VERIFY_SCHEMA = { type: 'object', properties: {
  cargoGreen: { type: 'boolean' }, parityGreen: { type: 'boolean' }, markersClean: { type: 'boolean' },
  failures: { type: 'array', items: { type: 'object', properties: { area: { type: 'string' }, detail: { type: 'string' } }, required: ['area', 'detail'] } } },
  required: ['cargoGreen', 'parityGreen', 'markersClean', 'failures'] }
let verdict = null
for (let round = 1; round <= 5; round++) {
  verdict = await agent(
`MACHINE-VERIFY the "${S}" §F port (repo root ${REPO}; read/run only — fix NOTHING).
1. cargo build ${CHECK} 2>&1 — green, zero NEW warnings. (NEVER --workspace: other lanes are mid-port.)
2. cargo test ${CHECK} 2>&1 — lib tests AND the parity golden tests pass.
3. grep -rn "todo!\\|TODO\\|FIXME" over the roster's src dirs (from ${DOC}) — ZERO hits (PORT-NOTE allowed).
4. Spot-check ported pub signatures against ${DOC}.
5. Adjudicate porter problems (verify each against doc/oracle): ${JSON.stringify(problems).slice(0, 4000)}
Return failures VERBATIM.`,
    { label: `verify#${round}:${S}`, phase: 'Verify', model: 'sonnet', schema: VERIFY_SCHEMA })
  if (!verdict) continue
  if (verdict.cargoGreen && verdict.parityGreen && verdict.markersClean && !verdict.failures.length) break
  if (round === 5) break
  log(`verify ${round}: ${verdict.failures.length} failures — fixing`)
  await agent(
`FIX these "${S}" §F verification failures (repo root ${REPO}, branch skeleton).
${F_RULES}
${STYLE}
GOLDENS ARE GROUND TRUTH: Rust diverging from ${HDIR}/goldens means the RUST PORT IS WRONG — re-read the cited oracle lines; NEVER regenerate/edit goldens or fixtures. A genuine FROZEN-doc error may be corrected only with the doc AND all affected files updated coherently — say so explicitly in your return (these become ledger entries).
FAILURES:
${verdict.failures.map(f => `- [${f.area}] ${f.detail}`).join('\n').slice(0, 9000)}
END: cargo build ${CHECK} + cargo test ${CHECK} green, marker grep clean.
Return JSON {ported: [...], divergenceNotes: [...], problems: ["<unfixed / doc amendments made>"]}.`,
    { label: `fix#${round}:${S}`, phase: 'Verify', model: 'opus', effort: 'high', schema: PORT_SCHEMA })
}

return {
  subsystem: S,
  units: portResults.map(r => ({ unit: r.unit, ported: (r.ported || []).length })),
  problems,
  divergence_notes: portResults.flatMap(r => (r.divergenceNotes || []).map(d => `[${r.unit}] ${d}`)),
  verdict,
}
