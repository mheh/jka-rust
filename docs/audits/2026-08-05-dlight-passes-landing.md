# 2026-08-05 - the dlight-passes landing (gh#31 step-007)

This record freezes the lane-review of gh#31 step-007, the dlight projection passes of the census-remainder plan: both style bodies ported behind the `r_dlightStyle` dispatch, `WorldVertex` widened with a CPU-read normal under a byte-identity binding, and the new `world_dlights_duel1.png` golden blessed under the user's review. The work ran as an opus code lane under `.claude/packets/31/step-007/packet.md` (commits `628ee500`, `b3de5f6c`, `35022be2`, `6695bca2`), a user-ordered Fable spot check reviewed the diff in place of the conformance clerk, and the style commit `d0cd911b` closed the comment findings. The branch merged to master at `84180bba`. Follow-ups land on gh#31, and the finding dispositions live in the packet's Amendments section.

## The Fable spot-check report, verbatim

### 1. The two style bodies, line by line - no transcription divergence found

I read `oracle/codemp/renderer/tr_shade.cpp:523-838` and `:840-1180` against `project_dlight_texture2` and `project_dlight_texture` in the worktree's `crates/mp/renderer-gpu/src/pipeline3d.rs`. Every item on the priority list matches:

- **Operators**: the six-bit clip is `dist[0] < -radius` / `else if dist[0] > radius` in both, all three axes, `clipall = 63` and `&=`. The backface test is exact: oracle `DotProduct(normal,origin)-DotProduct(normal,posa) <= 0.0f || DotProduct(normal,normal) < 1E-8f`, port `_DotProduct(normal, origin) - _DotProduct(normal, posa) <= 0.0f32 || _DotProduct(normal, normal) < 1E-8f32`. The range reject is `fac >= radius` in both. The style-0 texcoord clip is `< 0.0` / `else if > 1.0` and the height arms are `> radius` / `< -radius` / `else` in both.
- **Modulate formulas**: style 1 `modulate = 1.0f32 - ((fac * fac) / (radius * radius))` then `fac = 0.5f32 / (radius * radius - fac * fac).sqrt()` - oracle `:643-644` verbatim, and `sqrtf` maps to `f32::sqrt` (both correctly rounded single precision). Style 0 `modulate = 2.0f32 * (radius - dist[best_index]) * scale` after `Q_fabs`, with the `radius * 0.5f32` early-out - oracle `:1038-1043` verbatim.
- **Float width**: every literal in both bodies carries an `f32` suffix; I found no bare f64 literal and no widened intermediate. `VectorNormalize` reproduces Raven's double-`sqrt`-rounded-to-float.
- **`myftol`**: the port calls the shared `myftol` (`crates/mp/renderer/src/tr_shade_calc.rs:511`, `round_ties_even`), which matches the oracle's id386 FISTP arm - the retail Win32 build's arm; the `(int)(x)` truncating fallback is the non-x86 arm only (`tr_local.h:31-35`). Then `as u8` truncates modulo 256, the same as the C int-to-byte store. This is the same helper and the same choice every other color quantization site already ratified.
- **Dominant-axis scan**: the `greatest` scan with the double-signed condition, the `VectorCompare(normal, vec3_origin)` terrain fallback to axis 2, the `dUse` clamps (`maxGroundScale` 1.4 on z, `maxScale` 1.5 on x and y, floor 0.1), and the per-arm `lightScaleTolerance` component pairs (z arm checks n0/n1 plus the zero-normal re-test, y arm n0/n2, x arm n2/n1) all transcribe exactly. The finished file's claim that the cross-vertex `scale` carry is dead is correct: every arm writes `scale` before the texcoord and modulate reads.
- **Clamp order**: `dUse` is capped high before the 0.1 floor via `if/else if`, matching the oracle's order in all three arms.
- **Triangle keeps**: both styles skip on `clip_bits[a] & clip_bits[b] & clip_bits[c] != 0`; style 1 adds the all-four-sides projected-texcoord reject in the oracle's exact four-clause form; style 0 keeps the surface's own indices as the oracle's `hitIndexes[..] = a, b, c` does.
- **`SHADER_MAX_VERTEXES - 3` break**: present, `verts.len() >= SHADER_MAX_VERTEXES - 3` after the three pushes, equal to the oracle's post-increment check (`SHADER_MAX_VERTEXES` = 1000, verified).

