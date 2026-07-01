use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::qboolean;
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_R_INPVS`.
///
/// Raven wrapper: `return syscall( CG_R_INPVS, p1, p2 );`
/// Raven transport: `return R_inPVS((float *) VMA(1), (float *) VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:370-372`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:693-694`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRInpvsArgs {
    p1: *const vec3_t,
    p2: *const vec3_t,
}

impl CgRInpvsArgs {
    pub const fn new(p1: *const vec3_t, p2: *const vec3_t) -> Self {
        Self { p1, p2 }
    }

    pub const fn p1(&self) -> *const vec3_t {
        self.p1
    }

    pub const fn p2(&self) -> *const vec3_t {
        self.p2
    }
}

/// `CG_R_INPVS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:134`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:370-372`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:693-694`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:693-694`
pub struct CgRInpvs;

impl OutboundSysCall for CgRInpvs {
    type Import = SpCgameImport;
    type Args = CgRInpvsArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_INPVS;
}

impl EncodeSysCall for CgRInpvs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p1()), ptr_to_word(args.p2())])
    }
}

impl DecodeSysCallReturn for CgRInpvs {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
