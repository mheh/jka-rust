//! `be_aas_def.h`-local constants.
//!
//! Source: `oracle/codemp/botlib/be_aas_def.h:38-39`

/// Raven `MAX_PATH` — aliases `MAX_QPATH` when not already defined (guarded
/// `#ifndef MAX_PATH`, active on this build).
///
/// Source: `oracle/codemp/botlib/be_aas_def.h:38-39`
pub const MAX_PATH: usize = mp_qshared::shared::MAX_QPATH;
