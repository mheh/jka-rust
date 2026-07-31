//! `frame_exec` — walks one frame's [`FrameData`] event stream and turns it
//! into GPU work (R4a backend #1, wave 2).
//!
//! This is the render side of the seam `mp_renderer`'s `FrameData`/`FrameEvent`
//! pair defines: the sim/VM side appends events in trap-call order, this side
//! replays them in that exact order. Order is the whole contract — 2D
//! compositing has no depth test (`RB_SetGL2D` sets `GLS_DEPTHTEST_DISABLE`),
//! so "later wins" is the only correctness rule there is.
//!
//! **Staging (this wave): single-threaded first light.** The harness builds a
//! `FrameData` and executes it inline, the same frame. DEC-37 ruling 2 puts
//! this executor on a dedicated render thread eventually, and the signature
//! below is already shaped for that split: [`FrameExecutor::execute_frame`]
//! takes the frame stream by shared reference and otherwise touches only
//! render-thread-owned state ([`Gpu`], the batch, the pipeline cache). Handing
//! it a `FrameData` that arrived over a channel instead of one built two lines
//! earlier changes nothing about this file. The sim/render thread split is a
//! later R4 slice.
//!
//! Wave 2 renders `SetColor`, `DrawStretchPic` and `DrawString`. The
//! rotate-pic pair and the whole scene-composition half of the enum are
//! counted and skipped, with one `eprintln!` the first time each kind is
//! seen. Skipping, not panicking: a real `ui` frame emits scene events from
//! day one, and a loud-but-live frame loop is what makes the next slices
//! bisectable.
//!
//! **Resolving a draw.** A `DrawStretchPic` carries the `ShaderHandle` the
//! trap layer resolved; this side walks it the rest of the way, one layered
//! quad per active stage, exactly as `RB_StageIteratorGeneric` issues one pass
//! per active stage: [`crate::stage2d`] computes that stage's colour
//! (`ComputeColors`), its texture coordinates (`ComputeTexCoords`) and its
//! bound image (`R_BindAnimatedImage`), and the stage's own `stateBits` become
//! the blend mode — `GL_State(pStage->stateBits)`. A shader whose stages all
//! died in `FinishShader` (a `videoMap` with no cinematic behind it, a stage
//! whose image failed to load) draws nothing, which is what the oracle's
//! zero-iteration loop does.
//!
//! The white texel remains the fallback for the one case that is *not* a
//! zero-pass shader — an unregistered handle — and for an active stage whose
//! image has not been uploaded. Both log once per process rather than dropping
//! the quad.

use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::qfiles::font_style::SET_MASK;
use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::frame_state::FrameState;
use mp_renderer::render_state::gpu_resources::GpuResources;
use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::placeholders::{RefEntity, TrRefdef, WorldAsset};
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::renderer_cvars::RendererCvars;
use mp_renderer::render_state::shader_asset::ShaderHandle;
use mp_renderer::tr_font::{layout_font_string, FontDrawItem, FontState, Language_e};
use mp_renderer::tr_image::TrImageState;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_local::fog_t::fog_t;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_local::tr_ref_entity_t::trRefEntity_t;
use mp_renderer::tr_local::tr_refdef_t::trRefdef_t;
use mp_renderer::tr_local::view_parms_t::viewParms_t;
use mp_renderer::tr_main::{
    tr_ref_entity_from_ref_entity, DrawSurf, R_RenderView, SurfaceGeometry, TrMainScratch,
};
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_noise::NoiseState;
use wgpu::TextureView;

use crate::blend::{blend_state_from_gls, GLS_2D_DEFAULT};
use crate::gpu::Gpu;
use crate::gpu_images::GpuImages;
use crate::pipeline2d::{Pipeline2d, QuadBatch, Rect, UvRect};
use crate::pipeline3d::{Pipeline3d, WorldGeometry, WorldStats};
use crate::stage2d::{stage_color, stage_image, stage_texcoords, Stage2dWarnings, StageTime};

