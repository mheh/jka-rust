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
//! What this thread does NOT do yet: replay the engine's `FrameData`.
//! `FrameExecutor::execute_frame` still borrows sim-thread renderer state (the
//! asset registry, the image and font state, and the whole `WorldFrame` engine
//! view), so the frame stream cannot cross a thread boundary until that state
//! moves behind an owned per-frame package. That is the first-light work, not
//! the platform shell.

use std::sync::mpsc::Receiver;

use mp_renderer_gpu::{FrameError, FrameExecutor, Gpu, GpuImages};

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
pub fn run(mut gpu: Gpu, commands: Receiver<RenderCommand>) {
    let images = GpuImages::new(&gpu);
    let mut executor = FrameExecutor::new(&gpu, &images);
    let (mut width, mut height) = gpu.surface_size();
    let mut surface_warned = false;
    let mut presented = false;

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
            RenderCommand::Present => match gpu.begin_frame() {
                Ok(frame) => {
                    presented = true;
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
            },
            RenderCommand::Shutdown => break,
        }
    }
}
