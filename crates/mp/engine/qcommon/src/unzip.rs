//! `unzip.cpp` — vendored minizip reader (`unz*` API).
//!
//! Source: `oracle/codemp/qcommon/unzip.cpp`

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong};

use libc::{SEEK_CUR, SEEK_SET};

use crate::files::unz_file::unzFile;
use crate::files::unz_types::{
    tm_unz, uInt, uLong, unz_file_info, unz_file_info_internal, unz_global_info, unz_s, z_stream,
    FILE, ZF_DEFLATED, Z_OK, Z_STREAM_END,
};
use crate::files::unzip_consts::{
    CASESENSITIVITYDEFAULTVALUE, SIZECENTRALDIRITEM, SIZEZIPLOCALHEADER, UNZ_BADZIPFILE,
    UNZ_BUFSIZE, UNZ_END_OF_LIST_OF_FILE, UNZ_EOF, UNZ_ERRNO, UNZ_MAXFILENAMEINZIP, UNZ_OK,
    UNZ_PARAMERROR,
};

// Sweep: extern forward-declare eliminated. `inflate` is the vendored zlib32
// DEFLATE decompressor (`oracle/codemp/zlib32`), an unported external C
// library with no Rust home — referenced by its bare Raven name; reported.

/// Raven `unzlocal_getShort` — reads a little-endian 16-bit value from `fin` into `*pX`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:255-263`
pub fn unzlocal_getShort(fin: *mut FILE, pX: *mut uLong) -> c_int {
    let mut v: i16 = 0;
    unsafe {
        libc::fread(
            &mut v as *mut i16 as *mut libc::c_void,
            core::mem::size_of::<i16>(),
            1,
            fin,
        );
        *pX = i16::from_le(v) as uLong;
    }
    UNZ_OK
}

/// Raven `unzlocal_getLong` — reads a little-endian 32-bit value from `fin` into `*pX`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:265-273`
pub fn unzlocal_getLong(fin: *mut FILE, pX: *mut uLong) -> c_int {
    let mut v: i32 = 0;
    unsafe {
        libc::fread(
            &mut v as *mut i32 as *mut libc::c_void,
            core::mem::size_of::<i32>(),
            1,
            fin,
        );
        *pX = i32::from_le(v) as uLong;
    }
    UNZ_OK
}

/// Raven `strcmpcasenosensitive_internal` — case-insensitive ASCII strcmp.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:277-296`
pub fn strcmpcasenosensitive_internal(fileName1: *const c_char, fileName2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = fileName1;
        let mut p2 = fileName2;
        loop {
            let mut c1 = *p1 as u8 as i8;
            let mut c2 = *p2 as u8 as i8;
            p1 = p1.add(1);
            p2 = p2.add(1);
            if (c1 as u8 as char) >= 'a' && (c1 as u8 as char) <= 'z' {
                c1 -= 0x20;
            }
            if (c2 as u8 as char) >= 'a' && (c2 as u8 as char) <= 'z' {
                c2 -= 0x20;
            }
            if c1 == 0 {
                return if c2 == 0 { 0 } else { -1 };
            }
            if c2 == 0 {
                return 1;
            }
            if c1 < c2 {
                return -1;
            }
            if c1 > c2 {
                return 1;
            }
        }
    }
}

/// Raven `unzGetGlobalInfo` — copies the archive's global info struct out of `file`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:528-536`
pub fn unzGetGlobalInfo(file: unzFile, pglobal_info: *mut unz_global_info) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        *pglobal_info = (*s).gi;
    }
    UNZ_OK
}

/// Raven `unzlocal_DosDateToTmuDate` — decodes a packed DOS date/time into `*ptm`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:542-553`
pub fn unzlocal_DosDateToTmuDate(ulDosDate: uLong, ptm: *mut tm_unz) {
    let uDate: uLong = ulDosDate >> 16;
    unsafe {
        (*ptm).tm_mday = (uDate & 0x1f) as uInt;
        (*ptm).tm_mon = (((uDate & 0x1E0) / 0x20).wrapping_sub(1)) as uInt;
        (*ptm).tm_year = (((uDate & 0x0FE00) / 0x0200) + 1980) as uInt;

        (*ptm).tm_hour = ((ulDosDate & 0xF800) / 0x800) as uInt;
        (*ptm).tm_min = ((ulDosDate & 0x7E0) / 0x20) as uInt;
        (*ptm).tm_sec = (2 * (ulDosDate & 0x1f)) as uInt;
    }
}

