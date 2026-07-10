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
pub use trampoline::{arm_game_slot, game_syscall_trampoline, game_syscall_trampoline_words};
