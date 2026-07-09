use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// `G_PERPENDICULARVECTOR` outbound game-to-engine syscall.
///
/// C ABI: `PerpendicularVector( (float *)VMA(1), (const float *)VMA(2) )` → void
/// arg1: dst (*mut vec3_t, engine writes the result here)
/// arg2: src (*const vec3_t, input vector)
#[derive(Debug)]
pub struct GPerpendicularvectorArgs {
    dst: *mut vec3_t,
    src: *const vec3_t,
}

impl GPerpendicularvectorArgs {
    pub fn new(dst: *mut vec3_t, src: *const vec3_t) -> Self {
        Self { dst, src }
    }

    pub fn dst(&self) -> *mut vec3_t {
        self.dst
    }

    pub fn src(&self) -> *const vec3_t {
        self.src
    }
}

/// `G_PERPENDICULARVECTOR` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:285`
pub struct GPerpendicularvector;

impl OutboundSysCall for GPerpendicularvector {
    type Import = MpGameImport;
    type Args = GPerpendicularvectorArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_PERPENDICULARVECTOR;
}

impl EncodeSysCall for GPerpendicularvector {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.dst as *const _),
            ptr_to_word(a.src as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GPerpendicularvector {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
