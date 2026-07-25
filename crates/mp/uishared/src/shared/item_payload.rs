//! `ItemPayload` — the owned form of Raven's `itemDef_t::typeData` void pointer.

// The accessors keep the Raven payload-type names they replace the casts for.
#![allow(non_snake_case)]

use super::edit_field_def_s::EditFieldDef;
use super::list_box_def_s::ListBoxDef;
use super::model_def_s::ModelDef;
use super::multi_def_s::MultiDef;
use super::text_scroll_def_s::TextScrollDef;

/// The per-item-type payload Raven kept behind `void *typeData`, pool-allocated
/// and cast by the reader (`Item_ValidateTypeData` picks the struct from
/// `item->type`; every consumer casts back). The five reachable payload types
/// become one owned enum, so the cast is a `match` and the pool allocation is
/// ownership (porting-rules §C9).
///
/// Source: `oracle/codemp/ui/ui_shared.c:7279-7316` (`Item_ValidateTypeData`)
/// Source: `oracle/codemp/ui/ui_shared.h:298` (`void *typeData`)
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ItemPayload {
    /// `item->typeData == NULL` — the item type carries no payload.
    #[default]
    None,
    /// `ITEM_TYPE_LISTBOX`.
    ListBox(ListBoxDef),
    /// `ITEM_TYPE_EDITFIELD`, `ITEM_TYPE_NUMERICFIELD`, `ITEM_TYPE_YESNO`,
    /// `ITEM_TYPE_BIND`, `ITEM_TYPE_SLIDER`, `ITEM_TYPE_TEXT`.
    EditField(EditFieldDef),
    /// `ITEM_TYPE_MULTI`.
    Multi(MultiDef),
    /// `ITEM_TYPE_MODEL`.
    Model(ModelDef),
    /// `ITEM_TYPE_TEXTSCROLL`.
    TextScroll(TextScrollDef),
}

impl ItemPayload {
    /// Raven's `if (item->typeData)` guard.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, ItemPayload::None)
    }

    /// The `(listBoxDef_t *)item->typeData` cast.
    #[inline]
    pub fn listBox(&self) -> Option<&ListBoxDef> {
        match self {
            ItemPayload::ListBox(v) => Some(v),
            _ => None,
        }
    }

    /// Mutable [`Self::listBox`].
    #[inline]
    pub fn listBox_mut(&mut self) -> Option<&mut ListBoxDef> {
        match self {
            ItemPayload::ListBox(v) => Some(v),
            _ => None,
        }
    }

    /// The `(editFieldDef_t *)item->typeData` cast.
    #[inline]
    pub fn editField(&self) -> Option<&EditFieldDef> {
        match self {
            ItemPayload::EditField(v) => Some(v),
            _ => None,
        }
    }

    /// Mutable [`Self::editField`].
    #[inline]
    pub fn editField_mut(&mut self) -> Option<&mut EditFieldDef> {
        match self {
            ItemPayload::EditField(v) => Some(v),
            _ => None,
        }
    }

    /// The `(multiDef_t *)item->typeData` cast.
    #[inline]
    pub fn multi(&self) -> Option<&MultiDef> {
        match self {
            ItemPayload::Multi(v) => Some(v),
            _ => None,
        }
    }

    /// Mutable [`Self::multi`].
    #[inline]
    pub fn multi_mut(&mut self) -> Option<&mut MultiDef> {
        match self {
            ItemPayload::Multi(v) => Some(v),
            _ => None,
        }
    }

    /// The `(modelDef_t *)item->typeData` cast.
    #[inline]
    pub fn model(&self) -> Option<&ModelDef> {
        match self {
            ItemPayload::Model(v) => Some(v),
            _ => None,
        }
    }

    /// Mutable [`Self::model`].
    #[inline]
    pub fn model_mut(&mut self) -> Option<&mut ModelDef> {
        match self {
            ItemPayload::Model(v) => Some(v),
            _ => None,
        }
    }

    /// The `(textScrollDef_t *)item->typeData` cast.
    #[inline]
    pub fn textScroll(&self) -> Option<&TextScrollDef> {
        match self {
            ItemPayload::TextScroll(v) => Some(v),
            _ => None,
        }
    }

    /// Mutable [`Self::textScroll`].
    #[inline]
    pub fn textScroll_mut(&mut self) -> Option<&mut TextScrollDef> {
        match self {
            ItemPayload::TextScroll(v) => Some(v),
            _ => None,
        }
    }
}
