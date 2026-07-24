//! MP UI public ABI types (`ui_public.h`).
//!
//! The `uiExport_t`/`uiImport_t` enum duplicates that once sat here were
//! deleted as unused: `MpUiExport`/`MpUiImport` (`../exports.rs`,
//! `../imports.rs`) are the live seam vocabularies (DEC-36 D8).

pub mod ui_client_state_t;
pub mod ui_menu_command_t;
