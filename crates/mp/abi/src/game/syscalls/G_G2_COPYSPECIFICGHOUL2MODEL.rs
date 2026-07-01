use core::ffi::{c_int, c_void};

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_COPYSPECIFICGHOUL2MODEL` outbound game-to-engine syscall.
///
/// C ABI: `void trap_G2API_CopySpecificGhoul2Model(void *g2From, int modelFrom, void *g2To, int modelTo)`
#[derive(Debug)]
pub struct GG2Copyspecificghoul2ModelArgs {
    g2_from: *mut c_void,
    model_from: c_int,
    g2_to: *mut c_void,
    model_to: c_int,
}

impl GG2Copyspecificghoul2ModelArgs {
    pub fn new(
        g2_from: *mut c_void,
        model_from: c_int,
        g2_to: *mut c_void,
        model_to: c_int,
    ) -> Self {
        Self {
            g2_from,
            model_from,
            g2_to,
            model_to,
        }
    }

    pub fn g2_from(&self) -> *mut c_void {
        self.g2_from
    }
    pub fn model_from(&self) -> c_int {
        self.model_from
    }
    pub fn g2_to(&self) -> *mut c_void {
        self.g2_to
    }
    pub fn model_to(&self) -> c_int {
        self.model_to
    }
}

/// `G_G2_COPYSPECIFICGHOUL2MODEL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:524`
pub struct GG2Copyspecificghoul2Model;

impl OutboundSysCall for GG2Copyspecificghoul2Model {
    type Import = MpGameImport;
    type Args = GG2Copyspecificghoul2ModelArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2_COPYSPECIFICGHOUL2MODEL;
}

impl EncodeSysCall for GG2Copyspecificghoul2Model {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.g2_from),
            a.model_from as isize,
            ptr_to_word(a.g2_to),
            a.model_to as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Copyspecificghoul2Model {
    fn decode_return(_word: isize) -> Self::Output {}
}
