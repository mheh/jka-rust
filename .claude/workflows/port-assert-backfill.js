export const meta = {
  name: 'port-assert-backfill',
  description: 'Backfill missing size/offset asserts on #[repr(C)] files from clang ground truth',
  whenToUse: 'When #[repr(C)] ports lack size_of/offset_of! static-asserts; args = [{file, module, crate}]',
  phases: [
    { title: 'Backfill', detail: 'one agent per assert-less file, closure.py --asserts' },
    { title: 'Audit', detail: 'residual sweep + cargo check --workspace' },
  ],
}

// args: [{file: "crates/mp/qshared/src/.../usercmd.rs", module: "mp-game", crate: "mp_qshared"}]
const FILES = typeof args === 'string' ? JSON.parse(args) : args
if (!Array.isArray(FILES) || FILES.length === 0) {
  throw new Error('args must be a non-empty array of {file, module, crate}')
}

const TOOL = 'tools/closure-prototype/.venv/bin/python tools/closure-prototype/closure.py'

const RESULT_SCHEMA = {
  type: 'object',
  properties: {
    file: { type: 'string' },
    typesAsserted: { type: 'array', items: { type: 'string' } },
    typesSkipped: { type: 'array', items: { type: 'object', properties: {
      name: { type: 'string' }, reason: { type: 'string' } }, required: ['name', 'reason'] } },
    mismatches: { type: 'array', items: { type: 'object', properties: {
      name: { type: 'string' }, detail: { type: 'string' } }, required: ['name', 'detail'] } },
    cargoGreen: { type: 'boolean' },
    notes: { type: 'string' },
  },
  required: ['file', 'typesAsserted', 'typesSkipped', 'mismatches', 'cargoGreen'],
}

phase('Backfill')
log(`backfilling asserts in ${FILES.length} files`)

const results = await pipeline(
  FILES,
  item => agent(
`You are hardening one Rust file in the jka-rust port with layout static-asserts.
Work from the repo root. NEVER edit anything under oracle/.

FILE: ${item.file}   (crate: ${item.crate}, oracle parse module: ${item.module})

Steps:
1. Read the file. List every #[repr(C)] struct/union in it (skip enums and
   type aliases — house style does not assert those).
2. For each such type, get ground truth:
     ${TOOL} ${item.module} <TypeName> --asserts
   (run from repo root). If the tool can't find the type in that module's TU:
   (a) try the sibling module (mp-game <-> mp-cgame, etc.);
   (b) check for a RENAMED oracle counterpart — e.g. SP often uses a C++ class
       name (CCollisionRecord) where the Rust port kept the MP typedef name
       (CollisionRecord_t) — run the tool on the oracle name and use those
       numbers, noting the rename in notes;
   (c) only if there is genuinely no oracle counterpart (Rust-side ABI helper
       struct), add it to typesSkipped. NEVER hand-derive assert numbers —
       every asserted number must come from the tool.
3. Paste the assert block at the end of the file (after the type), matching
   house style exactly:
     const _: () = assert!(core::mem::size_of::<X>() == N);
     const _: () = assert!(core::mem::offset_of!(X, field) == M);
   - The size assert is REQUIRED. Offset asserts: include all fields for
     types with <= 10 fields; for bigger types pick ~4-6 representative
     anchors (first field, fields following pointer/array boundaries).
   - If the struct contains raw pointers or usize (pointer-width-dependent
     layout), gate EVERY pointer-dependent assert with
     #[cfg(target_pointer_width = "64")] on its own line above the const,
     exactly like crates/mp/qshared/src/common/mp/qcommon/saber/saber_info.rs.
   - Do not reorder fields, rename anything, or change any existing code.
4. Verify: cargo check -p ${item.crate}   — must be GREEN.
   *** If an assert FAILS to compile, that is a latent layout bug in the
   existing port. Do NOT silently edit struct fields to make it pass. Remove
   the failing assert, leave the file compiling, and report the type under
   mismatches with the expected-vs-actual detail. ***
5. Return the structured result. Your final message is data, not prose.`,
    { label: `backfill:${item.file.split('/').pop()}`, phase: 'Backfill',
      model: 'sonnet', schema: RESULT_SCHEMA }
  )
)

const ok = results.filter(Boolean)
const mismatches = ok.flatMap(r => r.mismatches.map(m => ({ file: r.file, ...m })))
const skipped = ok.flatMap(r => r.typesSkipped.map(s => ({ file: r.file, ...s })))
const asserted = ok.reduce((n, r) => n + r.typesAsserted.length, 0)
log(`${asserted} types asserted, ${mismatches.length} mismatches, ${skipped.length} skipped`)

phase('Audit')
const audit = await agent(
`Audit the assert-backfill that just ran on the jka-rust repo (repo root cwd).
1. Sweep for remaining assert-less files:
   for f in $(grep -rl "#\\[repr(C)\\]" crates --include="*.rs"); do grep -q "size_of" $f || echo $f; done
   Files expected to remain (had only Rust-side/skipped types): report them, don't fix.
2. cargo check --workspace — must be green; report any failures verbatim.
3. Spot-check 3 of the largest newly asserted types with the verified badge:
   tools/closure-prototype/.venv/bin/python tools/closure-prototype/closure.py <module> <Type>
   and confirm the badge line shows ☑ (not "NO SIZE ASSERT" / "SIZE MISMATCH").
Return JSON-ish summary of: remaining files, cargo status, badge results.`,
  { label: 'audit', phase: 'Audit', model: 'sonnet' }
)

return {
  filesProcessed: ok.length,
  filesFailed: FILES.length - ok.length,
  typesAsserted: asserted,
  mismatches,
  skipped,
  audit,
}
