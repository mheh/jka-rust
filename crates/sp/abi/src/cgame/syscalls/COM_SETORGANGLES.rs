use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `COM_SETORGANGLES`.
///
/// Raven wrapper: `void COM_SetOrgAngles( const vec3_t org, const vec3_t angles );`
/// Raven transport: `Com_SetOrgAngles((float *) VMA(1), (float *) VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:494-496`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:776-778`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComSetorganglesArgs {
    org: *const vec3_t,
    angles: *const vec3_t,
}

impl ComSetorganglesArgs {
    pub const fn new(org: *const vec3_t, angles: *const vec3_t) -> Self {
        Self { org, angles }
    }

    pub const fn org(&self) -> *const vec3_t {
        self.org
    }

    pub const fn angles(&self) -> *const vec3_t {
        self.angles
    }
}

/// `COM_SETORGANGLES` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:168`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:494-496`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:776-778`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:776-778`
pub struct ComSetorgangles;

impl OutboundSysCall for ComSetorgangles {
    type Import = SpCgameImport;
    type Args = ComSetorganglesArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::COM_SETORGANGLES;
}

impl EncodeSysCall for ComSetorgangles {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.org()), ptr_to_word(args.angles())])
    }
}

impl DecodeSysCallReturn for ComSetorgangles {
    fn decode_return(_word: isize) -> Self::Output {}
}
