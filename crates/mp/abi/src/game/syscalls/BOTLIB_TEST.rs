use core::ffi::{c_char, c_int};

use super::super::MpGameImport;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_TEST` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibTestArgs {
    parm0: c_int,
    parm1: *mut c_char,
    parm2: *mut vec3_t,
    parm3: *mut vec3_t,
}

impl BotlibTestArgs {
    pub fn new(parm0: c_int, parm1: *mut c_char, parm2: *mut vec3_t, parm3: *mut vec3_t) -> Self {
        Self {
            parm0,
            parm1,
            parm2,
            parm3,
        }
    }

    pub fn parm0(&self) -> c_int {
        self.parm0
    }
    pub fn parm1(&self) -> *mut c_char {
        self.parm1
    }
    pub fn parm2(&self) -> *mut vec3_t {
        self.parm2
    }
    pub fn parm3(&self) -> *mut vec3_t {
        self.parm3
    }
}

/// `BOTLIB_TEST` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:350`
pub struct BotlibTest;

impl OutboundSysCall for BotlibTest {
    type Import = MpGameImport;
    type Args = BotlibTestArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_TEST;
}

impl EncodeSysCall for BotlibTest {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.parm0 as isize,
            ptr_to_word(a.parm1),
            ptr_to_word(a.parm2),
            ptr_to_word(a.parm3),
        ])
    }
}

impl DecodeSysCallReturn for BotlibTest {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
