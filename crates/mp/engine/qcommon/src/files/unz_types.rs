#![allow(non_camel_case_types, non_snake_case)]
//! Vendored minizip (`unzip.h`) + zlib32 (`zip.h`) reader types.
//!
//! These are internal to the `.zip`/pk3 reader (they never cross the ABI seam,
//! so exact layout is not parity-load-bearing), grouped in one module as a
//! cohesive vendored unit rather than one-type-per-file. Source:
//! `oracle/codemp/qcommon/unzip.h`, `oracle/codemp/zlib32/zip.h`.

use core::ffi::{c_int, c_uint, c_ulong, c_void};

pub use libc::FILE;

/// zlib `uInt` — `unsigned int`. Source: `oracle/codemp/zlib32/zip.h:59`
pub type uInt = c_uint;
/// zlib `uLong` — `unsigned long`. Source: `oracle/codemp/zlib32/zip.h:60`
pub type uLong = c_ulong;

/// zlib `EStatus` return codes as ints (negative = error, positive = event).
/// Source: `oracle/codemp/zlib32/zip.h:88-96`
pub const Z_STREAM_ERROR: c_int = -3;
pub const Z_BUF_ERROR: c_int = -2;
pub const Z_DATA_ERROR: c_int = -1;
pub const Z_OK: c_int = 0;
pub const Z_STREAM_END: c_int = 1;

/// zlib `ZF_DEFLATED` — the deflate compression-method id.
/// Source: `oracle/codemp/zlib32/zip.h:61`
pub const ZF_DEFLATED: uLong = 8;

/// zlib `z_stream` — the (de)compression stream state.
///
/// Internal to the reader; `avail_in`/`avail_out` are modeled as `uInt` (the
/// standard-zlib width the ported reader arithmetic assumes) rather than
/// `zip.h`'s `ulong` — the values fit and z_stream never crosses the ABI seam.
/// Source: `oracle/codemp/zlib32/zip.h:127-146`
#[repr(C)]
pub struct z_stream {
    pub next_in: *mut u8,
    pub avail_in: uInt,
    pub total_in: uLong,
    pub next_out: *mut u8,
    pub avail_out: uInt,
    pub total_out: uLong,
    pub status: c_int,
    pub error: c_int,
    pub istate: *mut c_void,
    pub dstate: *mut c_void,
    pub quality: uLong,
}

/// minizip `tm_unz` — decoded DOS date/time.
/// Source: `oracle/codemp/qcommon/unzip.h:15-23`
#[repr(C)]
pub struct tm_unz {
    pub tm_sec: uInt,
    pub tm_min: uInt,
    pub tm_hour: uInt,
    pub tm_mday: uInt,
    pub tm_mon: uInt,
    pub tm_year: uInt,
}

/// minizip `unz_global_info` — archive-wide info from the end of central dir.
/// Source: `oracle/codemp/qcommon/unzip.h:27-31`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct unz_global_info {
    pub number_entry: uLong,
    pub size_comment: uLong,
}

/// minizip `unz_file_info` — public info about one file in the zip.
/// Source: `oracle/codemp/qcommon/unzip.h:34-54`
#[repr(C)]
pub struct unz_file_info {
    pub version: uLong,
    pub version_needed: uLong,
    pub flag: uLong,
    pub compression_method: uLong,
    pub dosDate: uLong,
    pub crc: uLong,
    pub compressed_size: uLong,
    pub uncompressed_size: uLong,
    pub size_filename: uLong,
    pub size_file_extra: uLong,
    pub size_file_comment: uLong,
    pub disk_num_start: uLong,
    pub internal_fa: uLong,
    pub external_fa: uLong,
    pub tmu_date: tm_unz,
}

/// minizip `unz_file_info_internal` — private per-file info.
/// Source: `oracle/codemp/qcommon/unzip.h:57-60`
#[repr(C)]
pub struct unz_file_info_internal {
    pub offset_curfile: uLong,
}

/// minizip `file_in_zip_read_info_s` — per-file decompression state.
/// Source: `oracle/codemp/qcommon/unzip.h:64-85`
#[repr(C)]
pub struct file_in_zip_read_info_s {
    pub read_buffer: *mut u8,
    pub stream: z_stream,

    pub pos_in_zipfile: uLong,
    pub stream_initialised: uLong,

    pub offset_local_extrafield: uLong,
    pub size_local_extrafield: uInt,
    pub pos_local_extrafield: uLong,

    pub crc32: uLong,
    pub crc32_wait: uLong,
    pub rest_read_compressed: uLong,
    pub rest_read_uncompressed: uLong,
    pub file: *mut FILE,
    pub compression_method: uLong,
    pub byte_before_the_zipfile: uLong,
}

/// minizip `unz_s` — internal info about the open zipfile.
/// Source: `oracle/codemp/qcommon/unzip.h:88-108`
#[repr(C)]
pub struct unz_s {
    pub file: *mut FILE,
    pub gi: unz_global_info,
    pub byte_before_the_zipfile: uLong,
    pub num_file: uLong,
    pub pos_in_central_dir: uLong,
    pub current_file_ok: uLong,
    pub central_pos: uLong,

    pub size_central_dir: uLong,
    pub offset_central_dir: uLong,

    pub cur_file_info: unz_file_info,
    pub cur_file_info_internal: unz_file_info_internal,
    pub pfile_in_zip_read: *mut file_in_zip_read_info_s,
    pub tmpFile: *mut u8,
    pub tmpPos: c_int,
    pub tmpSize: c_int,
}
