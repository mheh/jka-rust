# renderer port plan (MP `tr_*`; full process)

Companion: `scoping.md` (this folder) (census: the
renderer's module-facing seam — `refEntity_t`/`refdef_t`/`tr_types.h` — is
Class B copy-at-call). Scope: the full MP renderer (`oracle/codemp/renderer`,
~58.8k lines, C-style C++), growing the existing headless subset
(`crates/mp/renderer` tr_model lineage) into the real drawing renderer that
the future client engine island hosts. Same rulings as the ui plan: direct
idiomatic port, minimal deferrals, greppable markers.

## Marker law + translation dictionary

Identical to `ui-plan.md` (this folder) (marker law; dictionary)
— applied to `crates/mp/renderer` (and any new renderer crates). Census
greps target those crates. Renderer-specific dictionary additions:
- GL state/handles stay raw at the GL-call layer only; everything above is
  owned Rust (the unsafe-at-the-seam rule, seam = the GL binding).
- Raven's hunk/Z_Malloc renderer allocations → owned arenas on the renderer
  world (the registry already owns model memory — extend, never fork).
- Function-pointer surfaces (`refexport_t`/`refimport_t`) become traits at
  the engine boundary; internal fn-ptr tables (shader stage fns) → `match`.

## Minimal-deferral strategy

Same dependency-first machinery as ui, using the C++ profile tooling that
ported the engine island (`enginesweep.py`/`engineorder.py`/
`enginepackets.py`): one parse per TU → manifest + call graph → topological
waves; types before functions; packets carry threading digests; wave gates =
`cargo check` + banned-pattern greps + marker census delta. The frontend/
backend split (below) exists partly to keep waves deferral-free: frontend
waves never wait on GL work.

## Stages

- **R0 — strategy sit-down (user ratifies).** The load-bearing decisions:
  1. **Frontend/backend split.** Frontend = CPU side: shader-file parsing,
     BSP/world load, PVS/cull, surface sorting, curve tessellation, mdx
     skinning, light grid — deterministic, golden-testable without GL.
     Backend = GL command submission (RB_* / tr_backend).
  2. **Backend strategy — DEC-01 is the standing lean: wgpu**, as an
     idiomatic §F-track rewrite, explicitly NOT a fixed-function GL
     transcription; DEC-01 says "re-confirm specifics at that time" — R0 IS
     that re-confirmation. Consequence for validation: R4's gate becomes
     render-target/image goldens + wgpu command capture on fixed scenes
     (behavioral parity per §18), replacing the GL command-stream diff.
     Windowing/context via a thin platform layer (native tier).
  3. **Scope fence vs the engine island.** What the headless subset already
     owns (model/skin registry, mdx views + DEC-35 parsed sidecar) is the
     nucleus — the renderer port EXTENDS `crates/mp/renderer`, it does not
     start a parallel crate.
  4. **SP awareness.** SP's renderer is a close sibling (~55.9k); per §20,
     MP first, SP as diff later — but R2's type designs note SP divergences
     in doc comments where already known.
- **R1 — tooling prep (review-corrected).** The `mp-renderer` profile
  exists but is HEADER-ONLY (closure.py:210, no srcglob) — add a full
  function srcglob covering ALL `tr_*.cpp` (tr_bsp/tr_world/tr_scene/
  tr_curve/tr_light/tr_marks/tr_sky/tr_shade… are in NO existing srcglob;
  only 9 server-subset files live in mp-engine-ded's). Author
  renderer-world digest templates in `enginepackets.py` (it is not
  module-parameterized — engine-island digests don't retarget). Emit
  `renderer-fn-manifest.json`, call graph, wave partition; type inventory
  from `tr_local.h`/`tr_types.h`/`qfiles.h` renderer sections. Verify the
  profile parses `tr_main.cpp`, `tr_shader.cpp`, `tr_bsp.cpp` clean (R1
  gate, mirrors the ui U0 dry-run).
- **R2 — root-type sit-down (user ratifies).** Reconcile FIRST with the
  already-ported type layer (review finding): `crates/mp/renderer/src/
  tr_local/` carries glstate_t/shader_stage_t/texture_bundle_t/tex_mod_t…,
  plus `tr_public/refexport_t.rs` and `mdx_format/` — extend, never
  re-propose. Then the owned renderer world
  (`trGlobals_t`/`backEndState_t`/`glState_t` equivalents — one owned
  instance, threaded, no statics); shader/skin/image arenas beside the
  existing model registry; command-buffer ownership (backEndData double
  buffer); the frozen seam set (`refEntity_t`/`refdef_t`/`polyVert_t`/
  `glconfig_t` — repr(C) + asserts, already Class B); the `refexport_t`/
  `refimport_t` boundary as traits; image/texture upload path ownership.
- **R3 — frontend port.** The headless island already ports parts of
  tr_shader/tr_image/tr_model/tr_init/tr_main/tr_backend (mp-engine-ded
  lane + sv_renderer.rs) — R3 waves start from the REMAINDER; never a
  second divergent port of an already-ported fn (review finding). Topo
  waves gated by §F differential goldens in
  `tools/renderer-oracle/`: compile unmodified oracle frontend TUs
  standalone (stub GL + engine imports, oracle never edited), dump canonical
  outputs over committed fixtures — shader parse of shipped shader files,
  BSP load of mp/duel1 (surfaces/nodes/lightgrid digests), tessellation and
  mdx-skinning outputs on fixed inputs. Committed goldens; `cargo test`
  needs no C++ toolchain (established §18 pattern).
- **R4 — backend port.** Per DEC-01's wgpu lean (re-confirmed at R0): an
  idiomatic wgpu backend behind the faithful frontend, smallest surface
  first (clear/upload/draw for world + entities). Gates: render-target/
  image goldens + wgpu command capture on fixed scenes (§F behavioral
  parity — the frontend goldens pin everything CPU-side, so backend diffs
  localize to draw translation). Unsafe confined to the binding layer.
- **R5 — windowed dev harness.** Before any client-engine work: a minimal
  host that opens a window, loads a map through our FS/CM, renders a
  scripted flythrough with test refEntities. This is the renderer's "live
  boot" — the in-person gate. The full cl_* client engine island is a
  separate later plan; this harness is its forerunner and keeps the
  renderer independently testable.

## Sequencing & capacity

ui leads; R0-R2 (sit-downs + tooling) may run during ui waves; renderer
transcription starts once ui waves are self-sustaining, sized so the review
step never queues. cgame (its own plan, after ui) consumes the frozen
`tr_types.h` set from R2 — coordinate the assert layer so both land once.

## Plan validation record (2026-07-24)

- C++ walker machinery previously proven end-to-end on the engine island
  (qcommon/botlib/server + seven C++ subsystems).
- Renderer TU parse check deferred to R1 gate by design (same operation as
  the validated ui dry-run, C++ profile).
- Census: renderer's module-facing structs confirmed Class B (copy-at-call,
  `tr_scene.cpp:194-254` cite in the scoping doc).
- Adversarial review run 2026-07-24; findings folded in.
