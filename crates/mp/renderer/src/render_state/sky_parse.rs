//! `SkyParse` — the sky tables `ParseSkyParms` precomputes once, published
//! with the rest of the registry (W2-F3 sky split).

/// The two `tr_sky.cpp` tables `R_InitSkyTexCoords` fills at shader-parse time
/// and the world pass only reads.
///
/// The rest of `tr_sky.cpp`'s file-scope state is per-view scratch and lives on
/// `SkyState`, which the render thread owns. `sky_min`, `sky_max` and
/// `sky_clip` stay there too, against the letter of the ruling's parenthetical:
/// they read like parse-time constants but every path rewrites them before use.
/// `DrawSkyBox` sets `0`/`1`, `R_BuildCloudData` sets `1/256`/`255/256`, and
/// `RB_ClipSkyPolygons` copies `SKY_CLIP` in. Publishing them would put a
/// per-view write on the immutable side for no gain (user ruling 2026-08-03,
/// W2-F3).
///
/// `#[derive(Clone)]` — required by `Arc::make_mut` on the published
/// `RenderAssets`.
///
/// Source: `oracle/codemp/renderer/tr_sky.cpp:39-40` (the two statics),
/// `oracle/codemp/renderer/tr_sky.cpp:626-680` (`R_InitSkyTexCoords`)
#[derive(Clone, Default)]
pub struct SkyParse {
    /// Raven `s_cloudTexCoords[6][SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` —
    /// per-face cloud-layer texture coordinates, consumed by `FillCloudBox`.
    /// Empty until the first sky shader parses.
    pub cloud_tex_coords: Vec<Vec<Vec<[f32; 2]>>>,
    /// Raven `s_cloudTexP[6][SKY_SUBDIVISIONS+1][SKY_SUBDIVISIONS+1]` — the
    /// per-vertex cloud-layer intersection parameter.
    pub cloud_tex_p: Vec<Vec<Vec<f32>>>,
}
