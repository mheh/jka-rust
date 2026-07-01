use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_S_STARTLOCALSOUND`.
///
/// C ABI: `void trap_S_StartLocalSound( sfxHandle_t sfx, int channelNum )`.
/// Raven's wrapper forwards the sound handle and channel number as two payload
/// words, and the client switch reads them from `args[1]` and `args[2]`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:196-197`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2225`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:815-817`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartlocalsoundArgs {
    /// Sound handle (`sfxHandle_t`, Raven typedefs this as `int`), read as
    /// `args[1]`.
    sfx: c_int,
    /// Local sound channel number, read as `args[2]`.
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

/// `CG_S_STARTLOCALSOUND` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_S_STARTLOCALSOUND, sfx, channelNum );`
/// Raven transport: `S_StartLocalSound( args[1], args[2] ); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:98`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:196-197`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:196-197`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:815-817`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:815-817`
pub struct CgSStartlocalsound;

impl OutboundSysCall for CgSStartlocalsound {
    type Import = MpCgameImport;
    type Args = CgSStartlocalsoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STARTLOCALSOUND;
}

impl EncodeSysCall for CgSStartlocalsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.sfx() as isize, args.channel_num() as isize])
    }
}

impl DecodeSysCallReturn for CgSStartlocalsound {
    // Raven returns 0; the C wrapper is `void`.
    fn decode_return(_word: isize) -> Self::Output {}
}
