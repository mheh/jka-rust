# The ghoul2 golden null `re` slot, diagnosis and audit (gh#35)

This record freezes the gh#35 investigation and the audit that checked it. The session diagnosed the `ghoul2_vertex_golden --ignored` abort on 2026-08-04, reproduced it, tried one fix, reverted it, and posted the diagnosis as a comment on https://github.com/mheh/jka-rust/issues/35. A read-only Fable auditor then audited that diagnosis, under a brief that told it to break the claims rather than confirm them. No code changed: the working tree was clean before the audit and clean after it. Follow-up work goes to gh#35 itself.

The diagnosis under audit, in eight claims: (1) the abort path runs from `g2api_init_ghoul2_model` to `re_from_view`; (2) the `re` slot is NULL by construction in `boot::host_view`; (3) the port is faithful, because Raven guards the same branch with `if (currentVM && currentVM == gvm)`; (4) the cheap fix, seeding the `dedicated` cvar nonzero, is wrong because `tr_image.rs:2499` gates image loading on `com_dedicated`; (5) the surface reordering seen under that experiment was self-inflicted and is not a hidden regression; (6) the fixture was blessed on the server path before `00569c66` made the gate run Raven's real logic; (7) the fix is that `UiHost` owns a real `RendererFrontend`; (8) the re-bless will not move geometry, because the stormtrooper reports `numBones = 53` and the 72-bone remap does not fire.

## The audit report, verbatim

All evidence is collected. Here is the audit report.

### Claim verdicts

**Claim 1 - the abort path: VERIFIED (structurally).** Every link in the chain exists in the current tree. The test calls `init_ghoul2` at `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs:372`, which calls `g2api_init_ghoul2_model` at line 140 through `boot::host_view`. That init calls `g2_test_model_pointers` at `crates/mp/engine/ghoul2/src/api_models.rs:319`, which selects the client leg at lines 199-203 when the `dedicated` cvar is 0 and the gate is false. `register_model` (line 177) calls `EngineHost::model_register_client`, implemented at `crates/mp/engine/qcommon/src/common/engine_host_view.rs:407-414`, which fires the `RE_RegisterModel` hook installed at `crates/mp/renderer/src/hook_install.rs:69`. The hook body at lines 78-80 calls `re_from_view`, and line 58 dereferences the slot. The harness never creates a VM, `Engine::new` allocates `Common` zeroed (`crates/mp/engine/core/src/engine.rs:108-121`), so `vm_current_is_game` (engine_host_view.rs:419-427) returns false, and `boot_renderer` seeds `dedicated` to "0" at `crates/mp/renderer-gpu/src/ui_host/boot.rs:156`. I did not rerun the test, so the captured backtrace itself is taken as reported.

**Claim 2 - the slot is NULL by construction: VERIFIED.** `boot::host_view` sets `re: SlotRenderer::from_raw(null_mut())` at `crates/mp/renderer-gpu/src/ui_host/boot.rs:430`. `UiHost` (`crates/mp/renderer-gpu/src/ui_host/state.rs:30-70`) holds the carriers as separate fields under the DEC-42.3 header at line 38 and holds no `RendererFrontend`. No other code in the client tree seats that slot. The only other `SlotRenderer::from_raw` is also null, in `crates/mp/engine/server/src/botlib_import.rs:102`.

**Claim 3 - the port is faithful: VERIFIED.** The oracle gate is `if (currentVM && currentVM == gvm)` at `oracle/codemp/ghoul2/G2_API.cpp:572`, inside `G2_ShouldRegisterServer` at lines 570-583. `G2_TestModelPointers` guards with `(com_dedicated && com_dedicated->integer) || (G2_ShouldRegisterServer())` at oracle lines 2616-2617 and falls to `RE_RegisterModel` at line 2623. With no VM and `dedicated` 0, Raven takes the client leg. The Rust body at api_models.rs:149-162 matches the oracle line for line.

**Claim 4 - the cheap fix is wrong: mechanism VERIFIED, the run itself UNVERIFIABLE.** The image gate is real: `crates/mp/renderer/src/tr_image.rs:2499` returns `None` from `R_FindImageFile` when `com_dedicated` is nonzero, with Raven's own comment at line 2500. 480000 equals the 800x600 frame the goldens render (test constants at lines 72-73). The working tree is clean and boot.rs:156 still seeds "0", so nothing from the experiment is committed. I did not rerun the world goldens.

