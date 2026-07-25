//! `ui_shared.c` — the menu framework's logic, operating on the DEC-36 root
//! types: [`crate::shared::menu_system::MenuSystem`] (arena + handles),
//! [`crate::shared::display_state::DisplayState`] (the `DC->` data tail) and
//! the [`crate::shared::display_context::DisplayContext`] host trait.
//!
//! Source: `oracle/codemp/ui/ui_shared.c`

#![allow(non_snake_case)]

use core::f64::consts::PI as PI_F64;
use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_bg::public::anim_table::animTable;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::shared::q_math::{AnglesToAxis, VectorSet};
use mp_qshared::shared::q_string::{COM_Parse, GetIDForString};
use mp_qshared::shared::{
    cbufExec_t, pc_token_t, qtrue, stringID_table_t, vec3_t, vec4_t, CHAN_AUTO, CHAN_LOCAL_SOUND,
    MAX_QPATH, MAX_STRING_CHARS, MAX_TOKENLENGTH, SCREEN_HEIGHT, SCREEN_WIDTH, TT_NUMBER,
};
use native_string::{atof, atoi, latin1_to_string, string_to_latin1, Q_stricmp};

use crate::shared::capture_func::CaptureFunc;
use crate::shared::color_range_def_t::ColorRangeDef;
use crate::shared::display_context::DisplayContext;
use crate::shared::display_state::DisplayState;
use crate::shared::edit_field_def_s::{EditFieldDef, MAX_EDITFIELD};
use crate::shared::item_def_s::{ItemDef, MAX_COLOR_RANGES};
use crate::shared::item_id::ItemId;
use crate::shared::item_payload::ItemPayload;
use crate::shared::list_box_def_s::{ListBoxDef, MAX_LB_COLUMNS};
use crate::shared::menu_def_t::MenuDef;
use crate::shared::menu_id::MenuId;
use crate::shared::menu_system::{
    MenuSystem, DOUBLE_CLICK_DELAY, MAX_DEFERRED_SCRIPT, MAX_OPEN_MENUS,
};
use crate::shared::menudef::{
    FEEDER_LANGUAGES, FEEDER_PLAYER_SPECIES, ITEM_ALIGN_CENTER, ITEM_ALIGN_LEFT, ITEM_ALIGN_RIGHT,
    ITEM_TEXTSTYLE_BLINK, ITEM_TYPE_BIND, ITEM_TYPE_BUTTON, ITEM_TYPE_CHECKBOX, ITEM_TYPE_COMBO,
    ITEM_TYPE_EDITFIELD, ITEM_TYPE_LISTBOX, ITEM_TYPE_MODEL, ITEM_TYPE_MULTI,
    ITEM_TYPE_NUMERICFIELD, ITEM_TYPE_OWNERDRAW, ITEM_TYPE_RADIOBUTTON, ITEM_TYPE_SLIDER,
    ITEM_TYPE_TEXT, ITEM_TYPE_TEXTSCROLL, ITEM_TYPE_YESNO, LISTBOX_IMAGE, UI_FORCE_RANK_ABSORB,
    UI_FORCE_RANK_DRAIN, UI_FORCE_RANK_GRIP, UI_FORCE_RANK_HEAL, UI_FORCE_RANK_LEVITATION,
    UI_FORCE_RANK_LIGHTNING, UI_FORCE_RANK_PROTECT, UI_FORCE_RANK_PULL, UI_FORCE_RANK_PUSH,
    UI_FORCE_RANK_RAGE, UI_FORCE_RANK_SABERATTACK, UI_FORCE_RANK_SABERDEFEND,
    UI_FORCE_RANK_SABERTHROW, UI_FORCE_RANK_SEE, UI_FORCE_RANK_SPEED, UI_FORCE_RANK_TEAM_FORCE,
    UI_FORCE_RANK_TEAM_HEAL, UI_FORCE_RANK_TELEPATHY, UI_FORCE_SIDE, WINDOW_BORDER_FULL,
    WINDOW_BORDER_HORZ, WINDOW_BORDER_KCGRADIENT, WINDOW_BORDER_VERT, WINDOW_STYLE_CINEMATIC,
    WINDOW_STYLE_FILLED, WINDOW_STYLE_GRADIENT, WINDOW_STYLE_SHADER, WINDOW_STYLE_TEAMCOLOR,
};
use crate::shared::model_def_s::ModelDef;
use crate::shared::multi_def_s::{MultiDef, MAX_MULTI_CVARS};
use crate::shared::rect_def_t::RectDef;
use crate::shared::scroll_info_s::{
    SCROLL_TIME_ADJUST, SCROLL_TIME_ADJUSTOFFSET, SCROLL_TIME_FLOOR, SCROLL_TIME_START,
};
use crate::shared::text_scroll_def_s::{TextScrollDef, MAX_TEXTSCROLL_LINES};
use crate::shared::window_def_t::WindowDef;

/// Raven `#define WINDOW_MOUSEOVER 0x00000001`.
/// Source: `oracle/codemp/ui/ui_shared.h:22`
pub const WINDOW_MOUSEOVER: c_int = 0x0000_0001;
/// Raven `#define WINDOW_HASFOCUS 0x00000002`.
/// Source: `oracle/codemp/ui/ui_shared.h:23`
pub const WINDOW_HASFOCUS: c_int = 0x0000_0002;
/// Raven `#define WINDOW_VISIBLE 0x00000004`.
/// Source: `oracle/codemp/ui/ui_shared.h:24`
pub const WINDOW_VISIBLE: c_int = 0x0000_0004;
/// Raven `#define WINDOW_FADINGOUT 0x00000020`.
/// Source: `oracle/codemp/ui/ui_shared.h:27`
pub const WINDOW_FADINGOUT: c_int = 0x0000_0020;
/// Raven `#define WINDOW_FADINGIN 0x00000040`.
/// Source: `oracle/codemp/ui/ui_shared.h:28`
pub const WINDOW_FADINGIN: c_int = 0x0000_0040;
/// Raven `#define WINDOW_MOUSEOVERTEXT 0x00000080`.
/// Source: `oracle/codemp/ui/ui_shared.h:29`
pub const WINDOW_MOUSEOVERTEXT: c_int = 0x0000_0080;
/// Raven `#define WINDOW_DECORATION 0x00000010`.
/// Source: `oracle/codemp/ui/ui_shared.h:26`
pub const WINDOW_DECORATION: c_int = 0x0000_0010;
/// Raven `#define WINDOW_HORIZONTAL 0x00000400`.
/// Source: `oracle/codemp/ui/ui_shared.h:32`
pub const WINDOW_HORIZONTAL: c_int = 0x0000_0400;
/// Raven `#define WINDOW_INTRANSITION 0x00000100` — window is in transition.
/// Source: `oracle/codemp/ui/ui_shared.h:30`
pub const WINDOW_INTRANSITION: c_int = 0x0000_0100;
/// Raven `#define WINDOW_ORBITING 0x00010000` — item is in orbit.
/// Source: `oracle/codemp/ui/ui_shared.h:38`
pub const WINDOW_ORBITING: c_int = 0x0001_0000;
/// Raven `#define WINDOW_OOB_CLICK 0x00020000`.
/// Source: `oracle/codemp/ui/ui_shared.h:39`
pub const WINDOW_OOB_CLICK: c_int = 0x0002_0000;
/// Raven `#define WINDOW_WRAPPED 0x00040000`.
/// Source: `oracle/codemp/ui/ui_shared.h:40`
pub const WINDOW_WRAPPED: c_int = 0x0004_0000;
/// Raven `#define WINDOW_AUTOWRAPPED 0x00080000`.
/// Source: `oracle/codemp/ui/ui_shared.h:41`
pub const WINDOW_AUTOWRAPPED: c_int = 0x0008_0000;
/// Raven `#define WINDOW_FORCED 0x00100000`.
/// Source: `oracle/codemp/ui/ui_shared.h:42`
pub const WINDOW_FORCED: c_int = 0x0010_0000;
/// Raven `#define WINDOW_POPUP 0x00200000`.
/// Source: `oracle/codemp/ui/ui_shared.h:43`
pub const WINDOW_POPUP: c_int = 0x0020_0000;

/// Raven `#define WINDOW_INACTIVE 0x00000008`.
/// Source: `oracle/codemp/ui/ui_shared.h:25`
pub const WINDOW_INACTIVE: c_int = 0x0000_0008;
/// Raven `#define WINDOW_FORECOLORSET 0x00000200`.
/// Source: `oracle/codemp/ui/ui_shared.h:31`
pub const WINDOW_FORECOLORSET: c_int = 0x0000_0200;
/// Raven `#define WINDOW_BACKCOLORSET 0x00400000` — backcolor was explicitly
/// set.
/// Source: `oracle/codemp/ui/ui_shared.h:44`
pub const WINDOW_BACKCOLORSET: c_int = 0x0040_0000;
/// Raven `#define WINDOW_LB_LEFTARROW 0x00000800`.
/// Source: `oracle/codemp/ui/ui_shared.h:33`
pub const WINDOW_LB_LEFTARROW: c_int = 0x0000_0800;
/// Raven `#define WINDOW_LB_RIGHTARROW 0x00001000`.
/// Source: `oracle/codemp/ui/ui_shared.h:34`
pub const WINDOW_LB_RIGHTARROW: c_int = 0x0000_1000;
/// Raven `#define WINDOW_LB_THUMB 0x00002000`.
/// Source: `oracle/codemp/ui/ui_shared.h:35`
pub const WINDOW_LB_THUMB: c_int = 0x0000_2000;
/// Raven `#define WINDOW_LB_PGUP 0x00004000`.
/// Source: `oracle/codemp/ui/ui_shared.h:36`
pub const WINDOW_LB_PGUP: c_int = 0x0000_4000;
/// Raven `#define WINDOW_LB_PGDN 0x00008000`.
/// Source: `oracle/codemp/ui/ui_shared.h:37`
pub const WINDOW_LB_PGDN: c_int = 0x0000_8000;
/// Raven `#define WINDOW_TIMEDVISIBLE 0x00800000` — visibility timing ( NOT
/// implemented ).
/// Source: `oracle/codemp/ui/ui_shared.h:45`
pub const WINDOW_TIMEDVISIBLE: c_int = 0x0080_0000;
/// Raven `#define WINDOW_PLAYERCOLOR 0x01000000`.
/// Source: `oracle/codemp/ui/ui_shared.h:46`
pub const WINDOW_PLAYERCOLOR: c_int = 0x0100_0000;
/// Raven `#define WINDOW_INTRANSITIONMODEL 0x04000000` — delayed script
/// waiting to run.
/// Source: `oracle/codemp/ui/ui_shared.h:49`
const WINDOW_INTRANSITIONMODEL: c_int = 0x0400_0000;

/// Raven `#define ITF_G2VALID 0x0001` — indicates whether or not the item's
/// ghoul2 instance is valid.
/// Source: `oracle/codemp/ui/ui_shared.h:251`
const ITF_G2VALID: c_int = 0x0001;
/// Raven `#define ITF_ISCHARACTER 0x0002` — a character item, uses customRGBA.
/// Source: `oracle/codemp/ui/ui_shared.h:252`
pub const ITF_ISCHARACTER: c_int = 0x0002;
/// Raven `#define ITF_ISSABER 0x0004` — first saber item, draws blade.
/// Source: `oracle/codemp/ui/ui_shared.h:253`
pub const ITF_ISSABER: c_int = 0x0004;
/// Raven `#define ITF_ISSABER2 0x0008` — second saber item, draws blade.
/// Source: `oracle/codemp/ui/ui_shared.h:254`
pub const ITF_ISSABER2: c_int = 0x0008;
/// Raven `#define ITF_ISANYSABER (ITF_ISSABER|ITF_ISSABER2)` — either saber.
/// Source: `oracle/codemp/ui/ui_shared.h:256`
pub const ITF_ISANYSABER: c_int = ITF_ISSABER | ITF_ISSABER2;

/// Raven `#define RDF_NOWORLDMODEL 1` — used for player configuration screen.
///
/// No prior home in the port; ui_shared.c is its only caller so far.
/// Source: `oracle/codemp/cgame/tr_types.h:57`
const RDF_NOWORLDMODEL: c_int = 1;
/// Raven `#define RF_LIGHTING_ORIGIN 0x00080` — use `lightingOrigin` instead
/// of `origin`.
/// Source: `oracle/codemp/cgame/tr_types.h:28`
const RF_LIGHTING_ORIGIN: c_int = 0x0080;
/// Raven `#define RF_NOSHADOW 0x00040` — don't add stencil shadows.
/// Source: `oracle/codemp/cgame/tr_types.h:26`
const RF_NOSHADOW: c_int = 0x0040;

/// Raven `#define CVAR_ENABLE 0x00000001`.
/// Source: `oracle/codemp/ui/ui_shared.h:246`
const CVAR_ENABLE: c_int = 0x0000_0001;
/// Raven `#define CVAR_DISABLE 0x00000002`.
/// Source: `oracle/codemp/ui/ui_shared.h:247`
const CVAR_DISABLE: c_int = 0x0000_0002;
/// Raven `#define CVAR_SHOW 0x00000004`.
/// Source: `oracle/codemp/ui/ui_shared.h:248`
const CVAR_SHOW: c_int = 0x0000_0004;
/// Raven `#define CVAR_HIDE 0x00000008`.
/// Source: `oracle/codemp/ui/ui_shared.h:249`
const CVAR_HIDE: c_int = 0x0000_0008;

/// Raven `#define CURSOR_ARROW 0x00000002`.
/// Source: `oracle/codemp/ui/ui_shared.h:54`
pub const CURSOR_ARROW: c_int = 0x0000_0002;
/// Raven `#define CURSOR_SIZER 0x00000004`.
/// Source: `oracle/codemp/ui/ui_shared.h:55`
pub const CURSOR_SIZER: c_int = 0x0000_0004;

/// Raven `#define SCROLLBAR_SIZE 16.0`.
/// Source: `oracle/codemp/ui/ui_shared.h:99`
const SCROLLBAR_SIZE: f32 = 16.0;
/// Raven `#define SLIDER_THUMB_WIDTH 12.0`.
/// Source: `oracle/codemp/ui/ui_shared.h:102`
const SLIDER_THUMB_WIDTH: f32 = 12.0;
/// Raven `#define SLIDER_THUMB_HEIGHT 20.0`.
/// Source: `oracle/codemp/ui/ui_shared.h:103`
const SLIDER_THUMB_HEIGHT: f32 = 20.0;

/// Raven `#define K_CHAR_FLAG 1024` — or'd onto a keynum to mark a
/// translated-character event.
/// Source: `oracle/codemp/ui/keycodes.h:347`
const K_CHAR_FLAG: c_int = 1024;

// PORT-NOTE: `mp_uishared` is host-agnostic and carries no dependency on
// `mp_ui`'s `fakeAscii_t` keycode enum (see the `MAX_KEYS` note above), so the
// handful of `A_*` key codes this file's `*_HandleKey` fns compare against get
// local numeric twins — the same ordinal values as
// `crates/mp/ui/src/keycodes/fake_ascii_t.rs`'s `fakeAscii_t`.
/// Source: `oracle/codemp/ui/keycodes.h:8-341`
const A_BACKSPACE: c_int = 8;
const A_TAB: c_int = 9;
const A_ENTER: c_int = 10;
const A_KP_ENTER: c_int = 13;
const A_KP_PERIOD: c_int = 14;
const A_KP_0: c_int = 16;
const A_KP_1: c_int = 17;
const A_KP_2: c_int = 18;
const A_KP_3: c_int = 19;
const A_KP_4: c_int = 20;
const A_KP_6: c_int = 22;
const A_KP_7: c_int = 23;
const A_KP_8: c_int = 24;
const A_KP_9: c_int = 25;
const A_ESCAPE: c_int = 27;
const A_DELETE: c_int = 127;
const A_MOUSE1: c_int = 141;
const A_MOUSE2: c_int = 142;
const A_INSERT: c_int = 143;
const A_HOME: c_int = 144;
const A_PAGE_UP: c_int = 145;
const A_F11: c_int = 151;
const A_F12: c_int = 152;
const A_END: c_int = 157;
const A_PAGE_DOWN: c_int = 158;
const A_MOUSE3: c_int = 166;
const A_CURSOR_UP: c_int = 170;
const A_CURSOR_DOWN: c_int = 171;
const A_CURSOR_LEFT: c_int = 172;
const A_CURSOR_RIGHT: c_int = 173;

// PORT-NOTE: `q_shared.h`'s font enum is anonymous (`enum { FONT_NONE,
// FONT_SMALL=1, ... }`), so per the anonymous-enum convention these are
// `const`s; local until the family gets a canonical `mp_qshared` home.
/// Source: `oracle/codemp/game/q_shared.h:3176-3182`
const FONT_MEDIUM: c_int = 2;
const FONT_SMALL2: c_int = 4;

/// Raven `itemFlags[]`, the `ItemParse_flag` lookup table (`"WINDOW_INACTIVE"`
/// is the only entry the retail table carries before its NULL sentinel).
/// Source: `oracle/codemp/ui/ui_shared.c:152-160`
const ITEM_FLAGS: &[(&str, c_int)] = &[("WINDOW_INACTIVE", WINDOW_INACTIVE)];

/// Raven `ui_shared.c`'s file-local `#define HASH_TABLE_SIZE 2048`.
/// Source: `oracle/codemp/ui/ui_shared.c:257`
const HASH_TABLE_SIZE: i64 = 2048;

/// Raven `ui_shared.c`'s file-local `#define KEYWORDHASH_SIZE 512`.
/// Source: `oracle/codemp/ui/ui_shared.c:7324`
const KEYWORDHASH_SIZE: i32 = 512;

/// Raven `#define SLIDER_WIDTH 96.0`.
/// Source: `oracle/codemp/ui/ui_shared.h:100`
const SLIDER_WIDTH: f32 = 96.0;
/// Raven `#define SLIDER_HEIGHT 16.0`.
/// Source: `oracle/codemp/ui/ui_shared.h:101`
const SLIDER_HEIGHT: f32 = 16.0;

/// Raven `#define BLINK_DIVISOR 200`.
/// Source: `oracle/codemp/game/q_shared.h:485`
const BLINK_DIVISOR: c_int = 200;
/// Raven `#define PULSE_DIVISOR 75`.
/// Source: `oracle/codemp/game/q_shared.h:486`
const PULSE_DIVISOR: c_int = 75;

// PORT-NOTE: Raven's `MAX_KEYS` is the sentinel last member of the
// `fakeAscii_t` enum (`oracle/codemp/ui/keycodes.h:8-341`, 320 named codes
// before the sentinel — see the already-ported twin at
// `crates/mp/ui/src/keycodes/fake_ascii_t.rs`). `mp_uishared` is host-agnostic
// and carries no dependency on `mp_ui`'s keycode enum, so `Controls_
// GetKeyAssignment`'s loop bound gets this local numeric twin instead.
const MAX_KEYS: c_int = 320;

/// `Q_stricmp(a, b) == 0` — the equality-only shape every call site in this
/// file actually needs, delegating to the canonical `native_string` compare
/// (DEC-32 one-canonical-home).
#[inline]
fn stricmp_eq(a: &str, b: &str) -> bool {
    Q_stricmp(a, b) == 0
}

/// A zeroed `pc_token_t` — `mp_qshared`'s ABI struct carries no `Default` impl
/// (it crosses the seam by value, Class B), so the `ItemParse_*`/`MenuParse_*`
/// item fns below that call [`DisplayContext::PC_ReadToken`] build one by hand.
#[inline]
fn zero_pc_token() -> pc_token_t {
    pc_token_t {
        type_: 0,
        subtype: 0,
        intvalue: 0,
        floatvalue: 0.0,
        string: [0; MAX_TOKENLENGTH],
    }
}

/// Reads a NUL-terminated `pc_token_t::string` (Latin-1 wire bytes) into an
/// owned `String` via the canonical Latin-1 decoder — the byte-transparent
/// seam discipline (#13 string campaign).
#[inline]
fn pc_token_str(token: &pc_token_t) -> String {
    let bytes: Vec<u8> = token
        .string
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    latin1_to_string(&bytes)
}

/// Raven `UI_Alloc` — carve `size` bytes out of the 2 MB `memoryPool` bump
/// allocator.
///
/// PORT-NOTE (D2 pool retirement): `MenuSystem` builds `itemDef_t::typeData`
/// directly as an owned [`ItemPayload`](crate::shared::item_payload::ItemPayload)
/// value rather than carving it out of a shared pool, so there is no pool
/// state (`memoryPool`/`allocPoint`) left to allocate from. Closest
/// owned-shape equivalent: hand back `size` zeroed bytes directly — the shape
/// callers actually want (initialized scratch memory), without a pool cursor
/// or an out-of-memory ceiling (owned heap allocation does not fail here).
/// Source: `oracle/codemp/ui/ui_shared.c:209-234`
pub fn UI_Alloc(size: c_int) -> Vec<u8> {
    vec![0u8; size.max(0) as usize]
}

/// Raven `UI_InitMemory` — reset the bump-pool cursor and clear the
/// out-of-memory flag.
///
/// PORT-NOTE (D2 pool retirement): no-op — owned collections carry no pool
/// cursor to reset. Kept as a callable no-op for callers that still invoke
/// the housekeeping fn.
/// Source: `oracle/codemp/ui/ui_shared.c:241-247`
pub fn UI_InitMemory() {}

/// Raven `UI_OutOfMemory`.
///
/// PORT-NOTE (D2 pool retirement): owned collections (`Vec`/`String`) grow
/// through the allocator and never hit `MEM_POOL_SIZE`, so this ceiling
/// cannot be hit; always `false`.
/// Source: `oracle/codemp/ui/ui_shared.c:249-251`
pub fn UI_OutOfMemory() -> bool {
    false
}

/// Raven `hashForString` — case-folded weighted-sum hash into
/// `HASH_TABLE_SIZE` buckets.
///
/// PORT-NOTE: `tolower((unsigned char)c)` is libc's locale-aware fold;
/// `to_ascii_lowercase` folds only `A`-`Z` (the "C" locale's actual behavior,
/// which is what the retail build links). Raven assigns the result to a
/// SIGNED `char`, so bytes ≥ 0x80 (Latin-1 text) contribute a negative term —
/// the `as i8` reproduces that.
/// Source: `oracle/codemp/ui/ui_shared.c:263-277`
pub fn hashForString(str: &str) -> i64 {
    let mut hash: i64 = 0;
    for (i, b) in str.bytes().enumerate() {
        let letter = b.to_ascii_lowercase() as i8 as i64;
        hash += letter * (i as i64 + 119);
    }
    hash &= HASH_TABLE_SIZE - 1;
    hash
}

/// Raven `LerpColor` — lerp `a`→`b` by `t` into `c`, clamped to `[0,1]` per
/// component.
/// Source: `oracle/codemp/ui/ui_shared.c:429-442`
pub fn LerpColor(a: vec4_t, b: vec4_t, c: &mut vec4_t, t: f32) {
    for i in 0..4 {
        c[i] = a[i] + t * (b[i] - a[i]);
        if c[i] < 0.0 {
            c[i] = 0.0;
        } else if c[i] > 1.0 {
            c[i] = 1.0;
        }
    }
}

/// Raven `Float_Parse` — parse one token off `p` as a float; `true` on
/// success.
/// Source: `oracle/codemp/ui/ui_shared.c:449-458`
pub fn Float_Parse(p: &mut &str, f: &mut f32) -> bool {
    let (token, rest) = COM_Parse(p, false);
    *p = rest;
    if !token.is_empty() {
        *f = atof(&token) as f32;
        true
    } else {
        false
    }
}

/// Raven `Int_Parse` — parse one token off `p` as an int; `true` on success.
/// Source: `oracle/codemp/ui/ui_shared.c:528-538`
pub fn Int_Parse(p: &mut &str, i: &mut c_int) -> bool {
    let (token, rest) = COM_Parse(p, false);
    *p = rest;
    if !token.is_empty() {
        *i = atoi(&token);
        true
    } else {
        false
    }
}

// `Init_Display` — ui_shared.c:693-695.
//
// DEFERRED: Init_Display — DEC-36 D3 threads `DisplayContext` as a per-call
// `&mut dyn DisplayContext` argument, not a stored file-scope `DC` pointer
// (U3 ruling, 2026-07-24); every caller already holds its own `dc` reference
// directly, so there is no `DC` field left for this fn to assign and no
// owned-shape equivalent to write.
// Source: `oracle/codemp/ui/ui_shared.c:693-695`

/// Raven `GradientBar_Paint` — two-paint gradient bar (fill, then reset
/// color).
/// Source: `oracle/codemp/ui/ui_shared.c:701-706`
pub fn GradientBar_Paint(
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    rect: &RectDef,
    color: vec4_t,
) {
    dc.setColor(Some(color));
    dc.drawHandlePic(rect.x, rect.y, rect.w, rect.h, ds.Assets.gradientBar);
    dc.setColor(None);
}

/// Raven `Window_Init` — zero a window, then apply its non-zero defaults.
/// Source: `oracle/codemp/ui/ui_shared.c:717-722`
pub fn Window_Init(w: &mut WindowDef) {
    *w = WindowDef::default();
    w.borderSize = 1.0;
    w.foreColor = [1.0, 1.0, 1.0, 1.0];
    w.cinematic = -1;
}

/// Raven `Fade` — advance a fade-in/fade-out value on the `nextTime`
/// throttle, clearing the fade flag (and, for fade-out, `WINDOW_VISIBLE`)
/// once the fade completes.
/// Source: `oracle/codemp/ui/ui_shared.c:724-744`
pub fn Fade(
    ds: &DisplayState,
    flags: &mut c_int,
    f: &mut f32,
    clamp: f32,
    nextTime: &mut c_int,
    offsetTime: c_int,
    bFlags: bool,
    fadeAmount: f32,
) {
    if *flags & (WINDOW_FADINGOUT | WINDOW_FADINGIN) != 0 && ds.realTime > *nextTime {
        *nextTime = ds.realTime + offsetTime;
        if *flags & WINDOW_FADINGOUT != 0 {
            *f -= fadeAmount;
            if bFlags && *f <= 0.0 {
                *flags &= !(WINDOW_FADINGOUT | WINDOW_VISIBLE);
            }
        } else {
            *f += fadeAmount;
            if *f >= clamp {
                *f = clamp;
                if bFlags {
                    *flags &= !WINDOW_FADINGIN;
                }
            }
        }
    }
}

/// Raven `IsVisible`.
/// Source: `oracle/codemp/ui/ui_shared.c:1013-1015`
pub fn IsVisible(flags: c_int) -> bool {
    flags & WINDOW_VISIBLE != 0 && flags & WINDOW_FADINGOUT == 0
}

/// Raven `Rect_ContainsPoint` — strict interior containment (`>`/`<`, not
/// `>=`/`<=`).
/// Source: `oracle/codemp/ui/ui_shared.c:1017-1024`
pub fn Rect_ContainsPoint(rect: Option<&RectDef>, x: f32, y: f32) -> bool {
    if let Some(rect) = rect {
        if x > rect.x && x < rect.x + rect.w && y > rect.y && y < rect.y + rect.h {
            return true;
        }
    }
    false
}

/// Raven `Menu_GetMatchingItemByNumber` — the `index`-th item (0-based, among
/// matches) in `menu` whose name or group matches `name`.
/// Source: `oracle/codemp/ui/ui_shared.c:1049-1061`
pub fn Menu_GetMatchingItemByNumber(
    menus: &MenuSystem,
    menu: MenuId,
    index: c_int,
    name: &str,
) -> Option<ItemId> {
    let mut count = 0;
    for &id in &menus.menu(menu).items {
        let it = menus.item(id);
        if it
            .window
            .name
            .as_deref()
            .is_some_and(|n| stricmp_eq(n, name))
            || it
                .window
                .group
                .as_deref()
                .is_some_and(|g| stricmp_eq(g, name))
        {
            if count == index {
                return Some(id);
            }
            count += 1;
        }
    }
    None
}

/// Raven `Menu_FindItemByName` — the item in `menu` named `p`.
///
/// PORT-NOTE: Raven's `menu == NULL || p == NULL` guard becomes `menu:
/// Option<MenuId>`; `p` stays a non-nullable `&str` (an empty string is the
/// only representable "no name", which the loop already handles naturally).
/// Source: `oracle/codemp/ui/ui_shared.c:1220-1233`
pub fn Menu_FindItemByName(menus: &MenuSystem, menu: Option<MenuId>, p: &str) -> Option<ItemId> {
    let menu = menu?;
    for &id in &menus.menu(menu).items {
        if menus
            .item(id)
            .window
            .name
            .as_deref()
            .is_some_and(|n| stricmp_eq(p, n))
        {
            return Some(id);
        }
    }
    None
}

/// Raven `Script_SetTeamColor` — copy the host's current team color into the
/// item's `backColor`.
///
/// PORT-NOTE: Raven's `if (DC->getTeamColor)` null-checks the function
/// pointer; `DisplayContext` always implements every method, so the guard
/// collapses to an unconditional call. `args` is unused, matching Raven.
/// Source: `oracle/codemp/ui/ui_shared.c:1235-1248`
pub fn Script_SetTeamColor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    _args: &mut &str,
) -> bool {
    let color = dc.getTeamColor();
    menus.item_mut(item).window.backColor = color;
    true
}

/// Raven `Menus_FindByName` — the defined menu named `p`.
/// Source: `oracle/codemp/ui/ui_shared.c:1506-1514`
pub fn Menus_FindByName(menus: &MenuSystem, p: &str) -> Option<MenuId> {
    for (i, m) in menus.menus.iter().enumerate() {
        if m.window.name.as_deref().is_some_and(|n| stricmp_eq(n, p)) {
            return Some(MenuId::new(i));
        }
    }
    None
}

/// Raven `Script_Defer` — should the running script suspend on this item?
/// If the host's `deferScript` says yes, save the item and the unconsumed
/// script tail so the script can resume later; otherwise keep running.
///
/// PORT-NOTE: `Q_strncpyz(ui_deferredScript, *args, MAX_DEFERRED_SCRIPT)`
/// truncates at a byte boundary; the owned `String` truncation below walks
/// back to the nearest char boundary at or before that byte to stay valid
/// UTF-8 (the same observable truncation point for the ASCII/Latin-1 script
/// text this file actually carries).
/// Source: `oracle/codemp/ui/ui_shared.c:1767-1784`
pub fn Script_Defer(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    if dc.deferScript(args) {
        menus.ui_deferredScriptItem = Some(item);
        let mut s = (*args).to_string();
        if s.len() >= MAX_DEFERRED_SCRIPT {
            let mut cut = MAX_DEFERRED_SCRIPT - 1;
            while cut > 0 && !s.is_char_boundary(cut) {
                cut -= 1;
            }
            s.truncate(cut);
        }
        menus.ui_deferredScript = s;
        false
    } else {
        true
    }
}

/// Local twin of Raven `q_shared.c`'s `COM_ParseFloat` — parses one token off
/// `p` as a float, `true` on failure (Raven's polarity: EOF/empty token).
/// `mp_uishared` has no `QSharedScratch`/`BgTraps` seam to reach the ported
/// `COM_ParseFloat` twins in `mp_game`/`mp_bg`, so [`ParseRect`] gets this
/// minimal local equivalent instead.
/// Source: `oracle/codemp/game/q_shared.c:625-638`
fn parseFloatOrFail(p: &mut &str, f: &mut f32) -> bool {
    let (token, rest) = COM_Parse(p, false);
    *p = rest;
    if token.is_empty() {
        return true;
    }
    *f = atof(&token) as f32;
    false
}

/// Raven `ParseRect` — parse `x y w h` off `p` into `r`; `true` only if all
/// four floats parsed.
/// Source: `oracle/codemp/ui/ui_shared.c:2002-2018`
pub fn ParseRect(p: &mut &str, r: &mut RectDef) -> bool {
    if !parseFloatOrFail(p, &mut r.x) {
        if !parseFloatOrFail(p, &mut r.y) {
            if !parseFloatOrFail(p, &mut r.w) {
                if !parseFloatOrFail(p, &mut r.h) {
                    return true;
                }
            }
        }
    }
    false
}

/// Raven `Item_TextScroll_MaxScroll` — how many lines the scroll box can
/// scroll past.
///
/// PORT-NOTE: `iLineCount` is `pLines.len()` (see `TextScrollDef`'s
/// PORT-NOTE); a `typeData` payload mismatch (unreachable under this file's
/// own type dispatch) falls back to 0 rather than the null-deref UB Raven's
/// cast would hit.
/// Source: `oracle/codemp/ui/ui_shared.c:2482-2495`
pub fn Item_TextScroll_MaxScroll(menus: &MenuSystem, item: ItemId) -> c_int {
    let it = menus.item(item);
    let scrollPtr = match it.typeData.textScroll() {
        Some(s) => s,
        None => return 0,
    };
    let count = scrollPtr.pLines.len() as c_int;
    let max = count - (it.window.rect.h / scrollPtr.lineHeight) as c_int + 1;
    if max < 0 {
        0
    } else {
        max
    }
}

/// Raven `Item_ListBox_MaxScroll` — how many rows/columns the list box can
/// scroll past, feeder-populated count minus the visible span.
///
/// PORT-NOTE: see `Item_TextScroll_MaxScroll` — a payload-type mismatch falls
/// back to 0.
/// Source: `oracle/codemp/ui/ui_shared.c:2707-2722`
pub fn Item_ListBox_MaxScroll(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> c_int {
    let it = menus.item(item);
    let listPtr = match it.typeData.listBox() {
        Some(l) => l,
        None => return 0,
    };
    let count = dc.feederCount(it.special);
    // Raven assigns the whole float expression to `int max` — one truncation
    // at the end, not on the division alone.
    let max = if it.window.flags & WINDOW_HORIZONTAL != 0 {
        (count as f32 - it.window.rect.w / listPtr.elementWidth + 1.0) as c_int
    } else {
        (count as f32 - it.window.rect.h / listPtr.elementHeight + 1.0) as c_int
    };
    if max < 0 {
        0
    } else {
        max
    }
}

/// Raven `Item_Slider_ThumbPosition` — the on-screen x of a slider's thumb.
///
/// PORT-NOTE (§19 UB pick): Raven dereferences `editDef` unconditionally past
/// the `editDef == NULL && item->cvar` early-return, which is only defined
/// when `editDef` is non-NULL; the otherwise-unreachable
/// (`editDef == None && cvar == None`) case here falls back to
/// `EditFieldDef::default()` instead of a null deref. Likewise Raven hands a
/// NULL `item->cvar` straight to `getCVarValue` when `editDef` is non-NULL;
/// that reads as `""` here.
/// Source: `oracle/codemp/ui/ui_shared.c:2790-2821`
pub fn Item_Slider_ThumbPosition(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> f32 {
    let it = menus.item(item);
    let editDef = it.typeData.editField();

    let mut x = if it.text.is_some() {
        it.textRect.x + it.textRect.w + 8.0
    } else {
        it.window.rect.x
    };

    if editDef.is_none() && it.cvar.is_some() {
        return x;
    }

    let default_edit = EditFieldDef::default();
    let editDef = editDef.unwrap_or(&default_edit);

    let mut value = dc.getCVarValue(it.cvar.as_deref().unwrap_or(""));
    if value < editDef.minVal {
        value = editDef.minVal;
    } else if value > editDef.maxVal {
        value = editDef.maxVal;
    }

    let range = editDef.maxVal - editDef.minVal;
    value -= editDef.minVal;
    value /= range;
    value *= SLIDER_WIDTH;
    x += value;
    x
}

/// Raven `Item_SetMouseOver` — set/clear `WINDOW_MOUSEOVER` on `item`, if any.
/// Source: `oracle/codemp/ui/ui_shared.c:3111-3119`
pub fn Item_SetMouseOver(menus: &mut MenuSystem, item: Option<ItemId>, focus: bool) {
    if let Some(item) = item {
        let it = menus.item_mut(item);
        if focus {
            it.window.flags |= WINDOW_MOUSEOVER;
        } else {
            it.window.flags &= !WINDOW_MOUSEOVER;
        }
    }
}

/// Raven `Item_Multi_CountSettings` — number of settings in a multi-value
/// item's cycle list.
/// Source: `oracle/codemp/ui/ui_shared.c:3504-3510`
pub fn Item_Multi_CountSettings(menus: &MenuSystem, item: ItemId) -> c_int {
    match menus.item(item).typeData.multi() {
        Some(m) => m.cvarList.len() as c_int,
        None => 0,
    }
}

/// Raven `Item_Multi_FindCvarByValue` — the cycle-list index whose value
/// matches the item's current cvar value.
/// Source: `oracle/codemp/ui/ui_shared.c:3512-3536`
pub fn Item_Multi_FindCvarByValue(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> c_int {
    let it = menus.item(item);
    let multiPtr = match it.typeData.multi() {
        Some(m) => m,
        None => return 0,
    };

    let mut value = 0.0f32;
    let mut buff = String::new();
    if multiPtr.strDef {
        buff = dc.getCVarString(it.cvar.as_deref().unwrap_or(""), 2048);
    } else {
        value = dc.getCVarValue(it.cvar.as_deref().unwrap_or(""));
    }

    for i in 0..multiPtr.cvarList.len() {
        if multiPtr.strDef {
            if stricmp_eq(&buff, &multiPtr.cvarStr[i]) {
                return i as c_int;
            }
        } else if multiPtr.cvarValue[i] == value {
            return i as c_int;
        }
    }
    0
}

/// Raven `Item_Multi_Setting` — the display string for the cycle-list index
/// whose value matches the item's current cvar value (`""` if none matches
/// or there is no multi payload).
/// Source: `oracle/codemp/ui/ui_shared.c:3538-3579`
pub fn Item_Multi_Setting(menus: &MenuSystem, dc: &mut dyn DisplayContext, item: ItemId) -> String {
    let it = menus.item(item);
    let multiPtr = match it.typeData.multi() {
        Some(m) => m,
        None => return String::new(),
    };

    let mut value = 0.0f32;
    let mut buff = String::new();
    if multiPtr.strDef {
        if let Some(cvar) = it.cvar.as_deref() {
            buff = dc.getCVarString(cvar, 2048);
        }
    } else if let Some(cvar) = it.cvar.as_deref() {
        // Was a cvar given?
        value = dc.getCVarValue(cvar);
    }

    for i in 0..multiPtr.cvarList.len() {
        if multiPtr.strDef {
            if stricmp_eq(&buff, &multiPtr.cvarStr[i]) {
                return multiPtr.cvarList[i].clone();
            }
        } else if multiPtr.cvarValue[i] == value {
            return multiPtr.cvarList[i].clone();
        }
    }
    String::new()
}

/// Raven `Leaving_EditField` — reset an edit field's paint offset when focus
/// leaves it.
/// Source: `oracle/codemp/ui/ui_shared.c:3651-3662`
pub fn Leaving_EditField(menus: &mut MenuSystem, item: ItemId) {
    let editing = menus.g_editingField;
    let it = menus.item_mut(item);
    if editing && it.r#type == ITEM_TYPE_EDITFIELD {
        if let Some(editPtr) = it.typeData.editField_mut() {
            editPtr.paintOffset = 0;
        }
    }
}

/// Raven `Scroll_Slider_ThumbFunc` — drag the captured slider's cvar value to
/// track the cursor.
///
/// PORT-NOTE (§19 UB pick): Raven derefs `si->item` and casts `typeData`
/// unconditionally, both of which are always valid while a slider owns mouse
/// capture; the otherwise-unreachable (no captured item / non-edit-field
/// payload) case here returns without effect instead of a null deref.
/// Source: `oracle/codemp/ui/ui_shared.c:3997-4020`
pub fn Scroll_Slider_ThumbFunc(menus: &MenuSystem, ds: &DisplayState, dc: &mut dyn DisplayContext) {
    let item = match menus.scrollInfo.item {
        Some(id) => id,
        None => return,
    };
    let it = menus.item(item);
    let editDef = match it.typeData.editField() {
        Some(e) => e,
        None => return,
    };

    let x = if it.text.is_some() {
        it.textRect.x + it.textRect.w + 8.0
    } else {
        it.window.rect.x
    };

    let mut cursorx = ds.cursorx as f32;
    if cursorx < x {
        cursorx = x;
    } else if cursorx > x + SLIDER_WIDTH {
        cursorx = x + SLIDER_WIDTH;
    }
    let mut value = cursorx - x;
    value /= SLIDER_WIDTH;
    value *= editDef.maxVal - editDef.minVal;
    value += editDef.minVal;
    dc.setCVar(it.cvar.as_deref().unwrap_or(""), &format!("{:.6}", value));
}

/// Raven `Item_StopCapture` — mouse-capture release hook; empty body.
/// Source: `oracle/codemp/ui/ui_shared.c:4097-4099`
pub fn Item_StopCapture(_item: ItemId) {}

/// Raven `Window_CloseCinematic` — stop and clear a window's cinematic if one
/// is playing.
/// Source: `oracle/codemp/ui/ui_shared.c:4389-4394`
pub fn Window_CloseCinematic(window: &mut WindowDef, dc: &mut dyn DisplayContext) {
    if window.style == WINDOW_STYLE_CINEMATIC && window.cinematic >= 0 {
        dc.stopCinematic(window.cinematic);
        window.cinematic = -1;
    }
}

/// Raven `Display_VisibleMenuCount` — number of defined menus currently
/// forced open or visible.
/// Source: `oracle/codemp/ui/ui_shared.c:4442-4451`
pub fn Display_VisibleMenuCount(menus: &MenuSystem) -> c_int {
    menus
        .menus
        .iter()
        .filter(|m| m.window.flags & (WINDOW_FORCED | WINDOW_VISIBLE) != 0)
        .count() as c_int
}

/// Raven `ToWindowCoords` — offset a point into `window`'s client rect
/// (adding the border inset first, if bordered).
/// Source: `oracle/codemp/ui/ui_shared.c:4727-4734`
pub fn ToWindowCoords(x: &mut f32, y: &mut f32, window: &WindowDef) {
    if window.border != 0 {
        *x += window.borderSize;
        *y += window.borderSize;
    }
    *x += window.rect.x;
    *y += window.rect.y;
}

/// Raven `Controls_GetKeyAssignment` — the up-to-two keys bound to `command`.
///
/// PORT-NOTE: the `int *twokeys` out-param becomes the returned pair (index 0
/// = first match, index 1 = second, `-1` for "no match" exactly as Raven's
/// `twokeys[0] = twokeys[1] = -1` initializer).
/// Source: `oracle/codemp/ui/ui_shared.c:5302-5325`
pub fn Controls_GetKeyAssignment(dc: &mut dyn DisplayContext, command: &str) -> (c_int, c_int) {
    let mut twokeys = [-1_i32, -1_i32];
    let mut count = 0usize;

    for j in 0..MAX_KEYS {
        let b = dc.getBindingBuf(j, 256);
        if b.is_empty() {
            continue;
        }
        if stricmp_eq(&b, command) {
            twokeys[count] = j;
            count += 1;
            if count == 2 {
                break;
            }
        }
    }
    (twokeys[0], twokeys[1])
}

/// Raven `Controls_SetConfig` — push every bound control's key(s) to the host.
///
/// Raven's `restart` param gates only the commented-out mouse/sound-cvar tail
/// below (never live code even in the oracle); kept for signature parity.
/// Source: `oracle/codemp/ui/ui_shared.c:5362-5393`
pub fn Controls_SetConfig(menus: &MenuSystem, dc: &mut dyn DisplayContext, _restart: bool) {
    for b in &menus.g_bindings {
        if b.bind1 != -1 {
            dc.setBinding(b.bind1, b.command);
            if b.bind2 != -1 {
                dc.setBinding(b.bind2, b.command);
            }
        }
    }
}

/// Raven `BindingIDFromName` — the `g_bindings` row index for `name`, `-1` if
/// none.
/// Source: `oracle/codemp/ui/ui_shared.c:5396-5405`
pub fn BindingIDFromName(menus: &MenuSystem, name: &str) -> c_int {
    for (i, b) in menus.g_bindings.iter().enumerate() {
        if stricmp_eq(name, b.command) {
            return i as c_int;
        }
    }
    -1
}

/// Raven `BindingFromName` — rebuild `g_nameBind1`/`g_nameBind2` as the
/// display string for `cvar`'s current key binding(s) (`"key1 OR key2"` when
/// two are bound).
///
/// PORT-NOTE: Raven writes `keynumToStringBuf` straight into the file-scope
/// `g_nameBind1`/`g_nameBind2` buffers and `strcat`s onto them in place; this
/// keeps that shape against the owned `MenuSystem` fields. The
/// `trap_SP_GetStringTextString` return value is unchecked in the oracle (the
/// `sOR` buffer is `strcat`'d regardless), so a lookup failure here falls back
/// to appending an empty string rather than propagating `None`.
/// §19: Raven's `strcat` onto the 32-byte `g_nameBind1[32]` overruns whenever
/// `key1 OR key2` exceeds 31 bytes; the owned `String` takes the one defined
/// behavior (no truncation, no overrun).
/// Source: `oracle/codemp/ui/ui_shared.c:5410-5441`
pub fn BindingFromName(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, cvar: &str) {
    for i in 0..menus.g_bindings.len() {
        let b = menus.g_bindings[i];
        if stricmp_eq(cvar, b.command) {
            if b.bind1 == -1 {
                break;
            }
            menus.g_nameBind1 = dc.keynumToStringBuf(b.bind1, 32);

            if b.bind2 != -1 {
                menus.g_nameBind2 = dc.keynumToStringBuf(b.bind2, 32);
                let sOR = dc
                    .SP_GetStringTextString("MENUS_KEYBIND_OR", 32)
                    .unwrap_or_default();
                menus.g_nameBind1.push_str(&format!(" {} ", sOR));
                let bind2 = menus.g_nameBind2.clone();
                menus.g_nameBind1.push_str(&bind2);
            }
            return;
        }
    }
    menus.g_nameBind1 = "???".to_string();
}

/// Raven `Display_KeyBindPending` — is a key-bind capture in progress?
/// Source: `oracle/codemp/ui/ui_shared.c:5548-5550`
pub fn Display_KeyBindPending(menus: &MenuSystem) -> bool {
    menus.g_waitingForKey
}

/// Raven `UI_ScaleModelAxis` — scale each non-unit model axis in place.
/// Source: `oracle/codemp/ui/ui_shared.c:5668-5686`
pub fn UI_ScaleModelAxis(ent: &mut refEntity_t) {
    for i in 0..3 {
        let s = ent.modelScale[i];
        if s != 0.0 && s != 1.0 {
            for j in 0..3 {
                ent.axis[i][j] *= s;
            }
            ent.nonNormalizedAxes = qtrue;
        }
    }
}

/// Raven `Item_Image_Paint` — draw an image item's asset inset one pixel into
/// its window rect.
/// Source: `oracle/codemp/ui/ui_shared.c:5903-5908`
pub fn Item_Image_Paint(menus: &MenuSystem, dc: &mut dyn DisplayContext, item: Option<ItemId>) {
    let item = match item {
        Some(id) => id,
        None => return,
    };
    let it = menus.item(item);
    dc.drawHandlePic(
        it.window.rect.x + 1.0,
        it.window.rect.y + 1.0,
        it.window.rect.w - 2.0,
        it.window.rect.h - 2.0,
        it.asset,
    );
}

/// Raven `Menu_GetFocusedItem` — the item in `menu` that currently has
/// keyboard focus, if any.
/// Source: `oracle/codemp/ui/ui_shared.c:7024-7034`
pub fn Menu_GetFocusedItem(menus: &MenuSystem, menu: Option<MenuId>) -> Option<ItemId> {
    let menu = menu?;
    for &id in &menus.menu(menu).items {
        if menus.item(id).window.flags & WINDOW_HASFOCUS != 0 {
            return Some(id);
        }
    }
    None
}

/// Raven `Menu_GetFocused` — the visible defined menu that currently has
/// keyboard focus, if any.
/// Source: `oracle/codemp/ui/ui_shared.c:7036-7044`
pub fn Menu_GetFocused(menus: &MenuSystem) -> Option<MenuId> {
    for (i, m) in menus.menus.iter().enumerate() {
        if m.window.flags & WINDOW_HASFOCUS != 0 && m.window.flags & WINDOW_VISIBLE != 0 {
            return Some(MenuId::new(i));
        }
    }
    None
}

/// Raven `Menus_AnyFullScreenVisible` — is any full-screen menu currently
/// visible?
/// Source: `oracle/codemp/ui/ui_shared.c:7086-7094`
pub fn Menus_AnyFullScreenVisible(menus: &MenuSystem) -> bool {
    menus
        .menus
        .iter()
        .any(|m| m.window.flags & WINDOW_VISIBLE != 0 && m.fullScreen)
}

/// Raven `KeywordHash_Key` — case-folded weighted-sum hash into
/// `KEYWORDHASH_SIZE` buckets, for the menu/item parse keyword tables.
///
/// PORT-NOTE: `int register hash` is a 32-bit accumulator; `i32::wrapping_*`
/// reproduces C `int`'s defined-overflow-free wraparound exactly.
/// Source: `oracle/codemp/ui/ui_shared.c:7333-7345`
pub fn KeywordHash_Key(keyword: &str) -> c_int {
    let mut hash: i32 = 0;
    for (i, b) in keyword.bytes().enumerate() {
        let letter = if b.is_ascii_uppercase() {
            (b + (b'a' - b'A')) as i32
        } else {
            // Raven's `char keyword[i]` is SIGNED — bytes ≥ 0x80 go negative.
            b as i8 as i32
        };
        hash = hash.wrapping_add(letter.wrapping_mul(119 + i as i32));
    }
    hash ^= (hash >> 10) ^ (hash >> 20);
    hash & (KEYWORDHASH_SIZE - 1)
}

/// Raven `ItemParse_focusSound` — parse the item's mouse-focus sound asset.
/// Source: `oracle/codemp/ui/ui_shared.c:7388-7395`
pub fn ItemParse_focusSound(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);
    menus.item_mut(item).focusSound = dc.registerSound(&name);
    true
}

/// Raven `UI_InsertG2Pointer` — track a ghoul2 instance the ui created,
/// reusing a cleared slot if one exists.
///
/// PORT-NOTE (D2 pool retirement): Raven's `uiG2PtrTracker_t` intrusive
/// singly-linked list (`BG_Alloc`-backed nodes, reused by scanning for a
/// `ghoul2 == NULL` node) is `MenuSystem::ui_G2PtrTracker: Vec<*mut c_void>` —
/// a cleared slot is a `null` entry in the vec, so the reuse scan is an
/// iterator search and a fresh node is `Vec::push`; there is no node
/// allocation left for `BG_Alloc` to perform.
/// Source: `oracle/codemp/ui/ui_shared.c:7492-7514`
pub fn UI_InsertG2Pointer(menus: &mut MenuSystem, ghoul2: *mut c_void) {
    for slot in menus.ui_G2PtrTracker.iter_mut() {
        if slot.is_null() {
            *slot = ghoul2;
            return;
        }
    }
    menus.ui_G2PtrTracker.push(ghoul2);
}

/// Raven `UI_ClearG2Pointer` — release the tracked slot for `ghoul2` (if any)
/// so it can be reused.
/// Source: `oracle/codemp/ui/ui_shared.c:7517-7536`
pub fn UI_ClearG2Pointer(menus: &mut MenuSystem, ghoul2: *mut c_void) {
    if ghoul2.is_null() {
        return;
    }
    for slot in menus.ui_G2PtrTracker.iter_mut() {
        if *slot == ghoul2 {
            *slot = null_mut();
            break;
        }
    }
}

/// Raven `UI_CleanupGhoul2` — clean every tracked ghoul2 instance still alive.
///
/// PORT-NOTE (dead surface): Raven's `#ifdef _XBOX` tail (resetting the list
/// head) is dead on every retail/live target this port ships; dropped per
/// porting-rules §20.
/// Source: `oracle/codemp/ui/ui_shared.c:7539-7556`
pub fn UI_CleanupGhoul2(menus: &mut MenuSystem, dc: &mut dyn DisplayContext) {
    for slot in menus.ui_G2PtrTracker.iter_mut() {
        if !slot.is_null() && dc.G2_HaveWeGhoul2Models(*slot) {
            dc.G2API_CleanGhoul2Models(slot as *mut *mut c_void);
        }
    }
}

/// Raven `ItemParse_asset_shader` — parse the item's asset shader.
/// Source: `oracle/codemp/ui/ui_shared.c:7687-7694`
pub fn ItemParse_asset_shader(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);
    menus.item_mut(item).asset = dc.registerShaderNoMip(&name);
    true
}

/// Raven `ItemParse_decoration` — mark an item decoration-only (no mouse or
/// keyboard interaction).
/// Source: `oracle/codemp/ui/ui_shared.c:8023-8026`
pub fn ItemParse_decoration(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    menus.item_mut(item).window.flags |= WINDOW_DECORATION;
    true
}

/// Raven `ItemParse_wrapped` — mark an item's text as manually wrapped.
/// Source: `oracle/codemp/ui/ui_shared.c:8063-8066`
pub fn ItemParse_wrapped(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    menus.item_mut(item).window.flags |= WINDOW_WRAPPED;
    true
}

/// Raven `ItemParse_autowrapped` — mark an item's text as auto-wrapped.
/// Source: `oracle/codemp/ui/ui_shared.c:8069-8072`
pub fn ItemParse_autowrapped(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    menus.item_mut(item).window.flags |= WINDOW_AUTOWRAPPED;
    true
}

/// Raven `ItemParse_horizontalscroll` — mark a list box as horizontally
/// scrolling.
/// Source: `oracle/codemp/ui/ui_shared.c:8076-8079`
pub fn ItemParse_horizontalscroll(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    menus.item_mut(item).window.flags |= WINDOW_HORIZONTAL;
    true
}

/// Raven `ItemParse_background` — parse an item window's background shader.
/// Source: `oracle/codemp/ui/ui_shared.c:8376-8384`
pub fn ItemParse_background(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);
    menus.item_mut(item).window.background = dc.registerShaderNoMip(&name);
    true
}

/// Raven `Item_InitControls` — reset the per-type interactive state a fresh
/// item starts with (currently only list boxes carry any).
/// Source: `oracle/codemp/ui/ui_shared.c:9259-9283`
pub fn Item_InitControls(menus: &mut MenuSystem, item: Option<ItemId>) {
    let item = match item {
        Some(id) => id,
        None => return,
    };
    let it = menus.item_mut(item);
    if it.r#type == ITEM_TYPE_LISTBOX {
        it.cursorPos = 0;
        if let Some(listPtr) = it.typeData.listBox_mut() {
            listPtr.cursorPos = 0;
            listPtr.startPos = 0;
            listPtr.endPos = 0;
            listPtr.cursorPos = 0;
        }
    }
}

/// Raven `MenuParse_background` — parse a menu window's background shader.
///
/// PORT-NOTE: Raven's `itemDef_t *item` parameter is immediately cast to
/// `menuDef_t *menu` (every `MenuParse_*` handler shares the item-parser
/// callback signature but only ever runs against the enclosing menu); this
/// takes the `MenuId` the cast resolves to directly.
/// Source: `oracle/codemp/ui/ui_shared.c:9594-9603`
pub fn MenuParse_background(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);
    menus.menu_mut(menu).window.background = dc.registerShaderNoMip(&name);
    true
}

/// Raven `MenuParse_popup` — mark a menu as a popup.
///
/// PORT-NOTE: see `MenuParse_background` — the `itemDef_t *` cast to
/// `menuDef_t *` becomes a direct `MenuId`.
/// Source: `oracle/codemp/ui/ui_shared.c:9636-9640`
pub fn MenuParse_popup(menus: &mut MenuSystem, menu: MenuId, _handle: c_int) -> bool {
    menus.menu_mut(menu).window.flags |= WINDOW_POPUP;
    true
}

/// Raven `MenuParse_outOfBounds` — mark a menu to close on an out-of-bounds
/// click.
///
/// PORT-NOTE: see `MenuParse_background` — the `itemDef_t *` cast to
/// `menuDef_t *` becomes a direct `MenuId`.
/// Source: `oracle/codemp/ui/ui_shared.c:9643-9648`
pub fn MenuParse_outOfBounds(menus: &mut MenuSystem, menu: MenuId, _handle: c_int) -> bool {
    menus.menu_mut(menu).window.flags |= WINDOW_OOB_CLICK;
    true
}

/// Raven `Menu_Count` — the number of defined menus.
/// Source: `oracle/codemp/ui/ui_shared.c:9829-9831`
pub fn Menu_Count(menus: &MenuSystem) -> c_int {
    menus.menuCount()
}

/// Raven `Menu_Reset` — discard every defined menu.
/// Source: `oracle/codemp/ui/ui_shared.c:9850-9852`
pub fn Menu_Reset(menus: &mut MenuSystem) {
    menus.menus.clear();
}

// `Display_GetContext` — ui_shared.c:9854-9856.
//
// DEFERRED: Display_GetContext — DEC-36 D3 threads `DisplayContext` as a
// per-call `&mut dyn DisplayContext` argument, not a stored file-scope `DC`
// pointer (U3 ruling, 2026-07-24), matching `Init_Display` above; there is no
// stored `DC` left to hand back and no owned-shape equivalent to return.
// Source: `oracle/codemp/ui/ui_shared.c:9854-9856`

/// Raven `Window_CacheContents` — pre-roll and immediately stop a window's
/// cinematic so its first frame is cached.
/// Source: `oracle/codemp/ui/ui_shared.c:9928-9935`
pub fn Window_CacheContents(window: Option<&WindowDef>, dc: &mut dyn DisplayContext) {
    if let Some(window) = window {
        if !window.cinematicName.is_empty() {
            let cin = dc.playCinematic(&window.cinematicName, 0.0, 0.0, 0.0, 0.0);
            dc.stopCinematic(cin);
        }
    }
}

/// Raven `String_Alloc` — intern `p` into the shared string pool, returning
/// the pool's owned copy (`""` for an empty string, `NULL` unchanged).
///
/// PORT-NOTE (D2 pool retirement): `MenuSystem` fields are owned `String`s
/// directly (§B9/§C9), so there is no `strPool`/`strHandle` intern table left
/// to dedupe against; the closest owned-shape equivalent is handing back an
/// owned copy of `p` itself — every caller already wants one owned `String`,
/// which this now always allocates fresh rather than sharing a pooled buffer.
/// The `assert(0)`/pool-exhaustion tail (`len + strPoolIndex + 1 >=
/// STRING_POOL_SIZE`) cannot trigger against an unbounded `String` and is
/// dropped, matching `UI_OutOfMemory`'s PORT-NOTE.
/// Source: `oracle/codemp/ui/ui_shared.c:291-342`
pub fn String_Alloc(p: Option<&str>) -> Option<String> {
    let p = p?;
    if p.is_empty() {
        return Some(String::new());
    }
    Some(p.to_string())
}

/// Raven `String_Report` — print the string/memory pool fill percentages.
///
/// PORT-NOTE (D2 pool retirement): neither pool exists anymore (see
/// `UI_Alloc`/`String_Alloc`), so there is no fill percentage to compute;
/// prints the closest owned-shape equivalent instead of a `%` against a
/// retired ceiling. Raven's `Com_Printf` is unreachable from this
/// host-agnostic crate (it threads `UiContext`, `mp_ui`-only) — routed through
/// `dc.Print`, the same trap-seam swap every direct-print call in this file
/// takes.
/// Source: `oracle/codemp/ui/ui_shared.c:344-356`
pub fn String_Report(dc: &mut dyn DisplayContext) {
    dc.Print("Memory/String Pool Info\n");
    dc.Print("----------------\n");
    dc.Print("String Pool: owned Strings (D2 pool retirement) — no fixed-size ceiling.\n");
    dc.Print("Memory Pool: owned allocations (D2 pool retirement) — no fixed-size ceiling.\n");
}

/// Raven `PC_SourceWarning` — print a yellow `"WARNING: <file>, line <n>:
/// <msg>"` for a parse-source token.
///
/// PORT-NOTE: the variadic `format, ...` tail collapses to one pre-formatted
/// `message: &str` (dictionary: `va()`/`Com_sprintf` → `format!` at the call
/// site); `Com_Printf` is unreachable from this host-agnostic crate (see
/// `String_Report`) and is routed through `dc.Print`.
/// Source: `oracle/codemp/ui/ui_shared.c:385-400`
pub fn PC_SourceWarning(dc: &mut dyn DisplayContext, handle: c_int, message: &str) {
    let (_status, filename, line) = dc.PC_SourceFileAndLine(handle, 128);
    dc.Print(&format!(
        "^3WARNING: {}, line {}: {}\n",
        filename, line, message
    ));
}

/// Raven `PC_SourceError` — print a red `"ERROR: <file>, line <n>: <msg>"` for
/// a parse-source token.
///
/// PORT-NOTE: see `PC_SourceWarning`.
/// Source: `oracle/codemp/ui/ui_shared.c:407-422`
pub fn PC_SourceError(dc: &mut dyn DisplayContext, handle: c_int, message: &str) {
    let (_status, filename, line) = dc.PC_SourceFileAndLine(handle, 128);
    dc.Print(&format!(
        "^1ERROR: {}, line {}: {}\n",
        filename, line, message
    ));
}

/// Raven `Color_Parse` — parse four floats off `p` into `c`; `true` only if
/// all four parsed.
/// Source: `oracle/codemp/ui/ui_shared.c:492-503`
pub fn Color_Parse(p: &mut &str, c: &mut vec4_t) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !Float_Parse(p, &mut f) {
            return false;
        }
        c[i] = f;
    }
    true
}

/// Raven `Rect_Parse` — parse `x y w h` off `p` into `r`; `true` only if all
/// four floats parsed (the nested-`if` nesting Raven uses instead of
/// [`ParseRect`]'s early-return style — same behavior, different file
/// location's shape).
/// Source: `oracle/codemp/ui/ui_shared.c:571-582`
pub fn Rect_Parse(p: &mut &str, r: &mut RectDef) -> bool {
    if Float_Parse(p, &mut r.x) {
        if Float_Parse(p, &mut r.y) {
            if Float_Parse(p, &mut r.w) {
                if Float_Parse(p, &mut r.h) {
                    return true;
                }
            }
        }
    }
    false
}

/// Raven `Window_Paint` — paint one window's background/fill and border.
///
/// PORT-NOTE: `DC->getTeamColor` is checked for non-NULL before calling in
/// Raven; `DisplayContext` always implements every method (see
/// `Script_SetTeamColor`), so the guard collapses to an unconditional call.
/// `ui_char_color_red/green/blue` are `mp_ui`-registered cvars this
/// host-agnostic crate cannot reach as cached `vmCvar_t`s; read live through
/// the generic `dc.getCVarValue` accessor instead (same value Raven's cached
/// copy holds, refreshed every frame elsewhere).
/// Source: `oracle/codemp/ui/ui_shared.c:748-888`
pub fn Window_Paint(
    menus: &MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    w: &mut WindowDef,
    fadeAmount: f32,
    fadeClamp: f32,
    fadeCycle: c_int,
) {
    let mut fillRect = w.rect;

    if menus.debugMode {
        let color: vec4_t = [1.0, 1.0, 1.0, 1.0];
        dc.drawRect(w.rect.x, w.rect.y, w.rect.w, w.rect.h, 1.0, color);
    }

    if w.style == 0 && w.border == 0 {
        return;
    }

    if w.border != 0 {
        fillRect.x += w.borderSize;
        fillRect.y += w.borderSize;
        fillRect.w -= w.borderSize + 1.0;
        fillRect.h -= w.borderSize + 1.0;
    }

    let mut color: vec4_t = [0.0; 4];

    if w.style == WINDOW_STYLE_FILLED {
        // box, but possible a shader that needs filled
        if w.background != 0 {
            Fade(
                ds,
                &mut w.flags,
                &mut w.backColor[3],
                fadeClamp,
                &mut w.nextTime,
                fadeCycle,
                true,
                fadeAmount,
            );
            dc.setColor(Some(w.backColor));
            dc.drawHandlePic(fillRect.x, fillRect.y, fillRect.w, fillRect.h, w.background);
            dc.setColor(None);
        } else {
            dc.fillRect(fillRect.x, fillRect.y, fillRect.w, fillRect.h, w.backColor);
        }
    } else if w.style == WINDOW_STYLE_GRADIENT {
        // gradient bar
        GradientBar_Paint(ds, dc, &fillRect, w.backColor);
    } else if w.style == WINDOW_STYLE_SHADER {
        // PORT-NOTE: the `WINDOW_PLAYERCOLOR` block is the `#ifndef CGAME` (ui) arm,
        // per this file's convention.
        if w.flags & WINDOW_PLAYERCOLOR != 0 {
            // PORT-NOTE: Raven reads `ui_char_color_*.integer`; the `as c_int`
            // truncation is preserved.
            let mut playerColor: vec4_t = [0.0; 4];
            playerColor[0] = (dc.getCVarValue("ui_char_color_red") as c_int) as f32 / 255.0;
            playerColor[1] = (dc.getCVarValue("ui_char_color_green") as c_int) as f32 / 255.0;
            playerColor[2] = (dc.getCVarValue("ui_char_color_blue") as c_int) as f32 / 255.0;
            playerColor[3] = 1.0;
            dc.setColor(Some(playerColor));
        }

        if w.flags & WINDOW_FORECOLORSET != 0 {
            dc.setColor(Some(w.foreColor));
        }
        dc.drawHandlePic(fillRect.x, fillRect.y, fillRect.w, fillRect.h, w.background);
        dc.setColor(None);
    } else if w.style == WINDOW_STYLE_TEAMCOLOR {
        color = dc.getTeamColor();
        dc.fillRect(fillRect.x, fillRect.y, fillRect.w, fillRect.h, color);
    } else if w.style == WINDOW_STYLE_CINEMATIC {
        if w.cinematic == -1 {
            w.cinematic = dc.playCinematic(
                &w.cinematicName,
                fillRect.x,
                fillRect.y,
                fillRect.w,
                fillRect.h,
            );
            if w.cinematic == -1 {
                w.cinematic = -2;
            }
        }
        if w.cinematic >= 0 {
            dc.runCinematicFrame(w.cinematic);
            dc.drawCinematic(w.cinematic, fillRect.x, fillRect.y, fillRect.w, fillRect.h);
        }
    }

    if w.border == WINDOW_BORDER_FULL {
        // full
        // HACK HACK HACK
        if w.style == WINDOW_STYLE_TEAMCOLOR {
            if color[0] > 0.0 {
                // red
                color[0] = 1.0;
                color[1] = 0.5;
                color[2] = 0.5;
            } else {
                color[2] = 1.0;
                color[0] = 0.5;
                color[1] = 0.5;
            }
            color[3] = 1.0;
            dc.drawRect(w.rect.x, w.rect.y, w.rect.w, w.rect.h, w.borderSize, color);
        } else {
            dc.drawRect(
                w.rect.x,
                w.rect.y,
                w.rect.w,
                w.rect.h,
                w.borderSize,
                w.borderColor,
            );
        }
    } else if w.border == WINDOW_BORDER_HORZ {
        // top/bottom
        dc.setColor(Some(w.borderColor));
        dc.drawTopBottom(w.rect.x, w.rect.y, w.rect.w, w.rect.h, w.borderSize);
        dc.setColor(None);
    } else if w.border == WINDOW_BORDER_VERT {
        // left right
        dc.setColor(Some(w.borderColor));
        dc.drawSides(w.rect.x, w.rect.y, w.rect.w, w.rect.h, w.borderSize);
        dc.setColor(None);
    } else if w.border == WINDOW_BORDER_KCGRADIENT {
        // this is just two gradient bars along each horz edge
        let mut r = w.rect;
        r.h = w.borderSize;
        GradientBar_Paint(ds, dc, &r, w.borderColor);
        r.y = w.rect.y + w.rect.h - 1.0;
        GradientBar_Paint(ds, dc, &r, w.borderColor);
    }
}

/// Raven `Menu_ItemsMatchingGroup` — the number of `menu`'s items whose name
/// or group matches `name`.
///
/// PORT-NOTE: `Com_Printf` (the "item has neither name or group" warning) is
/// unreachable from this host-agnostic crate (see `String_Report`), so this
/// takes a `dc` param beyond the packet's "pure fn" classification — routed
/// through `dc.Print` like every other direct-print call site in this file.
/// Source: `oracle/codemp/ui/ui_shared.c:1026-1047`
pub fn Menu_ItemsMatchingGroup(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    name: &str,
) -> c_int {
    let mut count = 0;
    for &id in &menus.menu(menu).items {
        let it = menus.item(id);
        if it.window.name.is_none() && it.window.group.is_none() {
            dc.Print("^3WARNING: item has neither name or group\n");
            continue;
        }

        if it
            .window
            .name
            .as_deref()
            .is_some_and(|n| stricmp_eq(n, name))
            || it
                .window
                .group
                .as_deref()
                .is_some_and(|g| stricmp_eq(g, name))
        {
            count += 1;
        }
    }
    count
}

/// Raven `Item_TextScroll_ThumbPosition` — the on-screen y of a text-scroll
/// box's thumb.
///
/// PORT-NOTE (§19 UB pick): see `Item_ListBox_ThumbPosition` — an
/// otherwise-unreachable payload-type mismatch falls back to `startPos = 0`
/// instead of a null deref.
/// Source: `oracle/codemp/ui/ui_shared.c:2497-2517`
pub fn Item_TextScroll_ThumbPosition(menus: &MenuSystem, item: ItemId) -> c_int {
    let max = Item_TextScroll_MaxScroll(menus, item);
    let it = menus.item(item);
    let startPos = it.typeData.textScroll().map(|s| s.startPos).unwrap_or(0);

    let size = it.window.rect.h - (SCROLLBAR_SIZE * 2.0) - 2.0;
    let mut pos = if max > 0 {
        (size - SCROLLBAR_SIZE) / max as f32
    } else {
        0.0
    };
    pos *= startPos as f32;

    (it.window.rect.y + 1.0 + SCROLLBAR_SIZE + pos) as c_int
}

/// Raven `Item_TextScroll_HandleKey` — cursor/mouse/paging scroll input for a
/// text-scroll box.
///
/// PORT-NOTE: `down` is unused — Raven's own body never reads it either
/// (kept for signature parity).
/// Source: `oracle/codemp/ui/ui_shared.c:2593-2705`
pub fn Item_TextScroll_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    item: ItemId,
    key: c_int,
    _down: bool,
    force: bool,
) -> bool {
    let rect = menus.item(item).window.rect;
    let flags = menus.item(item).window.flags;

    if force
        || (Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
            && flags & WINDOW_HASFOCUS != 0)
    {
        let max = Item_TextScroll_MaxScroll(menus, item);
        // PORT-NOTE (§19 UB pick): a payload-type mismatch (Raven: null deref)
        // falls back to `lineHeight = 1.0`.
        let lineHeight = menus
            .item(item)
            .typeData
            .textScroll()
            .map(|s| s.lineHeight)
            .unwrap_or(1.0);
        // PORT-NOTE (§19 UB pick): Rust's saturating float->int cast stands in for
        // C's undefined out-of-range truncation.
        let viewmax = (rect.h / lineHeight) as c_int;

        if key == A_CURSOR_UP || key == A_KP_8 {
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos -= 1;
                if s.startPos < 0 {
                    s.startPos = 0;
                }
            }
            return true;
        }

        if key == A_CURSOR_DOWN || key == A_KP_2 {
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos += 1;
                if s.startPos > max {
                    s.startPos = max;
                }
            }
            return true;
        }

        // mouse hit
        if key == A_MOUSE1 || key == A_MOUSE2 {
            if flags & WINDOW_LB_LEFTARROW != 0 {
                if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                    s.startPos -= 1;
                    if s.startPos < 0 {
                        s.startPos = 0;
                    }
                }
            } else if flags & WINDOW_LB_RIGHTARROW != 0 {
                // one down
                if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                    s.startPos += 1;
                    if s.startPos > max {
                        s.startPos = max;
                    }
                }
            } else if flags & WINDOW_LB_PGUP != 0 {
                // page up
                if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                    s.startPos -= viewmax;
                    if s.startPos < 0 {
                        s.startPos = 0;
                    }
                }
            } else if flags & WINDOW_LB_PGDN != 0 {
                // page down
                if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                    s.startPos += viewmax;
                    if s.startPos > max {
                        s.startPos = max;
                    }
                }
            } else if flags & WINDOW_LB_THUMB != 0 {
                // Display_SetCaptureItem(item); — commented out in Raven.
            }

            return true;
        }

        if key == A_HOME || key == A_KP_7 {
            // home
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos = 0;
            }
            return true;
        }
        if key == A_END || key == A_KP_1 {
            // end
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos = max;
            }
            return true;
        }
        if key == A_PAGE_UP || key == A_KP_9 {
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos -= viewmax;
                if s.startPos < 0 {
                    s.startPos = 0;
                }
            }
            return true;
        }
        if key == A_PAGE_DOWN || key == A_KP_3 {
            if let Some(s) = menus.item_mut(item).typeData.textScroll_mut() {
                s.startPos += viewmax;
                if s.startPos > max {
                    s.startPos = max;
                }
            }
            return true;
        }
    }

    false
}

/// Raven `Item_ListBox_ThumbPosition` — the on-screen x/y of a list box's
/// thumb (horizontal or vertical, by `WINDOW_HORIZONTAL`).
///
/// PORT-NOTE (§19 UB pick): a payload-type mismatch falls back to
/// `startPos = 0` instead of a null deref (see `Item_ListBox_MaxScroll`).
/// Source: `oracle/codemp/ui/ui_shared.c:2724-2749`
pub fn Item_ListBox_ThumbPosition(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> c_int {
    let max = Item_ListBox_MaxScroll(menus, dc, item);
    let it = menus.item(item);
    let startPos = it.typeData.listBox().map(|l| l.startPos).unwrap_or(0);

    if it.window.flags & WINDOW_HORIZONTAL != 0 {
        let size = it.window.rect.w - (SCROLLBAR_SIZE * 2.0) - 2.0;
        let mut pos = if max > 0 {
            (size - SCROLLBAR_SIZE) / max as f32
        } else {
            0.0
        };
        pos *= startPos as f32;
        (it.window.rect.x + 1.0 + SCROLLBAR_SIZE + pos) as c_int
    } else {
        let size = it.window.rect.h - (SCROLLBAR_SIZE * 2.0) - 2.0;
        let mut pos = if max > 0 {
            (size - SCROLLBAR_SIZE) / max as f32
        } else {
            0.0
        };
        pos *= startPos as f32;
        (it.window.rect.y + 1.0 + SCROLLBAR_SIZE + pos) as c_int
    }
}

/// Raven `Item_Slider_OverSlider` — is `(x, y)` over the slider's thumb?
/// Source: `oracle/codemp/ui/ui_shared.c:2823-2835`
pub fn Item_Slider_OverSlider(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    x: f32,
    y: f32,
) -> c_int {
    let thumbX = Item_Slider_ThumbPosition(menus, dc, item);
    let it = menus.item(item);
    let r = RectDef {
        x: thumbX - (SLIDER_THUMB_WIDTH / 2.0),
        y: it.window.rect.y - 2.0,
        w: SLIDER_THUMB_WIDTH,
        h: SLIDER_THUMB_HEIGHT,
    };

    if Rect_ContainsPoint(Some(&r), x, y) {
        WINDOW_LB_THUMB
    } else {
        0
    }
}

/// Raven `Menu_HitTest` — the item in `menu` under `(x, y)`, if any.
/// Source: `oracle/codemp/ui/ui_shared.c:3101-3109`
pub fn Menu_HitTest(menus: &MenuSystem, menu: MenuId, x: f32, y: f32) -> Option<ItemId> {
    for &id in &menus.menu(menu).items {
        if Rect_ContainsPoint(Some(&menus.item(id).window.rect), x, y) {
            return Some(id);
        }
    }
    None
}

/// Raven `Item_OwnerDraw_HandleKey` — dispatch a mouse/key event to an
/// owner-draw item's host handler.
///
/// PORT-NOTE: `DC->ownerDrawHandleKey` is null-checked in Raven alongside
/// `item`; `DisplayContext` always implements every method (see
/// `Script_SetTeamColor`), so only the `item` half of the guard survives as
/// `Option<ItemId>`.
/// Source: `oracle/codemp/ui/ui_shared.c:3122-3162`
pub fn Item_OwnerDraw_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
    key: c_int,
) -> bool {
    let item = match item {
        Some(id) => id,
        None => return false,
    };

    let (ownerDraw, ownerDrawFlags, rect) = {
        let it = menus.item(item);
        (
            it.window.ownerDraw,
            it.window.ownerDrawFlags,
            it.window.rect,
        )
    };

    // yep this is an ugly hack
    if key == A_MOUSE1 || key == A_MOUSE2 {
        let isForceRank = matches!(
            ownerDraw,
            UI_FORCE_SIDE
                | UI_FORCE_RANK_HEAL
                | UI_FORCE_RANK_LEVITATION
                | UI_FORCE_RANK_SPEED
                | UI_FORCE_RANK_PUSH
                | UI_FORCE_RANK_PULL
                | UI_FORCE_RANK_TELEPATHY
                | UI_FORCE_RANK_GRIP
                | UI_FORCE_RANK_LIGHTNING
                | UI_FORCE_RANK_RAGE
                | UI_FORCE_RANK_PROTECT
                | UI_FORCE_RANK_ABSORB
                | UI_FORCE_RANK_TEAM_HEAL
                | UI_FORCE_RANK_TEAM_FORCE
                | UI_FORCE_RANK_DRAIN
                | UI_FORCE_RANK_SEE
                | UI_FORCE_RANK_SABERATTACK
                | UI_FORCE_RANK_SABERDEFEND
                | UI_FORCE_RANK_SABERTHROW
        );
        if isForceRank && !Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32) {
            return false;
        }
    }

    let mut special = menus.item(item).special;
    let result = dc.ownerDrawHandleKey(ownerDraw, ownerDrawFlags, &mut special, key);
    menus.item_mut(item).special = special;
    result
}

/// Raven `Item_YesNo_HandleKey` — toggle a yes/no cvar on click/enter.
/// Source: `oracle/codemp/ui/ui_shared.c:3477-3502`
pub fn Item_YesNo_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
) -> bool {
    let (rect, flags, cvar) = {
        let it = menus.item(item);
        (it.window.rect, it.window.flags, it.cvar.clone())
    };

    if Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
        && flags & WINDOW_HASFOCUS != 0
    {
        if let Some(cvar) = cvar {
            if key == A_MOUSE1 || key == A_ENTER || key == A_MOUSE2 || key == A_MOUSE3 {
                let cur = dc.getCVarValue(&cvar);
                // C `!DC->getCVarValue(...)`: nonzero -> 0, zero -> 1.
                let newval: c_int = if cur != 0.0 { 0 } else { 1 };
                dc.setCVar(&cvar, &format!("{}", newval));
                return true;
            }
        }
    }

    false
}

/// Raven `Item_Multi_HandleKey` — cycle a multi-value cvar item on
/// click/enter.
/// Source: `oracle/codemp/ui/ui_shared.c:3581-3648`
pub fn Item_Multi_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
) -> bool {
    if menus.item(item).typeData.multi().is_none() {
        return false;
    }

    let (rect, flags) = {
        let it = menus.item(item);
        (it.window.rect, it.window.flags)
    };

    if !(Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
        && flags & WINDOW_HASFOCUS != 0)
    {
        return false;
    }

    if !(key == A_MOUSE1 || key == A_ENTER || key == A_MOUSE2 || key == A_MOUSE3) {
        return false;
    }

    let mut current = Item_Multi_FindCvarByValue(menus, dc, item);
    let max = Item_Multi_CountSettings(menus, item);

    if key == A_MOUSE2 || key == A_CURSOR_LEFT {
        current -= 1;
        if current < 0 {
            current = max - 1;
        }
    } else {
        current += 1;
        if current >= max {
            current = 0;
        }
    }

    // PORT-NOTE (§19 UB pick): with `max == 0` Raven lands on `current == -1` and
    // reads out of bounds; no selection is the defined pick.
    if current < 0 || current >= max {
        return false;
    }

    let cvar = menus.item(item).cvar.clone().unwrap_or_default();
    let (strDef, cvarStr, cvarValue) = {
        let m = menus.item(item).typeData.multi().unwrap();
        (m.strDef, m.cvarStr.clone(), m.cvarValue.clone())
    };

    // PORT-NOTE (§19 UB pick): Raven indexes the parallel arrays past `count` when
    // fewer settings parsed than `count` claims; the missing slot is skipped here.
    if strDef {
        if let Some(s) = cvarStr.get(current as usize) {
            dc.setCVar(&cvar, s);
        }
    } else if let Some(&value) = cvarValue.get(current as usize) {
        if (value as c_int) as f32 == value {
            dc.setCVar(&cvar, &format!("{}", value as c_int));
        } else {
            dc.setCVar(&cvar, &format!("{:.6}", value));
        }
    }

    // its a feeder?
    let special = menus.item(item).special;
    if special != 0.0 {
        dc.feederSelection(special, current, Some(item));
    }

    true
}

/// Raven `Item_Slider_HandleKey` — drag-select a slider's cvar value from the
/// thumb rect on click/enter.
///
/// PORT-NOTE (dead surface, §20): the `#ifdef _XBOX` d-pad tail (never
/// compiled on any retail/live target) is dropped; `down` is unused, matching
/// Raven's own body.
/// Source: `oracle/codemp/ui/ui_shared.c:4101-4173`
pub fn Item_Slider_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
    _down: bool,
) -> bool {
    let (flags, cvar, rect, text, textRect) = {
        let it = menus.item(item);
        (
            it.window.flags,
            it.cvar.clone(),
            it.window.rect,
            it.text.clone(),
            it.textRect,
        )
    };

    if flags & WINDOW_HASFOCUS != 0
        && cvar.is_some()
        && Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
    {
        if key == A_MOUSE1 || key == A_ENTER || key == A_MOUSE2 || key == A_MOUSE3 {
            let editDef = menus.item(item).typeData.editField().copied();
            if let Some(editDef) = editDef {
                let width = SLIDER_WIDTH;
                let x = if text.is_some() {
                    textRect.x + textRect.w + 8.0
                } else {
                    rect.x
                };

                let mut testRect = rect;
                testRect.x = x;
                let mut value = SLIDER_THUMB_WIDTH / 2.0;
                testRect.x -= value;
                testRect.w = SLIDER_WIDTH + SLIDER_THUMB_WIDTH / 2.0;

                if Rect_ContainsPoint(Some(&testRect), ds.cursorx as f32, ds.cursory as f32) {
                    let work = ds.cursorx as f32 - x;
                    value = work / width;
                    value *= editDef.maxVal - editDef.minVal;
                    value += editDef.minVal;
                    dc.setCVar(cvar.as_deref().unwrap_or(""), &format!("{:.6}", value));
                    return true;
                }
            }
        }
    }

    dc.Print("slider handle key exit\n");
    false
}

/// Raven `Menu_CloseCinematics` — stop every cinematic `menu` (and its
/// owner-draw items) is running.
/// Source: `oracle/codemp/ui/ui_shared.c:4396-4407`
pub fn Menu_CloseCinematics(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let itemIds = menus.menu(menu).items.clone();
    Window_CloseCinematic(&mut menus.menu_mut(menu).window, dc);
    for id in itemIds {
        let it = menus.item_mut(id);
        Window_CloseCinematic(&mut it.window, dc);
        if it.r#type == ITEM_TYPE_OWNERDRAW {
            let ownerDraw = it.window.ownerDraw;
            dc.stopCinematic(0 - ownerDraw);
        }
    }
}

/// Raven `Rect_ToWindowCoords` — offset `rect`'s origin into `window`'s
/// client rect.
/// Source: `oracle/codemp/ui/ui_shared.c:4736-4738`
pub fn Rect_ToWindowCoords(rect: &mut RectDef, window: &WindowDef) {
    ToWindowCoords(&mut rect.x, &mut rect.y, window);
}

/// Raven `Item_SetTextExtents` — compute (and cache) an item's text rect,
/// recomputing only when the width is stale or the item's alignment/
/// language demands a fresh measurement every paint.
///
/// PORT-NOTE: `se_language.modificationCount` is an `mp_ui`-owned `vmCvar_t`
/// this host-agnostic crate cannot reach as cached state; threaded in as
/// `seLanguageModCount`, the value the caller reads off its own
/// `world.se_language`. The `#ifndef CGAME` guard picks the `ui` arm (this
/// crate's only linkage so far — cgame's twin will special-case this branch
/// out when it lands).
/// Source: `oracle/codemp/ui/ui_shared.c:4740-4791`
#[allow(clippy::too_many_arguments)]
pub fn Item_SetTextExtents(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    width: &mut c_int,
    height: &mut c_int,
    text: Option<&str>,
    seLanguageModCount: c_int,
) {
    let (itemText, itemType, textalignment, asset) = {
        let it = menus.item(item);
        (it.text.clone(), it.r#type, it.textalignment, it.asset)
    };
    let textPtr = match text {
        Some(s) => Some(s.to_string()),
        None => itemText.clone(),
    };
    let textPtr = match textPtr {
        Some(s) => s,
        None => return,
    };

    let it = menus.item(item);
    *width = it.textRect.w as c_int;
    *height = it.textRect.h as c_int;

    // keeps us from computing the widths and heights more than once
    if *width == 0
        || (itemType == ITEM_TYPE_OWNERDRAW && textalignment == ITEM_ALIGN_CENTER)
        || (itemText.as_deref().is_some_and(|t| t.starts_with('@')) && asset != seLanguageModCount)
    {
        let (textscale, iMenuFont, ownerDraw, cvar) = {
            let it = menus.item(item);
            (
                it.textscale,
                it.iMenuFont,
                it.window.ownerDraw,
                it.cvar.clone(),
            )
        };
        let mut originalWidth = dc.textWidth(&textPtr, textscale, iMenuFont);

        if itemType == ITEM_TYPE_OWNERDRAW
            && (textalignment == ITEM_ALIGN_CENTER || textalignment == ITEM_ALIGN_RIGHT)
        {
            originalWidth += dc.ownerDrawWidth(ownerDraw, textscale);
        } else if itemType == ITEM_TYPE_EDITFIELD && textalignment == ITEM_ALIGN_CENTER {
            if let Some(cvar) = cvar.as_deref() {
                let buff = dc.getCVarString(cvar, 256);
                originalWidth += dc.textWidth(&buff, textscale, iMenuFont);
            }
        }

        let w = dc.textWidth(&textPtr, textscale, iMenuFont);
        let h = dc.textHeight(&textPtr, textscale, iMenuFont);
        *width = w;
        *height = h;

        let it = menus.item_mut(item);
        it.textRect.w = w as f32;
        it.textRect.h = h as f32;
        it.textRect.x = it.textalignx;
        it.textRect.y = it.textaligny;
        if textalignment == ITEM_ALIGN_RIGHT {
            it.textRect.x = it.textalignx - originalWidth as f32;
        } else if textalignment == ITEM_ALIGN_CENTER {
            it.textRect.x = it.textalignx - (originalWidth / 2) as f32;
        }

        let window = it.window.clone();
        let mut tx = it.textRect.x;
        let mut ty = it.textRect.y;
        ToWindowCoords(&mut tx, &mut ty, &window);
        let it = menus.item_mut(item);
        it.textRect.x = tx;
        it.textRect.y = ty;

        // string package: mark language
        if it.text.as_deref().is_some_and(|t| t.starts_with('@')) {
            it.asset = seLanguageModCount;
        }
    }
}

/// Raven `Controls_GetConfig` — refresh every `g_bindings` row's `bind1`/
/// `bind2` from the host's live key bindings.
/// Source: `oracle/codemp/ui/ui_shared.c:5332-5355`
pub fn Controls_GetConfig(menus: &mut MenuSystem, dc: &mut dyn DisplayContext) {
    for i in 0..menus.g_bindings.len() {
        let command = menus.g_bindings[i].command;
        let (bind1, bind2) = Controls_GetKeyAssignment(dc, command);
        menus.g_bindings[i].bind1 = bind1;
        menus.g_bindings[i].bind2 = bind2;
    }
}

/// Raven `Item_Bind_HandleKey` — key-bind capture state machine: click/enter
/// arms capture, then the next key (or backspace/escape) resolves it.
/// Source: `oracle/codemp/ui/ui_shared.c:5552-5666`
pub fn Item_Bind_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
    down: bool,
) -> bool {
    let rect = menus.item(item).window.rect;

    if key == A_MOUSE1
        && Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
        && !menus.g_waitingForKey
    {
        if down {
            menus.g_waitingForKey = true;
            menus.g_bindItem = Some(item);
        }
        return true;
    } else if key == A_ENTER && !menus.g_waitingForKey {
        if down {
            menus.g_waitingForKey = true;
            menus.g_bindItem = Some(item);
        }
        return true;
    } else {
        if !menus.g_waitingForKey || menus.g_bindItem.is_none() {
            return false;
        }

        if key & K_CHAR_FLAG != 0 {
            return true;
        }

        match key {
            A_ESCAPE => {
                menus.g_waitingForKey = false;
                return true;
            }
            A_BACKSPACE => {
                let cvar = menus.item(item).cvar.clone().unwrap_or_default();
                let id = BindingIDFromName(menus, &cvar);
                if id != -1 {
                    let idx = id as usize;
                    if menus.g_bindings[idx].bind1 != -1 {
                        let bind1 = menus.g_bindings[idx].bind1;
                        dc.setBinding(bind1, "");
                    }
                    if menus.g_bindings[idx].bind2 != -1 {
                        let bind2 = menus.g_bindings[idx].bind2;
                        dc.setBinding(bind2, "");
                    }
                    menus.g_bindings[idx].bind1 = -1;
                    menus.g_bindings[idx].bind2 = -1;
                }
                Controls_SetConfig(menus, dc, true);
                menus.g_waitingForKey = false;
                menus.g_bindItem = None;
                return true;
            }
            96 => {
                // '`'
                return true;
            }
            _ => {}
        }
    }

    if key != -1 {
        for i in 0..menus.g_bindings.len() {
            if menus.g_bindings[i].bind2 == key {
                menus.g_bindings[i].bind2 = -1;
            }
            if menus.g_bindings[i].bind1 == key {
                menus.g_bindings[i].bind1 = menus.g_bindings[i].bind2;
                menus.g_bindings[i].bind2 = -1;
            }
        }
    }

    let cvar = menus.item(item).cvar.clone().unwrap_or_default();
    let id = BindingIDFromName(menus, &cvar);

    if id != -1 {
        let idx = id as usize;
        if key == -1 {
            if menus.g_bindings[idx].bind1 != -1 {
                let bind1 = menus.g_bindings[idx].bind1;
                dc.setBinding(bind1, "");
                menus.g_bindings[idx].bind1 = -1;
            }
            if menus.g_bindings[idx].bind2 != -1 {
                let bind2 = menus.g_bindings[idx].bind2;
                dc.setBinding(bind2, "");
                menus.g_bindings[idx].bind2 = -1;
            }
        } else if menus.g_bindings[idx].bind1 == -1 {
            menus.g_bindings[idx].bind1 = key;
        } else if menus.g_bindings[idx].bind1 != key && menus.g_bindings[idx].bind2 == -1 {
            menus.g_bindings[idx].bind2 = key;
        } else {
            let bind1 = menus.g_bindings[idx].bind1;
            let bind2 = menus.g_bindings[idx].bind2;
            dc.setBinding(bind1, "");
            dc.setBinding(bind2, "");
            menus.g_bindings[idx].bind1 = key;
            menus.g_bindings[idx].bind2 = -1;
        }
    }

    Controls_SetConfig(menus, dc, true);
    menus.g_waitingForKey = false;

    true
}

/// Raven `Menu_Init` — zero a menu, then apply its non-zero defaults.
/// Source: `oracle/codemp/ui/ui_shared.c:7015-7022`
pub fn Menu_Init(menu: &mut MenuDef, ds: &DisplayState) {
    *menu = MenuDef::default();
    menu.cursorItem = -1;
    menu.fadeAmount = ds.Assets.fadeAmount;
    menu.fadeClamp = ds.Assets.fadeClamp;
    menu.fadeCycle = ds.Assets.fadeCycle;
    Window_Init(&mut menu.window);
}

/// Raven `Menu_SetFeederSelection` — move a feeder-backed list's cursor to
/// `index` and notify the host.
///
/// PORT-NOTE: `menu == NULL` resolves via `name` (`Menus_FindByName`) or the
/// focused menu (`Menu_GetFocused`), matching Raven's fallback chain exactly;
/// `menu: Option<MenuId>`/`name: Option<&str>` carry the two nullable `char
/// *`/`menuDef_t *` params.
/// Source: `oracle/codemp/ui/ui_shared.c:7060-7084`
pub fn Menu_SetFeederSelection(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    feeder: c_int,
    index: c_int,
    name: Option<&str>,
) {
    let menu = match menu {
        Some(m) => Some(m),
        None => match name {
            Some(name) => Menus_FindByName(menus, name),
            None => Menu_GetFocused(menus),
        },
    };
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let itemIds = menus.menu(menu).items.clone();
    for id in itemIds {
        if menus.item(id).special == feeder as f32 {
            if index == 0 {
                // PORT-NOTE (§19 UB pick): Raven casts `typeData` to `listBoxDef_t *`
                // unconditionally; a non-listbox payload is a silent no-op here.
                if let Some(listPtr) = menus.item_mut(id).typeData.listBox_mut() {
                    listPtr.cursorPos = 0;
                    listPtr.startPos = 0;
                }
            }
            menus.item_mut(id).cursorPos = index;
            let cursorPos = menus.item(id).cursorPos;
            let special = menus.item(id).special;
            dc.feederSelection(special, cursorPos, None);
            return;
        }
    }
}

/// Raven `Item_Init` — zero an item, then apply its non-zero defaults.
/// Source: `oracle/codemp/ui/ui_shared.c:7120-7124`
pub fn Item_Init(item: &mut ItemDef) {
    *item = ItemDef::default();
    item.textscale = 0.55;
    Window_Init(&mut item.window);
}

/// Raven `Item_ValidateTypeData` — lazily attach an item's per-type
/// `typeData` payload, keyed by `item->type`.
///
/// PORT-NOTE: Raven `memset`s the `UI_Alloc`'d listbox/editfield/model
/// payloads but leaves multi/textscroll uninitialized heap; every
/// `ItemPayload` variant derives `Default`, so this port zeroes all five
/// uniformly (porting-rules §C9 — owned defaults, not a pool carve).
/// Source: `oracle/codemp/ui/ui_shared.c:7279-7316`
pub fn Item_ValidateTypeData(item: &mut ItemDef) {
    if !item.typeData.is_none() {
        return;
    }

    if item.r#type == ITEM_TYPE_LISTBOX {
        item.typeData = ItemPayload::ListBox(ListBoxDef::default());
    } else if item.r#type == ITEM_TYPE_EDITFIELD
        || item.r#type == ITEM_TYPE_NUMERICFIELD
        || item.r#type == ITEM_TYPE_YESNO
        || item.r#type == ITEM_TYPE_BIND
        || item.r#type == ITEM_TYPE_SLIDER
        || item.r#type == ITEM_TYPE_TEXT
    {
        let mut editDef = EditFieldDef::default();
        if item.r#type == ITEM_TYPE_EDITFIELD && editDef.maxPaintChars == 0 {
            editDef.maxPaintChars = MAX_EDITFIELD as c_int;
        }
        item.typeData = ItemPayload::EditField(editDef);
    } else if item.r#type == ITEM_TYPE_MULTI {
        item.typeData = ItemPayload::Multi(MultiDef::default());
    } else if item.r#type == ITEM_TYPE_MODEL {
        item.typeData = ItemPayload::Model(ModelDef::default());
    } else if item.r#type == ITEM_TYPE_TEXTSCROLL {
        item.typeData = ItemPayload::TextScroll(TextScrollDef::default());
    }
}

// `KeywordHash_Add` — ui_shared.c:7347-7358.
// `KeywordHash_Find` — ui_shared.c:7360-7371.
//
// DEFERRED: KeywordHash_Add, KeywordHash_Find — the `keywordHash_t` node type
// these operate on (a hand-rolled hash-bucket linked list over
// `itemParseKeywords[]`/`menuParseKeywords[]`, Raven's per-keyword C
// fn-pointer tables) is not ported; the translation dictionary routes closed
// C fn-pointer tables to `match` dispatch, which is the expected owned shape
// for the item/menu keyword parser once it lands — at that point this
// hash-bucket infrastructure has no owned-shape target to transcribe against
// (inventing an ad-hoc `keywordHash_t` port here would front-run that design
// point). Flagged as an escalation for the wave-planning follow-up.
// Source: `oracle/codemp/ui/ui_shared.c:7326-7371`

/// Raven `ItemParse_flag` — parse an item's `WINDOW_*` style flag keyword.
///
/// PORT-NOTE (§19 UB pick): Raven's loop bound is `while (styles[i])` — the
/// `styles[]` table (6 entries) is a different, longer array than `itemFlags[]`
/// (1 real entry + NULL sentinel) this loop actually reads, so past `i == 1`
/// the C loop reads out-of-bounds `itemFlagsDef_t` memory. The evident intent
/// (matching the sibling `alignment[]`/`types[]` keyword loops elsewhere in
/// this file) is `itemFlags[i].string`; this port iterates `itemFlags`'s own
/// table instead of reproducing the OOB read. `Com_Printf` is unreachable from
/// this host-agnostic crate (see `String_Report`) — routed through `dc.Print`.
/// Consequence: Raven's unmatched-token warning sat behind that OOB walk and
/// effectively never printed; this port prints it on every unmatched token.
/// Source: `oracle/codemp/ui/ui_shared.c:7975-8002`
pub fn ItemParse_flag(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);

    let mut matched = false;
    for &(flagName, flagValue) in ITEM_FLAGS {
        if stricmp_eq(&name, flagName) {
            menus.item_mut(item).window.flags |= flagValue;
            matched = true;
            break;
        }
    }

    if !matched {
        dc.Print(&format!("^3Unknown item style value '{}'", name));
    }

    true
}

/// Raven `Display_CaptureItem` — the defined menu whose window rect contains
/// `(x, y)`, if any.
/// Source: `oracle/codemp/ui/ui_shared.c:9858-9869`
pub fn Display_CaptureItem(menus: &MenuSystem, x: c_int, y: c_int) -> Option<MenuId> {
    for i in 0..menus.menus.len() {
        if Rect_ContainsPoint(Some(&menus.menus[i].window.rect), x as f32, y as f32) {
            return Some(MenuId::new(i));
        }
    }
    None
}

/// Raven `Display_CursorType` — the cursor shape for `(x, y)`: a resize
/// cursor over any menu's corner grip, else the default arrow.
/// Source: `oracle/codemp/ui/ui_shared.c:9903-9915`
pub fn Display_CursorType(menus: &MenuSystem, x: c_int, y: c_int) -> c_int {
    for m in &menus.menus {
        let r2 = RectDef {
            x: m.window.rect.x - 3.0,
            y: m.window.rect.y - 3.0,
            w: 7.0,
            h: 7.0,
        };
        if Rect_ContainsPoint(Some(&r2), x as f32, y as f32) {
            return CURSOR_SIZER;
        }
    }
    CURSOR_ARROW
}

/// Raven `Item_CacheContents` — pre-roll an item's cinematic (if it has one).
/// Source: `oracle/codemp/ui/ui_shared.c:9938-9943`
pub fn Item_CacheContents(menus: &MenuSystem, dc: &mut dyn DisplayContext, item: Option<ItemId>) {
    let window = item.map(|id| &menus.item(id).window);
    Window_CacheContents(window, dc);
}

/// Raven `Menu_OverActiveItem` — is `(x, y)` over any of `menu`'s
/// visible/forced, non-decoration items (respecting `ITEM_TYPE_TEXT`'s own
/// rect re-check)?
/// Source: `oracle/codemp/ui/ui_shared.c:9968-10005`
pub fn Menu_OverActiveItem(menus: &MenuSystem, menu: Option<MenuId>, x: f32, y: f32) -> bool {
    let menu = match menu {
        Some(m) => m,
        None => return false,
    };
    let m = menus.menu(menu);
    if m.window.flags & (WINDOW_VISIBLE | WINDOW_FORCED) == 0 {
        return false;
    }

    if Rect_ContainsPoint(Some(&m.window.rect), x, y) {
        for &id in &m.items {
            let it = menus.item(id);
            // turn off focus each item
            if it.window.flags & (WINDOW_VISIBLE | WINDOW_FORCED) == 0 {
                continue;
            }
            if it.window.flags & WINDOW_DECORATION != 0 {
                continue;
            }

            if Rect_ContainsPoint(Some(&it.window.rect), x, y) {
                if it.r#type == ITEM_TYPE_TEXT && it.text.is_some() {
                    if Rect_ContainsPoint(Some(&it.window.rect), x, y) {
                        return true;
                    } else {
                        continue;
                    }
                } else {
                    return true;
                }
            }
        }
    }

    false
}

// ---------------------------------------------------------------------
// wave 2
// ---------------------------------------------------------------------

/// Raven `PC_Float_Parse` — parse a (possibly negative-signed) numeric
/// source token as a float.
/// Source: `oracle/codemp/ui/ui_shared.c:465-485`
pub fn PC_Float_Parse(dc: &mut dyn DisplayContext, handle: c_int, f: &mut f32) -> bool {
    let mut token = zero_pc_token();
    let mut negative = false;

    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    if pc_token_str(&token).starts_with('-') {
        if !dc.PC_ReadToken(handle, &mut token) {
            return false;
        }
        negative = true;
    }
    if token.type_ != TT_NUMBER {
        PC_SourceError(
            dc,
            handle,
            &format!("expected float but found {}\n", pc_token_str(&token)),
        );
        return false;
    }
    *f = if negative {
        -token.floatvalue
    } else {
        token.floatvalue
    };
    true
}

/// Raven `PC_Int_Parse` — parse a (possibly negative-signed) numeric source
/// token as an int.
/// Source: `oracle/codemp/ui/ui_shared.c:545-564`
pub fn PC_Int_Parse(dc: &mut dyn DisplayContext, handle: c_int, i: &mut c_int) -> bool {
    let mut token = zero_pc_token();
    let mut negative = false;

    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    if pc_token_str(&token).starts_with('-') {
        if !dc.PC_ReadToken(handle, &mut token) {
            return false;
        }
        negative = true;
    }
    if token.type_ != TT_NUMBER {
        PC_SourceError(
            dc,
            handle,
            &format!("expected integer but found {}\n", pc_token_str(&token)),
        );
        return false;
    }
    *i = token.intvalue;
    if negative {
        *i = -*i;
    }
    true
}

/// Raven `String_Parse` — parse one token off `p`, interning it into `out`
/// via [`String_Alloc`].
/// Source: `oracle/codemp/ui/ui_shared.c:607-616`
pub fn String_Parse(p: &mut &str, out: &mut String) -> bool {
    let (token, rest) = COM_Parse(p, false);
    *p = rest;
    if !token.is_empty() {
        if let Some(alloc) = String_Alloc(Some(&token)) {
            *out = alloc;
            return true;
        }
    }
    false
}

/// Raven `PC_String_Parse` — read one source token into `out` via
/// [`String_Alloc`], special-casing the closing `"}"` to avoid interning a
/// fresh copy of it every call.
///
/// PORT-NOTE: Raven's `static char *squiggy = "}"` is a string literal, not
/// persisted state (see `MenuScratch`'s PORT-NOTE) — reproduced inline.
/// Source: `oracle/codemp/ui/ui_shared.c:623-643`
pub fn PC_String_Parse(dc: &mut dyn DisplayContext, handle: c_int, out: &mut String) -> bool {
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }

    let name = pc_token_str(&token);
    if stricmp_eq(&name, "}") {
        *out = "}".to_string();
    } else {
        *out = String_Alloc(Some(&name)).unwrap_or_default();
    }
    true
}

/// Raven `PC_Script_Parse` — read a `{ ... }` token run into `out` as one
/// re-quoted, space-separated script string.
///
/// PORT-NOTE: Raven accumulates into a fixed `char script[2048]` (silently
/// truncated by `Q_strcat`); the owned `String` grows unbounded instead,
/// matching this file's other reshaped fixed-buffer accumulators (e.g.
/// `BindingFromName`'s `g_nameBind1`).
/// Source: `oracle/codemp/ui/ui_shared.c:650-681`
pub fn PC_Script_Parse(dc: &mut dyn DisplayContext, handle: c_int, out: &mut String) -> bool {
    let mut script = String::new();
    let mut token = zero_pc_token();

    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    if !stricmp_eq(&pc_token_str(&token), "{") {
        return false;
    }

    loop {
        if !dc.PC_ReadToken(handle, &mut token) {
            return false;
        }
        let tokenStr = pc_token_str(&token);

        if stricmp_eq(&tokenStr, "}") {
            *out = String_Alloc(Some(&script)).unwrap_or_default();
            return true;
        }

        // §19: Raven tests `token.string[1] != '\0'` on a reused token buffer, so
        // an empty token still sees the previous token's stale byte and quotes.
        if tokenStr.len() != 1 {
            script.push_str(&format!("\"{}\"", tokenStr));
        } else {
            script.push_str(&tokenStr);
        }
        script.push(' ');
    }
}

/// Raven `Menu_ShowGroup` — set/clear `WINDOW_VISIBLE` (and, when hiding,
/// `WINDOW_HASFOCUS`) on every item in `menu` matching `groupName`.
/// Source: `oracle/codemp/ui/ui_shared.c:1444-1465`
pub fn Menu_ShowGroup(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    groupName: &str,
    showFlag: bool,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, groupName);
    for j in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, j, groupName) {
            let it = menus.item_mut(id);
            if showFlag {
                it.window.flags |= WINDOW_VISIBLE;
            } else {
                it.window.flags &= !(WINDOW_VISIBLE | WINDOW_HASFOCUS);
            }
        }
    }
}

/// Raven `Menu_ShowItemByName` — set/clear `WINDOW_VISIBLE` on every item in
/// `menu` matching `p`, stopping the item's cinematic (if any) when hiding.
/// Source: `oracle/codemp/ui/ui_shared.c:1467-1486`
pub fn Menu_ShowItemByName(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    p: &str,
    bShow: bool,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, p);
    for i in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, i, p) {
            let it = menus.item_mut(id);
            if bShow {
                it.window.flags |= WINDOW_VISIBLE;
            } else {
                it.window.flags &= !WINDOW_VISIBLE;
                // stop cinematics playing in the window
                if it.window.cinematic >= 0 {
                    dc.stopCinematic(it.window.cinematic);
                    it.window.cinematic = -1;
                }
            }
        }
    }
}

/// Raven `Menu_FadeItemByName` — start a fade-in/fade-out on every item in
/// `menu` matching `p`.
/// Source: `oracle/codemp/ui/ui_shared.c:1488-1504`
pub fn Menu_FadeItemByName(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    p: &str,
    fadeOut: bool,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, p);
    for i in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, i, p) {
            let it = menus.item_mut(id);
            if fadeOut {
                it.window.flags |= WINDOW_FADINGOUT | WINDOW_VISIBLE;
                it.window.flags &= !WINDOW_FADINGIN;
            } else {
                it.window.flags |= WINDOW_VISIBLE | WINDOW_FADINGIN;
                it.window.flags &= !WINDOW_FADINGOUT;
            }
        }
    }
}

/// Raven `Menu_Transition3ItemByName` — start a two-target ghoul2
/// bounds/FOV transition on every `ITEM_TYPE_MODEL` item in `menu` matching
/// `p`.
/// Source: `oracle/codemp/ui/ui_shared.c:1696-1744`
#[allow(clippy::too_many_arguments)]
pub fn Menu_Transition3ItemByName(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    p: &str,
    minx: f32,
    miny: f32,
    minz: f32,
    maxx: f32,
    maxy: f32,
    maxz: f32,
    fovtx: f32,
    fovty: f32,
    time: c_int,
    amt: f32,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, p);
    for i in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, i, p) {
            let it = menus.item_mut(id);
            if it.r#type == ITEM_TYPE_MODEL {
                it.window.flags |= WINDOW_INTRANSITIONMODEL | WINDOW_VISIBLE;
                it.window.offsetTime = time;

                if let Some(modelptr) = it.typeData.model_mut() {
                    modelptr.fov_x2 = fovtx;
                    modelptr.fov_y2 = fovty;
                    modelptr.g2maxs2 = [maxx, maxy, maxz];
                    modelptr.g2mins2 = [minx, miny, minz];

                    // Raven's `abs()` is the int form: each float delta truncates
                    // toward zero before the divide.
                    modelptr.g2maxsEffect[0] =
                        ((modelptr.g2maxs2[0] - modelptr.g2maxs[0]) as c_int).abs() as f32 / amt;
                    modelptr.g2maxsEffect[1] =
                        ((modelptr.g2maxs2[1] - modelptr.g2maxs[1]) as c_int).abs() as f32 / amt;
                    modelptr.g2maxsEffect[2] =
                        ((modelptr.g2maxs2[2] - modelptr.g2maxs[2]) as c_int).abs() as f32 / amt;

                    modelptr.g2minsEffect[0] =
                        ((modelptr.g2mins2[0] - modelptr.g2mins[0]) as c_int).abs() as f32 / amt;
                    modelptr.g2minsEffect[1] =
                        ((modelptr.g2mins2[1] - modelptr.g2mins[1]) as c_int).abs() as f32 / amt;
                    modelptr.g2minsEffect[2] =
                        ((modelptr.g2mins2[2] - modelptr.g2mins[2]) as c_int).abs() as f32 / amt;

                    modelptr.fov_Effectx =
                        ((modelptr.fov_x2 - modelptr.fov_x) as c_int).abs() as f32 / amt;
                    modelptr.fov_Effecty =
                        ((modelptr.fov_y2 - modelptr.fov_y) as c_int).abs() as f32 / amt;
                }
            }
        }
    }
}

/// Raven `Menu_ItemDisable` — set `disabled` on every item in `menu`
/// matching `name`, clearing its mouseover flag too.
/// Source: `oracle/codemp/ui/ui_shared.c:1845-1862`
pub fn Menu_ItemDisable(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    name: &str,
    disableFlag: c_int,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, name);
    // Loop through all items that have this name
    for j in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, j, name) {
            let it = menus.item_mut(id);
            it.disabled = disableFlag != 0;
            // Just in case it had focus
            it.window.flags &= !WINDOW_MOUSEOVER;
        }
    }
}

/// Raven `Menu_SetItemBackground` — set the background shader on every item
/// in `menu` matching `itemName`.
/// Source: `oracle/codemp/ui/ui_shared.c:2231-2251`
pub fn Menu_SetItemBackground(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    itemName: &str,
    background: &str,
) {
    // No menu???
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let count = Menu_ItemsMatchingGroup(menus, dc, menu, itemName);
    for j in 0..count {
        if let Some(id) = Menu_GetMatchingItemByNumber(menus, menu, j, itemName) {
            let shader = dc.registerShaderNoMip(background);
            menus.item_mut(id).window.background = shader;
        }
    }
}

/// Raven `Item_TextScroll_ThumbDrawPosition` — the thumb's draw-time y,
/// following the cursor while the box has mouse capture.
/// Source: `oracle/codemp/ui/ui_shared.c:2519-2537`
pub fn Item_TextScroll_ThumbDrawPosition(
    menus: &MenuSystem,
    ds: &DisplayState,
    item: ItemId,
) -> c_int {
    if menus.itemCapture == Some(item) {
        let rect = menus.item(item).window.rect;
        // Raven's `min`/`max` are `int`: the bounds truncate before the compare.
        let min = (rect.y + SCROLLBAR_SIZE + 1.0) as c_int;
        let max = (rect.y + rect.h - 2.0 * SCROLLBAR_SIZE - 1.0) as c_int;

        if ds.cursory as f32 >= min as f32 + SCROLLBAR_SIZE / 2.0
            && ds.cursory as f32 <= max as f32 + SCROLLBAR_SIZE / 2.0
        {
            return ds.cursory - (SCROLLBAR_SIZE / 2.0) as c_int;
        }

        return Item_TextScroll_ThumbPosition(menus, item);
    }

    Item_TextScroll_ThumbPosition(menus, item)
}

/// Raven `Item_TextScroll_OverLB` — which scrollbar hot-zone (if any)
/// `(x, y)` is over.
///
/// PORT-NOTE: Raven's `scrollPtr`/`count` locals are read (`iLineCount`) but
/// never used again in the body — a dead read, dropped here rather than
/// transcribed as an inert `typeData.textScroll()` fetch.
/// Source: `oracle/codemp/ui/ui_shared.c:2539-2585`
pub fn Item_TextScroll_OverLB(menus: &MenuSystem, item: ItemId, x: f32, y: f32) -> c_int {
    let rect = menus.item(item).window.rect;

    let mut r = RectDef {
        x: rect.x + rect.w - SCROLLBAR_SIZE,
        y: rect.y,
        w: SCROLLBAR_SIZE,
        h: SCROLLBAR_SIZE,
    };
    if Rect_ContainsPoint(Some(&r), x, y) {
        return WINDOW_LB_LEFTARROW;
    }

    r.y = rect.y + rect.h - SCROLLBAR_SIZE;
    if Rect_ContainsPoint(Some(&r), x, y) {
        return WINDOW_LB_RIGHTARROW;
    }

    let thumbstart = Item_TextScroll_ThumbPosition(menus, item);
    r.y = thumbstart as f32;
    if Rect_ContainsPoint(Some(&r), x, y) {
        return WINDOW_LB_THUMB;
    }

    r.y = rect.y + SCROLLBAR_SIZE;
    r.h = thumbstart as f32 - r.y;
    if Rect_ContainsPoint(Some(&r), x, y) {
        return WINDOW_LB_PGUP;
    }

    r.y = thumbstart as f32 + SCROLLBAR_SIZE;
    r.h = rect.y + rect.h - SCROLLBAR_SIZE;
    if Rect_ContainsPoint(Some(&r), x, y) {
        return WINDOW_LB_PGDN;
    }

    0
}

/// Raven `Item_ListBox_ThumbDrawPosition` — the thumb's draw-time x/y,
/// following the cursor while the box has mouse capture (horizontal or
/// vertical, by `WINDOW_HORIZONTAL`).
/// Source: `oracle/codemp/ui/ui_shared.c:2751-2788`
pub fn Item_ListBox_ThumbDrawPosition(
    menus: &MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> c_int {
    if menus.itemCapture == Some(item) {
        let it = menus.item(item);
        let rect = it.window.rect;
        // Raven's `min`/`max` are `int`: the bounds truncate before the compare.
        if it.window.flags & WINDOW_HORIZONTAL != 0 {
            let min = (rect.x + SCROLLBAR_SIZE + 1.0) as c_int;
            let max = (rect.x + rect.w - 2.0 * SCROLLBAR_SIZE - 1.0) as c_int;
            if ds.cursorx as f32 >= min as f32 + SCROLLBAR_SIZE / 2.0
                && ds.cursorx as f32 <= max as f32 + SCROLLBAR_SIZE / 2.0
            {
                return ds.cursorx - (SCROLLBAR_SIZE / 2.0) as c_int;
            } else {
                return Item_ListBox_ThumbPosition(menus, dc, item);
            }
        } else {
            let min = (rect.y + SCROLLBAR_SIZE + 1.0) as c_int;
            let max = (rect.y + rect.h - 2.0 * SCROLLBAR_SIZE - 1.0) as c_int;
            if ds.cursory as f32 >= min as f32 + SCROLLBAR_SIZE / 2.0
                && ds.cursory as f32 <= max as f32 + SCROLLBAR_SIZE / 2.0
            {
                return ds.cursory - (SCROLLBAR_SIZE / 2.0) as c_int;
            } else {
                return Item_ListBox_ThumbPosition(menus, dc, item);
            }
        }
    } else {
        Item_ListBox_ThumbPosition(menus, dc, item)
    }
}

/// Raven `Item_ListBox_OverLB` — which scrollbar hot-zone (if any)
/// `(x, y)` is over (horizontal, or vertical with the multi-column/image
/// layout carrying only page-up/page-down/thumb).
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) falls back to `elementWidth = 0.0`/
/// `elementStyle = 0` instead of Raven's unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:2837-2953`
pub fn Item_ListBox_OverLB(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    x: f32,
    y: f32,
) -> c_int {
    let it = menus.item(item);
    let _count = dc.feederCount(it.special);
    let listPtr = it.typeData.listBox();
    let rect = it.window.rect;

    if it.window.flags & WINDOW_HORIZONTAL != 0 {
        // check if on left arrow
        let mut r = RectDef {
            x: rect.x,
            y: rect.y + rect.h - SCROLLBAR_SIZE,
            w: SCROLLBAR_SIZE,
            h: SCROLLBAR_SIZE,
        };
        if Rect_ContainsPoint(Some(&r), x, y) {
            return WINDOW_LB_LEFTARROW;
        }

        // check if on right arrow
        r.x = rect.x + rect.w - SCROLLBAR_SIZE;
        if Rect_ContainsPoint(Some(&r), x, y) {
            return WINDOW_LB_RIGHTARROW;
        }

        // check if on thumb
        let thumbstart = Item_ListBox_ThumbPosition(menus, dc, item);
        r.x = thumbstart as f32;
        if Rect_ContainsPoint(Some(&r), x, y) {
            return WINDOW_LB_THUMB;
        }

        r.x = rect.x + SCROLLBAR_SIZE;
        r.w = thumbstart as f32 - r.x;
        if Rect_ContainsPoint(Some(&r), x, y) {
            return WINDOW_LB_PGUP;
        }

        r.x = thumbstart as f32 + SCROLLBAR_SIZE;
        r.w = rect.x + rect.w - SCROLLBAR_SIZE;
        if Rect_ContainsPoint(Some(&r), x, y) {
            return WINDOW_LB_PGDN;
        }
    }
    // Vertical Scroll
    else {
        let elementWidth = listPtr.map(|l| l.elementWidth).unwrap_or(0.0);
        let elementStyle = listPtr.map(|l| l.elementStyle).unwrap_or(0);

        // Multiple rows and columns (since it's more than twice as wide as an element)
        if rect.w > (elementWidth * 2.0) && elementStyle == LISTBOX_IMAGE {
            let mut r = RectDef {
                x: rect.x + rect.w - SCROLLBAR_SIZE,
                y: rect.y,
                w: SCROLLBAR_SIZE,
                h: SCROLLBAR_SIZE,
            };
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_PGUP;
            }

            r.y = rect.y + rect.h - SCROLLBAR_SIZE;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_PGDN;
            }

            let thumbstart = Item_ListBox_ThumbPosition(menus, dc, item);
            r.y = thumbstart as f32;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_THUMB;
            }
        } else {
            let mut r = RectDef {
                x: rect.x + rect.w - SCROLLBAR_SIZE,
                y: rect.y,
                w: SCROLLBAR_SIZE,
                h: SCROLLBAR_SIZE,
            };
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_LEFTARROW;
            }

            r.y = rect.y + rect.h - SCROLLBAR_SIZE;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_RIGHTARROW;
            }

            let thumbstart = Item_ListBox_ThumbPosition(menus, dc, item);
            r.y = thumbstart as f32;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_THUMB;
            }

            r.y = rect.y + SCROLLBAR_SIZE;
            r.h = thumbstart as f32 - r.y;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_PGUP;
            }

            r.y = thumbstart as f32 + SCROLLBAR_SIZE;
            r.h = rect.y + rect.h - SCROLLBAR_SIZE;
            if Rect_ContainsPoint(Some(&r), x, y) {
                return WINDOW_LB_PGDN;
            }
        }
    }
    0
}

/// Raven `Scroll_TextScroll_AutoFunc` — the text-scroll box's auto-scroll
/// capture-func tick: repeat the captured scroll key on the throttle, easing
/// the repeat interval down to the floor.
///
/// PORT-NOTE (§19 UB pick): Raven derefs `si->item` unconditionally (`si` is
/// always `&scrollInfo` while a scroll capture is active); the
/// otherwise-unreachable "no captured item" case here is a no-op instead of a
/// null deref (see `Scroll_Slider_ThumbFunc`).
/// Source: `oracle/codemp/ui/ui_shared.c:3831-3852`
pub fn Scroll_TextScroll_AutoFunc(menus: &mut MenuSystem, ds: &DisplayState) {
    let item = match menus.scrollInfo.item {
        Some(id) => id,
        None => return,
    };

    if ds.realTime > menus.scrollInfo.nextScrollTime {
        // need to scroll which is done by simulating a click to the item
        // this is done a bit sideways as the autoscroll "knows" that the item is a listbox
        // so it calls it directly
        let scrollKey = menus.scrollInfo.scrollKey;
        Item_TextScroll_HandleKey(menus, ds, item, scrollKey, true, false);
        menus.scrollInfo.nextScrollTime = ds.realTime + menus.scrollInfo.adjustValue;
    }

    if ds.realTime > menus.scrollInfo.nextAdjustTime {
        menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
        if menus.scrollInfo.adjustValue > SCROLL_TIME_FLOOR {
            menus.scrollInfo.adjustValue -= SCROLL_TIME_ADJUSTOFFSET;
        }
    }
}

/// Raven `Scroll_TextScroll_ThumbFunc` — the text-scroll box's thumb-drag
/// capture-func tick: track the cursor, then run the same auto-scroll
/// throttle as [`Scroll_TextScroll_AutoFunc`].
///
/// PORT-NOTE (§19 UB pick): see `Scroll_TextScroll_AutoFunc`.
/// Source: `oracle/codemp/ui/ui_shared.c:3854-3902`
pub fn Scroll_TextScroll_ThumbFunc(menus: &mut MenuSystem, ds: &DisplayState) {
    let item = match menus.scrollInfo.item {
        Some(id) => id,
        None => return,
    };

    if ds.cursory != menus.scrollInfo.yStart as c_int {
        let rect = menus.item(item).window.rect;
        let r = RectDef {
            x: rect.x + rect.w - SCROLLBAR_SIZE - 1.0,
            y: rect.y + SCROLLBAR_SIZE + 1.0,
            h: rect.h - (SCROLLBAR_SIZE * 2.0) - 2.0,
            w: SCROLLBAR_SIZE,
        };
        let max = Item_TextScroll_MaxScroll(menus, item);

        let mut pos = ((ds.cursory as f32 - r.y - SCROLLBAR_SIZE / 2.0) * max as f32
            / (r.h - SCROLLBAR_SIZE)) as c_int;
        if pos < 0 {
            pos = 0;
        } else if pos > max {
            pos = max;
        }

        if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
            scrollPtr.startPos = pos;
        }
        menus.scrollInfo.yStart = ds.cursory as f32;
    }

    if ds.realTime > menus.scrollInfo.nextScrollTime {
        // need to scroll which is done by simulating a click to the item
        // this is done a bit sideways as the autoscroll "knows" that the item is a listbox
        // so it calls it directly
        let scrollKey = menus.scrollInfo.scrollKey;
        Item_TextScroll_HandleKey(menus, ds, item, scrollKey, true, false);
        menus.scrollInfo.nextScrollTime = ds.realTime + menus.scrollInfo.adjustValue;
    }

    if ds.realTime > menus.scrollInfo.nextAdjustTime {
        menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
        if menus.scrollInfo.adjustValue > SCROLL_TIME_FLOOR {
            menus.scrollInfo.adjustValue -= SCROLL_TIME_ADJUSTOFFSET;
        }
    }
}

/// Raven `Display_CloseCinematics` — stop every defined menu's cinematics.
/// Source: `oracle/codemp/ui/ui_shared.c:4409-4414`
pub fn Display_CloseCinematics(menus: &mut MenuSystem, dc: &mut dyn DisplayContext) {
    let count = menus.menus.len();
    for i in 0..count {
        Menu_CloseCinematics(menus, dc, Some(MenuId::new(i)));
    }
}

/// Raven `ItemParse_asset_model_go` — load (or rebuild) an item's asset,
/// either a ghoul2 `.glm` model (tracked for shutdown cleanup) or a plain
/// `.md3` model.
///
/// PORT-NOTE (dead surface): the `#ifndef CGAME` guard picks the `ui` arm
/// unconditionally, matching this file's existing convention (see
/// `Window_Paint`'s PORT-NOTE) — this crate has only the `ui` linkage so far.
/// Source: `oracle/codemp/ui/ui_shared.c:7567-7657`
pub fn ItemParse_asset_model_go(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    name: &str,
    runTimeLength: &mut c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    *runTimeLength = 0;

    // §19: Raven reads `&name[strlen(name) - 4]`, under-running the buffer for a
    // name shorter than the suffix; the length guard is the defined choice.
    let nameBytes = name.as_bytes();
    if nameBytes.len() >= 4 && nameBytes[nameBytes.len() - 4..].eq_ignore_ascii_case(b".glm") {
        // it's a ghoul2 model then
        let ghoul2 = menus.item(item).ghoul2;
        if !ghoul2.is_null() {
            UI_ClearG2Pointer(menus, ghoul2); // remove from tracking list
            let mut ptr = menus.item(item).ghoul2;
            dc.G2API_CleanGhoul2Models(&mut ptr as *mut *mut c_void); // remove ghoul info
            menus.item_mut(item).ghoul2 = ptr;
            menus.item_mut(item).flags &= !ITF_G2VALID;
        }

        // §19: Raven derefs `modelPtr` unconditionally; a non-model `typeData`
        // reads as the zeroed field here instead.
        let g2skin = menus
            .item(item)
            .typeData
            .model()
            .map(|m| m.g2skin)
            .unwrap_or(0);
        let mut ghoul2Ptr = menus.item(item).ghoul2;
        let g2Model =
            dc.G2API_InitGhoul2Model(&mut ghoul2Ptr as *mut *mut c_void, name, 0, g2skin, 0, 0, 0);
        menus.item_mut(item).ghoul2 = ghoul2Ptr;

        if g2Model >= 0 {
            UI_InsertG2Pointer(menus, menus.item(item).ghoul2); // remember it so we can free it when the ui shuts down.
            menus.item_mut(item).flags |= ITF_G2VALID;

            // §19: same zeroed-field stand-in for Raven's bare `modelPtr` deref.
            let g2anim = menus
                .item(item)
                .typeData
                .model()
                .map(|m| m.g2anim)
                .unwrap_or(0);
            if g2anim != 0 {
                // does the menu request this model be playing an animation?
                let ghoul2 = menus.item(item).ghoul2;
                let GLAName = dc.G2API_GetGLAName(ghoul2, 0, MAX_QPATH as usize);

                if !GLAName.is_empty() {
                    if GLAName.rfind('/').is_some() {
                        // If this isn't true the gla path must be messed up somehow.
                        //
                        // DEFERRED: UI_ParseAnimationFile — ui reuses mp_bg's
                        // animation module instead of Raven's hand-maintained
                        // `bgAllAnims` copy (DEC-36 D5); `UI_ParseAnimationFile`
                        // itself already carries that deferral at U0
                        // (`crates/mp/ui/src/ui_main.rs:99-104`, same
                        // `uiHumanoidAnimations`/`bgAllAnims`/`uiNumAllAnims`
                        // fork), with no owned-shape reuse target designed yet —
                        // this branch's bone-anim playback off a parsed
                        // `animation.cfg` has nothing to transcribe against.
                        // Consequence: `*runTimeLength` stays 0 rather than the
                        // parsed animation's play length.
                        // Source: `oracle/codemp/ui/ui_shared.c:7602-7631`
                    }
                }
            }

            // §19: same zeroed-field stand-in for Raven's bare `modelPtr` deref.
            let g2skin = menus
                .item(item)
                .typeData
                .model()
                .map(|m| m.g2skin)
                .unwrap_or(0);
            if g2skin != 0 {
                let ghoul2 = menus.item(item).ghoul2;
                // this is going to set the surfs on/off matching the skin file
                dc.G2API_SetSkin(ghoul2, 0, g2skin, g2skin);
            }
        }
    } else if menus.item(item).asset == 0 {
        // guess it's just an md3
        let asset = dc.registerModel(name);
        menus.item_mut(item).asset = asset;
        menus.item_mut(item).flags &= !ITF_G2VALID;
    }

    true
}

/// Raven `ItemParse_model_g2skin` — parse an item's ghoul2 skin asset.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) drops the write instead of Raven's
/// unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:7811-7830`
pub fn ItemParse_model_g2skin(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }

    let name = pc_token_str(&token);
    if name.is_empty() {
        // it was parsed correctly so still return true.
        return true;
    }

    let skin = dc.R_RegisterSkin(&name);
    if let Some(modelPtr) = menus.item_mut(item).typeData.model_mut() {
        modelPtr.g2skin = skin;
    }

    true
}

/// Raven `ItemParse_model_g2anim` — parse an item's ghoul2 animation by name
/// off `mp_bg`'s reused `animTable` (DEC-36 D5).
///
/// PORT-NOTE: `animTable[n].id == n` for every entry (see the table's own
/// doc comment), so [`GetIDForString`]'s returned id is the same value
/// Raven's loop-index assignment (`modelPtr->g2anim = i`) would have written.
/// `Com_Printf` (the not-found warning) is unreachable from this
/// host-agnostic crate (see `String_Report`) — routed through `dc.Print`.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch drops the
/// write instead of Raven's unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:7833-7862`
pub fn ItemParse_model_g2anim(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }

    let name = pc_token_str(&token);
    if name.is_empty() {
        // it was parsed correctly so still return true.
        return true;
    }

    let id = GetIDForString(animTable.as_ptr() as *mut stringID_table_t, &name);
    if id != -1 {
        // found it
        if let Some(modelPtr) = menus.item_mut(item).typeData.model_mut() {
            modelPtr.g2anim = id;
        }
        return true;
    }

    dc.Print(&format!("Could not find '{}' in the anim table\n", name));
    true
}

/// Raven `ItemParse_model_g2skin_go` — set an item's ghoul2 skin by name (or
/// clear it for an empty/absent `skinName`).
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch drops the
/// clear-path write instead of Raven's unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:7865-7891`
pub fn ItemParse_model_g2skin_go(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    skinName: Option<&str>,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    if skinName.map(|s| s.is_empty()).unwrap_or(true) {
        // it was parsed correctly so still return true.
        if let Some(modelPtr) = menus.item_mut(item).typeData.model_mut() {
            modelPtr.g2skin = 0;
        }
        let ghoul2 = menus.item(item).ghoul2;
        dc.G2API_SetSkin(ghoul2, 0, 0, 0);
        return true;
    }
    let skinName = skinName.unwrap();

    // set skin
    let ghoul2 = menus.item(item).ghoul2;
    if !ghoul2.is_null() {
        let defSkin = dc.R_RegisterSkin(skinName);
        dc.G2API_SetSkin(ghoul2, 0, defSkin, defSkin);
    }

    true
}

/// Raven `ItemParse_model_g2anim_go` — set an item's ghoul2 animation by
/// name (looked up in `mp_bg`'s reused `animTable`, DEC-36 D5).
///
/// PORT-NOTE: unlike `ItemParse_model_g2anim`, Raven assigns
/// `animTable[i].id` here directly (not the loop index) — both are the same
/// value (see that fn's PORT-NOTE), so this is `GetIDForString`'s return
/// either way. `Com_Printf` is unreachable from this host-agnostic crate —
/// routed through `dc.Print` (adds a `dc` param beyond the packet's
/// classification, same as `Menu_ItemsMatchingGroup`).
/// Source: `oracle/codemp/ui/ui_shared.c:7894-7919`
pub fn ItemParse_model_g2anim_go(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    animName: Option<&str>,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let animName = match animName {
        Some(s) if !s.is_empty() => s,
        // it was parsed correctly so still return true.
        _ => return true,
    };

    let id = GetIDForString(animTable.as_ptr() as *mut stringID_table_t, animName);
    if id != -1 {
        // found it
        if let Some(modelPtr) = menus.item_mut(item).typeData.model_mut() {
            modelPtr.g2anim = id;
        }
        return true;
    }

    dc.Print(&format!(
        "Could not find '{}' in the anim table\n",
        animName
    ));
    true
}

/// Raven `ItemParse_notselectable` — mark a list box item as not selectable.
/// Source: `oracle/codemp/ui/ui_shared.c:8029-8037`
pub fn ItemParse_notselectable(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    let it = menus.item_mut(item);
    if it.r#type == ITEM_TYPE_LISTBOX {
        if let Some(listPtr) = it.typeData.listBox_mut() {
            listPtr.notselectable = true;
        }
    }
    true
}

/// Raven `ItemParse_scrollhidden` — mark a list box item's scrollbar hidden.
/// Source: `oracle/codemp/ui/ui_shared.c:8045-8056`
pub fn ItemParse_scrollhidden(menus: &mut MenuSystem, item: ItemId, _handle: c_int) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    let it = menus.item_mut(item);
    if it.r#type == ITEM_TYPE_LISTBOX {
        if let Some(listPtr) = it.typeData.listBox_mut() {
            listPtr.scrollhidden = true;
        }
    }
    true
}

// `Item_SetupKeywordHash` — ui_shared.c:8995-9002.
//
// DEFERRED: Item_SetupKeywordHash — builds `itemParseKeywordHash` by walking
// `itemParseKeywords[]` through `KeywordHash_Add`, both of which carry the
// same `keywordHash_t` deferral as `KeywordHash_Add`/`KeywordHash_Find` above
// (no owned-shape item/menu keyword-dispatch design landed yet); nothing to
// build the hash table against.
// Source: `oracle/codemp/ui/ui_shared.c:8995-9002`

// `Item_Parse` — ui_shared.c:9009-9040.
//
// DEFERRED: Item_Parse — dispatches item keywords through
// `KeywordHash_Find(itemParseKeywordHash, ...)`, the same deferred
// infrastructure as `Item_SetupKeywordHash` above.
// Source: `oracle/codemp/ui/ui_shared.c:9009-9040`

/// Raven `Item_TextScroll_BuildLines` — word-wrap a text-scroll item's
/// `text` into `typeData.pLines`, asian-aware byte-cursor line breaking.
///
/// PORT-NOTE: Raven walks `psText`/`sLineForDisplay` as raw byte cursors
/// (multi-byte glyphs advance more than one byte); this keeps that shape —
/// `Vec<u8>` buffers and byte offsets — rather than `&str`/`char`, so
/// `AnyLanguage_ReadCharFromString`'s advance count and the source-byte
/// line-break math translate directly. `pLines` (an owned `Vec<String>`, see
/// `TextScrollDef`'s PORT-NOTE) receives an empty `String` for Raven's "hole"
/// case (`scrollPtr->pLines[iLineCount]` left `NULL` while `iLineCount` still
/// increments, when a line collapses to pure leading whitespace), so the line
/// count — and with it the scroll extent — stays identical.
///
/// §19 UB pick: `sLineForDisplay[psBestLineBreakSrcPos - psReadPosAtLineStart]
/// = '\0'` truncates the OUTPUT buffer by a SOURCE-byte offset — exact only
/// when every character on the line was single-byte; a multi-byte codepoint
/// earlier on the line desyncs the two counts in Raven too (a latent
/// aliasing quirk, not something this port invents). The `Vec<u8>` twin
/// clamps the cut to the buffer's actual length instead of reading/writing
/// past it, and saturates the subtraction — the leading-space skip advances
/// `psReadPosAtLineStart` alone, so it can pass the best break position.
/// Raven also derefs `item->text` unconditionally; `None` falls back to an
/// empty string here instead of a null deref.
///
/// Source: `oracle/codemp/ui/ui_shared.c:9042-9255`
pub fn Item_TextScroll_BuildLines(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) {
    let (text, rectW, textscale, iMenuFont) = {
        let it = menus.item(item);
        (
            it.text.clone().unwrap_or_default(),
            it.window.rect.w,
            it.textscale,
            it.iMenuFont,
        )
    };
    let iBoxWidth = (rectW - SCROLLBAR_SIZE - 10.0) as c_int;

    // string reference
    let psText: Vec<u8> = if let Some(rest) = text.strip_prefix('@') {
        string_to_latin1(&dc.SP_GetStringTextString(rest, 2048).unwrap_or_default())
    } else {
        string_to_latin1(&text)
    };

    let mut lines: Vec<String> = Vec::new();
    let mut psCurrentTextReadPos: usize = 0;
    let mut psReadPosAtLineStart: usize = 0;
    let mut psBestLineBreakSrcPos: usize = 0;

    while psCurrentTextReadPos < psText.len() && lines.len() < MAX_TEXTSCROLL_LINES {
        // construct a line...
        psCurrentTextReadPos = psReadPosAtLineStart;
        let mut sLineForDisplay: Vec<u8> = Vec::new();
        let mut assigned = false;
        let mut psLastGood_s = psCurrentTextReadPos;

        while psCurrentTextReadPos < psText.len() {
            psLastGood_s = psCurrentTextReadPos;

            // read letter...
            let (uiLetter, iAdvanceCount, bIsTrailingPunctuation) =
                dc.AnyLanguage_ReadCharFromString(&psText[psCurrentTextReadPos..]);
            psCurrentTextReadPos += iAdvanceCount.max(0) as usize;

            // concat onto string so far...
            if uiLetter == 32 && sLineForDisplay.is_empty() {
                // unless it's a space at the start of a line, in which case ignore it.
                psReadPosAtLineStart += 1;
                continue;
            }

            if uiLetter > 255 {
                sLineForDisplay.push((uiLetter >> 8) as u8);
                sLineForDisplay.push((uiLetter & 0xFF) as u8);
            } else {
                sLineForDisplay.push((uiLetter & 0xFF) as u8);
            }

            if uiLetter == b'\n' as u32 {
                // explicit new line...
                sLineForDisplay.pop(); // kill the CR
                psReadPosAtLineStart = psCurrentTextReadPos;
                psBestLineBreakSrcPos = psCurrentTextReadPos;

                lines.push(latin1_to_string(&sLineForDisplay));
                assigned = true;
                break; // print this line
            } else if dc.textWidth(&latin1_to_string(&sLineForDisplay), textscale, iMenuFont)
                >= iBoxWidth
            {
                // reached screen edge, so cap off string at bytepos after last good position...
                if uiLetter > 255 && bIsTrailingPunctuation && !dc.Language_UsesSpaces() {
                    // Special case, don't consider line breaking if you're on an asian
                    // punctuation char of a language that doesn't use spaces... (breakpoint
                    // line only — no-op)
                } else {
                    if psBestLineBreakSrcPos == psReadPosAtLineStart {
                        psBestLineBreakSrcPos = psLastGood_s;
                    }

                    let cut = psBestLineBreakSrcPos
                        .saturating_sub(psReadPosAtLineStart)
                        .min(sLineForDisplay.len());
                    sLineForDisplay.truncate(cut);
                    psReadPosAtLineStart = psBestLineBreakSrcPos;
                    psCurrentTextReadPos = psBestLineBreakSrcPos;

                    lines.push(latin1_to_string(&sLineForDisplay));
                    assigned = true;
                    break; // print this line
                }
            }

            // record last-good linebreak pos... (ie if we've just concat'd a punctuation
            // point (western or asian) or space)
            if bIsTrailingPunctuation
                || uiLetter == b' ' as u32
                || (uiLetter > 255 && !dc.Language_UsesSpaces())
            {
                psBestLineBreakSrcPos = psCurrentTextReadPos;
            }
        }

        // then this is the last line and we've just run out of text, no CR, no overflow etc...
        // An empty buffer is Raven's NULL "hole" slot, which still bumps iLineCount.
        if !assigned {
            lines.push(latin1_to_string(&sLineForDisplay));
        }
    }

    if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
        scrollPtr.pLines = lines;
    }
}

// `Menu_SetupKeywordHash` — ui_shared.c:9765-9772.
//
// DEFERRED: Menu_SetupKeywordHash — builds `menuParseKeywordHash` by walking
// `menuParseKeywords[]` through `KeywordHash_Add`, the same deferred
// infrastructure as `Item_SetupKeywordHash` above.
// Source: `oracle/codemp/ui/ui_shared.c:9765-9772`

// `Menu_Parse` — ui_shared.c:9779-9810.
//
// DEFERRED: Menu_Parse — dispatches menu keywords through
// `KeywordHash_Find(menuParseKeywordHash, ...)`, the same deferred
// infrastructure as `Item_Parse` above.
// Source: `oracle/codemp/ui/ui_shared.c:9779-9810`

/// Raven `Menu_CacheContents` — pre-roll `menu`'s window cinematic, every
/// item's cinematic, and register its loop sound.
/// Source: `oracle/codemp/ui/ui_shared.c:9945-9958`
pub fn Menu_CacheContents(menus: &MenuSystem, dc: &mut dyn DisplayContext, menu: Option<MenuId>) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let m = menus.menu(menu);
    Window_CacheContents(Some(&m.window), dc);
    for &id in &m.items {
        Item_CacheContents(menus, dc, Some(id));
    }

    if !m.soundName.is_empty() {
        dc.registerSound(&m.soundName);
    }
}

// ---------------------------------------------------------------------
// wave 3
// ---------------------------------------------------------------------

/// Raven `String_Init` — reset the menu/open-menu counts and re-derive the
/// controls table.
///
/// PORT-NOTE (D2 pool retirement): `strHandle[]`/`strHandleCount`/
/// `strPoolIndex` reset the retired string-intern pool (see `String_Alloc`'s
/// PORT-NOTE) — `MenuSystem` has no such table, so nothing to zero.
/// `menuCount`/`openMenuCount` are `menus.menus`/`menus.menuStack` lengths;
/// clearing both arenas is the owned-shape equivalent.
///
/// PORT-NOTE: `Item_SetupKeywordHash`/`Menu_SetupKeywordHash` stay `//
/// DEFERRED:` (see their sites above) — the keyword-hash infrastructure they
/// build isn't ported, so there is nothing to call here.
///
/// PORT-NOTE: `if (DC && DC->getBindingBuf)` null-checks the file-scope `DC`
/// pointer and its `getBindingBuf` fn-pointer slot; `dc: &mut dyn
/// DisplayContext` is always live and always implements every method (same
/// collapse as `Script_SetTeamColor`'s PORT-NOTE), so the guard becomes an
/// unconditional call.
/// Source: `oracle/codemp/ui/ui_shared.c:363-378`
pub fn String_Init(menus: &mut MenuSystem, dc: &mut dyn DisplayContext) {
    menus.menus.clear();
    menus.menuStack.clear();
    UI_InitMemory();
    Controls_GetConfig(menus, dc);
}

/// Raven `PC_Color_Parse` — parse 4 floats from the source into `c`.
/// Source: `oracle/codemp/ui/ui_shared.c:510-521`
pub fn PC_Color_Parse(dc: &mut dyn DisplayContext, handle: c_int, c: &mut vec4_t) -> bool {
    for i in 0..4 {
        let mut f = 0.0;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        c[i] = f;
    }
    true
}

/// Raven `PC_Rect_Parse` — parse `x`, `y`, `w`, `h` from the source into `r`.
/// Source: `oracle/codemp/ui/ui_shared.c:589-600`
pub fn PC_Rect_Parse(dc: &mut dyn DisplayContext, handle: c_int, r: &mut RectDef) -> bool {
    if PC_Float_Parse(dc, handle, &mut r.x)
        && PC_Float_Parse(dc, handle, &mut r.y)
        && PC_Float_Parse(dc, handle, &mut r.w)
        && PC_Float_Parse(dc, handle, &mut r.h)
    {
        return true;
    }
    false
}

/// Raven `Item_SetScreenCoords` — reposition `item`'s window rect at
/// (`x`, `y`) plus its client offset (and border, if any), invalidating its
/// cached text rect and, for a text-scroll item, its wrapped lines.
///
/// PORT-NOTE: Raven's `item == NULL` guard becomes `item: Option<ItemId>`.
/// Source: `oracle/codemp/ui/ui_shared.c:891-930`
pub fn Item_SetScreenCoords(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
    x: f32,
    y: f32,
) {
    let item = match item {
        Some(item) => item,
        None => return,
    };

    let mut x = x;
    let mut y = y;
    {
        let it = menus.item_mut(item);
        if it.window.border != 0 {
            x += it.window.borderSize;
            y += it.window.borderSize;
        }

        it.window.rect.x = x + it.window.rectClient.x;
        it.window.rect.y = y + it.window.rectClient.y;
        it.window.rect.w = it.window.rectClient.w;
        it.window.rect.h = it.window.rectClient.h;

        // force the text rects to recompute
        it.textRect.w = 0.0;
        it.textRect.h = 0.0;
    }

    if menus.item(item).r#type == ITEM_TYPE_TEXTSCROLL {
        if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
            scrollPtr.startPos = 0;
            scrollPtr.endPos = 0;
        }

        Item_TextScroll_BuildLines(menus, dc, item);
    }
}

/// Raven `Script_SetColor` — set `item.window`'s `backColor`/`foreColor`/
/// `borderColor` from the script args (`<name> <r> <g> <b> <a>`).
///
/// PORT-NOTE: every `Script_*`/`ItemParse_*` handler in `commandDef_t
/// commandList[]` shares the uniform `(menus, dc, item, args) -> bool` shape
/// established by the already-ported `Script_SetTeamColor`/`Script_Defer`
/// (the C fn-ptr table this file's dictionary maps to a Rust dispatch); `dc`
/// is unused here (Raven's body never reaches `DC->`) but stays in the
/// signature for that uniformity.
/// Source: `oracle/codemp/ui/ui_shared.c:1063-1103`
pub fn Script_SetColor(
    menus: &mut MenuSystem,
    _dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    // expecting type of color to set and 4 args for the color
    if String_Parse(args, &mut name) {
        let it = menus.item_mut(item);
        let target: Option<fn(&mut WindowDef) -> &mut vec4_t> = if stricmp_eq(&name, "backcolor") {
            it.window.flags |= WINDOW_BACKCOLORSET;
            Some(|w: &mut WindowDef| &mut w.backColor)
        } else if stricmp_eq(&name, "forecolor") {
            it.window.flags |= WINDOW_FORECOLORSET;
            Some(|w: &mut WindowDef| &mut w.foreColor)
        } else if stricmp_eq(&name, "bordercolor") {
            Some(|w: &mut WindowDef| &mut w.borderColor)
        } else {
            None
        };

        if let Some(get) = target {
            let out = get(&mut it.window);
            for i in 0..4 {
                let mut f = 0.0;
                if !Float_Parse(args, &mut f) {
                    return true;
                }
                out[i] = f;
            }
        }
    }

    true
}

/// Raven `Script_SetAsset` — set an item's asset by name.
///
/// PORT-NOTE: Raven's `ITEM_TYPE_MODEL` branch is an empty block (`{ }`) in
/// the oracle source — no assignment happens; kept as a literal no-op.
/// Source: `oracle/codemp/ui/ui_shared.c:1105-1117`
pub fn Script_SetAsset(
    menus: &mut MenuSystem,
    _dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    // expecting name to set asset to
    if String_Parse(args, &mut name) {
        // check for a model
        if menus.item(item).r#type == ITEM_TYPE_MODEL {
            // (Raven: empty block — no-op)
        }
    }
    true
}

/// Raven `Script_SetBackground` — set `item.window.background` to the
/// registered shader named by the script arg.
/// Source: `oracle/codemp/ui/ui_shared.c:1119-1128`
pub fn Script_SetBackground(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    // expecting name to set asset to
    if String_Parse(args, &mut name) {
        let shader = dc.registerShaderNoMip(&name);
        menus.item_mut(item).window.background = shader;
    }
    true
}

/// Raven `Script_SetItemRectCvar` — copy a named item's `rectClient`/`rect`
/// (offset by the parent menu's rect) from 4 whitespace-separated floats
/// held in a cvar; zeroes the target rect on any parse failure.
/// Source: `oracle/codemp/ui/ui_shared.c:1130-1191`
pub fn Script_SetItemRectCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemName = String::new();
    let mut cvarName = String::new();
    let mut item2: Option<ItemId> = None;

    // expecting item group and cvar to get value from
    if String_Parse(args, &mut itemName) && String_Parse(args, &mut cvarName) {
        let parent = menus.item(item).parent;
        item2 = Menu_FindItemByName(menus, parent, &itemName);

        if let Some(id2) = item2 {
            // get cvar data
            let cvarBuf = dc.getCVarString(&cvarName, 1024);

            let mut holdBuf: &str = &cvarBuf;
            let mut holdVal = String::new();
            if String_Parse(&mut holdBuf, &mut holdVal) {
                let menuRect = parent
                    .map(|m| menus.menu(m).window.rect)
                    .unwrap_or_default();

                menus.item_mut(id2).window.rectClient.x = atof(&holdVal) as f32 + menuRect.x;
                if String_Parse(&mut holdBuf, &mut holdVal) {
                    menus.item_mut(id2).window.rectClient.y = atof(&holdVal) as f32 + menuRect.y;
                    if String_Parse(&mut holdBuf, &mut holdVal) {
                        menus.item_mut(id2).window.rectClient.w = atof(&holdVal) as f32;
                        if String_Parse(&mut holdBuf, &mut holdVal) {
                            menus.item_mut(id2).window.rectClient.h = atof(&holdVal) as f32;

                            let rc = menus.item(id2).window.rectClient;
                            let it2 = menus.item_mut(id2);
                            it2.window.rect.x = rc.x;
                            it2.window.rect.y = rc.y;
                            it2.window.rect.w = rc.w;
                            it2.window.rect.h = rc.h;

                            return true;
                        }
                    }
                }
            }
        }
    }

    // Default values in case things screw up
    if let Some(id2) = item2 {
        let it2 = menus.item_mut(id2);
        it2.window.rectClient.x = 0.0;
        it2.window.rectClient.y = 0.0;
        it2.window.rectClient.w = 0.0;
        it2.window.rectClient.h = 0.0;
    }

    // Com_Printf(S_COLOR_YELLOW"WARNING: SetItemRectCvar: problems. Set cvar to 0's\n" );

    true
}

/// Raven `Script_SetItemBackground` — set the background shader on every
/// item matching a named item/group in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1193-1204`
pub fn Script_SetItemBackground(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemName = String::new();
    let mut name = String::new();

    // expecting name of shader
    if String_Parse(args, &mut itemName) && String_Parse(args, &mut name) {
        let parent = menus.item(item).parent;
        Menu_SetItemBackground(menus, dc, parent, &itemName, &name);
    }
    true
}

/// Raven `Script_SetItemColor` — set `backcolor`/`forecolor`/`bordercolor`
/// on every item matching a named item/group (optionally a `*cvar`
/// indirection) in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1250-1312`
pub fn Script_SetItemColor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemname = String::new();
    let mut name = String::new();

    // expecting type of color to set and 4 args for the color
    if String_Parse(args, &mut itemname) && String_Parse(args, &mut name) {
        // Is is specifying a cvar to get the item name from?
        if itemname.starts_with('*') {
            let cvarName = itemname[1..].to_string();
            itemname = dc.getCVarString(&cvarName, 1024);
        }

        let parent = match menus.item(item).parent {
            Some(m) => m,
            None => return true,
        };
        let count = Menu_ItemsMatchingGroup(menus, dc, parent, &itemname);

        let mut color: vec4_t = [0.0; 4];
        if !Color_Parse(args, &mut color) {
            return true;
        }

        for j in 0..count {
            if let Some(id2) = Menu_GetMatchingItemByNumber(menus, parent, j, &itemname) {
                let it2 = menus.item_mut(id2);
                let out: Option<&mut vec4_t> = if stricmp_eq(&name, "backcolor") {
                    Some(&mut it2.window.backColor)
                } else if stricmp_eq(&name, "forecolor") {
                    it2.window.flags |= WINDOW_FORECOLORSET;
                    Some(&mut it2.window.foreColor)
                } else if stricmp_eq(&name, "bordercolor") {
                    Some(&mut it2.window.borderColor)
                } else {
                    None
                };

                if let Some(out) = out {
                    for i in 0..4 {
                        out[i] = color[i];
                    }
                }
            }
        }
    }

    true
}

/// Raven `Script_SetItemColorCvar` — like [`Script_SetItemColor`] but reads
/// the 4 color floats from a named cvar's whitespace-separated value instead
/// of the script args.
/// Source: `oracle/codemp/ui/ui_shared.c:1314-1401`
pub fn Script_SetItemColorCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemname = String::new();
    let mut name = String::new();

    // expecting type of color to set and 4 args for the color
    if String_Parse(args, &mut itemname) && String_Parse(args, &mut name) {
        // Is is specifying a cvar to get the item name from?
        if itemname.starts_with('*') {
            let cvarName = itemname[1..].to_string();
            itemname = dc.getCVarString(&cvarName, 1024);
        }

        let parent = match menus.item(item).parent {
            Some(m) => m,
            None => return true,
        };
        let count = Menu_ItemsMatchingGroup(menus, dc, parent, &itemname);

        // Get the cvar with the color
        let mut colorCvarName = String::new();
        if !String_Parse(args, &mut colorCvarName) {
            return true;
        }

        let mut color: vec4_t = [0.0; 4];
        let cvarBuf = dc.getCVarString(&colorCvarName, 1024);
        let mut holdBuf: &str = &cvarBuf;
        let mut holdVal = String::new();
        if String_Parse(&mut holdBuf, &mut holdVal) {
            color[0] = atof(&holdVal) as f32;
            if String_Parse(&mut holdBuf, &mut holdVal) {
                color[1] = atof(&holdVal) as f32;
                if String_Parse(&mut holdBuf, &mut holdVal) {
                    color[2] = atof(&holdVal) as f32;
                    if String_Parse(&mut holdBuf, &mut holdVal) {
                        color[3] = atof(&holdVal) as f32;
                    }
                }
            }
        }

        for j in 0..count {
            if let Some(id2) = Menu_GetMatchingItemByNumber(menus, parent, j, &itemname) {
                let it2 = menus.item_mut(id2);
                let out: Option<&mut vec4_t> = if stricmp_eq(&name, "backcolor") {
                    Some(&mut it2.window.backColor)
                } else if stricmp_eq(&name, "forecolor") {
                    it2.window.flags |= WINDOW_FORECOLORSET;
                    Some(&mut it2.window.foreColor)
                } else if stricmp_eq(&name, "bordercolor") {
                    Some(&mut it2.window.borderColor)
                } else {
                    None
                };

                if let Some(out) = out {
                    for i in 0..4 {
                        out[i] = color[i];
                    }
                }
            }
        }
    }

    true
}

/// Raven `Script_SetItemRect` — offset every item matching a named
/// item/group in `item`'s parent menu to the parsed rect (rect origin
/// relative to the parent menu's rect).
/// Source: `oracle/codemp/ui/ui_shared.c:1403-1442`
pub fn Script_SetItemRect(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemname = String::new();

    // expecting type of color to set and 4 args for the color
    if String_Parse(args, &mut itemname) {
        let parent = match menus.item(item).parent {
            Some(m) => m,
            None => return true,
        };
        let count = Menu_ItemsMatchingGroup(menus, dc, parent, &itemname);

        let mut rect = RectDef::default();
        if !Rect_Parse(args, &mut rect) {
            return true;
        }

        let menuRect = menus.menu(parent).window.rect;

        for j in 0..count {
            if let Some(id2) = Menu_GetMatchingItemByNumber(menus, parent, j, &itemname) {
                let it2 = menus.item_mut(id2);
                it2.window.rect.x = rect.x + menuRect.x;
                it2.window.rect.y = rect.y + menuRect.y;
                it2.window.rect.w = rect.w;
                it2.window.rect.h = rect.h;
            }
        }
    }
    true
}

/// Raven `Script_Open` — parse a menu name and open it (`Menus_OpenByName`).
/// Source: `oracle/codemp/ui/ui_shared.c:1632-1640`
pub fn Script_Open(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        Menus_OpenByName(menus, ds, dc, &name);
    }
    true
}

/// Raven `Script_Show` — set `WINDOW_VISIBLE` on every item matching a
/// named item/group in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1591-1599`
pub fn Script_Show(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        if let Some(parent) = menus.item(item).parent {
            Menu_ShowItemByName(menus, dc, parent, &name, true);
        }
    }
    true
}

/// Raven `Script_Hide` — clear `WINDOW_VISIBLE` on every item matching a
/// named item/group in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1601-1609`
pub fn Script_Hide(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        if let Some(parent) = menus.item(item).parent {
            Menu_ShowItemByName(menus, dc, parent, &name, false);
        }
    }
    true
}

/// Raven `Script_FadeIn` — start a fade-in on every item matching a named
/// item/group in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1611-1620`
pub fn Script_FadeIn(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        if let Some(parent) = menus.item(item).parent {
            Menu_FadeItemByName(menus, dc, parent, &name, false);
        }
    }

    true
}

/// Raven `Script_FadeOut` — start a fade-out on every item matching a named
/// item/group in `item`'s parent menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1622-1630`
pub fn Script_FadeOut(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        if let Some(parent) = menus.item(item).parent {
            Menu_FadeItemByName(menus, dc, parent, &name, true);
        }
    }
    true
}

/// Raven `Script_Disable` — set/clear the `disabled` flag (and clear
/// mouseover) on every item matching a named item/group (optionally a
/// `*cvar` indirection) in the currently-focused menu.
/// Source: `oracle/codemp/ui/ui_shared.c:1865-1891`
pub fn Script_Disable(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();

    if String_Parse(args, &mut name) {
        // Is is specifying a cvar to get the item name from?
        if name.starts_with('*') {
            let cvarName = name[1..].to_string();
            name = dc.getCVarString(&cvarName, 1024);
        }

        let mut value = 0;
        if Int_Parse(args, &mut value) {
            if let Some(menu) = Menu_GetFocused(menus) {
                Menu_ItemDisable(menus, dc, menu, &name, value);
            }
        }
    }

    true
}

/// Raven `Script_SetPlayerModel` — set the `model` cvar to the script arg.
/// Source: `oracle/codemp/ui/ui_shared.c:1986-1995`
pub fn Script_SetPlayerModel(
    _menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        dc.setCVar("model", &name);
    }

    true
}

/// Raven `Script_Transition3` — parse a full 3-source/3-target transition
/// (position, model-preview bounds, fov, duration, blend amount) off the
/// script args and start it on the named item/group.
///
/// PORT-NOTE: dead in Raven — `commandList[]` (`ui_shared.c:2196-2228`) has no
/// entry for it, so no script can reach it.
///
/// PORT-NOTE (§19 UB pick): the trailing warning reads `name`, which Raven
/// leaves uninitialized if the very first `String_Parse` fails; this port
/// initializes it empty instead of reproducing that read of uninitialized
/// memory.
/// Source: `oracle/codemp/ui/ui_shared.c:2062-2125`
pub fn Script_Transition3(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    let mut value = String::new();

    if String_Parse(args, &mut name) {
        if String_Parse(args, &mut value) {
            let minx = atof(&value) as f32;
            if String_Parse(args, &mut value) {
                let miny = atof(&value) as f32;
                if String_Parse(args, &mut value) {
                    let minz = atof(&value) as f32;
                    if String_Parse(args, &mut value) {
                        let maxx = atof(&value) as f32;
                        if String_Parse(args, &mut value) {
                            let maxy = atof(&value) as f32;
                            if String_Parse(args, &mut value) {
                                let maxz = atof(&value) as f32;
                                if String_Parse(args, &mut value) {
                                    let fovtx = atof(&value) as f32;
                                    if String_Parse(args, &mut value) {
                                        let fovty = atof(&value) as f32;
                                        if String_Parse(args, &mut value) {
                                            let time = atoi(&value);
                                            if String_Parse(args, &mut value) {
                                                let amt = atof(&value) as f32;
                                                // set up the variables
                                                if let Some(parent) = menus.item(item).parent {
                                                    Menu_Transition3ItemByName(
                                                        menus, dc, parent, &name, minx, miny, minz,
                                                        maxx, maxy, maxz, fovtx, fovty, time, amt,
                                                    );
                                                }
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    dc.Print(&format!(
        "^3WARNING: Script_Transition2: error parsing '{}'\n",
        name
    ));
    true
}

/// Raven `Script_SetCvar` — set a cvar from the script args.
/// Source: `oracle/codemp/ui/ui_shared.c:2150-2158`
pub fn Script_SetCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let _ = menus;
    let mut cvar = String::new();
    let mut val = String::new();
    if String_Parse(args, &mut cvar) && String_Parse(args, &mut val) {
        dc.setCVar(&cvar, &val);
    }
    true
}

/// Raven `Script_SetCvarToCvar` — copy one cvar's value into another.
/// Source: `oracle/codemp/ui/ui_shared.c:2160-2168`
pub fn Script_SetCvarToCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let _ = menus;
    let mut cvar = String::new();
    let mut val = String::new();
    if String_Parse(args, &mut cvar) && String_Parse(args, &mut val) {
        let cvarBuf = dc.getCVarString(&val, 1024);
        dc.setCVar(&cvar, &cvarBuf);
    }
    true
}

/// Raven `Script_Exec` — append a console command from the script args.
/// Source: `oracle/codemp/ui/ui_shared.c:2170-2176`
pub fn Script_Exec(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let _ = menus;
    let mut val = String::new();
    if String_Parse(args, &mut val) {
        dc.executeText(cbufExec_t::EXEC_APPEND as c_int, &format!("{} ; ", val));
    }
    true
}

/// Raven `Script_Play` — play a local sound named by the script arg.
/// Source: `oracle/codemp/ui/ui_shared.c:2178-2184`
pub fn Script_Play(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let _ = menus;
    let mut val = String::new();
    if String_Parse(args, &mut val) {
        let sfx = dc.registerSound(&val);
        dc.startLocalSound(sfx, CHAN_AUTO);
    }
    true
}

/// Raven `Script_playLooped` — start a looped background track named by the
/// script arg.
/// Source: `oracle/codemp/ui/ui_shared.c:2186-2193`
pub fn Script_playLooped(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let _ = menus;
    let mut val = String::new();
    if String_Parse(args, &mut val) {
        dc.stopBackgroundTrack();
        dc.startBackgroundTrack(&val, &val, false);
    }
    true
}

/// Raven `Menu_SetItemText` — set the display text (or `*cvar` indirection)
/// on every item matching `itemName` (by name or group) in `menu`.
/// Source: `oracle/codemp/ui/ui_shared.c:2254-2302`
pub fn Menu_SetItemText(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    itemName: &str,
    text: &str,
) {
    // No menu???
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let count = Menu_ItemsMatchingGroup(menus, dc, menu, itemName);

    for j in 0..count {
        let item = match Menu_GetMatchingItemByNumber(menus, menu, j, itemName) {
            Some(id) => id,
            None => continue,
        };

        if let Some(stripped) = text.strip_prefix('*') {
            let it = menus.item_mut(item);
            // Null this out because this would take presidence over cvar text.
            it.text = None;
            it.cvar = Some(stripped.to_string());
            // Just copying what was in ItemParse_cvar()
            // PORT-NOTE (§19 UB pick): Raven's unconditional `editFieldDef_t*` cast
            // type-puns non-edit-field payloads (ui_shared.c:2276-2283); write only
            // when the payload really is an edit field.
            if let Some(editPtr) = it.typeData.editField_mut() {
                editPtr.minVal = -1.0;
                editPtr.maxVal = -1.0;
                editPtr.defVal = -1.0;
            }
        } else {
            menus.item_mut(item).text = Some(text.to_string());
            if menus.item(item).r#type == ITEM_TYPE_TEXTSCROLL {
                if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
                    scrollPtr.startPos = 0;
                    scrollPtr.endPos = 0;
                }
                Item_TextScroll_BuildLines(menus, dc, item);
            }
        }
    }
}

/// Local dispatch table replacing Raven's `commandDef_t commandList[]`
/// (`ui_shared.c` file-scope, `scriptCommandCount` entries) that
/// `Item_RunScript` walks by `Q_stricmp`-ing the leading command token
/// against each entry's name.
///
/// PORT-NOTE: the keys below are the oracle table's literal command strings.
/// `open`, `close`, `setitemtext`, `setfocus`, `transition`, `orbit`, `scale`,
/// `rundeferred`, `transition2` are all ported as fns but not yet dispatched
/// here — they fall through to `None` (routed to `dc.runScript`, matching
/// Raven's un-dispatched-command path) until the dispatch wire-up wave adds
/// their arms. Raven's table has no `enable` entry (and none for
/// `transition3`), so neither is dispatchable here.
/// Source: `oracle/codemp/ui/ui_shared.c:2196-2228` (the table),
/// `oracle/codemp/ui/ui_shared.c:2306-2357` (the walk this replaces)
fn dispatch_script_command(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    command: &str,
    args: &mut &str,
) -> Option<bool> {
    if stricmp_eq(command, "setasset") {
        Some(Script_SetAsset(menus, dc, item, args))
    } else if stricmp_eq(command, "setbackground") {
        Some(Script_SetBackground(menus, dc, item, args))
    } else if stricmp_eq(command, "setitembackground") {
        Some(Script_SetItemBackground(menus, dc, item, args))
    } else if stricmp_eq(command, "setitemrectcvar") {
        Some(Script_SetItemRectCvar(menus, dc, item, args))
    } else if stricmp_eq(command, "setcolor") {
        Some(Script_SetColor(menus, dc, item, args))
    } else if stricmp_eq(command, "setitemcolor") {
        Some(Script_SetItemColor(menus, dc, item, args))
    } else if stricmp_eq(command, "setitemcolorcvar") {
        Some(Script_SetItemColorCvar(menus, dc, item, args))
    } else if stricmp_eq(command, "setteamcolor") {
        Some(Script_SetTeamColor(menus, dc, item, args))
    } else if stricmp_eq(command, "setitemrect") {
        Some(Script_SetItemRect(menus, dc, item, args))
    } else if stricmp_eq(command, "show") {
        Some(Script_Show(menus, dc, item, args))
    } else if stricmp_eq(command, "hide") {
        Some(Script_Hide(menus, dc, item, args))
    } else if stricmp_eq(command, "setcvar") {
        Some(Script_SetCvar(menus, dc, item, args))
    } else if stricmp_eq(command, "exec") {
        Some(Script_Exec(menus, dc, item, args))
    } else if stricmp_eq(command, "play") {
        Some(Script_Play(menus, dc, item, args))
    } else if stricmp_eq(command, "playlooped") {
        Some(Script_playLooped(menus, dc, item, args))
    } else if stricmp_eq(command, "setplayermodel") {
        Some(Script_SetPlayerModel(menus, dc, item, args))
    } else if stricmp_eq(command, "setcvartocvar") {
        Some(Script_SetCvarToCvar(menus, dc, item, args))
    } else if stricmp_eq(command, "fadein") {
        Some(Script_FadeIn(menus, dc, item, args))
    } else if stricmp_eq(command, "fadeout") {
        Some(Script_FadeOut(menus, dc, item, args))
    } else if stricmp_eq(command, "disable") {
        Some(Script_Disable(menus, dc, item, args))
    } else if stricmp_eq(command, "defer") {
        Some(Script_Defer(menus, dc, item, args))
    } else {
        None
    }
}

/// Raven `Item_RunScript` — tokenize `s` as a `;`-separated command script
/// and run each command through [`dispatch_script_command`] (Raven's
/// `commandList[]` walk), falling back to the host's own script parser
/// (`DC->runScript`) for anything not in that table.
///
/// PORT-NOTE: `Q_strcat(script, 2048, s)`'s fixed 2048-byte buffer truncates
/// at a valid char boundary (`Script_Defer`'s established pattern).
/// Source: `oracle/codemp/ui/ui_shared.c:2306-2357`
pub fn Item_RunScript(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, item: ItemId, s: &str) {
    if s.is_empty() {
        return;
    }

    let mut script = s.to_string();
    if script.len() >= 2048 {
        let mut cut = 2047;
        while cut > 0 && !script.is_char_boundary(cut) {
            cut -= 1;
        }
        script.truncate(cut);
    }

    let mut p: &str = &script;
    loop {
        let mut command = String::new();

        // expect command then arguments, ; ends command, empty ends script
        if !String_Parse(&mut p, &mut command) {
            return;
        }

        if command == ";" {
            continue;
        }

        match dispatch_script_command(menus, dc, item, &command, &mut p) {
            Some(ran) => {
                // Allow a script command to stop processing the script.
                if !ran {
                    return;
                }
            }
            // not in our auto list, pass to handler
            None => dc.runScript(&mut p),
        }
    }
}

/// Raven `Item_EnableShowViaCvar` — should `item` be enabled/shown given
/// `flag` and its `enableCvar`/`cvarTest` values?
///
/// PORT-NOTE: Raven's `Q_strncpyz(script, item->enableCvar, 2048)` copies
/// into a fixed buffer before parsing; `enableCvar` is already an owned
/// `String`, so the copy collapses to parsing it directly.
/// Source: `oracle/codemp/ui/ui_shared.c:2360-2395`
pub fn Item_EnableShowViaCvar(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    flag: c_int,
) -> bool {
    let it = menus.item(item);
    let enableCvar = it.enableCvar.clone();
    let cvarTest = it.cvarTest.clone();
    let cvarFlags = it.cvarFlags;

    if !enableCvar.is_empty() && !cvarTest.is_empty() {
        let buff = dc.getCVarString(&cvarTest, 2048);

        let mut p: &str = &enableCvar;
        loop {
            let mut val = String::new();
            // expect value then ; or empty, empty ends list
            if !String_Parse(&mut p, &mut val) {
                return !(cvarFlags & flag != 0);
            }

            if val == ";" {
                continue;
            }

            // enable it if any of the values are true
            if cvarFlags & flag != 0 {
                if stricmp_eq(&buff, &val) {
                    return true;
                }
            } else {
                // disable it if any of the values are true
                if stricmp_eq(&buff, &val) {
                    return false;
                }
            }
        }
    }
    true
}

/// Raven `Item_TextScroll_MouseEnter` — refresh a text-scroll item's
/// scrollbar hot-zone flags for the mouse at `(x, y)`.
/// Source: `oracle/codemp/ui/ui_shared.c:2587-2591`
pub fn Item_TextScroll_MouseEnter(menus: &mut MenuSystem, item: ItemId, x: f32, y: f32) {
    menus.item_mut(item).window.flags &= !(WINDOW_LB_LEFTARROW
        | WINDOW_LB_RIGHTARROW
        | WINDOW_LB_THUMB
        | WINDOW_LB_PGUP
        | WINDOW_LB_PGDN);
    let flags = Item_TextScroll_OverLB(menus, item, x, y);
    menus.item_mut(item).window.flags |= flags;
}

/// Raven `Item_ListBox_MouseEnter` — refresh a list box's scrollbar hot-zone
/// flags for the mouse at `(x, y)`, then (if no hot zone is hit) update the
/// selection cursor from the pointer position.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) is a no-op instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:2956-3026`
pub fn Item_ListBox_MouseEnter(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    x: f32,
    y: f32,
) {
    menus.item_mut(item).window.flags &= !(WINDOW_LB_LEFTARROW
        | WINDOW_LB_RIGHTARROW
        | WINDOW_LB_THUMB
        | WINDOW_LB_PGUP
        | WINDOW_LB_PGDN);

    let flags = Item_ListBox_OverLB(menus, dc, item, x, y);
    menus.item_mut(item).window.flags |= flags;

    let windowFlags = menus.item(item).window.flags;
    let blocked = windowFlags
        & (WINDOW_LB_LEFTARROW
            | WINDOW_LB_RIGHTARROW
            | WINDOW_LB_THUMB
            | WINDOW_LB_PGUP
            | WINDOW_LB_PGDN)
        != 0;

    if windowFlags & WINDOW_HORIZONTAL != 0 {
        if !blocked {
            let listPtr = match menus.item(item).typeData.listBox() {
                Some(l) => l.clone(),
                None => return,
            };
            // check for selection hit as we have exhausted buttons and thumb
            if listPtr.elementStyle == LISTBOX_IMAGE {
                let rect = menus.item(item).window.rect;
                let r = RectDef {
                    x: rect.x,
                    y: rect.y,
                    h: rect.h - SCROLLBAR_SIZE,
                    w: rect.w - listPtr.drawPadding as f32,
                };
                if Rect_ContainsPoint(Some(&r), x, y) {
                    let mut cursorPos =
                        ((x - r.x) / listPtr.elementWidth) as c_int + listPtr.startPos;
                    if cursorPos >= listPtr.endPos {
                        cursorPos = listPtr.endPos;
                    }
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                    }
                }
            }
            // else: text hit.. (Raven: empty block)
        }
    } else if !blocked {
        // Window Vertical Scroll — calc which element the mouse is over
        let listPtr = match menus.item(item).typeData.listBox() {
            Some(l) => l.clone(),
            None => return,
        };
        let rect = menus.item(item).window.rect;
        let r = RectDef {
            x: rect.x,
            y: rect.y,
            w: rect.w - SCROLLBAR_SIZE,
            h: rect.h - listPtr.drawPadding as f32,
        };
        if Rect_ContainsPoint(Some(&r), x, y) {
            let cursorPos;
            // Multiple rows and columns (since it's more than twice as wide as an element)
            if rect.w > (listPtr.elementWidth * 2.0) && listPtr.elementStyle == LISTBOX_IMAGE {
                let row = ((y - 2.0 - r.y) / listPtr.elementHeight) as c_int;
                // Raven's `(int) r.w / listPtr->elementWidth` casts `r.w` alone, so
                // the divide is float with a truncated numerator.
                let rowLength = ((r.w as c_int) as f32 / listPtr.elementWidth) as c_int;
                let column = ((x - r.x) / listPtr.elementWidth) as c_int;

                let mut cp = row * rowLength + column + listPtr.startPos;
                if cp >= listPtr.endPos {
                    cp = listPtr.endPos;
                }
                cursorPos = cp;
            } else {
                // single column
                let mut cp = ((y - 2.0 - r.y) / listPtr.elementHeight) as c_int + listPtr.startPos;
                if cp > listPtr.endPos {
                    cp = listPtr.endPos;
                }
                cursorPos = cp;
            }
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.cursorPos = cursorPos;
            }
        }
    }
}

/// Raven `Item_StartCapture` — give the mouse-capture scroll handler to
/// `item`'s scrollbar/thumb hot zone under the cursor (if any).
/// Source: `oracle/codemp/ui/ui_shared.c:4022-4095`
pub fn Item_StartCapture(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
) {
    let itemType = menus.item(item).r#type;
    let cursorx = ds.cursorx as f32;
    let cursory = ds.cursory as f32;

    if itemType == ITEM_TYPE_EDITFIELD
        || itemType == ITEM_TYPE_NUMERICFIELD
        || itemType == ITEM_TYPE_LISTBOX
    {
        let flags = Item_ListBox_OverLB(menus, dc, item, cursorx, cursory);
        if flags & (WINDOW_LB_LEFTARROW | WINDOW_LB_RIGHTARROW) != 0 {
            menus.scrollInfo.nextScrollTime = ds.realTime + SCROLL_TIME_START;
            menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
            menus.scrollInfo.adjustValue = SCROLL_TIME_START;
            menus.scrollInfo.scrollKey = key;
            menus.scrollInfo.scrollDir = flags & WINDOW_LB_LEFTARROW != 0;
            menus.scrollInfo.item = Some(item);
            menus.captureFunc = CaptureFunc::ScrollListBoxAuto;
            menus.itemCapture = Some(item);
        } else if flags & WINDOW_LB_THUMB != 0 {
            menus.scrollInfo.scrollKey = key;
            menus.scrollInfo.item = Some(item);
            menus.scrollInfo.xStart = cursorx;
            menus.scrollInfo.yStart = cursory;
            menus.captureFunc = CaptureFunc::ScrollListBoxThumb;
            menus.itemCapture = Some(item);
        }
    } else if itemType == ITEM_TYPE_TEXTSCROLL {
        let flags = Item_TextScroll_OverLB(menus, item, cursorx, cursory);
        if flags & (WINDOW_LB_LEFTARROW | WINDOW_LB_RIGHTARROW) != 0 {
            menus.scrollInfo.nextScrollTime = ds.realTime + SCROLL_TIME_START;
            menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
            menus.scrollInfo.adjustValue = SCROLL_TIME_START;
            menus.scrollInfo.scrollKey = key;
            menus.scrollInfo.scrollDir = flags & WINDOW_LB_LEFTARROW != 0;
            menus.scrollInfo.item = Some(item);
            menus.captureFunc = CaptureFunc::ScrollTextScrollAuto;
            menus.itemCapture = Some(item);
        } else if flags & WINDOW_LB_THUMB != 0 {
            menus.scrollInfo.scrollKey = key;
            menus.scrollInfo.item = Some(item);
            menus.scrollInfo.xStart = cursorx;
            menus.scrollInfo.yStart = cursory;
            menus.captureFunc = CaptureFunc::ScrollTextScrollThumb;
            menus.itemCapture = Some(item);
        }
    } else if itemType == ITEM_TYPE_SLIDER {
        let flags = Item_Slider_OverSlider(menus, dc, item, cursorx, cursory);
        if flags & WINDOW_LB_THUMB != 0 {
            menus.scrollInfo.scrollKey = key;
            menus.scrollInfo.item = Some(item);
            menus.scrollInfo.xStart = cursorx;
            menus.scrollInfo.yStart = cursory;
            menus.captureFunc = CaptureFunc::ScrollSliderThumb;
            menus.itemCapture = Some(item);
        }
    }
}

/// Raven `Item_TextScroll_Paint` — paint a text-scroll item's scrollbar,
/// refresh its lines from its cvar (if any), then paint the visible lines.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) is a no-op instead of Raven's
/// null-deref UB. `pLines`' NULL "hole" slots (see `TextScrollDef`'s
/// PORT-NOTE) are empty-string entries — the `if (!text) continue;` guard
/// becomes an empty-string check.
/// Source: `oracle/codemp/ui/ui_shared.c:5911-5974`
pub fn Item_TextScroll_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) {
    let mut scrollPtr = match menus.item(item).typeData.textScroll().cloned() {
        Some(s) => s,
        None => return,
    };

    // Raven reads `count` before the cvar-driven `Item_TextScroll_BuildLines`
    // rebuild below, so the paint loop walks the pre-rebuild line count.
    let count = scrollPtr.pLines.len() as c_int;

    let rect = menus.item(item).window.rect;

    // draw scrollbar to right side of the window
    let x = rect.x + rect.w - SCROLLBAR_SIZE - 1.0;
    let mut y = rect.y + 1.0;
    dc.drawHandlePic(
        x,
        y,
        SCROLLBAR_SIZE,
        SCROLLBAR_SIZE,
        ds.Assets.scrollBarArrowUp,
    );
    y += SCROLLBAR_SIZE - 1.0;

    scrollPtr.endPos = scrollPtr.startPos;
    let barSize = rect.h - (SCROLLBAR_SIZE * 2.0);
    dc.drawHandlePic(x, y, SCROLLBAR_SIZE, barSize + 1.0, ds.Assets.scrollBar);
    y += barSize - 1.0;
    dc.drawHandlePic(
        x,
        y,
        SCROLLBAR_SIZE,
        SCROLLBAR_SIZE,
        ds.Assets.scrollBarArrowDown,
    );

    // thumb
    let mut thumb = Item_TextScroll_ThumbDrawPosition(menus, ds, item) as f32;
    if thumb > y - SCROLLBAR_SIZE - 1.0 {
        thumb = y - SCROLLBAR_SIZE - 1.0;
    }
    dc.drawHandlePic(
        x,
        thumb,
        SCROLLBAR_SIZE,
        SCROLLBAR_SIZE,
        ds.Assets.scrollBarThumb,
    );

    if let Some(cvar) = menus.item(item).cvar.clone() {
        let cvartext = dc.getCVarString(&cvar, 1024);
        menus.item_mut(item).text = Some(cvartext);
        Item_TextScroll_BuildLines(menus, dc, item);
        // `Item_TextScroll_BuildLines` rewrote `typeData`; refresh the local copy.
        scrollPtr = menus
            .item(item)
            .typeData
            .textScroll()
            .cloned()
            .unwrap_or(scrollPtr);
        // Raven's earlier `endPos = startPos` write landed on the live struct and
        // survives the rebuild (`BuildLines` never touches either field).
        scrollPtr.endPos = scrollPtr.startPos;
    }

    // adjust size for item painting
    let it = menus.item(item);
    let mut size = it.window.rect.h - 2.0;
    let x = it.window.rect.x + it.textalignx + 1.0;
    let mut y = it.window.rect.y + it.textaligny + 1.0;
    let textscale = it.textscale;
    let foreColor = it.window.foreColor;
    let textStyle = it.textStyle;
    let iMenuFont = it.iMenuFont;

    let mut i = scrollPtr.startPos;
    while i < count {
        // A stale `count` past the rebuilt line list is Raven's memset-NULL slot.
        let text = scrollPtr
            .pLines
            .get(i as usize)
            .cloned()
            .unwrap_or_default();
        if text.is_empty() {
            i += 1;
            continue;
        }

        dc.drawText(
            x + 4.0,
            y,
            textscale,
            foreColor,
            &text,
            0.0,
            0,
            textStyle,
            iMenuFont,
        );

        size -= scrollPtr.lineHeight;
        if size < scrollPtr.lineHeight {
            scrollPtr.drawPadding = (scrollPtr.lineHeight - size) as c_int;
            break;
        }

        scrollPtr.endPos += 1;
        y += scrollPtr.lineHeight;
        i += 1;
    }

    if let Some(sp) = menus.item_mut(item).typeData.textScroll_mut() {
        *sp = scrollPtr;
    }
}

/// Raven `Item_ListBox_Paint` — paint a list box's scrollbar and visible
/// elements (image or text/column style, horizontal or vertical).
///
/// PORT-NOTE: Raven declares several of its loop locals (`count`, `i`, `i2`)
/// `float`; this port keeps them the natural `c_int` the feeder/element
/// counts already are (same collapse as `Item_TextScroll_MaxScroll`'s
/// PORT-NOTE). Debug-only (`_DEBUG`) and Xbox-only (`_XBOX`) build arms are
/// dropped per porting-rules §20 (dead surface on this port's targets). A
/// `typeData` payload-type mismatch (unreachable under this file's own type
/// dispatch) is a no-op instead of Raven's null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:5979-6367`
pub fn Item_ListBox_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) {
    let mut listPtr = match menus.item(item).typeData.listBox().cloned() {
        Some(l) => l,
        None => return,
    };

    // the listbox is horizontal or vertical and has a fixed size scroll bar going either direction
    // elements are enumerated from the DC and either text or image handles are acquired from the DC as well
    // textscale is used to size the text, textalignx and textaligny are used to size image elements
    // there is no clipping available so only the last completely visible item is painted
    let special = menus.item(item).special;
    let count = dc.feederCount(special);

    let maxIndex = if count != 0 { count - 1 } else { count };

    if listPtr.startPos > maxIndex {
        // probably changed feeders, so reset
        listPtr.startPos = 0;
        // Raven's reset lands on the live payload, which the scroll/thumb helpers
        // below re-read; flush it before they run.
        if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
            l.startPos = 0;
        }
    }

    let mut cursorPos = menus.item(item).cursorPos;
    if cursorPos > maxIndex {
        // probably changed feeders, so reset
        cursorPos = maxIndex;
        menus.item_mut(item).cursorPos = cursorPos;
        // NOTE : might consider moving this to any spot in here we change the cursor position
        dc.feederSelection(special, cursorPos, None);
    }

    let rect = menus.item(item).window.rect;
    let windowFlags = menus.item(item).window.flags;
    let borderSize = menus.item(item).window.borderSize;
    let borderColor = menus.item(item).window.borderColor;
    let outlineColor = menus.item(item).window.outlineColor;
    let textscale = menus.item(item).textscale;
    let foreColor = menus.item(item).window.foreColor;
    let textStyle = menus.item(item).textStyle;
    let iMenuFont = menus.item(item).iMenuFont;
    let textaligny = menus.item(item).textaligny;

    // default is vertical if horizontal flag is not here
    if windowFlags & WINDOW_HORIZONTAL != 0 {
        if !listPtr.scrollhidden {
            // draw scrollbar in bottom of the window
            // bar
            if Item_ListBox_MaxScroll(menus, dc, item) > 0 {
                let mut x = rect.x + 1.0;
                let y = rect.y + rect.h - SCROLLBAR_SIZE - 1.0;
                dc.drawHandlePic(
                    x,
                    y,
                    SCROLLBAR_SIZE,
                    SCROLLBAR_SIZE,
                    ds.Assets.scrollBarArrowLeft,
                );
                x += SCROLLBAR_SIZE - 1.0;
                let barWidth = rect.w - (SCROLLBAR_SIZE * 2.0);
                dc.drawHandlePic(x, y, barWidth + 1.0, SCROLLBAR_SIZE, ds.Assets.scrollBar);
                x += barWidth - 1.0;
                dc.drawHandlePic(
                    x,
                    y,
                    SCROLLBAR_SIZE,
                    SCROLLBAR_SIZE,
                    ds.Assets.scrollBarArrowRight,
                );
                // thumb
                let mut thumb = Item_ListBox_ThumbDrawPosition(menus, ds, dc, item) as f32;
                if thumb > x - SCROLLBAR_SIZE - 1.0 {
                    thumb = x - SCROLLBAR_SIZE - 1.0;
                }
                dc.drawHandlePic(
                    thumb,
                    y,
                    SCROLLBAR_SIZE,
                    SCROLLBAR_SIZE,
                    ds.Assets.scrollBarThumb,
                );
            } else if listPtr.startPos > 0 {
                listPtr.startPos = 0;
            }
        }

        listPtr.endPos = listPtr.startPos;
        let mut sizeWidth = rect.w - 2.0;

        if listPtr.elementStyle == LISTBOX_IMAGE {
            let mut x = rect.x + 1.0;
            let y = rect.y + 1.0;
            let mut i = listPtr.startPos;
            while i < count {
                let image = dc.feederItemImage(special, i);
                if image != 0 {
                    // PORT-NOTE: the `#ifndef CGAME` (ui) arm, per this file's convention.
                    if windowFlags & WINDOW_PLAYERCOLOR != 0 {
                        let color: vec4_t = [
                            (dc.getCVarValue("ui_char_color_red") as c_int) as f32 / 255.0,
                            (dc.getCVarValue("ui_char_color_green") as c_int) as f32 / 255.0,
                            (dc.getCVarValue("ui_char_color_blue") as c_int) as f32 / 255.0,
                            1.0,
                        ];
                        dc.setColor(Some(color));
                    }
                    dc.drawHandlePic(
                        x + 1.0,
                        y + 1.0,
                        listPtr.elementWidth - 2.0,
                        listPtr.elementHeight - 2.0,
                        image,
                    );
                }

                if i == cursorPos {
                    dc.drawRect(
                        x,
                        y,
                        listPtr.elementWidth - 1.0,
                        listPtr.elementHeight - 1.0,
                        borderSize,
                        borderColor,
                    );
                }

                sizeWidth -= listPtr.elementWidth;
                if sizeWidth < listPtr.elementWidth {
                    listPtr.drawPadding = sizeWidth as c_int;
                    break;
                }
                x += listPtr.elementWidth;
                listPtr.endPos += 1;
                i += 1;
            }
        }
        // else: text style — Raven's body is an empty block (no-op).
    } else {
        // A vertical list box
        if !listPtr.scrollhidden {
            // draw scrollbar to right side of the window
            let x = rect.x + rect.w - SCROLLBAR_SIZE - 1.0;
            let mut y = rect.y + 1.0;
            dc.drawHandlePic(
                x,
                y,
                SCROLLBAR_SIZE,
                SCROLLBAR_SIZE,
                ds.Assets.scrollBarArrowUp,
            );
            y += SCROLLBAR_SIZE - 1.0;

            listPtr.endPos = listPtr.startPos;
            // Raven's write lands on the live payload before the thumb helper below.
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.endPos = listPtr.endPos;
            }
            let barHeight = rect.h - (SCROLLBAR_SIZE * 2.0);
            dc.drawHandlePic(x, y, SCROLLBAR_SIZE, barHeight + 1.0, ds.Assets.scrollBar);
            y += barHeight - 1.0;
            dc.drawHandlePic(
                x,
                y,
                SCROLLBAR_SIZE,
                SCROLLBAR_SIZE,
                ds.Assets.scrollBarArrowDown,
            );
            // thumb
            let mut thumb = Item_ListBox_ThumbDrawPosition(menus, ds, dc, item) as f32;
            if thumb > y - SCROLLBAR_SIZE - 1.0 {
                thumb = y - SCROLLBAR_SIZE - 1.0;
            }
            dc.drawHandlePic(
                x,
                thumb,
                SCROLLBAR_SIZE,
                SCROLLBAR_SIZE,
                ds.Assets.scrollBarThumb,
            );
        }

        // adjust size for item painting
        let mut sizeWidth = rect.w - 2.0;
        let mut sizeHeight = rect.h - 2.0;

        if listPtr.elementStyle == LISTBOX_IMAGE {
            // Multiple rows and columns (since it's more than twice as wide as an element)
            if rect.w > (listPtr.elementWidth * 2.0) {
                let mut startPos = listPtr.startPos;
                let mut y = rect.y + 1.0;
                let mut i2 = startPos;
                while i2 < count {
                    let mut x = rect.x + 1.0;
                    sizeWidth = rect.w - 2.0;
                    // print a row
                    let mut i = startPos;
                    while i < count {
                        let image = dc.feederItemImage(special, i);
                        if image != 0 {
                            if windowFlags & WINDOW_PLAYERCOLOR != 0 {
                                let color: vec4_t = [
                                    (dc.getCVarValue("ui_char_color_red") as c_int) as f32 / 255.0,
                                    (dc.getCVarValue("ui_char_color_green") as c_int) as f32
                                        / 255.0,
                                    (dc.getCVarValue("ui_char_color_blue") as c_int) as f32 / 255.0,
                                    1.0,
                                ];
                                dc.setColor(Some(color));
                            }
                            dc.drawHandlePic(
                                x + 1.0,
                                y + 1.0,
                                listPtr.elementWidth - 2.0,
                                listPtr.elementHeight - 2.0,
                                image,
                            );
                        }

                        if i == cursorPos {
                            dc.drawRect(
                                x,
                                y,
                                listPtr.elementWidth - 1.0,
                                listPtr.elementHeight - 1.0,
                                borderSize,
                                borderColor,
                            );
                        }

                        sizeWidth -= listPtr.elementWidth;
                        if sizeWidth < listPtr.elementWidth {
                            listPtr.drawPadding = sizeWidth as c_int;
                            break;
                        }
                        x += listPtr.elementWidth;
                        listPtr.endPos += 1;
                        i += 1;
                    }

                    sizeHeight -= listPtr.elementHeight;
                    if sizeHeight < listPtr.elementHeight {
                        listPtr.drawPadding = sizeHeight as c_int;
                        break;
                    }
                    // NOTE : Is endPos supposed to be valid or not? It was being used as a valid entry but I changed those
                    // few spots that were causing bugs
                    listPtr.endPos += 1;
                    startPos = listPtr.endPos;
                    y += listPtr.elementHeight;
                    i2 += 1;
                }
            } else {
                // single column
                let x = rect.x + 1.0;
                let mut y = rect.y + 1.0;
                let mut i = listPtr.startPos;
                while i < count {
                    let image = dc.feederItemImage(special, i);
                    if image != 0 {
                        dc.drawHandlePic(
                            x + 1.0,
                            y + 1.0,
                            listPtr.elementWidth - 2.0,
                            listPtr.elementHeight - 2.0,
                            image,
                        );
                    }

                    if i == cursorPos {
                        dc.drawRect(
                            x,
                            y,
                            listPtr.elementWidth - 1.0,
                            listPtr.elementHeight - 1.0,
                            borderSize,
                            borderColor,
                        );
                    }

                    listPtr.endPos += 1;
                    sizeHeight -= listPtr.elementHeight;
                    if sizeHeight < listPtr.elementHeight {
                        listPtr.drawPadding = (listPtr.elementHeight - sizeHeight) as c_int;
                        break;
                    }
                    y += listPtr.elementHeight;
                    i += 1;
                }
            }
        } else {
            let x = rect.x + 1.0;
            // MPMOVED: the plain `y = rect.y + 1` assignment above this in the
            // oracle is immediately overwritten by this one before use.
            let mut y = rect.y + 1.0 - listPtr.elementHeight;
            let mut i = listPtr.startPos;

            while i < count {
                if listPtr.numColumns > 0 {
                    for j in 0..listPtr.numColumns {
                        let mut imageStartX = listPtr.columnInfo[j as usize].pos;
                        let (text, optionalImage1, optionalImage2, optionalImage3) =
                            dc.feederItemText(special, i, j);

                        let text = match text {
                            Some(t) => t,
                            None => continue,
                        };

                        let text = if let Some(stripped) = text.strip_prefix('@') {
                            // PORT-NOTE: `trap_SP_GetStringTextString` -> `dc.SP_GetStringTextString`;
                            // a failed lookup falls back to the empty string rather than Raven's
                            // unset `temp[MAX_STRING_CHARS]` buffer.
                            dc.SP_GetStringTextString(stripped, MAX_STRING_CHARS)
                                .unwrap_or_default()
                        } else {
                            text
                        };

                        // textyOffset stays 0 outside the `_XBOX` arm this port doesn't build.
                        dc.drawText(
                            x + 4.0 + listPtr.columnInfo[j as usize].pos as f32,
                            y + listPtr.elementHeight + textaligny,
                            textscale,
                            foreColor,
                            &text,
                            0.0,
                            listPtr.columnInfo[j as usize].maxChars,
                            textStyle,
                            iMenuFont,
                        );

                        if j < listPtr.numColumns - 1 {
                            imageStartX = listPtr.columnInfo[(j + 1) as usize].pos;
                        }
                        dc.setColor(None);
                        if optionalImage3 >= 0 {
                            dc.drawHandlePic(
                                imageStartX as f32 - listPtr.elementHeight * 3.0,
                                y + listPtr.elementHeight + 2.0,
                                listPtr.elementHeight,
                                listPtr.elementHeight,
                                optionalImage3,
                            );
                        }
                        if optionalImage2 >= 0 {
                            dc.drawHandlePic(
                                imageStartX as f32 - listPtr.elementHeight * 2.0,
                                y + listPtr.elementHeight + 2.0,
                                listPtr.elementHeight,
                                listPtr.elementHeight,
                                optionalImage2,
                            );
                        }
                        if optionalImage1 >= 0 {
                            dc.drawHandlePic(
                                imageStartX as f32 - listPtr.elementHeight,
                                y + listPtr.elementHeight + 2.0,
                                listPtr.elementHeight,
                                listPtr.elementHeight,
                                optionalImage1,
                            );
                        }
                    }
                } else {
                    let (text, optionalImage1, optionalImage2, optionalImage3) =
                        dc.feederItemText(special, i, 0);
                    if optionalImage1 >= 0 || optionalImage2 >= 0 || optionalImage3 >= 0 {
                        // (Raven: commented-out drawHandlePic — no-op)
                    } else if let Some(text) = text {
                        dc.drawText(
                            x + 4.0,
                            y + textaligny,
                            textscale,
                            foreColor,
                            &text,
                            0.0,
                            0,
                            textStyle,
                            iMenuFont,
                        );
                    }
                }

                if i == cursorPos {
                    dc.fillRect(
                        x + 2.0,
                        y + listPtr.elementHeight + 2.0,
                        rect.w - SCROLLBAR_SIZE - 4.0,
                        listPtr.elementHeight,
                        outlineColor,
                    );
                }

                sizeHeight -= listPtr.elementHeight;
                if sizeHeight < listPtr.elementHeight {
                    listPtr.drawPadding = (listPtr.elementHeight - sizeHeight) as c_int;
                    break;
                }
                listPtr.endPos += 1;
                y += listPtr.elementHeight;
                i += 1;
            }
        }
    }

    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
        *l = listPtr;
    }
}

/// Raven `ItemParse_name` — parse an item's window name.
/// Source: `oracle/codemp/ui/ui_shared.c:7380-7385`
pub fn ItemParse_name(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut name = String::new();
    if !PC_String_Parse(dc, handle, &mut name) {
        return false;
    }
    menus.item_mut(item).window.name = Some(name);
    true
}

/// Raven `ItemParse_text` — parse an item's display text.
/// Source: `oracle/codemp/ui/ui_shared.c:7399-7404`
pub fn ItemParse_text(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut text = String::new();
    if !PC_String_Parse(dc, handle, &mut text) {
        return false;
    }
    menus.item_mut(item).text = Some(text);
    true
}

/// Raven `ItemParse_descText` — parse an item's description text.
/// Source: `oracle/codemp/ui/ui_shared.c:7412-7422`
pub fn ItemParse_descText(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut descText = String::new();
    if !PC_String_Parse(dc, handle, &mut descText) {
        return false;
    }
    menus.item_mut(item).descText = descText;
    true
}

/// Raven `ItemParse_text2` — parse an item's second-line display text.
/// Source: `oracle/codemp/ui/ui_shared.c:7431-7441`
pub fn ItemParse_text2(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut text2 = String::new();
    if !PC_String_Parse(dc, handle, &mut text2) {
        return false;
    }
    menus.item_mut(item).text2 = text2;
    true
}

/// Raven `ItemParse_text2alignx` — parse the second-line text's x alignment.
/// Source: `oracle/codemp/ui/ui_shared.c:7448-7455`
pub fn ItemParse_text2alignx(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.item_mut(item).text2alignx)
}

/// Raven `ItemParse_text2aligny` — parse the second-line text's y alignment.
/// Source: `oracle/codemp/ui/ui_shared.c:7462-7469`
pub fn ItemParse_text2aligny(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.item_mut(item).text2aligny)
}

/// Raven `ItemParse_group` — parse an item's group name.
/// Source: `oracle/codemp/ui/ui_shared.c:7472-7477`
pub fn ItemParse_group(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut group = String::new();
    if !PC_String_Parse(dc, handle, &mut group) {
        return false;
    }
    menus.item_mut(item).window.group = Some(group);
    true
}

/// Raven `ItemParse_asset_model` — parse an item's model asset path, with the
/// `ui_char_model` name a special-cased indirection through that cvar.
///
/// PORT-NOTE: the `#ifndef CGAME` (ui) arm, per this file's convention.
/// Source: `oracle/codemp/ui/ui_shared.c:7659-7684`
pub fn ItemParse_asset_model(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let mut temp = pc_token_str(&token);

    if stricmp_eq(&temp, "ui_char_model") {
        let ui_char_model = dc.getCVarString("ui_char_model", MAX_QPATH as usize);
        temp = format!("models/players/{}/model.glm", ui_char_model);
    }

    let mut animRunLength: c_int = 0;
    ItemParse_asset_model_go(menus, dc, item, &temp, &mut animRunLength)
}

/// Raven `ItemParse_model_origin` — parse an item model's origin.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7697-7710`
pub fn ItemParse_model_origin(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };

    if PC_Float_Parse(dc, handle, &mut modelPtr.origin[0]) {
        if PC_Float_Parse(dc, handle, &mut modelPtr.origin[1]) {
            if PC_Float_Parse(dc, handle, &mut modelPtr.origin[2]) {
                return true;
            }
        }
    }
    false
}

/// Raven `ItemParse_model_fovx` — parse an item model's horizontal fov.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7713-7722`
pub fn ItemParse_model_fovx(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };
    PC_Float_Parse(dc, handle, &mut modelPtr.fov_x)
}

/// Raven `ItemParse_model_fovy` — parse an item model's vertical fov.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7725-7734`
pub fn ItemParse_model_fovy(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };
    PC_Float_Parse(dc, handle, &mut modelPtr.fov_y)
}

/// Raven `ItemParse_model_rotation` — parse an item model's rotation speed.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7737-7746`
pub fn ItemParse_model_rotation(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };
    PC_Int_Parse(dc, handle, &mut modelPtr.rotationSpeed)
}

/// Raven `ItemParse_model_angle` — parse an item model's static angle.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7749-7758`
pub fn ItemParse_model_angle(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };
    PC_Int_Parse(dc, handle, &mut modelPtr.angle)
}

/// Raven `ItemParse_model_g2mins` — parse an item model's ghoul2 mins.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7761-7774`
pub fn ItemParse_model_g2mins(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };

    if PC_Float_Parse(dc, handle, &mut modelPtr.g2mins[0]) {
        if PC_Float_Parse(dc, handle, &mut modelPtr.g2mins[1]) {
            if PC_Float_Parse(dc, handle, &mut modelPtr.g2mins[2]) {
                return true;
            }
        }
    }
    false
}

/// Raven `ItemParse_model_g2maxs` — parse an item model's ghoul2 maxs.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7777-7790`
pub fn ItemParse_model_g2maxs(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };

    if PC_Float_Parse(dc, handle, &mut modelPtr.g2maxs[0]) {
        if PC_Float_Parse(dc, handle, &mut modelPtr.g2maxs[1]) {
            if PC_Float_Parse(dc, handle, &mut modelPtr.g2maxs[2]) {
                return true;
            }
        }
    }
    false
}

/// Raven `ItemParse_model_g2scale` — parse an item model's ghoul2 scale.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) fails the parse instead of Raven's
/// null-deref UB.
/// Source: `oracle/codemp/ui/ui_shared.c:7793-7806`
pub fn ItemParse_model_g2scale(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let modelPtr = match menus.item_mut(item).typeData.model_mut() {
        Some(m) => m,
        None => return false,
    };

    if PC_Float_Parse(dc, handle, &mut modelPtr.g2scale[0]) {
        if PC_Float_Parse(dc, handle, &mut modelPtr.g2scale[1]) {
            if PC_Float_Parse(dc, handle, &mut modelPtr.g2scale[2]) {
                return true;
            }
        }
    }
    false
}

/// Raven `ItemParse_rectcvar` — read a cvar name token, then read the item's
/// `window.rectClient` (x/y/w/h) out of a space-separated cvar string.
///
/// Raven's trailing comment: "There may be no cvar built for this, and
/// that's okay. . . I guess." — a partial/missing cvar string leaves the rect
/// fields at whatever they were and still returns success.
/// Source: `oracle/codemp/ui/ui_shared.c:7922-7959`
pub fn ItemParse_rectcvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    // get Cvar name
    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let name = pc_token_str(&token);

    // get cvar data
    let cvarBuf = dc.getCVarString(&name, 1024);

    let mut holdBuf: &str = &cvarBuf;
    let mut holdVal = String::new();
    if String_Parse(&mut holdBuf, &mut holdVal) {
        menus.item_mut(item).window.rectClient.x = atof(&holdVal) as f32;
        if String_Parse(&mut holdBuf, &mut holdVal) {
            menus.item_mut(item).window.rectClient.y = atof(&holdVal) as f32;
            if String_Parse(&mut holdBuf, &mut holdVal) {
                menus.item_mut(item).window.rectClient.w = atof(&holdVal) as f32;
                if String_Parse(&mut holdBuf, &mut holdVal) {
                    menus.item_mut(item).window.rectClient.h = atof(&holdVal) as f32;
                    return true;
                }
            }
        }
    }

    // There may be no cvar built for this, and that's okay. . . I guess.
    true
}

/// Raven `ItemParse_style` — parse the item's window style.
/// Source: `oracle/codemp/ui/ui_shared.c:8010-8019`
pub fn ItemParse_style(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).window.style) {
        dc.Print("^3Unknown item style value");
        return false;
    }
    true
}

/// Raven `ItemParse_type` — parse the item type, then re-derive its
/// `typeData` payload from it.
/// Source: `oracle/codemp/ui/ui_shared.c:8087-8097`
pub fn ItemParse_type(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).r#type) {
        return false;
    }
    Item_ValidateTypeData(menus.item_mut(item));
    true
}

/// Raven `ItemParse_elementwidth` — parse a listbox item's element width.
///
/// PORT-NOTE (§19 UB pick): Raven casts `typeData` unconditionally after
/// `Item_ValidateTypeData` without a NULL check; a payload-type mismatch
/// (unreachable under this file's own type dispatch) fails the parse instead
/// of Raven's null-deref UB, matching the `ItemParse_model_g2scale` pick.
/// Source: `oracle/codemp/ui/ui_shared.c:8101-8110`
pub fn ItemParse_elementwidth(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let listPtr = match menus.item_mut(item).typeData.listBox_mut() {
        Some(l) => l,
        None => return false,
    };
    if !PC_Float_Parse(dc, handle, &mut listPtr.elementWidth) {
        return false;
    }
    true
}

/// Raven `ItemParse_elementheight` — parse a listbox item's element height.
///
/// PORT-NOTE (§19 UB pick): same unconditional-cast pick as
/// `ItemParse_elementwidth`.
/// Source: `oracle/codemp/ui/ui_shared.c:8114-8123`
pub fn ItemParse_elementheight(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));

    let listPtr = match menus.item_mut(item).typeData.listBox_mut() {
        Some(l) => l,
        None => return false,
    };
    if !PC_Float_Parse(dc, handle, &mut listPtr.elementHeight) {
        return false;
    }
    true
}

/// Raven `ItemParse_feeder` — parse the item's feeder id into `special`.
/// Source: `oracle/codemp/ui/ui_shared.c:8126-8131`
pub fn ItemParse_feeder(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).special) {
        return false;
    }
    true
}

/// Raven `ItemParse_elementtype` — parse a listbox item's element style.
/// Source: `oracle/codemp/ui/ui_shared.c:8135-8146`
pub fn ItemParse_elementtype(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }
    let listPtr = match menus.item_mut(item).typeData.listBox_mut() {
        Some(l) => l,
        None => return false,
    };
    if !PC_Int_Parse(dc, handle, &mut listPtr.elementStyle) {
        return false;
    }
    true
}

/// Raven `ItemParse_columns` — parse a listbox item's column count (capped
/// at `MAX_LB_COLUMNS`) and each column's pos/width/maxChars triple.
/// Source: `oracle/codemp/ui/ui_shared.c:8149-8177`
pub fn ItemParse_columns(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut num: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut num) {
        return false;
    }
    if num > MAX_LB_COLUMNS as c_int {
        num = MAX_LB_COLUMNS as c_int;
    }

    let listPtr = match menus.item_mut(item).typeData.listBox_mut() {
        Some(l) => l,
        None => return false,
    };
    listPtr.numColumns = num;
    for i in 0..num {
        let mut pos: c_int = 0;
        let mut width: c_int = 0;
        let mut maxChars: c_int = 0;
        if PC_Int_Parse(dc, handle, &mut pos)
            && PC_Int_Parse(dc, handle, &mut width)
            && PC_Int_Parse(dc, handle, &mut maxChars)
        {
            listPtr.columnInfo[i as usize].pos = pos;
            listPtr.columnInfo[i as usize].width = width;
            listPtr.columnInfo[i as usize].maxChars = maxChars;
        } else {
            return false;
        }
    }
    true
}

/// Raven `ItemParse_border` — parse the item's window border style.
/// Source: `oracle/codemp/ui/ui_shared.c:8179-8184`
pub fn ItemParse_border(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).window.border) {
        return false;
    }
    true
}

/// Raven `ItemParse_bordersize` — parse the item's window border size.
/// Source: `oracle/codemp/ui/ui_shared.c:8186-8191`
pub fn ItemParse_bordersize(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).window.borderSize) {
        return false;
    }
    true
}

/// Raven `ItemParse_visible` — set `WINDOW_VISIBLE` when the parsed value is
/// non-zero (never clears it).
/// Source: `oracle/codemp/ui/ui_shared.c:8193-8203`
pub fn ItemParse_visible(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut i) {
        return false;
    }
    if i != 0 {
        menus.item_mut(item).window.flags |= WINDOW_VISIBLE;
    }
    true
}

/// Raven `ItemParse_ownerdraw` — parse the item's ownerdraw id and force the
/// item type to `ITEM_TYPE_OWNERDRAW`.
/// Source: `oracle/codemp/ui/ui_shared.c:8205-8211`
pub fn ItemParse_ownerdraw(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).window.ownerDraw) {
        return false;
    }
    menus.item_mut(item).r#type = ITEM_TYPE_OWNERDRAW;
    true
}

/// Raven `ItemParse_align` — parse the item's alignment.
/// Source: `oracle/codemp/ui/ui_shared.c:8213-8218`
pub fn ItemParse_align(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).alignment) {
        return false;
    }
    true
}

/// Raven `ItemParse_isCharacter` — set/clear `ITF_ISCHARACTER` from the
/// parsed flag value.
/// Source: `oracle/codemp/ui/ui_shared.c:8227-8244`
pub fn ItemParse_isCharacter(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut flag: c_int = 0;
    if PC_Int_Parse(dc, handle, &mut flag) {
        if flag != 0 {
            menus.item_mut(item).flags |= ITF_ISCHARACTER;
        } else {
            menus.item_mut(item).flags &= !ITF_ISCHARACTER;
        }
        return true;
    }
    false
}

/// Raven `ItemParse_textalign` — parse the item's text alignment.
/// Source: `oracle/codemp/ui/ui_shared.c:8252-8262`
pub fn ItemParse_textalign(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).textalignment) {
        dc.Print("^3Unknown text alignment value");
        return false;
    }
    true
}

/// Raven `ItemParse_textalignx` — parse the item's text alignment x offset.
/// Source: `oracle/codemp/ui/ui_shared.c:8264-8269`
pub fn ItemParse_textalignx(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).textalignx) {
        return false;
    }
    true
}

/// Raven `ItemParse_textaligny` — parse the item's text alignment y offset.
/// Source: `oracle/codemp/ui/ui_shared.c:8271-8276`
pub fn ItemParse_textaligny(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).textaligny) {
        return false;
    }
    true
}

/// Raven `ItemParse_textscale` — parse the item's text scale.
/// Source: `oracle/codemp/ui/ui_shared.c:8278-8283`
pub fn ItemParse_textscale(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).textscale) {
        return false;
    }
    true
}

/// Raven `ItemParse_textstyle` — parse the item's text style.
/// Source: `oracle/codemp/ui/ui_shared.c:8285-8290`
pub fn ItemParse_textstyle(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).textStyle) {
        return false;
    }
    true
}

/// Raven `ItemParse_invertyesno` — parse the item's invert-yes/no flag.
/// Source: `oracle/codemp/ui/ui_shared.c:8298-8305`
pub fn ItemParse_invertyesno(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.item_mut(item).invertYesNo) {
        return false;
    }
    true
}

/// Raven `ItemParse_xoffset` — parse the item's x offset.
///
/// PORT-NOTE: faithful transcription of an oracle bug — `PC_Int_Parse`
/// *succeeding* returns `qfalse`, and failing falls through to `qtrue`
/// (porting-rules §A2: port ugly behavior faithfully, get green, refactor
/// behind the passing diff later).
/// Source: `oracle/codemp/ui/ui_shared.c:8312-8319`
pub fn ItemParse_xoffset(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if PC_Int_Parse(dc, handle, &mut menus.item_mut(item).xoffset) {
        return false;
    }
    true
}

/// Raven `ItemParse_backcolor` — parse the item's window back color (4
/// floats).
/// Source: `oracle/codemp/ui/ui_shared.c:8322-8333`
pub fn ItemParse_backcolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f: f32 = 0.0;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.item_mut(item).window.backColor[i] = f;
    }
    true
}

/// Raven `ItemParse_forecolor` — parse the item's window fore color (4
/// floats); a negative component is the player-color special case
/// (`WINDOW_PLAYERCOLOR`) and stops the parse early without an error.
/// Source: `oracle/codemp/ui/ui_shared.c:8335-8354`
pub fn ItemParse_forecolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f: f32 = 0.0;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }

        if f < 0.0 {
            // special case for player color
            menus.item_mut(item).window.flags |= WINDOW_PLAYERCOLOR;
            return true;
        }

        let it = menus.item_mut(item);
        it.window.foreColor[i] = f;
        it.window.flags |= WINDOW_FORECOLORSET;
    }
    true
}

/// Raven `ItemParse_bordercolor` — parse the item's window border color (4
/// floats).
/// Source: `oracle/codemp/ui/ui_shared.c:8356-8367`
pub fn ItemParse_bordercolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f: f32 = 0.0;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.item_mut(item).window.borderColor[i] = f;
    }
    true
}

/// Raven `ItemParse_cinematic` — parse the item's cinematic name.
/// Source: `oracle/codemp/ui/ui_shared.c:8386-8391`
pub fn ItemParse_cinematic(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_String_Parse(dc, handle, &mut menus.item_mut(item).window.cinematicName) {
        return false;
    }
    true
}

/// Raven `ItemParse_doubleClick` — parse a listbox item's double-click
/// script.
/// Source: `oracle/codemp/ui/ui_shared.c:8393-8407`
pub fn ItemParse_doubleClick(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let listPtr = match menus.item_mut(item).typeData.listBox_mut() {
        Some(l) => l,
        None => return false,
    };
    if !PC_Script_Parse(dc, handle, &mut listPtr.doubleClick) {
        return false;
    }
    true
}

/// Raven `ItemParse_onFocus` — parse the item's on-focus script.
/// Source: `oracle/codemp/ui/ui_shared.c:8409-8414`
pub fn ItemParse_onFocus(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).onFocus) {
        return false;
    }
    true
}

/// Raven `ItemParse_leaveFocus` — parse the item's leave-focus script.
/// Source: `oracle/codemp/ui/ui_shared.c:8416-8421`
pub fn ItemParse_leaveFocus(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).leaveFocus) {
        return false;
    }
    true
}

/// Raven `ItemParse_mouseEnter` — parse the item's mouse-enter script.
/// Source: `oracle/codemp/ui/ui_shared.c:8423-8428`
pub fn ItemParse_mouseEnter(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).mouseEnter) {
        return false;
    }
    true
}

/// Raven `ItemParse_mouseExit` — parse the item's mouse-exit script.
/// Source: `oracle/codemp/ui/ui_shared.c:8430-8435`
pub fn ItemParse_mouseExit(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).mouseExit) {
        return false;
    }
    true
}

/// Raven `ItemParse_mouseEnterText` — parse the item's mouse-enter-text
/// script.
/// Source: `oracle/codemp/ui/ui_shared.c:8437-8442`
pub fn ItemParse_mouseEnterText(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).mouseEnterText) {
        return false;
    }
    true
}

/// Raven `ItemParse_mouseExitText` — parse the item's mouse-exit-text
/// script.
/// Source: `oracle/codemp/ui/ui_shared.c:8444-8449`
pub fn ItemParse_mouseExitText(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).mouseExitText) {
        return false;
    }
    true
}

/// Raven `ItemParse_action` — parse the item's select (action) script.
/// Source: `oracle/codemp/ui/ui_shared.c:8451-8456`
pub fn ItemParse_action(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Script_Parse(dc, handle, &mut menus.item_mut(item).action) {
        return false;
    }
    true
}

/// Raven `ItemParse_special` — parse the item's `special` value.
/// Source: `oracle/codemp/ui/ui_shared.c:8458-8463`
pub fn ItemParse_special(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_Float_Parse(dc, handle, &mut menus.item_mut(item).special) {
        return false;
    }
    true
}

/// Raven `ItemParse_cvarTest` — parse the item's enable-test cvar name.
/// Source: `oracle/codemp/ui/ui_shared.c:8465-8470`
pub fn ItemParse_cvarTest(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if !PC_String_Parse(dc, handle, &mut menus.item_mut(item).cvarTest) {
        return false;
    }
    true
}

/// Raven `ItemParse_cvar` — parse an item's associated cvar name, resetting
/// its edit-field limits to Raven's `-1`/`-1`/`-1` sentinel when it has one.
///
/// PORT-NOTE (§19 UB pick): Raven's unconditional `editFieldDef_t*` cast
/// type-puns a non-edit-field payload's memory (ui_shared.c:8484-8497); a
/// payload-type mismatch drops the write here instead.
/// Source: `oracle/codemp/ui/ui_shared.c:8472-8500`
pub fn ItemParse_cvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    let mut cvar = String::new();
    if !PC_String_Parse(dc, handle, &mut cvar) {
        return false;
    }
    menus.item_mut(item).cvar = Some(cvar);

    let it = menus.item_mut(item);
    if !it.typeData.is_none() {
        match it.r#type {
            ITEM_TYPE_EDITFIELD
            | ITEM_TYPE_NUMERICFIELD
            | ITEM_TYPE_YESNO
            | ITEM_TYPE_BIND
            | ITEM_TYPE_SLIDER
            | ITEM_TYPE_TEXT => {
                if let Some(editPtr) = it.typeData.editField_mut() {
                    editPtr.minVal = -1.0;
                    editPtr.maxVal = -1.0;
                    editPtr.defVal = -1.0;
                }
            }
            _ => {}
        }
    }
    true
}

/// Raven `ItemParse_font` — parse an item's font index.
/// Source: `oracle/codemp/ui/ui_shared.c:8502-8510`
pub fn ItemParse_font(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    PC_Int_Parse(dc, handle, &mut menus.item_mut(item).iMenuFont)
}

/// Raven `ItemParse_maxChars` — parse an edit field's max character count.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) drops the write instead of Raven's
/// unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:8513-8527`
pub fn ItemParse_maxChars(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut maxChars: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut maxChars) {
        return false;
    }
    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
        editPtr.maxChars = maxChars;
    }
    true
}

/// Raven `ItemParse_maxPaintChars` — parse an edit field's max painted
/// character count.
///
/// PORT-NOTE (§19 UB pick): see `ItemParse_maxChars`.
/// Source: `oracle/codemp/ui/ui_shared.c:8529-8543`
pub fn ItemParse_maxPaintChars(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut maxChars: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut maxChars) {
        return false;
    }
    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
        editPtr.maxPaintChars = maxChars;
    }
    true
}

/// Raven `ItemParse_maxLineChars` — parse a text-scroll box's max characters
/// per line.
///
/// PORT-NOTE (§19 UB pick): a `typeData` payload-type mismatch (unreachable
/// under this file's own type dispatch) drops the write instead of Raven's
/// unconditional cast.
/// Source: `oracle/codemp/ui/ui_shared.c:8545-8563`
pub fn ItemParse_maxLineChars(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut maxChars: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut maxChars) {
        return false;
    }
    if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
        scrollPtr.maxLineChars = maxChars;
    }
    true
}

/// Raven `ItemParse_lineHeight` — parse a text-scroll box's line height.
///
/// PORT-NOTE (§19 UB pick): see `ItemParse_maxLineChars`.
/// Source: `oracle/codemp/ui/ui_shared.c:8565-8583`
pub fn ItemParse_lineHeight(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut height: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut height) {
        return false;
    }
    if let Some(scrollPtr) = menus.item_mut(item).typeData.textScroll_mut() {
        scrollPtr.lineHeight = height as f32;
    }
    true
}

/// Raven `ItemParse_cvarFloat` — parse a numeric edit field's cvar and its
/// default/min/max range, in one four-token chain.
///
/// PORT-NOTE (§19 UB pick): the `typeData` cast is unconditional in Raven
/// (the `editPtr` local is assigned once, before any parse); each parsed
/// value is written the moment its own `PC_*_Parse` succeeds (matching
/// Raven's inline out-param writes inside the `&&` chain — a later parse
/// failing still leaves the earlier writes applied), with a payload-type
/// mismatch dropping the edit-field write instead of Raven's unconditional
/// cast.
/// Source: `oracle/codemp/ui/ui_shared.c:8585-8599`
pub fn ItemParse_cvarFloat(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }

    let mut cvar = String::new();
    if !PC_String_Parse(dc, handle, &mut cvar) {
        return false;
    }
    menus.item_mut(item).cvar = Some(cvar);

    let mut defVal = 0.0f32;
    if !PC_Float_Parse(dc, handle, &mut defVal) {
        return false;
    }
    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
        editPtr.defVal = defVal;
    }

    let mut minVal = 0.0f32;
    if !PC_Float_Parse(dc, handle, &mut minVal) {
        return false;
    }
    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
        editPtr.minVal = minVal;
    }

    let mut maxVal = 0.0f32;
    if !PC_Float_Parse(dc, handle, &mut maxVal) {
        return false;
    }
    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
        editPtr.maxVal = maxVal;
    }

    true
}

/// Raven `ItemParse_cvarStrList` — parse a multi-value item's string cycle
/// list (`{ "label" "cvarvalue" ... }`), or special-case the `"feeder"`
/// keyword for the player-species/language pickers.
///
/// PORT-NOTE: `MultiDef`'s three parallel `[MAX_MULTI_CVARS]` arrays are
/// owned `Vec`s (see the type's own doc); `multiPtr->count = 0` is the vecs'
/// `clear()`. Raven's `(int)psString > 0` pointer-validity check (always
/// true once `PC_String_Parse` reports success) collapses.
///
/// DEFERRED: the `feeder == FEEDER_PLAYER_SPECIES`/`FEEDER_LANGUAGES`
/// branches populate the cycle list from `uiInfo.playerSpecies`/
/// `uiInfo.languageCount` (and the `currLanguage`/`languageString` file
/// statics) — `uiInfo` lives on `crates/mp/ui/src/world/ui_world.rs`'s
/// `UiWorld` (mp_ui-only), unreachable from this host-agnostic crate (no
/// `mp_ui` dependency, no `UiWorld`/`UiContext` in scope, and no
/// `DisplayContext`/`MenuSystem` field carries it). The `"feeder"` token
/// check and unconditional `return true` are transcribed; the population
/// loops are not — escalated, needs a host callback (e.g. a `DisplayContext`
/// method) to thread the species/language table through.
/// Source: `oracle/codemp/ui/ui_shared.c:8604-8694`
pub fn ItemParse_cvarStrList(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }
    if let Some(multiPtr) = menus.item_mut(item).typeData.multi_mut() {
        multiPtr.cvarList.clear();
        multiPtr.cvarStr.clear();
        multiPtr.strDef = true;
    }

    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let tokenStr = pc_token_str(&token);
    let special = menus.item(item).special;

    if stricmp_eq(&tokenStr, "feeder") && special == FEEDER_PLAYER_SPECIES as f32 {
        // DEFERRED: uiInfo.playerSpecies population — see fn doc.
        return true;
    }
    // languages
    if stricmp_eq(&tokenStr, "feeder") && special == FEEDER_LANGUAGES as f32 {
        // DEFERRED: uiInfo.languageCount population — see fn doc.
        return true;
    }

    if !tokenStr.starts_with('{') {
        return false;
    }

    let mut pass = 0;
    loop {
        let mut psString = String::new();
        if !PC_String_Parse(dc, handle, &mut psString) {
            PC_SourceError(dc, handle, "end of file inside menu item\n");
            return false;
        }

        if !psString.is_empty() {
            if psString.starts_with('}') {
                return true;
            }
            if psString.starts_with(',') || psString.starts_with(';') {
                continue;
            }
        }

        if let Some(multiPtr) = menus.item_mut(item).typeData.multi_mut() {
            if pass == 0 {
                multiPtr.cvarList.push(psString);
                pass = 1;
            } else {
                multiPtr.cvarStr.push(psString);
                pass = 0;
                if multiPtr.cvarList.len() >= MAX_MULTI_CVARS {
                    return false;
                }
            }
        }
    }
}

/// Raven `ItemParse_cvarFloatList` — parse a multi-value item's numeric
/// cycle list (`{ "label" value ... }`).
///
/// PORT-NOTE: see `ItemParse_cvarStrList` — `count = 0` is the vecs'
/// `clear()`; the `(int)string > 0` pointer-validity check collapses.
/// Source: `oracle/codemp/ui/ui_shared.c:8696-8759`
pub fn ItemParse_cvarFloatList(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    Item_ValidateTypeData(menus.item_mut(item));
    if menus.item(item).typeData.is_none() {
        return false;
    }
    if let Some(multiPtr) = menus.item_mut(item).typeData.multi_mut() {
        multiPtr.cvarList.clear();
        multiPtr.cvarValue.clear();
        multiPtr.strDef = false;
    }

    let mut token = zero_pc_token();
    if !dc.PC_ReadToken(handle, &mut token) {
        return false;
    }
    let tokenStr = pc_token_str(&token);
    if !tokenStr.starts_with('{') {
        return false;
    }

    loop {
        let mut string = String::new();
        if !PC_String_Parse(dc, handle, &mut string) {
            PC_SourceError(dc, handle, "end of file inside menu item\n");
            return false;
        }

        if !string.is_empty() {
            if string.starts_with('}') {
                return true;
            }
            if string.starts_with(',') || string.starts_with(';') {
                continue;
            }
        }

        // Raven writes `cvarList[count]` before the parse but only advances `count`
        // on success, so a failed parse leaves the pair uncommitted.
        let mut value = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut value) {
            return false;
        }

        if let Some(multiPtr) = menus.item_mut(item).typeData.multi_mut() {
            multiPtr.cvarList.push(string);
            multiPtr.cvarValue.push(value);
            if multiPtr.cvarList.len() >= MAX_MULTI_CVARS {
                return false;
            }
        }
    }
}

/// Raven `ItemParse_ownerdrawFlag` — OR an ownerdraw show-flag into the
/// item's window.
/// Source: `oracle/codemp/ui/ui_shared.c:8778-8785`
pub fn ItemParse_ownerdrawFlag(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut i) {
        return false;
    }
    menus.item_mut(item).window.ownerDrawFlags |= i;
    true
}

/// Raven `ItemParse_enableCvar` — parse the item's enable-cvar script and
/// mark it `CVAR_ENABLE`.
/// Source: `oracle/codemp/ui/ui_shared.c:8787-8793`
pub fn ItemParse_enableCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if PC_Script_Parse(dc, handle, &mut menus.item_mut(item).enableCvar) {
        menus.item_mut(item).cvarFlags = CVAR_ENABLE;
        return true;
    }
    false
}

/// Raven `ItemParse_disableCvar` — parse the item's enable-cvar script and
/// mark it `CVAR_DISABLE`.
/// Source: `oracle/codemp/ui/ui_shared.c:8795-8801`
pub fn ItemParse_disableCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if PC_Script_Parse(dc, handle, &mut menus.item_mut(item).enableCvar) {
        menus.item_mut(item).cvarFlags = CVAR_DISABLE;
        return true;
    }
    false
}

/// Raven `ItemParse_showCvar` — parse the item's enable-cvar script and mark
/// it `CVAR_SHOW`.
/// Source: `oracle/codemp/ui/ui_shared.c:8803-8809`
pub fn ItemParse_showCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if PC_Script_Parse(dc, handle, &mut menus.item_mut(item).enableCvar) {
        menus.item_mut(item).cvarFlags = CVAR_SHOW;
        return true;
    }
    false
}

/// Raven `ItemParse_hideCvar` — parse the item's enable-cvar script and mark
/// it `CVAR_HIDE`.
/// Source: `oracle/codemp/ui/ui_shared.c:8811-8817`
pub fn ItemParse_hideCvar(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    if PC_Script_Parse(dc, handle, &mut menus.item_mut(item).enableCvar) {
        menus.item_mut(item).cvarFlags = CVAR_HIDE;
        return true;
    }
    false
}

/// Raven `ItemParse_Appearance_slot` — parse the item's appearance-order
/// slot.
/// Source: `oracle/codemp/ui/ui_shared.c:8824-8831`
pub fn ItemParse_Appearance_slot(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.item_mut(item).appearanceSlot)
}

/// Raven `ItemParse_isSaber` — mark/unmark an item as drawing the first
/// saber blade.
///
/// PORT-NOTE: Raven's `#ifndef CGAME` guard restricts this whole body to the
/// ui host at compile time; `mp_uishared` carries no compile-time host
/// distinction (DEC-36 D3 threads the host through `dc`/state params at
/// runtime, not `#ifdef`), so the flag toggle below runs for every host.
///
/// DEFERRED: the saber-glow-cache/parms-load call
/// (`UI_CacheSaberGlowGraphics`, `UI_SaberLoadParms`, gated on
/// `ui_saber_parms_parsed`) — those are ported in `crates/mp/ui/src/
/// ui_saber.rs` taking `ctx: &mut UiContext`, and `ui_saber_parms_parsed`
/// lives on `ctx.world.saber` (`crates/mp/ui/src/world/ui_saber_state.rs`);
/// `mp_uishared` is host-agnostic (no `mp_ui` dependency, no `UiContext` in
/// scope), so neither is reachable from this fn. Escalated — needs a host
/// callback (e.g. a `DisplayContext` method) to thread this through.
/// Source: `oracle/codemp/ui/ui_shared.c:8833-8859`
pub fn ItemParse_isSaber(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if PC_Int_Parse(dc, handle, &mut i) {
        if i != 0 {
            menus.item_mut(item).flags |= ITF_ISSABER;
        } else {
            menus.item_mut(item).flags &= !ITF_ISSABER;
        }
        return true;
    }
    false
}

/// Raven `ItemParse_isSaber2` — mark/unmark an item as drawing the second
/// saber blade.
///
/// PORT-NOTE: see `ItemParse_isSaber`.
///
/// DEFERRED: see `ItemParse_isSaber`.
/// Source: `oracle/codemp/ui/ui_shared.c:8865-8890`
pub fn ItemParse_isSaber2(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if PC_Int_Parse(dc, handle, &mut i) {
        if i != 0 {
            menus.item_mut(item).flags |= ITF_ISSABER2;
        } else {
            menus.item_mut(item).flags &= !ITF_ISSABER2;
        }
        return true;
    }
    false
}

/// Raven `MenuParse_font` — parse a menu's font, registering the medium font
/// asset the first time any menu sets one.
///
/// PORT-NOTE: Raven's `itemDef_t *item` parameter is immediately cast to
/// `menuDef_t *menu` (this file's shared `MenuParse_*` callback signature);
/// this takes the `MenuId` the cast resolves to directly. `DC->Assets.*`
/// (the `DC->` data tail) becomes `ds.Assets.*`; the commented-out
/// `DC->registerFont` call stays dropped (dead even in the oracle).
/// Source: `oracle/codemp/ui/ui_shared.c:9291-9302`
pub fn MenuParse_font(
    menus: &mut MenuSystem,
    ds: &mut DisplayState,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut font = String::new();
    if !PC_String_Parse(dc, handle, &mut font) {
        return false;
    }
    if !ds.Assets.fontRegistered {
        ds.Assets.qhMediumFont = dc.RegisterFont(&font);
        ds.Assets.fontRegistered = true;
    }
    menus.menu_mut(menu).font = font;
    true
}

/// Raven `MenuParse_name` — parse a menu's window name.
///
/// PORT-NOTE: see `MenuParse_font` — the `itemDef_t *` cast to `menuDef_t *`
/// becomes a direct `MenuId`. Raven's `"main"` name check has a
/// commented-out body (`WINDOW_HASFOCUS`) and is dead even in the oracle.
/// Source: `oracle/codemp/ui/ui_shared.c:9304-9314`
pub fn MenuParse_name(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut name = String::new();
    if !PC_String_Parse(dc, handle, &mut name) {
        return false;
    }
    menus.menu_mut(menu).window.name = Some(name);
    true
}

/// Raven `MenuParse_fullscreen` — parse whether a menu covers the entire
/// screen.
///
/// PORT-NOTE: see `MenuParse_font` — the `itemDef_t *` cast to `menuDef_t *`
/// becomes a direct `MenuId`.
/// Source: `oracle/codemp/ui/ui_shared.c:9316-9322`
pub fn MenuParse_fullscreen(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut v: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut v) {
        return false;
    }
    menus.menu_mut(menu).fullScreen = v != 0;
    true
}

/// Raven `MenuParse_style` — parse a menu's window style.
///
/// PORT-NOTE: `Com_Printf` is unreachable from this host-agnostic crate (see
/// `String_Report`) — routed through `dc.Print`.
/// Source: `oracle/codemp/ui/ui_shared.c:9337-9348`
pub fn MenuParse_style(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).window.style) {
        dc.Print("^3Unknown menu style value");
        return false;
    }
    true
}

/// Raven `MenuParse_visible` — set `WINDOW_VISIBLE` on a menu if its parsed
/// flag is nonzero.
/// Source: `oracle/codemp/ui/ui_shared.c:9350-9361`
pub fn MenuParse_visible(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut i) {
        return false;
    }
    if i != 0 {
        menus.menu_mut(menu).window.flags |= WINDOW_VISIBLE;
    }
    true
}

/// Raven `MenuParse_onOpen` — parse a menu's on-open script.
/// Source: `oracle/codemp/ui/ui_shared.c:9363-9369`
pub fn MenuParse_onOpen(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Script_Parse(dc, handle, &mut menus.menu_mut(menu).onOpen)
}

/// Raven `MenuParse_onClose` — parse a menu's on-close script.
/// Source: `oracle/codemp/ui/ui_shared.c:9371-9377`
pub fn MenuParse_onClose(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Script_Parse(dc, handle, &mut menus.menu_mut(menu).onClose)
}

/// Raven `MenuParse_onAccept` — parse a menu's on-accept script.
/// Source: `oracle/codemp/ui/ui_shared.c:9385-9394`
pub fn MenuParse_onAccept(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Script_Parse(dc, handle, &mut menus.menu_mut(menu).onAccept)
}

/// Raven `MenuParse_onESC` — parse a menu's on-escape script.
/// Source: `oracle/codemp/ui/ui_shared.c:9396-9402`
pub fn MenuParse_onESC(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Script_Parse(dc, handle, &mut menus.menu_mut(menu).onESC)
}

/// Raven `MenuParse_border` — parse a menu window's border style.
/// Source: `oracle/codemp/ui/ui_shared.c:9406-9412`
pub fn MenuParse_border(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).window.border)
}

/// Raven `MenuParse_borderSize` — parse a menu window's border size.
/// Source: `oracle/codemp/ui/ui_shared.c:9414-9420`
pub fn MenuParse_borderSize(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.menu_mut(menu).window.borderSize)
}

/// Raven `MenuParse_backcolor` — parse a menu window's back color (4
/// floats).
/// Source: `oracle/codemp/ui/ui_shared.c:9422-9434`
pub fn MenuParse_backcolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.menu_mut(menu).window.backColor[i] = f;
    }
    true
}

/// Raven `MenuParse_descAlignment` — parse a menu's description-text
/// alignment.
///
/// PORT-NOTE: `Com_Printf` is unreachable from this host-agnostic crate (see
/// `String_Report`) — routed through `dc.Print`.
/// Source: `oracle/codemp/ui/ui_shared.c:9441-9452`
pub fn MenuParse_descAlignment(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    if !PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).descAlignment) {
        dc.Print("^3Unknown desc alignment value");
        return false;
    }
    true
}

/// Raven `MenuParse_descX` — parse a menu's description-text x position.
/// Source: `oracle/codemp/ui/ui_shared.c:9459-9468`
pub fn MenuParse_descX(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).descX)
}

/// Raven `MenuParse_descY` — parse a menu's description-text y position.
/// Source: `oracle/codemp/ui/ui_shared.c:9475-9484`
pub fn MenuParse_descY(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).descY)
}

/// Raven `MenuParse_descScale` — parse a menu's description-text scale.
/// Source: `oracle/codemp/ui/ui_shared.c:9491-9500`
pub fn MenuParse_descScale(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.menu_mut(menu).descScale)
}

/// Raven `MenuParse_descColor` — parse a menu's description-text color (4
/// floats).
/// Source: `oracle/codemp/ui/ui_shared.c:9507-9522`
pub fn MenuParse_descColor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.menu_mut(menu).descColor[i] = f;
    }
    true
}

/// Raven `MenuParse_forecolor` — parse a menu window's fore color (4
/// floats), special-casing a negative component as the player-color flag.
/// Source: `oracle/codemp/ui/ui_shared.c:9524-9542`
pub fn MenuParse_forecolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        if f < 0.0 {
            // special case for player color
            menus.menu_mut(menu).window.flags |= WINDOW_PLAYERCOLOR;
            return true;
        }
        menus.menu_mut(menu).window.foreColor[i] = f;
        menus.menu_mut(menu).window.flags |= WINDOW_FORECOLORSET;
    }
    true
}

/// Raven `MenuParse_bordercolor` — parse a menu window's border color (4
/// floats).
/// Source: `oracle/codemp/ui/ui_shared.c:9544-9556`
pub fn MenuParse_bordercolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.menu_mut(menu).window.borderColor[i] = f;
    }
    true
}

/// Raven `MenuParse_focuscolor` — parse a menu's focus color (4 floats).
/// Source: `oracle/codemp/ui/ui_shared.c:9558-9570`
pub fn MenuParse_focuscolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.menu_mut(menu).focusColor[i] = f;
    }
    true
}

/// Raven `MenuParse_disablecolor` — parse a menu's disable color (4 floats).
/// Source: `oracle/codemp/ui/ui_shared.c:9572-9583`
pub fn MenuParse_disablecolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    for i in 0..4 {
        let mut f = 0.0f32;
        if !PC_Float_Parse(dc, handle, &mut f) {
            return false;
        }
        menus.menu_mut(menu).disableColor[i] = f;
    }
    true
}

/// Raven `MenuParse_cinematic` — parse a menu window's cinematic name.
/// Source: `oracle/codemp/ui/ui_shared.c:9605-9612`
pub fn MenuParse_cinematic(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_String_Parse(dc, handle, &mut menus.menu_mut(menu).window.cinematicName)
}

/// Raven `MenuParse_ownerdrawFlag` — OR an ownerdraw flag into a menu
/// window's `ownerDrawFlags`.
/// Source: `oracle/codemp/ui/ui_shared.c:9614-9623`
pub fn MenuParse_ownerdrawFlag(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    let mut i: c_int = 0;
    if !PC_Int_Parse(dc, handle, &mut i) {
        return false;
    }
    menus.menu_mut(menu).window.ownerDrawFlags |= i;
    true
}

/// Raven `MenuParse_ownerdraw` — parse a menu window's `ownerDraw` id.
/// Source: `oracle/codemp/ui/ui_shared.c:9625-9632`
pub fn MenuParse_ownerdraw(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).window.ownerDraw)
}

/// Raven `MenuParse_soundLoop` — parse a menu's looping sound name.
/// Source: `oracle/codemp/ui/ui_shared.c:9650-9657`
pub fn MenuParse_soundLoop(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_String_Parse(dc, handle, &mut menus.menu_mut(menu).soundName)
}

/// Raven `MenuParse_fadeClamp` — parse a menu's fade clamp.
/// Source: `oracle/codemp/ui/ui_shared.c:9659-9666`
pub fn MenuParse_fadeClamp(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.menu_mut(menu).fadeClamp)
}

/// Raven `MenuParse_fadeAmount` — parse a menu's fade amount.
/// Source: `oracle/codemp/ui/ui_shared.c:9668-9675`
pub fn MenuParse_fadeAmount(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.menu_mut(menu).fadeAmount)
}

/// Raven `MenuParse_fadeCycle` — parse a menu's fade cycle.
/// Source: `oracle/codemp/ui/ui_shared.c:9678-9685`
pub fn MenuParse_fadeCycle(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Int_Parse(dc, handle, &mut menus.menu_mut(menu).fadeCycle)
}

// `MenuParse_itemDef` — ui_shared.c:9688-9700.
//
// DEFERRED: MenuParse_itemDef — its body is one call to `Item_Parse`, which
// stays `// DEFERRED:` at its own site (ui_shared.c:9009-9040) because the
// `keywordHash_t` item-keyword dispatch it drives isn't ported; only its
// caller, the deferred `menuParseKeywords[]` table, would reference this.
// Source: `oracle/codemp/ui/ui_shared.c:9688-9700`

/// Raven `MenuParse_appearanceIncrement` — parse a menu's appearance
/// increment.
/// Source: `oracle/codemp/ui/ui_shared.c:9706-9715`
pub fn MenuParse_appearanceIncrement(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Float_Parse(dc, handle, &mut menus.menu_mut(menu).appearanceIncrement)
}

/// Raven `Display_CacheAll` — cache the render assets of every defined menu.
/// Source: `oracle/codemp/ui/ui_shared.c:9960-9965`
pub fn Display_CacheAll(menus: &MenuSystem, dc: &mut dyn DisplayContext) {
    for i in 0..menus.menus.len() {
        Menu_CacheContents(menus, dc, Some(MenuId::new(i)));
    }
}

/// Raven `Item_UpdatePosition` — recompute `item`'s screen position from its
/// parent menu's origin (plus border inset, if bordered).
///
/// PORT-NOTE: Raven's `item == NULL || item->parent == NULL` guard becomes
/// `item: Option<ItemId>` plus the arena's `Option<MenuId>` parent link.
/// Source: `oracle/codemp/ui/ui_shared.c:933-956`
pub fn Item_UpdatePosition(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
) {
    let item = match item {
        Some(id) => id,
        None => return,
    };
    let parent = match menus.item(item).parent {
        Some(p) => p,
        None => return,
    };

    let m = menus.menu(parent);
    let mut x = m.window.rect.x;
    let mut y = m.window.rect.y;
    if m.window.border != 0 {
        x += m.window.borderSize;
        y += m.window.borderSize;
    }

    Item_SetScreenCoords(menus, dc, Some(item), x, y);
}

/// Raven `Menu_UpdatePosition` — recompute every item's screen position from
/// `menu`'s origin (plus border inset, if bordered).
///
/// PORT-NOTE: Raven's `menu == NULL` guard becomes `menu: Option<MenuId>`.
/// Source: `oracle/codemp/ui/ui_shared.c:959-977`
pub fn Menu_UpdatePosition(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let m = menus.menu(menu);
    let mut x = m.window.rect.x;
    let mut y = m.window.rect.y;
    if m.window.border != 0 {
        x += m.window.borderSize;
        y += m.window.borderSize;
    }

    let items = menus.menu(menu).items.clone();
    for it in items {
        Item_SetScreenCoords(menus, dc, Some(it), x, y);
    }
}

/// Raven `Menu_ClearFocus` — clear `WINDOW_HASFOCUS` on every item in `menu`
/// (running each cleared item's `leaveFocus` script), returning the item that
/// had focus, if any.
///
/// PORT-NOTE: Raven's `menu == NULL` guard becomes `menu: Option<MenuId>`;
/// `leaveFocus`'s pointer-truthy guard becomes `!leaveFocus.is_empty()`
/// (`Item_Action`'s pattern below).
/// Source: `oracle/codemp/ui/ui_shared.c:992-1011`
pub fn Menu_ClearFocus(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
) -> Option<ItemId> {
    let menu = menu?;
    let items = menus.menu(menu).items.clone();
    let mut ret = None;

    for id in items {
        if menus.item(id).window.flags & WINDOW_HASFOCUS != 0 {
            ret = Some(id);
        }
        menus.item_mut(id).window.flags &= !WINDOW_HASFOCUS;

        let leaveFocus = menus.item(id).leaveFocus.clone();
        if !leaveFocus.is_empty() {
            Item_RunScript(menus, dc, id, &leaveFocus);
        }
    }

    ret
}

/// Raven `Script_SetItemText` — the `setitemtext` script command: set the
/// display text of the item(s) named by the first token in `menu`'s sibling
/// items.
/// Source: `oracle/codemp/ui/ui_shared.c:1206-1217`
pub fn Script_SetItemText(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut itemName = String::new();
    let mut text = String::new();

    // expecting text
    if String_Parse(args, &mut itemName) && String_Parse(args, &mut text) {
        let parent = menus.item(item).parent;
        Menu_SetItemText(menus, dc, parent, &itemName, &text);
    }
    true
}

/// Raven `Menu_RunCloseScript` — run `menu`'s `onClose` script, if the menu is
/// visible and has one.
///
/// PORT-NOTE: Raven builds a transient stack-local `itemDef_t` carrying only
/// `parent = menu` to hand `Item_RunScript` a script-command context (its
/// `setitemrect`/`show`/etc. commands read `item->parent` for the enclosing
/// menu, never the item itself here); the arena equivalent is a scratch slot
/// pushed for the call and truncated off immediately after, so no permanent
/// item is left behind. `onClose`'s pointer-truthy guard becomes
/// `!onClose.is_empty()`.
/// Source: `oracle/codemp/ui/ui_shared.c:1527-1533`
pub fn Menu_RunCloseScript(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    let m = menus.menu(menu);
    if m.window.flags & WINDOW_VISIBLE == 0 || m.onClose.is_empty() {
        return;
    }
    let onClose = m.onClose.clone();

    let idx = menus.items.len();
    let scratch = ItemId::new(idx);
    menus.items.push(ItemDef {
        parent: Some(menu),
        ..Default::default()
    });
    Item_RunScript(menus, dc, scratch, &onClose);
    // §19 divergence: an onClose `defer` stored Raven's stack-local `itemDef_t`,
    // which `rundeferred` then read dead-frame (UB); clearing it is the defined pick.
    if menus.ui_deferredScriptItem == Some(scratch) {
        menus.ui_deferredScriptItem = None;
    }
    menus.items.truncate(idx);
}

/// Raven `Menus_HandleOOBClick` — mouse-click-outside-window handling: close
/// `menu` if the click landed outside its window with `WINDOW_OOB_CLICK` set,
/// activate/focus whichever open menu the cursor is actually over, and
/// unpause when no menu remains visible.
///
/// PORT-NOTE: `menu: Option<MenuId>` mirrors Raven's `if (menu) { ... }`
/// pointer-null guard. Raven's `#ifdef _XBOX` early-return arm (forced mouse
/// move + key forward) is Xbox-only surface, not compiled in the PC/MP
/// build; the `#ifndef _XBOX` branch below is the retail path.
/// `DC->Pause`'s fn-pointer-truthy guard drops per DEC-36 D3 (the trait
/// method is always implemented).
/// Source: `oracle/codemp/ui/ui_shared.c:4453-4487`
pub fn Menus_HandleOOBClick(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    key: c_int,
    down: bool,
) {
    let Some(menu) = menu else {
        return;
    };

    let cursorx = ds.cursorx as f32;
    let cursory = ds.cursory as f32;

    // basically the behaviour we are looking for is if there are windows in the stack.. see if
    // the cursor is within any of them.. if not close them otherwise activate them and pass the
    // key on.. force a mouse move to activate focus and script stuff
    if down && menus.menu(menu).window.flags & WINDOW_OOB_CLICK != 0 {
        Menu_RunCloseScript(menus, dc, Some(menu));
        menus.menu_mut(menu).window.flags &= !(WINDOW_HASFOCUS | WINDOW_VISIBLE);
    }

    let menuCount = menus.menus.len();
    for i in 0..menuCount {
        let candidate = MenuId::new(i);
        if Menu_OverActiveItem(menus, Some(candidate), cursorx, cursory) {
            Menu_RunCloseScript(menus, dc, Some(menu));
            menus.menu_mut(menu).window.flags &= !(WINDOW_HASFOCUS | WINDOW_VISIBLE);
            Menus_Activate(menus, dc, candidate);
            Menu_HandleMouseMove(menus, ds, dc, Some(candidate), cursorx, cursory);
            Menu_HandleKey(menus, ds, dc, Some(candidate), key, down);
        }
    }

    if Display_VisibleMenuCount(menus) == 0 {
        dc.Pause(false);
    }
    Display_CloseCinematics(menus, dc);
}

/// Raven `Menu_HandleKey` — the menu framework's key-event entry point:
/// bind-capture and edit-field routing first, then out-of-window click
/// detection, then per-item key handling, then the default per-key menu
/// navigation.
///
/// PORT-NOTE: Raven's fn-scope (non-`static`) `qboolean inHandler` is
/// reinitialized to `qfalse` on every call and never read after being set —
/// its `if (inHandler) return;` guard at entry can never fire and every
/// `inHandler = qfalse;` before a `return` is a dead store. Dropped; only the
/// genuinely persistent `static qboolean inHandleKey` (→
/// `menus.scratch.inHandleKey`) survives as state.
///
/// PORT-NOTE: `g_bindItem`/`g_editItem` are read unconditionally in Raven
/// (guarded only by `g_waitingForKey`/`g_editingField`, which are set only
/// alongside the matching item); the `if let Some(..)` here preserves that
/// invariant defensively without changing observed behavior.
///
/// PORT-NOTE: the `A_ESCAPE`/`onAccept` cases build a transient scratch item
/// carrying only `parent = menu` to hand `Item_RunScript` a script-command
/// context, same convention as [`Menu_RunCloseScript`]/`Menus_Activate`'s
/// `onOpen`.
///
/// PORT-NOTE: Raven's `A_JOY0`-`A_JOY4`/`A_AUX0`-`A_AUX16` cases are explicit
/// no-op `break;`s; the `_` catch-all below is behaviorally identical.
/// Source: `oracle/codemp/ui/ui_shared.c:4490-4725`
pub fn Menu_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    key: c_int,
    down: bool,
) {
    if menus.g_waitingForKey && down {
        if let Some(bindItem) = menus.g_bindItem {
            Item_Bind_HandleKey(menus, ds, dc, bindItem, key, down);
        }
        return;
    }

    if menus.g_editingField && down {
        if let Some(editItem) = menus.g_editItem {
            if !Item_TextField_HandleKey(menus, ds, dc, editItem, key) {
                menus.g_editingField = false;
                menus.g_editItem = None;
                return;
            } else if key == A_MOUSE1 || key == A_MOUSE2 || key == A_MOUSE3 {
                // switching fields so reset printed text of edit field
                Leaving_EditField(menus, editItem);
                menus.g_editingField = false;
                menus.g_editItem = None;
                Display_MouseMove(menus, ds, dc, None, ds.cursorx, ds.cursory);
            } else if key == A_TAB || key == A_CURSOR_UP || key == A_CURSOR_DOWN {
                return;
            }
        }
    }

    let Some(menu) = menu else {
        return;
    };

    // see if the mouse is within the window bounds and if so is this a mouse click
    if down
        && menus.menu(menu).window.flags & WINDOW_POPUP == 0
        && !Rect_ContainsPoint(
            Some(&menus.menu(menu).window.rect),
            ds.cursorx as f32,
            ds.cursory as f32,
        )
    {
        if !menus.scratch.inHandleKey && (key == A_MOUSE1 || key == A_MOUSE2 || key == A_MOUSE3) {
            menus.scratch.inHandleKey = true;
            Menus_HandleOOBClick(menus, ds, dc, Some(menu), key, down);
            menus.scratch.inHandleKey = false;
            return;
        }
    }

    // get the item with focus
    let mut item: Option<ItemId> = None;
    let itemCount = menus.menu(menu).items.len();
    for i in 0..itemCount {
        let candidate = menus.menu(menu).items[i];
        if menus.item(candidate).window.flags & WINDOW_HASFOCUS != 0 {
            item = Some(candidate);
        }
    }

    // Ignore if disabled
    if let Some(it) = item {
        if menus.item(it).disabled {
            return;
        }
    }

    if let Some(it) = item {
        if Item_HandleKey(menus, ds, dc, it, key, down) {
            // It is possible for an item to be disable after Item_HandleKey is run (like in Voice Chat)
            if !menus.item(it).disabled {
                Item_Action(menus, dc, Some(it));
            }
            return;
        }
    }

    if !down {
        return;
    }

    // default handling
    match key {
        A_F11 => {
            if dc.getCVarValue("developer") != 0.0 {
                menus.debugMode = !menus.debugMode;
            }
        }
        A_F12 => {
            if dc.getCVarValue("developer") != 0.0 {
                dc.executeText(cbufExec_t::EXEC_APPEND as c_int, "screenshot\n");
            }
        }
        A_KP_8 | A_CURSOR_UP => {
            Menu_SetPrevCursorItem(menus, ds, dc, menu);
        }
        A_ESCAPE => {
            if !menus.g_waitingForKey && !menus.menu(menu).onESC.is_empty() {
                let onESC = menus.menu(menu).onESC.clone();
                let idx = menus.items.len();
                let scratch = ItemId::new(idx);
                menus.items.push(ItemDef {
                    parent: Some(menu),
                    ..Default::default()
                });
                Item_RunScript(menus, dc, scratch, &onESC);
                menus.items.truncate(idx);
            }
            menus.g_waitingForKey = false;
        }
        A_TAB | A_KP_2 | A_CURSOR_DOWN => {
            Menu_SetNextCursorItem(menus, ds, dc, menu);
        }
        A_MOUSE1 | A_MOUSE2 => {
            if let Some(it) = item {
                let itype = menus.item(it).r#type;
                if itype == ITEM_TYPE_TEXT {
                    if Rect_ContainsPoint(
                        Some(&menus.item(it).window.rect),
                        ds.cursorx as f32,
                        ds.cursory as f32,
                    ) {
                        Item_Action(menus, dc, Some(it));
                    }
                } else if itype == ITEM_TYPE_EDITFIELD || itype == ITEM_TYPE_NUMERICFIELD {
                    if Rect_ContainsPoint(
                        Some(&menus.item(it).window.rect),
                        ds.cursorx as f32,
                        ds.cursory as f32,
                    ) {
                        Item_Action(menus, dc, Some(it));
                        menus.item_mut(it).cursorPos = 0;
                        menus.g_editingField = true;
                        menus.g_editItem = Some(it);
                        dc.setOverstrikeMode(true);
                    }
                }
                //JLFACCEPT
                // add new types here as needed
                /* Notes:
                    Most controls will use the dpad to move through the selection possibilies.  Buttons are the only exception.
                    Buttons will be assumed to all be on one menu together.  If the start or A button is pressed on a control focus, that
                    means that the menu is accepted and move onto the next menu.  If the start or A button is pressed on a button focus it
                    should just process the action and not support the accept functionality.
                */
                else if itype == ITEM_TYPE_MULTI
                    || itype == ITEM_TYPE_YESNO
                    || itype == ITEM_TYPE_SLIDER
                {
                    if Item_HandleAccept(menus, dc, it) {
                        // Item processed it overriding the menu processing
                        return;
                    } else if !menus.menu(menu).onAccept.is_empty() {
                        let onAccept = menus.menu(menu).onAccept.clone();
                        let idx = menus.items.len();
                        let scratch = ItemId::new(idx);
                        menus.items.push(ItemDef {
                            parent: Some(menu),
                            ..Default::default()
                        });
                        Item_RunScript(menus, dc, scratch, &onAccept);
                        menus.items.truncate(idx);
                    }
                } else if Rect_ContainsPoint(
                    Some(&menus.item(it).window.rect),
                    ds.cursorx as f32,
                    ds.cursory as f32,
                ) {
                    Item_Action(menus, dc, Some(it));
                }
            }
        }
        A_KP_ENTER | A_ENTER => {
            if let Some(it) = item {
                let itype = menus.item(it).r#type;
                if itype == ITEM_TYPE_EDITFIELD || itype == ITEM_TYPE_NUMERICFIELD {
                    menus.item_mut(it).cursorPos = 0;
                    menus.g_editingField = true;
                    menus.g_editItem = Some(it);
                    dc.setOverstrikeMode(true);
                } else {
                    Item_Action(menus, dc, item);
                }
            }
        }
        _ => {}
    }
}

/// Raven `Menu_Paint` — paint `menu`'s background/border/chrome and every
/// item it owns (respecting timed order-of-appearance), plus a debug-mode
/// extents box.
///
/// PORT-NOTE: `seLanguageModCount` threads in for the `Item_Paint` call, same
/// convention as `Item_Paint`'s own doc note. `DC->ownerDrawVisible`'s
/// fn-pointer-truthy guard drops per DEC-36 D3 (always implemented); only the
/// flags half of the condition survives.
/// Source: `oracle/codemp/ui/ui_shared.c:7212-7272`
pub fn Menu_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    forcePaint: bool,
    seLanguageModCount: c_int,
) {
    let Some(menu) = menu else {
        return;
    };

    if menus.menu(menu).window.flags & WINDOW_VISIBLE == 0 && !forcePaint {
        return;
    }

    let ownerDrawFlags = menus.menu(menu).window.ownerDrawFlags;
    if ownerDrawFlags != 0 && !dc.ownerDrawVisible(ownerDrawFlags) {
        return;
    }

    if forcePaint {
        menus.menu_mut(menu).window.flags |= WINDOW_FORCED;
    }

    // draw the background if necessary
    if menus.menu(menu).fullScreen {
        // implies a background shader
        // FIXME: make sure we have a default shader if fullscreen is set with no background
        let background = menus.menu(menu).window.background;
        dc.drawHandlePic(
            0.0,
            0.0,
            SCREEN_WIDTH as f32,
            SCREEN_HEIGHT as f32,
            background,
        );
    } else if menus.menu(menu).window.background != 0 {
        // this allows a background shader without being full screen
        // UI_DrawHandlePic(menu->window.rect.x, menu->window.rect.y, menu->window.rect.w, menu->window.rect.h, menu->backgroundShader);
    }

    // paint the background and or border
    let (fadeAmount, fadeClamp, fadeCycle) = {
        let m = menus.menu(menu);
        (m.fadeAmount, m.fadeClamp, m.fadeCycle)
    };
    let mut window = menus.menu(menu).window.clone();
    Window_Paint(menus, ds, dc, &mut window, fadeAmount, fadeClamp, fadeCycle);
    menus.menu_mut(menu).window = window;

    // Loop through all items for the menu and paint them
    let itemCount = menus.menu(menu).items.len();
    for i in 0..itemCount {
        let it = menus.menu(menu).items[i];
        if menus.item(it).appearanceSlot == 0 {
            Item_Paint(menus, ds, dc, Some(it), seLanguageModCount);
        } else {
            // Timed order of appearance
            if menus.menu(menu).appearanceTime < ds.realTime as f32 {
                // Time to show another item
                let increment = menus.menu(menu).appearanceIncrement;
                menus.menu_mut(menu).appearanceTime = ds.realTime as f32 + increment;
                menus.menu_mut(menu).appearanceCnt += 1;
            }

            if menus.item(it).appearanceSlot <= menus.menu(menu).appearanceCnt {
                Item_Paint(menus, ds, dc, Some(it), seLanguageModCount);
            }
        }
    }

    if menus.debugMode {
        let mut color: vec4_t = [0.0; 4];
        color[0] = 1.0;
        color[2] = 1.0;
        color[3] = 1.0;
        color[1] = 0.0;
        let rect = menus.menu(menu).window.rect;
        dc.drawRect(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    }
}

/// Raven `Script_RunDeferred` — the `rundeferred` script command: run the
/// script suspended by [`Script_Defer`], if one is pending.
///
/// PORT-NOTE: Raven's own body never reads `item`/`args` either (kept for
/// signature parity).
/// Source: `oracle/codemp/ui/ui_shared.c:1794-1806`
pub fn Script_RunDeferred(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    _args: &mut &str,
) -> bool {
    // Make sure there is something to run.
    if menus.ui_deferredScript.is_empty() || menus.ui_deferredScriptItem.is_none() {
        return true;
    }

    // Run the deferred script now
    let script = menus.ui_deferredScript.clone();
    let deferredItem = menus.ui_deferredScriptItem.unwrap();
    Item_RunScript(menus, dc, deferredItem, &script);

    true
}

/// Raven `Item_MouseEnter` — refresh `item`'s mouse-over/mouse-over-text flags
/// (and run the matching enter/exit scripts) for the mouse at `(x, y)`.
///
/// PORT-NOTE: Raven's `#ifndef _XBOX` arm (the retail/live one) is kept; the
/// `#else` (`item->flags & WINDOW_HASFOCUS`) is dead surface, dropped per
/// porting-rules §20.
/// Source: `oracle/codemp/ui/ui_shared.c:3028-3088`
pub fn Item_MouseEnter(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
    x: f32,
    y: f32,
) {
    let item = match item {
        Some(id) => id,
        None => return,
    };

    let disabled = menus.item(item).disabled;
    let mut r = menus.item(item).textRect;
    r.y -= r.h;
    // in the text rect?

    // items can be enabled and disabled
    if disabled {
        return;
    }

    // items can be enabled and disabled based on cvars
    let cvarFlags = menus.item(item).cvarFlags;
    if cvarFlags & (CVAR_ENABLE | CVAR_DISABLE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_ENABLE)
    {
        return;
    }

    if cvarFlags & (CVAR_SHOW | CVAR_HIDE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_SHOW)
    {
        return;
    }

    if Rect_ContainsPoint(Some(&r), x, y) {
        if menus.item(item).window.flags & WINDOW_MOUSEOVERTEXT == 0 {
            let mouseEnterText = menus.item(item).mouseEnterText.clone();
            Item_RunScript(menus, dc, item, &mouseEnterText);
            menus.item_mut(item).window.flags |= WINDOW_MOUSEOVERTEXT;
        }
        if menus.item(item).window.flags & WINDOW_MOUSEOVER == 0 {
            let mouseEnter = menus.item(item).mouseEnter.clone();
            Item_RunScript(menus, dc, item, &mouseEnter);
            menus.item_mut(item).window.flags |= WINDOW_MOUSEOVER;
        }
    } else {
        // not in the text rect
        if menus.item(item).window.flags & WINDOW_MOUSEOVERTEXT != 0 {
            // if we were
            let mouseExitText = menus.item(item).mouseExitText.clone();
            Item_RunScript(menus, dc, item, &mouseExitText);
            menus.item_mut(item).window.flags &= !WINDOW_MOUSEOVERTEXT;
        }
        if menus.item(item).window.flags & WINDOW_MOUSEOVER == 0 {
            let mouseEnter = menus.item(item).mouseEnter.clone();
            Item_RunScript(menus, dc, item, &mouseEnter);
            menus.item_mut(item).window.flags |= WINDOW_MOUSEOVER;
        }

        let itype = menus.item(item).r#type;
        if itype == ITEM_TYPE_LISTBOX {
            Item_ListBox_MouseEnter(menus, dc, item, x, y);
        } else if itype == ITEM_TYPE_TEXTSCROLL {
            Item_TextScroll_MouseEnter(menus, item, x, y);
        }
    }
}

/// Raven `Item_MouseLeave` — run `item`'s exit script(s) and clear its
/// mouse-over flags/list-box arrow hot zones.
/// Source: `oracle/codemp/ui/ui_shared.c:3090-3099`
pub fn Item_MouseLeave(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, item: Option<ItemId>) {
    let item = match item {
        Some(id) => id,
        None => return,
    };

    if menus.item(item).window.flags & WINDOW_MOUSEOVERTEXT != 0 {
        let mouseExitText = menus.item(item).mouseExitText.clone();
        Item_RunScript(menus, dc, item, &mouseExitText);
        menus.item_mut(item).window.flags &= !WINDOW_MOUSEOVERTEXT;
    }
    let mouseExit = menus.item(item).mouseExit.clone();
    Item_RunScript(menus, dc, item, &mouseExit);
    menus.item_mut(item).window.flags &= !(WINDOW_LB_RIGHTARROW | WINDOW_LB_LEFTARROW);
}

/// Raven `Item_ListBox_HandleKey` — list-box key/mouse handling: horizontal or
/// vertical cursor movement, scrollbar hit-testing, selection, and paging.
///
/// PORT-NOTE (§19 UB pick): Raven casts `item->typeData` to `listBoxDef_t *`
/// unconditionally; a payload-type mismatch (unreachable under this file's
/// own type dispatch, since only list-box items reach this handler) returns
/// `false` here instead of a null deref.
/// PORT-NOTE: Raven's `#ifndef _XBOX` arm (the retail/live one, cursor-key
/// early-return-on-clamp and mouse-position focus test) is kept throughout;
/// the `#else`/`#ifdef _XBOX` arms are dead surface, dropped per
/// porting-rules §20. `down` is unused — Raven's own body never reads it
/// either (kept for signature parity).
/// Source: `oracle/codemp/ui/ui_shared.c:3220-3475`
pub fn Item_ListBox_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
    _down: bool,
    force: bool,
) -> bool {
    if menus.item(item).typeData.listBox().is_none() {
        return false;
    }

    let special = menus.item(item).special;
    let rect = menus.item(item).window.rect;
    let windowFlags = menus.item(item).window.flags;

    let count = dc.feederCount(special);

    let inFocus = force
        || (Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
            && windowFlags & WINDOW_HASFOCUS != 0);
    if !inFocus {
        return false;
    }

    let max = Item_ListBox_MaxScroll(menus, dc, item);
    let viewmax: c_int;

    if windowFlags & WINDOW_HORIZONTAL != 0 {
        let elementWidth = menus.item(item).typeData.listBox().unwrap().elementWidth;
        viewmax = (rect.w / elementWidth) as c_int;

        if key == A_CURSOR_LEFT || key == A_KP_4 {
            let notselectable = menus.item(item).typeData.listBox().unwrap().notselectable;
            if !notselectable {
                let mut cursorPos = menus.item(item).typeData.listBox().unwrap().cursorPos;
                cursorPos -= 1;
                if cursorPos < 0 {
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = 0;
                    }
                    return false;
                }
                let mut startPos = menus.item(item).typeData.listBox().unwrap().startPos;
                if cursorPos < startPos {
                    startPos = cursorPos;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                        l.startPos = startPos;
                    }
                    return false;
                }
                if cursorPos >= startPos + viewmax {
                    startPos = cursorPos - viewmax + 1;
                }
                if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                    l.cursorPos = cursorPos;
                    l.startPos = startPos;
                }
                menus.item_mut(item).cursorPos = cursorPos;
                dc.feederSelection(special, cursorPos, None);
            } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos -= 1;
                if l.startPos < 0 {
                    l.startPos = 0;
                }
            }
            return true;
        }
        if key == A_CURSOR_RIGHT || key == A_KP_6 {
            let notselectable = menus.item(item).typeData.listBox().unwrap().notselectable;
            if !notselectable {
                let mut cursorPos = menus.item(item).typeData.listBox().unwrap().cursorPos;
                cursorPos += 1;
                let mut startPos = menus.item(item).typeData.listBox().unwrap().startPos;
                if cursorPos < startPos {
                    startPos = cursorPos;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                        l.startPos = startPos;
                    }
                    return false;
                }
                if cursorPos >= count {
                    cursorPos = count - 1;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                    }
                    return false;
                }
                if cursorPos >= startPos + viewmax {
                    startPos = cursorPos - viewmax + 1;
                }
                if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                    l.cursorPos = cursorPos;
                    l.startPos = startPos;
                }
                menus.item_mut(item).cursorPos = cursorPos;
                dc.feederSelection(special, cursorPos, None);
            } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos += 1;
                if l.startPos >= count {
                    l.startPos = count - 1;
                }
            }
            return true;
        }
    } else {
        let (elementWidth, elementHeight, elementStyle) = {
            let l = menus.item(item).typeData.listBox().unwrap();
            (l.elementWidth, l.elementHeight, l.elementStyle)
        };
        // Multiple rows and columns (since it's more than twice as wide as an element)
        if rect.w > (elementWidth * 2.0) && elementStyle == LISTBOX_IMAGE {
            viewmax = (rect.w / elementWidth) as c_int;
        } else {
            viewmax = (rect.h / elementHeight) as c_int;
        }

        if key == A_CURSOR_UP || key == A_KP_8 {
            let notselectable = menus.item(item).typeData.listBox().unwrap().notselectable;
            if !notselectable {
                let mut cursorPos = menus.item(item).typeData.listBox().unwrap().cursorPos;
                cursorPos -= 1;
                if cursorPos < 0 {
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = 0;
                    }
                    return false;
                }
                let mut startPos = menus.item(item).typeData.listBox().unwrap().startPos;
                if cursorPos < startPos {
                    startPos = cursorPos;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                        l.startPos = startPos;
                    }
                    return false;
                }
                if cursorPos >= startPos + viewmax {
                    startPos = cursorPos - viewmax + 1;
                }
                if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                    l.cursorPos = cursorPos;
                    l.startPos = startPos;
                }
                menus.item_mut(item).cursorPos = cursorPos;
                dc.feederSelection(special, cursorPos, None);
            } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos -= 1;
                if l.startPos < 0 {
                    l.startPos = 0;
                }
            }
            return true;
        }
        if key == A_CURSOR_DOWN || key == A_KP_2 {
            let notselectable = menus.item(item).typeData.listBox().unwrap().notselectable;
            if !notselectable {
                let mut cursorPos = menus.item(item).typeData.listBox().unwrap().cursorPos;
                cursorPos += 1;
                let mut startPos = menus.item(item).typeData.listBox().unwrap().startPos;
                if cursorPos < startPos {
                    startPos = cursorPos;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                        l.startPos = startPos;
                    }
                    return false;
                }
                if cursorPos >= count {
                    cursorPos = count - 1;
                    if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                        l.cursorPos = cursorPos;
                    }
                    return false;
                }
                if cursorPos >= startPos + viewmax {
                    startPos = cursorPos - viewmax + 1;
                }
                if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                    l.cursorPos = cursorPos;
                    l.startPos = startPos;
                }
                menus.item_mut(item).cursorPos = cursorPos;
                dc.feederSelection(special, cursorPos, None);
            } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos += 1;
                if l.startPos > max {
                    l.startPos = max;
                }
            }
            return true;
        }
    }

    // mouse hit
    if key == A_MOUSE1 || key == A_MOUSE2 {
        let flags = menus.item(item).window.flags;
        if flags & WINDOW_LB_LEFTARROW != 0 {
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos -= 1;
                if l.startPos < 0 {
                    l.startPos = 0;
                }
            }
        } else if flags & WINDOW_LB_RIGHTARROW != 0 {
            // one down
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos += 1;
                if l.startPos > max {
                    l.startPos = max;
                }
            }
        } else if flags & WINDOW_LB_PGUP != 0 {
            // page up
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos -= viewmax;
                if l.startPos < 0 {
                    l.startPos = 0;
                }
            }
        } else if flags & WINDOW_LB_PGDN != 0 {
            // page down
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.startPos += viewmax;
                if l.startPos > max {
                    l.startPos = max;
                }
            }
        } else if flags & WINDOW_LB_THUMB != 0 {
            // Display_SetCaptureItem(item); — commented out in the oracle.
        } else {
            // select an item
            let doubleClick = menus
                .item(item)
                .typeData
                .listBox()
                .map(|l| l.doubleClick.clone())
                .unwrap_or_default();
            if ds.realTime < menus.lastListBoxClickTime && !doubleClick.is_empty() {
                Item_RunScript(menus, dc, item, &doubleClick);
            }
            menus.lastListBoxClickTime = ds.realTime + DOUBLE_CLICK_DELAY;

            let prePos = menus.item(item).cursorPos;
            let lbCursorPos = menus
                .item(item)
                .typeData
                .listBox()
                .map(|l| l.cursorPos)
                .unwrap_or(prePos);
            menus.item_mut(item).cursorPos = lbCursorPos;
            if !dc.feederSelection(special, lbCursorPos, Some(item)) {
                menus.item_mut(item).cursorPos = prePos;
                if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                    l.cursorPos = prePos;
                }
            }
        }
        return true;
    }
    if key == A_HOME || key == A_KP_7 {
        // home
        if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
            l.startPos = 0;
        }
        return true;
    }
    if key == A_END || key == A_KP_1 {
        // end
        if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
            l.startPos = max;
        }
        return true;
    }
    if key == A_PAGE_UP || key == A_KP_9 {
        // page up
        let notselectable = menus
            .item(item)
            .typeData
            .listBox()
            .map(|l| l.notselectable)
            .unwrap_or(true);
        if !notselectable {
            let (mut cursorPos, mut startPos) = {
                let l = menus.item(item).typeData.listBox().unwrap();
                (l.cursorPos, l.startPos)
            };
            cursorPos -= viewmax;
            if cursorPos < 0 {
                cursorPos = 0;
            }
            if cursorPos < startPos {
                startPos = cursorPos;
            }
            if cursorPos >= startPos + viewmax {
                startPos = cursorPos - viewmax + 1;
            }
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.cursorPos = cursorPos;
                l.startPos = startPos;
            }
            menus.item_mut(item).cursorPos = cursorPos;
            dc.feederSelection(special, cursorPos, None);
        } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
            l.startPos -= viewmax;
            if l.startPos < 0 {
                l.startPos = 0;
            }
        }
        return true;
    }
    if key == A_PAGE_DOWN || key == A_KP_3 {
        // page down
        let notselectable = menus
            .item(item)
            .typeData
            .listBox()
            .map(|l| l.notselectable)
            .unwrap_or(true);
        if !notselectable {
            let (mut cursorPos, mut startPos) = {
                let l = menus.item(item).typeData.listBox().unwrap();
                (l.cursorPos, l.startPos)
            };
            cursorPos += viewmax;
            if cursorPos < startPos {
                startPos = cursorPos;
            }
            if cursorPos >= count {
                cursorPos = count - 1;
            }
            if cursorPos >= startPos + viewmax {
                startPos = cursorPos - viewmax + 1;
            }
            if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
                l.cursorPos = cursorPos;
                l.startPos = startPos;
            }
            menus.item_mut(item).cursorPos = cursorPos;
            dc.feederSelection(special, cursorPos, None);
        } else if let Some(l) = menus.item_mut(item).typeData.listBox_mut() {
            l.startPos += viewmax;
            if l.startPos > max {
                l.startPos = max;
            }
        }
        return true;
    }

    false
}

/// Raven `Item_HandleAccept` — run `item`'s `accept` script, if it has one.
/// Source: `oracle/codemp/ui/ui_shared.c:4271-4279`
pub fn Item_HandleAccept(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> bool {
    let accept = menus.item(item).accept.clone();
    if !accept.is_empty() {
        Item_RunScript(menus, dc, item, &accept);
        return true;
    }
    false
}

/// Raven `Item_Action` — run `item`'s `action` script, if any.
///
/// PORT-NOTE: Raven's `item == NULL` guard becomes `item: Option<ItemId>`.
/// Source: `oracle/codemp/ui/ui_shared.c:4321-4325`
pub fn Item_Action(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, item: Option<ItemId>) {
    if let Some(item) = item {
        let action = menus.item(item).action.clone();
        Item_RunScript(menus, dc, item, &action);
    }
}

/// Raven `Menus_Activate` — mark `menu` focused and visible, run its `onOpen`
/// script, start its background sound loop, and reset its appearance timer.
///
/// PORT-NOTE (dead surface): Raven's `#ifdef _XBOX` tail (three
/// `ui_hideXcallout` cvar resets) is dead on every retail/live target this
/// port ships; dropped per porting-rules §20. Its transient stack-local
/// `itemDef_t` (carrying `parent = menu` for `onOpen`'s script context) takes
/// the same scratch-arena-slot shape as `Menu_RunCloseScript`. `soundName`'s
/// pointer-truthy-and-non-empty guard becomes `!soundName.is_empty()`.
/// Source: `oracle/codemp/ui/ui_shared.c:4416-4440`
pub fn Menus_Activate(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, menu: MenuId) {
    menus.menu_mut(menu).window.flags |= WINDOW_HASFOCUS | WINDOW_VISIBLE;

    let onOpen = menus.menu(menu).onOpen.clone();
    if !onOpen.is_empty() {
        let idx = menus.items.len();
        let scratch = ItemId::new(idx);
        menus.items.push(ItemDef {
            parent: Some(menu),
            ..Default::default()
        });
        Item_RunScript(menus, dc, scratch, &onOpen);
        // §19 divergence: see `Menu_RunCloseScript` — a deferred scratch slot is
        // cleared rather than left dangling.
        if menus.ui_deferredScriptItem == Some(scratch) {
            menus.ui_deferredScriptItem = None;
        }
        menus.items.truncate(idx);
    }

    let soundName = menus.menu(menu).soundName.clone();
    if !soundName.is_empty() {
        // you don't want to stop the background track since it will reset s_rawend
        dc.startBackgroundTrack(&soundName, &soundName, false);
    }

    menus.menu_mut(menu).appearanceTime = 0.0;
    Display_CloseCinematics(menus, dc);
}

/// Raven `Item_TextColor` — the pulse/blink/disabled color an item's text
/// should paint with right now.
///
/// PORT-NOTE: Raven's `item->enableCvar && *item->enableCvar && item->cvarTest
/// && *item->cvarTest` (both pointer-truthy-and-non-empty) becomes
/// `!enableCvar.is_empty() && !cvarTest.is_empty()` (`Item_EnableShowViaCvar`'s
/// established pattern).
/// Source: `oracle/codemp/ui/ui_shared.c:4793-4826`
pub fn Item_TextColor(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    newColor: &mut vec4_t,
) {
    let parent = menus
        .item(item)
        .parent
        .expect("Item_TextColor: item has no parent");
    let m = menus.menu(parent);
    let fadeClamp = m.fadeClamp;
    let fadeCycle = m.fadeCycle;
    let fadeAmount = m.fadeAmount;
    let focusColor = m.focusColor;
    let disableColor = m.disableColor;

    {
        let it = menus.item_mut(item);
        Fade(
            ds,
            &mut it.window.flags,
            &mut it.window.foreColor[3],
            fadeClamp,
            &mut it.window.nextTime,
            fadeCycle,
            true,
            fadeAmount,
        );
    }

    let it = menus.item(item);
    if it.window.flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else if it.textStyle == ITEM_TEXTSTYLE_BLINK && (ds.realTime / BLINK_DIVISOR) & 1 == 0 {
        let foreColor = it.window.foreColor;
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * foreColor[i];
        }
        LerpColor(
            foreColor,
            lowLight,
            newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
        // items can be enabled and disabled based on cvars
    } else {
        *newColor = it.window.foreColor;
    }

    if it.disabled {
        *newColor = disableColor;
    }

    if !it.enableCvar.is_empty() && !it.cvarTest.is_empty() {
        let cvarFlags = it.cvarFlags;
        if cvarFlags & (CVAR_ENABLE | CVAR_DISABLE) != 0
            && !Item_EnableShowViaCvar(menus, dc, item, CVAR_ENABLE)
        {
            *newColor = disableColor;
        }
    }
}

/// Raven `ItemParse_rect` — parse an item window's client rectangle.
/// Source: `oracle/codemp/ui/ui_shared.c:7962-7967`
pub fn ItemParse_rect(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    PC_Rect_Parse(dc, handle, &mut menus.item_mut(item).window.rectClient)
}

/// Raven `ItemParse_outlinecolor` — parse an item window's outline color.
/// Source: `oracle/codemp/ui/ui_shared.c:8369-8374`
pub fn ItemParse_outlinecolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    PC_Color_Parse(dc, handle, &mut menus.item_mut(item).window.outlineColor)
}

/// Raven `ItemParse_addColorRange` — append a color range to an item's
/// `colorRanges` list, if there is room.
/// Source: `oracle/codemp/ui/ui_shared.c:8763-8776`
pub fn ItemParse_addColorRange(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    handle: c_int,
) -> bool {
    let mut color = ColorRangeDef::default();

    if PC_Float_Parse(dc, handle, &mut color.low)
        && PC_Float_Parse(dc, handle, &mut color.high)
        && PC_Color_Parse(dc, handle, &mut color.color)
    {
        let it = menus.item_mut(item);
        if (it.numColors as usize) < MAX_COLOR_RANGES {
            it.colorRanges[it.numColors as usize] = color;
            it.numColors += 1;
        }
        return true;
    }
    false
}

/// Raven `MenuParse_rect` — parse a menu window's rectangle.
///
/// PORT-NOTE: Raven's `itemDef_t *item` parameter is immediately cast to
/// `menuDef_t *menu` (see `MenuParse_background`'s PORT-NOTE); this takes the
/// `MenuId` the cast resolves to directly.
/// Source: `oracle/codemp/ui/ui_shared.c:9324-9330`
pub fn MenuParse_rect(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Rect_Parse(dc, handle, &mut menus.menu_mut(menu).window.rect)
}

/// Raven `MenuParse_outlinecolor` — parse a menu window's outline color.
///
/// PORT-NOTE: see `MenuParse_rect` — the `itemDef_t *` cast to `menuDef_t *`
/// becomes a direct `MenuId`.
/// Source: `oracle/codemp/ui/ui_shared.c:9586-9592`
pub fn MenuParse_outlinecolor(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    handle: c_int,
) -> bool {
    PC_Color_Parse(dc, handle, &mut menus.menu_mut(menu).window.outlineColor)
}

/// Raven `Menu_PostParse` — snap a full-screen menu's window to the virtual
/// screen rect, then recompute every item's screen position.
///
/// PORT-NOTE: Raven's `menu == NULL` guard becomes `menu: Option<MenuId>`.
/// Source: `oracle/codemp/ui/ui_shared.c:979-990`
pub fn Menu_PostParse(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, menu: Option<MenuId>) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };
    if menus.menu(menu).fullScreen {
        let w = &mut menus.menu_mut(menu).window;
        w.rect.x = 0.0;
        w.rect.y = 0.0;
        w.rect.w = 640.0;
        w.rect.h = 480.0;
    }
    Menu_UpdatePosition(menus, dc, Some(menu));
}

/// Raven `Menus_ShowByName` — activate the menu named `p`, if defined.
/// Source: `oracle/codemp/ui/ui_shared.c:1516-1521`
pub fn Menus_ShowByName(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, p: &str) {
    if let Some(menu) = Menus_FindByName(menus, p) {
        Menus_Activate(menus, dc, menu);
    }
}

/// Raven `Menus_CloseByName` — run the named menu's close script and hide it,
/// handing focus to whatever is now on top of the open-menu stack (if the
/// closed menu had it).
///
/// PORT-NOTE: Raven's `openMenuCount -= 1; menuStack[openMenuCount]->flags
/// |= WINDOW_HASFOCUS; menuStack[openMenuCount] = NULL;` triple is exactly
/// `menuStack.pop()` — the arena's `menuStack: Vec<MenuId>` makes
/// `openMenuCount` `menuStack.len()` (§B5).
/// Source: `oracle/codemp/ui/ui_shared.c:1535-1569`
pub fn Menus_CloseByName(menus: &mut MenuSystem, dc: &mut dyn DisplayContext, p: &str) {
    let menu = match Menus_FindByName(menus, p) {
        Some(m) => m,
        None => return,
    };

    // Run the close script for the menu
    Menu_RunCloseScript(menus, dc, Some(menu));

    // If this window had the focus then take it away
    if menus.menu(menu).window.flags & WINDOW_HASFOCUS != 0 {
        // If there is something still in the open menu list then
        // set it to have focus now
        if let Some(top) = menus.menuStack.pop() {
            menus.menu_mut(top).window.flags |= WINDOW_HASFOCUS;
        }
    }

    // Window is now invisible and doesnt have focus
    menus.menu_mut(menu).window.flags &= !(WINDOW_VISIBLE | WINDOW_HASFOCUS);
}

/// Raven `Menus_CloseAll` — run every defined menu's close script, hide them
/// all, and clear the open-menu stack.
/// Source: `oracle/codemp/ui/ui_shared.c:1573-1589`
pub fn Menus_CloseAll(menus: &mut MenuSystem, dc: &mut dyn DisplayContext) {
    menus.g_waitingForKey = false;

    for i in 0..menus.menus.len() {
        let id = MenuId::new(i);
        Menu_RunCloseScript(menus, dc, Some(id));
        menus.menu_mut(id).window.flags &= !(WINDOW_HASFOCUS | WINDOW_VISIBLE);
    }

    // Clear the menu stack
    menus.menuStack.clear();

    menus.FPMessageTime = 0;
}

/// The source rect of a [`Menu_TransitionItemByName`] transition: either the
/// caller's own `rectDef_t` or Raven's defaulted `&item->window.rect`, which is
/// a *live* pointer into the first matching item — `Item_UpdatePosition` writes
/// that rect at the end of every iteration, so later items read the mutated
/// values.
/// Source: `oracle/codemp/ui/ui_shared.c:1670-1673,1683`
#[derive(Clone, Copy)]
enum TransitionRectFrom {
    Explicit(RectDef),
    Live(ItemId),
}

/// Raven `Menu_TransitionItemByName` — kick off a rect-to-rect transition on
/// every item in `menu` matching group `p`.
///
/// PORT-NOTE: Raven's `if (!rectFrom) rectFrom = &item->window.rect;`
/// reassigns the *outer* local, so the first matching item lacking a
/// `rectFrom` fixes the source for every later item too — and it stays a live
/// pointer into that item's rect, which `Item_UpdatePosition` mutates each
/// iteration; [`TransitionRectFrom::Live`] re-reads it per use.
/// `abs()` on the `rectDef_t` field diffs is Raven's *integer* `abs` (float
/// args truncate to `int` first) — faithfully kept, not `f32::abs`.
/// Source: `oracle/codemp/ui/ui_shared.c:1660-1686`
#[allow(clippy::too_many_arguments)]
pub fn Menu_TransitionItemByName(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    p: &str,
    rectFrom: Option<RectDef>,
    rectTo: &RectDef,
    time: c_int,
    amt: f32,
) {
    let mut rectFrom = rectFrom.map(TransitionRectFrom::Explicit);
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, p);
    for i in 0..count {
        if let Some(item) = Menu_GetMatchingItemByNumber(menus, menu, i, p) {
            // if there are more than one of these with the same name, they'll
            // all use the FIRST one's FROM.
            let rf = match *rectFrom.get_or_insert(TransitionRectFrom::Live(item)) {
                TransitionRectFrom::Explicit(r) => r,
                TransitionRectFrom::Live(id) => menus.item(id).window.rect,
            };
            {
                let it = menus.item_mut(item);
                it.window.flags |= WINDOW_INTRANSITION | WINDOW_VISIBLE;
                it.window.offsetTime = time;
                it.window.rectClient = rf;
                it.window.rectEffects = *rectTo;
                it.window.rectEffects2.x = ((rectTo.x - rf.x) as c_int).abs() as f32 / amt;
                it.window.rectEffects2.y = ((rectTo.y - rf.y) as c_int).abs() as f32 / amt;
                it.window.rectEffects2.w = ((rectTo.w - rf.w) as c_int).abs() as f32 / amt;
                it.window.rectEffects2.h = ((rectTo.h - rf.h) as c_int).abs() as f32 / amt;
            }
            Item_UpdatePosition(menus, dc, Some(item));
        }
    }
}

/// Raven `Menu_OrbitItemByName` — start every item in `menu` matching group
/// `p` orbiting around `(cx, cy)`, starting from `(x, y)`.
/// Source: `oracle/codemp/ui/ui_shared.c:1826-1843`
#[allow(clippy::too_many_arguments)]
pub fn Menu_OrbitItemByName(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
    p: &str,
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    time: c_int,
) {
    let count = Menu_ItemsMatchingGroup(menus, dc, menu, p);
    for i in 0..count {
        if let Some(item) = Menu_GetMatchingItemByNumber(menus, menu, i, p) {
            {
                let it = menus.item_mut(item);
                it.window.flags |= WINDOW_ORBITING | WINDOW_VISIBLE;
                it.window.offsetTime = time;
                it.window.rectEffects.x = cx;
                it.window.rectEffects.y = cy;
                it.window.rectClient.x = x;
                it.window.rectClient.y = y;
            }
            Item_UpdatePosition(menus, dc, Some(item));
        }
    }
}

/// Raven `Script_SetFocus` — the `setfocus` script command: give focus to a
/// named sibling item (unless it's a decoration or already focused), run its
/// `onFocus` script and play the item-focus sound.
///
/// Raven's `#ifdef _XBOX` arm (`Item_SetFocus(focusItem, 0, 0)`) is dead on
/// every retail/live target (porting-rules §20); the `#else` arm
/// (`focusItem->window.flags |= WINDOW_HASFOCUS`) is transcribed.
/// Source: `oracle/codemp/ui/ui_shared.c:1956-1984`
pub fn Script_SetFocus(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();

    if String_Parse(args, &mut name) {
        let parent = menus.item(item).parent;
        if let Some(focusItem) = Menu_FindItemByName(menus, parent, &name) {
            let flags = menus.item(focusItem).window.flags;
            if flags & WINDOW_DECORATION == 0 && flags & WINDOW_HASFOCUS == 0 {
                Menu_ClearFocus(menus, dc, parent);

                menus.item_mut(focusItem).window.flags |= WINDOW_HASFOCUS;

                let onFocus = menus.item(focusItem).onFocus.clone();
                if !onFocus.is_empty() {
                    Item_RunScript(menus, dc, focusItem, &onFocus);
                }
                if ds.Assets.itemFocusSound != 0 {
                    dc.startLocalSound(ds.Assets.itemFocusSound, CHAN_LOCAL_SOUND);
                }
            }
        }
    }

    true
}

/// Raven `Item_SetFocus` — give `item` the focus (text items only if `(x,
/// y)` lands in their text rect, everything else unconditionally), clearing
/// whatever had it and running the relevant `onFocus` script(s) plus the
/// focus sound.
///
/// PORT-NOTE (§19 UB pick): Raven's closing loop derefs `parent`
/// unconditionally to find `item`'s cursor index, even though the comment at
/// `bk001206` notes `parent` "can be NULL"; the otherwise-unreachable
/// no-parent case here is a no-op instead of a null deref.
/// Source: `oracle/codemp/ui/ui_shared.c:2399-2480`
pub fn Item_SetFocus(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    x: f32,
    y: f32,
) -> bool {
    let flags = menus.item(item).window.flags;
    if flags & WINDOW_DECORATION != 0 || flags & WINDOW_HASFOCUS != 0 || flags & WINDOW_VISIBLE == 0
    {
        return false;
    }

    let parent = menus.item(item).parent;

    if menus.item(item).disabled {
        return false;
    }

    let cvarFlags = menus.item(item).cvarFlags;
    if cvarFlags & (CVAR_ENABLE | CVAR_DISABLE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_ENABLE)
    {
        return false;
    }

    if cvarFlags & (CVAR_SHOW | CVAR_HIDE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_SHOW)
    {
        return false;
    }

    let oldFocus = Menu_ClearFocus(menus, dc, parent);

    let mut sfx = ds.Assets.itemFocusSound;
    let mut playSound = false;

    let itemType = menus.item(item).r#type;
    if itemType == ITEM_TYPE_TEXT {
        let mut r = menus.item(item).textRect;
        r.y -= r.h;

        if Rect_ContainsPoint(Some(&r), x, y) {
            menus.item_mut(item).window.flags |= WINDOW_HASFOCUS;
            let focusSound = menus.item(item).focusSound;
            if focusSound != 0 {
                sfx = focusSound;
            }
            playSound = true;
        } else if let Some(oldFocusId) = oldFocus {
            menus.item_mut(oldFocusId).window.flags |= WINDOW_HASFOCUS;
            let onFocus = menus.item(oldFocusId).onFocus.clone();
            if !onFocus.is_empty() {
                Item_RunScript(menus, dc, oldFocusId, &onFocus);
            }
        }
    } else {
        menus.item_mut(item).window.flags |= WINDOW_HASFOCUS;
        let onFocus = menus.item(item).onFocus.clone();
        if !onFocus.is_empty() {
            Item_RunScript(menus, dc, item, &onFocus);
        }
        let focusSound = menus.item(item).focusSound;
        if focusSound != 0 {
            sfx = focusSound;
        }
        playSound = true;
    }

    if playSound {
        dc.startLocalSound(sfx, CHAN_LOCAL_SOUND);
    }

    if let Some(parentId) = parent {
        let items = menus.menu(parentId).items.clone();
        for (i, id) in items.iter().enumerate() {
            if *id == item {
                menus.menu_mut(parentId).cursorItem = i as c_int;
                break;
            }
        }
    }

    true
}

/// Raven `Scroll_ListBox_AutoFunc` — the listbox's auto-scroll capture-func
/// tick: repeat the captured scroll key on the throttle, easing the repeat
/// interval down to the floor.
///
/// PORT-NOTE (§19 UB pick): see `Scroll_TextScroll_AutoFunc` — Raven derefs
/// `si->item` unconditionally; the otherwise-unreachable "no captured item"
/// case here is a no-op.
/// Source: `oracle/codemp/ui/ui_shared.c:3904-3920`
pub fn Scroll_ListBox_AutoFunc(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
) {
    let item = match menus.scrollInfo.item {
        Some(id) => id,
        None => return,
    };

    if ds.realTime > menus.scrollInfo.nextScrollTime {
        // need to scroll which is done by simulating a click to the item
        // this is done a bit sideways as the autoscroll "knows" that the item is a listbox
        // so it calls it directly
        let scrollKey = menus.scrollInfo.scrollKey;
        Item_ListBox_HandleKey(menus, ds, dc, item, scrollKey, true, false);
        menus.scrollInfo.nextScrollTime = ds.realTime + menus.scrollInfo.adjustValue;
    }

    if ds.realTime > menus.scrollInfo.nextAdjustTime {
        menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
        if menus.scrollInfo.adjustValue > SCROLL_TIME_FLOOR {
            menus.scrollInfo.adjustValue -= SCROLL_TIME_ADJUSTOFFSET;
        }
    }
}

/// Raven `Scroll_ListBox_ThumbFunc` — the listbox's thumb-drag capture-func
/// tick: track the cursor along the scrollbar's axis (horizontal or
/// vertical, splitting rows for a multi-column image listbox), then run the
/// same auto-scroll throttle as [`Scroll_ListBox_AutoFunc`].
///
/// PORT-NOTE (§19 UB pick): Raven derefs `si->item` and casts `typeData`
/// unconditionally; the otherwise-unreachable (no captured item / non-listbox
/// payload) case here is a no-op instead of a null deref (see
/// `Scroll_Slider_ThumbFunc`).
/// Source: `oracle/codemp/ui/ui_shared.c:3922-3995`
pub fn Scroll_ListBox_ThumbFunc(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
) {
    let item = match menus.scrollInfo.item {
        Some(id) => id,
        None => return,
    };
    if menus.item(item).typeData.listBox().is_none() {
        return;
    }

    if menus.item(item).window.flags & WINDOW_HORIZONTAL != 0 {
        if ds.cursorx == menus.scrollInfo.xStart as c_int {
            return;
        }
        let rect = menus.item(item).window.rect;
        let r = RectDef {
            x: rect.x + SCROLLBAR_SIZE + 1.0,
            y: rect.y + rect.h - SCROLLBAR_SIZE - 1.0,
            h: SCROLLBAR_SIZE,
            w: rect.w - (SCROLLBAR_SIZE * 2.0) - 2.0,
        };
        let max = Item_ListBox_MaxScroll(menus, dc, item);

        let mut pos = ((ds.cursorx as f32 - r.x - SCROLLBAR_SIZE / 2.0) * max as f32
            / (r.w - SCROLLBAR_SIZE)) as c_int;
        if pos < 0 {
            pos = 0;
        } else if pos > max {
            pos = max;
        }
        if let Some(listPtr) = menus.item_mut(item).typeData.listBox_mut() {
            listPtr.startPos = pos;
        }
        menus.scrollInfo.xStart = ds.cursorx as f32;
    } else if ds.cursory != menus.scrollInfo.yStart as c_int {
        let rect = menus.item(item).window.rect;
        let r = RectDef {
            x: rect.x + rect.w - SCROLLBAR_SIZE - 1.0,
            y: rect.y + SCROLLBAR_SIZE + 1.0,
            h: rect.h - (SCROLLBAR_SIZE * 2.0) - 2.0,
            w: SCROLLBAR_SIZE,
        };
        let max = Item_ListBox_MaxScroll(menus, dc, item);

        let (elementWidth, elementStyle) = {
            let l = menus.item(item).typeData.listBox().unwrap();
            (l.elementWidth, l.elementStyle)
        };

        let mut pos: c_int;
        if rect.w > (elementWidth * 2.0) && elementStyle == LISTBOX_IMAGE {
            let rowLength = (rect.w / elementWidth) as c_int;
            let rowMax = max / rowLength;
            pos = ((ds.cursory as f32 - r.y - SCROLLBAR_SIZE / 2.0) * rowMax as f32
                / (r.h - SCROLLBAR_SIZE)) as c_int;
            pos *= rowLength;
        } else {
            pos = ((ds.cursory as f32 - r.y - SCROLLBAR_SIZE / 2.0) * max as f32
                / (r.h - SCROLLBAR_SIZE)) as c_int;
        }

        if pos < 0 {
            pos = 0;
        } else if pos > max {
            pos = max;
        }
        if let Some(listPtr) = menus.item_mut(item).typeData.listBox_mut() {
            listPtr.startPos = pos;
        }
        menus.scrollInfo.yStart = ds.cursory as f32;
    }

    if ds.realTime > menus.scrollInfo.nextScrollTime {
        // need to scroll which is done by simulating a click to the item
        // this is done a bit sideways as the autoscroll "knows" that the item is a listbox
        // so it calls it directly
        let scrollKey = menus.scrollInfo.scrollKey;
        Item_ListBox_HandleKey(menus, ds, dc, item, scrollKey, true, false);
        menus.scrollInfo.nextScrollTime = ds.realTime + menus.scrollInfo.adjustValue;
    }

    if ds.realTime > menus.scrollInfo.nextAdjustTime {
        menus.scrollInfo.nextAdjustTime = ds.realTime + SCROLL_TIME_ADJUST;
        if menus.scrollInfo.adjustValue > SCROLL_TIME_FLOOR {
            menus.scrollInfo.adjustValue -= SCROLL_TIME_ADJUSTOFFSET;
        }
    }
}

/// Raven `Item_HandleKey` — the generic per-item key router: release/start
/// mouse capture, then (on key-down) dispatch to the type-specific handler.
///
/// PORT-NOTE: Raven's `captureFunc`/`captureData` release triple becomes
/// clearing `MenuSystem::captureFunc`/`itemCapture` (`captureData` drops out
/// — see `CaptureFunc`'s doc comment). The `ITEM_TYPE_BUTTON` `#ifdef _XBOX`
/// arm is dead on every retail/live target (porting-rules §20); the `#else`
/// (`qfalse`) arm is transcribed. The commented-out
/// `Item_TextField_HandleKey` call under `ITEM_TYPE_EDITFIELD`/
/// `ITEM_TYPE_NUMERICFIELD` was never live in the oracle either — dropped.
/// Source: `oracle/codemp/ui/ui_shared.c:4176-4261`
pub fn Item_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
    down: bool,
) -> bool {
    if let Some(captured) = menus.itemCapture {
        Item_StopCapture(captured);
        menus.itemCapture = None;
        menus.captureFunc = CaptureFunc::None;
    } else if down && (key == A_MOUSE1 || key == A_MOUSE2 || key == A_MOUSE3) {
        Item_StartCapture(menus, ds, dc, item, key);
    }

    if !down {
        return false;
    }

    let itemType = menus.item(item).r#type;
    match itemType {
        ITEM_TYPE_BUTTON => false,
        ITEM_TYPE_RADIOBUTTON => false,
        ITEM_TYPE_CHECKBOX => false,
        ITEM_TYPE_EDITFIELD | ITEM_TYPE_NUMERICFIELD => {
            if key == A_MOUSE1 || key == A_MOUSE2 || key == A_ENTER {
                // §19: Raven casts `item->typeData` to `editFieldDef_t *`
                // unchecked; the typed payload no-ops on the (unreachable)
                // mismatch instead of reinterpreting foreign bytes.
                let it = menus.item(item);
                let hasCvarAndPtr = it.cvar.is_some() && it.typeData.editField().is_some();
                if hasCvarAndPtr {
                    if let Some(editPtr) = menus.item_mut(item).typeData.editField_mut() {
                        editPtr.paintOffset = 0;
                    }
                }
            }
            false
        }
        ITEM_TYPE_COMBO => false,
        ITEM_TYPE_LISTBOX => Item_ListBox_HandleKey(menus, ds, dc, item, key, down, false),
        ITEM_TYPE_TEXTSCROLL => Item_TextScroll_HandleKey(menus, ds, item, key, down, false),
        ITEM_TYPE_YESNO => Item_YesNo_HandleKey(menus, ds, dc, item, key),
        ITEM_TYPE_MULTI => Item_Multi_HandleKey(menus, ds, dc, item, key),
        ITEM_TYPE_OWNERDRAW => Item_OwnerDraw_HandleKey(menus, ds, dc, Some(item), key),
        ITEM_TYPE_BIND => Item_Bind_HandleKey(menus, ds, dc, item, key, down),
        ITEM_TYPE_SLIDER => Item_Slider_HandleKey(menus, ds, dc, item, key, down),
        _ => false,
    }
}

/// Raven `Item_Text_AutoWrapped_Paint` — paint `item`'s display text,
/// word-wrapping at its window width by re-measuring the growing line on
/// every character (Raven's O(n^2) walk, kept as-is).
///
/// PORT-NOTE: Raven walks `char *p`/`char buff[2048]` as raw bytes; this
/// walks `string_to_latin1(textPtr)`'s bytes the same way, decoding back to
/// `&str` only at the `DC->` call sites (Latin-1 discipline at the
/// trait-call byte seam). The comment "(this will break widechar languages)"
/// is Raven's own; preserved as-is (no Unicode-aware rewrap here).
/// Source: `oracle/codemp/ui/ui_shared.c:4828-4909`
pub fn Item_Text_AutoWrapped_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) {
    let it = menus.item(item);
    let textPtr = if let Some(t) = it.text.clone() {
        t
    } else if let Some(cvar) = it.cvar.clone() {
        dc.getCVarString(&cvar, 2048)
    } else {
        return;
    };

    // string reference
    let textPtr = if let Some(rest) = textPtr.strip_prefix('@') {
        dc.SP_GetStringTextString(rest, 2048).unwrap_or_default()
    } else {
        textPtr
    };

    if textPtr.is_empty() {
        return;
    }

    let mut color = vec4_t::default();
    Item_TextColor(menus, ds, dc, item, &mut color);

    let (textscale, iMenuFont, rectW, textalignment, textalignx, textStyle) = {
        let it = menus.item(item);
        (
            it.textscale,
            it.iMenuFont,
            it.window.rect.w,
            it.textalignment,
            it.textalignx,
            it.textStyle,
        )
    };
    let height = dc.textHeight(&textPtr, textscale, iMenuFont);
    let mut y = menus.item(item).textaligny;

    let p = string_to_latin1(&textPtr);
    let mut idx: usize = 0;
    // `buff` is Raven's `char buff[2048]` — the bytes up to its NUL — and `len`
    // its separate string length; a wrap zeroes `len` but leaves `buff` holding
    // the previous line, which the next `textWidth` still measures.
    // §19: Raven's fixed `buff[2048]` overruns on a >2047-byte segment (UB);
    // the unbounded `Vec` is the defined pick.
    let mut buff: Vec<u8> = Vec::new();
    let mut len: usize = 0;
    let mut newLine: usize = 0;
    let mut newLinePtr: usize = 0;
    let mut newLineWidth: c_int = 0;
    let mut textWidth: c_int = 0;

    loop {
        let ch = if idx < p.len() { p[idx] } else { 0u8 };
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == 0 {
            newLine = len;
            newLinePtr = idx + 1;
            newLineWidth = textWidth;
        }
        textWidth = dc.textWidth(&latin1_to_string(&buff), textscale, 0);
        if (newLine != 0 && (textWidth as f32) > rectW) || ch == b'\n' || ch == 0 {
            if len != 0 {
                let mut tx = menus.item(item).textRect.x;
                if textalignment == ITEM_ALIGN_LEFT {
                    tx = textalignx;
                } else if textalignment == ITEM_ALIGN_RIGHT {
                    tx = textalignx - newLineWidth as f32;
                } else if textalignment == ITEM_ALIGN_CENTER {
                    tx = textalignx - (newLineWidth / 2) as f32;
                }
                let mut ty = y;
                let window = menus.item(item).window.clone();
                ToWindowCoords(&mut tx, &mut ty, &window);
                {
                    let it = menus.item_mut(item);
                    it.textRect.x = tx;
                    it.textRect.y = ty;
                }

                // Raven `buff[newLine] = '\0'` — the NUL moves back to the
                // wrap point and stays there until the next character is written.
                buff.truncate(newLine);
                let line = latin1_to_string(&buff);
                dc.drawText(
                    tx, ty, textscale, color, &line, 0.0, 0, textStyle, iMenuFont,
                );
            }
            if ch == 0 {
                break;
            }
            y += height as f32 + 5.0;
            idx = newLinePtr;
            len = 0;
            newLine = 0;
            newLineWidth = 0;
            continue;
        }
        // Raven `buff[len++] = *p++; buff[len] = '\0';`
        buff.truncate(len);
        buff.push(ch);
        len += 1;
        idx += 1;
    }
}

/// Raven `Item_Text_Wrapped_Paint` — paint `item`'s display text split into
/// lines on `\r`, each spaced by the cached text height.
///
/// PORT-NOTE: see `Item_SetTextExtents` — `seLanguageModCount` threads in the
/// caller's `se_language.modificationCount` read (this crate cannot reach it
/// as cached state).
/// Source: `oracle/codemp/ui/ui_shared.c:4911-4959`
pub fn Item_Text_Wrapped_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    // now paint the text and/or any optional images
    // default to left
    let it = menus.item(item);
    let textPtr = if let Some(t) = it.text.clone() {
        t
    } else if let Some(cvar) = it.cvar.clone() {
        dc.getCVarString(&cvar, 1024)
    } else {
        return;
    };

    // string reference
    let textPtr = if let Some(rest) = textPtr.strip_prefix('@') {
        dc.SP_GetStringTextString(rest, 1024).unwrap_or_default()
    } else {
        textPtr
    };

    if textPtr.is_empty() {
        return;
    }

    let mut color = vec4_t::default();
    Item_TextColor(menus, ds, dc, item, &mut color);

    let mut width: c_int = 0;
    let mut height: c_int = 0;
    Item_SetTextExtents(
        menus,
        dc,
        item,
        &mut width,
        &mut height,
        Some(&textPtr),
        seLanguageModCount,
    );

    let it = menus.item(item);
    let x = it.textRect.x;
    let mut y = it.textRect.y;
    let textscale = it.textscale;
    let textStyle = it.textStyle;
    let iMenuFont = it.iMenuFont;

    // §19: Raven `strncpy`s each `\r`-segment into a fixed `char buff[1024]`,
    // which overruns on a >1023-byte segment (UB); slicing is the defined pick.
    let bytes = string_to_latin1(&textPtr);
    let mut start: usize = 0;
    loop {
        match bytes[start..].iter().position(|&b| b == b'\r') {
            Some(off) => {
                let p = start + off;
                let line = latin1_to_string(&bytes[start..p]);
                dc.drawText(x, y, textscale, color, &line, 0.0, 0, textStyle, iMenuFont);
                y += height as f32 + 2.0;
                start = p + 1;
            }
            None => break,
        }
    }
    let rest = latin1_to_string(&bytes[start..]);
    dc.drawText(x, y, textscale, color, &rest, 0.0, 0, textStyle, iMenuFont);
}

/// Raven `Menu_ScrollFeeder` — simulate a listbox scroll-key press on the
/// item in `menu` whose `special` matches `feeder`.
/// Source: `oracle/codemp/ui/ui_shared.c:7046-7056`
pub fn Menu_ScrollFeeder(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    feeder: c_int,
    down: bool,
) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };
    let items = menus.menu(menu).items.clone();
    for id in items {
        if menus.item(id).special == feeder as f32 {
            let key = if down { A_CURSOR_DOWN } else { A_CURSOR_UP };
            Item_ListBox_HandleKey(menus, ds, dc, id, key, true, true);
            return;
        }
    }
}

/// Raven `Script_Close` — close a menu group (or all menus) by name.
/// Source: `oracle/codemp/ui/ui_shared.c:1642-1657`
pub fn Script_Close(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    _item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        if stricmp_eq(&name, "all") {
            Menus_CloseAll(menus, dc);
        } else {
            Menus_CloseByName(menus, dc, &name);
        }
    }

    true
}

/// Raven `Script_Transition` — animate a named item group from one rect to
/// another.
/// Source: `oracle/codemp/ui/ui_shared.c:1808-1824`
pub fn Script_Transition(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    let mut rectFrom = RectDef::default();
    let mut rectTo = RectDef::default();
    let mut time = 0;
    let mut amt = 0.0f32;

    if String_Parse(args, &mut name)
        && Rect_Parse(args, &mut rectFrom)
        && Rect_Parse(args, &mut rectTo)
        && Int_Parse(args, &mut time)
        && Float_Parse(args, &mut amt)
    {
        if let Some(parent) = menus.item(item).parent {
            Menu_TransitionItemByName(menus, dc, parent, &name, Some(rectFrom), &rectTo, time, amt);
        }
    }

    true
}

/// Raven `Script_Scale` — scale every item in the named group about its own
/// center, transitioning each to its scaled rect.
/// Source: `oracle/codemp/ui/ui_shared.c:1895-1937`
pub fn Script_Scale(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        // Is is specifying a cvar to get the item name from?
        if let Some(rest) = name.strip_prefix('*') {
            name = dc.getCVarString(rest, 1024);
        }

        let parent = menus.item(item).parent;
        let count = match parent {
            Some(p) => Menu_ItemsMatchingGroup(menus, dc, p, &name),
            None => 0,
        };

        let mut scale = 0.0f32;
        if Float_Parse(args, &mut scale) {
            if let Some(parent) = parent {
                for j in 0..count {
                    if let Some(itemFound) = Menu_GetMatchingItemByNumber(menus, parent, j, &name) {
                        let rectSrc = menus.item(itemFound).window.rect;
                        let h = rectSrc.h * scale;
                        let w = rectSrc.w * scale;
                        let rectTo = RectDef {
                            w,
                            h,
                            x: rectSrc.x + (rectSrc.h - h) / 2.0,
                            y: rectSrc.y + (rectSrc.w - w) / 2.0,
                        };
                        Menu_TransitionItemByName(menus, dc, parent, &name, None, &rectTo, 1, 1.0);
                    }
                }
            }
        }
    }

    true
}

/// Raven `Script_Orbit` — orbit the named item group around a center point.
/// Source: `oracle/codemp/ui/ui_shared.c:1939-1954`
pub fn Script_Orbit(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let mut cx = 0.0f32;
        let mut cy = 0.0f32;
        let mut time = 0;
        if Float_Parse(args, &mut x)
            && Float_Parse(args, &mut y)
            && Float_Parse(args, &mut cx)
            && Float_Parse(args, &mut cy)
            && Int_Parse(args, &mut time)
        {
            if let Some(parent) = menus.item(item).parent {
                Menu_OrbitItemByName(menus, dc, parent, &name, x, y, cx, cy, time);
            }
        }
    }

    true
}

/// Raven `Script_Transition2` — animate a named item group to a rect parsed
/// with `ParseRect`/`COM_ParseFloat` rather than `Rect_Parse`/`Float_Parse`.
///
/// PORT-NOTE: `!COM_ParseFloat(...)` (true on parse *success*, Raven's
/// reversed polarity) is this file's local `parseFloatOrFail` twin (see
/// `ParseRect`'s doc), same call.
/// Source: `oracle/codemp/ui/ui_shared.c:2027-2047`
pub fn Script_Transition2(
    menus: &mut MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    args: &mut &str,
) -> bool {
    let mut name = String::new();
    if String_Parse(args, &mut name) {
        let mut rectTo = RectDef::default();
        let mut time = 0;
        let mut amt = 0.0f32;

        if ParseRect(args, &mut rectTo)
            && Int_Parse(args, &mut time)
            && !parseFloatOrFail(args, &mut amt)
        {
            if let Some(parent) = menus.item(item).parent {
                Menu_TransitionItemByName(menus, dc, parent, &name, None, &rectTo, time, amt);
            }
        } else {
            dc.Print(&format!(
                "^3WARNING: Script_Transition2: error parsing '{}'\n",
                name
            ));
        }
    }

    true
}

/// Raven `Item_Text_Paint` — paint an item's display text (and optional
/// second line), routing through the wrapped/auto-wrapped variants first.
///
/// PORT-NOTE: `seLanguageModCount` threads in the caller's
/// `se_language.modificationCount` read (see `Item_SetTextExtents`'s
/// doc) — added beyond the packet's literal C signature because this fn
/// calls `Item_SetTextExtents`/`Item_Text_Wrapped_Paint`, both of which need
/// it.
/// Source: `oracle/codemp/ui/ui_shared.c:4961-5017`
pub fn Item_Text_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    let flags = menus.item(item).window.flags;
    if flags & WINDOW_WRAPPED != 0 {
        Item_Text_Wrapped_Paint(menus, ds, dc, item, seLanguageModCount);
        return;
    }
    if flags & WINDOW_AUTOWRAPPED != 0 {
        Item_Text_AutoWrapped_Paint(menus, ds, dc, item);
        return;
    }

    let it = menus.item(item);
    let mut textPtr = match it.text.clone() {
        Some(t) => t,
        None => match it.cvar.clone() {
            Some(cvar) => dc.getCVarString(&cvar, 1024),
            None => return,
        },
    };

    // string reference
    if let Some(rest) = textPtr.strip_prefix('@') {
        textPtr = dc.SP_GetStringTextString(rest, 1024).unwrap_or_default();
    }

    // this needs to go here as it sets extents for cvar types as well
    let mut width: c_int = 0;
    let mut height: c_int = 0;
    Item_SetTextExtents(
        menus,
        dc,
        item,
        &mut width,
        &mut height,
        Some(&textPtr),
        seLanguageModCount,
    );

    if textPtr.is_empty() {
        return;
    }

    let mut color = vec4_t::default();
    Item_TextColor(menus, ds, dc, item, &mut color);

    let (x, y, textscale, textStyle, iMenuFont) = {
        let it = menus.item(item);
        (
            it.textRect.x,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        )
    };
    dc.drawText(
        x, y, textscale, color, &textPtr, 0.0, 0, textStyle, iMenuFont,
    );

    // Is there a second line of text?
    // PORT-NOTE: `text2` stays `String` — Raven's NULL test differs only for a
    // non-NULL empty `text2`, which `ItemParse_text2` never stores.
    let text2 = menus.item(item).text2.clone();
    if !text2.is_empty() {
        let mut textPtr2 = text2;
        if let Some(rest) = textPtr2.strip_prefix('@') {
            textPtr2 = dc.SP_GetStringTextString(rest, 1024).unwrap_or_default();
        }
        let mut color2 = vec4_t::default();
        Item_TextColor(menus, ds, dc, item, &mut color2);
        let (x2, y2, textscale2, textStyle2, iMenuFont2) = {
            let it = menus.item(item);
            (
                it.textRect.x + it.text2alignx,
                it.textRect.y + it.text2aligny,
                it.textscale,
                it.textStyle,
                it.iMenuFont,
            )
        };
        dc.drawText(
            x2, y2, textscale2, color2, &textPtr2, 0.0, 0, textStyle2, iMenuFont2,
        );
    }
}

/// Raven `Item_Model_Paint` — render a model-preview item's ghoul2/static
/// model into its own mini scene.
///
/// PORT-NOTE (dead surface): the `#ifndef CGAME` guards throughout pick the
/// `ui` arm (this crate's only linkage so far — cgame's twin will
/// special-case them out when it lands, same as `Item_SetTextExtents`).
///
/// DEFERRED: the "moves datapad anim" block (`uiInfo.moveAnimTime`,
/// `uiInfo.movesBaseAnim`, the multi-part saber/knockdown animation state
/// machine, `UI_UpdateCharacterSkin`) reads/writes `uiInfo`, which lives on
/// `crates/mp/ui/src/world/ui_world.rs`'s `UiWorld` (mp_ui-only) and is
/// unreachable from this host-agnostic crate — same shape as the
/// `ItemParse_cvarStrList` feeder-population DEFERRED.
/// Source: `oracle/codemp/ui/ui_shared.c:5709-5778`
///
/// DEFERRED: `UI_SaberDrawBlades`'s ported home
/// (`oracle/codemp/ui/ui_saber.c:952-1017`) takes `ctx: &mut UiContext`
/// (mp_ui-only per `tools/closure-prototype/out/ui/ported-signatures.txt`),
/// unreachable from this host-agnostic crate (no `mp_ui` dependency, no
/// `UiContext`/`DisplayContext` method carries the saber-blade draw). Needs a
/// `DisplayContext`-routed equivalent or a restructure that threads it back
/// through the host.
/// Source: `oracle/codemp/ui/ui_shared.c:5880-5883`
pub fn Item_Model_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) {
    // PORT-NOTE: `typeData.model()` stands in for Raven's unchecked
    // `(modelDef_t*)item->typeData` cast + NULL test (`listBox_mut` precedent).
    let modelPtr = match menus.item(item).typeData.model() {
        Some(m) => *m,
        None => return,
    };

    // DEFERRED: moves datapad anim block — see fn doc.

    let (rect, ghoul2, flags, asset) = {
        let it = menus.item(item);
        (it.window.rect, it.ghoul2, it.flags, it.asset)
    };

    // setup the refdef
    let mut refdef = refdef_t {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        fov_x: 0.0,
        fov_y: 0.0,
        vieworg: [0.0; 3],
        viewangles: [0.0; 3],
        viewaxis: [[0.0; 3]; 3],
        viewContents: 0,
        time: 0,
        rdflags: 0,
        areamask: [0; MAX_MAP_AREA_BYTES],
        text: [[0; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],
    };

    refdef.rdflags = RDF_NOWORLDMODEL;
    // `AxisClear( refdef.viewaxis )` — `native_math::qmath::AxisClear` is not
    // re-exported by `mp_qshared::shared::q_math` (and `native_math` is not a
    // dep of this crate), so its identity-basis body is written out here.
    VectorSet(&mut refdef.viewaxis[0], 1.0, 0.0, 0.0);
    VectorSet(&mut refdef.viewaxis[1], 0.0, 1.0, 0.0);
    VectorSet(&mut refdef.viewaxis[2], 0.0, 0.0, 1.0);
    let x = rect.x + 1.0;
    let y = rect.y + 1.0;
    let w = rect.w - 2.0;
    let h = rect.h - 2.0;

    refdef.x = (x * ds.xscale) as i32;
    refdef.y = (y * ds.yscale) as i32;
    refdef.width = (w * ds.xscale) as i32;
    refdef.height = (h * ds.yscale) as i32;

    // Raven declares `mins`/`maxs` uninitialized; both branches below assign
    // them, so the bindings stay deferred (Rust definite-init artifact).
    let mut mins: vec3_t;
    let mut maxs: vec3_t;
    if !ghoul2.is_null() {
        // ghoul2 models don't have bounds, so we have to parse them.
        mins = modelPtr.g2mins;
        maxs = modelPtr.g2maxs;

        if mins == [0.0; 3] && maxs == [0.0; 3] {
            // we'll use defaults then I suppose.
            VectorSet(&mut mins, -16.0, -16.0, -24.0);
            VectorSet(&mut maxs, 16.0, 16.0, 32.0);
        }
    } else {
        let (dcMins, dcMaxs) = dc.modelBounds(asset);
        mins = dcMins;
        maxs = dcMaxs;
    }

    let mut origin: vec3_t = [0.0; 3];
    origin[2] = -0.5 * (mins[2] + maxs[2]);
    origin[1] = 0.5 * (mins[1] + maxs[1]);

    // calculate distance so the model nearly fills the box
    let len = 0.5 * (maxs[2] - mins[2]);
    origin[0] = len / 0.268; // len / tan( fov/2 )

    refdef.fov_x = if modelPtr.fov_x != 0.0 {
        modelPtr.fov_x
    } else {
        ((refdef.width as f32 / 640.0 * 90.0) as i32) as f32
    };
    refdef.fov_y = if modelPtr.fov_y != 0.0 {
        modelPtr.fov_y
    } else {
        // Raven's `atan2`/`tan`/`M_PI` chain is `double` from `fov_x / 360`
        // onward (only the `/ 360` stays `float`), rounded to `float` once here.
        let t = (f64::from(refdef.fov_x / 360.0) * PI_F64).tan();
        (f64::from(refdef.height).atan2(f64::from(refdef.width) / t) * (360.0 / PI_F64)) as f32
    };

    dc.clearScene();
    refdef.time = ds.realTime;

    // add the model
    // `memset( &ent, 0, sizeof(ent) )`.
    let mut ent = refEntity_t::zeroed();

    // use item storage to track
    let mut angles: vec3_t = [0.0; 3];
    let isAnySaber = flags & ITF_ISANYSABER != 0 && flags & ITF_ISCHARACTER == 0;
    if isAnySaber {
        // hack to put saber on it's side
        if modelPtr.rotationSpeed != 0 {
            VectorSet(
                &mut angles,
                modelPtr.angle as f32 + refdef.time as f32 / modelPtr.rotationSpeed as f32,
                0.0,
                90.0,
            );
        } else {
            VectorSet(&mut angles, modelPtr.angle as f32, 0.0, 90.0);
        }
    } else if modelPtr.rotationSpeed != 0 {
        VectorSet(
            &mut angles,
            0.0,
            modelPtr.angle as f32 + refdef.time as f32 / modelPtr.rotationSpeed as f32,
            0.0,
        );
    } else {
        VectorSet(&mut angles, 0.0, modelPtr.angle as f32, 0.0);
    }

    AnglesToAxis(angles, ent.axis.as_mut_ptr());

    if !ghoul2.is_null() {
        ent.ghoul2 = ghoul2;
        ent.radius = 1000.0;
        ent.customSkin = modelPtr.g2skin;

        ent.modelScale = modelPtr.g2scale;
        UI_ScaleModelAxis(&mut ent);
        if flags & ITF_ISCHARACTER != 0 {
            // PORT-NOTE: `ui_char_color_*` read live through `dc.getCVarValue`
            // instead of Raven's cached `vmCvar_t`s — see `Window_Paint`.
            ent.shaderRGBA[0] = (dc.getCVarValue("ui_char_color_red") as c_int) as u8;
            ent.shaderRGBA[1] = (dc.getCVarValue("ui_char_color_green") as c_int) as u8;
            ent.shaderRGBA[2] = (dc.getCVarValue("ui_char_color_blue") as c_int) as u8;
            ent.shaderRGBA[3] = 255;
        }
        if flags & ITF_ISANYSABER != 0 {
            // DEFERRED: UI_SaberDrawBlades( item, origin, angles ) — see fn doc.
        }
    } else {
        ent.hModel = asset;
    }
    ent.origin = origin;
    ent.oldorigin = ent.origin;

    // Set up lighting
    ent.lightingOrigin = origin;
    ent.renderfx = RF_LIGHTING_ORIGIN | RF_NOSHADOW;

    dc.addRefEntityToScene(&ent);
    dc.renderScene(&refdef);
}

/// Raven `Menu_HandleMouseMove` — set mouse-over/focus state for `menu`'s
/// items under the cursor.
///
/// PORT-NOTE (dead surface): Raven's `#ifdef _XBOX return; #endif` guard at
/// the top (the whole fn is a no-op on that dead platform) is dropped, same
/// treatment as this file's other `_XBOX` arms.
/// Source: `oracle/codemp/ui/ui_shared.c:7126-7210`
pub fn Menu_HandleMouseMove(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: Option<MenuId>,
    x: f32,
    y: f32,
) {
    let menu = match menu {
        Some(m) => m,
        None => return,
    };

    if menus.menu(menu).window.flags & (WINDOW_VISIBLE | WINDOW_FORCED) == 0 {
        return;
    }

    if menus.itemCapture.is_some() {
        // Item_MouseMove(itemCapture, x, y);
        return;
    }

    if menus.g_waitingForKey || menus.g_editingField {
        return;
    }

    let mut focusSet = false;
    let items = menus.menu(menu).items.clone();

    // FIXME: this is the whole issue of focus vs. mouse over..
    // need a better overall solution as i don't like going through everything twice
    for pass in 0..2 {
        for &itemId in &items {
            let (itFlags, disabled, cvarFlags) = {
                let it = menus.item(itemId);
                (it.window.flags, it.disabled, it.cvarFlags)
            };

            if itFlags & (WINDOW_VISIBLE | WINDOW_FORCED) == 0 {
                continue;
            }

            if disabled {
                continue;
            }

            // items can be enabled and disabled based on cvars
            if cvarFlags & (CVAR_ENABLE | CVAR_DISABLE) != 0
                && !Item_EnableShowViaCvar(menus, dc, itemId, CVAR_ENABLE)
            {
                continue;
            }

            if cvarFlags & (CVAR_SHOW | CVAR_HIDE) != 0
                && !Item_EnableShowViaCvar(menus, dc, itemId, CVAR_SHOW)
            {
                continue;
            }

            let rect = menus.item(itemId).window.rect;
            if Rect_ContainsPoint(Some(&rect), x, y) {
                if pass == 1 {
                    let overItem = itemId;
                    let (overType, overHasText) = {
                        let it = menus.item(overItem);
                        (it.r#type, it.text.is_some())
                    };
                    if overType == ITEM_TYPE_TEXT && overHasText {
                        let overRect = menus.item(overItem).window.rect;
                        if !Rect_ContainsPoint(Some(&overRect), x, y) {
                            continue;
                        }
                    }
                    // if we are over an item
                    if IsVisible(menus.item(overItem).window.flags) {
                        // different one
                        Item_MouseEnter(menus, dc, Some(overItem), x, y);
                        // Item_SetMouseOver(overItem, qtrue);

                        // if item is not a decoration see if it can take focus
                        if !focusSet {
                            focusSet = Item_SetFocus(menus, ds, dc, overItem, x, y);
                        }
                    }
                }
            } else if itFlags & WINDOW_MOUSEOVER != 0 {
                Item_MouseLeave(menus, dc, Some(itemId));
                Item_SetMouseOver(menus, Some(itemId), false);
            }
        }
    }
}

// `Menu_New` — ui_shared.c:9817-9827.
//
// DEFERRED: Menu_New — its body is `Menu_Init` + `Menu_Parse` + (on success)
// `Menu_PostParse`; `Menu_Parse` stays `// DEFERRED:` at its own site
// (ui_shared.c:9779-9810) because the `keywordHash_t` menu-keyword dispatch
// it drives isn't ported, so this parse entrypoint has no reachable body and
// no caller in the ported tree.
// Source: `oracle/codemp/ui/ui_shared.c:9817-9827`

/// Raven `Menu_SetPrevCursorItem` — move `menu`'s cursor to the previous
/// focusable item, wrapping once; restores the original cursor if nothing
/// takes focus.
/// Source: `oracle/codemp/ui/ui_shared.c:4329-4358`
pub fn Menu_SetPrevCursorItem(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
) -> Option<ItemId> {
    let mut wrapped = false;
    let oldCursor = menus.menu(menu).cursorItem;
    let itemCount = menus.menu(menu).items.len() as c_int;

    if menus.menu(menu).cursorItem < 0 {
        menus.menu_mut(menu).cursorItem = itemCount - 1;
        wrapped = true;
    }

    while menus.menu(menu).cursorItem > -1 {
        menus.menu_mut(menu).cursorItem -= 1;
        if menus.menu(menu).cursorItem < 0 {
            if wrapped {
                break;
            }
            wrapped = true;
            menus.menu_mut(menu).cursorItem = itemCount - 1;
        }

        let cursorItem = menus.menu(menu).cursorItem;
        // Raven tolerates an entry `cursorItem >= itemCount`: it reads a NULL
        // slot, `Item_SetFocus` rejects it, and the loop keeps going.
        let item = match menus.menu(menu).items.get(cursorItem as usize) {
            Some(&i) => i,
            None => continue,
        };
        if Item_SetFocus(menus, ds, dc, item, ds.cursorx as f32, ds.cursory as f32) {
            let rect = menus.item(item).window.rect;
            Menu_HandleMouseMove(menus, ds, dc, Some(menu), rect.x + 1.0, rect.y + 1.0);
            return Some(item);
        }
    }
    menus.menu_mut(menu).cursorItem = oldCursor;
    None
}

/// Raven `Menu_SetNextCursorItem` — move `menu`'s cursor to the next
/// focusable item, wrapping once; restores the original cursor if nothing
/// takes focus.
///
/// PORT-NOTE (restructured): Raven's post-wrap pass reads the NULL slot at
/// `items[itemCount]` (in bounds while `itemCount < MAX_MENUITEMS`),
/// `Item_SetFocus` rejects NULL, and the loop test then exits — so the
/// `break` is behaviorally identical; porting-rules §19 covers only the
/// `itemCount == MAX_MENUITEMS` corner, where Raven's read is out of bounds.
/// Source: `oracle/codemp/ui/ui_shared.c:4360-4387`
pub fn Menu_SetNextCursorItem(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    menu: MenuId,
) -> Option<ItemId> {
    let mut wrapped = false;
    let oldCursor = menus.menu(menu).cursorItem;
    let itemCount = menus.menu(menu).items.len() as c_int;

    if menus.menu(menu).cursorItem == -1 {
        menus.menu_mut(menu).cursorItem = 0;
        wrapped = true;
    }

    while menus.menu(menu).cursorItem < itemCount {
        menus.menu_mut(menu).cursorItem += 1;
        if menus.menu(menu).cursorItem >= itemCount {
            if !wrapped {
                wrapped = true;
                menus.menu_mut(menu).cursorItem = 0;
            } else {
                // See the restructure PORT-NOTE above.
                break;
            }
        }

        let cursorItem = menus.menu(menu).cursorItem;
        let item = menus.menu(menu).items[cursorItem as usize];
        if Item_SetFocus(menus, ds, dc, item, ds.cursorx as f32, ds.cursory as f32) {
            let rect = menus.item(item).window.rect;
            Menu_HandleMouseMove(menus, ds, dc, Some(menu), rect.x + 1.0, rect.y + 1.0);
            return Some(item);
        }
    }
    menus.menu_mut(menu).cursorItem = oldCursor;
    None
}

/// Raven `Item_TextField_Paint` — paint a text-field item: its label via
/// `Item_Text_Paint`, then the (optionally `@`-string-referenced) cvar value,
/// drawn with a blink cursor when focused and mid-edit.
///
/// PORT-NOTE: `seLanguageModCount` threads in for the `Item_Text_Paint` call,
/// same shape as that fn's own doc note.
///
/// PORT-NOTE: the C `buff + editPtr->paintOffset` byte-offset slice is a
/// Latin-1 byte cut, not a `char` cut (native_string dictionary); reproduced
/// via the `string_to_latin1`/`latin1_to_string` round-trip so a multi-byte
/// decoded character does not shift the cut point.
///
/// PORT-NOTE (UB pick, porting-rules §19): a NULL `item->typeData` (Raven
/// dereferences `editPtr->paintOffset` unconditionally) is treated as
/// `paintOffset == 0`.
/// Source: `oracle/codemp/ui/ui_shared.c:5024-5062`
pub fn Item_TextField_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);

    let mut buff = String::new();
    let cvar = menus.item(item).cvar.clone();
    if let Some(cvar) = cvar {
        buff = dc.getCVarString(&cvar, 1024);
        if buff.starts_with('@') {
            // string reference
            buff = dc
                .SP_GetStringTextString(&buff[1..], 1024)
                .unwrap_or_default();
        }
    }

    let parent = menus
        .item(item)
        .parent
        .expect("Item_TextField_Paint: item has no parent");
    let focusColor = menus.menu(parent).focusColor;
    let flags = menus.item(item).window.flags;

    let mut newColor: vec4_t = [0.0; 4];
    if flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            &mut newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else {
        newColor = menus.item(item).window.foreColor;
    }

    let hasText = menus
        .item(item)
        .text
        .as_deref()
        .is_some_and(|t| !t.is_empty());
    let offset = if hasText { 8.0 } else { 0.0 };

    let editingField = menus.g_editingField;
    let paintOffset = menus
        .item(item)
        .typeData
        .editField()
        .map(|e| e.paintOffset)
        .unwrap_or(0);

    let buffBytes = string_to_latin1(&buff);
    let start = (paintOffset.max(0) as usize).min(buffBytes.len());
    let visible = latin1_to_string(&buffBytes[start..]);

    let it = menus.item(item);
    let (textRectX, textRectY, textRectW, textscale, textStyle, iMenuFont, cursorPos, windowRectW) = (
        it.textRect.x,
        it.textRect.y,
        it.textRect.w,
        it.textscale,
        it.textStyle,
        it.iMenuFont,
        it.cursorPos,
        it.window.rect.w,
    );

    if flags & WINDOW_HASFOCUS != 0 && editingField {
        let cursor = if dc.getOverstrikeMode() { b'_' } else { b'|' };
        dc.drawTextWithCursor(
            textRectX + textRectW + offset,
            textRectY,
            textscale,
            newColor,
            &visible,
            cursorPos - paintOffset,
            cursor,
            windowRectW as c_int,
            textStyle,
            iMenuFont,
        );
    } else {
        dc.drawText(
            textRectX + textRectW + offset,
            textRectY,
            textscale,
            newColor,
            &visible,
            0.0,
            windowRectW as c_int,
            textStyle,
            iMenuFont,
        );
    }
}

/// Raven `Item_YesNo_Paint` — paint a yes/no item: focus/blink color, then
/// the localized YES/NO string (label first if the item has text).
///
/// PORT-NOTE: the `#ifdef _XBOX` arm (an `xoffset`-adjusted `drawText` call)
/// is dead surface on this port's only build target and is dropped, matching
/// this file's other `_XBOX` arms.
/// Source: `oracle/codemp/ui/ui_shared.c:5064-5126`
pub fn Item_YesNo_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    let cvar = menus.item(item).cvar.clone();
    let value = match &cvar {
        Some(c) => dc.getCVarValue(c),
        None => 0.0,
    };

    let parent = menus
        .item(item)
        .parent
        .expect("Item_YesNo_Paint: item has no parent");
    let focusColor = menus.menu(parent).focusColor;
    let flags = menus.item(item).window.flags;

    let mut newColor: vec4_t = [0.0; 4];
    if flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            &mut newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else {
        newColor = menus.item(item).window.foreColor;
    }

    let sYES = dc
        .SP_GetStringTextString("MENUS_YES", 20)
        .unwrap_or_default();
    let sNO = dc
        .SP_GetStringTextString("MENUS_NO", 20)
        .unwrap_or_default();

    let invertYesNo = menus.item(item).invertYesNo;
    let yesnovalue = if invertYesNo != 0 {
        if value == 0.0 {
            &sYES
        } else {
            &sNO
        }
    } else if value != 0.0 {
        &sYES
    } else {
        &sNO
    };

    let hasText = menus.item(item).text.is_some();
    if hasText {
        Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);
        let it = menus.item(item);
        let (x, y, textscale, textStyle, iMenuFont) = (
            it.textRect.x + it.textRect.w + 8.0,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        );
        dc.drawText(
            x, y, textscale, newColor, yesnovalue, 0.0, 0, textStyle, iMenuFont,
        );
    } else {
        let it = menus.item(item);
        let (x, y, textscale, textStyle, iMenuFont) = (
            it.textRect.x,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        );
        dc.drawText(
            x, y, textscale, newColor, yesnovalue, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `Item_Multi_Paint` — paint a multi-choice item: focus/blink color,
/// then its current setting text (resolved through the `@` string-table or
/// `*` cvar-name indirection `Item_Multi_Setting` can return).
///
/// PORT-NOTE: the `#ifdef _XBOX` arm is dead surface, dropped as elsewhere in
/// this file.
/// Source: `oracle/codemp/ui/ui_shared.c:5128-5170`
pub fn Item_Multi_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    let parent = menus
        .item(item)
        .parent
        .expect("Item_Multi_Paint: item has no parent");
    let focusColor = menus.menu(parent).focusColor;
    let flags = menus.item(item).window.flags;

    let mut newColor: vec4_t = [0.0; 4];
    if flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            &mut newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else {
        newColor = menus.item(item).window.foreColor;
    }

    let mut text = Item_Multi_Setting(menus, dc, item);
    if let Some(rest) = text.strip_prefix('@') {
        // string reference
        text = dc
            .SP_GetStringTextString(rest, MAX_STRING_CHARS)
            .unwrap_or_default();
    } else if let Some(rest) = text.strip_prefix('*') {
        // Is is specifying a cvar to get the item name from?
        text = dc.getCVarString(rest, MAX_STRING_CHARS);
    }

    let hasText = menus.item(item).text.is_some();
    if hasText {
        Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);
        let it = menus.item(item);
        let (x, y, textscale, textStyle, iMenuFont) = (
            it.textRect.x + it.textRect.w + 8.0,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        );
        dc.drawText(
            x, y, textscale, newColor, &text, 0.0, 0, textStyle, iMenuFont,
        );
    } else {
        let it = menus.item(item);
        let (x, y, textscale, textStyle, iMenuFont) = (
            it.textRect.x + it.xoffset as f32,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        );
        dc.drawText(
            x, y, textscale, newColor, &text, 0.0, 0, textStyle, iMenuFont,
        );
    }
}

/// Raven `Item_Slider_Paint` — paint a slider item: focus/blink color, label
/// (if any), the slider bar, then the thumb at its current value position.
/// Source: `oracle/codemp/ui/ui_shared.c:5443-5473`
pub fn Item_Slider_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    let cvar = menus.item(item).cvar.clone();
    // PORT-NOTE: Raven computes `value` here but never reads it afterward
    // (ui_shared.c:5448-5473) — kept for parity as an unused binding.
    let _value = match &cvar {
        Some(c) => dc.getCVarValue(c),
        None => 0.0,
    };

    let parent = menus
        .item(item)
        .parent
        .expect("Item_Slider_Paint: item has no parent");
    let focusColor = menus.menu(parent).focusColor;
    let flags = menus.item(item).window.flags;

    let mut newColor: vec4_t = [0.0; 4];
    if flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            &mut newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else {
        newColor = menus.item(item).window.foreColor;
    }

    let y = menus.item(item).window.rect.y;
    let hasText = menus.item(item).text.is_some();
    let mut x;
    if hasText {
        Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);
        let it = menus.item(item);
        x = it.textRect.x + it.textRect.w + 8.0;
    } else {
        x = menus.item(item).window.rect.x;
    }

    dc.setColor(Some(newColor));
    let sliderBar = ds.Assets.sliderBar;
    dc.drawHandlePic(x, y, SLIDER_WIDTH, SLIDER_HEIGHT, sliderBar);

    x = Item_Slider_ThumbPosition(menus, dc, item);
    let sliderThumb = ds.Assets.sliderThumb;
    dc.drawHandlePic(
        x - (SLIDER_THUMB_WIDTH / 2.0),
        y - 2.0,
        SLIDER_THUMB_WIDTH,
        SLIDER_THUMB_HEIGHT,
        sliderThumb,
    );
}

/// Raven `Item_Bind_Paint` — paint a key-bind item: focus color (red when
/// this is the item awaiting a new binding), label, then the bound key
/// name(s), shrinking the scale until it fits on-screen.
///
/// PORT-NOTE (UB pick, porting-rules §19): Raven calls
/// `BindingFromName(item->cvar)` unconditionally once `item->text` is set,
/// even though `item->cvar` can independently be NULL (`Q_stricmp(NULL, …)`
/// would crash). Picking the defined empty-string reading for that case.
/// Source: `oracle/codemp/ui/ui_shared.c:5475-5546`
pub fn Item_Bind_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    seLanguageModCount: c_int,
) {
    let maxChars = menus
        .item(item)
        .typeData
        .editField()
        .map(|e| e.maxPaintChars)
        .unwrap_or(0);

    let cvar = menus.item(item).cvar.clone();
    let value = match &cvar {
        Some(c) => dc.getCVarValue(c),
        None => 0.0,
    };

    let parent = menus
        .item(item)
        .parent
        .expect("Item_Bind_Paint: item has no parent");
    let focusColor = menus.menu(parent).focusColor;
    let flags = menus.item(item).window.flags;

    let mut newColor: vec4_t = [0.0; 4];
    if flags & WINDOW_HASFOCUS != 0 {
        let lowLight: vec4_t = if menus.g_bindItem == Some(item) {
            [0.8 * 1.0, 0.8 * 0.0, 0.8 * 0.0, 0.8 * 1.0]
        } else {
            [
                0.8 * focusColor[0],
                0.8 * focusColor[1],
                0.8 * focusColor[2],
                0.8 * focusColor[3],
            ]
        };
        LerpColor(
            focusColor,
            lowLight,
            &mut newColor,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else {
        newColor = menus.item(item).window.foreColor;
    }

    let hasText = menus.item(item).text.is_some();
    if hasText {
        Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);
        BindingFromName(menus, dc, cvar.as_deref().unwrap_or(""));

        let mut textScale = menus.item(item).textscale;
        let iMenuFont = menus.item(item).iMenuFont;
        let g_nameBind1 = menus.g_nameBind1.clone();
        let mut textWidth = dc.textWidth(&g_nameBind1, textScale, iMenuFont) as f32;
        let it = menus.item(item);
        let startingXPos = (it.textRect.x + it.textRect.w + 8.0) as c_int;

        while (startingXPos as f32 + textWidth) >= SCREEN_WIDTH as f32 {
            textScale -= 0.05;
            textWidth = dc.textWidth(&g_nameBind1, textScale, iMenuFont) as f32;
        }

        let itemTextscale = menus.item(item).textscale;
        let mut yAdj = 0;
        if textScale != itemTextscale {
            let textHeight = dc.textHeight(&g_nameBind1, itemTextscale, iMenuFont);
            yAdj = textHeight - dc.textHeight(&g_nameBind1, textScale, iMenuFont);
        }

        let textRectY = menus.item(item).textRect.y;
        let textStyle = menus.item(item).textStyle;
        dc.drawText(
            startingXPos as f32,
            textRectY + yAdj as f32,
            textScale,
            newColor,
            &g_nameBind1,
            0.0,
            maxChars,
            textStyle,
            iMenuFont,
        );
    } else {
        let it = menus.item(item);
        let (textRectX, textRectY, textscale, textStyle, iMenuFont) = (
            it.textRect.x,
            it.textRect.y,
            it.textscale,
            it.textStyle,
            it.iMenuFont,
        );
        // PORT-NOTE: Raven's `(value != 0) ? "FIXME" : "FIXME"` — both arms
        // are the literal string `"FIXME"` in the oracle source; transcribed
        // verbatim (dead ternary in the oracle, not a translation gap).
        let _ = value;
        dc.drawText(
            textRectX, textRectY, textscale, newColor, "FIXME", 0.0, maxChars, textStyle, iMenuFont,
        );
    }
}

/// Raven `Item_OwnerDraw_Paint` — paint an owner-draw item through the
/// host's `ownerDrawItem` callback, resolving fade, per-value color ranges,
/// focus/blink/disabled color, and label offset first.
///
/// PORT-NOTE: Raven's `if (DC->ownerDrawItem)`/`DC->getValue` fn-pointer
/// null-checks are dropped — `DisplayContext::ownerDrawItem`/`getValue` are
/// non-optional trait methods, so the guards are always true.
/// Source: `oracle/codemp/ui/ui_shared.c:6370-6430`
pub fn Item_OwnerDraw_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
    seLanguageModCount: c_int,
) {
    let item = match item {
        Some(i) => i,
        None => return,
    };

    let parent = menus
        .item(item)
        .parent
        .expect("Item_OwnerDraw_Paint: item has no parent");
    let (fadeClamp, fadeCycle, fadeAmount, focusColor, disableColor) = {
        let m = menus.menu(parent);
        (
            m.fadeClamp,
            m.fadeCycle,
            m.fadeAmount,
            m.focusColor,
            m.disableColor,
        )
    };

    {
        let it = menus.item_mut(item);
        Fade(
            ds,
            &mut it.window.flags,
            &mut it.window.foreColor[3],
            fadeClamp,
            &mut it.window.nextTime,
            fadeCycle,
            true,
            fadeAmount,
        );
    }

    let mut color = menus.item(item).window.foreColor;

    let numColors = menus.item(item).numColors;
    if numColors > 0 {
        let ownerDraw = menus.item(item).window.ownerDraw;
        let f = dc.getValue(ownerDraw);
        let it = menus.item(item);
        for i in 0..numColors as usize {
            let range = it.colorRanges[i];
            if f >= range.low && f <= range.high {
                color = range.color;
                break;
            }
        }
    }

    let flags = menus.item(item).window.flags;
    let textStyle = menus.item(item).textStyle;
    if flags & WINDOW_HASFOCUS != 0 {
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * focusColor[i];
        }
        LerpColor(
            focusColor,
            lowLight,
            &mut color,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    } else if textStyle == ITEM_TEXTSTYLE_BLINK && (ds.realTime / BLINK_DIVISOR) & 1 == 0 {
        let foreColor = menus.item(item).window.foreColor;
        let mut lowLight: vec4_t = [0.0; 4];
        for i in 0..4 {
            lowLight[i] = 0.8 * foreColor[i];
        }
        LerpColor(
            foreColor,
            lowLight,
            &mut color,
            0.5 + 0.5 * ((ds.realTime / PULSE_DIVISOR) as f32).sin(),
        );
    }

    if menus.item(item).disabled {
        color = disableColor;
    }

    let cvarFlags = menus.item(item).cvarFlags;
    if cvarFlags & (CVAR_ENABLE | CVAR_DISABLE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_ENABLE)
    {
        color = disableColor;
    }

    let textIsSome = menus.item(item).text.is_some();
    if textIsSome {
        Item_Text_Paint(menus, ds, dc, item, seLanguageModCount);
        let hasNonEmptyText = menus
            .item(item)
            .text
            .as_deref()
            .is_some_and(|t| !t.is_empty());
        let it = menus.item(item);
        let (
            textRectX,
            textRectW,
            windowY,
            windowW,
            windowH,
            textaligny,
            ownerDraw,
            ownerDrawFlags,
            alignment,
            special,
            textscale,
            background,
            iMenuFont,
        ) = (
            it.textRect.x,
            it.textRect.w,
            it.window.rect.y,
            it.window.rect.w,
            it.window.rect.h,
            it.textaligny,
            it.window.ownerDraw,
            it.window.ownerDrawFlags,
            it.alignment,
            it.special,
            it.textscale,
            it.window.background,
            it.iMenuFont,
        );
        let x = if hasNonEmptyText {
            // +8 is an offset kludge to properly align owner draw items that
            // have text combined with them
            textRectX + textRectW + 8.0
        } else {
            textRectX + textRectW
        };
        dc.ownerDrawItem(
            x,
            windowY,
            windowW,
            windowH,
            0.0,
            textaligny,
            ownerDraw,
            ownerDrawFlags,
            alignment,
            special,
            textscale,
            color,
            background,
            textStyle,
            iMenuFont,
        );
    } else {
        let it = menus.item(item);
        dc.ownerDrawItem(
            it.window.rect.x,
            it.window.rect.y,
            it.window.rect.w,
            it.window.rect.h,
            it.textalignx,
            it.textaligny,
            it.window.ownerDraw,
            it.window.ownerDrawFlags,
            it.alignment,
            it.special,
            it.textscale,
            color,
            it.window.background,
            it.textStyle,
            it.iMenuFont,
        );
    }
}

/// Raven `Menus_ActivateByName` — activate the menu named `p`, pushing the
/// previously-focused menu onto the open-menu stack; clears focus on every
/// other menu.
///
/// PORT-NOTE (UB pick, porting-rules §19): a NULL `window.name`
/// (`Q_stricmp(NULL, p)` crashes in Raven) is treated as "never matches".
/// Source: `oracle/codemp/ui/ui_shared.c:7096-7117`
pub fn Menus_ActivateByName(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    p: &str,
) -> Option<MenuId> {
    let mut m: Option<MenuId> = None;
    let focus = Menu_GetFocused(menus);

    for i in 0..menus.menuCount() as usize {
        let id = MenuId::new(i);
        let name = menus.menu(id).window.name.clone();
        if name.as_deref().is_some_and(|n| stricmp_eq(n, p)) {
            m = Some(id);
            Menus_Activate(menus, dc, id);
            if menus.openMenuCount() < MAX_OPEN_MENUS as c_int {
                if let Some(focusMenu) = focus {
                    menus.menuStack.push(focusMenu);
                }
            }
        } else {
            menus.menu_mut(id).window.flags &= !WINDOW_HASFOCUS;
        }
    }

    Display_CloseCinematics(menus, dc);

    // Want to handle a mouse move on the new menu in case your already over
    // an item
    Menu_HandleMouseMove(menus, ds, dc, m, ds.cursorx as f32, ds.cursory as f32);

    m
}

/// Raven `Display_MouseMove` — with a `menu` handle, translate its window by
/// `(x, y)`; otherwise dispatch the mouse move to the focused popup menu, or
/// to every menu if none is a popup.
///
/// PORT-NOTE: the `#ifdef _XBOX` unconditional-`qtrue` early-return arm is
/// dead surface, dropped as elsewhere in this file.
/// Source: `oracle/codemp/ui/ui_shared.c:9873-9901`
pub fn Display_MouseMove(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    p: Option<MenuId>,
    x: c_int,
    y: c_int,
) -> bool {
    match p {
        None => {
            let menu = Menu_GetFocused(menus);
            if let Some(m) = menu {
                if menus.menu(m).window.flags & WINDOW_POPUP != 0 {
                    Menu_HandleMouseMove(menus, ds, dc, Some(m), x as f32, y as f32);
                    return true;
                }
            }
            for i in 0..menus.menuCount() as usize {
                let id = MenuId::new(i);
                Menu_HandleMouseMove(menus, ds, dc, Some(id), x as f32, y as f32);
            }
        }
        Some(m) => {
            menus.menu_mut(m).window.rect.x += x as f32;
            menus.menu_mut(m).window.rect.y += y as f32;
            Menu_UpdatePosition(menus, dc, Some(m));
        }
    }
    true
}

/// Raven `Menus_OpenByName` — open menu `p` by name.
/// Source: `oracle/codemp/ui/ui_shared.c:1523-1525`
pub fn Menus_OpenByName(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    p: &str,
) {
    Menus_ActivateByName(menus, ds, dc, p);
}

/// Raven `Item_TextField_HandleKey` — text/numeric-field key handler: cvar
/// insert/backspace/cursor navigation, then focus-switch keys (tab/arrows)
/// hand off to the next/previous cursor item.
///
/// PORT-NOTE: the C `char buff[2048]` byte buffer and its `memmove` splices
/// are reproduced with a fixed 2048-byte `Vec<u8>` of Latin-1 bytes
/// (`string_to_latin1`/`latin1_to_string` round-trip, `copy_within` for
/// `memmove` — translation dictionary) so the byte-offset arithmetic
/// (`cursorPos`, clamped `len`, `editPtr->maxChars`) matches exactly; the
/// trailing `!item->cvar` recheck inside the printable-char guard is always
/// false given the outer `if (item->cvar)` guard already returned, so it is
/// dropped.
///
/// PORT-NOTE (UB pick, porting-rules §19): Raven casts `item->typeData`
/// straight to `editFieldDef_t *` with no NULL check; a NULL `typeData` is
/// treated as a default-valued `EditFieldDef` (same pick as
/// `Item_TextField_Paint`), and a write-back through it silently no-ops.
///
/// porting-rules §19: with `maxChars == 0` Raven's `buff[cursorPos] = key`
/// and `memmove`s are unguarded past `buff[2048]` — a stack smash; the
/// fixed-size `Vec` panics instead of writing out of bounds.
///
/// Source: `oracle/codemp/ui/ui_shared.c:3664-3829`
pub fn Item_TextField_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: ItemId,
    key: c_int,
) -> bool {
    let mut key = key;

    let cvar = match menus.item(item).cvar.clone() {
        Some(c) => c,
        None => return false,
    };

    let mut editPtr = menus
        .item(item)
        .typeData
        .editField()
        .copied()
        .unwrap_or_default();

    let mut buff = string_to_latin1(&dc.getCVarString(&cvar, 2048));
    let mut len = buff.len() as c_int;
    buff.resize(2048, 0);
    if editPtr.maxChars != 0 && len > editPtr.maxChars {
        len = editPtr.maxChars;
    }

    let mut cursorPos = menus.item(item).cursorPos;

    let commit = |menus: &mut MenuSystem, cursorPos: c_int, editPtr: &EditFieldDef| {
        menus.item_mut(item).cursorPos = cursorPos;
        if let Some(ep) = menus.item_mut(item).typeData.editField_mut() {
            *ep = *editPtr;
        }
    };

    let nul_str = |buff: &[u8]| -> String {
        let nul = buff.iter().position(|&b| b == 0).unwrap_or(buff.len());
        latin1_to_string(&buff[..nul])
    };

    if key & K_CHAR_FLAG != 0 {
        key &= !K_CHAR_FLAG;

        // ctrl-h is backspace
        if key == (b'h' - b'a' + 1) as c_int {
            if cursorPos > 0 {
                let src = cursorPos as usize;
                let dst = (cursorPos - 1) as usize;
                let count = (len + 1 - cursorPos) as usize;
                buff.copy_within(src..src + count, dst);
                cursorPos -= 1;
                if cursorPos < editPtr.paintOffset {
                    editPtr.paintOffset -= 1;
                }
            }
            dc.setCVar(&cvar, &nul_str(&buff));
            commit(menus, cursorPos, &editPtr);
            return true;
        }

        // ignore any non printable chars
        if key < 32 {
            return true;
        }

        if menus.item(item).r#type == ITEM_TYPE_NUMERICFIELD
            && !(b'0' as c_int..=b'9' as c_int).contains(&key)
        {
            return false;
        }

        if !dc.getOverstrikeMode() {
            if len == MAX_EDITFIELD as c_int - 1
                || (editPtr.maxChars != 0 && len >= editPtr.maxChars)
            {
                return true;
            }
            let src = cursorPos as usize;
            let dst = (cursorPos + 1) as usize;
            let count = (len + 1 - cursorPos) as usize;
            buff.copy_within(src..src + count, dst);
        } else if editPtr.maxChars != 0 && cursorPos >= editPtr.maxChars {
            return true;
        }

        buff[cursorPos as usize] = key as u8;

        // rww - nul-terminate!
        if cursorPos + 1 < 2048 {
            buff[(cursorPos + 1) as usize] = 0;
        } else {
            buff[cursorPos as usize] = 0;
        }

        dc.setCVar(&cvar, &nul_str(&buff));

        if cursorPos < len + 1 {
            cursorPos += 1;
            if editPtr.maxPaintChars != 0 && cursorPos > editPtr.maxPaintChars {
                editPtr.paintOffset += 1;
            }
        }
        commit(menus, cursorPos, &editPtr);
    } else {
        if key == A_DELETE || key == A_KP_PERIOD {
            if cursorPos < len {
                let src = (cursorPos + 1) as usize;
                let dst = cursorPos as usize;
                let count = (len - cursorPos) as usize;
                buff.copy_within(src..src + count, dst);
                dc.setCVar(&cvar, &nul_str(&buff));
            }
            return true;
        }

        if key == A_CURSOR_RIGHT || key == A_KP_6 {
            if editPtr.maxPaintChars != 0 && cursorPos >= editPtr.maxPaintChars && cursorPos < len {
                cursorPos += 1;
                editPtr.paintOffset += 1;
                commit(menus, cursorPos, &editPtr);
                return true;
            }
            if cursorPos < len {
                cursorPos += 1;
            }
            commit(menus, cursorPos, &editPtr);
            return true;
        }

        if key == A_CURSOR_LEFT || key == A_KP_4 {
            if cursorPos > 0 {
                cursorPos -= 1;
            }
            if cursorPos < editPtr.paintOffset {
                editPtr.paintOffset -= 1;
            }
            commit(menus, cursorPos, &editPtr);
            return true;
        }

        if key == A_HOME || key == A_KP_7 {
            cursorPos = 0;
            editPtr.paintOffset = 0;
            commit(menus, cursorPos, &editPtr);
            return true;
        }

        if key == A_END || key == A_KP_1 {
            cursorPos = len;
            if cursorPos > editPtr.maxPaintChars {
                editPtr.paintOffset = len - editPtr.maxPaintChars;
            }
            commit(menus, cursorPos, &editPtr);
            return true;
        }

        if key == A_INSERT || key == A_KP_0 {
            let over = dc.getOverstrikeMode();
            dc.setOverstrikeMode(!over);
            return true;
        }
    }

    if key == A_TAB || key == A_CURSOR_DOWN || key == A_KP_2 {
        // switching fields so reset printed text of edit field
        Leaving_EditField(menus, item);
        menus.g_editingField = false;
        let parent = menus
            .item(item)
            .parent
            .expect("Item_TextField_HandleKey: item has no parent");
        let newItem = Menu_SetNextCursorItem(menus, ds, dc, parent);
        if let Some(newItem) = newItem {
            let t = menus.item(newItem).r#type;
            if t == ITEM_TYPE_EDITFIELD || t == ITEM_TYPE_NUMERICFIELD {
                menus.g_editItem = Some(newItem);
                menus.g_editingField = true;
            }
        }
    }

    if key == A_CURSOR_UP || key == A_KP_8 {
        // switching fields so reset printed text of edit field
        Leaving_EditField(menus, item);
        menus.g_editingField = false;
        let parent = menus
            .item(item)
            .parent
            .expect("Item_TextField_HandleKey: item has no parent");
        let newItem = Menu_SetPrevCursorItem(menus, ds, dc, parent);
        if let Some(newItem) = newItem {
            let t = menus.item(newItem).r#type;
            if t == ITEM_TYPE_EDITFIELD || t == ITEM_TYPE_NUMERICFIELD {
                menus.g_editItem = Some(newItem);
                menus.g_editingField = true;
            }
        }
    }

    if key == A_ENTER || key == A_KP_ENTER || key == A_ESCAPE {
        return false;
    }

    true
}

/// Helper factoring the four identical x/y/w/h transition-clamp blocks in
/// [`Item_Paint`]'s `WINDOW_INTRANSITION` handling (porting-rules §10 —
/// preserve behavior, not control-flow shape): step `cur` toward `target` by
/// `step`, clamping on overshoot. Returns `(newValue, reachedTarget)`, where
/// `reachedTarget` is Raven's per-axis `done++`.
/// Source: `oracle/codemp/ui/ui_shared.c:6477-6580`
fn Item_Paint_transitionAxis(cur: f32, target: f32, step: f32) -> (f32, bool) {
    if cur == target {
        (cur, true)
    } else if cur < target {
        let mut v = cur + step;
        if v > target {
            v = target;
            return (v, true);
        }
        (v, false)
    } else {
        let mut v = cur - step;
        if v < target {
            v = target;
            return (v, true);
        }
        (v, false)
    }
}

/// Raven `Item_Paint` — paint one item: orbit/transition its window, apply
/// ownerdraw visibility and cvar show/hide, paint desc-text on mouseover, the
/// window chrome, a debug-mode extents box, then dispatch to the type's paint
/// fn.
///
/// PORT-NOTE: the `#ifdef _TRANS3` model-transition block (`g2mins2`/
/// `g2maxs2`/fov transition) compiles into retail (`#define _TRANS3` at
/// `ui_shared.c:1694`) but is unreachable: `WINDOW_INTRANSITIONMODEL` is set
/// only by `Menu_Transition3ItemByName`, whose sole caller
/// `Script_Transition3` has no `commandList[]` entry (`ui_shared.c:2196-2228`;
/// same table gap as the `enable` note on `Script_Enable`).
/// DEFERRED: `_TRANS3` model-transition block — if `transition3` ever becomes
/// dispatchable, `ui_shared.c:6595-6838` must be ported.
/// Source: `oracle/codemp/ui/ui_shared.c:6592-6839`
///
/// PORT-NOTE: `item == NULL` becomes `item: Option<ItemId>` (same convention
/// as `Item_UpdatePosition`).
///
/// PORT-NOTE: `seLanguageModCount` threads in for the `Item_Text_Paint`-family
/// calls, same shape as those fns' own doc notes.
///
/// PORT-NOTE: `item->window.ownerDrawFlags && DC->ownerDrawVisible` checked a
/// fn-pointer for non-NULL; `DisplayContext::ownerDrawVisible` is always
/// implemented (DEC-36 D3), so only the flags half of the condition survives.
///
/// PORT-NOTE: `#ifndef _XBOX`/`#else` picks the non-`_XBOX` (`WINDOW_MOUSEOVER`)
/// arm — this is the PC/MP build.
///
/// PORT-NOTE: the `WINDOW_TIMEDVISIBLE` block is empty in Raven ("visibility
/// timing ( NOT implemented )" — `ui_shared.h:45`), so no code is emitted for
/// it here either.
/// Source: `oracle/codemp/ui/ui_shared.c:6433-7013`
pub fn Item_Paint(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    item: Option<ItemId>,
    seLanguageModCount: c_int,
) {
    let item = match item {
        Some(item) => item,
        None => return,
    };

    let parent = menus
        .item(item)
        .parent
        .expect("Item_Paint: item has no parent");

    // WINDOW_ORBITING
    if menus.item(item).window.flags & WINDOW_ORBITING != 0
        && ds.realTime > menus.item(item).window.nextTime
    {
        let offsetTime = menus.item(item).window.offsetTime;
        menus.item_mut(item).window.nextTime = ds.realTime + offsetTime;

        let rectClient = menus.item(item).window.rectClient;
        let rectEffects = menus.item(item).window.rectEffects;
        let w = rectClient.w / 2.0;
        let h = rectClient.h / 2.0;
        let rx = rectClient.x + w - rectEffects.x;
        let ry = rectClient.y + h - rectEffects.y;
        let a: f32 = (3.0 * PI_F64 / 180.0) as f32;
        let c = a.cos();
        let s = a.sin();
        menus.item_mut(item).window.rectClient.x = (rx * c - ry * s) + rectEffects.x - w;
        menus.item_mut(item).window.rectClient.y = (rx * s + ry * c) + rectEffects.y - h;

        Item_UpdatePosition(menus, dc, Some(item));
    }

    // WINDOW_INTRANSITION
    if menus.item(item).window.flags & WINDOW_INTRANSITION != 0
        && ds.realTime > menus.item(item).window.nextTime
    {
        let offsetTime = menus.item(item).window.offsetTime;
        menus.item_mut(item).window.nextTime = ds.realTime + offsetTime;

        let (rectClient, rectEffects, rectEffects2) = {
            let w = &menus.item(item).window;
            (w.rectClient, w.rectEffects, w.rectEffects2)
        };

        let (nx, dx) = Item_Paint_transitionAxis(rectClient.x, rectEffects.x, rectEffects2.x);
        let (ny, dy) = Item_Paint_transitionAxis(rectClient.y, rectEffects.y, rectEffects2.y);
        let (nw, dw) = Item_Paint_transitionAxis(rectClient.w, rectEffects.w, rectEffects2.w);
        let (nh, dh) = Item_Paint_transitionAxis(rectClient.h, rectEffects.h, rectEffects2.h);
        let mut done = 0;
        if dx {
            done += 1;
        }
        if dy {
            done += 1;
        }
        if dw {
            done += 1;
        }
        if dh {
            done += 1;
        }

        {
            let win = &mut menus.item_mut(item).window;
            win.rectClient.x = nx;
            win.rectClient.y = ny;
            win.rectClient.w = nw;
            win.rectClient.h = nh;
        }

        Item_UpdatePosition(menus, dc, Some(item));

        if done == 4 {
            menus.item_mut(item).window.flags &= !WINDOW_INTRANSITION;
        }
    }

    // DEFERRED: `_TRANS3` model-transition block (`WINDOW_INTRANSITIONMODEL`)
    // — compiled but unreachable: no `transition3` `commandList[]` entry ever
    // sets the flag (see the fn doc note).
    // Source: `oracle/codemp/ui/ui_shared.c:6595-6838`

    let ownerDrawFlags = menus.item(item).window.ownerDrawFlags;
    if ownerDrawFlags != 0 {
        if !dc.ownerDrawVisible(ownerDrawFlags) {
            menus.item_mut(item).window.flags &= !WINDOW_VISIBLE;
        } else {
            menus.item_mut(item).window.flags |= WINDOW_VISIBLE;
        }
    }

    let cvarFlags = menus.item(item).cvarFlags;
    if cvarFlags & (CVAR_SHOW | CVAR_HIDE) != 0
        && !Item_EnableShowViaCvar(menus, dc, item, CVAR_SHOW)
    {
        return;
    }

    // WINDOW_TIMEDVISIBLE — empty in Raven (not implemented).

    if menus.item(item).window.flags & WINDOW_VISIBLE == 0 {
        return;
    }

    // JLFMOUSE — `#ifndef _XBOX` arm (PC/MP build).
    if menus.item(item).window.flags & WINDOW_MOUSEOVER != 0 {
        let descText = menus.item(item).descText.clone();
        if !descText.is_empty() && !Display_KeyBindPending(menus) {
            let mut textPtr = descText;
            if let Some(rest) = textPtr.strip_prefix('@') {
                // porting-rules §19: Raven's `char temp[MAX_STRING_CHARS]` is
                // scoped to this `if` and `textPtr = temp` escapes it — the
                // later reads go through a dead stack pointer. Owned `String`
                // is the defined pick.
                textPtr = dc
                    .SP_GetStringTextString(rest, MAX_STRING_CHARS as usize)
                    .unwrap_or_default();
            }

            let mut color: vec4_t = [0.0; 4];
            Item_TextColor(menus, ds, dc, item, &mut color);

            let (parentDescScale, descAlignment, descX, descY, descColor) = {
                let p = menus.menu(parent);
                (p.descScale, p.descAlignment, p.descX, p.descY, p.descColor)
            };
            let fDescScaleCopy = if parentDescScale != 0.0 {
                parentDescScale
            } else {
                1.0
            };
            let mut fDescScale = fDescScaleCopy;
            let mut iYadj: c_int = 0;
            let textStyle = menus.item(item).textStyle;

            loop {
                let textWidth = dc.textWidth(&textPtr, fDescScale, FONT_SMALL2);

                let xPos = if descAlignment == ITEM_ALIGN_RIGHT {
                    descX - textWidth
                } else if descAlignment == ITEM_ALIGN_CENTER {
                    descX - (textWidth / 2)
                } else {
                    descX
                };

                if descAlignment == ITEM_ALIGN_CENTER && xPos + textWidth > SCREEN_WIDTH - 4 {
                    fDescScale -= 0.001;
                    continue;
                }

                if fDescScale != fDescScaleCopy {
                    let iOriginalTextHeight = dc.textHeight(&textPtr, fDescScaleCopy, FONT_MEDIUM);
                    iYadj = iOriginalTextHeight - dc.textHeight(&textPtr, fDescScale, FONT_MEDIUM);
                }

                dc.drawText(
                    xPos as f32,
                    (descY + iYadj) as f32,
                    fDescScale,
                    descColor,
                    &textPtr,
                    0.0,
                    0,
                    textStyle,
                    FONT_SMALL2,
                );
                break;
            }
        }
    }

    // paint the rect first..
    let (fadeAmount, fadeClamp, fadeCycle) = {
        let p = menus.menu(parent);
        (p.fadeAmount, p.fadeClamp, p.fadeCycle)
    };
    let mut window = menus.item(item).window.clone();
    Window_Paint(menus, ds, dc, &mut window, fadeAmount, fadeClamp, fadeCycle);
    menus.item_mut(item).window = window;

    // Draw box to show rectangle extents, in debug mode
    if menus.debugMode {
        let color: vec4_t = [0.0, 1.0, 0.0, 1.0];
        let rect = menus.item(item).window.rect;
        dc.drawRect(rect.x, rect.y, rect.w, rect.h, 1.0, color);
    }

    let itemType = menus.item(item).r#type;
    match itemType {
        ITEM_TYPE_OWNERDRAW => Item_OwnerDraw_Paint(menus, ds, dc, Some(item), seLanguageModCount),
        ITEM_TYPE_TEXT | ITEM_TYPE_BUTTON => {
            Item_Text_Paint(menus, ds, dc, item, seLanguageModCount)
        }
        ITEM_TYPE_RADIOBUTTON => {}
        ITEM_TYPE_CHECKBOX => {}
        ITEM_TYPE_EDITFIELD | ITEM_TYPE_NUMERICFIELD => {
            Item_TextField_Paint(menus, ds, dc, item, seLanguageModCount)
        }
        ITEM_TYPE_COMBO => {}
        ITEM_TYPE_LISTBOX => Item_ListBox_Paint(menus, ds, dc, item),
        ITEM_TYPE_TEXTSCROLL => Item_TextScroll_Paint(menus, ds, dc, item),
        ITEM_TYPE_MODEL => Item_Model_Paint(menus, ds, dc, item),
        ITEM_TYPE_YESNO => Item_YesNo_Paint(menus, ds, dc, item, seLanguageModCount),
        ITEM_TYPE_MULTI => Item_Multi_Paint(menus, ds, dc, item, seLanguageModCount),
        ITEM_TYPE_BIND => Item_Bind_Paint(menus, ds, dc, item, seLanguageModCount),
        ITEM_TYPE_SLIDER => Item_Slider_Paint(menus, ds, dc, item, seLanguageModCount),
        _ => {}
    }
}

/// Raven `Menu_PaintAll` — dispatch the active scroll-capture handler, then
/// paint every defined menu, then (debug mode) an FPS/cursor overlay.
///
/// PORT-NOTE (`captureFunc`): the closed `Scroll_*Func` pointer set
/// (`crate::shared::capture_func::CaptureFunc`) is dispatched by `match`;
/// `captureData` drops out (it was always `&scrollInfo`, which `MenuSystem`
/// already owns) per the `CaptureFunc` doc.
///
/// PORT-NOTE: `se_language.modificationCount` is an `mp_ui`-owned `vmCvar_t`
/// this host-agnostic crate cannot reach as cached state; threaded in as
/// `seLanguageModCount`, the value the caller reads off its own
/// `world.cvars.se_language`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:9833-9848`
pub fn Menu_PaintAll(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    seLanguageModCount: c_int,
) {
    match menus.captureFunc {
        CaptureFunc::None => {}
        CaptureFunc::ScrollListBoxAuto => Scroll_ListBox_AutoFunc(menus, ds, dc),
        CaptureFunc::ScrollListBoxThumb => Scroll_ListBox_ThumbFunc(menus, ds, dc),
        CaptureFunc::ScrollTextScrollAuto => Scroll_TextScroll_AutoFunc(menus, ds),
        CaptureFunc::ScrollTextScrollThumb => Scroll_TextScroll_ThumbFunc(menus, ds),
        CaptureFunc::ScrollSliderThumb => Scroll_Slider_ThumbFunc(menus, ds, dc),
    }

    for i in 0..menus.menus.len() {
        let menu = MenuId::new(i);
        Menu_Paint(menus, ds, dc, Some(menu), false, seLanguageModCount);
    }

    if menus.debugMode {
        let v: vec4_t = [1.0, 1.0, 1.0, 1.0];
        dc.drawText(
            5.0,
            25.0,
            0.75,
            v,
            &format!("fps: {:.6}", ds.FPS),
            0.0,
            0,
            0,
            0,
        );
        dc.drawText(
            5.0,
            45.0,
            0.75,
            v,
            &format!("x: {}  y:{}", ds.cursorx, ds.cursory),
            0.0,
            0,
            0,
            0,
        );
    }
}

/// Raven `Display_HandleKey` — route a keystroke to the item currently
/// capturing the mouse, else the focused menu.
///
/// Source: `oracle/codemp/ui/ui_shared.c:9918-9926`
pub fn Display_HandleKey(
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
    key: c_int,
    down: bool,
    x: c_int,
    y: c_int,
) {
    let menu = Display_CaptureItem(menus, x, y).or_else(|| Menu_GetFocused(menus));
    if let Some(menu) = menu {
        Menu_HandleKey(menus, ds, dc, Some(menu), key, down);
    }
}
