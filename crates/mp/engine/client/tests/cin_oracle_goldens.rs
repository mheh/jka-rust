//! The Rust half of the `tools/cin-oracle` byte gate (DEC-55.3, wayfinder
//! ticket gh#28).
//!
//! This drives the ported RoQ decode core over the same synthetic streams
//! `tools/cin-oracle/main.cpp` drives the unmodified `cl_cin.cpp` over, emits
//! the identical text, and asserts it equals the committed golden. The goldens
//! are text, so the test needs no C++ toolchain.
//!
//! The two drivers stay symmetric by construction: `RoQInterrupt` is outside the
//! byte gate, so both replicate its chunk-dispatch switch instead of calling it.
//! `readQuadInfo` is the one gated function this side cannot call, because it
//! takes a `&mut Common` that no test can build. The driver writes the same
//! fields and `tools/cin-oracle/README.md` records the divergence.
//!
//! Source: `oracle/codemp/client/cl_cin.cpp`

#![allow(non_snake_case)]

use std::ffi::{c_int, c_long, c_uint, c_ushort};
use std::fmt::Write;
use std::path::PathBuf;

use mp_engine_client::cin::cin_consts::{
    DEFAULT_CIN_HEIGHT, DEFAULT_CIN_WIDTH, ROQ_CODEBOOK, ROQ_QUAD_INFO, ROQ_QUAD_VQ,
    ZA_SOUND_MONO, ZA_SOUND_STEREO,
};
use mp_engine_client::cin::vq_blitter::VqBlitter;
use mp_engine_client::cl_cin::{
    blitVQQuad32fs, decodeCodeBook, initRoQ, setupQuad, yuv_to_rgb24, RllDecodeMonoToMono,
    RllDecodeMonoToStereo, RllDecodeStereoToMono, RllDecodeStereoToStereo, RoQPrepMcomp,
};
use mp_engine_client::client_host::Client;

/// The scenario list, in `tools/cin-oracle/run.sh` order.
const SCENARIOS: [&str; 9] = [
    "quadinfo",
    "quadinfo_ragged",
    "codebook",
    "codebook_partial",
    "vq_frames",
    "vq_nonsquare",
    "sound_mono",
    "sound_stereo",
    "rll_direct",
];

/// Sample indices the raw-entry lines print. These mirror `main.cpp`'s tables.
const TAB_IDX: [usize; 8] = [0, 1, 64, 85, 128, 170, 254, 255];
const SQR_IDX: [usize; 8] = [0, 1, 64, 127, 128, 129, 192, 255];
const MCOMP_IDX: [usize; 8] = [0, 1, 8, 15, 16, 136, 240, 255];
const QUAD_IDX: [usize; 8] = [0, 1, 2, 3, 4, 5, 19, 20];

const YUV_GRID: [c_long; 6] = [0, 51, 102, 153, 204, 255];
const YUV_SHOW: [[c_long; 3]; 10] = [
    [0, 0, 0],
    [255, 255, 255],
    [0, 255, 0],
    [255, 0, 255],
    [128, 128, 128],
    [16, 128, 128],
    [235, 128, 128],
    [81, 90, 240],
    [145, 54, 34],
    [41, 240, 110],
];

const VQ2_IDX: [usize; 6] = [0, 1, 2, 3, 4, 1023];
const VQ4_IDX: [usize; 6] = [0, 1, 15, 16, 17, 4095];
const VQ8_IDX: [usize; 6] = [0, 1, 63, 64, 65, 16383];

const RLL_FLAGS: [c_ushort; 5] = [0x0000, 0x8000, 0x1234, 0xff00, 0x00ff];

/// The `short sbuf[32768]` scratch `RoQInterrupt` decodes audio into.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:925`
const SBUF_LEN: usize = 32768;

// --- digests ----------------------------------------------------------------

/// FNV-1a over value-derived little-endian bytes, so the digest never depends on
/// the host's word width or padding. `main.cpp` carries the same four helpers.
fn fnv_init() -> u32 {
    2166136261u32
}

