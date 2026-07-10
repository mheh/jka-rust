//! Raven `matcomp.c` — bit-packing codec for the 24-byte compressed bone
//! matrix format used by the mdxa animation channel.
//!
//! Pure bit-packing math over `float mat[3][4]`, no globals, no I/O. Per
//! `TRM-D1`(a) (ruling 56a) this module lands in `mp_engine_ghoul2`, beside its
//! sole live consumer (`MC_UnCompressQuat` <- `UnCompressBone`,
//! `tr_ghoul2.cpp:1158-1162`, the frozen bone-eval chain) rather than
//! `mp_renderer` (its literal `renderer/` origin), because `G2SV-D5` forbids
//! `mp_engine_ghoul2` -> `mp_renderer` and matcomp is self-contained pure math
//! with no dependency on renderer state.
//!
//! Source: `oracle/codemp/renderer/matcomp.c`, `oracle/codemp/renderer/matcomp.h`

/// `MC_BITS_X`/`MC_BITS_Y`/`MC_BITS_Z`/`MC_BITS_VECT` — all 16 in this codec,
/// so every field's byte position (`MC_POS_*`) lands on an even offset and
/// every `MC_SHIFT_*` is 0 (each `POS`/`SHIFT` pair is `bits*i / 8` / `% 8`
/// and `bits` is a constant multiple of 8). That collapses the 12 masked,
/// shifted `unsigned int` OR-writes in `MC_Compress`/`MC_UnCompress` into 12
/// contiguous little-endian `u16` slots — the transcription below writes/reads
/// them as such.
///
/// Source: `oracle/codemp/renderer/matcomp.h:9-12`
const MC_BITS: i32 = 16;

/// `MC_SCALE_X`/`MC_SCALE_Y`/`MC_SCALE_Z`.
///
/// Source: `oracle/codemp/renderer/matcomp.h:14-16`
const MC_SCALE_X: f32 = 1.0 / 64.0;
const MC_SCALE_Y: f32 = 1.0 / 64.0;
const MC_SCALE_Z: f32 = 1.0 / 64.0;

/// `MC_SCALE_VECT` — `1.0f/(float)((1<<(MC_BITS_VECT-1))-2)`.
///
/// Source: `oracle/codemp/renderer/matcomp.c:12`
const MC_SCALE_VECT: f32 = 1.0 / (((1i32 << (MC_BITS - 1)) - 2) as f32);

/// One `MC_Compress` field: `(int)(v/scale)`, offset by the bias
/// `1<<(bits-1)`, then clamped to `[0, (1<<bits)-1]` (`matcomp.c:58-63` and
/// the 11 repeats of that same four-line shape).
fn mc_quantize(v: f32, scale: f32) -> u16 {
    let mut val = (v / scale) as i32;
    val += 1 << (MC_BITS - 1);
    if val >= 1 << MC_BITS {
        val = (1 << MC_BITS) - 1;
    }
    if val < 0 {
        val = 0;
    }
    val as u16
}

/// Raven `MC_Compress`.
///
/// Packs a `float mat[3][4]` bone matrix into the 24-byte (`MC_COMP_BYTES`)
/// quantized wire format. Per `TRM-D1`(a) this gets a §20 zero-caller note iff
/// verified dead at port: no caller anywhere in `codemp/` (compiles only
/// because the TU links).
///
/// Diverges from Raven's raw `*(unsigned int*)(comp+pos) |= val<<shift`
/// writes, which OR into a double-sized 48-byte local scratch buffer
/// specifically because the last few writes spill past byte 24 (Raven's own
/// comment `:156-159` calls this out as a bounds-checker-flagged UB
/// workaround, harmless only because the spilled high bytes are always zero).
/// Since every `MC_SHIFT_*` here is 0 (see `MC_BITS`), each field is exactly
/// a 2-byte little-endian write with no overlap — this port writes the 12
/// `u16`s directly into the caller's 24-byte `comp` with no OOB write and no
/// scratch buffer needed (§19: Raven's behavior here is UB it works around,
/// not behavior to reproduce).
///
/// Source: `oracle/codemp/renderer/matcomp.c:50-161`
pub fn mc_compress(mat: &[[f32; 4]; 3], comp: &mut [u8]) {
    let vals: [u16; 12] = [
        mc_quantize(mat[0][3], MC_SCALE_X),
        mc_quantize(mat[1][3], MC_SCALE_Y),
        mc_quantize(mat[2][3], MC_SCALE_Z),
        mc_quantize(mat[0][0], MC_SCALE_VECT),
        mc_quantize(mat[0][1], MC_SCALE_VECT),
        mc_quantize(mat[0][2], MC_SCALE_VECT),
        mc_quantize(mat[1][0], MC_SCALE_VECT),
        mc_quantize(mat[1][1], MC_SCALE_VECT),
        mc_quantize(mat[1][2], MC_SCALE_VECT),
        mc_quantize(mat[2][0], MC_SCALE_VECT),
        mc_quantize(mat[2][1], MC_SCALE_VECT),
        mc_quantize(mat[2][2], MC_SCALE_VECT),
    ];
    for (i, v) in vals.iter().enumerate() {
        let le = v.to_le_bytes();
        comp[i * 2] = le[0];
        comp[i * 2 + 1] = le[1];
    }
}

