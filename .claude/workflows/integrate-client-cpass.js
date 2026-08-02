export const meta = {
  name: 'integrate-client-cpass',
  description: 'Integrate the jamp client transcription on branch client-cpass: drive the workspace to green via triage -> parallel rounds (Client/Common/CollisionWorld field merges + file-group fixers, cascade-aware delta tripwire) -> serial finisher. Fixers repair call sites, imports, and carrier wiring only; they never change ported logic.',
  whenToUse: 'After the client-cpass transcription wave (398 fns, commits 636a425a..3321414e): qcommon fails first (cm_draw/cm_terrainmap) and gates the mp_engine_client census. Errors are dominated by carrier-hop misses (cl.snap vs cl.cl.snap), under-threaded receivers, and missing flat globals on Client.',
  phases: [
    { title: 'Triage', detail: 'one agent runs cargo check --workspace, writes a machine-readable per-file inventory (incl. E0609 receiver-struct bins), and bins erroring files into subsystem groups excluding the state-struct definition files' },
    { title: 'Fix rounds', detail: 'up to MAX_ROUNDS rounds; parallel state-struct field-merge agents alongside per-group file fixers, then a serial write-list applier, serial commit, re-inventory; cascade-aware DELTA TRIPWIRE' },
    { title: 'Finisher', detail: 'one serial opus finisher: trampolines via the sv_game thread-local slot, pending-lane stubs, marker sweep, drives cargo check --workspace to green' },
    { title: 'Report', detail: 'per-round totals, failing-crate progression, commits, consolidated blocked list' },
  ],
}

// Config is HARDCODED (workflow-args string bug): relaunch via scriptPath after edits.
const WT = '/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/client-cpass'
const MAIN = '/Users/milohehmsoth/Developer/Milo/jka-rust'
const MAX_ROUNDS = 6
const GROUPS = 8
const FINISHER_THRESHOLD = 120
const INV_DIR = `${WT}/target/integrate`

