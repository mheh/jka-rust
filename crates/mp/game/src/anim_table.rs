//! MP `animTable`: animation-name lookup table (`stringID_table_t[]`).
//!
//! The table lives at the bg tier, built from `mp_bg`'s `animNumber_t` and consumed there by
//! `bg_panimate`, `bg_saberLoad`, and `bg_vehicleLoad`.
//! This file re-exports it, so game importers and the prelude keep resolving `crate::anim_table::animTable` unchanged.
//! The home is `mp_bg::public::anim_table`.
//!
//! Source: `oracle/codemp/cgame/animtable.h:9-1789`

pub use mp_bg::public::anim_table::animTable;
