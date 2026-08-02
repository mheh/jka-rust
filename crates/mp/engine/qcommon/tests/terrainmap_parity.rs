#![allow(non_snake_case)]

//! Differential parity for the `CDraw32` raster and the `CTerrainMap` automap
//! against the committed goldens `tools/terrainmap-oracle/build.sh` records from
//! the unmodified Raven TUs (`cm_draw.cpp`, `cm_terrainmap.cpp`).
//!
//! The dump format mirrors `tools/terrainmap-oracle/main.cpp` exactly, so each
//! test rebuilds the golden text and compares it whole. Goldens are committed,
//! so this needs no C++ toolchain.
//!
//! Fixtures are synthetic, from `tools/terrainmap-oracle/fixtures/gen_fixtures.py`.
//! No retail game content is involved.
//!
//! Three lines of `golden/terrainmap.txt` record calls the port relocated:
//! the five `R_LoadImage` qpaths and the `R_CreateAutomapImage` arguments now
//! live in `mp_renderer::tr_terrainmap`, which a qcommon test cannot reach
//! (`mp_renderer` depends on this crate). This test states them as literals and
//! names the ported site beside each.

use std::fmt::Write as _;
use std::path::PathBuf;

use mp_engine_qcommon::cm::automap_image::AutomapImage;
use mp_engine_qcommon::cm::cdraw32::CDraw32;
use mp_engine_qcommon::cm::cm_terrainmap_consts::{
    SIDE_BLUE, SIDE_NONE, SIDE_RED, TM_HEIGHT, TM_WIDTH,
};
use mp_engine_qcommon::cm::cpixel32::CPixel32;
use mp_engine_qcommon::cm::cterrainmap::CTerrainMap;
use mp_engine_qcommon::cm::point::POINT;
use mp_engine_qcommon::cm::terrain_map_images::TerrainMapImages;
use mp_engine_qcommon::cm::terrain_map_landscape::TerrainMapLandscape;

/// `main.cpp`'s `DW`.
const DW: i64 = 32;
/// `main.cpp`'s `DH`.
const DH: i64 = 24;

fn rig_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/terrainmap-oracle")
}

