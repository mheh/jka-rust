export const meta = {
  name: 'integrate-engine-cpass',
  description: 'Integrate the engine C-track transcription on branch engine-cpass: drive the workspace from ~1,700+ first-layer compile errors to green via triage -> parallel rounds (state-struct field merges + file-group fixers, cascade-aware delta tripwire) -> serial finisher. Fixers repair call sites, imports, and state-field wiring only; they never change ported logic.',
  whenToUse: 'After the engine mega-pass: skeleton merged into engine-cpass (unified base), six §F lanes golden-green, 1,181 C-track fn bodies landed. Errors are dominated by E0609 (missing Raven-named state fields on receiver structs) and E0425/E0433 (import wiring). Multi-crate cascade: qcommon fails first; server/client/botlib/core surface as it greens.',
  phases: [
    { title: 'Triage', detail: 'one agent runs cargo check --workspace, writes a machine-readable per-file inventory (incl. E0609 receiver-struct bins), and bins erroring files into balanced groups excluding the state-struct definition files' },
    { title: 'Fix rounds', detail: 'up to MAX_ROUNDS rounds; each round runs parallel state-struct field-merge agents (one per receiver struct, they own their struct file exclusively) alongside per-group file fixers, then a serial write-list applier, serial commit, re-inventory; cascade-aware DELTA TRIPWIRE' },
    { title: 'Finisher', detail: 'one serial opus finisher: cm module reconciliation, Engine.bot field, write-list audit, drives cargo check --workspace to green' },
    { title: 'Report', detail: 'per-round totals, failing-crate progression, commits, consolidated blocked list' },
  ],
}

// Config is HARDCODED (workflow-args string bug): relaunch via scriptPath after edits.
const WT = '/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/engine-cpass'
const MAX_ROUNDS = 6
const GROUPS = 10
const FINISHER_THRESHOLD = 150
const INV_DIR = `${WT}/target/integrate`

// State-receiver structs and their definition files (grep-verified 2026-07-10).
// Field-merge agents OWN these files; file-group fixers NEVER edit them.
const STATE_STRUCTS = {
  Common: 'crates/mp/engine/qcommon/src/common/common.rs',
  CollisionWorld: 'crates/mp/engine/qcommon/src/collision_world.rs',
  Server: 'crates/mp/engine/server/src/server_host.rs',
  Client: 'crates/mp/engine/client/src/client_host.rs',
  BotLib: 'crates/mp/engine/botlib/src/lib.rs',
  Icarus: 'crates/mp/engine/icarus/src/lib.rs',
  Navigator: 'crates/mp/engine/server/src/npcnav/navigator.rs',
  Ghoul2System: 'crates/mp/engine/ghoul2/src/ghoul2_system.rs',
  RmManager: 'crates/mp/engine/rmg/src/rm_manager.rs',
  RoffSystem: 'crates/mp/engine/qcommon/src/roff/mod.rs',
  RenderModels: 'crates/mp/renderer/src/tr_model/render_models.rs',
}
const ENGINE_RS = 'crates/mp/engine/core/src/engine.rs' // write-list applier + finisher only
const RESERVED_FILES = Object.values(STATE_STRUCTS).concat([ENGINE_RS])

