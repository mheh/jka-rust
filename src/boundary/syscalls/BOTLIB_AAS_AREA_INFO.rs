use core::ffi::c_int;

use crate::codemp::game::be_aas_h::aas_areainfo_t;
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_AREA_INFO` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasAreaInfoArgs {
    areanum: c_int,
    info: *mut aas_areainfo_t,
}

impl BotlibAasAreaInfoArgs {
    pub fn new(areanum: c_int, info: *mut aas_areainfo_t) -> Self {
        Self { areanum, info }
    }

    pub fn areanum(&self) -> c_int {
        self.areanum
    }

    pub fn info(&self) -> *mut aas_areainfo_t {
        self.info
    }
}

pub struct BotlibAasAreaInfo;

impl OutboundSysCall for BotlibAasAreaInfo {
    type Args = BotlibAasAreaInfoArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_AREA_INFO;
}

impl EncodeSysCall for BotlibAasAreaInfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.areanum as isize,
            ptr_to_word(a.info),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasAreaInfo {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
