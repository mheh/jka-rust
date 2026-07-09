use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_KEY_SETCATCHER`.
///
/// Raven wrapper: `void trap_Key_SetCatcher( int catcher )`.
/// The MP client switch reads the catcher mask from `args[1]`, calls
/// `Key_SetCatcher(args[1])`, and returns `0`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:533-534`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:993-995`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgKeySetcatcherArgs {
    /// Key catcher mask, read by Raven as `args[1]`.
    catcher: c_int,
}

impl CgKeySetcatcherArgs {
    pub const fn new(catcher: c_int) -> Self {
        Self { catcher }
    }

    pub const fn catcher(&self) -> c_int {
        self.catcher
    }
}

/// `CG_KEY_SETCATCHER` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_KEY_SETCATCHER, catcher );`
/// Raven transport: `Key_SetCatcher( args[1] ); return 0;`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:196`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:533-534`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:993-995`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:993-995`
pub struct CgKeySetcatcher;

impl OutboundSysCall for CgKeySetcatcher {
    type Import = MpCgameImport;
    type Args = CgKeySetcatcherArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_SETCATCHER;
}

impl EncodeSysCall for CgKeySetcatcher {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.catcher() as isize])
    }
}

impl DecodeSysCallReturn for CgKeySetcatcher {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
