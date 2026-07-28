//! Dev harness for `mp_renderer_gpu` — opens a window, drives [`Gpu`]
//! through a clear-color frame loop, and exits on Escape or window close.
//!
//! Scaffold only (see the crate-level docs in `lib.rs`): this harness exists
//! to prove the window/device/surface plumbing works end to end on the dev
//! machine (macOS). It carries no game logic — R4a's ui 2D command surface
//! replaces the clear pass with real draw commands.

use std::sync::Arc;

use mp_renderer_gpu::Gpu;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
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
        window.request_redraw();

        self.window = Some(window);
        self.gpu = Some(gpu);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let (Some(window), Some(gpu)) = (self.window.as_ref(), self.gpu.as_mut()) else {
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
                    Ok(frame) => gpu.present(frame),
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

fn main() {
    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("run_app: dev harness event loop exited with an error");
}
