//! `mp_engine_rmg` crate — Raven `codemp/RMG/*` (random mission generator).
//!
//! Only the faithful C enums are ported here. The RMG C++ classes
//! (`CRMManager`, `CRMMission`, `CRMInstance*`, `CRMPathManager`,
//! `CRMLandScape`, ...) are the idiomatic C++ track, alongside the terrain
//! classes OpenJK dropped entirely.
//TODO: Port CRMManager (C++ track)
// Source: oracle/oracle/codemp/RMG/RM_Manager.h

pub mod rm_headers;
pub mod rm_path;
