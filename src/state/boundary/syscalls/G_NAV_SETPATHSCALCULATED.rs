use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GNavSetpathscalculated;

impl OutboundSysCall for GNavSetpathscalculated {
    type Args = GNavSetpathscalculatedArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_SETPATHSCALCULATED;
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
