export const meta = {
  name: 'integrate-client-cpass2',
  description: 'Wave 2 on branch client-cpass: build the client hosting spine (DEC-55), land the DEC-59 seams (direct RE_* calls, flate2 inflate mirror), compute the receiver signature law, rewire all ten client files in parallel, then fix rounds to a green workspace.',
  whenToUse: 'After integrate wave 1 (wf_00cecbd4-fcc): qcommon green, 1,481 mp_engine_client errors dominated by free common/view/host identifiers (~90 fns need receivers), the cl_renderer placeholder (66 sites), G2API renames (55 sites), and small genuine ports.',
  phases: [
    { title: 'Foundations', detail: 'three parallel builders on disjoint files: hosting spine (dispatch ctxs, trampolines, cl_from_view, Cmd adapters), the zlib inflate mirror, and the genuine symbol ports at canonical homes' },
    { title: 'Law', detail: 'two read-only computers emit JSON law: the per-fn receiver signature table and the renderer/G2API call mappings' },
    { title: 'Rewire', detail: 'one agent per client file applies the signature law and mappings to its file only; a merge agent adds the flat Client globals' },
    { title: 'Rounds', detail: 'inventory -> parallel per-file fixers, up to 3 rounds with the cascade-aware tripwire' },
    { title: 'Finisher', detail: 'serial opus finisher: residue, cinTable fn pointers, marker sweep, drives cargo check --workspace to green' },
    { title: 'Report', detail: 'totals, commits, blocked list' },
  ],
}

// Config is HARDCODED (workflow-args string bug): relaunch via scriptPath after edits.
const WT = '/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/client-cpass'
const MAIN = '/Users/milohehmsoth/Developer/Milo/jka-rust'
const ORACLE = `${MAIN}/oracle` // the worktree has no submodule checkout; read the oracle here, never edit it
const MAX_ROUNDS = 3
const FINISHER_THRESHOLD = 150
const INV_DIR = `${WT}/target/integrate2`

const CLIENT_DIR = 'crates/mp/engine/client/src'
const CLIENT_FILES = ['cl_main.rs', 'cl_cgame.rs', 'cl_input.rs', 'cl_cin.rs', 'cl_ui.rs', 'cl_parse.rs', 'cl_console.rs', 'cl_keys.rs', 'cl_scrn.rs', 'cl_net_chan.rs']
const CLIENT_HOST = `${CLIENT_DIR}/client_host.rs`
const ENGINE_RS = 'crates/mp/engine/core/src/engine.rs'

// ---- the exact inventory command every inventory agent runs (deterministic parse) ----
function invCmd(n) {
  return `Run EXACTLY (the cd is mandatory — cargo from any other directory fails instantly and its empty output is a FALSE GREEN):
cd ${WT} && mkdir -p target/integrate2 && cargo check --workspace --message-format=short 2>&1 | tee target/integrate2/raw-r${n}.txt | tail -3
Then run this EXACT python (substituting nothing — it is complete):
cd ${WT} && python3 - <<'PYEOF'
import re, json, collections
raw = open('target/integrate2/raw-r${n}.txt').read().splitlines()
pat = re.compile(r'^(.+?\\.rs):\\d+:\\d+:\\s+error(\\[(E\\d+)\\])?:\\s*(.*)$')
files = collections.defaultdict(lambda: {'errors':0,'codes':collections.Counter(),'samples':[]})
total = 0
for ln in raw:
    m = pat.match(ln)
    if not m: continue
    f, code, msg = m.group(1), m.group(3) or 'other', m.group(4)
    total += 1
    d = files[f]; d['errors'] += 1; d['codes'][code] += 1
    if len(d['samples']) < 6: d['samples'].append(ln[:300])
inv = {'total_errors': total,
 'files': [{'file':f,'errors':d['errors'],'codes':sorted(d['codes']),'samples':d['samples']} for f,d in sorted(files.items(), key=lambda kv:-kv[1]['errors'])]}
json.dump(inv, open('target/integrate2/inv-r${n}.json','w'), indent=1)
print(total, 'errors across', len(files), 'files')
PYEOF
Your returned \`tail\` must be the last line of the cargo output (from the tee'd tail -3): either "Finished" (green) or the "could not compile"/"previous errors" summary. If it says "did not match any packages" or "no such file", your cwd was wrong — fix and re-run before returning.`
}

