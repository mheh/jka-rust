#![allow(non_camel_case_types, non_snake_case)]

//! `CTerrainMap` — the RMG automap image.
//!
//! Two deliberate divergences from Raven, both forced by the crate graph and by
//! porting-rules §B3 (no globals):
//!
//! - Raven's constructor calls the renderer's `R_LoadImage` five times.
//!   `mp_engine_qcommon` sits below `mp_renderer`, so the caller loads them and
//!   passes a [`TerrainMapImages`].
//! - Raven's `Upload` calls the renderer's `R_CreateAutomapImage`. The port
//!   returns the finished RGBA raster and the renderer-side caller uploads it.
//!
//! Raven keeps `CCMLandScape *mLandscape` for the whole object lifetime, but
//! only `ConvertPos` reads it after the constructor returns. The port copies the
//! two vectors `ConvertPos` needs, so the automap holds no back pointer into the
//! collision world that owns it.
//!
//! Source: `oracle/codemp/qcommon/cm_terrainmap.cpp`

use core::ffi::c_int;
use core::ffi::c_long;

use native_math::qmath::RotatePointAroundVector;
use native_math::vector::vec3_t;

use crate::cm::automap_image::AutomapImage;
use crate::cm::cdraw32::CDraw32;
use crate::cm::cm_terrainmap_consts::{
    SIDE_BLUE, SIDE_RED, TM_BORDER, TM_HEIGHT, TM_REAL_HEIGHT, TM_REAL_WIDTH, TM_WIDTH,
};
use crate::cm::cpixel32::{CPixel32, ALPHA_PIX};
use crate::cm::point::POINT;
use crate::cm::terrain_map_images::TerrainMapImages;
use crate::cm_terrain::CmLandScape;
use crate::cm_terrainmap::SideColor;

/// The pixel count of one automap buffer.
const TM_PIXELS: usize = (TM_WIDTH * TM_HEIGHT) as usize;

/// Raven `CTerrainMap` — the automap image for the current landscape.
///
/// Type definition source: `oracle/codemp/qcommon/cm_terrainmap.h:17-60`
pub struct CTerrainMap {
    /// image to output
    mImage: Vec<CPixel32>,
    /// src data for image, color and bump
    mBufImage: Vec<CPixel32>,

    mSymBld: AutomapImage,
    mSymStart: AutomapImage,
    mSymEnd: AutomapImage,
    mSymObjective: AutomapImage,

    /// `mLandscape->GetMins()`, the only landscape read past the constructor.
    mins: vec3_t,
    /// `mLandscape->GetSize()`, the only landscape read past the constructor.
    size: vec3_t,

    /// Raven's `CDraw32` class statics, which survive between method calls.
    draw: CDraw32,
}

impl CTerrainMap {
    /// Raven `CTerrainMap::CTerrainMap` builds the automap image from the
    /// landscape: background, heightmap shading, then the land and water blend.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:46-97`
    pub fn new(landscape: &CmLandScape, images: TerrainMapImages) -> Self {
        let mut map = CTerrainMap {
            mImage: vec![CPixel32::default(); TM_PIXELS],
            // Raven never initializes `mBufImage`'s alpha outside the heightmap
            // rectangle. The port zeroes the whole buffer; the blend below skips
            // that border, so no uninitialized byte reaches the output
            // (porting-rules §F19).
            mBufImage: vec![CPixel32::default(); TM_PIXELS],
            mSymBld: images.building,
            mSymStart: images.start,
            mSymEnd: images.end,
            mSymObjective: images.objective,
            mins: landscape.mins(),
            size: landscape.size(),
            draw: CDraw32::new(),
        };

        map.ApplyBackground(&images.background);
        map.ApplyHeightmap(landscape);

        map.draw.SetBufferSize(
            TM_WIDTH as c_long,
            TM_HEIGHT as c_long,
            TM_WIDTH as c_long,
        );

        // create version with paths and water shown
        for y in 0..TM_HEIGHT {
            for x in 0..TM_WIDTH {
                let mut cp = map.mBufImage[(y * TM_WIDTH + x) as usize];
                let land = CLAMP_BYTE(((255 - cp.a as c_int) * 2) / 3);
                let water = CLAMP_BYTE((landscape.base_water_height() - cp.a as c_int) * 4);
                cp.a = 255;

                if x > TM_BORDER
                    && x < (TM_WIDTH - TM_BORDER)
                    && y > TM_BORDER
                    && y < (TM_WIDTH - TM_BORDER)
                {
                    cp = ALPHA_PIX(
                        CPixel32::new(0, 0, 0, 255),
                        cp,
                        land as c_long,
                        256 - land as c_long,
                    );
                    if water > 0 {
                        cp = ALPHA_PIX(
                            CPixel32::new(0, 0, 255, 255),
                            cp,
                            water as c_long,
                            256 - water as c_long,
                        );
                    }
                }

                map.draw
                    .PutPix(&mut map.mImage, x as c_long, y as c_long, cp);
            }
        }

        map
    }