**Claim 5 - the reordering was self-inflicted: conclusion SUPPORTED, the stated mechanism PARTLY REFUTED.** See section B.

**Claim 6 - why it never ran since 2026-08-02: VERIFIED, with one correction.** At `2c05db29` the gate returned `true` unconditionally unless `cl_running` was set, and `register_model` was a `host.error(ERR_DROP, ...)` stub (both shown by `git show 2c05db29:crates/mp/engine/ghoul2/src/api_models.rs`, gate at lines 151-169). The `dedicated` seed was already "0" at that commit, so the blessing ran the server registration path. `00569c66` (2026-08-02) is the commit that made the gate run Raven's real logic. The correction: the committed fixture bytes were last blessed at `bc856508`, 32 minutes after `2c05db29` on the same day (2026-07-31, `git log --follow` on the fixture). Both bless commits predate `00569c66`, so the substance holds. The "aborted ever since" statement rests on the clerk reproduction at `e9c7cf8c` recorded in the issue body, which I did not rerun.

**Claim 7 - the survey counts: VERIFIED except one number.** The 12 twins are exact by name and type: cvars, sim, img_state, frame, world_load, scene, noise, rng, font, world_effects, qs, sky_view (state.rs:39-55 against `crates/mp/renderer/src/renderer_frontend.rs:68-130`). The 7 missing fields are frame_data, frame_sink, pending_capture, pending_world, screenshot_last_number, screenshot_jpeg_last_number, automap. `models` stays outside by the frontend's own doc (renderer_frontend.rs:62-63). I count exactly 14 `let UiHost {` destructures plus the 1 literal at boot.rs:179. My dot-access count on receiver `host.` to the 12 twins is 47, not 53. The 53 may come from a wider counting rule, so the magnitude stands and the exact figure does not reproduce. The world_harness twin defect is real: `crates/mp/renderer-gpu/src/bin/world_harness.rs:1128-1140` uses the same `boot::host_view`.

**Claim 8 - re-bless expectation: VERIFIED.** The remap gate is `numBones == 72 && anim_name.contains("_humanoid")` at `crates/mp/renderer/src/tr_ghoul2.rs:2044-2046`. I read the retail header myself from `/Users/milohehmsoth/Developer/jka/jka_server/base/assets1.pk3`: ident `4d474c32`, version 6, `numBones = 53`, `animName = models/players/_humanoid/_humanoid`. The first conjunct fails, so the remap does not fire. I also checked the other client-load differences: `Q_strlwr` and the `_off` strip touch only the name bytes (tr_ghoul2.rs:2061-2083), the shader resolve touches only `shaderIndex`, and the `LittleLong` swaps are identity on this little-endian machine. One sharpening: with real per-surface shader indices the 22 keys untie, so the order will almost certainly move, not "possibly".

### A. Is there a cheaper honest fix?

Yes. Two candidates beat the wholesale absorption for this golden, at different prices.

**A1 - a harness-local hook override, about 10-15 lines, no re-bless.** After the install at boot.rs:131, overwrite `hooks.RE_RegisterModel` with a harness function that casts the `rm` slot (`rm_from_view` is public, hook_install.rs:36) and calls `RenderModels::register_server_model`. This reproduces the blessed configuration exactly: server-path registration with `dedicated` still 0, so the image gate at tr_image.rs:2499 never fires and the world goldens stay green. The committed fixture stays valid. Cost: the rig never runs Raven's client register path, so `RE_RegisterModel` and per-surface shader assignment stay untested, and the gh#31 entity-draw steps that want real shaders hit this same wall later. It also makes the harness deliberately unfaithful on this one path, which the diagnosis's own claim 3 argues against. The override is visible at the boot site, so it passes the no-silent-fake rule with a comment.