fn fnv_byte(h: &mut u32, b: u8) {
    *h ^= b as u32;
    *h = h.wrapping_mul(16777619u32);
}

fn fnv_u32(h: &mut u32, v: u32) {
    fnv_byte(h, (v & 0xff) as u8);
    fnv_byte(h, ((v >> 8) & 0xff) as u8);
    fnv_byte(h, ((v >> 16) & 0xff) as u8);
    fnv_byte(h, ((v >> 24) & 0xff) as u8);
}

fn fnv_bytes(p: &[u8]) -> u32 {
    let mut h = fnv_init();
    for &b in p {
        fnv_byte(&mut h, b);
    }
    h
}

/// Raven's `long` tables. The dump narrows each entry to 32 bits, the width the
/// shipped build stored.
fn fnv_longs(p: &[c_long]) -> u32 {
    let mut h = fnv_init();
    for &v in p {
        fnv_u32(&mut h, v as i32 as u32);
    }
    h
}

fn fnv_shorts(p: &[i16]) -> u32 {
    let mut h = fnv_init();
    for &v in p {
        let u = v as u16;
        fnv_byte(&mut h, (u & 0xff) as u8);
        fnv_byte(&mut h, (u >> 8) as u8);
    }
    h
}

fn fnv_ints(p: &[i32]) -> u32 {
    let mut h = fnv_init();
    for &v in p {
        fnv_u32(&mut h, v as u32);
    }
    h
}

/// One codebook texel: the vq books are `unsigned short` arrays holding 32-bit
/// RGBA, low half first.
fn vq_texel(v: &[c_ushort], i: usize) -> u32 {
    ((v[i * 2 + 1] as u32) << 16) | (v[i * 2] as u32)
}

fn fnv_vq(v: &[c_ushort]) -> u32 {
    let mut h = fnv_init();
    for i in 0..v.len() / 2 {
        fnv_u32(&mut h, vq_texel(v, i));
    }
    h
}

// --- the driver -------------------------------------------------------------

/// One scenario run. Holds the decoder plus the driver-side stream cursor that
/// `RoQInterrupt`'s loop control would own.
struct CinDriver {
    cl: Box<Client>,
    out: String,
    sbuf: Vec<i16>,
    frame_index: i32,
}

impl CinDriver {
    /// Puts the decoder at the state `CIN_PlayCinematic` leaves it in, minus the
    /// file system.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:1259-1293`
    fn new() -> Self {
        let mut cl = Box::new(Client::default());
        // A card that reports 2048 keeps `readQuadInfo` off the Rage Pro clamp,
        // so its `Com_Printf` never runs and the driver needs no `Common`.
        cl.cls.glconfig.maxTextureSize = 2048;
        cl.currentHandle = 0;
        cl.cinTable[0].CIN_WIDTH = DEFAULT_CIN_WIDTH as c_int;
        cl.cinTable[0].CIN_HEIGHT = DEFAULT_CIN_HEIGHT as c_int;
        cl.cinTable[0].playonwalls = 1;
        initRoQ(&mut cl);

        CinDriver {
            cl,
            out: String::new(),
            sbuf: vec![0i16; SBUF_LEN],
            frame_index: 0,
        }
    }