    /// Raven `CTerrainMap::ApplyBackground` fills `mImage` white, then scales
    /// the background tile into `mBufImage`'s color channels.
    ///
    /// The alpha channel is deliberately skipped (`outPos += 2`);
    /// `ApplyHeightmap` writes it next.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:128-173`
    fn ApplyBackground(&mut self, background: &AutomapImage) {
        self.mImage.fill(CPixel32::new(255, 255, 255, 255));

        if !background.is_loaded() {
            return;
        }

        let backgroundWidth = background.width;
        let backgroundHeight = background.height;
        let xInc = backgroundWidth as f32 / TM_WIDTH as f32;
        let yInc = backgroundHeight as f32 / TM_HEIGHT as f32;

        let mut outPos: usize = 0;
        let mut yRel: f32 = 0.0;
        for _y in 0..TM_HEIGHT {
            let mut xRel: f32 = 0.0;
            for _x in 0..TM_WIDTH {
                let pos = ((yRel as c_int) * backgroundWidth) + (xRel as c_int);
                let src = background.pixels[pos as usize];
                self.mBufImage[outPos].r = src.r;
                self.mBufImage[outPos].g = src.g;
                self.mBufImage[outPos].b = src.b;
                outPos += 1;
                xRel += xInc;
            }
            yRel += yInc;
        }
    }

    /// Raven `CTerrainMap::ApplyHeightmap` writes the five-tap heightmap
    /// average into `mBufImage`'s alpha channel, inside the border and with the
    /// x axis flipped.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:175-233`
    fn ApplyHeightmap(&mut self, landscape: &CmLandScape) {
        let inPos = landscape.height_map();
        let width = landscape.real_width();
        let height = landscape.real_height();

        let mut outPos = ((TM_BORDER * TM_WIDTH) + TM_BORDER) as usize;
        let xInc = width as f32 / TM_REAL_WIDTH as f32;
        let yInc = height as f32 / TM_REAL_HEIGHT as f32;

        // add in height map as alpha
        let mut yRel: f32 = 0.0;
        for _y in 0..TM_REAL_HEIGHT {
            // x is flipped!
            let mut xRel: f32 = width as f32;
            for _x in 0..TM_REAL_WIDTH {
                let mut count: u32 = 1;
                let mut tempColor = height_at(inPos, ((yRel as c_int) * width) + (xRel as c_int));
                if yRel >= 1.0 {
                    tempColor +=
                        height_at(inPos, (((yRel - 0.5) as c_int) * width) + (xRel as c_int));
                    count += 1;
                }
                if yRel <= (height - 2) as f32 {
                    tempColor +=
                        height_at(inPos, (((yRel + 0.5) as c_int) * width) + (xRel as c_int));
                    count += 1;
                }
                if xRel >= 1.0 {
                    tempColor +=
                        height_at(inPos, ((yRel as c_int) * width) + ((xRel - 0.5) as c_int));
                    count += 1;
                }
                if xRel <= (width - 2) as f32 {
                    tempColor +=
                        height_at(inPos, ((yRel as c_int) * width) + ((xRel + 0.5) as c_int));
                    count += 1;
                }
                tempColor /= count;

                self.mBufImage[outPos].a = tempColor as u8;
                outPos += 1;

                // x is flipped!
                xRel -= xInc;
            }
            outPos += (TM_BORDER * 2) as usize;

            yRel += yInc;
        }
    }

    /// Raven `CTerrainMap::ConvertPos` maps a world position into automap pixel
    /// space, flipping x and adding the border.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:236-247`
    pub fn ConvertPos(&self, x: &mut c_int, y: &mut c_int) {
        *x = (((*x as f32 - self.mins[0]) / self.size[0]) * TM_REAL_WIDTH as f32) as c_int;
        *y = (((*y as f32 - self.mins[1]) / self.size[1]) * TM_REAL_HEIGHT as f32) as c_int;

        // x is flipped!
        *x = TM_REAL_WIDTH - *x - 1;

        // border
        *x += TM_BORDER;
        *y += TM_BORDER;
    }

