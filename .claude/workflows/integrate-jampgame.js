export const meta = {
  name: 'integrate-jampgame',
  description: 'Integrate phase for the jampgame pass-3 port: drive mp_game from ~5,600 compile errors to green via triage -> bounded parallel fix rounds (delta tripwire) -> serial finisher. Fixers repair CALL SITES and mechanical mismatches only; they never change ported logic.',
  whenToUse: 'Launch AFTER pass-3 porters have landed all function bodies (mp_bg green, mp_game a large error tail). This is the INTEGRATE-only run: it does no porting, only symbol/call-site/mechanical resolution against the settled rulings. Worktree must be on branch skeleton with porter output committed.',
  phases: [
    { title: 'Triage', detail: 'one agent runs cargo check, writes a machine-readable per-file error inventory, and bins files into ~GROUPS balanced groups keeping same-subsystem files together' },
    { title: 'Fix rounds', detail: 'up to MAX_ROUNDS rounds; each round runs one parallel fixer per non-empty group (opus for the two heaviest, sonnet else), commits the round serially, then re-inventories; DELTA TRIPWIRE stops the loop if a round cuts errors <3% or increases them' },
    { title: 'Finisher', detail: 'when the tail is small (< FINISHER_THRESHOLD) or rounds are exhausted, one serial opus finisher drives the whole remaining list to green (cross-cutting fixes allowed), then cargo check --workspace with mp_bg/mp_qshared required green' },
    { title: 'Report', detail: 'per-round totals, final error count, commits made, and the consolidated blocked list (items needing a ruling / logic port, never retried)' },
  ],
}

// args: { worktree?, maxRounds?, groups?, finisherThreshold? }
const WT = (args && args.worktree) || '/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/agent-a43cc53200d2fdf54'
const MAX_ROUNDS = (args && args.maxRounds) || 5
const GROUPS = (args && args.groups) || 9
const FINISHER_THRESHOLD = (args && args.finisherThreshold) || 150
// Inventory JSON lands under target/ (always gitignored) so `git add -A` never sweeps it.
const INV_DIR = `${WT}/target/integrate`

// ---- contract assets carried into EVERY fixer prompt (the porter/fixer contract) ----
const CONTRACT_ASSETS = `CONTRACT ASSETS — READ THESE FIRST (they are LAW; do not re-derive from memory):
- Rosetta Stone: ${WT}/docs/porting/rosetta.md, the "## EXAMPLE SYNTAX" section — canonical mapped shapes for the vec3/q_math macro family (Vector*, DotProduct, CrossProduct, VectorNormalize, … ; assignment forms are _-prefixed: _VectorCopy/_VectorSubtract/_VectorAdd/_VectorScale/_VectorMA/_DotProduct), va()/printf, cstr helpers. Use these EXACT shapes at call sites; never invent an alternative.
- Rulings: ${WT}/docs/handoffs/jampgame-fork-discovery.md — the "Pass-3 design session rulings (2026-07-04)" and "Landing-pass rulings (2026-07-04)" sections. Key idioms you WILL hit:
  * EntityId arena-resolve idiom: stored entity fields are EntityId; resolve to a &/&mut gentity_t through the arena, assign/compare ids not pointers.
  * vec3 out-params: q_math helpers take by-value inputs and Option<&mut vec3_t> / &mut outputs — match the reshaped signatures shown in the packet/rosetta.
  * gentity_t.m_pVehicle stays c_void per the landing-pass deferral: a bare m_pVehicle deref needs the overlay cast idiom (cast the c_void ptr to the concrete vehicle type at the use site); never change the field type.
- No-shim rule: NEVER define a local helper/shim fn to paper over a missing symbol. If a free-standing const/type/fn is genuinely missing, port it to its canonical home (house style) or add the house marker; struct fields are NEVER missing symbols (wrong receiver type -> use the overlay/cast idiom).
- House TODO markers: unported deps get \`//TODO: Port <exact Raven subject>\` + \`// Source: <oracle path:line>\`. todo!() is allowed ONLY for genuinely-unported logic, with the marker — NEVER to silence a type/call error.
- rustfmt PARSE GATE (mandatory per file): after your last edit to a file, run \`rustfmt --edition 2021 --emit stdout <file> > /dev/null\`; any error means the file does not parse (rustc stops at parse errors and masks all downstream triage). Fix and re-run until clean. This is the ONLY compiler-adjacent command you may run — cargo is FORBIDDEN (parallel fixers share the tree; a stale inventory file is provided instead).`

