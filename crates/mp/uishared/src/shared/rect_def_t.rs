//! `RectDef` — Raven `rectDef_t`/`Rectangle`.

/// Raven `rectDef_t` (alias `Rectangle`) — a screen-space rectangle
/// (position + size).
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:112-119`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[doc(alias = "rectDef_t")]
#[doc(alias = "Rectangle")]
#[allow(non_snake_case)]
pub struct RectDef {
    /// horiz position
    pub x: f32,
    /// vert position
    pub y: f32,
    /// width
    pub w: f32,
    /// height;
    pub h: f32,
}
