export const meta = {
  name: 'client-cpass-transcribe',
  description: 'Client island C-track transcription: blind porters over out/client packets in the client-cpass worktree, stops after triage',
  phases: [
    { title: 'Plan', detail: 'read manifest, group packets by destination oracle file' },
    { title: 'Port', detail: 'one blind porter per file group; WIP commits; no cargo' },
    { title: 'Wire', detail: 'serial mod-decl wiring + receiver-aggregate check + symbol round' },
    { title: 'Triage', detail: 'cargo check error census; STOP (integration is a later workflow)' },
  ],
}

// args: { files?: ["cl_cgame.cpp", ...] } — restrict the wave to these oracle
// files. Default: every packeted file EXCEPT the snd_* TUs (their SoundSystem
// receiver split is unsettled, DEC-57 integration decision; snd waves launch
// after ticket #23).
const MAIN = '/Users/milohehmsoth/Developer/Milo/jka-rust'
const WT = `${MAIN}/.claude/worktrees/client-cpass`
const PKT = `${MAIN}/tools/closure-prototype/out/client/packets`
const ONLY = (args && args.files) || null

const STYLE = `HOUSE RULES: doc-comment + Source cite on every fn (oracle/codemp/<file>:<lines>); preserve Raven comments that clarify behavior; NEVER touch oracle/ or ${MAIN} outside the worktree; NEVER git commit; NEVER add a co-author trailer; NEVER run cargo (parallel writers share the tree); do not read .claude/workflows/.`
const ZERO_PARK = `ZERO-PARK (nothing stops you): every fn in your packets gets a real body. Missing symbol (type/const/fn not in the tree) -> write the reference EXACTLY as the packet resolves it and report it in missing_symbols; never stub, never invent an alternative, never define a local shim. Ambiguous logic -> most literal faithful reading + // PORT-NOTE(<topic>): <one line>. Callee signature looks wrong -> the packet's resolved signature is LAW; match it, report in shape_mismatches. todo!()/TODO/FIXME are FORBIDDEN in your output. MACRO TRAP: when a Raven vector macro (VectorMA/VectorScale/VectorAdvance) takes a rand-consuming or side-effecting argument, evaluate it once per component in Raven's component order - never hoist into one draw. Raven float constants like 0.08f widen as (0.08f32 as f64), never as a bare f64 literal.`
const DISCIPLINE = `MECHANICS: read ${PKT}/_PREAMBLE.md FIRST — it is LAW (the CLIENT ISLAND RULINGS DIGEST, state receivers in pinned order, CONSTS import tables — never redefine, never magic-number; three-kind statics rule; destination module rule). STATE FIELDS: spell the Raven global name VERBATIM on the receiver (cl.snap, clc.netchan, cls.state) — integration merges fields keeping Raven names; never invent idiomatic names. Receivers a body never uses still stay in the signature (LAW). Raw pointers deref explicitly ((*p).f). §20-DROPPED callee rows: the callee never ports; transcribe the calling branch as Raven wrote it and report the symbol in missing_symbols ONLY if the packet does not mark it dropped — marked drops are expected dead references, note them with // PORT-NOTE(s20). FX_* callees are the pending section-F lane: write the call exactly as the packet resolves it and report in missing_symbols. DO NOT edit any lib.rs/mod.rs — return the mod decls you need in mod_decls_needed instead. PARSE GATE (mandatory): rustfmt --edition 2021 --emit stdout <your files> > /dev/null until clean — the ONLY compiler-adjacent command you may run.`

phase('Plan')
const groupsR = await agent(
`Read ${PKT}/manifest.json. Using its data plus the packet filenames in ${PKT}/ (pattern <subsystem>__<seq>_<Fn>.md), produce porter groups: one group per (subsystem, oracle source file). Skip manifest entries marked dropped (rule20_drops) and entries with no packet (cpp-undocumented track). Each group: {subsystem, file, packets: ["<packet filename>.md", ...], loc: <total oracle LOC>, fns: <count>}. Return JSON {groups:[...]} sorted by loc descending. No prose.`,
  { label: 'plan-groups', phase: 'Plan', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: { groups: { type: 'array', items: { type: 'object', properties: { subsystem: { type: 'string' }, file: { type: 'string' }, packets: { type: 'array', items: { type: 'string' } }, loc: { type: 'number' }, fns: { type: 'number' } }, required: ['subsystem', 'file', 'packets'] } } }, required: ['groups'] } })
