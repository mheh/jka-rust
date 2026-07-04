export const meta = {
  name: 'port-jampgame-pass3',
  description: 'Pass 3: zero-park blind transcription of all remaining jampgame fns; porters never pause; memoized symbol-fixers run alongside; integration to green with delta tripwires',
  whenToUse: 'Launch AFTER pass-3 prep: rulings 12-20 implemented (BgState/PmoveContext/BgTraps/GameCallbacks landed + hand slice green), symbol backfill done, bg signature retrofit + EntityId field retrofit applied, packets regenerated into out/pass3/packets with manifest. Porters are blind transcribers: no cargo, no exploration, no parking.',
  phases: [
    { title: 'Port', detail: 'continuous flow, one blind porter per packet; per-file verdicts in code; symbol-fixers memoized by symbol, non-blocking; WIP commit every ~15 files' },
    { title: 'Integrate', detail: 'triage -> parallel per-group fixers (symbol-resolver contract) -> serial finisher with delta tripwire' },
    { title: 'Aggregate', detail: 'totals, PORT-NOTE audit queue, symbols fixed, unresolved report' },
  ],
}

// args: { worktree?, packetsDir?, manifestPath?, maxFiles? }
const WT = (args && args.worktree) || '/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/agent-a43cc53200d2fdf54'
const MAIN = '/Users/milohehmsoth/Developer/Milo/jka-rust'
const PKT = (args && args.packetsDir) || `${MAIN}/tools/closure-prototype/out/pass3/packets`
const MANIFEST = (args && args.manifestPath) || `${MAIN}/tools/closure-prototype/out/pass3/manifest.json`
const MAX_FILES = (args && args.maxFiles) || 999

const STYLE = `HOUSE RULES: doc-comment + Source cite on every item (oracle/oracle/codemp/game/<file>:<lines>); preserve Raven comments that clarify behavior; state conclusions not derivations; behavioral parity — port faithfully even if ugly; UB sites get the one defined behavior + <=2-line note per S19. NEVER touch oracle/. NEVER run cargo (parallel writers share the tree). NEVER git commit. NEVER add a co-author trailer.`

// Zero-park: nothing stops a porter. Rulings 1-20 are all settled (docs/handoffs/jampgame-fork-discovery.md).
const ZERO_PARK = `ZERO-PARK POLICY (supersedes the pass-1/2 park protocol): every fn in your packet gets a real body. NOTHING blocks you:
- Symbol/type/const does not exist yet -> write the call/reference EXACTLY as the packet cites it anyway, and report it in missing_symbols. A fixer ports the symbol; you do not wait, do not stub, do not invent an alternative.
- Genuinely ambiguous logic site -> transcribe the most LITERAL faithful reading and tag the line above with // PORT-NOTE(<topic>): <one line>. Notes are audited later; todo!() is FORBIDDEN in your output except where the packet itself says a fn is deferred-by-ruling (ICARUS internals etc.).
- Cross-file signature looks wrong -> the packet's resolved signature is LAW; write your call site to match it and report the discrepancy in shape_mismatches. Never re-declare, never adjust a callee.`

const DISCIPLINE = `MECHANICS (kills the pass-2 error classes): raw pointers are dereferenced explicitly — (*ent).client, never ent.client; every EntThink/EntUse/EntTouch/EntDie/EntPain/EntBlocked/EntReached use comes with its import line; add 'use crate::trap;' if you call traps; vec3 helpers use the reshaped q_math signatures shown in your packet (by-value inputs, &mut outputs); stored entity fields are EntityId per ruling 17 — assign/compare ids, not pointers; bg-tier bodies use PmoveContext/BgState/BgTraps/GameCallbacks exactly as your packet's threading digest shows; va()/printf family per the packet's mapping table (format!, owned String). WRITE DISCIPLINE: few LARGE Edit calls; touch ONLY fns listed in your packet; skip any fn already implemented.`

