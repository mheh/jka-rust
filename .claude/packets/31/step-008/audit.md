# Audit gh#31 step-008 - the 2D closure

Method: the auditor walked the oracle arms, the command path, the shader allocation, and the Rust neighbors before it opened the packet. Every cite below was opened at the quoted lines.

## The oracle picture, independently established

`RE_RotatePic`/`RE_RotatePic2` append a `rotatePicCommand_t` in call order (`oracle/codemp/renderer/tr_cmds.cpp:244-291`). `RB_RotatePic` (`tr_backend.cpp:1498-1541`) null-gates the stage-0 image, runs `RB_SetGL2D` only when `projection2D` is false, issues no `GL_State`, and draws an immediate-mode quad about the pivot `(x+w, y)` with corners `(-w,0) (0,0) (0,h) (-w,h)`. `RB_RotatePic2` (`:1547-1602`) gates on `numUnfoggedPasses`, installs `GL_State(stages[0].stateBits)`, pivots at `(x,y)` with corners at `±w/2, ±h/2`, and its tail restores `GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA` (`:1600-1602`). Neither arm flushes the pending `tess` batch, so a rotate pic renders before stretch pics submitted earlier and still unflushed. `projection2D` resets at `RB_BeginDrawingView` (`:491`) and `RB_SwapBuffers` (`:1881`).

## Row verdicts

**Row 1, `RB_RotatePic` blend (user ruling) - framing confirmed, default supported.** The arm truly issues no `GL_State`: `tr_backend.cpp:1498-1541` contains none. The inherited state is `GLS_2D_DEFAULT` after a fresh `RB_SetGL2D` (`:1282-1284`) or after any `RB_RotatePic2` (its tail restores the same bits, `:1600-1602`), and it is the last flushed shader's final stage bits when a `tess` flush intervened. Two facts the packet does not state, both favoring the proposed default. First, `golden_hud_2d` observes no flush between `RB_SetGL2D` and its rotate pics, so `GLS_2D_DEFAULT` is oracle-exact for the committed fixture, not an approximation there. Second, the only MP `CG_DrawRotatePic` caller is the rocket-lock wedge loop (`oracle/codemp/cgame/cg_draw.c:5945`), whose preceding flush state is genuinely frame-dependent, so the alternative register would also miss it. The packet's honesty about the alternative ("an unnamed approximation") is correct.

**Row 2, the golden file name (mechanical) - cleared.** No existing 2D test file exists to collide with, and the per-file helper-copy claim matches the crate: each of the four test files carries its own `golden_path`/`compare` set.

**Row 3, the retail asset gate (user ruling) - framing confirmed.** Idiom A verified: `world_golden.rs:420` carries `#[ignore = "needs retail assets and a GPU; run locally with --ignored"]` and `:243` hard-asserts the load (`assert!(loaded, "{map} did not load")`). Idiom B verified: `crates/jampgame/tests/referee.rs:905-912` prints `SKIP` and returns, `crates/cgame/tests/replay_referee.rs:184-190` the same. The proposed default matches every retail golden in this crate.

**Row 4, the bless flow (user ruling) - framing confirmed.** The step-007 procedure is as claimed: bless run, byte-identical re-run, STOP before the fixture commit for the user's eyes (`.claude/packets/31/step-007/packet.md:112-114`).

**Row 5, the font choices (mechanical) - cleared.** `ocr_a` is the cgame small font (`crates/mp/cgame/src/cg_main.rs:5267`), `RE_RegisterFont` at `crates/mp/renderer/src/tr_font.rs:1936`, `RE_Font_DrawString` at `:3051`, the glyph-pair recording at `:2772-2779`. All as cited.

**Row 6, the stat counter (mechanical) - cleared.** `skipped_rotate_pics` has no reader outside `frame_exec.rs` (workspace grep: lines 113, 135, 528 only). `skipped_events()` keeps its two bin readers (`dev_harness.rs:422`, `ui_harness.rs:304`) and survives the dropped term.

**Row 7, the unknown-shader fallback (mechanical) - cleared, one nit.** The mirror claim holds (`frame_exec.rs:891-895`). The nit: the oracle's `R_GetShaderByHandle` returns the default shader, whose stage 0 binds the checkerboard `defaultImage`, so the oracle would draw a checkerboard, not white. The white quad is the already-accepted stretch-pic idiom, so the default stands as the consistent choice.

**Row 8, the stage-0 image read (mechanical) - CHALLENGED on the factual account, default can stand with a note.** The single-pointer claim holds for `map`/`clampmap` stages. It fails for `animMap`: the oracle repurposes `bundle[0].image` to point at a Hunk array of `image_t *` pointers (`tr_shader.cpp:1441-1442`: `stage->bundle[0].image = (image_t*) Hunk_Alloc( numImageAnimations * sizeof( image_t* ), h_low ); memcpy(...)`), so `RB_RotatePic`'s `GL_Bind(image)` on an animMap shader reads pointer bytes as an `image_t` - type-confused garbage, not frame 0. The Rust `bundle[0].image` stays `None` for animMap stages (frames land in `image_animations` only, `crates/mp/renderer/src/tr_shader.rs:4659-4660`), so the proposed direct read maps oracle garbage to draw-nothing. That is a legitimate rule-19 pick of defined behavior, but it is an unnamed divergence: the row's rationale ("resolves no animation frame") describes the oracle wrongly, and the site needs the ≤2-line rule-19 note naming the animMap case. The video-map half of the row is confirmed: the Rust `videoMap` arm leaves the handle `-1` and the image unset (`tr_shader.rs:4673-4684`).

