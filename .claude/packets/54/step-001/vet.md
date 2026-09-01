# Vet gh#54 step-001 - the weather group

Range `1c791387..gh54-step-001-weather`, eleven commits.
The packet and its four Amendments were read whole.
Every `oracle/` cite the packet names was read at the cited lines before any commit was opened.
`finished.md` was not opened.

Sixteen findings, F1 through F16.
This report approves nothing. The session rules on each row.

## 1. Letter violations

### F1 - the branch name is not the contracted one

Packet, `## Write scopes`:

> Branch `gh54-step-001-weather-lane`, cut from `wf/54-renderer-complement`.

The branch is `gh54-step-001-weather`. `wf/54-renderer-complement` (`a081e584`) is an ancestor, so the base is correct.

### F2 - three unlisted items land in `pipeline3d.rs`

Packet, `### crates/mp/renderer-gpu/src/pipeline3d.rs`:

> One new method on `Pipeline3d`, so the weather pass reuses the depth texture the world pass wrote, the cached pipelines, and the globals buffer

Commit `20195f41` adds one method plus three unlisted items. One private struct:

```rust
struct WeatherRun {
    key: PipelineKey,
    image: Option<ImageHandle>,
    nearest: bool,
    alpha_func: u32,
    first_index: u32,
    index_count: u32,
    base_vertex: i32,
}
```

And two private free functions, `world_vertex_from_weather` and `quantize_color_channel` (`pipeline3d.rs:4466,4485`).
Row 10 contracts the quantization site, which covers the two functions. Nothing in the packet covers `WeatherRun`.
All three are private, so no `pub` surface widened and no ABI moved.

### F3 - the trap arms carry more than "one added call each"

Packet, `### crates/mp/engine/client/src/cl_cgame.rs and crates/mp/engine/client/src/cl_ui.rs`:

> Each `RE_RenderScene` trap arm gains one `RE_RenderWorldEffects` call directly after it [...] No signature changes and no other edit in either file.

Both arms gain sixteen lines, not one call (`cl_cgame.rs:2247-2263`, `cl_ui.rs:1373-1389`):

```rust
        let scene_refdef = match re.frame_data.events.last() {
            Some(FrameEvent::RenderScene { refdef, .. }) => Some(refdef.clone()),
            _ => None,
        };
        if let Some(refdef) = scene_refdef {
            RE_RenderWorldEffects(
```

`cl_ui.rs` also gains an import the packet does not list:

```rust
use mp_renderer::render_state::frame_event::FrameEvent;
```

No signature changed in either file. The extra code is the plumbing the ruled trap-side placement needs, and the packet did not state where the refdef comes from.

### F4 - commit 3 does not touch a file its own bundle row lists

Packet, commit bundle row 3:

> Files: `crates/mp/renderer/src/tr_worldeffects/world_effects.rs`, `crates/mp/renderer/src/tr_backend.rs`, `crates/mp/renderer/src/tr_surfacesprites.rs`.

Commit `f7af2115` touches two of the three. `tr_surfacesprites.rs` is unchanged across the whole range.
The commit body declares the narrowing and gives the reason:

> `tr_surfacesprites.rs` needs no edit. `R_SurfaceSpriteFrameUpdate` and `RB_DrawSurfaceSprites` keep their `&WindZoneState` parameter, and neither has a caller anywhere in the workspace.

That reason checks out against the packet's own survey, which named the three `WindZoneState` sites as parameter sites and not call sites.
A narrowed commit is not one of the three finding classes the review names. It is recorded here because the file list moved.

## 2. Oracle divergences

The packet names four ruled divergences: the sim-side compute with a render-side draw, the vertex colour quantization, the `SetViewportAndScissor` retirement, and divergence 4, the per-scene `rdflags` gate.
Two hunks diverge beyond those four.

### F5 - a fifth divergence at the world bounds read

Oracle, `oracle/codemp/renderer/tr_WorldEffects.cpp:559-563`:

```c
		if (!mWeatherZones.size())
		{
			Com_Printf("WARNING: No Weather Zones Encountered");
			AddWeatherZone(tr.world->bmodels[0].bounds[0], tr.world->bmodels[0].bounds[1]);
		}
```

Packet, `### Three stubbed reads exist, and the fourth is mooted`:

> The world bounds are `assets.world.as_ref().map(|w| w.bmodels[0].bounds)`.

Commit `7f8eebbb` lands a different expression, with a rule-19 fallback the packet never contracted:

```rust
            // Submodel 0 is the worldspawn brush model, so these bounds are the whole map.
            // Raven indexes `bmodels[0]` without a length test, which reads out of bounds for a world with no submodel.
            // §19 picks `None` for that case, which leaves the cache unbuilt rather than panicking.
            // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1546
            let world_bmodel_bounds = assets
                .world
                .as_ref()
                .and_then(|w| w.bmodels.first())
                .map(|b| b.bounds);
```

The pick is defensible under porting rule 19 and it is cited at the site.
It is still a fifth divergence, and rule 19 caps the site note at two lines. This one runs to three.

### F6 - the trap arms can step weather under a stale refdef

Oracle, `oracle/codemp/renderer/tr_scene.cpp:868`, is the last statement of `RE_RenderScene`, so the two early returns above it skip it:

```c
	RE_RenderWorldEffects();
```

The port's `RE_RenderScene` carries the same two early returns (`crates/mp/renderer/src/tr_scene.rs:1186-1188` and `:1200-1202`):

```rust
    if !assets.registered {
        return;
    }
```

```rust
    if common.cvar(cvars.r_norefresh).integer != 0 {
        return;
    }
```

Both return before the `RenderScene` push at `crates/mp/renderer/src/tr_scene.rs:1327`.
The trap arm does not know that. It reads the last event on the frame:

```rust
        let scene_refdef = match re.frame_data.events.last() {
            Some(FrameEvent::RenderScene { refdef, .. }) => Some(refdef.clone()),
            _ => None,
        };
        if let Some(refdef) = scene_refdef {
```

On a frame that submits two scenes with `r_norefresh` set, the second trap call finds the first scene's `RenderScene` event still last on the list.
The port then steps `Update` a second time against a refdef that is not this call's.
Raven runs `RE_RenderWorldEffects` zero times on that call.
The packet warned that a step-count difference compounds, because `Update` integrates velocity and advances two random streams:

> `CWeatherParticleCloud::Update` integrates velocity and advances two random streams, so a step-count or step-gate difference compounds across frames.

The same reasoning that made divergence 4 a ruling applies here, and this path has no ruling.
It is not reachable at `r_norefresh 0` with a registered renderer, which is normal play.

## 3. The named hunks

Each is quoted as it stands at `gh54-step-001-weather`.

### `RE_RenderWorldEffects`, `crates/mp/renderer/src/tr_cmds.rs:246-263`

```rust
/// Raven `RE_RenderWorldEffects` — steps the scene's weather and queues its batch.
/// Raven's bufferless `RC_WORLD_EFFECTS` marker becomes the frame's `WorldEffects` event, which carries the batch the pass draws.
/// The caller passes the submitted scene's own refdef, so the guard gates on that scene's flags and not on a later one's.
///
/// Raven queues the marker once per scene and the backend steps `Update` once per marker, so this fn runs once per `RE_RenderScene` and the counts match.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:291-300`
pub fn RE_RenderWorldEffects(
    frame: &mut FrameData,
    world_effects: &mut WorldEffectsState,
    assets: &RenderAssets,
    refdef: &TrRefdef,
    host: &mut EngineHostView,
) {
    let weather = world_effects.RB_RenderWorldEffects(assets, refdef, host);
    frame.events.push(FrameEvent::WorldEffects(weather));
}
```

The signature matches the packet's contracted signature exactly.

### Trap arm one, `crates/mp/engine/client/src/cl_cgame.rs:2247-2263`

```rust
        // Raven queues the weather command as the last act of a scene, so the step runs here.
        // `RE_RenderScene` seals the scene with its own refdef, and the weather guard reads that scene's flags rather than a later one's.
        // Source: oracle/codemp/renderer/tr_scene.cpp:868
        let scene_refdef = match re.frame_data.events.last() {
            Some(FrameEvent::RenderScene { refdef, .. }) => Some(refdef.clone()),
            _ => None,
        };
        if let Some(refdef) = scene_refdef {
            RE_RenderWorldEffects(
                &mut re.frame_data,
                &mut re.world_effects,
                &re.sim.published,
                &refdef,
                view,
            );
        }
```

### Trap arm two, `crates/mp/engine/client/src/cl_ui.rs:1373-1389`

The same sixteen lines, byte for byte.

Both arms sit directly after the `RE_RenderScene` call, the oracle's own placement.
Neither arm reaches the renderer through `re_from_view(view)` as the packet's row 2 described. Both use the already destructured `re` and pass `view` on. The effect is the same route.

### The `rdflags` gate, `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1660-1670`

```rust
        // Raven: "no world rendering or no world or no particle clouds"
        //
        // Raven's `||` chain short-circuits left to right in C, and so does Rust's, so the four terms keep their oracle order.
        // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1515-1521
        if assets.world.is_none()
            || refdef.rdflags & RDF_NOWORLDMODEL != 0
            || refdef.rdflags & RDF_SKYBOXPORTAL != 0
            || self.mParticleClouds.is_empty()
        {
            return weather;
        }
