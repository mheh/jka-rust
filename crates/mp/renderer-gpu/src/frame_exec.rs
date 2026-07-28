//! `frame_exec` — walks one frame's [`FrameData`] event stream and turns it
//! into GPU work (R4a backend #1, v0).
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
//! v0 renders `SetColor` + `DrawStretchPic`. Everything else — the rotate-pic
//! pair, `DrawString`, and the whole scene-composition half of the enum —
//! is counted and skipped, with one `eprintln!` the first time each kind is
//! seen. Skipping, not panicking: a real `ui` frame emits string and scene
//! events from day one, and a loud-but-live frame loop is what makes the next
//! slices bisectable.

use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use wgpu::TextureView;

use crate::blend::{blend_state_from_gls, GLS_2D_DEFAULT};
use crate::gpu::Gpu;
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
    /// `DrawStretchPic` events turned into geometry.
    pub quads: u32,
    /// `SetColor` events applied.
    pub color_changes: u32,
    /// `draw` calls the batch collapsed to (one per blend-state run).
    pub draw_calls: u32,
    /// `DrawString` events skipped — the font pipeline is a later slice.
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

/// The kinds of event v0 cannot render, tracked so each one logs once per
/// process rather than once per frame.
#[derive(Clone, Copy, Debug)]
enum Unsupported {
    DrawString,
    RotatePic,
    SceneEvent,
    Other,
}

impl Unsupported {
    const COUNT: usize = 4;

    fn slot(self) -> usize {
        match self {
            Unsupported::DrawString => 0,
            Unsupported::RotatePic => 1,
            Unsupported::SceneEvent => 2,
            Unsupported::Other => 3,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Unsupported::DrawString => "DrawString (font pipeline)",
            Unsupported::RotatePic => "DrawRotatePic/DrawRotatePic2",
            Unsupported::SceneEvent => "scene composition (RenderScene and friends)",
            Unsupported::Other => "world-effect / automap commands",
        }
    }
}

/// Owns the render-thread state one frame's execution needs: the 2D pipeline,
/// the reused geometry batch, and the warn-once flags.
pub struct FrameExecutor {
    pipeline: Pipeline2d,
    batch: QuadBatch,
    warned: [bool; Unsupported::COUNT],
}

impl FrameExecutor {
    /// Builds the executor's GPU resources against `gpu`'s device.
    pub fn new(gpu: &Gpu) -> FrameExecutor {
        FrameExecutor {
            pipeline: Pipeline2d::new(gpu),
            batch: QuadBatch::new(),
            warned: [false; Unsupported::COUNT],
        }
    }

    /// Replays `frame_data`'s events in order into `target`.
    ///
    /// The colour register is per-frame, not persistent: the oracle's
    /// `RB_SetGL2D` path re-establishes white at the top of every 2D pass, and
    /// `FrameData` is a complete frame, so starting from [`DEFAULT_COLOR`]
    /// keeps a dropped frame from tinting the next one.
    pub fn execute_frame(
        &mut self,
        gpu: &mut Gpu,
        target: &TextureView,
        frame_data: &FrameData,
    ) -> FrameStats {
        let mut stats = FrameStats::default();
        let mut color = DEFAULT_COLOR;
        // v0 stamps every quad with `RB_SetGL2D`'s blend state. Per-stage
        // `GLS_*` bits arrive with shader resolution next wave; the batcher
        // already splits runs on a blend change, so that is a one-line swap.
        let blend = blend_state_from_gls(GLS_2D_DEFAULT);

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
                    // v0 has one built-in white texture; resolving the handle
                    // to an uploaded image is next wave's work.
                    shader: _,
                } => {
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
                    );
                    stats.quads += 1;
                }

                FrameEvent::DrawString { .. } => {
                    stats.skipped_strings += 1;
                    self.warn_once(Unsupported::DrawString);
                }
                FrameEvent::DrawRotatePic { .. } | FrameEvent::DrawRotatePic2 { .. } => {
                    stats.skipped_rotate_pics += 1;
                    self.warn_once(Unsupported::RotatePic);
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
                    self.warn_once(Unsupported::SceneEvent);
                }

                FrameEvent::WorldEffectCommand(_) | FrameEvent::AutomapElevAdj(_) => {
                    stats.skipped_other += 1;
                    self.warn_once(Unsupported::Other);
                }
            }
        }

        stats.draw_calls = self.pipeline.draw(gpu, target, &self.batch);
        stats
    }

    /// Logs an unsupported event kind the first time it is seen.
    fn warn_once(&mut self, kind: Unsupported) {
        let slot = kind.slot();
        if self.warned[slot] {
            return;
        }
        self.warned[slot] = true;
        eprintln!(
            "mp_renderer_gpu: frame_exec v0 skips {} — not rendered yet",
            kind.describe()
        );
    }
}
