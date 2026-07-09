use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_S_MUTESOUND`.
///
/// Raven prototype: `void trap_S_MuteSound( int entityNum, int entchannel );`
/// Raven wrapper: `syscall( CG_S_MUTESOUND, entityNum, entchannel );`
/// Raven transport: `S_MuteSound( args[1], args[2] );`
///
/// Args source: `oracle/codemp/cgame/cg_local.h:2220`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:188-189`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:809-811`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgSMutesoundArgs {
    /// Entity number forwarded as Raven `args[1]`.
    entity_num: c_int,

    /// Entity sound channel forwarded as Raven `args[2]`.
    entchannel: c_int,
}

impl CgSMutesoundArgs {
    pub const fn new(entity_num: c_int, entchannel: c_int) -> Self {
        Self {
            entity_num,
            entchannel,
        }
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }

    pub const fn entchannel(&self) -> c_int {
        self.entchannel
    }
}

/// `CG_S_MUTESOUND` MP cgame imports syscall ABI token.
///
/// Raven enum: `CG_S_MUTESOUND`.
/// Raven prototype: `void trap_S_MuteSound( int entityNum, int entchannel );`
/// Raven wrapper: `syscall( CG_S_MUTESOUND, entityNum, entchannel );`
/// Raven transport: `S_MuteSound( args[1], args[2] ); return 0;`
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:96`
/// Args source: `oracle/codemp/cgame/cg_local.h:2220`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:188-189`
/// Output source: `oracle/codemp/cgame/cg_local.h:2220`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:188-189`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:809-811`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:809-811`
pub struct CgSMutesound;

impl OutboundSysCall for CgSMutesound {
    type Import = MpCgameImport;
    type Args = CgSMutesoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_MUTESOUND;
}

impl EncodeSysCall for CgSMutesound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num() as isize, args.entchannel() as isize])
    }
}

impl DecodeSysCallReturn for CgSMutesound {
    fn decode_return(_word: isize) -> Self::Output {
        // Raven's cgame wrapper is `void`; the client switch returns `0`.
    }
}
