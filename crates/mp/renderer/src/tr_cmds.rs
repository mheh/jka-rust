//! Raven `tr_cmds.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_cmds.cpp`

/// Raven `R_InitCommandBuffers` — command-buffer subsystem init.
///
/// Raven: retail body is empty; command-buffer state is now `FrameData`,
/// built per render pass rather than a persistent buffer requiring init.
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:72-73`
pub fn r_init_command_buffers() {}

/// Raven `R_ShutdownCommandBuffers` — command-buffer subsystem shutdown.
///
/// Raven: retail body is empty.
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:80-81`
pub fn r_shutdown_command_buffers() {}

// DEFERRED: R_GetCommandBuffer — `backEndData_t` (and its byte-packed
// `renderCommandList_t`/`cmds` buffer this fn hand-allocates slices of)
// dissolves under R2; `commands` IS `FrameData.events: Vec<FrameEvent>`.
// RC_* command payloads cross as typed `FrameEvent` variants pushed directly
// onto `FrameData.events` in their owning waves, never through a raw
// byte-buffer allocator — a Rust equivalent of this fn's `void*`-slice-
// carving would require raw pointers, banned by the interior-safety law.
// (R2 `## State ownership` row `backEndData`; R2 `### A1 disposition table`)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:140-160`
