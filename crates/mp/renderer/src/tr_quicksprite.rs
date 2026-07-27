//! Raven `tr_quicksprite.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_quicksprite.cpp`

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::shared::{vec2_t, vec4_t};

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_backend::GL_State;
use crate::tr_local::stage_vars::SHADER_MAX_VERTEXES;
use crate::tr_local::texture_bundle_t::textureBundle_t;

/// Raven `CQuickSpriteSystem` — the quad-batching helper the backend fills
/// via `StartGroup`/`Add`/`EndGroup` and flushes to GL as one draw call.
/// Per-subsystem state struct named by this wave (DEC-37 A13.3): render-
/// thread-only batch scratch, not a `RenderAssets`/`FrameState` member (no
/// row in `## State ownership` — this class is not a Raven global, it is
/// per-instance state a later wave's owner threads in as `&mut`).
///
/// Type definition source: `oracle/codemp/renderer/tr_quicksprite.h` (class
/// body not in this wave's packet; fields below cover only what
/// `CQuickSpriteSystem`/`~CQuickSpriteSystem`/`StartGroup`/`EndGroup`/`Add`
/// touch — later waves porting `Flush` and the remaining methods extend this
/// struct rather than forking a second one).
pub struct CQuickSpriteSystem {
    /// `mNextVert` — next free slot in the batch (always advances by 4, one
    /// quad at a time).
    pub next_vert: usize,
    /// `mGLStateBits` — GL state bits for the active group.
    pub gl_state_bits: u32,
    /// `mUseFog`.
    pub use_fog: bool,
    /// `mFogIndex`.
    pub fog_index: i32,
    /// `mTextureCoords[SHADER_MAX_VERTEXES]` — the fixed per-quad corner UVs
    /// the constructor lays down once (bottom-right/top-right/top-left/
    /// bottom-left, repeating every 4 slots).
    pub tex_coords: [vec2_t; SHADER_MAX_VERTEXES],
    /// `mVerts[SHADER_MAX_VERTEXES]` — batched vertex positions.
    pub verts: [vec4_t; SHADER_MAX_VERTEXES],
    /// `mColors[SHADER_MAX_VERTEXES]` — batched per-vertex colors.
    pub colors: [color4ub_t; SHADER_MAX_VERTEXES],
    /// `mFogTextureCoords[SHADER_MAX_VERTEXES]` — batched per-vertex fog UVs.
    pub fog_texture_coords: [vec2_t; SHADER_MAX_VERTEXES],
}

impl CQuickSpriteSystem {
    /// Raven `CQuickSpriteSystem::CQuickSpriteSystem`.
    ///
    /// Source: `oracle/codemp/renderer/tr_quicksprite.cpp:25-44`
    pub fn new() -> CQuickSpriteSystem {
        let mut tex_coords = [[0.0f32; 2]; SHADER_MAX_VERTEXES];

        let mut i = 0;
        while i < SHADER_MAX_VERTEXES {
            // Bottom right
            tex_coords[i + 0][0] = 1.0;
            tex_coords[i + 0][1] = 1.0;
            // Top right
            tex_coords[i + 1][0] = 1.0;
            tex_coords[i + 1][1] = 0.0;
            // Top left
            tex_coords[i + 2][0] = 0.0;
            tex_coords[i + 2][1] = 0.0;
            // Bottom left
            tex_coords[i + 3][0] = 0.0;
            tex_coords[i + 3][1] = 1.0;

            i += 4;
        }

        CQuickSpriteSystem {
            next_vert: 0,
            gl_state_bits: 0,
            use_fog: false,
            fog_index: 0,
            tex_coords,
            verts: [[0.0; 4]; SHADER_MAX_VERTEXES],
            colors: [[0; 4]; SHADER_MAX_VERTEXES],
            fog_texture_coords: [[0.0; 2]; SHADER_MAX_VERTEXES],
        }
    }