// State-receiver structs and their definition files.
// Field-merge agents OWN these files; file-group fixers NEVER edit them.
const STATE_STRUCTS = {
  Common: 'crates/mp/engine/qcommon/src/common/common.rs',
  CollisionWorld: 'crates/mp/engine/qcommon/src/collision_world.rs',
  Client: 'crates/mp/engine/client/src/client_host.rs',
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
const CARRIER_RULE = `CLIENT CARRIER RULE (the pass-wide E0609/E0425 theme):
\`Client\` (crates/mp/engine/client/src/client_host.rs) is the island carrier. Raven's five client globals live on it as boxed fields: cl.cl (clientActive_t), cl.clc (clientConnection_t), cl.cls (clientStatic_t), cl.kg (keyGlobals_t), cl.con (console_t). Boxes auto-deref, so cl.clc.reliableAcknowledge already compiles. Transcribed bodies kept Raven spellings, so:
- E0609 "no field \`X\` on \`Client\`" where X is a MEMBER of one of the five aggregate types (check crates/mp/engine/client/src/client/client_active_t.rs, client_connection_t.rs, client_static_t.rs, crates/mp/engine/client/src/keys/key_globals_s.rs, client/console_t.rs) -> insert the aggregate hop at the call site: cl.snap -> cl.cl.snap, cl.serverTime -> cl.cl.serverTime, cl.state -> cl.cls.state. This is YOUR fix, not the merge lane's.
- E0425 bare \`clc\`/\`cls\`/\`kg\`/\`con\` identifiers -> cl.clc / cl.cls / cl.kg / cl.con. If the fn lacks the \`cl: &mut Client\` receiver, thread it in pinned order AND update every call site.
- E0609 on \`Client\` where X is a FILE-SCOPE global or hoisted static (cvar handles like cl_shownet, cl_timeout; cl_main.cpp statics) -> SKIP IT: the state-merge lane adds that field to Client this round.
- The five aggregate types are ABI-frozen with layout asserts. NEVER add a field to clientActive_t / clientConnection_t / clientStatic_t / keyGlobals_t / console_t.`

const CONTRACT_ASSETS = `CONTRACT ASSETS — READ THESE FIRST (they are LAW; do not re-derive from memory):
- Rosetta Stone: ${WT}/docs/porting/rosetta.md, the "## EXAMPLE SYNTAX" section — canonical mapped shapes for the vec3/q_math macro family, va()/printf, cstr helpers. Use these EXACT shapes at call sites.
- Rulings: ${WT}/docs/decisions.md entries DEC-55..DEC-58 and ${WT}/tools/closure-prototype/modules/mp-client-rulings-digest.md — every ruling is SETTLED; never re-litigate.
- RECEIVER CONVENTION: client-pass functions thread &mut state receivers in the pinned parameter order (common: &mut Common, cm: &mut CollisionWorld, sv: &mut Server, cl: &mut Client, rm: &mut RenderModels, g2: &mut Ghoul2System, host: &mut dyn EngineHost — host always last). The transcribed signatures are LAW — call sites bend to them. If a fn body references a receiver it lacks (E0425 "cannot find value \`common\`"), thread the missing receiver param through in pinned order AND update every call site — mechanical, allowed.
- STATE FIELDS: bodies write \`receiver.<RavenGlobalName>\` verbatim for the flat receivers (common.com_frameTime, cm.NumSubBSP). The Client carrier follows the CARRIER RULE above. Raven names stay; non_snake_case is allowed crate-wide.
- Com_Error call sites are panics (ruling 1). rand-family calls route through common.qrand (irand/flrand/crandom/Q_irand — ruling 21), NEVER libc rand.
- Missing consts/types: GREP THE WORKSPACE FIRST (crates/mp/qshared, crates/mp/abi, crates/native, sibling engine crates, crates/mp/engine/client/src/client/) — most exist; import/re-export rather than re-port. Only port a genuinely-absent item, to its canonical home, house style with Source cite.
${CARRIER_RULE}`

const KNOWN_DEBT = `KNOWN DEBT you may hit:
- qcommon gate: cm_draw.rs and cm_terrainmap.rs (mp_engine_qcommon src root, C-track from this wave) fail first and BLOCK the type-check of mp_engine_client. Their group is the priority — the client crate's true error mass only surfaces after qcommon greens.
- PENDING-LANE SYMBOLS: S_* / snd (ticket #24), FX_* (tickets #26/#27), C_MP3_* (ticket #25) are unported lanes. Do NOT port or shim them from a fixer seat: report each in \`blocked\` with the lane tag — the serial finisher lands the house-marker stubs once, at one canonical home per lane.
- The accepted todo!: the CL_CgameSystemCalls trampoline (cl_cgame.rs) — the finisher's job; leave it.
- STALE MARKERS: the symbol round landed many consts (client_consts.rs, cin_consts.rs). If a \`//TODO: Port <name>\` marker names a symbol that now exists in the workspace, wire the import and DELETE the marker line plus its Source line. Keep genuinely-open markers.
- §F surfaces (ghoul2/tr_model/renderer frontend) are golden-green with parity tests — NEVER change their definitions or behavior; call sites bend to them.
- The dangerous_implicit_autorefs lint precedent: crate-level allow with a doc comment (see mp_game lib.rs) if it blocks a crate.`

const FIXER_CONTRACT = `FIXER CONTRACT — you fix CALL SITES and MECHANICAL mismatches, never ported logic:
1. missing SYMBOL (free-standing const/type/fn, E0425/E0433/E0432) -> import/re-export it if it exists anywhere in the workspace (grep first — an existing use IS the answer), else port it to its canonical home in house style with Source cite. Pending-lane symbols (S_*/FX_*/C_MP3_*) are the exception: report blocked, never port.
2. call-SHAPE mismatch (E0061 arg-count, E0308 type, E0614/E0608 deref/index, E0599 method) -> the DECLARED signature/type is LAW; bend the CALL SITE (add args, explicit deref, as-casts per the rosetta span-cast idiom, &/&mut adjust). NEVER edit a declared §F signature, NEVER rewrite or delete a fn body's logic, NEVER introduce todo!() to silence a type/call error.
3. E0609 field-missing: on \`Client\`, apply the CARRIER RULE partition (aggregate member = your hop fix; flat global = skip for the merge lane). On \`Common\`/\`CollisionWorld\` — SKIP IT; a parallel state-merge agent owns that struct file. Any OTHER type: the declared struct is LAW; bend the call site (wrong receiver? wrong deref?). Field-access failures are NEVER "missing symbols" to shim.
4. visibility/import (E0603/E0432/E0659) -> fix pub/use/dedupe at the module boundary.
5. unsafe hygiene (E0133) -> minimal unsafe block matching surrounding style.
NO-SHIM RULE: never define a local helper to paper over a missing symbol.
Anything needing a RULING or genuine LOGIC PORT -> do NOT guess: report it in \`blocked\` with {file, error, reason}. ANTI-TIME-BOX: return only when every file in your group is worked or genuinely blocked; "ran out of budget" is an INVALID blocked reason.
rustfmt PARSE GATE (mandatory per file): after your last edit to a file run \`rustfmt --edition 2021 --emit stdout <file> > /dev/null\`; any error means the file does not parse — fix and re-run. This is the ONLY compiler-adjacent command you may run — cargo is FORBIDDEN (parallel fixers share the tree; the inventory file is your error source).
RESERVED FILES you must NEVER edit (state-merge lane / finisher own them): ${RESERVED_FILES.join(', ')}.`

const STYLE = `NEVER touch oracle/ or ${MAIN} outside the worktree. NEVER run cargo (unless your prompt explicitly grants it). NEVER git reset/clean/checkout — the worktree history is load-bearing. NEVER add a co-author trailer. All commits use --no-gpg-sign. Preserve Raven comments; doc-comment + Source cite on newly-added items. New comment prose is STE: active voice, full sentences, no semicolons, one sentence per line.`

const MERGE_CONTRACT = `STATE-MERGE CONTRACT — you own EXACTLY ONE file (your struct's definition file) and edit NOTHING else:
PARTITION FIRST (Client only): a field that is a MEMBER of clientActive_t / clientConnection_t / clientStatic_t / keyGlobals_t / console_t is NOT yours — the fixer lane inserts the carrier hop (cl.cl.snap). Check the aggregate type files under crates/mp/engine/client/src/client/ and src/keys/ before adding anything. Add ONLY genuine file-scope globals and hoisted statics (cvar handles, cl_main.cpp statics, timers).
For each missing field: find the Raven global's declaration (grep ${WT}/oracle/codemp for the exact name; the decl file gives the C type and any initializer). Add \`pub <RavenName>: <type>\` keeping the Raven name VERBATIM (arrays stay arrays with oracle sizes, ints stay c_int-family, cvar_t* globals become the CvarHandle idiom already used in the file). Use already-ported types (grep crates/mp/qshared, crates/mp/abi, sibling engine crates) — never invent a type that exists.
ZERO-VALID PREFERENCE: prefer faithful zero-valid representations (C arrays, ints, floats, raw pointers, #[repr(C)] structs of those). If a field genuinely needs Vec/String/Box/Option-with-niche:
- Common/CollisionWorld: add it, and RETURN it in \`writelist\` as {path:"<engine-field>.<field>", note} (engine-field: Common=common, CollisionWorld=cm) — a serial applier adds the Engine::new() write entry.
- Client: it is built via the manual Default impl in the SAME file — extend that impl to cover your new field.
House style: doc-comment with the Raven name + Source cite per field (grouped cites fine for runs of related fields), STE prose. rustfmt parse gate before returning. Return JSON {struct, fields_added:[..], writelist:[{path,note}], blocked:[{field,reason}]}.`

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
  `TRIAGE for the client-cpass integrate run. Worktree ${WT}, branch client-cpass.
1. ${invCmd(1)}
2. Read back target/integrate/inv-r1.json. Bin the erroring files into up to ${GROUPS} groups balanced by TOTAL error count, keeping these natural clusters together: qcommon-gate (cm_draw.rs + cm_terrainmap.rs — always its OWN group), cl_main, cl_parse + cl_net_chan, cl_cgame, cl_ui, cl_input + cl_keys, cl_console + cl_scrn, cl_cin. EXCLUDE these reserved files from every group (state-merge lane owns them): ${RESERVED_FILES.join(', ')}. Every other erroring file in exactly one group; short name per group.
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
  // files that appeared in the inventory but belong to no group (the client crate surfacing
  // as qcommon greens) -> shard them by crate, chunked to ~150 errors per new group
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
    `STATE-STRUCT FIELD MERGE — round ${round}, struct \`${s.struct}\`. Worktree ${WT}, branch client-cpass.
Your file (the ONLY file you may edit): ${STATE_STRUCTS[s.struct]}.
Missing fields referenced by transcribed bodies (from E0609 inventory — the Client bin MIXES carrier-hop misses with genuine flat globals; partition per the contract): ${s.fields.join(', ')}.
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
    `INTEGRATE FIXER — round ${round}, group "${g.name}". Worktree ${WT}, branch client-cpass.
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
The state-merge lane added non-zero-valid fields that Engine::new()'s zeroed-alloc path must initialize in place. For EACH entry below, add \`addr_of_mut!((*p).<path>).write(Default::default());\` inside the existing unsafe write-list block in Engine::new() (${ENGINE_RS}), following the exact style of the existing entries, with a one-line comment citing the field. Skip any entry already present.
ENTRIES: ${JSON.stringify(writelist)}
rustfmt parse-gate the file. ${STYLE} Return JSON {applied:<n>}.`,
      { label: `writelist:r${round}`, phase: 'Fix rounds', model: 'sonnet', effort: 'low',
        schema: { type: 'object', properties: { applied: { type: 'number' } }, required: ['applied'] } }
    )
  }

  await commit(`client integrate round ${round}: ${active.length} groups, ${roundMerged} fields merged, ${roundFixed} fixes`, `commit:r${round}`)
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
  // CASCADE-AWARE DELTA TRIPWIRE: totals may legitimately RISE when qcommon greens and
  // mp_engine_client surfaces. Only trip when no new crate surfaced AND reduction < 3%.
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
    `FINISHER — serial, cross-cutting. Worktree ${WT}, branch client-cpass. The parallel rounds stopped with ~${prevTotal} errors remaining (${stopReason}). Latest inventory: ${INV_DIR}/ (highest-numbered inv-r*.json).
Drive \`cargo check --workspace\` to GREEN. You MAY make cross-cutting fixes spanning groups and the reserved files (state structs, ${ENGINE_RS}). Your known cross-cutting jobs beyond the residual list:
1. DISPATCHER TRAMPOLINES: CL_CgameSystemCalls (the accepted todo! in cl_cgame.rs) and the cl_ui.rs dispatcher need the boot-seam view slot. READ crates/mp/engine/server/src/sv_game.rs first — mirror its thread-local view mechanism exactly (same idiom, client-side slot). hook_install.rs already routes VmSlot::Cgvm | VmSlot::Uivm to null (ruling 33b) — do not change it.
2. PENDING-LANE STUBS: collect every blocked S_* / snd symbol into ONE new file crates/mp/engine/client/src/snd_stubs.rs (module doc: ticket #24 owns the real port), every FX_* into fx_stubs.rs (tickets #26/#27), C_MP3_* into mp3_stubs.rs (ticket #25). Each stub is the house pattern: \`//TODO: Port <sym>\` + \`// Source:\` cite + a body of \`todo!("Port <sym> — oracle/...")\`. Signatures come from the call sites bent to minimal faithful shapes. NEVER a silent no-op.
3. Cmd_AddCommand ADAPTER: client command registrations must match the tree's Cmd_AddCommand signature — grep the server/common callers for the established handler idiom and bend the client registrations to it.
4. cinTable VQ0/VQ1 fn-pointer fields: type them per the oracle decl (cl_cin.cpp) against the ported ROQ fns.
5. STALE MARKER SWEEP: grep '//TODO: Port' across crates/mp/engine/client and the two qcommon files; delete every marker whose symbol now resolves (wire the import), keep genuinely-open ones.
6. Write-list audit: every non-zero-valid field added this run on Common/CollisionWorld must have its Engine::new() write entry in ${ENGINE_RS}. Add missing ones.
The SAME fixer contract applies — call sites bend to declarations, no logic rewrites, no todo!() to silence a TYPE error (pending-lane stubs are the one sanctioned todo! class); genuinely-blocked items stay reported, not faked.
${FIXER_CONTRACT}
${KNOWN_DEBT}
${CONTRACT_ASSETS}
${STYLE}
EXCEPTIONS granted to you alone: you MAY run cargo (you are the only writer now — iterate \`cd ${WT} && cargo check --workspace\` until green) and you MAY edit reserved files. Parse-gate touched files. FINAL: git add -A && git commit --no-gpg-sign -m "client-cpass integrate: workspace green" (or "client-cpass integrate: finisher stop, <n> errors remain"). NEVER add a co-author trailer.
Return JSON {green, remaining_errors, fixed, commit, blocked:[{file,error,reason}]}.`,
    { phase: 'Finisher', label: 'finisher', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      green: { type: 'boolean' }, remaining_errors: { type: 'number' }, fixed: { type: 'number' },
      commit: { type: 'string' }, blocked: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' }, reason: { type: 'string' } }, required: ['error'] } },
    }, required: ['green'] } }
  )
  green = !!(finisher && finisher.green)
  if (finisher) {
    if (finisher.commit && finisher.commit !== 'nothing-to-commit') commits.push({ msg: 'client-cpass integrate: finisher', commit: finisher.commit })
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