// ---- the exact inventory command every triage/re-inventory agent runs (deterministic parse) ----
function invCmd(n) {
  return `Run EXACTLY (the cd is mandatory — cargo from any other directory fails instantly and its empty output is a FALSE GREEN):
cd ${WT} && mkdir -p target/integrate && cargo check --workspace --message-format=short 2>&1 | tee target/integrate/raw-r${n}.txt | tail -3
Then run this EXACT python (substituting nothing — it is complete):
cd ${WT} && python3 - <<'PYEOF'
import re, json, collections
raw = open('target/integrate/raw-r${n}.txt').read().splitlines()
pat = re.compile(r'^(.+?\\.rs):\\d+:\\d+:\\s+error(\\[(E\\d+)\\])?:\\s*(.*)$')
files = collections.defaultdict(lambda: {'errors':0,'codes':collections.Counter(),'samples':[]})
e609 = collections.defaultdict(set)
total = 0
for ln in raw:
    m = pat.match(ln)
    if not m: continue
    f, code, msg = m.group(1), m.group(3) or 'other', m.group(4)
    total += 1
    d = files[f]; d['errors'] += 1; d['codes'][code] += 1
    if len(d['samples']) < 4: d['samples'].append(ln[:300])
    if code == 'E0609':
        fm = re.search(r"no field \`(\\w+)\` on (?:type|struct) \`([^\`]+)\`", msg)
        if fm:
            ty = fm.group(2).replace('&mut ','').replace('&','').split('<')[0].split('::')[-1].strip()
            e609[ty].add(fm.group(1))
inv = {'total_errors': total,
 'files': [{'file':f,'errors':d['errors'],'codes':sorted(d['codes']),'samples':d['samples']} for f,d in sorted(files.items(), key=lambda kv:-kv[1]['errors'])],
 'e0609_by_struct': [{'struct':k,'fields':sorted(v),'count':len(v)} for k,v in sorted(e609.items(), key=lambda kv:-len(kv[1]))]}
json.dump(inv, open('target/integrate/inv-r${n}.json','w'), indent=1)
print(total, 'errors across', len(files), 'files')
PYEOF
Your returned \`tail\` must be the last line of the cargo output (from the tee'd tail -3): either "Finished" (green) or the "could not compile"/"previous errors" summary. If it says "did not match any packages" or "no such file", your cwd was wrong — fix and re-run before returning.`
}

// ---- contract assets carried into every fixer prompt ----
const CONTRACT_ASSETS = `CONTRACT ASSETS — READ THESE FIRST (they are LAW; do not re-derive from memory):
- Rosetta Stone: ${WT}/docs/porting/rosetta.md, the "## EXAMPLE SYNTAX" section — canonical mapped shapes for the vec3/q_math macro family, va()/printf, cstr helpers. Use these EXACT shapes at call sites.
- Rulings: ${WT}/docs/handoffs/engine-fork-discovery.md — every ruling is SETTLED; never re-litigate.
- RECEIVER CONVENTION: engine functions thread &mut state receivers in the pinned parameter order (common: &mut Common, cm: &mut CollisionWorld, sv: &mut Server, cl: &mut Client, bot: &mut BotLib, rm/rmg: &mut RmManager, icarus: &mut Icarus, nav: &mut Navigator, g2: &mut Ghoul2System, roff: &mut RoffSystem, host: &mut dyn EngineHost). The transcribed signatures came from a machine fixpoint and are LAW — call sites bend to them. If a fn body references a receiver it lacks (E0425 "cannot find value \`common\`"), thread the missing receiver param through in pinned order AND update every call site — mechanical, allowed.
- STATE FIELDS: bodies write \`receiver.<RavenGlobalName>\` verbatim (common.com_frameTime, cm.NumSubBSP, sv.svs, bot.aasworld). Raven names stay; non_snake_case is allowed crate-wide.
- Com_Error call sites are panics (ruling 1). rand-family calls route through common.qrand (irand/flrand/crandom/Q_irand — ruling 21), NEVER libc rand.
- Missing consts/types: GREP THE WORKSPACE FIRST (crates/mp/qshared, crates/mp/bg, crates/native, sibling engine crates) — most exist; import/re-export rather than re-port. Only port a genuinely-absent item, to its canonical home, house style with Source cite.`

const KNOWN_DEBT = `KNOWN DEBT you may hit:
- qcommon has BOTH a cm/ dir module (§F golden-green: cm::cm_trace etc.) and src-root cm_*.rs C-track files (cm_load, cm_shader, cm_test, cm_trace, ...). Do NOT delete or rewrite either side; wire imports so each compiles. True duplicate-symbol reconciliation is the FINISHER's job — if two same-named items collide irreconcilably from your seat, report blocked.
- BotLib is a pre-seeded EMPTY struct (crates/mp/engine/botlib/src/lib.rs) — its fields land via the state-merge lane this run. Engine.bot (the field on Engine) is the finisher's job (ruling 43).
- §F types (ghoul2/rmg/npcnav/roff/stringed/tr_model) are golden-green with committed parity tests — NEVER change their definitions or behavior; call sites bend to them.
- The dangerous_implicit_autorefs lint precedent: crate-level allow with a doc comment (see mp_game lib.rs) if it blocks a crate.`

