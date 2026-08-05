# Finished gh#31 step-006b - the polygon-offset depth bias

Branch `gh31-step-006b-polygon-offset`, cut from master after `git merge master --no-gpg-sign`.

## Assumptions and choices, keyed to their commits

**Commit 1 (the flag crossing, inert).**

- `polygon_offset` sits after `cull_type` in `ShaderAsset`. Raven's `shader_t` puts `polygonOffset` right after `cullType`, so the owned form keeps that adjacency.
- The field carries a `Source:` cite to both ends of the flag: the declaration at `oracle/codemp/renderer/tr_local.h:495` and the backend read at `oracle/codemp/renderer/tr_shade.cpp:2264-2267`.
- The conversion copies `state.polygon_offset` under the block's own whole-struct-copy law. The parser writes the scratch value at `tr_shader.rs:5431`, and no later pass rewrites it.

**Commit 2 (the bias).**

- `build_fog_stage_item` takes `depth_bias` as a plain `bool` parameter rather than the whole shader. The function reads nothing else from the shader, and all five call sites already hold one.
- The sky `box_key` sets `depth_bias: false` explicitly, with one comment line. The sky never carries `polygonOffset`, and an explicit `false` keeps the sky pass out of the doubled key space.
- The bias values bake into the pipeline from the retail defaults. Both `r_offsetfactor` and `r_offsetunits` are `CVAR_CHEAT`, and a changed cheat value does not re-key the cache, so a live cheat change would not rebuild the pipeline. One comment line at the site records this.
- wgpu's constant bias on `Depth32Float` steps by the float exponent, where GL steps by the format's minimum resolvable difference. The sign and the slope match the oracle, the magnitudes differ, and a second comment line at the site records it.
- The faithful and the PBR backends share `build_world_pipeline`, so both bias identically by construction.

**Commit 2, the two probes (added, run, and reverted before the commit).**

- Probe one added an `eprintln!` at the head of `build_world_pipeline` that printed `key.depth_bias`. Under the marks fixture it printed `depth_bias=true` for exactly one of the three pipelines, which proves the flag crosses the parser, `ShaderAsset`, the item builder, and the cache key.
- Probe two replaced `constant: -2` with `constant: 200000`. The marks golden then failed with 39810 pixels different and a max channel delta of 255, which proves the `DepthBiasState` is live on the Metal backend and not silently dropped.
- Both probes were reverted by an exact inverse edit before the commit. The `world_marks_duel1.actual.png` that probe two wrote was deleted. The committed diff is exactly the packet's surface contract, and the full gate battery was re-run on the reverted state.

## Pause triggers hit

**One trigger fired: `world_marks_duel1.png` did not move after the bias landed.**

The trigger names its own suspected cause, that the flag is not reaching the pipeline. The two probes above disprove that cause. The flag reaches the GPU and the bias is live.

The real cause is the fixture's camera. `golden_world_marks_duel1` sets view angles `[90.0, 0.0, 0.0]`, so the camera looks straight down at a floor-coplanar decal. The mark sorts `SS_DECAL`, draws after the floor with a `LessEqual` compare and no depth write, and at this camera it already passes the compare at every covered pixel. The committed image shows the rivetmark solid, with no z-fighting to fix. A shift further toward the viewer keeps every one of those pixels passing, so the color image cannot change. The flicker this step targets happens at oblique angles under a moving camera, where two coplanar depth values disagree by one unit in the last place in either direction, and this fixture holds neither condition.

I confirmed the shader really declares the directive. `gfx/damage/rivetmark` in `shaders/marks.shader` inside `assets1.pk3` carries `polygonOffset` on its own line, so the fixture does exercise a biased shader.

**The user's resolution, taken 2026-08-05.** The step ships without an image-golden observer. No re-bless, because nothing moved to bless. No new oblique fixture. The evidence for correctness is the two probes plus the byte-identical battery across every other golden, and live play after the merge is the visual gate. The user compares blob shadows and burn marks in motion against the flicker they reported.

No other trigger fired. No golden other than the named one could have moved, and none did. No fourth `PipelineKey` site exists beyond the three named builders and the sky `box_key`. The bias never over-shifted, because it never shifted an observable pixel.

## Deviations

1. **Commit 2 landed with one packet gate unmet.** The bundle listed "`world_marks_duel1` failing with a written actual image" as a gate for commit 2. That gate did not fire, for the reason above. The commit landed anyway, and its body records the miss and both probe results as plain sentences. The work is complete and every other gate is green, so committing preserved a reviewable artifact rather than leaving the lane's output uncommitted.
2. **The compile fallout the packet predicted does not exist.** The packet named the `ShaderAsset` literals in the `pipeline3d.rs` test module at `:4660-4700` as sites that must fill `false`. All four build from `..Default::default()`, so the new field needs no change there. Only `dev_harness.rs:391` is a full literal, and it took the `false`. No test file was touched in this lane.
3. **The bias comment runs to six lines, not the two the contract sketched.** Two of the sentences exceed the 150-column comment limit when written whole, so each breaks once at a clause boundary. The content is the two divergences the contract named, and nothing more.

## Commits and gate results

1. `2c7bde82` **render: the polygonOffset flag crosses into ShaderAsset**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 3 passed, all byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.

2. `9a757d85` **render: the decal depth bias reaches the pipeline**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 3 passed, all byte-identical, including `world_marks_duel1` against the packet's expectation.
   - `cargo fmt -p mp_renderer_gpu -- --check`: the diff count for `pipeline3d.rs` is 8, unchanged from the pre-existing baseline on master, so this lane added no format noise.

Every golden run was one foreground command with `--test-threads=1`, and `dedicated` stayed `"0"` in every rig run. The lockstep referee was not run: no commit touches `mp_game`, the server, or any `jampded` link-set crate. No fixture byte moved anywhere in this lane, and `world_marks_duel1.png` is the same file the step-006 lane blessed.

## Open gaps

- No committed image observes the bias at the oracle constants. The marks golden gates the mark's projection, its poly draw, and its color, but a straight-down camera cannot see a depth shift on a coplanar decal. The bias is therefore proven by the two reverted probes and by construction, not by a standing regression gate. A later step that wants a standing gate needs a fixture with an oblique view of a decal on a surface the camera also sees edge-on.
- The live check is pending. The user compares blob shadows and burn marks in motion after the merge, against the flicker they reported. A remaining flicker at the retail constants would point at the constant-bias granularity note in the code, which the packet calls a ruling and not a tuning knob.
- The bias is baked from the retail cvar defaults. `r_offsetfactor` and `r_offsetunits` are registered and readable, but a live change to either does not re-key the pipeline cache and so has no effect. Both are `CVAR_CHEAT`, so no retail play path reaches them.
