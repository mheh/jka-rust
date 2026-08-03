//! `RenderWorld` — the render thread's owned world (`R2-D1`).

use std::sync::Arc;

use crate::render_state::frame_state::FrameState;
use crate::render_state::render_assets::RenderAssets;

/// The render thread's one owned instance: the sim's published assets picked
/// up at the frame boundary (A9), and this frame's scratch. GPU state lives
/// on the render thread in `mp_renderer_gpu` (DEC-63.4). Replaces the
/// oracle's `tr`/`backEnd`/`glState` globals as a single threaded object
/// (porting-rules §B4/§B6).
pub struct RenderWorld {
    /// `RenderAssetsSim::published`, picked up at a frame boundary — immutable
    /// for the duration of the frame (A9).
    pub assets: Arc<RenderAssets>,
    pub frame: FrameState,
}
