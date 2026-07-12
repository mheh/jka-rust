#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qboolean, MAX_QPATH};

/// Raven `vm_t` — a running instance of a loaded/interpreted virtual machine module.
///
/// Raven: DO NOT MOVE OR CHANGE THESE WITHOUT CHANGING THE VM_OFFSET_* DEFINES
/// USED BY THE ASM CODE.
/// Type definition source: `oracle/codemp/qcommon/vm_local.h:111-146`
#[repr(C)]
pub struct vm_t {
    // the vm may be recursively entered
    pub programStack: i32,
    pub systemCall: Option<extern "C" fn(parms: *mut i32) -> i32>,

    //------------------------------------
    pub name: [core::ffi::c_char; MAX_QPATH as usize],

    // for dynamic linked modules
    pub dllHandle: *mut core::ffi::c_void,
    // Raven's `int (QDECL *entryPoint)( int callNum, ... )` is the 32-bit
    // C-variadic shape; our module exports the fixed-arity widened dual
    // (`RawVmMain`: command + 12 AbiWord args — the vmMain pair ruling). The
    // types MUST match exactly: on Darwin arm64 variadic args travel on the
    // stack while fixed-arity args are in registers, so calling the module
    // through a variadic type delivers garbage args (live boot bug,
    // 2026-07-12: frozen level.time / commandTime=200).
    pub entryPoint: Option<native_platform::entrypoints::RawVmMain>,

    // for interpreted modules
    pub currentlyInterpreting: qboolean,

    pub compiled: qboolean,
    pub codeBase: *mut u8,
    pub codeLength: i32,

    pub instructionPointers: *mut i32,
    pub instructionPointersLength: i32,

    pub dataBase: *mut u8,
    pub dataMask: i32,

    // if programStack < stackBottom, error
    pub stackBottom: i32,

    pub numSymbols: i32,
    pub symbols: *mut super::vm_symbol_s::vmSymbol_t,

    // for debug indenting
    pub callLevel: i32,
    // increment breakCount on function entry to this
    pub breakFunction: i32,
    pub breakCount: i32,
}

/// Raven C tag `vm_s` for the same type.
pub type vm_s = vm_t;

const _: () = assert!(core::mem::offset_of!(vm_t, programStack) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<vm_t>() == 184);
    assert!(core::mem::offset_of!(vm_t, systemCall) == 8);
    assert!(core::mem::offset_of!(vm_t, name) == 16);
    assert!(core::mem::offset_of!(vm_t, dllHandle) == 80);
    assert!(core::mem::offset_of!(vm_t, entryPoint) == 88);
    assert!(core::mem::offset_of!(vm_t, currentlyInterpreting) == 96);
    assert!(core::mem::offset_of!(vm_t, compiled) == 100);
    assert!(core::mem::offset_of!(vm_t, codeBase) == 104);
    assert!(core::mem::offset_of!(vm_t, codeLength) == 112);
    assert!(core::mem::offset_of!(vm_t, instructionPointers) == 120);
    assert!(core::mem::offset_of!(vm_t, instructionPointersLength) == 128);
    assert!(core::mem::offset_of!(vm_t, dataBase) == 136);
    assert!(core::mem::offset_of!(vm_t, dataMask) == 144);
    assert!(core::mem::offset_of!(vm_t, stackBottom) == 148);
    assert!(core::mem::offset_of!(vm_t, numSymbols) == 152);
    assert!(core::mem::offset_of!(vm_t, symbols) == 160);
    assert!(core::mem::offset_of!(vm_t, callLevel) == 168);
    assert!(core::mem::offset_of!(vm_t, breakFunction) == 172);
    assert!(core::mem::offset_of!(vm_t, breakCount) == 176);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<vm_t>() == 136);
    assert!(core::mem::offset_of!(vm_t, systemCall) == 4);
    assert!(core::mem::offset_of!(vm_t, name) == 8);
    assert!(core::mem::offset_of!(vm_t, dllHandle) == 72);
    assert!(core::mem::offset_of!(vm_t, entryPoint) == 76);
    assert!(core::mem::offset_of!(vm_t, currentlyInterpreting) == 80);
    assert!(core::mem::offset_of!(vm_t, compiled) == 84);
    assert!(core::mem::offset_of!(vm_t, codeBase) == 88);
    assert!(core::mem::offset_of!(vm_t, codeLength) == 92);
    assert!(core::mem::offset_of!(vm_t, instructionPointers) == 96);
    assert!(core::mem::offset_of!(vm_t, instructionPointersLength) == 100);
    assert!(core::mem::offset_of!(vm_t, dataBase) == 104);
    assert!(core::mem::offset_of!(vm_t, dataMask) == 108);
    assert!(core::mem::offset_of!(vm_t, stackBottom) == 112);
    assert!(core::mem::offset_of!(vm_t, numSymbols) == 116);
    assert!(core::mem::offset_of!(vm_t, symbols) == 120);
    assert!(core::mem::offset_of!(vm_t, callLevel) == 124);
    assert!(core::mem::offset_of!(vm_t, breakFunction) == 128);
    assert!(core::mem::offset_of!(vm_t, breakCount) == 132);
};