// ---- shared contract text ----
const CARRIER_RULE = `CLIENT CARRIER RULE:
\`Client\` (${CLIENT_HOST}) is the island carrier. Raven's five client globals are boxed fields on it: cl.cl (clientActive_t), cl.clc (clientConnection_t), cl.cls (clientStatic_t), cl.kg (keyGlobals_t), cl.con (console_t). Boxes auto-deref.
- E0609 "no field \`X\` on \`Client\`" where X is a MEMBER of one of the five aggregate types -> insert the aggregate hop: cl.snap -> cl.cl.snap, cl.state -> cl.cls.state. Your fix.
- E0425 bare \`clc\`/\`cls\`/\`kg\`/\`con\` -> cl.clc / cl.cls / cl.kg / cl.con.
- E0609 on \`Client\` where X is a FILE-SCOPE global or hoisted static (cvar handles, CIN_* statics, cl_main statics) -> the merge lane adds that field to Client; skip it.
- The five aggregate types are ABI-frozen with layout asserts. NEVER add a field to them.`

const RULINGS = `RULINGS IN FORCE (settled — never re-litigate):
- DEC-55 (docs/decisions.md): the client hosting seam. Trap ownership map, VmSlot::Uivm, copy-out seams transcribe in the dispatchers.
- DEC-59.1: engine-interior \`re.X(...)\` sites call the mp_renderer RE_* frontend DIRECTLY with their declared receivers. No refexport_t, no GetRefAPI, no REF_API_VERSION. The \`cl_renderer\` placeholder module never gets built — every \`crate::cl_renderer::re(common).X(args)\` site is rewritten per the mappings law.
- DEC-59.2: CL_ParseRMG inflate goes through mp_engine_qcommon::zlib_seam::inflate_sync_flush (flate2 Decompress). No z_stream port.
- Ruling 40: G2 API exports are snake_case g2api_* in mp_engine_ghoul2 (api_bolts, api_bones, api_collision, api_models, api_ragdoll, api_saveload, api_surfaces modules) with threaded g2 + host receivers.
- Ruling 1: Com_Error call sites are panics / com_error. Ruling 21: rand-family through common.qrand, never libc rand.
- Ruling 33b: hook_install.rs routes VmSlot::Cgvm | VmSlot::Uivm to null — NEVER change it; the client dispatch slots arm elsewhere (the spine).
- Pending lanes stay stubs: S_* (snd_stubs.rs, gh#24), FX_* (fx_stubs.rs, gh#26/#27), C_MP3_* (gh#25), CTerrainMap bodies (gh#29). Call the stubs; never implement them.`

const STYLE = `NEVER touch ${ORACLE} (read-only oracle) or ${MAIN} outside the worktree. NEVER git reset/clean/checkout — the worktree history is load-bearing. NEVER add a co-author trailer. All commits use --no-gpg-sign. Preserve Raven comments; doc-comment + Source cite on newly-added items. New comment prose is STE: active voice, full sentences, no semicolons, one sentence per line.
rustfmt PARSE GATE (mandatory per touched file): \`rustfmt --edition 2021 --emit stdout <file> > /dev/null\` must pass — the ONLY compiler-adjacent command you may run unless your prompt explicitly grants cargo.`

const NO_REWRITE = `You fix signatures, call sites, imports, and wiring — NEVER ported logic. NEVER rewrite or delete a fn body's logic, NEVER introduce todo!() to silence a type error, NEVER define a local shim for a missing symbol. Anything needing a new RULING or a genuine logic port -> report it in \`blocked\` with {file, error, reason}. ANTI-TIME-BOX: "ran out of budget" is an INVALID blocked reason.`

// ---- serialized commit chain ----
let commitChain = Promise.resolve()
const commits = []
function commit(msg, label, phase) {
  commitChain = commitChain.then(() => agent(
    `In ${WT}: git add -A && git commit --no-gpg-sign -m ${JSON.stringify(msg)} (skip if nothing staged). NEVER add a co-author trailer. Return JSON {commit:"<short hash or 'nothing-to-commit'>"}.`,
    { phase, label, model: 'haiku', effort: 'low', schema: { type: 'object', properties: { commit: { type: 'string' } }, required: ['commit'] } }
  ).then(r => { if (r && r.commit && r.commit !== 'nothing-to-commit') commits.push({ msg, commit: r.commit }); return r }))
  return commitChain
}

const blocked = []
const collect = (r, tag) => { if (r) for (const b of (r.blocked || [])) blocked.push({ stage: tag, ...b }) }

