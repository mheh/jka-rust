//! `ui_harness` — R4a wave 3: the ported `ui` menu framework hosted on our own
//! stack and rendered through backend #1.
//!
//! The milestone this bin exists to prove: a window opens, the engine's FS
//! subset indexes the retail `base/` pk3s, the real `R_Init` builds the
//! renderer's CPU frontend, the real `mp_uishared` menu framework parses
//! `ui/jampmenus.txt` with the production botlib precompiler, and every draw
//! call the menus make becomes a `FrameEvent` that [`FrameExecutor`] turns into
//! pixels — no faked registry, no synthetic texture, no stand-in tokenizer.
//!
//! Sibling bin `dev_harness` keeps the wave-1/2 synthetic test pattern; this
//! one drives real assets. Neither shares state with the other.
//!
//! Usage: `cargo run -p mp_renderer_gpu --bin ui_harness [-- <basepath> [menu]]`.
//! `JKA_HARNESS_FRAMES=<n>` exits after `n` frames (headless CI runs);
//! `JKA_HARNESS_SECONDS=<n>` exits after `n` seconds.

use std::sync::Arc;
use std::time::{Duration, Instant};

use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer_gpu::ui_host::boot::with_dc;
use mp_renderer_gpu::ui_host::{boot, BootConfig, UiHost};
use mp_renderer_gpu::{FrameExecutor, FrameStats, Gpu, GpuImages, SCREEN_HEIGHT, SCREEN_WIDTH};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::ui_shared::{
    Display_MouseMove, Menu_Count, Menu_GetFocused, Menu_HandleKey, Menu_PaintAll,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

/// Raven `A_MOUSE1` — the ui module's mouse-button keycode base
/// (`oracle/codemp/ui/keycodes.h`).
const A_MOUSE1: i32 = 178;

/// Raven `A_ENTER`/`A_ESCAPE`/`A_CURSOR_*` — the handful of keycodes the
/// harness maps so a menu is navigable.
///
/// Source: `oracle/codemp/ui/keycodes.h`
const A_TAB: i32 = 9;
const A_ENTER: i32 = 13;
const A_ESCAPE: i32 = 27;
const A_CURSOR_UP: i32 = 132;
const A_CURSOR_DOWN: i32 = 133;
const A_CURSOR_LEFT: i32 = 134;
const A_CURSOR_RIGHT: i32 = 135;

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    images: Option<GpuImages>,
    executor: Option<FrameExecutor>,
    host: UiHost,
    start: Instant,
    frames: u64,
    /// Frame budget from `JKA_HARNESS_FRAMES`, or `u64::MAX`.
    max_frames: u64,
    /// Wall-clock budget from `JKA_HARNESS_SECONDS`, or `None`.
    max_seconds: Option<u64>,
    /// The last window size, so cursor positions map into the 640x480 virtual
    /// screen the menus and `pipeline2d` both use.
    surface: (f32, f32),
    reported: bool,
}

