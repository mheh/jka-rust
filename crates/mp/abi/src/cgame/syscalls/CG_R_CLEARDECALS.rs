use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `CG_R_CLEARDECALS` MP cgame imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:150`
pub struct CgRCleardecals;

impl OutboundSysCall for CgRCleardecals {
    type Import = MpCgameImport;
    type Args = ();
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_CLEARDECALS;
}

impl EncodeSysCall for CgRCleardecals {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgRCleardecals {
    fn decode_return(_word: isize) -> Self::Output {}
}
