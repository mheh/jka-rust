# jka-rust

Idiomatic Rust reimplementation of the **entire** Jedi Academy codebase (MP, SP,
UI, renderer, engine) as a drop-in replacement, ported from Raven's C/C++ source.

## Oracle

`oracle/` holds the original Raven source (submodule
`github.com/mheh/jediacademy`) — the differential-testing **oracle**, never
edited. SP lives under `code/`, MP under `codemp/`. Every port is verified
against it. The earlier faithful-Rust-port harness lives externally at
`github.com/mheh/jedi-academy-rust` (no longer checked out here).

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
  (renderer deferral, wasm dropped per ruling 35, wire compat, …). Cite, never re-litigate.
- `docs/doc-standards.md` — template + gates for logic-port design docs
  (`docs/architecture/`, `docs/modules/`, `docs/subsystems/`).
- `docs/GOAL.md` — project goal (drop-in ABI compatibility checklists).
- `docs/abi-traps.md` — generated trap_* signature reference.
- `docs/audits/marker-inventory-2026-07-08.md` — validated open-work inventory
  (`TODO: Port` by verdict + PORT-NOTE re-grep); regenerated, never hand-edited.
- `docs/roadmap-final-stages.md` — ordered post-parity roadmap (referee gates
  everything). Stage 1 safe-state is frozen by DEC-31; the active track is the
  client port, planned in `docs/plans/2026-07-24-client-port/`.
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
engine-named modules and the `jampded` server executable on master pushes
(`.github/workflows/build.yml`, rolling `latest` release; all module lanes
enforced since the jampgame ILP32 assert pass).

**The MP dedicated-server engine is complete and hosts live play.** The
`jampDed` link set — `qcommon`, `botlib`, `server`, the seven C++ subsystems
(ghoul2-server, ICARUS, RMG/terrain, NPC nav, StringEd, ROFF, headless renderer
model/skin subset), and the native platform layer — is transcribed, integrated,
and closed: zero stubs, zero `TODO: Port` markers, zero extern forward-decl
blocks. Closure and boot-seam rulings are DEC-13…DEC-23; the ILP32 assert pass
(cfg-32 twin asserts, i686 cross-check) and every CI lane are enforced. The
lockstep-referee suite (mock + real-map mp/duel1 + ffa1 scenarios, up to 2000
frames / 430k syscalls; 9 tests) runs byte-identical oracle-vs-rust and gates
every commit;
the server boots, loads maps, and has hosted live players since 2026-07-12.

**The idiomatic consolidation campaigns are done and merged to master:** the #13
string campaign (owned `String`/`&str`, `bool`, Latin-1 wire discipline,
`CString` removal), DEC-32 dedup (one canonical home per fn under `native/*`),
DEC-34 (qsort — `bg_lib` body canonical, msvcrt tie-order closed), DEC-35 + task
#17 (ghoul2 block ownership — mdx views in `mp_host_interface`, `EngineHost`
hands out `MdxaRef`/`MdxmRef`, parsed-once sidecar), and task #19 (ctx threaded
through `G_ModelIndex`/`G_SoundIndex`/`G_EffectIndex`; `strap_world` down to 4
deliberate safe-state readers). The safe-state mechanical migration was frozen
by DEC-31 (2026-07-16); the idiom era superseded it. Typed entity-view refactors
are deferred to the post-full-port "great refactor."

**Next track (ruled):** the `ui` module first, then `cgame` + renderer, toward a
full `jamp` client — plans in
`docs/plans/2026-07-24-client-port/{scoping,ui-plan,renderer-plan}.md`.
Threading is permanently out of scope for this repo (fork-only).

- **MP** (`jamp` engine) ships 3 loadable DLLs: `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine) ships **only** `jagame`; SP cgame/ui are statically
  linked into the engine binary, not separate modules.