```

Against the oracle at `:1515-1521`:

```c
	if (!tr.world || 
		(tr.refdef.rdflags & RDF_NOWORLDMODEL) || 
		(backEnd.refdef.rdflags & RDF_SKYBOXPORTAL) || 
		!mParticleClouds.size()) 
	{	//  no world rendering or no world or no particle clouds
		return;
	}
```

The four terms keep their order.
Rust binds `&` tighter than `!=`, so `refdef.rdflags & RDF_NOWORLDMODEL != 0` parses as `(refdef.rdflags & RDF_NOWORLDMODEL) != 0`, which is the oracle's test. C binds the two the other way round, and the transcription is correct for Rust.
The two-refdef collapse is divergence 4, ruled cosmetic in row 2.
Nothing found wrong.

### `FrameEvent::WorldEffects`, `crates/mp/renderer/src/render_state/frame_event.rs:191-196`

```rust
    /// Raven's `RC_WORLD_EFFECTS` backend command, carrying the batch `RB_RenderWorldEffects` already built.
    /// The oracle queues it after the scene's `RC_DRAW_SURFS`, so this event follows `RenderScene` and the pass draws over the world.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:868`, `oracle/codemp/renderer/tr_cmds.cpp:291-300`
    WorldEffects(WeatherFrame),
```

Word for word the packet's contracted text. `WorldEffectCommand(String)` is untouched, as contracted.

### The executor arm and the view rebuild, `crates/mp/renderer-gpu/src/frame_exec.rs:655-698`

```rust
                FrameEvent::WorldEffects(weather) => {
                    // The oracle's `RB_RenderWorldEffects` runs under `backEnd.viewParms`, the view the scene ahead of it built.
                    // The two inputs `draw_weather` reads are derived, not carried: `R_RotateForViewer` needs the refdef's own
                    // origin and axis, and `R_SetupProjection` needs the visible bounds the world walk grew, which the view
                    // state still holds. Rebuilding both here reproduces that view without a carrier of its own.
                    // Source: oracle/codemp/renderer/tr_main.cpp:1612-1613, oracle/codemp/renderer/tr_WorldEffects.cpp:1523-1525
                    match scene_refdef {
                        Some(refdef) if cvars.skip_back_end == 0 && !weather.is_empty() => {
                            let mut view = zeroed_view_parms();
                            view.viewportX = refdef.x;
                            view.viewportY = refdef.y;
                            view.viewportWidth = refdef.width;
                            view.viewportHeight = refdef.height;
                            view.fovX = refdef.fov_x;
                            view.fovY = refdef.fov_y;
                            view.ori.origin = refdef.view_origin;
                            view.ori.axis = refdef.view_axis;
                            view.pvsOrigin = refdef.view_origin;

                            R_RotateForViewer(&mut view);
                            view.visBounds = self.view_state.view.vis_bounds;
                            R_SetupProjection(
                                &mut view,
                                refdef.rdflags,
                                refdef.fov_x,
                                refdef.fov_y,
                                assets.distance_cull,
                                cvars,
                            );

                            stats.world.weather_vertices += self.pipeline3d.draw_weather(
                                gpu,
                                target,
                                weather,
                                &view,
                                gpu_images,
                            );
                        }
                        None => {
                            // No scene came ahead of this event, so there is no view to draw it under.
                            self.warn_once(Warned::Other);
                        }
                        _ => {}
                    }
                }
```

The carrier of the refdef is set in the `RenderScene` arm:

```rust
        // The weather event carries no view of its own, so the walk keeps the refdef of the scene it follows.
        // Source: oracle/codemp/renderer/tr_scene.cpp:868
        let mut scene_refdef: Option<&TrRefdef> = None;
```

```rust
                    scene_refdef = Some(refdef);
```

The seven viewport and view fields are the same seven the world path fills at `crates/mp/renderer-gpu/src/frame_exec.rs:853-867`, in the same order and from the same refdef fields, so the viewport rect is provably identical to the world pass's.
The two matrices are rebuilt rather than reused. See section 8.
See F10 for `Warned::Other`, and F14 for the column wrap in the leading comment.

### `Pipeline3d::draw_weather`, `crates/mp/renderer-gpu/src/pipeline3d.rs:1569-1743`

```rust
    /// The GPU half of Raven `CWeatherParticleCloud::Render`: one draw per cloud, after the world pass, depth-tested and depth-write off.
    /// The pass sets its own viewport and scissor from `view`, the same values the world pass used, because `SetViewportAndScissor` is retired at the CPU site.
    /// The pass draws two-sided. That is faithful, not incidental: Raven sets `GL_Cull(CT_TWO_SIDED)` for weather at `oracle/codemp/renderer/tr_WorldEffects.cpp:1362`.
    /// The pass loads both attachments, so the world the previous pass drew stays and its depth still occludes the billboards.
    /// Returns the vertex count drawn, which the frame stats report.
    ///
    /// The batch carries `WeatherFrame` order, and later clouds blend over earlier ones.
    /// `WeatherFrame` carries no view of its own: the caller draws it under the view of the scene its event follows.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`
    pub fn draw_weather(
        &mut self,
        gpu: &Gpu,
        target: &TextureView,
        weather: &WeatherFrame,
        view: &viewParms_t,
        gpu_images: &mut GpuImages,
    ) -> u32 {
        if weather.is_empty() {
            return 0;
        }

        // Concatenate every cloud's billboards into one vertex block and one index block, keeping each cloud's own run.
        let mut vertices: Vec<WorldVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut runs: Vec<WeatherRun> = Vec::new();
        for batch in &weather.clouds {
            if batch.indices.is_empty() {
                continue;
            }
            let base_vertex = vertices.len() as i32;
            let first_index = indices.len() as u32;
            vertices.extend(batch.vertices.iter().map(world_vertex_from_weather));
            indices.extend_from_slice(&batch.indices);
            runs.push(WeatherRun {
                key: PipelineKey {
                    blend: blend_state_from_gls(batch.state_bits),
                    depth_equal: false,
                    // Neither weather blend mode sets `GLS_DEPTHMASK_TRUE`, so weather depth-tests and does not depth-write.
                    depth_write: false,
                    depth_bias: false,
                },
                image: batch.image,
                nearest: batch.nearest_filter,
                alpha_func: alpha_func_code(batch.state_bits),
                first_index,
                index_count: batch.indices.len() as u32,
                base_vertex,
            });
        }
        if runs.is_empty() {
            return 0;
        }

        // The world pass already submitted its encoder, so these writes land after it and the reuse of both buffers is safe.
        self.reserve_globals(gpu, 1);
        self.write_globals(
            gpu,
            &[world_clip_matrix(
                &view.world.modelMatrix,
                &view.projectionMatrix,
            )],
        );

        // One flags block per cloud, all of them the single-texture path, because a cloud binds one image and no lightmap.
        self.reserve_flags(gpu, runs.len());
        let mut flag_bytes = vec![0u8; runs.len() * SURFACE_FLAGS_STRIDE as usize];
        for (index, run) in runs.iter().enumerate() {
            let flags = SurfaceFlagsGpu {
                mode: MODE_SINGLE,
                tex_from_lightmap: 0,
                alpha_func: run.alpha_func,
                pbr_lit: 0,
            };
            let offset = index * SURFACE_FLAGS_STRIDE as usize;
            let src = bytemuck::bytes_of(&flags);
            flag_bytes[offset..offset + src.len()].copy_from_slice(src);
        }
        gpu.queue().write_buffer(&self.flags_buffer, 0, &flag_bytes);

        self.reserve_dynamic(gpu, vertices.len());
        gpu.queue()
            .write_buffer(&self.dynamic_buffer, 0, bytemuck::cast_slice(&vertices));
        self.reserve_dynamic_indices(gpu, indices.len());
        gpu.queue().write_buffer(
            &self.dynamic_index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );

        // Every pipeline the pass uses must exist before the pass borrows `self` immutably.
        for run in &runs {
            self.ensure_pipeline(gpu, run.key);
        }

        let bind_groups: Vec<wgpu::BindGroup> = runs
            .iter()
            .map(|run| {
                gpu_images.weather_bind_group(gpu, &self.texture_layout, run.image, run.nearest)
            })
            .collect();

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mp_renderer_gpu weather encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mp_renderer_gpu weather pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // `viewParms_t::viewportY` carries the unflipped, 0-at-the-top y, which is what wgpu wants.
            // Source: oracle/codemp/renderer/tr_backend.cpp:457-467
            pass.set_viewport(
                view.viewportX as f32,
                view.viewportY as f32,
                view.viewportWidth as f32,
                view.viewportHeight as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                view.viewportX.max(0) as u32,
                view.viewportY.max(0) as u32,
                view.viewportWidth.max(0) as u32,
                view.viewportHeight.max(0) as u32,
            );
            pass.set_index_buffer(
                self.dynamic_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            pass.set_vertex_buffer(0, self.dynamic_buffer.slice(..));

            for (index, run) in runs.iter().enumerate() {
                let pipeline = self
                    .pipelines
                    .get(&run.key)
                    .expect("a weather pipeline was created for every run's key above");
                let offset = (index as u64 * SURFACE_FLAGS_STRIDE) as u32;
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &self.globals_bind_group, &[0]);
                pass.set_bind_group(1, &bind_groups[index], &[]);
                pass.set_bind_group(2, &self.flags_bind_group, &[offset]);
                pass.draw_indexed(
                    run.first_index..run.first_index + run.index_count,
                    run.base_vertex,
                    0..1,
                );
            }
        }
        gpu.queue().submit(std::iter::once(encoder.finish()));

        vertices.len() as u32
    }
