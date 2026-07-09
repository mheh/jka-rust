use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_S_STARTLOCALSOUND`.
///
/// Raven wrapper: `syscall(CG_S_STARTLOCALSOUND, sfx, channelNum);`
/// Raven transport: `S_StartLocalSound(args[1], args[2]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:209-210`
/// Args source: `oracle/code/game/q_shared.h:186`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:580-586`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartlocalsoundArgs {
    sfx: c_int,
    channel_num: c_int,
}

impl CgSStartlocalsoundArgs {
    pub const fn new(sfx: c_int, channel_num: c_int) -> Self {
        Self { sfx, channel_num }
    }

    pub const fn sfx(&self) -> c_int {
        self.sfx
    }

    pub const fn channel_num(&self) -> c_int {
        self.channel_num
    }
}

/// `CG_S_STARTLOCALSOUND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:92`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:209-210`
/// Args source: `oracle/code/game/q_shared.h:186`
/// Output source: `oracle/code/client/cl_cgame.cpp:580-586`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:580-586`
pub struct CgSStartlocalsound;

impl OutboundSysCall for CgSStartlocalsound {
    type Import = SpCgameImport;
    type Args = CgSStartlocalsoundArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STARTLOCALSOUND;
}

impl EncodeSysCall for CgSStartlocalsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.sfx() as isize, args.channel_num() as isize])
    }
}

impl DecodeSysCallReturn for CgSStartlocalsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