fn fixture(name: &str) -> Vec<u8> {
    let path = rig_dir().join("fixtures").join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn golden(name: &str) -> String {
    let path = rig_dir().join("golden").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// `main.cpp`'s `fnv1a`.
fn fnv1a(data: &[u8]) -> u32 {
    let mut h: u32 = 2166136261;
    for b in data {
        h ^= *b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// `main.cpp`'s `resetPattern`.
fn reset_pattern(buf: &mut [CPixel32]) {
    for y in 0..DH {
        for x in 0..DW {
            buf[(y * DW + x) as usize] = CPixel32::new(
                (x as u8).wrapping_mul(5),
                (y as u8).wrapping_mul(7),
                (x ^ y) as u8,
                200,
            );
        }
    }
}

fn to_bytes(buf: &[CPixel32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(buf.len() * 4);
    for p in buf {
        out.extend_from_slice(&[p.r, p.g, p.b, p.a]);
    }
    out
}

/// `main.cpp`'s `dumpDrawBuf`.
fn dump_draw_buf(out: &mut String, label: &str, buf: &[CPixel32]) {
    writeln!(out, "-- {label}").unwrap();
    for y in 0..DH {
        write!(out, "row {y:02} ").unwrap();
        for x in 0..DW {
            let p = buf[(y * DW + x) as usize];
            write!(out, "{:02x}{:02x}{:02x}{:02x}", p.r, p.g, p.b, p.a).unwrap();
        }
        writeln!(out).unwrap();
    }
    writeln!(out, "hash {:08x}", fnv1a(&to_bytes(buf))).unwrap();
}

/// `main.cpp`'s `dumpClip`.
fn dump_clip(out: &mut String, draw: &CDraw32) {
    let (a, b, c, d) = draw.GetClip();
    writeln!(out, "clip {a} {b} {c} {d}").unwrap();
}

/// `main.cpp`'s `dumpBuffer`.
fn dump_buffer(out: &mut String, label: &str, buf: &[u8], width: i32, height: i32) {
    writeln!(out, "-- {label} {width} {height}").unwrap();
    if buf.is_empty() {
        writeln!(out, "empty").unwrap();
        return;
    }
    let row = width as usize * 4;
    for y in 0..height as usize {
        writeln!(
            out,
            "row {y:03} {:08x}",
            fnv1a(&buf[y * row..(y + 1) * row])
        )
        .unwrap();
    }
    writeln!(out, "hash {:08x}", fnv1a(buf)).unwrap();
}

fn rgba_to_pixels(bytes: &[u8]) -> Vec<CPixel32> {
    bytes
        .chunks_exact(4)
        .map(|p| CPixel32::new(p[0], p[1], p[2], p[3]))
        .collect()
}

/// Reproduce `main.cpp`'s `scenarioDraw` (golden `draw.txt`).
#[test]
fn draw_matches_oracle_golden() {
    let mut out = String::new();
    let mut buf = vec![CPixel32::default(); (DW * DH) as usize];

    let mut draw = CDraw32::new();
    draw.SetBufferSize(DW, DH, DW);

    writeln!(out, "== draw {DW}x{DH}").unwrap();
    dump_clip(&mut out, &draw);

    let src = rgba_to_pixels(&fixture("sym_start.rgba"));

    let solid = CPixel32::new(220, 40, 90, 255);
    let half = CPixel32::new(40, 220, 90, 128);
    let clear = CPixel32::new(0, 0, 0, 0);

    reset_pattern(&mut buf);
    draw.ClearBuffer(&mut buf, CPixel32::new(10, 20, 30, 40));
    draw.SetAlphaLines(&mut buf, 77, 4, 9);
    dump_draw_buf(&mut out, "clear", &buf);

    reset_pattern(&mut buf);
    draw.DrawLine(&mut buf, 0, 0, DW - 1, DH - 1, solid);
    draw.DrawLine(&mut buf, DW - 1, 0, 0, DH - 1, solid);
    draw.DrawLine(&mut buf, 2, 5, 29, 5, solid);
    draw.DrawLine(&mut buf, 29, 7, 2, 7, solid);
    draw.DrawLine(&mut buf, 10, 0, 10, DH - 1, solid);
    draw.DrawLine(&mut buf, 12, DH - 1, 12, 0, solid);
    draw.DrawLine(&mut buf, -20, -10, 50, 30, solid);
    draw.DrawLine(&mut buf, -5, -5, -1, -1, solid);
    dump_draw_buf(&mut out, "line_solid", &buf);

    reset_pattern(&mut buf);
    draw.DrawLine(&mut buf, 0, 0, DW - 1, DH - 1, half);
    draw.DrawLine(&mut buf, 2, 5, 29, 5, half);
    draw.DrawLine(&mut buf, 29, 7, 2, 7, half);
    draw.DrawLine(&mut buf, 10, 0, 10, DH - 1, half);
    draw.DrawLine(&mut buf, -20, -10, 50, 30, half);
    dump_draw_buf(&mut out, "line_alpha", &buf);

    reset_pattern(&mut buf);
    draw.DrawLineAve(&mut buf, 2, 3, 29, 3, solid);
    draw.DrawLineAve(&mut buf, 29, 6, 2, 6, solid);
    draw.DrawLineAve(&mut buf, 4, 0, 4, DH - 1, solid);
    draw.DrawLineAve(&mut buf, 6, DH - 1, 6, 0, solid);
    draw.DrawLineAve(&mut buf, 0, 0, DW - 1, DH - 1, half);
    dump_draw_buf(&mut out, "line_ave", &buf);

    reset_pattern(&mut buf);
    draw.DrawLineAA(&mut buf, 1, 1, 30, 22, solid);
    draw.DrawLineAA(&mut buf, 30, 1, 1, 22, half);
    draw.DrawLineAA(&mut buf, 1, 3, 30, 3, solid);
    draw.DrawLineAA(&mut buf, 5, 0, 5, DH - 1, solid);
    draw.DrawLineAA(&mut buf, 0, 0, 22, 22, solid);
    draw.DrawLineAA(&mut buf, -10, 5, 40, 19, solid);
    dump_draw_buf(&mut out, "line_aa", &buf);

    reset_pattern(&mut buf);
    draw.DrawRect(&mut buf, 3, 3, 10, 6, solid);
    draw.DrawRect(&mut buf, 25, 18, 12, 10, half);
    draw.DrawRectNC(&mut buf, 1, 15, 8, 4, solid);
    draw.DrawRectAve(&mut buf, 14, 10, 9, 7, solid);
    draw.DrawRect(&mut buf, 4, 4, 0, 5, solid);
    dump_draw_buf(&mut out, "rect", &buf);

    reset_pattern(&mut buf);
    draw.DrawBox(&mut buf, 2, 2, 12, 9, solid);
    draw.DrawBox(&mut buf, -3, -3, 10, 10, half);
    draw.DrawBoxNC(&mut buf, 20, 14, 10, 8, solid);
    draw.DrawBoxAve(&mut buf, 6, 12, 14, 10, solid);
    dump_draw_buf(&mut out, "box", &buf);

    reset_pattern(&mut buf);
    draw.DrawCircle(&mut buf, 16, 12, 7, solid, half);
    draw.DrawCircle(&mut buf, 4, 4, 3, solid, clear);
    draw.DrawCircle(&mut buf, 28, 20, 5, clear, solid);
    draw.DrawCircle(&mut buf, 16, 12, 1, solid, half);
    draw.DrawCircle(&mut buf, 16, 12, 0, solid, half);
    draw.DrawCircle(&mut buf, 2, 20, 9, solid, half);
    dump_draw_buf(&mut out, "circle", &buf);

    reset_pattern(&mut buf);
    draw.DrawCircleAve(&mut buf, 16, 12, 9, solid, half);
    draw.DrawCircleAve(&mut buf, 2, 2, 4, solid, half);
    draw.DrawCircleAve(&mut buf, 30, 22, 6, clear, solid);
    dump_draw_buf(&mut out, "circle_ave", &buf);

    reset_pattern(&mut buf);
    let tri = [pt(4, 2), pt(28, 8), pt(10, 21)];
    draw.DrawPolygon(&mut buf, 3, &tri, solid, half);
    dump_draw_buf(&mut out, "poly_tri", &buf);

    reset_pattern(&mut buf);
    let quad = [pt(2, 2), pt(29, 4), pt(14, 12), pt(27, 21)];
    draw.DrawPolygon(&mut buf, 4, &quad, solid, half);
    dump_draw_buf(&mut out, "poly_concave", &buf);

    reset_pattern(&mut buf);
    let arrow = [pt(16, 12), pt(11, 7), pt(26, 12), pt(11, 17)];
    draw.DrawPolygon(
        &mut buf,
        4,
        &arrow,
        CPixel32::new(0, 0, 0, 128),
        CPixel32::new(0, 0, 0, 128),
    );
    let arrow2 = [pt(15, 11), pt(10, 6), pt(25, 11), pt(10, 16)];
    draw.DrawPolygon(
        &mut buf,
        4,
        &arrow2,
        CPixel32::new(255, 255, 255, 255),
        CPixel32::new(255, 255, 255, 255),
    );
    dump_draw_buf(&mut out, "poly_arrow", &buf);

    reset_pattern(&mut buf);
    let flat = [pt(3, 9), pt(20, 9), pt(12, 9)];
    draw.DrawPolygon(&mut buf, 3, &flat, solid, half);
    draw.DrawPolygon(&mut buf, 0, &flat, solid, half);
    let away = [pt(-40, -40), pt(-30, -35), pt(-38, -28)];
    draw.DrawPolygon(&mut buf, 3, &away, solid, half);
    dump_draw_buf(&mut out, "poly_degenerate", &buf);

    reset_pattern(&mut buf);
    draw.Blit(&mut buf, 2, 2, 16, 16, &src, 0, 0, 16);
    draw.Blit(&mut buf, 20, 16, 16, 16, &src, 0, 0, 16);
    draw.Blit(&mut buf, -4, -3, 16, 16, &src, 0, 0, 16);
    dump_draw_buf(&mut out, "blit", &buf);

    reset_pattern(&mut buf);
    draw.BlitColor(&mut buf, 2, 2, 16, 16, &src, 0, 0, 16, solid);
    draw.BlitColor(&mut buf, 20, 16, 16, 16, &src, 0, 0, 16, half);
    draw.BlitColor(&mut buf, -4, -3, 16, 16, &src, 0, 0, 16, solid);
    dump_draw_buf(&mut out, "blit_color", &buf);

    reset_pattern(&mut buf);
    draw.Emboss(&mut buf, 4, 4, 16, 16, &src, 0, 0, 16);
    dump_draw_buf(&mut out, "emboss", &buf);

    reset_pattern(&mut buf);
    draw.SetClip(5, 4, 20, 15);
    dump_clip(&mut out, &draw);
    draw.DrawLine(&mut buf, 0, 0, DW - 1, DH - 1, solid);
    draw.DrawLine(&mut buf, DW - 1, 0, 0, DH - 1, solid);
    draw.DrawLine(&mut buf, 0, 9, DW - 1, 9, solid);
    draw.DrawRect(&mut buf, 0, 0, DW, DH, half);
    draw.DrawCircle(&mut buf, 12, 9, 8, solid, half);
    draw.DrawPolygon(&mut buf, 4, &quad, solid, half);
    draw.Blit(&mut buf, 0, 0, 16, 16, &src, 0, 0, 16);
    dump_draw_buf(&mut out, "clipped", &buf);

    draw.SetClip(0, 0, DW - 1, DH - 1);
    dump_clip(&mut out, &draw);

    assert_eq!(out, golden("draw.txt"));
}

fn pt(x: i64, y: i64) -> POINT {
    POINT { x, y }
}

/// Reproduce `main.cpp`'s `scenarioTerrainMap` (golden `terrainmap.txt`).
#[test]
fn terrainmap_matches_oracle_golden() {
    let mut out = String::new();

    let mut height = fixture("heightmap.bin");
    // one trailing pad byte: see the note in stubs/qcommon/cm_landscape.h
    height.push(0);

    let landscape = TerrainMapLandscape {
        height_map: &height,
        real_width: 65,
        real_height: 65,
        base_water_height: 40,
        mins: [-2048.0, -2048.0, -512.0],
        size: [4096.0, 4096.0, 1024.0],
    };

    let images = TerrainMapImages {
        background: AutomapImage::from_rgba(&fixture("bg.rgba"), 64, 64),
        start: AutomapImage::from_rgba(&fixture("sym_start.rgba"), 16, 16),
        end: AutomapImage::from_rgba(&fixture("sym_end.rgba"), 16, 16),
        objective: AutomapImage::from_rgba(&fixture("sym_objective.rgba"), 16, 16),
        building: AutomapImage::from_rgba(&fixture("sym_bld.rgba"), 16, 16),
    };

    writeln!(out, "== terrainmap {TM_WIDTH} {TM_HEIGHT}").unwrap();

    // The five loads are `mp_renderer::tr_terrainmap::R_LoadTerrainMapImages`,
    // in its field order, with Raven's qpaths.
    for name in [
        "gfx\\menus\\rmg\\01_bg",
        "gfx/menus/rmg/start",
        "gfx/menus/rmg/end",
        "gfx/menus/rmg/objective",
        "gfx/menus/rmg/building",
    ] {
        writeln!(out, "call R_LoadImage {name}").unwrap();
    }

    let mut map = CTerrainMap::new(landscape, images);

    map.SaveImageToDisk("t0", "m0", "s0");
    writeln!(
        out,
        "call PNG_Save {} {TM_WIDTH} {TM_HEIGHT} 4",
        CTerrainMap::SaveImagePath("t0", "m0", "s0")
    )
    .unwrap();
    dump_buffer(
        &mut out,
        "image_after_ctor",
        &map.image_rgba(),
        TM_WIDTH,
        TM_HEIGHT,
    );

    map.AddBuilding(-1000, 500, SIDE_BLUE);
    map.AddBuilding(1200, -800, SIDE_RED);
    map.AddBuilding(-3000, 3000, SIDE_NONE);
    map.AddStart(-1800, -1800, SIDE_BLUE);
    map.AddEnd(1800, 1800, SIDE_RED);
    map.AddObjective(0, 0, SIDE_NONE);
    map.AddNPC(300, -300, true);
    map.AddNPC(-300, 300, false);
    map.AddNode(700, 700);
    map.AddNode(-2040, -2040);
    map.AddWallRect(-500, -500, SIDE_BLUE);
    map.AddWallRect(-460, -500, SIDE_RED);
    map.AddWallRect(-420, -500, SIDE_NONE);

    map.SaveImageToDisk("t1", "m1", "s1");
    writeln!(
        out,
        "call PNG_Save {} {TM_WIDTH} {TM_HEIGHT} 4",
        CTerrainMap::SaveImagePath("t1", "m1", "s1")
    )
    .unwrap();
    dump_buffer(
        &mut out,
        "image_after_symbols",
        &map.image_rgba(),
        TM_WIDTH,
        TM_HEIGHT,
    );

    // The `R_CreateAutomapImage` arguments are
    // `mp_renderer::tr_terrainmap::R_UploadTerrainAutomap`'s literal list.
    let angles = [0.0, 37.5, 0.0];
    let pic = map.Upload(Some([100.0, -200.0, 64.0]), angles);
    writeln!(
        out,
        "call R_CreateAutomapImage *automap {TM_WIDTH} {TM_HEIGHT} 0 0 1 0"
    )
    .unwrap();
    dump_buffer(&mut out, "upload_with_player", &pic, TM_WIDTH, TM_HEIGHT);

    let pic = map.Upload(None, angles);
    writeln!(
        out,
        "call R_CreateAutomapImage *automap {TM_WIDTH} {TM_HEIGHT} 0 0 1 0"
    )
    .unwrap();
    dump_buffer(&mut out, "upload_no_player", &pic, TM_WIDTH, TM_HEIGHT);

    for (cx, cy) in [
        (0, 0),
        (-2048, -2048),
        (2047, 2047),
        (-1000, 500),
        (1200, -800),
        (700, 700),
        (-3000, 3000),
        (123, -456),
    ] {
        let (mut x, mut y) = (cx, cy);
        map.ConvertPos(&mut x, &mut y);
        x = x * TM_WIDTH / TM_WIDTH;
        y = y * TM_HEIGHT / TM_HEIGHT;
        writeln!(out, "convert {cx} {cy} -> {x} {y}").unwrap();
    }

    assert_eq!(out, golden("terrainmap.txt"));
}
