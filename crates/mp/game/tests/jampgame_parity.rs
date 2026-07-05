//! Differential parity test for the jampgame `q_math` + `bg_lib` ports against
//! the Raven oracle. Reproduces `tools/jampgame-oracle/`'s canonical bit-exact
//! dumps by calling the PORTED functions (`mp_game::q_math`,
//! `mp_game::bg_lib`, `mp_game::bg_channel::Rng`) and byte-compares to the
//! committed goldens under `tools/jampgame-oracle/golden/`.
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../tools/jampgame-oracle")
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
fn v3(o: &mut String, v: &[f32; 3]) {
    let _ = writeln!(o, "{:08x} {:08x} {:08x}", v[0].to_bits(), v[1].to_bits(), v[2].to_bits());
}
fn v4(o: &mut String, v: &[f32; 4]) {
    let _ = writeln!(
        o,
        "{:08x} {:08x} {:08x} {:08x}",
        v[0].to_bits(), v[1].to_bits(), v[2].to_bits(), v[3].to_bits()
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
            out.push([f32::from_bits(w[0]), f32::from_bits(w[1]), f32::from_bits(w[2])]);
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
            let _ = writeln!(o, "fl {:08x}", rng.flrand(0.0, 1.0).to_bits());
            let _ = writeln!(o, "fl {:08x}", rng.flrand(-1.0, 1.0).to_bits());
            let _ = writeln!(o, "fl {:08x}", rng.flrand(-100.0, 100.0).to_bits());
            let _ = writeln!(o, "qf {:08x}", rng.Q_flrand(0.0, 1000.0).to_bits());
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
        let _ = writeln!(o, "qrn {:08x}", Q_random(&mut seed as *mut c_int).to_bits());
    }
    seed = 12345;
    for _ in 0..30 {
        let _ = writeln!(o, "qcr {:08x}", Q_crandom(&mut seed as *mut c_int).to_bits());
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
        let _ = writeln!(o, "cc {} cs {} log2 {}", ClampChar(x) as i32, ClampShort(x) as i32, Q_log2(ax));
    }
    let cf: [u32; 10] = [
        0x3f800000, 0x40490fdb, 0x00000001, 0x80000000, 0x7f7fffff, 0x3dcccccd, 0xc0000000,
        0x42c80000, 0x00800000, 0xbf000000,
    ];
    for u in cf {
        let v = f32::from_bits(u);
        let a = if v < 0.0 { -v } else { v };
        let _ = writeln!(o, "rsqrt {:08x} fabs {:08x}", Q_rsqrt(a).to_bits(), Q_fabs(v).to_bits());
    }
    for y in 1..=6 {
        let _ = writeln!(o, "powf {:08x}", powf(1.5, y).to_bits());
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
        let (r, g, bl) = (f32::from_bits(c[0]), f32::from_bits(c[1]), f32::from_bits(c[2]));
        let _ = writeln!(o, "cb3 {:08x} cb4 {:08x}", ColorBytes3(r, g, bl), ColorBytes4(r, g, bl, 0.5));
        let inv = [r, g, bl];
        let mut out = [0.0; 3];
        let m = NormalizeColor(inv, &mut out);
        let _ = write!(o, "ncol {:08x} ", m.to_bits());
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
        let mut n = [f32::from_bits(nn[0]), f32::from_bits(nn[1]), f32::from_bits(nn[2])];
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
                f[0].to_bits(), f[1].to_bits(), f[2].to_bits(),
                r[0].to_bits(), r[1].to_bits(), r[2].to_bits(),
                up[0].to_bits(), up[1].to_bits(), up[2].to_bits()
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
            let _ = write!(o, "vn {:08x} ", len.to_bits());
            v3(o, &t);
        }
        {
            let mut t = [0.0; 3];
            let len = VectorNormalize2(a, &mut t);
            let _ = write!(o, "vn2 {:08x} ", len.to_bits());
            v3(o, &t);
        }
        let _ = writeln!(o, "vl {:08x} vls {:08x}", VectorLength(a).to_bits(), VectorLengthSquared(a).to_bits());
        let _ = writeln!(o, "dist {:08x} dsq {:08x}", Distance(a, b).to_bits(), DistanceSquared(a, b).to_bits());
        let _ = writeln!(o, "dh {:08x} dhs {:08x}", DistanceHorizontal(a, b).to_bits(), DistanceHorizontalSquared(a, b).to_bits());
        let _ = writeln!(o, "vcmp {} {}", VectorCompare(a, b), VectorCompare(a, a));
        {
            let mut t = [0.0; 3];
            CrossProduct(a, b, &mut t);
            let _ = write!(o, "cross ");
            v3(o, &t);
        }
        let _ = writeln!(o, "dot {:08x} _dot {:08x}", _DotProduct(a, b).to_bits(), _DotProduct(a, b).to_bits());
        let _ = writeln!(o, "dpn {:08x}", DotProductNormalize(a, b).to_bits());
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
                t[0].to_bits(), t[1].to_bits(), t[2].to_bits(),
                u[0].to_bits(), u[1].to_bits(), u[2].to_bits()
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
        let _ = writeln!(o, "angsub {:08x}", AngleSubtract(a[0], a[1]).to_bits());
        {
            let mut t = [0.0; 3];
            AnglesSubtract(a, b, &mut t);
            let _ = write!(o, "angssub ");
            v3(o, &t);
        }
        let _ = writeln!(o, "lerp {:08x}", LerpAngle(a[0], a[1], a[2]).to_bits());
        let _ = writeln!(
            o,
            "an360 {:08x} an180 {:08x} amod {:08x} adel {:08x}",
            AngleNormalize360(a[0]).to_bits(),
            AngleNormalize180(a[0]).to_bits(),
            AngleMod(a[0]).to_bits(),
            AngleDelta(a[0], a[1]).to_bits()
        );
        let _ = writeln!(o, "rfb {:08x}", RadiusFromBounds(a, b).to_bits());
        let _ = writeln!(o, "d2b {}", DirToByte(a));
        {
            let mut res = [0.0; 3];
            let ok = G_FindClosestPointOnLineSegment(a, b, c, &mut res);
            let _ = write!(o, "gclose {} ", ok);
            v3(o, &res);
        }
        let _ = writeln!(o, "gdist {:08x}", G_PointDistFromLineSegment(a, b, c).to_bits());
        {
            let mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];
            AnglesToAxis(a, axis.as_mut_ptr());
            let _ = writeln!(
                o,
                "a2a {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                axis[0][0].to_bits(), axis[0][1].to_bits(), axis[0][2].to_bits(),
                axis[1][0].to_bits(), axis[1][1].to_bits(), axis[1][2].to_bits(),
                axis[2][0].to_bits(), axis[2][1].to_bits(), axis[2][2].to_bits()
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
            let _ = writeln!(o, "rad {:08x} {:08x} {:08x}", axis[1][0].to_bits(), axis[1][1].to_bits(), axis[1][2].to_bits());
        }
        {
            let mut ax: [[f32; 3]; 3] = [[0.0; 3]; 3];
            AxisClear(ax.as_mut_ptr());
            let _ = writeln!(o, "aclear {:08x} {:08x} {:08x}", ax[0][0].to_bits(), ax[1][1].to_bits(), ax[2][2].to_bits());
        }
        {
            let m1 = [[a[0], a[1], a[2]], [b[0], b[1], b[2]], [c[0], c[1], c[2]]];
            let m2 = [[c[0], b[1], a[2]], [a[0], c[1], b[2]], [b[0], a[1], c[2]]];
            let mut ou = [[0.0; 3]; 3];
            MatrixMultiply(&m1, &m2, &mut ou);
            let _ = writeln!(o, "mm {:08x} {:08x} {:08x}", ou[0][0].to_bits(), ou[1][1].to_bits(), ou[2][2].to_bits());
        }
        {
            let (mut mn, mut mx) = ([0.0; 3], [0.0; 3]);
            ClearBounds(&mut mn, &mut mx);
            AddPointToBounds(a, &mut mn, &mut mx);
            AddPointToBounds(b, &mut mn, &mut mx);
            let _ = writeln!(
                o,
                "bounds {:08x} {:08x} {:08x} {:08x} {:08x} {:08x}",
                mn[0].to_bits(), mn[1].to_bits(), mn[2].to_bits(),
                mx[0].to_bits(), mx[1].to_bits(), mx[2].to_bits()
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

fn dump_rand_bg(o: &mut String) {
    o.push_str("== rand ==\n");
    let seeds: [u32; 5] = [0, 1, 12345, 0x7fffffff, 0xdeadbeef];
    let mut rng = Rng::new();
    for s in seeds {
        let _ = writeln!(o, "seed {:08x}", s);
        rng.srand(s);
        for _ in 0..64 {
            let _ = writeln!(o, "r {}", rng.rand());
        }
    }
}

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
        let _ = writeln!(o, "a {} atof {:016x}", idx, atof(p0).to_bits());
        let mut sp: *const c_char = cbuf.as_ptr() as *const c_char;
        let v = _atof(&mut sp as *mut *const c_char);
        let adv = (sp as usize) - (cbuf.as_ptr() as usize);
        let _ = writeln!(o, "a {} _atof {:016x} adv {}", idx, v.to_bits(), adv);
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
    qsort(arr.as_mut_ptr() as *mut c_void, n, core::mem::size_of::<c_int>(), f as usize as *mut c_void);
    let _ = writeln!(o, "ints {}", n);
    for v in &arr {
        let _ = writeln!(o, "{}", v);
    }

    let mut kv: Vec<Kv> = [
        (5, 0), (3, 1), (5, 2), (1, 3), (3, 4), (9, 5), (1, 6), (7, 7), (3, 8), (5, 9),
        (0, 10), (8, 11), (2, 12), (8, 13), (4, 14), (6, 15), (3, 16), (9, 17), (1, 18), (5, 19),
    ]
    .iter()
    .map(|&(key, payload)| Kv { key, payload })
    .collect();
    let kn = kv.len();
    let fk = cmp_kv as extern "C" fn(*const c_void, *const c_void) -> c_int;
    qsort(kv.as_mut_ptr() as *mut c_void, kn, core::mem::size_of::<Kv>(), fk as usize as *mut c_void);
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
    dump_rand_bg(&mut o);
    dump_atox(&mut o);
    dump_qsort(&mut o);
    dump_memmove(&mut o);
    o.push_str("== end ==\n");
    compare("bglib", &o);
}
