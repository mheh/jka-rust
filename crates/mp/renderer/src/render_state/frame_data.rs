//! `FrameData` — one frame's ordered event stream (`R2-D2`/A1).

use crate::render_state::frame_event::FrameEvent;

/// The frame's 2D/scene event stream, in trap-call order — what replaces the
/// oracle's `backEndData_t` double-buffer and its byte-packed
/// `renderCommandList_t` (`### A1 disposition table`).
///
/// The per-frame append counters the oracle keeps as file-scope statics
/// (`r_numentities`/`r_numdlights`/`r_numpolys`/`r_numpolyverts`,
/// `oracle/codemp/renderer/tr_scene.cpp:21-33`) are properties of the
/// `FrameData` under construction, not fields: an append's bound check reads
/// only sim-side `RenderAssets` state plus this stream (`### FrameData`'s
/// append-validation principle). Buffer recycling (fixed pool vs. free-list)
/// settles at R4 — `R2-D8` records the lean default (a fixed 2-3 buffer pool
/// with an explicit return channel); R2 freezes only the stream shape.
///
/// Source: `oracle/codemp/renderer/tr_local.h:2261-2273`
pub struct FrameData {
    pub events: Vec<FrameEvent>,
}
