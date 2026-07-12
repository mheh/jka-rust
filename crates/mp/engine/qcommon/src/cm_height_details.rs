//! `CmHeightDetails` (Raven `CCMHeightDetails`) — the per-terxel-height
//! surface/contents detail record.
//!
//! §F idiomatic reimplementation (porting-rules §17-21); own file per ruling
//! 39d (§21 one-Raven-class-per-file, beside `cm_terrain.rs`/`cm_patch.rs`).
//! A private construction/collision detail of `CmLandScape`: Raven stores
//! `HEIGHT_RESOLUTION` (`cm_landscape.h:13`, 256) of these in the owner's
//! `mHeightDetails` array (`cm_landscape.h:165`) — that array is a field of
//! `CmLandScape` (`cm_terrain.rs`), not repeated here. Never appears in any
//! pub Seam signature (Files roster, `rmg-terrain.md:1547`).
//!
//! Internal-only (not ABI-crossing): no `#[repr(C)]`/layout asserts required
//! (porting-rules §D12).
//!
//! Class definition source: `oracle/codemp/qcommon/cm_landscape.h:75-88`

/// Raven `CCMHeightDetails`.
///
/// Raven: "Surfaceflags per height" (`cm_landscape.h:165` field comment on the
/// owning array). Live: `CmLandScape::SetShaders` (`cm_terrain.cpp:30-35`)
/// reads `GetSurfaceFlags`/writes `SetFlags` per terxel height while parsing
/// `LoadTerrainDef`'s `altitudetexture` shader; `CmLandScape::GetSurfaceFlags`/
/// `GetContentFlags` (`cm_landscape.h:225-226`) forward to this type's
/// `GetSurfaceFlags`/`GetContents`, read from the live patch-collision build
/// (`cm_terrain.cpp:584`).
///
/// Raven's trivial empty ctor/dtor (`cm_landscape.h:81-82`) are **not** ported
/// as methods: the real zero-initialization of the whole
/// `mHeightDetails[HEIGHT_RESOLUTION]` array happens via
/// `memset(mHeightDetails, 0, sizeof(CCMHeightDetails) * HEIGHT_RESOLUTION)`
/// in `CCMLandScape::CCMLandScape` (`cm_terrain.cpp:125`), bypassing the
/// per-element ctor entirely — so this struct derives `Default` (all-zero
/// fields), the well-defined Rust equivalent of that memset (§19: Raven's
/// bare `int` fields are otherwise uninitialized between the no-op ctor and
/// the memset, a UB window with no observable effect here).
///
/// Type definition source: `oracle/codemp/qcommon/cm_landscape.h:75-88`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CmHeightDetails {
    /// Raven `mContents` — contents flags for this height band.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:78`
    contents: i32,

    /// Raven `mSurfaceFlags` — surface flags for this height band.
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:79`
    surface_flags: i32,
}

impl CmHeightDetails {
    // Accessors (Raven's own grouping comment, `cm_landscape.h:84`).

    /// Raven `CCMHeightDetails::GetSurfaceFlags`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:85`
    pub fn get_surface_flags(&self) -> i32 {
        self.surface_flags
    }

    /// Raven `CCMHeightDetails::GetContents`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:86`
    pub fn get_contents(&self) -> i32 {
        self.contents
    }

    /// Raven `CCMHeightDetails::SetFlags`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:87`
    pub fn set_flags(&mut self, con: i32, sf: i32) {
        self.contents = con;
        self.surface_flags = sf;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default mirrors Raven's ctor-bypassing `memset(..., 0, ...)` zero-init
    /// (`cm_terrain.cpp:125`) — all fields zero, not the no-op ctor.
    #[test]
    fn default_is_zeroed() {
        let d = CmHeightDetails::default();
        assert_eq!(d.get_contents(), 0);
        assert_eq!(d.get_surface_flags(), 0);
    }

    #[test]
    fn set_flags_then_read_back() {
        let mut d = CmHeightDetails::default();
        d.set_flags(7, -3);
        assert_eq!(d.get_contents(), 7);
        assert_eq!(d.get_surface_flags(), -3);
    }
}
