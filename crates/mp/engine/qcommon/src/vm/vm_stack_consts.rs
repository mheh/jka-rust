#![allow(non_camel_case_types, non_snake_case)]

/// Raven `STACK_SIZE` — bytes reserved for a QVM's data+stack segment in `VM_Create`.
/// Source: oracle/codemp/qcommon/vm.cpp:469
pub const STACK_SIZE: usize = 0x20000;

/// Raven `MAX_STACK` — depth of `VM_Call`'s opStack ring buffer.
/// Source: oracle/codemp/qcommon/vm.cpp:784
pub const MAX_STACK: usize = 256;

/// Raven `STACK_MASK` — `MAX_STACK - 1`, wraps opStack indices.
/// Source: oracle/codemp/qcommon/vm.cpp:785
pub const STACK_MASK: usize = MAX_STACK - 1;