const FIXER_CONTRACT = `FIXER CONTRACT — you fix CALL SITES and MECHANICAL mismatches, never ported logic:
1. missing SYMBOL (free-standing const/type/fn, E0425/E0433/E0432) -> import/re-export it if it exists anywhere in the workspace (grep first — an existing use IS the answer), else port it to its canonical home in house style with Source cite.
2. call-SHAPE mismatch (E0061 arg-count, E0308 type, E0614/E0608 deref/index, E0599 method) -> the DECLARED signature/type is LAW; bend the CALL SITE (add args, explicit deref, as-casts per the rosetta span-cast idiom, &/&mut adjust). NEVER edit a declared §F signature, NEVER rewrite or delete a fn body's logic, NEVER introduce todo!() to silence a type/call error.
3. E0609 field-missing: if the receiver is one of the STATE STRUCTS (Common, CollisionWorld, Server, Client, BotLib, Icarus, Navigator, Ghoul2System, RmManager, RoffSystem, RenderModels) — SKIP IT; a parallel state-merge agent owns that struct file and is adding the field this round. Any OTHER type: the declared struct is LAW; bend the call site (wrong receiver? wrong deref?). Field-access failures are NEVER "missing symbols" to shim.
4. visibility/import (E0603/E0432/E0659) -> fix pub/use/dedupe at the module boundary.
5. unsafe hygiene (E0133) -> minimal unsafe block matching surrounding style.
NO-SHIM RULE: never define a local helper to paper over a missing symbol.
Anything needing a RULING or genuine LOGIC PORT -> do NOT guess: report it in \`blocked\` with {file, error, reason}. ANTI-TIME-BOX: return only when every file in your group is worked or genuinely blocked; "ran out of budget" is an INVALID blocked reason.
rustfmt PARSE GATE (mandatory per file): after your last edit to a file run \`rustfmt --edition 2021 --emit stdout <file> > /dev/null\`; any error means the file does not parse — fix and re-run. This is the ONLY compiler-adjacent command you may run — cargo is FORBIDDEN (parallel fixers share the tree; the inventory file is your error source).
RESERVED FILES you must NEVER edit (state-merge lane / finisher own them): ${RESERVED_FILES.join(', ')}.`

const STYLE = `NEVER touch oracle/. NEVER run cargo (unless your prompt explicitly grants it). NEVER git reset/clean/checkout — the worktree history is load-bearing. NEVER add a co-author trailer. All commits use --no-gpg-sign. Preserve Raven comments; doc-comment + Source cite on newly-added items.`

const MERGE_CONTRACT = `STATE-MERGE CONTRACT — you own EXACTLY ONE file (your struct's definition file) and edit NOTHING else:
For each missing field: find the Raven global's declaration (grep ${WT}/oracle/codemp for the exact name; the decl file gives the C type and any initializer). Add \`pub <RavenName>: <type>\` keeping the Raven name VERBATIM (arrays stay arrays with oracle sizes, ints stay c_int-family, function-static hoists keep their hoisted name). Use already-ported types (grep crates/mp/qshared, crates/mp/bg, sibling engine crates) — never invent a type that exists.
ZERO-VALID PREFERENCE: this struct may be built through Engine::new()'s zeroed-alloc path. Prefer faithful zero-valid representations (C arrays, ints, floats, raw pointers, #[repr(C)] structs of those). If a field genuinely needs Vec/String/Box/BTreeMap/HashMap/Option-with-niche:
- Common/CollisionWorld/Server: add it, and RETURN it in \`writelist\` as {path:"<engine-field>.<field>", note} (engine-field: Common=common, CollisionWorld=cm, Server=sv) — a serial applier adds the Engine::new() write entry.
- Icarus/Navigator/Ghoul2System/RmManager/RoffSystem/RenderModels/Client/BotLib: these are built via Default (or by value) — make sure the struct's Default impl/derive covers your new field (update a manual Default impl in the SAME file if present).
House style: doc-comment with the Raven name + Source cite per field (grouped cites fine for runs of related fields). rustfmt parse gate before returning. Return JSON {struct, fields_added:[..], writelist:[{path,note}], blocked:[{field,reason}]}.`

