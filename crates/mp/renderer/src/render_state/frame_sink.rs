//! `FrameSink` — the sim thread's end of the frame channel.

use std::sync::mpsc::{Receiver, SyncSender};

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_package::FramePackage;

/// The sim thread's two channel ends: packages out, emptied event buffers back.
///
/// `Engine.re` holds this as an `Option`. A dedicated server, a harness, and
/// every test leave it `None`, and `RE_EndFrame` then clears the event stream
/// in place exactly as it did before the render thread existed.
///
/// The return channel is the fixed buffer pool `R2-D8` earmarked. A returned
/// `FrameData` keeps its allocation, so a steady-state frame allocates nothing
/// for its event stream.
pub struct FrameSink {
    /// Bounded, so a render thread that falls behind paces the sim thread
    /// instead of losing frames.
    pub packages: SyncSender<FramePackage>,
    /// Emptied event buffers coming back. A `try_recv` miss just means the
    /// render thread has not finished the last frame, so the next send builds
    /// a fresh buffer.
    pub recycled: Receiver<FrameData>,
}
