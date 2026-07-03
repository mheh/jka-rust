//! `CollisionWorld` — the `cmg` + `SubBSP` collision state (state-ownership STATE-D2).

/// The `Engine.cm` field: `cmg`/`SubBSP[32]`/`NumSubBSP`/trace counters. An
/// instance-shaped value (STATE-D2), zero/Default-initialized by `Engine::new`
/// (mirroring Raven's static zero-init of `clipMap_t cmg`), populated in place by
/// `CM_LoadMap`. Internals are subsystem detail (non-goal), placeheld here so the
/// frozen `Engine` struct can name the field.
///
/// Source: `oracle/oracle/codemp/qcommon/cm_load.cpp:37,60-61`
pub struct CollisionWorld {
    //TODO: Port CollisionWorld fields (cmg + SubBSP + trace counters)
    // Source: oracle/oracle/codemp/qcommon/cm_load.cpp:37,60-61
    _private: (),
}