const KNOWN_DEBT = `KNOWN DEBT the fixers will hit (apply when the file is in your group):
- SFL_*/SVF_* double-import ambiguity (E0659): dedupe at the IMPORT level (one canonical use path; drop the duplicate glob/import), never rename the constants.
- g_weapon.rs: G_Damage call sites are missing the threaded ctx argument — add it per the resolved G_Damage signature (call site bends to the declaration).
- w_saber.rs: \`()\` placeholder per-victim arrays (totalDmg, hitLoc, hitDismember, etc.) are REAL small type ports — port the local arrays faithfully from oracle codemp/game/w_saber.c into the fn as local variables; this is mechanical transcription, not new logic.
- bare m_pVehicle derefs: apply the overlay cast idiom (field stays c_void per deferral).`

const FIXER_CONTRACT = `FIXER CONTRACT — you fix CALL SITES and MECHANICAL mismatches, never ported logic:
1. missing SYMBOL (free-standing const/type/fn) -> re-export it if it exists but is private/unimported (prelude.rs pattern), else port it to its canonical home in house style. Field-access failures are NEVER missing symbols.
2. call-SHAPE mismatch (E0061 arg-count, E0308 type, E0609 field, E0614/E0608 deref/index, E0599 method) -> the DECLARED signature/field is LAW; bend the CALL SITE to it (add args, deref explicitly ((*p).f), turbofish, &/&mut adjust, arena-resolve EntityId, overlay-cast). NEVER edit a declared signature, NEVER rewrite or delete a fn body's logic, NEVER introduce todo!() to silence a type/call error.
3. visibility/import (E0603/E0432/E0659) -> fix pub/use/dedupe at the module boundary.
4. unsafe hygiene (E0133) -> wrap the seam access in the minimal unsafe block, matching surrounding style.
Anything that needs a RULING or a genuine LOGIC PORT (not mechanical) -> DO NOT guess: leave it, and report it in your \`blocked\` list with {file, error, reason}. Blocked items are surfaced to the human and NOT retried.
ANTI-TIME-BOX: you return ONLY when every file in your group has been worked or is genuinely ruling-blocked. "not investigated" / "ran out of budget" are INVALID blocked reasons — work the full inventory, largest files first. Before declaring any symbol/path blocked, grep the worktree for an existing use of it; an existing use IS the answer.
FAQ (verified answers — do not re-derive, do not contradict): the entity arena field is (*ctx.world).g_entities (NOT .entities); trace_t is in the prelude and has no Default — init with \`let mut tr: trace_t = unsafe { core::mem::zeroed() };\`; gNPC_t fields are reached through the NPC pointer with an extra deref (\`(*(*ent).NPC).goalRadius\`); RNG fns live on (*ctx.world).bg_state.rng (game tier) / self.bg.rng (bg tier); gclient access through the c_void client field needs the overlay cast \`(*((*ent).client as *mut gclient_t))\`.`

const STYLE = `NEVER touch oracle/. NEVER run cargo. NEVER add a co-author trailer. Preserve Raven comments; doc-comment + Source cite on any newly-ported item; behavioral parity over prettiness.`

// ---- serialized commit chain (parallel git commits race the index; serialize all commits through one promise) ----
let commitChain = Promise.resolve()
const commits = []
function commit(msg, label) {
  commitChain = commitChain.then(() => agent(
    `In ${WT}: git add -A && git commit -m ${JSON.stringify(msg)} (skip if nothing staged). NEVER add a co-author trailer. Return JSON {commit:"<hash or 'nothing-to-commit'>"}.`,
    { phase: 'Fix rounds', label, model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }
  ).then(r => { if (r && r.commit && r.commit !== 'nothing-to-commit') commits.push({ msg, commit: r.commit }); return r }))
  return commitChain
}

const INV_SCHEMA = { type: 'object', properties: {
  total_errors: { type: 'number' },
  files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } },
  tail: { type: 'string' },
}, required: ['total_errors', 'files', 'tail'] }

// A cargo run that failed to launch (wrong cwd, bad -p spec) produces ZERO grep-able errors —
// indistinguishable from green unless the cargo tail proves the check actually ran.
const CARGO_PROOF = `Run EXACTLY: cd ${WT} && cargo check -p mp_game --message-format=short 2>&1 (the cd is mandatory — a cargo run from any other directory fails instantly and its empty output counts as 0 errors, which is a FALSE GREEN). Return the LAST LINE of the cargo output verbatim as \`tail\` — it must contain either "Finished" (green) or the "could not compile"/"previous errors" summary; if your tail says "did not match any packages" or similar, your cwd is wrong: fix it and re-run before returning.`