/// `tr.identityLight` at its default (no overbright) — the colour a frame
/// starts at before any `SetColor`, matching the oracle's white default.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1353`
const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// What one [`FrameExecutor::execute_frame`] call did. Cheap to copy, and the
/// counters are what the dev harness (and later a `r_speeds`-style readout)
/// reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStats {
    /// Quads batched — one per active stage of every `DrawStretchPic`'s
    /// shader, plus every glyph a `DrawString` laid out.
    pub quads: u32,
    /// `DrawStretchPic` events whose shader had stages but none active, so no
    /// pass was drawn (the oracle's zero-iteration `RB_StageIteratorGeneric`).
    pub zero_pass_pics: u32,
    /// `SetColor` events applied.
    pub color_changes: u32,
    /// `draw` calls the batch collapsed to (one per blend+texture run).
    pub draw_calls: u32,
    /// Images uploaded this frame (`pending_uploads` drained at frame start).
    pub images_uploaded: u32,
    /// `DrawString` events laid out into glyph quads.
    pub strings: u32,
    /// Glyph quads those strings produced.
    pub glyphs: u32,
    /// `DrawString` events whose font index resolved to nothing — the string
    /// laid out empty, so nothing was drawn.
    pub skipped_strings: u32,
    /// `DrawRotatePic`/`DrawRotatePic2` events skipped.
    pub skipped_rotate_pics: u32,
    /// Scene-composition events skipped (lights, polys, decals, …) — later
    /// waves of backend #1's world path. `RenderScene`, `ClearScene`, and
    /// `AddRefEntityToScene` are no longer here when a world context is
    /// supplied.
    pub skipped_scene_events: u32,
    /// Everything else skipped (world-effect commands, automap elevation).
    pub skipped_other: u32,
    /// Ref-entities the last `RenderScene` rebuilt into `tr.refdef.entities`
    /// from the accumulated `AddRefEntityToScene` payloads.
    pub entities: u32,
    /// The last `RenderScene` event's world pass result. Stays at its default
    /// when no world context was supplied or no world was drawn.
    pub world: WorldStats,
}

impl FrameStats {
    /// Total events the executor could not render yet.
    pub fn skipped_events(&self) -> u32 {
        self.skipped_strings
            + self.skipped_rotate_pics
            + self.skipped_scene_events
            + self.skipped_other
    }
}

/// The event kinds this backend cannot render yet, plus the resolution
/// failures that fall back to white — tracked so each one logs once per
/// process rather than once per frame.
#[derive(Clone, Copy, Debug)]
enum Warned {
    RotatePic,
    SceneEvent,
    Other,
    /// A `DrawStretchPic`'s shader handle addressed no registered shader.
    UnknownShader,
    /// An active stage's image is missing or not uploaded yet.
    NoStageImage,
    /// A `DrawString`'s font index addressed no loaded font.
    UnknownFont,
    /// A glyph's page shader resolved to no uploaded image.
    NoGlyphImage,
    /// A `RenderScene` arrived before the world geometry was uploaded.
    NoWorldGeometry,
}

impl Warned {
    const COUNT: usize = 8;

    fn slot(self) -> usize {
        match self {
            Warned::RotatePic => 0,
            Warned::SceneEvent => 1,
            Warned::Other => 2,
            Warned::UnknownShader => 3,
            Warned::NoStageImage => 4,
            Warned::UnknownFont => 5,
            Warned::NoGlyphImage => 6,
            Warned::NoWorldGeometry => 7,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Warned::RotatePic => "skips DrawRotatePic/DrawRotatePic2 — not rendered yet",
            Warned::SceneEvent => {
                "skips scene composition (lights, polys, decals) — not rendered yet"
            }
            Warned::Other => "skips world-effect / automap commands — not rendered yet",
            Warned::UnknownShader => "drew a pic whose shader handle is not registered — white",
            Warned::NoStageImage => "drew a stage whose image is not uploaded — white",
            Warned::UnknownFont => "drew a string with an unloaded font handle — nothing drawn",
            Warned::NoGlyphImage => "drew a glyph whose page shader has no image — white",
            Warned::NoWorldGeometry => "got a RenderScene before the world geometry uploaded",
        }
    }
}