    fn dump_tables(&mut self) {
        let cl = &mut self.cl;
        let o = &mut self.out;
        let _ = writeln!(o, "TABLES");
        let _ = writeln!(o, "  yy crc {:08x}", fnv_longs(&cl.ROQ_YY_tab[..]));
        let _ = writeln!(o, "  ub crc {:08x}", fnv_longs(&cl.ROQ_UB_tab[..]));
        let _ = writeln!(o, "  ug crc {:08x}", fnv_longs(&cl.ROQ_UG_tab[..]));
        let _ = writeln!(o, "  vg crc {:08x}", fnv_longs(&cl.ROQ_VG_tab[..]));
        let _ = writeln!(o, "  vr crc {:08x}", fnv_longs(&cl.ROQ_VR_tab[..]));

        let tabs: [&[c_long]; 5] = [
            &cl.ROQ_YY_tab[..],
            &cl.ROQ_UB_tab[..],
            &cl.ROQ_UG_tab[..],
            &cl.ROQ_VG_tab[..],
            &cl.ROQ_VR_tab[..],
        ];
        let names = ["yy", "ub", "ug", "vg", "vr"];
        for (t, name) in tabs.iter().zip(names.iter()) {
            let _ = write!(o, "  {}", name);
            for &k in TAB_IDX.iter() {
                let _ = write!(o, " {}", t[k] as i32);
            }
            let _ = writeln!(o);
        }

        let _ = writeln!(o, "SQR crc {:08x}", fnv_shorts(&cl.cin.sqrTable[..]));
        let _ = write!(o, "  sqr");
        for &k in SQR_IDX.iter() {
            let _ = write!(o, " {}", cl.cin.sqrTable[k] as i32);
        }
        let _ = writeln!(o);

        let mut h = fnv_init();
        for &a in YUV_GRID.iter() {
            for &b in YUV_GRID.iter() {
                for &c in YUV_GRID.iter() {
                    fnv_u32(&mut h, yuv_to_rgb24(cl, a, b, c));
                }
            }
        }
        let _ = writeln!(o, "YUV crc {:08x}", h);
        for s in YUV_SHOW.iter() {
            let _ = writeln!(
                o,
                "  yuv {} {} {} {:08x}",
                s[0] as i32,
                s[1] as i32,
                s[2] as i32,
                yuv_to_rgb24(cl, s[0], s[1], s[2])
            );
        }
    }

    /// Raven `readQuadInfo`'s field writes.
    ///
    /// The port's `readQuadInfo` takes a `&mut Common` for one `Com_Printf` on
    /// the `maxTextureSize <= 256` arm, and no test can build a `Common`. The
    /// driver holds `maxTextureSize` at 2048, so that arm never runs and the
    /// remaining body is these field writes.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:788-827`
    fn read_quad_info(&mut self, payload: &[u8]) {
        let cl = &mut self.cl;
        let h = 0usize;
        cl.cinTable[h].xsize = payload[0] as c_uint + payload[1] as c_uint * 256;
        cl.cinTable[h].ysize = payload[2] as c_uint + payload[3] as c_uint * 256;
        cl.cinTable[h].maxsize = payload[4] as c_uint + payload[5] as c_uint * 256;
        cl.cinTable[h].minsize = payload[6] as c_uint + payload[7] as c_uint * 256;

        cl.cinTable[h].CIN_HEIGHT = cl.cinTable[h].ysize as c_int;
        cl.cinTable[h].CIN_WIDTH = cl.cinTable[h].xsize as c_int;

        cl.cinTable[h].samplesPerLine = cl.cinTable[h].CIN_WIDTH as c_long * 4;
        cl.cinTable[h].screenDelta = cl.cinTable[h].CIN_HEIGHT as c_long * cl.cinTable[h].samplesPerLine;

        cl.cinTable[h].VQ0 = cl.cinTable[h].VQNormal;
        cl.cinTable[h].VQ1 = cl.cinTable[h].VQBuffer;

        let linbuf = cl.cin.linbuf.as_ptr() as c_long;
        cl.cinTable[h].t[0] = (0 - linbuf) + linbuf + cl.cinTable[h].screenDelta;
        cl.cinTable[h].t[1] = (0 - (linbuf + cl.cinTable[h].screenDelta)) + linbuf;

        cl.cinTable[h].drawX = cl.cinTable[h].CIN_WIDTH as c_long;
        cl.cinTable[h].drawY = cl.cinTable[h].CIN_HEIGHT as c_long;
    }

