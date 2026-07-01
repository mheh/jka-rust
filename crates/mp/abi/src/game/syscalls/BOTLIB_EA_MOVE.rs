use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::pass_float;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_MOVE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaMoveArgs {
    client: c_int,
    dir: *const vec3_t,
    speed: f32,
}

impl BotlibEaMoveArgs {
    pub fn new(client: c_int, dir: *const vec3_t, speed: f32) -> Self {
        Self { client, dir, speed }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn dir(&self) -> *const vec3_t {
        self.dir
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }
}

/// `BOTLIB_EA_MOVE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:405`
pub struct BotlibEaMove;

impl OutboundSysCall for BotlibEaMove {
    type Import = MpGameImport;
    type Args = BotlibEaMoveArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE;
}

impl EncodeSysCall for BotlibEaMove {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, ptr_to_word(a.dir), pass_float(a.speed)])
    }
}

impl DecodeSysCallReturn for BotlibEaMove {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
