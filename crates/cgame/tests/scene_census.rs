//! Scene-trap census over recorded cgame traces (wayfinder #17, DEC-54).
//!
//! This walks the C6b journals directly - no module drive - and counts every
//! renderer-facing submission: refEntity types, polys, decals, dynamic lights,
//! 2D calls, FX primitives, and the shaders and effects each touches. Shader
//! and effect handles resolve to names through the registration traps the same
//! journal records. The output list is the live-play renderer gate and the
//! wave-plan seed (DEC-54).
//!
//! Run: `JKA_TRACES=<a.bin:b.bin:...> cargo test -p cgame --release --test
//! scene_census -- --ignored --nocapture`. Without `JKA_TRACES` the census
//! reads the four local traces under `$HOME/Developer/jka` (DEC-48.4: traces
//! stay out of git). `JKA_CENSUS_OUT=<path>` also writes the report to a file.

#![allow(non_snake_case)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

mod replay_support;
use replay_support::{Reader, Rec, REC_SYSCALL_ENTER, REC_SYSCALL_EXIT};

// Trap numbers, from tools/cgame-referee/trap-shapes.json.
const R_REGISTERMODEL: i64 = 54;
const R_REGISTERSKIN: i64 = 55;
const R_REGISTERSHADER: i64 = 56;
const R_REGISTERSHADERNOMIP: i64 = 57;
const R_REGISTERFONT: i64 = 58;
const R_FONT_DRAWSTRING: i64 = 62;
const CM_MARKFRAGMENTS: i64 = 34;
const R_CLEARSCENE: i64 = 200;
const R_ADDREFENTITYTOSCENE: i64 = 202;
const R_ADDPOLYTOSCENE: i64 = 203;
const R_ADDPOLYSTOSCENE: i64 = 204;
const R_ADDDECALTOSCENE: i64 = 205;
const R_ADDLIGHTTOSCENE: i64 = 207;
const R_ADDADDITIVELIGHTTOSCENE: i64 = 208;
const R_RENDERSCENE: i64 = 209;
const R_SETCOLOR: i64 = 210;
const R_DRAWSTRETCHPIC: i64 = 211;
const R_DRAWROTATEPIC: i64 = 214;
const R_DRAWROTATEPIC2: i64 = 215;
const FX_ADDLINE: i64 = 226;
const FX_REGISTER_EFFECT: i64 = 263;
const FX_PLAY_EFFECT: i64 = 264;
const FX_PLAY_ENTITY_EFFECT: i64 = 265;
const FX_PLAY_EFFECT_ID: i64 = 266;
const FX_PLAY_PORTAL_EFFECT_ID: i64 = 267;
const FX_PLAY_ENTITY_EFFECT_ID: i64 = 268;
const FX_PLAY_BOLTED_EFFECT_ID: i64 = 269;
const FX_ADDPOLY: i64 = 277;
const FX_ADDBEZIER: i64 = 278;
const FX_ADDPRIMITIVE: i64 = 279;
const FX_ADDSPRITE: i64 = 280;
const FX_ADDELECTRICITY: i64 = 281;

/// `refEntityType_t` names by value.
/// Source: `crates/mp/qshared/src/common/mp/cgame/ref_entity_type_t.rs`
const RETYPE_NAMES: [&str; 12] = [
    "RT_MODEL",
    "RT_POLY",
    "RT_SPRITE",
    "RT_ORIENTED_QUAD",
    "RT_BEAM",
    "RT_SABER_GLOW",
    "RT_ELECTRICITY",
    "RT_PORTALSURFACE",
    "RT_LINE",
    "RT_ORIENTEDLINE",
    "RT_CYLINDER",
    "RT_ENT_CHAIN",
];

