use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_RMG_INIT` outbound game-to-engine syscall.
///
/// Initialises the random-map generator against the terrain identified by
/// `terrain_id` (obtained from `G_CM_REGISTER_TERRAIN`).
#[derive(Debug)]
pub struct GRmgInitArgs {
    terrain_id: c_int,
}

impl GRmgInitArgs {
    pub fn new(terrain_id: c_int) -> Self {
        Self { terrain_id }
    }

    pub fn terrain_id(&self) -> c_int {
        self.terrain_id
    }
}

pub struct GRmgInit;

impl OutboundSysCall for GRmgInit {
    type Args = GRmgInitArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_RMG_INIT;
}

impl EncodeSysCall for GRmgInit {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.terrain_id as isize])
    }
}

impl DecodeSysCallReturn for GRmgInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
