# 2026-08-05 - the mark-fragments landing (gh#31 step-006)

This record freezes the lane-review of gh#31 step-006, the first step of the census-remainder plan: the mark walk retargeted from its `MarkNode` stand-ins onto the real `WorldAsset` arena, the `CG_CM_MARKFRAGMENTS` and both decal arms wired live, and the new `world_marks_duel1.png` golden blessed under the user's review. The work ran as an opus code lane under `.claude/packets/31/step-006/packet.md` (commits `ce238644`, `f0a953db`, `58e8fdba`, `4fa50b24`), a conformance clerk on opus walked the whole diff and re-ran every gate, and the style commit `67e3f3a0` closed the comment findings. The branch merged to master at `e793964b`. Follow-ups land on gh#31, and the finding dispositions live in the packet's Amendments section.

## The conformance clerk's report, verbatim

I read the packet whole, read both style skills, walked every hunk in `master..gh31-step-006-marks`, and re-ran every gate. Report follows.

---

# 1. Letter violations

Surfaces created or changed that the contract does not list. Five items.

**1a. `#[allow(clippy::too_many_arguments)]` added to `R_BoxSurfaces_r`.** The contract quotes the signature without an attribute.

```
 /// Source: `oracle/codemp/renderer/tr_marks.cpp:116-178`
+#[allow(clippy::too_many_arguments)]
 pub fn R_BoxSurfaces_r(
```

**1b. `#[allow(clippy::too_many_arguments)]` added to `R_MarkFragments`.** Same.

```
 /// Source: `oracle/codemp/renderer/tr_marks.cpp:245-448`
+#[allow(clippy::too_many_arguments)]
 pub fn R_MarkFragments(
```

**1c. `build_refdef` signature changed.** The contract says the new test boots "through the file's existing machinery" and names no signature change.

```
-/// Builds the frozen scene refdef at `eye`, looking straight ahead (yaw 0,
-/// pitch 0), through the fixed golden viewport.
-fn build_refdef(eye: [f32; 3]) -> refdef_t {
+/// Builds the frozen scene refdef at `eye`, looking along `angles`, through the
+/// fixed golden viewport.
+fn build_refdef(eye: [f32; 3], angles: [f32; 3]) -> refdef_t {
```

**1d. `record_scene` signature changed.** Same.

```
-fn record_scene(host: &mut UiHost, refdef: &refdef_t) -> FrameData {
-    let mut frame_data = FrameData { events: Vec::new() };
+/// The caller appends its own scene primitives to `frame_data` first, because the render command must sit after them.
+fn record_scene(host: &mut UiHost, refdef: &refdef_t, frame_data: &mut FrameData) {
```

1c and 1d are confessed in the finished file as Deviation 2. I record them here because the contract lists neither.

**1e. The `MarkState::view_count` doc text differs from the contract's quotation.** The contract quotes `/// \`tr.viewCount\` as the decal walk reads it. \`R_MarkFragments\` bumps it once per call.` The delivered text is:

```
+    /// `tr.viewCount` as this walk reads it.
+    /// `R_MarkFragments` bumps it once per call, and `R_BoxSurfaces_r` compares each candidate surface to it.
```

Nothing else. No new `pub` item beyond the two the contract names (`MarkState::surf_view_count`, `RendererFrontend::mark_state`). No `#[repr]` change, no cvar, no `FrameEvent` variant, no engine hook, no `#[no_mangle]`/`pub extern`, no `Cargo.toml` change in the range, no dependency added (`mp_engine_core` and `mp_engine_server` are pre-existing dependencies of `mp_renderer_gpu`). No file outside the write scopes.

---

# 2. The named hunks, verbatim

The full quoted hunks are preserved in the review transcript summary below. The clerk quoted, in order: the `MarkState` struct with its new `surf_view_count` field; the whole `R_BoxSurfaces_r` retarget (arena parameters, the plane-copy pattern at both `BoxOnPlaneSideRef` sites with its two-line justification, the leaf window over `world.mark_surfaces`, the shader-resolve `.expect`, and the stamp logic on `mark.surf_view_count`); the `R_MarkFragments` retarget (the `assets.world` guard, the stamp-array resize, the `SurfaceData::Grid`/`Face` arms reading `drawVert_t` and `FaceVertex`, and the `Skip | Triangles | Flare` ignore arm); the `RE_AddDecalToScene` signature hunk with `world_root` deleted; the `RendererFrontend::mark_state` field and seat; the full `CG_CM_MARKFRAGMENTS` arm (verified against `oracle/codemp/client/cl_cgame.cpp:805-806`) and `CG_R_ADDDECALTOSCENE` arm (verified against `:903-904`); the `FxHost::AddDecalToScene` Engine arm; and the `world_golden.rs` refactor with the whole `duel1_floor_mark` fixture. The clerk found nothing wrong inside the named hunks beyond the items reported in the other sections.

