//! `ScrollInfo` — Raven `scrollInfo_s`/`scrollInfo_t`.

use core::ffi::c_int;

use super::item_id::ItemId;

/// Raven `SCROLL_TIME_START`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:19`
pub const SCROLL_TIME_START: c_int = 500;
/// Raven `SCROLL_TIME_ADJUST`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:20`
pub const SCROLL_TIME_ADJUST: c_int = 150;
/// Raven `SCROLL_TIME_ADJUSTOFFSET`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:21`
pub const SCROLL_TIME_ADJUSTOFFSET: c_int = 40;
/// Raven `SCROLL_TIME_FLOOR`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:22`
pub const SCROLL_TIME_FLOOR: c_int = 20;

/// Raven `scrollInfo_s` (typedef `scrollInfo_t`) — the one in-flight
/// listbox/textscroll/slider auto-scroll, owned by
/// [`MenuSystem`](super::menu_system::MenuSystem) (Raven's file-scope
/// `static scrollInfo_t scrollInfo`).
///
/// Type definition source: `oracle/codemp/ui/ui_shared.c:24-33`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[doc(alias = "scrollInfo_s")]
#[doc(alias = "scrollInfo_t")]
#[allow(non_snake_case)]
pub struct ScrollInfo {
    pub nextScrollTime: c_int,
    pub nextAdjustTime: c_int,
    pub adjustValue: c_int,
    pub scrollKey: c_int,
    pub xStart: f32,
    pub yStart: f32,
    pub item: Option<ItemId>,
    pub scrollDir: bool,
}
