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
/// Source: `oracle/codemp/win32/win_wndproc.cpp:509,518,525`,
/// `oracle/codemp/win32/win_input.cpp:617`
#[derive(Clone, Copy, Debug)]
pub struct PlatformEvent {
    pub evType: sysEventType_t,
    pub evValue: c_int,
    pub evValue2: c_int,
}

/// The accumulated mouse delta, Raven's `IN_MouseMove` totals.
///
/// The window procedure reports many small motions per frame and Raven turned
/// them into one `SE_MOUSE` per `IN_Frame`. The pump adds here and the drain
/// takes the sum, so the ring still sees one event per frame.
///
/// Source: `oracle/codemp/win32/win_input.cpp:604-618`
#[derive(Default)]
struct MouseDelta {
    dx: AtomicI32,
    dy: AtomicI32,
}

/// The shared half of the bus: the mouse accumulator and the overflow flag.
#[derive(Default)]
struct PlatformShared {
    mouse: MouseDelta,
    /// Set when the pump had to drop an event, cleared once the drain reports it.
    overflowed: AtomicBool,
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
    /// Source: `oracle/codemp/win32/win_wndproc.cpp:509,518,525`
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
    /// Source: `oracle/codemp/win32/win_input.cpp:604-618`
    pub fn add_mouse_delta(&self, dx: c_int, dy: c_int) {
        self.shared.mouse.dx.fetch_add(dx, Ordering::Relaxed);
        self.shared.mouse.dy.fetch_add(dy, Ordering::Relaxed);
    }
}

impl PlatformEventSource {
    /// Take the next queued key or char event, or `None` when the pump is idle.
    pub fn next_event(&self) -> Option<PlatformEvent> {
        self.events.try_recv().ok()
    }

    /// Take the frame's accumulated mouse delta and reset it. Raven returns
    /// early on a zero delta, so a zero pair means "queue nothing".
    ///
    /// Source: `oracle/codemp/win32/win_input.cpp:613-617`
    pub fn take_mouse_delta(&self) -> (c_int, c_int) {
        let dx = self.shared.mouse.dx.swap(0, Ordering::Relaxed);
        let dy = self.shared.mouse.dy.swap(0, Ordering::Relaxed);
        (dx, dy)
    }

    /// Report and clear the overflow flag, so the drain warns once per burst.
    pub fn take_overflow(&self) -> bool {
        self.shared.overflowed.swap(false, Ordering::Relaxed)
    }
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
        sink.add_mouse_delta(3, -4);
        sink.add_mouse_delta(1, 2);
        assert_eq!(source.take_mouse_delta(), (4, -2));
        assert_eq!(source.take_mouse_delta(), (0, 0));
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
