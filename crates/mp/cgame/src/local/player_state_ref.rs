#![allow(non_camel_case_types, non_snake_case)]

/// Raven `playerState_t *playerState` on `centity_t`, as the DEC-46.2
/// resolution enum. Raven's pointer only ever aims at one of two places —
/// `cg.predictedPlayerState` (the locally predicted client) or a playerState
/// mirrored out of the snapshot (`cgSendPS`) — so the port records *which* and
/// use sites resolve it through `CgWorld` at the moment of the read, instead of
/// holding an aliasing pointer into it.
///
/// - `None`: not a player-backed entity — Raven's null pointer.
/// - `Predicted`: resolves to `cg.predictedPlayerState`.
/// - `Snap`: resolves to entity `n`'s snapshot playerState.
///
/// Source: `oracle/codemp/cgame/cg_local.h:336`, `docs/decisions.md` DEC-46
/// (ruling 2)
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[repr(u8)]
pub enum PlayerStateRef {
    #[default]
    None = 0,
    Predicted,
    Snap,
}
