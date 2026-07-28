//! `ui_host` — the R4a wave-3 harness that hosts the ported `ui` menu
//! framework on our own stack and renders it through backend #1.
//!
//! Three layers meet here, and nothing between them is faked:
//!
//! * [`boot`] brings up the **engine subset** the menus need — `Cvar_Init`,
//!   `Cbuf_Init`, `Com_InitZoneMemory`, `Cmd_Init`, `FS_InitFilesystem`
//!   against a retail `base/` — and then the **renderer CPU frontend** via the
//!   real [`mp_renderer::tr_init::R_Init`] over the DEC-42.3 carrier bundle.
//!   No server, no networking, no `Com_Init` (that path hard-requires the
//!   `SV_Init` hook); the ordered FS/cvar/cmd subset is lifted from
//!   `Com_Init`'s own prefix, the same way `jampgame`'s referee island does.
//! * [`display`] is the harness's [`DisplayContext`] implementor — the
//!   *different* implementor the plan calls for. Where `mp_ui`'s `UiContext`
//!   backs each slot onto a `trap_*` syscall into a C engine, this one backs
//!   them onto our own stack: draw calls append `FrameEvent`s to the frame's
//!   `FrameData`, registration calls enter `RE_RegisterShaderNoMip` /
//!   `RE_RegisterFont` / `RE_RegisterSkin` directly, cvars go to the real
//!   cvar table, and the menu-file tokenizer is the production
//!   `mp_engine_botlib` precompiler.
//! * [`crate::frame_exec`] executes the resulting `FrameData` — unmodified.
//!
//! **What is hosted, precisely.** The framework half of the ui module
//! (`ui_shared.c` → `mp_uishared`) runs for real: `String_Init`, `Menu_New`,
//! the whole `MenuParse_*`/`ItemParse_*` keyword pipeline, `Menus_ActivateByName`,
//! `Menu_PaintAll`, `Menu_HandleKey`, `Display_MouseMove`. The module half
//! (`ui_main.c` → `mp_ui`) is **not** callable here: every one of its
//! entry points (`_UI_Init`, `_UI_Refresh`, `_UI_KeyEvent`, …) takes a
//! concrete `&mut UiContext`, whose `engine` field is
//! `mp_engine_select::Engine` — the module-side C syscall transport. Reaching
//! it needs an engine-side UI syscall dispatcher (`CL_UISystemCalls`), which
//! is unported. So this harness runs `_UI_Init`'s *equivalent* prefix
//! ([`boot::ui_init_equivalent`]) and paints through the framework directly.
//! [`state::UiHost::ui`] is nonetheless the real [`mp_ui::UiState`], so the
//! day that dispatcher lands, the same owned state moves under it unchanged.

pub mod boot;
pub mod display;
pub mod state;

pub use boot::{boot, BootConfig};
pub use display::HarnessDc;
pub use state::{InputState, StubLog, UiHost};
