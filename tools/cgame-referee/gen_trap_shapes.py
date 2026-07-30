#!/usr/bin/env python3
# generates trap-shapes.json from the hand-classified table below.
# ground truth is the engine side: OpenJK CL_CgameSystemCalls dispatch
# (~/Developer/Milo/OpenJK/codemp/client/cl_cgameapi.cpp) - what the engine
# reads/writes through every arg. numbering is cgameImport_t
# (oracle/codemp/cgame/cg_public.h, byte-identical to OpenJK cgameImportLegacy_e).
#
# run: python3 gen_trap_shapes.py > trap-shapes.json
#
# arg kinds: scalar / in_str / in_buf / out_buf / inout_buf / double_ptr /
#            retained_ptr  (see README.md)
# ret kinds: void / scalar / handle / float   (float = FloatAsInt bit-return)

import json

OPENJK = "OpenJK/codemp/client/cl_cgameapi.cpp"

# --- arg-shape shorthands ------------------------------------------------
def S(t="int32", note=None):            # scalar value word
    a = {"kind": "scalar", "type": t}
    if note: a["note"] = note
    return a
def F(note=None):                        # float value word (VMF)
    a = {"kind": "scalar", "type": "float"}
    if note: a["note"] = note
    return a
def istr(t="char*", note=None):          # NUL-terminated string, engine reads
    a = {"kind": "in_str", "type": t}
    if note: a["note"] = note
    return a
def ibuf(t, size=None, len_arg=None, note=None):
    a = {"kind": "in_buf", "type": t}
    if size is not None: a["size_of"] = size
    if len_arg is not None: a["len_arg"] = len_arg
    if note: a["note"] = note
    return a
def obuf(t, size=None, len_arg=None, note=None):
    a = {"kind": "out_buf", "type": t}
    if size is not None: a["size_of"] = size
    if len_arg is not None: a["len_arg"] = len_arg
    if note: a["note"] = note
    return a
def iobuf(t, size=None, len_arg=None, note=None):
    a = {"kind": "inout_buf", "type": t}
    if size is not None: a["size_of"] = size
    if len_arg is not None: a["len_arg"] = len_arg
    if note: a["note"] = note
    return a
def vin(note=None):  return ibuf("vec3_t", 12, note=note)
def vout(note=None): return obuf("vec3_t", 12, note=note)
def viob(note=None): return iobuf("vec3_t", 12, note=note)
def dptr(note):      return {"kind": "double_ptr", "type": "void**", "size_of": 8, "note": note}
def g2h(note="opaque CGhoul2Info_v* host handle minted by the engine (InitGhoul2Model); a token, not a buffer to serialize"):
    return {"kind": "scalar", "type": "CGhoul2Info_v*", "size_of": 8, "note": note}

# --- the table -----------------------------------------------------------
# (name, ret, [args], optional note)  in cgameImport_t declaration order.
# each entry's `num` and `cite` line are filled in from the maps below.

