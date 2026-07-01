#![allow(non_camel_case_types)]

/// Raven `ForceReload_e` — dev-time forced reload/uncache of certain filetypes.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2692-2699`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceReload_e {
    eForceReload_NOTHING,
    eForceReload_BSP,
    eForceReload_MODELS,
    eForceReload_ALL,
}
