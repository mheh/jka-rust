use super::super::MpGameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// `G_NAV_SETPATHSCALCULATED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavSetpathscalculatedArgs {
    new_val: qboolean,
}

impl GNavSetpathscalculatedArgs {
    pub fn new(new_val: qboolean) -> Self {
        Self { new_val }
    }

    pub fn new_val(&self) -> qboolean {
        self.new_val
    }
}

/// `G_NAV_SETPATHSCALCULATED` MP game imports syscall ABI token.
///
/// Raven: rww - END NPC NAV TRAPS
/// Source: `oracle/codemp/game/g_public.h:339`
pub struct GNavSetpathscalculated;

impl OutboundSysCall for GNavSetpathscalculated {
    type Import = MpGameImport;
    type Args = GNavSetpathscalculatedArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_SETPATHSCALCULATED;
}

impl EncodeSysCall for GNavSetpathscalculated {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.new_val as isize)])
    }
}

impl DecodeSysCallReturn for GNavSetpathscalculated {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
