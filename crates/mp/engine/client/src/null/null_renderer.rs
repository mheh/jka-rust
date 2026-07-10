//! Raven's `null` renderer stubs — the DEDICATED/no-renderer build's
//! `RB_StageIterator*`/`R_SyncRenderThread` entry points, every body an
//! intentional no-op.
//!
//! Source: `oracle/codemp/null/null_renderer.cpp`

/// Raven `RB_StageIteratorGeneric`.
///
/// Source: `oracle/codemp/null/null_renderer.cpp:3-5`
pub fn RB_StageIteratorGeneric() {}

/// Raven `RB_StageIteratorSky`.
///
/// Source: `oracle/codemp/null/null_renderer.cpp:7-9`
pub fn RB_StageIteratorSky() {}

/// Raven `RB_StageIteratorVertexLitTexture`.
///
/// Source: `oracle/codemp/null/null_renderer.cpp:11-13`
pub fn RB_StageIteratorVertexLitTexture() {}

/// Raven `RB_StageIteratorLightmappedMultitexture`.
///
/// Source: `oracle/codemp/null/null_renderer.cpp:15-17`
pub fn RB_StageIteratorLightmappedMultitexture() {}

/// Raven `R_SyncRenderThread`.
///
/// Source: `oracle/codemp/null/null_renderer.cpp:19-21`
pub fn R_SyncRenderThread() {}
