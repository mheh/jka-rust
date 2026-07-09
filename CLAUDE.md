# jka-rust

Idiomatic Rust reimplementation of the **entire** Jedi Academy codebase (MP, SP,
UI, renderer, engine) as a drop-in replacement, ported from Raven's C/C++ source.

## Oracle

`oracle/oracle/` holds the original Raven source — the differential-testing
**oracle**, never edited. SP lives under `code/`, MP under `codemp/`. Every port
is verified against it. `oracle/` (the faithful Rust port) is the parity baseline
checked via `--features oracle`.

## Port style — read this before porting anything

@docs/porting-rules.md

Two conventions the rules above don't yet state explicitly but that the codebase
follows everywhere — apply them:

- **Enum-vs-alias fidelity.** `typedef enum {...} X` → `#[repr(i32)] enum X`;
  `typedef int X` + a separate anonymous enum → `type X = c_int` + `const`s. Do
  **not** flatten a named enum to an int alias (this caused real bugs:
  `spectatorState_t`, `alertEvent*` were wrongly flattened and had to be fixed).
- **One type per file**, in a folder mirroring the owning Raven header's subsystem.

## Verifying

rust-analyzer is stale in this workspace — **always confirm compilation with
`cargo build` / `cargo check`**, not the editor. Every ABI-crossing struct carries
`size_of`/`offset_of!` static-asserts; a green build validates layout parity.

## Key docs

- `docs/workspace-architecture.md` — crate graph and dependency tiers
  (native < qshared < bg < game; qshared → abi → game).
- `docs/decisions.md` — the DEC-xx ledger: user-settled architectural choices
  (renderer deferral, WASM transport, wire compat, …). Cite, never re-litigate.
- `docs/doc-standards.md` — template + gates for logic-port design docs
  (`docs/architecture/`, `docs/modules/`, `docs/subsystems/`).
- `docs/GOAL.md` — project goal (drop-in ABI compatibility checklists).
- `docs/abi-traps.md` — generated trap_* signature reference.
- `docs/audits/marker-inventory-2026-07-08.md` — validated open-work inventory
  (`TODO: Port` by verdict + PORT-NOTE re-grep); regenerated, never hand-edited.
- `docs/roadmap-final-stages.md` — ordered post-parity roadmap (referee gates
  everything; then safe-state migration and beyond).
- The completed type-port campaign docs (plan/scope/todo/oracle-types index)
  were removed 2026-07-06; history lives in git.

## Port tooling & the logic-port pipeline

The type port ran on: `tools/closure-prototype/sweep.py` (one libclang parse →
per-type packets: verbatim source slice + ready-to-paste layout asserts; badge
modes verify Rust asserts against clang ground truth) consumed by
`.claude/workflows/port-wave.js` (skeleton → parallel packet-porters who never
explore → machine verify). Principle: **tooling turns the oracle into
self-contained, machine-verifiable work orders; agents transcribe, a mechanical
referee judges.**

The logic-port pipeline was built and **ran**: `sweep.py` → `packets3.py`
(pass-3 function packets: threading digests, resolved-sig LAW extraction,
sharding) → the pass-3 port workflow (blind parallel transcribers) → the
integrate workflow (triage → bounded fix rounds → serial finisher) plus
`bulkfix.py` batch tooling (`--cast` int/float/enum span casts, `--overlay`
c_void-family modes). **jampgame transcription and integration are done**:
`mp_game` compiles with 0 errors (integrate phase: ~5,800 → 0), `cargo build
--workspace` green, merged to master 2026-07-05; `todo!()` stubs and open
`TODO: Port` markers both at zero (2026-07-06). CI builds and publishes
engine-named modules on master pushes (`.github/workflows/build.yml`, rolling
`latest` release; 32-bit lanes allowed-failure pending an ILP32 assert pass).
Remaining on jampgame: per-file oracle review and the referee swap — oracle
differential tests (single-threaded; the oracle has global state) replace
clang layout as ground truth — then the safe-state migration. The
`dangerous_implicit_autorefs` lint is allowed crate-wide in `mp_game`
(documented in its lib.rs) pending the safe-state migration.

- **MP** (`jamp` engine) ships 3 loadable DLLs: `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine) ships **only** `jagame`; SP cgame/ui are statically
  linked into the engine binary, not separate modules.
