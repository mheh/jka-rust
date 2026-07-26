//! `MenuSystem` — the owned menu framework (`ui_shared.c` file-scope state).

// Fields and accessors keep Raven's `ui_shared.c` global names.
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use super::bind_t::{default_bindings, Bind};
use super::capture_func::CaptureFunc;
use super::item_def_s::ItemDef;
use super::item_id::ItemId;
use super::menu_def_t::MenuDef;
use super::menu_id::MenuId;
use super::menu_scratch::MenuScratch;
use super::scroll_info_s::ScrollInfo;

/// Raven `#define MAX_MENUS 64`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:16`
pub const MAX_MENUS: usize = 64;

/// Raven `#define MAX_OPEN_MENUS 16`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:19`
pub const MAX_OPEN_MENUS: usize = 16;

/// Raven `#define MAX_DEFERRED_SCRIPT 2048`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:1754`
pub const MAX_DEFERRED_SCRIPT: usize = 2048;

/// Raven `#define MAX_MENUDEFFILE 4096`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:14`
pub const MAX_MENUDEFFILE: usize = 4096;

/// Raven `#define MAX_MENUFILE 32768`.
///
/// Source: `oracle/codemp/ui/ui_shared.h:15`
pub const MAX_MENUFILE: usize = 32768;

/// Number of languages `currLanguage` caches (`char currLanguage[32][128]`).
///
/// Source: `oracle/codemp/ui/ui_shared.c:8601`
pub const MAX_LANGUAGES: usize = 32;

/// Raven `#define DOUBLE_CLICK_DELAY 300`.
///
/// Source: `oracle/codemp/ui/ui_shared.c:119`
pub const DOUBLE_CLICK_DELAY: c_int = 300;

/// The menu framework `ui_shared.c` implements, as one owned value per host
/// module (DEC-36 D2). Raven compiled the file into both `ui` and `cgame`, each
/// linkage getting its own copy of every file-scope global below; here each
/// host owns one `MenuSystem` by composition (`UiWorld::menus`).
///
/// The `menuDef_t`/`itemDef_t` raw-pointer graph becomes two arenas plus
/// [`MenuId`]/[`ItemId`] handles (porting-rules §B5): `menuDef_t::items`,
/// `menuStack`, `itemDef_t::parent`, `g_bindItem`, `g_editItem`,
/// `itemCapture` and `scrollInfo.item` are all ids.
///
/// PORT-NOTE (retired allocators): Raven's two bump pools do not appear here.
/// `memoryPool[2 MB]`/`allocPoint`/`outOfMemory` (`UI_Alloc`) backed only
/// `itemDef_t::typeData` and the string-intern nodes, both of which are now
/// owned values ([`ItemPayload`](super::item_payload::ItemPayload) and
/// `String`); `strPool[384 KB]`/`strPoolIndex`/`strHandle`/`strHandleCount`
/// (`String_Alloc`) interned parsed menu strings, and DEC-36 D2 rules that
/// owned `String` fields satisfy the requirement without a shared intern
/// table. Porting-rules §C9 (manual alloc/free → ownership).
///
/// Source: `oracle/codemp/ui/ui_shared.c:97-160,284-288,1571,1756-1757,5407-5408,7487,8601`
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct MenuSystem {
    /// Raven `menuDef_t Menus[MAX_MENUS]` + `int menuCount` — the defined-menu
    /// arena. `menuCount` is `menus.len()`.
    /// Source: `oracle/codemp/ui/ui_shared.c:111-112`
    pub menus: Vec<MenuDef>,

    /// The item arena every [`MenuDef::items`] entry indexes. Raven had no such
    /// array: items were `UI_Alloc`'d out of the bump pool and reached only
    /// through `menu->items[]`, so the arena is the ownership that replaces
    /// the pool.
    /// Source: `oracle/codemp/ui/ui_shared.h:327` (`itemDef_t *items[256]`)
    pub items: Vec<ItemDef>,

    /// Raven `menuDef_t *menuStack[MAX_OPEN_MENUS]` + `int openMenuCount` —
    /// the open-menu stack. `openMenuCount` is `menuStack.len()`.
    /// Source: `oracle/codemp/ui/ui_shared.c:114-115`
    pub menuStack: Vec<MenuId>,

    /// Raven `static scrollInfo_t scrollInfo`.
    /// Source: `oracle/codemp/ui/ui_shared.c:97`
    pub scrollInfo: ScrollInfo,

    /// Raven `static void (*captureFunc)(void *p)` (+ its always-`&scrollInfo`
    /// `captureData`).
    /// Source: `oracle/codemp/ui/ui_shared.c:99-100`
    pub captureFunc: CaptureFunc,

    /// Raven `static itemDef_t *itemCapture` — item that has the mouse
    /// captured ( if any ).
    /// Source: `oracle/codemp/ui/ui_shared.c:101`
    pub itemCapture: Option<ItemId>,

    /// Raven `static qboolean g_waitingForKey`.
    /// Source: `oracle/codemp/ui/ui_shared.c:105`
    pub g_waitingForKey: bool,
    /// Raven `static qboolean g_editingField`.
    /// Source: `oracle/codemp/ui/ui_shared.c:106`
    pub g_editingField: bool,

    /// Raven `static itemDef_t *g_bindItem`.
    /// Source: `oracle/codemp/ui/ui_shared.c:108`
    pub g_bindItem: Option<ItemId>,
    /// Raven `static itemDef_t *g_editItem`.
    /// Source: `oracle/codemp/ui/ui_shared.c:109`
    pub g_editItem: Option<ItemId>,

    /// Raven `static qboolean debugMode`.
    /// Source: `oracle/codemp/ui/ui_shared.c:117`
    pub debugMode: bool,

    /// Raven `static int lastListBoxClickTime`.
    /// Source: `oracle/codemp/ui/ui_shared.c:120`
    pub lastListBoxClickTime: c_int,

    /// Raven `int FPMessageTime` — force-power message expiry, written here and
    /// read by `ui_main.c`.
    /// Source: `oracle/codemp/ui/ui_shared.c:1571`
    pub FPMessageTime: c_int,

    /// Raven `char ui_deferredScript[MAX_DEFERRED_SCRIPT]` — the suspended
    /// tail of a menu script.
    /// Source: `oracle/codemp/ui/ui_shared.c:1756`
    pub ui_deferredScript: String,
    /// Raven `itemDef_t *ui_deferredScriptItem`.
    /// Source: `oracle/codemp/ui/ui_shared.c:1757`
    pub ui_deferredScriptItem: Option<ItemId>,

    /// Raven `static bind_t g_bindings[]` + `static const int g_bindCount` —
    /// the controls table, seeded from the compiled-in defaults and then
    /// rewritten in place by `Controls_GetConfig`. `g_bindCount` is
    /// `g_bindings.len()`.
    /// Source: `oracle/codemp/ui/ui_shared.c:5190-5295`
    pub g_bindings: Vec<Bind>,

    /// Raven `char g_nameBind1[32]`.
    /// Source: `oracle/codemp/ui/ui_shared.c:5407`
    pub g_nameBind1: String,
    /// Raven `char g_nameBind2[32]`.
    /// Source: `oracle/codemp/ui/ui_shared.c:5408`
    pub g_nameBind2: String,

    /// Raven `char currLanguage[32][128]` — the localized language names the
    /// language multi-item cycles through.
    /// Source: `oracle/codemp/ui/ui_shared.c:8601`
    pub currLanguage: Vec<String>,

    /// Raven `uiG2PtrTracker_t *ui_G2PtrTracker` — the singly-linked list of
    /// ghoul2 instances the UI created, walked on shutdown to clean them up.
    /// The intrusive list becomes an owned `Vec` of the same opaque engine
    /// tokens (porting-rules §C9); the module never reads through them.
    /// Source: `oracle/codemp/ui/ui_shared.c:7481-7487`
    pub ui_G2PtrTracker: Vec<*mut c_void>,

    /// Framework function-local persistent scratch.
    pub scratch: MenuScratch,
}

