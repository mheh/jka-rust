//! `WindowDef` — Raven `windowDef_t`/`Window`.

use core::ffi::c_int;

use mp_qshared::shared::{qhandle_t, vec4_t};

use super::rect_def_t::RectDef;

/// Raven `windowDef_t` (alias `Window`) — the positional/border/style block
/// shared by menus and items.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:122-146`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "windowDef_t")]
#[doc(alias = "Window")]
#[allow(non_snake_case)]
pub struct WindowDef {
    /// client coord rectangle
    pub rect: RectDef,
    /// screen coord rectangle
    pub rectClient: RectDef,
    pub name: String,
    /// if it belongs to a group
    pub group: String,
    /// cinematic name
    pub cinematicName: String,
    /// cinematic handle
    pub cinematic: c_int,
    pub style: c_int,
    pub border: c_int,
    /// ownerDraw style
    pub ownerDraw: c_int,
    /// show flags for ownerdraw items
    pub ownerDrawFlags: c_int,
    pub borderSize: f32,
    /// visible, focus, mouseover, cursor
    pub flags: c_int,
    /// for various effects
    pub rectEffects: RectDef,
    /// for various effects
    pub rectEffects2: RectDef,
    /// time based value for various effects
    pub offsetTime: c_int,
    /// time next effect should cycle
    pub nextTime: c_int,
    /// text color
    pub foreColor: vec4_t,
    /// border color
    pub backColor: vec4_t,
    /// border color
    pub borderColor: vec4_t,
    /// border color
    pub outlineColor: vec4_t,
    /// background asset
    pub background: qhandle_t,
}
