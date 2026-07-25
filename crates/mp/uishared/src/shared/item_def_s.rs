//! `ItemDef` — Raven `itemDef_s`/`itemDef_t`.

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_qshared::shared::{qhandle_t, sfxHandle_t};

use super::color_range_def_t::ColorRangeDef;
use super::item_payload::ItemPayload;
use super::menu_id::MenuId;
use super::rect_def_t::RectDef;
use super::window_def_t::WindowDef;

/// Raven `#define MAX_COLOR_RANGES 10`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:18`
pub const MAX_COLOR_RANGES: usize = 10;

/// Raven `itemDef_s` (typedef `itemDef_t`) — a single UI item (text, button,
/// radiobutton, checkbox, textfield, listbox, combo, model, …) within a menu.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:258-305`
#[derive(Debug, Clone, PartialEq)]
#[doc(alias = "itemDef_s")]
#[doc(alias = "itemDef_t")]
#[allow(non_snake_case)]
pub struct ItemDef {
    /// common positional, border, style, layout info
    pub window: WindowDef,
    /// rectangle the text ( if any ) consumes
    pub textRect: RectDef,
    /// text, button, radiobutton, checkbox, textfield, listbox, combo
    pub r#type: c_int,
    /// left center right
    pub alignment: c_int,
    /// ( optional ) alignment for text within rect based on text width
    pub textalignment: c_int,
    /// ( optional ) text alignment x coord
    pub textalignx: f32,
    /// ( optional ) text alignment x coord
    pub textaligny: f32,
    /// scale percentage from 72pts
    pub textscale: f32,
    /// ( optional ) style, normal and shadowed are it for now
    pub textStyle: c_int,
    /// display text
    pub text: String,
    /// display text, 2nd line
    pub text2: String,
    /// ( optional ) text2 alignment x coord
    pub text2alignx: f32,
    /// ( optional ) text2 alignment y coord
    pub text2aligny: f32,
    /// menu owner (Raven's `void *parent`, always a `menuDef_t *`)
    pub parent: Option<MenuId>,
    /// handle to asset
    pub asset: qhandle_t,
    /// ghoul2 instance if available instead of a model. The engine `new`s and
    /// `delete`s the instance on its own heap and hands back only the handle
    /// value, so the module keeps it as an opaque token and never reads
    /// through it (scoping census, ghoul2 fields).
    pub ghoul2: *mut c_void,
    /// flags like g2valid, character, saber, saber2, etc.
    pub flags: c_int,
    /// mouse enter script
    pub mouseEnterText: String,
    /// mouse exit script
    pub mouseExitText: String,
    /// mouse enter script
    pub mouseEnter: String,
    /// mouse exit script
    pub mouseExit: String,
    /// select script
    pub action: String,
    // JLFACCEPT MPMOVED
    pub accept: String,
    // JLFDPADSCRIPT
    pub selectionNext: String,
    pub selectionPrev: String,
    /// select script
    pub onFocus: String,
    /// select script
    pub leaveFocus: String,
    /// associated cvar
    pub cvar: String,
    /// associated cvar for enable actions
    pub cvarTest: String,
    /// enable, disable, show, or hide based on value, this can contain a list
    pub enableCvar: String,
    /// what type of action to take on cvarenables
    pub cvarFlags: c_int,
    pub focusSound: sfxHandle_t,
    /// number of color ranges
    pub numColors: c_int,
    pub colorRanges: [ColorRangeDef; MAX_COLOR_RANGES],
    /// used for feeder id's etc.. diff per type
    pub special: f32,
    /// cursor position in characters
    pub cursorPos: c_int,
    /// type specific data (Raven's pool-allocated `void *typeData`)
    pub typeData: ItemPayload,
    /// Description text
    pub descText: String,
    /// order of appearance
    pub appearanceSlot: c_int,
    /// FONT_SMALL,FONT_MEDIUM,FONT_LARGE // changed from 'font' so I could see
    /// what didn't compile, and differentiate between font handles returned
    /// from RegisterFont -ste
    pub iMenuFont: c_int,
    /// Does this item ignore mouse and keyboard focus
    pub disabled: bool,
    pub invertYesNo: c_int,
    pub xoffset: c_int,
}

impl Default for ItemDef {
    /// Raven's `Item_Init` zeroes the item, then re-applies the window
    /// defaults; this is the zeroed half (`memset(item, 0, sizeof(itemDef_t))`)
    /// with owned fields at their empty values.
    ///
    /// Source: `oracle/codemp/ui/ui_shared.c` (`Item_Init`)
    fn default() -> Self {
        ItemDef {
            window: WindowDef::default(),
            textRect: RectDef::default(),
            r#type: 0,
            alignment: 0,
            textalignment: 0,
            textalignx: 0.0,
            textaligny: 0.0,
            textscale: 0.0,
            textStyle: 0,
            text: String::new(),
            text2: String::new(),
            text2alignx: 0.0,
            text2aligny: 0.0,
            parent: None,
            asset: 0,
            ghoul2: null_mut(),
            flags: 0,
            mouseEnterText: String::new(),
            mouseExitText: String::new(),
            mouseEnter: String::new(),
            mouseExit: String::new(),
            action: String::new(),
            accept: String::new(),
            selectionNext: String::new(),
            selectionPrev: String::new(),
            onFocus: String::new(),
            leaveFocus: String::new(),
            cvar: String::new(),
            cvarTest: String::new(),
            enableCvar: String::new(),
            cvarFlags: 0,
            focusSound: 0,
            numColors: 0,
            colorRanges: [ColorRangeDef::default(); MAX_COLOR_RANGES],
            special: 0.0,
            cursorPos: 0,
            typeData: ItemPayload::None,
            descText: String::new(),
            appearanceSlot: 0,
            iMenuFont: 0,
            disabled: false,
            invertYesNo: 0,
            xoffset: 0,
        }
    }
}
