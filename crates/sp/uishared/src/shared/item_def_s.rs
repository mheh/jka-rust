#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_float, c_int, c_void};

use sp_qshared::common::sp::ff::ff_handle_t::ffHandle_t;
use sp_qshared::common::sp::ghoul2::cghoul2_info_v::CGhoul2Info_v;
use sp_qshared::shared::{qhandle_t, sfxHandle_t};

use super::color_range_def_t::colorRangeDef_t;
use super::rect_def_t::rectDef_t;
use super::window_def_t::windowDef_t;

// Raven `#define MAX_COLOR_RANGES 10`.
// Source: `oracle/oracle/code/ui/ui_shared.h:270`
const MAX_COLOR_RANGES: usize = 10;

/// Raven `itemDef_s` — a single UI item (text, button, listbox, combo, etc.) within a menu.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:374-425`
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
    /// ( optional ) text alignment y coord
    pub textaligny: c_float,
    /// ( optional ) text2 alignment x coord
    pub text2alignx: c_float,
    /// ( optional ) text2 alignment y coord
    pub text2aligny: c_float,
    /// scale percentage from 72pts
    pub textscale: c_float,
    /// ( optional ) style, normal and shadowed are it for now
    pub textStyle: c_int,
    /// display text
    pub text: *const c_char,
    /// display text2
    pub text2: *const c_char,
    /// Description text
    pub descText: *const c_char,
    /// menu owner
    pub parent: *mut c_void,
    /// handle to asset
    pub asset: qhandle_t,
    /// ghoul2 instance if available instead of a model.
    pub ghoul2: CGhoul2Info_v,
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
    // JLFDPADSCRIPT MPMOVED
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
    // Raven: `#ifdef _IMMERSION` — force-feedback handle; only present under
    // Raven's `_IMMERSION` build, which this SP layout has enabled (per the
    // packet's verbatim offsets).
    pub focusForce: ffHandle_t,
    /// number of color ranges
    pub numColors: c_int,
    pub colorRanges: [colorRangeDef_t; MAX_COLOR_RANGES],
    /// used for feeder id's etc.. diff per type
    pub special: c_float,
    /// cursor position in characters
    pub cursorPos: c_int,
    /// type specific data ptr's
    pub typeData: *mut c_void,
    /// order of appearance
    pub appearanceSlot: c_int,
    /// used by ITEM_TYPE_MULTI that aren't linked to a particular cvar.
    pub value: c_int,
    /// FONT_SMALL,FONT_MEDIUM,FONT_LARGE
    pub font: c_int,
    pub invertYesNo: c_int,
    pub xoffset: c_int,
}

/// Raven `itemDef_t` — `typedef struct itemDef_s itemDef_t`.
///
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:374-425`
pub type itemDef_t = itemDef_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<itemDef_t>() == 712);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, window) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textRect) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, r#type) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, alignment) == 228);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textalignment) == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textalignx) == 236);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textaligny) == 240);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2alignx) == 244);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2aligny) == 248);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textscale) == 252);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, textStyle) == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text) == 264);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, text2) == 272);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, descText) == 280);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, parent) == 288);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, asset) == 296);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, ghoul2) == 300);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, flags) == 304);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseEnterText) == 312);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseExitText) == 320);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseEnter) == 328);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, mouseExit) == 336);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, action) == 344);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, accept) == 352);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, selectionNext) == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, selectionPrev) == 368);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, onFocus) == 376);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, leaveFocus) == 384);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvar) == 392);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvarTest) == 400);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, enableCvar) == 408);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cvarFlags) == 416);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, focusSound) == 420);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, focusForce) == 424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, numColors) == 428);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, colorRanges) == 432);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, special) == 672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, cursorPos) == 676);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, typeData) == 680);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, appearanceSlot) == 688);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, value) == 692);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, font) == 696);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, invertYesNo) == 700);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(itemDef_t, xoffset) == 704);
