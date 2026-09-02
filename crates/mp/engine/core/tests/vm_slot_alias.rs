//! The `vm_t` slot-aliasing pin for gh#56.
//!
//! `VM_Clear` wipes every `vmTable` slot, and `VM_Create` hands the first empty slot to the next module.
//! A slot address held across that pair therefore names whichever module arrives next.
//! No signal reports that change to the holder.
//! This test drives the real lookup loop over a seated table, so a future change to either function fails here.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_core::{engine_host_view, Engine};
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qcommon::vm_interpret_t::vmInterpret_t;
use mp_engine_qcommon::vm_fns::{VM_Clear, VM_Create};
use native_platform::entrypoints::{AbiCommand, AbiWord, RawVmMain};

/// This stub replaces a module's `vmMain`, so a seated slot looks loaded without a dylib on disk.
/// The test never calls it.
extern "C-unwind" fn stub_vm_main(
    _command: AbiCommand,
    _arg0: AbiWord,
    _arg1: AbiWord,
    _arg2: AbiWord,
    _arg3: AbiWord,
    _arg4: AbiWord,
    _arg5: AbiWord,
    _arg6: AbiWord,
    _arg7: AbiWord,
    _arg8: AbiWord,
    _arg9: AbiWord,
    _arg10: AbiWord,
    _arg11: AbiWord,
) -> AbiWord {
    0
}

/// This stub replaces the engine syscall table that `VM_Create` demands in its bad-parms guard.
/// The test never calls it.
extern "C" fn stub_syscall(_args: *mut c_int) -> c_int {
    0
}

/// Seats slot 0 under `name`, the way `demo_referee.rs` seats its probe module.
/// A seated name makes `VM_Create` return the slot from its name-match loop instead of reaching `Sys_LoadDll`.
fn seat_slot_zero(common: &mut Common, name: &str) {
    common.vmTable[0].name = name.to_string();
    let entry: RawVmMain = stub_vm_main;
    common.vmTable[0].entryPoint = Some(entry);
}

/// gh#56: a VM handle held across `VM_Clear` addresses the next module to take the slot.
/// The client owns `cl.uivm` and `cl.cgvm`, so the client must null them before a map load clears the table.
#[test]
fn a_cleared_slot_is_reused_by_the_next_module() {
    let mut engine: Box<Engine> = Engine::new();
    let mut view = engine_host_view(&mut engine);

    seat_slot_zero(view.common, "ui");
    let ui_slot = VM_Create(
        &mut view,
        "ui",
        Some(stub_syscall),
        vmInterpret_t::VMI_NATIVE,
    );
    assert!(
        !ui_slot.is_null(),
        "the name-match loop must return the seated slot"
    );

    VM_Clear(view.common);
    assert!(
        view.common.vmTable[0].name.is_empty(),
        "VM_Clear must empty the slot name"
    );
    assert!(
        view.common.vmTable[0].entryPoint.is_none(),
        "VM_Clear must drop the slot entry point"
    );

    seat_slot_zero(view.common, "jampgame");
    let game_slot = VM_Create(
        &mut view,
        "jampgame",
        Some(stub_syscall),
        vmInterpret_t::VMI_NATIVE,
    );

    assert_eq!(
        ui_slot, game_slot,
        "the game module took the address the ui handle still held"
    );
}