---

# 3. Ledger mismatches

Behavior visible in the diff that the finished file does not mention. Six items. The four confessed deviations and the four confessed open gaps are excluded.

**3a. The two `#[allow(clippy::too_many_arguments)]` attributes.** Not named anywhere in the finished file.

**3b. The fixture calls `PerpendicularVectorMP`, where the oracle calls `PerpendicularVector`.** Oracle `oracle/codemp/cgame/cg_marks.c:145` is `PerpendicularVector( axis[1], axis[0] );`.

**3c. The fixture passes a literal `0.0` where the oracle passes `CG_ImpactMark`'s `orientation` argument.** Ruling B named the eye, mark, radius, and shader, and the finished file names the pitch, the drop, and the modulate, but never the orientation.

**3d. `register_shader` mutates the published render-asset registry through `Arc::make_mut` inside the scene step, before `record_scene` runs.** Not mentioned.

**3e. `register_shader` takes a raw `*mut RendererFrontend` out of `host.re` and then destructures `host` mutably, and hands the raw pointer to `boot::host_view`.** The doc comment claims the `boot::load_world` precedent, and that precedent does exist verbatim at `crates/mp/renderer-gpu/src/ui_host/boot.rs:185` and `:209-212`. The finished file does not mention the pattern.

**3f. The fixture prints a new stdout line on every run.** Not mentioned.

---

# 4. The inventories

## Files changed against the write scopes

`git diff master..gh31-step-006-marks --name-status`: eight paths - `finished.md`, `cl_cgame.rs`, `fx_host.rs`, `world_marks_duel1.png` (added), `world_golden.rs`, `renderer_frontend.rs`, `tr_marks.rs`, `tr_scene.rs`. All eight sit inside the packet's write scopes. Nothing outside. `oracle/`, `tr_bsp.rs`, `tr_world.rs`, and every pre-existing fixture under `tests/goldens/` are untouched. `world_marks_duel1.png` is the only fixture created, which is the one the contract allows.

## Commits against the bundle

The branch is linear off master (`merge-base` = master = `542e66ed`). One-to-one with the bundle, no split, no reorder: `ce238644` = item 1 (the arena retarget, inert), `f0a953db` = item 2 (the live arms), `58e8fdba` = item 3 (the marks golden), `4fa50b24` = item 4 (the finished file).

## Commit messages against the rules

All four subjects are headings with no terminal period. All four bodies are unwrapped paragraphs with no em dash, no semicolon, and no contraction. `git interpret-trailers --parse` run on each of the four messages returned empty output. No trailer of any kind, on any commit.

---

# 5. Repo mechanics on added lines

416 added Rust lines examined.

- **`use` declaration inside a function body:** none.
- **`todo!()` or other placeholder without both markers:** none. The two pre-existing `//TODO: Port` blocks are deleted, and the re-grep returns no hit.
- **Newly ported item with no oracle `Source:` cite:** the three added test constants (`MARK_RADIUS`, `MARK_DROP`, `MARK_SHADER`) and the two added struct fields (`surf_view_count`, `mark_state`) carry docs and no `Source:` line - all five are fixture parameters or internal carriers, not ported oracle items. Every added item that does port an oracle name carries a cite, and the clerk verified all four against the source: `cl_cgame.cpp:805-806`, `cl_cgame.cpp:903-904`, `cg_marks.c:107-108`, `cg_local.h:56`. The `cg_marks.c:110-220` range overshot the function end by nine lines (the function ends at 211); the range came from the packet.
- **New extern forward-declaration block:** none.
- **`format!` that builds a wire string:** none on added lines.

One observation outside the five listed checks: the new import in `cl_cgame.rs` broke the file's alphabetical order.

---

# 6. House-style violations on added lines

**6a.** Four added comment lines in the `tr_marks.rs` module note exceeded the 150-column limit (measured 229, 200, 180, and 169 columns).

**6b.** One new comment block in `world_golden.rs` broke at the file's old 80-column width, mid-sentence (the joined line is about 103 columns).

**6c.** One added line in `tr_scene.rs` continued an old column-wrapped sentence spread over four preceding lines wrapped at roughly 72 columns.

**6d.** Three added comment sentences exceeded the STE 25-word cap (27, 28, and 26 words).

**6e.** Five added lines start lowercase with no period - four are Raven's own preserved comments (verified verbatim at `cg_marks.c:143,151,159,175-176`, including the "persistantly" spelling), one is the face-plane comment.

**6f.** Nine added `// SAFETY:` and `// Source:` lines take a verb-less noun-phrase shape, character-identical to lines already present in the same files.

**6g.** A semicolon inside the `#[ignore]` message string, character-identical to the two pre-existing tests' strings.

**6h.** An assert message that starts lowercase and carries no period, matching the file's existing assert messages.

