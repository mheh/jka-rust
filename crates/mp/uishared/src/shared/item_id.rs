//! `ItemId` — the arena handle that replaces Raven's `itemDef_t *`.

/// Index handle into [`MenuSystem::items`](super::menu_system::MenuSystem).
///
/// Raven addressed items by raw `itemDef_t *`: `menuDef_t::items[256]`,
/// `g_bindItem`/`g_editItem`/`itemCapture`, `scrollInfo.item` and the
/// `feederSelection` callback argument all carried one. Porting-rules §B5
/// replaces those pointers with an id into the owned arena.
///
/// Source: `oracle/codemp/ui/ui_shared.h:327` (`menuDef_t::items`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(pub u32);

impl ItemId {
    /// The handle for arena slot `index`.
    #[inline]
    pub const fn new(index: usize) -> Self {
        ItemId(index as u32)
    }

    /// This handle's arena slot.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