/// Raven `MC_UnCompress`.
///
/// Unpacks the 24-byte (`MC_COMP_BYTES`) quantized wire format back into a
/// `float mat[3][4]` bone matrix. Per `TRM-D1`(a) this gets a §20 zero-caller
/// note iff verified dead at port: the only appearance in `codemp/` is
/// commented-out (`tr_ghoul2.cpp:1705-1746`).
///
/// Source: `oracle/codemp/renderer/matcomp.c:163-217`
pub fn mc_uncompress(mat: &mut [[f32; 4]; 3], comp: &[u8]) {
    let read = |i: usize| -> i32 { u16::from_le_bytes([comp[i * 2], comp[i * 2 + 1]]) as i32 };

    let bias = 1 << (MC_BITS - 1);

    mat[0][3] = ((read(0) - bias) as f32) * MC_SCALE_X;
    mat[1][3] = ((read(1) - bias) as f32) * MC_SCALE_Y;
    mat[2][3] = ((read(2) - bias) as f32) * MC_SCALE_Z;

    mat[0][0] = ((read(3) - bias) as f32) * MC_SCALE_VECT;
    mat[0][1] = ((read(4) - bias) as f32) * MC_SCALE_VECT;
    mat[0][2] = ((read(5) - bias) as f32) * MC_SCALE_VECT;

    mat[1][0] = ((read(6) - bias) as f32) * MC_SCALE_VECT;
    mat[1][1] = ((read(7) - bias) as f32) * MC_SCALE_VECT;
    mat[1][2] = ((read(8) - bias) as f32) * MC_SCALE_VECT;

    mat[2][0] = ((read(9) - bias) as f32) * MC_SCALE_VECT;
    mat[2][1] = ((read(10) - bias) as f32) * MC_SCALE_VECT;
    mat[2][2] = ((read(11) - bias) as f32) * MC_SCALE_VECT;
}

