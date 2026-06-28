use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ANGLEVECTORS` outbound game-to-engine syscall.
///
/// Maps to `AngleVectors(angles, forward, right, up)` in `sv_game.cpp`.
/// All four arguments are raw pointers mirroring the C ABI's VMA(1)–VMA(4).
/// `forward`, `right`, and `up` are out-params: the engine writes through them.
#[derive(Debug)]
pub struct GAnglevectorsArgs {
    pub angles: *const vec3_t,
    pub forward: *mut vec3_t,
    pub right: *mut vec3_t,
    pub up: *mut vec3_t,
}

impl GAnglevectorsArgs {
    pub fn new(
        angles: *const vec3_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
    ) -> Self {
        Self { angles, forward, right, up }
    }

    pub fn angles(&self) -> *const vec3_t { self.angles }
    pub fn forward(&self) -> *mut vec3_t { self.forward }
    pub fn right(&self) -> *mut vec3_t { self.right }
    pub fn up(&self) -> *mut vec3_t { self.up }
}

pub struct GAnglevectors;

impl OutboundSysCall for GAnglevectors {
    type Import = GameImport;
    type Args = GAnglevectorsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ANGLEVECTORS;
}

impl EncodeSysCall for GAnglevectors {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.angles as *const _),
            ptr_to_word(a.forward as *const _),
            ptr_to_word(a.right as *const _),
            ptr_to_word(a.up as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GAnglevectors {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
