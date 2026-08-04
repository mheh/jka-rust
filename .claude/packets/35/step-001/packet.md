# Packet gh#35 step-001 - the ghoul2 golden rig fix: A1 control run, then A2 seated frontend

## Scope

This step closes gh#35, the `ghoul2_vertex_golden --ignored` abort on the NULL `re` slot. It executes the two fixes the audit named (`docs/audits/2026-08-04-ghoul2-golden-null-re-slot.md`, section A), in order, on one branch. The user ruled both in chat on 2026-08-04 ("do A1 and A2"), which is the explicit go for this lane.

- **A1, the control run, commit 1.** A harness hook override routes `RE_RegisterModel` to the server registration path, which reproduces the configuration the fixture was blessed under (`bc856508`, 2026-07-31). Its purpose is evidence: it answers the audit's open control question by comparing today's surface order against the committed fixture, and it proves the fixture's surface content still matches. The override is temporary and commit 2 removes it.
- **A2, the fix, commit 2.** Ghoul2 init runs with a real `RendererFrontend` seated in the view's `re` slot for the duration of the init call, built by swapping the 12 twin fields out of `UiHost` and back. The golden then runs Raven's client register path for the first time, and the fixture is re-blessed under it.

The step does not absorb `RendererFrontend` into `UiHost` wholesale. That remains a future option, and the audit records why the structural case for it survives (the 90-plus other `re_from_view` call sites stay unservable). The step does not touch `crates/mp/renderer/`, does not change `UiHost`'s fields, and does not change any engine crate.

Ground truth: the audit record `docs/audits/2026-08-04-ghoul2-golden-null-re-slot.md` and the diagnosis comments on gh#35. Key facts the design rests on:

1. The abort path ends at `re_from_view` (`crates/mp/renderer/src/hook_install.rs:58`), reached from `g2api_init_ghoul2_model` through the `RE_RegisterModel` hook. The slot is NULL by construction in `boot::host_view` (`crates/mp/renderer-gpu/src/ui_host/boot.rs:430`).
2. The client register hook touches only fields that are exact `UiHost` twins: `qs`, `world_load`, `sim.published`, `cvars`, `img_state`, `sky_view`, `world_effects` (`hook_install.rs:84-95`, audit section A2). No mutation lands in any of the 7 frontend-only fields, so a window that defaults them discards nothing.
3. Past the init call, nothing in the golden reads the `re` slot. The `dedicated` experiment proved this empirically: with registration unblocked, the test ran to the byte comparison.
4. The 72-bone `_humanoid` remap does not fire for the stormtrooper (`numBones = 53`, read from the retail `assets1.pk3` header), so the client and server load paths produce the same vertex content. Only the draw-surf order may move, because real shader indices untie the 22 sort keys.
5. The `dedicated` cvar must stay `"0"`. Seeding it nonzero stubs every image through `tr_image.rs:2499` and fails both world goldens on the full frame. This was tried on 2026-08-04 and reverted.

## Surface contract

**Commit 1, in `boot_renderer` (`crates/mp/renderer-gpu/src/ui_host/boot.rs`), after the two hook installs at `:130-131`:** reassign `engine.common.hooks.RE_RegisterModel` to the server-path registration. Prefer the direct reassignment `hooks.RE_RegisterModel = hooks.R_RegisterServerModel;` if the two hook signatures match; otherwise a private harness fn with the same body as `r_register_server_model_hook`. A comment states this is the gh#35 control configuration and that commit 2 removes it.

**Commit 2, new `pub fn` in `crates/mp/renderer-gpu/src/ui_host/boot.rs`:**

```rust
/// Inits one Ghoul2 model through the real client register path.
/// The client path needs the view's `re` slot seated, so this seats a real `RendererFrontend` for the
/// duration of the call: the 12 twin fields swap out of `UiHost`, the 7 frontend-only fields default,
/// and the twins swap back before return.
pub fn init_ghoul2(host: &mut UiHost, name: &str) -> Option<(Ghoul2System, Ghoul2Handle, qhandle_t)>
```

