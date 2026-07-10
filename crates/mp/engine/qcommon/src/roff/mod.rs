//! `CROFFSystem` — ROFF (Raven Object File Format) caching, playback, and
//! clean-up.
//!
//! C++-track idiomatic reimplementation (porting-rules §F) of
//! `oracle/codemp/qcommon/RoffSystem.{h,cpp}`, ported against the FROZEN design
//! `docs/subsystems/roff.md`. Ported under the **WinDed Release macro set**
//! (`-DNDEBUG -DDEDICATED -DBOTLIB`, ROFF-D3): the `#ifndef DEDICATED`
//! client-only branches (ApplyROFF `:835-843`, ClearLerp `:981-989`, ProcessNote
//! `:951-952`, the client syscall/getter twins) are §20 zero-caller drops, not
//! stubs.
//!
//! - `CROFFSystem` (the one Raven global `theROFFSystem`,
//!   `RoffSystem.cpp:8`, `.h:161,183`) → [`RoffSystem`], the aggregate owner:
//!   the id-keyed cache map, the per-entity playback `Vec`, and the unique-ID
//!   counter. Attached as a direct `Engine.roff` field (ROFF-D2, RULING 12),
//!   sibling of `Engine.cm`. Its upward services (FS reads, entity access,
//!   `svs.time`, the note-track `VM_Call`) are reached through the one
//!   [`mp_host_interface::EngineHost`] services trait (ROFF-D2, RULING 11),
//!   threaded as `&mut impl EngineHost` — never ambient state (§B3).
//! - `CROFFSystem::CROFF` → [`croff::Croff`] (one cached `.rof`).
//! - `CROFFSystem::SROFFEntity` → [`sroff_entity::SroffEntity`] (one playback
//!   entry).
//! - `TROFFHeader`/`TROFFEntry`/`TROFF2Header`/`TROFF2Entry` →
//!   [`mod@header`]: the `#[repr(C)]` on-disk v1/v2 layouts, used only for
//!   bit-exact parsing (ROFF-D4).
//!
//! Type definition source: `oracle/codemp/qcommon/RoffSystem.h:35-181`

use std::collections::BTreeMap;

pub mod croff;
pub mod header;
pub mod roff_system;
pub mod sroff_entity;

pub use croff::Croff;
pub use sroff_entity::SroffEntity;

/// Raven `ROFF_VERSION` — supported version-1 `.rof` file version number.
///
/// Source: `oracle/codemp/qcommon/RoffSystem.h:24`
pub const ROFF_VERSION: i32 = 1;

/// Raven `ROFF_NEW_VERSION` — supported version-2 `.rof` file version number.
///
/// Source: `oracle/codemp/qcommon/RoffSystem.h:25`
pub const ROFF_NEW_VERSION: i32 = 2;

/// Raven `ROFF_STRING` — the four-char header magic every `.rof` begins with.
///
/// Raven: "should match roff_string defined above". `IsROFF` compares it with
/// `!strcmp(hdr->mHeader, ROFF_STRING)` reading `mHeader` (`char[4]`, no NUL) as
/// a C-string that runs into `mVersion`'s low byte (ROFF-V1) — that faithful,
/// accidentally-passing compare lives in the parser, not here.
///
/// Source: `oracle/codemp/qcommon/RoffSystem.h:26`
pub const ROFF_STRING: &str = "ROFF";

/// Raven `ROFF_SAMPLE_RATE` — 10 Hz. v1 roffs default `mFrameTime =
/// 1000/ROFF_SAMPLE_RATE` (100 ms) and `mLerp = ROFF_SAMPLE_RATE`.
///
/// Source: `oracle/codemp/qcommon/RoffSystem.h:27`
pub const ROFF_SAMPLE_RATE: i32 = 10;

/// Raven `CROFFSystem theROFFSystem` — the single ROFF playback/cache singleton.
///
/// The aggregate owner: `mROFFList` (cached roffs, keyed by unique id), `mID`
/// (the id generator), and `mROFFEntList` (the roffing entities). Its methods —
/// the five seam arms (`clean`/`update_entities`/`cache`/`play`/`purge_ent`,
/// frozen in `## Seam definition`) plus the private parse/playback/cleanup
/// helpers — are transcribed in [`mod@roff_system`] as inherent `impl` blocks on
/// this type (same-crate, so those methods reach the private fields below).
///
/// `mROFFList` is an **ordered** map (ascending-id iteration is behaviour-visible
/// via `List`/`GetID`, ROFF-D4); `mROFFEntList` is a `Vec` walked in insertion
/// order by `UpdateEntities`. State is owned, never raw-pointer-aliased (§B5):
/// cache entries are reached by id, playback entries by index.
///
/// `Default` mirrors Raven's ctor (`mID = 0`, cleared lists,
/// `RoffSystem.cpp:161`); the owned `BTreeMap`/`Vec` supersede the dtor's
/// `Restart()` free (`:162`).
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:35-51`;
/// instance `oracle/codemp/qcommon/RoffSystem.cpp:8`, `.h:161,183`
#[derive(Default)]
pub struct RoffSystem {
    /// Raven `TROFFList mROFFList` — the `map<int, CROFF*>` of cached roffs,
    /// keyed by the unique id `NewID` mints. `BTreeMap` preserves the ordered
    /// map's ascending-id iteration (ROFF-D4).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:45,48`
    roff_list: BTreeMap<i32, Croff>,

    /// Raven `int mID` — the unique-id generator for new roff objects. `NewID`
    /// is `++mID`, so it increments before returning and never mints 0
    /// (`RoffSystem.h:146`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:49`
    id: i32,

    /// Raven `TROFFEntList mROFFEntList` — the `vector<SROFFEntity*>` of roffing
    /// entities, walked in insertion order by `UpdateEntities`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:46,51`
    roff_ent_list: Vec<SroffEntity>,
}
