//! `CgState` — the cgame module's one owned island, split into the three
//! borrows the `vmMain` shell hands out. Mirrors `mp_ui`'s `UiState`.

#![allow(non_snake_case)]

use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;

use super::cg_world::CgWorld;

/// Everything the cgame module owns across `vmMain` calls, split into three
/// disjoint borrows on every dispatch.
///
/// Raven kept all three as file-scope state in one link unit — `cg_t cg` /
/// `cgs_t cgs` / `centity_t cg_entities[]` (`cg_main.c:691-693`),
/// `displayContextDef_t cgDC` (`cg_main.c:8`), and `ui_shared.c`'s
/// `Menus[]`/`menuStack[]` pool globals, since cgame links `ui_shared.c` for
/// its HUD and scoreboard (`cgame.q3asm:47`, `JK2_cgame.vcproj:471`).
///
/// The split is `mp_ui`'s [`UiState`](mp_ui) shape verbatim (DEC-38 ruling 1):
/// the menu framework calls back into host state, so a ported fn must hold
/// `menus`/`cgDC` beside a live [`CgContext`](super::cg_context::CgContext).
/// Three disjoint fields, three disjoint borrows, no aliasing (§B4). Keeping
/// `menus` inside `CgWorld` would put the framework and the thing that owns it
/// behind the same `&mut`.
///
/// Source: `oracle/codemp/cgame/cg_main.c:8,691-693`, `docs/decisions.md`
/// DEC-38 (ruling 1, revised), DEC-46 (ruling 1)
pub struct CgState {
    /// Raven's cgame spine and every folded-in file-scope global.
    /// Source: `oracle/codemp/cgame/cg_main.c:691-693`
    pub world: Box<CgWorld>,

    /// The menu framework cgame shares with ui. Raven: `ui_shared.c`'s
    /// file-scope arrays, compiled into the cgame link unit.
    /// Source: `oracle/codemp/ui/ui_shared.c:111-115`
    pub menus: MenuSystem,

    /// Raven `displayContextDef_t cgDC`'s data tail (DEC-36 D3 — the fn-pointer
    /// half is the `DisplayContext` trait, whose cgame implementor lands with
    /// the C5 waves).
    /// Source: `oracle/codemp/cgame/cg_main.c:8`
    pub cgDC: DisplayState,
}

impl CgState {
    /// Builds the island on the heap. `CgWorld` is already boxed, but `cgDC`
    /// and `menus` are large enough to be worth writing in place too.
    ///
    /// Raven's `cg`/`cgs`/`cgDC` are zeroed file-scope structs `CG_Init` fills;
    /// the framework arrays start zeroed beside them.
    ///
    /// Source: `oracle/codemp/cgame/cg_main.c:3164-3167`
    pub fn new_boxed() -> Box<Self> {
        let mut boxed: Box<MaybeUninit<CgState>> = Box::new_uninit();
        let p: *mut CgState = boxed.as_mut_ptr();
        // SAFETY: `p` points at freshly allocated, correctly aligned storage
        // for one `CgState`; each write initializes a distinct field exactly
        // once before `assume_init`.
        unsafe {
            addr_of_mut!((*p).world).write(CgWorld::new_boxed());
            addr_of_mut!((*p).menus).write(MenuSystem::default());
            addr_of_mut!((*p).cgDC).write(DisplayState::default());
            boxed.assume_init()
        }
    }
}