// ---- Triage: cargo check -> inventory file + subsystem-balanced groups ----
phase('Triage')
const triage = await agent(
  `TRIAGE for the jampgame integrate run. Worktree ${WT}, branch skeleton.
1. ${CARGO_PROOF} (mp_bg is already green; mp_game carries the tail.)
2. Build a per-file inventory: for each .rs file with errors, its error count, the error codes present (e.g. E0609, E0308, E0061), and 2-3 sample messages.
3. WRITE the full inventory as JSON to ${INV_DIR}/inv-r1.json (mkdir -p ${INV_DIR} first). Shape: {total_errors, files:[{file, errors, codes:[..], samples:[..]}]}. This file is what fixers read.
4. Bin the files into ${GROUPS} groups balanced by TOTAL error count, keeping same-subsystem files together where natural (NPC_AI_* together, bg_* together, the g_weapon/g_combat/w_saber weapon cluster together, ICARUS together, etc.). Every erroring file in exactly one group; give each group a short name.
Return ONLY JSON {total_errors, files:[{file, errors}], groups:[{name, files:[..], errors}]}. No prose.`,
  { phase: 'Triage', label: 'triage', model: 'sonnet', effort: 'low', schema: { type: 'object', properties: {
    total_errors: { type: 'number' },
    files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } },
    groups: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, files: { type: 'array', items: { type: 'string' } }, errors: { type: 'number' } }, required: ['name', 'files'] } },
  }, required: ['total_errors', 'files', 'groups'] } }
)
const groups = triage.groups
log(`Triage: ${triage.total_errors} errors across ${triage.files.length} files, ${groups.length} groups`)

// ---- Fix rounds ----
const roundTotals = [{ round: 0, total: triage.total_errors }]
const blocked = []
let inv = triage
let prevTotal = triage.total_errors
let green = triage.total_errors === 0
let stopReason = green ? 'already-green' : null