BLOCK0 = [
 ("CG_PRINT", "void", [istr(note="format string, printed verbatim")]),
 ("CG_ERROR", "void", [istr(note="format string, drops to ERR_DROP")]),
 ("CG_MILLISECONDS", "scalar", []),
 ("CG_PRECISIONTIMER_START", "void", [dptr("engine mints a timing_c and writes its host ptr into *arg1; retained until PRECISIONTIMER_END frees it")]),
 ("CG_PRECISIONTIMER_END", "scalar", [S("void*", "raw host ptr from PRECISIONTIMER_START (args[1], not VMA); engine deletes it")]),
 ("CG_CVAR_REGISTER", "void", [obuf("vmCvar_t", note="engine fills the vmCvar_t"), istr(note="var name"), istr(note="default value"), S("int32", "flags")]),
 ("CG_CVAR_UPDATE", "void", [iobuf("vmCvar_t", note="reads name+modCount, writes current value")]),
 ("CG_CVAR_SET", "void", [istr(note="var name"), istr(note="value")]),
 ("CG_CVAR_VARIABLESTRINGBUFFER", "void", [istr(note="var name"), obuf("char", len_arg=3, note="value written, capped to args[3]"), S("int32", "bufsize")]),
 ("CG_CVAR_GETHIDDENVALUE", "scalar", [istr(note="var name")]),
 ("CG_ARGC", "scalar", []),
 ("CG_ARGV", "void", [S("int32", "arg index"), obuf("char", len_arg=3), S("int32", "bufsize")]),
 ("CG_ARGS", "void", [obuf("char", len_arg=2), S("int32", "bufsize")]),
 ("CG_FS_FOPENFILE", "scalar", [istr(note="qpath"), obuf("fileHandle_t", 4, note="handle written"), S("fsMode_t", "mode")]),
 ("CG_FS_READ", "void", [obuf("byte", len_arg=2, note="file bytes written into buffer"), S("int32", "len"), S("fileHandle_t", "handle")]),
 ("CG_FS_WRITE", "void", [ibuf("byte", len_arg=2, note="buffer bytes written to file"), S("int32", "len"), S("fileHandle_t", "handle")]),
 ("CG_FS_FCLOSEFILE", "void", [S("fileHandle_t", "handle")]),
 ("CG_FS_GETFILELIST", "scalar", [istr(note="path"), istr(note="extension"), obuf("char", len_arg=4, note="NUL-separated list"), S("int32", "bufsize")]),
 ("CG_SENDCONSOLECOMMAND", "void", [istr(note="command text")]),
 ("CG_ADDCOMMAND", "void", [istr(note="command name")]),
 ("CG_REMOVECOMMAND", "void", [istr(note="command name")]),
 ("CG_SENDCLIENTCOMMAND", "void", [istr(note="reliable command text")]),
 ("CG_UPDATESCREEN", "void", []),
 ("CG_CM_LOADMAP", "void", [istr(note="bsp name"), S("int32", "subBSP flag")], "DIVERGE: oracle branches on args[2] to CM_LoadSubBSP(va(\"maps/%s.bsp\", str+1)); OpenJK CL_CM_LoadMap(str, qbool). arg shapes identical."),
 ("CG_CM_NUMINLINEMODELS", "scalar", []),
 ("CG_CM_INLINEMODEL", "handle", [S("int32", "model index")]),
 ("CG_CM_TEMPBOXMODEL", "handle", [vin("mins"), vin("maxs")]),
 ("CG_CM_TEMPCAPSULEMODEL", "handle", [vin("mins"), vin("maxs")]),
 ("CG_CM_POINTCONTENTS", "scalar", [vin("point"), S("int32", "passEntityNum")]),
 ("CG_CM_TRANSFORMEDPOINTCONTENTS", "scalar", [vin("point"), S("int32", "passEntityNum"), vin("origin"), vin("angles")]),
 ("CG_CM_BOXTRACE", "void", [obuf("trace_t", note="trace result"), vin("start"), vin("mins"), vin("maxs"), vin("end"), S("int32", "passEntityNum"), S("int32", "contentmask")]),
 ("CG_CM_CAPSULETRACE", "void", [obuf("trace_t", note="trace result"), vin("start"), vin("mins"), vin("maxs"), vin("end"), S("int32", "passEntityNum"), S("int32", "contentmask")]),
 ("CG_CM_TRANSFORMEDBOXTRACE", "void", [obuf("trace_t"), vin("start"), vin("mins"), vin("maxs"), vin("end"), S("int32", "passEntityNum"), S("int32", "contentmask"), vin("origin"), vin("angles")]),
 ("CG_CM_TRANSFORMEDCAPSULETRACE", "void", [obuf("trace_t"), vin("start"), vin("mins"), vin("maxs"), vin("end"), S("int32", "passEntityNum"), S("int32", "contentmask"), vin("origin"), vin("angles")]),
 ("CG_CM_MARKFRAGMENTS", "scalar", [S("int32", "numPoints"), ibuf("vec3_t", 12, len_arg=1, note="points array, count = args[1]"), vin("projection"), S("int32", "maxPoints"), obuf("vec3_t", 12, note="pointBuffer OUT, up to args[4] verts (args[4]*12 bytes)"), S("int32", "maxFragments"), obuf("markFragment_t", note="fragment array OUT, up to args[6]")]),
 ("CG_S_GETVOICEVOLUME", "scalar", [S("int32", "entityNum")], "DIVERGE: oracle indexes s_entityWavVol[args[1]] directly; OpenJK CL_S_GetVoiceVolume. same shape."),
 ("CG_S_MUTESOUND", "void", [S("int32", "entityNum"), S("int32", "entchannel")]),
 ("CG_S_STARTSOUND", "void", [vin("origin, may be NULL"), S("int32", "entityNum"), S("int32", "entchannel"), S("sfxHandle_t", "sfx")]),
 ("CG_S_STARTLOCALSOUND", "void", [S("sfxHandle_t", "sfx"), S("int32", "channelNum")]),
 ("CG_S_CLEARLOOPINGSOUNDS", "void", []),
 ("CG_S_ADDLOOPINGSOUND", "void", [S("int32", "entityNum"), vin("origin"), vin("velocity"), S("sfxHandle_t", "sfx")]),
 ("CG_S_UPDATEENTITYPOSITION", "void", [S("int32", "entityNum"), vin("origin")]),
 ("CG_S_ADDREALLOOPINGSOUND", "void", [S("int32", "entityNum"), vin("origin"), vin("velocity"), S("sfxHandle_t", "sfx")], "aliased to S_AddLoopingSound in both engines."),
 ("CG_S_STOPLOOPINGSOUND", "void", [S("int32", "entityNum")]),
 ("CG_S_RESPATIALIZE", "void", [S("int32", "entityNum"), vin("origin"), ibuf("vec3_t[3]", 36, note="listener axis, 3 vec3"), S("int32", "inwater")]),
 ("CG_S_SHUTUP", "void", [S("qboolean", "shutup")]),
 ("CG_S_REGISTERSOUND", "handle", [istr(note="sound path")]),
 ("CG_S_STARTBACKGROUNDTRACK", "void", [istr(note="intro track"), istr(note="loop track"), S("int32", "bReturn")]),
 ("CG_S_UPDATEAMBIENTSET", "void", [istr(note="set name"), vin("origin")]),
 ("CG_AS_PARSESETS", "void", []),
 ("CG_AS_ADDPRECACHEENTRY", "void", [istr(note="name")]),
 ("CG_S_ADDLOCALSET", "scalar", [istr(note="name"), vin("listener_origin"), vin("origin"), S("int32", "entID"), S("int32", "time")]),
 ("CG_AS_GETBMODELSOUND", "scalar", [istr(note="name"), S("int32", "stage")]),
 ("CG_R_LOADWORLDMAP", "void", [istr(note="bsp name")]),
 ("CG_R_REGISTERMODEL", "handle", [istr(note="model path")]),
 ("CG_R_REGISTERSKIN", "handle", [istr(note="skin path")]),
 ("CG_R_REGISTERSHADER", "handle", [istr(note="shader name")]),
 ("CG_R_REGISTERSHADERNOMIP", "handle", [istr(note="shader name")]),
 ("CG_R_REGISTERFONT", "handle", [istr(note="font name")]),
 ("CG_R_FONT_STRLENPIXELS", "scalar", [istr(note="text"), S("int32", "fontHandle"), F("scale")]),
 ("CG_R_FONT_STRLENCHARS", "scalar", [istr(note="text")]),
 ("CG_R_FONT_STRHEIGHTPIXELS", "scalar", [S("int32", "fontHandle"), F("scale")]),
 ("CG_R_FONT_DRAWSTRING", "void", [S("int32", "ox"), S("int32", "oy"), istr(note="text"), ibuf("vec4_t", 16, note="rgba color"), S("int32", "setIndex"), S("int32", "iCharLimit"), F("scale")]),
 ("CG_LANGUAGE_ISASIAN", "scalar", []),
 ("CG_LANGUAGE_USESSPACES", "scalar", []),
 ("CG_ANYLANGUAGE_READCHARFROMSTRING", "scalar", [istr(note="text"), iobuf("int32", 4, note="byte position, read + advanced"), obuf("qboolean", 4, note="pbIsTrailingPunctuation OUT")]),
]

BLOCK100 = [
 ("CGAME_MEMSET", "void", [obuf("byte", len_arg=3, note="dest, filled"), S("int32", "value"), S("int32", "len")]),
 ("CGAME_MEMCPY", "void", [obuf("byte", len_arg=3, note="dest"), ibuf("byte", len_arg=3, note="src"), S("int32", "len")]),
 ("CGAME_STRNCPY", "scalar", [obuf("char", len_arg=3, note="dest"), istr(note="src"), S("int32", "len")], "returns dest ptr (args[1])."),
 ("CGAME_SIN", "float", [F("radians")]),
 ("CGAME_COS", "float", [F("radians")]),
 ("CGAME_ATAN2", "float", [F("y"), F("x")]),
 ("CGAME_SQRT", "float", [F("x")]),
 ("CGAME_MATRIXMULTIPLY", "void", [ibuf("vec3_t[3]", 36, note="in1"), ibuf("vec3_t[3]", 36, note="in2"), obuf("vec3_t[3]", 36, note="out")]),
 ("CGAME_ANGLEVECTORS", "void", [vin("angles"), vout("forward, may be NULL"), vout("right, may be NULL"), vout("up, may be NULL")]),
 ("CGAME_PERPENDICULARVECTOR", "void", [vout("dst"), vin("src")]),
 ("CGAME_FLOOR", "float", [F("x")]),
 ("CGAME_CEIL", "float", [F("x")]),
 ("CGAME_TESTPRINTINT", "void", [], "no-op in both engines (returns 0); args ignored."),
 ("CGAME_TESTPRINTFLOAT", "void", [], "no-op in both engines (returns 0); args ignored."),
 ("CGAME_ACOS", "float", [F("x")]),
 ("CGAME_ASIN", "float", [F("x")]),
]

