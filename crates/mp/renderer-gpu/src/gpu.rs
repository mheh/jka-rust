//! `Gpu` — owns the wgpu device/surface pair for one window, or an offscreen
//! target for a headless golden run.
//!
//! R4 seed only (see the crate-level docs in `lib.rs` for the DEC-37/DEC-44
//! citations): this is the device/surface plumbing every later slice
//! (R4a's 2D command surface, then the world/PBR backends) builds on top of.
//! No shader/pipeline/backend logic lives here yet.
//!
//! The headless path ([`Gpu::new_headless`]) draws into an offscreen texture
//! instead of a window surface. It exists for the R4 image-golden gate: a test
//! renders a fixed scene and reads the pixels back with [`read_target_rgba`].
//! The windowed path is unchanged.

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

/// The offscreen target's color format. The windowed surface picks its own
/// format from the adapter, and `Bgra8UnormSrgb` is the format a macOS/Metal
/// swapchain reports, so the headless golden matches what a window shows.
/// [`read_target_rgba`] swaps the read-back BGRA channels into RGBA.
///
/// This match holds only where the swapchain also reports `Bgra8UnormSrgb`.
/// A driver that reports a non-sRGB swapchain format renders the same scene
/// with different gamma, and the golden then differs from the window.
// TODO: R4 golden rig - read the windowed surface format here once a
// non-macOS/Metal target ships, so the headless format tracks the swapchain.
const HEADLESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

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

