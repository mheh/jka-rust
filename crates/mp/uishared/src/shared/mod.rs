//! MP `ui_shared.h`/`ui_shared.c` — the idiomatic menu framework.
//!
//! `menuDef_t`/`itemDef_t` and their payload structs land as owned Rust types
//! held in [`menu_system::MenuSystem`]'s two arenas, addressed by
//! [`menu_id::MenuId`]/[`item_id::ItemId`]; `displayContextDef_t` is replaced
//! by the [`display_context::DisplayContext`] trait plus the
//! [`display_state::DisplayState`] data tail (DEC-36 D2/D3).
//!
//! PORT-NOTE (dropped surface): Raven's `commandDef_t` (the
//! `commandList[]` name→handler table) and the `keywordHash_t` menu/item parse
//! tables are function-pointer dispatch tables; the translation dictionary
//! turns them into `match` dispatch in the parser/script runner, so no type
//! survives them. `configcvar_t` (`ui_shared.c:5182-5187`) is declared with
//! zero uses in either tree and is dropped under porting-rules §20.

pub mod bind_t;
pub mod cached_assets_t;
pub mod capture_func;
pub mod color_range_def_t;
pub mod column_info_s;
pub mod display_context;
pub mod display_state;
pub mod edit_field_def_s;
pub mod item_def_s;
pub mod item_id;
pub mod item_payload;
pub mod list_box_def_s;
pub mod menu_def_t;
pub mod menu_id;
pub mod menu_scratch;
pub mod menu_system;
pub mod model_def_s;
pub mod multi_def_s;
pub mod rect_def_t;
pub mod script_def_t;
pub mod scroll_info_s;
pub mod text_scroll_def_s;
pub mod window_def_t;