    // Raven `CQuickSpriteSystem::~CQuickSpriteSystem` — an empty body (no
    // manual cleanup); every field above is owned (`Vec`-free fixed arrays),
    // so Rust's default drop already reproduces it. No `Drop` impl needed.
    //
    // Source: `oracle/codemp/renderer/tr_quicksprite.cpp:46-49`

    /// Raven `CQuickSpriteSystem::StartGroup`.
    ///
    /// Source: `oracle/codemp/renderer/tr_quicksprite.cpp:147-164`
    pub fn start_group(&mut self, _bundle: &textureBundle_t, glbits: u32, fog_index: i32) {
        self.next_vert = 0;

        // mTexBundle = bundle;
        // DEFERRED: textureBundle_t identity — the R2 Group-2 disposition
        // table owns `textureBundle_t`'s replacement shape (image ->
        // Handle<Image>, tcGenVectors -> owned [vec3_t; 2], texMods ->
        // Vec<TexModInfo>) at the tr_shader/tr_image wave, not this one; the
        // interior-safety law forbids storing its current tier-2 raw-pointer
        // shape in this new struct, and `Flush` (the sole reader of
        // mTexBundle) is outside this wave's scope. Escalated rather than
        // storing a raw pointer or inventing an addressing scheme.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:151
        self.gl_state_bits = glbits;

        if fog_index != -1 {
            self.use_fog = true;
            self.fog_index = fog_index;
        } else {
            self.use_fog = false;
        }

        // DEFERRED: R4 — qglDisable(GL_CULL_FACE) (DEC-37 A13.2). The
        // fixed-function GL surface has no R3 home; the backend is an
        // idiomatic wgpu rewrite, not a GL transcription.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:163
    }

    /// Raven `CQuickSpriteSystem::EndGroup`.
    ///
    /// Source: `oracle/codemp/renderer/tr_quicksprite.cpp:167-173`
    pub fn end_group(&mut self) {
        // DEFERRED: R4 — CQuickSpriteSystem::Flush() draw-side effect
        // (DEC-37 A13.2): the batch's sole job is submitting mVerts/mColors/
        // mFogTextureCoords through the fixed-function GL pipeline; `Flush`
        // is not in this wave's fn list (not a resolved in-module callee)
        // and stays deferred whole rather than invented. PORT-NOTE: Flush
        // also resets mNextVert to 0 in the oracle — that reclaim lands with
        // Flush's own wave.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:169
        //
        // DEFERRED: R4 — qglColor4ub(255,255,255,255); qglEnable(GL_CULL_FACE)
        // (DEC-37 A13.2). Fixed-function GL surface, no R3 home.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:171-172
    }

    /// Raven `CQuickSpriteSystem::Add`.
    ///
    /// Source: `oracle/codemp/renderer/tr_quicksprite.cpp:178-222`
    pub fn add(&mut self, pointdata: [vec4_t; 4], color: color4ub_t, fog: Option<vec2_t>) {
        if self.next_vert > SHADER_MAX_VERTEXES - 4 {
            // Raven flushes the full batch here, which resets `mNextVert` to 0;
            // without that reclaim the writes below run off the end of the
            // vertex arrays, so this arm cannot fall through.
            todo!("Port CQuickSpriteSystem::Flush — oracle/codemp/renderer/tr_quicksprite.cpp:169")
        }

        self.verts[self.next_vert..self.next_vert + 4].copy_from_slice(&pointdata);

        // Set up color
        for offset in 0..4 {
            self.colors[self.next_vert + offset] = color;
        }

        match fog {
            Some(fog) => {
                for offset in 0..4 {
                    self.fog_texture_coords[self.next_vert + offset] = fog;
                }

                self.use_fog = true;
            }
            None => {
                self.use_fog = false;
            }
        }

        self.next_vert += 4;
    }

