# Finished file - packet gh#31 step-003

## Status

Both delivered commits are green. The absorption landed, the `re` slot is seated at every harness view except the 2D paint path, the gh#35 A1 override is gone, and the ghoul2 vertex fixture is re-blessed under Raven's client register path. No pause trigger fired. The branch is `gh31-step-003-absorption`, unmerged and unpushed.

## The commit-1 byte-identity results

Commit 1 landed the mechanical absorption with the A1 override still standing, so every fixture had to hold. All three suites did.

| Suite | Invocation | Result |
|---|---|---|
| ghoul2 vertex golden | `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` | 1 passed, byte-identical to the committed fixture, 53.91s |
| world goldens | `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` | 2 passed (duel1, ffa2), both byte-identical, 77.93s |
| scene goldens | `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1` | 7 passed, 40.96s |

The ghoul2 golden reaching its byte comparison and matching is the strong statement: the 15 construction and destructure sites, the 56 dot accesses, the 16 `host_view` seats, and the frame-pinned clone moved no behavior.

## The commit-2 multiset comparison

The re-bless ran with `JKA_GOLDEN_BLESS=1`, then a clean confirming run without it. **The blessed bytes equal the committed bytes exactly**, so `crates/mp/renderer-gpu/tests/goldens/ghoul2_verts_stormtrooper.bin` is unchanged in the tree and the commit carries no fixture diff.

The digest recipe was first re-derived against the pre-bless fixture and reproduced the gh#35 record exactly, so the comparison below uses a checked recipe. Per surface, the digest is SHA-256 over the surface's whole fixture record, the two `u32` counts plus the float and index payload. The multiset digest is SHA-256 over the concatenation of the 22 full hex digests in sorted order.

| Quantity | Recorded at the gh#35 step-001 close | The re-blessed fixture |
|---|---|---|
| bytes | 95472 | 95472 |
| surfaces | 22 | 22 |
| multiset digest | `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264` | `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264` |
| draw-surf order | the server-path order | the same order, permutation identity |

The packet expected the content to hold and the order to move. The content held. The order also held, and the model itself is the reason. Every `mdxmSurfHierarchy_t` in `models/players/stormtrooper/model.glm` carries an empty shader name (all 81 entries, read directly out of `assets1.pk3`). `R_FindShader` returns slot zero for an empty name (`crates/mp/renderer/src/tr_shader.rs:5590-5592`), so the client leg writes `shaderIndex = 0` (`tr_ghoul2.rs:2116-2121`), which is the value the server leg forced (`server_load.rs:508`). With custom shader 0 and skin 0, all 22 drawn surfaces still resolve to the slot-0 shader, the 22 sort keys stay tied, and `qsortFast` produces the same permutation it produced before. The survey's section C prediction was sound in mechanism and wrong in outcome for this one model, and the correction belongs on the gh#31 record.

The client leg did run. `dedicated` is `"0"` and the harness holds no VM, so `g2_test_model_pointers` takes the client leg, which is the same reasoning that predicted the gh#35 abort. With the override removed, the hook is the real `re_register_model_hook`, and a null slot there is the abort gh#35 recorded. The test passes, so the seated slot carried the client register path through the whole frame.

### The new baseline for a future re-bless

The fixture is unchanged, so the baseline is unchanged. Restated here so a future re-bless needs one file, not two. 22 surfaces, 95472 bytes. Per-surface SHA-256 digests in fixture order, first 16 hex digits each:

`f9e0e13bcfc0dab7`, `1b854050fc3031b5`, `bbe046193bc65ece`, `d6cfdfbcdfe13ed0`, `a9922d142c8c58c8`, `52be028677018ead`, `9a7595388d0b9657`, `01a2fff512738652`, `f851d08522c5860d`, `e92bcc43a65146da`, `6bf37b59311fbbf0`, `4999aca0fe7ac408`, `088d014278993d21`, `af71d9562eee20bd`, `d1c1fa0dffa1a518`, `8441f5dce6c4022c`, `d682ad4d0b0d0467`, `3bd47ed030af0e30`, `c7e13cbc11aa302b`, `7691b35b609f3276`, `a4b5f4629d9bf99a`, `0e047ed84c4dd2c8`

Multiset digest over the sorted full-length list: `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264`.

## Assumptions and choices, keyed to commits

**Commit 1, the shape of the seat.** `host_view` takes `re: *mut RendererFrontend` as a fifth parameter, which the contract named. Each site takes the pointer with `let re_ptr: *mut RendererFrontend = &mut host.re;` placed immediately before the `UiHost` destructure, because the two-level destructure consumes the `re` field and no whole-bundle binding survives it. The `models_ptr` precedent takes its pointer from the destructured binding instead, which the `re` seat cannot do.