// ================= Phase 1: Foundations =================
phase('Foundations')
log('Foundations: spine + zlib + symbol ports on disjoint files')

const BLOCKED_ITEMS = { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, error: { type: 'string' }, reason: { type: 'string' } }, required: ['reason'] } }

const [spine, zlib, symbols] = await parallel([
  () => agent(
    `BUILD THE CLIENT HOSTING SPINE — the DEC-55 core. Worktree ${WT}, branch client-cpass. Serial owner of: ${CLIENT_HOST}, the trampoline/dispatcher entry regions of ${CLIENT_DIR}/cl_cgame.rs and ${CLIENT_DIR}/cl_ui.rs, and whatever mp_engine_core / mp_host_interface wiring the spine needs. Do NOT touch the transcribed fn bodies outside those regions.
PRECEDENT IS LAW — READ FIRST: crates/mp/engine/server/src/sv_game.rs (game_system_calls_shim, the thread-local/boot-armed GameDispatchCtx, the EngineSlot{ctx, syscall} injection through ModuleRegistry::load_module) and crates/mp/engine/server/src/server_host.rs. Mirror that mechanism for the client:
1. CgameDispatchCtx and UiDispatchCtx twins (or one shared client ctx if the precedent shape allows), armed at module load, recovered in the trampolines. Replace the accepted todo! in CL_CgameSystemCalls_trampoline and make CL_UISystemCalls_trampoline recover its receivers the same way. hook_install.rs stays untouched (ruling 33b).
2. cl_from_view: the EngineHostView -> &mut Client accessor, the twin of sv_from_view (see sv_ccmds.rs:1024 for the adapter idiom).
3. Cmd_AddCommand adapters: the tree's CmdFunction is fn(&mut EngineHostView). Provide the <Name>_cmd adapter shape for client handlers (adapters live beside their handlers; add the ones cl_console/cl_main/cl_ui register — 7 known sites in cl_console.rs plus the cl_main/cl_ui registrations).
4. RENDERER REACH (DEC-59.1 prerequisite): decide and land how client code reaches the mp_renderer frontend receivers (FrameData / RenderModels) from EngineHostView/EngineHost, respecting DEC-55.2's state-partition law (synchronous paths read CPU RenderAssets only). Grep how the view exposes rm today (crates/mp/renderer/src uses EngineHostView already). Document the reach in the view type's doc comment — the Law phase reads it.
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT git commit. Return JSON {summary, files_touched:[..], renderer_reach:"<one-paragraph description of how client fns obtain the renderer receivers>", blocked:[..]}.`,
    { label: 'spine', phase: 'Foundations', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      summary: { type: 'string' }, files_touched: { type: 'array', items: { type: 'string' } }, renderer_reach: { type: 'string' }, blocked: BLOCKED_ITEMS,
    }, required: ['summary', 'files_touched', 'renderer_reach'] } }
  ),
  () => agent(
    `LAND DEC-59.2 — the flate2 inflate mirror. Worktree ${WT}, branch client-cpass. You own EXACTLY two files: crates/mp/engine/qcommon/src/zlib_seam.rs and ${CLIENT_DIR}/cl_parse.rs (ONLY its two CL_ParseRMG inflate blocks).
1. Add \`pub fn inflate_sync_flush(src: *const u8, avail_in: c_int, out: &mut [u8]) -> c_int\` beside deflate_sync_flush, backed by flate2's Decompress (zlib-wrapped), returning total_out. Mirror the existing fn's doc style and Source cite (oracle/codemp/client/cl_parse.cpp:465-521 — read it at ${ORACLE}/codemp/client/cl_parse.cpp).
2. Bend the two inflate blocks in cl_parse.rs::CL_ParseRMG to call it (heightmap and flatten map), removing the z_stream/inflateInit/inflate/inflateEnd references and the stale PORT-NOTE about the missing seam. Keep Raven's surrounding logic byte-faithful (sizes, total_out write-back, the avail_in quirk on the flatten arm — Raven passes clc.rmgHeightMapSize there, not size; PRESERVE that bug faithfully with a 2-line comment).
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT git commit. Return JSON {summary, files_touched:[..], blocked:[..]}.`,
    { label: 'zlib', phase: 'Foundations', model: 'sonnet', effort: 'medium', schema: { type: 'object', properties: {
      summary: { type: 'string' }, files_touched: { type: 'array', items: { type: 'string' } }, blocked: BLOCKED_ITEMS,
    }, required: ['summary', 'files_touched'] } }
  ),
  () => agent(
    `GENUINE SYMBOL PORTS at canonical homes. Worktree ${WT}, branch client-cpass. The wave-1 finisher confirmed these are absent from the whole workspace. Port each to its canonical home (house style, Source cite from ${ORACLE}, enum-vs-alias fidelity, one type per file where the type rule applies) and wire the mod/pub. FORBIDDEN: editing any of the ten ${CLIENT_DIR}/cl_*.rs files or ${CLIENT_HOST} — the Rewire phase wires the imports there.
The list (grep the workspace FIRST for each — if one exists after all, just fix its visibility/re-export): svc_strings (cl_parse.cpp server-command name table), Info_NextPair (q_shared), ColorIndex (q_shared macro), FONT_INDEX_ASIAN_NOTIFY (cl_console/cl_fonts constant), K_CHAR_FLAG (keycodes), Sys_GetClipboardData (a null/none-returning faithful mac arm is fine if the win32 arm is the only oracle body — cite and note), Sys_ShowIP, Sys_MonkeyShouldBeSpanked (client.h/cl_main.cpp), SE_CheckForLanguageUpdates (stringed pkg, ruling 50 home), PerpendicularVector (q_math — almost certainly already ported, check crates/native/math first), sys_begin_streamed_file/sys_end_streamed_file (check the server/common sys layer first).
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT git commit. Return JSON {summary, ported:[..], already_existed:[..], blocked:[..]}.`,
    { label: 'symbols', phase: 'Foundations', model: 'sonnet', effort: 'medium', schema: { type: 'object', properties: {
      summary: { type: 'string' }, ported: { type: 'array', items: { type: 'string' } }, already_existed: { type: 'array', items: { type: 'string' } }, blocked: BLOCKED_ITEMS,
    }, required: ['summary'] } }
  ),
])
collect(spine, 'spine'); collect(zlib, 'zlib'); collect(symbols, 'symbols')
if (!spine) { log('SPINE FAILED — aborting: the Law and Rewire phases depend on it'); return { green: false, aborted: 'spine agent returned null', blocked } }
log(`Foundations done. Renderer reach: ${String(spine.renderer_reach || '').slice(0, 200)}`)
await commit('client-cpass w2: hosting spine + DEC-59 seams + symbol ports', 'commit:foundations', 'Foundations')

