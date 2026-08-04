//! The window-to-sim event crossing (DEC-56.3).
//!
//! Raven pumped window messages inside `Sys_GetEvent` and let the window
//! procedure call `Sys_QueEvent` on the same thread. macOS makes the main
//! thread own the event loop, so the pump lives there and the sim thread drains
//! this bus into the frozen `SysEventQueue` ring at the two places Raven pumped:
//! the message loop slot and the `IN_Frame` slot.
//!
//! Source: `oracle/codemp/win32/win_main.cpp:1224-1235`,
//! `oracle/codemp/unix/unix_main.c:1007-1009,1027-1028`

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;

use crate::qcommon::sys_event_type_t::sysEventType_t;

/// Events the pump may hold before the sim thread drains them.
///
/// Raven's own ring is 256 entries and overflows loudly, so the bus matches it.
pub const PLATFORM_EVENT_QUEUE: usize = 256;

/// One window-sourced event: the pointer-free subset of `sysEvent_t`.
///
/// The three window event types Raven queues from its window procedure
/// (`SE_KEY`, `SE_CHAR`, `SE_MOUSE`) always pass a NULL payload, so the crossing
/// carries no pointer and stays `Send`. The payload-bearing types (`SE_CONSOLE`,
/// `SE_PACKET`) never come from the window and keep their existing sim-thread
/// path.
///
/// Source: `oracle/codemp/win32/win_wndproc.cpp:521,531,537`,
/// `oracle/codemp/win32/win_input.cpp:617`
#[derive(Clone, Copy, Debug)]
pub struct PlatformEvent {
    pub evType: sysEventType_t,
    pub evValue: c_int,
    pub evValue2: c_int,
}

/// Thousandths of a mouse count per accumulator unit. A raw delta is a float
/// and a queued `SE_MOUSE` is an int, so the accumulator keeps three decimal
/// places and the drain carries the remainder into the next frame.
const MOUSE_SCALE: c_int = 1000;

/// The accumulated mouse delta, Raven's `IN_MouseMove` totals, in `MOUSE_SCALE`
/// units.
///
/// The window procedure reports many small motions per frame and Raven turned
/// them into one `SE_MOUSE` per `IN_Frame`. The pump adds here and the drain
/// takes the whole counts, so the ring still sees one event per frame.
///
/// Source: `oracle/codemp/win32/win_input.cpp:604-618`
#[derive(Default)]
struct MouseDelta {
    dx: AtomicI32,
    dy: AtomicI32,
}

/// The shared half of the bus: the mouse accumulator, the quit request, the
/// overflow flag, the window's drawable size, and the two mouse-capture flags.
#[derive(Default)]
struct PlatformShared {
    mouse: MouseDelta,
    /// Set when the window closed. Raven read the same signal as a `WM_QUIT`
    /// from `GetMessage` and answered it with `Com_Quit_f`.
    /// Source: `oracle/codemp/win32/win_main.cpp:1226-1228`
    quit: AtomicBool,
    /// Set when the pump had to drop an event, cleared once the drain reports it.
    overflowed: AtomicBool,
    /// The window's drawable width, in physical pixels.
    /// It stays zero until the pump creates the window.
    ///
    /// The sim thread reads it once at renderer boot, the port's `GLimp_Init` stand-in.
    ///
    /// Source: `oracle/codemp/win32/win_glimp.cpp:713` (`GLimp_Init`, via `GLW_SetMode`'s `R_GetModeInfo`)
    drawable_width: AtomicI32,
    /// The paired height for `drawable_width`.
    drawable_height: AtomicI32,
    /// Set while the window has focus, Raven's `in_appactive`.
    /// This one runs pump to sim: the pump writes it from the focus event and the sim thread reads it.
    /// Source: `oracle/codemp/win32/win_input.cpp:70,690-702`
    app_active: AtomicBool,
    /// Set while the pointer must be captured, the answer `IN_Frame` reached before it called `IN_ActivateMouse` or `IN_DeactivateMouse`.
    /// This one runs sim to pump: the decision needs the key catchers, which only the sim thread holds, and the window only the pump can touch.
    /// Source: `oracle/codemp/win32/win_input.cpp:714-739`
    mouse_active: AtomicBool,
}

