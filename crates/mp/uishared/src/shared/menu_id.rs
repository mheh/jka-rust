//! `MenuId` — the arena handle that replaces Raven's `menuDef_t *`.

/// Index handle into [`MenuSystem::menus`](super::menu_system::MenuSystem).
///
/// Raven addressed menus by raw `menuDef_t *` into the file-scope
/// `Menus[MAX_MENUS]` array (`menuStack`, `Menu_GetFocused`,
/// `Menus_FindByName`, `itemDef_t::parent`). Porting-rules §B5 replaces those
/// pointers with an id into the owned arena.
///
/// Source: `oracle/codemp/ui/ui_shared.c:111-115`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MenuId(pub u32);

impl MenuId {
    /// The handle for arena slot `index`.
    #[inline]
    pub const fn new(index: usize) -> Self {
        MenuId(index as u32)
    }

    /// This handle's arena slot.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
