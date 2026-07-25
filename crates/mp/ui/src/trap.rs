//! `mod trap` outbound-call wrappers for every `trap_*` the ui manifest
//! records a call to (79 of the 151 `UI_*` imports; the rest gain wrappers
//! when a call site appears). One non-generic fn per call. C signatures:
//! `oracle/codemp/ui/ui_syscalls.c`; token mapping:
//! `crates/mp/abi/src/ui/syscalls/`.
//!
//! Hand-maintained, on the `mp_game::trap` template (campaign #20): wrappers
//! own the C seam — string args are `&str`, engine-filled `char` buffers come
//! back as `String` (`buffer_len` keeps the engine-side truncation width at
//! the call site), `qboolean` is `bool`, out-params are return values. The raw
//! pointer/`Args` shapes live only inside this module and `mp_abi`. Ghoul2
//! instances stay opaque engine tokens (`*mut c_void`) — the module never
//! reads through them.
//!
//! String encoding (#13 discipline). Asset and identifier arguments — file
//! paths, shader/skin/font/bone names, cvar names, string-package keys —
//! cross as UTF-8-transparent `cstr`/`buf_to_string`, matching
//! `mp_game::trap`. Arguments and buffers carrying free text the engine
//! treats as opaque bytes — argv, configstrings, drawn text, localized
//! string-package text, console command text, LAN server info/address/status
//! — use the bijective Latin-1 pair (`string_to_latin1`/`latin1_to_string`)
//! so all 256 byte values round-trip, exactly as
//! `mp_game::trap::SendServerCommand` does.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::{null, null_mut};
use std::ffi::CString;

use mp_abi::ui::public::ui_client_state_t::uiClientState_t;
use mp_abi::ui::syscalls::UI_ANYLANGUAGE_READCHARFROMSTRING::{
    UiAnylanguageReadcharfromstring, UiAnylanguageReadcharfromstringArgs,
};
use mp_abi::ui::syscalls::UI_ARGC::{UiArgc, UiArgcArgs};
use mp_abi::ui::syscalls::UI_ARGV::{UiArgv, UiArgvArgs};
use mp_abi::ui::syscalls::UI_CIN_DRAWCINEMATIC::{UiCinDrawcinematic, UiCinDrawcinematicArgs};
use mp_abi::ui::syscalls::UI_CIN_PLAYCINEMATIC::{UiCinPlaycinematic, UiCinPlaycinematicArgs};
use mp_abi::ui::syscalls::UI_CIN_RUNCINEMATIC::{UiCinRuncinematic, UiCinRuncinematicArgs};
use mp_abi::ui::syscalls::UI_CIN_SETEXTENTS::{UiCinSetextents, UiCinSetextentsArgs};
use mp_abi::ui::syscalls::UI_CIN_STOPCINEMATIC::{UiCinStopcinematic, UiCinStopcinematicArgs};
use mp_abi::ui::syscalls::UI_CMD_EXECUTETEXT::{UiCmdExecutetext, UiCmdExecutetextArgs};
use mp_abi::ui::syscalls::UI_CVAR_REGISTER::{UiCvarRegister, UiCvarRegisterArgs};
use mp_abi::ui::syscalls::UI_CVAR_SET::{UiCvarSet, UiCvarSetArgs};
use mp_abi::ui::syscalls::UI_CVAR_SETVALUE::{UiCvarSetvalue, UiCvarSetvalueArgs};
use mp_abi::ui::syscalls::UI_CVAR_UPDATE::{UiCvarUpdate, UiCvarUpdateArgs};
use mp_abi::ui::syscalls::UI_CVAR_VARIABLESTRINGBUFFER::{
    UiCvarVariablestringbuffer, UiCvarVariablestringbufferArgs,
};
use mp_abi::ui::syscalls::UI_CVAR_VARIABLEVALUE::{UiCvarVariablevalue, UiCvarVariablevalueArgs};
use mp_abi::ui::syscalls::UI_ERROR::{UiError, UiErrorArgs};
use mp_abi::ui::syscalls::UI_FS_FCLOSEFILE::{UiFsFclosefile, UiFsFclosefileArgs};
use mp_abi::ui::syscalls::UI_FS_FOPENFILE::{UiFsFopenfile, UiFsFopenfileArgs};
use mp_abi::ui::syscalls::UI_FS_GETFILELIST::{UiFsGetfilelist, UiFsGetfilelistArgs};
use mp_abi::ui::syscalls::UI_FS_READ::{UiFsRead, UiFsReadArgs};
use mp_abi::ui::syscalls::UI_FS_WRITE::{UiFsWrite, UiFsWriteArgs};
use mp_abi::ui::syscalls::UI_G2_ADDBOLT::{UiG2Addbolt, UiG2AddboltArgs};
use mp_abi::ui::syscalls::UI_G2_ATTACHG2MODEL::{UiG2Attachg2model, UiG2Attachg2modelArgs};
use mp_abi::ui::syscalls::UI_G2_CLEANMODELS::{UiG2Cleanmodels, UiG2CleanmodelsArgs};
use mp_abi::ui::syscalls::UI_G2_GETBOLT::{UiG2Getbolt, UiG2GetboltArgs};
use mp_abi::ui::syscalls::UI_G2_GETGLANAME::{UiG2Getglaname, UiG2GetglanameArgs};
use mp_abi::ui::syscalls::UI_G2_HASGHOUL2MODELONINDEX::{
    UiG2Hasghoul2Modelonindex, UiG2Hasghoul2ModelonindexArgs,
};
use mp_abi::ui::syscalls::UI_G2_HAVEWEGHOULMODELS::{
    UiG2Haveweghoulmodels, UiG2HaveweghoulmodelsArgs,
};
use mp_abi::ui::syscalls::UI_G2_INITGHOUL2MODEL::{UiG2Initghoul2Model, UiG2Initghoul2ModelArgs};
use mp_abi::ui::syscalls::UI_G2_PLAYANIM::{UiG2Playanim, UiG2PlayanimArgs};
use mp_abi::ui::syscalls::UI_G2_REMOVEGHOUL2MODEL::{
    UiG2Removeghoul2Model, UiG2Removeghoul2ModelArgs,
};
use mp_abi::ui::syscalls::UI_G2_SETSKIN::{UiG2Setskin, UiG2SetskinArgs};
use mp_abi::ui::syscalls::UI_G2_SETTIME::{UiG2Settime, UiG2SettimeArgs};
use mp_abi::ui::syscalls::UI_GETCLIENTSTATE::{UiGetclientstate, UiGetclientstateArgs};
use mp_abi::ui::syscalls::UI_GETCONFIGSTRING::{UiGetconfigstring, UiGetconfigstringArgs};
use mp_abi::ui::syscalls::UI_GETGLCONFIG::{UiGetglconfig, UiGetglconfigArgs};
use mp_abi::ui::syscalls::UI_KEY_CLEARSTATES::{UiKeyClearstates, UiKeyClearstatesArgs};
use mp_abi::ui::syscalls::UI_KEY_GETCATCHER::{UiKeyGetcatcher, UiKeyGetcatcherArgs};
use mp_abi::ui::syscalls::UI_KEY_SETCATCHER::{UiKeySetcatcher, UiKeySetcatcherArgs};
use mp_abi::ui::syscalls::UI_LANGUAGE_USESSPACES::{
    UiLanguageUsesspaces, UiLanguageUsesspacesArgs,
};
use mp_abi::ui::syscalls::UI_LAN_ADDSERVER::{UiLanAddserver, UiLanAddserverArgs};
use mp_abi::ui::syscalls::UI_LAN_COMPARESERVERS::{UiLanCompareservers, UiLanCompareserversArgs};
use mp_abi::ui::syscalls::UI_LAN_GETSERVERADDRESSSTRING::{
    UiLanGetserveraddressstring, UiLanGetserveraddressstringArgs,
};
use mp_abi::ui::syscalls::UI_LAN_GETSERVERCOUNT::{UiLanGetservercount, UiLanGetservercountArgs};
use mp_abi::ui::syscalls::UI_LAN_GETSERVERINFO::{UiLanGetserverinfo, UiLanGetserverinfoArgs};
use mp_abi::ui::syscalls::UI_LAN_GETSERVERPING::{UiLanGetserverping, UiLanGetserverpingArgs};
use mp_abi::ui::syscalls::UI_LAN_LOADCACHEDSERVERS::UiLanLoadcachedservers;
use mp_abi::ui::syscalls::UI_LAN_MARKSERVERVISIBLE::{
    UiLanMarkservervisible, UiLanMarkservervisibleArgs,
};
use mp_abi::ui::syscalls::UI_LAN_REMOVESERVER::{UiLanRemoveserver, UiLanRemoveserverArgs};
use mp_abi::ui::syscalls::UI_LAN_RESETPINGS::{UiLanResetpings, UiLanResetpingsArgs};
use mp_abi::ui::syscalls::UI_LAN_SAVECACHEDSERVERS::UiLanSavecachedservers;
use mp_abi::ui::syscalls::UI_LAN_SERVERISVISIBLE::{
    UiLanServerisvisible, UiLanServerisvisibleArgs,
};
use mp_abi::ui::syscalls::UI_LAN_SERVERSTATUS::{UiLanServerstatus, UiLanServerstatusArgs};
use mp_abi::ui::syscalls::UI_LAN_UPDATEVISIBLEPINGS::{
    UiLanUpdatevisiblepings, UiLanUpdatevisiblepingsArgs,
};
use mp_abi::ui::syscalls::UI_MILLISECONDS::{UiMilliseconds, UiMillisecondsArgs};
use mp_abi::ui::syscalls::UI_PC_FREE_SOURCE::{UiPcFreeSource, UiPcFreeSourceArgs};
use mp_abi::ui::syscalls::UI_PC_LOAD_GLOBAL_DEFINES::{
    UiPcLoadGlobalDefines, UiPcLoadGlobalDefinesArgs,
};
use mp_abi::ui::syscalls::UI_PC_LOAD_SOURCE::{UiPcLoadSource, UiPcLoadSourceArgs};
use mp_abi::ui::syscalls::UI_PC_READ_TOKEN::{UiPcReadToken, UiPcReadTokenArgs};
use mp_abi::ui::syscalls::UI_PC_REMOVE_ALL_GLOBAL_DEFINES::{
    UiPcRemoveAllGlobalDefines, UiPcRemoveAllGlobalDefinesArgs,
};
use mp_abi::ui::syscalls::UI_PC_SOURCE_FILE_AND_LINE::{
    UiPcSourceFileAndLine, UiPcSourceFileAndLineArgs,
};
use mp_abi::ui::syscalls::UI_PRINT::{UiPrint, UiPrintArgs};
use mp_abi::ui::syscalls::UI_REAL_TIME::{UiRealTime, UiRealTimeArgs};
use mp_abi::ui::syscalls::UI_R_ADDREFENTITYTOSCENE::{
    UiRAddrefentitytoscene, UiRAddrefentitytosceneArgs,
};
use mp_abi::ui::syscalls::UI_R_DRAWSTRETCHPIC::{UiRDrawstretchpic, UiRDrawstretchpicArgs};
use mp_abi::ui::syscalls::UI_R_FONT_DRAWSTRING::{UiRFontDrawstring, UiRFontDrawstringArgs};
use mp_abi::ui::syscalls::UI_R_FONT_STRHEIGHTPIXELS::{
    UiRFontStrheightpixels, UiRFontStrheightpixelsArgs,
};
use mp_abi::ui::syscalls::UI_R_FONT_STRLENPIXELS::{UiRFontStrlenpixels, UiRFontStrlenpixelsArgs};
use mp_abi::ui::syscalls::UI_R_REGISTERFONT::{UiRRegisterfont, UiRRegisterfontArgs};
use mp_abi::ui::syscalls::UI_R_REGISTERSHADERNOMIP::{
    UiRRegistershadernomip, UiRRegistershadernomipArgs,
};
use mp_abi::ui::syscalls::UI_R_REGISTERSKIN::{UiRRegisterskin, UiRRegisterskinArgs};
use mp_abi::ui::syscalls::UI_R_SETCOLOR::{UiRSetcolor, UiRSetcolorArgs};
use mp_abi::ui::syscalls::UI_R_SHADERNAMEFROMINDEX::{
    UiRShadernamefromindex, UiRShadernamefromindexArgs,
};
use mp_abi::ui::syscalls::UI_SP_GETLANGUAGENAME::{UiSpGetlanguagename, UiSpGetlanguagenameArgs};
use mp_abi::ui::syscalls::UI_SP_GETNUMLANGUAGES::{UiSpGetnumlanguages, UiSpGetnumlanguagesArgs};
use mp_abi::ui::syscalls::UI_SP_GETSTRINGTEXTSTRING::{
    UiSpGetstringtextstring, UiSpGetstringtextstringArgs,
};
use mp_abi::ui::syscalls::UI_S_REGISTERSOUND::{UiSRegistersound, UiSRegistersoundArgs};
use mp_abi::ui::syscalls::UI_S_STARTLOCALSOUND::{UiSStartlocalsound, UiSStartlocalsoundArgs};
use mp_abi::ui::syscalls::UI_UPDATESCREEN::{UiUpdatescreen, UiUpdatescreenArgs};
use mp_abi::Execute;
use mp_engine_select::Engine;
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::qcommon::qtime_t;
use mp_qshared::shared::{
    fileHandle_t, fsMode_t, mdxaBone_t, pc_token_t, qhandle_t, sfxHandle_t, vec3_t, vec4_t,
    vmCvar_t,
};
use native_string::{buf_to_string, cstr, latin1_to_string, string_to_latin1};

