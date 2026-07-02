---
name: port-types
description: Port Raven C/C++ types into the jka-rust crate graph, crate by crate, following the bottom-up wave plan. Use when the user wants to port types, continue the type port, start/resume a porting wave, port a specific crate/header (qshared, bg, cgame, renderer, …), or asks "what's next to port".
---

# port-types

Drives the faithful C→Rust type port for jka-rust. Ports are verified against the
oracle by layout (`size_of`/`offset_of!`) and by `cargo build`.

## Read these first (every session)

- `docs/type-port-plan.md` — the crate-by-crate **wave order** and current scope.
- `docs/type-port-todo.md` — **live per-type status**; the source of "what's next".
- `docs/porting-rules.md` — the binding rules (also loaded via CLAUDE.md).
- `docs/oracle-types.md` — mechanical index of every oracle type (`file:line`).
- `docs/workspace-architecture.md` — crate graph / tier placement.

Oracle C source: `oracle/oracle/codemp/` (MP) and `oracle/oracle/code/` (SP) —
**never edit**. Faithful Rust baseline: `oracle/` (`--features oracle`).

## Pick the target

1. From `type-port-plan.md`, take the **lowest incomplete wave** (bottom-up: a
   type is ported only after everything it depends on is frozen below it).
2. Within that wave, take a crate; within the crate, an oracle header.
3. **MP first, then SP as a diff** against the MP port.
4. Skip the separate workstreams: **vendored** (jpeg/png/zlib/mp3/smartheap/eax)
   → Rust crates, not ported; **C++ classes** (Ra*/Splines/goblib/renderer) →
   idiomatic reimpl, not byte-faithful.

## Port one type — checklist

- [ ] **Classify:** trivial (alias / small enum / fn-ptr sig) → batchable;
      heavy layout-critical struct (`playerState_t`, `gentity_t`, `refEntity_t`,
      …) → **one per commit**, never rushed.
- [ ] **Enum-vs-alias fidelity** (real bugs came from getting this wrong):
      `typedef enum {…} X` → `#[repr(i32)] enum`; `typedef int X` + separate anon
      enum → `type X = c_int` + `const`s. **Never flatten a named enum to an int.**
- [ ] **One type per file**, folder mirrors the owning Raven header's subsystem.
- [ ] **Tier placement:** lowest crate that needs it (native < qshared < bg <
      game). Genuinely Raven-free + identical MP/SP → `native/*`; anything that
      diverges MP/SP → per-mode `qshared`/etc.
- [ ] **Doc-comment + source ref** in the house style (see below). Keep MP+SP
      refs when the type differs per mode.
- [ ] **ABI-crossing struct** → `#[repr(C)]`, exact Raven field names/order, and
      `size_of`/`offset_of!` static-asserts against the header. Internal-only
      types get idiomatic Rust shape.
- [ ] **Unported deps** → `//TODO: Port <RavenIdent>` + `// Source:` ref; runnable
      stubs `todo!("Port <subject> — <oracle path:line>")`. Never a silent fake.
- [ ] **`cargo build`** (or `cargo check -p <crate>`) is GREEN — rust-analyzer is
      stale here, trust cargo only. A green build validates layout parity.
- [ ] Update `docs/type-port-todo.md` (☐→☑, add row if new).
- [ ] **Commit:** one type/struct/file per commit (batch only trivial aliases).
      End message with the Co-Authored-By line from the harness rules.

## Doc-comment + source-ref format

```rust
/// Raven `trajectory_t` — movement interpolation state.
///
/// Raven: <original Raven comment, if any>.
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2648-2657`
#[repr(C)]
pub struct Trajectory { /* Raven field names/order */ }
```

State the conclusion, cite the source; don't re-derive C mechanics. Add rationale
(≤2 lines) only when a Rust choice diverges from the obvious (e.g. a `#[repr]`
width or a wire-safe newtype).

## Delegate (fan out)

- **Scout** a header before porting: launch an `Explore` agent to inventory the
  header's types (name, kind, `file:line`, MP-vs-SP divergence, done-vs-missing).
  Give it the oracle paths and tell it not to edit.
- **Batch-port** trivial/medium types: fan out `general-purpose` agents, one per
  header or type-group, each handed this skill's checklist + the porting rules.
  Bottleneck is verification, so have each agent end on a green `cargo check`.
- **Heavy layout-critical structs:** do these yourself, one per commit, with full
  `offset_of!` asserts. Do not parallelize them — or run them through the
  `port-wave` workflow's serial Heavy phase (below), which enforces the same
  discipline with machine verification.

See `docs/type-port-plan.md` for the wave breakdown and delegation notes.

## Tools & workflows (preferred for batch waves)

Ground-truth layout tooling lives in `tools/closure-prototype/` (see its
NOTES.md): `closure.py` (dependency closures, call trees, `--asserts`
generation, verified ☑/✗/◐ ported badges from clang record layouts) and
`portpacket.py` (self-contained function port packets, `--json`).

Three Workflow scripts in `.claude/workflows/` orchestrate batch porting with
subagents + machine verify; output is left **uncommitted for review**:

- `port-assert-backfill` — args: `[{file, module, crate}]` (the assert-less
  `#[repr(C)]` files; compute via
  `for f in $(grep -rl "#\[repr(C)\]" crates --include="*.rs"); do grep -q size_of $f || echo $f; done`).
  One agent per file pastes clang-generated asserts; compile-failing asserts
  are reported as latent layout bugs, never silently "fixed".
- `port-wave` — args: `{mpModule, spModule, mpCrate, spCrate, headers[],
  onlyTypes?}`. Scout → skeleton (pre-wires lib.rs so porters never collide)
  → parallel MP port → SP-as-diff → serial heavy phase (high effort) →
  badge-sweep verify with fixer rounds → todo-doc update. Smoke-test new
  targets with `onlyTypes` first.
- `port-cpp-subsystem` — the C++ track (porting-rules §F: idiomatic
  reimplementation, differential verification; NOT byte-faithful). args:
  `{subsystem, mpCrate, spCrate, mpDir, spDir, mpOracle[], spOracle[],
  designPath?, hard?[], skipDocs?}`. Scout → high-effort Design doc +
  adversarial review (pass `designPath` to use a hand-reviewed doc instead)
  → differential harness (`tools/<subsystem>-oracle`, compiles the
  unmodified oracle TUs, committed goldens, Rust parity tests) → frozen
  skeleton (designed signatures, `todo!()` bodies) → parallel per-class
  porters (MP, then SP twin as diff) → parity+cargo verify with fixer
  rounds → todo-doc C++-track table. Exemplar: GP2
  (`crates/mp/engine/qcommon/src/gp2`, `tools/gp2-oracle`). For subsystems
  with heavy engine coupling (ghoul2), expect the Harness phase to shrink
  scope — it must record uncovered areas under `gaps`, never claim silent
  coverage.
