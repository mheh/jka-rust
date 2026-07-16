//! Differential parity test for the jampgame `q_math` + `bg_lib` ports against
//! the Raven oracle. Reproduces `tools/jampgame-oracle/`'s canonical bit-exact
//! dumps by calling the PORTED functions (`mp_game::q_math`,
//! `mp_game::bg_lib`, `mp_game::bg_channel::Rng`) and byte-compares to the
//! committed goldens under `tests/oracle/golden/`.
//!
//! Single-threaded by construction: the oracle keeps its RNG in file statics
//! (a fresh dumper process per family); the Rust side mirrors that with ONE
//! `Rng` per family and a single `#[test]` per family whose sub-checks run
//! sequentially. See `tools/jampgame-oracle/README.md`.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::fmt::Write as _;
use std::path::PathBuf;

use mp_game::bg_channel::Rng;
use mp_game::bg_lib::{__builtin___memmove_chk, _atof, atof, atoi, qsort};
use mp_game::q_math::*;
use mp_game::shared::cplane_t;

fn oracle_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/oracle")
}

fn compare(name: &str, got: &str) {
    let golden_path = oracle_dir().join("golden").join(format!("{name}.txt"));
    let golden = std::fs::read_to_string(&golden_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", golden_path.display()));
    if got == golden {
        return;
    }
    let g: Vec<&str> = golden.lines().collect();
    let o: Vec<&str> = got.lines().collect();
    for (i, (gl, ol)) in g.iter().zip(o.iter()).enumerate() {
        if gl != ol {
            panic!(
                "{name} parity mismatch at line {} (oracle vs port):\n  oracle: {gl}\n  port:   {ol}",
                i + 1
            );
        }
    }
    panic!(
        "{name} parity length mismatch: oracle {} lines, port {} lines",
        g.len(),
        o.len()
    );
}

// --- bit-exact print helpers (mirror dumpcommon.h) ---

/// `dumpcommon.h` `f2b` — NaN sign/payload is platform-defined (ARM default
/// qNaN 0x7fc00000, x86 SSE 0xffc00000), so any NaN canonicalizes to the
/// positive quiet NaN (§19 normalization), identically on both sides.
trait CBits {
    type Bits;
    fn cbits(self) -> Self::Bits;
}
impl CBits for f32 {
    type Bits = u32;
    fn cbits(self) -> u32 {
        let b = self.to_bits();
        if (b & 0x7f80_0000) == 0x7f80_0000 && (b & 0x007f_ffff) != 0 {
            return 0x7fc0_0000;
        }
        b
    }
}
/// `dumpcommon.h` `d2b` — same canonicalization for f64.
impl CBits for f64 {
    type Bits = u64;
    fn cbits(self) -> u64 {
        let b = self.to_bits();
        if (b & 0x7ff0_0000_0000_0000) == 0x7ff0_0000_0000_0000 && (b & 0x000f_ffff_ffff_ffff) != 0
        {
            return 0x7ff8_0000_0000_0000;
        }
        b
    }
}

fn v3(o: &mut String, v: &[f32; 3]) {
    let _ = writeln!(
        o,
        "{:08x} {:08x} {:08x}",
        v[0].cbits(),
        v[1].cbits(),
        v[2].cbits()
    );
}
fn v4(o: &mut String, v: &[f32; 4]) {
    let _ = writeln!(
        o,
        "{:08x} {:08x} {:08x} {:08x}",
        v[0].cbits(),
        v[1].cbits(),
        v[2].cbits(),
        v[3].cbits()
    );
}
fn dot_nz(v: &[f32; 3]) -> bool {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) != 0.0
}

