//! `module_loader` — the native module-loading mechanism (LOAD-D1/D6).
//!
//! Consolidates Raven's per-platform `Sys_LoadDll`
//! (`win_main.cpp:811-887`, `unix_main.c:346-444`) and `VM_Create`/`VM_Restart`
//! (`vm.cpp:471-472,391-458`) into the single LOAD-D1 loader. File tree pinned
//! mechanically by LOAD-D6 (one-type-per-file).

pub mod loaded_module;
pub mod loader;
pub mod naming;
pub mod restart_kind;
pub mod search_policy;
pub mod search_step;

pub use loaded_module::LoadedModule;
pub use loader::{sys_load_dll, unload_module};
pub use naming::ModuleNaming;
pub use restart_kind::RestartKind;
pub use search_policy::ModuleSearchPolicy;
pub use search_step::SearchStep;
