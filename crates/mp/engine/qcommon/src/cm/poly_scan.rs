#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_long;

use crate::cm::point::POINT;
use crate::cm::polyedge::POLYEDGE;

/// Raven's `cm_draw.cpp` scan-converter file statics (`n`, `pt`, `nact`,
/// `active`), gathered as one scratch record.
///
/// Raven hoists these to file scope "for speed". Every reader
/// (`del_edge`, `ins_edge`, `compare_ind`, `compare_active`) runs inside one
/// `CDraw32::DrawPolygon` call, so the port makes them a local the polygon
/// walk owns instead of a global (porting-rules §B3).
///
/// Type definition source: `oracle/codemp/qcommon/cm_draw.cpp:1080-1085`
pub struct PolyScan<'a> {
    /// number of vertices
    pub n: c_long,
    /// vertices
    pub pt: &'a [POINT],
    /// number of active edges
    pub nact: c_long,
    /// active edge list: edges crossing scanline y
    pub active: [POLYEDGE; 256],
}

impl<'a> PolyScan<'a> {
    /// Start an empty scan of `point`, the state `DrawPolygon` sets up before
    /// its first scanline (`n = nvert`, `pt = point`, `nact = 0`).
    ///
    /// Source: `oracle/codemp/qcommon/cm_draw.cpp:1210-1231`
    pub fn new(nvert: c_long, point: &'a [POINT]) -> Self {
        PolyScan {
            n: nvert,
            pt: point,
            nact: 0,
            active: [POLYEDGE::default(); 256],
        }
    }
}
