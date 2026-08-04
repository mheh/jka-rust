//! The render thread (DEC-37.2, DEC-56.2).
//!
//! This thread is the only owner of every real GPU object in the process: the
//! wgpu instance, device, queue and surface inside [`Gpu`], the uploaded
//! textures in `GpuImages`, and the pipelines inside `FrameExecutor`. Nothing
//! else in the process holds a handle to any of them, so no engine-interior
//! path can reach GPU state (the DEC-60.1 re-audit conclusion).
//!
//! The main thread builds the `Gpu` while it holds the window, because a macOS
//! surface must be created from the window's own thread, and then hands the
//! whole value across. Everything after that runs here.
//!
//! The frame stream reaches this thread as a `FramePackage`, which owns
//! everything the replay reads: the events, the registry generation behind an
//! `Arc`, the staged image uploads, and the frame's cvar values. Nothing is
//! borrowed back from the sim thread. The emptied event buffer goes back down
//! the return channel, so the next frame reuses its allocation.
//!
//! The world pass runs here too since W2-F7. A package carries the loaded
//! world on the frame it changes, so this thread uploads the geometry once and
//! then walks the BSP, the sky, the fog and the dynamic lights itself. The MD3
//! and Ghoul2 entity arms stay dark until the model blocks cross (wave 3).

use std::fs;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use mp_renderer::render_state::capture_format::CaptureFormat;
use mp_renderer::render_state::capture_request::CaptureRequest;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_package::FramePackage;
use mp_renderer::tr_init::{R_TakeScreenshot, R_TakeScreenshotJPEG};
use mp_renderer::tr_noise::{NoiseState, R_NoiseInit};
use mp_renderer_gpu::{read_texture_rgb_bottom_up, FrameError, FrameExecutor, Gpu, GpuImages};
use native_math::rng::Rng;

/// What the pump asks the render thread to do.
///
/// - `Resize`: the window changed size, so reconfigure the surface and the
///   depth target.
/// - `Present`: acquire, draw, and present one frame.
/// - `Shutdown`: release the GPU and end the thread.
pub enum RenderCommand {
    Resize { width: u32, height: u32 },
    Present,
    Shutdown,
}

/// Run the render thread until the pump sends `Shutdown` or drops the channel.
pub fn run(
    mut gpu: Gpu,
    commands: Receiver<RenderCommand>,
    packages: Receiver<FramePackage>,
    recycled: Sender<FrameData>,
) {
    let mut images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    let (mut width, mut height) = gpu.surface_size();
    let mut surface_warned = false;
    let mut presented = false;

    // `R_NoiseInit` seeds from a fixed `srand(1001)`, so this copy is
    // byte-identical to the sim thread's and needs no sharing.
    // Source: oracle/codemp/renderer/tr_noise.cpp:32-43
    let mut noise = NoiseState::default();
    R_NoiseInit(&mut noise, &mut Rng::new());

    // The frame being drawn. It is held rather than consumed, so a present that
    // arrives before the next package redraws the last one.
    let mut held: Option<FramePackage> = None;

    while let Ok(command) = commands.recv() {
        match command {
            RenderCommand::Resize {
                width: w,
                height: h,
            } => {
                width = w.max(1);
                height = h.max(1);
                gpu.resize(width, height);
                executor.resize(&gpu, width, height);
            }

            RenderCommand::Present => {
                // Take at most one new package per present, so a sim thread
                // running ahead cannot starve the window of frames.
                match packages.try_recv() {
                    Ok(package) => {
                        if let Some(previous) = held.replace(package) {
                            // Hand the emptied buffer back with its capacity.
                            let mut buffer = previous.frame_data;
                            buffer.events.clear();
                            let _ = recycled.send(buffer);
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                match gpu.begin_frame() {
                    Ok(frame) => {
                        presented = true;
                        let target = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        // With no package yet, the acquired frame presents as
                        // the clear colour, which is what the boot frames did
                        // before the stream crossed.
                        if let Some(package) = held.as_mut() {
                            executor.execute_package(
                                &mut gpu,
                                &target,
                                package,
                                &mut images,
                                &noise,
                            );

                            if let Some(capture) = package.capture.take() {
                                write_screenshot(&gpu, &frame.texture, &capture);
                                if !capture.silent {
                                    println!("Wrote {}", capture.os_path);
                                }
                            }
                        }

                        gpu.present(frame);
                    }

                    Err(FrameError::NeedsReconfigure) => {
                        gpu.resize(width, height);
                        executor.resize(&gpu, width, height);
                    }

                    Err(FrameError::Skip) => {
                        // The first frames before the window is mapped skip by
                        // design, so only a later gap is worth a line.
                        if presented && !surface_warned {
                            surface_warned = true;
                            eprintln!("jamp: the surface is not visible, so frames are skipped");
                        }
                    }
                }
            }

            RenderCommand::Shutdown => break,
        }
    }
}

/// Reads the drawn frame back and writes it in the format the request asked
/// for.
///
/// Raven's `FS_WriteFile` creates the path on the way, so this creates the
/// screenshots directory before the write.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:537-571`, `oracle/codemp/renderer/tr_init.cpp:578-596`
fn write_screenshot(gpu: &Gpu, texture: &wgpu::Texture, capture: &CaptureRequest) {
    let (width, height, rgb) = read_texture_rgb_bottom_up(gpu, texture);
    let encoded = match capture.format {
        CaptureFormat::Tga => R_TakeScreenshot(&rgb, width as i32, height as i32),
        CaptureFormat::Jpeg { quality } => {
            R_TakeScreenshotJPEG(&rgb, width as i32, height as i32, quality)
        }
    };

    let os_path = &capture.os_path;
    if let Some(parent) = Path::new(os_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("jamp: could not create {}: {e}", parent.display());
            return;
        }
    }
    if let Err(e) = fs::write(os_path, &encoded) {
        eprintln!("jamp: could not write {os_path}: {e}");
    }
}
