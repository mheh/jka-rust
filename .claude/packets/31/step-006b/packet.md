# Packet gh#31 step-006b - the polygon-offset depth bias

## Scope

This step fixes the decal flicker live play showed after the step-006 marks landing: blob shadows and burn marks z-fight under a moving camera. The diagnosis, verified in session: the parser reads `polygonOffset` (`crates/mp/renderer/src/tr_shader.rs:5429-5431`, `FinishShader`'s decal-sort rule at `:3818`), and the ABI `shader_t` carries the field (`tr_local/shader_s.rs:70`), but the flag dies at the `ShaderAsset` conversion (`tr_shader.rs:3438` copies every declared field, and `ShaderAsset` declares no polygon-offset field), so no GPU pipeline ever applies a depth bias. Every mark polygon draws at exactly the floor's depth, and the `LessEqual` compare then passes or fails per pixel by rasterization precision. Raven offsets every pass of a `polygonOffset` shader toward the viewer: `qglEnable(GL_POLYGON_OFFSET_FILL); qglPolygonOffset(r_offsetFactor->value, r_offsetUnits->value)` at the head of `RB_StageIteratorGeneric` (`oracle/codemp/renderer/tr_shade.cpp:2264-2267`), disabled after the fog pass (`:2361`), with `r_offsetfactor` default `"-1"` and `r_offsetunits` default `"-2"` (`oracle/codemp/renderer/tr_init.cpp:1135-1136`, both `CVAR_CHEAT`, ported at `crates/mp/renderer/src/tr_init.rs:850-851`).

The fix carries the flag across the conversion into `ShaderAsset`, adds a `depth_bias` bit to the render-pipeline cache key, and builds biased pipeline variants with wgpu's `DepthBiasState` - the native `glPolygonOffset` equivalent. All three census mark shaders carry the directive in the retail scripts (verified in `assets1.pk3`: `gfx/damage/rivetmark`, `gfx/effects/saberDamageGlow`, and `markShadow` each declare `polygonOffset`), so the fix covers the whole flickering set at once. The cache only materializes keys that draw items request, so the doubled key space costs pipelines only for shaders that carry the flag.

The step does not touch the frontend walk, the marks chain, the sort rule (`polygonOffset` already forces `SS_DECAL` at parse time, `tr_shader.rs:3818`), the 2D path (no depth buffer), or the sky path (`box_key` stays without the bit; the census gate excludes `SURF_SKY`). This step runs before step-007, and it needs no step-007 interaction: a `polygonOffset` shader sorts `SS_DECAL = 4 > SS_OPAQUE = 3` (`tr_local/shader_sort_t.rs:11-12`), so the future dlight gate can never select a biased surface.

## Rulings, taken by the user 2026-08-05

Ruling 1 is settled: the `world_marks_duel1.png` re-bless is authorized, exactly this one fixture, under the bless-STOP below. No open question remains.

**Ruling 1 - the `world_marks_duel1.png` re-bless (the expected one moved fixture).** The bias shifts the mark's depth values, so the committed mark golden is expected to change - that change IS the fix, a mark that resolves cleanly in front of its floor. The survey pins the expected re-bless list to exactly this one fixture: the two world goldens stay byte-identical because neither map's shader lump references any of the 133 retail `polygonOffset` shaders (verified by scanning the `duel1.bsp` and `ffa2.bsp` shader lumps against the extracted list), the scene suite's synthetic scripts declare no `polygonOffset`, and the entity and ghoul2 fixtures resolve their surfaces to the default shader. The re-bless runs under the step-006 bless-STOP: the lane blesses, re-runs clean, then STOPS before the commit, and the user's eyes gate the image. The expected visual change: the rivetmark decal renders solid and stable on the floor where the committed image may show partial or patchy coverage. Any OTHER fixture moving is a STOP defect, never a re-bless candidate.

## Surface contract

**`ShaderAsset` (`crates/mp/renderer/src/render_state/shader_asset.rs`) gains one field:**

```rust
/// `polygonOffset` - the shader draws every pass with the decal depth bias (`glPolygonOffset(-1, -2)` in the oracle).
pub polygon_offset: bool,
```

The conversion at `tr_shader.rs:3438` copies it from `state.polygon_offset`, per that block's own whole-struct-copy law. The literal `ShaderAsset { .. }` construction sites in the test module (`crates/mp/renderer-gpu/src/pipeline3d.rs:4660-4700`) and the dev harness (`crates/mp/renderer-gpu/src/bin/dev_harness.rs:391`) fill `false` as compile fallout.

**`PipelineKey` (`pipeline3d.rs:763`) gains one field**, `depth_bias: bool`, alongside `blend`/`depth_equal`/`depth_write`. Three item builders set it:

- `build_stage_item` (`:2503`, the static world stage item) and `build_cpu_surface_stage_item` (`:2714`, the shared CPU path: polys, MD3, Ghoul2, LOD grids) read `shader.polygon_offset` into the key. Both already receive the shader, so no signature changes.
- `build_fog_stage_item` (`:2834`) gains one parameter `depth_bias: bool`, and every call site passes `shader.polygon_offset`. The oracle keeps the offset enabled through the fog pass (`tr_shade.cpp:2264` enable, `:2361` disable after fog), so a decal's fog pass biases with its stages.
- The sky `box_key` (`:2387`) stays without the bit.

**`build_world_pipeline` (`:2934`) applies the bias.** The `bias: wgpu::DepthBiasState::default()` line becomes:

```rust
// Raven: qglPolygonOffset(r_offsetFactor->value, r_offsetUnits->value), factor -1, units -2.
// Source: oracle/codemp/renderer/tr_shade.cpp:2264-2267, oracle/codemp/renderer/tr_init.cpp:1135-1136
bias: if key.depth_bias {
    wgpu::DepthBiasState { constant: -2, slope_scale: -1.0, clamp: 0.0 }
} else {
    wgpu::DepthBiasState::default()
},
```

`slope_scale` is GL's factor and `constant` is GL's units, the standard translation. Two divergences get one comment line each at the site: the values bake into the pipeline from the retail defaults, because both cvars are `CVAR_CHEAT` and a changed cheat value does not re-key the cache; and wgpu's constant bias on `Depth32Float` steps by the float exponent where GL steps by the format's minimum resolvable difference, so the magnitudes differ while the sign and slope match, and the image golden pins the outcome. The faithful and PBR backends share this builder, so both bias identically by construction.

**The re-bless (Ruling 1).** `crates/mp/renderer-gpu/tests/goldens/world_marks_duel1.png` re-blesses per the procedure below. It is the only fixture this step may write.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate (DEC-49-class rulings come from the user only). No cvar registration (`r_offsetfactor`/`r_offsetunits` are registered), no `RenderCvarSnapshot` field, no frontend change (`tr_shader.rs` changes only at the conversion literal), no `shader_s.rs` or any `#[repr]` change, no WGSL change, no 2D or sky change, and no fixture change beyond the one named re-bless: every other committed golden and `ghoul2_verts_stormtrooper.bin` is read-only.

## Bless procedure for the re-blessed golden

1. After the bias lands, run `cargo test -p mp_renderer_gpu --test world_golden golden_world_marks_duel1 -- --ignored --test-threads=1` and confirm the mismatch writes `world_marks_duel1.actual.png`.
2. Re-bless with `JKA_GOLDEN_BLESS=1 cargo test -p mp_renderer_gpu --test world_golden golden_world_marks_duel1 -- --ignored --test-threads=1`, then re-run without the variable and confirm the byte-identical pass.
3. STOP before the commit that rewrites the PNG. The user compares the old and new images and approves. The mark must render solid and stable on the floor; an image where the mark dimmed, vanished, or bled through geometry is a defect, not a blessable golden.

## Pause triggers, named for this step

- Any golden other than `world_marks_duel1.png` moves - the two world goldens, the scene suite, the entity golden, or the ghoul2 fixture - in any byte or pixel. STOP: the survey says none of them draws a `polygonOffset` shader, so a moved one means the key change leaked onto unbiased pipelines.
- `world_marks_duel1.png` does NOT move after the bias lands. STOP: the fix changed nothing observable, so the flag is not reaching the pipeline and the diagnosis needs a fresh look.
- A fourth `PipelineKey` site or another item builder turns out to exist beyond the three named. STOP and name it.
- The bias visibly over-shifts at bless time (the mark floats or bleeds through steps). STOP: the constant's float-depth granularity note above is the suspect, and the magnitude is a ruling, not a tuning knob.
- Verification is `cargo build` / `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.
- `dedicated` stays `"0"` in every rig run.

## Commit bundle

1. **The flag crossing, inert.** The `ShaderAsset` field, the conversion copy, and the literal-site fallout. Nothing reads the field yet, so behavior is unchanged. Gates: `cargo build --workspace` with zero warnings, `cargo test --workspace`, both world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene suite green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`).
2. **The bias.** The `PipelineKey` field, the three builder wirings, and the `DepthBiasState` arm. Gates: `cargo build --workspace`, `cargo test --workspace`, the scene suite green, the entity golden byte-identical (`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`), the ghoul2 fixture byte-identical (`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`), `world_duel1` and `world_ffa2` byte-identical, and `world_marks_duel1` failing with a written actual image - the expected mismatch this commit's body records as a plain sentence.
3. **The re-bless.** The new `world_marks_duel1.png`, after the user approves per the bless procedure. Gates: the full battery, all suites green including the re-blessed golden at tolerance zero.
4. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. Gate results are written as plain sentences inside the body, so no line parses as a git trailer (step-005 lesson). All golden runs are serial with `--test-threads=1`, each as one foreground command with a long timeout. The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh31-step-006b-polygon-offset`, cut from master.

- `crates/mp/renderer/src/render_state/shader_asset.rs` - the field.
- `crates/mp/renderer/src/tr_shader.rs` - the conversion literal only.
- `crates/mp/renderer-gpu/src/pipeline3d.rs` - the key field, the three builders, the pipeline bias, the test-module literals.
- `crates/mp/renderer-gpu/src/bin/dev_harness.rs` - the literal fallout.
- `crates/mp/renderer-gpu/tests/goldens/world_marks_duel1.png` - the re-bless, under Ruling 1 only.
- Any caller `cargo check` shows broken by the new field or arity, edit-only to pass the new shape.
- `.claude/packets/31/step-006b/` for `finished.md`.

Everything else is read-only, including `oracle/`, `shader_s.rs`, `tr_init.rs`, every other fixture under `tests/goldens/`, `ghoul2_verts_stormtrooper.bin`, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

Ruling 1 is settled and the packet is ready for the lane, which spawns only on the user's explicit go. This step runs before step-007, which re-audits after this landing. After a clean lane-review: merge to master locally. No push, and no pull request.

## Amendments

**2026-08-05 - ruling 1 is taken.** The re-bless of `world_marks_duel1.png` is authorized on the recommendation, exactly this one fixture, gated on the bless-STOP comparison. The packet is ready for the lane.

**2026-08-05 - the STOP resolved: the step ships without an image observer.** The lane stopped on the "marks golden did not move" trigger and disproved the trigger's stated cause with two reverted probes. The straight-down camera cannot observe a coplanar depth shift, so the re-bless had nothing to act on and no fixture moved. The user ruled: ship on the probe evidence plus the byte-identical battery, and live play is the visual gate.

**2026-08-05 - lane-review closed, merge clean.** A Fable investigator (user-ordered) walked the diff, re-ran every gate green, and closed the direction question the lane's probes left open: a two-sided probe proved negative bias advances the decal toward the viewer on Metal (positive bias made the mark vanish, matching the golden's differing-pixel analysis), so `constant: -2, slope_scale: -1.0` carries the correct sign against the `Depth32Float`/`LessEqual`/near-zero depth setup. Findings, all cosmetic: the finished file's deviation 3 miscounts its own comment breaks, and one pre-existing semicolon sits in a `PipelineKey` doc line the lane touched one word of. No code change. The magnitude at oblique angles stays on live play, as ruled.

## Amendments

**2026-08-05 - the draft awaits the user audit.**
