//! Huffman 256-symbol round-trip: every byte value must encode via the
//! compressor and decode via the decompressor to itself, using the static
//! `MSG_H_DATA`-seeded trees exactly as `MSG_initHuffman` builds them. Guards
//! the wire path for non-ASCII payload bytes (the Latin-1 chat fix).

use mp_engine_qcommon::msg::MSG_H_DATA;
use mp_engine_qcommon::qcommon::huff::{
    Huff_Init, Huff_addRef, Huff_offsetReceive, Huff_offsetTransmit,
};
use mp_engine_qcommon::qcommon::huffman_t::huffman_t;

#[test]
fn all_256_symbols_round_trip() {
    unsafe {
        let mut huff: Box<huffman_t> = Box::new(core::mem::zeroed());
        Huff_Init(&mut *huff);
        for i in 0..256usize {
            for _ in 0..MSG_H_DATA[i] {
                Huff_addRef(&mut huff.compressor, i as u8);
                Huff_addRef(&mut huff.decompressor, i as u8);
            }
        }

        let mut buf = [0u8; 4096];
        let mut bloc: i32 = 0;
        for sym in 0..256i32 {
            Huff_offsetTransmit(&mut huff.compressor, sym, buf.as_mut_ptr(), &mut bloc);
        }
        let write_end = bloc;

        let mut rloc: i32 = 0;
        for sym in 0..256i32 {
            let mut ch: i32 = 0;
            Huff_offsetReceive(
                huff.decompressor.tree,
                &mut ch,
                buf.as_mut_ptr(),
                &mut rloc,
            );
            assert_eq!(ch, sym, "symbol {sym:#04x} corrupted in huff round-trip");
        }
        assert_eq!(rloc, write_end, "reader consumed different bit count");
    }
}
