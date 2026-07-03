//! `EntityId` — the entity handle (porting-rules §B5).

/// Raven's `gentity_t*` become an index into `GameWorld.entities`
/// (`mp_qshared::common::mp::gentity_t`, oracle home
/// `oracle/oracle/codemp/game/g_shared.h`). Module logic passes `(world, id)`
/// and re-indexes per access — GP2's `GpGroupId` precedent; no aliasing raw
/// pointers in safe code (§B5).
///
/// Source: `docs/architecture/state-ownership.md` § `EntityId` — the entity
/// handle (§B5).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);