    fn dump_quadinfo(&mut self, tag: &str) {
        let c = &self.cl.cinTable[0];
        let o = &mut self.out;
        let _ = writeln!(o, "QUADINFO {}", tag);
        let _ = writeln!(
            o,
            "  xsize {} ysize {} maxsize {} minsize {}",
            c.xsize, c.ysize, c.maxsize, c.minsize
        );
        let _ = writeln!(
            o,
            "  cinw {} cinh {} spl {} screendelta {}",
            c.CIN_WIDTH, c.CIN_HEIGHT, c.samplesPerLine as i32, c.screenDelta as i32
        );
        // `t[0]` and `t[1]` are address algebra that cancels to +/- screenDelta.
        // The dump narrows both to 32 bits so the value cannot depend on the
        // compile width.
        let _ = writeln!(
            o,
            "  t0 {} t1 {} drawx {} drawy {} vq0 {} vq1 {}",
            c.t[0] as i32,
            c.t[1] as i32,
            c.drawX as i32,
            c.drawY as i32,
            (c.VQ0 != VqBlitter::None) as i32,
            (c.VQ1 != VqBlitter::None) as i32
        );
    }

    /// The number of `qStatus` cels `setupQuad` lays out for the current size.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:762-764`
    fn quad_cel_count(&self) -> c_long {
        let c = &self.cl.cinTable[0];
        let mut n = (c.xsize as c_long * c.ysize as c_long) / 16;
        n += n / 4;
        n += 64;
        n
    }

    /// A `qStatus` entry as a signed byte offset from `cin.linbuf`, never an
    /// address. The end-of-quad nulls dump as -1.
    fn quad_offset(&self, bank: usize, i: usize) -> i32 {
        let p = self.cl.cin.qStatus[bank][i];
        if p.is_null() {
            return -1;
        }
        // SAFETY: every non-null entry points inside `cin.linbuf`, so the
        // difference is the entry's byte offset into that array.
        (p as isize - self.cl.cin.linbuf.as_ptr() as isize) as i32
    }

    fn dump_quads(&mut self, tag: &str) {
        let cels = self.quad_cel_count();
        let onquad = self.cl.cinTable[0].onQuad;
        let mut banks: [Vec<i32>; 2] = [Vec::new(), Vec::new()];
        for (bank, slot) in banks.iter_mut().enumerate() {
            for i in 0..cels as usize {
                slot.push(self.quad_offset(bank, i));
            }
        }
        let heads: [[i32; 8]; 2] = [
            core::array::from_fn(|k| self.quad_offset(0, QUAD_IDX[k])),
            core::array::from_fn(|k| self.quad_offset(1, QUAD_IDX[k])),
        ];
        let last = onquad as usize;
        let ends = [
            self.quad_offset(0, last - 1),
            self.quad_offset(0, last),
            self.quad_offset(1, last - 1),
            self.quad_offset(1, last),
        ];

        let o = &mut self.out;
        let _ = writeln!(
            o,
            "QUADS {} onquad {} cels {}",
            tag, onquad as i32, cels as i32
        );
        for (bank, slot) in banks.iter().enumerate() {
            let _ = writeln!(o, "  q{} crc {:08x}", bank, fnv_ints(slot));
        }
        for (bank, head) in heads.iter().enumerate() {
            let _ = write!(o, "  q{}", bank);
            for v in head.iter() {
                let _ = write!(o, " {}", v);
            }
            let _ = writeln!(o);
        }
        let _ = writeln!(o, "  qend {} {} {} {}", ends[0], ends[1], ends[2], ends[3]);
    }

    fn dump_mcomp(&mut self, tag: &str) {
        // Raven stores a signed delta in an `unsigned int` and adds it to a
        // 32-bit `byte *`. The dump takes the 32-bit reinterpretation, which is
        // what the pointer arithmetic means.
        let vals: Vec<i32> = self.cl.cin.mcomp.iter().map(|&v| v as i32).collect();
        let crc = fnv_ints(&vals);
        let o = &mut self.out;
        let _ = writeln!(o, "MCOMP {} crc {:08x}", tag, crc);
        let _ = write!(o, "  mcomp");
        for &k in MCOMP_IDX.iter() {
            let _ = write!(o, " {}", vals[k]);
        }
        let _ = writeln!(o);
    }