```

The signature matches the contract exactly.
`PipelineKey` gains no cull field, per row 12. `build_world_pipeline` already sets `cull_mode: None` (`pipeline3d.rs:3626`) and `PrimitiveTopology::TriangleList` (`:3622`), so the two-sided draw and the triangle list are what the reused pipeline gives.
See F2 for `WeatherRun`, and F13 and F15 for the doc lines.

### `GpuImages::weather_bind_group` and `sampler_nearest`, `crates/mp/renderer-gpu/src/gpu_images.rs`

The field, at `:97-102`:

```rust
    /// The unmipmapped nearest sampler Raven's `mFilterMode != 0` clouds bind.
    /// Weather sets both the min and the mag filter, and neither weather filter uses mips.
    /// The weather image always loads with `GL_CLAMP`, so one nearest-clamp sampler covers every cloud.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
    sampler_nearest: Sampler,
```

The builder, at `:395-435`:

```rust
    /// The weather bind group for one cloud: its image with the clamp wrap `R_FindImageFile` gave it, and the filter its `mFilterMode` chose.
    /// A cloud binds one image and no lightmap, so slot 2 takes the white texel and the world shader's second sample reduces to one.
    /// `layout` is the world pipeline's group-1 layout, which `Pipeline3d` owns, the same way `world_bind_group` and `view_bind_group` take it.
    ///
    /// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320,1364-1365`
    pub fn weather_bind_group(
        &self,
        gpu: &Gpu,
        layout: &BindGroupLayout,
        handle: Option<ImageHandle>,
        nearest: bool,
    ) -> BindGroup {
        let image = self.image_or_white(handle);
        let sampler = if nearest {
            &self.sampler_nearest
        } else {
            &self.sampler_clamp
        };
```

The signature matches the bind-group layout Amendment of 2026-08-31 exactly, and `draw_weather` passes `&self.texture_layout`.

The sampler builder, at `:563-575`:

```rust
/// The nearest-filtered clamping sampler a `mFilterMode != 0` weather cloud binds.
/// Both the min and the mag filter are nearest, and neither uses mips.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
fn create_nearest_sampler(device: &wgpu::Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("mp_renderer_gpu weather sampler (nearest)"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
```

The sampler state matches the oracle's `GL_NEAREST` on both filters with no mip filter.
See F8 for where this function was placed.

### `CWeatherParticleCloud::Update`'s rand-draw sites

Site one, the spawn-plane respawn at `crates/mp/renderer/src/tr_worldeffects/world_effects.rs:1345-1365`:

```rust
                if self.UseSpawnPlane() {
                    self.mParticles[part].mPosition = self.mCameraPosition;
                    // `CVec3` has no binary scalar multiply, so the float converts through the broadcast constructor and this line draws nothing.
                    // Source: oracle/codemp/Ravl/CVec.h:570,628
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] -=
                            self.mSpawnPlaneNorm[i] * self.mSpawnPlaneDistance;
                    }
                    // The same broadcast makes each of the next two lines exactly one `WE_flrand` draw, scaling x, y and z alike.
                    // A per-component draw would take six values instead of two and shift the stream for the rest of the session.
                    // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:1216-1217
                    let right_scale = WE_flrand(rng, -self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] +=
                            self.mSpawnPlaneRight[i] * right_scale;
                    }
                    let up_scale = WE_flrand(rng, -self.mSpawnPlaneSize, self.mSpawnPlaneSize);
                    for i in 0..3 {
                        self.mParticles[part].mPosition[i] += self.mSpawnPlaneUp[i] * up_scale;
                    }
                }
