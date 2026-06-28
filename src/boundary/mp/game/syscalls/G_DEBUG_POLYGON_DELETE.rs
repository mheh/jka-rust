use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_DEBUG_POLYGON_DELETE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GDebugPolygonDeleteArgs {
    id: c_int,
}

impl GDebugPolygonDeleteArgs {
    pub fn new(id: c_int) -> Self {
        Self { id }
    }

    pub fn id(&self) -> c_int {
        self.id
    }
}

pub struct GDebugPolygonDelete;

impl OutboundSysCall for GDebugPolygonDelete {
    type Import = GameImport;
    type Args = GDebugPolygonDeleteArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_DEBUG_POLYGON_DELETE;
}

impl EncodeSysCall for GDebugPolygonDelete {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.id as isize])
    }
}

impl DecodeSysCallReturn for GDebugPolygonDelete {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