/// `RF_*` renderfx flag names by bit.
/// Source: `crates/mp/qshared` RF_ consts (`q_shared.h`).
const RF_NAMES: [(u32, &str); 22] = [
    (0x00001, "RF_MINLIGHT"),
    (0x00002, "RF_THIRD_PERSON"),
    (0x00004, "RF_FIRST_PERSON"),
    (0x00008, "RF_DEPTHHACK"),
    (0x00010, "RF_NODEPTH"),
    (0x00020, "RF_VOLUMETRIC"),
    (0x00040, "RF_NOSHADOW"),
    (0x00080, "RF_LIGHTING_ORIGIN"),
    (0x00100, "RF_SHADOW_PLANE"),
    (0x00200, "RF_WRAP_FRAMES"),
    (0x00400, "RF_FORCE_ENT_ALPHA"),
    (0x00800, "RF_RGB_TINT"),
    (0x01000, "RF_SHADOW_ONLY"),
    (0x02000, "RF_DISTORTION"),
    (0x04000, "RF_FORKED"),
    (0x08000, "RF_TAPERED"),
    (0x10000, "RF_GROW"),
    (0x20000, "RF_DISINTEGRATE1"),
    (0x40000, "RF_DISINTEGRATE2"),
    (0x80000, "RF_SETANIMINDEX"),
    (0x100000, "RF_ALPHA_DEPTH"),
    (0x200000, "RF_FORCEPERS"),
];

fn i32_at(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

/// One census accumulator, used per trace and for the aggregate.
#[derive(Default)]
struct Census {
    /// Raw counts keyed by a stable label.
    counts: BTreeMap<String, u64>,
    /// Distinct shader names per submission surface.
    shaders: BTreeMap<&'static str, BTreeMap<String, u64>>,
    /// Effects played, by effect name.
    effects: BTreeMap<String, u64>,
    /// Models and skins registered.
    models: BTreeSet<String>,
    skins: BTreeSet<String>,
    fonts: BTreeSet<String>,
}

impl Census {
    fn bump(&mut self, key: &str) {
        *self.counts.entry(key.to_string()).or_insert(0) += 1;
    }
    fn shader(&mut self, surface: &'static str, name: String) {
        *self
            .shaders
            .entry(surface)
            .or_default()
            .entry(name)
            .or_insert(0) += 1;
    }
}

/// Handle-to-name maps built from the registration traps.
#[derive(Default)]
struct NameMaps {
    shaders: BTreeMap<i64, String>,
    effects: BTreeMap<i64, String>,
}

impl NameMaps {
    fn shader(&self, handle: i64) -> String {
        if handle == 0 {
            return String::from("<none>");
        }
        self.shaders
            .get(&(handle & 0xFFFF_FFFF))
            .cloned()
            .unwrap_or_else(|| format!("shader#{}", handle & 0xFFFF_FFFF))
    }
    fn effect(&self, id: i64) -> String {
        self.effects
            .get(&(id & 0xFFFF_FFFF))
            .cloned()
            .unwrap_or_else(|| format!("effect#{}", id & 0xFFFF_FFFF))
    }
}

/// The in_str blob of an enter record, as latin-1 text.
fn str_arg(rec: &Rec, idx: u8) -> Option<String> {
    rec.blobs
        .iter()
        .find(|b| b.arg_index == idx && b.kind == 1)
        .map(|b| b.bytes.iter().map(|&c| c as char).collect())
}

fn buf_arg<'a>(rec: &'a Rec, idx: u8) -> Option<&'a [u8]> {
    rec.blobs
        .iter()
        .find(|b| b.arg_index == idx && b.kind == 2)
        .map(|b| b.bytes.as_slice())
}

