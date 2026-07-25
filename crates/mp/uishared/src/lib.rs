//! `mp_uishared` crate — the MP menu framework of `ui_shared.h`/`ui_shared.c`,
//! reshaped to its idiomatic form by DEC-36 (ui-port stage U2/U3).
//!
//! The crate is host-agnostic: `ui` owns one [`shared::menu_system::MenuSystem`]
//! by composition on `UiWorld`, and `cgame` will own a second one on `CgWorld`
//! (Raven compiled `ui_shared.c` into both modules, each with its own copy of
//! `Menus[64]`, the open-menu stack and the allocation pools). Everything the
//! framework needs from its host is reached through the
//! [`shared::display_context::DisplayContext`] trait, which replaces Raven's
//! `displayContextDef_t` function-pointer table (DEC-36 D3).

pub mod shared;
