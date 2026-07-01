use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use abi_transport::pass_float;

/// Arguments for `CGAME_ASIN`.
///
/// The engine handler reads a single `VMF(1)` float and returns
/// `FloatAsInt( Q_asin( VMF(1) ) )`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:685`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:685`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:15`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:684`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:299`
#[derive(Debug)]
pub struct CgameAsinArgs {
    value: f32,
}

impl CgameAsinArgs {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `CGAME_ASIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:147`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:685`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:684`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:299`
pub struct CgameAsin;

impl OutboundSysCall for CgameAsin {
    type Import = MpCgameImport;
    type Args = CgameAsinArgs;
    type Output = f32;

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_ASIN;
}

impl EncodeSysCall for CgameAsin {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for CgameAsin {
    // The engine returns `FloatAsInt(...)`; reinterpret the word's low 32 bits as
    // the float result, mirroring the Raven `floatint_t` round-trip.
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
