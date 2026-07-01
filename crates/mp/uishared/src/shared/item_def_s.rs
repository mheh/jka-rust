#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_void};

use mp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t};

use super::color_range_def_t::colorRangeDef_t;
use super::rect_def_t::rectDef_t;
use super::window_def_t::windowDef_t;

// Raven `#define MAX_COLOR_RANGES 10`.
// Source: `oracle/oracle/codemp/ui/ui_shared.h:18`
const MAX_COLOR_RANGES: usize = 10;

/// Raven `itemDef_s` — a single UI item (text, button, listbox, combo, etc.) within a menu.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:258-305`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct itemDef_s {
    /// common positional, border, style, layout info
    pub window: windowDef_t,
    /// rectangle the text ( if any ) consumes
    pub textRect: rectDef_t,
    /// text, button, radiobutton, checkbox, textfield, listbox, combo
    pub r#type: c_int,
    /// left center right
    pub alignment: c_int,
    /// ( optional ) alignment for text within rect based on text width
    pub textalignment: c_int,
    /// ( optional ) text alignment x coord
    pub textalignx: c_float,
    /// ( optional ) text alignment x coord
    pub textaligny: c_float,
    /// scale percentage from 72pts
    pub textscale: c_float,
    /// ( optional ) style, normal and shadowed are it for now
    pub textStyle: c_int,
    /// display text
    pub text: *const c_char,
    /// display text, 2nd line
    pub text2: *const c_char,
    /// ( optional ) text2 alignment x coord
    pub text2alignx: c_float,
    /// ( optional ) text2 alignment y coord
    pub text2aligny: c_float,
    /// menu owner
    pub parent: *mut c_void,
    /// handle to asset
    pub asset: qhandle_t,
    /// ghoul2 instance if available instead of a model.
    pub ghoul2: *mut c_void,
    /// flags like g2valid, character, saber, saber2, etc.
    pub flags: c_int,
    /// mouse enter script
    pub mouseEnterText: *const c_char,
    /// mouse exit script
    pub mouseExitText: *const c_char,
    /// mouse enter script
    pub mouseEnter: *const c_char,
    /// mouse exit script
    pub mouseExit: *const c_char,
    /// select script
    pub action: *const c_char,
    // JLFACCEPT MPMOVED
    pub accept: *const c_char,
    // JLFDPADSCRIPT
    pub selectionNext: *const c_char,
    pub selectionPrev: *const c_char,
    /// select script
    pub onFocus: *const c_char,
    /// select script
    pub leaveFocus: *const c_char,
    /// associated cvar
    pub cvar: *const c_char,
    /// associated cvar for enable actions
    pub cvarTest: *const c_char,
    /// enable, disable, show, or hide based on value, this can contain a list
    pub enableCvar: *const c_char,
    /// what type of action to take on cvarenables
    pub cvarFlags: c_int,
    pub focusSound: sfxHandle_t,
    /// number of color ranges
    pub numColors: c_int,
    pub colorRanges: [colorRangeDef_t; MAX_COLOR_RANGES],
    /// used for feeder id's etc.. diff per type
    pub special: c_float,
    /// cursor position in characters
    pub cursorPos: c_int,
    /// type specific data ptr's
    pub typeData: *mut c_void,
    /// Description text
    pub descText: *const c_char,
    /// order of appearance
    pub appearanceSlot: c_int,
    /// FONT_SMALL,FONT_MEDIUM,FONT_LARGE // changed from 'font' so I could see what didn't
    /// compile, and differentiate between font handles returned from RegisterFont -ste
    pub iMenuFont: c_int,
    /// Does this item ignore mouse and keyboard focus
    pub disabled: qboolean,
    pub invertYesNo: c_int,
    pub xoffset: c_int,
}

/// Raven `itemDef_t` — `typedef struct itemDef_s itemDef_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_shared.h:258-305`
pub type itemDef_t = itemDef_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<itemDef_t>() == 704);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, window) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textRect) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, r#type) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, alignment) == 212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textalignment) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textalignx) == 220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textaligny) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textscale) == 228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textStyle) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2alignx) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2aligny) == 260);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, parent) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, asset) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, ghoul2) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, flags) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseEnterText) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseExitText) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseEnter) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseExit) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, action) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, accept) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, selectionNext) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, selectionPrev) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, onFocus) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, leaveFocus) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvar) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvarTest) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, enableCvar) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvarFlags) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, focusSound) == 404);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, numColors) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, colorRanges) == 412);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, special) == 652);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cursorPos) == 656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, typeData) == 664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, descText) == 672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, appearanceSlot) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, iMenuFont) == 684);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, disabled) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, invertYesNo) == 692);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, xoffset) == 696);
