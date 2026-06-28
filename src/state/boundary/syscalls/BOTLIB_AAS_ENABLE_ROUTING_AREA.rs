use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_ENABLE_ROUTING_AREA` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasEnableRoutingAreaArgs {
    areanum: c_int,
    enable: c_int,
}

impl BotlibAasEnableRoutingAreaArgs {
    pub fn new(areanum: c_int, enable: c_int) -> Self {
        Self { areanum, enable }
    }

    pub fn areanum(&self) -> c_int {
        self.areanum
    }

    pub fn enable(&self) -> c_int {
        self.enable
    }
}

pub struct BotlibAasEnableRoutingArea;

impl OutboundSysCall for BotlibAasEnableRoutingArea {
    type Args = BotlibAasEnableRoutingAreaArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_ENABLE_ROUTING_AREA;
}

impl EncodeSysCall for BotlibAasEnableRoutingArea {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.areanum as isize, a.enable as isize])
    }
}

impl DecodeSysCallReturn for BotlibAasEnableRoutingArea {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
