# Packet gh#36 step-001 - r_swapInterval wires to the present mode

## Scope

This step executes the gh#36 ruling of 2026-08-05: `r_swapInterval` wires to the wgpu present mode, and the sim-paced-by-render bounded channel stays as the faithful analogue of Raven's serial loop. The client's surface today hardcodes `PresentMode::Fifo` (`crates/mp/renderer-gpu/src/gpu.rs:103-109` via `get_default_config`, which picks Metal's first listed mode), so vsync is always on and the sim thread locks to the display rate. After this step, the retail default `r_swapInterval 0` selects `PresentMode::Immediate` when the surface supports it, the sim clock returns to `com_maxfps`, and a console flip of the cvar takes effect within one frame.

The Fable validation report (2026-08-05) grounds every mechanic here. Two findings bind the design. First, the MP oracle registers this cvar and never applies it (`oracle/codemp/renderer/tr_init.cpp:1068`, zero consumers in `codemp/`; retail left vsync to the Windows driver). The apply semantics port from the SP Mac glimp: a per-frame modified check in `GLimp_EndFrame` (`oracle/code/mac/mac_glimp.c:749-752`) with a forced first apply at init (`:712`). The site carries a two-line divergence note recording that. Second, the three windowed harness binaries drive the executor with `RenderCvarSnapshot::default()`, so the latch must live only in the client app's render thread - an executor-level or `Gpu`-default latch would flip `ui_harness`, `dev_harness`, and `world_harness` into uncapped spin loops.

The step does not touch the frame-sink channel discipline, `pump.rs` (`PRESENT_INTERVAL` stays at 2ms per the validation), `frame_package.rs`, `tr_init.rs` (the registration at `:781` already matches the oracle: `"0"`, `CVAR_ARCHIVE`), any harness binary, or any test scene.

## Surface contract

**`RenderCvarSnapshot` (`crates/mp/renderer/src/render_state/render_cvar_snapshot.rs`) gains one field**, read in `from_cvars` from the registered handle (`RendererCvars::r_swapInterval`, `renderer_cvars.rs:249-250`), `Default` value `0`:

```rust
/// `r_swapInterval` - default `"0"`. Zero asks for an unsynchronized present, nonzero for vsync.
pub swap_interval: i32,
```

**`Gpu` (`crates/mp/renderer-gpu/src/gpu.rs`) captures the surface capabilities and gains the latch method.** `Gpu::new` stores `supports_immediate: bool` from `surface.get_capabilities(&adapter).present_modes` before the adapter drops - the only reachable query point, and mandatory because wgpu raises a validation error on an unsupported mode (`wgpu-core` `UnsupportedPresentMode`), so the `Fifo` fallback must be our own check. The constructed default stays `Fifo` for every caller.

```rust
/// Latches `r_swapInterval` onto the surface: zero maps to `Immediate` when the surface offers it, else `Fifo`, and nonzero maps to `Fifo`.
/// A change reconfigures the surface with the same call the resize arm makes; a match is free, so the per-frame call costs nothing.
/// The headless arm is a no-op.
pub fn set_swap_interval(&mut self, interval: i32)
```

The method compares the mapped mode against `self.config.present_mode` and reconfigures only on change, before any frame acquire, the same discipline `Gpu::resize` (`gpu.rs:249-263`) follows. `Mailbox` never appears in the mapping: Metal never offers it, and hal treats it as unreachable. The headless placeholder at `gpu.rs:159` stays untouched.

**The render thread latches per frame (`crates/mp/client-app/src/render_thread.rs`).** In the `Present` arm, after the package take and before `begin_frame`, the thread calls `gpu.set_swap_interval(...)` with the held package's `cvars.swap_interval`. This is the snapshot-crossing discipline every renderer cvar already uses (DEC-37 A13.1: `RE_EndFrame` resolves the live table into the snapshot at `tr_cmds.rs:387`, the render side reads the frozen copy), and the per-frame compare is the faithful analogue of the SP Mac modified check. Boot frames present under the constructed `Fifo`, and the first package latches the cvar's mode, mirroring `mac_glimp.c:712`.

