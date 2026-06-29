use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `G_BOT_UPDATEWAYPOINTS` outbound game-to-engine syscall.
///
/// Hands the engine `wpnum` waypoint objects (`wps`, an array of `wpobject_t *`).
/// `wpobject_t` is not yet ported, so the array is passed opaquely as `void **`.
#[derive(Debug)]
pub struct GBotUpdatewaypointsArgs {
    wpnum: c_int,
    wps: *mut *mut c_void,
}

impl GBotUpdatewaypointsArgs {
    pub fn new(wpnum: c_int, wps: *mut *mut c_void) -> Self {
        Self { wpnum, wps }
    }

    pub fn wpnum(&self) -> c_int {
        self.wpnum
    }

    pub fn wps(&self) -> *mut *mut c_void {
        self.wps
    }
}

/// `G_BOT_UPDATEWAYPOINTS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:575`
pub struct GBotUpdatewaypoints;

impl OutboundSysCall for GBotUpdatewaypoints {
    type Import = GameImport;
    type Args = GBotUpdatewaypointsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_BOT_UPDATEWAYPOINTS;
}

impl EncodeSysCall for GBotUpdatewaypoints {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.wpnum as isize, ptr_to_word(a.wps as *const _)])
    }
}

impl DecodeSysCallReturn for GBotUpdatewaypoints {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