fn load_vectors() -> Vec<[f32; 3]> {
    let path = oracle_dir().join("fixtures/vectors.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut out = Vec::new();
    for line in text.lines() {
        let w: Vec<u32> = line
            .split_whitespace()
            .map(|h| u32::from_str_radix(h, 16).unwrap())
            .collect();
        if w.len() == 3 {
            out.push([
                f32::from_bits(w[0]),
                f32::from_bits(w[1]),
                f32::from_bits(w[2]),
            ]);
        }
    }
    out
}

// ============================ q_math family ============================

fn dump_rng(o: &mut String) {
    let seeds: [i32; 4] = [0x89abcdefu32 as i32, 0, 1, 0xdeadbeefu32 as i32];
    o.push_str("== rng ==\n");
    let mut rng = Rng::new();
    for s in seeds {
        let _ = writeln!(o, "seed {:08x}", s as u32);
        rng.Rand_Init(s);
        for _ in 0..100 {
            let _ = writeln!(o, "fl {:08x}", rng.flrand(0.0, 1.0).cbits());
            let _ = writeln!(o, "fl {:08x}", rng.flrand(-1.0, 1.0).cbits());
            let _ = writeln!(o, "fl {:08x}", rng.flrand(-100.0, 100.0).cbits());
            let _ = writeln!(o, "qf {:08x}", rng.Q_flrand(0.0, 1000.0).cbits());
            let _ = writeln!(o, "ir {}", rng.irand(0, 1));
            let _ = writeln!(o, "ir {}", rng.irand(0, 100));
            let _ = writeln!(o, "ir {}", rng.irand(-50, 50));
            let _ = writeln!(o, "qi {}", rng.Q_irand(0, 32767));
        }
    }
}

fn dump_qrand(o: &mut String) {
    o.push_str("== qrand ==\n");
    let mut seed: c_int = 12345;
    for _ in 0..30 {
        let _ = writeln!(o, "qr {}", Q_rand(&mut seed as *mut c_int));
    }
    seed = 12345;
    for _ in 0..30 {
        let _ = writeln!(o, "qrn {:08x}", Q_random(&mut seed as *mut c_int).cbits());
    }
    seed = 12345;
    for _ in 0..30 {
        let _ = writeln!(o, "qcr {:08x}", Q_crandom(&mut seed as *mut c_int).cbits());
    }
    seed = -1;
    for _ in 0..20 {
        let _ = writeln!(o, "qr {}", Q_rand(&mut seed as *mut c_int));
    }
}

fn dump_scalars(o: &mut String) {
    o.push_str("== scalars ==\n");
    let ci: [i32; 22] = [
        -300, -128, -1, 0, 1, 127, 128, 200, -32769, -32768, 32767, 32768, 100000, -100000, 1, 2,
        3, 255, 256, 1023, 1024, 0x7fffffff,
    ];
    for x in ci {
        let ax = if x < 0 { -x } else { x };
        let _ = writeln!(
            o,
            "cc {} cs {} log2 {}",
            ClampChar(x) as i32,
            ClampShort(x) as i32,
            Q_log2(ax)
        );
    }
    let cf: [u32; 10] = [
        0x3f800000, 0x40490fdb, 0x00000001, 0x80000000, 0x7f7fffff, 0x3dcccccd, 0xc0000000,
        0x42c80000, 0x00800000, 0xbf000000,
    ];
    for u in cf {
        let v = f32::from_bits(u);
        let a = if v < 0.0 { -v } else { v };
        let _ = writeln!(
            o,
            "rsqrt {:08x} fabs {:08x}",
            Q_rsqrt(a).cbits(),
            Q_fabs(v).cbits()
        );
    }
    for y in 1..=6 {
        let _ = writeln!(o, "powf {:08x}", powf(1.5, y).cbits());
    }
    let mut b = 0;
    while b < 162 {
        let mut d = [0.0f32; 3];
        ByteToDir(b, &mut d);
        let _ = write!(o, "b2d {} ", b);
        v3(o, &d);
        b += 17;
    }
    {
        let mut d = [0.0; 3];
        ByteToDir(-1, &mut d);
        let _ = write!(o, "b2d -1 ");
        v3(o, &d);
    }
    {
        let mut d = [0.0; 3];
        ByteToDir(500, &mut d);
        let _ = write!(o, "b2d 500 ");
        v3(o, &d);
    }
    let cc: [[u32; 3]; 4] = [
        [0x3f000000, 0x3e800000, 0x3f800000],
        [0, 0, 0],
        [0x3f800000, 0x3f800000, 0x3f800000],
        [0x3e4ccccd, 0x3f19999a, 0x3f733333],
    ];
    for c in cc {
        let (r, g, bl) = (
            f32::from_bits(c[0]),
            f32::from_bits(c[1]),
            f32::from_bits(c[2]),
        );
        let _ = writeln!(
            o,
            "cb3 {:08x} cb4 {:08x}",
            ColorBytes3(r, g, bl),
            ColorBytes4(r, g, bl, 0.5)
        );
        let inv = [r, g, bl];
        let mut out = [0.0; 3];
        let m = NormalizeColor(inv, &mut out);
        let _ = write!(o, "ncol {:08x} ", m.cbits());
        v3(o, &out);
    }
}

fn dump_n2ll(o: &mut String) {
    o.push_str("== n2ll ==\n");
    let ns: [[u32; 3]; 6] = [
        [0x3f800000, 0, 0],
        [0, 0x3f800000, 0],
        [0, 0, 0x3f800000],
        [0, 0, 0xbf800000],
        [0x3f0f5c29, 0x3f0f5c29, 0x3ef1a9fc],
        [0xbf13cd36, 0x3f13cd36, 0],
    ];
    for nn in ns {
        let mut n = [
            f32::from_bits(nn[0]),
            f32::from_bits(nn[1]),
            f32::from_bits(nn[2]),
        ];
        VectorNormalize(&mut n);
        let mut bytes = [0u8; 2];
        NormalToLatLong(n, bytes.as_mut_ptr());
        let _ = writeln!(o, "n2ll {:02x}{:02x}", bytes[0], bytes[1]);
    }
}

fn dump_planes(o: &mut String, vecs: &[[f32; 3]]) {
    o.push_str("== planes ==\n");
    let n = vecs.len();
    for i in 0..n {
        let nv = vecs[i];
        for ty in 0..=3u8 {
            let mut p: cplane_t = unsafe { core::mem::zeroed() };
            p.normal = nv;
            p.dist = nv[0];
            p.r#type = ty;
            SetPlaneSignbits(&mut p as *mut cplane_t);
            let j = (i + 1) % n;
            let k = (i + 2) % n;
            let side = BoxOnPlaneSide(vecs[j], vecs[k], &mut p as *mut cplane_t);
            let _ = writeln!(o, "plane {} ty {} sb {} side {}", i, ty, p.signbits, side);
        }
    }
}

fn dump_vecmath(o: &mut String, vecs: &[[f32; 3]]) {
    o.push_str("== vecmath ==\n");
    let n = vecs.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let k = (i + 2) % n;
        let a = vecs[i];
        let b = vecs[j];
        let c = vecs[k];
        let _ = writeln!(o, "i {}", i);

        {
            let (mut f, mut r, mut up) = ([0.0; 3], [0.0; 3], [0.0; 3]);
            AngleVectors(a, Some(&mut f), Some(&mut r), Some(&mut up));
            let _ = writeln!(
                o,
                "av {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                f[0].cbits(),
                f[1].cbits(),
                f[2].cbits(),
                r[0].cbits(),
                r[1].cbits(),
                r[2].cbits(),
                up[0].cbits(),
                up[1].cbits(),
                up[2].cbits()
            );
        }
        {
            let mut ang = [0.0; 3];
            vectoangles(a, &mut ang);
            let _ = write!(o, "va ");
            v3(o, &ang);
        }
        {
            let mut t = a;
            let len = VectorNormalize(&mut t);
            let _ = write!(o, "vn {:08x} ", len.cbits());
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            let len = VectorNormalize2(a, &mut t);
            let _ = write!(o, "vn2 {:08x} ", len.cbits());
            v3(o, &t);
        }
        let _ = writeln!(
            o,
            "vl {:08x} vls {:08x}",
            VectorLength(a).cbits(),
            VectorLengthSquared(a).cbits()
        );
        let _ = writeln!(
            o,
            "dist {:08x} dsq {:08x}",
            Distance(a, b).cbits(),
            DistanceSquared(a, b).cbits()
        );
        let _ = writeln!(
            o,
            "dh {:08x} dhs {:08x}",
            DistanceHorizontal(a, b).cbits(),
            DistanceHorizontalSquared(a, b).cbits()
        );
        let _ = writeln!(o, "vcmp {} {}", VectorCompare(a, b), VectorCompare(a, a));
        {
            let mut t = [0.0; 3];
            CrossProduct(a, b, &mut t);
            let _ = write!(o, "cross ");
            v3(o, &t);
        }
        let _ = writeln!(
            o,
            "dot {:08x} _dot {:08x}",
            _DotProduct(a, b).cbits(),
            _DotProduct(a, b).cbits()
        );
        let _ = writeln!(o, "dpn {:08x}", DotProductNormalize(a, b).cbits());
        {
            let mut t = a;
            VectorInverse(&mut t);
            let _ = write!(o, "vinv ");
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            _VectorMA(a, 2.5, b, &mut t);
            let _ = write!(o, "vma ");
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            _VectorAdd(a, b, &mut t);
            let _ = write!(o, "vadd ");
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            _VectorSubtract(a, b, &mut t);
            let _ = write!(o, "vsub ");
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            _VectorScale(a, 1.5, &mut t);
            let _ = write!(o, "vscale ");
            v3(o, &t);
        }
        {
            let q = [a[0], a[1], a[2], b[0]];
            let mut ov = [0.0; 4];
            Vector4Scale(q, 2.0, &mut ov);
            let _ = write!(o, "v4s ");
            v4(o, &ov);
        }
        if dot_nz(&b) {
            let mut t = [0.0; 3];
            ProjectPointOnPlane(&mut t, a, b);
            let _ = write!(o, "proj ");
            v3(o, &t);
        } else {
            o.push_str("proj SKIP\n");
        }
        {
            let (mut t, mut u) = ([0.0; 3], [0.0; 3]);
            MakeNormalVectors(a, &mut t, &mut u);
            let _ = writeln!(
                o,
                "mnv {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                t[0].cbits(),
                t[1].cbits(),
                t[2].cbits(),
                u[0].cbits(),
                u[1].cbits(),
                u[2].cbits()
            );
        }
        if dot_nz(&a) {
            let mut t = [0.0; 3];
            PerpendicularVector(&mut t, a);
            let _ = write!(o, "perp ");
            v3(o, &t);
        } else {
            o.push_str("perp SKIP\n");
        }
        if dot_nz(&a) {
            let mut t = [0.0; 3];
            RotatePointAroundVector(&mut t, a, b, a[0]);
            let _ = write!(o, "rot ");
            v3(o, &t);
        } else {
            o.push_str("rot SKIP\n");
        }
        {
            let mut pl = [0.0; 4];
            let ok = PlaneFromPoints(&mut pl, a, b, c);
            let _ = write!(o, "pfp {} ", ok);
            v4(o, &pl);
        }
        let _ = writeln!(o, "angsub {:08x}", AngleSubtract(a[0], a[1]).cbits());
        {
            let mut t = [0.0; 3];
            AnglesSubtract(a, b, &mut t);
            let _ = write!(o, "angssub ");
            v3(o, &t);
        }
        let _ = writeln!(o, "lerp {:08x}", LerpAngle(a[0], a[1], a[2]).cbits());
        let _ = writeln!(
            o,
            "an360 {:08x} an180 {:08x} amod {:08x} adel {:08x}",
            AngleNormalize360(a[0]).cbits(),
            AngleNormalize180(a[0]).cbits(),
            AngleMod(a[0]).cbits(),
            AngleDelta(a[0], a[1]).cbits()
        );
        let _ = writeln!(o, "rfb {:08x}", RadiusFromBounds(a, b).cbits());
        let _ = writeln!(o, "d2b {}", DirToByte(a));
        {
            let mut res = [0.0; 3];
            let ok = G_FindClosestPointOnLineSegment(a, b, c, &mut res);
            let _ = write!(o, "gclose {} ", ok);
            v3(o, &res);
        }
        let _ = writeln!(
            o,
            "gdist {:08x}",
            G_PointDistFromLineSegment(a, b, c).cbits()
        );
        {
            let mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];
            AnglesToAxis(a, axis.as_mut_ptr());
            let _ = writeln!(
                o,
                "a2a {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                axis[0][0].cbits(),
                axis[0][1].cbits(),
                axis[0][2].cbits(),
                axis[1][0].cbits(),
                axis[1][1].cbits(),
                axis[1][2].cbits(),
                axis[2][0].cbits(),
                axis[2][1].cbits(),
                axis[2][2].cbits()
            );
            let mut t = [0.0; 3];
            VectorRotate(b, axis.as_mut_ptr(), &mut t);
            let _ = write!(o, "vrot ");
            v3(o, &t);
            let mut cp: [[f32; 3]; 3] = [[0.0; 3]; 3];
            AxisCopy(axis.as_mut_ptr(), cp.as_mut_ptr());
            let _ = write!(o, "acopy ");
            v3(o, &cp[1]);
            RotateAroundDirection(axis.as_mut_ptr(), a[1]);
            let _ = writeln!(
                o,
                "rad {:08x} {:08x} {:08x}",
                axis[1][0].cbits(),
                axis[1][1].cbits(),
                axis[1][2].cbits()
            );
        }
        {
            let mut ax: [[f32; 3]; 3] = [[0.0; 3]; 3];
            AxisClear(ax.as_mut_ptr());
            let _ = writeln!(
                o,
                "aclear {:08x} {:08x} {:08x}",
                ax[0][0].cbits(),
                ax[1][1].cbits(),
                ax[2][2].cbits()
            );
        }
        {
            let m1 = [[a[0], a[1], a[2]], [b[0], b[1], b[2]], [c[0], c[1], c[2]]];
            let m2 = [[c[0], b[1], a[2]], [a[0], c[1], b[2]], [b[0], a[1], c[2]]];
            let mut ou = [[0.0; 3]; 3];
            MatrixMultiply(&m1, &m2, &mut ou);
            let _ = writeln!(
                o,
                "mm {:08x} {:08x} {:08x}",
                ou[0][0].cbits(),
                ou[1][1].cbits(),
                ou[2][2].cbits()
            );
        }
        {
            let (mut mn, mut mx) = ([0.0; 3], [0.0; 3]);
            ClearBounds(&mut mn, &mut mx);
            AddPointToBounds(a, &mut mn, &mut mx);
            AddPointToBounds(b, &mut mn, &mut mx);
            let _ = writeln!(
                o,
                "bounds {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                mn[0].cbits(),
                mn[1].cbits(),
                mn[2].cbits(),
                mx[0].cbits(),
                mx[1].cbits(),
                mx[2].cbits()
            );
        }
    }
}