impl Default for MenuSystem {
    /// Every other field zeroes/empties exactly as `#[derive(Default)]` would
    /// give it; `g_bindings` alone cannot, because Raven's `g_bindings[]` is
    /// file-scope *static* data (`oracle/codemp/ui/ui_shared.c:5190-5292`) —
    /// always populated from process start, never built by a runtime call a
    /// port could forget to make. `default_bindings()` reproduces that
    /// guarantee: a `MenuSystem` can't exist with an empty table (previously
    /// the derive gave every fresh `MenuSystem` an empty `Vec`, so
    /// `BindingFromName`'s `g_bindings` scan matched nothing and every
    /// controls-menu row painted "???", bound or not — the table itself had
    /// never been transcribed).
    fn default() -> Self {
        MenuSystem {
            menus: Default::default(),
            items: Default::default(),
            menuStack: Default::default(),
            scrollInfo: Default::default(),
            captureFunc: Default::default(),
            itemCapture: Default::default(),
            g_waitingForKey: Default::default(),
            g_editingField: Default::default(),
            g_bindItem: Default::default(),
            g_editItem: Default::default(),
            debugMode: Default::default(),
            lastListBoxClickTime: Default::default(),
            FPMessageTime: Default::default(),
            ui_deferredScript: Default::default(),
            ui_deferredScriptItem: Default::default(),
            g_bindings: default_bindings(),
            g_nameBind1: Default::default(),
            g_nameBind2: Default::default(),
            currLanguage: Default::default(),
            ui_G2PtrTracker: Default::default(),
            scratch: Default::default(),
        }
    }
}

impl MenuSystem {
    /// Borrow menu `id` out of the arena.
    #[inline]
    pub fn menu(&self, id: MenuId) -> &MenuDef {
        &self.menus[id.index()]
    }

    /// Mutable [`Self::menu`].
    #[inline]
    pub fn menu_mut(&mut self, id: MenuId) -> &mut MenuDef {
        &mut self.menus[id.index()]
    }

    /// Borrow item `id` out of the arena.
    #[inline]
    pub fn item(&self, id: ItemId) -> &ItemDef {
        &self.items[id.index()]
    }

    /// Mutable [`Self::item`].
    #[inline]
    pub fn item_mut(&mut self, id: ItemId) -> &mut ItemDef {
        &mut self.items[id.index()]
    }

    /// Raven `Menu_Count()` — the number of defined menus (`menuCount`).
    ///
    /// Source: `oracle/codemp/ui/ui_shared.c` (`Menu_Count`)
    #[inline]
    pub fn menuCount(&self) -> c_int {
        self.menus.len() as c_int
    }

    /// Raven `openMenuCount` — the depth of the open-menu stack.
    ///
    /// Source: `oracle/codemp/ui/ui_shared.c:115`
    #[inline]
    pub fn openMenuCount(&self) -> c_int {
        self.menuStack.len() as c_int
    }
}