### 2. The shared tail and draw arms - match

`dlight_stage_bundle` reproduces the qualification predicate: first stage under `num_unfogged_passes` with zero `GLS_SRCBLEND_BITS + GLS_DSTBLEND_BITS`, bundle image present, not a lightmap, no tex mods, and the tcGen environment/fog rejects only when `style_two` - exactly the oracle's style split (`:748-750` versus `:1100-1101`). The collapse of selection and bind into one bundle return is sound because the oracle's bind re-test (`:774`, `:1116`) is the same predicate as the selection. The multitexture arm blends `GLS_SRCBLEND_ONE | GLS_DSTBLEND_ONE`; the plain arm splits on `dl.additive != 0` into one-one or `GLS_SRCBLEND_DST_COLOR | GLS_DSTBLEND_ONE` (`:811-816`); both arms set `depth_equal: true, depth_write: false`. The `MODE_DLIGHT` WGSL arm computes `diffuse.rgb * dlight.rgb * input.color.rgb`, the fixed-function `vertexColor * tex0 * tex1` chain, and the PBR shader routes it faithfully with `pbr_lit` staying 0.

### 3. The six confessed deviations - all verified, all sound

1. **Transient `dead_code` allow**: gone at final state; `MODE_DLIGHT` carries no attribute on the branch head.
2. **Ghoul2 zero normal**: correct - only world and brush surfaces receive `R_DlightSurface` masks, and the parallel `normals` slice stays for the lighting evaluators.
3. **Style-0 texcoord binding**: see the blast-radius judgment below.
4. **`dlight_stage_bundle` collapse**: behavior-equivalent, shown above.
5. **`run_golden_scene` sixth parameter**: verified; the three older fixtures pass `false` and their goldens held.
6. **`depth_bias: shader.polygon_offset`**: this is not a divergence - it is the faithful reading. The oracle's polygon-offset window opens at `tr_shade.cpp:2264` and closes at `:2361`, and the dlight dispatch (`:2320-2339`) sits inside it. The other in-window passes in the port key the same bias.

### 4. `WorldVertex` widening - clean

The five static asserts pin size 44 and offsets 0/12/20/28/32. `VERTEX_ATTRIBUTES` is untouched, both WGSL diffs add fragment-shader arms only, and every fill site is covered by construction (a missed site would fail to compile). Fill semantics match the contract: face plane normal flooded, `drawVert_t` normal for grid/triangles, MD3's computed normal, zero for poly, sky, sprite, line, ghoul2, and the dlight pass's own output verts. The dynamic-stage rebuild carries `cpu[i].normal` through, so per-frame copies stay consistent.

### 5. The gate battery, re-run by me - all green, clean tree before and after

- `cargo build --workspace`: green, no warnings.
- `cargo test --workspace`: 136 suites ok, 0 failures.
- `world_golden --ignored --test-threads=1`: 4 passed (68.8s), the new `golden_world_dlights_duel1` among them at tolerance zero.
- `entity_golden`: 1 passed. `ghoul2_vertex_golden`: 1 passed. `scene_golden` (no `--ignored`): 7 passed, `golden_scene_dlights` included.
- `git status --porcelain`: empty before and after.

I also compared the two PNGs by eye: `world_dlights_duel1.png` shows a strong warm floor pool with distance falloff and brightened wall panels against the unlit `world_duel1.png`, so the visible-light proof holds.

### 6. Quick pass - commits, scopes, style