/// Raven `MC_UnCompressQuat`.
///
/// Reads a 16-bit-quantized quaternion + translation from `comp` and expands
/// it into a rotation+translation `float mat[3][4]`. The sole live matcomp
/// path: consumed by `UnCompressBone` (`tr_ghoul2.cpp:1158-1162`), inside the
/// frozen bone-eval chain.
///
/// Source: `oracle/codemp/renderer/matcomp.c:219-291`
pub fn mc_uncompress_quat(mat: &mut [[f32; 4]; 3], comp: &[u8]) {
    let read = |i: usize| -> f32 { u16::from_le_bytes([comp[i * 2], comp[i * 2 + 1]]) as f32 };

    let w = read(0) / 16383.0 - 2.0;
    let x = read(1) / 16383.0 - 2.0;
    let y = read(2) / 16383.0 - 2.0;
    let z = read(3) / 16383.0 - 2.0;

    let t_x = 2.0 * x;
    let t_y = 2.0 * y;
    let t_z = 2.0 * z;
    let t_wx = t_x * w;
    let t_wy = t_y * w;
    let t_wz = t_z * w;
    let t_xx = t_x * x;
    let t_xy = t_y * x;
    let t_xz = t_z * x;
    let t_yy = t_y * y;
    let t_yz = t_z * y;
    let t_zz = t_z * z;

    // rot...
    //
    mat[0][0] = 1.0 - (t_yy + t_zz);
    mat[0][1] = t_xy - t_wz;
    mat[0][2] = t_xz + t_wy;
    mat[1][0] = t_xy + t_wz;
    mat[1][1] = 1.0 - (t_xx + t_zz);
    mat[1][2] = t_yz - t_wx;
    mat[2][0] = t_xz - t_wy;
    mat[2][1] = t_yz + t_wx;
    mat[2][2] = 1.0 - (t_xx + t_yy);

    // xlat...
    //
    mat[0][3] = read(4) / 64.0 - 512.0;
    mat[1][3] = read(5) / 64.0 - 512.0;
    mat[2][3] = read(6) / 64.0 - 512.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MC_Compress`/`MC_UnCompress` round-trip: every field is quantized to
    /// 16 bits then rescaled back, so the result should reproduce the input
    /// within one quantization step (`MC_SCALE_VECT`/`MC_SCALE_X` etc).
    #[test]
    fn compress_uncompress_round_trip() {
        let mat: [[f32; 4]; 3] = [
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 1.0, 0.0, -20.5],
            [0.0, 0.0, 1.0, 3.25],
        ];
        let mut comp = [0u8; 24];
        mc_compress(&mat, &mut comp);
        let mut out: [[f32; 4]; 3] = [[0.0; 4]; 3];
        mc_uncompress(&mut out, &comp);

        for row in 0..3 {
            for col in 0..4 {
                let scale = match col {
                    3 if row == 0 => MC_SCALE_X,
                    3 if row == 1 => MC_SCALE_Y,
                    3 if row == 2 => MC_SCALE_Z,
                    _ => MC_SCALE_VECT,
                };
                assert!(
                    (mat[row][col] - out[row][col]).abs() <= scale + f32::EPSILON,
                    "row {row} col {col}: {} vs {}",
                    mat[row][col],
                    out[row][col]
                );
            }
        }
    }

    /// `MC_Compress` clamps out-of-range translation components instead of
    /// wrapping (`matcomp.c:60-63`, the `val>=1<<bits`/`val<0` clamp arms).
    #[test]
    fn compress_clamps_out_of_range() {
        let mat: [[f32; 4]; 3] = [
            [1.0, 0.0, 0.0, 1000.0],
            [0.0, 1.0, 0.0, -1000.0],
            [0.0, 0.0, 1.0, 0.0],
        ];
        let mut comp = [0u8; 24];
        mc_compress(&mat, &mut comp);
        // X clamps high -> max u16 (65535), Y clamps low -> 0.
        assert_eq!(u16::from_le_bytes([comp[0], comp[1]]), 0xFFFF);
        assert_eq!(u16::from_le_bytes([comp[2], comp[3]]), 0);
    }

    /// `MC_UnCompressQuat` on the identity quaternion (mid-scale zero: raw
    /// value `16383*2 = 32766` for w, `2*16383=32766`... actually identity is
    /// w=1,x=y=z=0 -> encoded raw = (component+2)*16383) should decode to the
    /// identity rotation matrix and zero translation.
    #[test]
    fn uncompress_quat_identity() {
        let encode = |v: f32| -> u16 { ((v + 2.0) * 16383.0).round() as u16 };
        let mut comp = [0u8; 24];
        let raw: [u16; 7] = [
            encode(1.0),           // w
            encode(0.0),           // x
            encode(0.0),           // y
            encode(0.0),           // z
            (512.0 * 64.0) as u16, // tx = 0 -> f/64 - 512 = 0 => f = 512*64
            (512.0 * 64.0) as u16, // ty
            (512.0 * 64.0) as u16, // tz
        ];
        for (i, v) in raw.iter().enumerate() {
            let le = v.to_le_bytes();
            comp[i * 2] = le[0];
            comp[i * 2 + 1] = le[1];
        }
        let mut mat: [[f32; 4]; 3] = [[0.0; 4]; 3];
        mc_uncompress_quat(&mut mat, &comp);

        let expected: [[f32; 4]; 3] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ];
        for row in 0..3 {
            for col in 0..4 {
                assert!(
                    (mat[row][col] - expected[row][col]).abs() < 1.0e-3,
                    "row {row} col {col}: {}",
                    mat[row][col]
                );
            }
        }
    }
}
