use core::ffi::{c_int, c_void};

/// MP UI engine-to-module syscall callback placeholder.
///
/// TODO: Replace with the typed MP UI syscall callback ABI once the entry
/// symbols are wired to the generated ABI surfaces.
pub type EngineSyscall = *const c_void;

/// Raven/OpenJK MP UI `dllEntry` export.
#[no_mangle]
pub extern "C" fn dllEntry(_syscall: EngineSyscall) {}

/// Raven/OpenJK MP UI `vmMain` export.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn vmMain(
    _command: c_int,
    _arg0: isize,
    _arg1: isize,
    _arg2: isize,
    _arg3: isize,
    _arg4: isize,
    _arg5: isize,
    _arg6: isize,
    _arg7: isize,
    _arg8: isize,
    _arg9: isize,
    _arg10: isize,
    _arg11: isize,
) -> isize {
    0
}

/// OpenJK MP UI table ABI export.
#[no_mangle]
pub extern "C" fn GetModuleAPI(_api_version: c_int, _import: *mut c_void) -> *mut c_void {
    core::ptr::null_mut()
}
