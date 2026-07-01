//! SP UI imports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/code/ui/ui_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpUiImport {
    /// Source: `oracle/oracle/code/ui/ui_public.h:152`
    UI_ERROR,

    /// Source: `oracle/oracle/code/ui/ui_public.h:153`
    UI_PRINT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:154`
    UI_MILLISECONDS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:155`
    UI_CVAR_SET,

    /// Source: `oracle/oracle/code/ui/ui_public.h:156`
    UI_CVAR_VARIABLEVALUE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:157`
    UI_CVAR_VARIABLESTRINGBUFFER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:158`
    UI_CVAR_SETVALUE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:159`
    UI_CVAR_RESET,

    /// Source: `oracle/oracle/code/ui/ui_public.h:160`
    UI_CVAR_CREATE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:161`
    UI_CVAR_INFOSTRINGBUFFER,

    /// 10
    /// Source: `oracle/oracle/code/ui/ui_public.h:162`
    UI_ARGC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:163`
    UI_ARGV,

    /// Source: `oracle/oracle/code/ui/ui_public.h:164`
    UI_CMD_EXECUTETEXT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:165`
    UI_FS_FOPENFILE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:166`
    UI_FS_READ,

    /// Source: `oracle/oracle/code/ui/ui_public.h:167`
    UI_FS_WRITE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:168`
    UI_FS_FCLOSEFILE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:169`
    UI_FS_GETFILELIST,

    /// Source: `oracle/oracle/code/ui/ui_public.h:170`
    UI_R_REGISTERMODEL,

    /// Source: `oracle/oracle/code/ui/ui_public.h:171`
    UI_R_REGISTERSKIN,

    /// 20
    /// Source: `oracle/oracle/code/ui/ui_public.h:172`
    UI_R_REGISTERSHADERNOMIP,

    /// Source: `oracle/oracle/code/ui/ui_public.h:173`
    UI_R_CLEARSCENE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:174`
    UI_R_ADDREFENTITYTOSCENE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:175`
    UI_R_ADDPOLYTOSCENE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:176`
    UI_R_ADDLIGHTTOSCENE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:177`
    UI_R_RENDERSCENE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:178`
    UI_R_SETCOLOR,

    /// Source: `oracle/oracle/code/ui/ui_public.h:179`
    UI_R_DRAWSTRETCHPIC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:180`
    UI_UPDATESCREEN,

    /// Source: `oracle/oracle/code/ui/ui_public.h:181`
    UI_CM_LERPTAG,

    /// 30
    /// Source: `oracle/oracle/code/ui/ui_public.h:182`
    UI_CM_LOADMODEL,

    /// Source: `oracle/oracle/code/ui/ui_public.h:183`
    UI_S_REGISTERSOUND,

    /// Source: `oracle/oracle/code/ui/ui_public.h:184`
    UI_S_STARTLOCALSOUND,

    /// Source: `oracle/oracle/code/ui/ui_public.h:185`
    UI_KEY_KEYNUMTOSTRINGBUF,

    /// Source: `oracle/oracle/code/ui/ui_public.h:186`
    UI_KEY_GETBINDINGBUF,

    /// Source: `oracle/oracle/code/ui/ui_public.h:187`
    UI_KEY_SETBINDING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:188`
    UI_KEY_ISDOWN,

    /// Source: `oracle/oracle/code/ui/ui_public.h:189`
    UI_KEY_GETOVERSTRIKEMODE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:190`
    UI_KEY_SETOVERSTRIKEMODE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:191`
    UI_KEY_CLEARSTATES,

    /// 40
    /// Source: `oracle/oracle/code/ui/ui_public.h:192`
    UI_KEY_GETCATCHER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:193`
    UI_KEY_SETCATCHER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:194`
    UI_GETCLIPBOARDDATA,

    /// Source: `oracle/oracle/code/ui/ui_public.h:195`
    UI_GETGLCONFIG,

    /// Source: `oracle/oracle/code/ui/ui_public.h:196`
    UI_GETCLIENTSTATE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:197`
    UI_GETCONFIGSTRING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:198`
    UI_LAN_GETPINGQUEUECOUNT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:199`
    UI_LAN_CLEARPING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:200`
    UI_LAN_GETPING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:201`
    UI_LAN_GETPINGINFO,

    /// 50
    /// Source: `oracle/oracle/code/ui/ui_public.h:202`
    UI_CVAR_REGISTER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:203`
    UI_CVAR_UPDATE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:204`
    UI_MEMORY_REMAINING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:205`
    UI_GET_CDKEY,

    /// Source: `oracle/oracle/code/ui/ui_public.h:206`
    UI_SET_CDKEY,

    /// Source: `oracle/oracle/code/ui/ui_public.h:207`
    UI_R_REGISTERFONT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:208`
    UI_R_MODELBOUNDS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:209`
    UI_PC_ADD_GLOBAL_DEFINE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:210`
    UI_PC_LOAD_SOURCE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:211`
    UI_PC_FREE_SOURCE,

    /// 60
    /// Source: `oracle/oracle/code/ui/ui_public.h:212`
    UI_PC_READ_TOKEN,

    /// Source: `oracle/oracle/code/ui/ui_public.h:213`
    UI_PC_SOURCE_FILE_AND_LINE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:214`
    UI_S_STOPBACKGROUNDTRACK,

    /// Source: `oracle/oracle/code/ui/ui_public.h:215`
    UI_S_STARTBACKGROUNDTRACK,

    /// Source: `oracle/oracle/code/ui/ui_public.h:216`
    UI_REAL_TIME,

    /// Source: `oracle/oracle/code/ui/ui_public.h:217`
    UI_LAN_GETSERVERCOUNT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:218`
    UI_LAN_GETSERVERADDRESSSTRING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:219`
    UI_LAN_GETSERVERINFO,

    /// Source: `oracle/oracle/code/ui/ui_public.h:220`
    UI_LAN_MARKSERVERVISIBLE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:221`
    UI_LAN_UPDATEVISIBLEPINGS,

    /// 70
    /// Source: `oracle/oracle/code/ui/ui_public.h:222`
    UI_LAN_RESETPINGS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:223`
    UI_LAN_LOADCACHEDSERVERS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:224`
    UI_LAN_SAVECACHEDSERVERS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:225`
    UI_LAN_ADDSERVER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:226`
    UI_LAN_REMOVESERVER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:227`
    UI_CIN_PLAYCINEMATIC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:228`
    UI_CIN_STOPCINEMATIC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:229`
    UI_CIN_RUNCINEMATIC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:230`
    UI_CIN_DRAWCINEMATIC,

    /// Source: `oracle/oracle/code/ui/ui_public.h:231`
    UI_CIN_SETEXTENTS,

    /// 80
    /// Source: `oracle/oracle/code/ui/ui_public.h:232`
    UI_R_REMAP_SHADER,

    /// Source: `oracle/oracle/code/ui/ui_public.h:233`
    UI_VERIFY_CDKEY,

    /// Source: `oracle/oracle/code/ui/ui_public.h:234`
    UI_LAN_SERVERSTATUS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:235`
    UI_LAN_GETSERVERPING,

    /// Source: `oracle/oracle/code/ui/ui_public.h:236`
    UI_LAN_SERVERISVISIBLE,

    /// Source: `oracle/oracle/code/ui/ui_public.h:237`
    UI_LAN_COMPARESERVERS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:239`
    UI_MEMSET = 100,

    /// Source: `oracle/oracle/code/ui/ui_public.h:240`
    UI_MEMCPY,

    /// Source: `oracle/oracle/code/ui/ui_public.h:241`
    UI_STRNCPY,

    /// Source: `oracle/oracle/code/ui/ui_public.h:242`
    UI_SIN,

    /// Source: `oracle/oracle/code/ui/ui_public.h:243`
    UI_COS,

    /// Source: `oracle/oracle/code/ui/ui_public.h:244`
    UI_ATAN2,

    /// Source: `oracle/oracle/code/ui/ui_public.h:245`
    UI_SQRT,

    /// Source: `oracle/oracle/code/ui/ui_public.h:246`
    UI_FLOOR,

    /// Source: `oracle/oracle/code/ui/ui_public.h:247`
    UI_CEIL,
}
