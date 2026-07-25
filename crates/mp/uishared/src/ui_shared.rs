//! `ui_shared.c` — the menu framework's logic, operating on the DEC-36 root
//! types: [`crate::shared::menu_system::MenuSystem`] (arena + handles),
//! [`crate::shared::display_state::DisplayState`] (the `DC->` data tail) and
//! the [`crate::shared::display_context::DisplayContext`] host trait.
//!
//! Source: `oracle/codemp/ui/ui_shared.c`

#![allow(non_snake_case)]
