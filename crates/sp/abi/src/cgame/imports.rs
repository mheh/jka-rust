//! SP cgame imports enum vocabulary.
//!
//! Transcribed from Raven `oracle/code/cgame/cg_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpCgameImport {
    /// Source: `oracle/code/cgame/cg_public.h:61`
    CG_PRINT,

    /// Source: `oracle/code/cgame/cg_public.h:62`
    CG_ERROR,

    /// Source: `oracle/code/cgame/cg_public.h:63`
    CG_MILLISECONDS,

    /// Source: `oracle/code/cgame/cg_public.h:64`
    CG_CVAR_REGISTER,

    /// Source: `oracle/code/cgame/cg_public.h:65`
    CG_CVAR_UPDATE,

    /// Source: `oracle/code/cgame/cg_public.h:66`
    CG_CVAR_SET,

    /// Source: `oracle/code/cgame/cg_public.h:67`
    CG_ARGC,

    /// Source: `oracle/code/cgame/cg_public.h:68`
    CG_ARGV,

    /// Source: `oracle/code/cgame/cg_public.h:69`
    CG_ARGS,

    /// Source: `oracle/code/cgame/cg_public.h:70`
    CG_FS_FOPENFILE,

    /// Source: `oracle/code/cgame/cg_public.h:71`
    CG_FS_READ,

    /// Source: `oracle/code/cgame/cg_public.h:72`
    CG_FS_WRITE,

    /// Source: `oracle/code/cgame/cg_public.h:73`
    CG_FS_FCLOSEFILE,

    /// Source: `oracle/code/cgame/cg_public.h:74`
    CG_SENDCONSOLECOMMAND,

    /// Source: `oracle/code/cgame/cg_public.h:75`
    CG_ADDCOMMAND,

    /// Source: `oracle/code/cgame/cg_public.h:76`
    CG_SENDCLIENTCOMMAND,

    /// Source: `oracle/code/cgame/cg_public.h:77`
    CG_UPDATESCREEN,

    /// Source: `oracle/code/cgame/cg_public.h:78`
    CG_RMG_INIT,

    /// Source: `oracle/code/cgame/cg_public.h:79`
    CG_CM_REGISTER_TERRAIN,

    /// Source: `oracle/code/cgame/cg_public.h:80`
    CG_RE_INIT_RENDERER_TERRAIN,

    /// Source: `oracle/code/cgame/cg_public.h:81`
    CG_CM_LOADMAP,

    /// Source: `oracle/code/cgame/cg_public.h:82`
    CG_CM_NUMINLINEMODELS,

    /// Source: `oracle/code/cgame/cg_public.h:83`
    CG_CM_INLINEMODEL,

    /// Source: `oracle/code/cgame/cg_public.h:84`
    CG_CM_TEMPBOXMODEL,

    /// Source: `oracle/code/cgame/cg_public.h:85`
    CG_CM_POINTCONTENTS,

    /// Source: `oracle/code/cgame/cg_public.h:86`
    CG_CM_TRANSFORMEDPOINTCONTENTS,

    /// Source: `oracle/code/cgame/cg_public.h:87`
    CG_CM_BOXTRACE,

    /// Source: `oracle/code/cgame/cg_public.h:88`
    CG_CM_TRANSFORMEDBOXTRACE,

    /// Source: `oracle/code/cgame/cg_public.h:89`
    CG_CM_MARKFRAGMENTS,

    /// Source: `oracle/code/cgame/cg_public.h:90`
    CG_CM_SNAPPVS,

    /// Source: `oracle/code/cgame/cg_public.h:91`
    CG_S_STARTSOUND,

    /// Source: `oracle/code/cgame/cg_public.h:92`
    CG_S_STARTLOCALSOUND,

    /// Source: `oracle/code/cgame/cg_public.h:93`
    CG_S_CLEARLOOPINGSOUNDS,

    /// Source: `oracle/code/cgame/cg_public.h:94`
    CG_S_ADDLOOPINGSOUND,

    /// Source: `oracle/code/cgame/cg_public.h:95`
    CG_S_STOPSOUNDS,

    /// Source: `oracle/code/cgame/cg_public.h:96`
    CG_S_UPDATEENTITYPOSITION,

    /// Source: `oracle/code/cgame/cg_public.h:97`
    CG_S_RESPATIALIZE,

    /// Source: `oracle/code/cgame/cg_public.h:98`
    CG_S_REGISTERSOUND,

    /// Source: `oracle/code/cgame/cg_public.h:99`
    CG_S_STARTBACKGROUNDTRACK,

    /// Source: `oracle/code/cgame/cg_public.h:108`
    CG_FF_STARTFX,

    /// Source: `oracle/code/cgame/cg_public.h:109`
    CG_FF_ENSUREFX,

    /// Source: `oracle/code/cgame/cg_public.h:110`
    CG_FF_STOPFX,

    /// Source: `oracle/code/cgame/cg_public.h:111`
    CG_FF_STOPALLFX,

    /// Source: `oracle/code/cgame/cg_public.h:117`
    CG_R_LOADWORLDMAP,

    /// Source: `oracle/code/cgame/cg_public.h:118`
    CG_R_REGISTERMODEL,

    /// Source: `oracle/code/cgame/cg_public.h:119`
    CG_R_REGISTERSKIN,

    /// Source: `oracle/code/cgame/cg_public.h:120`
    CG_R_REGISTERSHADER,

    /// Source: `oracle/code/cgame/cg_public.h:121`
    CG_R_REGISTERSHADERNOMIP,

    /// Source: `oracle/code/cgame/cg_public.h:122`
    CG_R_REGISTERFONT,

    /// Source: `oracle/code/cgame/cg_public.h:123`
    CG_R_FONTSTRLENPIXELS,

    /// Source: `oracle/code/cgame/cg_public.h:124`
    CG_R_FONTSTRLENCHARS,

    /// Source: `oracle/code/cgame/cg_public.h:125`
    CG_R_FONTHEIGHTPIXELS,

    /// Source: `oracle/code/cgame/cg_public.h:126`
    CG_R_FONTDRAWSTRING,

    /// Source: `oracle/code/cgame/cg_public.h:127`
    CG_LANGUAGE_ISASIAN,

    /// Source: `oracle/code/cgame/cg_public.h:128`
    CG_LANGUAGE_USESSPACES,

    /// Source: `oracle/code/cgame/cg_public.h:129`
    CG_ANYLANGUAGE_READFROMSTRING,

    /// Source: `oracle/code/cgame/cg_public.h:130`
    CG_R_SETREFRACTIONPROP,

    /// Source: `oracle/code/cgame/cg_public.h:131`
    CG_R_CLEARSCENE,

    /// Source: `oracle/code/cgame/cg_public.h:132`
    CG_R_ADDREFENTITYTOSCENE,

    /// Source: `oracle/code/cgame/cg_public.h:134`
    CG_R_INPVS,

    /// Source: `oracle/code/cgame/cg_public.h:136`
    CG_R_GETLIGHTING,

    /// Source: `oracle/code/cgame/cg_public.h:137`
    CG_R_ADDPOLYTOSCENE,

    /// Source: `oracle/code/cgame/cg_public.h:138`
    CG_R_ADDLIGHTTOSCENE,

    /// Source: `oracle/code/cgame/cg_public.h:139`
    CG_R_RENDERSCENE,

    /// Source: `oracle/code/cgame/cg_public.h:140`
    CG_R_SETCOLOR,

    /// Source: `oracle/code/cgame/cg_public.h:141`
    CG_R_DRAWSTRETCHPIC,

    /// CG_R_DRAWSCREENSHOT,
    /// Source: `oracle/code/cgame/cg_public.h:143`
    CG_R_MODELBOUNDS,

    /// Source: `oracle/code/cgame/cg_public.h:144`
    CG_R_LERPTAG,

    /// Source: `oracle/code/cgame/cg_public.h:145`
    CG_R_DRAWROTATEPIC,

    /// Source: `oracle/code/cgame/cg_public.h:146`
    CG_R_DRAWROTATEPIC2,

    /// Source: `oracle/code/cgame/cg_public.h:147`
    CG_R_SETRANGEFOG,

    /// Source: `oracle/code/cgame/cg_public.h:148`
    CG_R_LA_GOGGLES,

    /// Source: `oracle/code/cgame/cg_public.h:149`
    CG_R_SCISSOR,

    /// Source: `oracle/code/cgame/cg_public.h:150`
    CG_GETGLCONFIG,

    /// Source: `oracle/code/cgame/cg_public.h:151`
    CG_GETGAMESTATE,

    /// Source: `oracle/code/cgame/cg_public.h:152`
    CG_GETCURRENTSNAPSHOTNUMBER,

    /// Source: `oracle/code/cgame/cg_public.h:153`
    CG_GETSNAPSHOT,

    /// Source: `oracle/code/cgame/cg_public.h:155`
    CG_GETDEFAULTSTATE,

    /// Source: `oracle/code/cgame/cg_public.h:157`
    CG_GETSERVERCOMMAND,

    /// Source: `oracle/code/cgame/cg_public.h:158`
    CG_GETCURRENTCMDNUMBER,

    /// Source: `oracle/code/cgame/cg_public.h:159`
    CG_GETUSERCMD,

    /// Source: `oracle/code/cgame/cg_public.h:160`
    CG_SETUSERCMDVALUE,

    /// Source: `oracle/code/cgame/cg_public.h:161`
    CG_SETUSERCMDANGLES,

    /// Source: `oracle/code/cgame/cg_public.h:162`
    CG_S_UPDATEAMBIENTSET,

    /// Source: `oracle/code/cgame/cg_public.h:163`
    CG_S_ADDLOCALSET,

    /// Source: `oracle/code/cgame/cg_public.h:164`
    CG_AS_PARSESETS,

    /// Source: `oracle/code/cgame/cg_public.h:165`
    CG_AS_ADDENTRY,

    /// Source: `oracle/code/cgame/cg_public.h:166`
    CG_AS_GETBMODELSOUND,

    /// Source: `oracle/code/cgame/cg_public.h:167`
    CG_S_GETSAMPLELENGTH,

    /// Source: `oracle/code/cgame/cg_public.h:168`
    COM_SETORGANGLES,

    /// Ghoul2 Insert Start
    /// Source: `oracle/code/cgame/cg_public.h:172`
    CG_G2_LISTBONES,

    /// Source: `oracle/code/cgame/cg_public.h:173`
    CG_G2_LISTSURFACES,

    /// Source: `oracle/code/cgame/cg_public.h:174`
    CG_G2_HAVEWEGHOULMODELS,

    /// Source: `oracle/code/cgame/cg_public.h:175`
    CG_G2_SETMODELS,

    /// Ghoul2 Insert End
    /// Source: `oracle/code/cgame/cg_public.h:180`
    CG_R_GET_LIGHT_STYLE,

    /// Source: `oracle/code/cgame/cg_public.h:181`
    CG_R_SET_LIGHT_STYLE,

    /// Source: `oracle/code/cgame/cg_public.h:182`
    CG_R_GET_BMODEL_VERTS,

    /// Source: `oracle/code/cgame/cg_public.h:183`
    CG_R_WORLD_EFFECT_COMMAND,

    /// Source: `oracle/code/cgame/cg_public.h:185`
    CG_CIN_PLAYCINEMATIC,

    /// Source: `oracle/code/cgame/cg_public.h:186`
    CG_CIN_STOPCINEMATIC,

    /// Source: `oracle/code/cgame/cg_public.h:187`
    CG_CIN_RUNCINEMATIC,

    /// Source: `oracle/code/cgame/cg_public.h:188`
    CG_CIN_DRAWCINEMATIC,

    /// Source: `oracle/code/cgame/cg_public.h:189`
    CG_CIN_SETEXTENTS,

    /// Source: `oracle/code/cgame/cg_public.h:190`
    CG_Z_MALLOC,

    /// Source: `oracle/code/cgame/cg_public.h:191`
    CG_Z_FREE,

    /// Source: `oracle/code/cgame/cg_public.h:192`
    CG_UI_MENU_RESET,

    /// Source: `oracle/code/cgame/cg_public.h:193`
    CG_UI_MENU_NEW,

    /// Source: `oracle/code/cgame/cg_public.h:194`
    CG_UI_SETACTIVE_MENU,

    /// Source: `oracle/code/cgame/cg_public.h:195`
    CG_UI_MENU_OPENBYNAME,

    /// Source: `oracle/code/cgame/cg_public.h:196`
    CG_UI_PARSE_INT,

    /// Source: `oracle/code/cgame/cg_public.h:197`
    CG_UI_PARSE_STRING,

    /// Source: `oracle/code/cgame/cg_public.h:198`
    CG_UI_PARSE_FLOAT,

    /// Source: `oracle/code/cgame/cg_public.h:199`
    CG_UI_STARTPARSESESSION,

    /// Source: `oracle/code/cgame/cg_public.h:200`
    CG_UI_ENDPARSESESSION,

    /// Source: `oracle/code/cgame/cg_public.h:201`
    CG_UI_PARSEEXT,

    /// Source: `oracle/code/cgame/cg_public.h:202`
    CG_UI_MENUPAINT_ALL,

    /// Source: `oracle/code/cgame/cg_public.h:203`
    CG_UI_MENUCLOSE_ALL,

    /// Source: `oracle/code/cgame/cg_public.h:204`
    CG_UI_STRING_INIT,

    /// Source: `oracle/code/cgame/cg_public.h:205`
    CG_UI_GETMENUINFO,

    /// Source: `oracle/code/cgame/cg_public.h:206`
    CG_SP_GETSTRINGTEXTSTRING,

    /// Source: `oracle/code/cgame/cg_public.h:207`
    CG_UI_GETITEMTEXT,

    /// Source: `oracle/code/cgame/cg_public.h:208`
    CG_UI_GETITEMINFO,
}