```

Against the oracle at `:1211-1218`:

```c
				if (UseSpawnPlane())
				{
					part->mPosition		= mCameraPosition;
					part->mPosition		-= (mSpawnPlaneNorm* mSpawnPlaneDistance); 
					part->mPosition		+= (mSpawnPlaneRight*WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
					part->mPosition		+= (mSpawnPlaneUp*   WE_flrand(-mSpawnPlaneSize, mSpawnPlaneSize)); 
				}
```

One draw per line, hoisted into a named local and broadcast to all three components. Row 9 is honoured, and the four statements keep their order.
`WE_flrand` draws off the C runtime stream (`world_effects.rs:71-73`), which is what `oracle/codemp/renderer/tr_WorldEffects.cpp:13-15` does.

Site two, the two rotation picks at `world_effects.rs:1167-1179`:

```rust
        if self.mRotationChangeNext != -1 {
            if self.mRotationChangeNext == 0 {
                // The two picks draw from different streams: `mRotation` from the C runtime `rand`, and `mRotationChangeTimer` from `holdrand`.
                self.mRotation.Pick(rng, &mut self.mRotationDeltaTarget);
                // `Reset` writes `mRotationChangeTimer.mMin` twice and never writes `mMax`, so the range is reversed: min 2000, max 0.
                // `Q_irand` increments `max` first, so `((result * -1999) >> 15) + 2000` over `result` in `[0, 32767]` lands uniformly in `[1, 2000]`.
                // The clamp below therefore never fires, and the rotation interval varies.
                // Source: oracle/codemp/game/q_math.c:1464-1467
                self.mRotationChangeNext = self.mRotationChangeTimer.Pick(rng);
                if self.mRotationChangeNext <= 0 {
                    self.mRotationChangeNext = 1;
                }
```

The note is the row-11 rewrite, and its arithmetic checks out.
`Reset` in the port writes `mMin` twice (`world_effects.rs:1119,1122`) and never writes `mMax`, which stays 0 from the constructor (`:965`), so the reversed range is real.
`SIntRange::Pick` goes to `Q_irand` on `holdrand` (`:253-255`) and `mRotation.Pick` goes to `WE_flrand` on the C runtime stream, so the two-stream split is right.
The two picks keep their order.
The `mRotationChangeNext--` decrement and the whole asymmetric spin block below are unchanged context and still match the oracle at `:1090-1094`.
Nothing found wrong at either site.

The freeze gate placement is also correct. `if frozen { return; }` sits after the camera and range work and before `self.mParticleCountRender = 0;` (`world_effects.rs:1270-1281`), which is the oracle's order at `:1166-1176`.

### `golden_world_weather_ctf2` and both reseed sites, `crates/mp/renderer-gpu/tests/world_golden.rs:801-973`

Reseed one, before the command triple:

```rust
    // ---- pin both generator streams, build the weather, pin them again ---
    // The first pin covers the commands themselves. `CWeatherParticleCloud::Initialize` picks every particle's `mMass` off the
    // C runtime stream, and it runs inside `R_WorldEffectCommand`, so the snow and fog commands take 1060 draws before the
    // second pin. Mass divides the force, so an unpinned draw here gives each particle its own fall rate and the image moves.
    // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:928-935
    host.re.world_effects.rng.srand(WEATHER_SEED_CRT);
    host.re.world_effects.rng.Rand_Init(WEATHER_SEED_HOLDRAND);

    for command in CTF2_WEATHER_COMMANDS {
        weather_command(&mut host, command);
    }
```

Reseed two, after the commands and before stepping:

```rust
    // The second pin keeps the stepped stream independent of how many draws the command path took.
    host.re.world_effects.rng.srand(WEATHER_SEED_CRT);
    host.re.world_effects.rng.Rand_Init(WEATHER_SEED_HOLDRAND);
```

Both reseeds touch both streams, which is what the reseed Amendment ruled.
`Rng::srand` seeds the C runtime state and `Rng::Rand_Init` seeds `holdrand`, so the pinning is complete.
The commands and the step loop:

```rust
const CTF2_WEATHER_COMMANDS: [&str; 3] = ["snow", "fog", "constantwind (100 100 -100)"];
```

```rust
const WEATHER_STEPS: i32 = 60;
const WEATHER_STEP_MS: i32 = 33;
```

```rust
    let mut frame_data = FrameData { events: Vec::new() };
    for step in 0..WEATHER_STEPS {
        let mut refdef = build_refdef(eye, [0.0, 0.0, 0.0]);
        refdef.time = FROZEN_TIME_MS + step * WEATHER_STEP_MS;
        frame_data = FrameData { events: Vec::new() };
        record_scene(&mut host, &refdef, &mut frame_data);
        step_weather(&mut host, &mut frame_data);
    }
```

The clock advances by one step per call, so `frametime` is 33 on every call after the first and is not the zero a repeated frozen clock gives. That is what row 7 step 3 asked for.
The triple is the retail `SP_CreateSnow` order from `oracle/codemp/game/g_misc.c:2522-2527`, and the fixture asserts two clouds and one wind zone.
The empty-batch gate is present:

```rust
        // An empty batch would bless the plain ctf2 room, so the counter gates the golden.
        assert!(
            stats.world.weather_vertices > 0,
            "no weather vertex drawn: stats.world = {:?}",
            stats.world,
        );
```

Nothing found wrong. The fixture's live-path claim also holds: no reseed reaches `R_InitWorldEffects`, so the wall-clock `srand` is untouched in production.

### The colour quantization site, `crates/mp/renderer-gpu/src/pipeline3d.rs:4459-4488`

```rust
/// Converges one weather billboard vertex into the GPU vertex row, which is where the colour quantizes.
///
/// `qglColor4f` takes floats and the fixed-function pipeline converts them to fixed point before it interpolates.
/// `WorldVertex` carries `[u8; 4]`, and every other billboard in the port goes through it, so the rounding lands here at one site.
/// A weather cloud carries no lightmap and no normal, so those fields stay zero and the single-texture path never reads them.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1403`
fn world_vertex_from_weather(v: &WeatherVertex) -> WorldVertex {
    let color = [
        quantize_color_channel(v.color[0]),
        quantize_color_channel(v.color[1]),
        quantize_color_channel(v.color[2]),
        quantize_color_channel(v.color[3]),
    ];
    let vert = drawVert_t {
        xyz: v.position,
        st: v.st,
        lightmap: [[0.0; 2]; MAXLIGHTMAPS],
        normal: [0.0; 3],
        color: [color; MAXLIGHTMAPS],
    };
    WorldVertex::from_draw_vert(&vert)
}

/// One colour channel from the float `qglColor4f` value to the vertex row's byte: clamp, scale by 255, round to nearest.
/// The premultiplied blend mode makes every channel depend on this rounding, so truncation would move every pixel.
fn quantize_color_channel(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}
```

The rule row 10 pinned is `clamp to [0.0, 1.0]`, then times 255.0, then `.round()`, then cast to `u8`. The code does exactly that, in that order.
The upstream float colour is correct against `oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1403`, verified in `Render`:

```rust
            let color: [f32; 4] = if self.mBlendMode == 0 {
                [self.mColor[0], self.mColor[1], self.mColor[2], alpha]
            } else {
                [
                    self.mColor[0] * alpha,
                    self.mColor[1] * alpha,
                    self.mColor[2] * alpha,
                    self.mColor[3] * alpha,
                ]
            };
```

See F16 for the first doc word.

## 4. The inventories

### Files against the write scopes

Sixteen files changed. Every one is inside the write scopes:

```
 .claude/packets/54/step-001/finished.md            | 107 +++
 .claude/packets/54/step-001/packet.md              |  18 +-
 crates/mp/engine/client/src/cl_cgame.rs            |  20 +-
 crates/mp/engine/client/src/cl_ui.rs               |  19 +-
 crates/mp/renderer-gpu/src/frame_exec.rs           |  55 +-
 crates/mp/renderer-gpu/src/gpu_images.rs           |  67 ++
 crates/mp/renderer-gpu/src/pipeline3d.rs           | 224 ++++++-
 .../tests/goldens/world_weather_ctf2.png           | Bin 0 -> 516659 bytes
 crates/mp/renderer-gpu/tests/world_golden.rs       | 280 ++++++++
 crates/mp/renderer/src/render_state/frame_event.rs |   6 +
 crates/mp/renderer/src/render_state/mod.rs         |   1 +
 .../mp/renderer/src/render_state/weather_frame.rs  |  52 ++
 crates/mp/renderer/src/tr_backend.rs               |  16 +-
 crates/mp/renderer/src/tr_cmds.rs                  |  33 +-
 crates/mp/renderer/src/tr_scene.rs                 |   6 +-
 .../renderer/src/tr_worldeffects/world_effects.rs  | 738 +++++++++++++++++----
```

No `Cargo.toml`, no WGSL, no fixture but the one new PNG, nothing under `crates/mp/game/`, `crates/mp/cgame/`, `crates/mp/ui/`, `crates/mp/uishared/`, or `crates/sp/`. No new dependency.
One scoped file, `crates/mp/renderer/src/tr_surfacesprites.rs`, is untouched. See F4.

The goldens directory holds twenty-two files, the twenty-one that existed plus `world_weather_ctf2.png`. No other fixture moved.
The new PNG's SHA-256 is `4044611f92816317b3c5f94a03bd9c60f04bb61a5906b69771aff9851696aa05`, which is the hash the row-7 bless Amendment recorded.

### Commits against the bundle

The eight contracted subjects all land, in the contracted order, word for word:

| bundle | commit | subject |
| --- | --- | --- |
| 1 | `7f8eebbb` | `fix(gh#54 s001): the four world-effects reads land` |
| 2 | `67e87764` | `feat(gh#54 s001): the weather particle loop` |
| 3 | `f7af2115` | `feat(gh#54 s001): the wind zone state finds its owner` |
| 4 | `580ddf97` | `feat(gh#54 s001): the cloud renders into a frame batch` |
| 5 | `20195f41` | `feat(gh#54 s001): the weather pass draws a frame batch` |
| 6 | `6de7631c` | `feat(gh#54 s001): the frame drives world effects` |
| 7 | `3830ec2f` | `test(gh#54 s001): the weather world golden` |
| 8 | `f7ffe366` | `process(gh#54 s001): finished file` |

No commit is widened. No commit is bundled. No split and no reorder.

### F7 - three unplanned commits

Three commits are not in the bundle:

- `0dfed259 process(gh#54 s001): the bind-group layout amendment`
- `b7d9f27d process(gh#54 s001): the reseed amendment`
- `51183d1b process(gh#54 s001): the row-7 bless amendment`

Each touches only `.claude/packets/54/step-001/packet.md`, which the write scopes allow "for session-directed `packet.md` tail appends".
Each body names a user ruling of 2026-08-31, and each corresponding Amendment is in the packet.
They are still unplanned commits against the bundle, and the review names an unplanned commit a finding.

### Commit messages

Every subject is a heading. Every body is unwrapped STE prose in plain sentences, with the gate results written as a prose paragraph so no line parses as a trailer.
`git log --format=%(trailers)` returns empty for all eleven commits. No `Co-Authored-By`, no generated-with footer, no signature (`%G?` is `N` on all eleven, matching `--no-gpg-sign`).
Two bodies carry pet vocabulary. See F12.

## 5. Repo mechanics on added lines

Clean on four of the five checks.

- No `use` declaration inside a function body. The only added `use` below file scope is `use super::*;` inside `#[cfg(test)] mod tests`, which the porting rules exempt.
- No `todo!()`, no `unimplemented!()`, no other placeholder. `grep -n "TODO: Port\|todo!" crates/mp/renderer/src/tr_worldeffects/world_effects.rs` returns nothing, so all five markers are closed.
- No new extern forward-declaration block. `grep -n 'extern'` over the added lines returns nothing.
- No `format!` call at all on the added lines, so none builds a wire string.
- Every newly ported item carries an oracle `Source:` cite. The items without one are port inventions with no oracle line: `WeatherFrame::is_empty`, `WeatherRun`, `quantize_color_channel`, and `WorldStats::weather_vertices`.

Three findings.

### F8 - a new function steals an existing function's doc comment

Commit `20195f41` inserts `create_nearest_sampler` between `create_sampler`'s doc comment and `create_sampler` itself.
The file now reads (`crates/mp/renderer-gpu/src/gpu_images.rs:558-579`):

```rust
/// Bilinear in both directions, matching `R_RegisterShaderNoMip`'s
/// `GL_LINEAR` 2D images; `address_mode` carries the image's
/// `wrapClampMode`.
///
/// Source: `oracle/codemp/renderer/tr_shader.cpp` (`R_RegisterShaderNoMip`)
/// The nearest-filtered clamping sampler a `mFilterMode != 0` weather cloud binds.
/// Both the min and the mag filter are nearest, and neither uses mips.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1364-1365`
fn create_nearest_sampler(device: &wgpu::Device) -> Sampler {
```

```rust
fn create_sampler(device: &wgpu::Device, address_mode: AddressMode, label: &str) -> Sampler {
```

Two consequences. `create_nearest_sampler` now documents itself as bilinear and carries two contradictory `Source:` cites. `create_sampler` is left with no doc comment at all.
The diff itself shows the insertion point, so this is a mechanical slip and not a judgement call.

### F9 - format pollution in a file that was rustfmt-clean

`crates/mp/renderer/src/tr_worldeffects/world_effects.rs` had zero `rustfmt --check` hunks at `1c791387`. It now has three, two of them in lane-added code:

```rust
use native_math::qmath::{MakeNormalVectors, VectorNormalize, _DotProduct};
```

rustfmt wants `{_DotProduct, MakeNormalVectors, VectorNormalize}`.

```rust
                self.wind.global_wind_speed =
                    VectorNormalize(&mut self.wind.global_wind_direction);
```

rustfmt wants that on one line, and the joined line is 90 columns.

```rust
        cloud.Update(&mut rng, &outside, [0.0; 3], VIEW_AXIS, false, [0.0; 3], 0.02);
```

rustfmt wants that broken.

`crates/mp/renderer-gpu/tests/world_golden.rs` gains one new hunk from the lane's out-of-order import:

```rust
use mp_renderer::tr_cmds::RE_RenderWorldEffects;
```

placed after `use mp_renderer::tr_scene::{...}`.

The other eight touched files carry the same hunk count they carried at the base, so the drift is confined to these two.
The workspace is not rustfmt-clean overall and rustfmt is not one of the packet's gates.

### F10 - the executor's weather warning reuses a slot whose text is now false

`crates/mp/renderer-gpu/src/frame_exec.rs:692-694`:

```rust
                        None => {
                            // No scene came ahead of this event, so there is no view to draw it under.
                            self.warn_once(Warned::Other);
                        }
```

`Warned::Other`'s message is (`frame_exec.rs:180`):

```rust
            Warned::Other => "skips world-effect / automap commands — not rendered yet",
```

The lane's own commit 6 makes that sentence false: world effects are rendered now.
`warn_once` is a one-shot per slot, so any earlier `Other` warn on the frame silences the weather case, and the message that does print names the wrong condition.

## 6. House-style violations on added lines

Read: `~/.claude/skills/house-style/SKILL.md` and `~/.claude/skills/asd-ste100/SKILL.md`.

### F11 - two em dashes

The house rule: "No em dashes. The plain hyphen `-` is the only dash."

`crates/mp/renderer/src/render_state/weather_frame.rs:1`:

```rust
//! `weather_frame` — the seam payload one frame of weather billboards crosses on.
```

`crates/mp/renderer/src/tr_cmds.rs:246`:

```rust
/// Raven `RE_RenderWorldEffects` — steps the scene's weather and queues its batch.
```

The packet's contracted text for the second one uses a plain hyphen. The lane substituted an em dash.

### F12 - pet vocabulary, "seam"

The house rule bans "minted, materialized, canonical, rides, seam, dial".

`crates/mp/renderer/src/render_state/weather_frame.rs:1`:

```rust
//! `weather_frame` — the seam payload one frame of weather billboards crosses on.
```

Commit `20195f41`'s body:

> `FrameEvent::WorldEffects` carries the batch across the seam, and `Pipeline3d::draw_weather` draws it.

Commit `580ddf97`'s body:

> `render_state/weather_frame.rs` lands the three seam carriers: `WeatherVertex`, `WeatherCloudBatch` and `WeatherFrame`.

### F13 - eight comment lines pass the 150-column limit

The house rule: "Any comment line over 150 columns? Break it at a clause boundary."

| columns | line |
| --- | --- |
| 176 | `/// The \`orientationr_t\` marker is therefore moot, not stale: the placeholder \`ViewParms\` still carries no \`ori\` field, and this fn takes the value from the refdef instead.` |
| 175 | `/// \`assets\` is the sim-published \`RenderAssets\`, which carries \`tr.world\`, and \`host\` carries \`mOutside.Cache\`'s engine and collision access plus \`Com_Printf\`'s \`Common\`.` |
| 169 | `/// \`RE_RenderScene\` fills that orientation straight from the scene refdef (\`oracle/codemp/renderer/tr_scene.cpp:848-851\`), so the refdef gives the identical values.` |
| 168 | `/// The pass draws two-sided. That is faithful, not incidental: Raven sets \`GL_Cull(CT_TWO_SIDED)\` for weather at \`oracle/codemp/renderer/tr_WorldEffects.cpp:1362\`.` |
| 160 | `/// The pass sets its own viewport and scissor from \`view\`, the same values the world pass used, because \`SetViewportAndScissor\` is retired at the CPU site.` |
| 154 | `/// Raven queues the marker once per scene and the backend steps \`Update\` once per marker, so this fn runs once per \`RE_RenderScene\` and the counts match.` |
| 154 | `/// Raven reads \`RDF_NOWORLDMODEL\` off \`tr.refdef\` and \`RDF_SKYBOXPORTAL\` off \`backEnd.refdef\`, two copies that hold different scenes at backend time.` |
| 151 | `// \`Q_irand\` increments \`max\` first, so \`((result * -1999) >> 15) + 2000\` over \`result\` in \`[0, 32767]\` lands uniformly in \`[1, 2000]\`.` |

Two of the eight are the packet's contracted `draw_weather` doc text, which was itself over the limit.

### F14 - four comment blocks are column-wrapped mid-clause

The house rule: "A line break is a semantic act, never a length fix." A break under 150 columns that lands mid-clause is the machine tell the rule bans.

`crates/mp/renderer-gpu/src/frame_exec.rs:656-659`:

```rust
                    // The two inputs `draw_weather` reads are derived, not carried: `R_RotateForViewer` needs the refdef's own
                    // origin and axis, and `R_SetupProjection` needs the visible bounds the world walk grew, which the view
                    // state still holds. Rebuilding both here reproduces that view without a carrier of its own.
```

`crates/mp/renderer-gpu/tests/world_golden.rs:828-830`:

```rust
    // The first pin covers the commands themselves. `CWeatherParticleCloud::Initialize` picks every particle's `mMass` off the
    // C runtime stream, and it runs inside `R_WorldEffectCommand`, so the snow and fog commands take 1060 draws before the
    // second pin. Mass divides the force, so an unpinned draw here gives each particle its own fall rate and the image moves.
```

`crates/mp/renderer-gpu/tests/world_golden.rs:795-796`:

```rust
/// The rig loads no collision world and runs no cgame, so the point cache reads every cell as outside and the zone list falls back to the
/// whole map. Both are rig properties. This golden proves the draw path and byte stability, and it proves nothing about zone or cache behavior.
```

`crates/mp/renderer/src/render_state/weather_frame.rs:38-39`:

```rust
/// This type deliberately carries no view. The positional invariant stands in its place: one weather batch per frame, built from the world
/// scene's refdef, with its `FrameEvent` emitted inside that scene's event span, so the executor draws it under the view that built it.
```

Every one of these breaks lands mid-clause under 150 columns.

### F15 - an antithesis construction

The standing rule bans "It's not X, it's Y" rhetoric.

`crates/mp/renderer-gpu/src/pipeline3d.rs:1571`:

```rust
    /// The pass draws two-sided. That is faithful, not incidental: Raven sets `GL_Cull(CT_TWO_SIDED)` for weather at `oracle/codemp/renderer/tr_WorldEffects.cpp:1362`.
```

Commit `20195f41`'s body repeats it:

> It draws two-sided, which is faithful rather than incidental: Raven sets `GL_Cull(CT_TWO_SIDED)` for weather.

The plain statement, "the pass draws two-sided, because Raven sets `GL_Cull(CT_TWO_SIDED)` at `:1362`", carries the same fact.
The packet's row 12 supplied this sentence, so the lane transcribed a ruled line.
The other "not X" lines in the diff state a value and its wrong alternative ("the alpha ceiling is `mColor[3]`, not 1.0"), which is a fact and not rhetoric.

### F16 - a wrong word in a doc comment

STE requires each word in its approved meaning.

`crates/mp/renderer-gpu/src/pipeline3d.rs:4459`:

```rust
/// Converges one weather billboard vertex into the GPU vertex row, which is where the colour quantizes.
```

The function converts a vertex. It does not converge one.

## 7. The gate battery, re-run

Every gate the packet's commit bundle names was run at the branch tip with the exact invocation the packet gives.
Every run was foreground and serial. No claim in a commit body was trusted.

| gate | invocation | result |
| --- | --- | --- |
| build | `cargo build --workspace` | `Finished dev profile ... in 8.31s`, zero warnings |
| workspace tests | `cargo test --workspace -- --test-threads=1` | exit 0, no failure line |
| world goldens | `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` | `6 passed; 0 failed` in 118.15s |
| scene goldens | `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1` | `11 passed; 0 failed` |
| entity goldens | `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1` | `2 passed; 0 failed` |
| ghoul2 vertex golden | `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |
| hud goldens | `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` | `1 passed; 0 failed; 1 ignored` |
| hud goldens, ignored | `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |

The six world goldens are `golden_world_dlights_duel1`, `golden_world_duel1`, `golden_world_ffa2`, `golden_world_marks_duel1`, `golden_world_subbsp_ffa2`, and `golden_world_weather_ctf2`.
Twenty-two fixtures, all at `CHANNEL_TOLERANCE` zero for the four image suites and a byte compare for the vertex `.bin`.
No `.actual.png` was written by any run, and `git status --porcelain` is empty after every run, so all twenty-one pre-existing fixtures are byte-identical and the new one matches its blessed bytes.

The determinism claim in the reseed Amendment was re-run as well, three isolated runs of the new golden alone:

```
--- run 1
world_weather_ctf2: 2048 weather vertices over 891 world draw calls
test result: ok. 1 passed; 0 failed
--- run 2
world_weather_ctf2: 2048 weather vertices over 891 world draw calls
test result: ok. 1 passed; 0 failed
--- run 3
world_weather_ctf2: 2048 weather vertices over 891 world draw calls
test result: ok. 1 passed; 0 failed
```

2048 weather vertices over 891 world draw calls on every run, which is the exact figure the row-7 bless Amendment recorded.

The lockstep referee was not run. The packet excludes it, and no commit touches `mp_game`, the server, or any `jampded` link-set crate. That exclusion checks out against the file inventory.

## 8. The unverified list

Each of these could not be checked mechanically. None is assumed fine.

1. **The rebuilt view's two matrices.** The executor arm reconstructs `viewParms_t` and runs `R_RotateForViewer` and `R_SetupProjection` over it, with `visBounds` copied from `self.view_state.view.vis_bounds`. The world pass gets its matrices from `R_RenderView`, which also runs `R_SetupFrustum` and stamps `frameSceneNum` and `frameCount`. Whether `world.modelMatrix` and `projectionMatrix` come out bit-identical between the two paths is asserted by nothing. The seven viewport and view fields are provably the same seven the world path fills, so the viewport rect is identical. The matrices are not proven.
2. **`sampler_nearest` and `nearest_filter = true`.** No gate binds them. Row 8 concedes this: the `snow` preset leaves `mFilterMode` at 0, so the ctf2 golden takes the linear path only. Rain in live play is the only check.
3. **Divergence 4.** The per-scene `rdflags` gate is verified by live play alone, by the packet's own statement.
4. **F6's stale-refdef path.** Reachable only when `r_norefresh` is set or before registration, with two scenes on one frame. No test covers it.
5. **`pbr_lit: 0` on the weather flags block.** The golden runs the default cvar snapshot. Behavior with the PBR backend on is not gated.
6. **Zone and cache behavior.** The rig loads no collision world and runs no cgame, so no `misc_weather_zone` reaches `R_AddWeatherZone` and every point tests as outside. Live ctf2 has three brush zones. The packet names this as a rig property and the golden proves nothing about it.
7. **The commit-body gate claims for the intermediate commits.** The battery was re-run at the branch tip only. Each commit's own claim that the twenty-one fixtures were byte-identical at that commit was not independently re-run.
8. **`finished.md`.** Not opened, per the brief. Its content, its deviation list, and its three ruled open gaps are unchecked here.
9. **The DEC-66 determinism amendment.** `docs/decisions.md` is unchanged in this range. The packet says the amendment lands when the step merges, so this is a pending obligation and not a finding.
10. **The image against its defect conditions.** The user blessed the PNG on 2026-08-31. The bytes at `HEAD` hash to the blessed SHA-256, so the blessed image is the committed image. No independent eyes-on judgement was made here.
11. **The `pk3` asset set.** The goldens ran against whatever `JKA_BASEPATH` resolves to on this machine. A different retail asset set could move the image.

---

# Fix round, 2026-08-31

Four new commits on `gh54-step-001-weather`, walked one at a time, every hunk.
`finished.md` was not opened.
F12 stays rejected and is not re-flagged.

Eight findings, FR1 through FR8. Nothing in the original sixteen survives except one residual line of F14.
This report approves nothing.

## What each fix closes

### F6, closed. `ecbb586f`

Both arms now read, byte-identical from the `SAFETY` line down (`cl_cgame.rs:2240-2274`, `cl_ui.rs:1366-1400`):

```rust
        let events_before = re.frame_data.events.len();
        RE_RenderScene(
```

```rust
        let scene_refdef = match re.frame_data.events.last() {
            Some(FrameEvent::RenderScene { refdef, .. })
                if re.frame_data.events.len() > events_before =>
            {
                Some(refdef.clone())
            }
            _ => None,
        };
```

The semantics were checked against the oracle by exhausting every exit from the port's `RE_RenderScene`.

- `if !assets.registered { return; }` (`tr_scene.rs:1186-1188`). No push. `len() == events_before`. Guard fails. Zero steps.
- `if common.cvar(cvars.r_norefresh).integer != 0 { return; }` (`tr_scene.rs:1200-1202`). No push. Guard fails. Zero steps.
- `com_error(errorParm_t::ERR_DROP, ...)` on a null worldmodel. `com_error` is declared `pub fn com_error(level: errorParm_t, msg: String) -> !` (`crates/mp/engine/qcommon/src/common/error.rs:63`), so it diverges and the arm is never re-entered. Zero steps, which is what Raven's `Com_Error` longjmp gives.
- Otherwise the fn runs to its end. `R_AddDecals` may push `AddPolyToScene` events (`tr_scene.rs:307`), and the unconditional `RenderScene` push at `tr_scene.rs:1327` is the final statement with no return between the two.

So the count grows if and only if this call reached the seal, and when it did the last event is this call's own `RenderScene`.
Under `r_norefresh` weather steps zero times, which matches `oracle/codemp/renderer/tr_scene.cpp:868` sitting below both of Raven's early returns.
The guard is correct and complete.

### F5, closed. `ecbb586f`

The site note trims to rule 19's two-line cap and names the divergence (`world_effects.rs:1702-1704`):

```rust
            // Divergence 5: Raven reads `bmodels[0]` with no length test, so a world with no submodel reads out of bounds.
            // §19 picks `None`, which leaves the cache unbuilt rather than panicking.
            // Source: oracle/codemp/renderer/tr_WorldEffects.cpp:562,1546
```

`:562` is the oracle's own unguarded `AddWeatherZone(tr.world->bmodels[0].bounds[0], tr.world->bmodels[0].bounds[1]);`, verified at that line. The expression itself is unchanged, as the disposition ruled.

### F8, closed. `23e8f64c`

`create_sampler`'s doc block moves back below `create_nearest_sampler` (`gpu_images.rs:574-578`), so each function carries its own doc and its own `Source:` cite. Neither is bare and neither carries two cites.

### F9, closed and then some. `23e8f64c`

Every touched file is back to the rustfmt hunk count it had at `1c791387`, measured with `rustfmt --edition 2021 --check`:

| file | base | head |
| --- | --- | --- |
| `world_effects.rs` | 0 | 0 |
| `tr_cmds.rs` | 3 | 3 |
| `frame_exec.rs` | 10 | 10 |
| `pipeline3d.rs` | 15 | 15 |
| `gpu_images.rs` | 1 | 1 |
| `cl_cgame.rs` | 9 | 9 |
| `cl_ui.rs` | 8 | 8 |
| `world_golden.rs` | 7 | 7 |
| `tr_backend.rs` | 3 | 3 |
| `tr_scene.rs` | 2 | 2 |

The two new files, `weather_frame.rs` and `frame_event.rs`, are clean.
The fix restored five sites where the finding named four. See FR4.

### F10, closed. `23e8f64c`

```rust
            Warned::Other => {
                "skips an automap or world-effect-command event, and skips a weather batch that no scene preceded"
            }
```

Checked against both call sites. `frame_exec.rs:755-758` covers `FrameEvent::WorldEffectCommand(_) | FrameEvent::AutomapElevAdj(_)`, and `frame_exec.rs:694` covers the weather-no-scene case. The message now names exactly what the slot covers, and nothing it claims is false.

### F11, closed. `d5296657`

`grep '—'` over every line the whole lane adds to a `.rs` file returns nothing. Both em dashes are plain hyphens.

### F13, closed. `d5296657`

`awk 'length > 150'` over every added `.rs` line returns nothing. The worst added comment line is now 142 columns.

### F14, closed at three of four sites. `d5296657`

`weather_frame.rs:38-40`, `world_effects.rs:1139-1142` and `:1651-1652`, `frame_exec.rs:658-661`, `world_golden.rs:831-834`, and the trap-arm block `ecbb586f` introduced all re-break at sentence ends or at one clause boundary where the joined sentence passes 150. Measured joins: 232, 169, 154, all over the cap, so each break is required.
One site does not clear the test. See FR3.

### F15, closed. `d5296657`

```rust
    /// The pass draws two-sided, which Raven asks for at `oracle/codemp/renderer/tr_WorldEffects.cpp:1362`.
```

The sentence states the fact and denies no alternative. The packet's own contracted text was corrected to match in `4258fe82`, so the source and the contract now agree.

### F16, closed. `23e8f64c`

`/// Converts one weather billboard vertex into the GPU vertex row` (`pipeline3d.rs:4459`).

### F1, F2, F3, F4, F7 - ruled, not code

`4258fe82` corrects the branch name in the write scopes and in the disposition, and rewrites the contracted `draw_weather` doc text. Both verified in the diff. The remaining rows are recorded verdicts with no code. See FR1 for a defect in that record.

## `d5296657` is comments-only

Proven by diff inspection, not by assertion. Sixty-five lines change in the commit's `.rs` diff at `--unified=0`. Every one begins with `///`, `//!`, or `//`:

```
git show d5296657 --unified=0 -- '*.rs' | grep -E '^[-+]' | grep -v -e '^+++' -e '^---' \
  | grep -vE '^[-+][[:space:]]*(///|//!|//)'
```

returns nothing.

The same test over the whole fix round (`f7ffe366..HEAD`) leaves exactly three behavioral changes: the F6 guard in both arms, the `Warned::Other` string literal, and four rustfmt restores. Nothing else in the fix round touches a code token.

## Findings

### FR1 - the walk Amendment records the wrong items under F4 and F7

`4258fe82` appends, at item 5:

> **F4 and F7 stand as confessed and ruled.** No action. F4 is the `weather_bind_group` layout parameter, already an Amendment of 2026-08-31. F7 is the double reseed, already an Amendment of the same date.

The vet's F4 is the commit-3 file-list narrowing: `crates/mp/renderer/src/tr_surfacesprites.rs` is in commit 3's own bundle row and is untouched across the whole range.
The vet's F7 is the three unplanned commits `0dfed259`, `b7d9f27d` and `51183d1b`, each a `packet.md` amendment outside the eight-commit bundle.

The two descriptions in the Amendment are `finished.md`'s deviation list, not the vet's findings. The verdict "no action" may be the right verdict for both real findings, but the packet now names the wrong two items, and a later reader will take F4 for the layout parameter.

### FR2 - the trap arms' SAFETY comment is separated from the `unsafe` it justifies

Before `ecbb586f` the comment sat directly above its call. It now sits ten lines above (`cl_cgame.rs:2240-2250`):

```rust
        // SAFETY: `VMA(1)` is the module's `refdef_t` (porting-rules §D11).
        // Raven queues the weather command as the last act of a scene, so the step runs here.
        // The count taken before the call is what makes the step follow this scene and no other.
        // `RE_RenderScene` returns before it pushes anything under `r_norefresh`, and the last event is then an earlier scene's.
        // A grown count plus a `RenderScene` tail therefore means this call sealed that event.
        // Weather steps zero times under `r_norefresh`, which is what retail does.
        // The backend never reaches `RB_RenderWorldEffects` on such a frame.
        // Source: oracle/codemp/renderer/tr_scene.cpp:868, crates/mp/renderer/src/tr_scene.rs:1200-1202
        let events_before = re.frame_data.events.len();
        RE_RenderScene(
            unsafe { &*fd },
```

`cl_ui.rs:1366-1376` is identical. The weather block belongs below `let events_before` or above the SAFETY line, so the SAFETY note keeps its subject.
This is the same class as F8, and the fix round introduced it.

### FR3 - one F14 site still breaks on width alone

`crates/mp/renderer-gpu/tests/world_golden.rs:796-797`:

```rust
/// The rig loads no collision world and runs no cgame, so the point cache reads every cell as outside,
/// and the zone list falls back to the whole map.
```

The house rule's own test: "join the broken lines back together - if the break sits under 150 and width was its only reason, the break is wrong."
Joined, the sentence is exactly 150 columns:

```
/// The rig loads no collision world and runs no cgame, so the point cache reads every cell as outside, and the zone list falls back to the whole map.
```

150 is at the cap and legal on one line, so width was the break's only reason.
This is borderline by one column. The other three F14 sites join to 232, 169 and 154 and are correctly broken.

### FR4 - `23e8f64c`'s body undercounts its own diff

The body says:

> F9: the four rustfmt drifts this lane introduced are restored. Three are in `world_effects.rs`: the import order in the `native_math::qmath` group, one wrapped assignment in `RB_RenderWorldEffects`, and one wrapped call in the respawn test. The fourth is the `tr_cmds` import placement in `world_golden.rs`.

The diff restores a fifth, in `frame_exec.rs`:

```rust
-                            stats.world.weather_vertices += self.pipeline3d.draw_weather(
-                                gpu,
-                                target,
-                                weather,
-                                &view,
-                                gpu_images,
-                            );
+                            stats.world.weather_vertices += self
+                                .pipeline3d
+                                .draw_weather(gpu, target, weather, &view, gpu_images);
```

The extra restore is correct and welcome. The body is wrong about its own scope.
The vet's own F9 also missed this site, and that error is recorded here against the first-round report.

### FR5 - the F9 restore reintroduces two house-syntax violations

House style: "When a call wraps, EVERY parameter gets its own line - no packing several per line. Short calls stay inline. This binds test code the same as source code."

`world_effects.rs:2382-2384`:

```rust
        cloud.Update(
            &mut rng, &outside, [0.0; 3], VIEW_AXIS, false, [0.0; 3], 0.02,
        );
```

`frame_exec.rs:688-690`:

```rust
                            stats.world.weather_vertices += self
                                .pipeline3d
                                .draw_weather(gpu, target, weather, &view, gpu_images);
```

Both wrap and both pack. Both forms are rustfmt's own output, so they are the direct and unavoidable consequence of the ruled F9 disposition: rustfmt and the house syntax rule disagree here, and the disposition chose rustfmt.
Named so the session can rule it dead on arrival rather than have it resurface.

### FR6 - the fixture's helper no longer matches the trap arm, and its doc says it does

`crates/mp/renderer-gpu/tests/world_golden.rs:756-765`:

```rust
/// Steps the weather once for the scene `frame_data` just recorded, and appends the batch event.
///
/// This is the trap arm's own shape: take the refdef `RE_RenderScene` sealed off the last event, then call `RE_RenderWorldEffects`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:868`
fn step_weather(host: &mut UiHost, frame_data: &mut FrameData) {
    let scene_refdef = match frame_data.events.last() {
        Some(FrameEvent::RenderScene { refdef, .. }) => Some(refdef.clone()),
        _ => None,
    };
```

`ecbb586f` gave the trap arm a count guard and left the helper on the old unguarded match, so the doc's claim is stale.
The consequence matters more than the doc: the F6 guard has no test. The fixture calls `record_scene` immediately before `step_weather`, so the helper always finds a fresh event, and no test in the workspace exercises the `r_norefresh` or unregistered path the guard exists for.

### FR7 - two semicolons in prose in the walk Amendment

House style takes its punctuation from ASD-STE100, which does not use the semicolon to join clauses in prose.
`.claude/packets/54/step-001/packet.md`, the walk Amendment item 4:

> F2: the three private helpers `pipeline3d.rs` gained are named and accepted - `WeatherRun`, the resolved per-cloud GPU form; `world_vertex_from_weather`; and `quantize_color_channel`, which is row 10's "one named site" for the colour rounding.

The rest of the added packet text is clean: no em dash, no other semicolon.

### FR8 - a suppressed weather batch is invisible in the stats

Pre-existing to the fix round. Commit `20195f41` introduced it and the first round did not raise it, so it is named now.

`frame_exec.rs:755-758`, the other `Warned::Other` site:

```rust
                FrameEvent::WorldEffectCommand(_) | FrameEvent::AutomapElevAdj(_) => {
                    stats.skipped_other += 1;
                    self.warn_once(Warned::Other);
                }
```

`frame_exec.rs:692-694`, the weather site:

```rust
                        None => {
                            // No scene came ahead of this event, so there is no view to draw it under.
                            self.warn_once(Warned::Other);
                        }
```

The weather arm does not bump `stats.skipped_other`. `warn_once` is one-shot per slot, so once any `Other` warn has fired the weather skip prints nothing either, and the drop is then observable nowhere.
The F10 disposition scoped the fix to the message text, so this is outside that row.

## The gate battery, re-run at the head

Every gate the packet names, with the packet's exact invocation, foreground and serial. No commit-body claim trusted.

| gate | result |
| --- | --- |
| `cargo build --workspace` | `Finished dev profile ... in 6.68s`, zero warnings |
| `cargo test --workspace -- --test-threads=1` | exit 0, 137 `test result: ok` lines, no failure |
| `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` | `6 passed; 0 failed` in 118.62s |
| `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1` | `11 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1` | `2 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` | `1 passed; 0 failed; 1 ignored` |
| `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |

Twenty-two fixtures, all green, the four image suites at `CHANNEL_TOLERANCE` zero and the ghoul2 suite on a byte compare.
`git diff f7ffe366..HEAD -- crates/mp/renderer-gpu/tests/goldens/` is empty, so the fix round moved no fixture.
`crates/mp/renderer-gpu/tests/goldens/world_weather_ctf2.png` hashes to `4044611f92816317b3c5f94a03bd9c60f04bb61a5906b69771aff9851696aa05`, the blessed value.
No `.actual.png` written by any run, and the working tree is clean after all of them apart from this untracked file.

One isolated re-run of the weather golden after the fix round:

```
world_weather_ctf2: 2048 weather vertices over 891 world draw calls
test result: ok. 1 passed; 0 failed
```

2048 vertices over 891 world draw calls, unchanged from before the fix round, which is expected: the fixture drives `step_weather` and not the trap arms.

The lockstep referee stays off the list. The fix round touches `crates/mp/engine/client/`, which is not a `jampded` link-set crate, and no `mp_game` or server file.

## The four commit messages

| commit | subject | trailers | signature |
| --- | --- | --- | --- |
| `4258fe82` | `process(gh#54 s001): the lane-review walk amendment` | none | `N` |
| `ecbb586f` | `fix(gh#54 s001): the weather step follows its own scene` | none | `N` |
| `23e8f64c` | `fix(gh#54 s001): the lane-review mechanical batch` | none | `N` |
| `d5296657` | `style(gh#54 s001): the comment restyle batch` | none | `N` |

Every subject is a heading noun phrase with the ticket and step prefix.
Every body is unwrapped STE prose. No em dash, no semicolon, no `Co-Authored-By`, no generated-with footer, no line that parses as a trailer.
The three code commits each carry a gate paragraph that opens with prose, "The gates all pass."
`4258fe82` carries no gate paragraph, which matches the other packet-only `process` commits in the bundle.
`23e8f64c`'s body is inaccurate about its own diff. See FR4.

## Unverified, fix round

1. **The F6 guard has no test.** See FR6. The `r_norefresh` and unregistered paths are reasoned through, not exercised.
2. **The stale-refdef behavior before the fix.** No fixture reproduced the old defect, so the fix is verified by reading and by case exhaustion, not by a red-to-green test.
3. **`Warned::Other`'s one-shot suppression.** See FR8. Not exercised by any test.
4. Every item on the first round's unverified list still stands, except item 4, which F6 closes.

---

# Final pass, 2026-08-31

Two new commits, `d4328541` and `45041154`, walked hunk by hunk.
`finished.md` was opened only at its `## Open gaps` section, to verify the FR6 record landed as ruled. Nothing else was read from it.

One finding, FP1. All eight fix-round findings close, seven of them cleanly and FR8 with a doc that did not follow the code.

## What each fix closes

### FR1, closed. `d4328541`

The walk Amendment's item 5 now reads:

> **F4 and F7 stand as confessed and ruled.** No action. F4 is the commit-3 file-list narrowing, where `tr_surfacesprites.rs` needed no edit because neither function that takes a `&WindZoneState` has a caller. F7 is the three Amendment commits the lane landed mid-run, which the finished file already records.

Both descriptions now match the vet's own F4 and F7, and both are factually correct against the range: `crates/mp/renderer/src/tr_surfacesprites.rs` is untouched from `1c791387` to the head, and `0dfed259`, `b7d9f27d` and `51183d1b` are the three mid-run packet commits. The verdicts are unchanged, which is what the disposition ruled.

### FR7, closed. `d4328541`

Item 4's two semicolons become sentence breaks:

> `WeatherRun` is the resolved per-cloud GPU form. `world_vertex_from_weather` converts one billboard vertex into the GPU vertex row. `quantize_color_channel` is row 10's "one named site" for the colour rounding.

`grep ';'` over every line the whole lane adds to `packet.md` now returns nothing, and so does `grep '—'`.

### FR4 and FR5, recorded. `d4328541`

The new dated Amendment carries both. Item 2 states the true five-site count and names the fifth as the `draw_weather` call wrap in `frame_exec.rs`, with the landed body standing by precedent. Item 3 records the FR5 rejection and states the boundary: comments follow the house line-break rule, code follows rustfmt. Both match the ruled dispositions.

### FR2, closed. `45041154`

The safety line sits beside the `unsafe` it justifies again (`cl_cgame.rs:2247-2250`):

```rust
        let events_before = re.frame_data.events.len();
        // SAFETY: `VMA(1)` is the module's `refdef_t` (porting-rules §D11).
        RE_RenderScene(
            unsafe { &*fd },
```

`cl_ui.rs:1373-1376` is identical. The weather block moves above `let events_before`, the statement it explains, so both comments now sit directly above their subjects.

### FR3, closed. `45041154`

`crates/mp/renderer-gpu/tests/world_golden.rs:799`, measured:

```
/// The rig loads no collision world and runs no cgame, so the point cache reads every cell as outside, and the zone list falls back to the whole map.
```

150 columns exactly, which the inclusive cap allows on one line. The break that had width as its only reason is gone.

### FR6, closed. `45041154`

`world_golden.rs:756-763`:

```rust
/// Steps the weather once for the scene `frame_data` just recorded, and appends the batch event.
///
/// This takes the refdef `RE_RenderScene` sealed off the last event and calls `RE_RenderWorldEffects`, which is the trap arm's shape minus its guard.
/// The arms also compare the event count across the call, so a scene that pushed nothing cannot hand the weather step an earlier scene's refdef.
/// The fixture needs no such guard, because it holds `r_norefresh` at its default and every step records a scene.
/// The guard therefore has no coverage here, and live play verifies it.
```

The false claim is gone and every new claim checks out. `r_norefresh` registers with default `"0"` and `CVAR_CHEAT` (`crates/mp/renderer/src/tr_init.rs:835`), and the rig never sets it. `step_weather` panics when no scene precedes it, and the fixture passes, so every step records a scene. The four lines measure 150, 145, 114 and 72 columns, all at or under the cap, each a whole sentence.

The gap record landed in `finished.md` at line 103, inside `## Open gaps`, directly after the divergence-4 gap at 101 and before the DEC-66 gap at 105. That is the ruled placement:

> **The trap arms' scene guard has no automated coverage.** [...] `RE_RenderScene` pushes nothing under `r_norefresh` (`crates/mp/renderer/src/tr_scene.rs:1200-1202`), which is the one case the guard exists for. The weather fixture holds `r_norefresh` at its default and records a scene on every step, so the guard never takes its false branch there. This is the same class as divergence 4 above, and live play verifies both.

The cite at `tr_scene.rs:1200-1202` still points at the `r_norefresh` return.

### FR8, closed in code. `45041154`

`frame_exec.rs:692-697`:

```rust
                        None => {
                            // No scene came ahead of this event, so there is no view to draw it under.
                            // The counter matches the sibling arm that shares this warning slot.
                            stats.skipped_other += 1;
                            self.warn_once(Warned::Other);
                        }
```

This is the sibling arm's shape at `frame_exec.rs:757-760`, counter first and warning second. It is the one code change in either commit.
No test reads `skipped_other`, and `FrameStats::skipped_events` has no caller in the workspace, so the change is inert for every gate. See FP1 for what the fix left behind.

## `45041154` carries exactly one code line

Proven by diff, not asserted. `git show 45041154 --unified=0 -- '*.rs'` with every comment line filtered out leaves a single line:

```
+                            stats.skipped_other += 1;
```

Which matches the commit body's own claim, "This is the one code change in the commit."
`d4328541` touches `packet.md` alone and no source file.

## The finding

### FP1 - `skipped_other`'s field doc does not name the third thing it now counts

`crates/mp/renderer-gpu/src/frame_exec.rs:121-122`:

```rust
    /// Everything else skipped (world-effect commands, automap elevation).
    pub skipped_other: u32,
```

The FR8 fix gave the field a third contributor, a weather batch that no scene preceded, and left the doc naming two.
The sibling text was already corrected for exactly this reason. `Warned::Other`'s message, reworded in `23e8f64c`, names all three (`frame_exec.rs:180-182`):

```rust
            Warned::Other => {
                "skips an automap or world-effect-command event, and skips a weather batch that no scene preceded"
            }
```

So the field doc is now the one place in the file that undercounts what the slot covers. This is the same class as F10 and it is newly introduced by the fix round, in the one commit that changed code.
A one-line doc edit closes it.

## Everything else, swept clean at the head

Over every line the whole lane adds to a `.rs` file, from `1c791387` to the head:

| check | result |
| --- | --- |
| em dash | none |
| line over 150 columns | none |
| `todo!` / `unimplemented!` / `TODO` / `extern` / `format!` | none |
| `use` inside a function body | only `use super::*;` in `#[cfg(test)] mod tests`, which the porting rules exempt |
| pet vocabulary other than the rejected "seam" | none |
| antithesis construction | none |
| semicolon in comment prose | none, the only hits are inside code spans such as `[f32; 3]` |
| comment line broken mid-clause | none, the only hits are `Source:` cite lines, which are data |

Over every line the lane adds to `packet.md`: no em dash, no semicolon.

rustfmt drift against `1c791387`, per touched file, all unchanged:

| file | base | head |
| --- | --- | --- |
| `world_effects.rs` | 0 | 0 |
| `frame_exec.rs` | 10 | 10 |
| `cl_cgame.rs` | 9 | 9 |
| `cl_ui.rs` | 8 | 8 |
| `world_golden.rs` | 7 | 7 |
| `gpu_images.rs` | 1 | 1 |
| `pipeline3d.rs` | 15 | 15 |
| `tr_cmds.rs` | 3 | 3 |
| `tr_backend.rs` | 3 | 3 |
| `tr_scene.rs` | 2 | 2 |

## The two commit messages

| commit | subject | trailers | signature |
| --- | --- | --- | --- |
| `d4328541` | `process(gh#54 s001): the fix-round record corrections` | none | `N` |
| `45041154` | `fix(gh#54 s001): the fix-round mechanical batch` | none | `N` |

Both subjects are heading noun phrases with the ticket and step prefix. Both bodies are unwrapped STE prose with no em dash, no semicolon, no `Co-Authored-By`, no generated-with footer, and no line that parses as a trailer.
`45041154` carries a gate paragraph opening with prose, "The gates all pass." `d4328541` is packet-only and carries none, which matches every other `process` commit in the range.
Both bodies are accurate about their own diffs this time.

## The gate battery, re-run at the head

| gate | result |
| --- | --- |
| `cargo build --workspace` | `Finished dev profile ... in 6.57s`, zero warnings |
| `cargo test --workspace -- --test-threads=1` | exit 0, 137 `test result: ok` lines, no failure |
| `cargo test -p mp_renderer_gpu --test world_golden -- --ignored --test-threads=1` | `6 passed; 0 failed` in 117.90s |
| `cargo test -p mp_renderer_gpu --test scene_golden -- --test-threads=1` | `11 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test entity_golden -- --ignored --test-threads=1` | `2 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test ghoul2_vertex_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |
| `cargo test -p mp_renderer_gpu --test hud_golden -- --test-threads=1` | `1 passed; 0 failed; 1 ignored` |
| `cargo test -p mp_renderer_gpu --test hud_golden -- --ignored --test-threads=1` | `1 passed; 0 failed` |

Twenty-two fixtures, all green, the four image suites at `CHANNEL_TOLERANCE` zero and the ghoul2 suite on a byte compare.
`git diff 1c791387..HEAD -- crates/mp/renderer-gpu/tests/goldens/` shows the one new PNG and nothing else, so all twenty-one pre-existing fixtures are byte-identical across the entire lane.
`crates/mp/renderer-gpu/tests/goldens/world_weather_ctf2.png` hashes to `4044611f92816317b3c5f94a03bd9c60f04bb61a5906b69771aff9851696aa05`, the blessed value.
No `.actual.png` written by any run. The working tree is clean apart from this untracked file.

One isolated re-run of the weather golden:

```
world_weather_ctf2: 2048 weather vertices over 891 world draw calls
test result: ok. 1 passed; 0 failed
```

Unchanged from every earlier round.

The lockstep referee stays off the list. No commit in the lane touches `mp_game`, the server, or any `jampded` link-set crate.

## Unverified, final pass

1. **The FR8 counter.** `skipped_other` has no reader in the workspace and `skipped_events` has no caller, so the bump is exercised by nothing.
2. **The scene guard's false branch.** Still uncovered by any test, now recorded as an open gap in `finished.md`. Verified by reading and by case exhaustion, not by a test.
3. Every item on the earlier unverified lists still stands, except the two the fix rounds closed.
