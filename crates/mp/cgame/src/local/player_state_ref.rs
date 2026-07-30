#![allow(non_camel_case_types, non_snake_case)]

/// Raven `playerState_t *playerState` on `centity_t`, as the DEC-46.2
/// resolution enum. Raven's pointer only ever aims at one of four places —
/// `cg.predictedPlayerState` (the locally predicted client),
/// `cg.predictedVehicleState` (the vehicle it pilots, during the predict
/// window), a playerState mirrored out of the snapshot (`cgSendPS`), or
/// `cg.snap->ps` itself on the demo/follow/no-predict path
/// (`cg_snapshot.c:193` → `cg_playerstate.c:242`) — so the port records
/// *which* and use sites resolve it through `CgWorld` at the moment of the
/// read, instead of holding an aliasing pointer into it.
///
/// `CgWorld.bg_ents` rows carry the same referent as a raw pointer for the bg
/// tier; every site that changes an entity's arm updates its bg view row in
/// the same breath (DEC-47.2).
///
/// - `None`: not a player-backed entity — Raven's null pointer.
/// - `Predicted`: resolves to `cg.predictedPlayerState`.
/// - `PredictedVehicle`: resolves to `cg.predictedVehicleState`.
/// - `Snap`: resolves to entity `n`'s snapshot playerState
///   (`cgSendPSPool[n]`).
/// - `ActiveSnap(slot)`: resolves to `cg.activeSnapshots[slot].ps` — the
///   address Raven captures as `&cg.snap->ps`. The slot is pinned at
///   assignment time, exactly like the raw pointer, so a later snapshot
///   transition does not retarget it.
///
/// Source: `oracle/codemp/cgame/cg_local.h:336`, `docs/decisions.md` DEC-46
/// (ruling 2), DEC-47 (ruling 2)
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[repr(u8)]
pub enum PlayerStateRef {
    #[default]
    None = 0,
    Predicted,
    PredictedVehicle,
    Snap,
    ActiveSnap(u8),
}
