//! Raven `Q_strncpyz` — the bounded, always-NUL-terminated C-buffer string
//! copy, canonical safe form: `&str` source into a `c_char` destination
//! slice (`destsize` = `dest.len()`). Raw-pointer seams build the slice with
//! `slice::from_raw_parts_mut` at the call site; fixed arrays coerce.
//!
//! Source: `oracle/codemp/game/q_shared.c:826-840`

use core::ffi::c_char;

/// Raven `Q_strncpyz` over bytes — the canonical body for sites still holding
/// C data (byte-exact, no UTF-8 pass). `src`'s logical string ends at its
/// first NUL or the slice end, whichever comes first (fixed C arrays carry
/// garbage past the terminator).
pub fn Q_strncpyzBytes(dest: &mut [c_char], src: &[u8], destsize: usize) {
    let destsize = destsize.min(dest.len());
    if destsize == 0 {
        return;
    }
    let src_len = src.iter().position(|&b| b == 0).unwrap_or(src.len());
    let n = src_len.min(destsize - 1);
    for (d, s) in dest.iter_mut().zip(src[..n].iter()) {
        *d = *s as c_char;
    }
    dest[n] = 0;
}

/// Raven `Q_strncpyz`: copy up to `destsize-1` bytes of `src` and always
/// NUL-terminate. A zero `destsize` writes nothing (Raven's `destsize < 1` is
/// a `Com_Error`), and a `destsize` past `dest.len()` clamps to the slice
/// (§19: C overflowed there).
pub fn Q_strncpyz(dest: &mut [c_char], src: &str, destsize: usize) {
    Q_strncpyzBytes(dest, src.as_bytes(), destsize);
}

#[cfg(test)]
mod q_strncpyz_tests {
    use super::Q_strncpyz;
    use core::ffi::c_char;

    #[test]
    fn copies_and_nul_terminates() {
        let mut buf = [0x7f as c_char; 8];
        Q_strncpyz(&mut buf, "abc", 8);
        assert_eq!(
            &buf[..4],
            &[b'a' as c_char, b'b' as c_char, b'c' as c_char, 0]
        );
    }

    #[test]
    fn truncates_to_destsize_minus_one() {
        let mut buf = [0x7f as c_char; 8];
        Q_strncpyz(&mut buf, "abcdef", 4);
        assert_eq!(
            &buf[..4],
            &[b'a' as c_char, b'b' as c_char, b'c' as c_char, 0]
        );
    }

    #[test]
    fn oversized_destsize_clamps_to_the_slice() {
        let mut buf = [0x7f as c_char; 4];
        Q_strncpyz(&mut buf, "abcdef", 64);
        assert_eq!(buf, [b'a' as c_char, b'b' as c_char, b'c' as c_char, 0]);
    }

    #[test]
    fn zero_destsize_is_a_no_op() {
        let mut buf = [0x7f as c_char; 4];
        Q_strncpyz(&mut buf, "abc", 0);
        assert_eq!(buf, [0x7f as c_char; 4]);
    }

    #[test]
    fn bytes_form_stops_at_src_nul() {
        let mut buf = [0x7f as c_char; 8];
        super::Q_strncpyzBytes(&mut buf, b"ab\0garbage", 8);
        assert_eq!(&buf[..3], &[b'a' as c_char, b'b' as c_char, 0]);
    }

    #[test]
    fn bytes_form_is_byte_exact_above_ascii() {
        let mut buf = [0 as c_char; 4];
        super::Q_strncpyzBytes(&mut buf, &[0xE9, 0xFF], 4);
        assert_eq!(&buf[..3], &[0xE9u8 as c_char, 0xFFu8 as c_char, 0]);
    }
}
