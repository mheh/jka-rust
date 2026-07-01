# Typed Boundary Over Swappable Engine Backends

## Summary
`src/engine` is the JKA engine reimagined in idiomatic Rust. `src/abi/` (a crate-root
seam, depended on by both `src/game` and `src/engine`) is the typed boundary between the game
module and the engine: every engine↔game call is a typed `OutboundSysCall` (game→engine) or `InboundVmCall`
(engine→game). Execution is supplied by a **backend** that implements those calls. Two backends
share one set of call definitions:

- **`CEngine`** — today's C engine, reached over the variadic syscall pointer (the only `unsafe`).
- **`RustEngine`** — the future native rewrite, servicing the same typed calls as ordinary safe Rust.

The backend is selected at compile time, so dispatch is fully static and zero-cost. `trap::X(..)`
stays the call-site syntax; only the executor behind it changes.

## Principles
- **Behavioral faithfulness.** Unobtrusive *behavioral* change; structural/idiomatic-Rust
  rewriting is in scope.
- **Preserve call-site syntax.** The typed boundary sits *behind* `trap::*`; `codemp/game` call
  sites do not move.
- **Static dispatch by type, not number.** The call site names the concrete call type;
  `IMPORT`/`COMMAND` exist only to cross the C wire.
- **One owned engine instance.** Scattered engine-side `static mut` / atomics collapse into a
  single owned state object reached through one controlled accessor. Behavior-preserving because
  there was always exactly one logical engine. (`static mut` eliminated; one `static` handle with
  interior mutability is acceptable — this is the "stateful" instance.)
- **Scope = the engine.** The game module's own globals (`level`, `g_entities`, gclients) stay
  faithful for now — not folded into this work.

## Core types
Per-call identity + shape (already exist):
- `OutboundSysCall { type Args; type Output; const IMPORT: GameImport }`
- `InboundVmCall   { type Args; type Output; const COMMAND: GameExport }`

Transport — close the currently half-built halves:
- outbound: `EncodeSysCall` (args → words, exists) + **`DecodeSysCallReturn`** (return word →
  `Output`, add)
- inbound: `DecodeVmMain` (words → args, exists) + **`EncodeVmMainReturn`** (`Output` → return
  word, add)

Execution — the new seam:
- **`Execute<C: OutboundSysCall> { fn execute(&self, args: C::Args) -> C::Output }`** — each
  backend blanket-implements it under that backend's own capability bound. (Differing bounds are
  exactly why "how to run a call" can't live on `OutboundSysCall` itself.)

## Backends
**`CEngine`** — one blanket impl covers every call; per-call knowledge is the encode/decode:

```rust
impl<C> Execute<C> for CEngine
where C: EncodeSysCall + DecodeSysCallReturn {
    fn execute(&self, args: C::Args) -> C::Output {
        let words = C::encode_syscall(&args);
        let ret = unsafe { raw_syscall_words(C::IMPORT, words.args()) }; // sole unsafe choke point
        C::decode_return(ret)
    }
}
```

`raw_syscall_words` localizes the variadic-ABI wrinkle (you cannot forward a runtime `&[isize]`
to a C variadic fn): it spells out arity or passes a fixed 12-slot frame.

**`RustEngine`** — owns engine state; per-call handlers are plain safe Rust, `IMPORT` unused:

```rust
trait RunNative: OutboundSysCall {
    fn run(engine: &RustEngine, args: Self::Args) -> Self::Output;
}
impl<C> Execute<C> for RustEngine where C: RunNative {
    fn execute(&self, args: C::Args) -> C::Output { C::run(self, args) }
}
impl RunNative for GCvarRegister {
    fn run(e: &RustEngine, a: GCvarRegisterArgs) {
        e.cvars.register(a.var_name(), a.default_value(), a.flags());
    }
}
```

## Call sites and backend selection
```rust
fn Cvar_Register(/* … */) {
    ENGINE.execute::<GCvarRegister>(GCvarRegisterArgs::new(/* … */));
}
// type Engine = CEngine;    today
// type Engine = RustEngine; after the rewrite
```
`trap::Cvar_Register` delegates to this; flipping the `Engine` alias reroutes every typed call
with no call-site change.

## Inbound (engine→game)
The `vm_main` switch decodes args with `DecodeVmMain` and encodes the return with
`EncodeVmMainReturn`, instead of indexing raw `args[]` and hand-returning `0`. `RustEngine` calls
the game's typed `InboundVmCall` handlers directly, bypassing the switch entirely.

## Dropped from the prior plan
The world-routed redesign is removed: `WorldState` / multi-world routing, `WorldId` / `ClientId`,
global/world **queues**, and **sync/async classification** + acks. The model stays single-instance
and synchronous. "GlobalState" survives only as the single owned engine instance — not as a
request router.

## Incremental steps (each a small, oracle-green commit)
1. Add `DecodeSysCallReturn` + `EncodeVmMainReturn`; impl for the worked examples
   (`G_CVAR_REGISTER`, `GAME_RUN_FRAME`). Closes the transport asymmetry.
2. Add `Execute<C>` and the `raw_syscall_words` arity helper.
3. Add the `CEngine` backend (blanket impl). No call sites change yet.
4. **Tracer bullet:** route `trap::Cvar_Register` through `CEngine`; confirm oracle parity.
5. Route `GAME_RUN_FRAME` through decode/encode in `vm_main`.
6. Flesh typed `Args`/encode/decode for more calls; widen `trap::*` delegation incrementally.
7. Later: `RustEngine` skeleton + `RunNative` for one call; prove the `Engine` alias swap compiles
   both backends.
8. Later: consolidate engine-side globals into the owned instance.

## Test plan
- Round-trip unit tests per representative call: typed `Args` → words, and return word → `Output`.
- Oracle parity held whenever a real call is routed through `CEngine`
  (`cargo test --features oracle -- --test-threads=1`).
- `cargo check` / `cargo test` green at every step.

## Assumptions
- Engine-side scope only; game-module globals stay faithful for now.
- One owned engine instance reached via a single controlled accessor (not literally zero globals).
- Backend chosen at compile time (`type Engine` alias); static, zero-cost dispatch.
- `trap::*` call-site syntax preserved; behavior preserved throughout.