**The stale-skip guard, same arm.** When the mode is `Immediate`, `held` is `Some`, and `packages.try_recv()` missed, the arm skips the acquire and draw instead of replaying the held package. This restores Raven's serial-loop shape - one present per sim frame, present rate governed by `com_maxfps` - and removes the ~6x stale-replay churn the validation quantified. Under `Fifo` the redraw-on-miss behavior stays, because it is what keeps the window painted, and the `held = None` boot presents stay in both modes.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate. No change to `frame_sink.rs`, `pump.rs`, `main.rs`, `sim.rs`, `tr_cmds.rs`, `tr_init.rs`, `frame_package.rs`, any harness binary, any test, or any fixture. No `RenderCommand` variant. No executor change.

## Pause triggers, named for this step

- Any golden moves. STOP: all four suites are headless with no surface, so a moved image means the change leaked outside the present path.
- The capability query cannot reach the adapter, or the latch cannot live in the render thread alone. STOP: the placement constraint is binding, and a relocation is a ruling.
- The stale-skip guard needs state the `Present` arm does not already hold. STOP and name it.
- Verification is `cargo build` / `cargo check` plus the golden suites, never rust-analyzer, which is stale in this workspace.

## Commit bundle

1. **The latch surface, inert.** The snapshot field, the `Gpu` capability capture, and `set_swap_interval` with no caller. Behavior is unchanged: nothing calls the method. Gates: `cargo build --workspace` with zero warnings, `cargo test --workspace`, all four world goldens byte-identical (`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`), the scene suite green (`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`, no `--ignored`), the entity golden and the ghoul2 fixture byte-identical (each with `--ignored --test-threads=1`).
2. **The render-thread latch and the stale-skip guard.** The per-frame `set_swap_interval` call and the `Immediate` skip-on-miss, with the divergence note at the latch site. Gates: the full battery of commit 1.
3. **The finished file**, per the packet skill: assumptions keyed to commits, deviations or the word "none", the commit list with gate results, and open gaps. The live latch check (a console flip of `r_swapInterval` taking effect within one frame, and `cg_drawfps` rising above the display rate at 0) is the user's post-merge gate and lands in open gaps.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind. Gate results are written as plain sentences inside the body, so no line parses as a git trailer. All golden runs are serial with `--test-threads=1`, each as one foreground command with a long timeout. The lockstep referee is not required: no commit touches `mp_game`, the server, or any `jampded` link-set crate.

## Write scopes

Branch `gh36-step-001-swap-interval`, cut from master.

- `crates/mp/renderer/src/render_state/render_cvar_snapshot.rs` - the field.
- `crates/mp/renderer-gpu/src/gpu.rs` - the capability capture and the method.
- `crates/mp/client-app/src/render_thread.rs` - the latch call and the stale-skip guard.
- Any caller `cargo check` shows broken by the new shape, edit-only to pass it.
- `.claude/packets/36/step-001/` for `finished.md`.

Everything else is read-only, including `oracle/`, every fixture, every harness binary, and `~/Developer/jka/` beyond read-only pk3 reads.

## Disposition

The ruling and the validation are both taken, and the packet is ready for the lane on the user's go. After a clean lane-review: merge to master locally, then rebuild and restage the client so the user feels the latch in the live rig. No push, and no pull request.

## Amendments

**2026-08-05 - drafted from the Fable validation report.** The report's four corrections are folded: the mandatory capability capture in `Gpu::new`, the MP-oracle-never-applies-it divergence note, the stale-skip guard under `Immediate`, and the render-thread-only placement constraint. The user's ruling and validation order stand as the go context.

**2026-08-05 - lane-review closed, full-read.** The session read the whole diff (125 added lines, four files) against the contract and smoke-ran the gates on the worktree. One deviation, accepted: `Gpu` gained the two-line `present_mode()` readout, because the stale-skip guard must key on the actual configured mode to keep redraw-on-miss alive on a `Fifo`-fallback surface - keying on the requested interval would have broken the binding constraint. Everything else is the contract verbatim, the divergence note carries all three oracle cites, and the harnesses are untouched. The live latch check stays the user's post-merge gate.
