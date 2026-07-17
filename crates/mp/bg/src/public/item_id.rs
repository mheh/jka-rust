//! Index into `bg_itemlist` — replaces Raven's `gitem_t*`.

use core::ffi::{c_char, c_int};
use core::num::NonZeroU16;

use super::bg_itemlist::{bg_itemlist, ITEM_CLASSNAMES_C};
use super::g_item::GItem;

/// Index into `bg_itemlist` — the wire `modelindex` an item entity carries.
///
/// Replaces Raven's `gitem_t*`: only the table index crosses the engine seam
/// (`s.modelindex`), never the struct. Slot 0 (the sentinel) is unrepresentable,
/// so `Option<ItemId>` is 2 bytes with `None` living in the 0 niche.
/// Source: `oracle/codemp/game/bg_public.h:1122-1138`
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ItemId(NonZeroU16);

impl ItemId {
    /// Wire `modelindex` → `ItemId`; `None` for slot 0 or an out-of-range index.
    #[inline]
    pub fn from_modelindex(modelindex: c_int) -> Option<ItemId> {
        if modelindex < 1 || modelindex as usize >= bg_itemlist.len() {
            return None;
        }
        NonZeroU16::new(modelindex as u16).map(ItemId)
    }

    /// The wire `modelindex` (index into `bg_itemlist`).
    #[inline]
    pub fn modelindex(self) -> c_int {
        self.0.get() as c_int
    }

    /// The master-item-table entry this id points at.
    #[inline]
    pub fn item(self) -> &'static GItem {
        &bg_itemlist[self.0.get() as usize]
    }

    /// The classname as a `'static` NUL-terminated `*const c_char` — bridge for
    /// the still-raw `gentity_t::classname` field (retires when it flips).
    #[inline]
    pub fn classname_cstr(self) -> *const c_char {
        ITEM_CLASSNAMES_C[self.0.get() as usize].as_ptr()
    }
}
