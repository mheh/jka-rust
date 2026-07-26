//! MP UI public ABI types (`ui_public.h`).
//!
//! The `uiExport_t`/`uiImport_t` enum duplicates that once sat here were
//! deleted as unused: `MpUiExport`/`MpUiImport` (`../exports.rs`,
//! `../imports.rs`) are the live seam vocabularies (DEC-36 D8).

use core::ffi::c_int;

pub mod ui_client_state_t;
pub mod ui_menu_command_t;

/// Raven MP `UI_API_VERSION` — the ui module's `UI_GETAPIVERSION` return
/// value contract.
///
/// Source: `oracle/codemp/ui/ui_public.h:6`
pub const UI_API_VERSION: c_int = 7;