phase('Fix rounds')
for (let round = 1; round <= MAX_ROUNDS && !green; round++) {
  const invByFile = new Map((inv.files || []).map(f => [f.file, f.errors]))
  const groupErrors = g => g.files.reduce((a, f) => a + (invByFile.get(f) || 0), 0)
  const active = groups.map(g => ({ ...g, cur: groupErrors(g) })).filter(g => g.cur > 0).sort((a, b) => b.cur - a.cur)
  if (!active.length) { green = true; stopReason = 'green'; break }
  if (active.length + 2 > 16) log(`WARN: ${active.length} active groups approaches the ~16 concurrency cap`)
  const heavy = new Set(active.slice(0, 2).map(g => g.name)) // two heaviest -> opus
  const invPath = `${INV_DIR}/inv-r${round}.json`
  log(`Round ${round}: ${active.length} active groups, ${prevTotal} errors (heaviest: ${active.slice(0, 2).map(g => `${g.name}=${g.cur}`).join(', ')})`)

  const results = await parallel(active.map((g, i) => () => agent(
    `INTEGRATE FIXER — round ${round}, group "${g.name}". Worktree ${WT}, branch skeleton.
YOUR FILES ONLY (${g.cur} errors this round): ${g.files.join(', ')}.
Current error detail is in ${invPath} (JSON {files:[{file,errors,codes,samples}]}) — Read it and take only the entries for your files. Do NOT run cargo; the inventory file is your error source.
Work FILE BY FILE. For each of your files: read the errors, apply the fixes per the contract, then run the rustfmt PARSE GATE on that file before moving on.
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
  )))
  for (const r of results.filter(Boolean)) for (const b of (r.blocked || [])) blocked.push({ round, ...b })
  const roundFixed = results.filter(Boolean).reduce((a, r) => a + (r.fixed || 0), 0)

  // serial commit for the round, then re-inventory (recount for the tripwire + next round)
  await commit(`Integrate round ${round}: ${active.length} groups, ${roundFixed} fixes`, `commit:r${round}`)
  let reInv = await agent(
    `RE-INVENTORY after integrate round ${round}. Worktree ${WT}.
${CARGO_PROOF}
Rebuild the per-file inventory (count + codes + 2-3 samples per erroring file) and WRITE it as JSON to ${INV_DIR}/inv-r${round + 1}.json (mkdir -p if needed).
Return ONLY JSON {total_errors, files:[{file, errors}], tail}. No prose.`,
    { phase: 'Fix rounds', label: `re-triage:r${round}`, model: 'haiku', effort: 'low', schema: INV_SCHEMA }
  )
  if (reInv.total_errors === 0 && !/finished/i.test(String(reInv.tail || ''))) {
    log(`Recount claims 0 errors but cargo tail is "${String(reInv.tail || '').slice(0, 80)}" — refuting with an independent sonnet recount`)
    reInv = await agent(
      `VERIFY a suspicious green claim after integrate round ${round}. Worktree ${WT}.
${CARGO_PROOF}
Rebuild the per-file inventory and WRITE it to ${INV_DIR}/inv-r${round + 1}.json. Return ONLY JSON {total_errors, files:[{file, errors}], tail}.`,
      { phase: 'Fix rounds', label: `re-triage-verify:r${round}`, model: 'sonnet', effort: 'low', schema: INV_SCHEMA }
    )
  }
  inv = reInv
  const newTotal = reInv.total_errors
  roundTotals.push({ round, total: newTotal, fixed: roundFixed })
  const delta = prevTotal - newTotal
  log(`Round ${round} done: ${prevTotal} -> ${newTotal} (-${delta}, ${roundFixed} claimed fixed)`)

  if (newTotal === 0) { green = true; stopReason = 'green'; break }
  // DELTA TRIPWIRE: a round must strictly decrease errors by >=3%; else the residue needs a ruling, not more rounds.
  if (newTotal >= prevTotal * 0.97) { stopReason = `delta-tripwire (round ${round}: ${prevTotal} -> ${newTotal}, <3% reduction)`; log(`DELTA TRIPWIRE: ${stopReason} — stopping the loop`); prevTotal = newTotal; break }
  prevTotal = newTotal
  if (newTotal < FINISHER_THRESHOLD) { stopReason = `small-tail (${newTotal} < ${FINISHER_THRESHOLD})`; log(`Tail below finisher threshold — handing to finisher`); break }
}
if (!green && !stopReason) stopReason = `rounds-exhausted (${MAX_ROUNDS})`

// ---- Finisher: one serial opus agent takes the whole remaining list to green ----
let finisher = null
if (!green) {
  phase('Finisher')
  log(`Finisher: ${prevTotal} errors remain (${stopReason})`)
  finisher = await agent(
    `FINISHER — serial, cross-cutting. Worktree ${WT}, branch skeleton. The parallel rounds stopped with ~${prevTotal} mp_game errors remaining (${stopReason}). Latest per-file inventory: ${INV_DIR}/inv-r${roundTotals.length}.json.
Drive the WHOLE remaining error list to green. You MAY make cross-cutting fixes that span groups/shared files (GameWorld/BgState field merges keeping Raven names, dispatch enums, trap/g_strap wiring, prelude re-exports). The SAME contract applies — call sites bend to declarations, no logic rewrites, no todo!() to silence errors; genuinely-blocked items (needing a ruling / logic port) stay reported, not faked.
${FIXER_CONTRACT}
${KNOWN_DEBT}
${CONTRACT_ASSETS}
${STYLE}
You may run cargo (you are the only writer now) — iterate cargo check -p mp_game until green, parse-gate touched files. FINAL STEP: run \`cargo check --workspace\` to catch cross-crate fallout — mp_bg and mp_qshared MUST be green. Then git add -A && git commit -m "Integrate: jampgame to green" (or "Integrate: finisher stop, <n> errors remain" if not green). NEVER add a co-author trailer.
Return JSON {green, mp_game_errors, workspace_green, mp_bg_green, mp_qshared_green, fixed, commit, blocked:[{file,error,reason}]}.`,
    { phase: 'Finisher', label: 'finisher', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      green: { type: 'boolean' }, mp_game_errors: { type: 'number' }, workspace_green: { type: 'boolean' },
      mp_bg_green: { type: 'boolean' }, mp_qshared_green: { type: 'boolean' }, fixed: { type: 'number' },
      commit: { type: 'string' }, blocked: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' }, reason: { type: 'string' } }, required: ['error'] } },
    }, required: ['green'] } }
  )
  green = !!(finisher && finisher.green)
  if (finisher) {
    if (finisher.commit && finisher.commit !== 'nothing-to-commit') commits.push({ msg: green ? 'Integrate: jampgame to green' : 'Integrate: finisher stop', commit: finisher.commit })
    for (const b of (finisher.blocked || [])) blocked.push({ round: 'finisher', ...b })
    if (typeof finisher.mp_game_errors === 'number') prevTotal = finisher.mp_game_errors
    roundTotals.push({ round: 'finisher', total: finisher.mp_game_errors ?? prevTotal, fixed: finisher.fixed })
  }
}
await commitChain

// ---- Report ----
phase('Report')
// dedupe blocked by file+error
const seen = new Set()
const blockedUniq = blocked.filter(b => { const k = `${b.file || ''}::${b.error}`; if (seen.has(k)) return false; seen.add(k); return true })
const report = {
  green,
  final_mp_game_errors: green ? 0 : prevTotal,
  workspace_green: finisher ? !!finisher.workspace_green : green,
  mp_bg_green: finisher ? finisher.mp_bg_green : true,
  mp_qshared_green: finisher ? finisher.mp_qshared_green : true,
  stop_reason: stopReason,
  round_totals: roundTotals,
  commits,
  blocked: blockedUniq,
}
log(`INTEGRATE DONE: green=${green}, final=${report.final_mp_game_errors} errors, ${commits.length} commits, ${blockedUniq.length} blocked items`)
return report
