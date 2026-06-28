# jka-rust

An idiomatic, opinionated, per-module Rust reimplementation of Star Wars Jedi
Knight: Jedi Academy (MP).

## Relationship to `jedi-academy-rust`

`jedi-academy-rust` is a faithful, 1:1 C-mirror port of Raven's source. **This
project treats it as an oracle**, not a dependency — exactly as `jedi-academy-rust`
treats Raven's original C. It is vendored under [`oracle/`](oracle) (git submodule).

Because the port is pure Rust, parity is checked by **differential testing**:
compile both implementations and run them against identical inputs, comparing
outputs (gated behind `--features oracle`). No FFI, no extracted C.

This repo does NOT reuse the port's module layout or internals — it takes its own
opinionated structure per module.

## Status

Scaffold. The boundary definitions prototyped in the port (typed
`OutboundSysCall` / `EncodeSysCall` / `DecodeSysCallReturn` defs for all ~329
game→engine syscalls, plus the transport traits) are imported under
[`src/boundary/`](src/boundary) as starting material. They currently reference the
port's types (`crate::codemp::game::*`) and so do **not** compile here yet — the
next step is to source those types under this crate's own layout (re-port
opinionatedly, or via a thin shared ABI-types crate) and wire the modules into
`src/lib.rs`. The ABI catalog is in `src/boundary/TRAPS.md`.
