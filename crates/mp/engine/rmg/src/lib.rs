//! `mp_engine_rmg` crate — Raven `codemp/RMG/*` (random mission generator).
//!
//! Only the faithful C enums are ported here. The RMG C++ classes
//! (`CRMManager`, `CRMMission`, `CRMInstance*`, `CRMPathManager`,
//! `CRMLandScape`, ...) are the idiomatic C++ track, alongside the terrain
//! classes OpenJK dropped entirely.
//!
//! Under RMG-D1 (ruling 25) the generation subtree is dead code on the DEDICATED
//! engine; only the LIVE `RmManager` lifecycle (through `LoadMission`'s early-out)
//! is ported — `rm_manager.rs` (Files roster, class `CRMManager`).

pub mod rm_headers;
pub mod rm_manager;
pub mod rm_path;
