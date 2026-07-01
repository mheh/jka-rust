use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_WORLD_EFFECT_COMMAND`.
///
/// Raven wrapper: `syscall( CG_R_WORLD_EFFECT_COMMAND, command );`
/// Raven transport: `re.WorldEffectCommand((const char *) VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:514-516`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:813-815`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRWorldEffectCommandArgs {
    command: *const c_char,
}

impl CgRWorldEffectCommandArgs {
    pub const fn new(command: *const c_char) -> Self {
        Self { command }
    }
}

/// `CG_R_WORLD_EFFECT_COMMAND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:183`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:514-516`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:813-815`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:813-815`
pub struct CgRWorldEffectCommand;

impl OutboundSysCall for CgRWorldEffectCommand {
    type Import = SpCgameImport;
    type Args = CgRWorldEffectCommandArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_WORLD_EFFECT_COMMAND;
}

impl EncodeSysCall for CgRWorldEffectCommand {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.command)])
    }
}

impl DecodeSysCallReturn for CgRWorldEffectCommand {
    fn decode_return(_word: isize) -> Self::Output {}
}