const PORT_SCHEMA = { type: 'object', properties: {
  file: { type: 'string' }, fns_filled: { type: 'number' }, fns_deferred_by_ruling: { type: 'number' },
  port_notes: { type: 'array', items: { type: 'object', properties: { fn: { type: 'string' }, topic: { type: 'string' }, note: { type: 'string' } }, required: ['fn', 'topic'] } },
  missing_symbols: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, kind: { type: 'string' }, source: { type: 'string' } }, required: ['name'] } },
  shape_mismatches: { type: 'array', items: { type: 'object', properties: { callee: { type: 'string' }, detail: { type: 'string' } }, required: ['callee'] } },
}, required: ['file', 'fns_filled'] }

phase('Port')
const manifest = await agent(
  `Read ${MANIFEST} (JSON array or {packets:[...]}). Return ONLY JSON {packets:[{file, packet, fns, loc}]} sorted by loc descending. No prose.`,
  { phase: 'Port', label: 'read-manifest', model: 'haiku', effort: 'low', schema: { type: 'object', properties: { packets: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, packet: { type: 'string' }, fns: { type: 'number' }, loc: { type: 'number' } }, required: ['file', 'packet'] } } }, required: ['packets'] } }
)
const files = manifest.packets.slice(0, MAX_FILES)
log(`Pass 3: ${files.length} packets, zero-park blind transcription`)

const OPUS_FILES = new Set(['ai_main.c', 'w_saber.c', 'NPC_AI_Jedi.c', 'bg_pmove.c', 'g_vehicles.c', 'g_combat.c'])
const tierFor = p => OPUS_FILES.has(p.file) || (p.loc || 0) > 3500 ? 'opus' : (p.loc || 0) < 900 ? 'haiku' : 'sonnet'

// ---- non-blocking, memoized symbol fixers (fixer = symbol resolver, never logic) ----
const symbolFixers = new Map()   // symbol name -> promise; N reporters share ONE fixer
const symbolsFixed = []
function fixSymbol(sym) {
  if (!symbolFixers.has(sym.name)) {
    symbolFixers.set(sym.name, agent(
      `SYMBOL FIXER (resolver contract — you resolve symbols, never logic). Worktree ${WT}, branch skeleton. The symbol \`${sym.name}\` (${sym.kind || 'unknown kind'}${sym.source ? ', oracle: ' + sym.source : ''}) is referenced by freshly-ported jampgame bodies but does not resolve.
1. grep the worktree crates/mp/ first: if it EXISTS but is private/unimported -> make it pub and add the re-export where bare references resolve (prelude.rs pattern).
2. If it does not exist -> port it faithfully from the oracle (find it; single const/enum/type/table/helper-fn; house style: doc-comment + Source cite, Raven name, enum-vs-alias fidelity, one type per file beside its subsystem siblings; wire mod decls).
3. NEVER modify a fn body or an existing signature. Call sites bend toward declarations, never the reverse — and you touch neither.
${STYLE} Return JSON {name: "${sym.name}", action: "exported"|"ported"|"already-ok"|"is-state-not-const", path: "<rust file>"}.`,
      { label: `sym:${sym.name}`, phase: 'Port', model: 'haiku', effort: 'low', schema: { type: 'object', properties: { name: { type: 'string' }, action: { type: 'string' }, path: { type: 'string' } }, required: ['name', 'action'] } }
    ).then(r => { if (r) symbolsFixed.push(r); return r }))
  }
  return symbolFixers.get(sym.name)
}

// ---- serialized WIP committer (no parallel git races) ----
let commitChain = Promise.resolve()
let sinceCommit = 0, committed = 0
function maybeCommit(force) {
  sinceCommit++
  if (!force && sinceCommit < 15) return
  const n = ++committed; sinceCommit = 0
  commitChain = commitChain.then(() => agent(
    `In ${WT}: git add -A && git commit -m "Pass-3 WIP ${n}: porter output (pre-integration, NOT green)". NEVER add a co-author trailer. Return JSON {commit: "<hash or 'nothing-to-commit'>"}.`,
    { label: `wip-commit-${n}`, phase: 'Port', model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }
  ))
}

