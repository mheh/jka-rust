//! `FramePackage` — one finished frame, owned, on its way to the render thread.

use std::sync::Arc;

use crate::render_state::capture_request::CaptureRequest;
use crate::render_state::frame_data::FrameData;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use crate::tr_image::PendingUpload;

/// Everything the render thread needs to draw one frame, and nothing it would
/// have to borrow back from the sim thread.
///
/// `RE_EndFrame` builds one of these per frame and sends it down a bounded
/// channel. The bound paces the sim thread rather than dropping a frame, which
/// is the no-drop posture the draw path was designed for: every event in the
/// stream is a draw the client already decided to issue.
///
/// New construct, no Raven counterpart. The oracle's `backEndData_t`
/// double-buffer served the same purpose for its own SMP split.
pub struct FramePackage {
    /// The frame's ordered event stream. The render thread returns the emptied
    /// buffer so the next frame reuses its allocation (`R2-D8`).
    pub frame_data: FrameData,
    /// `backEnd.refdef.floatTime` in seconds, the 2D pass's shader clock.
    pub float_time: f32,
    /// The `r_*` values this frame was built against, read once on the sim
    /// side so the replay never reaches a live cvar table.
    pub cvars: RenderCvarSnapshot,
    /// The registry generation this frame's handles resolve against. Cloning
    /// the `Arc` is what keeps the render thread off the sim's registry.
    pub assets: Arc<RenderAssets>,
    /// Pixels staged by `R_CreateImage` since the last frame, drained out of
    /// `TrImageState::pending_uploads`. The render thread turns them into
    /// textures before it replays the events, so a shader registered this
    /// frame is drawable by the time its quad binds.
    pub uploads: Vec<(ImageHandle, PendingUpload)>,
    /// A pending `screenshot_tga`, answered after the frame presents.
    pub capture: Option<CaptureRequest>,
}
