# Vet report - gh#31 step-008, the 2D closure

Range `master..gh31-step-008-2d-closure`, five commits, walked in order with `git show`, every hunk. Oracle cites read at the cited lines before any commit. `finished.md` was not opened; commit f6750b72 was checked by `--stat` and message only.

Finding count: 7.

## 1. Letter violations

**Finding 1 - the README commit is outside every write scope and outside the bundle.** Commit `ccea53c9 docs: README refreshed to the house style` rewrites `README.md` (77 insertions, 103 deletions). The packet's write scopes are `frame_exec.rs`, `pipeline2d.rs`, `hud_golden.rs`, the two new PNGs, callers broken by the stat rename, and `.claude/packets/31/step-008/`. The packet closes the scope list with "Everything else is read-only". The commit bundle names four commits and none of them is a README refresh. Sample hunk:

```
-## Status (2026-08-05)
+## Status
...
+The prose follows the house style: unwrapped paragraphs, no em dashes, no semicolons, no contractions, and plain voice.
```

(the last line is from the commit body). No packet Amendment authorizes this commit. Whether the user ordered it mid-lane is on the unverified list.

No other letter violation. The `pub` surface added is exactly the contracted `push_quad_xy`. No signature drift, no `#[repr]` change, no cvar, no `FrameEvent` variant, no dependency, no trap or dispatcher arm. Noted, not counted as findings: both rotate methods carry `#[allow(clippy::too_many_arguments)]`, which the packet's signature blocks do not show but which `draw_stretch_pic` (`frame_exec.rs:939`) already carries; commit 1 also edits the `frame_exec.rs` module doc paragraph, inside the scoped file, to state the new arm coverage.

## 2. Oracle divergences

None beyond the packet's two sanctioned ones, which are both present and noted at their sites.

- Geometry: `draw_rotate_pic` pivots at `[rect.x + rect.w, rect.y]` with local corners `[-w,0],[0,0],[0,h],[-w,h]`, matching `qglTranslatef(cmd->x+cmd->w,cmd->y,0)` and the four `qglVertex2f` calls at `tr_backend.cpp:1516-1533`. `draw_rotate_pic2` pivots at `[rect.x, rect.y]` with half-extent corners, matching `:1575-1597`, and `0.5f` multiplies stay `f32` (`rect.w * 0.5`).
- Texture pairing: `uv.corners()` yields `(s1,t1),(s2,t1),(s2,t2),(s1,t2)`, the oracle's `qglTexCoord2f` order in both arms.
- Rotation: `rotated_corners` computes `x' = lx*cos - ly*sin`, `y' = lx*sin + ly*cos` in `f32`, the `qglRotatef(a,0,0,1)` matrix, applied in the 640x480 virtual space before ortho, same as the oracle's matrix stack.
- Gates: arm 1 has no `num_unfogged_passes` gate (oracle has none at `:1498-1541`); arm 2 gates on `num_unfogged_passes` before the stage read (oracle `:1560`). The empty-`stages` return 0 matches the oracle's NULL-image skip on the zero-filled stage (`tr_shader.cpp:2782-2783`), per the ratified parity note.
- Blend: arm 1 uses `blend_state_from_gls(GLS_2D_DEFAULT)` per ratified row 1, with the two-line note; arm 2 uses `blend_state_from_gls(stage.state_bits)`, matching `GL_State(shader->stages[0].stateBits)` at `:1571`.
- The `gpu_images.contains` filter has no oracle counterpart, but the contract commands it verbatim ("filtered by `gpu_images.contains`"). Observation, not a finding: a registered-but-not-uploaded image skips silently here, while `draw_stretch_pic` warns `Warned::NoStageImage` for the same condition.
- The animMap claim in the rule-19 note is true in this tree: the Rust animMap parse fills only `image_animations` and leaves `bundle[0].image` at `None` (`crates/mp/renderer/src/tr_shader.rs:4659-4660`, `texture_bundle.rs:30`), so the direct read draws nothing where the oracle binds the array base (`tr_shader.cpp:1441-1442`).

## 3. The named hunks, verbatim

**`draw_rotate_pic`** (`frame_exec.rs`, commit c459deea):

