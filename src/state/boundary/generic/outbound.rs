use crate::ffi::GameImport;

/// A typed game-to-engine syscall.
///
/// These are the callbacks jampgame makes through the syscall pointer the
/// executable provided via `dllEntry`.
pub trait OutboundSysCall {
    type Args;
    type Output;

    const IMPORT: GameImport;
}

/// Route an outbound game-to-engine syscall.
pub trait OutboundSysCallExecutor {
    fn call_outbound<C>(&self, args: C::Args) -> C::Output
    where
        C: OutboundSysCall;
}