impl App {
    fn new(host: UiHost) -> App {
        App {
            window: None,
            gpu: None,
            images: None,
            executor: None,
            host,
            start: Instant::now(),
            frames: 0,
            max_frames: std::env::var("JKA_HARNESS_FRAMES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(u64::MAX),
            max_seconds: std::env::var("JKA_HARNESS_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok()),
            surface: (SCREEN_WIDTH, SCREEN_HEIGHT),
            reported: false,
        }
    }

    /// Paints one ui frame: the framework's `Menu_PaintAll` writes into the
    /// `HarnessDc`'s `FrameData`, which comes back here for execution. This is
    /// `_UI_Refresh`'s paint half — the cvar-update / server-refresh /
    /// find-player halves are `mp_ui`-owned and unreachable from here.
    fn ui_frame(&mut self) -> FrameData {
        let realtime = self.start.elapsed().as_millis() as i32;
        with_dc(&mut self.host, |dc, ui| {
            ui.uiDC.frameTime = realtime - ui.uiDC.realTime;
            ui.uiDC.realTime = realtime;
            let ds = &ui.uiDC;
            Menu_PaintAll(&mut ui.menus, ds, dc, 0);
            // The cursor, drawn last exactly as `_UI_Refresh` does.
            dc.setColor(None);
            dc.drawHandlePic(
                ui.uiDC.cursorx as f32 - 16.0,
                ui.uiDC.cursory as f32 - 16.0,
                32.0,
                32.0,
                ui.uiDC.cursor,
            );
            core::mem::replace(&mut dc.frame_data, FrameData { events: Vec::new() })
        })
    }

    /// `_UI_MouseEvent`'s core: move the virtual cursor, clamp it to the
    /// 640x480 screen, then let the framework hit-test.
    fn mouse_to(&mut self, px: f32, py: f32) {
        let (sw, sh) = self.surface;
        let x = (px / sw * SCREEN_WIDTH).clamp(0.0, SCREEN_WIDTH - 1.0);
        let y = (py / sh * SCREEN_HEIGHT).clamp(0.0, SCREEN_HEIGHT - 1.0);
        with_dc(&mut self.host, |dc, ui| {
            let dx = x as i32 - ui.uiDC.cursorx;
            let dy = y as i32 - ui.uiDC.cursory;
            ui.uiDC.cursorx = x as i32;
            ui.uiDC.cursory = y as i32;
            let ds = &ui.uiDC;
            Display_MouseMove(&mut ui.menus, ds, dc, None, dx, dy);
        });
    }

    /// `_UI_KeyEvent`'s core: hand the key to the focused menu.
    fn key(&mut self, key: i32, down: bool) {
        with_dc(&mut self.host, |dc, ui| {
            if Menu_Count(&ui.menus) > 0 {
                let focused = Menu_GetFocused(&ui.menus);
                let ds = &ui.uiDC;
                Menu_HandleKey(&mut ui.menus, ds, dc, focused, key, down);
            }
        });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("jka-rust ui harness");
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window: failed to open the ui harness window"),
        );
        let gpu = Gpu::new(window.clone());
        let images = GpuImages::new(&gpu);
        let executor = FrameExecutor::new(&gpu, &images);
        let size = window.inner_size();
        self.surface = (size.width.max(1) as f32, size.height.max(1) as f32);
        window.request_redraw();

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.images = Some(images);
        self.executor = Some(executor);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.window.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.surface = (size.width.max(1) as f32, size.height.max(1) as f32);
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_to(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let key = match button {
                    MouseButton::Left => A_MOUSE1,
                    MouseButton::Right => A_MOUSE1 + 1,
                    MouseButton::Middle => A_MOUSE1 + 2,
                    _ => return,
                };
                self.key(key, state == ElementState::Pressed);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                // Escape closes the harness rather than the menu: the menu's
                // own ESC script would quit to a game that does not exist yet.
                if code == KeyCode::Escape && state == ElementState::Pressed {
                    self.finish();
                    event_loop.exit();
                    return;
                }
                if let Some(k) = map_key(code) {
                    self.key(k, state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                self.redraw(event_loop);
            }
            _ => {}
        }
    }
}

impl App {
    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let frame_data = self.ui_frame();
        // `RB_SetGL2D`'s shader clock: `backEnd.refdef.floatTime =
        // ri.Milliseconds() * 0.001f`.
        // Source: `oracle/codemp/renderer/tr_backend.cpp:1289-1291`
        let float_time = self.host.ui.uiDC.realTime as f32 * 0.001;
        let (Some(window), Some(gpu), Some(images), Some(executor)) = (
            self.window.as_ref(),
            self.gpu.as_mut(),
            self.images.as_mut(),
            self.executor.as_mut(),
        ) else {
            return;
        };

        match gpu.begin_frame() {
            Ok(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let stats = executor.execute_frame(
                    gpu,
                    &view,
                    &frame_data,
                    &self.host.sim.published,
                    &mut self.host.img_state,
                    images,
                    &mut self.host.font,
                    &self.host.noise,
                    float_time,
                    // No live cvar table in the harness, so the retail defaults
                    // apply.
                    RenderCvarSnapshot::default(),
                    // Menu frames draw 2D only; no world scene.
                    None,
                );
                if !self.reported {
                    self.reported = true;
                    report("first frame", &stats);
                }
                gpu.present(frame);
            }
            Err(_) => {
                let size = window.inner_size();
                gpu.resize(size.width, size.height);
            }
        }

        self.frames += 1;
        let over_time = self
            .max_seconds
            .is_some_and(|s| self.start.elapsed() >= Duration::from_secs(s));
        if self.frames >= self.max_frames || over_time {
            self.finish();
            event_loop.exit();
            return;
        }
        window.request_redraw();
    }

    /// The run's closing log line: frame count, the last frame's work, and
    /// every stubbed `DisplayContext` slot the menus reached for.
    fn finish(&mut self) {
        println!(
            "ui_harness: {} frames in {:.1}s",
            self.frames,
            self.start.elapsed().as_secs_f32()
        );
        let stubs = self.host.stubs.report();
        println!(
            "ui_harness: stubs reached — {}",
            if stubs.is_empty() { "none" } else { &stubs }
        );
    }
}

fn report(label: &str, stats: &FrameStats) {
    println!(
        "ui_harness: {label} — {} images uploaded, {} quads ({} glyphs across {} strings), \
         {} color changes, {} draw calls, {} events skipped, {} zero-pass pics",
        stats.images_uploaded,
        stats.quads,
        stats.glyphs,
        stats.strings,
        stats.color_changes,
        stats.draw_calls,
        stats.skipped_events(),
        stats.zero_pass_pics
    );
}

/// winit key -> Raven `A_*` keycode, for the keys a menu actually needs.
///
/// Source: `oracle/codemp/ui/keycodes.h`
fn map_key(code: KeyCode) -> Option<i32> {
    Some(match code {
        KeyCode::Enter | KeyCode::NumpadEnter => A_ENTER,
        KeyCode::Tab => A_TAB,
        KeyCode::Backspace => A_ESCAPE + 100, // A_BACKSPACE
        KeyCode::ArrowUp => A_CURSOR_UP,
        KeyCode::ArrowDown => A_CURSOR_DOWN,
        KeyCode::ArrowLeft => A_CURSOR_LEFT,
        KeyCode::ArrowRight => A_CURSOR_RIGHT,
        _ => return None,
    })
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut cfg = BootConfig::default();
    if let Some(basepath) = args.next() {
        cfg.basepath = basepath;
    }
    if let Some(menu) = args.next() {
        cfg.start_menu = menu;
    }

    let host = boot(&cfg);

    let event_loop = EventLoop::new().expect("EventLoop::new: failed to create the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(host);
    event_loop
        .run_app(&mut app)
        .expect("run_app: ui harness event loop exited with an error");
}
