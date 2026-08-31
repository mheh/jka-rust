//! `weather_frame` — the seam payload one frame of weather billboards crosses on.
//!
//! Raven's `CWeatherParticleCloud::Render` is a self-contained fixed-function GL block with no `shader_t`, no `tess`, and no `drawSurf_t`.
//! These three carrier types are port-invented, not ported Raven types, so the one-type-per-file rule does not split them.
//!
//! Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1311-1480`

use mp_qshared::shared::vec3_t;

use crate::render_state::image_asset::ImageHandle;

/// One vertex of a weather billboard: `qglVertex3f`'s position, `qglTexCoord2f`'s texcoord, and the colour `qglColor4f` chose for this particle.
/// The positions are absolute world coordinates, because the oracle loads the plain world model matrix and pushes nothing else.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1393-1459`
#[derive(Clone, Copy)]
pub struct WeatherVertex {
    pub position: vec3_t,
    pub st: [f32; 2],
    pub color: [f32; 4],
}

/// One cloud's billboards for this frame, with the image and the GL blend bits `Render` binds before it draws them.
/// `nearest_filter` is Raven's `mFilterMode != 0`, the per-cloud `GL_NEAREST` min and mag filter. Neither weather filter uses mips.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1319-1320,1364-1365`
pub struct WeatherCloudBatch {
    pub image: Option<ImageHandle>,
    pub state_bits: u32,
    pub nearest_filter: bool,
    pub vertices: Vec<WeatherVertex>,
    pub indices: Vec<u32>,
}

/// Every cloud's batch for one frame, in `mParticleClouds` order.
/// The order is the oracle's own draw order, and later clouds blend over earlier ones.
///
/// This type deliberately carries no view. The positional invariant stands in its place: one weather batch per frame, built from the world
/// scene's refdef, with its `FrameEvent` emitted inside that scene's event span, so the executor draws it under the view that built it.
/// A future multi-view consumer revisits this as its own ruling.
///
/// Source: `oracle/codemp/renderer/tr_WorldEffects.cpp:1569-1574`
pub struct WeatherFrame {
    pub clouds: Vec<WeatherCloudBatch>,
}

impl WeatherFrame {
    /// Whether the frame drew no cloud, so the executor arm can skip it without opening a pass.
    pub fn is_empty(&self) -> bool {
        self.clouds.is_empty()
    }
}
