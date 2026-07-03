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
/// over the concrete call type `C`. Took the retired placeholder
/// `OutboundSysCallExecutor`'s place (zero impls; its removal also retired the
/// `message.rs` helper layer — SEAM-D5). Per-call backend selection differs by
/// capability bound, which is why "how to run a call" cannot live on
/// `OutboundSysCall` itself.
///
/// Source: `docs/architecture/engine-seam.md` § Outbound execution trait (SEAM-D5).
pub trait Execute<C: OutboundSysCall> {
    fn execute(&self, args: C::Args) -> C::Output;
}
