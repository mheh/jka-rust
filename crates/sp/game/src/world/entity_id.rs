//! SP `EntityId` — the entity handle (porting-rules §B5).

/// Raven's `gentity_t*` become an index into `GameWorld.entities`
/// (`sp_qshared::common::sp::gentity_t`, oracle home
/// `oracle/code/game/g_shared.h`). Module logic passes `(world, id)`
/// and re-indexes per access — GP2's `GpGroupId` precedent; no aliasing raw
/// pointers in safe code (§B5). SP mirror per DEC-04.
///
/// Source: `docs/architecture/state-ownership.md` § `EntityId` — the entity
/// handle (§B5).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);