```rust
    /// `RB_RotatePic`: one quad rotated `angle` degrees about its own top-right corner, textured from stage 0's bundle image and colored by the `RE_SetColor` register.
    /// Returns the quads batched: one, or zero when stage 0 binds no image.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:1498-1541`
    #[allow(clippy::too_many_arguments)]
    fn draw_rotate_pic(
        &mut self,
        shader: ShaderHandle,
        rect: Rect,
        uv: UvRect,
        angle: f32,
        color: [f32; 4],
        assets: &RenderAssets,
        gpu_images: &GpuImages,
    ) -> u32 {
        // The pivot is the rectangle's top-right corner, and the local corners put the unrotated quad where a stretch pic would draw it.
        // Source: `oracle/codemp/renderer/tr_backend.cpp:1516-1533`
        let pivot = [rect.x + rect.w, rect.y];
        let local = [
            [-rect.w, 0.0],
            [0.0, 0.0],
            [0.0, rect.h],
            [-rect.w, rect.h],
        ];
        let xy = rotated_corners(pivot, local, angle);

        // The oracle issues no `GL_State` for this arm, so it draws in whatever state the last flush left.
        // We draw at `GLS_2D_DEFAULT`, the state `RB_SetGL2D` installs (`tr_backend.cpp:1282-1284`), because this backend has no flush point to inherit from.
        let blend = blend_state_from_gls(GLS_2D_DEFAULT);

        let Some(asset) = assets.shaders.get(shader) else {
            self.warn_once(Warned::UnknownShader);
            self.batch
                .push_quad_xy(xy, uv.corners(), color, blend, None::<ImageHandle>);
            return 1;
        };
        let Some(stage) = asset.stages.first() else {
            return 0;
        };
        let Some(image) = stage.bundle[0].image.filter(|image| gpu_images.contains(*image)) else {
            return 0;
        };

        // Rule 19: the oracle reads `&stages[0].bundle[0].image[0]`, so for an `animMap` stage it binds the animation array's base as if that pointer were an image.
        // Source: `oracle/codemp/renderer/tr_shader.cpp:1441-1442`
        self.batch
            .push_quad_xy(xy, uv.corners(), color, blend, Some(image));
        1
    }
```

The GLS_2D_DEFAULT blend and its two-line note match ratified row 1. The stage-0 read and its rule-19 note match ratified row 8, with the wording gap in Finding 6.

**`draw_rotate_pic2`** (`frame_exec.rs`, commit c459deea):

```rust
    /// `RB_RotatePic2`: the same quad rotated about `rect.x`/`rect.y`, which is its center here, drawn with stage 0's own blend bits.
    /// The pass gates on `num_unfogged_passes` before it reads stage 0, exactly as the oracle does.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:1547-1602`
    #[allow(clippy::too_many_arguments)]
    fn draw_rotate_pic2(
        &mut self,
        shader: ShaderHandle,
        rect: Rect,
        uv: UvRect,
        angle: f32,
        color: [f32; 4],
        assets: &RenderAssets,
        gpu_images: &GpuImages,
    ) -> u32 {
        // The `x`/`y` pair is the quad's center in this arm, not its top-left corner.
        // Source: `oracle/codemp/renderer/tr_backend.cpp:1575-1597`
        let pivot = [rect.x, rect.y];
        let (half_w, half_h) = (rect.w * 0.5, rect.h * 0.5);
        let local = [
            [-half_w, -half_h],
            [half_w, -half_h],
            [half_w, half_h],
            [-half_w, half_h],
        ];
        let xy = rotated_corners(pivot, local, angle);

        let Some(asset) = assets.shaders.get(shader) else {
            self.warn_once(Warned::UnknownShader);
            self.batch.push_quad_xy(
                xy,
                uv.corners(),
                color,
                blend_state_from_gls(GLS_2D_DEFAULT),
                None::<ImageHandle>,
            );
            return 1;
        };
        if asset.num_unfogged_passes == 0 {
            return 0;
        }
        let Some(stage) = asset.stages.first() else {
            return 0;
        };
        let Some(image) = stage.bundle[0].image.filter(|image| gpu_images.contains(*image)) else {
            return 0;
        };

        // Rule 19: the oracle reads `&stages[0].bundle[0].image[0]`, so for an `animMap` stage it binds the animation array's base as if that pointer were an image.
        // Source: `oracle/codemp/renderer/tr_shader.cpp:1441-1442`
        self.batch.push_quad_xy(
            xy,
            uv.corners(),
            color,
            blend_state_from_gls(stage.state_bits),
            Some(image),
        );
        1
    }
