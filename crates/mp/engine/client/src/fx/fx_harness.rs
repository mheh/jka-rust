//! The differential-parity rig for the FX system (porting-rules §18, DEC-61.5).
//!
//! `tools/fx-oracle` compiles the unmodified Raven FX translation units and dumps
//! a text emission stream over synthetic `.efx` fixtures.
//! This module gives the Rust port the same scripted inputs and the same capture
//! sink, so `cargo test` reproduces the committed goldens with no C++ toolchain.
//!
//! Source: `tools/fx-oracle/README.md`

#![allow(non_camel_case_types, non_snake_case)]

use std::collections::BTreeMap;
use std::collections::VecDeque;

use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{cplane_t, ENTITYNUM_NONE};
use native_math::rng::QRand;
use native_math::vector::vec3_t;

/// One scripted `G2API_GetBoltMatrix` reply: whether the bolt exists, its origin, its axis.
///
/// Source: `tools/fx-oracle/host.cpp:453-457`
pub type FxBoltReply = (bool, vec3_t, [vec3_t; 3]);

/// An all-zero `trace_t`, which `trace_t` itself cannot derive.
pub fn fx_zero_trace() -> trace_t {
    trace_t {
        allsolid: 0,
        startsolid: 0,
        entityNum: 0,
        fraction: 0.0,
        endpos: [0.0; 3],
        plane: cplane_t {
            normal: [0.0; 3],
            dist: 0.0,
            r#type: 0,
            signbits: 0,
            pad: [0; 2],
        },
        surfaceFlags: 0,
        contents: 0,
    }
}

/// Print a float the way the C++ dumper does: the raw 32-bit IEEE-754 pattern.
///
/// Decimal text would hide a one-bit difference, so the goldens never carry it.
pub fn fx_f32(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}

/// Print a three-float vector as three bit patterns.
pub fn fx_v3(v: &vec3_t) -> String {
    format!("{} {} {}", fx_f32(v[0]), fx_f32(v[1]), fx_f32(v[2]))
}

/// The scripted engine the parity test puts behind the FX system.
///
/// Every field is either an input the scenario script pins or the captured
/// output stream the golden compares.
/// It is `None` in a live client, where the FX system calls the engine directly.
pub struct FxHarness {
    /// The generator the FX draws come from. The scenario seeds it.
    pub rng: QRand,
    /// One record per outbound call, in call order. This is the golden body.
    pub out: Vec<String>,

    /// Scripted `CG_TRACE`/`CG_G2TRACE` replies. The last entry repeats.
    pub traces: VecDeque<trace_t>,
    /// Scripted `CG_POINT_CONTENTS` replies. The last entry repeats.
    pub point_contents: VecDeque<i32>,
    /// Scripted `GetOriginAxisFromBolt` replies. The last entry repeats.
    pub bolts: VecDeque<FxBoltReply>,
    /// Scripted `CG_GET_LERP_ORIGIN` replies. The last entry repeats.
    pub lerp_origins: VecDeque<vec3_t>,

    /// Registered shader names, in first-registration order. The handle is the index plus one.
    pub shaders: Vec<String>,
    /// Registered model names, in first-registration order.
    pub models: Vec<String>,
    /// Registered sound names, in first-registration order.
    pub sounds: Vec<String>,

    /// The synthetic `.efx` fixtures, keyed by the path the scheduler builds.
    pub files: BTreeMap<String, String>,
    /// Open file handles: handle to (contents, read cursor).
    pub open_files: BTreeMap<i32, (String, usize)>,
    /// The next file handle to hand out.
    pub next_file_handle: i32,
}

impl Default for FxHarness {
    fn default() -> Self {
        FxHarness {
            rng: QRand::default(),
            out: Vec::new(),
            traces: VecDeque::new(),
            point_contents: VecDeque::new(),
            bolts: VecDeque::new(),
            lerp_origins: VecDeque::new(),
            shaders: Vec::new(),
            models: Vec::new(),
            sounds: Vec::new(),
            files: BTreeMap::new(),
            open_files: BTreeMap::new(),
            next_file_handle: 1,
        }
    }
}

impl FxHarness {
    /// Append one golden record.
    pub fn emit(&mut self, record: String) {
        self.out.push(record);
    }