/// Raven `unzGetCurrentFileInfoPosition` — reports the current file's byte offset within the
/// archive's central directory.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:786-796`
pub fn unzGetCurrentFileInfoPosition(file: unzFile, pos: *mut c_ulong) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        *pos = (*s).pos_in_central_dir as c_ulong;
    }
    UNZ_OK
}

/// Raven `unzReadCurrentFile` — reads (and inflates, if compressed) up to `len` bytes of the
/// current file into `buf`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:1050-1168`
pub fn unzReadCurrentFile(file: unzFile, buf: *mut (), len: c_uint) -> c_int {
    let mut err = UNZ_OK;
    let mut iRead: c_uint = 0;
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        let pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        if (*pfile_in_zip_read_info).read_buffer.is_null() {
            return UNZ_END_OF_LIST_OF_FILE;
        }
        if len == 0 {
            return 0;
        }

        (*pfile_in_zip_read_info).stream.next_out = buf as *mut u8;
        (*pfile_in_zip_read_info).stream.avail_out = len as uInt;

        if (len as uLong) > (*pfile_in_zip_read_info).rest_read_uncompressed {
            (*pfile_in_zip_read_info).stream.avail_out =
                (*pfile_in_zip_read_info).rest_read_uncompressed as uInt;
        }

        while (*pfile_in_zip_read_info).stream.avail_out > 0 {
            if (*pfile_in_zip_read_info).stream.avail_in == 0
                && (*pfile_in_zip_read_info).rest_read_compressed > 0
            {
                let mut uReadThis: uInt = UNZ_BUFSIZE as uInt;
                if (*pfile_in_zip_read_info).rest_read_compressed < uReadThis as uLong {
                    uReadThis = (*pfile_in_zip_read_info).rest_read_compressed as uInt;
                }
                if uReadThis == 0 {
                    return UNZ_EOF;
                }
                if (*s).cur_file_info.compressed_size
                    == (*pfile_in_zip_read_info).rest_read_compressed
                {
                    if libc::fseek(
                        (*pfile_in_zip_read_info).file,
                        ((*pfile_in_zip_read_info).pos_in_zipfile
                            + (*pfile_in_zip_read_info).byte_before_the_zipfile)
                            as c_long,
                        SEEK_SET,
                    ) != 0
                    {
                        return UNZ_ERRNO;
                    }
                }
                if libc::fread(
                    (*pfile_in_zip_read_info).read_buffer as *mut libc::c_void,
                    uReadThis as usize,
                    1,
                    (*pfile_in_zip_read_info).file,
                ) != 1
                {
                    return UNZ_ERRNO;
                }
                (*pfile_in_zip_read_info).pos_in_zipfile += uReadThis as uLong;
                (*pfile_in_zip_read_info).rest_read_compressed -= uReadThis as uLong;

                (*pfile_in_zip_read_info).stream.next_in = (*pfile_in_zip_read_info).read_buffer;
                (*pfile_in_zip_read_info).stream.avail_in = uReadThis;
            }

            if (*pfile_in_zip_read_info).compression_method == 0 {
                let uDoCopy: uInt = if (*pfile_in_zip_read_info).stream.avail_out
                    < (*pfile_in_zip_read_info).stream.avail_in
                {
                    (*pfile_in_zip_read_info).stream.avail_out
                } else {
                    (*pfile_in_zip_read_info).stream.avail_in
                };

                for i in 0..uDoCopy {
                    *(*pfile_in_zip_read_info).stream.next_out.add(i as usize) =
                        *(*pfile_in_zip_read_info).stream.next_in.add(i as usize);
                }

                (*pfile_in_zip_read_info).rest_read_uncompressed -= uDoCopy as uLong;
                (*pfile_in_zip_read_info).stream.avail_in -= uDoCopy;
                (*pfile_in_zip_read_info).stream.avail_out -= uDoCopy;
                (*pfile_in_zip_read_info).stream.next_out = (*pfile_in_zip_read_info)
                    .stream
                    .next_out
                    .add(uDoCopy as usize);
                (*pfile_in_zip_read_info).stream.next_in = (*pfile_in_zip_read_info)
                    .stream
                    .next_in
                    .add(uDoCopy as usize);
                (*pfile_in_zip_read_info).stream.total_out += uDoCopy as uLong;
                iRead += uDoCopy;
            } else {
                let uTotalOutBefore: uLong = (*pfile_in_zip_read_info).stream.total_out;

                err = inflate(&mut (*pfile_in_zip_read_info).stream);

                let uTotalOutAfter: uLong = (*pfile_in_zip_read_info).stream.total_out;
                let uOutThis: uLong = uTotalOutAfter - uTotalOutBefore;

                (*pfile_in_zip_read_info).rest_read_uncompressed -= uOutThis;

                iRead += (uTotalOutAfter - uTotalOutBefore) as uInt;

                if err == Z_STREAM_END {
                    return if iRead == 0 { UNZ_EOF } else { iRead as c_int };
                }
                if err != Z_OK {
                    break;
                }
            }
        }

        if err == Z_OK {
            return iRead as c_int;
        }
        err
    }
}