/// Walks one trace and feeds both the per-trace and the aggregate census.
fn walk(path: &PathBuf, maps: &mut NameMaps, out: &mut [&mut Census]) {
    let mut reader = Reader::open(path).expect("open trace");
    // The registration name waits on its exit record for the handle.
    let mut pending: Option<(i64, String)> = None;

    while let Some(rec) = reader.take() {
        match rec.rec_type {
            REC_SYSCALL_ENTER => {
                let num = rec.cmd;
                match num {
                    R_REGISTERSHADER | R_REGISTERSHADERNOMIP | FX_REGISTER_EFFECT
                    | R_REGISTERMODEL | R_REGISTERSKIN | R_REGISTERFONT => {
                        if let Some(name) = str_arg(&rec, 0) {
                            pending = Some((num, name));
                        }
                    }
                    R_ADDREFENTITYTOSCENE => {
                        if let Some(b) = buf_arg(&rec, 0) {
                            let re_type = i32_at(b, 0).clamp(0, 11) as usize;
                            let renderfx = i32_at(b, 4) as u32;
                            let h_model = i32_at(b, 8);
                            let custom_shader = i32_at(b, 76) as i64;
                            let ghoul2 = i64::from_le_bytes(b[208..216].try_into().unwrap());
                            for c in out.iter_mut() {
                                c.bump(&format!("refent/{}", RETYPE_NAMES[re_type]));
                                if h_model != 0 {
                                    c.bump(&format!("refent/{}/hModel", RETYPE_NAMES[re_type]));
                                }
                                if ghoul2 != 0 {
                                    c.bump(&format!("refent/{}/ghoul2", RETYPE_NAMES[re_type]));
                                }
                                for (bit, name) in RF_NAMES {
                                    if renderfx & bit != 0 {
                                        c.bump(&format!("renderfx/{name}"));
                                    }
                                }
                                if custom_shader != 0 {
                                    c.shader("refEntity.customShader", maps.shader(custom_shader));
                                }
                            }
                        }
                    }
                    R_ADDPOLYTOSCENE | R_ADDPOLYSTOSCENE => {
                        let shader = maps.shader(rec.words.get(1).copied().unwrap_or(0));
                        // The recorder journals raw 64-bit arg registers, so
                        // scalar ints carry junk above bit 31.
                        let verts = (rec.words.get(2).copied().unwrap_or(0) & 0xFFFF_FFFF) as u64;
                        let n = if num == R_ADDPOLYSTOSCENE {
                            ((rec.words.get(4).copied().unwrap_or(1) & 0xFFFF_FFFF) as u64).max(1)
                        } else {
                            1
                        };
                        for c in out.iter_mut() {
                            *c.counts.entry(String::from("poly/calls")).or_insert(0) += n;
                            *c.counts.entry(String::from("poly/verts")).or_insert(0) += verts * n;
                            c.shader("poly", shader.clone());
                        }
                    }
                    R_ADDDECALTOSCENE => {
                        let shader = maps.shader(rec.words.get(1).copied().unwrap_or(0));
                        for c in out.iter_mut() {
                            c.bump("decal/calls");
                            c.shader("decal", shader.clone());
                        }
                    }
                    R_ADDLIGHTTOSCENE => {
                        for c in out.iter_mut() {
                            c.bump("dlight/calls");
                        }
                    }
                    R_ADDADDITIVELIGHTTOSCENE => {
                        for c in out.iter_mut() {
                            c.bump("dlight/additive");
                        }
                    }
                    R_RENDERSCENE => {
                        if let Some(b) = buf_arg(&rec, 0) {
                            // rdflags at offset 92 (refdef_t layout asserts).
                            let rdflags = i32_at(b, 92) as u32;
                            for c in out.iter_mut() {
                                c.bump("scene/RenderScene");
                                if rdflags != 0 {
                                    c.bump(&format!("scene/rdflags/{rdflags:#x}"));
                                }
                            }
                        }
                    }
                    R_CLEARSCENE => {
                        for c in out.iter_mut() {
                            c.bump("scene/ClearScene");
                        }
                    }
                    R_SETCOLOR => {
                        for c in out.iter_mut() {
                            c.bump("2d/SetColor");
                        }
                    }
                    R_DRAWSTRETCHPIC => {
                        let shader = maps.shader(rec.words.get(9).copied().unwrap_or(0));
                        for c in out.iter_mut() {
                            c.bump("2d/DrawStretchPic");
                            c.shader("2d", shader.clone());
                        }
                    }
                    R_DRAWROTATEPIC | R_DRAWROTATEPIC2 => {
                        let shader = maps.shader(rec.words.get(10).copied().unwrap_or(0));
                        for c in out.iter_mut() {
                            c.bump("2d/DrawRotatePic");
                            c.shader("2d", shader.clone());
                        }
                    }
                    R_FONT_DRAWSTRING => {
                        for c in out.iter_mut() {
                            c.bump("2d/Font_DrawString");
                        }
                    }
                    CM_MARKFRAGMENTS => {
                        for c in out.iter_mut() {
                            c.bump("marks/MarkFragments");
                        }
                    }
                    FX_ADDLINE | FX_ADDPOLY | FX_ADDBEZIER | FX_ADDPRIMITIVE | FX_ADDSPRITE
                    | FX_ADDELECTRICITY => {
                        let name = match num {
                            FX_ADDLINE => "fx/AddLine",
                            FX_ADDPOLY => "fx/AddPoly",
                            FX_ADDBEZIER => "fx/AddBezier",
                            FX_ADDPRIMITIVE => "fx/AddPrimitive",
                            FX_ADDSPRITE => "fx/AddSprite",
                            _ => "fx/AddElectricity",
                        };
                        for c in out.iter_mut() {
                            c.bump(name);
                        }
                    }
                    FX_PLAY_EFFECT | FX_PLAY_ENTITY_EFFECT => {
                        if let Some(name) = str_arg(&rec, 0) {
                            for c in out.iter_mut() {
                                c.bump("fx/PlayEffect(name)");
                                *c.effects.entry(name.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                    FX_PLAY_EFFECT_ID | FX_PLAY_PORTAL_EFFECT_ID | FX_PLAY_ENTITY_EFFECT_ID
                    | FX_PLAY_BOLTED_EFFECT_ID => {
                        let name = maps.effect(rec.words.get(1).copied().unwrap_or(0));
                        for c in out.iter_mut() {
                            c.bump("fx/PlayEffect(id)");
                            *c.effects.entry(name.clone()).or_insert(0) += 1;
                        }
                    }
                    _ => {}
                }
            }
            REC_SYSCALL_EXIT => {
                if let Some((num, name)) = pending.take() {
                    if rec.cmd == num {
                        let handle = rec.ret & 0xFFFF_FFFF;
                        match num {
                            R_REGISTERSHADER | R_REGISTERSHADERNOMIP => {
                                maps.shaders.insert(handle, name);
                            }
                            FX_REGISTER_EFFECT => {
                                maps.effects.insert(handle, name);
                            }
                            R_REGISTERMODEL => {
                                for c in out.iter_mut() {
                                    c.models.insert(name.clone());
                                }
                            }
                            R_REGISTERSKIN => {
                                for c in out.iter_mut() {
                                    c.skins.insert(name.clone());
                                }
                            }
                            R_REGISTERFONT => {
                                for c in out.iter_mut() {
                                    c.fonts.insert(name.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn render(label: &str, c: &Census, full: bool) -> String {
    let mut s = format!("\n## {label}\n\n");
    s.push_str("| submission | count |\n|---|---:|\n");
    for (k, v) in &c.counts {
        s.push_str(&format!("| {k} | {v} |\n"));
    }
    if full {
        for (surface, names) in &c.shaders {
            s.push_str(&format!(
                "\n### shaders via {surface} ({} distinct)\n\n",
                names.len()
            ));
            for (n, v) in names {
                s.push_str(&format!("- `{n}` x{v}\n"));
            }
        }
        s.push_str(&format!("\n### effects played ({} distinct)\n\n", c.effects.len()));
        for (n, v) in &c.effects {
            s.push_str(&format!("- `{n}` x{v}\n"));
        }
        s.push_str(&format!("\n### models registered ({})\n\n", c.models.len()));
        for n in &c.models {
            s.push_str(&format!("- `{n}`\n"));
        }
        s.push_str(&format!("\n### skins registered ({})\n\n", c.skins.len()));
        for n in &c.skins {
            s.push_str(&format!("- `{n}`\n"));
        }
        s.push_str(&format!("\n### fonts registered ({})\n\n", c.fonts.len()));
        for n in &c.fonts {
            s.push_str(&format!("- `{n}`\n"));
        }
    }
    s
}

#[test]
#[ignore = "census tool - needs local traces (DEC-48.4); run with --ignored"]
fn scene_trap_census() {
    let home = std::env::var("HOME").unwrap_or_default();
    let traces: Vec<PathBuf> = match std::env::var("JKA_TRACES") {
        Ok(list) => list.split(':').map(PathBuf::from).collect(),
        Err(_) => ["swoop1", "sabers1", "spectator", "ffa1"]
            .iter()
            .map(|t| PathBuf::from(&home).join(format!("Developer/jka/trace-{t}.bin")))
            .collect(),
    };

    let mut aggregate = Census::default();
    let mut report = String::from("# Scene-trap census (wayfinder #17, DEC-54)\n");
    for path in &traces {
        if !path.exists() {
            eprintln!("SKIP: no trace at {}", path.display());
            continue;
        }
        // Handle maps are per-process, so each trace starts fresh.
        let mut maps = NameMaps::default();
        let mut per = Census::default();
        {
            let mut sinks: [&mut Census; 2] = [&mut per, &mut aggregate];
            walk(path, &mut maps, &mut sinks);
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        report.push_str(&render(stem, &per, false));
    }
    report.push_str(&render("aggregate (all traces)", &aggregate, true));

    println!("{report}");
    if let Ok(out) = std::env::var("JKA_CENSUS_OUT") {
        std::fs::write(&out, &report).expect("write census report");
        eprintln!("census written to {out}");
    }
}
