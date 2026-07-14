//! Dump our engine's post-`MSG_initHuffman` Huffman code for every symbol
//! 0..255 in the same format as the oracle-side dumper
//! (scratchpad/huffdump/src/main.cpp), for differential comparison —
//! 2026-07-14 connect-drop hunt. Run with `-- --nocapture`.
#![allow(non_snake_case)]

use mp_engine_core::engine::Engine;
use mp_engine_core::host_view::engine_host_view;
use mp_engine_qcommon::msg::{MSG_BeginReading, MSG_Init, MSG_ReadBits, MSG_WriteBits};
use mp_qshared::common::mp::qcommon::msg_t::msg_t;

#[test]
fn huffcodes() {
    let mut engine = Engine::new();
    let mut view = engine_host_view(&mut engine);
    // Retail override files are fully commented out; skip the FS-dependent
    // override pass (see snapdecode.rs).
    view.common.g_nOverrideChecked = true;

    for sym in 0..256 {
        let mut buf = [0u8; 64];
        let mut m: msg_t = unsafe { core::mem::zeroed() };
        MSG_Init(&mut view, &mut m, buf.as_mut_ptr(), buf.len() as i32);
        MSG_WriteBits(view.common, &mut m, sym, 8);
        let nbits = m.bit;
        let mut line = format!("{sym:3} {nbits:2} ");
        for b in 0..nbits {
            line.push(if (buf[(b >> 3) as usize] >> (b & 7)) & 1 != 0 { '1' } else { '0' });
        }
        // decode self-check
        MSG_BeginReading(&mut m);
        let got = MSG_ReadBits(view.common, &mut m, 8);
        let roff = m.bit;
        line.push_str(if got == sym && roff == nbits { " ok" } else { " DECODE-MISMATCH" });
        println!("{line}");
    }
}
