use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX` outbound game-to-engine syscall.
///
/// C ABI: `void trap_AAS_PresenceTypeBoundingBox(int presencetype, vec3_t mins, vec3_t maxs)`
///
/// The engine writes the bounding box for the given presence type into `mins`
/// and `maxs` (out-params); the call returns void.
#[derive(Debug)]
pub struct BotlibAasPresenceTypeBoundingBoxArgs {
    presencetype: c_int,
    mins: *mut f32,
    maxs: *mut f32,
}

impl BotlibAasPresenceTypeBoundingBoxArgs {
    pub fn new(presencetype: c_int, mins: *mut f32, maxs: *mut f32) -> Self {
        Self {
            presencetype,
            mins,
            maxs,
        }
    }

    pub fn presencetype(&self) -> c_int {
        self.presencetype
    }

    pub fn mins(&self) -> *mut f32 {
        self.mins
    }

    pub fn maxs(&self) -> *mut f32 {
        self.maxs
    }
}

/// `BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:362`
pub struct BotlibAasPresenceTypeBoundingBox;

impl OutboundSysCall for BotlibAasPresenceTypeBoundingBox {
    type Import = GameImport;
    type Args = BotlibAasPresenceTypeBoundingBoxArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX;
}

impl EncodeSysCall for BotlibAasPresenceTypeBoundingBox {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.presencetype as isize,
            ptr_to_word(a.mins),
            ptr_to_word(a.maxs),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasPresenceTypeBoundingBox {
    fn decode_return(_word: isize) -> Self::Output {}
}
