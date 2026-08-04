# Finished file - packet gh#35 step-001

## Status

Commit 1 (A1, the control run) is done and green. Commit 2 (A2, the seated frontend) is STOPPED on a pause trigger and is NOT committed. The branch head is commit 1. The A2 source edits stay uncommitted in the working tree so the reviewer can read them, and the fixture is untouched.

## The commit-1 control-run verdict

The control run is byte-identical to the committed fixture. `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` passed on the first attempt, which means the test reached the byte comparison and the bytes matched exactly.

Stated against the two outcomes the packet named: the draw-surf order is identical to the fixture, so the reorder seen on 2026-08-04 belongs to the `dedicated` experiment and mainline moved nothing between `bc856508` (2026-07-31) and today. The audit's residual hole in section B is now closed. Correction 2 of the audit ("the reorder is plausible, not proven") can be upgraded to proven-by-elimination: with the image stub removed and every mainline commit since the bless present, the order did not move.

The surface multiset check is subsumed by byte identity, which is the stronger statement. For the record, the fixture parses to 22 surfaces and 95472 bytes. Its per-surface SHA-256 digests, in fixture order, are `f9e0e13bcfc0dab7`, `1b854050fc3031b5`, `bbe046193bc65ece`, `d6cfdfbcdfe13ed0`, `a9922d142c8c58c8`, `52be028677018ead`, `9a7595388d0b9657`, `01a2fff512738652`, `f851d08522c5860d`, `e92bcc43a65146da`, `6bf37b59311fbbf0`, `4999aca0fe7ac408`, `088d014278993d21`, `af71d9562eee20bd`, `d1c1fa0dffa1a518`, `8441f5dce6c4022c`, `d682ad4d0b0d0467`, `3bd47ed030af0e30`, `c7e13cbc11aa302b`, `7691b35b609f3276`, `a4b5f4629d9bf99a`, `0e047ed84c4dd2c8` (first 16 hex digits each). The multiset digest over the sorted list is `f3551f6abb73953d71b6d293306a8889914e34c6a467cb11d44fc632fbef7264`. A future re-bless compares against these.

## The pause trigger that stopped commit 2

Trigger: reality contradicts the plan, and the work needs a surface the contract does not list.

Fact 3 of the packet is false. It states "Past the init call, nothing in the golden reads the `re` slot", and cites the `dedicated` experiment as empirical proof. The experiment proved nothing about the `re` slot, because a nonzero `dedicated` short-circuits every `dedicated || g2_should_register_server(host)` guard in the ghoul2 crate to the server register path. It masked two call sites, not one.

With the frontend seated for the init call only, the init succeeded. The `.glm` and the `.gla` both disk-loaded through Raven's client `RE_RegisterModel`. The abort then moved to draw time, at the same `re_from_view` line. The backtrace:

```
mp_renderer::hook_install::re_from_view            hook_install.rs:58
mp_renderer::hook_install::re_register_model_hook  hook_install.rs:80
EngineHostView::model_register_client              engine_host_view.rs:413
mp_engine_ghoul2::api_models::register_model       api_models.rs:177
mp_engine_ghoul2::misc::g2_setup_model_pointers    misc.rs:403
mp_engine_ghoul2::misc::g2_setup_model_pointers_v  misc.rs:465
mp_renderer::tr_ghoul2::r_add_ghoul_surfaces       tr_ghoul2.rs:2285
mp_renderer::tr_main::R_AddEntitySurfaces          tr_main.rs:2070
mp_renderer::tr_main::R_GenerateDrawSurfs          tr_main.rs:2311
mp_renderer::tr_main::R_RenderView                 tr_main.rs:2798
mp_renderer_gpu::frame_exec::FrameExecutor::render_world    frame_exec.rs:811
mp_renderer_gpu::frame_exec::FrameExecutor::execute_frame   frame_exec.rs:572
ghoul2_vertex_golden::golden_ghoul2_verts_stormtrooper      ghoul2_vertex_golden.rs:442
```

`G2_SetupModelPointers` re-registers the model on the entity walk of every frame, which is faithful Raven behavior. The client register path therefore needs a seated `re` slot for the whole frame render, not for the init call alone. The `re` slot the draw reads comes from the `EntityWalkHost` view the golden builds at `ghoul2_vertex_golden.rs:437`, which uses the same `boot::host_view` and the same null slot.

