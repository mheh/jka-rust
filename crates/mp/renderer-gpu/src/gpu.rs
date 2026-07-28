//! `Gpu` — owns the wgpu device/surface pair for one window.
//!
//! R4 seed only (see the crate-level docs in `lib.rs` for the DEC-37/DEC-44
//! citations): this is the device/surface plumbing every later slice
//! (R4a's 2D command surface, then the world/PBR backends) builds on top of.
//! No shader/pipeline/backend logic lives here yet.

use std::sync::Arc;

use winit::window::Window;

/// The clear color a frame starts from before any draw commands land. R4a
/// replaces this with the real 2D/scene render passes; the scaffold clears
/// to a fixed color so the dev harness has something visible to confirm the
/// pipeline end to end.
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.06,
    g: 0.08,
    b: 0.14,
    a: 1.0,
};

/// Errors [`Gpu::begin_frame`] can return when acquiring the next surface
/// texture. The caller (the dev harness today, the render thread's frame
/// loop later) decides how to react — `Lost`/`Outdated` mean reconfigure and
/// retry, `Timeout`/`Occluded` mean skip the frame.
#[derive(Debug)]
pub enum FrameError {
    /// The surface must be reconfigured (`resize`) before trying again.
    NeedsReconfigure,
    /// The frame should be skipped; the surface will recover on its own.
    Skip,
}

/// Owns the wgpu `Instance`/`Surface`/`Device`/`Queue`/`SurfaceConfiguration`
/// for one window. DEC-37 ruling 2: this lives on the render thread, never
/// the sim/VM thread; DEC-37 ruling 3: nothing behind this type is reachable
/// from a trap query.
pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    /// Creates the device/surface pair for `window`. Blocks the calling
    /// thread on the adapter/device requests via `pollster` — acceptable at
    /// startup (there is no frame loop to stall yet).
    pub fn new(window: Arc<Window>) -> Gpu {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("create_surface: window handle must be valid");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("request_adapter: no compatible GPU adapter found");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("mp_renderer_gpu device"),
            ..Default::default()
        }))
        .expect("request_device: adapter refused the device request");

        let config = surface
            .get_default_config(&adapter, width, height)
            .expect("get_default_config: surface incompatible with adapter");
        surface.configure(&device, &config);

        Gpu {
            surface,
            device,
            queue,
            config,
        }
    }

    /// The device every render-thread resource is created against.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// The queue buffer writes and command submissions go through.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// The surface's colour format — what every render pipeline's colour
    /// target must be built for.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// The surface's current size in physical pixels. Raven's 2D commands are
    /// authored in a 640x480 virtual space and scaled by the viewport, so this
    /// is a readout, not a coordinate system (see `pipeline2d`).
    pub fn surface_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Reconfigures the surface for a new window size. A no-op for a
    /// degenerate (minimized) size; wgpu requires both dimensions nonzero.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Acquires the next surface texture, clears it to [`CLEAR_COLOR`], and
    /// submits the encoded clear pass. Returns the acquired texture so the
    /// caller can present it once the frame's remaining draw commands (none
    /// yet — R4a adds the 2D command surface) have been recorded.
    pub fn begin_frame(&mut self) -> Result<wgpu::SurfaceTexture, FrameError> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(FrameError::NeedsReconfigure);
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return Err(FrameError::Skip),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mp_renderer_gpu clear encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mp_renderer_gpu clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(frame)
    }

    /// Presents a texture previously returned by [`begin_frame`].
    pub fn present(&self, frame: wgpu::SurfaceTexture) {
        self.queue.present(frame);
    }
}