// ---- serialized commit chain ----
let commitChain = Promise.resolve()
const commits = []
function commit(msg, label) {
  commitChain = commitChain.then(() => agent(
    `In ${WT}: git add -A && git commit --no-gpg-sign -m ${JSON.stringify(msg)} (skip if nothing staged). NEVER add a co-author trailer. Return JSON {commit:"<short hash or 'nothing-to-commit'>"}.`,
    { phase: 'Fix rounds', label, model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }
  ).then(r => { if (r && r.commit && r.commit !== 'nothing-to-commit') commits.push({ msg, commit: r.commit }); return r }))
  return commitChain
}

const INV_SCHEMA = { type: 'object', properties: {
  total_errors: { type: 'number' },
  files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } },
  e0609_by_struct: { type: 'array', items: { type: 'object', properties: { struct: { type: 'string' }, fields: { type: 'array', items: { type: 'string' } } }, required: ['struct', 'fields'] } },
  tail: { type: 'string' },
}, required: ['total_errors', 'files', 'tail'] }

// ---- Triage ----
phase('Triage')
const triage = await agent(
  `TRIAGE for the engine-cpass integrate run. Worktree ${WT}, branch engine-cpass.
1. ${invCmd(1)}
2. Read back target/integrate/inv-r1.json. Bin the erroring files into ${GROUPS} groups balanced by TOTAL error count, keeping same-subsystem files together (cm_* cluster, cvar/cmd/cbuf, files_*, msg/net/huffman, sv_* cluster, cl_* cluster, botlib be_*/l_* cluster, vm_*, common/misc). EXCLUDE these reserved files from every group (state-merge lane owns them): ${RESERVED_FILES.join(', ')}. Every other erroring file in exactly one group; short name per group.
Return ONLY JSON {total_errors, files:[{file,errors}], e0609_by_struct:[{struct,fields:[..]}], groups:[{name, files:[..], errors}], tail}. No prose.`,
  { phase: 'Triage', label: 'triage', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: {
    total_errors: { type: 'number' },
    files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } },
    e0609_by_struct: { type: 'array', items: { type: 'object', properties: { struct: { type: 'string' }, fields: { type: 'array', items: { type: 'string' } } }, required: ['struct', 'fields'] } },
    groups: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, files: { type: 'array', items: { type: 'string' } }, errors: { type: 'number' } }, required: ['name', 'files'] } },
    tail: { type: 'string' },
  }, required: ['total_errors', 'files', 'groups', 'tail'] } }
)
const groups = triage.groups.map(g => ({ ...g, files: g.files.filter(f => !RESERVED_FILES.some(r => f.endsWith(r) || r.endsWith(f))) }))
log(`Triage: ${triage.total_errors} errors across ${triage.files.length} files, ${groups.length} groups, E0609 structs: ${(triage.e0609_by_struct || []).map(s => `${s.struct}=${s.fields.length}`).join(', ') || 'none'}`)

