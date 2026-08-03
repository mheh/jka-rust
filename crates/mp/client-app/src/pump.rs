//! The window pump (DEC-56.2, DEC-56.3), the main thread's only job.
//!
//! Raven's `MainWndProc` turned window messages into `Sys_QueEvent` calls on
//! the same thread that ran the game. macOS owns the main thread for the event
//! loop, so this handler does the translation and posts across the platform
//! bus, and the sim thread drains it inside `Sys_GetEvent`.
//!
//! The handler also owns the window and starts the render thread once the
//! window exists, because a macOS surface must be created from the window's own
//! thread.
//!
//! Source: `oracle/codemp/win32/win_wndproc.cpp:301-540`

use std::sync::mpsc::{sync_channel, Receiver, Sender, SyncSender};
use std::sync::Arc;
use std::thread::Builder;
use std::time::{Duration, Instant};

use mp_engine_qcommon::common::platform_events::{PlatformEvent, PlatformEventSink};
use mp_engine_qcommon::qcommon::sys_event_type_t::sysEventType_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::frame_package::FramePackage;
use mp_renderer_gpu::Gpu;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::keymap::{map_char, map_key, map_mouse_button, wheel_key};
use crate::render_thread::{self, RenderCommand};

/// Render commands the pump may hold before the render thread takes them.
/// One frame in flight plus a little slack for resize bursts.
const RENDER_QUEUE: usize = 4;

/// Trackpad pixels that make one wheel notch. macOS reports a gesture as a
/// stream of small pixel deltas, and Raven's `WHEEL_DELTA` was one notch.
const WHEEL_PIXELS_PER_NOTCH: f64 = 40.0;

/// How long the pump waits between present requests. Faster than any display,
/// so it never paces the picture, and slow enough to keep the main thread off a
/// spin.
const PRESENT_INTERVAL: Duration = Duration::from_micros(2000);

/// The window title, Raven's `WINDOW_CLASS_NAME` banner.
///
/// Source: `oracle/codemp/win32/win_glimp.cpp` (`WINDOW_CLASS_NAME`)
const WINDOW_TITLE: &str = "Jedi Academy";

/// The main thread's whole state: the window, the sink it posts input to, and
/// the render thread's command channel.
pub struct Pump {
    events: PlatformEventSink,
    window: Option<Arc<Window>>,
    render: Option<SyncSender<RenderCommand>>,
    /// The frame channel's render end, held until the window exists and the
    /// render thread can take it.
    packages: Option<Receiver<FramePackage>>,
    /// The return channel's render end, handed across with `packages`.
    recycled: Option<Sender<FrameData>>,
    /// Trackpad pixels not yet spent on a wheel notch.
    wheel_pixels: f64,
}

impl Pump {
    pub fn new(
        events: PlatformEventSink,
        packages: Receiver<FramePackage>,
        recycled: Sender<FrameData>,
    ) -> Pump {
        Pump {
            events,
            window: None,
            render: None,
            packages: Some(packages),
            recycled: Some(recycled),
            wheel_pixels: 0.0,
        }
    }

    /// Post one key or char event, Raven's `Sys_QueEvent` call from the window
    /// procedure.
    fn queue(&self, kind: sysEventType_t, value: i32, value2: i32) {
        self.events.queue(PlatformEvent {
            evType: kind,
            evValue: value,
            evValue2: value2,
        });
    }

    /// A wheel notch, which Raven turns into a key press and release pair.
    ///
    /// Source: `oracle/codemp/win32/win_wndproc.cpp:345-355`
    fn queue_wheel(&self, up: bool) {
        let key = wheel_key(up);
        self.queue(sysEventType_t::SE_KEY, key, 1);
        self.queue(sysEventType_t::SE_KEY, key, 0);
    }

    fn send_render(&self, command: RenderCommand) {
        if let Some(render) = self.render.as_ref() {
            // A full queue means the render thread is still on the last frame,
            // so this one is dropped rather than stalling the window.
            let _ = render.try_send(command);
        }
    }
}

