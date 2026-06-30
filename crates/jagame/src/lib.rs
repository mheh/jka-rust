use core::ffi::c_void;

/// Raven/OpenJK SP game `GetGameAPI` export.
#[no_mangle]
pub extern "C" fn GetGameAPI(_import: *mut c_void) -> *mut c_void {
    core::ptr::null_mut()
}
