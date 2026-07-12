use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_R_INPVS`.
///
/// Raven wrapper: `qboolean trap_R_inPVS( const vec3_t p1, const vec3_t p2, byte *mask )`.
/// Raven transport passes both points through `VMA(1..2)` and the mutable
/// visibility mask through `VMA(3)`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:616-617`
/// Args source: `oracle/codemp/cgame/cg_local.h:2394`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1095-1096`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRInpvsArgs {
    p1: *const vec3_t,
    p2: *const vec3_t,
    mask: *mut u8,
}

impl CgRInpvsArgs {
    pub const fn new(p1: *const vec3_t, p2: *const vec3_t, mask: *mut u8) -> Self {
        Self { p1, p2, mask }
    }
}

/// `CG_R_INPVS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:217`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:616-617`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1095-1096`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1095-1096`
pub struct CgRInpvs;

impl OutboundSysCall for CgRInpvs {
    type Import = MpCgameImport;
    type Args = CgRInpvsArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_INPVS;
}

impl EncodeSysCall for CgRInpvs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.p1),
            ptr_to_word(args.p2),
            ptr_to_word(args.mask),
        ])
    }
}

impl DecodeSysCallReturn for CgRInpvs {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