The body absorbs the two identical local `init_ghoul2` helpers (`tests/ghoul2_vertex_golden.rs:127-...` and `bin/world_harness.rs:1128-1140`), which both then call this one. Private helpers inside `boot.rs` for the swap are fine. The frontend value is built from existing `mp_renderer` pub constructors and the swapped-in twins only; the construction of placeholder twin values must be side-effect-free (no cvar registration, no engine mutation). Commit 2 also removes the commit-1 override, updates `host_view`'s doc (the "no path this harness runs reads them" sentence gains the ghoul2-init exception), and updates the golden's module doc (client path, bless provenance).

**The fixture** `crates/mp/renderer-gpu/tests/goldens/ghoul2_verts_stormtrooper.bin` is re-blessed in commit 2 under the client path.

Anything not on this list is out of scope, and the agent must not add it. No new third-party crate, no `UiHost` field change, no edit in `crates/mp/renderer/` or any engine crate, no new pub item in any crate but `mp_renderer_gpu`.

## Pause triggers, named for this step

- The commit-1 control run shows surface CONTENT that differs from the fixture (the multiset of per-surface byte blobs, order ignored). That is a real vertex regression, not an ordering artifact. STOP.
- The client register path touches a frontend field outside the 12 twins during the window. STOP; that breaks the discard argument.
- Seating the frontend needs a new pub item in `mp_renderer`, or any edit outside the write scopes. STOP.
- The commit-2 re-blessed fixture's surface multiset differs from the old fixture's. Fact 4 says it must not. STOP.
- Either world golden moves by one pixel at any commit. STOP.

## Commit bundle

1. **A1, the control run.** The hook override lands with its comment. Run the ghoul2 golden serially. Record the verdict in `finished.md` either way: order identical to the fixture pins the 2026-08-04 experiment reorder on the experiment; order moved reveals mainline drift since 2026-07-31. In both cases verify the surface multiset matches the fixture (python heredoc analysis is fine) and delete any `.actual.bin` before committing. Gates: `cargo build --workspace`, `cargo test --workspace`, world goldens byte-identical, the multiset check.
2. **A2, the seated frontend.** The override goes, `boot::init_ghoul2` lands, both callers migrate to it and their local copies are deleted, the docs update, and the fixture re-blesses (`JKA_GOLDEN_BLESS=1`, then a confirming clean run). Verify the new fixture's surface multiset equals the old fixture's (`git show` the old bytes). Gates: `cargo build --workspace`, `cargo test --workspace`, the ghoul2 golden green against the new fixture, world goldens byte-identical, the multiset check.
3. **The finished file.** `finished.md` in this folder, per the packet skill.

Golden invocations, exactly: `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` and `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`. The serial flag is mandatory; without it the boots crash in the pk3 inflate path. The default `BootConfig` basepath is valid on this machine.

Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind.

## Write scopes

Branch `gh35-step-001-golden-rig`, cut from master.

- `crates/mp/renderer-gpu/src/ui_host/boot.rs`
- `crates/mp/renderer-gpu/tests/ghoul2_vertex_golden.rs`
- `crates/mp/renderer-gpu/tests/goldens/ghoul2_verts_stormtrooper.bin`
- `crates/mp/renderer-gpu/src/bin/world_harness.rs`
- `.claude/packets/35/step-001/` for `finished.md`

Everything else is read-only, including `oracle/`, all of `crates/mp/renderer/`, and every engine crate.

## Disposition

Hold on the branch. Lane-review runs against this packet with a conformance clerk (user instruction 2026-08-04), and a clean review merges to master locally. No push, and no pull request. The session appends the control-run verdict to the audit record and closes gh#35 after the merge.

## Amendments

**2026-08-04 - ratified at draft.** The user ruled "do A1 and A2" in chat after reading the audit, which is the explicit go. No separate packet audit round was held; the surface comes verbatim from the audited record.