/// Raven `unztell` — bytes read so far from the current file's decompression stream.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:1174-1187`
pub fn unztell(file: unzFile) -> c_long {
    if file.is_null() {
        return UNZ_PARAMERROR as c_long;
    }
    unsafe {
        let s = file as *mut unz_s;
        let pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR as c_long;
        }

        (*pfile_in_zip_read_info).stream.total_out as c_long
    }
}

/// Raven `unzeof` — whether the current file's decompression stream is exhausted.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:1193-1209`
pub fn unzeof(file: unzFile) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        let pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        if (*pfile_in_zip_read_info).rest_read_uncompressed == 0 {
            1
        } else {
            0
        }
    }
}

/// Raven `unzGetLocalExtrafield` — reads the current file's local-header extra field into `buf`
/// (or, if `buf` is null, reports its size).
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:1225-1263`
pub fn unzGetLocalExtrafield(file: unzFile, buf: *mut (), len: c_uint) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        let pfile_in_zip_read_info = (*s).pfile_in_zip_read;

        if pfile_in_zip_read_info.is_null() {
            return UNZ_PARAMERROR;
        }

        let size_to_read: uLong = (*pfile_in_zip_read_info).size_local_extrafield as uLong
            - (*pfile_in_zip_read_info).pos_local_extrafield;

        if buf.is_null() {
            return size_to_read as c_int;
        }

        let read_now: uInt = if (len as uLong) > size_to_read {
            size_to_read as uInt
        } else {
            len as uInt
        };

        if read_now == 0 {
            return 0;
        }

        if libc::fseek(
            (*pfile_in_zip_read_info).file,
            ((*pfile_in_zip_read_info).offset_local_extrafield
                + (*pfile_in_zip_read_info).pos_local_extrafield) as c_long,
            SEEK_SET,
        ) != 0
        {
            return UNZ_ERRNO;
        }

        if libc::fread(
            buf as *mut libc::c_void,
            size_to_read as usize,
            1,
            (*pfile_in_zip_read_info).file,
        ) != 1
        {
            return UNZ_ERRNO;
        }

        read_now as c_int
    }
}

/// Raven `unzGetGlobalComment` — reads the archive's global comment into `szComment`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:1310-1335`
pub fn unzGetGlobalComment(file: unzFile, szComment: *mut c_char, uSizeBuf: uLong) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;

        let mut uReadThis: uLong = uSizeBuf;
        if uReadThis > (*s).gi.size_comment {
            uReadThis = (*s).gi.size_comment;
        }

        if libc::fseek((*s).file, ((*s).central_pos + 22) as c_long, SEEK_SET) != 0 {
            return UNZ_ERRNO;
        }

        if uReadThis > 0 {
            *szComment = 0;
            if libc::fread(
                szComment as *mut libc::c_void,
                uReadThis as usize,
                1,
                (*s).file,
            ) != 1
            {
                return UNZ_ERRNO;
            }
        }

        if !szComment.is_null() && uSizeBuf > (*s).gi.size_comment {
            *szComment.add((*s).gi.size_comment as usize) = 0;
        }
        uReadThis as c_int
    }
}

/// Raven `unzStringFileNameCompare` — compares two filenames per the requested case-sensitivity
/// mode.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:318-327`
pub fn unzStringFileNameCompare(
    fileName1: *const c_char,
    fileName2: *const c_char,
    mut iCaseSensitivity: c_int,
) -> c_int {
    if iCaseSensitivity == 0 {
        iCaseSensitivity = CASESENSITIVITYDEFAULTVALUE;
    }

    if iCaseSensitivity == 1 {
        return unsafe { libc::strcmp(fileName1, fileName2) };
    }

    strcmpcasenosensitive_internal(fileName1, fileName2)
}