    /// Raven `CTerrainMap::AddStart` draws the start symbol tinted by side.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:249-256`
    pub fn AddStart(&mut self, x: c_int, y: c_int, side: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        let CTerrainMap {
            mImage,
            mSymStart: sym,
            draw,
            ..
        } = self;
        draw.BlitColor(
            mImage,
            (x - sym.width / 2) as c_long,
            (y - sym.height / 2) as c_long,
            sym.width as c_long,
            sym.height as c_long,
            &sym.pixels,
            0,
            0,
            sym.width as c_long,
            SideColor(side),
        );
    }

    /// Raven `CTerrainMap::AddEnd` draws the end symbol tinted by side.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:258-265`
    pub fn AddEnd(&mut self, x: c_int, y: c_int, side: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        let CTerrainMap {
            mImage,
            mSymEnd: sym,
            draw,
            ..
        } = self;
        draw.BlitColor(
            mImage,
            (x - sym.width / 2) as c_long,
            (y - sym.height / 2) as c_long,
            sym.width as c_long,
            sym.height as c_long,
            &sym.pixels,
            0,
            0,
            sym.width as c_long,
            SideColor(side),
        );
    }

    /// Raven `CTerrainMap::AddObjective` draws the objective symbol tinted by
    /// side.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:267-274`
    pub fn AddObjective(&mut self, x: c_int, y: c_int, side: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        let CTerrainMap {
            mImage,
            mSymObjective: sym,
            draw,
            ..
        } = self;
        draw.BlitColor(
            mImage,
            (x - sym.width / 2) as c_long,
            (y - sym.height / 2) as c_long,
            sym.width as c_long,
            sym.height as c_long,
            &sym.pixels,
            0,
            0,
            sym.width as c_long,
            SideColor(side),
        );
    }

    /// Raven `CTerrainMap::AddBuilding` draws the building symbol tinted by
    /// side.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:276-283`
    pub fn AddBuilding(&mut self, x: c_int, y: c_int, side: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        let CTerrainMap {
            mImage,
            mSymBld: sym,
            draw,
            ..
        } = self;
        draw.BlitColor(
            mImage,
            (x - sym.width / 2) as c_long,
            (y - sym.height / 2) as c_long,
            sym.width as c_long,
            sym.height as c_long,
            &sym.pixels,
            0,
            0,
            sym.width as c_long,
            SideColor(side),
        );
    }

