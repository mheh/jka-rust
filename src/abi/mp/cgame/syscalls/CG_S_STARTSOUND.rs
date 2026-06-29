use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_S_STARTSOUND`.
///
/// C ABI: `void trap_S_StartSound(vec3_t origin, int entityNum,
/// int entchannel, sfxHandle_t sfx)`.
///
/// Raven forwards `origin` as a `vec3_t` pointer, then reads `entityNum`,
/// `entchannel`, and `sfx` as raw syscall words. `sfxHandle_t` is
/// `typedef int` in Raven `q_shared.h`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:192-193`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2221`
/// Args source: `oracle/oracle/codemp/game/q_shared.h:361`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:812-814`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartsoundArgs {
    /// Sound origin, decoded by Raven as `(float *)VMA(1)`.
    origin: *const vec3_t,
    /// Entity number, read by Raven as raw `args[2]`.
    entity_num: c_int,
    /// Entity sound channel, read by Raven as raw `args[3]`.
    entchannel: c_int,
    /// `sfxHandle_t` sound handle, read by Raven as raw `args[4]`.
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

/// `CG_S_STARTSOUND` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_S_STARTSOUND, origin, entityNum,
/// entchannel, sfx );`
/// Raven transport: `S_StartSound( (float *)VMA(1), args[2], args[3],
/// args[4] );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:97`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:192-193`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2221`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:192-194`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:812-814`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:812-814`
pub struct CgSStartsound;

impl OutboundSysCall for CgSStartsound {
    type Import = MpCgameImport;
    type Args = CgSStartsoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STARTSOUND;
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
    // `S_StartSound` is void; Raven returns 0 from the switch arm.
    fn decode_return(_word: isize) -> Self::Output {}
}