/// Raven `trap_AnyLanguage_ReadCharFromString` — `UI_ANYLANGUAGE_READCHARFROMSTRING`
/// (token: `mp_abi::ui::syscalls::UI_ANYLANGUAGE_READCHARFROMSTRING`).
///
/// C: `unsigned int trap_AnyLanguage_ReadCharFromString(const char *psText, int *piAdvanceCount, qboolean *pbIsTrailingPunctuation)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:146-149`
///
/// Returns `(char, advance_count, is_trailing_punctuation)`. `psText` is raw
/// wire bytes, not `&str`: the engine's advance count indexes those bytes, so
/// callers walk the slice by that count directly.
pub fn AnyLanguage_ReadCharFromString(engine: &Engine, psText: &[u8]) -> (c_uint, c_int, bool) {
    let mut text = psText.to_vec();
    text.push(0);
    let mut advance_count: c_int = 0;
    let mut trailing_punctuation: c_int = 0;
    let ch = <Engine as Execute<UiAnylanguageReadcharfromstring>>::execute(
        engine,
        UiAnylanguageReadcharfromstringArgs::new(
            text.as_ptr() as *const c_char,
            &mut advance_count,
            &mut trailing_punctuation,
        ),
    );
    (ch, advance_count, trailing_punctuation != 0)
}

/// Raven `trap_Argc` — `UI_ARGC` (token: `mp_abi::ui::syscalls::UI_ARGC`).
///
/// C: `int trap_Argc(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:71-73`
pub fn Argc(engine: &Engine) -> c_int {
    <Engine as Execute<UiArgc>>::execute(engine, UiArgcArgs::new())
}

