//! MP cgame imports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/cgame/cg_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpCgameImport {
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:57`
    CG_PRINT = 0,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:58`
    CG_ERROR,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:59`
    CG_MILLISECONDS,

    /// Also for profiling.. do not use for game related tasks.
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:62`
    CG_PRECISIONTIMER_START,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:63`
    CG_PRECISIONTIMER_END,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:65`
    CG_CVAR_REGISTER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:66`
    CG_CVAR_UPDATE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:67`
    CG_CVAR_SET,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:68`
    CG_CVAR_VARIABLESTRINGBUFFER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:69`
    CG_CVAR_GETHIDDENVALUE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:70`
    CG_ARGC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:71`
    CG_ARGV,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:72`
    CG_ARGS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:73`
    CG_FS_FOPENFILE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:74`
    CG_FS_READ,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:75`
    CG_FS_WRITE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:76`
    CG_FS_FCLOSEFILE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:77`
    CG_FS_GETFILELIST,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:78`
    CG_SENDCONSOLECOMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:79`
    CG_ADDCOMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:80`
    CG_REMOVECOMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:81`
    CG_SENDCLIENTCOMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:82`
    CG_UPDATESCREEN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:83`
    CG_CM_LOADMAP,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:84`
    CG_CM_NUMINLINEMODELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:85`
    CG_CM_INLINEMODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:86`
    CG_CM_TEMPBOXMODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:87`
    CG_CM_TEMPCAPSULEMODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:88`
    CG_CM_POINTCONTENTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:89`
    CG_CM_TRANSFORMEDPOINTCONTENTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:90`
    CG_CM_BOXTRACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:91`
    CG_CM_CAPSULETRACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:92`
    CG_CM_TRANSFORMEDBOXTRACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:93`
    CG_CM_TRANSFORMEDCAPSULETRACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:94`
    CG_CM_MARKFRAGMENTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:95`
    CG_S_GETVOICEVOLUME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:96`
    CG_S_MUTESOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:97`
    CG_S_STARTSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:98`
    CG_S_STARTLOCALSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:99`
    CG_S_CLEARLOOPINGSOUNDS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:100`
    CG_S_ADDLOOPINGSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:101`
    CG_S_UPDATEENTITYPOSITION,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:102`
    CG_S_ADDREALLOOPINGSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:103`
    CG_S_STOPLOOPINGSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:104`
    CG_S_RESPATIALIZE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:105`
    CG_S_SHUTUP,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:106`
    CG_S_REGISTERSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:107`
    CG_S_STARTBACKGROUNDTRACK,

    /// rww - AS trap implem
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:110`
    CG_S_UPDATEAMBIENTSET,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:111`
    CG_AS_PARSESETS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:112`
    CG_AS_ADDPRECACHEENTRY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:113`
    CG_S_ADDLOCALSET,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:114`
    CG_AS_GETBMODELSOUND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:116`
    CG_R_LOADWORLDMAP,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:117`
    CG_R_REGISTERMODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:118`
    CG_R_REGISTERSKIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:119`
    CG_R_REGISTERSHADER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:120`
    CG_R_REGISTERSHADERNOMIP,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:121`
    CG_R_REGISTERFONT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:122`
    CG_R_FONT_STRLENPIXELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:123`
    CG_R_FONT_STRLENCHARS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:124`
    CG_R_FONT_STRHEIGHTPIXELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:125`
    CG_R_FONT_DRAWSTRING,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:126`
    CG_LANGUAGE_ISASIAN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:127`
    CG_LANGUAGE_USESSPACES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:128`
    CG_ANYLANGUAGE_READCHARFROMSTRING,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:130`
    CGAME_MEMSET = 100,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:131`
    CGAME_MEMCPY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:132`
    CGAME_STRNCPY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:133`
    CGAME_SIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:134`
    CGAME_COS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:135`
    CGAME_ATAN2,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:136`
    CGAME_SQRT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:137`
    CGAME_MATRIXMULTIPLY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:138`
    CGAME_ANGLEVECTORS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:139`
    CGAME_PERPENDICULARVECTOR,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:140`
    CGAME_FLOOR,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:141`
    CGAME_CEIL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:143`
    CGAME_TESTPRINTINT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:144`
    CGAME_TESTPRINTFLOAT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:146`
    CGAME_ACOS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:147`
    CGAME_ASIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:149`
    CG_R_CLEARSCENE = 200,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:150`
    CG_R_CLEARDECALS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:151`
    CG_R_ADDREFENTITYTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:152`
    CG_R_ADDPOLYTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:153`
    CG_R_ADDPOLYSTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:154`
    CG_R_ADDDECALTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:155`
    CG_R_LIGHTFORPOINT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:156`
    CG_R_ADDLIGHTTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:157`
    CG_R_ADDADDITIVELIGHTTOSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:158`
    CG_R_RENDERSCENE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:159`
    CG_R_SETCOLOR,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:160`
    CG_R_DRAWSTRETCHPIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:161`
    CG_R_MODELBOUNDS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:162`
    CG_R_LERPTAG,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:163`
    CG_R_DRAWROTATEPIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:164`
    CG_R_DRAWROTATEPIC2,

    /// linear fogging, with settable range -rww
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:165`
    CG_R_SETRANGEFOG,

    /// set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:166`
    CG_R_SETREFRACTIONPROP,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:167`
    CG_R_REMAP_SHADER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:168`
    CG_R_GET_LIGHT_STYLE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:169`
    CG_R_SET_LIGHT_STYLE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:170`
    CG_R_GET_BMODEL_VERTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:171`
    CG_R_GETDISTANCECULL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:173`
    CG_R_GETREALRES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:174`
    CG_R_AUTOMAPELEVADJ,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:175`
    CG_R_INITWIREFRAMEAUTO,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:177`
    CG_FX_ADDLINE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:179`
    CG_GETGLCONFIG,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:180`
    CG_GETGAMESTATE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:181`
    CG_GETCURRENTSNAPSHOTNUMBER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:182`
    CG_GETSNAPSHOT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:183`
    CG_GETDEFAULTSTATE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:184`
    CG_GETSERVERCOMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:185`
    CG_GETCURRENTCMDNUMBER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:186`
    CG_GETUSERCMD,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:187`
    CG_SETUSERCMDVALUE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:188`
    CG_SETCLIENTFORCEANGLE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:189`
    CG_SETCLIENTTURNEXTENT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:190`
    CG_OPENUIMENU,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:191`
    CG_TESTPRINTINT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:192`
    CG_TESTPRINTFLOAT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:193`
    CG_MEMORY_REMAINING,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:194`
    CG_KEY_ISDOWN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:195`
    CG_KEY_GETCATCHER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:196`
    CG_KEY_SETCATCHER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:197`
    CG_KEY_GETKEY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:199`
    CG_PC_ADD_GLOBAL_DEFINE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:200`
    CG_PC_LOAD_SOURCE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:201`
    CG_PC_FREE_SOURCE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:202`
    CG_PC_READ_TOKEN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:203`
    CG_PC_SOURCE_FILE_AND_LINE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:204`
    CG_PC_LOAD_GLOBAL_DEFINES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:205`
    CG_PC_REMOVE_ALL_GLOBAL_DEFINES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:207`
    CG_S_STOPBACKGROUNDTRACK,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:208`
    CG_REAL_TIME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:209`
    CG_SNAPVECTOR,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:210`
    CG_CIN_PLAYCINEMATIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:211`
    CG_CIN_STOPCINEMATIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:212`
    CG_CIN_RUNCINEMATIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:213`
    CG_CIN_DRAWCINEMATIC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:214`
    CG_CIN_SETEXTENTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:216`
    CG_GET_ENTITY_TOKEN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:217`
    CG_R_INPVS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:219`
    CG_FX_REGISTER_EFFECT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:220`
    CG_FX_PLAY_EFFECT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:221`
    CG_FX_PLAY_ENTITY_EFFECT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:222`
    CG_FX_PLAY_EFFECT_ID,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:223`
    CG_FX_PLAY_PORTAL_EFFECT_ID,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:224`
    CG_FX_PLAY_ENTITY_EFFECT_ID,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:225`
    CG_FX_PLAY_BOLTED_EFFECT_ID,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:226`
    CG_FX_ADD_SCHEDULED_EFFECTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:227`
    CG_FX_INIT_SYSTEM,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:228`
    CG_FX_SET_REFDEF,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:229`
    CG_FX_FREE_SYSTEM,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:230`
    CG_FX_ADJUST_TIME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:231`
    CG_FX_DRAW_2D_EFFECTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:232`
    CG_FX_RESET,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:233`
    CG_FX_ADDPOLY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:234`
    CG_FX_ADDBEZIER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:235`
    CG_FX_ADDPRIMITIVE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:236`
    CG_FX_ADDSPRITE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:237`
    CG_FX_ADDELECTRICITY,

    /// CG_SP_PRINT,
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:240`
    CG_SP_GETSTRINGTEXTSTRING,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:242`
    CG_ROFF_CLEAN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:243`
    CG_ROFF_UPDATE_ENTITIES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:244`
    CG_ROFF_CACHE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:245`
    CG_ROFF_PLAY,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:246`
    CG_ROFF_PURGE_ENT,

    /// rww - dynamic vm memory allocation!
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:250`
    CG_TRUEMALLOC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:251`
    CG_TRUEFREE,

    /// Ghoul2 Insert Start
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:256`
    CG_G2_LISTSURFACES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:257`
    CG_G2_LISTBONES,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:258`
    CG_G2_SETMODELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:259`
    CG_G2_HAVEWEGHOULMODELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:260`
    CG_G2_GETBOLT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:261`
    CG_G2_GETBOLT_NOREC,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:262`
    CG_G2_GETBOLT_NOREC_NOROT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:263`
    CG_G2_INITGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:264`
    CG_G2_SETSKIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:265`
    CG_G2_COLLISIONDETECT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:266`
    CG_G2_COLLISIONDETECTCACHE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:267`
    CG_G2_CLEANMODELS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:268`
    CG_G2_ANGLEOVERRIDE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:269`
    CG_G2_PLAYANIM,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:270`
    CG_G2_GETBONEANIM,

    /// trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:271`
    CG_G2_GETBONEFRAME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:272`
    CG_G2_GETGLANAME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:273`
    CG_G2_COPYGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:274`
    CG_G2_COPYSPECIFICGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:275`
    CG_G2_DUPLICATEGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:276`
    CG_G2_HASGHOUL2MODELONINDEX,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:277`
    CG_G2_REMOVEGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:278`
    CG_G2_SKINLESSMODEL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:279`
    CG_G2_GETNUMGOREMARKS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:280`
    CG_G2_ADDSKINGORE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:281`
    CG_G2_CLEARSKINGORE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:282`
    CG_G2_SIZE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:283`
    CG_G2_ADDBOLT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:284`
    CG_G2_ATTACHENT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:285`
    CG_G2_SETBOLTON,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:286`
    CG_G2_SETROOTSURFACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:287`
    CG_G2_SETSURFACEONOFF,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:288`
    CG_G2_SETNEWORIGIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:289`
    CG_G2_DOESBONEEXIST,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:290`
    CG_G2_GETSURFACERENDERSTATUS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:292`
    CG_G2_GETTIME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:293`
    CG_G2_SETTIME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:295`
    CG_G2_ABSURDSMOOTHING,

    /// rww - RAGDOLL_BEGIN
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:300`
    CG_G2_SETRAGDOLL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:301`
    CG_G2_ANIMATEG2MODELS,

    /// rww - RAGDOLL_END
    /// additional ragdoll options -rww
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:307`
    CG_G2_RAGPCJCONSTRAINT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:308`
    CG_G2_RAGPCJGRADIENTSPEED,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:309`
    CG_G2_RAGEFFECTORGOAL,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:310`
    CG_G2_GETRAGBONEPOS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:311`
    CG_G2_RAGEFFECTORKICK,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:312`
    CG_G2_RAGFORCESOLVE,

    /// rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
    /// by using the majority of gil's existing bone angling stuff from the ragdoll code.
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:316`
    CG_G2_SETBONEIKSTATE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:317`
    CG_G2_IKMOVE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:319`
    CG_G2_REMOVEBONE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:321`
    CG_G2_ATTACHINSTANCETOENTNUM,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:322`
    CG_G2_CLEARATTACHEDINSTANCE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:323`
    CG_G2_CLEANENTATTACHMENTS,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:324`
    CG_G2_OVERRIDESERVER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:326`
    CG_G2_GETSURFACENAME,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:328`
    CG_SET_SHARED_BUFFER,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:330`
    CG_CM_REGISTER_TERRAIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:331`
    CG_RMG_INIT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:332`
    CG_RE_INIT_RENDERER_TERRAIN,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:333`
    CG_R_WEATHER_CONTENTS_OVERRIDE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:334`
    CG_R_WORLDEFFECTCOMMAND,

    /// Adding trap to get weather working
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:336`
    CG_WE_ADDWEATHERZONE,
}