/// The pump half, owned by the main thread.
///
/// A send never blocks: the main thread must keep servicing the window, so a
/// full queue drops the event and raises the overflow flag, the same posture
/// Raven's ring takes when it overruns.
pub struct PlatformEventSink {
    events: SyncSender<PlatformEvent>,
    shared: Arc<PlatformShared>,
}

/// The drain half, held by `Common` and read inside `Sys_GetEvent`.
pub struct PlatformEventSource {
    events: Receiver<PlatformEvent>,
    shared: Arc<PlatformShared>,
}

/// Build one bus. The sink goes to the pump thread, the source into `Common`.
pub fn platform_event_bus() -> (PlatformEventSink, PlatformEventSource) {
    let (tx, rx) = sync_channel(PLATFORM_EVENT_QUEUE);
    let shared = Arc::new(PlatformShared::default());
    let sink = PlatformEventSink {
        events: tx,
        shared: Arc::clone(&shared),
    };
    let source = PlatformEventSource { events: rx, shared };
    (sink, source)
}

impl PlatformEventSink {
    /// Queue one key or char event. Raven's window procedure queues both from
    /// `MainWndProc`, so this is that call.
    ///
    /// Raven's own ring drops the OLDEST event on overflow, which cannot strand
    /// a key down without its up. A `SyncSender` can only refuse the newest, so
    /// a burst past 256 events between two drains could strand one. The sim
    /// drains every frame, so the queue holds one frame of input.
    ///
    /// Source: `oracle/codemp/win32/win_wndproc.cpp:521,531,537`
    pub fn queue(&self, event: PlatformEvent) {
        match self.events.try_send(event) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.shared.overflowed.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Add one raw mouse motion to the frame's running total.
    ///
    /// The delta arrives as a float and Raven read whole DirectInput counts, so
    /// the fraction stays in the accumulator instead of truncating away. Without
    /// that, a slow drag of sub-unit motions would report no movement at all.
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:604-618`
    pub fn add_mouse_delta(&self, dx: f64, dy: f64) {
        add_milli(&self.shared.mouse.dx, dx);
        add_milli(&self.shared.mouse.dy, dy);
    }

    /// Ask the engine to quit, Raven's `WM_QUIT` answer.
    ///
    /// Source: `oracle/codemp/win32/win_main.cpp:1226-1228`
    pub fn request_quit(&self) {
        self.shared.quit.store(true, Ordering::Relaxed);
    }

    /// Publish the window's drawable size, in physical pixels.
    ///
    /// The pump calls this once the window exists, and again on every resize.
    /// The sim thread reads it once at renderer boot, the port's `GLimp_Init` stand-in.
    ///
    /// Source: `oracle/codemp/win32/win_glimp.cpp:713` (`GLimp_Init`, via `GLW_SetMode`'s `R_GetModeInfo`)
    pub fn publish_drawable_size(&self, width: c_int, height: c_int) {
        self.shared.drawable_width.store(width, Ordering::Relaxed);
        self.shared.drawable_height.store(height, Ordering::Relaxed);
    }

    /// Report whether the window has focus, Raven's `IN_Activate` call from `WM_ACTIVATE`.
    ///
    /// Raven also dropped the flag for a minimized window.
    /// macOS takes focus away when a window minimizes, so focus alone carries the decision here.
    ///
    /// Source: `oracle/codemp/win32/win_wndproc.cpp:71-95,404-414`,
    /// `oracle/codemp/win32/win_input.cpp:690-702`
    pub fn publish_app_active(&self, active: bool) {
        self.shared.app_active.store(active, Ordering::Relaxed);
    }

    /// Read the sim thread's mouse-capture decision, which the pump applies to the window.
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:714-739`
    pub fn mouse_active(&self) -> bool {
        self.shared.mouse_active.load(Ordering::Relaxed)
    }
}

/// Add one float delta to a thousandths-of-a-count accumulator.
fn add_milli(slot: &AtomicI32, value: f64) {
    slot.fetch_add(
        (value * MOUSE_SCALE as f64).round() as c_int,
        Ordering::Relaxed,
    );
}

impl PlatformEventSource {
    /// Take the next queued key or char event, or `None` when the pump is idle.
    pub fn next_event(&self) -> Option<PlatformEvent> {
        self.events.try_recv().ok()
    }

