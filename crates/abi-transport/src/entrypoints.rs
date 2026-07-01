use core::ffi::{c_int, c_void};

pub type AbiCommand = c_int;
pub type AbiWord = isize;
pub type RawSyscall = *const c_void;
pub type RawImportTable = *mut c_void;
pub type RawExportTable = *mut c_void;

pub type RawDllEntry = extern "C" fn(syscall: RawSyscall);
pub type RawVmMain = extern "C" fn(
    command: AbiCommand,
    arg0: AbiWord,
    arg1: AbiWord,
    arg2: AbiWord,
    arg3: AbiWord,
    arg4: AbiWord,
    arg5: AbiWord,
    arg6: AbiWord,
    arg7: AbiWord,
    arg8: AbiWord,
    arg9: AbiWord,
    arg10: AbiWord,
    arg11: AbiWord,
) -> AbiWord;
pub type RawGetModuleApi =
    extern "C" fn(api_version: AbiCommand, import: RawImportTable) -> RawExportTable;
pub type RawGetGameApi = extern "C" fn(import: RawImportTable) -> RawExportTable;

pub mod qvm {
    use super::{AbiCommand, AbiWord, RawExportTable, RawImportTable, RawSyscall};

    /// Raven/OpenJK QVM-style `dllEntry` export.
    #[no_mangle]
    pub extern "C" fn dllEntry(_syscall: RawSyscall) {}

    /// Raven/OpenJK QVM-style `vmMain` export.
    #[no_mangle]
    #[allow(clippy::too_many_arguments)]
    pub extern "C" fn vmMain(
        _command: AbiCommand,
        _arg0: AbiWord,
        _arg1: AbiWord,
        _arg2: AbiWord,
        _arg3: AbiWord,
        _arg4: AbiWord,
        _arg5: AbiWord,
        _arg6: AbiWord,
        _arg7: AbiWord,
        _arg8: AbiWord,
        _arg9: AbiWord,
        _arg10: AbiWord,
        _arg11: AbiWord,
    ) -> AbiWord {
        0
    }

    /// Raven/OpenJK table-style `GetModuleAPI` export.
    #[no_mangle]
    pub extern "C" fn GetModuleAPI(
        _api_version: AbiCommand,
        _import: RawImportTable,
    ) -> RawExportTable {
        core::ptr::null_mut()
    }
}

pub mod sp_game {
    use super::{RawExportTable, RawImportTable};

    /// Raven/OpenJK SP game `GetGameAPI` export.
    #[no_mangle]
    pub extern "C" fn GetGameAPI(_import: RawImportTable) -> RawExportTable {
        core::ptr::null_mut()
    }
}