BLOCK200 = [
 ("CG_R_CLEARSCENE", "void", []),
 ("CG_R_CLEARDECALS", "void", []),
 ("CG_R_ADDREFENTITYTOSCENE", "void", [ibuf("refEntity_t", note="render entity")]),
 ("CG_R_ADDPOLYTOSCENE", "void", [S("qhandle_t", "shader"), S("int32", "numVerts"), ibuf("polyVert_t", len_arg=2, note="verts, count args[2]")]),
 ("CG_R_ADDPOLYSTOSCENE", "void", [S("qhandle_t", "shader"), S("int32", "numVerts"), ibuf("polyVert_t", len_arg=2, note="verts, count = args[2]*args[4] (numVerts per poly x numPolys)"), S("int32", "numPolys")]),
 ("CG_R_ADDDECALTOSCENE", "void", [S("qhandle_t", "shader"), vin("origin"), vin("dir"), F("orientation"), F("r"), F("g"), F("b"), F("a"), S("qboolean", "alphaFade"), F("radius"), S("qboolean", "temporary")]),
 ("CG_R_LIGHTFORPOINT", "scalar", [vin("point"), vout("ambientLight"), vout("directedLight"), vout("lightDir")]),
 ("CG_R_ADDLIGHTTOSCENE", "void", [vin("origin"), F("intensity"), F("r"), F("g"), F("b")]),
 ("CG_R_ADDADDITIVELIGHTTOSCENE", "void", [vin("origin"), F("intensity"), F("r"), F("g"), F("b")]),
 ("CG_R_RENDERSCENE", "void", [ibuf("refdef_t", note="scene refdef")]),
 ("CG_R_SETCOLOR", "void", [ibuf("vec4_t", 16, note="rgba, may be NULL")]),
 ("CG_R_DRAWSTRETCHPIC", "void", [F("x"), F("y"), F("w"), F("h"), F("s1"), F("t1"), F("s2"), F("t2"), S("qhandle_t", "shader")]),
 ("CG_R_MODELBOUNDS", "void", [S("qhandle_t", "model"), vout("mins"), vout("maxs")]),
 ("CG_R_LERPTAG", "scalar", [obuf("orientation_t", note="tag orientation OUT"), S("qhandle_t", "model"), S("int32", "startFrame"), S("int32", "endFrame"), F("frac"), istr(note="tagName")]),
 ("CG_R_DRAWROTATEPIC", "void", [F("x"), F("y"), F("w"), F("h"), F("s1"), F("t1"), F("s2"), F("t2"), F("a"), S("qhandle_t", "shader")]),
 ("CG_R_DRAWROTATEPIC2", "void", [F("x"), F("y"), F("w"), F("h"), F("s1"), F("t1"), F("s2"), F("t2"), F("a"), S("qhandle_t", "shader")]),
 ("CG_R_SETRANGEFOG", "void", [F("range")], "DIVERGE: oracle sets tr.rangedFog directly; OpenJK re->SetRangedFog. same shape."),
 ("CG_R_SETREFRACTIONPROP", "void", [F("alpha"), F("stretch"), S("qboolean", "prePost"), S("qboolean", "negate")], "DIVERGE: oracle sets tr_distortion* globals; OpenJK re->SetRefractionProperties. same shape."),
 ("CG_R_REMAP_SHADER", "void", [istr(note="oldShader"), istr(note="newShader"), istr(note="timeOffset")]),
 ("CG_R_GET_LIGHT_STYLE", "void", [S("int32", "style"), obuf("byte", 4, note="color[4] OUT")]),
 ("CG_R_SET_LIGHT_STYLE", "void", [S("int32", "style"), S("int32", "color packed")]),
 ("CG_R_GET_BMODEL_VERTS", "void", [S("int32", "bmodelIndex"), obuf("vec3_t", 12, note="verts OUT, up to MAX_PATCH_PLANES; count returned by engine"), vout("normal")]),
 ("CG_R_GETDISTANCECULL", "void", [obuf("float", 4, note="distanceCull OUT")]),
 ("CG_R_GETREALRES", "void", [obuf("int32", 4, note="width OUT"), obuf("int32", 4, note="height OUT")]),
 ("CG_R_AUTOMAPELEVADJ", "void", [F("factor")]),
 ("CG_R_INITWIREFRAMEAUTO", "scalar", []),
 ("CG_FX_ADDLINE", "void", [vin("start"), vin("end"), F("size1"), F("size2"), F("sizeParm"), F("alpha1"), F("alpha2"), F("alphaParm"), vin("sRGB"), vin("eRGB"), F("rgbParm"), S("int32", "killTime"), S("qhandle_t", "shader"), S("int32", "flags")]),
 ("CG_GETGLCONFIG", "void", [obuf("glconfig_t", note="filled by engine")]),
 ("CG_GETGAMESTATE", "void", [obuf("gameState_t", note="filled by engine")]),
 ("CG_GETCURRENTSNAPSHOTNUMBER", "void", [obuf("int32", 4, note="snapshotNumber OUT"), obuf("int32", 4, note="serverTime OUT")]),
 ("CG_GETSNAPSHOT", "scalar", [S("int32", "snapshotNumber"), obuf("snapshot_t", note="filled by engine")]),
 ("CG_GETDEFAULTSTATE", "scalar", [S("int32", "index"), obuf("entityState_t", note="filled by engine")]),
 ("CG_GETSERVERCOMMAND", "scalar", [S("int32", "serverCommandNumber")], "sets up Cmd_Argv for the command; no buffer arg (cgame reads via CG_ARGV afterward)."),
 ("CG_GETCURRENTCMDNUMBER", "scalar", []),
 ("CG_GETUSERCMD", "scalar", [S("int32", "cmdNumber"), obuf("usercmd_t", note="filled by engine")]),
 ("CG_SETUSERCMDVALUE", "void", [S("int32", "userCmdValue"), F("sensitivityScale"), F("mPitchOverride"), F("mYawOverride"), F("mSensitivityOverride"), S("int32", "fpSel"), S("int32", "invenSel"), S("qboolean", "bUseFighterPitch")], "8 value args; oracle stashes args[8] in cl_bUseFighterPitch then calls CL_SetUserCmdValue with 7."),
 ("CG_SETCLIENTFORCEANGLE", "void", [S("int32", "time"), vin("angle")]),
 ("CG_SETCLIENTTURNEXTENT", "void", [], "no-op stub in both engines (returns 0); declared args unread."),
 ("CG_OPENUIMENU", "void", [S("int32", "menu")], "DIVERGE: oracle VM_Call(uivm, UI_SET_ACTIVE_MENU, args[1]); OpenJK CL_OpenUIMenu. same shape."),
 ("CG_TESTPRINTINT", "void", [], "no-op (not in dispatch; falls through to default assert in both). unreachable."),
 ("CG_TESTPRINTFLOAT", "void", [], "no-op (not in dispatch; falls through to default assert in both). unreachable."),
 ("CG_MEMORY_REMAINING", "scalar", []),
 ("CG_KEY_ISDOWN", "scalar", [S("int32", "keynum")]),
 ("CG_KEY_GETCATCHER", "scalar", []),
 ("CG_KEY_SETCATCHER", "void", [S("int32", "catcher")]),
 ("CG_KEY_GETKEY", "scalar", [istr(note="binding")]),
 ("CG_PC_ADD_GLOBAL_DEFINE", "scalar", [istr(note="define")]),
 ("CG_PC_LOAD_SOURCE", "scalar", [istr(note="filename")]),
 ("CG_PC_FREE_SOURCE", "scalar", [S("int32", "handle")]),
 ("CG_PC_READ_TOKEN", "scalar", [S("int32", "handle"), obuf("pc_token_t", note="token OUT")]),
 ("CG_PC_SOURCE_FILE_AND_LINE", "scalar", [S("int32", "handle"), obuf("char", note="filename OUT, caller MAX_QPATH buffer, no len arg"), obuf("int32", 4, note="line OUT")]),
 ("CG_PC_LOAD_GLOBAL_DEFINES", "scalar", [istr(note="filename")]),
 ("CG_PC_REMOVE_ALL_GLOBAL_DEFINES", "void", []),
 ("CG_S_STOPBACKGROUNDTRACK", "void", []),
 ("CG_REAL_TIME", "scalar", [obuf("qtime_t", note="qtime_s OUT")]),
 ("CG_SNAPVECTOR", "void", [iobuf("vec3_t", 12, note="rounded in place")]),
 ("CG_CIN_PLAYCINEMATIC", "scalar", [istr(note="arg0 name"), S("int32", "x"), S("int32", "y"), S("int32", "w"), S("int32", "h"), S("int32", "systemBits")]),
 ("CG_CIN_STOPCINEMATIC", "scalar", [S("int32", "handle")]),
 ("CG_CIN_RUNCINEMATIC", "scalar", [S("int32", "handle")]),
 ("CG_CIN_DRAWCINEMATIC", "void", [S("int32", "handle")]),
 ("CG_CIN_SETEXTENTS", "void", [S("int32", "handle"), S("int32", "x"), S("int32", "y"), S("int32", "w"), S("int32", "h")]),
 ("CG_GET_ENTITY_TOKEN", "scalar", [obuf("char", len_arg=2, note="token OUT"), S("int32", "size")]),
 ("CG_R_INPVS", "scalar", [vin("p1"), vin("p2"), ibuf("byte", note="PVS/area mask, engine reads")]),
 ("CG_FX_REGISTER_EFFECT", "handle", [istr(note="effect path")]),
 ("CG_FX_PLAY_EFFECT", "void", [istr(note="effect path"), vin("origin"), vin("fwd/axis"), S("int32", "vol"), S("int32", "rad")]),
 ("CG_FX_PLAY_ENTITY_EFFECT", "void", [], "DEAD: assert(0) in both engines; args unread. never call in replay."),
 ("CG_FX_PLAY_EFFECT_ID", "void", [S("int32", "id"), vin("origin"), vin("fwd/axis"), S("int32", "vol"), S("int32", "rad")]),
 ("CG_FX_PLAY_PORTAL_EFFECT_ID", "void", [S("int32", "id"), vin("origin"), vin("fwd/axis"), S("int32", "vol"), S("int32", "rad")]),
 ("CG_FX_PLAY_ENTITY_EFFECT_ID", "void", [S("int32", "id"), vin("origin"), ibuf("vec3_t[3]", 36, note="axis, 3 vec3"), S("int32", "boltNum"), S("int32", "entNum"), S("int32", "modelNum"), S("int32", "iLoopTime")]),
 ("CG_FX_PLAY_BOLTED_EFFECT_ID", "scalar", [S("int32", "id"), vin("origin"), g2h("ghoul2 handle via raw args[3] (NOT VMA); engine derefs *(CGhoul2Info_v*)args[3]"), S("int32", "boltNum"), S("int32", "entNum"), S("int32", "modelNum"), S("int32", "iLoopTime"), S("qboolean", "isRelative")]),
 ("CG_FX_ADD_SCHEDULED_EFFECTS", "void", [S("qboolean", "portal")]),
 ("CG_FX_INIT_SYSTEM", "scalar", [ibuf("refdef_t", note="refdef")]),
 ("CG_FX_SET_REFDEF", "void", [ibuf("refdef_t", note="refdef")]),
 ("CG_FX_FREE_SYSTEM", "scalar", []),
 ("CG_FX_ADJUST_TIME", "void", [S("int32", "time")]),
 ("CG_FX_DRAW_2D_EFFECTS", "void", [F("screenXScale"), F("screenYScale")]),
 ("CG_FX_RESET", "void", []),
 ("CG_FX_ADDPOLY", "void", [ibuf("addpolyArgStruct_t", note="poly args")]),
 ("CG_FX_ADDBEZIER", "void", [ibuf("addbezierArgStruct_t", note="bezier args")]),
 ("CG_FX_ADDPRIMITIVE", "void", [ibuf("effectTrailArgStruct_t", note="trail args")]),
 ("CG_FX_ADDSPRITE", "void", [ibuf("addspriteArgStruct_t", note="sprite args")]),
 ("CG_FX_ADDELECTRICITY", "void", [ibuf("addElectricityArgStruct_t", note="electricity args")]),
 ("CG_SP_GETSTRINGTEXTSTRING", "scalar", [istr(note="reference"), obuf("char", len_arg=3, note="localized text OUT"), S("int32", "size")]),
 ("CG_ROFF_CLEAN", "scalar", []),
 ("CG_ROFF_UPDATE_ENTITIES", "void", []),
 ("CG_ROFF_CACHE", "scalar", [istr(note="roff filename")]),
 ("CG_ROFF_PLAY", "scalar", [S("int32", "entID"), S("int32", "roffID"), S("qboolean", "doTranslation")]),
 ("CG_ROFF_PURGE_ENT", "scalar", [S("int32", "entID")]),
 ("CG_TRUEMALLOC", "void", [dptr("engine VM_Shifted_Alloc writes the allocated host ptr into *arg1; block is engine-owned"), S("int32", "size")]),
 ("CG_TRUEFREE", "void", [dptr("engine reads *arg1, frees it, nulls the slot")]),
 ("CG_G2_LISTSURFACES", "void", [g2h("single CGhoul2Info* (oracle casts args[1] raw)")]),
 ("CG_G2_LISTBONES", "void", [g2h("single CGhoul2Info*"), S("int32", "frame")]),
 ("CG_G2_SETMODELS", "void", [g2h(), ibuf("qhandle_t", note="modelList array, NUL/index-terminated"), ibuf("qhandle_t", note="skinList array")]),
 ("CG_G2_HAVEWEGHOULMODELS", "scalar", [g2h()]),
 ("CG_G2_GETBOLT", "scalar", [g2h(), S("int32", "modelIndex"), S("int32", "boltIndex"), obuf("mdxaBone_t", 48, note="bolt matrix OUT"), vin("angles"), vin("position"), S("int32", "frameNum"), iobuf("qhandle_t", note="modelList"), vin("scale")]),
 ("CG_G2_GETBOLT_NOREC", "scalar", [g2h(), S("int32", "modelIndex"), S("int32", "boltIndex"), obuf("mdxaBone_t", 48, note="bolt matrix OUT"), vin("angles"), vin("position"), S("int32", "frameNum"), iobuf("qhandle_t", note="modelList"), vin("scale")], "oracle sets gG2_GBMNoReconstruct before same call."),
 ("CG_G2_GETBOLT_NOREC_NOROT", "scalar", [g2h(), S("int32", "modelIndex"), S("int32", "boltIndex"), obuf("mdxaBone_t", 48, note="bolt matrix OUT"), vin("angles"), vin("position"), S("int32", "frameNum"), iobuf("qhandle_t", note="modelList"), vin("scale")], "oracle sets gG2_GBMUseSPMethod before same call."),
 ("CG_G2_INITGHOUL2MODEL", "scalar", [dptr("CGhoul2Info_v** slot; engine allocates the instance vector on first use and writes the host ptr back into *arg1 (in/out)"), istr(note="model name"), S("int32", "modelIndex"), S("qhandle_t", "customSkin"), S("qhandle_t", "customShader"), S("int32", "modelFlags"), S("int32", "lodBias")]),
 ("CG_G2_SETSKIN", "scalar", [g2h(), S("int32", "modelIndex"), S("qhandle_t", "customSkin"), S("qhandle_t", "renderSkin")]),
 ("CG_G2_COLLISIONDETECT", "void", [obuf("CollisionRecord_t", note="collRecMap OUT array, MAX_G2_COLLISIONS=16"), g2h(), vin("angles"), vin("position"), S("int32", "frameNumber"), S("int32", "entNum"), vout("rayStart"), vout("rayEnd"), vin("scale"), S("int32", "traceFlags"), S("int32", "useLod"), F("fRadius")], "rayStart/rayEnd (args 7,8) are passed in and may be adjusted; treat as in vec3."),
 ("CG_G2_COLLISIONDETECTCACHE", "void", [obuf("CollisionRecord_t", note="collRecMap OUT array"), g2h(), vin("angles"), vin("position"), S("int32", "frameNumber"), S("int32", "entNum"), vin("rayStart"), vin("rayEnd"), vin("scale"), S("int32", "traceFlags"), S("int32", "useLod"), F("fRadius")]),
 ("CG_G2_ANGLEOVERRIDE", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="boneName"), vin("angles"), S("int32", "flags"), S("Eorientations", "up"), S("Eorientations", "right"), S("Eorientations", "forward"), iobuf("qhandle_t", note="modelList"), S("int32", "blendTime"), S("int32", "currentTime")]),
 ("CG_G2_CLEANMODELS", "void", [dptr("CGhoul2Info_v** slot; engine frees the vector and nulls the slot (in/out)")]),
 ("CG_G2_PLAYANIM", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="boneName"), S("int32", "startFrame"), S("int32", "endFrame"), S("int32", "flags"), F("animSpeed"), S("int32", "blendTime"), F("setFrame"), S("int32", "currentTime")]),
 ("CG_G2_GETBONEANIM", "scalar", [g2h(), istr(note="boneName"), S("int32", "currentTime"), obuf("float", 4, note="currentFrame OUT"), obuf("int32", 4, note="startFrame OUT"), obuf("int32", 4, note="endFrame OUT"), obuf("int32", 4, note="flags OUT"), obuf("float", 4, note="animSpeed OUT"), obuf("int32", 4, note="modelList OUT"), S("int32", "modelIndex")]),
 ("CG_G2_GETBONEFRAME", "scalar", [g2h(), istr(note="boneName"), S("int32", "currentTime"), obuf("float", 4, note="currentFrame OUT"), obuf("int32", 4, note="modelList OUT"), S("int32", "modelIndex")], "trimmed GetBoneAnim: engine discards startFrame/endFrame/flags/animSpeed internally."),
 ("CG_G2_GETGLANAME", "void", [g2h(), S("int32", "modelIndex"), obuf("char", note="GLA name OUT, caller buffer, no len (strcpy)")]),
 ("CG_G2_COPYGHOUL2INSTANCE", "scalar", [g2h("source instance"), g2h("dest instance"), S("int32", "modelIndex")]),
 ("CG_G2_COPYSPECIFICGHOUL2MODEL", "void", [g2h("source"), S("int32", "modelFrom"), g2h("dest"), S("int32", "modelTo")]),
 ("CG_G2_DUPLICATEGHOUL2INSTANCE", "void", [g2h("source instance"), dptr("CGhoul2Info_v** dest slot; engine allocates a copy and writes host ptr into *arg2")]),
 ("CG_G2_HASGHOUL2MODELONINDEX", "scalar", [dptr("CGhoul2Info_v** slot; engine reads the ptr and indexes it"), S("int32", "modelIndex")]),
 ("CG_G2_REMOVEGHOUL2MODEL", "scalar", [dptr("CGhoul2Info_v** slot; engine removes a model and may reallocate, writing back (in/out)"), S("int32", "modelIndex")]),
 ("CG_G2_SKINLESSMODEL", "scalar", [g2h(), S("int32", "modelIndex")]),
 ("CG_G2_GETNUMGOREMARKS", "scalar", [g2h(), S("int32", "modelIndex")], "returns 0 unless engine built with _G2_GORE."),
 ("CG_G2_ADDSKINGORE", "void", [g2h(), ibuf("SSkinGoreData", note="gore data")], "no-op unless _G2_GORE."),
 ("CG_G2_CLEARSKINGORE", "void", [g2h()], "no-op unless _G2_GORE."),
 ("CG_G2_SIZE", "scalar", [g2h()]),
 ("CG_G2_ADDBOLT", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="boneName")]),
 ("CG_G2_ATTACHENT", "scalar", [obuf("int32", 4, note="boltInfo OUT"), g2h("ghlInfoTo"), S("int32", "toBoltIndex"), S("int32", "entNum"), S("int32", "toModelNum")]),
 ("CG_G2_SETBOLTON", "void", [g2h(), S("int32", "boltIndex"), S("int32", "flags")]),
 ("CG_G2_SETROOTSURFACE", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="surfaceName")]),
 ("CG_G2_SETSURFACEONOFF", "scalar", [g2h(), istr(note="surfaceName"), S("int32", "flags")]),
 ("CG_G2_SETNEWORIGIN", "scalar", [g2h(), S("int32", "boltIndex")]),
 ("CG_G2_DOESBONEEXIST", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="boneName")]),
 ("CG_G2_GETSURFACERENDERSTATUS", "scalar", [g2h(), S("int32", "modelIndex"), istr(note="surfaceName")]),
 ("CG_G2_GETTIME", "scalar", []),
 ("CG_G2_SETTIME", "void", [S("int32", "time"), S("int32", "clock")]),
 ("CG_G2_ABSURDSMOOTHING", "void", [g2h(), S("qboolean", "status")]),
 ("CG_G2_SETRAGDOLL", "void", [g2h(), ibuf("sharedRagDollParams_t", note="ragdoll params; NULL resets the ragdoll")]),
 ("CG_G2_ANIMATEG2MODELS", "void", [g2h(), S("int32", "acurTime"), ibuf("sharedRagDollUpdateParams_t", note="update params; NULL = early return")]),
 ("CG_G2_RAGPCJCONSTRAINT", "scalar", [g2h(), istr(note="boneName"), vin("min"), vin("max")]),
 ("CG_G2_RAGPCJGRADIENTSPEED", "scalar", [g2h(), istr(note="boneName"), F("speed")]),
 ("CG_G2_RAGEFFECTORGOAL", "scalar", [g2h(), istr(note="boneName"), vin("pos, may be NULL")]),
 ("CG_G2_GETRAGBONEPOS", "scalar", [g2h(), istr(note="boneName"), vout("pos OUT"), vin("entAngles"), vin("entPos"), vin("entScale")]),
 ("CG_G2_RAGEFFECTORKICK", "scalar", [g2h(), istr(note="boneName"), vin("velocity")]),
 ("CG_G2_RAGFORCESOLVE", "scalar", [g2h(), S("qboolean", "force")]),
 ("CG_G2_SETBONEIKSTATE", "scalar", [g2h(), S("int32", "time"), istr(note="boneName"), S("int32", "ikState"), ibuf("sharedSetBoneIKStateParams_t", note="params, may be NULL")]),
 ("CG_G2_IKMOVE", "scalar", [g2h(), S("int32", "time"), ibuf("sharedIKMoveParams_t", note="move params")]),
 ("CG_G2_REMOVEBONE", "scalar", [g2h(), istr(note="boneName"), S("int32", "modelIndex")]),
 ("CG_G2_ATTACHINSTANCETOENTNUM", "void", [g2h(), S("int32", "entNum"), S("qboolean", "server")]),
 ("CG_G2_CLEARATTACHEDINSTANCE", "void", [S("int32", "entNum")]),
 ("CG_G2_CLEANENTATTACHMENTS", "void", []),
 ("CG_G2_OVERRIDESERVER", "scalar", [g2h()]),
 ("CG_G2_GETSURFACENAME", "void", [g2h(), S("int32", "surfNumber"), S("int32", "modelIndex"), obuf("char", note="surface name OUT, caller buffer, no len (strcpy)")]),
 ("CG_SET_SHARED_BUFFER", "void", [{"kind": "retained_ptr", "type": "char*", "size_of": 2048, "note": "engine STORES the pointer (cl.mSharedMemory / RegisterSharedMemory) and reads through it during later G2/FX traps and VM calls; buffer is MAX_CG_SHARED_BUFFER_SIZE = 2048 (cg_public.h:593). replay must keep a live 2048-byte region and re-point the engine at it, not copy-at-call."}], "the one engine-retained pointer; model specially."),
 ("CG_CM_REGISTER_TERRAIN", "scalar", [istr(note="terrain config")], "DIVERGE: OpenJK returns 0 (RMG stripped); oracle CM_RegisterTerrain(...)->GetTerrainId(). shape recorded from oracle; OpenJK ignores the arg."),
 ("CG_RMG_INIT", "void", [S("int32", "count"), istr(note="terrain string")], "DIVERGE: OpenJK returns 0 (no-op); oracle runs RM_CreateRandomModels(args[1], VMA(2)). shape recorded from oracle; OpenJK reads neither arg."),
 ("CG_RE_INIT_RENDERER_TERRAIN", "void", [istr(note="info string")], "DIVERGE: OpenJK returns 0 (no-op); oracle RE_InitRendererTerrain(VMA(1)). OpenJK ignores the arg."),
 ("CG_R_WEATHER_CONTENTS_OVERRIDE", "void", [S("int32", "contents")], "no-op in both engines (assignment commented out); arg unread."),
 ("CG_R_WORLDEFFECTCOMMAND", "void", [istr(note="weather command")]),
 ("CG_WE_ADDWEATHERZONE", "void", [vin("mins"), vin("maxs")]),
]