/// Raven `trap_Argv` — `UI_ARGV` (token: `mp_abi::ui::syscalls::UI_ARGV`).
///
/// C: `void trap_Argv(int n, char *buffer, int bufferLength)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:75-77`
pub fn Argv(engine: &Engine, n: c_int, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    // SAFETY: `buffer` outlives the synchronous syscall the Args feed.
    let args =
        unsafe { UiArgvArgs::new(n, buffer.as_mut_ptr() as *mut c_char, buffer_len as c_int) };
    <Engine as Execute<UiArgv>>::execute(engine, args);
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_CIN_DrawCinematic` — `UI_CIN_DRAWCINEMATIC`
/// (token: `mp_abi::ui::syscalls::UI_CIN_DRAWCINEMATIC`).
///
/// C: `void trap_CIN_DrawCinematic(int handle)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:423-425`
pub fn CIN_DrawCinematic(engine: &Engine, handle: c_int) {
    <Engine as Execute<UiCinDrawcinematic>>::execute(engine, UiCinDrawcinematicArgs::new(handle))
}

/// Raven `trap_CIN_PlayCinematic` — `UI_CIN_PLAYCINEMATIC`
/// (token: `mp_abi::ui::syscalls::UI_CIN_PLAYCINEMATIC`).
///
/// C: `int trap_CIN_PlayCinematic(const char *arg0, int xpos, int ypos, int width, int height, int bits)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:405-407`
#[allow(clippy::too_many_arguments)]
pub fn CIN_PlayCinematic(
    engine: &Engine,
    arg0: &str,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    bits: c_int,
) -> c_int {
    let arg0_c = cstr(arg0);
    <Engine as Execute<UiCinPlaycinematic>>::execute(
        engine,
        UiCinPlaycinematicArgs::new(arg0_c.as_ptr(), xpos, ypos, width, height, bits),
    )
}

/// Raven `trap_CIN_RunCinematic` — `UI_CIN_RUNCINEMATIC`
/// (token: `mp_abi::ui::syscalls::UI_CIN_RUNCINEMATIC`).
///
/// C: `e_status trap_CIN_RunCinematic(int handle)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:417-419`
pub fn CIN_RunCinematic(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<UiCinRuncinematic>>::execute(engine, UiCinRuncinematicArgs::new(handle))
}

/// Raven `trap_CIN_SetExtents` — `UI_CIN_SETEXTENTS`
/// (token: `mp_abi::ui::syscalls::UI_CIN_SETEXTENTS`).
///
/// C: `void trap_CIN_SetExtents(int handle, int x, int y, int w, int h)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:429-431`
pub fn CIN_SetExtents(engine: &Engine, handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    <Engine as Execute<UiCinSetextents>>::execute(
        engine,
        UiCinSetextentsArgs::new(handle, x, y, w, h),
    )
}

/// Raven `trap_CIN_StopCinematic` — `UI_CIN_STOPCINEMATIC`
/// (token: `mp_abi::ui::syscalls::UI_CIN_STOPCINEMATIC`).
///
/// C: `e_status trap_CIN_StopCinematic(int handle)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:411-413`
pub fn CIN_StopCinematic(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<UiCinStopcinematic>>::execute(engine, UiCinStopcinematicArgs::new(handle))
}

/// Raven `trap_Cmd_ExecuteText` — `UI_CMD_EXECUTETEXT`
/// (token: `mp_abi::ui::syscalls::UI_CMD_EXECUTETEXT`).
///
/// C: `void trap_Cmd_ExecuteText(int exec_when, const char *text)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:79-81`
pub fn Cmd_ExecuteText(engine: &Engine, exec_when: c_int, text: &str) {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<UiCmdExecutetext>>::execute(
        engine,
        UiCmdExecutetextArgs::new(exec_when, text_c.as_ptr()),
    )
}

/// Raven `trap_Cvar_Register` — `UI_CVAR_REGISTER`
/// (token: `mp_abi::ui::syscalls::UI_CVAR_REGISTER`).
///
/// C: `void trap_Cvar_Register(vmCvar_t *cvar, const char *var_name, const char *value, int flags)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:33-35`
pub fn Cvar_Register(
    engine: &Engine,
    cvar: Option<&mut vmCvar_t>,
    var_name: &str,
    value: &str,
    flags: c_int,
) {
    let cvar_ptr = cvar.map_or(null_mut(), |c| c as *mut vmCvar_t);
    let var_name_c = cstr(var_name);
    let value_c = cstr(value);
    // SAFETY: every pointer outlives the synchronous syscall the Args feed.
    let args =
        unsafe { UiCvarRegisterArgs::new(cvar_ptr, var_name_c.as_ptr(), value_c.as_ptr(), flags) };
    <Engine as Execute<UiCvarRegister>>::execute(engine, args)
}

/// Raven `trap_Cvar_Set` — `UI_CVAR_SET` (token: `mp_abi::ui::syscalls::UI_CVAR_SET`).
///
/// C: `void trap_Cvar_Set(const char *var_name, const char *value)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:41-43`
pub fn Cvar_Set(engine: &Engine, var_name: &str, value: &str) {
    let var_name_c = cstr(var_name);
    let value_c = cstr(value);
    // SAFETY: both strings outlive the synchronous syscall the Args feed.
    let args = unsafe { UiCvarSetArgs::new(var_name_c.as_ptr(), value_c.as_ptr()) };
    <Engine as Execute<UiCvarSet>>::execute(engine, args)
}

/// Raven `trap_Cvar_Set` with a NULL `value` — the engine substitutes the
/// cvar's `resetString`, i.e. a reset-to-default.
///
/// C: `trap_Cvar_Set(var_name, NULL)`
/// Source: `oracle/codemp/qcommon/cvar.cpp:322-323`, `oracle/codemp/ui/ui_main.c:6032-6033`
pub fn Cvar_Reset(engine: &Engine, var_name: &str) {
    let var_name_c = cstr(var_name);
    // SAFETY: the name outlives the synchronous syscall the Args feed.
    let args = unsafe { UiCvarSetArgs::new(var_name_c.as_ptr(), null()) };
    <Engine as Execute<UiCvarSet>>::execute(engine, args)
}

/// Raven `trap_Cvar_SetValue` — `UI_CVAR_SETVALUE`
/// (token: `mp_abi::ui::syscalls::UI_CVAR_SETVALUE`).
///
/// C: `void trap_Cvar_SetValue(const char *var_name, float value)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:55-57`
pub fn Cvar_SetValue(engine: &Engine, var_name: &str, value: f32) {
    let var_name_c = cstr(var_name);
    <Engine as Execute<UiCvarSetvalue>>::execute(
        engine,
        UiCvarSetvalueArgs::new(var_name_c.as_ptr(), value),
    )
}

/// Raven `trap_Cvar_Update` — `UI_CVAR_UPDATE`
/// (token: `mp_abi::ui::syscalls::UI_CVAR_UPDATE`).
///
/// C: `void trap_Cvar_Update(vmCvar_t *cvar)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:37-39`
pub fn Cvar_Update(engine: &Engine, cvar: &mut vmCvar_t) {
    <Engine as Execute<UiCvarUpdate>>::execute(engine, UiCvarUpdateArgs::new(cvar as *mut vmCvar_t))
}

/// Raven `trap_Cvar_VariableStringBuffer` — `UI_CVAR_VARIABLESTRINGBUFFER`
/// (token: `mp_abi::ui::syscalls::UI_CVAR_VARIABLESTRINGBUFFER`).
///
/// C: `void trap_Cvar_VariableStringBuffer(const char *var_name, char *buffer, int bufsize)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:51-53`
pub fn Cvar_VariableStringBuffer(engine: &Engine, var_name: &str, buffer_len: usize) -> String {
    let var_name_c = cstr(var_name);
    let mut buffer = vec![0u8; buffer_len];
    // SAFETY: name and buffer outlive the synchronous syscall the Args feed.
    let args = unsafe {
        UiCvarVariablestringbufferArgs::new(
            var_name_c.as_ptr(),
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        )
    };
    <Engine as Execute<UiCvarVariablestringbuffer>>::execute(engine, args);
    buf_to_string(&buffer)
}

/// Raven `trap_Cvar_VariableValue` — `UI_CVAR_VARIABLEVALUE`
/// (token: `mp_abi::ui::syscalls::UI_CVAR_VARIABLEVALUE`).
///
/// C: `float trap_Cvar_VariableValue(const char *var_name)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:45-49`
pub fn Cvar_VariableValue(engine: &Engine, var_name: &str) -> f32 {
    let var_name_c = cstr(var_name);
    <Engine as Execute<UiCvarVariablevalue>>::execute(
        engine,
        UiCvarVariablevalueArgs::new(var_name_c.as_ptr()),
    )
}

/// Raven `trap_Error` — `UI_ERROR` (token: `mp_abi::ui::syscalls::UI_ERROR`).
///
/// C: `void trap_Error(const char *string)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:25-27`
pub fn Error(engine: &Engine, string: &str) {
    let string_c = cstr(string);
    // SAFETY: the message outlives the synchronous syscall the Args feed.
    let args = unsafe { UiErrorArgs::new(string_c.as_ptr()) };
    <Engine as Execute<UiError>>::execute(engine, args)
}

/// Raven `trap_FS_FCloseFile` — `UI_FS_FCLOSEFILE`
/// (token: `mp_abi::ui::syscalls::UI_FS_FCLOSEFILE`).
///
/// C: `void trap_FS_FCloseFile(fileHandle_t f)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:95-97`
pub fn FS_FCloseFile(engine: &Engine, f: fileHandle_t) {
    <Engine as Execute<UiFsFclosefile>>::execute(engine, UiFsFclosefileArgs::new(f))
}

/// Raven `trap_FS_FOpenFile` — `UI_FS_FOPENFILE`
/// (token: `mp_abi::ui::syscalls::UI_FS_FOPENFILE`).
///
/// C: `int trap_FS_FOpenFile(const char *qpath, fileHandle_t *f, fsMode_t mode)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:83-85`
pub fn FS_FOpenFile(engine: &Engine, qpath: &str, f: &mut fileHandle_t, mode: fsMode_t) -> c_int {
    let qpath_c = cstr(qpath);
    // SAFETY: path and handle slot outlive the synchronous syscall the Args feed.
    let args = unsafe { UiFsFopenfileArgs::new(qpath_c.as_ptr(), f as *mut fileHandle_t, mode) };
    <Engine as Execute<UiFsFopenfile>>::execute(engine, args)
}

/// Raven `trap_FS_GetFileList` — `UI_FS_GETFILELIST`
/// (token: `mp_abi::ui::syscalls::UI_FS_GETFILELIST`).
///
/// C: `int trap_FS_GetFileList(const char *path, const char *extension, char *listbuf, int bufsize)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:99-101`
///
/// `listbuf` stays a caller byte buffer: the engine packs NUL-separated names,
/// and callers walk them by offset.
pub fn FS_GetFileList(engine: &Engine, path: &str, extension: &str, listbuf: &mut [u8]) -> c_int {
    let path_c = cstr(path);
    let extension_c = cstr(extension);
    // SAFETY: both strings and the list buffer outlive the synchronous syscall.
    let args = unsafe {
        UiFsGetfilelistArgs::new(
            path_c.as_ptr(),
            extension_c.as_ptr(),
            listbuf.as_mut_ptr() as *mut c_char,
            listbuf.len() as c_int,
        )
    };
    <Engine as Execute<UiFsGetfilelist>>::execute(engine, args)
}

/// Raven `trap_FS_Read` — `UI_FS_READ` (token: `mp_abi::ui::syscalls::UI_FS_READ`).
///
/// C: `void trap_FS_Read(void *buffer, int len, fileHandle_t f)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:87-89`
pub fn FS_Read(engine: &Engine, buffer: &mut [u8], f: fileHandle_t) {
    // SAFETY: `buffer` outlives the synchronous syscall the Args feed.
    let args = unsafe { UiFsReadArgs::new(buffer.as_mut_ptr(), buffer.len() as c_int, f) };
    <Engine as Execute<UiFsRead>>::execute(engine, args)
}

/// Raven `trap_FS_Write` — `UI_FS_WRITE` (token: `mp_abi::ui::syscalls::UI_FS_WRITE`).
///
/// C: `void trap_FS_Write(const void *buffer, int len, fileHandle_t f)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:91-93`
pub fn FS_Write(engine: &Engine, buffer: &[u8], f: fileHandle_t) {
    <Engine as Execute<UiFsWrite>>::execute(
        engine,
        UiFsWriteArgs::new(buffer.as_ptr(), buffer.len() as c_int, f),
    )
}

/// Raven `trap_G2API_AddBolt` — `UI_G2_ADDBOLT`
/// (token: `mp_abi::ui::syscalls::UI_G2_ADDBOLT`).
///
/// C: `int trap_G2API_AddBolt(void *ghoul2, int modelIndex, const char *boneName)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:599-602`
pub fn G2API_AddBolt(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
) -> c_int {
    <Engine as Execute<UiG2Addbolt>>::execute(
        engine,
        UiG2AddboltArgs::new(ghoul2, modelIndex, cstr(boneName)),
    )
}

/// Raven `trap_G2API_AttachG2Model` — `UI_G2_ATTACHG2MODEL`
/// (token: `mp_abi::ui::syscalls::UI_G2_ATTACHG2MODEL`).
///
/// C: `qboolean trap_G2API_AttachG2Model(void *ghoul2From, int modelIndexFrom, void *ghoul2To, int toBoltIndex, int toModel)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:661-664`
pub fn G2API_AttachG2Model(
    engine: &Engine,
    ghoul2From: *mut c_void,
    modelIndexFrom: c_int,
    ghoul2To: *mut c_void,
    toBoltIndex: c_int,
    toModel: c_int,
) -> bool {
    <Engine as Execute<UiG2Attachg2model>>::execute(
        engine,
        UiG2Attachg2modelArgs::new(ghoul2From, modelIndexFrom, ghoul2To, toBoltIndex, toModel),
    ) != 0
}

/// Raven `trap_G2API_CleanGhoul2Models` — `UI_G2_CLEANMODELS`
/// (token: `mp_abi::ui::syscalls::UI_G2_CLEANMODELS`).
///
/// C: `void trap_G2API_CleanGhoul2Models(void **ghoul2Ptr)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:540-543`
///
/// The engine writes the caller's instance slot back to `NULL`, so the
/// pointer-to-token stays a raw slot pointer at this seam.
pub fn G2API_CleanGhoul2Models(engine: &Engine, ghoul2Ptr: *mut *mut c_void) {
    <Engine as Execute<UiG2Cleanmodels>>::execute(engine, UiG2CleanmodelsArgs::new(ghoul2Ptr))
}

/// Raven `trap_G2API_GetBoltMatrix` — `UI_G2_GETBOLT`
/// (token: `mp_abi::ui::syscalls::UI_G2_GETBOLT`).
///
/// C: `qboolean trap_G2API_GetBoltMatrix(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:475-479`
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boltIndex: c_int,
    matrix: &mut mdxaBone_t,
    angles: &vec3_t,
    position: &vec3_t,
    frameNum: c_int,
    modelList: Option<&mut qhandle_t>,
    scale: &vec3_t,
) -> bool {
    let model_list = modelList.map_or(null_mut(), |m| m as *mut qhandle_t);
    <Engine as Execute<UiG2Getbolt>>::execute(
        engine,
        UiG2GetboltArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix as *mut mdxaBone_t,
            angles as *const vec3_t,
            position as *const vec3_t,
            frameNum,
            model_list,
            scale as *const vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_GetGLAName` — `UI_G2_GETGLANAME`
/// (token: `mp_abi::ui::syscalls::UI_G2_GETGLANAME`).
///
/// C: `void trap_G2API_GetGLAName(void *ghoul2, int modelIndex, char *fillBuf)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:569-572`
///
/// Raven passes an unbounded `char[MAX_QPATH]`; `buffer_len` names that width
/// at the call site.
pub fn G2API_GetGLAName(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    buffer_len: usize,
) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<UiG2Getglaname>>::execute(
        engine,
        UiG2GetglanameArgs::new(ghoul2, modelIndex, buffer.as_mut_ptr() as *mut c_char),
    );
    buf_to_string(&buffer)
}

/// Raven `trap_G2API_HasGhoul2ModelOnIndex` — `UI_G2_HASGHOUL2MODELONINDEX`
/// (token: `mp_abi::ui::syscalls::UI_G2_HASGHOUL2MODELONINDEX`).
///
/// C: `qboolean trap_G2API_HasGhoul2ModelOnIndex(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:589-592`
pub fn G2API_HasGhoul2ModelOnIndex(
    engine: &Engine,
    ghlInfo: *mut c_void,
    modelIndex: c_int,
) -> bool {
    <Engine as Execute<UiG2Hasghoul2Modelonindex>>::execute(
        engine,
        UiG2Hasghoul2ModelonindexArgs::new(ghlInfo, modelIndex),
    ) != 0
}

/// Raven `trap_G2API_InitGhoul2Model` — `UI_G2_INITGHOUL2MODEL`
/// (token: `mp_abi::ui::syscalls::UI_G2_INITGHOUL2MODEL`).
///
/// C: `int trap_G2API_InitGhoul2Model(void **ghoul2Ptr, const char *fileName, int modelIndex, qhandle_t customSkin, qhandle_t customShader, int modelFlags, int lodBias)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:493-497`
///
/// The engine allocates the instance into the caller's slot, so the
/// pointer-to-token stays a raw slot pointer at this seam.
#[allow(clippy::too_many_arguments)]
pub fn G2API_InitGhoul2Model(
    engine: &Engine,
    ghoul2Ptr: *mut *mut c_void,
    fileName: &str,
    modelIndex: c_int,
    customSkin: qhandle_t,
    customShader: qhandle_t,
    modelFlags: c_int,
    lodBias: c_int,
) -> c_int {
    <Engine as Execute<UiG2Initghoul2Model>>::execute(
        engine,
        UiG2Initghoul2ModelArgs::new(
            ghoul2Ptr,
            cstr(fileName),
            modelIndex,
            customSkin,
            customShader,
            modelFlags,
            lodBias,
        ),
    )
}

/// Raven `trap_G2API_RemoveGhoul2Model` — `UI_G2_REMOVEGHOUL2MODEL`
/// (token: `mp_abi::ui::syscalls::UI_G2_REMOVEGHOUL2MODEL`).
///
/// C: `qboolean trap_G2API_RemoveGhoul2Model(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:594-597`
pub fn G2API_RemoveGhoul2Model(engine: &Engine, ghlInfo: *mut c_void, modelIndex: c_int) -> bool {
    <Engine as Execute<UiG2Removeghoul2Model>>::execute(
        engine,
        UiG2Removeghoul2ModelArgs::new(ghlInfo, modelIndex),
    ) != 0
}

/// Raven `trap_G2API_SetBoneAnim` — `UI_G2_PLAYANIM`
/// (token: `mp_abi::ui::syscalls::UI_G2_PLAYANIM`).
///
/// C: `qboolean trap_G2API_SetBoneAnim(void *ghoul2, const int modelIndex, const char *boneName, const int startFrame, const int endFrame, const int flags, const float animSpeed, const int currentTime, const float setFrame, const int blendTime)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:552-556`
#[allow(clippy::too_many_arguments)]
pub fn G2API_SetBoneAnim(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    startFrame: c_int,
    endFrame: c_int,
    flags: c_int,
    animSpeed: f32,
    currentTime: c_int,
    setFrame: f32,
    blendTime: c_int,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<UiG2Playanim>>::execute(
        engine,
        UiG2PlayanimArgs::new(
            ghoul2,
            modelIndex,
            bone_name_c.as_ptr(),
            startFrame,
            endFrame,
            flags,
            animSpeed,
            currentTime,
            setFrame,
            blendTime,
        ),
    ) != 0
}

/// Raven `trap_G2API_SetSkin` — `UI_G2_SETSKIN`
/// (token: `mp_abi::ui::syscalls::UI_G2_SETSKIN`).
///
/// C: `qboolean trap_G2API_SetSkin(void *ghoul2, int modelIndex, qhandle_t customSkin, qhandle_t renderSkin)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:499-502`
pub fn G2API_SetSkin(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    customSkin: qhandle_t,
    renderSkin: qhandle_t,
) -> bool {
    <Engine as Execute<UiG2Setskin>>::execute(
        engine,
        UiG2SetskinArgs::new(ghoul2, modelIndex, customSkin, renderSkin),
    ) != 0
}

/// Raven `trap_G2API_SetTime` — `UI_G2_SETTIME`
/// (token: `mp_abi::ui::syscalls::UI_G2_SETTIME`).
///
/// C: `void trap_G2API_SetTime(int time, int clock)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:629-632`
pub fn G2API_SetTime(engine: &Engine, time: c_int, clock: c_int) {
    <Engine as Execute<UiG2Settime>>::execute(engine, UiG2SettimeArgs::new(time, clock))
}

/// Raven `trap_G2_HaveWeGhoul2Models` — `UI_G2_HAVEWEGHOULMODELS`
/// (token: `mp_abi::ui::syscalls::UI_G2_HAVEWEGHOULMODELS`).
///
/// C: `qboolean trap_G2_HaveWeGhoul2Models(void *ghoul2)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:470-473`
pub fn G2_HaveWeGhoul2Models(engine: &Engine, ghoul2: *mut c_void) -> bool {
    <Engine as Execute<UiG2Haveweghoulmodels>>::execute(
        engine,
        UiG2HaveweghoulmodelsArgs::new(ghoul2),
    ) != 0
}

/// Raven `trap_GetClientState` — `UI_GETCLIENTSTATE`
/// (token: `mp_abi::ui::syscalls::UI_GETCLIENTSTATE`).
///
/// C: `void trap_GetClientState(uiClientState_t *state)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:258-260`
pub fn GetClientState(engine: &Engine, state: &mut uiClientState_t) {
    <Engine as Execute<UiGetclientstate>>::execute(
        engine,
        UiGetclientstateArgs::new(state as *mut uiClientState_t as *mut c_void),
    )
}

/// Raven `trap_GetConfigString` — `UI_GETCONFIGSTRING`
/// (token: `mp_abi::ui::syscalls::UI_GETCONFIGSTRING`).
///
/// C: `int trap_GetConfigString(int index, char *buff, int buffsize)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:266-268`
///
/// `qfalse` (index out of range) -> `None`.
pub fn GetConfigString(engine: &Engine, index: c_int, buffer_len: usize) -> Option<String> {
    let mut buffer = vec![0u8; buffer_len];
    let ok = <Engine as Execute<UiGetconfigstring>>::execute(
        engine,
        UiGetconfigstringArgs::new(
            index,
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        ),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    (ok != 0).then(|| latin1_to_string(&buffer[..nul]))
}

/// Raven `trap_GetGlconfig` — `UI_GETGLCONFIG`
/// (token: `mp_abi::ui::syscalls::UI_GETGLCONFIG`).
///
/// C: `void trap_GetGlconfig(glconfig_t *glconfig)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:262-264`
pub fn GetGlconfig(engine: &Engine, glconfig: &mut glconfig_t) {
    <Engine as Execute<UiGetglconfig>>::execute(
        engine,
        UiGetglconfigArgs::new(glconfig as *mut glconfig_t as *mut c_void),
    )
}

/// Raven `trap_GetLanguageName` — `UI_SP_GETLANGUAGENAME`
/// (token: `mp_abi::ui::syscalls::UI_SP_GETLANGUAGENAME`).
///
/// C: `void trap_GetLanguageName(const int languageIndex, char *buffer)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:443-446`
///
/// Raven passes an unbounded caller array; `buffer_len` names that width at
/// the call site.
pub fn GetLanguageName(engine: &Engine, languageIndex: c_int, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<UiSpGetlanguagename>>::execute(
        engine,
        UiSpGetlanguagenameArgs::new(languageIndex, buffer.as_mut_ptr() as *mut c_char),
    );
    buf_to_string(&buffer)
}

/// Raven `trap_Key_ClearStates` — `UI_KEY_CLEARSTATES`
/// (token: `mp_abi::ui::syscalls::UI_KEY_CLEARSTATES`).
///
/// C: `void trap_Key_ClearStates(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:242-244`
pub fn Key_ClearStates(engine: &Engine) {
    <Engine as Execute<UiKeyClearstates>>::execute(engine, UiKeyClearstatesArgs::new())
}

/// Raven `trap_Key_GetCatcher` — `UI_KEY_GETCATCHER`
/// (token: `mp_abi::ui::syscalls::UI_KEY_GETCATCHER`).
///
/// C: `int trap_Key_GetCatcher(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:246-248`
pub fn Key_GetCatcher(engine: &Engine) -> c_int {
    <Engine as Execute<UiKeyGetcatcher>>::execute(engine, UiKeyGetcatcherArgs::new())
}

/// Raven `trap_Key_SetCatcher` — `UI_KEY_SETCATCHER`
/// (token: `mp_abi::ui::syscalls::UI_KEY_SETCATCHER`).
///
/// C: `void trap_Key_SetCatcher(int catcher)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:250-252`
pub fn Key_SetCatcher(engine: &Engine, catcher: c_int) {
    <Engine as Execute<UiKeySetcatcher>>::execute(engine, UiKeySetcatcherArgs::new(catcher))
}

/// Raven `trap_LAN_AddServer` — `UI_LAN_ADDSERVER`
/// (token: `mp_abi::ui::syscalls::UI_LAN_ADDSERVER`).
///
/// C: `int trap_LAN_AddServer(int source, const char *name, const char *addr)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:330-332`
pub fn LAN_AddServer(engine: &Engine, source: c_int, name: &str, addr: &str) -> c_int {
    let name_c = cstr(name);
    let addr_c = cstr(addr);
    <Engine as Execute<UiLanAddserver>>::execute(
        engine,
        UiLanAddserverArgs::new(source, name_c.as_ptr(), addr_c.as_ptr()),
    )
}

/// Raven `trap_LAN_CompareServers` — `UI_LAN_COMPARESERVERS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_COMPARESERVERS`).
///
/// C: `int trap_LAN_CompareServers(int source, int sortKey, int sortDir, int s1, int s2)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:338-340`
pub fn LAN_CompareServers(
    engine: &Engine,
    source: c_int,
    sortKey: c_int,
    sortDir: c_int,
    s1: c_int,
    s2: c_int,
) -> c_int {
    <Engine as Execute<UiLanCompareservers>>::execute(
        engine,
        UiLanCompareserversArgs::new(source, sortKey, sortDir, s1, s2),
    )
}

/// Raven `trap_LAN_GetServerAddressString` — `UI_LAN_GETSERVERADDRESSSTRING`
/// (token: `mp_abi::ui::syscalls::UI_LAN_GETSERVERADDRESSSTRING`).
///
/// C: `void trap_LAN_GetServerAddressString(int source, int n, char *buf, int buflen)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:274-276`
pub fn LAN_GetServerAddressString(
    engine: &Engine,
    source: c_int,
    n: c_int,
    buffer_len: usize,
) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<UiLanGetserveraddressstring>>::execute(
        engine,
        UiLanGetserveraddressstringArgs::new(
            source,
            n,
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        ),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_LAN_GetServerCount` — `UI_LAN_GETSERVERCOUNT`
/// (token: `mp_abi::ui::syscalls::UI_LAN_GETSERVERCOUNT`).
///
/// C: `int trap_LAN_GetServerCount(int source)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:270-272`
pub fn LAN_GetServerCount(engine: &Engine, source: c_int) -> c_int {
    <Engine as Execute<UiLanGetservercount>>::execute(engine, UiLanGetservercountArgs::new(source))
}

/// Raven `trap_LAN_GetServerInfo` — `UI_LAN_GETSERVERINFO`
/// (token: `mp_abi::ui::syscalls::UI_LAN_GETSERVERINFO`).
///
/// C: `void trap_LAN_GetServerInfo(int source, int n, char *buf, int buflen)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:278-280`
pub fn LAN_GetServerInfo(engine: &Engine, source: c_int, n: c_int, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<UiLanGetserverinfo>>::execute(
        engine,
        UiLanGetserverinfoArgs::new(
            source,
            n,
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        ),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_LAN_GetServerPing` — `UI_LAN_GETSERVERPING`
/// (token: `mp_abi::ui::syscalls::UI_LAN_GETSERVERPING`).
///
/// C: `int trap_LAN_GetServerPing(int source, int n)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:282-284`
pub fn LAN_GetServerPing(engine: &Engine, source: c_int, n: c_int) -> c_int {
    <Engine as Execute<UiLanGetserverping>>::execute(engine, UiLanGetserverpingArgs::new(source, n))
}

/// Raven `trap_LAN_LoadCachedServers` — `UI_LAN_LOADCACHEDSERVERS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_LOADCACHEDSERVERS`).
///
/// C: `void trap_LAN_LoadCachedServers(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:298-300`
pub fn LAN_LoadCachedServers(engine: &Engine) {
    <Engine as Execute<UiLanLoadcachedservers>>::execute(engine, ())
}

/// Raven `trap_LAN_MarkServerVisible` — `UI_LAN_MARKSERVERVISIBLE`
/// (token: `mp_abi::ui::syscalls::UI_LAN_MARKSERVERVISIBLE`).
///
/// C: `void trap_LAN_MarkServerVisible(int source, int n, qboolean visible)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:318-320`
pub fn LAN_MarkServerVisible(engine: &Engine, source: c_int, n: c_int, visible: bool) {
    <Engine as Execute<UiLanMarkservervisible>>::execute(
        engine,
        UiLanMarkservervisibleArgs::new(source, n, c_int::from(visible)),
    )
}

/// Raven `trap_LAN_RemoveServer` — `UI_LAN_REMOVESERVER`
/// (token: `mp_abi::ui::syscalls::UI_LAN_REMOVESERVER`).
///
/// C: `void trap_LAN_RemoveServer(int source, const char *addr)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:334-336`
pub fn LAN_RemoveServer(engine: &Engine, source: c_int, addr: &str) {
    let addr_c = cstr(addr);
    <Engine as Execute<UiLanRemoveserver>>::execute(
        engine,
        UiLanRemoveserverArgs::new(source, addr_c.as_ptr()),
    )
}

/// Raven `trap_LAN_ResetPings` — `UI_LAN_RESETPINGS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_RESETPINGS`).
///
/// C: `void trap_LAN_ResetPings(int n)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:302-304`
pub fn LAN_ResetPings(engine: &Engine, n: c_int) {
    <Engine as Execute<UiLanResetpings>>::execute(engine, UiLanResetpingsArgs::new(n))
}

/// Raven `trap_LAN_SaveCachedServers` — `UI_LAN_SAVECACHEDSERVERS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_SAVECACHEDSERVERS`).
///
/// C: `void trap_LAN_SaveCachedServers(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:294-296`
pub fn LAN_SaveCachedServers(engine: &Engine) {
    <Engine as Execute<UiLanSavecachedservers>>::execute(engine, ())
}

/// Raven `trap_LAN_ServerIsVisible` — `UI_LAN_SERVERISVISIBLE`
/// (token: `mp_abi::ui::syscalls::UI_LAN_SERVERISVISIBLE`).
///
/// C: `int trap_LAN_ServerIsVisible(int source, int n)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:322-324`
pub fn LAN_ServerIsVisible(engine: &Engine, source: c_int, n: c_int) -> c_int {
    <Engine as Execute<UiLanServerisvisible>>::execute(
        engine,
        UiLanServerisvisibleArgs::new(source, n),
    )
}

/// Raven `trap_LAN_ServerStatus` — `UI_LAN_SERVERSTATUS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_SERVERSTATUS`).
///
/// C: `int trap_LAN_ServerStatus(const char *serverAddress, char *serverStatus, int maxLen)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:290-292`
///
/// Returns `(status, server_status)`. Raven also calls this with a null
/// address and a zero-length buffer purely to reset the engine's pending
/// query (`ui_main.c:8053`, `ui_main.c:8194`), so both stay optional: `None`
/// address and `buffer_len == 0` reproduce that call exactly, and the returned
/// string is then empty.
pub fn LAN_ServerStatus(
    engine: &Engine,
    serverAddress: Option<&str>,
    buffer_len: usize,
) -> (c_int, String) {
    let address_c = serverAddress.map(cstr);
    let address_ptr = address_c.as_ref().map_or(null(), |a| a.as_ptr());
    let mut buffer = vec![0u8; buffer_len];
    let status_ptr = if buffer_len == 0 {
        null_mut()
    } else {
        buffer.as_mut_ptr() as *mut c_char
    };
    let status = <Engine as Execute<UiLanServerstatus>>::execute(
        engine,
        UiLanServerstatusArgs::new(address_ptr, status_ptr, buffer_len as c_int),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    (status, latin1_to_string(&buffer[..nul]))
}

/// Raven `trap_LAN_UpdateVisiblePings` — `UI_LAN_UPDATEVISIBLEPINGS`
/// (token: `mp_abi::ui::syscalls::UI_LAN_UPDATEVISIBLEPINGS`).
///
/// C: `qboolean trap_LAN_UpdateVisiblePings(int source)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:326-328`
pub fn LAN_UpdateVisiblePings(engine: &Engine, source: c_int) -> bool {
    <Engine as Execute<UiLanUpdatevisiblepings>>::execute(
        engine,
        UiLanUpdatevisiblepingsArgs::new(source),
    ) != 0
}

/// Raven `trap_Language_UsesSpaces` — `UI_LANGUAGE_USESSPACES`
/// (token: `mp_abi::ui::syscalls::UI_LANGUAGE_USESSPACES`).
///
/// C: `qboolean trap_Language_UsesSpaces(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:141-144`
pub fn Language_UsesSpaces(engine: &Engine) -> bool {
    <Engine as Execute<UiLanguageUsesspaces>>::execute(engine, UiLanguageUsesspacesArgs::new()) != 0
}

/// Raven `trap_Milliseconds` — `UI_MILLISECONDS`
/// (token: `mp_abi::ui::syscalls::UI_MILLISECONDS`).
///
/// C: `int trap_Milliseconds(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:29-31`
pub fn Milliseconds(engine: &Engine) -> c_int {
    <Engine as Execute<UiMilliseconds>>::execute(engine, UiMillisecondsArgs::new())
}

/// Raven `trap_PC_FreeSource` — `UI_PC_FREE_SOURCE`
/// (token: `mp_abi::ui::syscalls::UI_PC_FREE_SOURCE`).
///
/// C: `int trap_PC_FreeSource(int handle)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:370-372`
pub fn PC_FreeSource(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<UiPcFreeSource>>::execute(engine, UiPcFreeSourceArgs::new(handle))
}

/// Raven `trap_PC_LoadGlobalDefines` — `UI_PC_LOAD_GLOBAL_DEFINES`
/// (token: `mp_abi::ui::syscalls::UI_PC_LOAD_GLOBAL_DEFINES`).
///
/// C: `int trap_PC_LoadGlobalDefines(const char *filename)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:382-385`
pub fn PC_LoadGlobalDefines(engine: &Engine, filename: &str) -> c_int {
    let filename_c = cstr(filename);
    <Engine as Execute<UiPcLoadGlobalDefines>>::execute(
        engine,
        UiPcLoadGlobalDefinesArgs::new(filename_c.as_ptr()),
    )
}

/// Raven `trap_PC_LoadSource` — `UI_PC_LOAD_SOURCE`
/// (token: `mp_abi::ui::syscalls::UI_PC_LOAD_SOURCE`).
///
/// C: `int trap_PC_LoadSource(const char *filename)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:366-368`
pub fn PC_LoadSource(engine: &Engine, filename: &str) -> c_int {
    let filename_c = cstr(filename);
    <Engine as Execute<UiPcLoadSource>>::execute(
        engine,
        UiPcLoadSourceArgs::new(filename_c.as_ptr()),
    )
}

/// Raven `trap_PC_ReadToken` — `UI_PC_READ_TOKEN`
/// (token: `mp_abi::ui::syscalls::UI_PC_READ_TOKEN`).
///
/// C: `int trap_PC_ReadToken(int handle, pc_token_t *pc_token)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:374-376`
///
/// The token block is engine-filled in place: it keeps its `#[repr(C)]` shape
/// and stays a caller `&mut`.
pub fn PC_ReadToken(engine: &Engine, handle: c_int, pc_token: &mut pc_token_t) -> bool {
    <Engine as Execute<UiPcReadToken>>::execute(
        engine,
        UiPcReadTokenArgs::new(handle, pc_token as *mut pc_token_t),
    ) != 0
}

/// Raven `trap_PC_RemoveAllGlobalDefines` — `UI_PC_REMOVE_ALL_GLOBAL_DEFINES`
/// (token: `mp_abi::ui::syscalls::UI_PC_REMOVE_ALL_GLOBAL_DEFINES`).
///
/// C: `void trap_PC_RemoveAllGlobalDefines(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:387-390`
pub fn PC_RemoveAllGlobalDefines(engine: &Engine) {
    <Engine as Execute<UiPcRemoveAllGlobalDefines>>::execute(
        engine,
        UiPcRemoveAllGlobalDefinesArgs::new(),
    )
}

/// Raven `trap_PC_SourceFileAndLine` — `UI_PC_SOURCE_FILE_AND_LINE`
/// (token: `mp_abi::ui::syscalls::UI_PC_SOURCE_FILE_AND_LINE`).
///
/// C: `int trap_PC_SourceFileAndLine(int handle, char *filename, int *line)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:378-380`
///
/// Returns `(status, filename, line)`. Raven passes an unbounded caller array
/// for the name; `buffer_len` names that width at the call site.
pub fn PC_SourceFileAndLine(
    engine: &Engine,
    handle: c_int,
    buffer_len: usize,
) -> (c_int, String, c_int) {
    let mut buffer = vec![0u8; buffer_len];
    let mut line: c_int = 0;
    let status = <Engine as Execute<UiPcSourceFileAndLine>>::execute(
        engine,
        UiPcSourceFileAndLineArgs::new(handle, buffer.as_mut_ptr() as *mut c_char, &mut line),
    );
    (status, buf_to_string(&buffer), line)
}

/// Raven `trap_Print` — `UI_PRINT` (token: `mp_abi::ui::syscalls::UI_PRINT`).
///
/// C: `void trap_Print(const char *string)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:21-23`
pub fn Print(engine: &Engine, string: &str) {
    let string_c = cstr(string);
    // SAFETY: the message outlives the synchronous syscall the Args feed.
    let args = unsafe { UiPrintArgs::new(string_c.as_ptr()) };
    <Engine as Execute<UiPrint>>::execute(engine, args)
}

/// Raven `trap_R_AddRefEntityToScene` — `UI_R_ADDREFENTITYTOSCENE`
/// (token: `mp_abi::ui::syscalls::UI_R_ADDREFENTITYTOSCENE`).
///
/// C: `void trap_R_AddRefEntityToScene(const refEntity_t *re)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:174-176`
pub fn R_AddRefEntityToScene(engine: &Engine, re: &refEntity_t) {
    <Engine as Execute<UiRAddrefentitytoscene>>::execute(
        engine,
        UiRAddrefentitytosceneArgs::new(re as *const refEntity_t as *const c_void),
    )
}

/// Raven `trap_R_DrawStretchPic` — `UI_R_DRAWSTRETCHPIC`
/// (token: `mp_abi::ui::syscalls::UI_R_DRAWSTRETCHPIC`).
///
/// C: `void trap_R_DrawStretchPic(float x, float y, float w, float h, float s1, float t1, float s2, float t2, qhandle_t hShader)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:194-196`
#[allow(clippy::too_many_arguments)]
pub fn R_DrawStretchPic(
    engine: &Engine,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    hShader: qhandle_t,
) {
    <Engine as Execute<UiRDrawstretchpic>>::execute(
        engine,
        UiRDrawstretchpicArgs::new(x, y, w, h, s1, t1, s2, t2, hShader),
    )
}

/// Raven `trap_R_Font_DrawString` — `UI_R_FONT_DRAWSTRING`
/// (token: `mp_abi::ui::syscalls::UI_R_FONT_DRAWSTRING`).
///
/// C: `void trap_R_Font_DrawString(int ox, int oy, const char *text, const float *rgba, const int setIndex, int iCharLimit, const float scale)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:131-134`
#[allow(clippy::too_many_arguments)]
pub fn R_Font_DrawString(
    engine: &Engine,
    ox: c_int,
    oy: c_int,
    text: &str,
    rgba: &vec4_t,
    setIndex: c_int,
    iCharLimit: c_int,
    scale: f32,
) {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<UiRFontDrawstring>>::execute(
        engine,
        UiRFontDrawstringArgs::new(
            ox,
            oy,
            text_c.as_ptr(),
            rgba.as_ptr(),
            setIndex,
            iCharLimit,
            scale,
        ),
    )
}

/// Raven `trap_R_Font_HeightPixels` — `UI_R_FONT_STRHEIGHTPIXELS`
/// (token: `mp_abi::ui::syscalls::UI_R_FONT_STRHEIGHTPIXELS`).
///
/// C: `int trap_R_Font_HeightPixels(const int iFontIndex, const float scale)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:126-129`
pub fn R_Font_HeightPixels(engine: &Engine, iFontIndex: c_int, scale: f32) -> c_int {
    <Engine as Execute<UiRFontStrheightpixels>>::execute(
        engine,
        UiRFontStrheightpixelsArgs::new(iFontIndex, scale),
    )
}

/// Raven `trap_R_Font_StrLenPixels` — `UI_R_FONT_STRLENPIXELS`
/// (token: `mp_abi::ui::syscalls::UI_R_FONT_STRLENPIXELS`).
///
/// C: `int trap_R_Font_StrLenPixels(const char *text, const int iFontIndex, const float scale)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:116-119`
pub fn R_Font_StrLenPixels(engine: &Engine, text: &str, iFontIndex: c_int, scale: f32) -> c_int {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<UiRFontStrlenpixels>>::execute(
        engine,
        UiRFontStrlenpixelsArgs::new(text_c.as_ptr(), iFontIndex, scale),
    )
}

/// Raven `trap_R_RegisterFont` — `UI_R_REGISTERFONT`
/// (token: `mp_abi::ui::syscalls::UI_R_REGISTERFONT`).
///
/// C: `qhandle_t trap_R_RegisterFont(const char *fontName)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:111-114`
pub fn R_RegisterFont(engine: &Engine, fontName: &str) -> qhandle_t {
    let font_name_c = cstr(fontName);
    <Engine as Execute<UiRRegisterfont>>::execute(
        engine,
        UiRRegisterfontArgs::new(font_name_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterShaderNoMip` — `UI_R_REGISTERSHADERNOMIP`
/// (token: `mp_abi::ui::syscalls::UI_R_REGISTERSHADERNOMIP`).
///
/// C: `qhandle_t trap_R_RegisterShaderNoMip(const char *name)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:151-161`
///
/// Raven: a leading `*` names a cvar holding the real shader name; a non-empty
/// value is registered in its place. The 1024-byte read width is Raven's
/// `char buf[1024]`.
pub fn R_RegisterShaderNoMip(engine: &Engine, name: &str) -> qhandle_t {
    if let Some(cvar_name) = name.strip_prefix('*') {
        let buf = Cvar_VariableStringBuffer(engine, cvar_name, 1024);
        if !buf.is_empty() {
            let buf_c = cstr(&buf);
            return <Engine as Execute<UiRRegistershadernomip>>::execute(
                engine,
                UiRRegistershadernomipArgs::new(buf_c.as_ptr()),
            );
        }
    }
    let name_c = cstr(name);
    <Engine as Execute<UiRRegistershadernomip>>::execute(
        engine,
        UiRRegistershadernomipArgs::new(name_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterSkin` — `UI_R_REGISTERSKIN`
/// (token: `mp_abi::ui::syscalls::UI_R_REGISTERSKIN`).
///
/// C: `qhandle_t trap_R_RegisterSkin(const char *name)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:107-109`
pub fn R_RegisterSkin(engine: &Engine, name: &str) -> qhandle_t {
    <Engine as Execute<UiRRegisterskin>>::execute(engine, UiRRegisterskinArgs::new(cstr(name)))
}

/// Raven `trap_R_SetColor` — `UI_R_SETCOLOR`
/// (token: `mp_abi::ui::syscalls::UI_R_SETCOLOR`).
///
/// C: `void trap_R_SetColor(const float *rgba)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:190-192`
///
/// `None` is Raven's `NULL`, which resets the renderer to white.
pub fn R_SetColor(engine: &Engine, rgba: Option<&vec4_t>) {
    let rgba_ptr = rgba.map_or(null(), |c| c.as_ptr());
    <Engine as Execute<UiRSetcolor>>::execute(engine, UiRSetcolorArgs::new(rgba_ptr))
}

/// Raven `trap_R_ShaderNameFromIndex` — `UI_R_SHADERNAMEFROMINDEX`
/// (token: `mp_abi::ui::syscalls::UI_R_SHADERNAMEFROMINDEX`).
///
/// C: `void trap_R_ShaderNameFromIndex(char *name, int index)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:165-168`
///
/// Raven passes an unbounded caller array; `buffer_len` names that width at
/// the call site.
pub fn R_ShaderNameFromIndex(engine: &Engine, index: c_int, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<UiRShadernamefromindex>>::execute(
        engine,
        UiRShadernamefromindexArgs::new(buffer.as_mut_ptr() as *mut c_char, index),
    );
    buf_to_string(&buffer)
}

/// Raven `trap_RealTime` — `UI_REAL_TIME`
/// (token: `mp_abi::ui::syscalls::UI_REAL_TIME`).
///
/// C: `int trap_RealTime(qtime_t *qtime)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:400-402`
pub fn RealTime(engine: &Engine, qtime: &mut qtime_t) -> c_int {
    <Engine as Execute<UiRealTime>>::execute(engine, UiRealTimeArgs::new(qtime as *mut qtime_t))
}

/// Raven `trap_SP_GetNumLanguages` — `UI_SP_GETNUMLANGUAGES`
/// (token: `mp_abi::ui::syscalls::UI_SP_GETNUMLANGUAGES`).
///
/// C: `int trap_SP_GetNumLanguages(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:438-441`
pub fn SP_GetNumLanguages(engine: &Engine) -> c_int {
    <Engine as Execute<UiSpGetnumlanguages>>::execute(engine, UiSpGetnumlanguagesArgs::new())
}

/// Raven `trap_SP_GetStringTextString` — `UI_SP_GETSTRINGTEXTSTRING`
/// (token: `mp_abi::ui::syscalls::UI_SP_GETSTRINGTEXTSTRING`).
///
/// C: `int trap_SP_GetStringTextString(const char *text, char *buffer, int bufferLength)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:448-451`
///
/// `qfalse` (no such string-package reference) -> `None`.
pub fn SP_GetStringTextString(engine: &Engine, text: &str, buffer_len: usize) -> Option<String> {
    let text_c = cstr(text);
    let mut buffer = vec![0u8; buffer_len];
    let found = <Engine as Execute<UiSpGetstringtextstring>>::execute(
        engine,
        UiSpGetstringtextstringArgs::new(
            text_c.as_ptr(),
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        ),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    (found != 0).then(|| latin1_to_string(&buffer[..nul]))
}

/// Raven `trap_S_RegisterSound` — `UI_S_REGISTERSOUND`
/// (token: `mp_abi::ui::syscalls::UI_S_REGISTERSOUND`).
///
/// C: `sfxHandle_t trap_S_RegisterSound(const char *sample)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:214-216`
pub fn S_RegisterSound(engine: &Engine, sample: &str) -> sfxHandle_t {
    let sample_c = cstr(sample);
    <Engine as Execute<UiSRegistersound>>::execute(
        engine,
        UiSRegistersoundArgs::new(sample_c.as_ptr()),
    )
}

/// Raven `trap_S_StartLocalSound` — `UI_S_STARTLOCALSOUND`
/// (token: `mp_abi::ui::syscalls::UI_S_STARTLOCALSOUND`).
///
/// C: `void trap_S_StartLocalSound(sfxHandle_t sfx, int channelNum)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:210-212`
pub fn S_StartLocalSound(engine: &Engine, sfx: sfxHandle_t, channelNum: c_int) {
    <Engine as Execute<UiSStartlocalsound>>::execute(
        engine,
        UiSStartlocalsoundArgs::new(sfx, channelNum),
    )
}

/// Raven `trap_UpdateScreen` — `UI_UPDATESCREEN`
/// (token: `mp_abi::ui::syscalls::UI_UPDATESCREEN`).
///
/// C: `void trap_UpdateScreen(void)`
/// Source: `oracle/codemp/ui/ui_syscalls.c:202-204`
pub fn UpdateScreen(engine: &Engine) {
    <Engine as Execute<UiUpdatescreen>>::execute(engine, UiUpdatescreenArgs::new())
}