/// Raven `unzlocal_GetCurrentFileInfoInternal` — parses the central-directory record at the
/// current position into `pfile_info`/`pfile_info_internal`, and optionally reads the filename,
/// extra field, and comment out alongside it.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:558-713`
pub fn unzlocal_GetCurrentFileInfoInternal(
    file: unzFile,
    pfile_info: *mut unz_file_info,
    pfile_info_internal: *mut unz_file_info_internal,
    szFileName: *mut c_char,
    fileNameBufferSize: uLong,
    extraField: *mut (),
    extraFieldBufferSize: uLong,
    szComment: *mut c_char,
    commentBufferSize: uLong,
) -> c_int {
    let mut err = UNZ_OK;
    let mut lSeek: c_long = 0;

    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        // §19: `file_info`/`file_info_internal` are read field-by-field below before any use;
        // zero-init to avoid reading uninitialized locals if a `getShort`/`getLong` call fails
        // partway through.
        let mut file_info: unz_file_info = core::mem::zeroed();
        let file_info_internal: unz_file_info_internal;
        let mut uMagic: uLong = 0;

        if libc::fseek(
            (*s).file,
            ((*s).pos_in_central_dir + (*s).byte_before_the_zipfile) as c_long,
            SEEK_SET,
        ) != 0
        {
            err = UNZ_ERRNO;
        }

        /* we check the magic */
        if err == UNZ_OK {
            if unzlocal_getLong((*s).file, &mut uMagic) != UNZ_OK {
                err = UNZ_ERRNO;
            } else if uMagic != 0x02014b50 {
                err = UNZ_BADZIPFILE;
            }
        }
        if unzlocal_getShort((*s).file, &mut file_info.version) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.version_needed) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.flag) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.compression_method) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, &mut file_info.dosDate) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        unzlocal_DosDateToTmuDate(file_info.dosDate, &mut file_info.tmu_date);

        if unzlocal_getLong((*s).file, &mut file_info.crc) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, &mut file_info.compressed_size) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, &mut file_info.uncompressed_size) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.size_filename) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.size_file_extra) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.size_file_comment) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.disk_num_start) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut file_info.internal_fa) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, &mut file_info.external_fa) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        let mut offset_curfile: uLong = 0;
        if unzlocal_getLong((*s).file, &mut offset_curfile) != UNZ_OK {
            err = UNZ_ERRNO;
        }
        file_info_internal = unz_file_info_internal { offset_curfile };

        lSeek += file_info.size_filename as c_long;
        if err == UNZ_OK && !szFileName.is_null() {
            let uSizeRead: uLong;
            if file_info.size_filename < fileNameBufferSize {
                *szFileName.add(file_info.size_filename as usize) = 0;
                uSizeRead = file_info.size_filename;
            } else {
                uSizeRead = fileNameBufferSize;
            }

            if file_info.size_filename > 0 && fileNameBufferSize > 0 {
                if libc::fread(
                    szFileName as *mut libc::c_void,
                    uSizeRead as usize,
                    1,
                    (*s).file,
                ) != 1
                {
                    err = UNZ_ERRNO;
                }
            }
            lSeek -= uSizeRead as c_long;
        }

        if err == UNZ_OK && !extraField.is_null() {
            let uSizeRead: uLong = if file_info.size_file_extra < extraFieldBufferSize {
                file_info.size_file_extra
            } else {
                extraFieldBufferSize
            };

            if lSeek != 0 {
                if libc::fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                    lSeek = 0;
                } else {
                    err = UNZ_ERRNO;
                }
            }
            if file_info.size_file_extra > 0 && extraFieldBufferSize > 0 {
                if libc::fread(
                    extraField as *mut libc::c_void,
                    uSizeRead as usize,
                    1,
                    (*s).file,
                ) != 1
                {
                    err = UNZ_ERRNO;
                }
            }
            lSeek += (file_info.size_file_extra - uSizeRead) as c_long;
        } else {
            lSeek += file_info.size_file_extra as c_long;
        }

        if err == UNZ_OK && !szComment.is_null() {
            let uSizeRead: uLong;
            if file_info.size_file_comment < commentBufferSize {
                *szComment.add(file_info.size_file_comment as usize) = 0;
                uSizeRead = file_info.size_file_comment;
            } else {
                uSizeRead = commentBufferSize;
            }

            if lSeek != 0 {
                if libc::fseek((*s).file, lSeek, SEEK_CUR) == 0 {
                    lSeek = 0;
                } else {
                    err = UNZ_ERRNO;
                }
            }
            if file_info.size_file_comment > 0 && commentBufferSize > 0 {
                if libc::fread(
                    szComment as *mut libc::c_void,
                    uSizeRead as usize,
                    1,
                    (*s).file,
                ) != 1
                {
                    err = UNZ_ERRNO;
                }
            }
            lSeek += (file_info.size_file_comment - uSizeRead) as c_long;
        } else {
            lSeek += file_info.size_file_comment as c_long;
        }

        if err == UNZ_OK && !pfile_info.is_null() {
            *pfile_info = file_info;
        }

        if err == UNZ_OK && !pfile_info_internal.is_null() {
            *pfile_info_internal = file_info_internal;
        }

        let _ = lSeek;
        err
    }
}