# --- assemble ------------------------------------------------------------
# OpenJK cl_cgameapi.cpp case-line map (ground truth cite).
OPENJK_LINE = {
 "CGAME_MEMSET":839,"CGAME_MEMCPY":843,"CGAME_STRNCPY":847,"CGAME_SIN":851,"CGAME_COS":854,
 "CGAME_ATAN2":857,"CGAME_SQRT":860,"CGAME_MATRIXMULTIPLY":863,"CGAME_ANGLEVECTORS":867,
 "CGAME_PERPENDICULARVECTOR":871,"CGAME_FLOOR":875,"CGAME_CEIL":878,"CGAME_TESTPRINTINT":881,
 "CGAME_TESTPRINTFLOAT":884,"CGAME_ACOS":887,"CGAME_ASIN":890,
 "CG_PRINT":893,"CG_ERROR":897,"CG_MILLISECONDS":901,"CG_PRECISIONTIMER_START":906,
 "CG_PRECISIONTIMER_END":910,"CG_CVAR_REGISTER":913,"CG_CVAR_UPDATE":917,"CG_CVAR_SET":921,
 "CG_CVAR_VARIABLESTRINGBUFFER":925,"CG_CVAR_GETHIDDENVALUE":929,"CG_ARGC":932,"CG_ARGV":935,
 "CG_ARGS":939,"CG_FS_FOPENFILE":943,"CG_FS_READ":946,"CG_FS_WRITE":950,"CG_FS_FCLOSEFILE":954,
 "CG_FS_GETFILELIST":958,"CG_SENDCONSOLECOMMAND":961,"CG_ADDCOMMAND":965,"CG_REMOVECOMMAND":969,
 "CG_SENDCLIENTCOMMAND":973,"CG_UPDATESCREEN":977,"CG_CM_LOADMAP":986,"CG_CM_NUMINLINEMODELS":990,
 "CG_CM_INLINEMODEL":993,"CG_CM_TEMPBOXMODEL":996,"CG_CM_TEMPCAPSULEMODEL":999,"CG_CM_POINTCONTENTS":1002,
 "CG_CM_TRANSFORMEDPOINTCONTENTS":1005,"CG_CM_BOXTRACE":1008,"CG_CM_CAPSULETRACE":1012,
 "CG_CM_TRANSFORMEDBOXTRACE":1016,"CG_CM_TRANSFORMEDCAPSULETRACE":1020,"CG_CM_MARKFRAGMENTS":1024,
 "CG_S_GETVOICEVOLUME":1027,"CG_S_MUTESOUND":1030,"CG_S_STARTSOUND":1034,"CG_S_STARTLOCALSOUND":1038,
 "CG_S_CLEARLOOPINGSOUNDS":1042,"CG_S_ADDLOOPINGSOUND":1046,"CG_S_ADDREALLOOPINGSOUND":1050,
 "CG_S_STOPLOOPINGSOUND":1054,"CG_S_UPDATEENTITYPOSITION":1058,"CG_S_RESPATIALIZE":1062,
 "CG_S_SHUTUP":1066,"CG_S_REGISTERSOUND":1070,"CG_S_STARTBACKGROUNDTRACK":1073,"CG_S_UPDATEAMBIENTSET":1077,
 "CG_AS_PARSESETS":1081,"CG_AS_ADDPRECACHEENTRY":1085,"CG_S_ADDLOCALSET":1089,"CG_AS_GETBMODELSOUND":1092,
 "CG_R_LOADWORLDMAP":1095,"CG_R_REGISTERMODEL":1099,"CG_R_REGISTERSKIN":1102,"CG_R_REGISTERSHADER":1105,
 "CG_R_REGISTERSHADERNOMIP":1108,"CG_R_REGISTERFONT":1111,"CG_R_FONT_STRLENPIXELS":1114,
 "CG_R_FONT_STRLENCHARS":1117,"CG_R_FONT_STRHEIGHTPIXELS":1120,"CG_R_FONT_DRAWSTRING":1123,
 "CG_LANGUAGE_ISASIAN":1127,"CG_LANGUAGE_USESSPACES":1130,"CG_ANYLANGUAGE_READCHARFROMSTRING":1133,
 "CG_R_CLEARSCENE":1136,"CG_R_CLEARDECALS":1140,"CG_R_ADDREFENTITYTOSCENE":1144,"CG_R_ADDPOLYTOSCENE":1148,
 "CG_R_ADDPOLYSTOSCENE":1152,"CG_R_ADDDECALTOSCENE":1156,"CG_R_LIGHTFORPOINT":1160,"CG_R_ADDLIGHTTOSCENE":1163,
 "CG_R_ADDADDITIVELIGHTTOSCENE":1167,"CG_R_RENDERSCENE":1171,"CG_R_SETCOLOR":1175,"CG_R_DRAWSTRETCHPIC":1179,
 "CG_R_MODELBOUNDS":1183,"CG_R_LERPTAG":1187,"CG_R_DRAWROTATEPIC":1190,"CG_R_DRAWROTATEPIC2":1194,
 "CG_R_SETRANGEFOG":1198,"CG_R_SETREFRACTIONPROP":1202,"CG_GETGLCONFIG":1206,"CG_GETGAMESTATE":1210,
 "CG_GETCURRENTSNAPSHOTNUMBER":1214,"CG_GETSNAPSHOT":1218,"CG_GETDEFAULTSTATE":1221,"CG_GETSERVERCOMMAND":1224,
 "CG_GETCURRENTCMDNUMBER":1227,"CG_GETUSERCMD":1230,"CG_SETUSERCMDVALUE":1233,"CG_SETCLIENTFORCEANGLE":1237,
 "CG_SETCLIENTTURNEXTENT":1241,"CG_OPENUIMENU":1244,"CG_MEMORY_REMAINING":1248,"CG_KEY_ISDOWN":1251,
 "CG_KEY_GETCATCHER":1254,"CG_KEY_SETCATCHER":1257,"CG_KEY_GETKEY":1261,"CG_PC_ADD_GLOBAL_DEFINE":1264,
 "CG_PC_LOAD_SOURCE":1267,"CG_PC_FREE_SOURCE":1270,"CG_PC_READ_TOKEN":1273,"CG_PC_SOURCE_FILE_AND_LINE":1276,
 "CG_PC_LOAD_GLOBAL_DEFINES":1279,"CG_PC_REMOVE_ALL_GLOBAL_DEFINES":1282,"CG_S_STOPBACKGROUNDTRACK":1286,
 "CG_REAL_TIME":1290,"CG_SNAPVECTOR":1293,"CG_CIN_PLAYCINEMATIC":1297,"CG_CIN_STOPCINEMATIC":1300,
 "CG_CIN_RUNCINEMATIC":1303,"CG_CIN_DRAWCINEMATIC":1306,"CG_CIN_SETEXTENTS":1310,"CG_R_REMAP_SHADER":1314,
 "CG_R_GET_LIGHT_STYLE":1318,"CG_R_SET_LIGHT_STYLE":1322,"CG_R_GET_BMODEL_VERTS":1326,"CG_R_GETDISTANCECULL":1330,
 "CG_R_GETREALRES":1337,"CG_R_AUTOMAPELEVADJ":1345,"CG_R_INITWIREFRAMEAUTO":1349,"CG_GET_ENTITY_TOKEN":1352,
 "CG_R_INPVS":1355,"CG_FX_ADDLINE":1359,"CG_FX_REGISTER_EFFECT":1365,"CG_FX_PLAY_EFFECT":1368,
 "CG_FX_PLAY_ENTITY_EFFECT":1372,"CG_FX_PLAY_EFFECT_ID":1376,"CG_FX_PLAY_PORTAL_EFFECT_ID":1380,
 "CG_FX_PLAY_ENTITY_EFFECT_ID":1384,"CG_FX_PLAY_BOLTED_EFFECT_ID":1388,"CG_FX_ADD_SCHEDULED_EFFECTS":1391,
 "CG_FX_DRAW_2D_EFFECTS":1395,"CG_FX_INIT_SYSTEM":1399,"CG_FX_SET_REFDEF":1402,"CG_FX_FREE_SYSTEM":1406,
 "CG_FX_ADJUST_TIME":1409,"CG_FX_RESET":1413,"CG_FX_ADDPOLY":1417,"CG_FX_ADDBEZIER":1420,"CG_FX_ADDPRIMITIVE":1423,
 "CG_FX_ADDSPRITE":1426,"CG_FX_ADDELECTRICITY":1429,"CG_ROFF_CLEAN":1452,"CG_ROFF_UPDATE_ENTITIES":1455,
 "CG_ROFF_CACHE":1459,"CG_ROFF_PLAY":1462,"CG_ROFF_PURGE_ENT":1465,"CG_TRUEMALLOC":1468,"CG_TRUEFREE":1472,
 "CG_G2_LISTSURFACES":1476,"CG_G2_LISTBONES":1480,"CG_G2_HAVEWEGHOULMODELS":1484,"CG_G2_SETMODELS":1487,
 "CG_G2_GETBOLT":1491,"CG_G2_GETBOLT_NOREC":1494,"CG_G2_GETBOLT_NOREC_NOROT":1497,"CG_G2_INITGHOUL2MODEL":1500,
 "CG_G2_SETSKIN":1503,"CG_G2_COLLISIONDETECT":1506,"CG_G2_COLLISIONDETECTCACHE":1510,"CG_G2_ANGLEOVERRIDE":1514,
 "CG_G2_CLEANMODELS":1517,"CG_G2_PLAYANIM":1521,"CG_G2_GETBONEANIM":1524,"CG_G2_GETBONEFRAME":1527,
 "CG_G2_GETGLANAME":1530,"CG_G2_COPYGHOUL2INSTANCE":1534,"CG_G2_COPYSPECIFICGHOUL2MODEL":1537,
 "CG_G2_DUPLICATEGHOUL2INSTANCE":1541,"CG_G2_HASGHOUL2MODELONINDEX":1545,"CG_G2_REMOVEGHOUL2MODEL":1548,
 "CG_G2_SKINLESSMODEL":1551,"CG_G2_GETNUMGOREMARKS":1554,"CG_G2_ADDSKINGORE":1557,"CG_G2_CLEARSKINGORE":1561,
 "CG_G2_SIZE":1565,"CG_G2_ADDBOLT":1568,"CG_G2_ATTACHENT":1571,"CG_G2_SETBOLTON":1574,"CG_G2_SETROOTSURFACE":1578,
 "CG_G2_SETSURFACEONOFF":1581,"CG_G2_SETNEWORIGIN":1584,"CG_G2_DOESBONEEXIST":1587,"CG_G2_GETSURFACERENDERSTATUS":1590,
 "CG_G2_GETTIME":1593,"CG_G2_SETTIME":1596,"CG_G2_ABSURDSMOOTHING":1600,"CG_G2_SETRAGDOLL":1605,
 "CG_G2_ANIMATEG2MODELS":1609,"CG_G2_RAGPCJCONSTRAINT":1614,"CG_G2_RAGPCJGRADIENTSPEED":1617,
 "CG_G2_RAGEFFECTORGOAL":1620,"CG_G2_GETRAGBONEPOS":1623,"CG_G2_RAGEFFECTORKICK":1626,"CG_G2_RAGFORCESOLVE":1629,
 "CG_G2_SETBONEIKSTATE":1632,"CG_G2_IKMOVE":1635,"CG_G2_REMOVEBONE":1638,"CG_G2_ATTACHINSTANCETOENTNUM":1641,
 "CG_G2_CLEARATTACHEDINSTANCE":1645,"CG_G2_CLEANENTATTACHMENTS":1649,"CG_G2_OVERRIDESERVER":1653,
 "CG_G2_GETSURFACENAME":1656,"CG_SP_GETSTRINGTEXTSTRING":1660,"CG_SET_SHARED_BUFFER":1663,
 "CG_CM_REGISTER_TERRAIN":1667,"CG_RMG_INIT":1670,"CG_RE_INIT_RENDERER_TERRAIN":1673,
 "CG_R_WEATHER_CONTENTS_OVERRIDE":1676,"CG_R_WORLDEFFECTCOMMAND":1680,"CG_WE_ADDWEATHERZONE":1684,
}
# CG_TESTPRINTINT / CG_TESTPRINTFLOAT have no case in the dispatch (fall through
# to the default assert); cite the cg_public.h declaration instead.
CGPUBLIC = "oracle/codemp/cgame/cg_public.h"

