use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_S_STARTSOUND`.
///
/// Raven wrapper: `syscall(CG_S_STARTSOUND, origin, entityNum, entchannel, sfx);`
/// Raven transport: `S_StartSound((float *)VMA(1), args[2], (soundChannel_t)args[3], args[4]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:185-186`
/// Args source: `oracle/oracle/code/game/q_shared.h:186`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:553-561`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartsoundArgs {
    origin: *const vec3_t,
    entity_num: c_int,
    entchannel: c_int,
    sfx: c_int,
}

impl CgSStartsoundArgs {
    pub const fn new(
        origin: *const vec3_t,
        entity_num: c_int,
        entchannel: c_int,
        sfx: c_int,
    ) -> Self {
        Self {
            origin,
            entity_num,
            entchannel,
            sfx,
        }
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }

    pub const fn entchannel(&self) -> c_int {
        self.entchannel
    }

    pub const fn sfx(&self) -> c_int {
        self.sfx
    }
}

/// `CG_S_STARTSOUND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:91`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:185-186`
/// Args source: `oracle/oracle/code/game/q_shared.h:186`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:553-561`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:553-561`
pub struct CgSStartsound;

impl OutboundSysCall for CgSStartsound {
    type Import = SpCgameImport;
    type Args = CgSStartsoundArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STARTSOUND;
}

impl EncodeSysCall for CgSStartsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.origin()),
            args.entity_num() as isize,
            args.entchannel() as isize,
            args.sfx() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSStartsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
