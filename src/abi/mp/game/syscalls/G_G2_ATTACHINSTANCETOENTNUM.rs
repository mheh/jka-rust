use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::{ffi::GameImport, shared::qboolean};

/// `G_G2_ATTACHINSTANCETOENTNUM` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2AttachinstancetoentnumArgs {
    ghoul2: *mut c_void,
    entity_num: c_int,
    server: qboolean,
}

impl GG2AttachinstancetoentnumArgs {
    pub fn new(ghoul2: *mut c_void, entity_num: c_int, server: qboolean) -> Self {
        Self {
            ghoul2,
            entity_num,
            server,
        }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn entity_num(&self) -> c_int {
        self.entity_num
    }
    pub fn server(&self) -> qboolean {
        self.server
    }
}

/// `G_G2_ATTACHINSTANCETOENTNUM` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:564`
pub struct GG2Attachinstancetoentnum;

impl OutboundSysCall for GG2Attachinstancetoentnum {
    type Import = GameImport;
    type Args = GG2AttachinstancetoentnumArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_G2_ATTACHINSTANCETOENTNUM;
}

impl EncodeSysCall for GG2Attachinstancetoentnum {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2()),
            a.entity_num() as isize,
            a.server() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Attachinstancetoentnum {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