    /// Raven `CTerrainMap::AddNPC` draws a green or red ring for one NPC.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:285-294`
    pub fn AddNPC(&mut self, x: c_int, y: c_int, friendly: bool) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        let edge = if friendly {
            CPixel32::new(0, 192, 0, 255)
        } else {
            CPixel32::new(192, 0, 0, 255)
        };
        self.draw.DrawCircle(
            &mut self.mImage,
            x as c_long,
            y as c_long,
            3,
            edge,
            CPixel32::new(0, 0, 0, 0),
        );
    }

    /// Raven `CTerrainMap::AddNode` draws the white ring of a nav node.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:296-302`
    pub fn AddNode(&mut self, x: c_int, y: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        self.draw.DrawCircle(
            &mut self.mImage,
            x as c_long,
            y as c_long,
            20,
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(0, 0, 0, 0),
        );
    }

    /// Raven `CTerrainMap::AddWallRect` draws a 3-by-3 box for one wall cell.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:304-321`
    pub fn AddWallRect(&mut self, x: c_int, y: c_int, side: c_int) {
        let (mut x, mut y) = (x, y);
        self.ConvertPos(&mut x, &mut y);

        // Raven's own color table here, half-alpha, not `SideColor`.
        let color = match side {
            SIDE_BLUE => CPixel32::new(0, 0, 192, 128),
            SIDE_RED => CPixel32::new(192, 0, 0, 128),
            _ => CPixel32::new(192, 192, 192, 128),
        };
        self.draw.DrawBox(
            &mut self.mImage,
            (x - 1) as c_long,
            (y - 1) as c_long,
            3,
            3,
            color,
        );
    }

    /// Raven `CTerrainMap::AddPlayer` draws the player arrowhead and its shadow.
    ///
    /// Raven calls this only from `Upload`, where the drawing target is
    /// `mBufImage`; the port names that target directly.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:323-363`
    pub fn AddPlayer(&mut self, origin: vec3_t, angles: vec3_t) {
        let pt: [vec3_t; 4] = [
            [0.0, 0.0, 0.0],
            [-5.0, -5.0, 0.0],
            [10.0, 0.0, 0.0],
            [-5.0, 5.0, 0.0],
        ];
        let mut poly = [POINT::default(); 4];

        let facing = angles[1];

        let up: vec3_t = [0.0, 0.0, 1.0];

        let mut x = origin[0] as c_int;
        let mut y = origin[1] as c_int;
        self.ConvertPos(&mut x, &mut y);
        x += 1;
        y += 1;

        for i in 0..4 {
            let mut p: vec3_t = [0.0; 3];
            RotatePointAroundVector(&mut p, up, pt[i], facing);
            poly[i].x = (-p[0] + x as f32) as c_long;
            poly[i].y = (p[1] + y as f32) as c_long;
        }

        // draw arrowhead shadow
        self.draw.DrawPolygon(
            &mut self.mBufImage,
            4,
            &poly,
            CPixel32::new(0, 0, 0, 128),
            CPixel32::new(0, 0, 0, 128),
        );

        // draw arrowhead
        for p in poly.iter_mut() {
            p.x -= 1;
            p.y -= 1;
        }
        self.draw.DrawPolygon(
            &mut self.mBufImage,
            4,
            &poly,
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(255, 255, 255, 255),
        );
    }

    /// Raven `CTerrainMap::Upload` composes the finished automap: `mImage` over
    /// `mBufImage`, the player arrowhead, then a fully opaque alpha channel.
    ///
    /// Raven then hands `mBufImage` to `R_CreateAutomapImage` as `"*automap"`.
    /// This port returns the RGBA bytes instead, and `mp_renderer`'s
    /// `R_UploadTerrainAutomap` makes that call.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:365-387`
    pub fn Upload(&mut self, player_origin: Option<vec3_t>, player_angles: vec3_t) -> Vec<u8> {
        // copy completed map to mBufImage
        self.draw.SetBufferSize(
            TM_WIDTH as c_long,
            TM_HEIGHT as c_long,
            TM_WIDTH as c_long,
        );

        {
            let CTerrainMap {
                mImage,
                mBufImage,
                draw,
                ..
            } = self;
            draw.Blit(
                mBufImage,
                0,
                0,
                TM_WIDTH as c_long,
                TM_HEIGHT as c_long,
                mImage,
                0,
                0,
                TM_WIDTH as c_long,
            );
        }

        // now draw player's location on map
        if let Some(origin) = player_origin {
            self.AddPlayer(origin, player_angles);
        }

        self.draw.SetAlphaBuffer(&mut self.mBufImage, 255);

        to_rgba(&self.mBufImage)
    }

    /// Raven `CTerrainMap::SaveImageToDisk` writes the automap image as a PNG
    /// under `save/`.
    ///
    /// Source: `oracle/codemp/qcommon/cm_terrainmap.cpp:389-395`
    pub fn SaveImageToDisk(&self, terrainName: &str, missionName: &str, seed: &str) {
        let _name = format!("save/{terrainName}_{missionName}_{seed}.png");
        //TODO: Port PNG_Save
        // Source: oracle/codemp/png/png.cpp:582-645
        // Deliberate no-op: the encoder is a whole unported TU (`codemp/png/`)
        // outside this file's port, and the only Raven caller is the
        // `rmg_saveautomap` debug arm.
    }

    /// The finished automap raster, as the renderer wants it.
    pub fn image_rgba(&self) -> Vec<u8> {
        to_rgba(&self.mImage)
    }
}

/// Raven `CLAMP(v, 0, 255)` on an `int` expression.
///
/// Source: `oracle/codemp/qcommon/cm_draw.h:29`
fn CLAMP_BYTE(v: c_int) -> c_int {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v
    }
}

/// One heightmap tap.
///
/// `ApplyHeightmap` starts `xRel` at `width`, so Raven's index runs into the
/// next row and, on the last row, one byte past the buffer. The port reads `0`
/// past the end and keeps the in-buffer row wrap exactly (porting-rules §F19).
fn height_at(map: &[u8], index: c_int) -> u32 {
    map.get(index as usize).copied().unwrap_or(0) as u32
}

/// Flatten a pixel buffer to the renderer's RGBA byte order.
fn to_rgba(pixels: &[CPixel32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for p in pixels {
        out.push(p.r);
        out.push(p.g);
        out.push(p.b);
        out.push(p.a);
    }
    out
}
