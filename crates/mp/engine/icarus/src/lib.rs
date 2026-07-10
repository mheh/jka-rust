//! `mp_engine_icarus` — the MP ICARUS scripting subsystem (§F idiomatic
//! reimplementation of `oracle/codemp/icarus/`).
//!
//! The crate root defines [`Icarus`], the fork-2 aggregate that owns every
//! ICARUS file-scope global as a field (no `static mut`); it attaches to
//! `mp_engine_core::Engine` as a plain `Default`-initialized `icarus` field
//! (ICARUS-D3 / rulings 11/12) reached through the ICARUS-D2 `EngineHostView`
//! split-borrow. Per-class subsystems live in the module dirs below.
//! Source: `docs/subsystems/icarus.md` § State ownership;
//! `docs/handoffs/engine-fork-discovery.md:126-130` (ruling 12)

// A handful of §F fns are faithful no-ops whose parameters are retained for
// signature fidelity but intentionally unused — chiefly the MP-unsupported
// `CGCam_*` camera targets (`Q3_Interface.cpp:689-758`), which ignore their
// vector/scalar args and just emit the "NOT SUPPORTED IN MP" warning.
#![allow(unused_variables)]

use std::collections::HashMap;

use mp_qshared::shared::limits::MAX_GENTITIES;

use crate::game_interface::pscript_s::Pscript;
use crate::instance::icarus_instance::{IcarusInstance, SequencerId};
use crate::interface::interface_export_s::InterfaceExport;
use crate::taskmanager::ctask_manager::TaskManager;

pub mod blockstream;
pub mod game_interface;
pub mod instance;
pub mod interface;
pub mod interpreter;
pub mod memory;
pub mod q3_interface;
pub mod q3_registers;
pub mod sequence;
pub mod sequencer;
pub mod taskmanager;
pub mod tokenizer;

/// The ICARUS subsystem aggregate — the synthesized owner of every ICARUS
/// file-scope global (State-ownership table). Not a Raven class.
///
/// Constructed by [`Default`] (hand-written, see below) and attached as the
/// `icarus` field on `mp_engine_core::Engine`; "is ICARUS initialized?" is
/// answered by Raven's own NULL-flags (`instance.is_some()`,
/// `sequencers[n].is_some()`), not by wrapping the subsystem in `Option`.
/// Source: `docs/subsystems/icarus.md` § State ownership
pub struct Icarus {
    /// Raven `ICARUS_Instance *iICARUS` — `Some` mirrors `iICARUS != NULL`.
    pub instance: Option<IcarusInstance>,
    /// Raven `CSequencer *gSequencers[MAX_GENTITIES]` — non-owning per-entity
    /// index into `IcarusInstance.sequencers`.
    pub sequencers: Box<[Option<SequencerId>; MAX_GENTITIES]>,
    /// Raven `CTaskManager *gTaskManagers[MAX_GENTITIES]` — owning per-entity.
    pub task_managers: Box<[Option<TaskManager>; MAX_GENTITIES]>,
    /// Raven `bufferlist_t ICARUS_BufferList` — cached `.IBI` blobs.
    pub buffer_list: HashMap<String, Pscript>,
    /// Raven `entlist_t ICARUS_EntList` — script-name → entnum.
    pub ent_list: HashMap<String, i32>,
    /// Raven `int ICARUS_entFilter = -1`.
    pub ent_filter: i32,
    /// Raven `interface_export_t interface_export` — the outbound `I_*` table.
    pub interface_export: InterfaceExport,
    /// Raven `varString_m varStrings` — script string variables.
    pub var_strings: HashMap<String, String>,
    /// Raven `varFloat_m varFloats` — script float variables.
    pub var_floats: HashMap<String, f32>,
    /// Raven `varString_m varVectors` — script vector variables.
    pub var_vectors: HashMap<String, String>,
    /// Raven `int numVariables = 0`.
    pub num_variables: i32,
}

impl Default for Icarus {
    /// Hand-written (NOT `#[derive]`): the two slot arrays have no blanket
    /// `[T; N]: Default` impl and must be built explicitly, and `ent_filter`
    /// must seed `-1` (Raven's `int ICARUS_entFilter = -1`,
    /// `GameInterface.cpp:23`; derive's `0` would flip the debug-print gate).
    /// `interface_export` seeds the real `Q3_*`/`I_*` fns (see
    /// `interface/interface_export_s.rs`).
    fn default() -> Self {
        Icarus {
            instance: None,
            sequencers: Box::new(std::array::from_fn(|_| None)),
            task_managers: Box::new(std::array::from_fn(|_| None)),
            buffer_list: HashMap::new(),
            ent_list: HashMap::new(),
            ent_filter: -1,
            interface_export: InterfaceExport::default(),
            var_strings: HashMap::new(),
            var_floats: HashMap::new(),
            var_vectors: HashMap::new(),
            num_variables: 0,
        }
    }
}