## Why the fix is not a mechanical extension

The two call sites that draw a Ghoul2 entity both sit inside the write scopes: `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs:437` and `crates/mp/renderer-gpu/src/bin/world_harness.rs:914`. The other three `EntityWalkHost` sites (`tests/world_golden.rs:247`, `tests/scene_golden.rs:414`, `src/ui_host/boot.rs:816`) draw no Ghoul2 entity, so they never reach `r_add_ghoul_surfaces` and they stay green. Scope is not the blocker.

The shape is. A window that stays open across `execute_frame` cannot be the swap-in-swap-out pair the contract describes, for one concrete reason: while the twins sit in the frontend, the `UiHost` fields hold placeholders, and the golden hands `sim.published`, `world_load`, `img_state` and `noise` from `UiHost` straight into `execute_frame`. A frame-length window forces the draw to read its receivers off the frontend instead. That is a different design, it changes `init_ghoul2`'s contract signature, and it needs a new pub item, for example a `with_frontend` scope helper. All three are named pause triggers, so the lane stops and does not rule.

Three options exist for the user, and this lane picks none of them:

1. A frame-length frontend window in the two ghoul2-drawing harness call sites, with the draw reading its receivers off the frontend for the length of the window.
2. The wholesale absorption of `RendererFrontend` into `UiHost` that the audit's section C priced, which the packet deliberately excluded.
3. The A1 override kept as the harness's permanent configuration, which the audit rejected as deliberately unfaithful on this one path.

## Assumptions and choices, keyed to commits

**Commit 1.** `EngineHooks::RE_RegisterModel` and `EngineHooks::R_RegisterServerModel` share one signature, `Option<fn(&mut EngineHostView, &str) -> qhandle_t>` (`engine_hooks.rs:165-168`), so the packet's preferred direct reassignment worked and no harness function was needed. The override sits directly after the two hook installs in `boot_renderer`, which is where the packet placed it.

**Commit 2, uncommitted.** `RendererFrontend::new()` is the pub constructor the packet asked for. It registers no cvar and mutates no engine state, so its placeholder twins are side-effect-free. The 12 twins swap through a private `swap_frontend_twins` helper in `boot.rs`. The seat happens by assigning `view.re` on the view `init_ghoul2` builds for itself, rather than by adding a parameter to `host_view`, which keeps `host_view`'s signature and its ten call sites untouched.

The audit's claim that the client register hook touches only twin fields is confirmed by reading, and the init call proved it by running to completion. The reachable `re_from_view` users during a ghoul2 init are `re_register_model_hook` (`qs`, `world_load`, `sim.published`, `cvars`, `img_state`, `sky_view`, `world_effects`) and `shader_hash_table_exists_hook` (`sim.published`). Both sets are twins. The frontend-only 7 fields took no mutation, so pause trigger 2 did not fire.

## Deviations

One. The packet's commit 2 is not delivered, for the pause trigger above. Its source edits are written and they compile clean with no warnings, but they are not committed, because the ghoul2 golden aborts at draw time and a red gate must not land.

## Commits

1. `45480710` - `test(gh#35 s001): the ghoul2 golden control run on the server register path`. Gates: `cargo build --workspace` green, `cargo test --workspace` green (exit 0), `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` green and byte-identical to the fixture, `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` green on both maps, the multiset check subsumed by byte identity. No `.actual.bin` was produced at any point.
2. This finished file.

## Open gaps

The uncommitted A2 edits touch three files: `crates/mp/renderer-gpu/src/ui_host/boot.rs` (the override removed, `init_ghoul2` and `swap_frontend_twins` added, the `host_view` doc and the `re` slot comment updated), `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs` (the local helper deleted, the call migrated, the module doc updated with the bless provenance, four imports dropped), and `crates/mp/renderer-gpu/src/bin/world_harness.rs` (the local helper deleted, the call migrated, two imports dropped). The reviewer reads them with `git diff`. Reverting them costs nothing and re-doing them costs one agent turn, so the lane left the choice open.

The fixture is unchanged and still valid, because the branch head runs the same server register path the fixture was blessed under.

`world_harness` has the same draw-time abort as the golden, and it has had it since `00569c66` on 2026-08-02. Neither the audit nor gh#35 records that, and the issue should.

The audit record's fact-3 claim and the issue comment that carries it both need the correction above. The session appends it when it closes gh#35.