impl ApplicationHandler for Pump {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes().with_title(WINDOW_TITLE);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("create_window: the client could not open its window"),
        );

        // The surface is built here, on the thread that owns the window, and
        // then the whole GPU moves to the render thread for good.
        let gpu = Gpu::new(Arc::clone(&window));
        let (tx, rx) = sync_channel(RENDER_QUEUE);
        let packages = self
            .packages
            .take()
            .expect("resumed: the frame channel was already handed to a render thread");
        let recycled = self
            .recycled
            .take()
            .expect("resumed: the return channel was already handed to a render thread");
        Builder::new()
            .name("jamp-render".to_string())
            .spawn(move || render_thread::run(gpu, rx, packages, recycled))
            .expect("spawn: the client could not start its render thread");

        window.request_redraw();
        self.window = Some(window);
        self.render = Some(tx);
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        // Raw motion, the DirectInput mouse Raven preferred over cursor
        // positions. The sim thread sums it into one `SE_MOUSE` per frame.
        // Source: `oracle/codemp/win32/win_input.cpp:410-447`
        if let DeviceEvent::MouseMotion { delta } = event {
            self.events.add_mouse_delta(delta.0, delta.1);
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            // Raven had no `WM_CLOSE` arm: the close fell through to
            // `DefWindowProc`, and `Sys_GetEvent`'s pump answered the resulting
            // `WM_QUIT` with `Com_Quit_f`. So the pump only asks, and the sim
            // thread runs the real shutdown (config write, disconnect, FS) and
            // exits the process from `Sys_Quit`.
            // Source: `oracle/codemp/win32/win_main.cpp:1226-1228`
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                self.events.request_quit();
            }
            WindowEvent::Resized(size) => {
                self.send_render(RenderCommand::Resize {
                    width: size.width,
                    height: size.height,
                });
            }
            //TODO: Port IN_ActivateMouse
            // Source: oracle/codemp/win32/win_input.cpp:498-560. Raven grabbed
            // the pointer while the app was active and released it for the
            // console and the menus, which it read from `Key_GetCatcher`. The
            // pump has no key catcher to read until the client hooks are live,
            // and a grab with no release path traps the user, so the grab lands
            // with first light. Raw motion already reaches `SE_MOUSE` without
            // it.
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        text,
                        ..
                    },
                ..
            } => {
                let down = state == ElementState::Pressed;
                if let PhysicalKey::Code(code) = physical_key {
                    if let Some(key) = map_key(code) {
                        self.queue(sysEventType_t::SE_KEY, key, down as i32);
                    }
                    // The console key never produces a character, so the
                    // console open stroke does not type a backquote.
                    if code == KeyCode::Backquote {
                        return;
                    }
                }
                // `WM_CHAR` follows the key down, auto-repeats included.
                if down {
                    if let Some(text) = text {
                        for character in text.chars() {
                            if let Some(value) = map_char(character) {
                                self.queue(sysEventType_t::SE_CHAR, value, 0);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(key) = map_mouse_button(button) {
                    let down = state == ElementState::Pressed;
                    self.queue(sysEventType_t::SE_KEY, key, down as i32);
                }
            }
            // Raven's `WM_MOUSEWHEEL` arrived pre-quantised at `WHEEL_DELTA`,
            // one notch per message. A line delta is already a notch; a pixel
            // delta is a raw trackpad gesture, so it accumulates to a notch
            // rather than firing a weapon switch per pixel.
            // Source: `oracle/codemp/win32/win_wndproc.cpp:345-355`
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    for _ in 0..y.abs().floor() as u32 {
                        self.queue_wheel(y > 0.0);
                    }
                }
                MouseScrollDelta::PixelDelta(position) => {
                    self.wheel_pixels += position.y;
                    while self.wheel_pixels.abs() >= WHEEL_PIXELS_PER_NOTCH {
                        let up = self.wheel_pixels > 0.0;
                        self.wheel_pixels -= WHEEL_PIXELS_PER_NOTCH * if up { 1.0 } else { -1.0 };
                        self.queue_wheel(up);
                    }
                }
            },
            WindowEvent::RedrawRequested => {
                self.send_render(RenderCommand::Present);
            }
            _ => {}
        }
    }

    /// Ask for the next frame here rather than inside `RedrawRequested`: winit
    /// only guarantees a request made outside the redraw itself.
    ///
    /// The wait deadline keeps the main thread off a spin. It paces only the
    /// present rate, well above any display refresh; the sim thread keeps its
    /// own clock through `com_maxfps`.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + PRESENT_INTERVAL));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.send_render(RenderCommand::Shutdown);
    }
}

/// The control flow the pump boots under. `about_to_wait` replaces it with a
/// deadline once the first frame is requested.
pub const PUMP_CONTROL_FLOW: ControlFlow = ControlFlow::Poll;
