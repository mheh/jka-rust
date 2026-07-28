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
//! trap layer resolved; this side walks it the rest of the way — shader ->
//! first active stage -> `bundle[0].image` -> uploaded texture — and takes
//! that stage's `stateBits` as the blend mode, which is exactly what
//! `RB_StageIteratorGeneric`'s `GL_State(pStage->stateBits)` did. Every link
//! is fallible (an unregistered shader, a stage-less default shader, an image
//! whose upload has not run) and every failure degrades to the white texel
//! with a once-per-kind log rather than dropping the quad.

use mp_engine_qcommon::qfiles::font_style::SET_MASK;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::shader_asset::ShaderHandle;
use mp_renderer::tr_font::{layout_font_string, FontDrawItem, FontState, Language_e};
use mp_renderer::tr_image::TrImageState;
use wgpu::{BlendState, TextureView};

use crate::blend::{blend_state_from_gls, GLS_2D_DEFAULT};
use crate::gpu::Gpu;
use crate::gpu_images::GpuImages;
use crate::pipeline2d::{Pipeline2d, QuadBatch, Rect, UvRect};

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
    /// Quads batched — `DrawStretchPic` events plus every glyph a
    /// `DrawString` laid out.
    pub quads: u32,
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
    /// Scene-composition events skipped (`AddRefEntityToScene`,
    /// `RenderScene`, lights, polys, decals, …) — backend #1's world path.
    pub skipped_scene_events: u32,
    /// Everything else skipped (world-effect commands, automap elevation).
    pub skipped_other: u32,
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
    /// A shader resolved, but no active stage named an uploaded image.
    NoStageImage,
    /// A `DrawString`'s font index addressed no loaded font.
    UnknownFont,
    /// A glyph's page shader resolved to no uploaded image.
    NoGlyphImage,
}

impl Warned {
    const COUNT: usize = 7;

    fn slot(self) -> usize {
        match self {
            Warned::RotatePic => 0,
            Warned::SceneEvent => 1,
            Warned::Other => 2,
            Warned::UnknownShader => 3,
            Warned::NoStageImage => 4,
            Warned::UnknownFont => 5,
            Warned::NoGlyphImage => 6,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Warned::RotatePic => "skips DrawRotatePic/DrawRotatePic2 — not rendered yet",
            Warned::SceneEvent => {
                "skips scene composition (RenderScene and friends) — not rendered yet"
            }
            Warned::Other => "skips world-effect / automap commands — not rendered yet",
            Warned::UnknownShader => "drew a pic whose shader handle is not registered — white",
            Warned::NoStageImage => "drew a pic whose shader has no uploaded stage image — white",
            Warned::UnknownFont => "drew a string with an unloaded font handle — nothing drawn",
            Warned::NoGlyphImage => "drew a glyph whose page shader has no image — white",
        }
    }
}

/// Owns the render-thread state one frame's execution needs: the 2D pipeline,
/// the reused geometry batch, and the warn-once flags.
pub struct FrameExecutor {
    pipeline: Pipeline2d,
    batch: QuadBatch,
    warned: [bool; Warned::COUNT],
}

impl FrameExecutor {
    /// Builds the executor's GPU resources against `gpu`'s device. `images`
    /// supplies the texture bind-group layout the 2D pipeline is built
    /// against.
    pub fn new(gpu: &Gpu, images: &GpuImages) -> FrameExecutor {
        FrameExecutor {
            pipeline: Pipeline2d::new(gpu, images),
            batch: QuadBatch::new(),
            warned: [false; Warned::COUNT],
        }
    }

    /// Replays `frame_data`'s events in order into `target`.
    ///
    /// The colour register is per-frame, not persistent: the oracle's
    /// `RB_SetGL2D` path re-establishes white at the top of every 2D pass, and
    /// `FrameData` is a complete frame, so starting from [`DEFAULT_COLOR`]
    /// keeps a dropped frame from tinting the next one.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_frame(
        &mut self,
        gpu: &mut Gpu,
        target: &TextureView,
        frame_data: &FrameData,
        assets: &RenderAssets,
        img_state: &mut TrImageState,
        gpu_images: &mut GpuImages,
        fonts: &mut FontState,
    ) -> FrameStats {
        let mut stats = FrameStats::default();
        let mut color = DEFAULT_COLOR;

        // Frame start: every image `R_CreateImage` staged since the last
        // frame becomes a texture, so a shader registered mid-frame is
        // drawable by the time its quad is bound.
        stats.images_uploaded = gpu_images.upload_pending(gpu, img_state, assets) as u32;

        self.batch.clear();

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
                    let (image, blend) = self.resolve_shader(*shader, assets, gpu_images);
                    self.batch.push_quad(
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
                        blend,
                        image,
                    );
                    stats.quads += 1;
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

                FrameEvent::ClearScene
                | FrameEvent::ClearDecals
                | FrameEvent::AddRefEntityToScene(_)
                | FrameEvent::AddPolyToScene { .. }
                | FrameEvent::AddPolysToScene { .. }
                | FrameEvent::AddLightToScene { .. }
                | FrameEvent::AddAdditiveLightToScene { .. }
                | FrameEvent::AddDecalToScene { .. }
                | FrameEvent::SetRangeFog(_)
                | FrameEvent::SetRefractionProp { .. }
                | FrameEvent::RenderScene { .. } => {
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

    /// Walks a draw's `ShaderHandle` to the texture it binds and the blend
    /// state it draws with: shader -> first active stage -> `bundle[0].image`
    /// -> uploaded texture, with the stage's `stateBits` decoded as the blend
    /// mode.
    ///
    /// "First active stage" is the single-pass reduction of
    /// `RB_StageIteratorGeneric`'s loop, which walks every active stage and
    /// re-issues `GL_State(pStage->stateBits)` per pass. 2D pics are
    /// overwhelmingly one-stage, so wave 2 renders stage 0 and leaves
    /// multi-pass 2D shaders (`animMap` frames, multi-stage menu art) to the
    /// wave that brings `tess`-equivalent multi-pass iteration.
    ///
    /// A missing link degrades to `(white, RB_SetGL2D's blend)` — the state
    /// the oracle's 2D pass is already in — never a dropped quad.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp` (`RB_StageIteratorGeneric`);
    /// `oracle/codemp/renderer/tr_backend.cpp:1282-1284` (`RB_SetGL2D`)
    //TODO: Port RB_StageIteratorGeneric's multi-stage 2D pass
    // Source: oracle/codemp/renderer/tr_shade.cpp
    fn resolve_shader(
        &mut self,
        shader: ShaderHandle,
        assets: &RenderAssets,
        gpu_images: &GpuImages,
    ) -> (Option<ImageHandle>, BlendState) {
        let default = (None, blend_state_from_gls(GLS_2D_DEFAULT));

        let Some(asset) = assets.shaders.get(shader) else {
            self.warn_once(Warned::UnknownShader);
            return default;
        };
        let Some(stage) = asset.stages.iter().find(|stage| stage.active) else {
            // The default shader and every not-yet-parsed shader are
            // stage-less; that is ordinary, not an error worth a log.
            return default;
        };

        let blend = blend_state_from_gls(stage.state_bits);
        match stage.bundle[0].image {
            Some(image) if gpu_images.contains(image) => (Some(image), blend),
            _ => {
                self.warn_once(Warned::NoStageImage);
                (None, blend)
            }
        }
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