// ================= Phase 2: Law =================
phase('Law')
const LAW_NOTE = `Spine's renderer reach (from the builder itself): ${spine.renderer_reach}`

const [siglaw, mappings] = await parallel([
  () => agent(
    `COMPUTE THE RECEIVER SIGNATURE LAW. Worktree ${WT}, branch client-cpass. READ-ONLY except your output file. Write ${INV_DIR}/siglaw.json.
The transcribed client fns reference \`common\`/\`view\`/\`host\`/renderer state as free identifiers (~90 fns; see ${WT}/target/integrate/raw-r3.txt for every E0425 site). Compute, for EVERY fn in the ten ${CLIENT_DIR}/cl_*.rs files that needs receivers threaded, its FINAL signature:
- Pinned receiver order: common: &mut Common, cm: &mut CollisionWorld, sv: &mut Server, cl: &mut Client, g2: &mut Ghoul2System, host: &mut dyn EngineHost — host last.
- COLLAPSE RULE (the wave-1 finisher's law): a fn needing BOTH common AND host — or needing the renderer reach — takes \`view: &mut EngineHostView\` instead and reads view.common / the renderer state through the view (verify the exact field spellings against the spine's landed code in ${CLIENT_HOST} and crates/mp/host-interface). &mut Common + &mut dyn EngineHost alias, so they can never be siblings.
- The closure is transitive: a fn inherits the receivers of every in-crate callee. The finisher already threaded cl_net_chan.rs and the cl_parse.rs entry points — treat those signatures as fixed reference points.
- Do NOT change fns that already compile.
Output shape: {"fns":[{"name","file","signature","note"}], "conventions":{"view_reach":"<how view yields common/cl/renderer, exact expressions>"}}. Also return that same JSON.`,
    { label: 'siglaw', phase: 'Law', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      fns: { type: 'array', items: { type: 'object', properties: { name: { type: 'string' }, file: { type: 'string' }, signature: { type: 'string' }, note: { type: 'string' } }, required: ['name', 'file', 'signature'] } },
      conventions: { type: 'object', properties: { view_reach: { type: 'string' } }, required: ['view_reach'] },
    }, required: ['fns', 'conventions'] } }
  ),
  () => agent(
    `COMPUTE THE CALL MAPPINGS LAW. Worktree ${WT}, branch client-cpass. READ-ONLY except your output file. Write ${INV_DIR}/mappings.json with two tables:
1. RENDERER (DEC-59.1): every distinct \`crate::cl_renderer::re(...)\` method used in the ten cl_*.rs files (grep them) -> the ported mp_renderer fn (grep crates/mp/renderer/src for RE_* / Language_IsAsian / Font_* twins), its full signature, and its rust path. A method with NO ported twin gets {"unported": true} and the oracle cite (the renderer census, gh#31, owns it later — the rewire agent leaves a house marker).
2. G2API (ruling 40): every G2API_* identifier used in cl_cgame.rs -> the snake_case g2api_* fn in crates/mp/engine/ghoul2 (grep its api_* modules), full signature, rust path. For the eight with no direct name twin (G2API_CopySpecificG2Model, G2API_HaveWeGhoul2Models, G2API_ListBones, G2API_ListSurfaces, G2API_OverrideServerWithClientData, G2API_ResetRagDoll, G2API_SetGhoul2ModelIndexes, CRagDollUpdateParams) search by behavior (read the oracle body at ${ORACLE}/codemp/ghoul2 and the crate's fn docs); genuinely absent -> {"unported": true} + cite.
${LAW_NOTE}
Output/return shape: {"renderer":[{"method","target","signature","path","unported"}], "g2api":[{"raven","target","signature","path","unported"}]}.`,
    { label: 'mappings', phase: 'Law', model: 'sonnet', effort: 'medium', schema: { type: 'object', properties: {
      renderer: { type: 'array', items: { type: 'object', properties: { method: { type: 'string' }, target: { type: 'string' }, signature: { type: 'string' }, path: { type: 'string' }, unported: { type: 'boolean' } }, required: ['method'] } },
      g2api: { type: 'array', items: { type: 'object', properties: { raven: { type: 'string' }, target: { type: 'string' }, signature: { type: 'string' }, path: { type: 'string' }, unported: { type: 'boolean' } }, required: ['raven'] } },
    }, required: ['renderer', 'g2api'] } }
  ),
])
if (!siglaw || !mappings) { log('LAW PHASE FAILED — aborting before Rewire'); return { green: false, aborted: 'law agent returned null', blocked } }
log(`Law: ${siglaw.fns.length} signatures, ${mappings.renderer.length} renderer mappings (${mappings.renderer.filter(m => m.unported).length} unported), ${mappings.g2api.length} g2api mappings (${mappings.g2api.filter(m => m.unported).length} unported)`)