**A2 - a scoped frontend window, about 40-60 lines, re-bless required.** In `init_ghoul2`, mem-swap the 12 twin fields out of `UiHost` into a `RendererFrontend`, default the 7 extras, point the `re` slot at it, run the init, and swap the twins back. This works because the client register hook touches only twin fields: `qs`, `world_load`, `sim.published`, `cvars`, `img_state`, `sky_view`, `world_effects` (hook_install.rs:84-95). No mutation lands in a discarded extra. It runs the real client path, so it needs the same re-bless the big fix needs, and it fixes only the init window. The 90-plus other `re_from_view` call sites in `cl_scrn`, `cl_ui`, `cl_cgame`, and `fx_host` stay unservable by the harness, so the structural motivation for the big fix survives.

**Rejected candidates, with reasons.** Changing what the golden asserts cannot work, because the abort happens inside init, before any capture exists. A null-check fallback inside `re_register_model_hook` would reroute a genuine client boot-order defect to the server path in production, against the invariant documented at hook_install.rs:73-76. A throwaway `RendererFrontend` that is not swapped from the harness fields is dishonest: the shader registrations during `RE_RegisterModel` would land in a registry the draw never reads.

### B. Is claim 5 sound?

The conclusion is supported. The because-clause is wrong.

The mechanism family is confirmed: the sort key packs `shader->sortedIndex` at bit 18 (`QSORT_SHADERNUM_SHIFT`, `crates/mp/renderer/src/tr_main.rs:128`, packed at lines 868-874), and the ghoul2 path feeds it the resolved shader's `sorted_index` (tr_ghoul2.rs:906-931). So the shader half of the key does control ghoul2 surface order.

The stated contrast does not apply to the two runs that were compared. The blessed run took the server registration path (claim 6), and the `dedicated` experiment also takes the server registration path, because `g2_test_model_pointers` short-circuits on the cvar (api_models.rs:199-201). Both runs therefore had `shaderIndex` forced to 0 on every surface (server_load.rs:508), and nothing pokes the stored shader requests, because no code in `renderer-gpu` calls `models_level_load_end`. With custom shader 0 and skin 0, all 22 surfaces resolve to the slot-0 shader (tr_ghoul2.rs:887-893), so all 22 sort keys were identical in both runs. A "server forces 0, client assigns real" split cannot distinguish two runs that both forced 0.

The correct statement: the experiment's image stub changed the shader registry. World shaders that fail their image loads resolve differently, their `sortedIndex` values move, and the slot-0 shader's `sortedIndex` can move too. `qsortFast` (tr_main.rs:1178) is an unstable quicksort, so a 22-element block of tied keys lands in a permutation that depends on the whole surrounding array. Changed world keys therefore permute the tied ghoul2 block. That is an experiment-caused reorder through the shader half of the key, by a different route than the diagnosis states.

One residual hole. The experiment differs from the blessed run in two ways at once: the image stub, and every mainline commit since `bc856508` (2026-07-31). The diagnosis attributes the whole reorder to the first and does not exclude the second. The discriminating control is cheap: run today's tree in the A1 configuration (server path, `dedicated` 0, images on) and compare order to the fixture. An identical order pins the reorder on the experiment. A moved order reveals mainline order drift the experiment masked. I did not run this, so the reorder is explained as plausible, not proven. The byte-identity of the surfaces is the user's own measurement and does mean no vertex-content regression is in play either way.

### C. What does the proposed fix break?

**Mechanical churn.** 14 destructure sites, 1 struct literal, and about 47-53 dot accesses across `boot.rs`, `state.rs`, the three goldens, and the two bins. Every destructure becomes two-level. The flat-struct borrow rationale at state.rs:26-29 survives, because field borrows of `host.re.*` stay disjoint, but `models` must stay outside `re` or the destructures that borrow both stop compiling. The proposal already keeps it outside.

**The one live behavior change is `frame_data`.** The frontend owns a persistent `FrameData` (renderer_frontend.rs:86). The harness builds a fresh one per paint and per golden run (boot.rs:390, ghoul2_vertex_golden.rs:389, and the world_golden and world_harness twins). `RE_ClearScene` appends a `ClearScene` event and does not empty the vector (`crates/mp/renderer/src/tr_scene.rs:244-247`). The live path that empties the stream is `RE_EndFrame` (renderer_frontend.rs:87-90), which the harness never calls. If the harness adopts `re.frame_data` wholesale without an explicit per-frame clear, events accumulate and replay into every later frame. The fix must pick one: keep per-call construction and leave `re.frame_data` inert, or add an explicit clear at frame start. That decision is forced, and it is the only semantic dependence on the split shape I found. No harness test asserts the split shape itself.

