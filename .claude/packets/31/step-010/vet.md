# Vet - gh#31 step-010, the renderfx closure

Diff range as delivered. The named base branch `gh31-step-009-fx-minirefents` no longer exists: it merged into `wf/31-renderer-census` as pull request #48, and its tip commit is `61a5062b`. Every walk below uses `61a5062b..gh31-step-010-renderfx` (head `9f7e2ca4`), which `git merge-base` confirms is the same range.

The oracle was read first, at every line the packet cites: `oracle/codemp/cgame/tr_types.h:14-58`, `oracle/code/renderer/tr_types.h:30-55`, `oracle/codemp/renderer/tr_backend.cpp:590-679`, `:770-879`, `:1000-1219`, `oracle/codemp/renderer/tr_shade.cpp:1530-1589`, `:2140-2214`, `oracle/codemp/renderer/tr_shade_calc.cpp:1540-1671`, `oracle/codemp/renderer/tr_mesh.cpp:385-409`, `oracle/code/renderer/tr_mesh.cpp:410-424`, `oracle/codemp/renderer/tr_ghoul2.cpp:2580-2629`, `:3515-3532`, `oracle/code/renderer/tr_ghoul2.cpp:2790-2810`, `oracle/codemp/renderer/tr_init.cpp:1135-1142`, `oracle/codemp/renderer/tr_local.h:1334-1340`, `oracle/codemp/game/q_math.c:525-540`, `oracle/codemp/renderer/tr_main.cpp:15-30`.

18 findings. `finished.md` was not opened.

---

## 1. Letter violations

### F1 - `Pipeline3d::draw_items`, an unlisted new item (commit `69971bf1`)

The contract for `crates/mp/renderer-gpu/src/pipeline3d.rs` enumerates its new items exhaustively: one private field, one private type `ScreenImage`, two `StageDrawItem` fields, one `draw` parameter, and "Four new private free functions". Everything else is described as "Behavior changes inside existing bodies". Commit 2 lands a fifth new item, and it is a method, not one of the four free functions:

