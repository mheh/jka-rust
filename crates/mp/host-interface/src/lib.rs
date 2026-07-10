//! `mp_host_interface` — the Stage-0 host seam for the MP dedicated-engine port.
//!
//! Two traits transcribe the two service boundaries the ported engine subsystems
//! cross:
//!
//! * [`EngineHost`] — the in-engine service surface the §F C++-track subsystems
//!   (icarus, RMG, ghoul2, server NPCNav, ROFF) call on the host: trace, FS
//!   read/free/write/list, print/error, `VM_Call`, the shared-memory window,
//!   `flrand`/`irand`, `gentity`, the cvar services (register/integer/string/
//!   take-modified), `svs.time`, and the loader model-memory accessors
//!   (rulings 24 + 36 + 55). Per `engine-fork-discovery.md` ruling 11 the §F
//!   methods take `(&mut SubsystemState, &mut impl EngineHost)`; the aggregate
//!   `Engine` implements it through a split-borrow view struct, and dispatch
//!   tables store `&mut dyn EngineHost` (ruling 24), so the trait is
//!   dyn-compatible (no generic methods, no `Self`-by-value returns).
//! * [`PlatformHost`] — the `Sys_*`/`NET_*` platform seam (fork-8 ruling; UDP
//!   surface per ruling 33a): the real binary implements it with std; the
//!   referee injects a deterministic impl (fixed clock, scripted packets).
//!   Dylib loading is NOT here — the ported module loader already exists in
//!   `native_platform`.
//!
//! The [`mock`] module (always compiled — no feature gate, so the referee and
//! every host-taking subsystem's goldens link it with no build-matrix branch)
//! provides [`mock::MockHost`], a fixture-backed implementation of BOTH traits
//! per ruling 32: FS reads served from a caller-provided path→bytes map,
//! print/error captured into buffers, `flrand`/`irand` backed by a faithful
//! replica of Raven's `q_math.c` `holdrand` LCG.
//!
//! Crate path/name pinned by ruling 24: `crates/mp/host-interface`, package
//! `mp_host_interface`.

#![allow(non_camel_case_types, non_snake_case)]

pub mod engine_host;
pub mod platform_host;
pub mod vm_slot;
pub mod mock;

pub use engine_host::EngineHost;
pub use platform_host::PlatformHost;
pub use vm_slot::VmSlot;
