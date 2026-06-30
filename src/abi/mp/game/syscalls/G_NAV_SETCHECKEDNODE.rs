use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_SETCHECKEDNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavSetcheckednodeArgs {
    way_point: c_int,
    ent: c_int,
    value: c_int,
}

impl GNavSetcheckednodeArgs {
    pub fn new(way_point: c_int, ent: c_int, value: c_int) -> Self {
        Self {
            way_point,
            ent,
            value,
        }
    }

    pub fn way_point(&self) -> c_int {
        self.way_point
    }
    pub fn ent(&self) -> c_int {
        self.ent
    }
    pub fn value(&self) -> c_int {
        self.value
    }
}

/// `G_NAV_SETCHECKEDNODE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:336`
pub struct GNavSetcheckednode;

impl OutboundSysCall for GNavSetcheckednode {
    type Import = MpGameImport;
    type Args = GNavSetcheckednodeArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_SETCHECKEDNODE;
}

impl EncodeSysCall for GNavSetcheckednode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.way_point as isize, a.ent as isize, a.value as isize])
    }
}

impl DecodeSysCallReturn for GNavSetcheckednode {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