// ---- continuous flow: porter -> verdict (plain code) -> optional single retry; NEVER waits on other files ----
const reports = [], anomalies = []
const results = await parallel(files.map(p => async () => {
  if (budget.total && budget.remaining() < 150_000) { log(`BUDGET GUARD: skipping ${p.packet}`); return null }
  const model = tierFor(p)
  const prompt =
`Pass-3 BLIND PORTER for ${p.packet} (${p.fns} fns, ${p.loc} LOC of ${p.file}). Worktree ${WT}, branch skeleton.
YOUR ENTIRE INPUT: (1) packet ${PKT}/${p.packet} — rulings digest (LAW: forks 1-20 all settled), cited oracle source for your fns, final resolved Rust signatures of everything you call, threading digest (ctx/PmoveContext/BgTraps/GameCallbacks per fn), state field map, va/printf mapping table; (2) your module under ${WT}/crates/mp/game/src/. Read nothing else, explore nothing, never run cargo.
${ZERO_PARK}
${DISCIPLINE}
${STYLE}
Return JSON: {file, fns_filled, fns_deferred_by_ruling, port_notes:[{fn,topic,note}], missing_symbols:[{name,kind,source}], shape_mismatches:[{callee,detail}]}`
  let r = await agent(prompt, { label: `port:${p.packet}`, phase: 'Port', model, effort: model === 'opus' ? 'medium' : 'low', schema: PORT_SCHEMA })

  // verdict in plain code — observer, not gate
  const expected = p.fns || 0
  if (r && expected > 3 && (r.fns_filled + (r.fns_deferred_by_ruling || 0)) < expected * 0.5) {
    anomalies.push({ packet: p.packet, filled: r.fns_filled, expected })
    log(`ANOMALY ${p.packet}: ${r.fns_filled}/${expected} filled — one retry`)
    r = await agent(prompt + `\nRETRY NOTE: a prior attempt filled only ${r.fns_filled}/${expected} fns. Fill EVERY fn listed in the packet that still has a todo!() body.`,
      { label: `retry:${p.packet}`, phase: 'Port', model, effort: 'medium', schema: PORT_SCHEMA }) || r
  }
  if (r) {
    reports.push(r)
    // fire-and-share symbol fixers; do NOT await — porters never pause
    for (const s of (r.missing_symbols || [])) fixSymbol(s)
    const topics = {}
    for (const n of (r.port_notes || [])) topics[n.topic] = (topics[n.topic] || 0) + 1
    for (const t in topics) if (topics[t] >= 8) log(`NOTE-CLUSTER ${p.packet}: PORT-NOTE(${t}) x${topics[t]} — audit candidate`)
    maybeCommit(false)
  }
  return r
}))
await Promise.all([...symbolFixers.values()])
maybeCommit(true)
await commitChain
const done = results.filter(Boolean)
log(`Port done: ${done.length}/${files.length} packets, ${symbolsFixed.length} symbols resolved alongside, ${anomalies.length} anomalies retried`)

phase('Integrate')
const SHARED = 'crates/mp/game/src/{world/,bg_channel/,game_globals.rs,ent_fn_enums.rs,trap.rs,g_strap.rs,lib.rs,prelude.rs}'
const triage = await agent(
  `In ${WT}: run cargo check --workspace --message-format=short 2>&1. Group errors by .rs file. Return ONLY JSON {total_errors, groups:[{files:[...], errors:n}]} — 6-10 groups of roughly equal error count, each file in exactly one group; put errors in shared files (${SHARED}) into files:["__shared__"]. No prose.`,
  { phase: 'Integrate', label: 'triage', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: { total_errors: { type: 'number' }, groups: { type: 'array', items: { type: 'object', properties: { files: { type: 'array', items: { type: 'string' } }, errors: { type: 'number' } }, required: ['files'] } } }, required: ['total_errors', 'groups'] } }
)
log(`Triage: ${triage.total_errors} errors, ${triage.groups.length} groups`)

