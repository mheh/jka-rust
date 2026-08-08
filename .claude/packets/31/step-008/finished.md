# Finished gh#31 step-008 - the 2D closure

Branch `gh31-step-008-2d-closure`, cut from master at `ee6ef25e`. Three code commits plus this one. No push.

## Assumptions and choices

### Commit 1, `c459deea feat(gh#31 s008): the rotate-pic executor arms`

The `push_quad_st` refactor derives its four corners as `[[x0,y0],[x1,y0],[x1,y1],[x0,y1]]` and calls `push_quad_xy`. The run-merge tail moved into `push_quad_xy` unchanged, so the merge rule still sees every quad exactly once. Byte-identity on all thirteen older fixtures is the proof that the refactor changed no existing draw.

`rotated_corners` uses the packet's formula verbatim, in `f32`, and runs in the 640x480 virtual space before the ortho transform. I checked the result by hand against the oracle for both arms. `RE_RotatePic` at the packet's coordinates puts its far corner at virtual `y = -26.5`, so that quad clips at the top edge of the frame. That follows from the top-right pivot and the packet's own numbers, and it is not a defect.

Both rotate arms return 0 when the stage-0 image is missing or not uploaded, and neither one warns. `draw_stretch_pic` raises `Warned::NoStageImage` in the same situation, but the packet's step 4 specifies a plain `None` return here, so the arms stay silent. A rotate pic with no stage-0 image draws nothing in both trees.

Both new methods carry `#[allow(clippy::too_many_arguments)]`, matching `draw_stretch_pic`. Each takes eight arguments including `self`, which is one over the default clippy threshold.

The unknown-shader fallback pushes through `push_quad_xy` at the rotated corners rather than through `push_white`, because `push_white` takes a `Rect` and cannot express a rotated quad. The blend and the missing image are the same as `push_white` uses.

The module doc at the head of `frame_exec.rs` said the rotate-pic pair was counted and skipped. That sentence is now false, so it was rewritten. No other prose in the file changed.

### Commit 2, `9154cf58 test(gh#31 s008): the synthetic 2D golden`

The packet's commit bundle puts `golden_hud_2d` in commit 2 and `golden_hud_font` in commit 3, so commit 2 carried the file with the synthetic test alone. The three retail helpers, `boot_retail`, `register_font` and `draw_font_string`, plus the two language constants and the `tr_font` import, all landed in commit 3 with the test that uses them. Splitting them that way keeps `cargo build --workspace` warning-free at commit 2, since dead helpers would raise `dead_code`.

The synthetic host boots through `boot::boot_renderer`, the same entry `scene_golden.rs` uses. The temp tree carries `mpdefault.cfg`, `productid.txt` and one shader script, all written through the atomic-rename helper, and the directory name carries the script's hash so a stale tree is never reused.

The script defines one shader, `gfx/hud/reference`, which maps `$whiteimage` with `rgbGen vertex` and `alphaGen vertex`. The reference stretch pic draws through it. Both rotate pics register `gfx/hud/rotate`, which the script does not define and no image file backs, so they resolve to `tr.defaultShader` and bind `tr.defaultImage`.

### Commit 3, `b3b36ade test(gh#31 s008): the retail font-string golden`

The retail host boots through `boot::boot` with the `JKA_BASEPATH` override, the idiom `world_golden.rs` uses. `boot::boot` adds `ui_init_equivalent` on top of `boot_renderer`, which is the proven retail path in this crate.

The string is passed as a byte literal. It is pure ASCII, so no Latin-1 conversion helper is needed and the test takes no new import for it.

The language carriers are two file-local constants, `GOLDEN_LANGUAGE` and `GOLDEN_LANGUAGE_MODCOUNT`. `display.rs` holds equivalents but keeps them private to that module.

The `ocr_a` cite was corrected from the packet's `crates/mp/cgame/src/cg_main.rs:5267` to the oracle line the port came from, `oracle/codemp/cgame/cg_main.c:3748`.

## Deviations

One, and it is a packet wording point rather than a code change.

The packet's row 4 expects the two rotate quads to show a "checkerboard" from `tr.defaultImage`. Raven's `R_CreateDefaultImage` builds a box, not a checkerboard: a 255 border around an interior that is memset to 32 in all four channels, alpha included. The rotate quads therefore read as bright outlined frames with a dark, mostly transparent interior. The rotation stays plainly visible from the border, so the ratified defect condition, three axis-aligned quads, is not met. I reported this at the bless STOP and the user blessed the image. No code changed for it.

## Commits and gates

