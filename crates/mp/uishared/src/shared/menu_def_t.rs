//! `MenuDef` — Raven `menuDef_t`.

use core::ffi::c_int;

use mp_qshared::shared::vec4_t;

use super::item_id::ItemId;
use super::window_def_t::WindowDef;

/// Raven `#define MAX_MENUITEMS 256`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:17`
pub const MAX_MENUITEMS: usize = 256;

/// Raven `menuDef_t` — a UI menu definition (window plus the items it owns).
///
/// PORT-NOTE: Raven's `itemDef_t *items[MAX_MENUITEMS]` + `itemCount` become
/// `Vec<ItemId>` into [`MenuSystem::items`](super::menu_system::MenuSystem)
/// (porting-rules §B5); `itemCount` is `items.len()`.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:307-336`
#[derive(Debug, Clone, PartialEq, Default)]
#[doc(alias = "menuDef_t")]
#[allow(non_snake_case)]
pub struct MenuDef {
    pub window: WindowDef,
    /// font
    pub font: String,
    /// covers entire screen
    pub fullScreen: bool,
    pub fontIndex: c_int,
    /// which item as the cursor
    pub cursorItem: c_int,
    pub fadeCycle: c_int,
    pub fadeClamp: f32,
    pub fadeAmount: f32,
    /// run when the menu is first opened
    pub onOpen: String,
    /// run when the menu is closed
    pub onClose: String,
    // JLFACCEPT
    /// run when menu is closed with acceptance
    pub onAccept: String,
    /// run when the menu is closed
    pub onESC: String,
    /// background loop sound for menu
    pub soundName: String,
    /// focus color for items
    pub focusColor: vec4_t,
    /// focus color for items
    pub disableColor: vec4_t,
    /// items this menu contains
    pub items: Vec<ItemId>,
    /// X position of description
    pub descX: c_int,
    /// X position of description
    pub descY: c_int,
    /// description text color for items
    pub descColor: vec4_t,
    /// Description of alignment
    pub descAlignment: c_int,
    /// Description scale
    pub descScale: f32,
    /// when next item should appear
    pub appearanceTime: f32,
    /// current item displayed
    pub appearanceCnt: c_int,
    pub appearanceIncrement: f32,
}
