//! `CollisionWorld` — the `cmg` + `SubBSP` collision state (state-ownership STATE-D2).

use crate::cm::ccmshader::CCMShader;
use crate::cm_terrain::CmLandScape;

/// The `Engine.cm` field: `cmg`/`SubBSP[32]`/`NumSubBSP`/trace counters. An
/// instance-shaped value (STATE-D2), zero/Default-initialized by `Engine::new`
/// (mirroring Raven's static zero-init of `clipMap_t cmg`), populated in place by
/// `CM_LoadMap`. Internals are subsystem detail (non-goal), placeheld here so the
/// frozen `Engine` struct can name the field.
///
/// Source: `oracle/codemp/qcommon/cm_load.cpp:37,60-61`
pub struct CollisionWorld {
    //TODO: Port CollisionWorld fields (cmg + SubBSP + trace counters)
    // Source: oracle/codemp/qcommon/cm_load.cpp:37,60-61
    _private: (),

    /// Raven `cmg.landScape` — the nullable terrain landscape, `Some` only on an
    /// RMG terrain map. Set by `register_terrain` / the Wave-20 `G_RMG_INIT` arm;
    /// `None` before (Raven's `CCMLandScape *landScape` NULL pointer, faithful as
    /// `Option`, RMG-D1 / ruling 28). The `CmLandScape` struct definition lands at
    /// the wave-0–4 collision consumers (RMG-D5 struct-definition-lands-early), so
    /// this field type-checks before the wave-16 construction bodies. The
    /// `terrain_*` forwarders (Seam §C, ruling 38) — inherent `impl CollisionWorld`
    /// methods that resolve `self.land_scape` internally — are transcribed in
    /// `cm_terrain.rs` (Files roster, class `CCMLandScape`), not here.
    ///
    /// Source: `oracle/codemp/qcommon/cm_landscape.h:135`; `cm_local.h:155`
    pub land_scape: Option<CmLandScape>,
}

impl CollisionWorld {
    /// `CCMShader *CM_GetShaderInfo( const char *name )` — decl `cm_local.h:303`,
    /// def `cm_shader.cpp:498`. `&mut self` because a miss registers a fresh
    /// `CCMShader` into `cmShaderTable` (`Hunk_Alloc` + `CM_ParseShader`,
    /// `cm_shader.cpp:512-522`), so the call mutates the clipmap;
    /// `Option<&CCMShader>` mirrors the returned pointer. Frozen extern binding
    /// per `docs/subsystems/rmg-terrain.md` (ruling 41 / RMG-D5), reached by
    /// `CmLandScape::load_terrain_def`'s `altitudetexture`/`water` cases.
    ///
    /// **Owned by the `cm` C-track qcommon packet (the wider-clipmap shader
    /// machinery — `cmShaderTable`/`Hunk_Alloc`/`CM_GetShaderText`/
    /// `CM_ParseShader`, `cm_shader.cpp:28-30,475-522`), landing with the `cm`
    /// waves.** That lane has not landed in this tree (its `CollisionWorld`
    /// fields are the sibling `//TODO: Port CollisionWorld fields` above), so
    /// this is a callable no-op that returns `None` (no shader registered) to
    /// unblock the terrain lane build; `load_terrain_def` is exercised by no
    /// runnable golden (golden #4 is a reported blocker), so the no-op is
    /// observationally inert here. Reconciled — full port + faithful table
    /// lookup — when the cm-shader wave lands.
    //TODO: Port CM_GetShaderInfo
    // Source: oracle/codemp/qcommon/cm_shader.cpp:498-524
    pub fn cm_get_shader_info(&mut self, name: &str) -> Option<&CCMShader> {
        let _ = name;
        None
    }
}