    fn dump_codebook(&mut self, tag: &str, flags: c_ushort) {
        let (mut two, mut four): (c_long, c_long);
        if flags == 0 {
            two = 256;
            four = 256;
        } else {
            two = (flags >> 8) as c_long;
            if two == 0 {
                two = 256;
            }
            four = (flags & 0xff) as c_long;
        }
        four *= 2;

        let crc2 = fnv_vq(&self.cl.vq2[..]);
        let crc4 = fnv_vq(&self.cl.vq4[..]);
        let crc8 = fnv_vq(&self.cl.vq8[..]);
        let raw2: Vec<u32> = VQ2_IDX.iter().map(|&i| vq_texel(&self.cl.vq2[..], i)).collect();
        let raw4: Vec<u32> = VQ4_IDX.iter().map(|&i| vq_texel(&self.cl.vq4[..], i)).collect();
        let raw8: Vec<u32> = VQ8_IDX.iter().map(|&i| vq_texel(&self.cl.vq8[..], i)).collect();

        let o = &mut self.out;
        let _ = writeln!(
            o,
            "CODEBOOK {} flags {:04x} two {} four {}",
            tag, flags, two as i32, four as i32
        );
        let _ = writeln!(
            o,
            "  vq2 crc {:08x} vq4 crc {:08x} vq8 crc {:08x}",
            crc2, crc4, crc8
        );
        for (name, raw) in [("vq2", &raw2), ("vq4", &raw4), ("vq8", &raw8)] {
            let _ = write!(o, "  {}", name);
            for v in raw.iter() {
                let _ = write!(o, " {:08x}", v);
            }
            let _ = writeln!(o);
        }
    }

    fn dump_frame(&mut self, buf_off: usize) {
        let c = &self.cl.cinTable[0];
        let half = c.screenDelta as usize;
        let numquads = c.numQuads as i32;
        let roqf0 = c.roqF0 as i32;
        let roqf1 = c.roqF1 as i32;
        let nbuf0 = c.normalBuffer0 as i32;
        let xsize = c.xsize as c_long;
        let ysize = c.ysize as c_long;
        let spl = c.samplesPerLine as usize;

        let live = fnv_bytes(&self.cl.cin.linbuf[..half * 2]);
        let h0 = fnv_bytes(&self.cl.cin.linbuf[..half]);
        let h1 = fnv_bytes(&self.cl.cin.linbuf[half..half * 2]);

        // An 8x8 texel grid over the half this frame decoded into, so a mismatch
        // localises to a block instead of only tripping the whole-surface digest.
        let xstep = (xsize / 8).max(1) as usize;
        let ystep = (ysize / 8).max(1) as usize;
        let mut grid = [[0u32; 8]; 8];
        for (gy, row) in grid.iter_mut().enumerate() {
            for (gx, texel) in row.iter_mut().enumerate() {
                let at = buf_off + (gy * ystep) * spl + (gx * xstep) * 4;
                let p = &self.cl.cin.linbuf[at..at + 4];
                *texel = p[0] as u32 | (p[1] as u32) << 8 | (p[2] as u32) << 16 | (p[3] as u32) << 24;
            }
        }

        let index = self.frame_index;
        let o = &mut self.out;
        let _ = writeln!(
            o,
            "FRAME {} numquads {} roqf0 {} roqf1 {} nbuf0 {} bufhalf {}",
            index,
            numquads,
            roqf0,
            roqf1,
            nbuf0,
            (buf_off / half) as i32
        );
        let _ = writeln!(o, "  live crc {:08x}", live);
        let _ = writeln!(o, "  half0 crc {:08x} half1 crc {:08x}", h0, h1);
        for row in grid.iter() {
            let _ = write!(o, "  grid");
            for texel in row.iter() {
                let _ = write!(o, " {:08x}", texel);
            }
            let _ = writeln!(o);
        }
        self.frame_index += 1;
    }

