//! `world_spike` — R4 world wave 3 feasibility spike.
//!
//! Boots the same engine subset and renderer CPU frontend as `ui_harness`,
//! then loads a real BSP with `RE_LoadWorldMap` and drives one `R_RenderView`
//! at a spawn point inside the map. It prints the sorted draw-surface count,
//! the split by `SurfaceGeometry` arm, and the marked-leaf count.
//!
//! The spike proves the world draw-surface chain end to end: the PVS walk
//! (`R_MarkLeaves`/`R_RecursiveWorldNode`) appends through the
//! `SurfaceGeometry::World` arm into the one frontend draw list, and
//! `R_SortDrawSurfs` sorts it. No window and no backend draw are needed — the
//! spike stops at the sorted CPU list.
//!
//! Usage: `cargo run -p mp_renderer_gpu --bin world_spike [-- <basepath> [map]]`.

use mp_renderer_gpu::ui_host::boot;
use mp_renderer_gpu::ui_host::boot::BootConfig;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut cfg = BootConfig::default();
    if let Some(basepath) = args.next() {
        cfg.basepath = basepath;
    }
    let map = args
        .next()
        .unwrap_or_else(|| String::from("maps/mp/duel1.bsp"));

    let mut host = boot::boot(&cfg);
    let r = boot::load_world_and_render(&mut host, &map);

    println!("world_spike: --- report ---");
    println!("world_spike: loaded = {}", r.loaded);
    println!("world_spike: eye = {:?}", r.eye);
    println!("world_spike: total sorted drawSurfs = {}", r.total_draw_surfs);
    println!(
        "world_spike: World = {} (face {}, grid {}, tris {}, flare {}, skip {})",
        r.world, r.world_face, r.world_grid, r.world_triangles, r.world_flare, r.world_skip
    );
    println!(
        "world_spike: Face = {}, Triangles = {}, Poly = {}, Other = {}",
        r.face, r.triangles, r.poly, r.other
    );
    println!("world_spike: visible leaves (approx c_leafs) = {}", r.visible_leaves);

    let ok = r.loaded && r.total_draw_surfs > 0 && r.world > 0;
    println!(
        "world_spike: SUCCESS BAR ({}): loads, drawSurfs > 0, at least one World arm",
        if ok { "MET" } else { "NOT MET" }
    );
}