**The other 6 new fields are inert in the harness.** `frame_sink`, `pending_capture`, and `pending_world` start `None`, the two screenshot counters start -1, and `automap` is empty. One note for later work: the harness switches worlds through `executor.set_world` directly, so a future frontend-driven world load that sets `pending_world` would find nothing draining it.

### Unverified

- The abort backtrace, the release-profile SIGSEGV named in the issue body, and the continuity of the abort since `00569c66`. I did not run the golden, per instructions. The chain is verified from source only.
- The 480000-pixel world-golden failure under the `dedicated` experiment. The gate mechanism is verified, the run is not repeated.
- The exact permutation seen in the experiment reorder, and the byte-identity of the reordered surfaces. Both are the user's measurements. The control run named in section B was not run.
- Whether mainline commits since `bc856508` moved draw-surf order independently of the experiment.
- The exact "53 further dot accesses". My rule counts 47 on receiver `host.`.
- The client `.gla` load path against the server one. Claim 8's "geometry unchanged" covers the `.glm` walk, which I diffed, but I did not diff the animation-file load legs.
- The `cl_running` cvar value in the harness at bless time. I infer 0 from the absent registration, and the blessed fixture's existence implies the error arm never fired.

## Corrections the audit forced

The audit changed three things in the record, and the issue comment on gh#35 states the superseded version of the first one.

1. **Claim 5's mechanism was wrong.** The diagnosis said the reorder came from the server path forcing `shaderIndex = 0` while the client path assigns real shaders. Both compared runs took the server path, so both forced 0 and all 22 sort keys were tied in each. The real route is that the image stub moved the world shaders' `sortedIndex` values, and `qsortFast` is an unstable quicksort, so a tied 22-element block permutes with the surrounding array. The conclusion, that the reorder is experiment-caused rather than a vertex regression, stands.
2. **The reorder is plausible, not proven.** The experiment changed two things at once: the image stub, and every mainline commit since 2026-07-31. A control run in the A1 configuration discriminates them, and it has not been run.
3. **Two counts move.** The fixture was last blessed at `bc856508`, not `2c05db29`, 32 minutes later the same day. The dot-access count is 47 on the auditor's rule, against 53 from the survey.

## Rulings

None yet. The choice between A1, A2 and the wholesale absorption is open, and gh#35 stays open with nothing committed against it.

## The step-001 lane and the close, appended 2026-08-04

The user ruled "do A1 and A2", and one opus lane ran packet `.claude/packets/35/step-001/packet.md` on branch `gh35-step-001-golden-rig`. A1 landed, A2 stopped on a pause trigger, and the user then ruled the close: A1 stands, and the client path moves to gh#31 step-003. The merge is `56a80eef`.

### The lane's report, verbatim

Lane stopped on a pause trigger. A1 landed; A2 did not.

**Commits on `gh35-step-001-golden-rig`:** `45480710` - `test(gh#35 s001): the ghoul2 golden control run on the server register path`, and `5bb7ff24` - `process(gh#35 s001): finished file, control-run verdict and the A2 stop`.

**Control-run verdict: byte-identical to the committed fixture.** The golden passed on the first run under the A1 configuration, so it reached the byte comparison and matched exactly. Order is identical to the fixture and content is identical. The 2026-08-04 reorder therefore belongs to the reverted `dedicated` experiment, and mainline moved nothing between the bless at `bc856508` (2026-07-31) and today. The audit's residual hole in section B closes, and its correction 2 upgrades from "plausible" to proven by elimination. The multiset check is subsumed by byte identity; the per-surface digest list is recorded in `finished.md` for future re-blesses.

**Gates (commit 1):** `cargo build --workspace` green. `cargo test --workspace` green, exit 0. Ghoul2 golden green. Both world goldens byte-identical. No `.actual.bin` produced at any point.