const FIXER_CONTRACT = `FIXER CONTRACT: you resolve SYMBOLS and mechanics, never logic. (1) missing symbol -> port/re-export it. (2) call-shape mismatch -> call sites bend toward declarations, NEVER edit a declared signature or fn body's logic. (3) mechanical: use lines, derefs ((*p).f), turbofish, &mut adjustments. Anything semantic -> add // PORT-NOTE(<topic>) and leave it compiling-wrong is NOT allowed — if you cannot fix mechanically, leave the error and list it in your return. ${STYLE}`

await parallel(triage.groups.filter(g => g.files[0] !== '__shared__').map((g, i) => () => agent(
  `Parallel integration fixer ${i + 1}. Worktree ${WT}. YOUR FILES ONLY: ${g.files.join(', ')}. cargo check --workspace --message-format=short 2>&1, take only errors in your files, fix per contract. If a fix requires editing a shared file (${SHARED}), SKIP and list it. ${FIXER_CONTRACT} Do NOT commit. Return JSON {fixed, skipped_shared:[{file,error}], unfixable:[{file,error}]}.`,
  { label: `fix:g${i + 1}`, phase: 'Integrate', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: { fixed: { type: 'number' }, skipped_shared: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' } }, required: ['error'] } }, unfixable: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' } }, required: ['error'] } } }, required: ['fixed'] } }
)))

// serial finisher with DELTA TRIPWIRE: a round that barely moves = design hole, stop instead of inventing
let green = false, lastErrors = triage.total_errors > 0 ? triage.total_errors : 999999
for (let round = 1; round <= 4 && !green; round++) {
  const fin = await agent(
    `Final integration fixer, round ${round}. Worktree ${WT}. cargo check --workspace 2>&1; fix ALL remaining errors incl. shared files (GameWorld/BgState field merges keep Raven names + dedupe; dispatch enums; trap/g_strap wiring). ${FIXER_CONTRACT}
When green: git add -A && git commit -m "Pass-3 checkpoint: integration green". NEVER add a co-author trailer. Return JSON {green, errors_remaining, errors_fixed, commit}.`,
    { label: `finish:r${round}`, phase: 'Integrate', model: round <= 2 ? 'sonnet' : 'opus', schema: { type: 'object', properties: { green: { type: 'boolean' }, errors_remaining: { type: 'number' }, errors_fixed: { type: 'number' }, commit: { type: 'string' } }, required: ['green'] } }
  )
  green = !!(fin && fin.green)
  const rem = fin && typeof fin.errors_remaining === 'number' ? fin.errors_remaining : null
  log(`Finish r${round}: ${green ? 'GREEN' : `${rem ?? '?'} errors remain`}`)
  if (!green && rem !== null) {
    if (rem > lastErrors * 0.8) { log(`DELTA TRIPWIRE: round ${round} reduced errors by <20% (${lastErrors} -> ${rem}) — stopping; residue needs a human/ruling, not more fixer rounds`); break }
    lastErrors = rem
  }
}
if (!green) await agent(
  `In ${WT}: git add -A && git commit -m "Pass-3 WIP: integration stopped pre-green (delta tripwire) — residue in report". NEVER add a co-author trailer. Return JSON {commit:"<hash>"}.`,
  { phase: 'Integrate', label: 'tripwire-commit', model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }
)

phase('Aggregate')
const notes = reports.flatMap(r => (r.port_notes || []).map(n => ({ ...n, file: r.file })))
const mismatches = reports.flatMap(r => (r.shape_mismatches || []).map(m => ({ ...m, file: r.file })))
const totals = {
  packets: done.length,
  fns_filled: reports.reduce((a, r) => a + (r.fns_filled || 0), 0),
  fns_deferred_by_ruling: reports.reduce((a, r) => a + (r.fns_deferred_by_ruling || 0), 0),
  symbols_resolved: symbolsFixed.length,
  port_notes: notes.length,
  shape_mismatches: mismatches.length,
  anomalies_retried: anomalies.length,
  integration_green: green,
}
log(`PASS 3 DONE: ${totals.fns_filled} filled, ${totals.port_notes} notes to audit, green=${green}`)
return { totals, port_notes: notes, shape_mismatches: mismatches, symbols_fixed: symbolsFixed, anomalies }
