#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;
use core::ptr::null_mut;

use native_platform::entrypoints::RawVmMain;

use super::vm_symbol_s::vmSymbol_t;

/// Raven `vm_t` — a running instance of a loaded/interpreted virtual machine module.
///
/// Raven: DO NOT MOVE OR CHANGE THESE WITHOUT CHANGING THE VM_OFFSET_* DEFINES
/// USED BY THE ASM CODE. (That warning covers Raven's x86-asm interpreter;
/// this port's transcriptions use the fields directly, and `vm_t` never
/// crosses the module ABI — modules see only the syscall pointer and
/// `vmMain` — so the type is engine-internal and holds an idiomatic `String`
/// name (§D12 internal-only shape; the old `repr(C)` asserts went with it).)
/// Type definition source: `oracle/codemp/qcommon/vm_local.h:111-146`
pub struct vm_t {
    // the vm may be recursively entered
    pub programStack: i32,
    pub systemCall: Option<extern "C" fn(parms: *mut i32) -> i32>,

    //------------------------------------
    pub name: String,

    // for dynamic linked modules
    pub dllHandle: *mut c_void,
    // Raven's `int (QDECL *entryPoint)( int callNum, ... )` is the 32-bit
    // C-variadic shape; our module exports the fixed-arity widened dual
    // (`RawVmMain`: command + 12 AbiWord args — the vmMain pair ruling). The
    // types MUST match exactly: on Darwin arm64 variadic args travel on the
    // stack while fixed-arity args are in registers, so calling the module
    // through a variadic type delivers garbage args (live boot bug,
    // 2026-07-12: frozen level.time / commandTime=200).
    pub entryPoint: Option<RawVmMain>,

    // for interpreted modules
    pub currentlyInterpreting: bool,

    pub compiled: bool,
    pub codeBase: *mut u8,
    pub codeLength: i32,

    pub instructionPointers: *mut i32,
    pub instructionPointersLength: i32,

    pub dataBase: *mut u8,
    pub dataMask: i32,

    // if programStack < stackBottom, error
    pub stackBottom: i32,

    pub numSymbols: i32,
    pub symbols: *mut vmSymbol_t,

    // for debug indenting
    pub callLevel: i32,
    // increment breakCount on function entry to this
    pub breakFunction: i32,
    pub breakCount: i32,
}

/// Raven C tag `vm_s` for the same type.
pub type vm_s = vm_t;

/// The `Com_Memset(vm, 0, sizeof(vm_t))` replacement: every field at its C
/// zero value, the owned `name` empty. Assigning it drops the old `name`.
impl Default for vm_t {
    fn default() -> Self {
        vm_t {
            programStack: 0,
            systemCall: None,
            name: String::new(),
            dllHandle: null_mut(),
            entryPoint: None,
            currentlyInterpreting: false,
            compiled: false,
            codeBase: null_mut(),
            codeLength: 0,
            instructionPointers: null_mut(),
            instructionPointersLength: 0,
            dataBase: null_mut(),
            dataMask: 0,
            stackBottom: 0,
            numSymbols: 0,
            symbols: null_mut(),
            callLevel: 0,
            breakFunction: 0,
            breakCount: 0,
        }
    }
}
