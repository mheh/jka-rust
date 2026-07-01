//! MP UI imports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/ui/ui_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpUiImport {
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:18`
    UI_ERROR,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:19`
    UI_PRINT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:20`
    UI_MILLISECONDS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:21`
    UI_CVAR_SET,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:22`
    UI_CVAR_VARIABLEVALUE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:23`
    UI_CVAR_VARIABLESTRINGBUFFER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:24`
    UI_CVAR_SETVALUE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:25`
    UI_CVAR_RESET,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:26`
    UI_CVAR_CREATE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:27`
    UI_CVAR_INFOSTRINGBUFFER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:28`
    UI_ARGC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:29`
    UI_ARGV,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:30`
    UI_CMD_EXECUTETEXT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:31`
    UI_FS_FOPENFILE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:32`
    UI_FS_READ,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:33`
    UI_FS_WRITE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:34`
    UI_FS_FCLOSEFILE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:35`
    UI_FS_GETFILELIST,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:36`
    UI_R_REGISTERMODEL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:37`
    UI_R_REGISTERSKIN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:38`
    UI_R_REGISTERSHADERNOMIP,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:39`
    UI_R_SHADERNAMEFROMINDEX,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:40`
    UI_R_CLEARSCENE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:41`
    UI_R_ADDREFENTITYTOSCENE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:42`
    UI_R_ADDPOLYTOSCENE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:43`
    UI_R_ADDLIGHTTOSCENE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:44`
    UI_R_RENDERSCENE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:45`
    UI_R_SETCOLOR,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:46`
    UI_R_DRAWSTRETCHPIC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:47`
    UI_UPDATESCREEN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:48`
    UI_CM_LERPTAG,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:49`
    UI_CM_LOADMODEL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:50`
    UI_S_REGISTERSOUND,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:51`
    UI_S_STARTLOCALSOUND,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:52`
    UI_KEY_KEYNUMTOSTRINGBUF,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:53`
    UI_KEY_GETBINDINGBUF,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:54`
    UI_KEY_SETBINDING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:55`
    UI_KEY_ISDOWN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:56`
    UI_KEY_GETOVERSTRIKEMODE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:57`
    UI_KEY_SETOVERSTRIKEMODE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:58`
    UI_KEY_CLEARSTATES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:59`
    UI_KEY_GETCATCHER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:60`
    UI_KEY_SETCATCHER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:61`
    UI_GETCLIPBOARDDATA,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:62`
    UI_GETGLCONFIG,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:63`
    UI_GETCLIENTSTATE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:64`
    UI_GETCONFIGSTRING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:65`
    UI_LAN_GETPINGQUEUECOUNT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:66`
    UI_LAN_CLEARPING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:67`
    UI_LAN_GETPING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:68`
    UI_LAN_GETPINGINFO,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:69`
    UI_CVAR_REGISTER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:70`
    UI_CVAR_UPDATE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:71`
    UI_MEMORY_REMAINING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:72`
    UI_GET_CDKEY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:73`
    UI_SET_CDKEY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:74`
    UI_VERIFY_CDKEY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:75`
    UI_R_REGISTERFONT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:76`
    UI_R_FONT_STRLENPIXELS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:77`
    UI_R_FONT_STRLENCHARS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:78`
    UI_R_FONT_STRHEIGHTPIXELS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:79`
    UI_R_FONT_DRAWSTRING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:80`
    UI_LANGUAGE_ISASIAN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:81`
    UI_LANGUAGE_USESSPACES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:82`
    UI_ANYLANGUAGE_READCHARFROMSTRING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:83`
    UI_R_MODELBOUNDS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:84`
    UI_PC_ADD_GLOBAL_DEFINE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:85`
    UI_PC_LOAD_SOURCE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:86`
    UI_PC_FREE_SOURCE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:87`
    UI_PC_READ_TOKEN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:88`
    UI_PC_SOURCE_FILE_AND_LINE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:89`
    UI_PC_LOAD_GLOBAL_DEFINES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:90`
    UI_PC_REMOVE_ALL_GLOBAL_DEFINES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:92`
    UI_S_STOPBACKGROUNDTRACK,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:93`
    UI_S_STARTBACKGROUNDTRACK,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:94`
    UI_REAL_TIME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:95`
    UI_LAN_GETSERVERCOUNT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:96`
    UI_LAN_GETSERVERADDRESSSTRING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:97`
    UI_LAN_GETSERVERINFO,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:98`
    UI_LAN_MARKSERVERVISIBLE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:99`
    UI_LAN_UPDATEVISIBLEPINGS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:100`
    UI_LAN_RESETPINGS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:101`
    UI_LAN_LOADCACHEDSERVERS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:102`
    UI_LAN_SAVECACHEDSERVERS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:103`
    UI_LAN_ADDSERVER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:104`
    UI_LAN_REMOVESERVER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:105`
    UI_CIN_PLAYCINEMATIC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:106`
    UI_CIN_STOPCINEMATIC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:107`
    UI_CIN_RUNCINEMATIC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:108`
    UI_CIN_DRAWCINEMATIC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:109`
    UI_CIN_SETEXTENTS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:110`
    UI_R_REMAP_SHADER,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:111`
    UI_LAN_SERVERSTATUS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:112`
    UI_LAN_GETSERVERPING,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:113`
    UI_LAN_SERVERISVISIBLE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:114`
    UI_LAN_COMPARESERVERS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:116`
    UI_MEMSET = 100,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:117`
    UI_MEMCPY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:118`
    UI_STRNCPY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:119`
    UI_SIN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:120`
    UI_COS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:121`
    UI_ATAN2,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:122`
    UI_SQRT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:123`
    UI_MATRIXMULTIPLY,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:124`
    UI_ANGLEVECTORS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:125`
    UI_PERPENDICULARVECTOR,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:126`
    UI_FLOOR,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:127`
    UI_CEIL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:129`
    UI_TESTPRINTINT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:130`
    UI_TESTPRINTFLOAT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:132`
    UI_ACOS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:133`
    UI_ASIN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:135`
    UI_SP_GETNUMLANGUAGES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:136`
    UI_SP_GETLANGUAGENAME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:137`
    UI_SP_GETSTRINGTEXTSTRING = 200,

    /// Ghoul2 Insert Start
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:142`
    UI_G2_LISTSURFACES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:143`
    UI_G2_LISTBONES,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:144`
    UI_G2_SETMODELS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:145`
    UI_G2_HAVEWEGHOULMODELS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:146`
    UI_G2_GETBOLT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:147`
    UI_G2_GETBOLT_NOREC,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:148`
    UI_G2_GETBOLT_NOREC_NOROT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:149`
    UI_G2_INITGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:150`
    UI_G2_COLLISIONDETECT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:151`
    UI_G2_COLLISIONDETECTCACHE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:152`
    UI_G2_CLEANMODELS,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:153`
    UI_G2_ANGLEOVERRIDE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:154`
    UI_G2_PLAYANIM,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:155`
    UI_G2_GETBONEANIM,

    /// trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:156`
    UI_G2_GETBONEFRAME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:157`
    UI_G2_GETGLANAME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:158`
    UI_G2_COPYGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:159`
    UI_G2_COPYSPECIFICGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:160`
    UI_G2_DUPLICATEGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:161`
    UI_G2_HASGHOUL2MODELONINDEX,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:162`
    UI_G2_REMOVEGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:163`
    UI_G2_ADDBOLT,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:164`
    UI_G2_SETBOLTON,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:165`
    UI_G2_SETROOTSURFACE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:166`
    UI_G2_SETSURFACEONOFF,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:167`
    UI_G2_SETNEWORIGIN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:169`
    UI_G2_GETTIME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:170`
    UI_G2_SETTIME,

    /// rww - RAGDOLL_BEGIN
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:175`
    UI_G2_SETRAGDOLL,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:176`
    UI_G2_ANIMATEG2MODELS,

    /// rww - RAGDOLL_END
    /// rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
    /// by using the majority of gil's existing bone angling stuff from the ragdoll code.
    /// Source: `oracle/oracle/codemp/ui/ui_public.h:183`
    UI_G2_SETBONEIKSTATE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:184`
    UI_G2_IKMOVE,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:186`
    UI_G2_GETSURFACENAME,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:187`
    UI_G2_SETSKIN,

    /// Source: `oracle/oracle/codemp/ui/ui_public.h:188`
    UI_G2_ATTACHG2MODEL,
}
