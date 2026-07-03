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
- `docs/type-port-todo.md` — live per-type port status (MP + SP; type port is
  complete — Waves 0–7).
- `docs/type-port-scope.md`, `docs/oracle-types.md` — type-port scope and the
  mechanical oracle type index (reference).
- `docs/GOAL.md` — project goal (drop-in ABI compatibility checklists).
- `docs/abi-traps.md` — generated trap_* signature reference.
- `docs/engine-plan.md` — legacy engine sketch; being superseded by
  `docs/architecture/engine-seam.md` (see decisions ledger).

## Port tooling & the future logic-port pipeline

The type port ran on: `tools/closure-prototype/sweep.py` (one libclang parse →
per-type packets: verbatim source slice + ready-to-paste layout asserts; badge
modes verify Rust asserts against clang ground truth) consumed by
`.claude/workflows/port-wave.js` (skeleton → parallel packet-porters who never
explore → machine verify). Principle: **tooling turns the oracle into
self-contained, machine-verifiable work orders; agents transcribe, a mechanical
referee judges.**

For the logic port, half exists: `closure.py fn:Name --tree` (function call-tree
closure, stops at the trap_* seam) and `portpacket.py <module> <Fn>` (full
single-function packet: body + cites, inlined helpers, type closure w/ badges,
referenced globals, syscall surface). **To build after slice 0 proves the shape
by hand:** a batch function sweep (manifest generator), a `port-logic` workflow
consuming function packets, call-graph ordering (cyclic — needs SCC, not the
type port's DAG levels), fn-pointer dispatch resolution (think/touch/die), and
the referee swap: frozen design docs + oracle differential tests replace clang
layout as ground truth. Goal: per-function cost drops to mechanical throughput;
model-ladder porters (Haiku/Sonnet) with escalations batched, not interactive.

- **MP** (`jamp` engine) ships 3 loadable DLLs: `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine) ships **only** `jagame`; SP cgame/ui are statically
  linked into the engine binary, not separate modules.