**Checks that found nothing:** no em dash on any added line in any file including `finished.md` and all four commit bodies, no semicolon in any added prose comment, no pet vocabulary, no contraction, no comment that narrates mechanics.

---

# 7. The gate claims, re-run

Every run was one foreground command from the worktree root, with `JKA_GOLDEN_BLESS` confirmed unset, and `git status --porcelain` empty before and after every run.

| Gate the finished file claims | Real output |
|---|---|
| `cargo build --workspace`, green, zero warnings | Exit 0, forced rebuild of the seven downstream crates, no warning line. **Matches.** |
| `cargo test --workspace` | Exit 0. 517 passed, 0 failed, 20 ignored. **Matches.** |
| world goldens, 3 passed at tolerance zero | `golden_world_duel1 ... ok`, `golden_world_ffa2 ... ok`, `golden_world_marks_duel1 ... ok`, finished in 53.43s. `CHANNEL_TOLERANCE` is `0`, and the two older PNGs do not appear in the diff. **Matches.** |
| scene suite, 7 passed | All seven ok in 3.85s. **Matches.** |
| entity golden, 1 passed | ok in 15.54s. **Matches.** |
| ghoul2 vertex golden, 1 passed | ok in 15.57s. **Matches.** |
| marker re-grep, no hit | No hit, exit 1. **Matches.** |

Extra evidence on the new golden: a `--nocapture` run printed `world_marks_duel1: 1 fragments under the eye at [1248.0, 928.0, 128.0]`, confirming the nonzero assert is live and the z-192-minus-64 arithmetic holds. The PNG is 293,032 bytes, 800x600 RGBA, 7,146 distinct values (not flat), with 23,810 alpha-modified pixels bounded by a 200x200 square centered on the viewport - consistent with the finished file's blend-alpha note. No `*.actual.png` mismatch artifact exists.

Every gate the finished file claims re-ran and produced the claimed result. I found no false gate claim.

---

# 8. The unverified list

1. Whether the blessed image shows the mark the user approved - the clerk's 200x200 alpha measurement is a measurement, not the approval.
2. Whether the user actually approved the image before commit `58e8fdba` - no repo artifact records it.
3. Whether one fragment is the oracle-correct answer for that projection - no differential harness exists for `R_MarkFragments`.
4. Whether the retargeted walk is oracle-faithful on any map other than duel1 and ffa2.
5. Whether `SceneState::last_time` equals `tr.refdef.time` at the moment cgame calls the decal trap in a live frame - argued from the write order, exercised by no test, and the census recorded zero decal calls.
6. Whether the two `.expect` calls can fire on other maps.
7. Whether `R_MarkFragments`'s return value always equals `fragment_buffer.len()` - the copy-out sizes from the buffer, the return travels separately, and nothing asserts agreement.
8. Whether the `points` slice built from `VMA(2)` with the module's count is always inside module memory - the oracle trusts the same count.
9. The three restored live effects - impact marks, saber damage glow, blob shadow - are exercised by no test.
10. Warning counts for crates upstream of the five changed files (cached, not re-emitted).
11. Clippy and rustfmt - not claimed, not run; the two `#[allow]` attributes suppress a lint that would otherwise fire.
12. The 32-bit lane and CI - not run, not claimed.
13. The lockstep referee - not run; the file-list exemption confirmed.
14. Whether `world_golden.rs`'s new raw-pointer split is sound - textually matches the `boot.rs` precedent; no Miri on a wgpu test.
15. Whether the packet's stale oracle cites were the only ones - the four added-code cites verified, the untouched files not audited.
16. `docs/decisions.md` DEC-43.1, cited in the new PORT-NOTE - not opened.

## Rulings

**2026-08-05 - the lane-review disposition.** The findings closed as: fix the five style items in one commit on the branch, record the accepted findings as a packet Amendment, then merge per the packet's disposition. The dispositions live in the Amendments section of `.claude/packets/31/step-006/packet.md` (the clippy attributes accepted as inert, the test-helper signature changes accepted as private machinery, the four fixture details accepted, deviations 1, 3, and 4 accepted as packet-gap fills, and the clerk's convention-matching flags discarded).

**2026-08-05 - the image gate.** The user reviewed the blessed `world_marks_duel1.png` through its opaque preview and approved it before commit `58e8fdba` landed. This record is the repo artifact of that approval (clerk unverified item 2).

## Follow-ups

**2026-08-05 - the style commit.** `67e3f3a0` closed the five style items: the four over-width module-note lines re-broken, the wrapped doc block joined, the `cg_marks.c:110-211` cite corrected, the continued wrapped sentence in `tr_scene.rs` re-broken under 150 columns, and the import order restored. `cargo check --workspace --all-targets` clean.

**2026-08-05 - the merge.** `gh31-step-006-marks` merged to master at `e793964b`, master green on `cargo check --workspace`. The step-007 hand-forward (dlights) is the closing comment on gh#31.