    /// Raven `CQuickSpriteSystem::Flush`.
    ///
    /// Only the CPU-portable slice survives untouched here: the empty-batch
    /// early-out and the final `mNextVert = 0` reclaim. Every other line is
    /// either a fixed-function GL call (DEC-37 A13.2, no R3 home) or gated on
    /// a state carrier no prior wave has landed — see the per-block notes.
    ///
    /// Source: `oracle/codemp/renderer/tr_quicksprite.cpp:52-144`
    pub fn flush(
        &mut self,
        gpu: &mut GpuResources,
        _assets: &RenderAssets,
        _frame: &mut FrameState,
        _common: &Common,
        _cvars: &RendererCvars,
    ) {
        if self.next_vert == 0 {
            return;
        }

        // R_BindAnimatedImage( mTexBundle );
        // DEFERRED: mTexBundle is not a stored field on CQuickSpriteSystem —
        // `StartGroup` (wave 1) deferred storing it under the interior-
        // safety law (see StartGroup's own doc comment above); `Flush`, its
        // sole reader, is the fn that would need the carrier and is out of
        // that wave's scope. No R3 behavior is lost by deferring the call
        // itself: `R_BindAnimatedImage`'s own body
        // (`crates/mp/renderer/src/tr_shade.rs:231-249`) is already DEFERRED
        // R4 for every branch except `bundle.is_video_map`, which this call
        // site could never legitimately reach without a real bundle to
        // inspect.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:80
        GL_State(gpu, self.gl_state_bits);

        // DEFERRED: R4 — qglTexCoordPointer/qglEnableClientState(GL_TEXTURE_
        // COORD_ARRAY)/qglEnableClientState(GL_COLOR_ARRAY)/qglColorPointer/
        // qglVertexPointer (DEC-37 A13.2). Fixed-function GL immediate-mode
        // array setup, no R3 home; the backend is an idiomatic wgpu rewrite.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:86-92

        // DEFERRED: R4 — `if ( qglLockArraysEXT ) { qglLockArraysEXT(0,
        // mNextVert); GLimp_LogComment(...); }` (DEC-37 A13.2, same ruling as
        // this packet's `Flush`/`qglLockArraysEXT` STATE HOMES row: no R3
        // home for the fixed-function-extension pointer check).
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:94-98

        // DEFERRED: R4 — qglDrawArrays(GL_QUADS, 0, mNextVert) (DEC-37
        // A13.2). The main-pass draw call itself.
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:100

        // backEnd.pc.c_vertexes += mNextVert;
        // backEnd.pc.c_indexes += mNextVert;
        // backEnd.pc.c_totalIndexes += mNextVert;
        // DEFERRED: `FrameState::counters` (`BackEndCounters`,
        // `render_state/placeholders.rs`) is still the R3 wave-0 landing
        // placeholder with zero fields — its own module doc lists it among
        // the types "untouched by wave-0, stay empty"; no prior wave has
        // landed `backEndCounters_t`'s `c_vertexes`/`c_indexes`/
        // `c_totalIndexes` onto it, and this file may not extend a struct
        // outside its own module (porting-rules §17 / the wave contract).
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:102-104

        // if (mUseFog && (r_drawfog->integer != 2 || mFogIndex !=
        // tr.world->globalFog)) { ... }
        // DEFERRED: the whole software-fog pass. `tr.world->globalFog` and
        // `tr.world->fogs[mFogIndex]` (`fog_t::colorInt`) are unavailable —
        // `WorldAsset`'s own doc comment states its fog array "land[s] with
        // the rest of the tr_bsp/tr_world waves", not yet ported.
        // Even with those, the pass is GL-only beyond the fog-struct
        // read (`GL_Bind`/`GL_State`/qgl* calls, DEC-37 A13.2), so deferring
        // the block wholesale — rather than reading the fog data just to
        // discard it into deferred GL calls — matches this file's existing
        // convention (`R_DrawElements`/`DrawNormals` in `tr_shade.rs`).
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:106-132

        // DEFERRED: R4 — `if (qglUnlockArraysEXT) { qglUnlockArraysEXT();
        // GLimp_LogComment(...); }` (DEC-37 A13.2, same ruling as this
        // packet's `Flush`/`qglUnlockArraysEXT` STATE HOMES row).
        // Source: oracle/codemp/renderer/tr_quicksprite.cpp:136-141

        self.next_vert = 0;
    }
}