// ================= Phase 3: Rewire =================
phase('Rewire')
const HEAVY = new Set(['cl_main.rs', 'cl_cgame.rs', 'cl_input.rs'])
const rewireResults = await parallel(CLIENT_FILES.map(f => () => agent(
  `REWIRE ${CLIENT_DIR}/${f} — you own THIS ONE FILE and edit nothing else. Worktree ${WT}, branch client-cpass.
LAW FILES (read both first): ${INV_DIR}/siglaw.json (final signatures — apply every entry whose "file" is ${f} to the fn DEFINITIONS here, and bend every CALL SITE in this file to the callee's law signature, wherever the callee lives) and ${INV_DIR}/mappings.json (renderer + g2api call rewrites per DEC-59.1 / ruling 40; an "unported" mapping gets a \`//TODO: Port <name>\` house marker + Source cite at the site and the call left in Raven shape behind it).
Error context for this file: ${WT}/target/integrate/raw-r3.txt (grep "${f}:").
Also in this file: wire imports for symbols the Foundations phase landed (grep the workspace before declaring anything missing), apply the CARRIER RULE, delete stale \`//TODO: Port <name>\` markers whose symbol now resolves (wire the import), and keep every PORT-NOTE that still states a true open question — delete the ones the spine/law resolved.
${CARRIER_RULE}
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT run cargo. Do NOT git commit. Return JSON {file, fns_rewired, sites_bent, blocked:[..]}.`,
  { label: `rewire:${f}`, phase: 'Rewire', model: HEAVY.has(f) ? 'opus' : 'sonnet', effort: HEAVY.has(f) ? 'medium' : 'low',
    schema: { type: 'object', properties: {
      file: { type: 'string' }, fns_rewired: { type: 'number' }, sites_bent: { type: 'number' }, blocked: BLOCKED_ITEMS,
    }, required: ['file', 'fns_rewired'] } }
)).concat([() => agent(
  `STATE MERGE — flat Client globals. Worktree ${WT}, branch client-cpass. You own EXACTLY ${CLIENT_HOST}.
Add the file-scope globals and hoisted statics the transcribed bodies reference as Client fields (E0609 on Client in ${WT}/target/integrate/raw-r3.txt, minus aggregate members — the rewire lane hops those): the CIN statics (CIN_hold, CIN_loop, CIN_silent, CIN_shader, CIN_system), cvar handles, cl_main/cl_input statics. For each: grep ${ORACLE}/codemp/client for the decl; keep the Raven name VERBATIM; use the file's existing idioms (CvarHandle for cvar_t*, arrays with oracle sizes). Extend the manual Default impl in the same file for every non-zero-valid field. Doc-comment + Source cite per field, STE prose.
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT run cargo. Do NOT git commit. Return JSON {fields_added:[..], blocked:[..]}.`,
  { label: 'merge:Client', phase: 'Rewire', model: 'sonnet', effort: 'medium',
    schema: { type: 'object', properties: { fields_added: { type: 'array', items: { type: 'string' } }, blocked: BLOCKED_ITEMS }, required: ['fields_added'] } }
)]))
for (let i = 0; i < rewireResults.length; i++) collect(rewireResults[i], i < CLIENT_FILES.length ? `rewire:${CLIENT_FILES[i]}` : 'merge:Client')
const rewired = rewireResults.slice(0, CLIENT_FILES.length).filter(Boolean)
log(`Rewire done: ${rewired.reduce((a, r) => a + (r.fns_rewired || 0), 0)} fns rewired, ${rewired.reduce((a, r) => a + (r.sites_bent || 0), 0)} sites bent, Client +${(rewireResults[CLIENT_FILES.length] || { fields_added: [] }).fields_added.length} fields`)
await commit('client-cpass w2: receiver rewiring + renderer/g2api mappings + Client globals', 'commit:rewire', 'Rewire')

