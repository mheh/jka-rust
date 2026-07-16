//! MP `animTable` — animation-name lookup table (`stringID_table_t[]`).
//!
//! Relocated to the bg tier (built from `mp_bg`'s `animNumber_t`; consumed by
//! `bg_panimate`/`bg_saberLoad`/`bg_vehicleLoad`) so the bg crate can reach it;
//! re-exported here so game importers and the prelude keep resolving
//! `crate::anim_table::animTable` unchanged. Canonical home:
//! `mp_bg::public::anim_table`.
//!
//! Source: `oracle/codemp/cgame/animtable.h:9-1789`

pub use mp_bg::public::anim_table::animTable;
