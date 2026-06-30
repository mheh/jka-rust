use core::ffi::c_int;

use crate::{ffi::GameImport, shared::qboolean};

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_AREAS_CONNECTED` outbound game-to-engine syscall.
///
/// Queries whether `area1` and `area2` are connected, i.e. not separated by a
/// closed area portal. Mirrors the C ABI: two `int` inputs, `qboolean` return.
#[derive(Debug)]
pub struct GAreasConnectedArgs {
    area1: c_int,
    area2: c_int,
}

impl GAreasConnectedArgs {
    pub fn new(area1: c_int, area2: c_int) -> Self {
        Self { area1, area2 }
    }

    pub fn area1(&self) -> c_int {
        self.area1
    }

    pub fn area2(&self) -> c_int {
        self.area2
    }
}

/// `G_AREAS_CONNECTED` MP game imports syscall ABI token.
///
/// Raven: ( int area1, int area2 );
/// Source: `oracle/oracle/codemp/game/g_public.h:197`
pub struct GAreasConnected;

impl OutboundSysCall for GAreasConnected {
    type Import = GameImport;
    type Args = GAreasConnectedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_AREAS_CONNECTED;
}

impl EncodeSysCall for GAreasConnected {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.area1 as isize, a.area2 as isize])
    }
}

impl DecodeSysCallReturn for GAreasConnected {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