if (!groupsR || !groupsR.groups.length) throw new Error('group planning failed')
const groups = groupsR.groups.filter(g =>
  ONLY ? ONLY.includes(g.file) : !g.file.startsWith('snd_'))
const held = groupsR.groups.length - groups.length
log(`${groups.length} file groups this wave (${held} held back), ${groups.reduce((a, g) => a + (g.fns || g.packets.length), 0)} fns`)

phase('Port')
const PORT_SCHEMA = { type: 'object', properties: {
  file: { type: 'string' }, fns_filled: { type: 'number' },
  port_notes: { type: 'array', items: { type: 'object', properties: { fn: { type: 'string' }, topic: { type: 'string' }, note: { type: 'string' } }, required: ['fn', 'topic'] } },
  missing_symbols: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, kind: { type: 'string' }, source: { type: 'string' } }, required: ['name'] } },
  shape_mismatches: { type: 'array', items: { type: 'object', properties: { callee: { type: 'string' }, detail: { type: 'string' } }, required: ['callee'] } },
  mod_decls_needed: { type: 'array', items: { type: 'string' } },
}, required: ['file', 'fns_filled', 'mod_decls_needed'] }
const tierFor = g => (g.loc || 0) > 2500 ? 'opus' : 'sonnet'
let commitChain = Promise.resolve(), sinceCommit = 0, committed = 0
function maybeCommit(force) {
  sinceCommit++
  if (!force && sinceCommit < 8) return
  const n = ++committed; sinceCommit = 0
  commitChain = commitChain.then(() => agent(
    `In ${WT} (branch client-cpass): git add -A && git commit --no-gpg-sign -m "client-cpass WIP ${n}: porter output (pre-integration, NOT green)". No co-author trailer. Return JSON {commit:"<hash or nothing-to-commit>"}.`,
    { label: `wip-${n}`, phase: 'Port', model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }))
}
const reports = []
await parallel(groups.map(g => async () => {
  const model = tierFor(g)
  const r = await agent(
`BLIND C-TRACK PORTER for oracle ${g.file} (${g.packets.length} packets, ~${g.loc} LOC). Work in the WORKTREE ${WT} (branch client-cpass) ONLY.
YOUR ENTIRE INPUT: (1) ${PKT}/_PREAMBLE.md; (2) your packets: ${g.packets.map(p => `${PKT}/${p}`).join(' ')} — each carries the resolved signature (LAW, receivers included), verbatim oracle body, receiver-annotated callee table, CONSTS/TYPE-ROSETTA import tables, state table, and a DESTINATION line; (3) the destination file it names (under ${WT}/crates/...) plus the rosetta-cited import paths. Explore nothing else.
Create the destination module and transcribe every packet's body into its resolved signature.
${ZERO_PARK}
${DISCIPLINE}
${STYLE}
Return JSON {file, fns_filled, port_notes, missing_symbols, shape_mismatches, mod_decls_needed:["<crate lib.rs>: pub mod <stem>;", ...]}.`,
    { label: `port:${g.file}`, phase: 'Port', model, effort: model === 'opus' ? 'medium' : 'low', schema: PORT_SCHEMA })
  if (r) {
    reports.push(r)
    const expected = g.fns || g.packets.length
    if (r.fns_filled < expected * 0.6 && expected > 3) log(`ANOMALY ${g.file}: ${r.fns_filled}/${expected} filled`)
    maybeCommit(false)
  }
  return r
}))
maybeCommit(true)
await commitChain
log(`Port done: ${reports.length}/${groups.length} groups, ${reports.reduce((a, r) => a + (r.fns_filled || 0), 0)} fns filled`)