All four commit subjects are headings, the bodies are STE prose, and `git interpret-trailers --parse` is empty on each. The diff touches exactly the packet's write scopes and nothing else; `scene_golden.rs` changed docs only; the one marker `//TODO: Port ProjectDlightTexture2 bmodel-entity pass` sits at the gate with its `// Source:` line. **One style finding**: six added comment lines exceed the 150-column limit (151, 154, 171, 184, 190, 264 columns). The longest ones transcribe the packet's own contract text verbatim - the packet seeded the `collect_dlight_items` doc line at that length - so the lane followed its orders.

### Divergences ranked by severity

1. **Float-width or operator slips - none found.** This is the class the check exists for, and it came up empty.
2. **Low - the style-1 vertex cap applies per surface, not per tess batch.** The oracle feeds surfaces to `tess` in batches of at most 1000 vertices and resets `numIndexes` per batch; the port runs the pass once over the whole surface, so a surface with more than ~332 lit triangles clips earlier than the oracle's batched runs would. This is the backend's standing architecture (no pass in this backend batches), not a slip in this lane, and no fixture exercises it. Worth a line in the session's memory, not a fix in this lane.
3. **Negligible - the style-0 texcoord deviation** (below).
4. **Style - the six over-length comment lines.**

### Judgment on the style-0 texcoord deviation

The blast radius is close to nil. Three conditions must stack: `r_dlightStyle 0` (the cvar ships at 1 and nothing in the codebase sets 0), a shader that produces a qualifying multitexture stage, and a surface whose last-iterated stage computed texcoords that differ from the base `st`. Note also what the oracle actually binds there: `tess.svars.texcoords[0]` is whatever the final stage of the preceding stage loop left in the unit-0 client array - for a two-stage lightmapped wall that is the lightmap coordinates, not the diffuse coordinates, so the oracle samples the diffuse image with stale, often wrong coordinates. The port's base-`st` choice is the same input style 1 deliberately uses (Raven's own comment at `:684` calls the svars read "wrong"). No retail world surface hits the difference in a default configuration because style 0 never runs. The confession is honest, marked at the site, and the open-gaps list names it. I would not spend a fixture on it.

### What remains unverifiable

- `project_dlight_texture` (style 0) has no image gate; only the transcription stands behind it, as the finished file admits.
- `stage_image` frame selection versus `R_BindAnimatedImage`, and the `GLS_SRCBLEND_DST_COLOR` decode in `blend.rs`, are pre-existing surfaces outside this diff; the golden exercises the non-additive blend visibly but I did not re-derive them.
- The per-batch cap semantics (finding 2) have no differential coverage.
- Live cgame submissions (weapon fire, saber glow) remain future work, as the finished file states.

### Recommendation

**Merge as-is.** The transcriptions are faithful at the float and operator level, all six confessed deviations check out (one of them, `depth_bias`, is in fact the faithful reading rather than a deviation), the full gate battery re-ran green under my own hands, and the golden shows the light. The six over-length comment lines and the per-surface cap note are worth carrying as session follow-ups, not lane blockers - the longest comment line came from the packet's own contract text, so a fix there would touch the packet template too.

## Rulings

**2026-08-05 - the image gate.** The user compared the blessed `world_dlights_duel1.png` against the unlit `world_duel1.png` in chat and approved it before commit `35022be2` landed. This record is the repo artifact of that approval.

**2026-08-05 - the lane-review disposition.** Merge with one style commit, per the reviewer's recommendation. The style-0 texcoord divergence is accepted at near-nil blast radius, the per-surface vertex cap is recorded as a standing backend architecture note, and future packets keep their sketched doc lines under the column limit. The dispositions live in the Amendments section of `.claude/packets/31/step-007/packet.md`.

## Follow-ups

**2026-08-05 - the style commit.** `d0cd911b` re-broke the six over-length comment lines at sentence and clause boundaries. `cargo check --workspace --all-targets` clean, and the post-commit column scan shows no line over 150.

**2026-08-05 - the merge.** `gh31-step-007-dlights` merged to master at `84180bba`, master green on `cargo check --workspace`. The step-008 hand-forward (the 2D closure) is the closing comment on gh#31.
