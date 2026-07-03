/// A typed game-to-engine syscall.
///
/// These are the callbacks jampgame makes through the syscall pointer the
/// executable provided via `dllEntry`.
pub trait OutboundSysCall {
    type Import;
    type Args;
    type Output;

    const IMPORT: Self::Import;
}

/// The single seam every outbound backend implements (SEAM-D5), genericized
/// over the concrete call type `C`. Replaces the placeholder marker trait
/// `OutboundSysCallExecutor` below (which has zero impls); per-call backend
/// selection differs by capability bound, which is why "how to run a call"
/// cannot live on `OutboundSysCall` itself.
///
/// Source: `docs/architecture/engine-seam.md` § Outbound execution trait (SEAM-D5).
pub trait Execute<C: OutboundSysCall> {
    fn execute(&self, args: C::Args) -> C::Output;
}

/// Route an outbound game-to-engine syscall.
///
/// Placeholder marker trait with zero impls — superseded by `Execute<C>`
/// (SEAM-D5). Retained here (not deleted) to keep the existing `message.rs`
/// helper layer compiling in this skeleton seed; its removal (which retires
/// `message.rs`) is a follow-up (see skeleton FINDINGS).
pub trait OutboundSysCallExecutor {
    fn call_outbound<C>(&self, args: C::Args) -> C::Output
    where
        C: OutboundSysCall;
}