Every commit ran the packet's full battery. `cargo build --workspace` reported zero warnings each time, `cargo test --workspace -- --test-threads=1` reported no failures, and every golden suite ran serially as one foreground command.

1. `c459deea feat(gh#31 s008): the rotate-pic executor arms`. Thirteen fixtures byte-identical: four world goldens, seven scene goldens, `entity_duel1`, `ghoul2_verts_stormtrooper`. No committed golden moved, so the `push_quad_st` refactor is proven pure.
2. `9154cf58 test(gh#31 s008): the synthetic 2D golden`. The same thirteen byte-identical, plus `hud_golden` green. `hud_2d.png` blessed at 12,073 covered pixels and re-ran byte-identical at zero channel tolerance. User verdict 2026-08-07: "blessed".
3. `b3b36ade test(gh#31 s008): the retail font-string golden`. Fourteen fixtures byte-identical, `hud_2d` now among them, plus the `hud_golden` ignored lane green. `hud_font_ocr_a.png` blessed at 376 covered pixels and re-ran byte-identical at zero channel tolerance. User verdict 2026-08-07: "low res but blessed".

The lockstep referee was not required. No commit touches `mp_game`, the server, or any `jampded` link-set crate.

## The fix round

The lane-review vet returned seven findings and the disposition walk ratified all seven. Four of them need code, and `98a55693 fix(gh#31 s008): the vet-round corrections` lands those four in one commit.

Finding 2, the semicolons in the two added assert strings. The missing-golden message and the mismatch message are both rewritten as plain sentences. The sibling golden files stay untouched.

Finding 3, the semicolons in added prose. The module-doc sentence about the default shader and the boot comment about the registered flag are both split into two sentences. The semicolon that separates two cites on a `Source:` line stays, per the ratified disposition.

Finding 4, the column-wrapped comments. Every added comment in `hud_golden.rs` now carries one sentence per line under the 150-column limit, and the `rotate_pics` doc in `frame_exec.rs` is split onto two lines. Four sentences that would pass 150 whole are broken at a clause boundary instead.

Finding 6, the rule-19 note. Both `animMap` site notes now carry the ratified second half, "and this read draws nothing instead", written as its own sentence so the line stays under the limit.

Findings 1, 5 and 7 need no code. Finding 1 authorizes the `ccea53c9` README rider after the fact. Finding 5 exempts "seam" from the pet-vocabulary ban. Finding 7 corrects the packet's row-4 bless criterion in place and rules out a code change and a re-bless.

The full battery ran on the fix commit. `cargo build --workspace` reported zero warnings, `cargo test --workspace -- --test-threads=1` reported no failures, and all fifteen committed fixtures stayed byte-identical: four world goldens, seven scene goldens, `entity_duel1`, `ghoul2_verts_stormtrooper`, `hud_2d` and `hud_font_ocr_a`. Both `hud_golden` lanes ran green. No behavior changed, since every edit is a comment or an assert message.

## Open gaps

The font golden is low resolution, as the user noted. The mechanical cause is the render target: `GOLDEN_WIDTH` and `GOLDEN_HEIGHT` are 320 by 240, and the 640x480 virtual 2D screen maps to that whole target, so every glyph draws at half its authored size. The string's glyph band is 9 pixels tall. Raising the target to 640x480 would restore the authored size at the cost of a re-bless, and that is a later decision, not this lane's.

The command-order interleave stays as the one accepted divergence. The oracle draws a rotate pic ahead of the stretch pics still pending in `tess`, and this backend appends it in command order. The note at the executor arm names the disruptor zoom layering flip as its concrete face.

The 2D color register stays in floats. `RB_SetColor` truncates to bytes, and this backend does not. That is the pre-existing stretch-pic behavior, and this step introduced no new quantization.

Neither rotate arm evaluates a `stage2d` pass. The oracle runs no `tcMod`, no `rgbGen` and no animation frame here, so a shader that needs one to look right would mean the reading is wrong, not that the code is missing a call.

Two residuals fall outside the four ratified corrections, and both need a one-line ruling rather than a guess from this lane. First, the `ROTATE_SHADER` doc in `hud_golden.rs` still calls `tr.defaultImage` a "procedural checkerboard", which finding 7 established is wrong, and the module doc six screens above already says "bordered box". Finding 7 ruled no code change, so the word stands. Second, four added comment lines in `frame_exec.rs` sit over the 150-column limit at lines 1130, 1131, 1605 and 1632. The vet flagged the wrapping in `hud_golden.rs` only, so finding 4 does not reach them.
