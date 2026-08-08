# Synopsis gh#31 step-008 - the 2D closure

## Intent

This step ports the two rotate-pic executor arms, the last dark row of the census 2D group (`2d/DrawRotatePic`, 547 submissions, both `DrawRotatePic` and `DrawRotatePic2` counted there). It also gives the group its first two goldens, because no committed fixture draws a single 2D quad today.

## Surface contract

- `QuadBatch::push_quad_xy` (new), with `push_quad_st` refactored to call it.
- `FrameStats::rotate_pics` replacing `FrameStats::skipped_rotate_pics`.
- `Warned::RotatePic` deleted.
- `FrameExecutor::draw_rotate_pic` and `FrameExecutor::draw_rotate_pic2` (private).
- `rotated_corners` (private free fn).
- The two executor arms replacing the skip arm.
- `crates/mp/renderer-gpu/tests/hud_golden.rs` with `golden_hud_2d` and `golden_hud_font`.
- `tests/goldens/hud_2d.png` and `tests/goldens/hud_font_ocr_a.png`.

Anything not on this list is out of scope. No ABI change, no cvar, no `FrameEvent` variant, no new crate.

## Commits

1. `feat(gh#31 s008): the rotate-pic executor arms` - the batch method, both draw methods, the arms, the stat.
2. `test(gh#31 s008): the synthetic 2D golden` - the new test file and `hud_2d.png`, after the bless STOP.
3. `test(gh#31 s008): the retail font-string golden` - `golden_hud_font` and `hud_font_ocr_a.png`, after its own bless STOP.
4. `process(gh#31 s008): finished file`.

Every commit gates on `cargo build --workspace` with zero warnings, `cargo test --workspace -- --test-threads=1`, and all thirteen existing fixtures byte-identical (world, scene, entity, ghoul2 suites, each serial).

## Open rows

1. **user ruling - the `RB_RotatePic` blend state.** The oracle issues no `GL_State` for this arm, so it inherits the last one. Default: draw at `GLS_2D_DEFAULT` with a two-line note; the alternative is a frame-local state register that would still not be exact.
2. **mechanical - the golden file.** Default: one new file `hud_golden.rs` holding both tests.
3. **user ruling - the retail font golden's asset gate.** Two idioms exist in the workspace. Default: `#[ignore]` plus a hard assert, matching every other retail golden in this crate.
4. **user ruling - the bless flow.** Default: the step-007 procedure once per golden, with a STOP before each PNG commit and named defect conditions for both images.
5. **mechanical - the font and string.** Default: `ocr_a`, `"jka-rust ^1font ^7golden"` at `(32, 64)`, scale 1.0.
6. **mechanical - the stat counter.** Default: `rotate_pics` replaces `skipped_rotate_pics`; no reader outside `frame_exec.rs`.
7. **mechanical - the unknown-shader fallback.** Default: mirror `draw_stretch_pic`, warn once and push a white rotated quad.
8. **mechanical - the stage-0 image read.** Default: read `bundle[0].image` directly, never `stage_image`; the oracle resolves no animation frame here.

## Dispatch flags

- Oracle ambiguity: **true**. `RB_RotatePic` inherits its GL state, and it reads `stages[0]` past a zero-length allocation for a zero-pass shader.
- New state home: **false**. Row 1's alternative would be a frame-local beside `color`, not a field.
- ABI or parity-gate surface: **true**. Two new committed goldens join the gate battery. No ABI change.
- Divergence proposal: **true**. The zero-stage read maps to draw-nothing, the rotate pic appends in command order instead of jumping ahead of the pending batch, and row 1's default names the inherited blend as a divergence.
