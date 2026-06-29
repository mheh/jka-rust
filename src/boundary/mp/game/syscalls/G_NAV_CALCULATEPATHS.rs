use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_CALCULATEPATHS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCalculatepathsArgs {
    recalc: qboolean,
}

impl GNavCalculatepathsArgs {
    pub fn new(recalc: qboolean) -> Self {
        Self { recalc }
    }

    pub fn recalc(&self) -> qboolean {
        self.recalc
    }
}

/// `G_NAV_CALCULATEPATHS` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:303`
pub struct GNavCalculatepaths;

impl OutboundSysCall for GNavCalculatepaths {
    type Import = GameImport;
    type Args = GNavCalculatepathsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CALCULATEPATHS;
}

impl EncodeSysCall for GNavCalculatepaths {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.recalc as isize])
    }
}

impl DecodeSysCallReturn for GNavCalculatepaths {
    fn decode_return(_word: isize) -> Self::Output {}
}