    fn dump_audio(
        &mut self,
        mode: &str,
        size: c_uint,
        signed_output: i32,
        flag: c_ushort,
        ret: c_long,
        out_shorts: i32,
    ) {
        let crc = fnv_shorts(&self.sbuf[..out_shorts as usize]);
        let idx: [i32; 8] = [
            0,
            1,
            2,
            3,
            out_shorts / 2,
            out_shorts - 3,
            out_shorts - 2,
            out_shorts - 1,
        ];
        let picks: [i16; 8] = core::array::from_fn(|k| {
            let mut i = idx[k];
            if i < 0 {
                i = 0;
            }
            if i >= out_shorts {
                i = out_shorts - 1;
            }
            self.sbuf[i as usize]
        });

        let o = &mut self.out;
        let _ = writeln!(
            o,
            "AUDIO {} size {} signed {} flag {:04x} ret {} out {}",
            mode, size, signed_output, flag, ret as i32, out_shorts
        );
        let _ = writeln!(o, "  crc {:08x}", crc);
        let _ = write!(o, "  s");
        for v in picks.iter() {
            let _ = write!(o, " {}", *v as i32);
        }
        let _ = writeln!(o);
    }

    /// `RoQInterrupt`'s switch, minus the file I/O, the console, the loop
    /// control and the `S_RawSamples` hand-off. `main.cpp` runs the identical
    /// switch.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:949-1008`
    fn dispatch(&mut self) {
        let roq_id = self.cl.cinTable[0].roq_id;
        match roq_id {
            ROQ_QUAD_INFO => {
                if self.cl.cinTable[0].numQuads == -1 {
                    let payload: [u8; 8] = self.cl.cin.file[..8].try_into().unwrap();
                    self.read_quad_info(&payload);
                    self.dump_quadinfo("readquadinfo");
                    setupQuad(&mut self.cl, 0, 0);
                    self.dump_quads("setupquad");
                }
                if self.cl.cinTable[0].numQuads != 1 {
                    self.cl.cinTable[0].numQuads = 0;
                }
            }

            ROQ_CODEBOOK => {
                let flags = self.cl.cinTable[0].roq_flags as c_ushort;
                let input = self.cl.cin.file.as_mut_ptr();
                decodeCodeBook(&mut self.cl, input, flags);
                self.dump_codebook("decodecodebook", flags);
            }

            ROQ_QUAD_VQ => {
                let odd = (self.cl.cinTable[0].numQuads & 1) != 0;
                let bank = usize::from(odd);
                let (roqf0, roqf1) = (self.cl.cinTable[0].roqF0, self.cl.cinTable[0].roqF1);
                self.cl.cinTable[0].normalBuffer0 = self.cl.cinTable[0].t[bank];
                RoQPrepMcomp(&mut self.cl, roqf0, roqf1);
                self.dump_mcomp(if odd { "bank1" } else { "bank0" });

                let status = self.cl.cin.qStatus[bank].as_mut_ptr();
                let framedata = self.cl.cin.file.as_mut_ptr();
                blitVQQuad32fs(&mut self.cl, status, framedata);

                let half = self.cl.cinTable[0].screenDelta as usize;
                let buf_off = if odd { half } else { 0 };
                // SAFETY: `buf_off` is 0 or `screenDelta`, both inside `linbuf`.
                self.cl.cinTable[0].buf =
                    unsafe { self.cl.cin.linbuf.as_mut_ptr().add(buf_off) };

                if self.cl.cinTable[0].numQuads == 0 {
                    let n = (self.cl.cinTable[0].samplesPerLine
                        * self.cl.cinTable[0].ysize as c_long) as usize;
                    self.cl.cin.linbuf.copy_within(0..n, half);
                }
                self.cl.cinTable[0].numQuads += 1;
                self.dump_frame(buf_off);
            }

            ZA_SOUND_MONO => {
                let size = self.cl.cinTable[0].RoQFrameSize;
                let flags = self.cl.cinTable[0].roq_flags as c_ushort;
                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let from = self.cl.cin.file.as_mut_ptr();
                let to = self.sbuf.as_mut_ptr();
                let ssize = RllDecodeMonoToStereo(&mut self.cl, from, to, size, 0, flags);
                self.dump_audio("mono2stereo", size, 0, flags, ssize, size as i32 * 2);
            }

            ZA_SOUND_STEREO => {
                let size = self.cl.cinTable[0].RoQFrameSize;
                let flags = self.cl.cinTable[0].roq_flags as c_ushort;
                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let from = self.cl.cin.file.as_mut_ptr();
                let to = self.sbuf.as_mut_ptr();
                let ssize = RllDecodeStereoToStereo(&mut self.cl, from, to, size, 0, flags);
                self.dump_audio("stereo2stereo", size, 0, flags, ssize, size as i32);
            }

            _ => panic!("cin-oracle: the fixture holds unhandled chunk id {:04x}", roq_id),
        }
    }

