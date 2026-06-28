//! Single-player UI export enum vocabulary.
//!
//! Raven `oracle/oracle/code/ui/ui_public.h` does not define a `uiExport_t` enum for this surface.
//! The SP UI function-table ABI is intentionally not ported in this enum-only pass.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpUiExport {}