#[test]
fn qmath_parity() {
    let vecs = load_vectors();
    let mut o = String::new();
    dump_rng(&mut o);
    dump_qrand(&mut o);
    dump_scalars(&mut o);
    dump_n2ll(&mut o);
    dump_planes(&mut o, &vecs);
    dump_vecmath(&mut o, &vecs);
    o.push_str("== end ==\n");
    compare("qmath", &o);
}

// ============================ bg_lib family ============================

// bg_lib's rand/srand are QVM-only surface (retail's JK2_game.vcproj excludes
// bg_lib.c; the native module links the MSVC CRT rand — see
// bg_channel::rng). Their 69069-LCG dump section was retired 2026-07-14;
// Rng's CRT rand is pinned by a unit test in rng.rs instead.

fn dump_atox(o: &mut String) {
    let path = oracle_dir().join("fixtures/strings.txt");
    let data = std::fs::read(&path).unwrap();
    o.push_str("== atox ==\n");
    let mut idx = 0;
    for rec in data.split(|&x| x == 0) {
        let mut cbuf: Vec<u8> = rec.to_vec();
        cbuf.push(0);
        let p0 = cbuf.as_ptr() as *const c_char;
        let _ = writeln!(o, "a {} atoi {}", idx, atoi(p0));
        let _ = writeln!(o, "a {} atof {:016x}", idx, atof(p0).cbits());
        let mut sp: *const c_char = cbuf.as_ptr() as *const c_char;
        let v = _atof(&mut sp as *mut *const c_char);
        let adv = (sp as usize) - (cbuf.as_ptr() as usize);
        let _ = writeln!(o, "a {} _atof {:016x} adv {}", idx, v.cbits(), adv);
        idx += 1;
    }
    let _ = writeln!(o, "atox count {}", idx);
}

