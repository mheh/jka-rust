#![allow(non_camel_case_types)]

/// Raven `ForceReload_e` development force-reload/uncache selector.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:3166-3173`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceReload_e {
    eForceReload_NOTHING,
    // eForceReload_BSP,	// Raven: not used in MP codebase
    eForceReload_MODELS,
    eForceReload_ALL,
}
