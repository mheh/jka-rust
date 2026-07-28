//! `mp_cgame` — the MP client-game module (`cgame`), Raven's
//! `oracle/codemp/cgame/`. One of the three loadable MP DLLs (`jampgame`,
//! `cgame`, `ui`); it renders the client's view of the world, predicts the
//! local player, and draws the HUD.
//!
//! The cgame track (task #46) runs the stage skeleton DEC-45 ratified,
//! mirroring the ui track's U0-U6:
//!
//! - **C0** cgpackets tooling, **C1** crate/abi reconciliation audit
//! - **C2** root-type sit-down — settled as DEC-46: `CgWorld` is the three-way
//!   `cg`/`cgs`/`entities` spine, `CEntity` is owned with resolution enums,
//!   effect pools are generation-counted slabs with an explicit LRU queue,
//!   fn-ptr tables become closed enums, and `cg.sharedBuffer` is a pinned
//!   `Box<[u8; 2048]>`
//! - **C3** the trap layer — [`trap`], this stage: all 215 `trap_*` wrappers of
//!   `oracle/codemp/cgame/cg_syscalls.c`, on ui's `trap.rs` pattern (DEC-45.3:
//!   the module track uses a syscall-shaped trap layer, forced by the OpenJK
//!   ABI; `EngineHostView` stays the in-engine convention)
//! - **C4** the `CgGameCallbacks` implementor, **C5** transcription waves,
//!   **C6** gates (live rounds under `openjk.app` + the demo referee)
//!
//! The cgame root types [`local`] and [`lights`] landed under C1/C2; `CgWorld`
//! itself arrives with C4/C5.

pub mod bg_channel;
pub mod lights;
pub mod local;
pub mod trap;
