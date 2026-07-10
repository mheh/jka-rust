//! `CROFFSystem::SROFFEntity` — one playback-list entry.

use mp_qshared::shared::vec3_t;

/// Raven `CROFFSystem::SROFFEntity` — one entry of `mROFFEntList`: an entity
/// currently being driven by a cached roff.
///
/// Plain data, same as Raven: the struct declares no methods of its own — all
/// manipulation (`Play` constructs one, `ApplyROFF`/`UpdateEntities` advance and
/// retire it) lives on `RoffSystem` in [`super::roff_system`] (same crate, so
/// its methods reach these fields; `pub(crate)` keeps them out of the public
/// seam). Not `#[repr(C)]`: this type never crosses the module ABI, only lives
/// inside `RoffSystem`'s owned `Vec` (§B5 — by index, never a raw pointer).
///
/// Class definition source: `oracle/codemp/qcommon/RoffSystem.h:125-139`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SroffEntity {
    /// Raven `mEntID` — the entity number currently being roffed, resolved via
    /// `SV_GentityNum` (the `EngineHost` entity service). Matches the seam's
    /// `ent_id: i32` (not an arena `EntityId`: this indexes the shared
    /// `sharedEntity_t` table, not a game-side entity arena).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:128`
    pub(crate) m_ent_id: i32,

    /// Raven `mROFFID` — the id of the cached roff (`RoffSystem::roff_list` key)
    /// applied to `m_ent_id`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:130`
    pub(crate) m_roff_id: i32,

    /// Raven `mNextROFFTime` — the `svs.time` at which the next frame advance
    /// is due.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:131`
    pub(crate) m_next_roff_time: i32,

    /// Raven `mROFFFrame` — the current roff frame index being applied.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:132`
    pub(crate) m_roff_frame: i32,

    /// Raven `mKill` — flag to kill (retire) a roffing entity; set by
    /// `UpdateEntities` when `ApplyROFF` finishes or its roff is missing, and
    /// swept in a second pass.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:134`
    pub(crate) m_kill: bool,

    /// Raven `mSignal` — reserved for signalling ICARUS when a roff completes.
    /// Raven note: the signal was never hooked up; `Play` sets it `true` but
    /// nothing reads it (dead field, kept for layout fidelity — no live caller
    /// to drop per §20, so it stays rather than being invented a use).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:135`
    pub(crate) m_signal: bool,

    /// Raven `mTranslated` — whether this roff's origin offset is rotated to
    /// fit the entity's initial orientation (`ApplyROFF`'s
    /// `AngleVectors`/`VectorMA` branch). Set from `Play`'s `doTranslation`
    /// (the seam's `do_translation` param).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:136`
    pub(crate) m_translated: bool,

    /// Raven `mIsClient` — set from `Play`'s `isClient` (the seam's
    /// `is_client` param); `UpdateEntities` skips entries whose flag doesn't
    /// match the pass it's running. Under the WinDed/DEDICATED macro set every
    /// live caller passes `false` (ROFF-D3), so this is always `false` at
    /// runtime, but the field and comparison are kept for signature fidelity.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:137`
    pub(crate) m_is_client: bool,

    /// Raven `mStartAngles` — the entity's `s.apos.trBase` at `Play` time,
    /// copied via `VectorCopy`; used by `ApplyROFF` to rotate the per-frame
    /// origin offset when `m_translated` is set.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:138`
    pub(crate) m_start_angles: vec3_t,
}
