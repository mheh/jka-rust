use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKEDNODE` outbound game-to-engine syscall.
///
/// Mirrors `trap_Nav_CheckedNode(way_point: i32, ent: i32) -> i32`.
#[derive(Debug)]
pub struct GNavCheckednodeArgs {
    way_point: c_int,
    ent: c_int,
}

impl GNavCheckednodeArgs {
    pub fn new(way_point: c_int, ent: c_int) -> Self {
        Self { way_point, ent }
    }

    pub fn way_point(&self) -> c_int {
        self.way_point
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }
}

/// `G_NAV_CHECKEDNODE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:335`
pub struct GNavCheckednode;

impl OutboundSysCall for GNavCheckednode {
    type Import = MpGameImport;
    type Args = GNavCheckednodeArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CHECKEDNODE;
}

impl EncodeSysCall for GNavCheckednode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.way_point as isize, a.ent as isize])
    }
}

impl DecodeSysCallReturn for GNavCheckednode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