/// Raven `unzlocal_CheckCurrentFileCoherencyHeader` — re-reads the current file's local header
/// and cross-checks it against the central-directory record already loaded on `s`.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:877-957`
pub fn unzlocal_CheckCurrentFileCoherencyHeader(
    s: *mut unz_s,
    piSizeVar: *mut uInt,
    poffset_local_extrafield: *mut uLong,
    psize_local_extrafield: *mut uInt,
) -> c_int {
    let mut uMagic: uLong = 0;
    let mut uData: uLong = 0;
    let mut uFlags: uLong = 0;
    let mut size_filename: uLong = 0;
    let mut size_extra_field: uLong = 0;
    let mut err = UNZ_OK;

    unsafe {
        *piSizeVar = 0;
        *poffset_local_extrafield = 0;
        *psize_local_extrafield = 0;

        if libc::fseek(
            (*s).file,
            ((*s).cur_file_info_internal.offset_curfile + (*s).byte_before_the_zipfile) as c_long,
            SEEK_SET,
        ) != 0
        {
            return UNZ_ERRNO;
        }

        if err == UNZ_OK {
            if unzlocal_getLong((*s).file, &mut uMagic) != UNZ_OK {
                err = UNZ_ERRNO;
            } else if uMagic != 0x04034b50 {
                err = UNZ_BADZIPFILE;
            }
        }
        if unzlocal_getShort((*s).file, &mut uData) != UNZ_OK {
            err = UNZ_ERRNO;
        }
        /*
        else if ((err==UNZ_OK) && (uData!=s->cur_file_info.wVersion))
            err=UNZ_BADZIPFILE;
        */
        if unzlocal_getShort((*s).file, &mut uFlags) != UNZ_OK {
            err = UNZ_ERRNO;
        }

        if unzlocal_getShort((*s).file, &mut uData) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if err == UNZ_OK && uData != (*s).cur_file_info.compression_method {
            err = UNZ_BADZIPFILE;
        }

        if err == UNZ_OK
            && (*s).cur_file_info.compression_method != 0
            && (*s).cur_file_info.compression_method != ZF_DEFLATED
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
            /* date/time */
            err = UNZ_ERRNO;
        }

        if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
            /* crc */
            err = UNZ_ERRNO;
        } else if err == UNZ_OK && uData != (*s).cur_file_info.crc && (uFlags & 8) == 0 {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
            /* size compr */
            err = UNZ_ERRNO;
        } else if err == UNZ_OK && uData != (*s).cur_file_info.compressed_size && (uFlags & 8) == 0
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getLong((*s).file, &mut uData) != UNZ_OK {
            /* size uncompr */
            err = UNZ_ERRNO;
        } else if err == UNZ_OK
            && uData != (*s).cur_file_info.uncompressed_size
            && (uFlags & 8) == 0
        {
            err = UNZ_BADZIPFILE;
        }

        if unzlocal_getShort((*s).file, &mut size_filename) != UNZ_OK {
            err = UNZ_ERRNO;
        } else if err == UNZ_OK && size_filename != (*s).cur_file_info.size_filename {
            err = UNZ_BADZIPFILE;
        }

        *piSizeVar += size_filename as uInt;

        if unzlocal_getShort((*s).file, &mut size_extra_field) != UNZ_OK {
            err = UNZ_ERRNO;
        }
        *poffset_local_extrafield = (*s).cur_file_info_internal.offset_curfile
            + SIZEZIPLOCALHEADER as uLong
            + size_filename;
        *psize_local_extrafield = size_extra_field as uInt;

        *piSizeVar += size_extra_field as uInt;

        err
    }
}