def entries_with_numbers():
    out = []
    # block 0..65
    num = 0
    for e in BLOCK0:
        out.append((num, e)); num += 1
    # block 100..115
    num = 100
    for e in BLOCK100:
        out.append((num, e)); num += 1
    # block 200..350
    num = 200
    for e in BLOCK200:
        out.append((num, e)); num += 1
    return out

def cite_for(name):
    if name in OPENJK_LINE:
        return f"{OPENJK}:{OPENJK_LINE[name]}"
    return f"{CGPUBLIC} (declared, no dispatch case)"

def build():
    traps = []
    for num, e in entries_with_numbers():
        name, ret, args = e[0], e[1], e[2]
        note = e[3] if len(e) > 3 else None
        entry = {"num": num, "name": name, "ret": ret, "args": args, "cite": cite_for(name)}
        if note:
            entry["note"] = note
        traps.append(entry)
    return {
        "schema": "cgame-trap-shapes/1",
        "engine": "OpenJK codemp client (openjk.app) — CL_CgameSystemCalls",
        "numbering": "cgameImport_t (oracle/codemp/cgame/cg_public.h), byte-identical to OpenJK cgameImportLegacy_e",
        "arg_kinds": ["scalar", "in_str", "in_buf", "out_buf", "inout_buf", "double_ptr", "retained_ptr"],
        "ret_kinds": ["void", "scalar", "handle", "float"],
        "note": "arg index N in `args[]` maps to VMA(N+1)/args[N+1] in the dispatch (args[0] is the trap number). size_of is the engine-native (LP64) sizeof of the named type in bytes where fixed; len_arg names the 0-based args[] index holding the byte/element count. See README.md.",
        "count": len(traps),
        "traps": traps,
    }

if __name__ == "__main__":
    print(json.dumps(build(), indent=2))
