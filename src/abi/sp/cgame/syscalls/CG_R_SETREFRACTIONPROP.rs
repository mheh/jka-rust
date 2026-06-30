use super::super::SpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::{abi::pass_float, shared::qboolean};

/// `CG_R_SETREFRACTIONPROP` SP cgame imports syscall ABI token.
///
/// Arguments for `CG_R_SETREFRACTIONPROP`.
///
/// Raven: set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww.
/// Raven wrapper: `syscall(CG_R_SETREFRACTIONPROP, PASSFLOAT(alpha), PASSFLOAT(stretch), prepost, negate);`
/// Raven transport: `tr_distortionAlpha = VMF(1); tr_distortionStretch = VMF(2); tr_distortionPrePost = (qboolean)args[3]; tr_distortionNegate = (qboolean)args[4]; return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:356-359`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:680-685`
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

    pub const fn prepost(&self) -> qboolean {
        self.prepost
    }

    pub const fn negate(&self) -> qboolean {
        self.negate
    }
}

/// `CG_R_SETREFRACTIONPROP` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:130`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:356-359`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:680-685`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:680-685`
pub struct CgRSetrefractionprop;

impl OutboundSysCall for CgRSetrefractionprop {
    type Import = SpCgameImport;
    type Args = CgRSetrefractionpropArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SETREFRACTIONPROP;
}

impl EncodeSysCall for CgRSetrefractionprop {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            pass_float(args.alpha()),
            pass_float(args.stretch()),
            args.prepost() as isize,
            args.negate() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgRSetrefractionprop {
    fn decode_return(_word: isize) -> Self::Output {}
}
