use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_S_GETSAMPLELENGTH`.
///
/// Raven wrapper: `return syscall( CG_S_GETSAMPLELENGTH, sfx);`
/// Raven transport: `return S_GetSampleLengthInMilliSeconds(args[1]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:237-238`
/// Args source: `oracle/oracle/code/game/q_shared.h:186`
/// Output source: `oracle/oracle/code/client/snd_public.h:18`
/// Output source: `oracle/oracle/code/client/snd_dma.cpp:1662-1664`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:610-611`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSGetsamplelengthArgs {
    sfx: c_int,
}

impl CgSGetsamplelengthArgs {
    pub const fn new(sfx: c_int) -> Self {
        Self { sfx }
    }

    pub const fn sfx(&self) -> c_int {
        self.sfx
    }
}

/// `CG_S_GETSAMPLELENGTH` SP cgame imports syscall ABI token.
///
/// SP's `CL_CgameSystemCalls` returns `int`, so the engine `float` is converted
/// to an integer syscall word before the cgame wrapper returns it as `float`.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:167`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:237-238`
/// Args source: `oracle/oracle/code/game/q_shared.h:186`
/// Output source: `oracle/oracle/code/client/snd_public.h:18`
/// Output source: `oracle/oracle/code/client/snd_dma.cpp:1662-1664`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:435`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:610-611`
pub struct CgSGetsamplelength;

impl OutboundSysCall for CgSGetsamplelength {
    type Import = SpCgameImport;
    type Args = CgSGetsamplelengthArgs;
    type Output = f32;

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_GETSAMPLELENGTH;
}

impl EncodeSysCall for CgSGetsamplelength {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.sfx() as isize])
    }
}

impl DecodeSysCallReturn for CgSGetsamplelength {
    fn decode_return(word: isize) -> Self::Output {
        (word as c_int) as f32
    }
}