// ================= Phase 4: Rounds =================
phase('Rounds')
const INV_SCHEMA = { type: 'object', properties: {
  total_errors: { type: 'number' },
  files: { type: 'array', items: { type: 'object', properties: { file: { type: 'string' }, errors: { type: 'number' } }, required: ['file', 'errors'] } },
  tail: { type: 'string' },
}, required: ['total_errors', 'files', 'tail'] }

let prevTotal = null
let green = false
let stopReason = null
const roundTotals = []
for (let round = 1; round <= MAX_ROUNDS && !green; round++) {
  const inv = await agent(
    `INVENTORY, wave-2 round ${round}. Worktree ${WT}.
${invCmd(round)}
Return ONLY JSON {total_errors, files:[{file,errors}], tail}. No prose.`,
    { phase: 'Rounds', label: `inv:r${round}`, model: 'haiku', effort: 'low', schema: INV_SCHEMA }
  )
  const total = inv.total_errors
  roundTotals.push({ round: `inv-${round}`, total })
  log(`Round ${round} inventory: ${total} errors across ${(inv.files || []).length} files`)
  if (total === 0) { green = true; stopReason = 'green'; break }
  if (total < FINISHER_THRESHOLD) { stopReason = `small-tail (${total} < ${FINISHER_THRESHOLD})`; prevTotal = total; break }
  if (prevTotal !== null && total >= prevTotal * 0.97) { stopReason = `delta-tripwire (${prevTotal} -> ${total})`; prevTotal = total; break }
  prevTotal = total

  const errFiles = (inv.files || []).filter(f => !f.file.endsWith('client_host.rs'))
  const fixResults = await parallel(errFiles.map(f => () => agent(
    `ROUND-${round} FIXER for ${f.file} (${f.errors} errors). Worktree ${WT}, branch client-cpass. You own THIS ONE FILE.
Error detail: ${INV_DIR}/inv-r${round}.json (your file's entry has codes+samples). LAW: ${INV_DIR}/siglaw.json + ${INV_DIR}/mappings.json. Do NOT run cargo.
Fix per contract: signatures per law, call sites bend to declarations, imports/visibility, carrier hops, borrow-order fixes that keep Raven's effect order (E0499/E0502: split the statement, never reorder effects).
${CARRIER_RULE}
${RULINGS}
${NO_REWRITE}
${STYLE}
Do NOT git commit. Return JSON {file, fixed, blocked:[..]}.`,
    { label: `fix:r${round}:${f.file.split('/').pop()}`, phase: 'Rounds', model: f.errors > 150 ? 'opus' : 'sonnet', effort: f.errors > 150 ? 'medium' : 'low',
      schema: { type: 'object', properties: { file: { type: 'string' }, fixed: { type: 'number' }, blocked: BLOCKED_ITEMS }, required: ['fixed'] } }
  )))
  fixResults.forEach((r, i) => collect(r, `fix:r${round}:${errFiles[i] ? errFiles[i].file : i}`))
  await commit(`client-cpass w2 round ${round}: ${errFiles.length} files, ${fixResults.filter(Boolean).reduce((a, r) => a + (r.fixed || 0), 0)} fixes`, `commit:r${round}`, 'Rounds')
}

