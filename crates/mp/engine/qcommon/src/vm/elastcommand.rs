#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ELastCommand` — the last emitted instruction kind, tracked by the x86
/// VM JIT peephole optimizer.
///
/// Type definition source: `oracle/codemp/qcommon/vm_x86.cpp:76-82`
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ELastCommand {
    LAST_COMMAND_NONE = 0,
    LAST_COMMAND_MOV_EDI_EAX = 1,
    LAST_COMMAND_SUB_DI_4 = 2,
    LAST_COMMAND_SUB_DI_8 = 3,
}

const _: () = assert!(core::mem::size_of::<ELastCommand>() == 4);