**The stop: fact 3 of the packet is false.** It claims nothing past the init call reads the `re` slot, citing the `dedicated` experiment as proof. That experiment proved nothing about the slot: a nonzero `dedicated` short-circuits every `dedicated || g2_should_register_server(host)` guard in the ghoul2 crate to the server path, so it masked two call sites, not one. With the frontend seated for the init call, the init succeeded - the `.glm` and `.gla` both disk-loaded through Raven's client `RE_RegisterModel`. The abort then moved to draw time, same line: `re_from_view` (`crates/mp/renderer/src/hook_install.rs:58`) reached from `misc::g2_setup_model_pointers` (`crates/mp/engine/ghoul2/src/misc.rs:403`) through `r_add_ghoul_surfaces`, `R_AddEntitySurfaces`, `R_RenderView`, `FrameExecutor::execute_frame`. `G2_SetupModelPointers` re-registers the model on every frame's entity walk, which is faithful Raven behavior. The client path needs a seated `re` for the whole frame render, not for the init call. The shape is the blocker: while the twins sit in the frontend, the `UiHost` fields hold placeholders, and the golden feeds `sim.published`, `world_load`, `img_state`, `noise` from `UiHost` straight into `execute_frame`. A frame-length window forces the draw to read its receivers off the frontend. That changes `init_ghoul2`'s contract signature and needs a new pub item. The lane did not rule on it.

**Deviations:** one - commit 2 not delivered. Its edits compiled clean but sat uncommitted, because the golden aborts at draw time and a red gate must not land. The session preserved them as `.claude/packets/35/step-001/a2-attempt.patch` and cleaned the tree before review.

**Two further findings:** `world_harness` has had the same draw-time abort since `00569c66` (2026-08-02) and neither document recorded it, and the fact-3 claim in this record plus the issue comment carrying it both needed correcting.

### The clerk's verification

Lane-review ran with a conformance clerk on opus, per the user's instruction. It walked the whole diff and found no letter violation in the delivered code: the one added statement is the exact reassignment the contract names, at the named place, and the two hook fields have one signature (`engine_hooks.rs:165,168`). Its gate re-runs, verbatim:

| Gate | Claim | The clerk's run |
|---|---|---|
| `cargo build --workspace` | green | exit 0, `Finished dev profile in 1m 13s` |
| `cargo test --workspace` | green, exit 0 | exit 0. 516 passed, 0 failed, 18 ignored |
| the ghoul2 golden, serial | green, byte-identical | exit 0, `1 passed; 0 failed`, 125.81s, no `.actual.bin` produced |
| the world goldens, serial | green on both maps | exit 0, `2 passed; 0 failed`, 73.31s |
| the multiset check | subsumed by byte identity | re-derived independently: 95472 bytes, 22 surfaces, all 22 digest prefixes and the multiset digest `f3551f6a...ef7264` match `finished.md` |

Its ledger findings, all accepted with dispositions: (a) the landed comment forward-referenced an undelivered commit 2, closed by the reword at `a4b84cd8`; (b) the override's crate-wide reach was unstated - every `mp_renderer_gpu` boot now takes the server register path for ghoul2, which also clears `world_harness`'s latent crash; (c) and (d) the finished file described the pre-preservation tree and its line cites key to the A2 patch, both caused by the session's evidence-preservation commit `b9eef39f` and documented in the packet Amendments; (e) the preserved patch's module doc asserts a re-bless that did not happen, which is evidence, not delivered code. Its house-style flags on "lane talk" in the process files were ruled non-violations: "lane" is this repo's process vocabulary in packet and finished files, the same standing as "seam" in port comments.

## Ruling: option 1, appended 2026-08-04

The user ruled the close on 2026-08-04: A1 stands as the harness's standing configuration, gh#35 closes on the merge, and the honest client path arrives with the wholesale absorption - `UiHost` owns a real `RendererFrontend` - which the gh#31 step-003 packet must name as a prerequisite for its entity image goldens. The absorption inherits the audit's section C scoping: 14 destructure sites plus 1 literal, 47 to 53 dot accesses, `models` stays outside, and the forced `frame_data` decision (keep per-call construction, or clear at frame start).

Correction to this record's own section A: A2 as scoped there cannot exist. Fact 3 of the step-001 packet, which restated section A2's window premise, is refuted by the draw-time re-register at `misc.rs:403`. Section B's residual hole is closed by the control run: the order is byte-identical, so the experiment reorder is proven experiment-caused and mainline carries no drift since `bc856508`.