    /// Walks one fixture stream. The header parse mirrors `RoQ_init`, and every
    /// later chunk header is parsed the way `RoQInterrupt`'s tail does, out of
    /// the eight bytes that trail the payload.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:1026-1030,1062-1083`
    fn run_stream(&mut self, name: &str, stream: &[u8]) {
        {
            let c = &mut self.cl.cinTable[0];
            c.roqFPS = stream[6] as c_long + stream[7] as c_long * 256;
            if c.roqFPS == 0 {
                c.roqFPS = 30;
            }
            c.numQuads = -1;
            c.roq_id = stream[8] as c_uint + stream[9] as c_uint * 256;
            c.RoQFrameSize = stream[10] as c_uint
                + stream[11] as c_uint * 256
                + stream[12] as c_uint * 65536;
            c.roq_flags = stream[14] as c_long + stream[15] as c_long * 256;
        }
        let _ = writeln!(
            self.out,
            "STREAM {} bytes {} fps {} roqid {:04x}",
            name,
            stream.len(),
            self.cl.cinTable[0].roqFPS as i32,
            self.cl.cinTable[0].roq_id
        );

        let mut pos = 16usize;
        let mut chunk = 0i32;
        loop {
            // `Sys_StreamedRead(cin.file, RoQFrameSize+8, 1, iFile)`: the payload
            // plus the next chunk header land in `cin.file` together.
            let size = self.cl.cinTable[0].RoQFrameSize as usize;
            let want = size + 8;
            assert!(
                pos + want <= stream.len(),
                "cin-oracle: {} runs past the end of the stream at chunk {}",
                name,
                chunk
            );
            self.cl.cin.file.iter_mut().for_each(|b| *b = 0);
            self.cl.cin.file[..want].copy_from_slice(&stream[pos..pos + want]);

            {
                let c = &self.cl.cinTable[0];
                let _ = writeln!(
                    self.out,
                    "CHUNK {} id {:04x} size {} flags {:04x} f0 {} f1 {}",
                    chunk,
                    c.roq_id,
                    c.RoQFrameSize,
                    c.roq_flags as c_ushort,
                    c.roqF0 as i32,
                    c.roqF1 as i32
                );
            }
            self.dispatch();

            pos += want;
            let hdr: [u8; 8] = self.cl.cin.file[size..size + 8].try_into().unwrap();
            let c = &mut self.cl.cinTable[0];
            c.roq_id = hdr[0] as c_uint + hdr[1] as c_uint * 256;
            c.RoQFrameSize =
                hdr[2] as c_uint + hdr[3] as c_uint * 256 + hdr[4] as c_uint * 65536;
            c.roq_flags = hdr[6] as c_long + hdr[7] as c_long * 256;
            c.roqF0 = hdr[7] as i8 as c_long;
            c.roqF1 = hdr[6] as i8 as c_long;
            chunk += 1;

            // The generator ends every fixture with an all-zero terminator chunk.
            if c.roq_id == 0 {
                break;
            }
        }
        let _ = writeln!(
            self.out,
            "STREAMEND chunks {} frames {}",
            chunk, self.frame_index
        );
    }

