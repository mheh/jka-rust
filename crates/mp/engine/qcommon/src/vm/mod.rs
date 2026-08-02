//! `vm` types.

pub mod elastcommand;
pub mod engine_slot;
pub mod module_registry;
pub mod module_slot;
pub mod module_transport;
pub mod opcode_t;
pub mod slot_id;
pub mod trampoline;
pub mod vm_local_consts;
pub mod vm_s;
pub mod vm_stack_consts;
pub mod vm_symbol_s;
pub mod vm_x86_cpp_consts;
pub mod vmptr_t;

pub use engine_slot::{EngineSlot, SlotSyscall};
pub use module_registry::{ModuleRegistry, MAX_VM};
pub use module_slot::ModuleSlot;
pub use module_transport::ModuleTransport;
pub use slot_id::SlotId;
pub use trampoline::{
    arm_cgame_slot, arm_game_slot, arm_ui_slot, cgame_syscall_trampoline,
    cgame_syscall_trampoline_words, game_syscall_trampoline, game_syscall_trampoline_words,
    ui_syscall_trampoline, ui_syscall_trampoline_words,
};

pub use crate::vm_fns::VM_Call;