/// The render-side world state one `RenderScene` event needs to run
/// `R_RenderView` and draw. The harness split-borrows its host and engine into
/// this bundle each frame, the same borrows `load_world_and_render` builds.
///
/// The scratch buffers are empty this wave. Polys and lights are later waves,
/// so `dlights`/`fogs` stay empty and the terrain surface is the null-landscape
/// one. The entity list is not passed in: the executor accumulates
/// `AddRefEntityToScene` payloads itself and rebuilds `tr.refdef.entities` from
/// them at `RenderScene` (DEC-50).
pub struct WorldFrame<'a, 'e> {
    pub engine_view: &'a mut EngineHostView<'e>,
    pub assets: &'a mut RenderAssets,
    pub cvars: &'a mut RendererCvars,
    pub frame: &'a mut FrameState,
    /// The engine's Ghoul2 instance owner, threaded into `R_RenderView` so the
    /// entity walk reaches `R_AddGhoulSurfaces`. A caller with no live Ghoul2
    /// state (the golden test, the world spike) threads an empty owned system.
    pub g2: &'a mut Ghoul2System,
    pub gpu_res: &'a mut GpuResources,
    pub models: &'a RenderModels,
    pub land_scape: &'a srfTerrain_t,
    pub land: &'a CmLandScape,
    pub dlights: &'a mut [dlight_t],
    pub fogs: &'a [fog_t],
    pub scratch: &'a mut TrMainScratch,
}

/// Owns the render-thread state one frame's execution needs: the 2D and world
/// pipelines, the uploaded world geometry, the reused 2D batch, and the
/// warn-once flags.
pub struct FrameExecutor {
    pipeline: Pipeline2d,
    pipeline3d: Pipeline3d,
    /// The uploaded world mesh, `None` until [`FrameExecutor::set_world`] runs
    /// after a map load.
    world_geometry: Option<WorldGeometry>,
    batch: QuadBatch,
    /// The frame's accumulated ref-entities, in trap-call order, the render
    /// -side stand-in for `backEndData->entities`. `AddRefEntityToScene`
    /// appends. Each frame's first event replay clears the list, the render
    /// -side `R_ToggleSmpFrame`. The trap caps the count at `TR_WORLDENT`, so
    /// this list never overflows the oracle bound.
    scene_entities: Vec<RefEntity>,
    /// The start of the current scene's window into `scene_entities`, the render
    /// -side `r_firstSceneEntity`. `RenderScene` draws the slice from this index
    /// to the list end, then advances it to the end, so a later scene in the
    /// same frame sees only the entities added after it. `ClearScene` advances
    /// it without drawing, and a frame start resets it to 0.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:56,76,859`
    first_scene_entity: usize,
    warned: [bool; Warned::COUNT],
    stage_warnings: Stage2dWarnings,
}

impl FrameExecutor {
    /// Builds the executor's GPU resources against `gpu`'s device. `images`
    /// supplies the texture bind-group layout the 2D pipeline is built
    /// against.
    pub fn new(gpu: &Gpu, images: &GpuImages) -> FrameExecutor {
        FrameExecutor {
            pipeline: Pipeline2d::new(gpu, images),
            pipeline3d: Pipeline3d::new(gpu),
            world_geometry: None,
            batch: QuadBatch::new(),
            scene_entities: Vec::new(),
            first_scene_entity: 0,
            warned: [false; Warned::COUNT],
            stage_warnings: Stage2dWarnings::default(),
        }
    }

    /// Uploads the loaded world's geometry so the world pass can draw it. Call
    /// once after `RE_LoadWorldMap` and before the first frame that renders a
    /// scene.
    pub fn set_world(&mut self, gpu: &Gpu, world: &WorldAsset) {
        self.world_geometry = Some(WorldGeometry::upload(gpu, world));
    }

