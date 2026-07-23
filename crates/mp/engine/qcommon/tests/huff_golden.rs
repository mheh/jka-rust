//! Differential golden test for the adaptive-Huffman port (`huff.rs`).
//!
//! The committed goldens under `tools/huff-oracle/goldens/` are dumped by the
//! UNMODIFIED oracle `codemp/qcommon/huffman.cpp` (see `tools/huff-oracle/`):
//!   - `codes.txt` — the frozen prefix code each of symbols 0..256 gets from the
//!     `MSG_initHuffman`-seeded compressor, emitted through `send()`.
//!   - `seq.txt`   — the concatenated bitstream of transmitting `[0x00..0xFF]`.
//!   - `chat.txt`  — the bitstream of a chat string containing the symptomatic
//!     `0xB0` byte.
//!
//! This test seeds the Rust tree the way `MSG_initHuffman` does (from
//! `MSG_H_DATA`) and reproduces each dump, then compares byte-for-byte against
//! the goldens. A divergence pinpoints exactly which symbol's wire code the port
//! got wrong. Goldens are committed, so no C++ toolchain is needed at test time
//! (porting-rules §F18).

use mp_engine_qcommon::msg::MSG_H_DATA;
use mp_engine_qcommon::qcommon::huff::{Huff_Init, Huff_addRef, Huff_offsetTransmit};
use mp_engine_qcommon::qcommon::huffman_t::huffman_t;

const CODES_GOLDEN: &str = include_str!("../../../../../tools/huff-oracle/goldens/codes.txt");
const SEQ_GOLDEN: &str = include_str!("../../../../../tools/huff-oracle/goldens/seq.txt");
const CHAT_GOLDEN: &str = include_str!("../../../../../tools/huff-oracle/goldens/chat.txt");

/// Seed the compressor tree exactly as `MSG_initHuffman` (msg.cpp:3219-3234).
unsafe fn seed() -> Box<huffman_t> {
    let mut huff: Box<huffman_t> = Box::new(core::mem::zeroed());
    Huff_Init(&mut *huff);
    for i in 0..256usize {
        for _ in 0..MSG_H_DATA[i] {
            Huff_addRef(&mut huff.compressor, i as u8);
            Huff_addRef(&mut huff.decompressor, i as u8);
        }
    }
    huff
}

/// Bits `[0, nbits)` of `buf`, LSB-first within each byte (add_bit order).
fn push_bits(out: &mut String, buf: &[u8], nbits: i32) {
    for b in 0..nbits {
        let bit = (buf[(b >> 3) as usize] >> (b & 7)) & 1;
        out.push((b'0' + bit) as char);
    }
}

/// Reproduce `codes.txt`.
fn dump_codes(huff: &mut huffman_t) -> String {
    let mut out = String::new();
    for sym in 0..256i32 {
        let mut buf = [0u8; 64];
        let mut bloc: i32 = 0;
        unsafe {
            Huff_offsetTransmit(&mut huff.compressor, sym, buf.as_mut_ptr(), &mut bloc);
        }
        out.push_str(&format!("{sym:3}: "));
        push_bits(&mut out, &buf, bloc);
        out.push_str(&format!("  ({bloc} bits)\n"));
    }
    out
}

/// Reproduce `seq.txt` / `chat.txt`.
fn dump_stream(huff: &mut huffman_t, name: &str, seq: &[u8]) -> String {
    let mut buf = [0u8; 8192];
    let mut bloc: i32 = 0;
    for &ch in seq {
        unsafe {
            Huff_offsetTransmit(&mut huff.compressor, ch as i32, buf.as_mut_ptr(), &mut bloc);
        }
    }
    let nbytes = ((bloc + 7) >> 3) as usize;
    let mut out = format!("{name} bloc={bloc} bytes={nbytes}\n");
    for i in 0..nbytes {
        out.push_str(&format!("{:02x}", buf[i]));
        if (i + 1) % 32 == 0 {
            out.push('\n');
        }
    }
    if nbytes % 32 != 0 {
        out.push('\n');
    }
    out
}

#[test]
fn codes_match_oracle() {
    let mut huff = unsafe { seed() };
    let got = dump_codes(&mut huff);
    if got != CODES_GOLDEN {
        // Report the first diverging symbol line for a pinpoint failure.
        for (g, o) in got.lines().zip(CODES_GOLDEN.lines()) {
            assert_eq!(g, o, "first diverging per-symbol code");
        }
        assert_eq!(got, CODES_GOLDEN, "codes.txt differs (length/tail)");
    }
}

#[test]
fn seq_stream_matches_oracle() {
    let mut huff = unsafe { seed() };
    let seq: Vec<u8> = (0..=255u8).collect();
    let got = dump_stream(&mut huff, "seq", &seq);
    assert_eq!(got, SEQ_GOLDEN, "seq bitstream differs from oracle");
}

#[test]
fn chat_stream_matches_oracle() {
    let mut huff = unsafe { seed() };
    let chat: [u8; 20] = [
        b'c', b'h', b'a', b't', b' ', b'"', b'P', b'^', b'7', 0x19, b':', b' ', b'y', b'o', b' ',
        0xb0, b'/', b'.', b's', b'"',
    ];
    let got = dump_stream(&mut huff, "chat", &chat);
    assert_eq!(got, CHAT_GOLDEN, "chat bitstream differs from oracle");
}