// ================= Phase 5: Finisher =================
let finisher = null
if (!green) {
  phase('Finisher')
  log(`Finisher: ~${prevTotal ?? '?'} errors remain (${stopReason || 'rounds-exhausted'})`)
  finisher = await agent(
    `FINISHER — serial, cross-cutting. Worktree ${WT}, branch client-cpass. Wave-2 rounds stopped (${stopReason || 'rounds-exhausted'}). Drive \`cargo check --workspace\` to GREEN.
You MAY run cargo (you are the only writer) and MAY edit any file including ${CLIENT_HOST} and ${ENGINE_RS}. Known residual jobs:
1. cinTable VQ0/VQ1 fn-pointer fields: type them per the oracle decl (${ORACLE}/codemp/client/cl_cin.cpp) against the ported ROQ fns, now that cl_cin.rs type-checks enough to reach them.
2. Cross-file signature drift: if two rewire agents disagreed on a callee signature, the siglaw entry (${INV_DIR}/siglaw.json) is the referee.
3. STALE MARKER SWEEP: grep '//TODO: Port' across crates/mp/engine/client; delete every marker whose symbol now resolves (wire the import), keep genuinely-open ones (unported renderer methods, pending lanes).
4. If a Client field added this run is non-zero-valid and Client's Default impl misses it, fix the Default impl.
${CARRIER_RULE}
${RULINGS}
${NO_REWRITE}
${STYLE}
FINAL: git add -A && git commit --no-gpg-sign -m "client-cpass w2: workspace green" (or "client-cpass w2: finisher stop, <n> errors remain"). NEVER add a co-author trailer.
Return JSON {green, remaining_errors, fixed, commit, blocked:[..]}.`,
    { phase: 'Finisher', label: 'finisher', model: 'opus', effort: 'high', schema: { type: 'object', properties: {
      green: { type: 'boolean' }, remaining_errors: { type: 'number' }, fixed: { type: 'number' }, commit: { type: 'string' }, blocked: BLOCKED_ITEMS,
    }, required: ['green'] } }
  )
  green = !!(finisher && finisher.green)
  if (finisher) {
    if (finisher.commit && finisher.commit !== 'nothing-to-commit') commits.push({ msg: 'client-cpass w2: finisher', commit: finisher.commit })
    collect(finisher, 'finisher')
    if (typeof finisher.remaining_errors === 'number') prevTotal = finisher.remaining_errors
    roundTotals.push({ round: 'finisher', total: finisher.remaining_errors })
  }
}
await commitChain

// ================= Report =================
phase('Report')
const seen = new Set()
const blockedUniq = blocked.filter(b => { const k = `${b.file || ''}::${b.error || b.reason}`; if (seen.has(k)) return false; seen.add(k); return true })
const report = {
  green,
  final_errors: green ? 0 : prevTotal,
  stop_reason: stopReason || (green ? 'green' : 'finisher-stop'),
  round_totals: roundTotals,
  spine_summary: spine.summary,
  renderer_unported: (mappings.renderer || []).filter(m => m.unported).map(m => m.method),
  g2api_unported: (mappings.g2api || []).filter(m => m.unported).map(m => m.raven),
  commits,
  blocked: blockedUniq,
}
log(`WAVE 2 DONE: green=${green}, final=${report.final_errors} errors, ${commits.length} commits, ${blockedUniq.length} blocked items`)
return report