    /// Take the frame's whole mouse counts, leaving the fraction for the next
    /// frame. Raven returns early on a zero delta, so a zero pair means "queue
    /// nothing".
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:613-617`
    pub fn take_mouse_delta(&self) -> (c_int, c_int) {
        (
            take_whole(&self.shared.mouse.dx),
            take_whole(&self.shared.mouse.dy),
        )
    }

    /// Report and clear the quit request the window close raised.
    pub fn take_quit(&self) -> bool {
        self.shared.quit.swap(false, Ordering::Relaxed)
    }

    /// Report and clear the overflow flag, so the drain warns once per burst.
    pub fn take_overflow(&self) -> bool {
        self.shared.overflowed.swap(false, Ordering::Relaxed)
    }

    /// Read the window's drawable size, in physical pixels.
    ///
    /// Both values stay zero until the pump creates the window.
    pub fn drawable_size(&self) -> (c_int, c_int) {
        (
            self.shared.drawable_width.load(Ordering::Relaxed),
            self.shared.drawable_height.load(Ordering::Relaxed),
        )
    }

    /// Read whether the window has focus, Raven's `in_appactive`.
    ///
    /// It stays false until the pump reports the first focus event, the same start Raven's zero-initialized global had.
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:70,690-702`
    pub fn app_active(&self) -> bool {
        self.shared.app_active.load(Ordering::Relaxed)
    }

    /// Publish this frame's mouse-capture decision for the pump to apply.
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:714-739`
    pub fn publish_mouse_active(&self, active: bool) {
        self.shared.mouse_active.store(active, Ordering::Relaxed);
    }
}

/// Take the whole counts out of a `MOUSE_SCALE` accumulator and leave the
/// fraction behind.
fn take_whole(slot: &AtomicI32) -> c_int {
    let total = slot.load(Ordering::Relaxed);
    let whole = total / MOUSE_SCALE;
    slot.fetch_sub(whole * MOUSE_SCALE, Ordering::Relaxed);
    whole
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_events_cross_in_order() {
        let (sink, source) = platform_event_bus();
        for value in 0..3 {
            sink.queue(PlatformEvent {
                evType: sysEventType_t::SE_KEY,
                evValue: value,
                evValue2: 1,
            });
        }
        let drained: Vec<c_int> = std::iter::from_fn(|| source.next_event())
            .map(|e| e.evValue)
            .collect();
        assert_eq!(drained, vec![0, 1, 2]);
    }

    #[test]
    fn mouse_deltas_sum_and_reset() {
        let (sink, source) = platform_event_bus();
        sink.add_mouse_delta(3.0, -4.0);
        sink.add_mouse_delta(1.0, 2.0);
        assert_eq!(source.take_mouse_delta(), (4, -2));
        assert_eq!(source.take_mouse_delta(), (0, 0));
    }

    #[test]
    fn sub_count_motion_accumulates_instead_of_vanishing() {
        let (sink, source) = platform_event_bus();
        for _ in 0..4 {
            sink.add_mouse_delta(0.4, -0.4);
        }
        // Four quarter-counts make one whole count each way, and the remainder
        // waits for the next frame rather than truncating to nothing.
        assert_eq!(source.take_mouse_delta(), (1, -1));
        for _ in 0..2 {
            sink.add_mouse_delta(0.4, -0.4);
        }
        assert_eq!(source.take_mouse_delta(), (1, -1));
    }

    #[test]
    fn a_quit_request_reports_once() {
        let (sink, source) = platform_event_bus();
        assert!(!source.take_quit());
        sink.request_quit();
        assert!(source.take_quit());
        assert!(!source.take_quit());
    }

    #[test]
    fn the_two_mouse_capture_flags_cross_both_ways() {
        let (sink, source) = platform_event_bus();
        // Both start false: no focus reported yet, and no capture decided yet.
        assert!(!source.app_active());
        assert!(!sink.mouse_active());

        sink.publish_app_active(true);
        assert!(source.app_active());

        source.publish_mouse_active(true);
        assert!(sink.mouse_active());
        // The decision is a level, not an edge, so a read leaves it standing.
        assert!(sink.mouse_active());
    }

    #[test]
    fn a_full_queue_drops_and_flags() {
        let (sink, source) = platform_event_bus();
        for _ in 0..PLATFORM_EVENT_QUEUE + 1 {
            sink.queue(PlatformEvent {
                evType: sysEventType_t::SE_CHAR,
                evValue: b'x' as c_int,
                evValue2: 0,
            });
        }
        assert!(source.take_overflow());
        assert!(!source.take_overflow());
    }
}
