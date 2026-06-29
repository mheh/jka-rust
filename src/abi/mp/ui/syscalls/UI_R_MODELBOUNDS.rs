use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `UI_R_MODELBOUNDS`.
///
/// Raven wrapper: `syscall( UI_R_MODELBOUNDS, model, mins, maxs );`
/// Raven transport: `re.ModelBounds( args[1], (float *)VMA(2), (float *)VMA(3) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:198-199`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:946`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:988-990`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiRModelboundsArgs {
    model: c_int,
    mins: *mut vec3_t,
    maxs: *mut vec3_t,
}

impl UiRModelboundsArgs {
    pub const fn new(model: c_int, mins: *mut vec3_t, maxs: *mut vec3_t) -> Self {
        Self { model, mins, maxs }
    }
}

/// `UI_R_MODELBOUNDS` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:83`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:198-199`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:988-990`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:988-990`
pub struct UiRModelbounds;

impl OutboundSysCall for UiRModelbounds {
    type Import = MpUiImport;
    type Args = UiRModelboundsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_MODELBOUNDS;
}

impl EncodeSysCall for UiRModelbounds {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.model as isize,
            ptr_to_word(args.mins),
            ptr_to_word(args.maxs),
        ])
    }
}

impl DecodeSysCallReturn for UiRModelbounds {
    fn decode_return(_word: isize) -> Self::Output {}
}
