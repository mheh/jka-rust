use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::{abi::pass_float, shared::qboolean};

/// Arguments for `CG_R_SETREFRACTIONPROP`.
///
/// Raven: set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww.
/// Raven wrapper: `syscall(CG_R_SETREFRACTIONPROP, PASSFLOAT(alpha), PASSFLOAT(stretch), prepost, negate);`
/// Raven transport: `tr_distortionAlpha = VMF(1); tr_distortionStretch = VMF(2); tr_distortionPrePost = (qboolean)args[3]; tr_distortionNegate = (qboolean)args[4]; return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:401-404`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2292`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:947-952`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRSetrefractionpropArgs {
    alpha: f32,
    stretch: f32,
    prepost: qboolean,
    negate: qboolean,
}

impl CgRSetrefractionpropArgs {
    pub const fn new(alpha: f32, stretch: f32, prepost: qboolean, negate: qboolean) -> Self {
        Self {
            alpha,
            stretch,
            prepost,
            negate,
        }
    }

    pub const fn alpha(&self) -> f32 {
        self.alpha
    }

    pub const fn stretch(&self) -> f32 {
        self.stretch
    }
}

/// `CG_R_SETREFRACTIONPROP` MP cgame imports syscall ABI token.
///
/// Raven: set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:166`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:401-404`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:947-952`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:947-952`
pub struct CgRSetrefractionprop;

impl OutboundSysCall for CgRSetrefractionprop {
    type Import = MpCgameImport;
    type Args = CgRSetrefractionpropArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SETREFRACTIONPROP;
}

impl EncodeSysCall for CgRSetrefractionprop {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.alpha()),
            pass_float(args.stretch()),
            args.prepost as isize,
            args.negate as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRSetrefractionprop {
    fn decode_return(_word: isize) -> Self::Output {}
}
