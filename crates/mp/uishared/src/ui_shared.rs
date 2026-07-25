//! `ui_shared.c` — the menu framework's logic, operating on the DEC-36 root
//! types: [`crate::shared::menu_system::MenuSystem`] (arena + handles),
//! [`crate::shared::display_state::DisplayState`] (the `DC->` data tail) and
//! the [`crate::shared::display_context::DisplayContext`] host trait.
//!
//! Source: `oracle/codemp/ui/ui_shared.c`

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::q_string::COM_Parse;
use mp_qshared::shared::{pc_token_t, qtrue, vec4_t, MAX_TOKENLENGTH};
use native_string::{atof, atoi, latin1_to_string, Q_stricmp};

use crate::shared::display_context::DisplayContext;
use crate::shared::display_state::DisplayState;
use crate::shared::edit_field_def_s::{EditFieldDef, MAX_EDITFIELD};
use crate::shared::item_def_s::ItemDef;
use crate::shared::item_id::ItemId;
use crate::shared::item_payload::ItemPayload;
use crate::shared::list_box_def_s::ListBoxDef;
use crate::shared::menu_def_t::MenuDef;
use crate::shared::menu_id::MenuId;
use crate::shared::menu_system::{MenuSystem, MAX_DEFERRED_SCRIPT};
use crate::shared::menudef::{
    ITEM_ALIGN_CENTER, ITEM_ALIGN_RIGHT, ITEM_TYPE_BIND, ITEM_TYPE_EDITFIELD, ITEM_TYPE_LISTBOX,
    ITEM_TYPE_MODEL, ITEM_TYPE_MULTI, ITEM_TYPE_NUMERICFIELD, ITEM_TYPE_OWNERDRAW,
    ITEM_TYPE_SLIDER, ITEM_TYPE_TEXT, ITEM_TYPE_TEXTSCROLL, ITEM_TYPE_YESNO, UI_FORCE_RANK_ABSORB,
    UI_FORCE_RANK_DRAIN, UI_FORCE_RANK_GRIP, UI_FORCE_RANK_HEAL, UI_FORCE_RANK_LEVITATION,
    UI_FORCE_RANK_LIGHTNING, UI_FORCE_RANK_PROTECT, UI_FORCE_RANK_PULL, UI_FORCE_RANK_PUSH,
    UI_FORCE_RANK_RAGE, UI_FORCE_RANK_SABERATTACK, UI_FORCE_RANK_SABERDEFEND,
    UI_FORCE_RANK_SABERTHROW, UI_FORCE_RANK_SEE, UI_FORCE_RANK_SPEED, UI_FORCE_RANK_TEAM_FORCE,
    UI_FORCE_RANK_TEAM_HEAL, UI_FORCE_RANK_TELEPATHY, UI_FORCE_SIDE, WINDOW_BORDER_FULL,
    WINDOW_BORDER_HORZ, WINDOW_BORDER_KCGRADIENT, WINDOW_BORDER_VERT, WINDOW_STYLE_CINEMATIC,
    WINDOW_STYLE_FILLED, WINDOW_STYLE_GRADIENT, WINDOW_STYLE_SHADER, WINDOW_STYLE_TEAMCOLOR,
};
use crate::shared::model_def_s::ModelDef;
use crate::shared::multi_def_s::MultiDef;
use crate::shared::rect_def_t::RectDef;
use crate::shared::text_scroll_def_s::TextScrollDef;
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
/// Raven `#define WINDOW_DECORATION 0x00000010`.
/// Source: `oracle/codemp/ui/ui_shared.h:26`
pub const WINDOW_DECORATION: c_int = 0x0000_0010;
/// Raven `#define WINDOW_HORIZONTAL 0x00000400`.
/// Source: `oracle/codemp/ui/ui_shared.h:32`
pub const WINDOW_HORIZONTAL: c_int = 0x0000_0400;
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
/// Raven `#define WINDOW_PLAYERCOLOR 0x01000000`.
/// Source: `oracle/codemp/ui/ui_shared.h:46`
pub const WINDOW_PLAYERCOLOR: c_int = 0x0100_0000;

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
const A_ENTER: c_int = 10;
const A_KP_1: c_int = 17;
const A_KP_2: c_int = 18;
const A_KP_3: c_int = 19;
const A_KP_7: c_int = 23;
const A_KP_8: c_int = 24;
const A_KP_9: c_int = 25;
const A_ESCAPE: c_int = 27;
const A_MOUSE1: c_int = 141;
const A_MOUSE2: c_int = 142;
const A_HOME: c_int = 144;
const A_PAGE_UP: c_int = 145;
const A_END: c_int = 157;
const A_PAGE_DOWN: c_int = 158;
const A_MOUSE3: c_int = 166;
const A_CURSOR_UP: c_int = 170;
const A_CURSOR_DOWN: c_int = 171;
const A_CURSOR_LEFT: c_int = 172;
const A_CURSOR_RIGHT: c_int = 173;

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
        if stricmp_eq(&it.window.name, name)
            || (!it.window.group.is_empty() && stricmp_eq(&it.window.group, name))
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
        if stricmp_eq(p, &menus.item(id).window.name) {
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
        if stricmp_eq(&m.window.name, p) {
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
/// (`editDef == None && cvar` empty) case here falls back to
/// `EditFieldDef::default()` instead of a null deref.
/// Source: `oracle/codemp/ui/ui_shared.c:2790-2821`
pub fn Item_Slider_ThumbPosition(
    menus: &MenuSystem,
    dc: &mut dyn DisplayContext,
    item: ItemId,
) -> f32 {
    let it = menus.item(item);
    let editDef = it.typeData.editField();

    let mut x = if !it.text.is_empty() {
        it.textRect.x + it.textRect.w + 8.0
    } else {
        it.window.rect.x
    };

    if editDef.is_none() && !it.cvar.is_empty() {
        return x;
    }

    let default_edit = EditFieldDef::default();
    let editDef = editDef.unwrap_or(&default_edit);

    let mut value = dc.getCVarValue(&it.cvar);
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
        buff = dc.getCVarString(&it.cvar, 2048);
    } else {
        value = dc.getCVarValue(&it.cvar);
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
        if !it.cvar.is_empty() {
            buff = dc.getCVarString(&it.cvar, 2048);
        }
    } else if !it.cvar.is_empty() {
        value = dc.getCVarValue(&it.cvar);
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

    let x = if !it.text.is_empty() {
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
    dc.setCVar(&it.cvar, &format!("{:.6}", value));
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
        // PORT-NOTE: Raven distinguishes NULL (never parsed) from
        // `String_Alloc("")` here; owned `String` collapses both — `Option<String>`
        // conversion scheduled (U4 follow-up).
        if it.window.name.is_empty() && it.window.group.is_empty() {
            dc.Print("^3WARNING: item has neither name or group\n");
            continue;
        }

        if stricmp_eq(&it.window.name, name)
            || (!it.window.group.is_empty() && stricmp_eq(&it.window.group, name))
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

    // PORT-NOTE: Raven distinguishes NULL (never parsed) from `String_Alloc("")`
    // here; owned `String` collapses both — `Option<String>` conversion scheduled
    // (U4 follow-up).
    if Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
        && flags & WINDOW_HASFOCUS != 0
        && !cvar.is_empty()
    {
        if key == A_MOUSE1 || key == A_ENTER || key == A_MOUSE2 || key == A_MOUSE3 {
            let cur = dc.getCVarValue(&cvar);
            // C `!DC->getCVarValue(...)`: nonzero -> 0, zero -> 1.
            let newval: c_int = if cur != 0.0 { 0 } else { 1 };
            dc.setCVar(&cvar, &format!("{}", newval));
            return true;
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

    let cvar = menus.item(item).cvar.clone();
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
        && !cvar.is_empty()
        && Rect_ContainsPoint(Some(&rect), ds.cursorx as f32, ds.cursory as f32)
    {
        if key == A_MOUSE1 || key == A_ENTER || key == A_MOUSE2 || key == A_MOUSE3 {
            let editDef = menus.item(item).typeData.editField().copied();
            if let Some(editDef) = editDef {
                let width = SLIDER_WIDTH;
                // PORT-NOTE: Raven distinguishes NULL (never parsed) from
                // `String_Alloc("")` here; owned `String` collapses both —
                // `Option<String>` conversion scheduled (U4 follow-up).
                let x = if !text.is_empty() {
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
                    dc.setCVar(&cvar, &format!("{:.6}", value));
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
/// out when it lands). Raven's `textPtr == NULL` bail has no owned-`String`
/// counterpart — an empty text still runs the body, as in Raven.
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
        Some(s) => s.to_string(),
        None => itemText.clone(),
    };

    let it = menus.item(item);
    *width = it.textRect.w as c_int;
    *height = it.textRect.h as c_int;

    // keeps us from computing the widths and heights more than once
    if *width == 0
        || (itemType == ITEM_TYPE_OWNERDRAW && textalignment == ITEM_ALIGN_CENTER)
        || (itemText.starts_with('@') && asset != seLanguageModCount)
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
        } else if itemType == ITEM_TYPE_EDITFIELD
            && textalignment == ITEM_ALIGN_CENTER
            && !cvar.is_empty()
        {
            let buff = dc.getCVarString(&cvar, 256);
            originalWidth += dc.textWidth(&buff, textscale, iMenuFont);
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
        if it.text.starts_with('@') {
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
                let cvar = menus.item(item).cvar.clone();
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

    let cvar = menus.item(item).cvar.clone();
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
                if it.r#type == ITEM_TYPE_TEXT && !it.text.is_empty() {
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