```rust
    /// Draws the stage items `order` names, in that order, into an open pass.
    /// The depth window tracks per pass, so a run that never leaves the normal window sets no viewport and keeps the pass default.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2136-2158` (the per-stage draw)
    #[allow(clippy::too_many_arguments)]
    fn draw_items(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        items: &[StageDrawItem],
        order: &[usize],
        bind_groups: &[wgpu::BindGroup],
        item_group: &[usize],
        screen_group: Option<&wgpu::BindGroup>,
        geometry: &WorldGeometry,
        mode: BackendMode,
        viewport: (f32, f32, f32, f32),
        stats: &mut WorldStats,
    ) {
```

Violated clause, packet "Surface contract", `pipeline3d.rs`: "Four new private free functions:" followed by the four signatures, then "Behavior changes inside existing bodies". The body is unchanged by the move, and row 7's per-surface replay needs two callers, so the extraction is mechanically forced. It is still an unlisted surface.

### F2 - `collect_stage_items` changes its signature (commit `69971bf1`)

```rust
-    ) -> Vec<StageDrawItem> {
+    ) -> (Vec<StageDrawItem>, Vec<Range<usize>>) {
```

Violated clause, packet "Surface contract", `pipeline3d.rs`: "`collect_stage_items` fills the two new `StageDrawItem` fields." That is the only clause the packet writes for this function. The return type change, and the new `use core::ops::Range;` it forces, are not in the contract.

### F3 - `screen_image` is not `EntityFx::resolve`'s `distortion` bit (commits `69971bf1`, `78461ecf`)

Contract clause: "`screen_image` is `EntityFx::resolve`'s `distortion` bit." Three of the five construction sites do not do that.

Fog pass, commit 2, `pipeline3d.rs`:

```rust
+            post_render_ent: None,
+            // The fog pass binds the fog image, so a fogged distortion surface still takes its own diffuse here.
+            screen_image: false,
```

Two-texture collapse, commit 4, `pipeline3d.rs`:

```rust
-                screen_image: fx.distortion,
+                // A multitextured stage diverts to `DrawMultitextured`, which has no distortion arm, so it keeps its own diffuse.
+                // Source: oracle/codemp/renderer/tr_shade.cpp:2147-2150
+                screen_image: false,
```

Both are oracle-correct against `oracle/codemp/renderer/tr_shade.cpp:2147-2150` (`DrawMultitextured` has no distortion arm) and `:1182-1209` (the fog pass binds `tr.fogImage`), so the deviation is toward the oracle and away from the packet's own text. The third site is the non-dynamic single-texture arm at `pipeline3d.rs:3125`, which still reads `screen_image: fx.distortion` but is now unreachable when that bit is set, because commit 4 added `|| fx.distortion` to the `dynamic` predicate. It is dead as written.

### F4 - an unlisted divergence: PBR mode never binds the capture (commit `78461ecf`)

```rust
            let screen_group = match (wants_screen && last_post_ent == entity_num, mode) {
                (true, BackendMode::Faithful) => self.screen_image.as_ref().map(|image| {
                    gpu_images.view_bind_group(gpu, &self.texture_layout, &image.view, None)
                }),
                _ => None,
            };
```

Violated clause, packet "Divergence notes, each ≤2 lines at its site". That section enumerates three notes, and the 2026-08-30 amendment struck the third, leaving two. A distortion stage under `BackendMode::Pbr` binding its own diffuse is a third behavior the list does not carry. The site note is present and truthful, but the divergence itself was never ratified.

### F5 - a fifth `TODO: Port` marker (commit `92641c97`)

```rust
+        //TODO: Port RenderSurfaces's _G2_GORE overlay pushes
+        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2623-2715
```

Violated clauses: row 8, "**Ratified: four marker sites.**"; and the write scope, "`crates/mp/renderer/src/tr_ghoul2.rs` - three marker conversions at `:840-842`, `:909-911`, and `:2459-2462`, comment text only." This is a fourth marker in `tr_ghoul2.rs` and a fifth overall. The commit body justifies it ("Splitting the ghoul2 note also gave the `_G2_GORE` overlay chain its own marker. That deferral shared the old note and would otherwise have lost its greppable subject"), and the justification holds against `docs/porting-rules.md`'s discernibility test. It is still outside the ratified set.

### F6 - three lane commits write `packet.md` (commits `a09b8e38`, `c7da3493`, `035334af`)

```
 .claude/packets/31/step-010/packet.md | 11 +++++++++++
```

Violated clause, packet "Write scopes": "`.claude/packets/31/step-010/` for `finished.md`." The folder is granted for one file. Three commits amend `packet.md` instead. Each records a coordinator ratification in its own body, and each appends below the Amendments heading without touching the body above it.

---

## 2. Oracle divergences

### F7 - a failed capture falls back to the stage's own diffuse (commit `78461ecf`)

Port:

```rust
            // A surface whose capture did not run keeps its cached group and binds its own diffuse, which is the oracle's answer to a false projection.
```

```rust
            // A distortion stage samples the captured frame, and it falls back to its own diffuse where no capture stands behind it.
            let texture_group = match (item.screen_image, screen_group) {
                (true, Some(group)) => group,
                _ => &bind_groups[item_group[draw_index]],
            };
```

Oracle, `oracle/codemp/renderer/tr_shade.cpp:2163-2169`:

```c
			if ( (tess.shader == tr.distortionShader) || 
				 (backEnd.currentEntity && (backEnd.currentEntity->e.renderfx & RF_DISTORTION)) )
			{ //special distortion effect -rww
				//tr.screenImage should have been set for this specific entity before we got in here.
				GL_Bind( tr.screenImage );
				GL_Cull(CT_TWO_SIDED);
			}
```

The bind is unconditional. On a false `R_WorldCoordToScreenCoord` the oracle's capture at `oracle/codemp/renderer/tr_backend.cpp:1176-1208` is skipped, but the arm still binds `tr.screenImage`, which then holds whatever the last successful capture left, or the 8 by 8 white placeholder `R_CreateBuiltinImages` allocated (`oracle/codemp/renderer/tr_image.cpp:2776`). It never binds `pStage->bundle[0]`. The packet's clause "which is the oracle's behavior when `R_WorldCoordToScreenCoord` returns false" (Surface contract, `pipeline3d.rs`) is not supported by those lines. The port's answer is defensible as a rule-19 pick, but it is a divergence and it carries no divergence note.

### F8 - the struck mirror note survives in the shipped comment (commit `a110a866`, not cleaned by `a09b8e38`)

`crates/mp/renderer-gpu/src/pipeline3d.rs:4680-4681`:

```rust
    // The `vid_width - x` term mirrors the square about the vertical centreline. That is Raven's own behavior and it ports as written.
    // The `vid_height - y` term converts a top-down y to the framebuffer's bottom-up origin, so the row flips back here.
```

The packet's 2026-08-30 amendment rules the opposite: "Divergence note 3, 'The horizontal mirror is preserved', is struck. There is no mirror, so there is nothing to note at the site." The lane's own investigation, folded into the same amendment and independently checkable against `oracle/codemp/game/q_math.c:530-536` (`AnglesToAxis` fills `axis[1]` from the negated right vector, Raven's own comment "angle vectors returns 'right' instead of 'y axis'") and `oracle/codemp/renderer/tr_main.cpp:17-27` (`s_flipMatrix` maps our y to GL `-x`), establishes that `vidWidth - x` converts a right-edge column back to a left-edge column. The comment now asserts a mirror that the packet rules does not exist, in the one place a future reader will look. `git show a09b8e38 --stat` shows that commit touched `packet.md` only. The code itself is correct; the comment is not.

### F9 - the colour buffer starts at zero where the oracle's is stale (commit `9b47e5c9` makes it observable)

`crates/mp/renderer-gpu/src/pipeline3d.rs:4363`:

```rust
        let mut evaluated = vec![[0u8; 4]; count];
```

Oracle, `oracle/codemp/renderer/tr_shade_calc.cpp:1570-1574`:

```c
			if ( dis < threshold * threshold )
			{
				// completely disintegrated
				colors[i*4+3] = 0x00;
			}
```

`RF_DISINTEGRATE1`'s first band writes alpha only and leaves red, green and blue at whatever `tess.svars.colors` held. The port hands the function an all-zero buffer. The line is not new in this range, but commit `9b47e5c9` is what first lets the branch run at all, so this is the first commit at which the difference is reachable. The affected vertices carry alpha `0x00` and `RF_DISINTEGRATE1` forces the alpha test (`oracle/codemp/renderer/tr_shade.cpp:2043-2047`, ported at `pipeline3d.rs:4413-4417`), which discards them, so the difference is very likely unobservable. Not verified against a fixture.

### F10 - unguarded `i32` arithmetic after a saturating cast (commit `a110a866`)

```rust
fn world_coord_to_screen_coord(
    world_coord: vec3_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(i32, i32)> {
    let (x, y) = world_coord_to_screen_coord_float(world_coord, refdef, vid_width, vid_height)?;
    Some((x as i32, y as i32))
}
```

```rust
    let mut c_x = vid_width - x - (rad / 2);
    let mut c_y = vid_height - y - (rad / 2);
    if c_x + rad > vid_width {
```

Oracle, `oracle/codemp/renderer/tr_backend.cpp:642-643` and `:1179-1182`:

```c
	*x = (int)xF;
	*y = (int)yF;
```
```c
					cX = glConfig.vidWidth-x-(rad/2);
					cY = glConfig.vidHeight-y-(rad/2);

					if (cX+rad > glConfig.vidWidth)
```

For in-range values the two agree exactly. Rust's `f32 as i32` saturates to `i32::MIN`/`i32::MAX` where C's `(int)` on an out-of-range float is undefined. `vid_width - i32::MIN` and `c_x + rad` at `c_x` near `i32::MAX` then overflow, which panics in a debug build. The projection early-return only guards `transformed[2] < 0.01`, so a point at that depth and roughly 100,000 units off the view axis reaches it. No site note covers this, and the packet's two surviving divergence notes cover the radius, not the projection.

### Verified and clean

The float widths and the evaluation order in `world_coord_to_screen_coord_float` match the oracle exactly. `xcenter / transformed[2]` is an int-to-float division in float, `90.0 / fov_x` is a double division, and the product lands back in a float, which is what the port writes as `((xcenter as f32 / transformed[2]) as f64 * (90.0 / refdef.fov_x as f64)) as f32`. `transformed[2] < 0.01` is a double compare in both. The dot-product order right, up, forward matches `:618-620`. No macro argument is hoisted. No random draw appears in any body this step transcribes, which agrees with the packet's own statement.

The `MAX_POST_RENDERS` cap counts surfaces, not stages, and a deferred surface that produced no items still consumes a slot, matching the oracle's enqueue-before-draw at `:778-788`. The LIFO drain, the `lastPostEnt` run compare and its retry, the per-entity capture gate, and the capture-then-draw interleave all read as the oracle writes them.

---

## 3. The named hunks

### `lighting_ref_entity` (`crates/mp/renderer-gpu/src/pipeline3d.rs:4261-4280`)

```rust
/// Builds the small [`RefEntity`] the diffuse-lighting evaluators read from a
/// `trRefEntity_t`. `R_SetupEntityLighting` folded the entity light into
/// `lightDir`/`ambientLight`/`directedLight`, and the evaluators read only
/// those plus `shaderRGBA`, so the rest stays at its default.
fn lighting_ref_entity(ent: &trRefEntity_t) -> RefEntity {
    RefEntity {
        light_dir: ent.lightDir,
        ambient_light: ent.ambientLight,
        directed_light: ent.directedLight,
        shader_rgba: ent.e.shaderRGBA,
        // The disintegrate arms branch on `renderfx` and measure against `oldorigin` and `endTime`.
        // Without these three the burn tests read zero, so both arms return the colour buffer untouched.
        // Source: oracle/codemp/renderer/tr_shade_calc.cpp:1545-1671
        renderfx: ent.e.renderfx,
        old_origin: ent.e.oldorigin,
        end_time: ent.e.endTime,
        ..Default::default()
    }
}
```

The three fields are correct against `oracle/codemp/renderer/tr_shade_calc.cpp:1553-1557` (`ent->endTime`, `ent->renderfx`) and `:1566` (`e.oldorigin`), and `refEntity_t::endTime` is `float` in both trees (`oracle/codemp/cgame/tr_types.h:237`, `crates/mp/qshared/src/common/mp/cgame/ref_entity_t.rs:178`). See F16 for the doc comment. `crates/mp/renderer/src/tr_main.rs:1673` (`ref_entity_from_tr`) already carried all three, so this builder was the one that fell behind.

### The per-surface partition and the LIFO drain (`crates/mp/renderer-gpu/src/pipeline3d.rs:1391-1399`, `:1459-1461`)

```rust
        // The sorted pass draws everything the post-render enqueue left behind.
        // A deferred surface reaches its own pass below, because the oracle continues past it without opening a draw surf.
        // Source: oracle/codemp/renderer/tr_backend.cpp:832-835
        let mut deferred = vec![false; items.len()];
        for range in &post_surfaces {
            for flag in &mut deferred[range.clone()] {
                *flag = true;
            }
        }
        let main_order: Vec<usize> = (0..items.len()).filter(|index| !deferred[*index]).collect();
```

```rust
        for range in post_surfaces.iter().rev() {
            let order: Vec<usize> = range.clone().collect();
```

The enqueue that fills `post_surfaces`, in `collect_stage_items`:

```rust
            // `RB_RenderDrawSurfList` takes a non-world entity's surface out of the sorted draw and replays it last.
            // The test is a three-way or over the entity's renderfx, and it stops once the queue holds `MAX_POST_RENDERS` surfaces.
            // Every stage this surface produced moves together, so the mark lands on the whole run rather than one item.
            // Source: oracle/codemp/renderer/tr_backend.cpp:778-835
            if post_renders >= MAX_POST_RENDERS {
                continue;
            }
            let (entity_num, _shader, _fog, _dlighted) =
                R_DecomposeSort(surf.sort, &assets.sorted_shaders);
            if entity_num == TR_WORLDENT {
                continue;
            }
            let Some(ent) = entities.get(entity_num as usize) else {
                continue;
            };
            if ent.e.renderfx & (RF_DISTORTION | RF_FORCEPOST | RF_FORCE_ENT_ALPHA) == 0 {
                continue;
            }
            post_renders += 1;
            for item in &mut items[first_item..] {
                item.post_render_ent = Some(entity_num);
            }
            if first_item < items.len() {
                post_surfaces.push(first_item..items.len());
            }
```

Faithful to `oracle/codemp/renderer/tr_backend.cpp:778-788` and `:1008-1011`. Stage order inside a surface is preserved and only the surface order reverses, which is row 7's ratified shape. Nothing found.

### `capture_screen_image` and its keep-alive vector (`crates/mp/renderer-gpu/src/pipeline3d.rs:4686-4756`, `:1450-1451`)

```rust
/// Encodes one distortion entity's capture into `screen_image`, rebuilding the texture when the side length changed.
/// The oracle re-specifies its texture on every copy, so a rebuild here is the faithful shape rather than a cache miss.
/// A rebuild moves the replaced texture into `keep_alive`, which the caller holds until the submit so an already-encoded pass keeps its binding.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1144-1209`
fn capture_screen_image(
    gpu: &Gpu,
    encoder: &mut wgpu::CommandEncoder,
    target_texture: &wgpu::Texture,
    screen_image: &mut Option<ScreenImage>,
    keep_alive: &mut Vec<ScreenImage>,
    rect: (u32, u32, u32),
) {
    let (x, y, side) = rect;
    let stale = screen_image
        .as_ref()
        .map(|image| image.side != side)
        .unwrap_or(true);
    if stale {
        let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("mp_renderer_gpu screen image"),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: target_texture.format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        if let Some(old) = screen_image.replace(ScreenImage {
            side,
            texture,
            view,
        }) {
            keep_alive.push(old);
        }
    }

    let image = screen_image
        .as_ref()
        .expect("the screen image is built above when the side length changed");
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: target_texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &image.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
    );
}
```

```rust
        // A rebuilt screen image replaces the texture an already-encoded pass binds, so the replaced one lives here until the submit.
        let mut keep_alive: Vec<ScreenImage> = Vec::new();
```

`keep_alive` is written and never read, and it drops at the end of `draw`, after `gpu.queue().submit(...)`. That is its stated purpose. The rebuild-on-side-change shape matches `qglCopyTexImage2D`'s re-specification at `oracle/codemp/renderer/tr_backend.cpp:1204`. Row 3's ratified usage flags (`TEXTURE_BINDING | COPY_DST`) and the target's own format are both present. Nothing found.

### The `RF_DISTORTION` stage arm and `GpuImages::view_bind_group` (`crates/mp/renderer-gpu/src/pipeline3d.rs:1485-1497`, `:1603-1607`; `crates/mp/renderer-gpu/src/gpu_images.rs:387-421`)

```rust
            // `RB_IterateStagesGeneric` binds `tr.screenImage` for every stage of a distortion entity.
            // The view changes with each capture, so this group is built here rather than in the shared cache walk above.
            // A surface whose capture did not run keeps its cached group and binds its own diffuse, which is the oracle's answer to a false projection.
            // The PBR backend takes a four-texture layout this two-texture group does not fit, so it keeps its own diffuse as well.
            // Source: oracle/codemp/renderer/tr_shade.cpp:2163-2169
            let wants_screen = items[range.clone()].iter().any(|item| item.screen_image);
            let screen_group = match (wants_screen && last_post_ent == entity_num, mode) {
                (true, BackendMode::Faithful) => self.screen_image.as_ref().map(|image| {
                    gpu_images.view_bind_group(gpu, &self.texture_layout, &image.view, None)
                }),
                _ => None,
            };
```

```rust
            // A distortion stage samples the captured frame, and it falls back to its own diffuse where no capture stands behind it.
            let texture_group = match (item.screen_image, screen_group) {
                (true, Some(group)) => group,
                _ => &bind_groups[item_group[draw_index]],
            };
```

```rust
    /// Builds the world texture bind group with an explicit diffuse view, the shape the distortion stage needs.
    /// The screen image is render-thread scratch with no `ImageHandle`, so it binds by view and takes the clamping sampler.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2167` (`GL_Bind( tr.screenImage )`)
    pub fn view_bind_group(
        &self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        diffuse: &TextureView,
        lightmap: Option<ImageHandle>,
    ) -> BindGroup {
```

The signature matches the contract exactly. The `last_post_ent == entity_num` gate correctly covers both the surface that captured and the later surfaces of the same entity that skipped it, which is the oracle's shape. See F4 and F7 for the two divergences in this hunk.

### The `v` flip in `build_dynamic_block` (`crates/mp/renderer-gpu/src/pipeline3d.rs:4336-4345`)

```rust
    // The two worlds store a copied rect in opposite row order: `glCopyTexImage2D` stores it bottom-up and `copy_texture_to_texture` top-down.
    // A screen-image stage flips `v` here, which cancels that and leaves the sampling where the oracle puts it.
    // Only the single-texture branch binds the screen image, so a two-texture collapse keeps its own coordinates.
    if fx.distortion && !two_texture {
        for coord in st.iter_mut() {
            coord[1] = 1.0 - coord[1];
        }
    }
```

It runs after `apply_tex_mods`, so it flips the evaluated coordinates, which is what the contract asks for. It is keyed on `fx.distortion && !two_texture` rather than on the item's `screen_image` field; the two agree on every path that reaches this function, because `build_fog_stage_item` does not call it and the two-texture arm passes `two_texture: true`. The note is three lines where the packet asked for "A `≤2`-line note at the site". Minor, not counted as a separate finding.

### `screen_capture_rect` and the projection helpers beside it (`crates/mp/renderer-gpu/src/pipeline3d.rs:4591-4684`)

```rust
/// Raven `R_WorldCoordToScreenCoordFloat` - projects a world point onto the screen in top-down pixel coordinates.
/// It returns `None` where the point sits at or behind the eye plane, Raven's `transformed[2] < 0.01` early return.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:597-635`
fn world_coord_to_screen_coord_float(
    world_coord: vec3_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(f32, f32)> {
    let xcenter = vid_width / 2;
    let ycenter = vid_height / 2;

    let vfwd = refdef.view_axis[0];
    let vright = refdef.view_axis[1];
    let vup = refdef.view_axis[2];

    let mut local = vec3_origin;
    VectorSubtract(world_coord, refdef.view_origin, &mut local);

    let transformed = [
        _DotProduct(local, vright),
        _DotProduct(local, vup),
        _DotProduct(local, vfwd),
    ];

    // The point sits at or behind the eye plane, so it projects nowhere on the screen.
    if (transformed[2] as f64) < 0.01 {
        return None;
    }

    // The two scale terms divide by a double `90.0` and land back in a float, which is the C promotion here.
    let xzi = ((xcenter as f32 / transformed[2]) as f64 * (90.0 / refdef.fov_x as f64)) as f32;
    let yzi = ((ycenter as f32 / transformed[2]) as f64 * (90.0 / refdef.fov_y as f64)) as f32;

    let x = xcenter as f32 + xzi * transformed[0];
    // The y counts down from the top of the screen, because the up component subtracts.
    let y = ycenter as f32 - yzi * transformed[1];
    Some((x, y))
}
```

```rust
/// The source rectangle one distortion capture copies, as `(x, y, side)` in the target texture's top-down coordinates.
/// Raven's `cY` counts from the framebuffer's bottom edge, and the flip back to a top-down origin happens here.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1172-1198`
fn screen_capture_rect(
    e: &refEntity_t,
    refdef: &TrRefdef,
    vid_width: i32,
    vid_height: i32,
) -> Option<(u32, u32, u32)> {
    let (x, y) = world_coord_to_screen_coord(e.origin, refdef, vid_width, vid_height)?;
    let rad = e.radius as i32;
    // A zero or negative radius leaves the oracle an unsampleable texture and wgpu rejects the copy outright.
    // The port takes the defined answer and skips the capture, so the stage binds its own diffuse.
    if rad <= 0 {
        return None;
    }
    // Raven's clamp turns negative once `rad` passes a target dimension, and the copy then reads outside the frame.
    // The side length clamps to the smaller dimension first, which keeps the rect inside the target.
    let rad = rad.min(vid_width).min(vid_height);

    let mut c_x = vid_width - x - (rad / 2);
    let mut c_y = vid_height - y - (rad / 2);
    if c_x + rad > vid_width {
        c_x = vid_width - rad;
    } else if c_x < 0 {
        c_x = 0;
    }
    if c_y + rad > vid_height {
        c_y = vid_height - rad;
    } else if c_y < 0 {
        c_y = 0;
    }

    // The `vid_width - x` term mirrors the square about the vertical centreline. That is Raven's own behavior and it ports as written.
    // The `vid_height - y` term converts a top-down y to the framebuffer's bottom-up origin, so the row flips back here.
    let top = vid_height - c_y - rad;
    Some((c_x as u32, top as u32, rad as u32))
}
```

The arithmetic matches the oracle term for term, including `(int)e.radius` truncation and the two clamps in the oracle's own `if`/`else if` order. The two surviving divergence notes are present. See F8 for the third note that should have been struck, and F10 for the overflow.

### `Gpu::headless_texture` and the `target_texture` threading (`crates/mp/renderer-gpu/src/gpu.rs:246-252`; `crates/mp/renderer-gpu/src/frame_exec.rs:351,409,723`; `crates/mp/renderer-gpu/src/pipeline3d.rs:1200`)

```rust
    /// The offscreen texture the headless target draws into, the copy source a mid-frame capture reads.
    /// Panics on the windowed path, which has no offscreen texture.
    pub fn headless_texture(&self) -> &wgpu::Texture {
        let RenderTarget::Headless(texture) = &self.target else {
            panic!("headless_texture: the gpu is windowed, so it has no offscreen texture");
        };
        texture
    }
```

```rust
        target_texture: &wgpu::Texture,
```

```rust
    /// `target_texture` is the colour attachment's own texture.
    /// A mid-frame screen capture copies out of it, and wgpu takes a texture for that copy rather than a view.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &TextureView,
        target_texture: &wgpu::Texture,
```

Twelve sites change, exactly the count row 2 names: nine `execute_frame` calls (`dev_harness.rs:156`, `ui_harness.rs:245`, `world_harness.rs:958`, `frame_exec.rs:374`, and the five test files), one `execute_package` call (`render_thread.rs:139`), and the two internal forwards at `frame_exec.rs:617` and `:907`. No usage flag, format or resize behavior changed in `gpu.rs`. Nothing found.

### The four `RF_NOSHADOW` marker sites

`crates/mp/renderer/src/tr_ghoul2.rs:840-846`:

```rust
/// TODO: Port RenderSurfaces's stencil- and projection-shadow pushes
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2586-2622`
/// Both arms add a draw surf for a shadow shader, and this workspace has no shadow backend to add one to.
/// MP's projection arm tests `RF_SHADOW_PLANE` alone, and SP's also tests `RF_NOSHADOW` (`oracle/code/renderer/tr_ghoul2.cpp:2801`), so a port keeps MP's shape.
///
/// The `_G2_GORE` overlay chain is deferred in this arm too, and it lands with the gore backend wave.
```

`crates/mp/renderer/src/tr_ghoul2.rs:912-919`:

```rust
        //TODO: Port RenderSurfaces's stencil- and projection-shadow pushes
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2586-2622
        // Both arms need the stencil-shadow backend, which this workspace does not have.
        // The stencil arm gates on `r_shadows == 2` and is the only `RF_NOSHADOW` read here, and the retail default is 1, so it never runs at the shipped setting.
        // MP's projection arm gates on `r_shadows == 3` and tests `RF_SHADOW_PLANE` alone, while SP's also tests `RF_NOSHADOW` (`oracle/code/renderer/tr_ghoul2.cpp:2801`).
        // That asymmetry is load bearing, so a port keeps MP's shape.
        //TODO: Port RenderSurfaces's _G2_GORE overlay pushes
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2623-2715
```

`crates/mp/renderer/src/tr_ghoul2.rs:2466-2469`:

```rust
        //TODO: Port R_AddGhoulSurfaces's bInShadowRange RF_NOSHADOW adjust
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:3525-3528
        // This is the one renderer write of the flag, and `bInShadowRange` needs the stencil-shadow backend this workspace does not have.
        // The `RF_SHADOW_PLANE` and `RF_NOSHADOW` imports return with that wave.
```

`crates/mp/renderer/src/tr_mesh.rs:512-517`:

```rust
        //TODO: Port R_AddMD3Surfaces's stencil- and projection-shadow pushes
        // Source: oracle/codemp/renderer/tr_mesh.cpp:388-405
        // Both arms add a draw surf for `tr.shadowShader` or `tr.projectionShadowShader`, and this workspace has no shadow backend to add one to.
        // The stencil arm gates on `r_shadows == 2` and is the only `RF_NOSHADOW` read here.
        // The projection arm gates on `r_shadows == 3` and tests `RF_SHADOW_PLANE` alone, and SP agrees with MP at this site.
        // The retail default is 1 (`oracle/codemp/renderer/tr_init.cpp:1139`), so neither arm runs at the shipped setting.
```

Every claim in the four notes checks out against the oracle. `oracle/codemp/renderer/tr_ghoul2.cpp:2614-2622` tests `RF_SHADOW_PLANE` alone, `oracle/code/renderer/tr_ghoul2.cpp:2798-2801` adds `&& !(RS.renderfx & ( RF_NOSHADOW ) )`, and both md3 projection arms (`oracle/codemp/renderer/tr_mesh.cpp:400-404`, `oracle/code/renderer/tr_mesh.cpp:415-419`) test `RF_SHADOW_PLANE` alone, so the asymmetry is correctly scoped to ghoul2. `oracle/codemp/renderer/tr_init.cpp:1139` is `r_shadows = Cvar_Get( "cg_shadows", "1", 0 );`. The site at `tr_ghoul2.rs:840` writes the marker as `/// TODO: Port` with a space, not the `//TODO: Port` of `docs/porting-rules.md`; `grep -rn "TODO: Port"` still finds it. See F5 for the fifth marker and F14 for three over-length lines here.

---

## 4. The inventories

### Files changed against the write scopes

| File | Scope | Verdict |
|---|---|---|
| `crates/mp/renderer-gpu/src/pipeline3d.rs` | granted | in scope |
| `crates/mp/renderer-gpu/src/gpu.rs` | `headless_texture` only | in scope, one accessor added |
| `crates/mp/renderer-gpu/src/gpu_images.rs` | `view_bind_group` only | in scope, one method added |
| `crates/mp/renderer-gpu/src/frame_exec.rs` | new parameter on three fns only | in scope |
| `crates/mp/renderer/src/tr_backend.rs` | const plus one note line | in scope |
| `crates/mp/renderer/src/tr_mesh.rs` | one marker conversion | in scope |
| `crates/mp/renderer/src/tr_ghoul2.rs` | three marker conversions | F5, four landed |
| `crates/mp/renderer-gpu/tests/scene_golden.rs` | `scene_distortion` and its test | in scope |
| `crates/mp/renderer-gpu/tests/entity_golden.rs` | the new test and its scene | F13 |
| `crates/mp/renderer-gpu/tests/goldens/scene_distortion.png` | new | in scope |
| `crates/mp/renderer-gpu/tests/goldens/entity_renderfx_duel1.png` | new | in scope |
| `crates/mp/client-app/src/render_thread.rs` | edit-only | in scope |
| `bin/dev_harness.rs`, `ui_harness.rs`, `world_harness.rs` | edit-only | in scope |
| `tests/world_golden.rs`, `ghoul2_vertex_golden.rs`, `hud_golden.rs` | edit-only | in scope |
| `.claude/packets/31/step-010/packet.md` | folder granted for `finished.md` | F6 |
| `.claude/packets/31/step-010/finished.md` | granted | in scope |

`scene_renderfx_tint.png` did not move, so row 5 never fired. No file under `crates/mp/engine/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/` changed. `tr_shade.rs`, `tr_surface.rs`, `tr_shadows.rs`, `stage2d.rs`, `pipeline2d.rs` and every WGSL shader are untouched. `git diff --stat -- '*Cargo.toml' '*.wgsl'` is empty, so no dependency and no shader changed. The three added `pub` items are exactly the three the contract lists:

```
+    pub fn headless_texture(&self) -> &wgpu::Texture {
+    pub fn view_bind_group(
+pub const MAX_POST_RENDERS: usize = 128;
```

No `FrameEvent` variant, no cvar, no ABI or `WorldVertex` change appears in the diff.

### F13 - `entity_golden.rs` carries an unplanned refactor (commit `ee2693d5`)

```rust
+/// The absolute path of the committed golden named `stem`.
+fn golden_path_for(stem: &str) -> PathBuf {
+    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.png"))
+}
+
+/// The absolute path the actual image lands at on a mismatch of the golden named `stem`.
+fn actual_path_for(stem: &str) -> PathBuf {
+    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/goldens/{stem}.actual.png"))
+}
```

Write scope: "`crates/mp/renderer-gpu/tests/entity_golden.rs` - `golden_entity_renderfx_duel1` and its scene." The two existing helpers are rewritten to delegate. Mechanically needed for a second golden in the file, and it is a test file, so it is the smallest of the letter findings.

### Commits against the bundle

The bundle names eight commits. Twelve landed.

| Delivered | Bundle | Verdict |
|---|---|---|
| `feae3d9d feat(gh#31 s010): the frame target texture reaches the backend` | 1 | as planned |
| `69971bf1 feat(gh#31 s010): the post-render deferral` | 2 | as planned |
| `a110a866 feat(gh#31 s010): the per-entity screen capture` | 3 | as planned |
| `78461ecf feat(gh#31 s010): the RF_DISTORTION stage arm` | 4 | as planned |
| `a09b8e38 process(gh#31 s010): the mid-lane amendment` | - | F12, unplanned |
| `af7f04d9 test(gh#31 s010): the distortion golden` | 5 | as planned |
| `9b47e5c9 fix(gh#31 s010): the disintegrate arms read their own entity fields` | - | F12, unplanned |
| `c7da3493 process(gh#31 s010): the dead disintegrate arms amendment` | - | F12, unplanned |
| `ee2693d5 test(gh#31 s010): the disintegrate and volumetric golden` | 6 | as planned |
| `035334af process(gh#31 s010): the renderfx golden bless amendment` | - | F12, unplanned |
| `92641c97 docs(gh#31 s010): the RF_NOSHADOW disposition` | 7 | as planned |
| `9f7e2ca4 process(gh#31 s010): finished file` | 8 | as planned |

### F12 - four unplanned commits

Three are packet amendments (`a09b8e38`, `c7da3493`, `035334af`) and one is a code fix (`9b47e5c9`) to a function the packet declared already correct: "Both are already ported and live ... This step adds no code for them, only a gate." The packet's 2026-08-30 amendment records the user ratification: "The user ratified the one-hunk fix in this lane ... and the change lands as its own commit ahead of the golden." It does land ahead of `ee2693d5`. No commit is bundled: each of the eight planned commits carries only its own bundle item, and no bundle item is split across two commits. The fix widens the step's scope past its stated no-code claim, which is a finding even with the ratification recorded.

### Commit messages

Every subject is a heading noun phrase with the `type(gh#31 s010):` prefix. Every body is unwrapped STE-flavored prose with no em dash, no semicolon and no contraction. No commit carries a `Co-Authored-By` trailer or a generated-with footer. `git log --format=%G?` returns `N` on all twelve, so none is GPG-signed.

### F11 - the `Gates:` paragraph parses as a git trailer

Bundle clause: "Every commit uses `--no-gpg-sign`, a heading subject, an STE body, and no trailer of any kind ... Gate results are written as plain sentences inside the body, so no line parses as a git trailer."

`git log --format="%H|%(trailers)" 61a5062b..gh31-step-010-renderfx` returns the whole `Gates: ...` paragraph as a trailer on eight commits: `feae3d9d`, `69971bf1`, `a110a866`, `78461ecf`, `af7f04d9`, `9b47e5c9`, `ee2693d5`, `92641c97`. Example, `feae3d9d`:

```
TRAILERS:Gates: cargo build --workspace green with that one warning. cargo test --workspace passed serially. All eighteen committed fixtures compared byte-identical across the five golden suites: scene_golden ten tests, world_golden four, entity_golden one, ghoul2_vertex_golden one, hud_golden two.
```

`Gates:` is a token followed by a colon in the message's last paragraph, which is exactly git's trailer form. The clause is violated as written.

---

## 5. Repo mechanics on added lines

Clean. Every check ran over the added lines of the whole range.

- **No `use` inside a function body.** The only added `use` lines are `use core::ops::Range;`, `use mp_renderer::tr_backend::MAX_POST_RENDERS;`, the widened `placeholders` group, the widened `tr_types` group, and two test-file imports. All sit at a file top. `RF_FORCEPOST` merged into the existing `tr_types` group and `TrRefdef` into the existing `placeholders` group, as the contract asks. No inline fully qualified path appears in an added expression; `_DotProduct`, `VectorSubtract`, `vec3_origin` and `R_DecomposeSort` are all short names imported at the head of `pipeline3d.rs`.
- **No `todo!()` or other placeholder added.** `grep` over the added lines finds no `todo!`, no `unimplemented!`. The one added `panic!` is the documented headless-only guard in `Gpu::headless_texture`, not a stub.
- **Every newly ported item carries a `Source:` cite.** `MAX_POST_RENDERS`, `ScreenImage`, both `StageDrawItem` fields, `draw_items`, the four free functions, `view_bind_group` and the four markers all cite. `Gpu::headless_texture` carries none, and it is backend plumbing with no Raven twin, which the contract's own doc text also writes without a cite.
- **No new extern forward-declaration block.**
- **No `format!` builds a wire string.** The two added `format!` calls build filesystem paths in a test file.
- No `unsafe` block is added.

---

## 6. House-style violations on added lines

Read: `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md`.

Clean on em dashes (zero added), semicolons in prose (zero added), banned voice (no casual deflation, no lowercase sentence starts, no `(???)`, no anthropomorphism), and comment placement (no added comment narrates mechanics without carrying knowledge).

### F14 - eleven added comment lines pass 150 columns

House rule: "Any comment line over 150 columns? Break it at a clause boundary."

```
crates/mp/renderer-gpu/src/pipeline3d.rs:1487 (152)             // A surface whose capture did not run keeps its cached group and binds its own diffuse, which is the oracle's answer to a false projection.
crates/mp/renderer/src/tr_ghoul2.rs:843 (161) /// MP's projection arm tests `RF_SHADOW_PLANE` alone, and SP's also tests `RF_NOSHADOW` (`oracle/code/renderer/tr_ghoul2.cpp:2801`), so a port keeps MP's shape.
crates/mp/renderer/src/tr_ghoul2.rs:915 (163)         // The stencil arm gates on `r_shadows == 2` and is the only `RF_NOSHADOW` read here, and the retail default is 1, so it never runs at the shipped setting.
crates/mp/renderer/src/tr_ghoul2.rs:916 (172)         // MP's projection arm gates on `r_shadows == 3` and tests `RF_SHADOW_PLANE` alone, while SP's also tests `RF_NOSHADOW` (`oracle/code/renderer/tr_ghoul2.cpp:2801`).
crates/mp/renderer-gpu/tests/scene_golden.rs:761 (152) /// The sprite draws at screen (220, 104) and spans 64 pixels, and its capture square is 40 pixels, so the sprite carries a magnified crop of the block.
crates/mp/renderer-gpu/tests/scene_golden.rs:763 (156) /// Those 5 rows land at the top of the sprite, because the two worlds store a copied rect in opposite row order and the port reproduces the oracle's order.
crates/mp/renderer-gpu/tests/scene_golden.rs:765 (156) /// `R_WorldCoordToScreenCoordFloat` projects against `viewaxis[1]`, which `AnglesToAxis` fills with the left vector, so its `x` counts from the right edge.
crates/mp/renderer-gpu/tests/scene_golden.rs:766 (161) /// Raven's `cX = vidWidth - x - rad/2` turns that back into a left-edge column, so the capture lands on the entity's own screen position and not a mirror of it.
crates/mp/renderer-gpu/tests/entity_golden.rs:105 (171) /// `RB_CalcDisintegrateColors` compares `oldorigin` against `tess.xyz`, which is model space for a mesh, so a world-space origin would put the burn nowhere near the skin.
crates/mp/renderer-gpu/tests/entity_golden.rs:111 (155) /// The model carries 702 vertices over a 315 unit span, and 184 of them fall inside that radius, so the burn takes a readable bite rather than one corner.
crates/mp/renderer-gpu/tests/entity_golden.rs:491 (153) /// `RB_CalcDisintegrateColors` compares `oldorigin` against `tess.xyz`, which is model space for a mesh, so the burn centre here is a model-space point.
```

Each has a clause boundary to break at. The lengths above exclude the diff's `+` marker.

### F15 - one added comment line carries two sentences

ASD-STE100 self-lint 8: "Any code comment with two sentences on one line? Split them, one sentence per line."

`crates/mp/renderer-gpu/src/pipeline3d.rs:4680`:

```rust
    // The `vid_width - x` term mirrors the square about the vertical centreline. That is Raven's own behavior and it ports as written.
```

This is the same line as F8, so it also needs to be deleted rather than split.

### F16 - `lighting_ref_entity`'s doc comment contradicts its body (commit `9b47e5c9`)

`crates/mp/renderer-gpu/src/pipeline3d.rs:4261-4264`:

```rust
/// `R_SetupEntityLighting` folded the entity light into
/// `lightDir`/`ambientLight`/`directedLight`, and the evaluators read only
/// those plus `shaderRGBA`, so the rest stays at its default.
```

The commit adds `renderfx`, `old_origin` and `end_time` three lines below and leaves the sentence standing. House rule: "Do not leave an old-voice comment as written. Lint every comment you touch."

### F17 - a number disagrees between the doc and the ratified amendment (commit `ee2693d5`)

`crates/mp/renderer-gpu/tests/entity_golden.rs:97-98`:

```rust
/// This distance shrinks each copy to about 255 pixels, which is what lets three of them share one image.
```

The commit body and the packet's 2026-08-30 amendment both say "about 270 pixels". House rule: "Real numbers and names when known." One of the two is wrong.

### F18 - lane vocabulary on two added lines

`crates/mp/renderer/src/tr_ghoul2.rs:845` and `:2469`:

```rust
/// The `_G2_GORE` overlay chain is deferred in this arm too, and it lands with the gore backend wave.
```
```rust
        // The `RF_SHADOW_PLANE` and `RF_NOSHADOW` imports return with that wave.
```

"wave" is process vocabulary, not a term in the code's domain. Both lines carry the word forward from the `DEFERRED:` notes they replace, so this is inherited rather than invented.

`colour` and `centre` are British spellings against the STE American-spelling rule, but `pipeline3d.rs` already carries 53 uses of `colour`, so the added lines follow the file. Not counted.

---

## 7. The gate battery, re-run

Run at `9f7e2ca4`, working tree clean before and after, every command as the packet writes it, each in the foreground.

**`cargo build --workspace`**

```
   Compiling mp_renderer_gpu v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/renderer-gpu)
   Compiling mp_app v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/app)
   Compiling mp_client_app v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/client-app)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.44s
```

Green, zero warnings.

**`cargo test --workspace -- --test-threads=1`**

137 `test result: ok` lines, zero `test result: FAILED`, zero `error`, zero `warning`.

**`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`**

```
running 11 tests
test golden_scene_depthhack ... ok
test golden_scene_distortion ... ok
test golden_scene_dlights ... ok
test golden_scene_fx_cylinder ... ok
test golden_scene_fx_electricity ... ok
test golden_scene_fx_oriented_quad ... ok
test golden_scene_lines ... ok
test golden_scene_polys ... ok
test golden_scene_renderfx_tint ... ok
test golden_scene_saber_glow ... ok
test golden_scene_sprites ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.83s
```

**`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`**

```
running 4 tests
test golden_world_dlights_duel1 ... ok
test golden_world_duel1 ... ok
test golden_world_ffa2 ... ok
test golden_world_marks_duel1 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.34s
```

**`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`**

```
running 2 tests
test golden_entity_duel1 ... ok
test golden_entity_renderfx_duel1 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.12s
```

**`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`**

```
running 1 test
test golden_ghoul2_verts_stormtrooper ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.82s
```

**`cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`**

```
running 2 tests
test golden_hud_2d ... ok
test golden_hud_font ... ignored, needs retail assets and a GPU; run locally with --ignored

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.55s
```

**`cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1`**

```
running 1 test
test golden_hud_font ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 6.35s
```

Twenty committed fixtures compare byte-identical at `CHANNEL_TOLERANCE` 0 (`scene_golden.rs:73`): eleven scene, four world, two entity, one ghoul2, two hud. `git status --porcelain` is empty after the runs, so no `.actual.png` was written and no fixture moved. The whole battery is green on re-run. The commit bodies' fixture counts (eighteen, then nineteen, then twenty) match the count at each commit.

---

## 8. The unverified list

Named plainly. None of these is assumed fine.

1. **The windowed capture path has no test.** `render_thread.rs` passes `&frame.texture`, a swapchain surface texture, into a `copy_texture_to_texture` source. The packet asserts the surface is configured `COPY_SRC` at `gpu.rs:116-118`, and I read that claim but ran no windowed frame. Whether a real swapchain image accepts the copy, and whether an sRGB surface format produces the same sampled values as the headless format, is unverified.
2. **Two distortion entities in one frame.** Row 7's whole argument is that capture N must see deferred draws 1 through N-1, and that one screen-image slot suffices because each capture is consumed immediately. `scene_distortion` draws one distortion sprite. The interleave, the `lastPostEnt` A-B-A double capture, and the keep-alive vector's rebuild path are all unexercised by any committed fixture.
3. **The `MAX_POST_RENDERS` cap.** No fixture defers 128 surfaces, so the fall-through to the sorted pass past the cap is unexercised.
4. **The screen-image rebuild.** No fixture changes the requested side length between captures, so `keep_alive` is never pushed to and the mid-encode texture replacement never runs.
5. **A false projection.** No fixture puts a distortion entity behind the eye plane, so the F7 fallback, the `lastPostEnt` retry and the `rad <= 0` and over-size clamps are unexercised.
6. **The PBR arm.** `BackendMode::Pbr` has no golden in this suite, so the F4 divergence is untested.
7. **The post-render pass viewport.** `draw_items` re-enters each post-render pass with `depth_range = DepthRange::Normal` and calls `set_viewport` only when an item differs, so a deferred surface at the normal window takes the pass default viewport rather than the view's `viewportX`/`viewportY` rect. Every committed scene renders at the full target, so the two agree there. Whether a sub-rect view diverges is unverified, and the same convention already governs the main pass, so it is not new.
8. **Debug-build overflow (F10).** I reasoned about the reachable input range but did not construct a case that panics.
9. **The all-zero colour base (F9).** The argument that alpha `0x00` discards the affected vertices is reasoning, not a rendered comparison.
10. **The two blessed PNGs.** I ran the tests and both compare byte-identical, which proves reproducibility, not correctness of the image. Whether `scene_distortion.png` and `entity_renderfx_duel1.png` show what the packet's defect conditions require is the user's eyes-on ruling of 2026-08-30, recorded in the packet. I did not look at the images.
11. **`scene_renderfx_tint.png` under the deferral.** It did not move, which the commit body attributes to the four force-alpha sprites not overlapping. That the deferral in fact reordered them is asserted by the commit body ("A probe run of that scene showed the two force-alpha sprites deferring and draining in reverse") and not re-run here.
12. **The lockstep referee.** The packet excuses it, because no commit touches `mp_game`, the server or a `jampded` link-set crate, and the diffstat confirms that. Not run.
13. **The `finished.md` content.** Not opened, per the vet brief. Its claims are unchecked.

---

# Fix round - 2026-08-30

Range `9f7e2ca4..gh31-step-010-renderfx`, head `7187ec04`, three commits, every hunk walked in order.

- `8adbfa9a fix(gh#31 s010): the lane-review findings`
- `74480cdc process(gh#31 s010): the lane-review walk amendment`
- `7187ec04 process(gh#31 s010): the finished file fix-round update`

**2 findings, both minor.** Every fix the coordinator named checks out. `finished.md` was not opened: `7187ec04` was read with `git show --stat --format=%B`, which shows the message and the stat and no patch.

## The eight fixes, each checked against its finding

### F7 - the divergence note is present and truthful

The false clause is gone from the group-build site:

```rust
-            // A surface whose capture did not run keeps its cached group and binds its own diffuse, which is the oracle's answer to a false projection.
+            // A surface whose capture did not run keeps its cached group, which holds its own diffuse.
```

The note lands at the bind, two lines, one sentence per line:

```rust
+            // Divergence: the oracle binds `tr.screenImage` unconditionally.
+            // A skipped capture there samples the last successful one or the built-in 8 by 8 white image, and the port binds the stage's own diffuse.
             let texture_group = match (item.screen_image, screen_group) {
```

Both sentences check out. `oracle/codemp/renderer/tr_shade.cpp:2163-2169` binds `tr.screenImage` with no guard, and `oracle/codemp/renderer/tr_image.cpp:2776` allocates it as an 8 by 8 white placeholder. The behavior is unchanged, which is what the ruling ordered.

### F8 and F15 - the mirror sentence is replaced by the no-mirror fact

```rust
-    // The `vid_width - x` term mirrors the square about the vertical centreline. That is Raven's own behavior and it ports as written.
+    // `AnglesToAxis` fills `viewaxis[1]` with the left vector, so the helper's `x` counts from the right edge of the screen.
+    // The `vid_width - x` term converts it back, which puts the capture on the entity's own screen position.
     // The `vid_height - y` term converts a top-down y to the framebuffer's bottom-up origin, so the row flips back here.
```

Verified against the oracle rather than against the amendment's summary. `oracle/codemp/game/q_math.c:530-536` is `AngleVectors( angles, axis[0], right, axis[2] ); VectorSubtract( vec3_origin, right, axis[1] );`, so `viewaxis[1]` holds the negated right vector. `oracle/codemp/renderer/tr_backend.cpp:613,618` reads it as `vright` and writes `transformed[0] = DotProduct(local,vright)`, so a point to the left takes a larger `transformed[0]`, and `:631` gives it a larger `x`. The helper's `x` therefore counts from the right edge, and `vidWidth - x` converts it back. Both sentences are true. One sentence per line, and the `vid_height - y` sentence stands as ordered. F15 closes with it, because the two-sentence line is the line that was replaced.

### F9 - the note and its quarter-weight bound

```rust
+        // Divergence: `RF_DISINTEGRATE1`'s first band writes alpha only, so the oracle leaves rgb at the previous surface's `tess` scratch.
+        // This buffer supplies zeros instead, and the band's own alpha test discards those vertices.
+        // The difference therefore reaches only the border blend, at no more than a quarter vertex weight.
         let mut evaluated = vec![[0u8; 4]; count];
```

Sentence one is right: `oracle/codemp/renderer/tr_shade_calc.cpp:1570-1574` writes `colors[i*4+3] = 0x00` and nothing else, and the port's `crates/mp/renderer/src/tr_shade_calc.rs:606-608` transcribes that as `c[3] = 0x00;` with the other three channels untouched.

Sentence three's bound is right, and it is tight. `RF_DISINTEGRATE1` forces `GLS_ATEST_GE_C0` (`oracle/codemp/renderer/tr_shade.cpp:2046`, ported at `crates/mp/renderer-gpu/src/pipeline3d.rs:4831`), so a surviving fragment carries an interpolated alpha of at least `0xC0`. With `w0` the barycentric weight of a zero-alpha vertex and the other two vertices at most `0xff`, survival needs `255 * (1 - w0) >= 192`, which gives `w0 <= 0.2471`. That is under a quarter. See M1 for the note's length.

### F10 - the `i64` widening

```rust
+    let x = x as i64;
+    let y = y as i64;
+    let rad_wide = rad as i64;
+    let vid_width_wide = vid_width as i64;
+    let vid_height_wide = vid_height as i64;
+
+    let mut c_x = vid_width_wide - x - (rad_wide / 2);
+    let mut c_y = vid_height_wide - y - (rad_wide / 2);
+    if c_x + rad_wide > vid_width_wide {
+        c_x = vid_width_wide - rad_wide;
     } else if c_x < 0 {
         c_x = 0;
     }
```

Bit-identical in range. Every input is an `i32` widened without loss, the operator order and the `if`/`else if` order are unchanged, and `rad_wide / 2` truncates the same way as `rad / 2` because the `rad <= 0` guard above it makes `rad` positive. Overflow is now impossible: the widest term is `vid_height_wide - i32::MIN - rad_wide`, about `6.4e9`, well inside `i64`. The two output casts stay exact, because both clamps leave `c_x` in `0..=vid_width - rad` and `c_y` in `0..=vid_height - rad`, so `c_x as u32` and `top as u32` never truncate.

The rule-19 note is present and names the pick:

```rust
+    // C's out-of-range float-to-int cast is undefined, and Rust's saturates to the `i32` bounds instead.
+    // The clamps below run in `i64` so a saturated projection cannot overflow, and every in-range input gives the same result either way.
```

The saturating cast itself still lives in `world_coord_to_screen_coord` one function above, and the note lives where the consequence lives. Readable either way, not a finding.

### F14 - the 150-column limit

Zero added comment lines in this range pass 150 columns. Measured over every touched file, no line this lane authored anywhere in the step now passes it:

```
crates/mp/renderer/src/tr_ghoul2.rs:1094 (218)  ... "Port G2_ConstructUsedBoneList bone-marking body ..."
crates/mp/client-app/src/render_thread.rs:107 (161) // Latch `r_swapInterval` onto the surface ...
crates/mp/client-app/src/render_thread.rs:109 (152) // The apply semantics port from the SP Mac glimp ...
crates/mp/client-app/src/render_thread.rs:115 (155) // An unsynchronized surface presents as fast as the thread asks ...
```

All four are pre-existing. `tr_ghoul2.rs:1094` is a `todo!()` string, not a comment. The three `render_thread.rs` lines are the DEC-37 swap-interval work of an earlier step, and this step's only edit to that file is the one added `&frame.texture,` argument.

Each of the eleven flagged lines broke at a clause boundary, and each break is semantic. Sample:

```rust
-/// The sprite draws at screen (220, 104) and spans 64 pixels, and its capture square is 40 pixels, so the sprite carries a magnified crop of the block.
+/// The sprite draws at screen (220, 104) and spans 64 pixels, and its capture square is 40 pixels.
+/// The sprite therefore carries a magnified crop of the block.
```

### F16 - the doc rewrite

```rust
+/// Builds the small [`RefEntity`] the colour evaluators read from a `trRefEntity_t`.
+/// `R_SetupEntityLighting` folded the entity light into `lightDir`, `ambientLight` and `directedLight`, which the diffuse arms read.
+/// The colour short-circuits read `shaderRGBA`, and the two disintegrate arms additionally read `renderfx`, `oldorigin` and `endTime`.
+/// Those seven fields are what this builder carries, and every other field keeps its default.
```

The count is right: `light_dir`, `ambient_light`, `directed_light`, `shader_rgba`, `renderfx`, `old_origin`, `end_time` is seven. The claim now matches the body it heads, and the field list matches what `RB_CalcDisintegrateColors`, `RB_CalcDisintegrateVertDeform` and the volumetric block read at `oracle/codemp/renderer/tr_shade_calc.cpp:1553-1557,1566,1649` and `oracle/codemp/renderer/tr_shade.cpp:1577`.

### F17 - 270

```rust
-/// This distance shrinks each copy to about 255 pixels, which is what lets three of them share one image.
+/// This distance shrinks each copy to about 270 pixels, which is what lets three of them share one image.
```

Now agrees with the `ee2693d5` body and with the packet's 2026-08-30 amendment. Which of the two numbers the blessed image actually measures is the lane's claim, not something I measured.

### F18 - the wave rewording

```rust
-/// The `_G2_GORE` overlay chain is deferred in this arm too, and it lands with the gore backend wave.
+// The `_G2_GORE` overlay chain is deferred in this arm too, and its own marker sits at the push site below.
+// The gore fields (`scale`/`fade`/`impactTime`) stay off `G2SurfaceRef` until that backend lands.
```

```rust
-        // The `RF_SHADOW_PLANE` and `RF_NOSHADOW` imports return with that wave.
+        // The `RF_SHADOW_PLANE` and `RF_NOSHADOW` imports return when that backend lands.
```

`grep -in "wave"` over the added lines of this range returns nothing. The remaining 50 hits in `tr_ghoul2.rs` are all pre-existing lines this step never touched.

### The row-8 rider - exactly five markers

`grep -n "TODO: Port"` over the two renderer files:

```
crates/mp/renderer/src/tr_mesh.rs:512:        //TODO: Port R_AddMD3Surfaces's stencil- and projection-shadow pushes
crates/mp/renderer/src/tr_ghoul2.rs:845://TODO: Port RenderSurfaces's stencil- and projection-shadow pushes
crates/mp/renderer/src/tr_ghoul2.rs:912:        //TODO: Port RenderSurfaces's stencil- and projection-shadow pushes
crates/mp/renderer/src/tr_ghoul2.rs:920:        //TODO: Port RenderSurfaces's _G2_GORE overlay pushes
crates/mp/renderer/src/tr_ghoul2.rs:2468:        //TODO: Port R_AddGhoulSurfaces's bInShadowRange RF_NOSHADOW adjust
```

Five, matching the ratified set. Every one takes the exact `//TODO: Port <subject>` form of `docs/porting-rules.md`, with no space after the slashes, and every one is followed immediately by its `// Source:` line. The `/// TODO: Port` doc-comment variant the first walk flagged is gone: the block moved below the doc comment and above `pub fn render_surfaces`, so the doc comment now ends at its own `/// Source: oracle/codemp/renderer/tr_ghoul2.cpp:2521-2735` and the deferral reads as plain `//` lines at the item. The SP asymmetry cite moved onto its own `// Source:` line in both ghoul2 blocks rather than sitting inline in a sentence, which is what took those two lines under 150 columns.

## The amendment commit

`74480cdc` writes `.claude/packets/31/step-010/packet.md` with `22 0` in `git show --numstat`, and `grep -c "^-[^-]"` over that file's portion returns `0`. The hunk header is `@@ -486,3 +486,25 @@`, so the whole entry appends below the previous amendment's last line. A pure tail append, and every line above it is untouched. The ten-row ledger reads consistently with the vet it answers: F7, F8/F15, F9, F10 and the F14/F16/F17/F18 group take code fixes, and F1, F2, F3, F4, F5, F6, F11, F12 and F13 close by ruling with four contract clauses changed.

## The three commit messages

| Commit | Subject | Body | `%G?` | `%(trailers)` |
|---|---|---|---|---|
| `8adbfa9a` | `fix(gh#31 s010): the lane-review findings` | STE, unwrapped, no em dash, no semicolon, no contraction | `N` | empty |
| `74480cdc` | `process(gh#31 s010): the lane-review walk amendment` | same | `N` | empty |
| `7187ec04` | `process(gh#31 s010): the finished file fix-round update` | same | `N` | empty |

Every subject is a heading noun phrase with the `type(gh#31 s010):` prefix. The new gate-paragraph convention holds: `8adbfa9a`'s final paragraph opens "Every fixture held.", `74480cdc`'s opens "The vet record joins the packet folder in the same commit.", and `7187ec04` has no gate paragraph. `git log --format=%(trailers)` returns nothing for all three, so F11's fix is confirmed by git's own parser rather than by inspection. No commit is signed, and none carries a co-author trailer or a generated-with footer.

## The only executable change

`git diff 9f7e2ca4..gh31-step-010-renderfx -- crates/`, with comment and blank lines filtered out, returns the `i64` widening and nothing else:

```
-    let mut c_x = vid_width - x - (rad / 2);
-    let mut c_y = vid_height - y - (rad / 2);
-    if c_x + rad > vid_width {
-        c_x = vid_width - rad;
+    let x = x as i64;
+    let y = y as i64;
+    let rad_wide = rad as i64;
+    let vid_width_wide = vid_width as i64;
+    let vid_height_wide = vid_height as i64;
+    let mut c_x = vid_width_wide - x - (rad_wide / 2);
+    let mut c_y = vid_height_wide - y - (rad_wide / 2);
+    if c_x + rad_wide > vid_width_wide {
+        c_x = vid_width_wide - rad_wide;
-    if c_y + rad > vid_height {
-        c_y = vid_height - rad;
+    if c_y + rad_wide > vid_height_wide {
+        c_y = vid_height_wide - rad_wide;
-    let top = vid_height - c_y - rad;
+    let top = vid_height_wide - c_y - rad_wide;
```

Everything else in the round is comment text. Twenty byte-identical fixtures is exactly what that must produce, and it is what the battery returns.

## Repo mechanics and house style on the round's added lines

Clean. No `use` inside a function body, no `todo!()` or other placeholder, no new extern block, no `format!` building a wire string, no `unsafe`. No em dash, no en dash, no semicolon in prose, no comment line carrying two sentences, no pet vocabulary, no banned voice. Every added comment sits at the site of the fact it carries.

## Findings

### M1 - the F9 divergence note runs to three lines

```rust
        // Divergence: `RF_DISINTEGRATE1`'s first band writes alpha only, so the oracle leaves rgb at the previous surface's `tess` scratch.
        // This buffer supplies zeros instead, and the band's own alpha test discards those vertices.
        // The difference therefore reaches only the border blend, at no more than a quarter vertex weight.
```

Packet heading: "## Divergence notes, each ≤2 lines at its site". The walk's ledger changed four contract clauses and did not change this one, so the two-line cap still binds. The three facts the ledger's own F9 wording asks for do not compress into two lines without dropping one, so the cap and the ruling pull against each other. The note itself is accurate. Trivial, and it needs a ruling on which of the two clauses gives.

The note also states the quarter bound without naming where it comes from. A reader cannot check it without finding `GLS_ATEST_GE_C0` at `pipeline3d.rs:4831`. Under the house KNOWLEDGE-LOCALITY rule that argues for a cite rather than a shorter note, which points the opposite way from the cap.

### M2 - `vet.md` is a third file in the packet folder

`74480cdc` adds `.claude/packets/31/step-010/vet.md`, 881 lines. The write scope grants the folder "for `finished.md`", and the walk's F6 ruling widened that to "Session-directed amendment appends to `packet.md`". Neither line names a vet record. The lane-review ceremony is what puts the file there and the coordinator directed it, so this is a gap in the written scope rather than an unsanctioned write.

## The gate battery, re-run at `7187ec04`

Working tree clean before and after. Every command as the packet writes it, each in the foreground, each serial.

**`cargo build --workspace`**

```
   Compiling mp_renderer_gpu v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/renderer-gpu)
   Compiling mp_app v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/app)
   Compiling mp_client_app v0.1.0 (/Users/milohehmsoth/Developer/Milo/jka-rust/crates/mp/client-app)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.61s
```

Green, zero warnings. The `//TODO` block between the doc comment and `pub fn render_surfaces` draws no diagnostic.

**`cargo test --workspace -- --test-threads=1`** - 137 `test result: ok`, 0 `test result: FAILED`, no error and no warning line.

**`cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1`**

```
running 11 tests
test golden_scene_depthhack ... ok
test golden_scene_distortion ... ok
test golden_scene_dlights ... ok
test golden_scene_fx_cylinder ... ok
test golden_scene_fx_electricity ... ok
test golden_scene_fx_oriented_quad ... ok
test golden_scene_lines ... ok
test golden_scene_polys ... ok
test golden_scene_renderfx_tint ... ok
test golden_scene_saber_glow ... ok
test golden_scene_sprites ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.00s
```

**`cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1`**

```
running 4 tests
test golden_world_dlights_duel1 ... ok
test golden_world_duel1 ... ok
test golden_world_ffa2 ... ok
test golden_world_marks_duel1 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.66s
```

**`cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1`**

```
running 2 tests
test golden_entity_duel1 ... ok
test golden_entity_renderfx_duel1 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.23s
```

**`cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1`**

```
running 1 test
test golden_ghoul2_verts_stormtrooper ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.20s
```

**`cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1`** and **`--ignored`**

```
test golden_hud_2d ... ok
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.63s

test golden_hud_font ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 6.50s
```

Twenty committed fixtures byte-identical at `CHANNEL_TOLERANCE` 0. `git status --porcelain` empty after the runs, so no `.actual.png` was written and no fixture moved.

## The unverified list, after the round

The first walk's thirteen items all stand. The fix round changed one expression and no coverage, so it added no test and closed no gap. Two items change wording:

- Item 8, the debug-build overflow, is now closed by construction rather than unverified: `i64` cannot overflow for any `i32` input, so no case exists to construct.
- Item 9, the all-zero colour base, stays unverified as a rendered comparison. The quarter-weight bound is now derived from `GLS_ATEST_GE_C0` rather than asserted, and that derivation is arithmetic, not a measurement.

One item is added.

14. **The 270-pixel copy width.** F17 changed the doc from 255 to 270 to match the commit body and the packet. I did not measure the blessed image, so which number the image carries is unchecked.