/// Where a [`Gpu`] draws. The windowed arm owns a window surface. The headless
/// arm owns an offscreen texture that stands in for the surface texture, so a
/// golden test can read the pixels back.
///
/// - `Windowed`: the surface acquired for one window.
/// - `Headless`: the offscreen render-attachment texture, also `COPY_SRC` for
///   read-back.
enum RenderTarget {
    Windowed(wgpu::Surface<'static>),
    Headless(wgpu::Texture),
}

/// Owns the wgpu `Instance`/`Surface`/`Device`/`Queue`/`SurfaceConfiguration`
/// for one window. DEC-37 ruling 2: this lives on the render thread, never
/// the sim/VM thread; DEC-37 ruling 3: nothing behind this type is reachable
/// from a trap query.
pub struct Gpu {
    target: RenderTarget,
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
            target: RenderTarget::Windowed(surface),
            device,
            queue,
            config,
        }
    }

    /// Creates a device with no window and an offscreen `width`x`height` target
    /// for a headless golden run. Requests the adapter with no compatible
    /// surface, so this works where no window system exists.
    ///
    /// The offscreen texture is `RENDER_ATTACHMENT | COPY_SRC` so a render pass
    /// draws into it and [`read_target_rgba`] copies it out.
    pub fn new_headless(width: u32, height: u32) -> Gpu {
        Gpu::try_new_headless(width, height).expect("request_adapter: no compatible GPU adapter")
    }

    /// [`Gpu::new_headless`], but returns `None` where the platform offers no
    /// adapter at all. The image-golden gate calls this so a machine with no
    /// GPU skips its scenes instead of failing them.
    pub fn try_new_headless(width: u32, height: u32) -> Option<Gpu> {
        let width = width.max(1);
        let height = height.max(1);

        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        }))
        .ok()?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("mp_renderer_gpu headless device"),
            ..Default::default()
        }))
        .ok()?;

        // The config stands in for a windowed surface's configuration. Only the
        // format and size are read (through `surface_format`/`surface_size`), so
        // the present-mode and alpha fields carry ordinary swapchain defaults.
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: HEADLESS_FORMAT,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        };
        let texture = create_offscreen_texture(&device, &config);

        Some(Gpu {
            target: RenderTarget::Headless(texture),
            device,
            queue,
            config,
        })
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

    /// A view of the offscreen texture to draw into. The golden test passes
    /// this where the windowed harness passes the acquired surface texture's
    /// view. Panics on the windowed path, which has no offscreen texture.
    pub fn headless_view(&self) -> wgpu::TextureView {
        let RenderTarget::Headless(texture) = &self.target else {
            panic!("headless_view: the gpu is windowed, so it has no offscreen texture");
        };
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// Clears the offscreen target to [`CLEAR_COLOR`], the clear the windowed
    /// path gets from [`begin_frame`]. The world pass loads the color target,
    /// so a headless run must clear it first or uncovered pixels come from
    /// wgpu zero-init instead of [`CLEAR_COLOR`]. Panics on the windowed path.
    pub fn clear_headless(&self, target: &wgpu::TextureView) {
        let RenderTarget::Headless(_) = &self.target else {
            panic!("clear_headless: the gpu is windowed; begin_frame clears instead");
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mp_renderer_gpu headless clear encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mp_renderer_gpu headless clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
    }

    /// Reconfigures the target for a new size. A no-op for a degenerate
    /// (minimized) size. wgpu requires both dimensions nonzero. The windowed
    /// arm reconfigures the surface. The headless arm replaces the offscreen
    /// texture.
    ///
    /// The headless arm invalidates any [`headless_view`] taken before the
    /// resize. The caller must take a fresh view after a resize, or a draw
    /// lands in the old texture and [`read_target_rgba`] reads the new one.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        match &mut self.target {
            RenderTarget::Windowed(surface) => {
                surface.configure(&self.device, &self.config);
            }
            RenderTarget::Headless(texture) => {
                *texture = create_offscreen_texture(&self.device, &self.config);
            }
        }
    }

    /// Acquires the next surface texture, clears it to [`CLEAR_COLOR`], and
    /// submits the encoded clear pass. Returns the acquired texture so the
    /// caller can present it once the frame's remaining draw commands (none
    /// yet — R4a adds the 2D command surface) have been recorded.
    ///
    /// Windowed only: the headless path draws into [`headless_view`] directly.
    pub fn begin_frame(&mut self) -> Result<wgpu::SurfaceTexture, FrameError> {
        let RenderTarget::Windowed(surface) = &self.target else {
            panic!("begin_frame: the gpu is headless; draw into headless_view instead");
        };
        let frame = match surface.get_current_texture() {
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

/// Builds the offscreen render-attachment texture the headless target owns.
/// The `COPY_SRC` usage lets [`read_target_rgba`] copy the pixels out.
fn create_offscreen_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mp_renderer_gpu headless target"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Reads the headless target's pixels back as tightly packed RGBA8, returning
/// the width, the height, and `width * height * 4` bytes. Panics on the
/// windowed path, which has no offscreen texture.
///
/// The copy pads each row up to `COPY_BYTES_PER_ROW_ALIGNMENT`, so this strips
/// the padding after the map. The offscreen format is BGRA, so this swaps each
/// pixel's blue and red channels into RGBA order.
pub fn read_target_rgba(gpu: &Gpu) -> (u32, u32, Vec<u8>) {
    let RenderTarget::Headless(texture) = &gpu.target else {
        panic!("read_target_rgba: the gpu is windowed, so it has no offscreen texture");
    };
    let width = gpu.config.width;
    let height = gpu.config.height;
    let bytes_per_pixel = 4u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;

    // wgpu requires the copy's bytes-per-row to be a multiple of 256.
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mp_renderer_gpu readback buffer"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mp_renderer_gpu readback encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(std::iter::once(encoder.finish()));

    // Map the whole buffer, then block until the copy and map complete.
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("poll: waiting for the readback map failed");

    let mapped = slice
        .get_mapped_range()
        .expect("get_mapped_range: readback buffer was not mapped");

    // Strip the row padding and swap BGRA into RGBA.
    let mut rgba = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
    for row in 0..height {
        let start = (row * padded_bytes_per_row) as usize;
        let row_bytes = &mapped[start..start + unpadded_bytes_per_row as usize];
        for pixel in row_bytes.chunks_exact(4) {
            rgba.push(pixel[2]);
            rgba.push(pixel[1]);
            rgba.push(pixel[0]);
            rgba.push(pixel[3]);
        }
    }

    drop(mapped);
    buffer.unmap();

    (width, height, rgba)
}