// ---- Fix rounds ----
const roundTotals = [{ round: 0, total: triage.total_errors }]
const blocked = []
const mergedFields = {}
let inv = triage
let prevTotal = triage.total_errors
let prevCrates = new Set()
const crateOf = f => { const m = String(f).match(/crates\/[^\s]*?\/(?:src|tests)\//); return m ? m[0] : 'other' }
for (const f of (triage.files || [])) prevCrates.add(crateOf(f.file))
let green = triage.total_errors === 0
let stopReason = green ? 'already-green' : null

phase('Fix rounds')
for (let round = 1; round <= MAX_ROUNDS && !green; round++) {
  const invByFile = new Map((inv.files || []).map(f => [f.file, f.errors]))
  const groupErrors = g => g.files.reduce((a, f) => a + (invByFile.get(f) || 0), 0)
  // files that appeared in the inventory but belong to no group (new crates surfacing as
  // upstream greens) -> shard them by crate, chunked to ~150 errors per new group
  const grouped = new Set(groups.flatMap(g => g.files))
  const newEntries = (inv.files || []).filter(f => !grouped.has(f.file) && !RESERVED_FILES.some(r => f.file.endsWith(r) || r.endsWith(f.file)))
  if (newEntries.length) {
    const byCrate = new Map()
    for (const f of newEntries) { const c = crateOf(f.file); if (!byCrate.has(c)) byCrate.set(c, []); byCrate.get(c).push(f) }
    let shard = 0
    for (const [c, list] of byCrate) {
      list.sort((a, b) => b.errors - a.errors)
      let bucket = [], mass = 0
      const flush = () => { if (bucket.length) { groups.push({ name: `new-r${round}-${shard++}`, files: bucket.map(f => f.file) }); bucket = []; mass = 0 } }
      for (const f of list) { bucket.push(f); mass += f.errors; if (mass >= 150) flush() }
      flush()
    }
    log(`Round ${round}: ${newEntries.length} newly-surfaced files sharded into ${shard} new groups`)
  }
  const active = groups.map(g => ({ ...g, cur: groupErrors(g) })).filter(g => g.cur > 0).sort((a, b) => b.cur - a.cur)

  // state-merge bins: E0609 receivers that are known state structs, minus fields already merged
  const structBins = (inv.e0609_by_struct || [])
    .filter(s => STATE_STRUCTS[s.struct])
    .map(s => ({ ...s, fields: s.fields.filter(f => !(mergedFields[s.struct] || new Set()).has(f)) }))
    .filter(s => s.fields.length > 0)

  if (!active.length && !structBins.length) { green = true; stopReason = 'green'; break }
  const heavy = new Set(active.slice(0, 2).map(g => g.name))
  const invPath = `${INV_DIR}/inv-r${round}.json`
  log(`Round ${round}: ${active.length} file groups + ${structBins.length} struct merges, ${prevTotal} errors (heaviest: ${active.slice(0, 2).map(g => `${g.name}=${g.cur}`).join(', ')})`)

  const structThunks = structBins.map(s => () => agent(
    `STATE-STRUCT FIELD MERGE — round ${round}, struct \`${s.struct}\`. Worktree ${WT}, branch engine-cpass.
Your file (the ONLY file you may edit): ${STATE_STRUCTS[s.struct]}.
Missing Raven-named fields referenced by transcribed bodies (from E0609 inventory): ${s.fields.join(', ')}.
Context for each field's USE sites is in ${invPath} (grep its samples), and the authoritative decl is in the oracle.
${MERGE_CONTRACT}
${STYLE}`,
    { label: `merge:${s.struct}`, phase: 'Fix rounds', model: s.fields.length > 30 ? 'opus' : 'sonnet', effort: 'medium',
      schema: { type: 'object', properties: {
        struct: { type: 'string' }, fields_added: { type: 'array', items: { type: 'string' } },
        writelist: { type: 'array', items: { type: 'object', properties: { path: { type: 'string' }, note: { type: 'string' } }, required: ['path'] } },
        blocked: { type: 'array', items: { type: 'object', properties: { field: { type: 'string' }, reason: { type: 'string' } }, required: ['reason'] } },
      }, required: ['struct', 'fields_added'] } }
  ))

  const fixThunks = active.map(g => () => agent(
    `INTEGRATE FIXER — round ${round}, group "${g.name}". Worktree ${WT}, branch engine-cpass.
YOUR FILES ONLY (${g.cur} errors this round): ${g.files.join(', ')}.
Current error detail is in ${invPath} (JSON {files:[{file,errors,codes,samples}]}) — Read it and take only your files' entries. Do NOT run cargo; the inventory is your error source.${round >= 2 ? `
INVENTORY GUARANTEE: entries carry codes+samples; a missing-detail entry is a tooling fault — report ONE blocked item for it and still work your other files; do not zero-work the group.` : ''}
Work FILE BY FILE: read the errors, fix per contract, rustfmt parse-gate the file, move on.
${FIXER_CONTRACT}
${KNOWN_DEBT}
${CONTRACT_ASSETS}
${STYLE}
Do NOT git commit (a serial committer handles it). Return JSON {group, fixed, files_touched:[..], blocked:[{file,error,reason}]}.`,
    { label: `fix:${g.name}`, phase: 'Fix rounds', model: heavy.has(g.name) ? 'opus' : 'sonnet', effort: heavy.has(g.name) ? 'medium' : 'low',
      schema: { type: 'object', properties: {
        group: { type: 'string' }, fixed: { type: 'number' },
        files_touched: { type: 'array', items: { type: 'string' } },
        blocked: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' }, reason: { type: 'string' } }, required: ['error'] } },
      }, required: ['fixed'] } }
  ))

  const results = await parallel(structThunks.concat(fixThunks))
  const structResults = results.slice(0, structThunks.length).filter(Boolean)
  const fixResults = results.slice(structThunks.length).filter(Boolean)
  for (const r of structResults) {
    const set = mergedFields[r.struct] || (mergedFields[r.struct] = new Set())
    for (const f of (r.fields_added || [])) set.add(f)
    for (const b of (r.blocked || [])) blocked.push({ round, file: STATE_STRUCTS[r.struct], error: `field ${b.field || '?'} on ${r.struct}`, reason: b.reason })
  }
  for (const r of fixResults) for (const b of (r.blocked || [])) blocked.push({ round, ...b })
  const roundFixed = fixResults.reduce((a, r) => a + (r.fixed || 0), 0)
  const roundMerged = structResults.reduce((a, r) => a + (r.fields_added || []).length, 0)

  // serial write-list applier (owns engine.rs) before the commit
  const writelist = structResults.flatMap(r => r.writelist || [])
  if (writelist.length) {
    await agent(
      `WRITE-LIST APPLIER (serial — you are the only writer of ${ENGINE_RS}). Worktree ${WT}.
The state-merge lane added non-zero-valid fields that Engine::new()'s zeroed-alloc path must initialize in place. For EACH entry below, add \`addr_of_mut!((*p).<path>).write(Default::default());\` inside the existing unsafe write-list block in Engine::new() (${ENGINE_RS}), following the exact style of the existing entries (see the g2/rmg/nav/roff/stringed lines), with a one-line comment citing the field. Skip any entry already present.
ENTRIES: ${JSON.stringify(writelist)}
rustfmt parse-gate the file. ${STYLE} Return JSON {applied:<n>}.`,
      { label: `writelist:r${round}`, phase: 'Fix rounds', model: 'sonnet', effort: 'low',
        schema: { type: 'object', properties: { applied: { type: 'number' } }, required: ['applied'] } }
    )
  }

  await commit(`cpass integrate round ${round}: ${active.length} groups, ${roundMerged} fields merged, ${roundFixed} fixes`, `commit:r${round}`)
  let reInv = await agent(
    `RE-INVENTORY after integrate round ${round}. Worktree ${WT}.
${invCmd(round + 1)}
Your RETURN stays thin: ONLY JSON {total_errors, files:[{file,errors}], e0609_by_struct:[{struct,fields:[..]}], tail}. No prose. (The inv JSON file you wrote carries codes+samples for the next round's fixers — they are forbidden from running cargo and depend on it.)`,
    { phase: 'Fix rounds', label: `re-triage:r${round}`, model: 'haiku', effort: 'low', schema: INV_SCHEMA }
  )
  if (reInv.total_errors === 0 && !/finished/i.test(String(reInv.tail || ''))) {
    log(`Recount claims 0 errors but cargo tail is "${String(reInv.tail || '').slice(0, 80)}" — refuting with an independent sonnet recount`)
    reInv = await agent(
      `VERIFY a suspicious green claim after integrate round ${round}. Worktree ${WT}.
${invCmd(round + 1)}
Return ONLY JSON {total_errors, files:[{file,errors}], e0609_by_struct:[{struct,fields:[..]}], tail}.`,
      { phase: 'Fix rounds', label: `re-triage-verify:r${round}`, model: 'sonnet', effort: 'low', schema: INV_SCHEMA }
    )
  }
  inv = reInv
  const newTotal = reInv.total_errors
  const newCrates = new Set((reInv.files || []).map(f => crateOf(f.file)))
  const surfaced = [...newCrates].filter(c => !prevCrates.has(c))
  roundTotals.push({ round, total: newTotal, fixed: roundFixed, merged: roundMerged, surfaced })
  log(`Round ${round} done: ${prevTotal} -> ${newTotal} (${roundMerged} fields merged, ${roundFixed} claimed fixes${surfaced.length ? `; NEW crates surfaced: ${surfaced.join(', ')}` : ''})`)

  if (newTotal === 0) { green = true; stopReason = 'green'; break }
  // CASCADE-AWARE DELTA TRIPWIRE: totals may legitimately RISE when a crate greens and
  // downstream crates surface. Only trip when no new crate surfaced AND reduction < 3%.
  if (!surfaced.length && newTotal >= prevTotal * 0.97) {
    stopReason = `delta-tripwire (round ${round}: ${prevTotal} -> ${newTotal}, <3% reduction, no cascade)`
    log(`DELTA TRIPWIRE: ${stopReason} — stopping the loop`)
    prevTotal = newTotal; prevCrates = newCrates; break
  }
  prevTotal = newTotal; prevCrates = newCrates
  if (newTotal < FINISHER_THRESHOLD) { stopReason = `small-tail (${newTotal} < ${FINISHER_THRESHOLD})`; log(`Tail below finisher threshold — handing to finisher`); break }
}
if (!green && !stopReason) stopReason = `rounds-exhausted (${MAX_ROUNDS})`

// ---- Finisher ----
let finisher = null
if (!green) {
  phase('Finisher')
  log(`Finisher: ~${prevTotal} errors remain (${stopReason})`)
  finisher = await agent(
    `FINISHER — serial, cross-cutting. Worktree ${WT}, branch engine-cpass. The parallel rounds stopped with ~${prevTotal} errors remaining (${stopReason}). Latest inventory: ${INV_DIR}/ (highest-numbered inv-r*.json).
Drive \`cargo check --workspace\` to GREEN. You MAY make cross-cutting fixes spanning groups and the reserved files (state structs, ${ENGINE_RS}). Your known cross-cutting jobs beyond the residual list:
1. cm module reconciliation: qcommon's cm/ dir (§F golden-green) vs src-root cm_*.rs (C-track). Wire both to compile; where the SAME symbol exists twice, the §F version is canonical for its scope and the C-track caller imports it — the C-track variants of cm_patch/cm_randomterrain/cm_terrain also live in pre-merge history (git log --all) if you need to consult them. NEVER delete §F code or weaken its parity tests.
2. Engine.bot: add \`pub bot: mp_engine_botlib::BotLib\` to Engine (${ENGINE_RS}) with a ruling-43 cite + Cargo dep if missing + a Default write in Engine::new() if BotLib is not all-zero-valid.
3. Write-list audit: \`git diff c9e4208e --stat\` the state-struct files; EVERY non-zero-valid field added this run on Common/CollisionWorld/Server must have its Engine::new() write entry. Add missing ones.
The SAME fixer contract applies — call sites bend to declarations, no logic rewrites, no todo!() to silence errors; genuinely-blocked items stay reported, not faked.
${FIXER_CONTRACT}
${KNOWN_DEBT}
${CONTRACT_ASSETS}
${STYLE}
EXCEPTIONS granted to you alone: you MAY run cargo (you are the only writer now — iterate \`cd ${WT} && cargo check --workspace\` until green) and you MAY edit reserved files. Parse-gate touched files. FINAL: git add -A && git commit --no-gpg-sign -m "cpass integrate: workspace green" (or "cpass integrate: finisher stop, <n> errors remain"). NEVER add a co-author trailer.
Return JSON {green, remaining_errors, fixed, commit, blocked:[{file,error,reason}]}.`,
    { phase: 'Finisher', label: 'finisher', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      green: { type: 'boolean' }, remaining_errors: { type: 'number' }, fixed: { type: 'number' },
      commit: { type: 'string' }, blocked: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' }, reason: { type: 'string' } }, required: ['error'] } },
    }, required: ['green'] } }
  )
  green = !!(finisher && finisher.green)
  if (finisher) {
    if (finisher.commit && finisher.commit !== 'nothing-to-commit') commits.push({ msg: 'cpass integrate: finisher', commit: finisher.commit })
    for (const b of (finisher.blocked || [])) blocked.push({ round: 'finisher', ...b })
    if (typeof finisher.remaining_errors === 'number') prevTotal = finisher.remaining_errors
    roundTotals.push({ round: 'finisher', total: finisher.remaining_errors ?? prevTotal, fixed: finisher.fixed })
  }
}
await commitChain

// ---- Report ----
phase('Report')
const seen = new Set()
const blockedUniq = blocked.filter(b => { const k = `${b.file || ''}::${b.error}`; if (seen.has(k)) return false; seen.add(k); return true })
const report = {
  green,
  final_errors: green ? 0 : prevTotal,
  stop_reason: stopReason,
  round_totals: roundTotals,
  fields_merged: Object.fromEntries(Object.entries(mergedFields).map(([k, v]) => [k, [...v].length])),
  commits,
  blocked: blockedUniq,
}
log(`INTEGRATE DONE: green=${green}, final=${report.final_errors} errors, ${commits.length} commits, ${blockedUniq.length} blocked items`)
return report