extern "C" fn cmp_int(a: *const c_void, b: *const c_void) -> c_int {
    let x = unsafe { *(a as *const c_int) };
    let y = unsafe { *(b as *const c_int) };
    (x > y) as c_int - (x < y) as c_int
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Kv {
    key: c_int,
    payload: c_int,
}

extern "C" fn cmp_kv(a: *const c_void, b: *const c_void) -> c_int {
    let x = unsafe { (*(a as *const Kv)).key };
    let y = unsafe { (*(b as *const Kv)).key };
    (x > y) as c_int - (x < y) as c_int
}

fn dump_qsort(o: &mut String) {
    o.push_str("== qsort ==\n");
    let path = oracle_dir().join("fixtures/ints.txt");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut arr: Vec<c_int> = text.lines().filter_map(|l| l.trim().parse().ok()).collect();
    let n = arr.len();
    let f = cmp_int as extern "C" fn(*const c_void, *const c_void) -> c_int;
    qsort(
        arr.as_mut_ptr() as *mut c_void,
        n,
        core::mem::size_of::<c_int>(),
        f as usize as *mut c_void,
    );
    let _ = writeln!(o, "ints {}", n);
    for v in &arr {
        let _ = writeln!(o, "{}", v);
    }

    let mut kv: Vec<Kv> = [
        (5, 0),
        (3, 1),
        (5, 2),
        (1, 3),
        (3, 4),
        (9, 5),
        (1, 6),
        (7, 7),
        (3, 8),
        (5, 9),
        (0, 10),
        (8, 11),
        (2, 12),
        (8, 13),
        (4, 14),
        (6, 15),
        (3, 16),
        (9, 17),
        (1, 18),
        (5, 19),
    ]
    .iter()
    .map(|&(key, payload)| Kv { key, payload })
    .collect();
    let kn = kv.len();
    let fk = cmp_kv as extern "C" fn(*const c_void, *const c_void) -> c_int;
    qsort(
        kv.as_mut_ptr() as *mut c_void,
        kn,
        core::mem::size_of::<Kv>(),
        fk as usize as *mut c_void,
    );
    let _ = writeln!(o, "kv {}", kn);
    for e in &kv {
        let _ = writeln!(o, "{} {}", e.key, e.payload);
    }
}

fn print_buf(o: &mut String, tag: &str, b: &[u8]) {
    let mut s = tag.to_string();
    for x in b {
        let _ = write!(s, " {:02x}", x);
    }
    s.push('\n');
    o.push_str(&s);
}

fn dump_memmove(o: &mut String) {
    o.push_str("== memmove ==\n");
    let mut dummy = 0u8;
    let bos = &mut dummy as *mut u8 as *mut c_void;
    let cases: [(&str, usize, usize, usize); 5] = [
        ("fwd", 4, 0, 16),
        ("back", 0, 4, 16),
        ("nonov", 20, 0, 8),
        ("same", 0, 0, 12),
        ("zero", 1, 0, 0),
    ];
    for (tag, doff, soff, count) in cases {
        let mut b = [0u8; 32];
        for (i, e) in b.iter_mut().enumerate() {
            *e = i as u8;
        }
        let dst = unsafe { b.as_mut_ptr().add(doff) } as *mut c_void;
        let src = unsafe { b.as_ptr().add(soff) } as *const c_void;
        __builtin___memmove_chk(dst, src, count, bos);
        print_buf(o, tag, &b);
    }
}

#[test]
fn bglib_parity() {
    let mut o = String::new();
    dump_atox(&mut o);
    dump_qsort(&mut o);
    dump_memmove(&mut o);
    o.push_str("== end ==\n");
    compare("bglib", &o);
}

// ============================ bg_saberLoad family ============================
//
// Reproduces `tests/oracle/golden/saberload.txt` by driving the
// PORTED `WP_SaberLoadParms` + `WP_SaberParseParms` over the same
// `fixtures/sabers/*.sab`. A tiny `TestTraps` serves the fixtures dir through
// the real `BgTraps` FS seam and mints deterministic skin handles; the
// context-free sound registrations reach a configstring-servicing mock engine
// through the real `g_strap` seam (real 1,2,3… indices), and their order is
// observed through the port's `saber_snd_tape_*` seam. See
// `tools/jampgame-oracle/README.md`.
mod saberload {
    use super::{compare, oracle_dir, CBits};
    use core::ffi::{c_char, c_int, c_void};
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use std::ffi::CStr;
    use std::fmt::Write as _;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use mp_abi::game::imports::MpGameImport;
    use mp_engine_qcommon::vm::{arm_game_slot, game_syscall_trampoline};
    use mp_engine_select::Engine;
    use mp_game::bg_channel::{BgState, BgTraps, GameCallbacksImpl};
    use mp_game::bg_saberLoad::{
        saber_snd_tape_drain, saber_snd_tape_enable, WP_SaberLoadParms, WP_SaberParseParms,
    };
    use mp_game::g_strap::init_strap_engine;
    use mp_game::prelude::*;

    // The saber names parsed, in order. Mirrors main_saberload.c's g_names.
    const NAMES: &[&str] = &[
        "Kyle",
        "staff_saber",
        "edge_saber",
        "broken_saber",
        "nonexistent_xyz",
        "",
    ];

    // ---- strap-seam mock engine (configstrings) ----------------------------
    // `BG_SoundIndex` -> `G_SoundIndex` is the real configstring registration
    // (`G_FindConfigstringIndex` over `CS_SOUNDS`), reached ctx-free through the
    // `g_strap` seam cell. Serve it with a real engine stand-in: the C-variadic
    // inbound trampoline (`game_syscall_trampoline` armed via `arm_game_slot` —
    // the jampgame smoke-test precedent, crates/jampgame/tests/common/mod.rs)
    // dispatching to a configstring table that persists across all six parses,
    // like a real engine's per-level table. Distinct names mint 1,2,3…; repeats
    // dedup. The oracle dumper's G_SoundIndex stub mirrors these semantics.
    thread_local! {
        static CONFIGSTRINGS: RefCell<BTreeMap<isize, Vec<u8>>> =
            const { RefCell::new(BTreeMap::new()) };
    }

    /// Fixed-arity dispatch target behind the trampoline: services
    /// `G_GET_CONFIGSTRING`/`G_SET_CONFIGSTRING` only; any other import on the
    /// saber-load path is a test bug and panics loudly.
    extern "C-unwind" fn cs_syscall(_ctx: *mut c_void, args: *const isize) -> isize {
        const GET: isize = MpGameImport::G_GET_CONFIGSTRING as isize;
        const SET: isize = MpGameImport::G_SET_CONFIGSTRING as isize;
        // SAFETY: `args` is the trampoline's 16-word frame (`args[0]` = import
        // number, the rest the syscall's words in encode order).
        unsafe {
            match *args {
                GET => {
                    // ( int num, char *buffer, int bufferSize ) — the stored
                    // string ("" when unset), NUL-terminated, truncated.
                    let (num, buf, size) = (*args.add(1), *args.add(2) as *mut u8, *args.add(3));
                    let s =
                        CONFIGSTRINGS.with(|cs| cs.borrow().get(&num).cloned().unwrap_or_default());
                    if !buf.is_null() && size > 0 {
                        let n = s.len().min(size as usize - 1);
                        core::ptr::copy_nonoverlapping(s.as_ptr(), buf, n);
                        *buf.add(n) = 0;
                    }
                    0
                }
                SET => {
                    // ( int num, const char *string )
                    let (num, ptr) = (*args.add(1), *args.add(2) as *const c_char);
                    let s = if ptr.is_null() {
                        Vec::new()
                    } else {
                        CStr::from_ptr(ptr).to_bytes().to_vec()
                    };
                    CONFIGSTRINGS.with(|cs| {
                        cs.borrow_mut().insert(num, s);
                    });
                    0
                }
                other => panic!("saberload mock engine: unmodeled syscall #{other}"),
            }
        }
    }

    /// A fixtures-backed `BgTraps`: the FS calls read `<dir>/sabers/*`, and
    /// `r_register_skin` mints a per-saber counter with a name log. Every other
    /// trait method is unreachable in the saber-load path.
    struct TestTraps {
        dir: PathBuf, // fixtures dir; sabers live in <dir>/sabers
        files: RefCell<Vec<Option<(Vec<u8>, usize)>>>, // handle -> (bytes, pos); index 0 unused
        skin_ctr: Cell<c_int>,
        skin_log: RefCell<Vec<(c_int, String)>>,
    }

    impl TestTraps {
        fn new(dir: PathBuf) -> Self {
            Self {
                dir,
                files: RefCell::new(vec![None]), // reserve handle 0
                skin_ctr: Cell::new(0),
                skin_log: RefCell::new(Vec::new()),
            }
        }
        // Reset the per-saber registration observation state.
        fn reset_regs(&self) {
            self.skin_ctr.set(0);
            self.skin_log.borrow_mut().clear();
        }
        fn take_skins(&self) -> Vec<(c_int, String)> {
            std::mem::take(&mut *self.skin_log.borrow_mut())
        }
        // Map a Raven vpath ("ext_data/sabers[/name]") onto the fixtures tree.
        fn mappath(&self, vpath: &str) -> PathBuf {
            let rel = vpath.strip_prefix("ext_data/").unwrap_or(vpath);
            self.dir.join(rel)
        }
    }

    impl BgTraps for TestTraps {
        fn fs_getfilelist(
            &self,
            path: *const c_char,
            extension: *const c_char,
            listbuf: *mut c_char,
            bufsize: c_int,
        ) -> c_int {
            let dir = self.mappath(&cstr_to_str_p(path));
            let ext = cstr_to_str_p(extension);
            let mut names: Vec<String> = match std::fs::read_dir(&dir) {
                Ok(rd) => rd
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| n.ends_with(&ext))
                    .collect(),
                Err(_) => return 0,
            };
            names.sort(); // byte-lexicographic, matches the C dumper's qsort/strcmp
            let n = names.len() as c_int;
            let mut off = 0usize;
            for nm in &names {
                let b = nm.as_bytes();
                if off + b.len() + 1 > bufsize as usize {
                    break;
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(b.as_ptr(), listbuf.add(off) as *mut u8, b.len());
                    *listbuf.add(off + b.len()) = 0;
                }
                off += b.len() + 1;
            }
            n
        }

        fn fs_fopen(&self, qpath: *const c_char, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
            let _ = mode;
            let real = self.mappath(&cstr_to_str_p(qpath));
            match std::fs::read(&real) {
                Ok(bytes) => {
                    let len = bytes.len() as c_int;
                    let mut files = self.files.borrow_mut();
                    let h = files.len() as fileHandle_t;
                    files.push(Some((bytes, 0)));
                    unsafe {
                        if !f.is_null() {
                            *f = h;
                        }
                    }
                    len
                }
                Err(_) => {
                    unsafe {
                        if !f.is_null() {
                            *f = 0;
                        }
                    }
                    -1
                }
            }
        }

        fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
            let mut files = self.files.borrow_mut();
            let idx = f as usize;
            if idx == 0 || idx >= files.len() {
                return;
            }
            if let Some((bytes, pos)) = files[idx].as_mut() {
                let want = len as usize;
                let avail = bytes.len().saturating_sub(*pos);
                let n = want.min(avail);
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes[*pos..].as_ptr(), buffer as *mut u8, n);
                }
                *pos += n;
            }
        }

        fn fs_write(&self, _buffer: *const c_void, _len: c_int, _f: fileHandle_t) {}

        fn fs_fclose(&self, f: fileHandle_t) {
            let mut files = self.files.borrow_mut();
            let idx = f as usize;
            if idx != 0 && idx < files.len() {
                files[idx] = None;
            }
        }

        fn r_register_skin(&self, name: *const c_char) -> qhandle_t {
            let id = self.skin_ctr.get() + 1;
            self.skin_ctr.set(id);
            self.skin_log.borrow_mut().push((id, cstr_to_str_p(name)));
            id
        }

        // --- unreachable in the saber-load path ---
        fn trace(
            &self,
            _r: *mut trace_t,
            _s: *const vec3_t,
            _mn: *const vec3_t,
            _mx: *const vec3_t,
            _e: *const vec3_t,
            _p: c_int,
            _c: c_int,
        ) {
            unreachable!()
        }
        fn pointcontents(&self, _p: *const vec3_t, _e: c_int) -> c_int {
            unreachable!()
        }
        fn g2api_init_ghoul2_model(
            &self,
            _a: *mut *mut c_void,
            _b: *const c_char,
            _c: c_int,
            _d: qhandle_t,
            _e: qhandle_t,
            _f: c_int,
            _g: c_int,
        ) -> c_int {
            unreachable!()
        }
        fn g2api_clean_ghoul2_models(&self, _a: *mut *mut c_void) {
            unreachable!()
        }
        fn g2api_add_bolt(&self, _a: *mut c_void, _b: c_int, _c: *const c_char) -> c_int {
            unreachable!()
        }
        fn g2api_get_bolt_matrix(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: c_int,
            _d: *mut mdxaBone_t,
            _e: *const vec3_t,
            _f: *const vec3_t,
            _g: c_int,
            _h: *mut qhandle_t,
            _i: *const vec3_t,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_get_bolt_matrix_no_reconstruct(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: c_int,
            _d: *mut mdxaBone_t,
            _e: *const vec3_t,
            _f: *const vec3_t,
            _g: c_int,
            _h: *mut qhandle_t,
            _i: *const vec3_t,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_get_bolt_matrix_no_rec_no_rot(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: c_int,
            _d: *mut mdxaBone_t,
            _e: *const vec3_t,
            _f: *const vec3_t,
            _g: c_int,
            _h: *mut qhandle_t,
            _i: *const vec3_t,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_set_bone_angles(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *const c_char,
            _d: *const vec3_t,
            _e: c_int,
            _f: c_int,
            _g: c_int,
            _h: c_int,
            _i: *mut qhandle_t,
            _j: c_int,
            _k: c_int,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_set_bone_anim(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *const c_char,
            _d: c_int,
            _e: c_int,
            _f: c_int,
            _g: f32,
            _h: c_int,
            _i: f32,
            _j: c_int,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_get_bone_anim(
            &self,
            _a: *mut c_void,
            _b: *const c_char,
            _c: c_int,
            _d: *mut f32,
            _e: *mut c_int,
            _f: *mut c_int,
            _g: *mut c_int,
            _h: *mut f32,
            _i: *mut c_int,
            _j: c_int,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_set_rag_doll(&self, _a: *mut c_void, _b: *mut sharedRagDollParams_t) {
            unreachable!()
        }
        fn g2api_animate_g2_models(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *mut sharedRagDollUpdateParams_t,
        ) {
            unreachable!()
        }
        fn g2api_set_bone_ik_state(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *const c_char,
            _d: c_int,
            _e: *mut sharedSetBoneIKStateParams_t,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_ik_move(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *mut sharedIKMoveParams_t,
        ) -> qboolean {
            unreachable!()
        }
        fn g2api_get_surface_render_status(
            &self,
            _a: *mut c_void,
            _b: c_int,
            _c: *const c_char,
        ) -> c_int {
            unreachable!()
        }
        fn fx_play_effect_id(
            &self,
            _a: c_int,
            _b: *const vec3_t,
            _c: *const vec3_t,
            _d: c_int,
            _e: c_int,
        ) {
            unreachable!()
        }
        fn snap_vector(&self, _v: *mut f32) {
            unreachable!()
        }
        fn cvar_register(
            &self,
            _a: *mut vmCvar_t,
            _b: *const c_char,
            _c: *const c_char,
            _d: c_int,
        ) {
            unreachable!()
        }
    }

    fn cstr_to_str_p(p: *const c_char) -> String {
        unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
    }

    // Read a fixed C-char array field as a displayable string (up to NUL).
    fn field_str(buf: &[c_char]) -> String {
        let bytes: Vec<u8> = buf
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[test]
    fn saberload_parity() {
        // Arm the `g_strap` seam (the GAME_INIT stand-in) BEFORE any parse —
        // `WP_SaberSetDefaults` registers sounds immediately. The `Engine` is a
        // process-static because the seam cell keeps a raw pointer to it.
        arm_game_slot(core::ptr::null_mut(), cs_syscall);
        static ENGINE: OnceLock<Engine> = OnceLock::new();
        init_strap_engine(
            ENGINE.get_or_init(|| Engine::new(game_syscall_trampoline as *const c_void)),
        );
        // STAGE-2a: the ctx-less boundary fns now reborrow a REAL world from the
        // strap cell (a &mut field can't be null) — arm it with an owned zeroed
        // world; the tested path never touches it (same contract as before).
        let w: &'static mut mp_game::world::GameWorld =
            Box::leak(mp_game::world::GameWorld::zeroed_boxed());
        let w_ptr = w as *mut mp_game::world::GameWorld;
        mp_game::g_strap::init_strap_world(w_ptr);

        let dir = oracle_dir().join("fixtures");
        let traps = TestTraps::new(dir);
        let mut bg = BgState::new();

        saber_snd_tape_enable();
        WP_SaberLoadParms(&mut bg, &traps);

        // `BG_SoundIndex` now routes through `GameCallbacks::sound_index`, whose
        // game impl calls the same ctx-free `G_SoundIndex` (world unused on this
        // path); build the handle over the strap world + engine.
        let mut callbacks = GameCallbacksImpl {
            world: w_ptr,
            engine: ENGINE.get().unwrap(),
        };

        let mut o = String::new();
        o.push_str("== saberload ==\n");

        for name in NAMES {
            let _ = saber_snd_tape_drain(); // clear any leftover
            traps.reset_regs();

            let mut saber: saberInfo_t = unsafe { core::mem::zeroed() };
            let cname = cstr(name);
            let ret = WP_SaberParseParms(cname.as_ptr(), &mut saber, &mut bg, &traps, &mut callbacks);

            let pi = |o: &mut String, tag: &str, v: c_int| {
                let _ = writeln!(o, "{tag} {v}");
            };
            let pfh = |o: &mut String, tag: &str, v: f32| {
                let _ = writeln!(o, "{tag} {:08x}", v.cbits());
            };
            let qstr = |o: &mut String, tag: &str, s: &str| {
                let _ = writeln!(o, "{tag} \"{s}\"");
            };

            let _ = writeln!(o, "saber \"{name}\"");
            pi(&mut o, "ret", if ret != qfalse { 1 } else { 0 });
            qstr(&mut o, "name", &field_str(&saber.name));
            qstr(&mut o, "fullName", &field_str(&saber.fullName));
            pi(&mut o, "type", saber.r#type as c_int);
            qstr(&mut o, "model", &field_str(&saber.model));
            pi(&mut o, "skin", saber.skin);
            pi(&mut o, "soundOn", saber.soundOn);
            pi(&mut o, "soundLoop", saber.soundLoop);
            pi(&mut o, "soundOff", saber.soundOff);
            pi(&mut o, "numBlades", saber.numBlades);
            for i in 0..MAX_BLADES {
                let b = &saber.blade[i];
                let _ = writeln!(
                    o,
                    "blade{i} color {} radius {:08x} lengthMax {:08x}",
                    b.color as c_int,
                    b.radius.cbits(),
                    b.lengthMax.cbits()
                );
            }
            pi(&mut o, "stylesLearned", saber.stylesLearned);
            pi(&mut o, "stylesForbidden", saber.stylesForbidden);
            pi(&mut o, "maxChain", saber.maxChain);
            pi(&mut o, "forceRestrictions", saber.forceRestrictions);
            pi(&mut o, "lockBonus", saber.lockBonus);
            pi(&mut o, "parryBonus", saber.parryBonus);
            pi(&mut o, "breakParryBonus", saber.breakParryBonus);
            pi(&mut o, "breakParryBonus2", saber.breakParryBonus2);
            pi(&mut o, "disarmBonus", saber.disarmBonus);
            pi(&mut o, "disarmBonus2", saber.disarmBonus2);
            pi(&mut o, "singleBladeStyle", saber.singleBladeStyle as c_int);
            pi(&mut o, "saberFlags", saber.saberFlags);
            pi(&mut o, "saberFlags2", saber.saberFlags2);
            pi(&mut o, "spinSound", saber.spinSound);
            let _ = writeln!(
                o,
                "swingSound {} {} {}",
                saber.swingSound[0], saber.swingSound[1], saber.swingSound[2]
            );
            pfh(&mut o, "moveSpeedScale", saber.moveSpeedScale);
            pfh(&mut o, "animSpeedScale", saber.animSpeedScale);
            pi(&mut o, "kataMove", saber.kataMove);
            pi(&mut o, "lungeAtkMove", saber.lungeAtkMove);
            pi(&mut o, "jumpAtkUpMove", saber.jumpAtkUpMove);
            pi(&mut o, "jumpAtkFwdMove", saber.jumpAtkFwdMove);
            pi(&mut o, "jumpAtkBackMove", saber.jumpAtkBackMove);
            pi(&mut o, "jumpAtkRightMove", saber.jumpAtkRightMove);
            pi(&mut o, "jumpAtkLeftMove", saber.jumpAtkLeftMove);
            pi(&mut o, "readyAnim", saber.readyAnim);
            pi(&mut o, "drawAnim", saber.drawAnim);
            pi(&mut o, "putawayAnim", saber.putawayAnim);
            pi(&mut o, "tauntAnim", saber.tauntAnim);
            pi(&mut o, "bowAnim", saber.bowAnim);
            pi(&mut o, "meditateAnim", saber.meditateAnim);
            pi(&mut o, "flourishAnim", saber.flourishAnim);
            pi(&mut o, "gloatAnim", saber.gloatAnim);
            pi(&mut o, "bladeStyle2Start", saber.bladeStyle2Start);
            pi(&mut o, "trailStyle", saber.trailStyle);
            pi(&mut o, "g2MarksShader", saber.g2MarksShader);
            pi(&mut o, "g2WeaponMarkShader", saber.g2WeaponMarkShader);
            let _ = writeln!(
                o,
                "hitSound {} {} {}",
                saber.hitSound[0], saber.hitSound[1], saber.hitSound[2]
            );
            let _ = writeln!(
                o,
                "blockSound {} {} {}",
                saber.blockSound[0], saber.blockSound[1], saber.blockSound[2]
            );
            let _ = writeln!(
                o,
                "bounceSound {} {} {}",
                saber.bounceSound[0], saber.bounceSound[1], saber.bounceSound[2]
            );
            pi(&mut o, "blockEffect", saber.blockEffect);
            pi(&mut o, "hitPersonEffect", saber.hitPersonEffect);
            pi(&mut o, "hitOtherEffect", saber.hitOtherEffect);
            pi(&mut o, "bladeEffect", saber.bladeEffect);
            pfh(&mut o, "knockbackScale", saber.knockbackScale);
            pfh(&mut o, "damageScale", saber.damageScale);
            pfh(&mut o, "splashRadius", saber.splashRadius);
            pi(&mut o, "splashDamage", saber.splashDamage);
            pfh(&mut o, "splashKnockback", saber.splashKnockback);
            pi(&mut o, "trailStyle2", saber.trailStyle2);
            pi(&mut o, "g2MarksShader2", saber.g2MarksShader2);
            pi(&mut o, "g2WeaponMarkShader2", saber.g2WeaponMarkShader2);
            let _ = writeln!(
                o,
                "hit2Sound {} {} {}",
                saber.hit2Sound[0], saber.hit2Sound[1], saber.hit2Sound[2]
            );
            let _ = writeln!(
                o,
                "block2Sound {} {} {}",
                saber.block2Sound[0], saber.block2Sound[1], saber.block2Sound[2]
            );
            let _ = writeln!(
                o,
                "bounce2Sound {} {} {}",
                saber.bounce2Sound[0], saber.bounce2Sound[1], saber.bounce2Sound[2]
            );
            pi(&mut o, "blockEffect2", saber.blockEffect2);
            pi(&mut o, "hitPersonEffect2", saber.hitPersonEffect2);
            pi(&mut o, "hitOtherEffect2", saber.hitOtherEffect2);
            pi(&mut o, "bladeEffect2", saber.bladeEffect2);
            pfh(&mut o, "knockbackScale2", saber.knockbackScale2);
            pfh(&mut o, "damageScale2", saber.damageScale2);
            pfh(&mut o, "splashRadius2", saber.splashRadius2);
            pi(&mut o, "splashDamage2", saber.splashDamage2);
            pfh(&mut o, "splashKnockback2", saber.splashKnockback2);

            let sounds = saber_snd_tape_drain();
            for (id, nm) in traps.take_skins() {
                let _ = writeln!(o, "regskin {id} \"{nm}\"");
            }
            for nm in sounds {
                let _ = writeln!(o, "regsound \"{nm}\"");
            }
            o.push_str("--\n");
        }

        o.push_str("== end ==\n");
        compare("saberload", &o);
    }
}
