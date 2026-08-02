//! zlib32 deflate/inflate seam for the RMG terrain download blocks.
//!
//! Raven compresses the landscape height/flatten maps into the client
//! gamestate message with a single `deflate(&zdata, Z_SYNC_FLUSH)` pass
//! (`oracle/codemp/server/sv_client.cpp:768-803`) and the client reverses each
//! block with a single `inflate(&zdata)` pass (`oracle/codemp/client/cl_parse.cpp:465-521`).
//! Per DEC-59.2 and the user ruling 2026-07-11 (sibling `unzip.rs`, which backs
//! zlib32's raw-DEFLATE inflate with flate2's `Decompress`), both sides are
//! backed by flate2 (`Compress` / `Decompress`) rather than porting zlib32's
//! `deflate`/`inflate`.

use core::ffi::c_int;

use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress};

/// Raven's `deflateInit(&zdata, Z_MAX_COMPRESSION)` + single
/// `deflate(&zdata, Z_SYNC_FLUSH)` + `deflateEnd(&zdata)` sequence over one
/// landscape map (`sv_client.cpp:772-787` / `790-803`), returning the
/// `zdata.total_out` byte count. `deflateInit` produces a zlib-wrapped stream
/// (2-byte header + adler32) at `Z_MAX_COMPRESSION` (level 9).
///
/// Source: `oracle/codemp/server/sv_client.cpp:768-803`
pub fn deflate_sync_flush(src: *const u8, avail_in: c_int, out: &mut [u8]) -> c_int {
    // SAFETY: mirrors Raven's `zdata.next_in = src; zdata.avail_in = avail_in` —
    // `src` points to `avail_in` bytes of caller-owned landscape map data.
    let input = unsafe { core::slice::from_raw_parts(src, avail_in as usize) };

    // deflateInit(&zdata, Z_MAX_COMPRESSION): zlib-wrapped, level 9.
    let mut zdata = Compress::new(Compression::best(), true);

    // deflate(&zdata, Z_SYNC_FLUSH): single sync-flush pass into `out`.
    let _ = zdata.compress(input, out, FlushCompress::Sync);

    // (unsigned short)zdata.total_out at the call site; deflateEnd is the
    // `Compress` drop here.
    zdata.total_out() as c_int
}

/// Raven's `inflateInit(&zdata, Z_SYNC_FLUSH)` + single `inflate(&zdata)` +
/// `inflateEnd(&zdata)` sequence over one landscape map
/// (`cl_parse.cpp:480-494` / `505-515`), returning the `zdata.total_out` byte
/// count. `inflateInit` expects the zlib-wrapped stream `deflate_sync_flush`
/// produced (2-byte header + adler32).
///
/// Source: `oracle/codemp/client/cl_parse.cpp:465-521`
pub fn inflate_sync_flush(src: *const u8, avail_in: c_int, out: &mut [u8]) -> c_int {
    // SAFETY: mirrors Raven's `zdata.next_in = src; zdata.avail_in = avail_in` —
    // `src` points to `avail_in` bytes of caller-owned compressed map data.
    let input = unsafe { core::slice::from_raw_parts(src, avail_in as usize) };

    // inflateInit(&zdata, Z_SYNC_FLUSH): zlib-wrapped stream.
    let mut zdata = Decompress::new(true);

    // inflate(&zdata): single sync-flush pass into `out`.
    let _ = zdata.decompress(input, out, FlushDecompress::Sync);

    // zdata.total_out at the call site; inflateEnd is the `Decompress` drop
    // here.
    zdata.total_out() as c_int
}
