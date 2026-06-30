use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_VIEW` outbound game-to-engine syscall.
///
/// Sets bot `client`'s view angles to `viewangles`.
#[derive(Debug)]
pub struct BotlibEaViewArgs {
    client: c_int,
    viewangles: *const vec3_t,
}

impl BotlibEaViewArgs {
    pub fn new(client: c_int, viewangles: *const vec3_t) -> Self {
        Self { client, viewangles }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn viewangles(&self) -> *const vec3_t {
        self.viewangles
    }
}

/// `BOTLIB_EA_VIEW` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:406`
pub struct BotlibEaView;

impl OutboundSysCall for BotlibEaView {
    type Import = GameImport;
    type Args = BotlibEaViewArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_VIEW;
}

impl EncodeSysCall for BotlibEaView {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, ptr_to_word(a.viewangles)])
    }
}

impl DecodeSysCallReturn for BotlibEaView {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