/// Raven `unzGetCurrentFileInfo` — public wrapper over
/// [`unzlocal_GetCurrentFileInfoInternal`] with no internal-info out-param.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:722-731`
pub fn unzGetCurrentFileInfo(
    file: unzFile,
    pfile_info: *mut unz_file_info,
    szFileName: *mut c_char,
    fileNameBufferSize: uLong,
    extraField: *mut (),
    extraFieldBufferSize: uLong,
    szComment: *mut c_char,
    commentBufferSize: uLong,
) -> c_int {
    unzlocal_GetCurrentFileInfoInternal(
        file,
        pfile_info,
        core::ptr::null_mut(),
        szFileName,
        fileNameBufferSize,
        extraField,
        extraFieldBufferSize,
        szComment,
        commentBufferSize,
    )
}

/// Raven `unzGoToFirstFile` — repositions `file` at the first entry in the central directory.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:737-751`
pub fn unzGoToFirstFile(file: unzFile) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        (*s).pos_in_central_dir = (*s).offset_central_dir;
        (*s).num_file = 0;
        let err = unzlocal_GetCurrentFileInfoInternal(
            file,
            &mut (*s).cur_file_info,
            &mut (*s).cur_file_info_internal,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = (err == UNZ_OK) as uLong;
        err
    }
}

/// Raven `unzGoToNextFile` — advances `file` to the next central-directory entry.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:759-780`
pub fn unzGoToNextFile(file: unzFile) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;
        if (*s).current_file_ok == 0 {
            return UNZ_END_OF_LIST_OF_FILE;
        }
        if (*s).num_file + 1 == (*s).gi.number_entry {
            return UNZ_END_OF_LIST_OF_FILE;
        }

        (*s).pos_in_central_dir += SIZECENTRALDIRITEM as uLong
            + (*s).cur_file_info.size_filename
            + (*s).cur_file_info.size_file_extra
            + (*s).cur_file_info.size_file_comment;
        (*s).num_file += 1;
        let err = unzlocal_GetCurrentFileInfoInternal(
            file,
            &mut (*s).cur_file_info,
            &mut (*s).cur_file_info_internal,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = (err == UNZ_OK) as uLong;
        err
    }
}

/// Raven `unzSetCurrentFileInfoPosition` — jumps `file`'s central-directory cursor to `pos` and
/// re-parses the entry there.
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:802-817`
pub fn unzSetCurrentFileInfoPosition(file: unzFile, pos: c_ulong) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }
    unsafe {
        let s = file as *mut unz_s;

        (*s).pos_in_central_dir = pos as uLong;
        let err = unzlocal_GetCurrentFileInfoInternal(
            file,
            &mut (*s).cur_file_info,
            &mut (*s).cur_file_info_internal,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
        );
        (*s).current_file_ok = (err == UNZ_OK) as uLong;
    }
    UNZ_OK
}

/// Raven `unzLocateFile` — scans the central directory for `szFileName`, leaving `file`
/// positioned there on success (and restored to its prior position on failure).
///
/// Source: `oracle/codemp/qcommon/unzip.cpp:827-867`
pub fn unzLocateFile(file: unzFile, szFileName: *const c_char, iCaseSensitivity: c_int) -> c_int {
    if file.is_null() {
        return UNZ_PARAMERROR;
    }

    unsafe {
        if libc::strlen(szFileName) >= UNZ_MAXFILENAMEINZIP as usize {
            return UNZ_PARAMERROR;
        }

        let s = file as *mut unz_s;
        if (*s).current_file_ok == 0 {
            return UNZ_END_OF_LIST_OF_FILE;
        }

        let num_fileSaved: uLong = (*s).num_file;
        let pos_in_central_dirSaved: uLong = (*s).pos_in_central_dir;

        let mut err = unzGoToFirstFile(file);

        while err == UNZ_OK {
            let mut szCurrentFileName: [c_char; UNZ_MAXFILENAMEINZIP as usize + 1] =
                [0; UNZ_MAXFILENAMEINZIP as usize + 1];
            unzGetCurrentFileInfo(
                file,
                core::ptr::null_mut(),
                szCurrentFileName.as_mut_ptr(),
                (szCurrentFileName.len() - 1) as uLong,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                0,
            );
            if unzStringFileNameCompare(szCurrentFileName.as_ptr(), szFileName, iCaseSensitivity)
                == 0
            {
                return UNZ_OK;
            }
            err = unzGoToNextFile(file);
        }

        (*s).num_file = num_fileSaved;
        (*s).pos_in_central_dir = pos_in_central_dirSaved;
        err
    }
}
