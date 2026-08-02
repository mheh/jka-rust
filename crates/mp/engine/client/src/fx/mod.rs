//! The client effect engine (DEC-55.2, DEC-61).
//!
//! Raven's `Fx*` translation units become one owned `FxSystem` under `Engine.fx`,
//! a single `FxPrimitive` enum in place of the twelve-class hierarchy, and direct
//! calls where `SFxHelper` used to forward.
//!
//! Two pieces of Raven surface are dropped rather than ported, per porting-rules
//! §20. `CFxScheduler::MaterialImpact` has an entirely commented-out body, and
//! `CFxScheduler::CreateEffect( CPrimitiveTemplate*, SScheduledEffect* )` has no
//! caller in either tree.
//!
//! Source: `oracle/codemp/client/FxSystem.cpp`, `FxScheduler.cpp`,
//! `FxPrimitives.cpp`, `FxTemplate.cpp`, `FxUtil.cpp`, `FXExport.cpp`

pub mod cbezier;
pub mod ccylinder;
pub mod ceffect;
pub mod celectricity;
pub mod cemitter;
pub mod cflash;
pub mod cfx_range;
pub mod clight;
pub mod cline;
pub mod cmedia_handles;
pub mod coriented_particle;
pub mod cparticle;
pub mod cpoly;
pub mod cprimitive_template;
pub mod ctail;
pub mod ctrail;
pub mod emat_impact_effect;
pub mod eprim_type;
pub mod fx_clock;
pub mod fx_export;
pub mod fx_flags;
pub mod fx_harness;
pub mod fx_host;
pub mod fx_primitive;
pub mod fx_scheduler;
pub mod fx_system;
pub mod fx_template_parse;
pub mod fx_util;
pub mod seffect_template;

pub use fx_harness::FxHarness;
pub use fx_host::FxHost;
pub use fx_system::FxSystem;
