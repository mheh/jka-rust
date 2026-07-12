use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_GETPROJECTEDNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetprojectednodeArgs {
    origin: *const vec3_t,
    node_id: i32,
}

impl GNavGetprojectednodeArgs {
    pub fn new(origin: *const vec3_t, node_id: i32) -> Self {
        Self { origin, node_id }
    }

    pub fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub fn node_id(&self) -> i32 {
        self.node_id
    }
}

/// `G_NAV_GETPROJECTEDNODE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:317`
pub struct GNavGetprojectednode;

impl OutboundSysCall for GNavGetprojectednode {
    type Import = MpGameImport;
    type Args = GNavGetprojectednodeArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETPROJECTEDNODE;
}

impl EncodeSysCall for GNavGetprojectednode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.origin()), a.node_id() as isize])
    }
}

impl DecodeSysCallReturn for GNavGetprojectednode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
