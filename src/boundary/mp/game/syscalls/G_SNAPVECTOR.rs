use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::vec3_t;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_SNAPVECTOR` outbound game-to-engine syscall.
///
/// Rounds each component of the 3-float vector `v` to the integer grid the
/// engine uses for network snapshots so client and server agree bit-for-bit.
/// C ABI: `void trap_SnapVector(float *v)`.
#[derive(Debug)]
pub struct GSnapvectorArgs {
    v: *mut vec3_t,
}

impl GSnapvectorArgs {
    pub fn new(v: *mut vec3_t) -> Self {
        Self { v }
    }

    pub fn v(&self) -> *mut vec3_t {
        self.v
    }
}

pub struct GSnapvector;

impl OutboundSysCall for GSnapvector {
    type Import = GameImport;
    type Args = GSnapvectorArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SNAPVECTOR;
}

impl EncodeSysCall for GSnapvector {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.v as *const u8)])
    }
}

impl DecodeSysCallReturn for GSnapvector {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
