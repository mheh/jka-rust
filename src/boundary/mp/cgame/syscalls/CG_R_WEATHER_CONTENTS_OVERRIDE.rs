use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_WEATHER_CONTENTS_OVERRIDE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRWeatherContentsOverrideArgs {
    contents: c_int,
}

impl CgRWeatherContentsOverrideArgs {
    pub const fn new(contents: c_int) -> Self {
        Self { contents }
    }

    pub const fn contents(&self) -> c_int {
        self.contents
    }
}

/// `CG_R_WEATHER_CONTENTS_OVERRIDE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:333`
pub struct CgRWeatherContentsOverride;

impl OutboundSysCall for CgRWeatherContentsOverride {
    type Import = MpCgameImport;
    type Args = CgRWeatherContentsOverrideArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_WEATHER_CONTENTS_OVERRIDE;
}

impl EncodeSysCall for CgRWeatherContentsOverride {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.contents() as isize])
    }
}

impl DecodeSysCallReturn for CgRWeatherContentsOverride {
    fn decode_return(_word: isize) -> Self::Output {}
}
