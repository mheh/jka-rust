# Finished gh#36 step-001 - r_swapInterval wires to the present mode

Branch `gh36-step-001-swap-interval`, cut from master after `git merge master --no-gpg-sign`.

## Assumptions and choices, keyed to their commits

**Commit 1 (the latch surface, inert).**

- `swap_interval` sits at the end of `RenderCvarSnapshot`, after `dlight_style`, and not inside the W2-F1 world-walk block. It is a present-path value, so it does not belong to that group.
- `from_cvars` reads `common.cvar(cvars.r_swapInterval).integer`, and `Default` carries `0`. Both match the registration at `tr_init.rs:781`, which already matches the oracle.
- `Gpu::new` reads `surface.get_capabilities(&adapter).present_modes` before `get_default_config`, and stores one `bool` rather than the whole capability list. The latch asks only one question, so one flag answers it.
- `new_headless` stores `supports_immediate: false`. The headless arm never presents, and `set_swap_interval` returns early on it anyway.
- `set_swap_interval` returns early on the headless arm, computes the mode, returns early on a match, then reconfigures. This is the discipline `Gpu::resize` follows on its own windowed arm.
- The constructed default is untouched. `get_default_config` still picks the surface's first listed mode, which is `Fifo` on Metal, so every caller boots exactly as before.

**Commit 2 (the render-thread latch and the stale-skip guard).**

- The latch call reads `held.as_ref()`, so a present with no package held latches nothing and the boot frames keep the constructed mode.
- The `try_recv` outcome is recorded in a local `took_package` flag. The guard needs to tell a miss from a take, and the arm did not already carry that.
- The guard is `!took_package && held.is_some() && gpu.present_mode() == wgpu::PresentMode::Immediate`, and a hit runs `continue` on the command loop. The skip therefore drops the acquire, the draw and the present together.
- The latch runs before the guard. On a miss the interval cannot have changed, so the order only keeps the mode current and costs nothing.
- The divergence note sits above the latch call and cites three oracle lines: the MP registration that never applies, the SP Mac per-frame `modified` check, and the SP forced first apply.

## Deviations

1. **`Gpu` gains a second public method, `present_mode`.** The surface contract names one method, `set_swap_interval`, whose signature returns nothing. The stale-skip guard has to know whether the effective mode is `Immediate`, and that depends on `supports_immediate`, which is private. `present_mode` is a two-line readout beside `surface_format` and `surface_size`, over state the `Present` arm already owns through `gpu`. The alternative readings both break a binding constraint: keying the guard on `swap_interval == 0` alone would skip on a surface that fell back to `Fifo`, and changing `set_swap_interval` to return the mode would change the contracted signature. This is named here rather than treated as a stop, because the guard needs no new state, only a read of the state the latch just wrote.

## Pause triggers hit

None. No golden moved a byte at either commit. The capability query reached the adapter inside `Gpu::new` as the contract expected, and the latch lives in the render thread alone. The guard needed no channel, no sim-thread value and no new package field, only the local take/miss flag and the readout named in deviation 1.

## Commits and gate results

1. `522063d7` **feat(gh#36 s001): the swap-interval latch surface, inert**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 4 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.

2. `a4c37a0f` **feat(gh#36 s001): the render thread latches the swap interval**
   - `cargo build --workspace`: green, zero warnings.
   - `cargo test --workspace`: green.
   - `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`: 4 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`: 7 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.
   - `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`: 1 passed, byte-identical.

3. This file.

`git status --porcelain` listed only the edited sources after every golden run, so no fixture moved. Every golden run was one foreground command with `--test-threads=1`. The lockstep referee was not run: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Open gaps

- The live latch check is the user's post-merge gate. A console flip of `r_swapInterval` must take effect within one frame, and `cg_drawfps` must rise above the display rate at `0`. No headless suite can see either, because all four golden suites run with no surface.
- The stale-skip guard is unproven outside a live window. Its whole effect is on present cadence, which no golden observes.
- `supports_immediate` is read once at `Gpu::new`. A surface whose capabilities change under it, for example on a display change, keeps the boot answer until the client restarts.
- The mapping collapses every nonzero interval to `Fifo`. Raven's AGL call passed the integer straight through, so an interval of 2 halved the rate there. wgpu offers no such mode, and the ruling asks only for on and off.
