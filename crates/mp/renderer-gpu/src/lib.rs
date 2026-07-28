//! `mp_renderer_gpu` — the wgpu backend crate ruled by DEC-37 (R0 sit-down)
//! ruling 16 and DEC-44 (R4 kickoff).
//!
//! **This crate is R4's home.** `mp_renderer` stays CPU-only forever: assets,
//! shader-script parse, cull/sort, and `tr_model` all live there so
//! `jampded`'s dedicated-server link — and every CPU-only oracle-differential
//! golden in `mp_renderer`'s own test suite — stays GPU-free (`cargo build -p
//! mp_renderer` never touches `wgpu`/`winit`). This crate is the sibling that
//! owns the GPU: window/surface/device/queue and, eventually, the two
//! uber-shader backends (DEC-37 ruling 5) sit here, one door above
//! `mp_renderer`.
//!
//! DEC-37's threading topology (ruling 2) puts the render thread — not the
//! sim/VM thread — in charge of this crate's state: it owns the GPU and runs
//! cull → sort → skinning dispatch → encode → submit → present. The
//! state-partition law (ruling 3) is the hard edge: **no trap query may touch
//! GPU state**; every synchronous seam query (`CG_R_*`/`UI_R_*`) reads
//! `RenderAssets` (CPU-side, in `mp_renderer`) only, never anything owned by
//! [`Gpu`].
//!
//! DEC-44's stage order (ruling 2) lands backend #1 (faithful uber-shader)
//! first, gated per-slice: R4a is the 13-fn ui 2D command surface
//! (`R_RegisterShaderNoMip`, `R_DrawStretchPic`, `R_SetColor`, `R_Font_*`,
//! …) rendered end-to-end through this backend; backend #2 (PBR, materials
//! only per DEC-44 ruling 3) starts once the world slice (R3) is
//! gate-green. Reconciling `mp_renderer`'s `GpuResources` stub — the
//! CPU-registry-to-GPU-resource bridge — against this crate's device/queue
//! is an R4a design item, not scaffold scope: this file only stands up the
//! device/surface plumbing every later slice needs.
//! ## R4a wave 2 — real textures and text (this crate's current state)
//!
//! Landed: [`frame_exec`] walks a `FrameData` event stream in trap-call order,
//! [`pipeline2d`] rasterises quads through Raven's 640x480 virtual screen,
//! [`blend`] decodes `mp_renderer`'s `GLS_*` state bits into pipeline blend
//! states, and [`gpu_images`] uploads `R_CreateImage`'s staged pixels into
//! textures. A `DrawStretchPic` now resolves its shader to a real texture and
//! its stage's blend mode; a `DrawString` is laid out into glyph quads by
//! `tr_font`'s own per-glyph walk.
//!
//! **Staging: single-threaded first light.** DEC-37 ruling 2's sim/render
//! thread split is a later R4 slice — today the dev harness builds a
//! `FrameData` and executes it inline the same frame. The executor's signature
//! is already split-shaped (borrowed frame stream in, render-thread state
//! only), so that slice moves the caller, not this crate's API.
//!
//! Not yet rendered (counted and skipped, never panicked): the rotate-pic pair
//! and every scene-composition event. Uploads are level-0 only — `Upload32`'s
//! mipmap chain is still ahead.

pub mod blend;
pub mod frame_exec;
mod gpu;
pub mod gpu_images;
pub mod pipeline2d;
pub mod ui_host;

pub use blend::{blend_state_from_gls, ALPHA_BLEND, GLS_2D_DEFAULT};
pub use frame_exec::{FrameExecutor, FrameStats};
pub use gpu::{FrameError, Gpu};
pub use gpu_images::{GpuImage, GpuImages};
pub use pipeline2d::{Pipeline2d, QuadBatch, Rect, UvRect, SCREEN_HEIGHT, SCREEN_WIDTH};
