//! Dev harness for `mp_renderer_gpu` — opens a window, builds one frame's
//! `FrameData` per redraw, executes it through [`FrameExecutor`], and presents.
//! Exits on Escape or window close.
//!
//! R4a wave 1 (2D first light): this is the end-to-end proof that event
//! *production* -> *execution* -> *pixels* works. The harness plays the sim
//! side of the seam — it appends `SetColor`/`DrawStretchPic` events the way
//! `ui`/`cgame` traps will — and the render side replays them.
//!
//! **Single-threaded staging.** The frame stream is built and executed inline,
//! the same frame. DEC-37 ruling 2's sim/render thread split is a later R4
//! slice; when it lands, only this file changes — the `FrameData` arrives over
//! a channel instead of from `test_pattern()`, and `execute_frame` is called
//! unmodified.

use std::sync::Arc;

use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::shader_asset::ShaderHandle;
use mp_renderer_gpu::{FrameExecutor, FrameStats, Gpu};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    executor: Option<FrameExecutor>,
    /// The first frame's stats, printed once so a headless run leaves proof in
    /// the log that the events reached the GPU.
    reported: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes().with_title("jka-rust dev harness");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window: failed to open the dev harness window"),
        );
        let gpu = Gpu::new(window.clone());
        let executor = FrameExecutor::new(&gpu);
        window.request_redraw();

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.executor = Some(executor);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(window), Some(gpu), Some(executor)) = (
            self.window.as_ref(),
            self.gpu.as_mut(),
            self.executor.as_mut(),
        ) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(size) => {
                gpu.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                match gpu.begin_frame() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let stats = executor.execute_frame(gpu, &view, &test_pattern());
                        if !self.reported {
                            self.reported = true;
                            report(&stats);
                        }
                        gpu.present(frame);
                    }
                    Err(_) => {
                        // Reconfigure at the current window size and retry next frame.
                        let size = window.inner_size();
                        gpu.resize(size.width, size.height);
                    }
                }
                window.request_redraw();
            }
            _ => {}
        }
    }
}

/// Builds the frame stream: four overlapping quads in the 640x480 virtual
/// screen — opaque red, green and blue staggered down the diagonal, then a
/// half-transparent white band across their overlap so the alpha blend is
/// visible — plus one `DrawString` to exercise the executor's skip counter.
///
/// Every quad carries `ShaderHandle::slot_zero()`, the registries' default
/// entry (A12): v0 samples one built-in white texel regardless, so the handle
/// is carried, not resolved.
fn test_pattern() -> FrameData {
    let mut events = Vec::new();

    let mut quad = |rgba: [f32; 4], x: f32, y: f32, w: f32, h: f32| {
        events.push(FrameEvent::SetColor(rgba));
        events.push(FrameEvent::DrawStretchPic {
            x,
            y,
            w,
            h,
            s1: 0.0,
            t1: 0.0,
            s2: 1.0,
            t2: 1.0,
            shader: ShaderHandle::slot_zero(),
        });
    };

    quad([1.0, 0.0, 0.0, 1.0], 80.0, 80.0, 220.0, 180.0);
    quad([0.0, 1.0, 0.0, 1.0], 200.0, 140.0, 220.0, 180.0);
    quad([0.0, 0.0, 1.0, 1.0], 320.0, 200.0, 220.0, 180.0);
    quad([1.0, 1.0, 1.0, 0.5], 160.0, 160.0, 320.0, 160.0);

    events.push(FrameEvent::DrawString {
        ox: 16,
        oy: 16,
        text: String::from("mp_renderer_gpu 2D first light"),
        rgba: [1.0, 1.0, 1.0, 1.0],
        set_index: 0,
        char_limit: 0,
        scale: 1.0,
    });

    FrameData { events }
}

fn report(stats: &FrameStats) {
    println!(
        "dev_harness: first frame executed — {} quads, {} color changes, \
         {} draw calls, {} events skipped",
        stats.quads,
        stats.color_changes,
        stats.draw_calls,
        stats.skipped_events()
    );
}

fn main() {
    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("run_app: dev harness event loop exited with an error");
}