## Divergence-note verdicts

**Note 1, the zero-stage shader - DISPUTED.** The packet claims "that array is a zero-byte `Hunk_Alloc`, so the read is out of bounds". False: `GeneratePermanentShader` allocates at least one stage (`tr_shader.cpp:2782-2783`: `size = numUnfoggedPasses ? ... : sizeof( stages[0] )`), and `Hunk_Alloc` zero-fills (`z_memman_pc.cpp:791-793` passes `qtrue` as `Z_Malloc`'s `bZeroit`, memset at `:183`). A zero-pass shader therefore reads one zeroed stage, its `image` is `NULL`, and `RB_RotatePic`'s `if (image)` skips the draw. Draw-nothing is oracle parity, not a rule-19 divergence. The Rust plan (`stages.first()` -> `None` -> 0) needs no change - the Rust permanent copy also holds only active stages (`tr_shader.rs:3424-3433`) - but the divergence note must be rewritten as a parity comment or dropped.

**Note 2, the float color register - confirmed.** `RB_SetColor` quantizes to bytes (`tr_backend.cpp:1409-1412`), the backend keeps floats, and the step adds no new quantization. This is the standing stretch-pic divergence, correctly labeled.

**Note 3, the command-order interleave - confirmed real, framing CHALLENGED as underweighted.** The mechanism is exactly as the packet says. The auditor adds the observability answer the session will need. Not observable in either new fixture: `golden_hud_2d`'s three quads do not overlap (the reference quad spans x 64-192, the first rotate pic spans x 203-384 after its 45-degree turn about `(384, 64)`, the second spans roughly x 69-251 at y 229-411), so the goldens are stable under either ordering and pin neither. Observable on the live client: the disruptor zoom draws the full-screen mask stretch pic (`cg_draw.c:349`: `CG_DrawPic( 0, 0, 640, 480, cgs.media.disruptorMask )`) immediately before the full-screen insert rotate pic (`:365`), a complete overlap. The oracle renders the insert first (immediate mode) and the mask after (batch flush), so the mask wins the overlapping pixels. Command-order append flips that: the insert lands on top of the mask. That is a visible layering change on a retail screen, not a corner case, and a one-line site note is thin for it. The user should rule on the divergence knowing the disruptor-zoom flip is its concrete face.

## Claim checks

- Chain cites all confirmed: `cl_cgame.rs:2300` (`CG_R_DRAWROTATEPIC` arm), `tr_cmds.rs:180`/`:214` (frontends), `frame_event.rs:152-178` (both variants), `frame_exec.rs:527-530` (skip arm), `scene_census.rs:279` (both traps into one row), census count 547 (`scene-trap-census.md:11`).
- Geometry and pivot claims confirmed against `tr_backend.cpp:1517-1533`. The `rotated_corners` formula matches the GL rotation matrix, and the TL/TR/BR/BL order matches both `RB_StretchPic` and `RB_RotatePic`'s emission order.
- `push_quad_st` corner block and run-merge tail confirmed at `pipeline2d.rs:187-231`. The refactor slots as described.
- The checkerboard test expectation holds: the default shader's stage 0 binds `assets.default_image` (`tr_shader.rs:4356`).
- Fixture census confirmed: 13 committed fixtures. But the packet says "all three world goldens" in the gate battery and the first pause trigger, and there are FOUR (`world_duel1`, `world_ffa2`, `world_marks_duel1`, `world_dlights_duel1`; tests at `world_golden.rs:419-599`). The "thirteen" total is right, so "three" is a stale pre-step-007 count. Fix both mentions to four.
- Minor cite drift, cosmetic: `RB_SetGL2D`'s `GL_State` is `:1282-1284` (packet says `:1284-1286`), the `RB_RotatePic2` tail is `:1600-1602` (packet says `:1597-1600`).
- Minor description drift: the cgame sites at `cg_draw.rs:2455,2584` are the radar icons and `:3182,3241` the disruptor zoom, so "vehicle HUD dials" is loose. The line numbers themselves are correct.

## Summary for the walk

Rows 1-7 stand as drafted (row 1, 3, 4 framings confirmed for the user; rows 2, 5, 6, 7 cleared). Row 8 needs its rationale corrected and a rule-19 animMap note added, default unchanged. Divergence note 1 is wrong as written and must become a parity statement. Divergence note 3 needs the disruptor-zoom flip named so the ruling is informed. The commit bundle must say four world goldens, not three.
