//! zlib32 deflate seam for the RMG terrain download blocks.
//!
//! Raven compresses the landscape height/flatten maps into the client
//! gamestate message with a single `deflate(&zdata, Z_SYNC_FLUSH)` pass
//! (`oracle/codemp/server/sv_client.cpp:768-803`). Per the user ruling
//! 2026-07-11 (sibling `unzip.rs`, which backs zlib32's raw-DEFLATE inflate with
//! flate2's `Decompress`), the deflate side is backed by flate2's `Compress`
//! rather than porting zlib32's `deflate`.

use core::ffi::c_int;

use flate2::{Compress, Compression, FlushCompress};

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