```

**The executor arms that replaced the skip arm** (`frame_exec.rs`, commit c459deea), with the divergence site note:

```rust
                // Accepted divergence: the oracle draws a rotate pic immediately, ahead of the stretch pics still pending in `tess`, and this backend appends it in command order.
                // The disruptor zoom shows the face of it: the oracle flushes the mask over the insert, and we layer the insert over the mask (`oracle/codemp/cgame/cg_draw.c:349-350,365`).
                FrameEvent::DrawRotatePic {
                    x,
                    y,
                    w,
                    h,
                    s1,
                    t1,
                    s2,
                    t2,
                    angle,
                    shader,
                } => {
                    stats.rotate_pics += self.draw_rotate_pic(
                        *shader,
                        Rect {
                            x: *x,
                            y: *y,
                            w: *w,
                            h: *h,
                        },
                        UvRect {
                            s1: *s1,
                            t1: *t1,
                            s2: *s2,
                            t2: *t2,
                        },
                        *angle,
                        color,
                        assets,
                        gpu_images,
                    );
                }
```

The `DrawRotatePic2` arm is the same shape with `draw_rotate_pic2`. `color` is the frame-local `SetColor` register (`let mut color = DEFAULT_COLOR;` at `:427`, updated at `:449`). The site note names the disruptor-zoom layering flip with the packet's cite; the cited lines were read and they are the mask `CG_DrawPic` and the insert `CG_DrawRotatePic2(320, 240, 640, 480, -level, cgs.media.disruptorInsert)`.

**`rotated_corners`** (`frame_exec.rs`, commit c459deea):

```rust
/// The four screen-space corners of a quad whose local corners `local` sit around `pivot`, rotated `angle` degrees.
/// The rotation runs in the 640x480 virtual space `qglRotatef` ran in, before the ortho transform, so the sense matches the oracle.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1516-1518`
fn rotated_corners(pivot: [f32; 2], local: [[f32; 2]; 4], angle: f32) -> [[f32; 2]; 4] {
    let (sin, cos) = angle.to_radians().sin_cos();
    local.map(|[lx, ly]| {
        [
            pivot[0] + lx * cos - ly * sin,
            pivot[1] + lx * sin + ly * cos,
        ]
    })
}
```

This is the packet's body verbatim, all `f32`.

**`push_quad_xy` and the `push_quad_st` refactor** (`pipeline2d.rs`, commit c459deea):

```rust
        let (x0, y0) = (rect.x, rect.y);
        let (x1, y1) = (rect.x + rect.w, rect.y + rect.h);
        let xy = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];

        self.push_quad_xy(xy, st, color, blend, image);
    }

    /// Appends one screen-space quad with independent per-corner positions, the shape a rotated pic needs.
    /// `xy` and `st` are both in `RB_StretchPic`'s vertex order: top-left, top-right, bottom-right, bottom-left.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:1519-1533`
    pub fn push_quad_xy(
        &mut self,
        xy: [[f32; 2]; 4],
        st: [[f32; 2]; 4],
        color: [f32; 4],
        blend: BlendState,
        image: Option<ImageHandle>,
    ) {
```

The vertex construction and the run-merge tail sit inside `push_quad_xy` unchanged (the diff split falls before the `Vertex2d` builds, so the tail carries no churn). The corner derivation writes the same four positions the old body wrote. Byte-identity across all thirteen prior fixtures was re-run and holds (section 7).

**`golden_hud_2d`** (`hud_golden.rs`, commit 9154cf58): no `#[ignore]`; GPU self-skip via `Gpu::try_new_headless` returning `None` (the `scene_golden.rs` idiom); synthetic temp tree with `mpdefault.cfg`, `productid.txt`, one `.shader` script through `write_atomic`; the three draws and the four pre-bless asserts:

```rust
    assert_eq!(
        stats.rotate_pics, 2,
        "both rotate pics must batch a quad - stats = {stats:?}",
    );
    assert!(stats.quads > 0, "no stretch-pic quad - stats = {stats:?}");
    assert!(stats.draw_calls > 0, "no draw call - stats = {stats:?}");
    let covered = coverage(&actual);
    assert!(covered > 0, "nothing drawn - stats = {stats:?}");
```

The draw parameters match the packet's three calls exactly (white stretch pic at 64,64,128,128; `RE_RotatePic` at 256,64,128,128,45.0; `RE_RotatePic2` at 160,320,128,128,30.0, with the two `SetColor` tints). `CHANNEL_TOLERANCE` is 0.

**`golden_hud_font`** (`hud_golden.rs`, commit b3b36ade), the row-3 gate:

```rust
#[test]
#[ignore = "needs retail assets and a GPU; run locally with --ignored"]
fn golden_hud_font() {
    let mut gpu = Gpu::new_headless(GOLDEN_WIDTH, GOLDEN_HEIGHT);

    let mut host = boot_retail();
    let font = register_font(&mut host, "ocr_a");
    assert!(font != 0, "RE_RegisterFont(\"ocr_a\") returned zero");
```

`#[ignore]` plus the hard assert per ratified row 3, `Gpu::new_headless` (no self-skip), `JKA_BASEPATH` override in `boot_retail`, the string `b"jka-rust ^1font ^7golden"` at `(32, 64)`, scale `1.0`, max width `-1`, white, at `FROZEN_TIME_MS`.

## 4. The inventories

Files changed in the range against the write scopes:

| File | Scope |
|---|---|
| `crates/mp/renderer-gpu/src/frame_exec.rs` | in scope |
| `crates/mp/renderer-gpu/src/pipeline2d.rs` | in scope |
| `crates/mp/renderer-gpu/tests/hud_golden.rs` | in scope, new |
| `crates/mp/renderer-gpu/tests/goldens/hud_2d.png` | in scope, new |
| `crates/mp/renderer-gpu/tests/goldens/hud_font_ocr_a.png` | in scope, new |
| `.claude/packets/31/step-008/finished.md` | in scope |
| `README.md` | **out of scope - Finding 1** |

Commits against the bundle: c459deea = bundle 1, 9154cf58 = bundle 2, b3b36ade = bundle 3, f6750b72 = bundle 4, all with the exact planned subjects. `ccea53c9` is the unplanned fifth commit (Finding 1). No split, no reorder, no widened or bundled planned commit.

Commit messages: all five carry a heading subject and an STE-shaped prose body. `git log --format=%B` over the range matched no `Co-Authored-By`, no generated-with footer, no trailer of any kind. Gate results are plain sentences inside the bodies. The fixture counts in the bodies are arithmetically right (thirteen prior fixtures in commit 1, fourteen in commit 3).

## 5. Repo mechanics on added lines

- Function-body `use` declarations: none. All imports sit at the file top of `hud_golden.rs`; the destructuring `let UiHost { .. } = host;` blocks are patterns, not imports.
- `todo!()` or placeholder without `//TODO: Port` + `// Source:`: none (zero `todo!` on added lines).
- Newly ported items without an oracle `Source:` cite: none. `push_quad_xy`, `rotated_corners`, both rotate methods, both tests, and the site comments all carry cites, and every cite was checked against the oracle text.
- Extern forward-declaration blocks: none.
- `format!` building a wire string: none. The `format!` calls in `hud_golden.rs` build temp-file names and a directory hash name, and the one in the pre-existing `tr_shader.rs` context is unchanged.

## 6. House-style violations on added lines

Both skills were read by path before this section.

**Finding 2 - semicolons in error messages.** STE bans the semicolon, and error messages are a strict surface. Two added assert/panic strings carry one:

```rust
        "golden missing at {}; run once with JKA_GOLDEN_BLESS=1 to write it",
```

```rust
            "{stem} golden mismatch: {} pixels differ, max channel delta {}; wrote actual image to {}",
```

Both are verbatim copies of the `world_golden.rs:387,410` / `scene_golden.rs:480,500` idiom. The `#[ignore = "needs retail assets and a GPU; run locally with --ignored"]` string also carries one, but the packet dictates that string verbatim, so it is not counted.

**Finding 3 - semicolons in comment prose.** Three added sites:

```rust
//! two rotate pics resolve to the default shader, whose bordered box makes the
//! rotation visible; a flat square would not.
```

```rust
    // The ui boot path is what normally sets this; every draw trap drops its
    // submission while it is false.
```

(the second copies `scene_golden.rs:249` nearly verbatim), and the multi-cite separator:

```rust
/// Source: `oracle/codemp/renderer/tr_font.cpp:1430-1614`;
/// `oracle/codemp/cgame/cg_main.c:3748` (`ocr_a` is the cgame small font)
```

**Finding 4 - column-wrapped comment lines in `hud_golden.rs`.** The module doc and several `///` docs break mid-sentence at roughly 80 columns, under the 150 limit, which the house line-break gate names as the machine tell. Example:

```rust
//! [`golden_hud_2d`] is synthetic and asset-free. It boots against a temp game
//! tree this file writes, the same recipe `scene_golden.rs` uses, and draws
//! three quads: one axis-aligned stretch pic as the reference, one
```

The wrap width mirrors the sibling test files, but the standing rule is that wrap width is never inferred from the surrounding file. By contrast the added `frame_exec.rs` comments are one sentence per line at full width. Related, with provenance: the `rotate_pics` field doc holds two sentences on one line, but that exact line is the packet's own contract text quoted verbatim, so the defect originates in the packet.

**Finding 5 - pet vocabulary "seam" on added lines.** Two sites:

```rust
//! ([`FROZEN_TIME_MS`], the fixed-dt seam DEC-58.1 names), so two runs submit
```

```rust
/// The frozen clock in milliseconds - the fixed-dt seam on the com clock
```

Both copy `scene_golden.rs:12,64`. The house-style skill lists "seam" in the banned pet vocabulary; the word is also entrenched project terminology, so this needs a session ruling, not a silent fix.

No em dash on any added line in `crates/`. No banned-voice pattern, no `(???)`, no casual deflation on added lines.

## 7. The gate battery, re-run

Every invocation is the packet's, run on the branch head, real output:

| Gate | Invocation | Result |
|---|---|---|
| Build | `cargo build --workspace` | `Finished dev profile in 6.97s`, 0 warnings |
| Workspace tests | `cargo test --workspace -- --test-threads=1` | 518 passed, 0 failed (includes `golden_hud_2d`) |
| World goldens | `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` | 4 passed (`dlights_duel1`, `duel1`, `ffa2`, `marks_duel1`), 71.36s |
| Scene goldens | `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1` | 7 passed, 4.01s |
| Entity golden | `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1` | 1 passed, 15.82s |
| Ghoul2 vertex golden | `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` | 1 passed, 16.41s |
| Hud golden, synthetic | `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` | `golden_hud_2d` passed, `golden_hud_font` ignored, 0.82s |
| Hud golden, retail | `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1` | `golden_hud_font` passed, 5.87s |

Every golden comparison runs at `CHANNEL_TOLERANCE` 0 (world/scene/entity suites use their own committed tolerances), so a pass is byte-identity; the byte-identical claims in the commit bodies were not trusted and are re-confirmed by these runs. `grep -rn skipped_rotate_pics crates/` and `grep -rn "Warned::RotatePic" crates/` both return nothing, so the counter rename left no stale reader.

I also inspected both blessed PNGs against row 4's defect conditions. `hud_2d.png` shows one axis-aligned white reference quad, one 45-degree diamond and one roughly 30-degree tilted quad at the positions the pivot math predicts, so neither rotate pic is axis-aligned. `hud_font_ocr_a.png` shows legible glyphs reading `jka-rust font golden` with the `^1` run in red.

**Finding 7 - row 4's bless criterion misnames the default image.** The packet requires "two visibly rotated checkerboard quads", and the packet body says the rotate pics "carry the procedural checkerboard `tr.defaultImage`". The blessed image shows bordered boxes, not checkerboards, and the lane's own docs and commit message say "bordered box" throughout. The rotation-visibility intent of the criterion is met; the checkerboard wording is a packet-text defect the bless surfaced, and the session should confirm the bordered box is the true `tr.defaultImage` face rather than a wrong texture.

## Finding 6 - the rule-19 note drops half the packet's required content

Ratified row 8 requires the note to say the oracle "binds the animation array's base as if it were an image, **and this read draws nothing instead**". The landed note (quoted in section 3) carries the first half only; the draw-nothing half lives in the adjacent `return 0` code but not in the note's words. The claim itself is verified true (section 2, animMap parse).

## 8. The unverified list

- `finished.md` - forbidden to this vet; its content, the deviation list, and its gate-result claims are unverified.
- The two row-4 STOPs - whether the user viewed each PNG before its commit is process history outside the repo; the commit timestamps (21:01 and 21:07, six minutes apart) neither prove nor disprove it.
- The README commit's authorization - no packet Amendment names it; whether the user ordered it mid-lane is unknown here.
- Pixel-level oracle parity of the two new goldens - no oracle GL harness renders these scenes, so the images are verified geometrically (pivot math, angles, colors) and visually, not against an oracle render.
- Cross-adapter stability of the two new PNGs at tolerance 0 - only this machine's adapter ran.
- The font golden's glyph layout against `oracle/codemp/renderer/tr_font.cpp:1430-1614` beyond the visual check - the layout path is pre-existing code this step did not change.
- The live-client faces (radar icons, disruptor zoom, rocket-lock wedge) - no live client run was performed.

---

# Fix-round walk - 2026-08-07, commits 98a55693 / e9abd6ca / 22059f28 / 20382c8d

The amended packet was re-read first. The 2026-08-07 vet-disposition amendment closes all seven first-walk findings: `ccea53c9` is authorized after the fact as a session-caused rider (finding 1), "seam" is exempt as the repo's domain term (finding 5), and row 4, row 7 and the surface contract now carry the bordered-box wording (finding 7). Findings 2, 3, 4 and 6 route to the fix round. Both code commits were walked every hunk with `git show`; the two record commits were checked by `--stat` and message only, and `finished.md` stays unopened.

Fix-round finding count: 2.

## Commit 98a55693 - the vet-round corrections

Every changed line is a `//!`, `///` or `//` comment line or one of the two message string literals. No code line changed.

- Finding 2 resolved. The two strings are now sentence-form: `"the golden is missing at {}. Run once with JKA_GOLDEN_BLESS=1 to write it."` and `"the {stem} golden does not match. {} pixels differ, and the largest channel delta is {}. The actual image is at {}."`. The sibling golden files are untouched, per the disposition.
- Finding 3 resolved. The module-doc sentence splits into `The two rotate pics resolve to the default shader, whose bordered box makes the rotation visible.` / `A flat square would not.`, and the boot comment splits into `// The ui boot path is what normally sets this.` / `// Every draw trap drops its submission while it is false.`. The multi-cite `Source:` semicolon stays, per the disposition.
- Finding 4 resolved. Every added comment in `hud_golden.rs` now carries one sentence per line, and the `rotate_pics` doc splits onto two lines. The `awk 'length > 150'` scan over comment lines in both files returns zero.
- Finding 6 resolved in both arms: `// Rule 19: the oracle reads \`&stages[0].bundle[0].image[0]\`, so an \`animMap\` stage binds the animation array's base as if it were an image.` / `// This read draws nothing instead.` Two prose lines plus the `Source:` cite, inside the ratified cap.

**Fix-round finding 1 - British spelling on a touched line.** The reflowed backend-caveat doc keeps `Rasterisation`:

```rust
//! **Backend caveat.** Rasterisation is the GPU's.
```

STE requires American spelling (`Rasterization`), and the house rule lints every comment a commit touches. The word predates the fix round (it landed in commit 9154cf58 and my first walk missed it), so this is a carried defect the touch made lintable, not a regression.

**Fix-round finding 2 - lowercase sentence starts in the rewritten messages.** Both new strings open lowercase: `"the golden is missing at {}. Run once..."` and `"the {stem} golden does not match. {} pixels differ..."`. The banned-voice list names lowercase sentence starts, and each message mixes a lowercase first sentence with capitalized later ones. Counter-pressure, stated for the ruling: Rust panic-message convention and the untouched sibling files are lowercase, so a fix here diverges from the crate idiom the disposition preserved.

## Commit e9abd6ca - fix-round record

`--stat`: `.claude/packets/31/step-008/finished.md` only, 18 insertions. In scope. Heading subject, STE body, no trailer. Not opened.

## Commit 22059f28 - the residual wording and column breaks

Every changed line is a comment line. No code line changed.

- The `ROTATE_SHADER` doc now reads `Stage 0 of that shader binds \`tr.defaultImage\`, the white-bordered box \`R_CreateDefaultImage\` builds.` I read `oracle/codemp/renderer/tr_image.cpp:2731-2758` to check the claim: `Com_Memset(data, 32, ...)` fills the interior at value 32 and the four border runs write 255, so the wording is oracle-true and matches the amended packet body, row 4 and row 7.
- The four over-150 comment lines in `frame_exec.rs` (the two interleave-note lines, the `draw_rotate_pic` doc first line, the blend-note second line) each break at a clause boundary with the wording unchanged. The joined originals all pass 150, so the breaks are legal under the line-break gate, and the over-150 scan now returns zero in both files.

No violation found in this commit.

## Commit 20382c8d - residual record

`--stat`: `finished.md` only, 3 insertions 1 deletion. In scope. Heading subject, STE body, no trailer. Not opened.

## Inventories and mechanics, fix round

Files touched across the four commits: `frame_exec.rs`, `hud_golden.rs`, `finished.md` - all in scope, nothing else. No new `pub` item, signature, dependency, cvar, variant or arm. No em dash, no fn-body `use`, no `todo!()`, no extern block, no `format!` wire string on any added line. No non-EOL semicolon remains on added lines outside the ratified `Source:` separator and the packet-dictated `#[ignore]` string. Commit-message trailer scan over all four: no match.

## The gate, re-run on 22059f28

| Gate | Result |
|---|---|
| `cargo build --workspace` | `Finished dev profile in 2.95s`, 0 warnings |
| `cargo test --workspace -- --test-threads=1` | 518 passed, 0 failed (includes the seven scene goldens and `golden_hud_2d`, byte-identical at their committed tolerances) |
| `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1` (extra, beyond the named gate) | `golden_hud_font` passed, 5.81s, byte-identical |

Unverified this round: the world, entity and ghoul2 golden suites were not re-run (the coordinator named the two-gate battery, and no code line changed in either fix commit), and `finished.md` remains unread. The tree is restored to `master`.

## Addendum walk - 2026-08-07, commits 08b818d4 / 405c50a5

Commit `08b818d4 fix(gh#31 s008): the carried-spelling note` was walked with `git show`, every hunk. The diff is exactly one added comment line in `hud_golden.rs` and nothing else:

```rust
// The British spelling "Rasterisation" below is a carried defect, kept by the user ruling of 2026-08-07.
```

The line is one sentence, 101 characters, under the 150-column limit, with no semicolon, no em dash, no contraction, a capitalized start and a period. It records the ruling with its date. It is a `//` line inside the `//!` module-doc block, so it stays a source-only note and does not render into rustdoc, which matches the ruling's shape. The two assert messages are untouched, per the lowercase-idiom ruling. The commit message carries a heading subject, an STE body, and no trailer (scanned).

Commit `405c50a5 process(gh#31 s008): carried-spelling record` was checked by `--stat` and message only: `.claude/packets/31/step-008/finished.md`, 2 insertions, in scope, heading subject, STE body, no trailer. Not opened.

The gate, re-run at the branch head (405c50a5): `cargo build --workspace` finished in 7.05s with 0 warnings, and `cargo test --workspace -- --test-threads=1` passed 518 tests with every one of the 137 test-result lines reporting 0 failed. The tree is restored to `master`.

Addendum finding count: 0.