phase('Wire')
const decls = [...new Set(reports.flatMap(r => r.mod_decls_needed || []))]
await agent(
`SERIAL WIRING for the client C-track pass. Worktree ${WT} (branch client-cpass) ONLY.
1. Wire every reported mod decl into its crate root (dedupe; match each lib.rs's existing style/order): ${JSON.stringify(decls).slice(0, 7000)}
2. Receiver aggregates: the Client carrier exists in the tree (crates/mp/engine/client/src/client_host.rs, ticket #18 spine). If ported signatures name a receiver aggregate that does not exist yet, pre-seed it as a documented empty pub struct with Default — EXCEPT SoundSystem-internal splits and FX types (their lanes own them; leave dangling, integration reconciles).
3. Ensure each crate root that gained Raven-named fn modules carries the #![allow(non_snake_case)]-family allows consistent with what mp_game does (crate-level, documented one-liner) — check first, only add if missing.
4. rustfmt parse gate every file you touched; then git add -A && git commit --no-gpg-sign -m "client-cpass: mod wiring + receiver aggregates". No co-author trailer.
${STYLE.replace('NEVER git commit; ', '')}
Return JSON {wired: n, created: [...], problems: [...]}.`,
  { label: 'wire', phase: 'Wire', model: 'opus', schema: { type: 'object', properties: { wired: { type: 'number' }, created: { type: 'array', items: { type: 'string' } }, problems: { type: 'array', items: { type: 'string' } } }, required: ['wired', 'problems'] } })
const syms = [...new Map(reports.flatMap(r => r.missing_symbols || []).map(s => [s.name, s])).values()]
if (syms.length) {
  log(`${syms.length} distinct missing symbols — one batched resolution round`)
  await agent(
`SYMBOL RESOLUTION ROUND (client C-track pass). Worktree ${WT}, branch client-cpass. Porters reported these missing symbols (deduped): ${JSON.stringify(syms).slice(0, 9000)}
For each: (1) grep ${WT}/crates/ — if it exists but is unimported/private, fix visibility/re-export at its canonical home; (2) if genuinely absent, port it faithfully from the oracle (single const/type/helper; house style, Source cite, enum-vs-alias fidelity, one type per file beside subsystem siblings; wire mods); (3) if it is an FX_* symbol, a SoundSystem-internal split, a C_MP3_* decoder call, or a CDraw32/CTerrainMap method, SKIP — a pending lane owns it (tickets #26/#24/#25/#29). NEVER modify a ported fn body or signature. Commit at the end: git add -A && git commit --no-gpg-sign -m "client-cpass: symbol resolution round". No co-author trailer.
Return JSON {resolved: n, skipped_lane: [...], unresolved: [...]}.`,
    { label: 'symbols', phase: 'Wire', model: 'opus', effort: 'high', schema: { type: 'object', properties: { resolved: { type: 'number' }, skipped_lane: { type: 'array', items: { type: 'string' } }, unresolved: { type: 'array', items: { type: 'string' } } }, required: ['resolved', 'unresolved'] } })
}

phase('Triage')
const triage = await agent(
`In ${WT}: cargo check --workspace --message-format=short 2>&1 (expected NOT green — the FX/snd/mp3/automap lanes are absent by design). Census the errors: total count, count by error code, count by crate, top 12 files by error count, and how many errors reference the known pending-lane symbols (FX_*, C_MP3_*, CDraw32, CTerrainMap, SoundSystem internals). Return JSON {total_errors, by_code: {}, by_crate: {}, top_files: [{file, errors}], lane_referencing: n}. No fixes, no prose.`,
  { label: 'triage', phase: 'Triage', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: { total_errors: { type: 'number' }, by_code: { type: 'object' }, by_crate: { type: 'object' }, top_files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } }, lane_referencing: { type: 'number' } }, required: ['total_errors'] } })

const notes = reports.flatMap(r => (r.port_notes || []).map(n => ({ ...n, file: r.file })))
return {
  totals: {
    groups: reports.length,
    fns_filled: reports.reduce((a, r) => a + (r.fns_filled || 0), 0),
    port_notes: notes.length,
    missing_symbols: syms.length,
    shape_mismatches: reports.flatMap(r => r.shape_mismatches || []).length,
    triage_errors: triage ? triage.total_errors : null,
  },
  triage,
  port_notes_sample: notes.slice(0, 40),
  shape_mismatches: reports.flatMap(r => (r.shape_mismatches || []).map(m => ({ ...m, file: r.file }))).slice(0, 40),
}
