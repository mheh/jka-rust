# Decision Ledger

Architectural decisions settled with the user (2026-07-01 session) that design docs
cite but never re-litigate. Design docs reference entries by ID (`DEC-xx`).
Amendments are appended as dated notes — entries are never silently rewritten.

Per-doc design decisions (`<PREFIX>-Dn`) live in each design doc's `## Decisions`
section; this ledger holds only cross-cutting choices.

---

## DEC-01 — Renderer port deferred; wgpu when it happens

The renderer is **not ported until much later** in the project. Until then the
engine boots headless: MP dedicated uses Raven's own null path
(`oracle/oracle/codemp/null/`), and MP client / SP get a **null-renderer
`refexport_t` stub** behind the same seam (renderer types are already ported,
Wave 7). When the renderer is ported, **wgpu is the likely backend** (an
idiomatic C++-track-style rewrite, not a fixed-function GL transcription) —
re-confirm specifics at that time.

Implications: `docs/subsystems/renderer.md` covers the seam + null-renderer
deferral strategy only; lifecycle docs must define headless boot for all three
executables.

> **Amendment 2026-07-02:** the earlier note that SP boots the renderer early
> inside `Com_Init` was wrong — that block
> (`oracle/oracle/code/qcommon/common.cpp:965-981`) is `#ifdef _XBOX` dead
> code. On PC, SP initializes the renderer inside `CL_Init` like MP (A3
> survey). Headless design needs no special SP early-boot handling.

## DEC-02 — Windowing/input: winit

`native/platform` uses **winit** (+ `raw-window-handle`) for window and input.
The platform doc defines the adapter between winit's event-loop model and
Raven's poll-style `Sys_*`/input pump.

## DEC-03 — Audio: cpal + faithful mixer

Raven's software mixer (`snd_dma`/`snd_mix`) is ported **faithfully** and outputs
through **cpal**. MP3 decode via `minimp3` (existing vendored-replacement
policy); EAX and force-feedback stay dropped. Keeps mixing behavior
parity-testable at the sample level.

## DEC-04 — Cross-mode sharing: strict per-mode during porting

Every Raven-derived engine subsystem is **duplicated per-mode** during the port —
porters never face a sharing decision, each crate diffs against exactly one
oracle subtree, and crate boundaries stay 1:1 with Raven compile-lists.
Unification of a provably-identical MP/SP pair is allowed **only as a
post-parity refactor** behind green differential tests (porting-rules §A2).
`native/` remains math/platform/containers only.

## DEC-05 — Module loading scope (and the WASM transport)

In scope:

1. **Rust engine ↔ Rust modules** — core scenario, all platforms, via
   `native/platform` dylib loading honoring Raven's entry symbols.
2. **Rust modules loaded by real engines** — retail `jamp` 1.01 (requires
   `i686-pc-windows` module builds) and OpenJK native builds (account for the
   Raven-vs-OpenJK ABI divergences documented in
   `tools/closure-prototype/NOTES.md`).
3. **Rust engine hosting real/mod DLLs** (`jampgamex86.dll` replacements: JA+,
   MBII, …) — **`i686-pc-windows` engine build only** (retail DLLs are 32-bit
   Windows PE). Mod-ecosystem compatibility is the goal that justifies this.
4. Classic QVM bytecode interpreter: **out of scope** (JKA shipped native DLLs).
5. **WASM module transport: first-class target from the start.** The module
   transport is pluggable — `NativeDll | Static | Wasm` — and
   `architecture/engine-seam.md` + `architecture/module-loading.md` must design
   the WASM variant explicitly (wasm32 linear-memory pointer translation à la
   `VM_ArgPtr`, handle-only trap surface, 32-bit in-module layouts). Module
   crates get `wasm32` build checks in CI from the beginning so portability
   never regresses. The wasmtime host itself lands **after** native-DLL parity
   is proven (a sandbox is no place to debug parity).

## DEC-06 — Network: full 1.01 wire compatibility

Exact protocol 26 on the wire — huffman coding, delta snapshots,
challenge/connect flow — so Rust peers interoperate with real 1.01/OpenJK peers.
Live interop doubles as the strongest oracle (see DEC-09 layer 2), and it is
what "drop-in `jampded`" means.

## DEC-07 — SP cgame/ui: statically linked via the vmachine shim

`sp/app` statically links `sp/cgame` + `sp/ui`; Raven's fake-VM shim
(`oracle/oracle/code/client/vmachine.cpp`) survives as a thin dispatch layer
preserving the `VM_Call` ABI shape — matching shipped `jasp`. The retail
load-from-DLL variant is not ported. Resolves workspace-architecture's "SP
transport" open item.

## DEC-08 — Com_Error recovery: panic + catch_unwind

`Com_Error(ERR_DROP, …)`'s dynamic escape to `Com_Frame` becomes a **typed
panic payload** caught by `catch_unwind` at the frame boundary, running the
same recovery Raven does. Requires `panic = "unwind"` and a per-subsystem audit
that unwound state resets match Raven's error paths. Result-threading was
rejected: it reshapes thousands of faithful signatures away from oracle
control flow.

> **Amendment 2026-07-02:** Raven's mechanism is C++ `throw`/`catch` (string
> exceptions like `"DROPPED\n"` caught in `Com_Frame` —
> `oracle/oracle/codemp/qcommon/common.cpp:1762`,
> `oracle/oracle/code/qcommon/common.cpp:1450`), not setjmp/longjmp as first
> recorded. Recovery runs *before* the throw; the catch prints and returns.
> This maps onto panic+catch_unwind even more directly — the decision stands
> unchanged. (A3 survey.)

## DEC-09 — Engine verification: TU harnesses + live peers

Two layers, used by every subsystem doc's `## Verification strategy`:

1. **TU-level golden harnesses** (the `tools/gp2-oracle` pattern): compile the
   unmodified oracle TU standalone with stub headers, golden-diff canonical
   dumps against the Rust port — for huffman, msg pack/unpack, cvar parsing,
   netchan framing, and anything else that runs standalone.
2. **Live-peer slice acceptance** (per DEC-06): scripted sessions between the
   Rust engine and real binaries (retail/OpenJK), diffing observable behavior —
   console output, cvar dumps, wire traffic captures.

Building the full Raven engine from oracle source was rejected as a project in
itself; OpenJK serves as the buildable near-oracle where a live peer is needed.
