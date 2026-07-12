//! MD4 / `Com_BlockChecksum` parity against RFC 1320.
//!
//! Raven's md4.cpp is the RSA reference implementation with 32-bit `UINT4`
//! (retail win32/linux-i386 `unsigned long`); the RFC 1320 appendix vectors
//! ARE the retail digests. `sv_pure` hangs off these digests
//! (`pack->checksum` / `pack->pure_checksum`, `files_pc.cpp:1513-1514`), so
//! a divergent word here desyncs every pure client.

use core::ffi::c_int;

use mp_engine_core::engine::Engine;
use mp_engine_qcommon::md4::md4_ctx::MD4_CTX;
use mp_engine_qcommon::md4_fns::{
    Com_BlockChecksum, Com_BlockChecksumKey, MD4Final, MD4Init, MD4Update,
};

fn md4_hex(engine: &mut Engine, msg: &[u8]) -> String {
    let mut digest = [0u8; 16];
    let mut ctx: MD4_CTX = unsafe { core::mem::zeroed() };
    MD4Init(&mut ctx);
    MD4Update(&mut ctx, msg.as_ptr(), msg.len() as u32);
    MD4Final(&mut engine.common, digest.as_mut_ptr(), &mut ctx);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// RFC 1320 appendix A.5 test suite, verbatim.
#[test]
fn rfc1320_vectors() {
    let mut engine = Engine::new();
    let vectors: &[(&[u8], &str)] = &[
        (b"", "31d6cfe0d16ae931b73c59d7e0c089c0"),
        (b"a", "bde52cb31de33e46245e05fbdbd6fb24"),
        (b"abc", "a448017aaf21d8525fc10ae87aa6729d"),
        (b"message digest", "d9130a8164549fe818874806e1c7014b"),
        (
            b"abcdefghijklmnopqrstuvwxyz",
            "d79e1c308aa5bbcdeea8ed63df412da9",
        ),
        (
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            "043f8582f241db351ce627e153e7f0e4",
        ),
        (
            b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
            "e33b4ddc9c38f2199c3e7b164fcc0536",
        ),
    ];
    for (msg, expect) in vectors {
        assert_eq!(&md4_hex(&mut engine, msg), expect, "MD4({:?})", msg);
    }
}

/// `Com_BlockChecksum` folds the digest words with XOR (`md4.cpp:267-280`);
/// cross-check the fold against the digest bytes for a multi-block input.
#[test]
fn block_checksum_folds_digest() {
    let mut engine = Engine::new();
    let msg = vec![0xa5u8; 1000]; // spans multiple 64-byte blocks

    let mut digest = [0u8; 16];
    let mut ctx: MD4_CTX = unsafe { core::mem::zeroed() };
    MD4Init(&mut ctx);
    MD4Update(&mut ctx, msg.as_ptr(), msg.len() as u32);
    MD4Final(&mut engine.common, digest.as_mut_ptr(), &mut ctx);

    let word = |i: usize| i32::from_le_bytes(digest[i * 4..i * 4 + 4].try_into().unwrap());
    let expect = (word(0) ^ word(1) ^ word(2) ^ word(3)) as u32;

    let got = Com_BlockChecksum(
        &mut engine.common,
        msg.as_ptr() as *const (),
        msg.len() as c_int,
    );
    assert_eq!(got, expect);
}

/// `Com_BlockChecksumKey(buf, len, key)` == MD4 over key-bytes ++ buf, folded
/// (`md4.cpp:282-296`) — the `pure_checksum` path with the little-endian
/// `fs_checksumFeed` prepended.
#[test]
fn block_checksum_key_prepends_key() {
    let mut engine = Engine::new();
    let msg = *b"pure-checksum-fixture";
    let key: c_int = 0x1234_5678;

    let mut keyed = key.to_le_bytes().to_vec();
    keyed.extend_from_slice(&msg);
    let mut digest = [0u8; 16];
    let mut ctx: MD4_CTX = unsafe { core::mem::zeroed() };
    MD4Init(&mut ctx);
    MD4Update(&mut ctx, keyed.as_ptr(), keyed.len() as u32);
    MD4Final(&mut engine.common, digest.as_mut_ptr(), &mut ctx);
    let word = |i: usize| i32::from_le_bytes(digest[i * 4..i * 4 + 4].try_into().unwrap());
    let expect = (word(0) ^ word(1) ^ word(2) ^ word(3)) as u32;

    let mut buf = msg;
    let got = Com_BlockChecksumKey(
        &mut engine.common,
        buf.as_mut_ptr() as *mut (),
        buf.len() as c_int,
        key,
    );
    assert_eq!(got, expect);
}