    /// The four `RllDecode*` entry points over a deterministic byte sweep.
    /// `RoQInterrupt` never reaches `RllDecodeMonoToMono` or
    /// `RllDecodeStereoToMono`, so this scenario is the only cover they get.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:184-305`
    fn run_rll(&mut self) {
        let mut from = [0u8; 1024];
        for (i, b) in from.iter_mut().enumerate() {
            *b = (i & 0xff) as u8;
        }
        let _ = writeln!(self.out, "RLL sweep crc {:08x}", fnv_bytes(&from));

        for &flag in RLL_FLAGS.iter() {
            for sgn in 0..2i32 {
                let signed_output = sgn as i8;
                let src = from.as_mut_ptr();

                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let to = self.sbuf.as_mut_ptr();
                let ret = RllDecodeMonoToMono(&mut self.cl, src, to, 256, signed_output, flag);
                self.dump_audio("mono2mono", 256, sgn, flag, ret, 256);

                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let to = self.sbuf.as_mut_ptr();
                let ret = RllDecodeMonoToStereo(&mut self.cl, src, to, 256, signed_output, flag);
                self.dump_audio("mono2stereo", 256, sgn, flag, ret, 512);

                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let to = self.sbuf.as_mut_ptr();
                let ret = RllDecodeStereoToStereo(&mut self.cl, src, to, 256, signed_output, flag);
                self.dump_audio("stereo2stereo", 256, sgn, flag, ret, 256);

                self.sbuf.iter_mut().for_each(|s| *s = 0);
                let to = self.sbuf.as_mut_ptr();
                let ret = RllDecodeStereoToMono(&mut self.cl, src, to, 128, signed_output, flag);
                self.dump_audio("stereo2mono", 128, sgn, flag, ret, 128);
            }
        }
    }
}

// --- the gate ---------------------------------------------------------------

fn harness_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tools/cin-oracle")
}

fn run_scenario(name: &str) -> String {
    let mut driver = CinDriver::new();
    let _ = writeln!(driver.out, "== cin-oracle {} ==", name);
    driver.dump_tables();

    if name == "rll_direct" {
        driver.run_rll();
    } else {
        let path = harness_dir().join("fixtures").join(format!("{}.roq", name));
        let stream = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("cin-oracle: cannot read {}: {}", path.display(), e));
        assert!(
            stream.len() >= 16,
            "cin-oracle: {} is shorter than a RoQ header",
            path.display()
        );
        driver.run_stream(name, &stream);
    }

    let _ = writeln!(driver.out, "== end ==");
    driver.out
}

/// Names the first line that differs, so a mismatch reads as one fact.
fn assert_matches_golden(name: &str, got: &str) {
    let path = harness_dir().join("golden").join(format!("{}.txt", name));
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cin-oracle: cannot read {}: {}", path.display(), e));
    if got == want {
        return;
    }
    for (i, (a, b)) in want.lines().zip(got.lines()).enumerate() {
        if a != b {
            panic!(
                "cin-oracle {}: line {} differs\n  oracle: {}\n  rust:   {}",
                name,
                i + 1,
                a,
                b
            );
        }
    }
    panic!(
        "cin-oracle {}: the dumps agree line for line but the lengths differ ({} oracle lines, {} rust lines)",
        name,
        want.lines().count(),
        got.lines().count()
    );
}

#[test]
fn cin_decode_core_matches_oracle_goldens() {
    for name in SCENARIOS {
        assert_matches_golden(name, &run_scenario(name));
    }
}
