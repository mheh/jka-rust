//! `mod trap` outbound-call wrappers for all 215 `trap_*` functions of
//! `oracle/codemp/cgame/cg_syscalls.c` — cgame's complete import surface. One
//! non-generic fn per call, in `cg_syscalls.c` order. C signatures:
//! `oracle/codemp/cgame/cg_syscalls.c`; transport authority:
//! `CL_CgameSystemCalls` in `oracle/codemp/client/cl_cgame.cpp`; token mapping:
//! `crates/mp/abi/src/cgame/syscalls/`.
//!
//! The import enum carries 233 entries. The 16 `CGAME_*` float-bridge entries
//! of the `= 100` block (`CGAME_MEMSET`, `CGAME_SIN`, `CGAME_SQRT`, …) are
//! QVM-only artifacts with no `trap_*` wrapper in Raven's DLL build, so this
//! module has none either. `CG_TESTPRINTINT`/`CG_TESTPRINTFLOAT` are reached by
//! the debug helpers `testPrintInt`/`testPrintFloat`, which carry no `trap_`
//! prefix and no caller; they are likewise absent.
//! Source: `oracle/codemp/cgame/cg_public.h:130-147`, `:191-192`;
//! `oracle/codemp/cgame/cg_syscalls.c:513-519`
//!
//! Hand-maintained, on the `mp_ui::trap` template (stage C3, DEC-45.3):
//! wrappers own the C seam — string args are `&str`, engine-filled `char`
//! buffers come back as `String` (`buffer_len` keeps the engine-side truncation
//! width at the call site), `qboolean` is `bool`, out-params are return values.
//! The raw pointer/`Args` shapes live only inside this module and `mp_abi`.
//! Ghoul2 instances stay opaque engine tokens (`*mut c_void`) — the module
//! never reads through them.
//!
//! String encoding (#13 discipline). Asset and identifier arguments — file
//! paths, shader/skin/model/font/effect/terrain names, cvar and command names,
//! key bindings, G2 bone/surface/tag/GLA names, string-package keys — cross as
//! UTF-8-transparent `cstr`/`buf_to_string`, matching `mp_ui::trap` and
//! `mp_game::trap`. Arguments and buffers carrying free text the engine treats
//! as opaque bytes — argv, cvar value buffers, console and client command text,
//! drawn font text, localized string-package text, world-effect commands — use
//! the bijective Latin-1 pair (`string_to_latin1`/`latin1_to_string`) so all
//! 256 byte values round-trip, exactly as `mp_game::trap::SendServerCommand`
//! does.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr::{null, null_mut};
use std::ffi::CString;