    /// Recreates the world depth texture on a window resize.
    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        self.pipeline3d.resize(gpu, width, height);
    }

    /// Replays `frame_data`'s events in order into `target`.
    ///
    /// The colour register is per-frame, not persistent: the oracle's
    /// `RB_SetGL2D` path re-establishes white at the top of every 2D pass, and
    /// `FrameData` is a complete frame, so starting from [`DEFAULT_COLOR`]
    /// keeps a dropped frame from tinting the next one.
    ///
    /// `float_time` is the 2D pass's shader clock in seconds —
    /// `backEnd.refdef.floatTime`, which `RB_SetGL2D` sets from
    /// `ri.Milliseconds()` at the top of every 2D pass. Every animated stage
    /// (`rgbGen wave`, `tcMod scroll`/`rotate`, `animMap`) is driven from it.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:1289-1291`
    #[allow(clippy::too_many_arguments)]
    pub fn execute_frame(
        &mut self,
        gpu: &mut Gpu,
        target: &TextureView,
        frame_data: &FrameData,
        assets: &RenderAssets,
        image_assets: &RenderAssets,
        img_state: &mut TrImageState,
        gpu_images: &mut GpuImages,
        fonts: &mut FontState,
        noise: &NoiseState,
        float_time: f32,
        mut world: Option<&mut WorldFrame>,
    ) -> FrameStats {
        // Two registries by design (A9): shader registration writes the direct
        // `assets` instance, image registration writes the sim-published Arc
        // master (`Arc::make_mut(&mut sim.published)` in `tr_image.rs`), so
        // stage image handles resolve against `image_assets` = the published
        // side. A single-registry caller passes the same reference twice.
        let mut stats = FrameStats::default();
        let mut color = DEFAULT_COLOR;

        // Frame start: every image `R_CreateImage` staged since the last
        // frame becomes a texture, so a shader registered mid-frame is
        // drawable by the time its quad is bound. A world frame keeps its
        // textures (lightmaps, surface diffuse maps) in its own asset registry,
        // so uploads resolve there.
        let upload_assets: &RenderAssets = match world.as_deref() {
            Some(w) => &*w.assets,
            None => image_assets,
        };
        stats.images_uploaded = gpu_images.upload_pending(gpu, img_state, upload_assets) as u32;

        self.batch.clear();

        // Frame start, the render-side `R_ToggleSmpFrame`: reset the whole
        // ref-entity list and the scene window. A frame that renders without a
        // `ClearScene` must not inherit the last frame's entities.
        // Source: oracle/codemp/renderer/tr_scene.cpp:55-56
        self.scene_entities.clear();
        self.first_scene_entity = 0;

        for event in &frame_data.events {
            match event {
                FrameEvent::SetColor(rgba) => {
                    color = *rgba;
                    stats.color_changes += 1;
                }
                FrameEvent::DrawStretchPic {
                    x,
                    y,
                    w,
                    h,
                    s1,
                    t1,
                    s2,
                    t2,
                    shader,
                } => {
                    let drawn = self.draw_stretch_pic(
                        *shader,
                        Rect {
                            x: *x,
                            y: *y,
                            w: *w,
                            h: *h,
                        },
                        UvRect {
                            s1: *s1,
                            t1: *t1,
                            s2: *s2,
                            t2: *t2,
                        },
                        color,
                        assets,
                        gpu_images,
                        noise,
                        float_time,
                    );
                    stats.quads += drawn;
                    if drawn == 0 {
                        stats.zero_pass_pics += 1;
                    }
                }

                FrameEvent::DrawString {
                    ox,
                    oy,
                    text,
                    rgba,
                    set_index,
                    char_limit,
                    scale,
                } => {
                    let drawn = self.draw_string(
                        assets,
                        gpu_images,
                        fonts,
                        &mut color,
                        *ox,
                        *oy,
                        text,
                        *rgba,
                        *set_index,
                        *char_limit,
                        *scale,
                    );
                    if drawn == 0 {
                        stats.skipped_strings += 1;
                    } else {
                        stats.strings += 1;
                        stats.glyphs += drawn;
                        stats.quads += drawn;
                    }
                }
                FrameEvent::DrawRotatePic { .. } | FrameEvent::DrawRotatePic2 { .. } => {
                    stats.skipped_rotate_pics += 1;
                    self.warn_once(Warned::RotatePic);
                }

                FrameEvent::RenderScene {
                    refdef,
                    light_styles,
                    disable_dynamic_light,
                } => {
                    match world.as_deref_mut() {
                        // DEC-50: the render side rebuilds the view and runs
                        // `R_RenderView` itself, then draws the world surfaces.
                        Some(w) => {
                            stats.entities =
                                (self.scene_entities.len() - self.first_scene_entity) as u32;
                            stats.world = self.render_world(
                                gpu,
                                target,
                                w,
                                frame_data,
                                refdef,
                                light_styles,
                                *disable_dynamic_light,
                                gpu_images,
                                noise,
                            );
                        }
                        // No world context, so this frame cannot draw a scene.
                        None => {
                            stats.skipped_scene_events += 1;
                            self.warn_once(Warned::SceneEvent);
                        }
                    }

                    // The next scene in this frame tacks on after this one, so
                    // the window start moves to the current list end.
                    // Source: oracle/codemp/renderer/tr_scene.cpp:859
                    self.first_scene_entity = self.scene_entities.len();
                }

                FrameEvent::AddRefEntityToScene(re) => {
                    // Accumulate the ref-entity for this scene. `RenderScene`
                    // converts the list into `tr.refdef.entities`. The trap
                    // already dropped anything past `TR_WORLDENT`, so no cap is
                    // needed here.
                    self.scene_entities.push(re.clone());
                }

                FrameEvent::ClearScene => {
                    // Move the scene window past every entity added so far,
                    // without drawing. The oracle keeps the buffer and only
                    // advances `r_firstSceneEntity`, so a following scene starts
                    // empty. Polys and lights are later waves.
                    // Source: oracle/codemp/renderer/tr_scene.cpp:76
                    self.first_scene_entity = self.scene_entities.len();
                }

                FrameEvent::ClearDecals
                | FrameEvent::AddPolyToScene { .. }
                | FrameEvent::AddPolysToScene { .. }
                | FrameEvent::AddLightToScene { .. }
                | FrameEvent::AddAdditiveLightToScene { .. }
                | FrameEvent::AddDecalToScene { .. }
                | FrameEvent::SetRangeFog(_)
                | FrameEvent::SetRefractionProp { .. } => {
                    stats.skipped_scene_events += 1;
                    self.warn_once(Warned::SceneEvent);
                }

                FrameEvent::WorldEffectCommand(_) | FrameEvent::AutomapElevAdj(_) => {
                    stats.skipped_other += 1;
                    self.warn_once(Warned::Other);
                }
            }
        }

        stats.draw_calls = self.pipeline.draw(gpu, target, &self.batch, gpu_images);
        stats
    }

    /// Runs one scene's world pass (DEC-50): builds the view parameters from
    /// the `RenderScene` payload, runs `R_RenderView` against the render-side
    /// world assets, then draws the sorted world surfaces. The world pass
    /// clears color and depth, so it is the frame's base pass and the 2D batch
    /// composites over it.
    ///
    /// `R_RenderView` builds `view.projectionMatrix` through its own
    /// `R_SetupProjection` call, which runs after the world walk bounds the
    /// view. This fn reads that matrix straight for the GPU clip transform.
    #[allow(clippy::too_many_arguments)]
    fn render_world<'f>(
        &mut self,
        gpu: &mut Gpu,
        target: &TextureView,
        world: &mut WorldFrame,
        frame_data: &'f FrameData,
        refdef: &TrRefdef,
        light_styles: &[[u8; 4]; MAX_LIGHT_STYLES],
        disable_dynamic_light: bool,
        gpu_images: &GpuImages,
        noise: &NoiseState,
    ) -> WorldStats {
        let Some(geometry) = self.world_geometry.as_ref() else {
            self.warn_once(Warned::NoWorldGeometry);
            return WorldStats::default();
        };

        // Rebuild `tr.refdef.entities` from this scene's window into the
        // accumulated payloads (DEC-50). The window starts at
        // `first_scene_entity`, so a later scene in the same frame sees only the
        // entities added after the last `RenderScene` or `ClearScene`.
        // `R_AddEntitySurfaces` iterates this list, and the brush-model arm
        // appends each inline submodel surface through the world draw-surf path.
        let mut entities: Vec<trRefEntity_t> = self.scene_entities[self.first_scene_entity..]
            .iter()
            .map(tr_ref_entity_from_ref_entity)
            .collect();

        // Build the view parameters from the scene refdef.
        let mut parms = zeroed_view_parms();
        parms.viewportX = refdef.x;
        // The oracle flips y into GL's 0-at-the-bottom space here:
        // `viewportY = glConfig.vidHeight - (refdef.y + refdef.height)`.
        // Nothing reads `viewportY` yet, so we carry the unflipped value until
        // the backend calls `set_viewport` for a sub-viewport.
        //TODO: Port viewportY GL flip
        // Source: oracle/codemp/renderer/tr_scene.cpp:838
        parms.viewportY = refdef.y;
        parms.viewportWidth = refdef.width;
        parms.viewportHeight = refdef.height;
        parms.fovX = refdef.fov_x;
        parms.fovY = refdef.fov_y;
        parms.ori.origin = refdef.view_origin;
        parms.ori.axis = refdef.view_axis;
        parms.pvsOrigin = refdef.view_origin;

        // Install the scene refdef so the world walk reads this frame's
        // areamask, view-cluster mask, and time. `R_MarkLeaves` reads
        // `frame.refdef.areamask`/`areamask_modified`, and the shader clock
        // reads `frame.refdef.time`.
        world.frame.refdef = refdef.clone();
        world.frame.scene_light_styles = *light_styles;

        // Bump the per-scene counters, the render-side stand-in for the
        // oracle's `tr.frameSceneNum++`/`tr.sceneCount++`. Ruling 3 keeps that
        // off the trap-time `RE_RenderScene`, so the render-side driver does it.
        // Source: oracle/codemp/renderer/tr_scene.cpp:829-830
        world.frame.frame_scene_num += 1;
        world.frame.scene_count += 1;
        let frame_scene_num = world.frame.frame_scene_num;

        let refdef_rdflags = refdef.rdflags;
        let fov_x = refdef.fov_x;
        let fov_y = refdef.fov_y;
        let refdef_time = refdef.time;
        // Dlights replay from `AddLightToScene` events in a later wave. None
        // land yet, so the disable decision drops an already-empty set.
        let refdef_num_dlights = if disable_dynamic_light {
            0
        } else {
            world.dlights.len() as i32
        };
        let distance_cull = world.assets.distance_cull;

        // SAFETY: `trRefdef_t` is a frozen `#[repr(C)]` POD (scalars, fixed
        // arrays, and raw pointers whose all-zero value is null).
        // `R_AddTerrainSurfaces` reads `rdflags` and `vieworg` off this struct,
        // so the zeroed value is only correct while the null landscape keeps
        // the terrain walk inert. A live landscape must carry the real refdef.
        let abi_refdef: trRefdef_t = unsafe { core::mem::zeroed() };
        let mut draw_surfs: Vec<DrawSurf<SurfaceGeometry<'f>>> = Vec::new();
        let mut view = zeroed_view_parms();

        R_RenderView(
            &parms,
            frame_scene_num,
            refdef_time,
            &mut view,
            world.engine_view,
            world.assets,
            world.cvars,
            world.frame,
            world.g2,
            world.gpu_res,
            frame_data,
            &abi_refdef,
            refdef_rdflags,
            fov_x,
            fov_y,
            refdef_num_dlights,
            world.dlights,
            world.fogs,
            distance_cull,
            world.land_scape,
            world.land,
            0,
            &mut entities,
            world.scratch,
            world.models,
            &mut draw_surfs,
        );

        self.pipeline3d.draw(
            gpu,
            target,
            &draw_surfs,
            geometry,
            world.assets,
            gpu_images,
            noise,
            refdef.float_time,
            &view,
            &entities,
            world.scratch,
            world.models,
            world.g2,
        )
    }

    /// Draws one `DrawStretchPic` as `RB_StageIteratorGeneric` does: one
    /// layered quad per active stage, in stage order, each with its own image,
    /// its own `GL_State(pStage->stateBits)` blend and its own
    /// `ComputeColors`/`ComputeTexCoords` result. Returns the number of quads
    /// batched.
    ///
    /// Every stage draws the same screen rectangle — the oracle re-tessellates
    /// nothing between passes, it only re-binds and re-issues `tess`'s
    /// geometry — so the layering is pure blend order, which the batch already
    /// preserves (2D has no depth test; later wins).
    ///
    /// Zero active stages means zero passes, which is a shader whose stages all
    /// died in `FinishShader` — a `videoMap` with no cinematic behind it, or a
    /// stage whose image failed to load. `RB_IterateStagesGeneric` breaks on
    /// the first inactive stage, so the oracle draws nothing there and so does
    /// this; the white texel is reserved for a handle that names no shader at
    /// all.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:1953-2231`
    /// (`RB_IterateStagesGeneric`); `oracle/codemp/renderer/tr_backend.cpp:1282-1284`
    /// (`RB_SetGL2D`)
    #[allow(clippy::too_many_arguments)]
    fn draw_stretch_pic(
        &mut self,
        shader: ShaderHandle,
        rect: Rect,
        uv: UvRect,
        color: [f32; 4],
        assets: &RenderAssets,
        gpu_images: &GpuImages,
        noise: &NoiseState,
        float_time: f32,
    ) -> u32 {
        let Some(asset) = assets.shaders.get(shader) else {
            self.warn_once(Warned::UnknownShader);
            self.push_white(rect, uv, color);
            return 1;
        };
        let time = StageTime::new(float_time, asset.time_offset);
        let mut drawn = 0;
        for stage in asset.stages.iter().filter(|stage| stage.active) {
            let bundle = &stage.bundle[0];
            let image = stage_image(bundle, time.shader_time).filter(|image| {
                let uploaded = gpu_images.contains(*image);
                if !uploaded {
                    self.warn_once(Warned::NoStageImage);
                }
                uploaded
            });

            let mut st = uv.corners();
            stage_texcoords(
                bundle,
                &mut st,
                time,
                noise,
                assets,
                &asset.name,
                &mut self.stage_warnings,
            );

            self.batch.push_quad_st(
                rect,
                st,
                stage_color(
                    stage,
                    color,
                    time,
                    noise,
                    assets,
                    &asset.name,
                    &mut self.stage_warnings,
                ),
                blend_state_from_gls(stage.state_bits),
                image,
            );
            drawn += 1;
        }
        drawn
    }

    /// The white-texel fallback quad, drawn in the state `RB_SetGL2D` leaves
    /// the pass in.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:1282-1284`
    fn push_white(&mut self, rect: Rect, uv: UvRect, color: [f32; 4]) {
        self.batch.push_quad(
            rect,
            uv,
            color,
            blend_state_from_gls(GLS_2D_DEFAULT),
            None::<ImageHandle>,
        );
    }

    /// Lays a `DrawString` out into glyph quads and pushes them into the
    /// batch, returning how many landed.
    ///
    /// The layout is `mp_renderer`'s — `tr_font`'s own per-glyph walk, reused
    /// through `layout_font_string` — so advance, kerning, dropshadow,
    /// `^N` colour codes and the `iMaxPixelWidth` clip all behave as
    /// `RE_Font_DrawString` does. The trap records the string whole, so the
    /// per-glyph stretch-pics the oracle emitted inline are re-derived here
    /// instead of arriving as events.
    ///
    /// Each glyph's page shader is a raw `qhandle_t` (`CFontInfo::mShader`),
    /// resolved through `handle_at_slot` — the "slot = index" rule for oracle
    /// code that stores plain ints (DEC-42.2).
    ///
    /// The colour register is left where the layout leaves it, matching
    /// `RE_Font_DrawString`'s own `RE_SetColor` side effects: the oracle does
    /// not restore the caller's colour either.
    ///
    /// Source: `oracle/codemp/renderer/tr_font.cpp:1430-1614`
    //TODO: Port GetFont's SBCS-override / Asian-page resolution at the backend
    // Source: oracle/codemp/renderer/tr_font.cpp:1341-1428
    // `GetFont` needs the whole engine carrier list (filesystem, cvars,
    // shader registration) to substitute an alternate single-byte font or
    // load an Asian glyph page. A backend replaying a recorded frame has
    // none of it, so the event's own font index is used directly — correct
    // for western fonts, which is every font wave 2 can build.
    #[allow(clippy::too_many_arguments)]
    fn draw_string(
        &mut self,
        assets: &RenderAssets,
        gpu_images: &GpuImages,
        fonts: &mut FontState,
        color: &mut [f32; 4],
        ox: i32,
        oy: i32,
        text: &str,
        rgba: [f32; 4],
        set_index: i32,
        char_limit: i32,
        scale: f32,
    ) -> u32 {
        // `text` is Latin-1 on the wire (the seam's `const char *`), and
        // `tr_font` reads it byte-wise for its MBCS decode.
        let bytes = latin1_bytes(text);

        let items = layout_font_string(
            fonts,
            Language_e::eWestern,
            set_index & SET_MASK as i32,
            ox,
            oy,
            &bytes,
            Some(rgba),
            set_index,
            char_limit,
            scale,
        );
        if items.is_empty() {
            self.warn_once(Warned::UnknownFont);
            return 0;
        }

        let blend = blend_state_from_gls(GLS_2D_DEFAULT);
        let mut drawn = 0;
        for item in &items {
            match *item {
                FontDrawItem::Color(rgba) => *color = rgba.unwrap_or(DEFAULT_COLOR),
                FontDrawItem::Glyph(glyph) => {
                    let image = self.glyph_image(glyph.h_shader, assets, gpu_images);
                    self.batch.push_quad(
                        Rect {
                            x: glyph.x,
                            y: glyph.y,
                            w: glyph.w,
                            h: glyph.h,
                        },
                        UvRect {
                            s1: glyph.s1,
                            t1: glyph.t1,
                            s2: glyph.s2,
                            t2: glyph.t2,
                        },
                        *color,
                        blend,
                        image,
                    );
                    drawn += 1;
                }
            }
        }
        drawn
    }

    /// The uploaded texture behind a glyph page's raw shader handle.
    fn glyph_image(
        &mut self,
        h_shader: i32,
        assets: &RenderAssets,
        gpu_images: &GpuImages,
    ) -> Option<ImageHandle> {
        let image = u32::try_from(h_shader)
            .ok()
            .and_then(|slot| assets.shaders.handle_at_slot(slot))
            .and_then(|handle| assets.shaders.get(handle))
            .and_then(|asset| asset.stages.iter().find(|stage| stage.active))
            .and_then(|stage| stage.bundle[0].image)
            .filter(|image| gpu_images.contains(*image));

        if image.is_none() {
            self.warn_once(Warned::NoGlyphImage);
        }
        image
    }

    /// Logs a skipped event kind or a fallback the first time it is seen.
    fn warn_once(&mut self, kind: Warned) {
        let slot = kind.slot();
        if self.warned[slot] {
            return;
        }
        self.warned[slot] = true;
        eprintln!("mp_renderer_gpu: frame_exec {}", kind.describe());
    }
}

/// The seam's `const char *` is Latin-1 (DEC-38's wire-string discipline), so
/// a `char` above U+00FF cannot have come from a trap; it is clamped to `?`
/// rather than dropped, keeping the glyph count equal to the character count.
fn latin1_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' })
        .collect()
}

/// A zeroed `viewParms_t`, the value `Com_Memset(&tr.viewParms, 0, ...)` gives
/// it before per-view setup fills it.
///
/// `viewParms_t` is a frozen `#[repr(C)]` struct of scalars, fixed arrays, and
/// `#[repr(C)]` sub-structs, so an all-zero bit pattern is a valid value.
fn zeroed_view_parms() -> viewParms_t {
    // SAFETY: POD `#[repr(C)]`; see the doc comment.
    unsafe { core::mem::zeroed() }
}
