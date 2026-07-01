# Porting Rules

Rules for converting Raven C/C++ (`oracle/oracle/**`) into idiomatic Rust under
the crate graph in `docs/workspace-architecture.md`. Verified against the faithful
port (`oracle/`) by differential testing (`--features oracle`).

## A. What "correct" means

1. **Behavioral parity at the ABI seam, verified against oracle.** A port is done
   when its observable behavior matches the oracle under `--features oracle`.
   Internals are free; the seam is not.
2. **No speculative behavior.** If Raven's behavior is unclear, port it faithfully
   first — even if ugly — get it green, *then* refactor behind the passing diff.
   Never guess a "cleaner" behavior.

## B. State (the spine)

3. **No `static mut`, no hidden globals.** Raven's `level`, `g_entities`,
   `gclients`, cvar tables, and engine singletons do not become Rust globals. Each
   subsystem owns its state in a struct.
4. **State is threaded, not reached.** Handlers receive their world explicitly
   (`&mut GameWorld`, `&Engine`) rather than touching ambient state. The ABI
   entrypoints own the one instance and pass it inward.
5. **Entities by index/handle, not pointer.** Raven's `gentity_t*` become
   `EntityId(u32)` into an owned arena; no aliasing raw pointers in safe code.
6. **One owned instance per logical singleton.** Where Raven truly had one global
   (the engine), model it as one owned object reached through a single controlled
   accessor — not scattered statics. (See `src/engine/PLAN.md`.)

## C. C -> Rust idiom translation

Mechanical defaults below. **Specific renames and per-type idiom choices are
decided in discussion when we port that code — not pre-baked here.**

7. **Out-params -> return values; `qboolean`/error-int -> `bool`/`Result`.**
8. **`#define` -> `const`/`enum`; function-pointer tables -> traits.**
9. **Manual alloc/free -> ownership**; `Z_Malloc`/pools -> `Vec`/arenas/`Box`.
10. **Preserve control-flow behavior, not control-flow shape** — a C `goto`/early
    return may become idiomatic Rust as long as the diff stays green.

## D. The ABI seam

11. **Unsafe is confined to the seam.** The variadic syscall choke point and layout
    casts are the only `unsafe`; everything above is safe Rust.
12. **ABI-crossing types keep exact layout** — `#[repr(C)]`, Raven field
    names/order, and `offset_of!`/size static-asserts against the headers.
    Internal-only types get idiomatic Rust shape and naming.

## E. Process

13. **Slice-driven, not manifest-driven.** Port what a runnable vertical slice
    needs; fill ABI `Args`/`Output` when a live call exercises them.
14. **Unported deps are explicit** — never a silent fake. See markers below.
15. **Green at every commit** — `cargo build` + oracle parity for anything wired
    live; one function/struct/file per commit.
16. **Stopping point = a real engine call works end-to-end and matches oracle**,
    not "N files typed."

## Comment & source-reference rules

Every ported item keeps the current codebase style:

- **Preserve Raven comments** where they clarify behavior.
- **Doc-comment + source ref on every item**, in today's format:

  ```rust
  /// `trajectory_t`.
  ///
  /// Raven: <original Raven comment, if any>.
  /// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2648-2657`
  pub struct Trajectory { /* ... */ }
  ```

- When a type/const differs SP vs MP, keep **both** source refs (existing style).

- **State the conclusion, not the derivation.** The oracle source holds the full
  definition, so the comment cites it rather than re-deriving it. Shape:
  `/// Raven \`X\` <one-line desc>.` + a `Source:` / `Type definition source:` cite.
  Add rationale only when a Rust choice diverges from the obvious (e.g. a `#[repr]`
  width or a wire-safe newtype), and cap it at ~2 lines. If you find yourself
  re-explaining C mechanics the cited lines already show, cut it.

  ```rust
  // Raven's `typedef char memtag_t` is 1 byte, not int-wide; `#[repr(i8)]` matches
  // that width.
  // Source: `oracle/oracle/codemp/game/q_shared.h:3101-3107`
  #[repr(i8)]
  ```

## Unported-work markers

Every not-yet-ported placeholder uses one consistent, greppable pattern so the
remaining work is discernible by subject and location.

**Rule: the marker is always `//TODO: Port <subject>`**, where `<subject>` is the
exact Raven identifier (type, function, constant, or module). Discernibility comes
from the subject name; the shared `TODO: Port` prefix makes every open item
greppable.

```
//TODO: Port gentity_t
//TODO: Port math
//TODO: Port MAX_QPATH
//TODO: Port CG_TRACE args
```

- Every marker is followed by a **`// Source:` ref** to the Raven location:

  ```rust
  //TODO: Port gentity_t
  // Source: oracle/oracle/codemp/game/g_local.h:137
  pub m_pVehicle: (),
  ```

- **Executable stubs that would run** panic loudly and echo the same subject, so a
  hit is discernible in a backtrace and greppable in source:

  ```rust
  fn G_Spawn() -> EntityId {
      todo!("Port G_Spawn — oracle/oracle/codemp/game/g_utils.c:...")
  }
  ```

- **A deliberately callable no-op** (rare, must be justified) still carries a
  `//TODO: Port <subject>` + `// Source:` and a one-line reason.

- Normalize any legacy `FIXME: create type` / `//TODO: Port args` markers to this
  scheme as they are touched.

Discernibility test: `grep -rn "TODO: Port"` lists every open item, each named by
its exact Raven subject.