use mp_abi::cgame::public::snapshot_t::snapshot_t;
use mp_abi::cgame::syscalls::CG_ADDCOMMAND::{CgAddcommand, CgAddcommandArgs};
use mp_abi::cgame::syscalls::CG_ANYLANGUAGE_READCHARFROMSTRING::{
    CgAnylanguageReadcharfromstring, CgAnylanguageReadcharfromstringArgs,
};
use mp_abi::cgame::syscalls::CG_ARGC::{CgArgc, CgArgcArgs};
use mp_abi::cgame::syscalls::CG_ARGS::{CgArgs, CgArgsArgs};
use mp_abi::cgame::syscalls::CG_ARGV::{CgArgv, CgArgvArgs};
use mp_abi::cgame::syscalls::CG_AS_ADDPRECACHEENTRY::{
    CgAsAddprecacheentry, CgAsAddprecacheentryArgs,
};
use mp_abi::cgame::syscalls::CG_AS_GETBMODELSOUND::{CgAsGetbmodelsound, CgAsGetbmodelsoundArgs};
use mp_abi::cgame::syscalls::CG_AS_PARSESETS::{CgAsParsesets, CgAsParsesetsArgs};
use mp_abi::cgame::syscalls::CG_CIN_DRAWCINEMATIC::{CgCinDrawcinematic, CgCinDrawcinematicArgs};
use mp_abi::cgame::syscalls::CG_CIN_PLAYCINEMATIC::{CgCinPlaycinematic, CgCinPlaycinematicArgs};
use mp_abi::cgame::syscalls::CG_CIN_RUNCINEMATIC::{CgCinRuncinematic, CgCinRuncinematicArgs};
use mp_abi::cgame::syscalls::CG_CIN_SETEXTENTS::{CgCinSetextents, CgCinSetextentsArgs};
use mp_abi::cgame::syscalls::CG_CIN_STOPCINEMATIC::{CgCinStopcinematic, CgCinStopcinematicArgs};
use mp_abi::cgame::syscalls::CG_CM_BOXTRACE::{CgCmBoxtrace, CgCmBoxtraceArgs};
use mp_abi::cgame::syscalls::CG_CM_CAPSULETRACE::{CgCmCapsuletrace, CgCmCapsuletraceArgs};
use mp_abi::cgame::syscalls::CG_CM_INLINEMODEL::{CgCmInlinemodel, CgCmInlinemodelArgs};
use mp_abi::cgame::syscalls::CG_CM_LOADMAP::{CgCmLoadmap, CgCmLoadmapArgs};
use mp_abi::cgame::syscalls::CG_CM_MARKFRAGMENTS::markFragment_t;
use mp_abi::cgame::syscalls::CG_CM_MARKFRAGMENTS::{CgCmMarkfragments, CgCmMarkfragmentsArgs};
use mp_abi::cgame::syscalls::CG_CM_NUMINLINEMODELS::{
    CgCmNuminlinemodels, CgCmNuminlinemodelsArgs,
};
use mp_abi::cgame::syscalls::CG_CM_POINTCONTENTS::{CgCmPointcontents, CgCmPointcontentsArgs};
use mp_abi::cgame::syscalls::CG_CM_REGISTER_TERRAIN::{
    CgCmRegisterTerrain, CgCmRegisterTerrainArgs,
};
use mp_abi::cgame::syscalls::CG_CM_TEMPBOXMODEL::{CgCmTempboxmodel, CgCmTempboxmodelArgs};
use mp_abi::cgame::syscalls::CG_CM_TEMPCAPSULEMODEL::{
    CgCmTempcapsulemodel, CgCmTempcapsulemodelArgs,
};
use mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDBOXTRACE::{
    CgCmTransformedboxtrace, CgCmTransformedboxtraceArgs,
};
use mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDCAPSULETRACE::{
    CgCmTransformedcapsuletrace, CgCmTransformedcapsuletraceArgs,
};
use mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDPOINTCONTENTS::{
    CgCmTransformedpointcontents, CgCmTransformedpointcontentsArgs,
};
use mp_abi::cgame::syscalls::CG_CVAR_GETHIDDENVALUE::{
    CgCvarGethiddenvalue, CgCvarGethiddenvalueArgs,
};
use mp_abi::cgame::syscalls::CG_CVAR_REGISTER::{CgCvarRegister, CgCvarRegisterArgs};
use mp_abi::cgame::syscalls::CG_CVAR_SET::{CgCvarSet, CgCvarSetArgs};
use mp_abi::cgame::syscalls::CG_CVAR_UPDATE::{CgCvarUpdate, CgCvarUpdateArgs};
use mp_abi::cgame::syscalls::CG_CVAR_VARIABLESTRINGBUFFER::{
    CgCvarVariablestringbuffer, CgCvarVariablestringbufferArgs,
};
use mp_abi::cgame::syscalls::CG_ERROR::{CgError, CgErrorArgs};
use mp_abi::cgame::syscalls::CG_FS_FCLOSEFILE::{CgFsFclosefile, CgFsFclosefileArgs};
use mp_abi::cgame::syscalls::CG_FS_FOPENFILE::{CgFsFopenfile, CgFsFopenfileArgs};
use mp_abi::cgame::syscalls::CG_FS_GETFILELIST::{CgFsGetfilelist, CgFsGetfilelistArgs};
use mp_abi::cgame::syscalls::CG_FS_READ::{CgFsRead, CgFsReadArgs};
use mp_abi::cgame::syscalls::CG_FS_WRITE::{CgFsWrite, CgFsWriteArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDBEZIER::{CgFxAddbezier, CgFxAddbezierArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDELECTRICITY::{CgFxAddelectricity, CgFxAddelectricityArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDLINE::{CgFxAddline, CgFxAddlineArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDPOLY::{CgFxAddpoly, CgFxAddpolyArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDPRIMITIVE::{CgFxAddprimitive, CgFxAddprimitiveArgs};
use mp_abi::cgame::syscalls::CG_FX_ADDSPRITE::{CgFxAddsprite, CgFxAddspriteArgs};
use mp_abi::cgame::syscalls::CG_FX_ADD_SCHEDULED_EFFECTS::{
    CgFxAddScheduledEffects, CgFxAddScheduledEffectsArgs,
};
use mp_abi::cgame::syscalls::CG_FX_ADJUST_TIME::{CgFxAdjustTime, CgFxAdjustTimeArgs};
use mp_abi::cgame::syscalls::CG_FX_DRAW_2D_EFFECTS::{CgFxDraw2dEffects, CgFxDraw2dEffectsArgs};
use mp_abi::cgame::syscalls::CG_FX_FREE_SYSTEM::{CgFxFreeSystem, CgFxFreeSystemArgs};
use mp_abi::cgame::syscalls::CG_FX_INIT_SYSTEM::{CgFxInitSystem, CgFxInitSystemArgs};
use mp_abi::cgame::syscalls::CG_FX_PLAY_BOLTED_EFFECT_ID::{
    CgFxPlayBoltedEffectId, CgFxPlayBoltedEffectIdArgs,
};
use mp_abi::cgame::syscalls::CG_FX_PLAY_EFFECT::{CgFxPlayEffect, CgFxPlayEffectArgs};
use mp_abi::cgame::syscalls::CG_FX_PLAY_EFFECT_ID::{CgFxPlayEffectId, CgFxPlayEffectIdArgs};
use mp_abi::cgame::syscalls::CG_FX_PLAY_ENTITY_EFFECT::{
    CgFxPlayEntityEffect, CgFxPlayEntityEffectArgs,
};
use mp_abi::cgame::syscalls::CG_FX_PLAY_ENTITY_EFFECT_ID::{
    CgFxPlayEntityEffectId, CgFxPlayEntityEffectIdArgs,
};
use mp_abi::cgame::syscalls::CG_FX_PLAY_PORTAL_EFFECT_ID::{
    CgFxPlayPortalEffectId, CgFxPlayPortalEffectIdArgs,
};
use mp_abi::cgame::syscalls::CG_FX_REGISTER_EFFECT::{CgFxRegisterEffect, CgFxRegisterEffectArgs};
use mp_abi::cgame::syscalls::CG_FX_RESET::{CgFxReset, CgFxResetArgs};
use mp_abi::cgame::syscalls::CG_FX_SET_REFDEF::{CgFxSetRefdef, CgFxSetRefdefArgs};
use mp_abi::cgame::syscalls::CG_G2_ABSURDSMOOTHING::{
    CgG2Absurdsmoothing, CgG2AbsurdsmoothingArgs,
};
use mp_abi::cgame::syscalls::CG_G2_ADDBOLT::{CgG2Addbolt, CgG2AddboltArgs};
use mp_abi::cgame::syscalls::CG_G2_ADDSKINGORE::{CgG2Addskingore, CgG2AddskingoreArgs};
use mp_abi::cgame::syscalls::CG_G2_ANGLEOVERRIDE::{CgG2Angleoverride, CgG2AngleoverrideArgs};
use mp_abi::cgame::syscalls::CG_G2_ANIMATEG2MODELS::{
    CgG2Animateg2models, CgG2Animateg2modelsArgs,
};
use mp_abi::cgame::syscalls::CG_G2_ATTACHENT::{CgG2Attachent, CgG2AttachentArgs};
use mp_abi::cgame::syscalls::CG_G2_ATTACHINSTANCETOENTNUM::{
    CgG2Attachinstancetoentnum, CgG2AttachinstancetoentnumArgs,
};
use mp_abi::cgame::syscalls::CG_G2_CLEANENTATTACHMENTS::{
    CgG2Cleanentattachments, CgG2CleanentattachmentsArgs,
};
use mp_abi::cgame::syscalls::CG_G2_CLEANMODELS::{CgG2Cleanmodels, CgG2CleanmodelsArgs};
use mp_abi::cgame::syscalls::CG_G2_CLEARATTACHEDINSTANCE::{
    CgG2Clearattachedinstance, CgG2ClearattachedinstanceArgs,
};
use mp_abi::cgame::syscalls::CG_G2_CLEARSKINGORE::{CgG2Clearskingore, CgG2ClearskingoreArgs};
use mp_abi::cgame::syscalls::CG_G2_COLLISIONDETECT::{
    CgG2Collisiondetect, CgG2CollisiondetectArgs,
};
use mp_abi::cgame::syscalls::CG_G2_COLLISIONDETECTCACHE::{
    CgG2Collisiondetectcache, CgG2CollisiondetectcacheArgs,
};
use mp_abi::cgame::syscalls::CG_G2_COPYGHOUL2INSTANCE::{
    CgG2Copyghoul2instance, CgG2Copyghoul2instanceArgs,
};
use mp_abi::cgame::syscalls::CG_G2_COPYSPECIFICGHOUL2MODEL::{
    CgG2Copyspecificghoul2model, CgG2Copyspecificghoul2modelArgs,
};
use mp_abi::cgame::syscalls::CG_G2_DOESBONEEXIST::{CgG2Doesboneexist, CgG2DoesboneexistArgs};
use mp_abi::cgame::syscalls::CG_G2_DUPLICATEGHOUL2INSTANCE::{
    CgG2Duplicateghoul2instance, CgG2Duplicateghoul2instanceArgs,
};
use mp_abi::cgame::syscalls::CG_G2_GETBOLT::{CgG2Getbolt, CgG2GetboltArgs};
use mp_abi::cgame::syscalls::CG_G2_GETBOLT_NOREC::{CgG2GetboltNorec, CgG2GetboltNorecArgs};
use mp_abi::cgame::syscalls::CG_G2_GETBOLT_NOREC_NOROT::{
    CgG2GetboltNorecNorot, CgG2GetboltNorecNorotArgs,
};
use mp_abi::cgame::syscalls::CG_G2_GETBONEANIM::{CgG2Getboneanim, CgG2GetboneanimArgs};
use mp_abi::cgame::syscalls::CG_G2_GETBONEFRAME::{CgG2Getboneframe, CgG2GetboneframeArgs};
use mp_abi::cgame::syscalls::CG_G2_GETGLANAME::{CgG2Getglaname, CgG2GetglanameArgs};
use mp_abi::cgame::syscalls::CG_G2_GETNUMGOREMARKS::{
    CgG2Getnumgoremarks, CgG2GetnumgoremarksArgs,
};
use mp_abi::cgame::syscalls::CG_G2_GETRAGBONEPOS::{CgG2Getragbonepos, CgG2GetragboneposArgs};
use mp_abi::cgame::syscalls::CG_G2_GETSURFACENAME::{CgG2Getsurfacename, CgG2GetsurfacenameArgs};
use mp_abi::cgame::syscalls::CG_G2_GETSURFACERENDERSTATUS::{
    CgG2Getsurfacerenderstatus, CgG2GetsurfacerenderstatusArgs,
};
use mp_abi::cgame::syscalls::CG_G2_GETTIME::{CgG2Gettime, CgG2GettimeArgs};
use mp_abi::cgame::syscalls::CG_G2_HASGHOUL2MODELONINDEX::{
    CgG2Hasghoul2modelonindex, CgG2Hasghoul2modelonindexArgs,
};
use mp_abi::cgame::syscalls::CG_G2_HAVEWEGHOULMODELS::{
    CgG2Haveweghoulmodels, CgG2HaveweghoulmodelsArgs,
};
use mp_abi::cgame::syscalls::CG_G2_IKMOVE::{CgG2Ikmove, CgG2IkmoveArgs};
use mp_abi::cgame::syscalls::CG_G2_INITGHOUL2MODEL::{
    CgG2Initghoul2model, CgG2Initghoul2modelArgs,
};
use mp_abi::cgame::syscalls::CG_G2_LISTBONES::{CgG2Listbones, CgG2ListbonesArgs};
use mp_abi::cgame::syscalls::CG_G2_LISTSURFACES::{CgG2Listsurfaces, CgG2ListsurfacesArgs};
use mp_abi::cgame::syscalls::CG_G2_OVERRIDESERVER::{CgG2Overrideserver, CgG2OverrideserverArgs};
use mp_abi::cgame::syscalls::CG_G2_PLAYANIM::{CgG2Playanim, CgG2PlayanimArgs};
use mp_abi::cgame::syscalls::CG_G2_RAGEFFECTORGOAL::{
    CgG2Rageffectorgoal, CgG2RageffectorgoalArgs,
};
use mp_abi::cgame::syscalls::CG_G2_RAGEFFECTORKICK::{
    CgG2Rageffectorkick, CgG2RageffectorkickArgs,
};
use mp_abi::cgame::syscalls::CG_G2_RAGFORCESOLVE::{CgG2Ragforcesolve, CgG2RagforcesolveArgs};
use mp_abi::cgame::syscalls::CG_G2_RAGPCJCONSTRAINT::{
    CgG2Ragpcjconstraint, CgG2RagpcjconstraintArgs,
};
use mp_abi::cgame::syscalls::CG_G2_RAGPCJGRADIENTSPEED::{
    CgG2Ragpcjgradientspeed, CgG2RagpcjgradientspeedArgs,
};
use mp_abi::cgame::syscalls::CG_G2_REMOVEBONE::{CgG2Removebone, CgG2RemoveboneArgs};
use mp_abi::cgame::syscalls::CG_G2_REMOVEGHOUL2MODEL::{
    CgG2Removeghoul2model, CgG2Removeghoul2modelArgs,
};
use mp_abi::cgame::syscalls::CG_G2_SETBOLTON::{CgG2Setbolton, CgG2SetboltonArgs};
use mp_abi::cgame::syscalls::CG_G2_SETBONEIKSTATE::{CgG2Setboneikstate, CgG2SetboneikstateArgs};
use mp_abi::cgame::syscalls::CG_G2_SETMODELS::{CgG2Setmodels, CgG2SetmodelsArgs};
use mp_abi::cgame::syscalls::CG_G2_SETNEWORIGIN::{CgG2Setneworigin, CgG2SetneworiginArgs};
use mp_abi::cgame::syscalls::CG_G2_SETRAGDOLL::{CgG2Setragdoll, CgG2SetragdollArgs};
use mp_abi::cgame::syscalls::CG_G2_SETROOTSURFACE::{CgG2Setrootsurface, CgG2SetrootsurfaceArgs};
use mp_abi::cgame::syscalls::CG_G2_SETSKIN::{CgG2Setskin, CgG2SetskinArgs};
use mp_abi::cgame::syscalls::CG_G2_SETSURFACEONOFF::{
    CgG2Setsurfaceonoff, CgG2SetsurfaceonoffArgs,
};
use mp_abi::cgame::syscalls::CG_G2_SETTIME::{CgG2Settime, CgG2SettimeArgs};
use mp_abi::cgame::syscalls::CG_G2_SIZE::{CgG2Size, CgG2SizeArgs};
use mp_abi::cgame::syscalls::CG_G2_SKINLESSMODEL::{CgG2Skinlessmodel, CgG2SkinlessmodelArgs};
use mp_abi::cgame::syscalls::CG_GETCURRENTCMDNUMBER::{
    CgGetcurrentcmdnumber, CgGetcurrentcmdnumberArgs,
};
use mp_abi::cgame::syscalls::CG_GETCURRENTSNAPSHOTNUMBER::{
    CgGetcurrentsnapshotnumber, CgGetcurrentsnapshotnumberArgs,
};
use mp_abi::cgame::syscalls::CG_GETDEFAULTSTATE::{CgGetdefaultstate, CgGetdefaultstateArgs};
use mp_abi::cgame::syscalls::CG_GETGAMESTATE::{CgGetgamestate, CgGetgamestateArgs};
use mp_abi::cgame::syscalls::CG_GETGLCONFIG::{CgGetglconfig, CgGetglconfigArgs};
use mp_abi::cgame::syscalls::CG_GETSERVERCOMMAND::{CgGetservercommand, CgGetservercommandArgs};
use mp_abi::cgame::syscalls::CG_GETSNAPSHOT::{CgGetsnapshot, CgGetsnapshotArgs};
use mp_abi::cgame::syscalls::CG_GETUSERCMD::{CgGetusercmd, CgGetusercmdArgs};
use mp_abi::cgame::syscalls::CG_GET_ENTITY_TOKEN::{CgGetEntityToken, CgGetEntityTokenArgs};
use mp_abi::cgame::syscalls::CG_KEY_GETCATCHER::{CgKeyGetcatcher, CgKeyGetcatcherArgs};
use mp_abi::cgame::syscalls::CG_KEY_GETKEY::{CgKeyGetkey, CgKeyGetkeyArgs};
use mp_abi::cgame::syscalls::CG_KEY_ISDOWN::{CgKeyIsdown, CgKeyIsdownArgs};
use mp_abi::cgame::syscalls::CG_KEY_SETCATCHER::{CgKeySetcatcher, CgKeySetcatcherArgs};
use mp_abi::cgame::syscalls::CG_LANGUAGE_ISASIAN::{CgLanguageIsasian, CgLanguageIsasianArgs};
use mp_abi::cgame::syscalls::CG_LANGUAGE_USESSPACES::{
    CgLanguageUsesspaces, CgLanguageUsesspacesArgs,
};
use mp_abi::cgame::syscalls::CG_MEMORY_REMAINING::{CgMemoryRemaining, CgMemoryRemainingArgs};
use mp_abi::cgame::syscalls::CG_MILLISECONDS::{CgMilliseconds, CgMillisecondsArgs};
use mp_abi::cgame::syscalls::CG_OPENUIMENU::{CgOpenuimenu, CgOpenuimenuArgs};
use mp_abi::cgame::syscalls::CG_PC_ADD_GLOBAL_DEFINE::{
    CgPcAddGlobalDefine, CgPcAddGlobalDefineArgs,
};
use mp_abi::cgame::syscalls::CG_PC_FREE_SOURCE::{CgPcFreeSource, CgPcFreeSourceArgs};
use mp_abi::cgame::syscalls::CG_PC_LOAD_GLOBAL_DEFINES::{
    CgPcLoadGlobalDefines, CgPcLoadGlobalDefinesArgs,
};
use mp_abi::cgame::syscalls::CG_PC_LOAD_SOURCE::{CgPcLoadSource, CgPcLoadSourceArgs};
use mp_abi::cgame::syscalls::CG_PC_READ_TOKEN::{CgPcReadToken, CgPcReadTokenArgs};
use mp_abi::cgame::syscalls::CG_PC_REMOVE_ALL_GLOBAL_DEFINES::{
    CgPcRemoveAllGlobalDefines, CgPcRemoveAllGlobalDefinesArgs,
};
use mp_abi::cgame::syscalls::CG_PC_SOURCE_FILE_AND_LINE::{
    CgPcSourceFileAndLine, CgPcSourceFileAndLineArgs,
};
use mp_abi::cgame::syscalls::CG_PRECISIONTIMER_END::{
    CgPrecisiontimerEnd, CgPrecisiontimerEndArgs,
};
use mp_abi::cgame::syscalls::CG_PRECISIONTIMER_START::{
    CgPrecisiontimerStart, CgPrecisiontimerStartArgs,
};
use mp_abi::cgame::syscalls::CG_PRINT::{CgPrint, CgPrintArgs};
use mp_abi::cgame::syscalls::CG_REAL_TIME::{CgRealTime, CgRealTimeArgs};
use mp_abi::cgame::syscalls::CG_REMOVECOMMAND::{CgRemovecommand, CgRemovecommandArgs};
use mp_abi::cgame::syscalls::CG_RE_INIT_RENDERER_TERRAIN::{
    CgReInitRendererTerrain, CgReInitRendererTerrainArgs,
};
use mp_abi::cgame::syscalls::CG_RMG_INIT::{CgRmgInit, CgRmgInitArgs};
use mp_abi::cgame::syscalls::CG_ROFF_CACHE::{CgRoffCache, CgRoffCacheArgs};
use mp_abi::cgame::syscalls::CG_ROFF_CLEAN::{CgRoffClean, CgRoffCleanArgs};
use mp_abi::cgame::syscalls::CG_ROFF_PLAY::{CgRoffPlay, CgRoffPlayArgs};
use mp_abi::cgame::syscalls::CG_ROFF_PURGE_ENT::{CgRoffPurgeEnt, CgRoffPurgeEntArgs};
use mp_abi::cgame::syscalls::CG_ROFF_UPDATE_ENTITIES::{
    CgRoffUpdateEntities, CgRoffUpdateEntitiesArgs,
};
use mp_abi::cgame::syscalls::CG_R_ADDADDITIVELIGHTTOSCENE::{
    CgRAddadditivelighttoscene, CgRAddadditivelighttosceneArgs,
};
use mp_abi::cgame::syscalls::CG_R_ADDDECALTOSCENE::{CgRAdddecaltoscene, CgRAdddecaltosceneArgs};
use mp_abi::cgame::syscalls::CG_R_ADDLIGHTTOSCENE::{CgRAddlighttoscene, CgRAddlighttosceneArgs};
use mp_abi::cgame::syscalls::CG_R_ADDPOLYSTOSCENE::{CgRAddpolystoscene, CgRAddpolystosceneArgs};
use mp_abi::cgame::syscalls::CG_R_ADDPOLYTOSCENE::{CgRAddpolytoscene, CgRAddpolytosceneArgs};
use mp_abi::cgame::syscalls::CG_R_ADDREFENTITYTOSCENE::{
    CgRAddrefentitytoscene, CgRAddrefentitytosceneArgs,
};
use mp_abi::cgame::syscalls::CG_R_AUTOMAPELEVADJ::{CgRAutomapelevadj, CgRAutomapelevadjArgs};
use mp_abi::cgame::syscalls::CG_R_CLEARDECALS::CgRCleardecals;
use mp_abi::cgame::syscalls::CG_R_CLEARSCENE::{CgRClearscene, CgRClearsceneArgs};
use mp_abi::cgame::syscalls::CG_R_DRAWROTATEPIC::{CgRDrawrotatepic, CgRDrawrotatepicArgs};
use mp_abi::cgame::syscalls::CG_R_DRAWROTATEPIC2::{CgRDrawrotatepic2, CgRDrawrotatepic2Args};
use mp_abi::cgame::syscalls::CG_R_DRAWSTRETCHPIC::{CgRDrawstretchpic, CgRDrawstretchpicArgs};
use mp_abi::cgame::syscalls::CG_R_FONT_DRAWSTRING::{CgRFontDrawstring, CgRFontDrawstringArgs};
use mp_abi::cgame::syscalls::CG_R_FONT_STRHEIGHTPIXELS::{
    CgRFontStrheightpixels, CgRFontStrheightpixelsArgs,
};
use mp_abi::cgame::syscalls::CG_R_FONT_STRLENCHARS::{CgRFontStrlenchars, CgRFontStrlencharsArgs};
use mp_abi::cgame::syscalls::CG_R_FONT_STRLENPIXELS::{
    CgRFontStrlenpixels, CgRFontStrlenpixelsArgs,
};
use mp_abi::cgame::syscalls::CG_R_GETDISTANCECULL::{CgRGetdistancecull, CgRGetdistancecullArgs};
use mp_abi::cgame::syscalls::CG_R_GETREALRES::{CgRGetrealres, CgRGetrealresArgs};
use mp_abi::cgame::syscalls::CG_R_GET_BMODEL_VERTS::{CgRGetBmodelVerts, CgRGetBmodelVertsArgs};
use mp_abi::cgame::syscalls::CG_R_GET_LIGHT_STYLE::{CgRGetLightStyle, CgRGetLightStyleArgs};
use mp_abi::cgame::syscalls::CG_R_INITWIREFRAMEAUTO::{
    CgRInitwireframeauto, CgRInitwireframeautoArgs,
};
use mp_abi::cgame::syscalls::CG_R_INPVS::{CgRInpvs, CgRInpvsArgs};
use mp_abi::cgame::syscalls::CG_R_LERPTAG::{CgRLerptag, CgRLerptagArgs};
use mp_abi::cgame::syscalls::CG_R_LIGHTFORPOINT::{CgRLightforpoint, CgRLightforpointArgs};
use mp_abi::cgame::syscalls::CG_R_LOADWORLDMAP::{CgRLoadworldmap, CgRLoadworldmapArgs};
use mp_abi::cgame::syscalls::CG_R_MODELBOUNDS::{CgRModelbounds, CgRModelboundsArgs};
use mp_abi::cgame::syscalls::CG_R_REGISTERFONT::{CgRRegisterfont, CgRRegisterfontArgs};
use mp_abi::cgame::syscalls::CG_R_REGISTERMODEL::{CgRRegistermodel, CgRRegistermodelArgs};
use mp_abi::cgame::syscalls::CG_R_REGISTERSHADER::{CgRRegistershader, CgRRegistershaderArgs};
use mp_abi::cgame::syscalls::CG_R_REGISTERSHADERNOMIP::{
    CgRRegistershadernomip, CgRRegistershadernomipArgs,
};
use mp_abi::cgame::syscalls::CG_R_REGISTERSKIN::{CgRRegisterskin, CgRRegisterskinArgs};
use mp_abi::cgame::syscalls::CG_R_REMAP_SHADER::{CgRRemapShader, CgRRemapShaderArgs};
use mp_abi::cgame::syscalls::CG_R_RENDERSCENE::{CgRRenderscene, CgRRendersceneArgs};
use mp_abi::cgame::syscalls::CG_R_SETCOLOR::{CgRSetcolor, CgRSetcolorArgs};
use mp_abi::cgame::syscalls::CG_R_SETRANGEFOG::{CgRSetrangefog, CgRSetrangefogArgs};
use mp_abi::cgame::syscalls::CG_R_SETREFRACTIONPROP::{
    CgRSetrefractionprop, CgRSetrefractionpropArgs,
};
use mp_abi::cgame::syscalls::CG_R_SET_LIGHT_STYLE::{CgRSetLightStyle, CgRSetLightStyleArgs};
use mp_abi::cgame::syscalls::CG_R_WEATHER_CONTENTS_OVERRIDE::{
    CgRWeatherContentsOverride, CgRWeatherContentsOverrideArgs,
};
use mp_abi::cgame::syscalls::CG_R_WORLDEFFECTCOMMAND::{
    CgRWorldeffectcommand, CgRWorldeffectcommandArgs,
};
use mp_abi::cgame::syscalls::CG_SENDCLIENTCOMMAND::{CgSendclientcommand, CgSendclientcommandArgs};
use mp_abi::cgame::syscalls::CG_SENDCONSOLECOMMAND::{
    CgSendconsolecommand, CgSendconsolecommandArgs,
};
use mp_abi::cgame::syscalls::CG_SETCLIENTFORCEANGLE::{
    CgSetclientforceangle, CgSetclientforceangleArgs,
};
use mp_abi::cgame::syscalls::CG_SETCLIENTTURNEXTENT::{
    CgSetclientturnextent, CgSetclientturnextentArgs,
};
use mp_abi::cgame::syscalls::CG_SETUSERCMDVALUE::{CgSetusercmdvalue, CgSetusercmdvalueArgs};
use mp_abi::cgame::syscalls::CG_SET_SHARED_BUFFER::{CgSetSharedBuffer, CgSetSharedBufferArgs};
use mp_abi::cgame::syscalls::CG_SNAPVECTOR::{CgSnapvector, CgSnapvectorArgs};
use mp_abi::cgame::syscalls::CG_SP_GETSTRINGTEXTSTRING::{
    CgSpGetstringtextstring, CgSpGetstringtextstringArgs,
};
use mp_abi::cgame::syscalls::CG_S_ADDLOCALSET::{CgSAddlocalset, CgSAddlocalsetArgs};
use mp_abi::cgame::syscalls::CG_S_ADDLOOPINGSOUND::{CgSAddloopingsound, CgSAddloopingsoundArgs};
use mp_abi::cgame::syscalls::CG_S_ADDREALLOOPINGSOUND::{
    CgSAddrealloopingsound, CgSAddrealloopingsoundArgs,
};
use mp_abi::cgame::syscalls::CG_S_CLEARLOOPINGSOUNDS::{
    CgSClearloopingsounds, CgSClearloopingsoundsArgs,
};
use mp_abi::cgame::syscalls::CG_S_GETVOICEVOLUME::{CgSGetvoicevolume, CgSGetvoicevolumeArgs};
use mp_abi::cgame::syscalls::CG_S_MUTESOUND::{CgSMutesound, CgSMutesoundArgs};
use mp_abi::cgame::syscalls::CG_S_REGISTERSOUND::{CgSRegistersound, CgSRegistersoundArgs};
use mp_abi::cgame::syscalls::CG_S_RESPATIALIZE::{CgSRespatialize, CgSRespatializeArgs};
use mp_abi::cgame::syscalls::CG_S_SHUTUP::{CgSShutup, CgSShutupArgs};
use mp_abi::cgame::syscalls::CG_S_STARTBACKGROUNDTRACK::{
    CgSStartbackgroundtrack, CgSStartbackgroundtrackArgs,
};
use mp_abi::cgame::syscalls::CG_S_STARTLOCALSOUND::{CgSStartlocalsound, CgSStartlocalsoundArgs};
use mp_abi::cgame::syscalls::CG_S_STARTSOUND::{CgSStartsound, CgSStartsoundArgs};
use mp_abi::cgame::syscalls::CG_S_STOPBACKGROUNDTRACK::{
    CgSStopbackgroundtrack, CgSStopbackgroundtrackArgs,
};
use mp_abi::cgame::syscalls::CG_S_STOPLOOPINGSOUND::{
    CgSStoploopingsound, CgSStoploopingsoundArgs,
};
use mp_abi::cgame::syscalls::CG_S_UPDATEAMBIENTSET::{
    CgSUpdateambientset, CgSUpdateambientsetArgs,
};
use mp_abi::cgame::syscalls::CG_S_UPDATEENTITYPOSITION::{
    CgSUpdateentityposition, CgSUpdateentitypositionArgs,
};
use mp_abi::cgame::syscalls::CG_TRUEFREE::{CgTruefree, CgTruefreeArgs};
use mp_abi::cgame::syscalls::CG_TRUEMALLOC::{CgTruemalloc, CgTruemallocArgs};
use mp_abi::cgame::syscalls::CG_UPDATESCREEN::{CgUpdatescreen, CgUpdatescreenArgs};
use mp_abi::cgame::syscalls::CG_WE_ADDWEATHERZONE::{CgWeAddweatherzone, CgWeAddweatherzoneArgs};
use mp_abi::Execute;
use mp_engine_select::Engine;
use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::qcommon::{
    entityState_t, qtime_t, sharedRagDollParams_t, sharedRagDollUpdateParams_t,
    sharedSetBoneIKStateParams_t, usercmd_t,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{
    addElectricityArgStruct_t, addbezierArgStruct_t, addpolyArgStruct_t, addspriteArgStruct_t,
    clipHandle_t, effectTrailArgStruct_t, fileHandle_t, fsMode_t, gameState_t, mdxaBone_t,
    orientation_t, pc_token_t, qhandle_t, sfxHandle_t, sharedIKMoveParams_t, vec3_t, vec4_t,
    vmCvar_t, CollisionRecord_t,
};
use native_string::{buf_to_string, cstr, latin1_to_string, string_to_latin1};

/// Raven `trap_Print` — `CG_PRINT` (token: `mp_abi::cgame::syscalls::CG_PRINT`).
///
/// C: `void trap_Print( const char *fmt )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:21-23`
pub fn Print(engine: &Engine, fmt: &str) {
    let fmt_c = cstr(fmt);
    // SAFETY: the message outlives the synchronous syscall the Args feed.
    let args = unsafe { CgPrintArgs::new(fmt_c.as_ptr()) };
    <Engine as Execute<CgPrint>>::execute(engine, args)
}

/// Raven `trap_Error` — `CG_ERROR` (token: `mp_abi::cgame::syscalls::CG_ERROR`).
///
/// C: `void trap_Error( const char *fmt )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:25-27`
pub fn Error(engine: &Engine, fmt: &str) {
    let fmt_c = cstr(fmt);
    // SAFETY: the message outlives the synchronous syscall the Args feed.
    let args = unsafe { CgErrorArgs::new(fmt_c.as_ptr()) };
    <Engine as Execute<CgError>>::execute(engine, args)
}

/// Raven `trap_Milliseconds` — `CG_MILLISECONDS`
/// (token: `mp_abi::cgame::syscalls::CG_MILLISECONDS`).
///
/// C: `int trap_Milliseconds( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:29-31`
pub fn Milliseconds(engine: &Engine) -> c_int {
    <Engine as Execute<CgMilliseconds>>::execute(engine, CgMillisecondsArgs::new())
}

/// Raven `trap_PrecisionTimer_Start` — `CG_PRECISIONTIMER_START`
/// (token: `mp_abi::cgame::syscalls::CG_PRECISIONTIMER_START`).
///
/// C: `void trap_PrecisionTimer_Start(void **theNewTimer)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:39-42`
///
/// `theNewTimer` is an opaque engine-owned timer slot the syscall fills
/// in-place; it is never dereferenced on the cgame side.
pub fn PrecisionTimer_Start(engine: &Engine, theNewTimer: *mut *mut c_void) {
    <Engine as Execute<CgPrecisiontimerStart>>::execute(
        engine,
        CgPrecisiontimerStartArgs::new(theNewTimer),
    )
}

/// Raven `trap_PrecisionTimer_End` — `CG_PRECISIONTIMER_END`
/// (token: `mp_abi::cgame::syscalls::CG_PRECISIONTIMER_END`).
///
/// C: `int trap_PrecisionTimer_End(void *theTimer)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:45-48`
pub fn PrecisionTimer_End(engine: &Engine, theTimer: *mut c_void) -> c_int {
    <Engine as Execute<CgPrecisiontimerEnd>>::execute(
        engine,
        CgPrecisiontimerEndArgs::new(theTimer),
    )
}

/// Raven `trap_Cvar_Register` — `CG_CVAR_REGISTER`
/// (token: `mp_abi::cgame::syscalls::CG_CVAR_REGISTER`).
///
/// C: `void trap_Cvar_Register( vmCvar_t *vmCvar, const char *varName, const char *defaultValue, int flags )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:50-52`
pub fn Cvar_Register(
    engine: &Engine,
    vmCvar: Option<&mut vmCvar_t>,
    varName: &str,
    defaultValue: &str,
    flags: c_int,
) {
    let vm_cvar_ptr = vmCvar.map_or(null_mut(), |c| c as *mut vmCvar_t);
    let var_name_c = cstr(varName);
    let default_value_c = cstr(defaultValue);
    // SAFETY: every pointer outlives the synchronous syscall the Args feed.
    let args = unsafe {
        CgCvarRegisterArgs::new(
            vm_cvar_ptr,
            var_name_c.as_ptr(),
            default_value_c.as_ptr(),
            flags,
        )
    };
    <Engine as Execute<CgCvarRegister>>::execute(engine, args)
}

/// Raven `trap_Cvar_Update` — `CG_CVAR_UPDATE`
/// (token: `mp_abi::cgame::syscalls::CG_CVAR_UPDATE`).
///
/// C: `void trap_Cvar_Update( vmCvar_t *vmCvar )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:54-56`
pub fn Cvar_Update(engine: &Engine, vmCvar: &mut vmCvar_t) {
    <Engine as Execute<CgCvarUpdate>>::execute(
        engine,
        CgCvarUpdateArgs::new(vmCvar as *mut vmCvar_t),
    )
}

/// Raven `trap_Cvar_Set` — `CG_CVAR_SET` (token: `mp_abi::cgame::syscalls::CG_CVAR_SET`).
///
/// C: `void trap_Cvar_Set( const char *var_name, const char *value )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:58-60`
pub fn Cvar_Set(engine: &Engine, var_name: &str, value: &str) {
    let var_name_c = cstr(var_name);
    let value_c = cstr(value);
    // SAFETY: both strings outlive the synchronous syscall the Args feed.
    let args = unsafe { CgCvarSetArgs::new(var_name_c.as_ptr(), value_c.as_ptr()) };
    <Engine as Execute<CgCvarSet>>::execute(engine, args)
}

/// Raven `trap_Cvar_VariableStringBuffer` — `CG_CVAR_VARIABLESTRINGBUFFER`
/// (token: `mp_abi::cgame::syscalls::CG_CVAR_VARIABLESTRINGBUFFER`).
///
/// The out-buffer carries a cvar *value* — free text the engine treats as
/// opaque bytes — so it decodes Latin-1, one `char` per wire byte.
///
/// C: `void trap_Cvar_VariableStringBuffer( const char *var_name, char *buffer, int bufsize )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:62-64`
pub fn Cvar_VariableStringBuffer(engine: &Engine, var_name: &str, buffer_len: usize) -> String {
    let var_name_c = cstr(var_name);
    let mut buffer = vec![0u8; buffer_len];
    // SAFETY: name and buffer outlive the synchronous syscall the Args feed.
    let args = unsafe {
        CgCvarVariablestringbufferArgs::new(
            var_name_c.as_ptr(),
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        )
    };
    <Engine as Execute<CgCvarVariablestringbuffer>>::execute(engine, args);
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_Cvar_GetHiddenVarValue` — `CG_CVAR_GETHIDDENVALUE`
/// (token: `mp_abi::cgame::syscalls::CG_CVAR_GETHIDDENVALUE`).
///
/// C: `int trap_Cvar_GetHiddenVarValue(const char *name)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:66-69`
pub fn Cvar_GetHiddenVarValue(engine: &Engine, name: &str) -> c_int {
    let name_c = cstr(name);
    // SAFETY: the name outlives the synchronous syscall the Args feed.
    let args = unsafe { CgCvarGethiddenvalueArgs::new(name_c.as_ptr()) };
    <Engine as Execute<CgCvarGethiddenvalue>>::execute(engine, args)
}

/// Raven `trap_Argc` — `CG_ARGC` (token: `mp_abi::cgame::syscalls::CG_ARGC`).
///
/// C: `int trap_Argc( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:71-73`
pub fn Argc(engine: &Engine) -> c_int {
    <Engine as Execute<CgArgc>>::execute(engine, CgArgcArgs::new())
}

/// Raven `trap_Argv` — `CG_ARGV` (token: `mp_abi::cgame::syscalls::CG_ARGV`).
///
/// C: `void trap_Argv( int n, char *buffer, int bufferLength )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:75-77`
pub fn Argv(engine: &Engine, n: c_int, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    // SAFETY: `buffer` outlives the synchronous syscall the Args feed.
    let args =
        unsafe { CgArgvArgs::new(n, buffer.as_mut_ptr() as *mut c_char, buffer_len as c_int) };
    <Engine as Execute<CgArgv>>::execute(engine, args);
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_Args` — `CG_ARGS` (token: `mp_abi::cgame::syscalls::CG_ARGS`).
///
/// C: `void trap_Args( char *buffer, int bufferLength )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:79-81`
pub fn Args(engine: &Engine, buffer_len: usize) -> String {
    let mut buffer = vec![0u8; buffer_len];
    // SAFETY: `buffer` outlives the synchronous syscall the Args feed.
    let args = unsafe { CgArgsArgs::new(buffer.as_mut_ptr() as *mut c_char, buffer_len as c_int) };
    <Engine as Execute<CgArgs>>::execute(engine, args);
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_FS_FOpenFile` — `CG_FS_FOPENFILE`
/// (token: `mp_abi::cgame::syscalls::CG_FS_FOPENFILE`).
///
/// C: `int trap_FS_FOpenFile( const char *qpath, fileHandle_t *f, fsMode_t mode )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:83-85`
pub fn FS_FOpenFile(engine: &Engine, qpath: &str, f: &mut fileHandle_t, mode: fsMode_t) -> c_int {
    let qpath_c = cstr(qpath);
    // SAFETY: path and handle slot outlive the synchronous syscall the Args feed.
    let args = unsafe { CgFsFopenfileArgs::new(qpath_c.as_ptr(), f as *mut fileHandle_t, mode) };
    <Engine as Execute<CgFsFopenfile>>::execute(engine, args)
}

/// Raven `trap_FS_Read` — `CG_FS_READ` (token: `mp_abi::cgame::syscalls::CG_FS_READ`).
///
/// C: `void trap_FS_Read( void *buffer, int len, fileHandle_t f )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:87-89`
pub fn FS_Read(engine: &Engine, buffer: &mut [u8], f: fileHandle_t) {
    // SAFETY: `buffer` outlives the synchronous syscall the Args feed.
    let args = unsafe { CgFsReadArgs::new(buffer.as_mut_ptr(), buffer.len() as c_int, f) };
    <Engine as Execute<CgFsRead>>::execute(engine, args)
}

/// Raven `trap_FS_Write` — `CG_FS_WRITE` (token: `mp_abi::cgame::syscalls::CG_FS_WRITE`).
///
/// C: `void trap_FS_Write( const void *buffer, int len, fileHandle_t f )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:91-93`
pub fn FS_Write(engine: &Engine, buffer: &[u8], f: fileHandle_t) {
    <Engine as Execute<CgFsWrite>>::execute(
        engine,
        CgFsWriteArgs::new(buffer.as_ptr(), buffer.len() as c_int, f),
    )
}

/// Raven `trap_FS_FCloseFile` — `CG_FS_FCLOSEFILE`
/// (token: `mp_abi::cgame::syscalls::CG_FS_FCLOSEFILE`).
///
/// C: `void trap_FS_FCloseFile( fileHandle_t f )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:95-97`
pub fn FS_FCloseFile(engine: &Engine, f: fileHandle_t) {
    <Engine as Execute<CgFsFclosefile>>::execute(engine, CgFsFclosefileArgs::new(f))
}

/// Raven `trap_FS_GetFileList` — `CG_FS_GETFILELIST`
/// (token: `mp_abi::cgame::syscalls::CG_FS_GETFILELIST`).
///
/// C: `int trap_FS_GetFileList( const char *path, const char *extension, char *listbuf, int bufsize )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:99-101`
///
/// `listbuf` stays a caller byte buffer: the engine packs NUL-separated names,
/// and callers walk them by offset.
pub fn FS_GetFileList(engine: &Engine, path: &str, extension: &str, listbuf: &mut [u8]) -> c_int {
    let path_c = cstr(path);
    let extension_c = cstr(extension);
    // SAFETY: both strings and the list buffer outlive the synchronous syscall.
    let args = unsafe {
        CgFsGetfilelistArgs::new(
            path_c.as_ptr(),
            extension_c.as_ptr(),
            listbuf.as_mut_ptr() as *mut c_char,
            listbuf.len() as c_int,
        )
    };
    <Engine as Execute<CgFsGetfilelist>>::execute(engine, args)
}

/// Raven `trap_SendConsoleCommand` — `CG_SENDCONSOLECOMMAND`
/// (token: `mp_abi::cgame::syscalls::CG_SENDCONSOLECOMMAND`).
///
/// C: `void trap_SendConsoleCommand( const char *text )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:103-105`
pub fn SendConsoleCommand(engine: &Engine, text: &str) {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<CgSendconsolecommand>>::execute(
        engine,
        CgSendconsolecommandArgs::new(text_c.as_ptr()),
    )
}

/// Raven `trap_AddCommand` — `CG_ADDCOMMAND` (token: `mp_abi::cgame::syscalls::CG_ADDCOMMAND`).
///
/// C: `void trap_AddCommand( const char *cmdName )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:107-109`
pub fn AddCommand(engine: &Engine, cmdName: &str) {
    let cmd_name_c = cstr(cmdName);
    <Engine as Execute<CgAddcommand>>::execute(engine, CgAddcommandArgs::new(cmd_name_c.as_ptr()))
}

/// Raven `trap_RemoveCommand` — `CG_REMOVECOMMAND`
/// (token: `mp_abi::cgame::syscalls::CG_REMOVECOMMAND`).
///
/// C: `void trap_RemoveCommand( const char *cmdName )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:111-113`
pub fn RemoveCommand(engine: &Engine, cmdName: &str) {
    let cmd_name_c = cstr(cmdName);
    <Engine as Execute<CgRemovecommand>>::execute(
        engine,
        CgRemovecommandArgs::new(cmd_name_c.as_ptr()),
    )
}

/// Raven `trap_SendClientCommand` — `CG_SENDCLIENTCOMMAND`
/// (token: `mp_abi::cgame::syscalls::CG_SENDCLIENTCOMMAND`).
///
/// C: `void trap_SendClientCommand( const char *s )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:115-117`
pub fn SendClientCommand(engine: &Engine, s: &str) {
    let s_c = CString::new(string_to_latin1(s)).unwrap();
    // SAFETY: the command string outlives the synchronous syscall the Args feed.
    let args = unsafe { CgSendclientcommandArgs::new(s_c.as_ptr()) };
    <Engine as Execute<CgSendclientcommand>>::execute(engine, args)
}

/// Raven `trap_UpdateScreen` — `CG_UPDATESCREEN`
/// (token: `mp_abi::cgame::syscalls::CG_UPDATESCREEN`).
///
/// C: `void trap_UpdateScreen( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:119-121`
pub fn UpdateScreen(engine: &Engine) {
    <Engine as Execute<CgUpdatescreen>>::execute(engine, CgUpdatescreenArgs::new())
}

/// Raven `trap_CM_LoadMap` — `CG_CM_LOADMAP` (token: `mp_abi::cgame::syscalls::CG_CM_LOADMAP`).
///
/// C: `void trap_CM_LoadMap( const char *mapname, qboolean SubBSP )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:123-125`
pub fn CM_LoadMap(engine: &Engine, mapname: &str, SubBSP: bool) {
    let mapname_c = cstr(mapname);
    // SAFETY: the map name outlives the synchronous syscall the Args feed.
    let args = unsafe { CgCmLoadmapArgs::new(mapname_c.as_ptr(), c_int::from(SubBSP)) };
    <Engine as Execute<CgCmLoadmap>>::execute(engine, args)
}

/// Raven `trap_CM_NumInlineModels` — `CG_CM_NUMINLINEMODELS`
/// (token: `mp_abi::cgame::syscalls::CG_CM_NUMINLINEMODELS`).
///
/// C: `int trap_CM_NumInlineModels( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:127-129`
pub fn CM_NumInlineModels(engine: &Engine) -> c_int {
    <Engine as Execute<CgCmNuminlinemodels>>::execute(engine, CgCmNuminlinemodelsArgs::new())
}

/// Raven `trap_CM_InlineModel` — `CG_CM_INLINEMODEL`
/// (token: `mp_abi::cgame::syscalls::CG_CM_INLINEMODEL`).
///
/// C: `clipHandle_t trap_CM_InlineModel( int index )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:131-133`
pub fn CM_InlineModel(engine: &Engine, index: c_int) -> c_int {
    <Engine as Execute<CgCmInlinemodel>>::execute(engine, CgCmInlinemodelArgs::new(index))
}

/// Raven `trap_CM_TempBoxModel` — `CG_CM_TEMPBOXMODEL`
/// (token: `mp_abi::cgame::syscalls::CG_CM_TEMPBOXMODEL`).
///
/// C: `clipHandle_t trap_CM_TempBoxModel( const vec3_t mins, const vec3_t maxs )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:135-137`
///
/// Returns a `clipHandle_t`.
pub fn CM_TempBoxModel(engine: &Engine, mins: &vec3_t, maxs: &vec3_t) -> c_int {
    <Engine as Execute<CgCmTempboxmodel>>::execute(
        engine,
        CgCmTempboxmodelArgs::new(mins as *const vec3_t, maxs as *const vec3_t),
    )
}

/// Raven `trap_CM_TempCapsuleModel` — `CG_CM_TEMPCAPSULEMODEL`
/// (token: `mp_abi::cgame::syscalls::CG_CM_TEMPCAPSULEMODEL`).
///
/// C: `clipHandle_t trap_CM_TempCapsuleModel( const vec3_t mins, const vec3_t maxs )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:139-141`
///
/// Returns a `clipHandle_t`.
pub fn CM_TempCapsuleModel(engine: &Engine, mins: &vec3_t, maxs: &vec3_t) -> c_int {
    <Engine as Execute<CgCmTempcapsulemodel>>::execute(
        engine,
        CgCmTempcapsulemodelArgs::new(mins as *const vec3_t, maxs as *const vec3_t),
    )
}

/// Raven `trap_CM_PointContents` — `CG_CM_POINTCONTENTS`
/// (token: `mp_abi::cgame::syscalls::CG_CM_POINTCONTENTS`).
///
/// C: `int trap_CM_PointContents( const vec3_t p, clipHandle_t model )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:143-145`
pub fn CM_PointContents(engine: &Engine, p: &vec3_t, model: c_int) -> c_int {
    <Engine as Execute<CgCmPointcontents>>::execute(
        engine,
        CgCmPointcontentsArgs::new(p as *const vec3_t, model),
    )
}

/// Raven `trap_CM_TransformedPointContents` — `CG_CM_TRANSFORMEDPOINTCONTENTS`
/// (token: `mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDPOINTCONTENTS`).
///
/// C: `int trap_CM_TransformedPointContents( const vec3_t p, clipHandle_t model, const vec3_t origin, const vec3_t angles )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:147-149`
pub fn CM_TransformedPointContents(
    engine: &Engine,
    p: &vec3_t,
    model: c_int,
    origin: &vec3_t,
    angles: &vec3_t,
) -> c_int {
    <Engine as Execute<CgCmTransformedpointcontents>>::execute(
        engine,
        CgCmTransformedpointcontentsArgs::new(
            p as *const vec3_t,
            model,
            origin as *const vec3_t,
            angles as *const vec3_t,
        ),
    )
}

/// Raven `trap_CM_BoxTrace` — `CG_CM_BOXTRACE` (token: `mp_abi::cgame::syscalls::CG_CM_BOXTRACE`).
///
/// C: `void trap_CM_BoxTrace( trace_t *results, const vec3_t start, const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model, int brushmask )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:151-155`
#[allow(clippy::too_many_arguments)]
pub fn CM_BoxTrace(
    engine: &Engine,
    results: &mut trace_t,
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    model: c_int,
    brushmask: c_int,
) {
    // SAFETY: `results` and every vector outlive the synchronous syscall the Args feed.
    let args = unsafe {
        CgCmBoxtraceArgs::new(
            results as *mut trace_t,
            start as *const vec3_t,
            end as *const vec3_t,
            mins as *const vec3_t,
            maxs as *const vec3_t,
            model,
            brushmask,
        )
    };
    <Engine as Execute<CgCmBoxtrace>>::execute(engine, args)
}

/// Raven `trap_CM_CapsuleTrace` — `CG_CM_CAPSULETRACE`
/// (token: `mp_abi::cgame::syscalls::CG_CM_CAPSULETRACE`).
///
/// C: `void trap_CM_CapsuleTrace( trace_t *results, const vec3_t start, const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model, int brushmask )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:157-161`
#[allow(clippy::too_many_arguments)]
pub fn CM_CapsuleTrace(
    engine: &Engine,
    results: &mut trace_t,
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    model: c_int,
    brushmask: c_int,
) {
    <Engine as Execute<CgCmCapsuletrace>>::execute(
        engine,
        CgCmCapsuletraceArgs::new(
            results as *mut trace_t,
            start as *const vec3_t,
            end as *const vec3_t,
            mins as *const vec3_t,
            maxs as *const vec3_t,
            model,
            brushmask,
        ),
    )
}

/// Raven `trap_CM_TransformedBoxTrace` — `CG_CM_TRANSFORMEDBOXTRACE`
/// (token: `mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDBOXTRACE`).
///
/// C: `void trap_CM_TransformedBoxTrace( trace_t *results, const vec3_t start, const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model, int brushmask, const vec3_t origin, const vec3_t angles )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:163-168`
#[allow(clippy::too_many_arguments)]
pub fn CM_TransformedBoxTrace(
    engine: &Engine,
    results: &mut trace_t,
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    model: c_int,
    brushmask: c_int,
    origin: &vec3_t,
    angles: &vec3_t,
) {
    // SAFETY: `results` and every vector outlive the synchronous syscall the Args feed.
    let args = unsafe {
        CgCmTransformedboxtraceArgs::new(
            results as *mut trace_t,
            start as *const vec3_t,
            end as *const vec3_t,
            mins as *const vec3_t,
            maxs as *const vec3_t,
            model,
            brushmask,
            origin as *const vec3_t,
            angles as *const vec3_t,
        )
    };
    <Engine as Execute<CgCmTransformedboxtrace>>::execute(engine, args)
}

/// Raven `trap_CM_TransformedCapsuleTrace` — `CG_CM_TRANSFORMEDCAPSULETRACE`
/// (token: `mp_abi::cgame::syscalls::CG_CM_TRANSFORMEDCAPSULETRACE`).
///
/// C: `void trap_CM_TransformedCapsuleTrace( trace_t *results, const vec3_t start, const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model, int brushmask, const vec3_t origin, const vec3_t angles )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:170-175`
#[allow(clippy::too_many_arguments)]
pub fn CM_TransformedCapsuleTrace(
    engine: &Engine,
    results: &mut trace_t,
    start: &vec3_t,
    end: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    model: c_int,
    brushmask: c_int,
    origin: &vec3_t,
    angles: &vec3_t,
) {
    <Engine as Execute<CgCmTransformedcapsuletrace>>::execute(
        engine,
        CgCmTransformedcapsuletraceArgs::new(
            results as *mut trace_t,
            start as *const vec3_t,
            end as *const vec3_t,
            mins as *const vec3_t,
            maxs as *const vec3_t,
            model,
            brushmask,
            origin as *const vec3_t,
            angles as *const vec3_t,
        ),
    )
}

/// Raven `trap_CM_MarkFragments` — `CG_CM_MARKFRAGMENTS`
/// (token: `mp_abi::cgame::syscalls::CG_CM_MARKFRAGMENTS`).
///
/// C: `int trap_CM_MarkFragments( int numPoints, const vec3_t *points, const vec3_t projection, int maxPoints, vec3_t pointBuffer, int maxFragments, markFragment_t *fragmentBuffer )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:177-182`
///
/// Raven's explicit `numPoints`/`maxPoints`/`maxFragments` counts are dropped
/// in favor of the corresponding slice lengths.
pub fn CM_MarkFragments(
    engine: &Engine,
    points: &[vec3_t],
    projection: &vec3_t,
    pointBuffer: &mut [vec3_t],
    fragmentBuffer: &mut [markFragment_t],
) -> c_int {
    // SAFETY: `points`, `projection`, and both output buffers outlive the
    // synchronous syscall the Args feed.
    let args = unsafe {
        CgCmMarkfragmentsArgs::new(
            points.len() as c_int,
            points.as_ptr(),
            projection as *const vec3_t,
            pointBuffer.len() as c_int,
            pointBuffer.as_mut_ptr(),
            fragmentBuffer.len() as c_int,
            fragmentBuffer.as_mut_ptr(),
        )
    };
    <Engine as Execute<CgCmMarkfragments>>::execute(engine, args)
}

/// Raven `trap_S_GetVoiceVolume` — `CG_S_GETVOICEVOLUME`
/// (token: `mp_abi::cgame::syscalls::CG_S_GETVOICEVOLUME`).
///
/// C: `int trap_S_GetVoiceVolume( int entityNum )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:184-186`
pub fn S_GetVoiceVolume(engine: &Engine, entityNum: c_int) -> c_int {
    <Engine as Execute<CgSGetvoicevolume>>::execute(engine, CgSGetvoicevolumeArgs::new(entityNum))
}

/// Raven `trap_S_MuteSound` — `CG_S_MUTESOUND` (token: `mp_abi::cgame::syscalls::CG_S_MUTESOUND`).
///
/// C: `void trap_S_MuteSound( int entityNum, int entchannel )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:188-190`
pub fn S_MuteSound(engine: &Engine, entityNum: c_int, entchannel: c_int) {
    <Engine as Execute<CgSMutesound>>::execute(engine, CgSMutesoundArgs::new(entityNum, entchannel))
}

/// Raven `trap_S_StartSound` — `CG_S_STARTSOUND` (token: `mp_abi::cgame::syscalls::CG_S_STARTSOUND`).
///
/// C: `void trap_S_StartSound( vec3_t origin, int entityNum, int entchannel, sfxHandle_t sfx )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:192-194`
///
/// `None` is Raven's NULL origin — the engine then plays the sound at
/// `entityNum`'s own tracked position instead of a fixed point.
pub fn S_StartSound(
    engine: &Engine,
    origin: Option<&vec3_t>,
    entityNum: c_int,
    entchannel: c_int,
    sfx: sfxHandle_t,
) {
    let origin = origin.map_or(null(), |o| o as *const vec3_t);
    <Engine as Execute<CgSStartsound>>::execute(
        engine,
        CgSStartsoundArgs::new(origin, entityNum, entchannel, sfx),
    )
}

/// Raven `trap_S_StartLocalSound` — `CG_S_STARTLOCALSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_S_STARTLOCALSOUND`).
///
/// C: `void trap_S_StartLocalSound( sfxHandle_t sfx, int channelNum )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:196-198`
pub fn S_StartLocalSound(engine: &Engine, sfx: sfxHandle_t, channelNum: c_int) {
    <Engine as Execute<CgSStartlocalsound>>::execute(
        engine,
        CgSStartlocalsoundArgs::new(sfx, channelNum),
    )
}

/// Raven `trap_S_ClearLoopingSounds` — `CG_S_CLEARLOOPINGSOUNDS`
/// (token: `mp_abi::cgame::syscalls::CG_S_CLEARLOOPINGSOUNDS`).
///
/// C: `void trap_S_ClearLoopingSounds(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:200-202`
pub fn S_ClearLoopingSounds(engine: &Engine) {
    <Engine as Execute<CgSClearloopingsounds>>::execute(engine, CgSClearloopingsoundsArgs::new())
}

/// Raven `trap_S_AddLoopingSound` — `CG_S_ADDLOOPINGSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_S_ADDLOOPINGSOUND`).
///
/// C: `void trap_S_AddLoopingSound( int entityNum, const vec3_t origin, const vec3_t velocity, sfxHandle_t sfx )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:204-206`
pub fn S_AddLoopingSound(
    engine: &Engine,
    entityNum: c_int,
    origin: &vec3_t,
    velocity: &vec3_t,
    sfx: sfxHandle_t,
) {
    <Engine as Execute<CgSAddloopingsound>>::execute(
        engine,
        CgSAddloopingsoundArgs::new(
            entityNum,
            origin as *const vec3_t,
            velocity as *const vec3_t,
            sfx,
        ),
    )
}

/// Raven `trap_S_UpdateEntityPosition` — `CG_S_UPDATEENTITYPOSITION`
/// (token: `mp_abi::cgame::syscalls::CG_S_UPDATEENTITYPOSITION`).
///
/// C: `void trap_S_UpdateEntityPosition( int entityNum, const vec3_t origin )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:208-210`
pub fn S_UpdateEntityPosition(engine: &Engine, entityNum: c_int, origin: &vec3_t) {
    <Engine as Execute<CgSUpdateentityposition>>::execute(
        engine,
        CgSUpdateentitypositionArgs::new(entityNum, origin as *const vec3_t),
    )
}

/// Raven `trap_S_AddRealLoopingSound` — `CG_S_ADDREALLOOPINGSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_S_ADDREALLOOPINGSOUND`).
///
/// C: `void trap_S_AddRealLoopingSound( int entityNum, const vec3_t origin, const vec3_t velocity, sfxHandle_t sfx )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:212-214`
pub fn S_AddRealLoopingSound(
    engine: &Engine,
    entityNum: c_int,
    origin: &vec3_t,
    velocity: &vec3_t,
    sfx: sfxHandle_t,
) {
    <Engine as Execute<CgSAddrealloopingsound>>::execute(
        engine,
        CgSAddrealloopingsoundArgs::new(
            entityNum,
            origin as *const vec3_t,
            velocity as *const vec3_t,
            sfx,
        ),
    )
}

/// Raven `trap_S_StopLoopingSound` — `CG_S_STOPLOOPINGSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_S_STOPLOOPINGSOUND`).
///
/// C: `void trap_S_StopLoopingSound( int entityNum )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:216-218`
pub fn S_StopLoopingSound(engine: &Engine, entityNum: c_int) {
    <Engine as Execute<CgSStoploopingsound>>::execute(
        engine,
        CgSStoploopingsoundArgs::new(entityNum),
    )
}

/// Raven `trap_S_Respatialize` — `CG_S_RESPATIALIZE`
/// (token: `mp_abi::cgame::syscalls::CG_S_RESPATIALIZE`).
///
/// C: `void trap_S_Respatialize( int entityNum, const vec3_t origin, vec3_t axis[3], int inwater )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:220-222`
///
/// `axis` is read-only at this seam despite the C prototype's non-const array.
pub fn S_Respatialize(
    engine: &Engine,
    entityNum: c_int,
    origin: &vec3_t,
    axis: &[vec3_t; 3],
    inwater: c_int,
) {
    <Engine as Execute<CgSRespatialize>>::execute(
        engine,
        CgSRespatializeArgs::new(
            entityNum,
            origin as *const vec3_t,
            axis.as_ptr() as *const vec3_t,
            inwater,
        ),
    )
}

/// Raven `trap_S_ShutUp` — `CG_S_SHUTUP` (token: `mp_abi::cgame::syscalls::CG_S_SHUTUP`).
///
/// C: `void trap_S_ShutUp(qboolean shutUpFactor)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:224-227`
pub fn S_ShutUp(engine: &Engine, shutUpFactor: bool) {
    <Engine as Execute<CgSShutup>>::execute(engine, CgSShutupArgs::new(c_int::from(shutUpFactor)))
}

/// Raven `trap_S_RegisterSound` — `CG_S_REGISTERSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_S_REGISTERSOUND`).
///
/// C: `sfxHandle_t trap_S_RegisterSound( const char *sample )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:229-231`
pub fn S_RegisterSound(engine: &Engine, sample: &str) -> sfxHandle_t {
    let sample_c = cstr(sample);
    <Engine as Execute<CgSRegistersound>>::execute(
        engine,
        CgSRegistersoundArgs::new(sample_c.as_ptr()),
    )
}

/// Raven `trap_S_StartBackgroundTrack` — `CG_S_STARTBACKGROUNDTRACK`
/// (token: `mp_abi::cgame::syscalls::CG_S_STARTBACKGROUNDTRACK`).
///
/// C: `void trap_S_StartBackgroundTrack( const char *intro, const char *loop, qboolean bReturnWithoutStarting )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:233-235`
pub fn S_StartBackgroundTrack(
    engine: &Engine,
    intro: &str,
    loop_: &str,
    bReturnWithoutStarting: bool,
) {
    let intro_c = cstr(intro);
    let loop_c = cstr(loop_);
    <Engine as Execute<CgSStartbackgroundtrack>>::execute(
        engine,
        CgSStartbackgroundtrackArgs::new(
            intro_c.as_ptr(),
            loop_c.as_ptr(),
            c_int::from(bReturnWithoutStarting),
        ),
    )
}

/// Raven `trap_S_UpdateAmbientSet` — `CG_S_UPDATEAMBIENTSET`
/// (token: `mp_abi::cgame::syscalls::CG_S_UPDATEAMBIENTSET`).
///
/// C: `void trap_S_UpdateAmbientSet( const char *name, vec3_t origin )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:237-240`
///
/// `origin` is non-const in C but is an input here.
pub fn S_UpdateAmbientSet(engine: &Engine, name: &str, origin: &vec3_t) {
    let name_c = cstr(name);
    <Engine as Execute<CgSUpdateambientset>>::execute(
        engine,
        CgSUpdateambientsetArgs::new(name_c.as_ptr(), origin as *const vec3_t),
    )
}

/// Raven `trap_AS_ParseSets` — `CG_AS_PARSESETS` (token: `mp_abi::cgame::syscalls::CG_AS_PARSESETS`).
///
/// C: `void trap_AS_ParseSets( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:242-245`
pub fn AS_ParseSets(engine: &Engine) {
    <Engine as Execute<CgAsParsesets>>::execute(engine, CgAsParsesetsArgs::new())
}

/// Raven `trap_AS_AddPrecacheEntry` — `CG_AS_ADDPRECACHEENTRY`
/// (token: `mp_abi::cgame::syscalls::CG_AS_ADDPRECACHEENTRY`).
///
/// C: `void trap_AS_AddPrecacheEntry( const char *name )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:247-250`
pub fn AS_AddPrecacheEntry(engine: &Engine, name: &str) {
    let name_c = cstr(name);
    <Engine as Execute<CgAsAddprecacheentry>>::execute(
        engine,
        CgAsAddprecacheentryArgs::new(name_c.as_ptr()),
    )
}

/// Raven `trap_S_AddLocalSet` — `CG_S_ADDLOCALSET` (token: `mp_abi::cgame::syscalls::CG_S_ADDLOCALSET`).
///
/// C: `int trap_S_AddLocalSet( const char *name, vec3_t listener_origin, vec3_t origin, int entID, int time )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:252-255`
///
/// `listener_origin`/`origin` are non-const in C but are inputs here.
pub fn S_AddLocalSet(
    engine: &Engine,
    name: &str,
    listener_origin: &vec3_t,
    origin: &vec3_t,
    entID: c_int,
    time: c_int,
) -> c_int {
    let name_c = cstr(name);
    <Engine as Execute<CgSAddlocalset>>::execute(
        engine,
        CgSAddlocalsetArgs::new(
            name_c.as_ptr(),
            listener_origin as *const vec3_t,
            origin as *const vec3_t,
            entID,
            time,
        ),
    )
}

/// Raven `trap_AS_GetBModelSound` — `CG_AS_GETBMODELSOUND`
/// (token: `mp_abi::cgame::syscalls::CG_AS_GETBMODELSOUND`).
///
/// C: `sfxHandle_t trap_AS_GetBModelSound( const char *name, int stage )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:257-260`
pub fn AS_GetBModelSound(engine: &Engine, name: &str, stage: c_int) -> sfxHandle_t {
    let name_c = cstr(name);
    <Engine as Execute<CgAsGetbmodelsound>>::execute(
        engine,
        CgAsGetbmodelsoundArgs::new(name_c.as_ptr(), stage),
    )
}

/// Raven `trap_R_LoadWorldMap` — `CG_R_LOADWORLDMAP` (token: `mp_abi::cgame::syscalls::CG_R_LOADWORLDMAP`).
///
/// C: `void trap_R_LoadWorldMap( const char *mapname )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:262-264`
pub fn R_LoadWorldMap(engine: &Engine, mapname: &str) {
    let mapname_c = cstr(mapname);
    <Engine as Execute<CgRLoadworldmap>>::execute(
        engine,
        CgRLoadworldmapArgs::new(mapname_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterModel` — `CG_R_REGISTERMODEL`
/// (token: `mp_abi::cgame::syscalls::CG_R_REGISTERMODEL`).
///
/// C: `qhandle_t trap_R_RegisterModel( const char *name )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:266-268`
pub fn R_RegisterModel(engine: &Engine, name: &str) -> qhandle_t {
    let name_c = cstr(name);
    <Engine as Execute<CgRRegistermodel>>::execute(
        engine,
        CgRRegistermodelArgs::new(name_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterSkin` — `CG_R_REGISTERSKIN` (token: `mp_abi::cgame::syscalls::CG_R_REGISTERSKIN`).
///
/// C: `qhandle_t trap_R_RegisterSkin( const char *name )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:270-272`
pub fn R_RegisterSkin(engine: &Engine, name: &str) -> qhandle_t {
    let name_c = cstr(name);
    <Engine as Execute<CgRRegisterskin>>::execute(engine, CgRRegisterskinArgs::new(name_c.as_ptr()))
}

/// Raven `trap_R_RegisterShader` — `CG_R_REGISTERSHADER`
/// (token: `mp_abi::cgame::syscalls::CG_R_REGISTERSHADER`).
///
/// C: `qhandle_t trap_R_RegisterShader( const char *name )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:274-276`
pub fn R_RegisterShader(engine: &Engine, name: &str) -> qhandle_t {
    let name_c = cstr(name);
    <Engine as Execute<CgRRegistershader>>::execute(
        engine,
        CgRRegistershaderArgs::new(name_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterShaderNoMip` — `CG_R_REGISTERSHADERNOMIP`
/// (token: `mp_abi::cgame::syscalls::CG_R_REGISTERSHADERNOMIP`).
///
/// C: `qhandle_t trap_R_RegisterShaderNoMip( const char *name )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:278-280`
pub fn R_RegisterShaderNoMip(engine: &Engine, name: &str) -> qhandle_t {
    let name_c = cstr(name);
    <Engine as Execute<CgRRegistershadernomip>>::execute(
        engine,
        CgRRegistershadernomipArgs::new(name_c.as_ptr()),
    )
}

/// Raven `trap_R_RegisterFont` — `CG_R_REGISTERFONT` (token: `mp_abi::cgame::syscalls::CG_R_REGISTERFONT`).
///
/// C: `qhandle_t trap_R_RegisterFont( const char *fontName )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:282-285`
pub fn R_RegisterFont(engine: &Engine, fontName: &str) -> qhandle_t {
    let font_name_c = cstr(fontName);
    <Engine as Execute<CgRRegisterfont>>::execute(
        engine,
        CgRRegisterfontArgs::new(font_name_c.as_ptr()),
    )
}

/// Raven `trap_R_Font_StrLenPixels` — `CG_R_FONT_STRLENPIXELS`
/// (token: `mp_abi::cgame::syscalls::CG_R_FONT_STRLENPIXELS`).
///
/// C: `int trap_R_Font_StrLenPixels(const char *text, const int iFontIndex, const float scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:287-290`
pub fn R_Font_StrLenPixels(engine: &Engine, text: &str, iFontIndex: c_int, scale: f32) -> c_int {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<CgRFontStrlenpixels>>::execute(
        engine,
        CgRFontStrlenpixelsArgs::new(text_c.as_ptr(), iFontIndex, scale),
    )
}

/// Raven `trap_R_Font_StrLenChars` — `CG_R_FONT_STRLENCHARS`
/// (token: `mp_abi::cgame::syscalls::CG_R_FONT_STRLENCHARS`).
///
/// C: `int trap_R_Font_StrLenChars(const char *text)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:292-295`
pub fn R_Font_StrLenChars(engine: &Engine, text: &str) -> c_int {
    let text_c = CString::new(string_to_latin1(text)).unwrap();
    <Engine as Execute<CgRFontStrlenchars>>::execute(engine, CgRFontStrlencharsArgs::new(text_c))
}

/// Raven `trap_R_Font_HeightPixels` — `CG_R_FONT_STRHEIGHTPIXELS`
/// (token: `mp_abi::cgame::syscalls::CG_R_FONT_STRHEIGHTPIXELS`).
///
/// C: `int trap_R_Font_HeightPixels(const int iFontIndex, const float scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:297-300`
pub fn R_Font_HeightPixels(engine: &Engine, iFontIndex: c_int, scale: f32) -> c_int {
    <Engine as Execute<CgRFontStrheightpixels>>::execute(
        engine,
        CgRFontStrheightpixelsArgs::new(iFontIndex, scale),
    )
}

/// Raven `trap_R_Font_DrawString` — `CG_R_FONT_DRAWSTRING`
/// (token: `mp_abi::cgame::syscalls::CG_R_FONT_DRAWSTRING`).
///
/// C: `void trap_R_Font_DrawString(int ox, int oy, const char *text, const float *rgba, const int setIndex, int iCharLimit, const float scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:302-305`
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
    <Engine as Execute<CgRFontDrawstring>>::execute(
        engine,
        CgRFontDrawstringArgs::new(ox, oy, text_c, rgba.as_ptr(), setIndex, iCharLimit, scale),
    )
}

/// Raven `trap_Language_IsAsian` — `CG_LANGUAGE_ISASIAN`
/// (token: `mp_abi::cgame::syscalls::CG_LANGUAGE_ISASIAN`).
///
/// C: `qboolean trap_Language_IsAsian(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:307-310`
pub fn Language_IsAsian(engine: &Engine) -> bool {
    <Engine as Execute<CgLanguageIsasian>>::execute(engine, CgLanguageIsasianArgs::new()) != 0
}

/// Raven `trap_Language_UsesSpaces` — `CG_LANGUAGE_USESSPACES`
/// (token: `mp_abi::cgame::syscalls::CG_LANGUAGE_USESSPACES`).
///
/// C: `qboolean trap_Language_UsesSpaces(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:312-315`
pub fn Language_UsesSpaces(engine: &Engine) -> bool {
    <Engine as Execute<CgLanguageUsesspaces>>::execute(engine, CgLanguageUsesspacesArgs::new()) != 0
}

/// Raven `trap_AnyLanguage_ReadCharFromString` — `CG_ANYLANGUAGE_READCHARFROMSTRING`
/// (token: `mp_abi::cgame::syscalls::CG_ANYLANGUAGE_READCHARFROMSTRING`).
///
/// C: `unsigned int trap_AnyLanguage_ReadCharFromString( const char *psText, int *piAdvanceCount, qboolean *pbIsTrailingPunctuation)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:317-320`
///
/// Returns `(char, advance_count, is_trailing_punctuation)`. `psText` is raw
/// wire bytes, not `&str`: the engine's advance count indexes those bytes, so
/// callers walk the slice by that count directly.
pub fn AnyLanguage_ReadCharFromString(engine: &Engine, psText: &[u8]) -> (c_uint, c_int, bool) {
    let mut text = psText.to_vec();
    text.push(0);
    let mut advance_count: c_int = 0;
    let mut trailing_punctuation: c_int = 0;
    let ch = <Engine as Execute<CgAnylanguageReadcharfromstring>>::execute(
        engine,
        CgAnylanguageReadcharfromstringArgs::new(
            text.as_ptr() as *const c_char,
            &mut advance_count,
            &mut trailing_punctuation,
        ),
    );
    (ch, advance_count, trailing_punctuation != 0)
}

/// Raven `trap_R_ClearScene` — `CG_R_CLEARSCENE` (token: `mp_abi::cgame::syscalls::CG_R_CLEARSCENE`).
///
/// C: `void trap_R_ClearScene( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:322-324`
pub fn R_ClearScene(engine: &Engine) {
    <Engine as Execute<CgRClearscene>>::execute(engine, CgRClearsceneArgs::new())
}

/// Raven `trap_R_ClearDecals` — `CG_R_CLEARDECALS` (token: `mp_abi::cgame::syscalls::CG_R_CLEARDECALS`).
///
/// C: `void trap_R_ClearDecals ( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:326-329`
pub fn R_ClearDecals(engine: &Engine) {
    <Engine as Execute<CgRCleardecals>>::execute(engine, ())
}

/// Raven `trap_R_AddRefEntityToScene` — `CG_R_ADDREFENTITYTOSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_ADDREFENTITYTOSCENE`).
///
/// C: `void trap_R_AddRefEntityToScene( const refEntity_t *re )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:331-333`
pub fn R_AddRefEntityToScene(engine: &Engine, re: &refEntity_t) {
    <Engine as Execute<CgRAddrefentitytoscene>>::execute(
        engine,
        CgRAddrefentitytosceneArgs::new(re as *const refEntity_t as *const c_void),
    )
}

/// Raven `trap_R_AddPolyToScene` — `CG_R_ADDPOLYTOSCENE` (token: `mp_abi::cgame::syscalls::CG_R_ADDPOLYTOSCENE`).
///
/// C: `void trap_R_AddPolyToScene( qhandle_t hShader , int numVerts, const polyVert_t *verts )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:335-337`
///
/// `numVerts` is dropped in favor of `verts.len()`.
pub fn R_AddPolyToScene(engine: &Engine, hShader: qhandle_t, verts: &[polyVert_t]) {
    <Engine as Execute<CgRAddpolytoscene>>::execute(
        engine,
        CgRAddpolytosceneArgs::new(
            hShader,
            verts.len() as c_int,
            verts.as_ptr() as *const c_void,
        ),
    )
}

/// Raven `trap_R_AddPolysToScene` — `CG_R_ADDPOLYSTOSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_ADDPOLYSTOSCENE`).
///
/// C: `void trap_R_AddPolysToScene( qhandle_t hShader , int numVerts, const polyVert_t *verts, int num )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:339-341`
///
/// `numVerts` is dropped in favor of `verts.len()`; Raven's separate `num`
/// (poly count) is kept.
pub fn R_AddPolysToScene(engine: &Engine, hShader: qhandle_t, verts: &[polyVert_t], num: c_int) {
    <Engine as Execute<CgRAddpolystoscene>>::execute(
        engine,
        CgRAddpolystosceneArgs::new(
            hShader,
            verts.len() as c_int,
            verts.as_ptr() as *const c_void,
            num,
        ),
    )
}

/// Raven `trap_R_AddDecalToScene` — `CG_R_ADDDECALTOSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_ADDDECALTOSCENE`).
///
/// C: `void trap_R_AddDecalToScene ( qhandle_t shader, const vec3_t origin, const vec3_t dir, float orientation, float r, float g, float b, float a, qboolean alphaFade, float radius, qboolean temporary )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:343-346`
#[allow(clippy::too_many_arguments)]
pub fn R_AddDecalToScene(
    engine: &Engine,
    shader: qhandle_t,
    origin: &vec3_t,
    dir: &vec3_t,
    orientation: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    alphaFade: bool,
    radius: f32,
    temporary: bool,
) {
    // SAFETY: `origin` and `dir` outlive the synchronous syscall the Args feed.
    let args = unsafe {
        CgRAdddecaltosceneArgs::new(
            shader,
            origin as *const vec3_t,
            dir as *const vec3_t,
            orientation,
            r,
            g,
            b,
            a,
            c_int::from(alphaFade),
            radius,
            c_int::from(temporary),
        )
    };
    <Engine as Execute<CgRAdddecaltoscene>>::execute(engine, args)
}

/// Raven `trap_R_LightForPoint` — `CG_R_LIGHTFORPOINT`
/// (token: `mp_abi::cgame::syscalls::CG_R_LIGHTFORPOINT`).
///
/// C: `int trap_R_LightForPoint( vec3_t point, vec3_t ambientLight, vec3_t directedLight, vec3_t lightDir )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:348-350`
pub fn R_LightForPoint(
    engine: &Engine,
    point: &vec3_t,
    ambientLight: &mut vec3_t,
    directedLight: &mut vec3_t,
    lightDir: &mut vec3_t,
) -> c_int {
    // SAFETY: `point` outlives the synchronous syscall the Args feed; the three
    // light buffers are writable for its duration.
    let args = unsafe {
        CgRLightforpointArgs::new(
            point as *const vec3_t,
            ambientLight as *mut vec3_t,
            directedLight as *mut vec3_t,
            lightDir as *mut vec3_t,
        )
    };
    <Engine as Execute<CgRLightforpoint>>::execute(engine, args)
}

/// Raven `trap_R_AddLightToScene` — `CG_R_ADDLIGHTTOSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_ADDLIGHTTOSCENE`).
///
/// C: `void trap_R_AddLightToScene( const vec3_t org, float intensity, float r, float g, float b )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:352-354`
pub fn R_AddLightToScene(engine: &Engine, org: &vec3_t, intensity: f32, r: f32, g: f32, b: f32) {
    <Engine as Execute<CgRAddlighttoscene>>::execute(
        engine,
        CgRAddlighttosceneArgs::new(org as *const vec3_t, intensity, r, g, b),
    )
}

/// Raven `trap_R_AddAdditiveLightToScene` — `CG_R_ADDADDITIVELIGHTTOSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_ADDADDITIVELIGHTTOSCENE`).
///
/// C: `void trap_R_AddAdditiveLightToScene( const vec3_t org, float intensity, float r, float g, float b )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:356-358`
pub fn R_AddAdditiveLightToScene(
    engine: &Engine,
    org: &vec3_t,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
) {
    <Engine as Execute<CgRAddadditivelighttoscene>>::execute(
        engine,
        CgRAddadditivelighttosceneArgs::new(org as *const vec3_t, intensity, r, g, b),
    )
}

/// Raven `trap_R_RenderScene` — `CG_R_RENDERSCENE`
/// (token: `mp_abi::cgame::syscalls::CG_R_RENDERSCENE`).
///
/// C: `void trap_R_RenderScene( const refdef_t *fd )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:360-362`
pub fn R_RenderScene(engine: &Engine, fd: &refdef_t) {
    <Engine as Execute<CgRRenderscene>>::execute(
        engine,
        CgRRendersceneArgs::new(fd as *const refdef_t as *const c_void),
    )
}

/// Raven `trap_R_SetColor` — `CG_R_SETCOLOR`
/// (token: `mp_abi::cgame::syscalls::CG_R_SETCOLOR`).
///
/// C: `void trap_R_SetColor( const float *rgba )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:364-366`
///
/// `None` is Raven's `NULL`, which resets the renderer to white.
pub fn R_SetColor(engine: &Engine, rgba: Option<&vec4_t>) {
    let rgba_ptr = rgba.map_or(null(), |c| c.as_ptr());
    // SAFETY: `rgba_ptr` is either null or points to four readable floats that
    // outlive the synchronous syscall the Args feed.
    let args = unsafe { CgRSetcolorArgs::new(rgba_ptr) };
    <Engine as Execute<CgRSetcolor>>::execute(engine, args)
}

/// Raven `trap_R_DrawStretchPic` — `CG_R_DRAWSTRETCHPIC`
/// (token: `mp_abi::cgame::syscalls::CG_R_DRAWSTRETCHPIC`).
///
/// C: `void trap_R_DrawStretchPic( float x, float y, float w, float h, float s1, float t1, float s2, float t2, qhandle_t hShader )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:368-371`
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
    <Engine as Execute<CgRDrawstretchpic>>::execute(
        engine,
        CgRDrawstretchpicArgs::new(x, y, w, h, s1, t1, s2, t2, hShader),
    )
}

/// Raven `trap_R_ModelBounds` — `CG_R_MODELBOUNDS`
/// (token: `mp_abi::cgame::syscalls::CG_R_MODELBOUNDS`).
///
/// C: `void trap_R_ModelBounds( clipHandle_t model, vec3_t mins, vec3_t maxs )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:373-375`
pub fn R_ModelBounds(engine: &Engine, model: clipHandle_t, mins: &mut vec3_t, maxs: &mut vec3_t) {
    <Engine as Execute<CgRModelbounds>>::execute(
        engine,
        CgRModelboundsArgs::new(model, mins as *mut vec3_t, maxs as *mut vec3_t),
    )
}

/// Raven `trap_R_LerpTag` — `CG_R_LERPTAG`
/// (token: `mp_abi::cgame::syscalls::CG_R_LERPTAG`).
///
/// C: `int trap_R_LerpTag( orientation_t *tag, clipHandle_t mod, int startFrame, int endFrame, float frac, const char *tagName )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:377-380`
///
/// Raven's `mod` parameter shadows the Rust keyword; spelled `r#mod` here.
#[allow(clippy::too_many_arguments)]
pub fn R_LerpTag(
    engine: &Engine,
    tag: &mut orientation_t,
    r#mod: clipHandle_t,
    startFrame: c_int,
    endFrame: c_int,
    frac: f32,
    tagName: &str,
) -> c_int {
    let tag_name_c = cstr(tagName);
    <Engine as Execute<CgRLerptag>>::execute(
        engine,
        CgRLerptagArgs::new(
            tag as *mut orientation_t as *mut c_void,
            r#mod,
            startFrame,
            endFrame,
            frac,
            tag_name_c.as_ptr(),
        ),
    )
}

/// Raven `trap_R_DrawRotatePic` — `CG_R_DRAWROTATEPIC`
/// (token: `mp_abi::cgame::syscalls::CG_R_DRAWROTATEPIC`).
///
/// C: `void trap_R_DrawRotatePic( float x, float y, float w, float h, float s1, float t1, float s2, float t2,float a, qhandle_t hShader )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:382-386`
#[allow(clippy::too_many_arguments)]
pub fn R_DrawRotatePic(
    engine: &Engine,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    hShader: qhandle_t,
) {
    <Engine as Execute<CgRDrawrotatepic>>::execute(
        engine,
        CgRDrawrotatepicArgs::new(x, y, w, h, s1, t1, s2, t2, a, hShader),
    )
}

/// Raven `trap_R_DrawRotatePic2` — `CG_R_DRAWROTATEPIC2`
/// (token: `mp_abi::cgame::syscalls::CG_R_DRAWROTATEPIC2`).
///
/// C: `void trap_R_DrawRotatePic2( float x, float y, float w, float h, float s1, float t1, float s2, float t2,float a, qhandle_t hShader )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:388-392`
#[allow(clippy::too_many_arguments)]
pub fn R_DrawRotatePic2(
    engine: &Engine,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    hShader: qhandle_t,
) {
    <Engine as Execute<CgRDrawrotatepic2>>::execute(
        engine,
        CgRDrawrotatepic2Args::new(x, y, w, h, s1, t1, s2, t2, a, hShader),
    )
}

/// Raven `trap_R_SetRangeFog` — `CG_R_SETRANGEFOG`
/// (token: `mp_abi::cgame::syscalls::CG_R_SETRANGEFOG`).
///
/// C: `void trap_R_SetRangeFog(float range)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:395-398`
pub fn R_SetRangeFog(engine: &Engine, range: f32) {
    <Engine as Execute<CgRSetrangefog>>::execute(engine, CgRSetrangefogArgs::new(range))
}

/// Raven `trap_R_SetRefractProp` — `CG_R_SETREFRACTIONPROP`
/// (token: `mp_abi::cgame::syscalls::CG_R_SETREFRACTIONPROP`).
///
/// C: `void trap_R_SetRefractProp(float alpha, float stretch, qboolean prepost, qboolean negate)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:401-404`
pub fn R_SetRefractProp(engine: &Engine, alpha: f32, stretch: f32, prepost: bool, negate: bool) {
    <Engine as Execute<CgRSetrefractionprop>>::execute(
        engine,
        CgRSetrefractionpropArgs::new(alpha, stretch, c_int::from(prepost), c_int::from(negate)),
    )
}

/// Raven `trap_R_RemapShader` — `CG_R_REMAP_SHADER`
/// (token: `mp_abi::cgame::syscalls::CG_R_REMAP_SHADER`).
///
/// C: `void trap_R_RemapShader( const char *oldShader, const char *newShader, const char *timeOffset )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:406-409`
pub fn R_RemapShader(engine: &Engine, oldShader: &str, newShader: &str, timeOffset: &str) {
    <Engine as Execute<CgRRemapShader>>::execute(
        engine,
        CgRRemapShaderArgs::new(cstr(oldShader), cstr(newShader), cstr(timeOffset)),
    )
}

/// Raven `trap_R_GetLightStyle` — `CG_R_GET_LIGHT_STYLE`
/// (token: `mp_abi::cgame::syscalls::CG_R_GET_LIGHT_STYLE`).
///
/// C: `void trap_R_GetLightStyle(int style, color4ub_t color)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:411-414`
pub fn R_GetLightStyle(engine: &Engine, style: c_int, color: &mut color4ub_t) {
    // SAFETY: `color` outlives the synchronous syscall the Args feed and is
    // writable for its duration.
    let args = unsafe { CgRGetLightStyleArgs::new(style, color as *mut color4ub_t) };
    <Engine as Execute<CgRGetLightStyle>>::execute(engine, args)
}

/// Raven `trap_R_SetLightStyle` — `CG_R_SET_LIGHT_STYLE`
/// (token: `mp_abi::cgame::syscalls::CG_R_SET_LIGHT_STYLE`).
///
/// C: `void trap_R_SetLightStyle(int style, int color)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:416-419`
pub fn R_SetLightStyle(engine: &Engine, style: c_int, color: c_int) {
    <Engine as Execute<CgRSetLightStyle>>::execute(engine, CgRSetLightStyleArgs::new(style, color))
}

/// Raven `trap_R_GetBModelVerts` — `CG_R_GET_BMODEL_VERTS`
/// (token: `mp_abi::cgame::syscalls::CG_R_GET_BMODEL_VERTS`).
///
/// C: `void trap_R_GetBModelVerts(int bmodelIndex, vec3_t *verts, vec3_t normal )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:421-424`
///
/// The abi's `normal` field is `*const vec3_t` — the engine reads the view
/// normal, it does not write it — so `normal` stays a caller `&vec3_t`.
pub fn R_GetBModelVerts(
    engine: &Engine,
    bmodelIndex: c_int,
    verts: &mut [vec3_t],
    normal: &vec3_t,
) {
    <Engine as Execute<CgRGetBmodelVerts>>::execute(
        engine,
        CgRGetBmodelVertsArgs::new(bmodelIndex, verts.as_mut_ptr(), normal as *const vec3_t),
    )
}

/// Raven `trap_R_GetDistanceCull` — `CG_R_GETDISTANCECULL`
/// (token: `mp_abi::cgame::syscalls::CG_R_GETDISTANCECULL`).
///
/// C: `void trap_R_GetDistanceCull(float *f)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:426-429`
pub fn R_GetDistanceCull(engine: &Engine) -> f32 {
    let mut f: f32 = 0.0;
    <Engine as Execute<CgRGetdistancecull>>::execute(
        engine,
        CgRGetdistancecullArgs::new(&mut f as *mut f32),
    );
    f
}

/// Raven `trap_R_GetRealRes` — `CG_R_GETREALRES`
/// (token: `mp_abi::cgame::syscalls::CG_R_GETREALRES`).
///
/// C: `void trap_R_GetRealRes(int *w, int *h)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:432-435`
pub fn R_GetRealRes(engine: &Engine) -> (c_int, c_int) {
    let mut w: c_int = 0;
    let mut h: c_int = 0;
    <Engine as Execute<CgRGetrealres>>::execute(
        engine,
        CgRGetrealresArgs::new(&mut w as *mut c_int, &mut h as *mut c_int),
    );
    (w, h)
}

/// Raven `trap_R_AutomapElevAdj` — `CG_R_AUTOMAPELEVADJ`
/// (token: `mp_abi::cgame::syscalls::CG_R_AUTOMAPELEVADJ`).
///
/// C: `void trap_R_AutomapElevAdj(float newHeight)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:439-442`
pub fn R_AutomapElevAdj(engine: &Engine, newHeight: f32) {
    <Engine as Execute<CgRAutomapelevadj>>::execute(engine, CgRAutomapelevadjArgs::new(newHeight))
}

/// Raven `trap_R_InitWireframeAutomap` — `CG_R_INITWIREFRAMEAUTO`
/// (token: `mp_abi::cgame::syscalls::CG_R_INITWIREFRAMEAUTO`).
///
/// C: `qboolean trap_R_InitWireframeAutomap(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:445-448`
pub fn R_InitWireframeAutomap(engine: &Engine) -> bool {
    <Engine as Execute<CgRInitwireframeauto>>::execute(engine, CgRInitwireframeautoArgs::new()) != 0
}

/// Raven `trap_FX_AddLine` — `CG_FX_ADDLINE`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDLINE`).
///
/// C: `void trap_FX_AddLine( const vec3_t start, const vec3_t end, float size1, float size2, float sizeParm, float alpha1, float alpha2, float alphaParm, const vec3_t sRGB, const vec3_t eRGB, float rgbParm, int killTime, qhandle_t shader, int flags)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:450-459`
#[allow(clippy::too_many_arguments)]
pub fn FX_AddLine(
    engine: &Engine,
    start: &vec3_t,
    end: &vec3_t,
    size1: f32,
    size2: f32,
    sizeParm: f32,
    alpha1: f32,
    alpha2: f32,
    alphaParm: f32,
    sRGB: &vec3_t,
    eRGB: &vec3_t,
    rgbParm: f32,
    killTime: c_int,
    shader: qhandle_t,
    flags: c_int,
) {
    <Engine as Execute<CgFxAddline>>::execute(
        engine,
        CgFxAddlineArgs::new(
            start as *const vec3_t,
            end as *const vec3_t,
            size1,
            size2,
            sizeParm,
            alpha1,
            alpha2,
            alphaParm,
            sRGB as *const vec3_t,
            eRGB as *const vec3_t,
            rgbParm,
            killTime,
            shader,
            flags,
        ),
    )
}

/// Raven `trap_GetGlconfig` — `CG_GETGLCONFIG`
/// (token: `mp_abi::cgame::syscalls::CG_GETGLCONFIG`).
///
/// C: `void trap_GetGlconfig( glconfig_t *glconfig )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:461-463`
pub fn GetGlconfig(engine: &Engine, glconfig: &mut glconfig_t) {
    <Engine as Execute<CgGetglconfig>>::execute(
        engine,
        CgGetglconfigArgs::new(glconfig as *mut glconfig_t as *mut c_void),
    )
}

/// Raven `trap_GetGameState` — `CG_GETGAMESTATE`
/// (token: `mp_abi::cgame::syscalls::CG_GETGAMESTATE`).
///
/// C: `void trap_GetGameState( gameState_t *gamestate )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:465-467`
pub fn GetGameState(engine: &Engine, gamestate: &mut gameState_t) {
    // SAFETY: `gamestate` outlives the synchronous syscall the Args feed and is
    // writable for its duration.
    let args = unsafe { CgGetgamestateArgs::new(gamestate as *mut gameState_t as *mut c_void) };
    <Engine as Execute<CgGetgamestate>>::execute(engine, args)
}

/// Raven `trap_GetCurrentSnapshotNumber` — `CG_GETCURRENTSNAPSHOTNUMBER`
/// (token: `mp_abi::cgame::syscalls::CG_GETCURRENTSNAPSHOTNUMBER`).
///
/// C: `void trap_GetCurrentSnapshotNumber( int *snapshotNumber, int *serverTime )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:469-471`
pub fn GetCurrentSnapshotNumber(engine: &Engine) -> (c_int, c_int) {
    let mut snapshotNumber: c_int = 0;
    let mut serverTime: c_int = 0;
    // SAFETY: both out-pointers outlive the synchronous syscall the Args feed.
    let args = unsafe {
        CgGetcurrentsnapshotnumberArgs::new(
            &mut snapshotNumber as *mut c_int,
            &mut serverTime as *mut c_int,
        )
    };
    <Engine as Execute<CgGetcurrentsnapshotnumber>>::execute(engine, args);
    (snapshotNumber, serverTime)
}

/// Raven `trap_GetSnapshot` — `CG_GETSNAPSHOT`
/// (token: `mp_abi::cgame::syscalls::CG_GETSNAPSHOT`).
///
/// C: `qboolean trap_GetSnapshot( int snapshotNumber, snapshot_t *snapshot )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:473-475`
pub fn GetSnapshot(engine: &Engine, snapshotNumber: c_int, snapshot: &mut snapshot_t) -> bool {
    // SAFETY: `snapshot` outlives the synchronous syscall the Args feed and is
    // writable for its duration.
    let args = unsafe {
        CgGetsnapshotArgs::new(snapshotNumber, snapshot as *mut snapshot_t as *mut c_void)
    };
    <Engine as Execute<CgGetsnapshot>>::execute(engine, args) != 0
}

/// Raven `trap_GetDefaultState` — `CG_GETDEFAULTSTATE`
/// (token: `mp_abi::cgame::syscalls::CG_GETDEFAULTSTATE`).
///
/// C: `qboolean trap_GetDefaultState(int entityIndex, entityState_t *state )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:477-480`
pub fn GetDefaultState(engine: &Engine, entityIndex: c_int, state: &mut entityState_t) -> bool {
    <Engine as Execute<CgGetdefaultstate>>::execute(
        engine,
        CgGetdefaultstateArgs::new(entityIndex, state as *mut entityState_t),
    ) != 0
}

/// Raven `trap_GetServerCommand` — `CG_GETSERVERCOMMAND`
/// (token: `mp_abi::cgame::syscalls::CG_GETSERVERCOMMAND`).
///
/// C: `qboolean trap_GetServerCommand( int serverCommandNumber )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:482-484`
pub fn GetServerCommand(engine: &Engine, serverCommandNumber: c_int) -> bool {
    <Engine as Execute<CgGetservercommand>>::execute(
        engine,
        CgGetservercommandArgs::new(serverCommandNumber),
    ) != 0
}

/// Raven `trap_GetCurrentCmdNumber` — `CG_GETCURRENTCMDNUMBER`
/// (token: `mp_abi::cgame::syscalls::CG_GETCURRENTCMDNUMBER`).
///
/// C: `int trap_GetCurrentCmdNumber( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:486-488`
pub fn GetCurrentCmdNumber(engine: &Engine) -> c_int {
    <Engine as Execute<CgGetcurrentcmdnumber>>::execute(engine, CgGetcurrentcmdnumberArgs::new())
}

/// Raven `trap_GetUserCmd` — `CG_GETUSERCMD`
/// (token: `mp_abi::cgame::syscalls::CG_GETUSERCMD`).
///
/// C: `qboolean trap_GetUserCmd( int cmdNumber, usercmd_t *ucmd )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:490-492`
pub fn GetUserCmd(engine: &Engine, cmdNumber: c_int, ucmd: &mut usercmd_t) -> bool {
    // SAFETY: `ucmd` outlives the synchronous syscall the Args feed and is
    // writable for its duration.
    let args = unsafe { CgGetusercmdArgs::new(cmdNumber, ucmd as *mut usercmd_t) };
    <Engine as Execute<CgGetusercmd>>::execute(engine, args) != 0
}

/// Raven `trap_SetUserCmdValue` — `CG_SETUSERCMDVALUE`
/// (token: `mp_abi::cgame::syscalls::CG_SETUSERCMDVALUE`).
///
/// C: `void trap_SetUserCmdValue( int stateValue, float sensitivityScale, float mPitchOverride, float mYawOverride, float mSensitivityOverride, int fpSel, int invenSel, qboolean fighterControls )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:494-496`
#[allow(clippy::too_many_arguments)]
pub fn SetUserCmdValue(
    engine: &Engine,
    stateValue: c_int,
    sensitivityScale: f32,
    mPitchOverride: f32,
    mYawOverride: f32,
    mSensitivityOverride: f32,
    fpSel: c_int,
    invenSel: c_int,
    fighterControls: bool,
) {
    <Engine as Execute<CgSetusercmdvalue>>::execute(
        engine,
        CgSetusercmdvalueArgs::new(
            stateValue,
            sensitivityScale,
            mPitchOverride,
            mYawOverride,
            mSensitivityOverride,
            fpSel,
            invenSel,
            c_int::from(fighterControls),
        ),
    )
}

/// Raven `trap_SetClientForceAngle` — `CG_SETCLIENTFORCEANGLE`
/// (token: `mp_abi::cgame::syscalls::CG_SETCLIENTFORCEANGLE`).
///
/// C: `void trap_SetClientForceAngle(int time, vec3_t angle)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:498-501`
///
/// Raven's `vec3_t angle` is non-`const` but read-only at the seam, so it
/// crosses as `&vec3_t` and is cast to the abi's writable pointer shape.
pub fn SetClientForceAngle(engine: &Engine, time: c_int, angle: &vec3_t) {
    <Engine as Execute<CgSetclientforceangle>>::execute(
        engine,
        CgSetclientforceangleArgs::new(time, angle as *const vec3_t as *mut vec3_t),
    )
}

/// Raven `trap_SetClientTurnExtent` — `CG_SETCLIENTTURNEXTENT`
/// (token: `mp_abi::cgame::syscalls::CG_SETCLIENTTURNEXTENT`).
///
/// C: `void trap_SetClientTurnExtent(float turnAdd, float turnSub, int turnTime)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:503-506`
pub fn SetClientTurnExtent(engine: &Engine, turnAdd: f32, turnSub: f32, turnTime: c_int) {
    <Engine as Execute<CgSetclientturnextent>>::execute(
        engine,
        CgSetclientturnextentArgs::new(turnAdd, turnSub, turnTime),
    )
}

/// Raven `trap_OpenUIMenu` — `CG_OPENUIMENU`
/// (token: `mp_abi::cgame::syscalls::CG_OPENUIMENU`).
///
/// C: `void trap_OpenUIMenu(int menuID)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:508-511`
pub fn OpenUIMenu(engine: &Engine, menuID: c_int) {
    <Engine as Execute<CgOpenuimenu>>::execute(engine, CgOpenuimenuArgs::new(menuID))
}

/// Raven `trap_MemoryRemaining` — `CG_MEMORY_REMAINING`
/// (token: `mp_abi::cgame::syscalls::CG_MEMORY_REMAINING`).
///
/// C: `int trap_MemoryRemaining( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:521-523`
pub fn MemoryRemaining(engine: &Engine) -> c_int {
    <Engine as Execute<CgMemoryRemaining>>::execute(engine, CgMemoryRemainingArgs::new())
}

/// Raven `trap_Key_IsDown` — `CG_KEY_ISDOWN`
/// (token: `mp_abi::cgame::syscalls::CG_KEY_ISDOWN`).
///
/// C: `qboolean trap_Key_IsDown( int keynum )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:525-527`
pub fn Key_IsDown(engine: &Engine, keynum: c_int) -> bool {
    <Engine as Execute<CgKeyIsdown>>::execute(engine, CgKeyIsdownArgs::new(keynum)) != 0
}

/// Raven `trap_Key_GetCatcher` — `CG_KEY_GETCATCHER`
/// (token: `mp_abi::cgame::syscalls::CG_KEY_GETCATCHER`).
///
/// C: `int trap_Key_GetCatcher( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:529-531`
pub fn Key_GetCatcher(engine: &Engine) -> c_int {
    <Engine as Execute<CgKeyGetcatcher>>::execute(engine, CgKeyGetcatcherArgs::new())
}

/// Raven `trap_Key_SetCatcher` — `CG_KEY_SETCATCHER`
/// (token: `mp_abi::cgame::syscalls::CG_KEY_SETCATCHER`).
///
/// C: `void trap_Key_SetCatcher( int catcher )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:533-535`
pub fn Key_SetCatcher(engine: &Engine, catcher: c_int) {
    <Engine as Execute<CgKeySetcatcher>>::execute(engine, CgKeySetcatcherArgs::new(catcher))
}

/// Raven `trap_Key_GetKey` — `CG_KEY_GETKEY`
/// (token: `mp_abi::cgame::syscalls::CG_KEY_GETKEY`).
///
/// C: `int trap_Key_GetKey( const char *binding )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:537-539`
pub fn Key_GetKey(engine: &Engine, binding: &str) -> c_int {
    let binding_c = cstr(binding);
    // SAFETY: `binding_c` outlives the synchronous syscall the Args feed.
    let args = unsafe { CgKeyGetkeyArgs::new(binding_c.as_ptr()) };
    <Engine as Execute<CgKeyGetkey>>::execute(engine, args)
}

/// Raven `trap_PC_AddGlobalDefine` — `CG_PC_ADD_GLOBAL_DEFINE`
/// (token: `mp_abi::cgame::syscalls::CG_PC_ADD_GLOBAL_DEFINE`).
///
/// C: `int trap_PC_AddGlobalDefine( char *define )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:541-543`
pub fn PC_AddGlobalDefine(engine: &Engine, define: &str) -> c_int {
    let define_c = cstr(define);
    <Engine as Execute<CgPcAddGlobalDefine>>::execute(
        engine,
        CgPcAddGlobalDefineArgs::new(define_c.as_ptr() as *mut c_char),
    )
}

/// Raven `trap_PC_LoadSource` — `CG_PC_LOAD_SOURCE`
/// (token: `mp_abi::cgame::syscalls::CG_PC_LOAD_SOURCE`).
///
/// C: `int trap_PC_LoadSource( const char *filename )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:545-547`
pub fn PC_LoadSource(engine: &Engine, filename: &str) -> c_int {
    let filename_c = cstr(filename);
    <Engine as Execute<CgPcLoadSource>>::execute(
        engine,
        CgPcLoadSourceArgs::new(filename_c.as_ptr()),
    )
}

/// Raven `trap_PC_FreeSource` — `CG_PC_FREE_SOURCE`
/// (token: `mp_abi::cgame::syscalls::CG_PC_FREE_SOURCE`).
///
/// C: `int trap_PC_FreeSource( int handle )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:549-551`
pub fn PC_FreeSource(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<CgPcFreeSource>>::execute(engine, CgPcFreeSourceArgs::new(handle))
}

/// Raven `trap_PC_ReadToken` — `CG_PC_READ_TOKEN`
/// (token: `mp_abi::cgame::syscalls::CG_PC_READ_TOKEN`).
///
/// C: `int trap_PC_ReadToken( int handle, pc_token_t *pc_token )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:553-555`
///
/// The token block is engine-filled in place: it keeps its `#[repr(C)]` shape
/// and stays a caller `&mut` (ui `PC_ReadToken` precedent).
pub fn PC_ReadToken(engine: &Engine, handle: c_int, pc_token: &mut pc_token_t) -> bool {
    <Engine as Execute<CgPcReadToken>>::execute(
        engine,
        CgPcReadTokenArgs::new(handle, pc_token as *mut pc_token_t),
    ) != 0
}

/// Raven `trap_PC_SourceFileAndLine` — `CG_PC_SOURCE_FILE_AND_LINE`
/// (token: `mp_abi::cgame::syscalls::CG_PC_SOURCE_FILE_AND_LINE`).
///
/// C: `int trap_PC_SourceFileAndLine( int handle, char *filename, int *line )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:557-559`
///
/// Returns `(status, filename, line)`. Raven passes an unbounded caller array
/// for the name; `buffer_len` names that width at the call site (ui
/// `PC_SourceFileAndLine` precedent).
pub fn PC_SourceFileAndLine(
    engine: &Engine,
    handle: c_int,
    buffer_len: usize,
) -> (c_int, String, c_int) {
    let mut buffer = vec![0u8; buffer_len];
    let mut line: c_int = 0;
    let status = <Engine as Execute<CgPcSourceFileAndLine>>::execute(
        engine,
        CgPcSourceFileAndLineArgs::new(handle, buffer.as_mut_ptr() as *mut c_char, &mut line),
    );
    (status, buf_to_string(&buffer), line)
}

/// Raven `trap_PC_LoadGlobalDefines` — `CG_PC_LOAD_GLOBAL_DEFINES`
/// (token: `mp_abi::cgame::syscalls::CG_PC_LOAD_GLOBAL_DEFINES`).
///
/// C: `int trap_PC_LoadGlobalDefines( const char* filename )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:561-564`
pub fn PC_LoadGlobalDefines(engine: &Engine, filename: &str) -> c_int {
    let filename_c = cstr(filename);
    <Engine as Execute<CgPcLoadGlobalDefines>>::execute(
        engine,
        CgPcLoadGlobalDefinesArgs::new(filename_c.as_ptr()),
    )
}

/// Raven `trap_PC_RemoveAllGlobalDefines` — `CG_PC_REMOVE_ALL_GLOBAL_DEFINES`
/// (token: `mp_abi::cgame::syscalls::CG_PC_REMOVE_ALL_GLOBAL_DEFINES`).
///
/// C: `void trap_PC_RemoveAllGlobalDefines( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:566-569`
pub fn PC_RemoveAllGlobalDefines(engine: &Engine) {
    <Engine as Execute<CgPcRemoveAllGlobalDefines>>::execute(
        engine,
        CgPcRemoveAllGlobalDefinesArgs::new(),
    )
}

/// Raven `trap_S_StopBackgroundTrack` — `CG_S_STOPBACKGROUNDTRACK`
/// (token: `mp_abi::cgame::syscalls::CG_S_STOPBACKGROUNDTRACK`).
///
/// C: `void trap_S_StopBackgroundTrack( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:571-573`
pub fn S_StopBackgroundTrack(engine: &Engine) {
    <Engine as Execute<CgSStopbackgroundtrack>>::execute(engine, CgSStopbackgroundtrackArgs::new())
}

/// Raven `trap_RealTime` — `CG_REAL_TIME`
/// (token: `mp_abi::cgame::syscalls::CG_REAL_TIME`).
///
/// C: `int trap_RealTime(qtime_t *qtime)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:575-577`
pub fn RealTime(engine: &Engine, qtime: &mut qtime_t) -> c_int {
    <Engine as Execute<CgRealTime>>::execute(engine, CgRealTimeArgs::new(qtime as *mut qtime_t))
}

/// Raven `trap_SnapVector` — `CG_SNAPVECTOR`
/// (token: `mp_abi::cgame::syscalls::CG_SNAPVECTOR`).
///
/// C: `void trap_SnapVector( float *v )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:579-581`
///
/// Read-modify-write: the engine snaps `v` in place.
pub fn SnapVector(engine: &Engine, v: &mut vec3_t) {
    <Engine as Execute<CgSnapvector>>::execute(engine, CgSnapvectorArgs::new(v as *mut vec3_t))
}

/// Raven `trap_CIN_PlayCinematic` — `CG_CIN_PLAYCINEMATIC`
/// (token: `mp_abi::cgame::syscalls::CG_CIN_PLAYCINEMATIC`).
///
/// C: `int trap_CIN_PlayCinematic( const char *arg0, int xpos, int ypos, int width, int height, int bits)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:584-586`
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
    <Engine as Execute<CgCinPlaycinematic>>::execute(
        engine,
        CgCinPlaycinematicArgs::new(arg0_c.as_ptr(), xpos, ypos, width, height, bits),
    )
}

/// Raven `trap_CIN_StopCinematic` — `CG_CIN_STOPCINEMATIC`
/// (token: `mp_abi::cgame::syscalls::CG_CIN_STOPCINEMATIC`).
///
/// C: `e_status trap_CIN_StopCinematic(int handle)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:590-592`
pub fn CIN_StopCinematic(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<CgCinStopcinematic>>::execute(engine, CgCinStopcinematicArgs::new(handle))
}

/// Raven `trap_CIN_RunCinematic` — `CG_CIN_RUNCINEMATIC`
/// (token: `mp_abi::cgame::syscalls::CG_CIN_RUNCINEMATIC`).
///
/// C: `e_status trap_CIN_RunCinematic (int handle)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:596-598`
pub fn CIN_RunCinematic(engine: &Engine, handle: c_int) -> c_int {
    <Engine as Execute<CgCinRuncinematic>>::execute(engine, CgCinRuncinematicArgs::new(handle))
}

/// Raven `trap_CIN_DrawCinematic` — `CG_CIN_DRAWCINEMATIC`
/// (token: `mp_abi::cgame::syscalls::CG_CIN_DRAWCINEMATIC`).
///
/// C: `void trap_CIN_DrawCinematic (int handle)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:602-604`
pub fn CIN_DrawCinematic(engine: &Engine, handle: c_int) {
    <Engine as Execute<CgCinDrawcinematic>>::execute(engine, CgCinDrawcinematicArgs::new(handle))
}

/// Raven `trap_CIN_SetExtents` — `CG_CIN_SETEXTENTS`
/// (token: `mp_abi::cgame::syscalls::CG_CIN_SETEXTENTS`).
///
/// C: `void trap_CIN_SetExtents (int handle, int x, int y, int w, int h)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:608-610`
pub fn CIN_SetExtents(engine: &Engine, handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    <Engine as Execute<CgCinSetextents>>::execute(
        engine,
        CgCinSetextentsArgs::new(handle, x, y, w, h),
    )
}

/// Raven `trap_GetEntityToken` — `CG_GET_ENTITY_TOKEN`
/// (token: `mp_abi::cgame::syscalls::CG_GET_ENTITY_TOKEN`).
///
/// C: `qboolean trap_GetEntityToken( char *buffer, int bufferSize )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:612-614`
///
/// `qfalse` (no more tokens) -> `None` (`mp_game::trap::GetEntityToken` precedent).
pub fn GetEntityToken(engine: &Engine, buffer_len: usize) -> Option<String> {
    let mut buffer = vec![0u8; buffer_len];
    let more = <Engine as Execute<CgGetEntityToken>>::execute(
        engine,
        CgGetEntityTokenArgs::new(buffer.as_mut_ptr() as *mut c_char, buffer_len as c_int),
    );
    (more != 0).then(|| buf_to_string(&buffer))
}

/// Raven `trap_R_inPVS` — `CG_R_INPVS`
/// (token: `mp_abi::cgame::syscalls::CG_R_INPVS`).
///
/// C: `qboolean trap_R_inPVS( const vec3_t p1, const vec3_t p2, byte *mask )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:616-618`
pub fn R_inPVS(engine: &Engine, p1: &vec3_t, p2: &vec3_t, mask: &mut [u8]) -> bool {
    <Engine as Execute<CgRInpvs>>::execute(
        engine,
        CgRInpvsArgs::new(p1 as *const vec3_t, p2 as *const vec3_t, mask.as_mut_ptr()),
    ) != 0
}

/// Raven `trap_FX_RegisterEffect` — `CG_FX_REGISTER_EFFECT`
/// (token: `mp_abi::cgame::syscalls::CG_FX_REGISTER_EFFECT`).
///
/// C: `int trap_FX_RegisterEffect(const char *file)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:621-624`
pub fn FX_RegisterEffect(engine: &Engine, file: &str) -> c_int {
    let file_c = cstr(file);
    <Engine as Execute<CgFxRegisterEffect>>::execute(
        engine,
        CgFxRegisterEffectArgs::new(file_c.as_ptr()),
    )
}

/// Raven `trap_FX_PlayEffect` — `CG_FX_PLAY_EFFECT`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_EFFECT`).
///
/// C: `void trap_FX_PlayEffect( const char *file, vec3_t org, vec3_t fwd, int vol, int rad )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:626-629`
pub fn FX_PlayEffect(
    engine: &Engine,
    file: &str,
    org: &vec3_t,
    fwd: &vec3_t,
    vol: c_int,
    rad: c_int,
) {
    let file_c = cstr(file);
    <Engine as Execute<CgFxPlayEffect>>::execute(
        engine,
        CgFxPlayEffectArgs::new(
            file_c.as_ptr(),
            org as *const vec3_t,
            fwd as *const vec3_t,
            vol,
            rad,
        ),
    )
}

/// Raven `trap_FX_PlayEntityEffect` — `CG_FX_PLAY_ENTITY_EFFECT`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_ENTITY_EFFECT`).
///
/// C: `void trap_FX_PlayEntityEffect( const char *file, vec3_t org, vec3_t axis[3], const int boltInfo, const int entNum, int vol, int rad )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:631-635`
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayEntityEffect(
    engine: &Engine,
    file: &str,
    org: &vec3_t,
    axis: &[vec3_t; 3],
    boltInfo: c_int,
    entNum: c_int,
    vol: c_int,
    rad: c_int,
) {
    let file_c = cstr(file);
    <Engine as Execute<CgFxPlayEntityEffect>>::execute(
        engine,
        CgFxPlayEntityEffectArgs::new(
            file_c.as_ptr(),
            org as *const vec3_t,
            axis.as_ptr(),
            boltInfo,
            entNum,
            vol,
            rad,
        ),
    )
}

/// Raven `trap_FX_PlayEffectID` — `CG_FX_PLAY_EFFECT_ID`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_EFFECT_ID`).
///
/// C: `void trap_FX_PlayEffectID( int id, vec3_t org, vec3_t fwd, int vol, int rad )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:637-640`
pub fn FX_PlayEffectID(
    engine: &Engine,
    id: c_int,
    org: &vec3_t,
    fwd: &vec3_t,
    vol: c_int,
    rad: c_int,
) {
    <Engine as Execute<CgFxPlayEffectId>>::execute(
        engine,
        CgFxPlayEffectIdArgs::new(id, org as *const vec3_t, fwd as *const vec3_t, vol, rad),
    )
}

/// Raven `trap_FX_PlayPortalEffectID` — `CG_FX_PLAY_PORTAL_EFFECT_ID`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_PORTAL_EFFECT_ID`).
///
/// C: `void trap_FX_PlayPortalEffectID( int id, vec3_t org, vec3_t fwd, int vol, int rad )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:642-645`
///
/// Raven's wrapper takes `vol`/`rad` and then forwards only `id`/`org`/`fwd`;
/// the client switch still reads `args[4]`/`args[5]`. The declared parameters
/// are kept so call sites transcribe verbatim, and dropped at the syscall
/// exactly as Raven drops them.
pub fn FX_PlayPortalEffectID(
    engine: &Engine,
    id: c_int,
    org: &vec3_t,
    fwd: &vec3_t,
    _vol: c_int,
    _rad: c_int,
) {
    <Engine as Execute<CgFxPlayPortalEffectId>>::execute(
        engine,
        CgFxPlayPortalEffectIdArgs::new(id, org as *const vec3_t, fwd as *const vec3_t),
    )
}

/// Raven `trap_FX_PlayEntityEffectID` — `CG_FX_PLAY_ENTITY_EFFECT_ID`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_ENTITY_EFFECT_ID`).
///
/// C: `void trap_FX_PlayEntityEffectID( int id, vec3_t org, vec3_t axis[3], const int boltInfo, const int entNum, int vol, int rad )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:647-651`
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayEntityEffectID(
    engine: &Engine,
    id: c_int,
    org: &vec3_t,
    axis: &[vec3_t; 3],
    boltInfo: c_int,
    entNum: c_int,
    vol: c_int,
    rad: c_int,
) {
    <Engine as Execute<CgFxPlayEntityEffectId>>::execute(
        engine,
        CgFxPlayEntityEffectIdArgs::new(
            id,
            org as *const vec3_t,
            axis.as_ptr(),
            boltInfo,
            entNum,
            vol,
            rad,
        ),
    )
}

/// Raven `trap_FX_PlayBoltedEffectID` — `CG_FX_PLAY_BOLTED_EFFECT_ID`
/// (token: `mp_abi::cgame::syscalls::CG_FX_PLAY_BOLTED_EFFECT_ID`).
///
/// C: `void trap_FX_PlayBoltedEffectID( int id, vec3_t org, void *ghoul2, const int boltNum, const int entNum, const int modelNum, int iLooptime, qboolean isRelative )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:653-657`
///
/// `ghoul2` stays an opaque engine token — never dereferenced.
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayBoltedEffectID(
    engine: &Engine,
    id: c_int,
    org: &vec3_t,
    ghoul2: *mut c_void,
    boltNum: c_int,
    entNum: c_int,
    modelNum: c_int,
    i_loop_time: c_int,
    isRelative: bool,
) {
    <Engine as Execute<CgFxPlayBoltedEffectId>>::execute(
        engine,
        CgFxPlayBoltedEffectIdArgs::new(
            id,
            org as *const vec3_t,
            ghoul2,
            boltNum,
            entNum,
            modelNum,
            i_loop_time,
            c_int::from(isRelative),
        ),
    )
}

/// Raven `trap_FX_AddScheduledEffects` — `CG_FX_ADD_SCHEDULED_EFFECTS`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADD_SCHEDULED_EFFECTS`).
///
/// C: `void trap_FX_AddScheduledEffects( qboolean skyPortal )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:659-662`
pub fn FX_AddScheduledEffects(engine: &Engine, skyPortal: bool) {
    <Engine as Execute<CgFxAddScheduledEffects>>::execute(
        engine,
        CgFxAddScheduledEffectsArgs::new(c_int::from(skyPortal)),
    )
}

/// Raven `trap_FX_Draw2DEffects` — `CG_FX_DRAW_2D_EFFECTS`
/// (token: `mp_abi::cgame::syscalls::CG_FX_DRAW_2D_EFFECTS`).
///
/// C: `void trap_FX_Draw2DEffects ( float screenXScale, float screenYScale )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:664-667`
pub fn FX_Draw2DEffects(engine: &Engine, screenXScale: f32, screenYScale: f32) {
    <Engine as Execute<CgFxDraw2dEffects>>::execute(
        engine,
        CgFxDraw2dEffectsArgs::new(screenXScale, screenYScale),
    )
}

/// Raven `trap_FX_InitSystem` — `CG_FX_INIT_SYSTEM`
/// (token: `mp_abi::cgame::syscalls::CG_FX_INIT_SYSTEM`).
///
/// C: `int trap_FX_InitSystem( refdef_t* refdef )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:669-672`
pub fn FX_InitSystem(engine: &Engine, refdef: &mut refdef_t) -> c_int {
    <Engine as Execute<CgFxInitSystem>>::execute(
        engine,
        CgFxInitSystemArgs::new(refdef as *mut refdef_t as *mut c_void),
    )
}

/// Raven `trap_FX_SetRefDef` — `CG_FX_SET_REFDEF`
/// (token: `mp_abi::cgame::syscalls::CG_FX_SET_REFDEF`).
///
/// C: `void trap_FX_SetRefDef( refdef_t* refdef )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:674-677`
pub fn FX_SetRefDef(engine: &Engine, refdef: &mut refdef_t) {
    <Engine as Execute<CgFxSetRefdef>>::execute(
        engine,
        CgFxSetRefdefArgs::new(refdef as *mut refdef_t as *mut c_void),
    )
}

/// Raven `trap_FX_FreeSystem` — `CG_FX_FREE_SYSTEM`
/// (token: `mp_abi::cgame::syscalls::CG_FX_FREE_SYSTEM`).
///
/// C: `qboolean trap_FX_FreeSystem( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:679-682`
pub fn FX_FreeSystem(engine: &Engine) -> bool {
    <Engine as Execute<CgFxFreeSystem>>::execute(engine, CgFxFreeSystemArgs::new()) != 0
}

/// Raven `trap_FX_Reset` — `CG_FX_RESET`
/// (token: `mp_abi::cgame::syscalls::CG_FX_RESET`).
///
/// C: `void trap_FX_Reset ( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:684-687`
pub fn FX_Reset(engine: &Engine) {
    <Engine as Execute<CgFxReset>>::execute(engine, CgFxResetArgs::new())
}

/// Raven `trap_FX_AdjustTime` — `CG_FX_ADJUST_TIME`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADJUST_TIME`).
///
/// C: `void trap_FX_AdjustTime( int time )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:689-692`
pub fn FX_AdjustTime(engine: &Engine, time: c_int) {
    <Engine as Execute<CgFxAdjustTime>>::execute(engine, CgFxAdjustTimeArgs::new(time))
}

/// Raven `trap_FX_AddPoly` — `CG_FX_ADDPOLY`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDPOLY`).
///
/// C: `void trap_FX_AddPoly( addpolyArgStruct_t *p )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:695-698`
pub fn FX_AddPoly(engine: &Engine, p: &mut addpolyArgStruct_t) {
    <Engine as Execute<CgFxAddpoly>>::execute(
        engine,
        CgFxAddpolyArgs::new(p as *mut addpolyArgStruct_t as *mut c_void),
    )
}

/// Raven `trap_FX_AddBezier` — `CG_FX_ADDBEZIER`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDBEZIER`).
///
/// C: `void trap_FX_AddBezier( addbezierArgStruct_t *p )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:700-703`
pub fn FX_AddBezier(engine: &Engine, p: &mut addbezierArgStruct_t) {
    <Engine as Execute<CgFxAddbezier>>::execute(
        engine,
        CgFxAddbezierArgs::new(p as *mut addbezierArgStruct_t as *mut c_void),
    )
}

/// Raven `trap_FX_AddPrimitive` — `CG_FX_ADDPRIMITIVE`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDPRIMITIVE`).
///
/// C: `void trap_FX_AddPrimitive( effectTrailArgStruct_t *p )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:705-708`
pub fn FX_AddPrimitive(engine: &Engine, p: &mut effectTrailArgStruct_t) {
    <Engine as Execute<CgFxAddprimitive>>::execute(
        engine,
        CgFxAddprimitiveArgs::new(p as *mut effectTrailArgStruct_t as *mut c_void),
    )
}

/// Raven `trap_FX_AddSprite` — `CG_FX_ADDSPRITE`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDSPRITE`).
///
/// C: `void trap_FX_AddSprite( addspriteArgStruct_t *p )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:710-713`
pub fn FX_AddSprite(engine: &Engine, p: &mut addspriteArgStruct_t) {
    <Engine as Execute<CgFxAddsprite>>::execute(
        engine,
        CgFxAddspriteArgs::new(p as *mut addspriteArgStruct_t as *mut c_void),
    )
}

/// Raven `trap_FX_AddElectricity` — `CG_FX_ADDELECTRICITY`
/// (token: `mp_abi::cgame::syscalls::CG_FX_ADDELECTRICITY`).
///
/// C: `void trap_FX_AddElectricity( addElectricityArgStruct_t *p )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:715-718`
pub fn FX_AddElectricity(engine: &Engine, p: &mut addElectricityArgStruct_t) {
    <Engine as Execute<CgFxAddelectricity>>::execute(
        engine,
        CgFxAddelectricityArgs::new(p as *mut addElectricityArgStruct_t as *mut c_void),
    )
}

/// Raven `trap_SP_GetStringTextString` — `CG_SP_GETSTRINGTEXTSTRING`
/// (token: `mp_abi::cgame::syscalls::CG_SP_GETSTRINGTEXTSTRING`).
///
/// C: `int trap_SP_GetStringTextString(const char *text, char *buffer, int bufferLength)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:725-728`
///
/// The engine fills the buffer either way, so the buffer is the whole answer
/// and the `qboolean` carries nothing extra: on a miss it writes the marker
/// `"??<key>"` and returns `qfalse`, which is exactly what Raven's callers go
/// on to print (`oracle/codemp/client/cl_cgame.cpp:1668-1680`).
pub fn SP_GetStringTextString(engine: &Engine, text: &str, buffer_len: usize) -> String {
    let text_c = cstr(text);
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<CgSpGetstringtextstring>>::execute(
        engine,
        CgSpGetstringtextstringArgs::new(
            text_c.as_ptr(),
            buffer.as_mut_ptr() as *mut c_char,
            buffer_len as c_int,
        ),
    );
    let nul = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
    latin1_to_string(&buffer[..nul])
}

/// Raven `trap_ROFF_Clean` — `CG_ROFF_CLEAN` (token: `mp_abi::cgame::syscalls::CG_ROFF_CLEAN`).
///
/// C: `qboolean trap_ROFF_Clean( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:730-733`
pub fn ROFF_Clean(engine: &Engine) -> bool {
    <Engine as Execute<CgRoffClean>>::execute(engine, CgRoffCleanArgs::new()) != 0
}

/// Raven `trap_ROFF_UpdateEntities` — `CG_ROFF_UPDATE_ENTITIES`
/// (token: `mp_abi::cgame::syscalls::CG_ROFF_UPDATE_ENTITIES`).
///
/// C: `void trap_ROFF_UpdateEntities( void )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:735-738`
pub fn ROFF_UpdateEntities(engine: &Engine) {
    <Engine as Execute<CgRoffUpdateEntities>>::execute(engine, CgRoffUpdateEntitiesArgs::new())
}

/// Raven `trap_ROFF_Cache` — `CG_ROFF_CACHE` (token: `mp_abi::cgame::syscalls::CG_ROFF_CACHE`).
///
/// C: `int trap_ROFF_Cache( char *file )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:740-743`
pub fn ROFF_Cache(engine: &Engine, file: &str) -> c_int {
    let file_c = cstr(file);
    <Engine as Execute<CgRoffCache>>::execute(
        engine,
        CgRoffCacheArgs::new(file_c.as_ptr() as *mut c_char),
    )
}

/// Raven `trap_ROFF_Play` — `CG_ROFF_PLAY` (token: `mp_abi::cgame::syscalls::CG_ROFF_PLAY`).
///
/// C: `qboolean trap_ROFF_Play( int entID, int roffID, qboolean doTranslation )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:745-748`
pub fn ROFF_Play(engine: &Engine, entID: c_int, roffID: c_int, doTranslation: bool) -> bool {
    <Engine as Execute<CgRoffPlay>>::execute(
        engine,
        CgRoffPlayArgs::new(entID, roffID, c_int::from(doTranslation)),
    ) != 0
}

/// Raven `trap_ROFF_Purge_Ent` — `CG_ROFF_PURGE_ENT`
/// (token: `mp_abi::cgame::syscalls::CG_ROFF_PURGE_ENT`).
///
/// C: `qboolean trap_ROFF_Purge_Ent( int entID )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:750-753`
pub fn ROFF_Purge_Ent(engine: &Engine, entID: c_int) -> bool {
    <Engine as Execute<CgRoffPurgeEnt>>::execute(engine, CgRoffPurgeEntArgs::new(entID)) != 0
}

/// Raven `trap_TrueMalloc` — `CG_TRUEMALLOC` (token: `mp_abi::cgame::syscalls::CG_TRUEMALLOC`).
///
/// C: `void trap_TrueMalloc(void **ptr, int size)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:757-760`
///
/// The engine allocates into the caller's slot, so the pointer-to-slot stays
/// a raw slot pointer at this seam (allocator seam, not a typed value).
pub fn TrueMalloc(engine: &Engine, ptr: *mut *mut c_void, size: c_int) {
    <Engine as Execute<CgTruemalloc>>::execute(engine, CgTruemallocArgs::new(ptr, size))
}

/// Raven `trap_TrueFree` — `CG_TRUEFREE` (token: `mp_abi::cgame::syscalls::CG_TRUEFREE`).
///
/// C: `void trap_TrueFree(void **ptr)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:762-765`
///
/// The engine clears the caller's slot, so the pointer-to-slot stays a raw
/// slot pointer at this seam (allocator seam, not a typed value).
pub fn TrueFree(engine: &Engine, ptr: *mut *mut c_void) {
    <Engine as Execute<CgTruefree>>::execute(engine, CgTruefreeArgs::new(ptr))
}

/// Raven `trap_G2_ListModelSurfaces` — `CG_G2_LISTSURFACES`
/// (token: `mp_abi::cgame::syscalls::CG_G2_LISTSURFACES`).
///
/// C: `void trap_G2_ListModelSurfaces(void *ghlInfo)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:771-774`
pub fn G2_ListModelSurfaces(engine: &Engine, ghlInfo: *mut c_void) {
    <Engine as Execute<CgG2Listsurfaces>>::execute(engine, CgG2ListsurfacesArgs::new(ghlInfo))
}

/// Raven `trap_G2_ListModelBones` — `CG_G2_LISTBONES`
/// (token: `mp_abi::cgame::syscalls::CG_G2_LISTBONES`).
///
/// C: `void trap_G2_ListModelBones(void *ghlInfo, int frame)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:776-779`
pub fn G2_ListModelBones(engine: &Engine, ghlInfo: *mut c_void, frame: c_int) {
    <Engine as Execute<CgG2Listbones>>::execute(engine, CgG2ListbonesArgs::new(ghlInfo, frame))
}

/// Raven `trap_G2_SetGhoul2ModelIndexes` — `CG_G2_SETMODELS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SETMODELS`).
///
/// C: `void trap_G2_SetGhoul2ModelIndexes(void *ghoul2, qhandle_t *modelList, qhandle_t *skinList)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:781-784`
pub fn G2_SetGhoul2ModelIndexes(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelList: &mut [qhandle_t],
    skinList: &mut [qhandle_t],
) {
    <Engine as Execute<CgG2Setmodels>>::execute(
        engine,
        CgG2SetmodelsArgs::new(ghoul2, modelList.as_mut_ptr(), skinList.as_mut_ptr()),
    )
}

/// Raven `trap_G2_HaveWeGhoul2Models` — `CG_G2_HAVEWEGHOULMODELS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_HAVEWEGHOULMODELS`).
///
/// C: `qboolean trap_G2_HaveWeGhoul2Models(void *ghoul2)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:786-789`
pub fn G2_HaveWeGhoul2Models(engine: &Engine, ghoul2: *mut c_void) -> bool {
    <Engine as Execute<CgG2Haveweghoulmodels>>::execute(
        engine,
        CgG2HaveweghoulmodelsArgs::new(ghoul2),
    ) != 0
}

/// Raven `trap_G2API_GetBoltMatrix` — `CG_G2_GETBOLT`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETBOLT`).
///
/// C: `qboolean trap_G2API_GetBoltMatrix(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:791-795`
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
    let modelList = modelList.map_or(null_mut(), |m| m as *mut qhandle_t);
    <Engine as Execute<CgG2Getbolt>>::execute(
        engine,
        CgG2GetboltArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix as *mut mdxaBone_t,
            angles as *const vec3_t,
            position as *const vec3_t,
            frameNum,
            modelList,
            scale as *const vec3_t as *mut vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_GetBoltMatrix_NoReconstruct` — `CG_G2_GETBOLT_NOREC`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETBOLT_NOREC`).
///
/// C: `qboolean trap_G2API_GetBoltMatrix_NoReconstruct(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:797-801`
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix_NoReconstruct(
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
    let modelList = modelList.map_or(null_mut(), |m| m as *mut qhandle_t);
    <Engine as Execute<CgG2GetboltNorec>>::execute(
        engine,
        CgG2GetboltNorecArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix as *mut mdxaBone_t,
            angles as *const vec3_t,
            position as *const vec3_t,
            frameNum,
            modelList,
            scale as *const vec3_t as *mut vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_GetBoltMatrix_NoRecNoRot` — `CG_G2_GETBOLT_NOREC_NOROT`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETBOLT_NOREC_NOROT`).
///
/// C: `qboolean trap_G2API_GetBoltMatrix_NoRecNoRot(void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:803-807`
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoltMatrix_NoRecNoRot(
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
    let modelList = modelList.map_or(null_mut(), |m| m as *mut qhandle_t);
    <Engine as Execute<CgG2GetboltNorecNorot>>::execute(
        engine,
        CgG2GetboltNorecNorotArgs::new(
            ghoul2,
            modelIndex,
            boltIndex,
            matrix as *mut mdxaBone_t,
            angles as *const vec3_t,
            position as *const vec3_t,
            frameNum,
            modelList,
            scale as *const vec3_t as *mut vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_InitGhoul2Model` — `CG_G2_INITGHOUL2MODEL`
/// (token: `mp_abi::cgame::syscalls::CG_G2_INITGHOUL2MODEL`).
///
/// C: `int trap_G2API_InitGhoul2Model(void **ghoul2Ptr, const char *fileName, int modelIndex, qhandle_t customSkin, qhandle_t customShader, int modelFlags, int lodBias)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:809-813`
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
    let file_name_c = cstr(fileName);
    <Engine as Execute<CgG2Initghoul2model>>::execute(
        engine,
        CgG2Initghoul2modelArgs::new(
            ghoul2Ptr,
            file_name_c.as_ptr(),
            modelIndex,
            customSkin,
            customShader,
            modelFlags,
            lodBias,
        ),
    )
}

/// Raven `trap_G2API_SetSkin` — `CG_G2_SETSKIN` (token: `mp_abi::cgame::syscalls::CG_G2_SETSKIN`).
///
/// C: `qboolean trap_G2API_SetSkin(void *ghoul2, int modelIndex, qhandle_t customSkin, qhandle_t renderSkin)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:815-818`
pub fn G2API_SetSkin(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    customSkin: qhandle_t,
    renderSkin: qhandle_t,
) -> bool {
    <Engine as Execute<CgG2Setskin>>::execute(
        engine,
        CgG2SetskinArgs::new(ghoul2, modelIndex, customSkin, renderSkin),
    ) != 0
}

/// Raven `trap_G2API_CollisionDetect` — `CG_G2_COLLISIONDETECT`
/// (token: `mp_abi::cgame::syscalls::CG_G2_COLLISIONDETECT`).
///
/// C: `void trap_G2API_CollisionDetect ( CollisionRecord_t *collRecMap, void* ghoul2, const vec3_t angles, const vec3_t position, int frameNumber, int entNum, const vec3_t rayStart, const vec3_t rayEnd, const vec3_t scale, int traceFlags, int useLod, float fRadius )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:820-836`
///
/// `collRecMap` is the engine-written result array; the four vectors are
/// read-only at the seam and cast to the abi's writable pointer shape.
#[allow(clippy::too_many_arguments)]
pub fn G2API_CollisionDetect(
    engine: &Engine,
    collRecMap: &mut [CollisionRecord_t],
    ghoul2: *mut c_void,
    angles: &vec3_t,
    position: &vec3_t,
    frameNumber: c_int,
    entNum: c_int,
    rayStart: &vec3_t,
    rayEnd: &vec3_t,
    scale: &vec3_t,
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32,
) {
    <Engine as Execute<CgG2Collisiondetect>>::execute(
        engine,
        CgG2CollisiondetectArgs::new(
            collRecMap.as_mut_ptr(),
            ghoul2,
            angles.as_ptr(),
            position.as_ptr(),
            frameNumber,
            entNum,
            rayStart.as_ptr() as *mut f32,
            rayEnd.as_ptr() as *mut f32,
            scale.as_ptr() as *mut f32,
            traceFlags,
            useLod,
            fRadius,
        ),
    )
}

/// Raven `trap_G2API_CollisionDetectCache` — `CG_G2_COLLISIONDETECTCACHE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_COLLISIONDETECTCACHE`).
///
/// C: `void trap_G2API_CollisionDetectCache ( CollisionRecord_t *collRecMap, void* ghoul2, const vec3_t angles, const vec3_t position, int frameNumber, int entNum, const vec3_t rayStart, const vec3_t rayEnd, const vec3_t scale, int traceFlags, int useLod, float fRadius )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:838-854`
///
/// `collRecMap` is the engine-written result array; the four vectors are
/// read-only at the seam and cast to the abi's writable pointer shape.
#[allow(clippy::too_many_arguments)]
pub fn G2API_CollisionDetectCache(
    engine: &Engine,
    collRecMap: &mut [CollisionRecord_t],
    ghoul2: *mut c_void,
    angles: &vec3_t,
    position: &vec3_t,
    frameNumber: c_int,
    entNum: c_int,
    rayStart: &vec3_t,
    rayEnd: &vec3_t,
    scale: &vec3_t,
    traceFlags: c_int,
    useLod: c_int,
    fRadius: f32,
) {
    <Engine as Execute<CgG2Collisiondetectcache>>::execute(
        engine,
        CgG2CollisiondetectcacheArgs::new(
            collRecMap.as_mut_ptr(),
            ghoul2,
            angles.as_ptr(),
            position.as_ptr(),
            frameNumber,
            entNum,
            rayStart.as_ptr() as *mut f32,
            rayEnd.as_ptr() as *mut f32,
            scale.as_ptr() as *mut f32,
            traceFlags,
            useLod,
            fRadius,
        ),
    )
}

/// Raven `trap_G2API_CleanGhoul2Models` — `CG_G2_CLEANMODELS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_CLEANMODELS`).
///
/// C: `void trap_G2API_CleanGhoul2Models(void **ghoul2Ptr)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:856-859`
///
/// The engine writes the caller's instance slot back to `NULL`, so the
/// pointer-to-token stays a raw slot pointer at this seam.
pub fn G2API_CleanGhoul2Models(engine: &Engine, ghoul2Ptr: *mut *mut c_void) {
    <Engine as Execute<CgG2Cleanmodels>>::execute(engine, CgG2CleanmodelsArgs::new(ghoul2Ptr))
}

/// Raven `trap_G2API_SetBoneAngles` — `CG_G2_ANGLEOVERRIDE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_ANGLEOVERRIDE`).
///
/// C: `qboolean trap_G2API_SetBoneAngles(void *ghoul2, int modelIndex, const char *boneName, const vec3_t angles, const int flags, const int up, const int right, const int forward, qhandle_t *modelList, int blendTime , int currentTime )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:861-866`
#[allow(clippy::too_many_arguments)]
pub fn G2API_SetBoneAngles(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    angles: &vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: Option<&mut qhandle_t>,
    blendTime: c_int,
    currentTime: c_int,
) -> bool {
    let modelList = modelList.map_or(null_mut(), |m| m as *mut qhandle_t);
    <Engine as Execute<CgG2Angleoverride>>::execute(
        engine,
        CgG2AngleoverrideArgs::new(
            ghoul2,
            modelIndex,
            cstr(boneName),
            angles as *const vec3_t,
            flags,
            up,
            right,
            forward,
            modelList,
            blendTime,
            currentTime,
        ),
    ) != 0
}

/// Raven `trap_G2API_SetBoneAnim` — `CG_G2_PLAYANIM`
/// (token: `mp_abi::cgame::syscalls::CG_G2_PLAYANIM`).
///
/// C: `qboolean trap_G2API_SetBoneAnim(void *ghoul2, const int modelIndex, const char *boneName, const int startFrame, const int endFrame, const int flags, const float animSpeed, const int currentTime, const float setFrame , const int blendTime )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:868-872`
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
    <Engine as Execute<CgG2Playanim>>::execute(
        engine,
        CgG2PlayanimArgs::new(
            ghoul2,
            modelIndex,
            cstr(boneName),
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

/// Raven `trap_G2API_GetBoneAnim` — `CG_G2_GETBONEANIM`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETBONEANIM`).
///
/// C: `qboolean trap_G2API_GetBoneAnim(void *ghoul2, const char *boneName, const int currentTime, float *currentFrame, int *startFrame, int *endFrame, int *flags, float *animSpeed, int *modelList, const int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:874-878`
///
/// `modelList` is nullable in Raven — bg's `BG_IK_MoveArm` passes NULL
/// (`bg_pmove.c:8721`) — so `None` rides through as a null pointer.
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoneAnim(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    currentTime: c_int,
    currentFrame: &mut f32,
    startFrame: &mut c_int,
    endFrame: &mut c_int,
    flags: &mut c_int,
    animSpeed: &mut f32,
    modelList: Option<&mut c_int>,
    modelIndex: c_int,
) -> bool {
    let bone_name_c = cstr(boneName);
    let model_list_ptr = modelList.map_or(null_mut(), |m| m as *mut c_int);
    <Engine as Execute<CgG2Getboneanim>>::execute(
        engine,
        CgG2GetboneanimArgs::new(
            ghoul2,
            bone_name_c.as_ptr(),
            currentTime,
            currentFrame as *mut f32,
            startFrame as *mut c_int,
            endFrame as *mut c_int,
            flags as *mut c_int,
            animSpeed as *mut f32,
            model_list_ptr,
            modelIndex,
        ),
    ) != 0
}

/// Raven `trap_G2API_GetBoneFrame` — `CG_G2_GETBONEFRAME`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETBONEFRAME`).
///
/// C: `qboolean trap_G2API_GetBoneFrame(void *ghoul2, const char *boneName, const int currentTime, float *currentFrame, int *modelList, const int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:880-883`
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetBoneFrame(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    currentTime: c_int,
    currentFrame: &mut f32,
    modelList: &mut c_int,
    modelIndex: c_int,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Getboneframe>>::execute(
        engine,
        CgG2GetboneframeArgs::new(
            ghoul2,
            bone_name_c.as_ptr(),
            currentTime,
            currentFrame as *mut f32,
            modelList as *mut c_int,
            modelIndex,
        ),
    ) != 0
}

/// Raven `trap_G2API_GetGLAName` — `CG_G2_GETGLANAME`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETGLANAME`).
///
/// C: `void trap_G2API_GetGLAName(void *ghoul2, int modelIndex, char *fillBuf)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:885-888`
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
    <Engine as Execute<CgG2Getglaname>>::execute(
        engine,
        CgG2GetglanameArgs::new(ghoul2, modelIndex, buffer.as_mut_ptr() as *mut c_char),
    );
    buf_to_string(&buffer)
}

/// Raven `trap_G2API_CopyGhoul2Instance` — `CG_G2_COPYGHOUL2INSTANCE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_COPYGHOUL2INSTANCE`).
///
/// C: `int trap_G2API_CopyGhoul2Instance(void *g2From, void *g2To, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:890-893`
pub fn G2API_CopyGhoul2Instance(
    engine: &Engine,
    g2From: *mut c_void,
    g2To: *mut c_void,
    modelIndex: c_int,
) -> c_int {
    <Engine as Execute<CgG2Copyghoul2instance>>::execute(
        engine,
        CgG2Copyghoul2instanceArgs::new(g2From, g2To, modelIndex),
    )
}

/// Raven `trap_G2API_CopySpecificGhoul2Model` — `CG_G2_COPYSPECIFICGHOUL2MODEL`
/// (token: `mp_abi::cgame::syscalls::CG_G2_COPYSPECIFICGHOUL2MODEL`).
///
/// C: `void trap_G2API_CopySpecificGhoul2Model(void *g2From, int modelFrom, void *g2To, int modelTo)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:895-898`
pub fn G2API_CopySpecificGhoul2Model(
    engine: &Engine,
    g2From: *mut c_void,
    modelFrom: c_int,
    g2To: *mut c_void,
    modelTo: c_int,
) {
    <Engine as Execute<CgG2Copyspecificghoul2model>>::execute(
        engine,
        CgG2Copyspecificghoul2modelArgs::new(g2From, modelFrom, g2To, modelTo),
    )
}

/// Raven `trap_G2API_DuplicateGhoul2Instance` — `CG_G2_DUPLICATEGHOUL2INSTANCE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_DUPLICATEGHOUL2INSTANCE`).
///
/// C: `void trap_G2API_DuplicateGhoul2Instance(void *g2From, void **g2To)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:900-903`
///
/// The engine writes the caller's duplicated-instance slot, so the
/// pointer-to-token stays a raw slot pointer at this seam.
pub fn G2API_DuplicateGhoul2Instance(engine: &Engine, g2From: *mut c_void, g2To: *mut *mut c_void) {
    <Engine as Execute<CgG2Duplicateghoul2instance>>::execute(
        engine,
        CgG2Duplicateghoul2instanceArgs::new(g2From, g2To),
    )
}

/// Raven `trap_G2API_HasGhoul2ModelOnIndex` — `CG_G2_HASGHOUL2MODELONINDEX`
/// (token: `mp_abi::cgame::syscalls::CG_G2_HASGHOUL2MODELONINDEX`).
///
/// C: `qboolean trap_G2API_HasGhoul2ModelOnIndex(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:905-908`
pub fn G2API_HasGhoul2ModelOnIndex(
    engine: &Engine,
    ghlInfo: *mut c_void,
    modelIndex: c_int,
) -> bool {
    <Engine as Execute<CgG2Hasghoul2modelonindex>>::execute(
        engine,
        CgG2Hasghoul2modelonindexArgs::new(ghlInfo, modelIndex),
    ) != 0
}

/// Raven `trap_G2API_RemoveGhoul2Model` — `CG_G2_REMOVEGHOUL2MODEL`
/// (token: `mp_abi::cgame::syscalls::CG_G2_REMOVEGHOUL2MODEL`).
///
/// C: `qboolean trap_G2API_RemoveGhoul2Model(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:910-913`
pub fn G2API_RemoveGhoul2Model(engine: &Engine, ghlInfo: *mut c_void, modelIndex: c_int) -> bool {
    <Engine as Execute<CgG2Removeghoul2model>>::execute(
        engine,
        CgG2Removeghoul2modelArgs::new(ghlInfo, modelIndex),
    ) != 0
}

/// Raven `trap_G2API_SkinlessModel` — `CG_G2_SKINLESSMODEL`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SKINLESSMODEL`).
///
/// C: `qboolean trap_G2API_SkinlessModel(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:915-918`
pub fn G2API_SkinlessModel(engine: &Engine, ghlInfo: *mut c_void, modelIndex: c_int) -> bool {
    <Engine as Execute<CgG2Skinlessmodel>>::execute(
        engine,
        CgG2SkinlessmodelArgs::new(ghlInfo, modelIndex),
    ) != 0
}

/// Raven `trap_G2API_GetNumGoreMarks` — `CG_G2_GETNUMGOREMARKS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETNUMGOREMARKS`).
///
/// C: `int trap_G2API_GetNumGoreMarks(void *ghlInfo, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:920-923`
pub fn G2API_GetNumGoreMarks(engine: &Engine, ghlInfo: *mut c_void, modelIndex: c_int) -> c_int {
    <Engine as Execute<CgG2Getnumgoremarks>>::execute(
        engine,
        CgG2GetnumgoremarksArgs::new(ghlInfo, modelIndex),
    )
}

/// Raven `trap_G2API_AddSkinGore` — `CG_G2_ADDSKINGORE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_ADDSKINGORE`).
///
/// C: `void trap_G2API_AddSkinGore(void *ghlInfo,SSkinGoreData *gore)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:925-928`
///
/// `gore` stays an opaque engine token: `SSkinGoreData` has no MP-side Rust
/// type (the only Rust definition lives in the engine ghoul2 crate, which
/// mp_cgame must not depend on).
//TODO: Port SSkinGoreData
// Source: oracle/codemp/game/q_shared.h:3111-3145
pub fn G2API_AddSkinGore(engine: &Engine, ghlInfo: *mut c_void, gore: *mut c_void) {
    <Engine as Execute<CgG2Addskingore>>::execute(engine, CgG2AddskingoreArgs::new(ghlInfo, gore))
}

/// Raven `trap_G2API_ClearSkinGore` — `CG_G2_CLEARSKINGORE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_CLEARSKINGORE`).
///
/// C: `void trap_G2API_ClearSkinGore ( void* ghlInfo )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:930-933`
pub fn G2API_ClearSkinGore(engine: &Engine, ghlInfo: *mut c_void) {
    <Engine as Execute<CgG2Clearskingore>>::execute(engine, CgG2ClearskingoreArgs::new(ghlInfo))
}

/// Raven `trap_G2API_Ghoul2Size` — `CG_G2_SIZE` (token: `mp_abi::cgame::syscalls::CG_G2_SIZE`).
///
/// C: `int trap_G2API_Ghoul2Size ( void* ghlInfo )`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:935-938`
pub fn G2API_Ghoul2Size(engine: &Engine, ghlInfo: *mut c_void) -> c_int {
    <Engine as Execute<CgG2Size>>::execute(engine, CgG2SizeArgs::new(ghlInfo))
}

/// Raven `trap_G2API_AddBolt` — `CG_G2_ADDBOLT` (token: `mp_abi::cgame::syscalls::CG_G2_ADDBOLT`).
///
/// C: `int trap_G2API_AddBolt(void *ghoul2, int modelIndex, const char *boneName)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:940-943`
pub fn G2API_AddBolt(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
) -> c_int {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Addbolt>>::execute(
        engine,
        CgG2AddboltArgs::new(ghoul2, modelIndex, bone_name_c.as_ptr()),
    )
}

/// Raven `trap_G2API_AttachEnt` — `CG_G2_ATTACHENT` (token: `mp_abi::cgame::syscalls::CG_G2_ATTACHENT`).
///
/// C: `qboolean trap_G2API_AttachEnt(int *boltInfo, void *ghlInfoTo, int toBoltIndex, int entNum, int toModelNum)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:945-948`
pub fn G2API_AttachEnt(
    engine: &Engine,
    boltInfo: &mut c_int,
    ghlInfoTo: *mut c_void,
    toBoltIndex: c_int,
    entNum: c_int,
    toModelNum: c_int,
) -> bool {
    <Engine as Execute<CgG2Attachent>>::execute(
        engine,
        CgG2AttachentArgs::new(
            boltInfo as *mut c_int,
            ghlInfoTo,
            toBoltIndex,
            entNum,
            toModelNum,
        ),
    ) != 0
}

/// Raven `trap_G2API_SetBoltInfo` — `CG_G2_SETBOLTON` (token: `mp_abi::cgame::syscalls::CG_G2_SETBOLTON`).
///
/// C: `void trap_G2API_SetBoltInfo(void *ghoul2, int modelIndex, int boltInfo)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:950-953`
pub fn G2API_SetBoltInfo(engine: &Engine, ghoul2: *mut c_void, modelIndex: c_int, boltInfo: c_int) {
    <Engine as Execute<CgG2Setbolton>>::execute(
        engine,
        CgG2SetboltonArgs::new(ghoul2, modelIndex, boltInfo),
    )
}

/// Raven `trap_G2API_SetRootSurface` — `CG_G2_SETROOTSURFACE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SETROOTSURFACE`).
///
/// C: `qboolean trap_G2API_SetRootSurface(void *ghoul2, const int modelIndex, const char *surfaceName)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:955-958`
pub fn G2API_SetRootSurface(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    surfaceName: &str,
) -> bool {
    let surface_name_c = cstr(surfaceName);
    <Engine as Execute<CgG2Setrootsurface>>::execute(
        engine,
        CgG2SetrootsurfaceArgs::new(ghoul2, modelIndex, surface_name_c.as_ptr()),
    ) != 0
}

/// Raven `trap_G2API_SetSurfaceOnOff` — `CG_G2_SETSURFACEONOFF`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SETSURFACEONOFF`).
///
/// C: `qboolean trap_G2API_SetSurfaceOnOff(void *ghoul2, const char *surfaceName, const int flags)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:960-963`
pub fn G2API_SetSurfaceOnOff(
    engine: &Engine,
    ghoul2: *mut c_void,
    surfaceName: &str,
    flags: c_int,
) -> bool {
    let surface_name_c = cstr(surfaceName);
    <Engine as Execute<CgG2Setsurfaceonoff>>::execute(
        engine,
        CgG2SetsurfaceonoffArgs::new(ghoul2, surface_name_c.as_ptr(), flags),
    ) != 0
}

/// Raven `trap_G2API_SetNewOrigin` — `CG_G2_SETNEWORIGIN`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SETNEWORIGIN`).
///
/// C: `qboolean trap_G2API_SetNewOrigin(void *ghoul2, const int boltIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:965-968`
pub fn G2API_SetNewOrigin(engine: &Engine, ghoul2: *mut c_void, boltIndex: c_int) -> bool {
    <Engine as Execute<CgG2Setneworigin>>::execute(
        engine,
        CgG2SetneworiginArgs::new(ghoul2, boltIndex),
    ) != 0
}

/// Raven `trap_G2API_DoesBoneExist` — `CG_G2_DOESBONEEXIST`
/// (token: `mp_abi::cgame::syscalls::CG_G2_DOESBONEEXIST`).
///
/// C: `qboolean trap_G2API_DoesBoneExist(void *ghoul2, int modelIndex, const char *boneName)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:971-974`
pub fn G2API_DoesBoneExist(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Doesboneexist>>::execute(
        engine,
        CgG2DoesboneexistArgs::new(ghoul2, modelIndex, bone_name_c.as_ptr()),
    ) != 0
}

/// Raven `trap_G2API_GetSurfaceRenderStatus` — `CG_G2_GETSURFACERENDERSTATUS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETSURFACERENDERSTATUS`).
///
/// C: `int trap_G2API_GetSurfaceRenderStatus(void *ghoul2, const int modelIndex, const char *surfaceName)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:976-979`
pub fn G2API_GetSurfaceRenderStatus(
    engine: &Engine,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    surfaceName: &str,
) -> c_int {
    let surface_name_c = cstr(surfaceName);
    <Engine as Execute<CgG2Getsurfacerenderstatus>>::execute(
        engine,
        CgG2GetsurfacerenderstatusArgs::new(ghoul2, modelIndex, surface_name_c.as_ptr()),
    )
}

/// Raven `trap_G2API_GetTime` — `CG_G2_GETTIME` (token: `mp_abi::cgame::syscalls::CG_G2_GETTIME`).
///
/// C: `int trap_G2API_GetTime(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:981-984`
pub fn G2API_GetTime(engine: &Engine) -> c_int {
    <Engine as Execute<CgG2Gettime>>::execute(engine, CgG2GettimeArgs::new())
}

/// Raven `trap_G2API_SetTime` — `CG_G2_SETTIME` (token: `mp_abi::cgame::syscalls::CG_G2_SETTIME`).
///
/// C: `void trap_G2API_SetTime(int time, int clock)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:986-989`
pub fn G2API_SetTime(engine: &Engine, time: c_int, clock: c_int) {
    <Engine as Execute<CgG2Settime>>::execute(engine, CgG2SettimeArgs::new(time, clock))
}

/// Raven `trap_G2API_AbsurdSmoothing` — `CG_G2_ABSURDSMOOTHING`
/// (token: `mp_abi::cgame::syscalls::CG_G2_ABSURDSMOOTHING`).
///
/// C: `void trap_G2API_AbsurdSmoothing(void *ghoul2, qboolean status)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:992-995`
pub fn G2API_AbsurdSmoothing(engine: &Engine, ghoul2: *mut c_void, status: bool) {
    <Engine as Execute<CgG2Absurdsmoothing>>::execute(
        engine,
        CgG2AbsurdsmoothingArgs::new(ghoul2, c_int::from(status)),
    )
}

/// Raven `trap_G2API_SetRagDoll` — `CG_G2_SETRAGDOLL` (token: `mp_abi::cgame::syscalls::CG_G2_SETRAGDOLL`).
///
/// C: `void trap_G2API_SetRagDoll(void *ghoul2, sharedRagDollParams_t *params)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:998-1001`
pub fn G2API_SetRagDoll(
    engine: &Engine,
    ghoul2: *mut c_void,
    params: Option<&mut sharedRagDollParams_t>,
) {
    // NULL params is Raven's rag-doll reset arm
    let params_ptr = params.map_or(null_mut(), |p| p as *mut sharedRagDollParams_t);
    <Engine as Execute<CgG2Setragdoll>>::execute(
        engine,
        CgG2SetragdollArgs::new(ghoul2, params_ptr),
    )
}

/// Raven `trap_G2API_AnimateG2Models` — `CG_G2_ANIMATEG2MODELS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_ANIMATEG2MODELS`).
///
/// C: `void trap_G2API_AnimateG2Models(void *ghoul2, int time, sharedRagDollUpdateParams_t *params)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1003-1006`
pub fn G2API_AnimateG2Models(
    engine: &Engine,
    ghoul2: *mut c_void,
    time: c_int,
    params: &mut sharedRagDollUpdateParams_t,
) {
    <Engine as Execute<CgG2Animateg2models>>::execute(
        engine,
        CgG2Animateg2modelsArgs::new(ghoul2, time, params as *mut sharedRagDollUpdateParams_t),
    )
}

/// Raven `trap_G2API_RagPCJConstraint` — `CG_G2_RAGPCJCONSTRAINT`
/// (token: `mp_abi::cgame::syscalls::CG_G2_RAGPCJCONSTRAINT`).
///
/// C: `qboolean trap_G2API_RagPCJConstraint(void *ghoul2, const char *boneName, vec3_t min, vec3_t max)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1010-1013`
pub fn G2API_RagPCJConstraint(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    min: &vec3_t,
    max: &vec3_t,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Ragpcjconstraint>>::execute(
        engine,
        CgG2RagpcjconstraintArgs::new(
            ghoul2,
            bone_name_c.as_ptr(),
            min as *const vec3_t as *mut vec3_t,
            max as *const vec3_t as *mut vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_RagPCJGradientSpeed` — `CG_G2_RAGPCJGRADIENTSPEED`
/// (token: `mp_abi::cgame::syscalls::CG_G2_RAGPCJGRADIENTSPEED`).
///
/// C: `qboolean trap_G2API_RagPCJGradientSpeed(void *ghoul2, const char *boneName, const float speed)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1015-1018`
pub fn G2API_RagPCJGradientSpeed(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    speed: f32,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Ragpcjgradientspeed>>::execute(
        engine,
        CgG2RagpcjgradientspeedArgs::new(ghoul2, bone_name_c.as_ptr(), speed),
    ) != 0
}

/// Raven `trap_G2API_RagEffectorGoal` — `CG_G2_RAGEFFECTORGOAL`
/// (token: `mp_abi::cgame::syscalls::CG_G2_RAGEFFECTORGOAL`).
///
/// `pos` is nullable in Raven — NULL clears the bone's over-goal instead of
/// setting one (`oracle/codemp/ghoul2/G2_API.cpp:1552-1555`), so `None` rides
/// through as a null pointer.
///
/// C: `qboolean trap_G2API_RagEffectorGoal(void *ghoul2, const char *boneName, vec3_t pos)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1020-1023`
pub fn G2API_RagEffectorGoal(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    pos: Option<&vec3_t>,
) -> bool {
    let bone_name_c = cstr(boneName);
    let pos_ptr = pos.map_or(null_mut(), |p| p as *const vec3_t as *mut vec3_t);
    <Engine as Execute<CgG2Rageffectorgoal>>::execute(
        engine,
        CgG2RageffectorgoalArgs::new(ghoul2, bone_name_c.as_ptr(), pos_ptr),
    ) != 0
}

/// Raven `trap_G2API_GetRagBonePos` — `CG_G2_GETRAGBONEPOS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETRAGBONEPOS`).
///
/// C: `qboolean trap_G2API_GetRagBonePos(void *ghoul2, const char *boneName, vec3_t pos, vec3_t entAngles, vec3_t entPos, vec3_t entScale)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1025-1028`
#[allow(clippy::too_many_arguments)]
pub fn G2API_GetRagBonePos(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    pos: &mut vec3_t,
    entAngles: &vec3_t,
    entPos: &vec3_t,
    entScale: &vec3_t,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Getragbonepos>>::execute(
        engine,
        CgG2GetragboneposArgs::new(
            ghoul2,
            bone_name_c.as_ptr(),
            pos as *mut vec3_t,
            entAngles as *const vec3_t,
            entPos as *const vec3_t,
            entScale as *const vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_RagEffectorKick` — `CG_G2_RAGEFFECTORKICK`
/// (token: `mp_abi::cgame::syscalls::CG_G2_RAGEFFECTORKICK`).
///
/// C: `qboolean trap_G2API_RagEffectorKick(void *ghoul2, const char *boneName, vec3_t velocity)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1030-1033`
pub fn G2API_RagEffectorKick(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    velocity: &vec3_t,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Rageffectorkick>>::execute(
        engine,
        CgG2RageffectorkickArgs::new(
            ghoul2,
            bone_name_c.as_ptr(),
            velocity as *const vec3_t as *mut vec3_t,
        ),
    ) != 0
}

/// Raven `trap_G2API_RagForceSolve` — `CG_G2_RAGFORCESOLVE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_RAGFORCESOLVE`).
///
/// C: `qboolean trap_G2API_RagForceSolve(void *ghoul2, qboolean force)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1035-1038`
pub fn G2API_RagForceSolve(engine: &Engine, ghoul2: *mut c_void, force: bool) -> bool {
    <Engine as Execute<CgG2Ragforcesolve>>::execute(
        engine,
        CgG2RagforcesolveArgs::new(ghoul2, c_int::from(force)),
    ) != 0
}

/// Raven `trap_G2API_SetBoneIKState` — `CG_G2_SETBONEIKSTATE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_SETBONEIKSTATE`).
///
/// C: `qboolean trap_G2API_SetBoneIKState(void *ghoul2, int time, const char *boneName, int ikState, sharedSetBoneIKStateParams_t *params)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1040-1043`
///
/// `boneName` is nullable in Raven: NULL selects the engine's init/reset-IK
/// branch, so `None` here encodes a null pointer on the wire.
pub fn G2API_SetBoneIKState(
    engine: &Engine,
    ghoul2: *mut c_void,
    time: c_int,
    boneName: Option<&str>,
    ikState: c_int,
    params: Option<&mut sharedSetBoneIKStateParams_t>,
) -> bool {
    let bone_name_c = boneName.map(cstr);
    let bone_name_ptr = bone_name_c.as_ref().map_or(null(), |b| b.as_ptr());
    let params_ptr = params.map_or(null_mut(), |p| p as *mut sharedSetBoneIKStateParams_t);
    <Engine as Execute<CgG2Setboneikstate>>::execute(
        engine,
        CgG2SetboneikstateArgs::new(ghoul2, time, bone_name_ptr, ikState, params_ptr),
    ) != 0
}

/// Raven `trap_G2API_IKMove` — `CG_G2_IKMOVE` (token: `mp_abi::cgame::syscalls::CG_G2_IKMOVE`).
///
/// C: `qboolean trap_G2API_IKMove(void *ghoul2, int time, sharedIKMoveParams_t *params)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1045-1048`
pub fn G2API_IKMove(
    engine: &Engine,
    ghoul2: *mut c_void,
    time: c_int,
    params: &mut sharedIKMoveParams_t,
) -> bool {
    <Engine as Execute<CgG2Ikmove>>::execute(
        engine,
        CgG2IkmoveArgs::new(ghoul2, time, params as *mut sharedIKMoveParams_t),
    ) != 0
}

/// Raven `trap_G2API_RemoveBone` — `CG_G2_REMOVEBONE` (token: `mp_abi::cgame::syscalls::CG_G2_REMOVEBONE`).
///
/// C: `qboolean trap_G2API_RemoveBone(void *ghoul2, const char *boneName, int modelIndex)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1050-1053`
pub fn G2API_RemoveBone(
    engine: &Engine,
    ghoul2: *mut c_void,
    boneName: &str,
    modelIndex: c_int,
) -> bool {
    let bone_name_c = cstr(boneName);
    <Engine as Execute<CgG2Removebone>>::execute(
        engine,
        CgG2RemoveboneArgs::new(ghoul2, bone_name_c.as_ptr(), modelIndex),
    ) != 0
}

/// Raven `trap_G2API_AttachInstanceToEntNum` — `CG_G2_ATTACHINSTANCETOENTNUM`
/// (token: `mp_abi::cgame::syscalls::CG_G2_ATTACHINSTANCETOENTNUM`).
///
/// C: `void trap_G2API_AttachInstanceToEntNum(void *ghoul2, int entityNum, qboolean server)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1060-1063`
pub fn G2API_AttachInstanceToEntNum(
    engine: &Engine,
    ghoul2: *mut c_void,
    entityNum: c_int,
    server: bool,
) {
    <Engine as Execute<CgG2Attachinstancetoentnum>>::execute(
        engine,
        CgG2AttachinstancetoentnumArgs::new(ghoul2, entityNum, c_int::from(server)),
    )
}

/// Raven `trap_G2API_ClearAttachedInstance` — `CG_G2_CLEARATTACHEDINSTANCE`
/// (token: `mp_abi::cgame::syscalls::CG_G2_CLEARATTACHEDINSTANCE`).
///
/// C: `void trap_G2API_ClearAttachedInstance(int entityNum)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1065-1068`
pub fn G2API_ClearAttachedInstance(engine: &Engine, entityNum: c_int) {
    <Engine as Execute<CgG2Clearattachedinstance>>::execute(
        engine,
        CgG2ClearattachedinstanceArgs::new(entityNum),
    )
}

/// Raven `trap_G2API_CleanEntAttachments` — `CG_G2_CLEANENTATTACHMENTS`
/// (token: `mp_abi::cgame::syscalls::CG_G2_CLEANENTATTACHMENTS`).
///
/// C: `void trap_G2API_CleanEntAttachments(void)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1070-1073`
pub fn G2API_CleanEntAttachments(engine: &Engine) {
    <Engine as Execute<CgG2Cleanentattachments>>::execute(
        engine,
        CgG2CleanentattachmentsArgs::new(),
    )
}

/// Raven `trap_G2API_OverrideServer` — `CG_G2_OVERRIDESERVER`
/// (token: `mp_abi::cgame::syscalls::CG_G2_OVERRIDESERVER`).
///
/// C: `qboolean trap_G2API_OverrideServer(void *serverInstance)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1075-1078`
pub fn G2API_OverrideServer(engine: &Engine, serverInstance: *mut c_void) -> bool {
    <Engine as Execute<CgG2Overrideserver>>::execute(
        engine,
        CgG2OverrideserverArgs::new(serverInstance),
    ) != 0
}

/// Raven `trap_G2API_GetSurfaceName` — `CG_G2_GETSURFACENAME`
/// (token: `mp_abi::cgame::syscalls::CG_G2_GETSURFACENAME`).
///
/// C: `void trap_G2API_GetSurfaceName(void *ghoul2, int surfNumber, int modelIndex, char *fillBuf)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1080-1083`
///
/// Raven passes an unbounded `char[MAX_QPATH]`; `buffer_len` names that width
/// at the call site.
pub fn G2API_GetSurfaceName(
    engine: &Engine,
    ghoul2: *mut c_void,
    surfNumber: c_int,
    modelIndex: c_int,
    buffer_len: usize,
) -> String {
    let mut buffer = vec![0u8; buffer_len];
    <Engine as Execute<CgG2Getsurfacename>>::execute(
        engine,
        CgG2GetsurfacenameArgs::new(
            ghoul2,
            surfNumber,
            modelIndex,
            buffer.as_mut_ptr() as *mut c_char,
        ),
    );
    buf_to_string(&buffer)
}

/// Raven `trap_CG_RegisterSharedMemory` — `CG_SET_SHARED_BUFFER`
/// (token: `mp_abi::cgame::syscalls::CG_SET_SHARED_BUFFER`).
///
/// C: `void trap_CG_RegisterSharedMemory(char *memory)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1085-1088`
///
/// Registers cgame's shared-memory block with the engine.
/// `MAX_CG_SHARED_BUFFER_SIZE` is 2048 (`oracle/codemp/cgame/cg_public.h:593`).
/// Per DEC-46.6 the registered block is the pinned `Box<[u8; 2048]>` the
/// future `CgWorld` owns; this wrapper takes the array by `&mut` until that
/// type lands.
pub fn CG_RegisterSharedMemory(engine: &Engine, memory: &mut [u8; 2048]) {
    <Engine as Execute<CgSetSharedBuffer>>::execute(
        engine,
        CgSetSharedBufferArgs::new(memory.as_mut_ptr() as *mut c_char),
    )
}

/// Raven `trap_CM_RegisterTerrain` — `CG_CM_REGISTER_TERRAIN`
/// (token: `mp_abi::cgame::syscalls::CG_CM_REGISTER_TERRAIN`).
///
/// C: `int trap_CM_RegisterTerrain(const char *config)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1090-1093`
pub fn CM_RegisterTerrain(engine: &Engine, config: &str) -> c_int {
    let config_c = cstr(config);
    <Engine as Execute<CgCmRegisterTerrain>>::execute(
        engine,
        CgCmRegisterTerrainArgs::new(config_c.as_ptr()),
    )
}

/// Raven `trap_RMG_Init` — `CG_RMG_INIT` (token: `mp_abi::cgame::syscalls::CG_RMG_INIT`).
///
/// C: `void trap_RMG_Init(int terrainID, const char *terrainInfo)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1095-1098`
pub fn RMG_Init(engine: &Engine, terrainID: c_int, terrainInfo: &str) {
    let terrain_info_c = cstr(terrainInfo);
    <Engine as Execute<CgRmgInit>>::execute(
        engine,
        CgRmgInitArgs::new(terrainID, terrain_info_c.as_ptr()),
    )
}

/// Raven `trap_RE_InitRendererTerrain` — `CG_RE_INIT_RENDERER_TERRAIN`
/// (token: `mp_abi::cgame::syscalls::CG_RE_INIT_RENDERER_TERRAIN`).
///
/// C: `void trap_RE_InitRendererTerrain(const char *info)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1100-1103`
pub fn RE_InitRendererTerrain(engine: &Engine, info: &str) {
    let info_c = cstr(info);
    <Engine as Execute<CgReInitRendererTerrain>>::execute(
        engine,
        CgReInitRendererTerrainArgs::new(info_c.as_ptr()),
    )
}

/// Raven `trap_R_WeatherContentsOverride` — `CG_R_WEATHER_CONTENTS_OVERRIDE`
/// (token: `mp_abi::cgame::syscalls::CG_R_WEATHER_CONTENTS_OVERRIDE`).
///
/// C: `void trap_R_WeatherContentsOverride(int contents)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1105-1108`
pub fn R_WeatherContentsOverride(engine: &Engine, contents: c_int) {
    <Engine as Execute<CgRWeatherContentsOverride>>::execute(
        engine,
        CgRWeatherContentsOverrideArgs::new(contents),
    )
}

/// Raven `trap_R_WorldEffectCommand` — `CG_R_WORLDEFFECTCOMMAND`
/// (token: `mp_abi::cgame::syscalls::CG_R_WORLDEFFECTCOMMAND`).
///
/// C: `void trap_R_WorldEffectCommand(const char *cmd)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1110-1113`
pub fn R_WorldEffectCommand(engine: &Engine, cmd: &str) {
    let cmd_c = CString::new(string_to_latin1(cmd)).unwrap();
    <Engine as Execute<CgRWorldeffectcommand>>::execute(
        engine,
        CgRWorldeffectcommandArgs::new(cmd_c.as_ptr()),
    )
}

/// Raven `trap_WE_AddWeatherZone` — `CG_WE_ADDWEATHERZONE`
/// (token: `mp_abi::cgame::syscalls::CG_WE_ADDWEATHERZONE`).
///
/// C: `void trap_WE_AddWeatherZone(const vec3_t mins, const vec3_t maxs)`
/// Source: `oracle/codemp/cgame/cg_syscalls.c:1115-1118`
pub fn WE_AddWeatherZone(engine: &Engine, mins: &vec3_t, maxs: &vec3_t) {
    <Engine as Execute<CgWeAddweatherzone>>::execute(
        engine,
        CgWeAddweatherzoneArgs::new(mins as *const vec3_t, maxs as *const vec3_t),
    )
}