    /// Hand out a deterministic handle for `name` and record the registration.
    ///
    /// A repeat name returns the first handle and still records, because the
    /// oracle stub records every call.
    pub fn register(kind: &str, table: &mut Vec<String>, out: &mut Vec<String>, name: &str) -> i32 {
        let handle = match table.iter().position(|n| n == name) {
            Some(i) => i as i32 + 1,
            None => {
                table.push(name.to_string());
                table.len() as i32
            }
        };
        out.push(format!("{kind} {name} -> {handle}"));
        handle
    }

    /// Pop the next scripted trace reply. The last entry repeats once the queue drains.
    ///
    /// An empty queue answers with the miss reply: the trace ran clean to `end`.
    ///
    /// Source: `tools/fx-oracle/host.cpp:554-562`
    pub fn next_trace(&mut self, end: vec3_t) -> trace_t {
        let reply = if self.traces.len() > 1 {
            self.traces.pop_front()
        } else {
            self.traces.front().copied()
        };
        match reply {
            Some(tr) => tr,
            None => {
                let mut tr = fx_zero_trace();
                tr.fraction = 1.0;
                tr.endpos = end;
                tr.plane.normal = [0.0, 0.0, 1.0];
                tr.entityNum = ENTITYNUM_NONE as _;
                tr
            }
        }
    }

    /// Pop the next scripted point-contents reply. The last entry repeats, and a miss answers `0`.
    ///
    /// Source: `tools/fx-oracle/host.cpp:573-584`
    pub fn next_point_contents(&mut self) -> i32 {
        if self.point_contents.len() > 1 {
            self.point_contents.pop_front().unwrap_or(0)
        } else {
            self.point_contents.front().copied().unwrap_or(0)
        }
    }

    /// Pop the next scripted bolt reply. A miss means the bolt does not exist.
    ///
    /// Source: `tools/fx-oracle/host.cpp:672-703`
    pub fn next_bolt(&mut self) -> FxBoltReply {
        let reply = if self.bolts.len() > 1 {
            self.bolts.pop_front()
        } else {
            self.bolts.front().copied()
        };
        reply.unwrap_or((false, [0.0; 3], [[0.0; 3]; 3]))
    }

    /// Pop the next scripted lerp-origin reply. A miss answers the origin.
    ///
    /// Source: `tools/fx-oracle/host.cpp:586-599`
    pub fn next_lerp_origin(&mut self) -> vec3_t {
        let reply = if self.lerp_origins.len() > 1 {
            self.lerp_origins.pop_front()
        } else {
            self.lerp_origins.front().copied()
        };
        reply.unwrap_or([0.0; 3])
    }

    /// Format one `miniRefEntity_t` the way the C++ dumper prints `REFENT`/`MINIREFENT`.
    pub fn refent_fields(ent: &miniRefEntity_t) -> String {
        let mut axis = String::new();
        for a in ent.axis.iter() {
            for v in a.iter() {
                if !axis.is_empty() {
                    axis.push(' ');
                }
                axis.push_str(&fx_f32(*v));
            }
        }
        format!(
            "reType {} renderfx {} hModel {} origin {} oldorigin {} axis {} nonNormalizedAxes {} \
             radius {} rotation {} shaderTime {} customShader {} shaderRGBA {} {} {} {} \
             shaderTexCoord {} {} frame {}",
            ent.reType as i32,
            ent.renderfx,
            ent.hModel,
            fx_v3(&ent.origin),
            fx_v3(&ent.oldorigin),
            axis,
            ent.nonNormalizedAxes,
            fx_f32(ent.radius),
            fx_f32(ent.rotation),
            fx_f32(ent.shaderTime),
            ent.customShader,
            ent.shaderRGBA[0],
            ent.shaderRGBA[1],
            ent.shaderRGBA[2],
            ent.shaderRGBA[3],
            fx_f32(ent.shaderTexCoord[0]),
            fx_f32(ent.shaderTexCoord[1]),
            ent.frame,
        )
    }

    /// Format one poly vertex the way the C++ dumper prints `POLYV`.
    pub fn polyvert_fields(index: usize, v: &polyVert_t) -> String {
        format!(
            "POLYV {} xyz {} st {} {} modulate {} {} {} {}",
            index,
            fx_v3(&v.xyz),
            fx_f32(v.st[0]),
            fx_f32(v.st[1]),
            v.modulate[0],
            v.modulate[1],
            v.modulate[2],
            v.modulate[3],
        )
    }
}
