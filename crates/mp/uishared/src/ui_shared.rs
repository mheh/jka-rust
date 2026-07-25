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
use crate::shared::edit_field_def_s::EditFieldDef;
use crate::shared::item_id::ItemId;
use crate::shared::menu_id::MenuId;
use crate::shared::menu_system::{MenuSystem, MAX_DEFERRED_SCRIPT};
use crate::shared::rect_def_t::RectDef;
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

/// Raven `#define WINDOW_STYLE_CINEMATIC 5`.
/// Source: `oracle/ui/menudef.h:48`
pub const WINDOW_STYLE_CINEMATIC: c_int = 5;

/// Raven `#define ITEM_TYPE_EDITFIELD 4`.
/// Source: `oracle/ui/menudef.h:13`
pub const ITEM_TYPE_EDITFIELD: c_int = 4;
/// Raven `#define ITEM_TYPE_LISTBOX 6`.
/// Source: `oracle/ui/menudef.h:15`
pub const ITEM_TYPE_LISTBOX: c_int = 6;

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