**Commit 1, the destructures.** Every site uses one nested pattern, `re: RendererFrontend { .. }` inside the `UiHost` pattern, rather than a two-statement split. One statement keeps the borrow split visible at the site and reads the same as the old flat pattern.

**Commit 1, the four pinned sites.** The four `execute_frame` entity-walk sites drop `sim` from the destructure entirely, because `&sim.published` was its only use and `&pinned` replaces it. `load_world_and_render` keeps `Arc::make_mut` and carries a comment naming why it is exempt.

**Commit 1, the boot-time bundle.** `boot_renderer` builds `RendererFrontend::new()` before the engine subset, beside `RenderModels::default()`, and moves it into the `UiHost` literal. The 12 old twin seeds equalled the constructor's field for field, so no seed value moved. The engine-subset block seats the pointer at its own `host_view` call, which keeps every view in this file uniform.

**Commit 1, ruling 1.** `re.frame_data` is inert. Every paint and every golden keeps its per-call `FrameData`, and the `R_Init` block carries a one-line comment stating the ruling at the one place a reader would ask.

**Commit 1, the doc split.** The `state.rs` docs, the `host_view` doc, and the `renderer_frontend.rs` seating note landed in commit 1, because they describe the absorption that commit delivers. The golden's module-doc provenance landed in commit 2, because it states the re-bless outcome.

**Commit 2, the provenance wording.** The draft text in `a2-attempt.patch` asserted a moved order and a real per-surface shader index. Both are false for this model, so the landed paragraph states the measured outcome and its cause instead.

## Deviations

Two, both procedural.

1. **The scene-golden invocation.** The brief's command was `cargo test -p mp_renderer_gpu --test scene_golden -- --ignored --test-threads=1`. The seven scene goldens carry no `#[ignore]`, so that command reports `0 passed, 7 filtered out` and gates nothing. Every scene-golden run in this lane used `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, which runs all seven. They also run inside `cargo test --workspace`.
2. **The fixture did not change.** The packet's commit 2 named a re-bless, and the re-bless ran, but its output equals the committed bytes. The commit therefore carries no fixture diff. The bless run and the confirming clean run both passed, so the re-bless is done, not skipped.

No scope deviation. No surface outside the contract changed, no `RE_*` signature moved, and the only new `pub` surface is the `host_view` parameter.

## Commits

1. `0def8283` - `refactor(gh#31 s003): UiHost owns a real RendererFrontend, the re slot seated`. Gates: `cargo build --workspace` green, `cargo test --workspace` green (exit 0), the ghoul2 vertex golden byte-identical, both world goldens byte-identical, the 7 scene goldens green. No `.actual.bin` or `.actual.png` at any point.
2. `849251f9` - `refactor(gh#31 s003): the client register path, and the ghoul2 re-bless`. Gates: `cargo build --workspace` green, `cargo test --workspace` green (exit 0, 516 passed, 0 failed, 18 ignored), the ghoul2 golden green on a clean run against the re-blessed fixture, both world goldens byte-identical, the 7 scene goldens green, the multiset comparison recorded above. No `.actual.bin` or `.actual.png` at any point.
3. This finished file.

## Open gaps

The survey's re-bless prediction needs a correction on the gh#31 record: a `.glm` whose surface hierarchy carries empty shader names gets `shaderIndex = 0` on the client leg as well, so the client path alone does not untie the ghoul2 sort keys. A model that names its shaders in the hierarchy, or a draw through a `.skin`, is where the untie shows. Step-004 should not assume the order is now free to move.

The seated slot's per-field aliasing is the pre-existing `rm`-slot class, not a new one. A hook reached during a frame forms `&mut RendererFrontend` from the raw slot while the executor holds `&mut` borrows of `world_load`, `img_state`, and `noise` from the same bundle. The frame-pinned clone closes the one case with a real data write behind a live read, `sim.published`. The rest is governed by the per-slot rule at `crates/mp/renderer/src/hook_install.rs:7-11`, which the survey classed as pre-existing and out of this step. It is worth a ruling before the harness grows a second in-frame hook.

`re.frame_sink`, `re.pending_capture`, `re.pending_world`, and the two screenshot counters are inert in the harness, and `re.frame_data` is inert by ruling 1. A future frontend-driven world load that sets `pending_world` would find nothing draining it, because the harness switches worlds through `executor.set_world` directly.

The scene goldens carry no `#[ignore]`, unlike the world and ghoul2 goldens, so they run in every `cargo test --workspace` and add about 41 seconds to it. That is a pre-existing difference and this lane did not change it.
