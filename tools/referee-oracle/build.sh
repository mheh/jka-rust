#!/bin/sh
# referee-oracle: build Raven's UNMODIFIED jampgame (QAGAME) game module from the
# oracle source as a loadable dylib — `build/liboraclejampgame.dylib`.
#
# This is the reference module for the Stage-R referee differential harness: the
# oracle DLL and our Rust `jampgame` cdylib are driven over identical inputs and
# byte-diffed. Phase 1 (this script + tests/oracle_smoke.rs) only proves the
# artifact builds and our mock-engine harness can drive its full lifecycle.
#
# The oracle tree (oracle/**) is NEVER edited. Every game/*.c is compiled
# straight from a throwaway COPY under build/src/, patched in-place by the
# python step below: Raven's OWN `#define qboolean int` is activated, SnapVector
# is retargeted to retail-rounding rint(), float-arg libm calls (atan2f,
# sqrtf, ...) are forced back to the double family MSVC's C compile promoted
# them to (so this C++-compiled oracle matches retail/our f64 parity bar), and
# vmMain's arg/return words are widened int -> intptr_t so the dylib survives
# pointer-carrying engine->game calls on LP64 hosts.
#
# Requires a real GCC (Homebrew `gcc`): the unmodified 32-bit-era C++ source needs
# `-fpermissive` to accept its `FOFS` pointer->int casts on a 64-bit host, which
# Apple clang does not provide (`-fpermissive` is a silent no-op there). Install
# with `brew install gcc`. See README.md for the full rationale.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle
G="$ORACLE/codemp/game"
SHIM="$(pwd)/shim/oracle_shim.h"

# --- pick a real GCC (NOT Apple clang) ----------------------------------------
CXX="${CXX:-}"
if [ -z "$CXX" ]; then
	for c in g++-16 g++-15 g++-14 g++-13 g++-12 g++; do
		if command -v "$c" >/dev/null 2>&1; then CXX="$c"; break; fi
		if [ -x "/opt/homebrew/bin/$c" ]; then CXX="/opt/homebrew/bin/$c"; break; fi
	done
fi
if [ -z "$CXX" ] || ! command -v "$CXX" >/dev/null 2>&1; then
	echo "error: no real GCC found (need g++-1x from 'brew install gcc')." >&2
	echo "       Apple clang cannot build the unmodified source (no -fpermissive)." >&2
	echo "       Set CXX=/path/to/g++ to override." >&2
	exit 1
fi
case "$("$CXX" --version 2>&1 | head -1)" in
	*clang*) echo "error: CXX ($CXX) is Apple clang; a real GCC is required." >&2; exit 1;;
esac
echo "referee-oracle: using $CXX"

# --- lay down a throwaway build tree (oracle is never touched) -----------------
rm -rf build
mkdir -p build/src build/obj
cp -R "$ORACLE/codemp" build/src/codemp
mkdir -p build/src/ui
cp "$ORACLE/ui/menudef.h" build/src/ui/   # g_cmds.c: ../../ui/menudef.h (voice chats)

# The SOLE source change: activate Raven's own `#define qboolean int` — the
# define it already ships under `#ifdef _XBOX` ("don't want strict type checking
# on the qboolean", q_shared.h:353-355). enum qboolean and int are both 4 bytes,
# values 0/1, so this is ABI-identical; it only lets the C++11+-strict `bool ==`
# results assign to qboolean the way MSVC allowed in the original build.
python3 - <<'PY'
p = "build/src/codemp/game/q_shared.h"
s = open(p).read()
old = "typedef enum {qfalse, qtrue}\tqboolean;\n#ifdef _XBOX\n#define\tqboolean\tint"
new = ("typedef enum {qfalse, qtrue}\tqboolean;\n"
       "#if 1 /* referee-oracle: activate Raven's own XBOX qboolean=int (ABI-identical) */\n"
       "#define\tqboolean\tint")
assert old in s, "q_shared.h qboolean block not found — oracle changed?"
s = s.replace(old, new, 1)

# retail-win32 SnapVector (x87 fld/fistp) is the port's parity bar — see
# crates/mp/game/src/bg_misc.rs's round_ties_even. The ONLY place -D__linux__
# diverges from that is this macro: it TRUNCATES via (int) casts instead of
# rounding. Patch the harness copy's __linux__ branch to rint(), which under
# the default FP environment is round-to-nearest-even — exactly fistp's
# semantics — so the oracle dylib built here matches retail-win32 rounding
# instead of the Linux-only truncation. (math.h is already included above.)
old_snap = ("#ifdef __linux__\n"
            "#define\tSnapVector(v) {v[0]=((int)(v[0]));v[1]=((int)(v[1]));v[2]=((int)(v[2]));}\n")
new_snap = ("#ifdef __linux__\n"
            "#define\tSnapVector(v) {v[0]=(float)rint(v[0]);v[1]=(float)rint(v[1]);v[2]=(float)rint(v[2]);}\n")
assert old_snap in s, "q_shared.h __linux__ SnapVector macro not found — oracle changed?"
s = s.replace(old_snap, new_snap, 1)

# retail (MSVC, compiled as C) promotes float args to double and calls the
# double libm family; compiling this tree as C++ instead resolves the float
# overloads (atan2f, sqrtf, ...), a 1-ULP-class divergence from the retail
# parity bar our Rust port targets (see docs/decisions.md). Force the double
# path with function-like macros immediately after <math.h> — the macro name
# is not re-expanded inside its own expansion, so these are safe self-refs.
old_mathinc = "#include <math.h>\n"
new_mathinc = (
    "#include <math.h>\n"
    "/* referee-oracle: retail (MSVC, compiled as C) promotes float args to double\n"
    "   and calls double libm; compiling as C++ resolves float overloads (atan2f,\n"
    "   sqrtf, ...) instead — a 1-ULP-class divergence from the retail parity bar.\n"
    "   Function-like macros force the double path; the macro name is not\n"
    "   re-expanded inside its own expansion, so these are safe self-references. */\n"
    "#define sqrt(x) sqrt((double)(x))\n"
    "#define sin(x) sin((double)(x))\n"
    "#define cos(x) cos((double)(x))\n"
    "#define tan(x) tan((double)(x))\n"
    "#define asin(x) asin((double)(x))\n"
    "#define acos(x) acos((double)(x))\n"
    "#define atan(x) atan((double)(x))\n"
    "#define atan2(y,x) atan2((double)(y),(double)(x))\n"
    "#define pow(x,y) pow((double)(x),(double)(y))\n"
    "#define fmod(x,y) fmod((double)(x),(double)(y))\n"
    "#define exp(x) exp((double)(x))\n"
    "#define log(x) log((double)(x))\n"
)
assert old_mathinc in s, "q_shared.h '#include <math.h>' not found — oracle changed?"
s = s.replace(old_mathinc, new_mathinc, 1)

# Caveat: q_shared.h separately re-declares `double fmod(double,double);` as its
# own prototype (line ~1603) — the fmod(...) macro above expands that
# declarator too ("fmod" followed by "(" is enough), producing an illegal
# `fmod((double)(double x), ...)` redeclaration. Bracket just this one
# declaration with #undef/#define so the macro stays live for every actual
# call site but not for this redundant prototype.
old_fmodproto = "double\tfmod( double x, double y );\n"
new_fmodproto = (
    "#undef fmod /* referee-oracle: shield this prototype from the double-promotion macro above */\n"
    "double\tfmod( double x, double y );\n"
    "#define fmod(x,y) fmod((double)(x),(double)(y))\n"
)
assert old_fmodproto in s, "q_shared.h fmod() prototype not found — oracle changed?"
s = s.replace(old_fmodproto, new_fmodproto, 1)

# retail links the MSVC CRT rand()/srand() into the native module (bg_lib.c is
# ExcludedFromBuild in every JK2_game.vcproj win32 config AND its rand/srand
# sit under #ifdef Q3_VM, bg_lib.c:754 — the 69069 LCG is QVM-only). This
# host's libc rand() is a different LCG, so bot-AI rolls (chat/aim/camping)
# diverge from retail (found by the lockstep referee, 2026-07-14: first
# divergent bot decision at the first chat gate). Route rand/srand to a
# retail-exact MSVC-semantics clone: 32-bit holdrand (unsigned long is 32-bit
# on win32/win64 alike), init 1, next = holdrand*214013+2531011, return
# (holdrand>>16)&0x7fff. State is a single shared variable defined in the
# q_math.c copy below.
old_limitsinc = "#include <limits.h>\n"
new_limitsinc = (
    "#include <limits.h>\n"
    "/* referee-oracle: retail-win32 rand()/srand() are the MSVC CRT LCG; this\n"
    "   host's libc differs. See tools/referee-oracle/build.sh for rationale. */\n"
    "extern unsigned int jka_msvc_holdrand;\n"
    "static inline int jka_msvc_rand(void) {\n"
    "    jka_msvc_holdrand = jka_msvc_holdrand * 214013u + 2531011u;\n"
    "    return (int)((jka_msvc_holdrand >> 16) & 0x7fff);\n"
    "}\n"
    "static inline void jka_msvc_srand(unsigned int seed) { jka_msvc_holdrand = seed; }\n"
    "#define rand() jka_msvc_rand()\n"
    "#define srand(s) jka_msvc_srand((unsigned int)(s))\n"
)
assert old_limitsinc in s, "q_shared.h '#include <limits.h>' not found — oracle changed?"
s = s.replace(old_limitsinc, new_limitsinc, 1)

open(p, "w").write(s)

# The MSVC-rand state (one shared definition; declared extern in q_shared.h).
p = "build/src/codemp/game/q_math.c"
s = open(p).read()
s += "\n/* referee-oracle: MSVC CRT rand() state (see q_shared.h patch). */\nunsigned int jka_msvc_holdrand = 1u;\n"
open(p, "w").write(s)


# bg_lib.c defines its own rand/srand OUTSIDE the Q3_VM guard (the #endif at
# ~761 closes early) — shield those definitions from the macros. They compile
# as dead C++-mangled symbols either way (retail excludes bg_lib.c entirely);
# the shield just keeps the definitions parseable.
p = "build/src/codemp/game/bg_lib.c"
s = open(p).read()
old_randseed = "static int randSeed = 0;"
new_randseed = ("#undef rand /* referee-oracle: shield bg_lib's own defs from the MSVC-rand macros */\n"
                "#undef srand\n"
                "static int randSeed = 0;")
assert old_randseed in s, "bg_lib.c randSeed not found — oracle changed?"
s = s.replace(old_randseed, new_randseed, 1)
open(p, "w").write(s)
PY

# REF_PROBES diagnostic layer (permanent, env-gated): saber trace / muzzle / collide dumps.
# Diff the two engines' dump streams; first mismatch = the event. See memory/plan.
python3 - <<'PY'
p = "build/src/codemp/game/w_saber.c"
s = open(p).read()

helper = (
    "/* REF_PROBES tap */\n"
    "static unsigned int g6b(float f){union{float f;unsigned int u;}x;x.f=f;return x.u;}\n"
    "static void g6t(const char *tag, const float *a, const float *b, const float *mn, const float *mx, float fr, int en){\n"
    "\tif (!getenv(\"REF_PROBES\")) return;\n"
    "\tfprintf(stderr, \"SAB_TRACE %s t=%i a=%08x,%08x,%08x b=%08x,%08x,%08x mn=%08x mx=%08x fr=%08x en=%i\\n\",\n"
    "\t\ttag, level.time, g6b(a[0]),g6b(a[1]),g6b(a[2]), g6b(b[0]),g6b(b[1]),g6b(b[2]), g6b(mn[0]), g6b(mx[0]), g6b(fr), en);\n"
    "}\n\n"
    "static GAME_INLINE qboolean G_G2TraceCollide")
old_fn = "static GAME_INLINE qboolean G_G2TraceCollide"
assert s.count(old_fn) == 1
s = s.replace(old_fn, helper, 1)

old_cmp = "\t\tif (G2Trace[0].mEntityNum != g2Hit->s.number)"
assert s.count(old_cmp) == 1
new_cmp = ("\t\tif (getenv(\"REF_PROBES\"))\n"
           "\t\t\tfprintf(stderr, \"SAB_CD t=%i en=%i m=%i\\n\", level.time, (int)tr->entityNum, G2Trace[0].mEntityNum);\n"
           + old_cmp)
s = s.replace(old_cmp, new_cmp, 1)

site1 = "\n\t\t\ttrap_Trace(&tr, saberEnd, saberTrMins, saberTrMaxs, saberStart, self->s.number, trMask);\n"
site2 = "\n\t\t\t\ttrap_Trace(&tr, saberEnd, saberTrMins, saberTrMaxs, saberStart, self->s.number, trMask);\n"
assert s.count(site2) == 1, s.count(site2)
s = s.replace(site2, site2 + "\t\t\t\tg6t(\"i2\", saberEnd, saberStart, saberTrMins, saberTrMaxs, tr.fraction, (int)tr.entityNum);\n", 1)
old_st = "\n\t\t\t\tself->client->saber[rSaberNum].blade[rBladeNum].storageTime = level.time;"
assert s.count(old_st) == 1, s.count(old_st)
new_st = ("\n\t\t\t\tif (getenv(\"REF_PROBES\")) { bladeInfo_t *g6bl = &self->client->saber[rSaberNum].blade[rBladeNum];\n"
    "\t\t\t\t\tfprintf(stderr, \"SAB_MUZZLE t=%i sn=%i bn=%i pO=%08x,%08x,%08x pA=%08x,%08x,%08x mP=%08x,%08x,%08x mD=%08x,%08x,%08x\\n\",\n"
    "\t\t\t\t\tlevel.time, rSaberNum, rBladeNum, g6b(properOrigin[0]),g6b(properOrigin[1]),g6b(properOrigin[2]),\n"
    "\t\t\t\t\tg6b(properAngles[0]),g6b(properAngles[1]),g6b(properAngles[2]),\n"
    "\t\t\t\t\tg6b(g6bl->muzzlePoint[0]),g6b(g6bl->muzzlePoint[1]),g6b(g6bl->muzzlePoint[2]),\n"
    "\t\t\t\t\tg6b(g6bl->muzzleDir[0]),g6b(g6bl->muzzleDir[1]),g6b(g6bl->muzzleDir[2])); }"
    + old_st)
s = s.replace(old_st, new_st, 1)

old_lerp = "{\n\tvec3_t baseOld, endOld;\n\tvec3_t mp1, mp2;\n\tvec3_t md1, md2;\n"
assert s.count(old_lerp) == 1, s.count(old_lerp)
new_lerp = (old_lerp +
    "\tif (getenv(\"REF_PROBES\")) { saberTrail_t *g6tb = &self->client->saber[saberNum].blade[bladeNum].trail;\n"
    "\t\tfprintf(stderr, \"SAB_TRAIL t=%i sn=%i bn=%i bN=%08x,%08x,%08x eN=%08x,%08x,%08x trB=%08x,%08x,%08x trT=%08x,%08x,%08x lt=%i\\n\",\n"
    "\t\tlevel.time, saberNum, bladeNum, g6b(baseNew[0]),g6b(baseNew[1]),g6b(baseNew[2]), g6b(endNew[0]),g6b(endNew[1]),g6b(endNew[2]),\n"
    "\t\tg6b(g6tb->base[0]),g6b(g6tb->base[1]),g6b(g6tb->base[2]), g6b(g6tb->tip[0]),g6b(g6tb->tip[1]),g6b(g6tb->tip[2]), g6tb->lastTime); }\n")
s = s.replace(old_lerp, new_lerp, 1)

site3 = "\n\t\t\ttrap_Trace(&tr, saberStart, saberTrMins, saberTrMaxs, saberEndExtrapolated, self->s.number, trMask);\n"
assert s.count(site3) == 1, s.count(site3)
s = s.replace(site3, site3 + "\t\t\tg6t(\"ex\", saberStart, saberEndExtrapolated, saberTrMins, saberTrMaxs, tr.fraction, (int)tr.entityNum);\n", 1)
assert s.count(site1) == 1, s.count(site1)
s = s.replace(site1, site1 + "\t\t\tg6t(\"i1\", saberEnd, saberStart, saberTrMins, saberTrMaxs, tr.fraction, (int)tr.entityNum);\n", 1)

# LOOK_TGT: caller-side look inputs (G_G2PlayerAngles humanoid branch).
old_fwd = "#define LOOK_DEFAULT_SPEED"
assert s.count(old_fwd) == 1
s = s.replace(old_fwd, "static unsigned int g6b(float f); /* REF_PROBES fwd decl */\n" + old_fwd, 1)
old_h = "\t\tlookAngles[PITCH] = 0;\n"
assert s.count(old_h) == 1, s.count(old_h)
new_h = (old_h +
    "\t\tif (getenv(\"REF_PROBES\"))\n"
    "\t\t\tfprintf(stderr, \"LOOK_TGT t=%i en=%i hlt=%i lkt=%i li=%08x,%08x,%08x\\n\",\n"
    "\t\t\tlevel.time, ent->s.number, ent->client->ps.hasLookTarget, ent->client->lookTime,\n"
    "\t\t\tg6b(lookAngles[0]),g6b(lookAngles[1]),g6b(lookAngles[2]));\n")
s = s.replace(old_h, new_h, 1)
open(p, "w").write(s)
PY

# REF_PROBES diagnostic layer (permanent, env-gated): G_PickDeathAnim inputs + RNG state.
python3 - <<'PY'
p = "build/src/codemp/game/g_combat.c"
s = open(p).read()
helper = ("/* REF_PROBES tap */\n"
          "extern unsigned int jka_msvc_holdrand;\n"
          "static unsigned int g6b4(float f){union{float f;unsigned int u;}x;x.f=f;return x.u;}\n\n"
          "int G_PickDeathAnim(")
old_fn = "int G_PickDeathAnim("
assert s.count(old_fn) == 1, s.count(old_fn)
s = s.replace(old_fn, helper, 1)

old_call = "\t\tanim = G_PickDeathAnim(self, self->pos1, damage, meansOfDeath, HL_NONE);"
assert s.count(old_call) == 1, s.count(old_call)
new_call = ("\t\t{ unsigned int g6rin = jka_msvc_holdrand;\n"
    + old_call + "\n"
    "\t\tif (getenv(\"REF_PROBES\"))\n"
    "\t\t\tfprintf(stderr, \"DEATH_ANIM t=%i en=%i dmg=%i mod=%i p=%08x,%08x,%08x rin=%08x rout=%08x anim=%i\\n\",\n"
    "\t\t\tlevel.time, self->s.number, damage, meansOfDeath,\n"
    "\t\t\tg6b4(self->pos1[0]),g6b4(self->pos1[1]),g6b4(self->pos1[2]),\n"
    "\t\t\tg6rin, jka_msvc_holdrand, anim); }\n")
s = s.replace(old_call, new_call, 1)
open(p, "w").write(s)
PY

# REF_PROBES diagnostic layer (permanent, env-gated): CalcMuzzlePoint pre-snap muzzle + aim.
python3 - <<'PY'
p = "build/src/codemp/game/g_weapon.c"
s = open(p).read()
helper = ("/* REF_PROBES tap */\n"
          "static unsigned int g6b3(float f){union{float f;unsigned int u;}x;x.f=f;return x.u;}\n\n"
          "void CalcMuzzlePoint ( gentity_t *ent,")
old_fn = "void CalcMuzzlePoint ( gentity_t *ent,"
assert s.count(old_fn) == 1
s = s.replace(old_fn, helper, 1)

fn_start = s.index(helper)
snap_line = "\tSnapVector( muzzlePoint );"
snap_at = s.index(snap_line, fn_start)
dump = ("\tif (getenv(\"REF_PROBES\"))\n"
    "\t\tfprintf(stderr, \"MUZZLE t=%i en=%i w=%i pre=%08x,%08x,%08x tb=%08x,%08x,%08x fw=%08x,%08x,%08x\\n\",\n"
    "\t\tlevel.time, ent->s.number, weapontype,\n"
    "\t\tg6b3(muzzlePoint[0]),g6b3(muzzlePoint[1]),g6b3(muzzlePoint[2]),\n"
    "\t\tg6b3(ent->s.pos.trBase[0]),g6b3(ent->s.pos.trBase[1]),g6b3(ent->s.pos.trBase[2]),\n"
    "\t\tg6b3(forward[0]),g6b3(forward[1]),g6b3(forward[2]));\n")
s = s[:snap_at] + dump + s[snap_at:]
open(p, "w").write(s)
PY

# REF_PROBES diagnostic layer (permanent, env-gated): BG_UpdateLookAngles in/out taps.
python3 - <<'PY'
p = "build/src/codemp/game/bg_pmove.c"
s = open(p).read()

helper = ("/* REF_PROBES tap */\n"
          "#include <stdio.h>\n"
          "#include <stdlib.h>\n"
          "static unsigned int g6b2(float f){union{float f;unsigned int u;}x;x.f=f;return x.u;}\n\n"
          "//Adjust the head/neck desired angles\nvoid BG_UpdateLookAngles(")
old_fn = "//Adjust the head/neck desired angles\nvoid BG_UpdateLookAngles("
assert s.count(old_fn) == 1
s = s.replace(old_fn, helper, 1)

old_call = "\tBG_UpdateLookAngles(lookTime, lastHeadAngles, time, lookAngles, lookSpeed, -50.0f, 50.0f, -70.0f, 70.0f, -30.0f, 30.0f);"
assert s.count(old_call) == 1
new_call = ("\tif (getenv(\"REF_PROBES\"))\n"
    "\t\tfprintf(stderr, \"LOOK_UPD t=%i en=%i li=%08x,%08x,%08x lt=%i lh=%08x,%08x,%08x\\n\",\n"
    "\t\ttime, cent->number, g6b2(lookAngles[0]),g6b2(lookAngles[1]),g6b2(lookAngles[2]),\n"
    "\t\tlookTime, g6b2(lastHeadAngles[0]),g6b2(lastHeadAngles[1]),g6b2(lastHeadAngles[2]));\n"
    + old_call + "\n"
    "\tif (getenv(\"REF_PROBES\"))\n"
    "\t\tfprintf(stderr, \"LOOK_WB t=%i en=%i lo=%08x,%08x,%08x\\n\",\n"
    "\t\ttime, cent->number, g6b2(lookAngles[0]),g6b2(lookAngles[1]),g6b2(lookAngles[2]));\n")
s = s.replace(old_call, new_call, 1)
open(p, "w").write(s)
PY

# vmMain word-width patch (engine lockstep referee, G1): Raven's
# `int vmMain(int command, int arg0..arg11)` is a 32-bit-era signature. Our
# engine's VM_Call passes 12 pointer-width words (RawVmMain — see
# crates/mp/engine/qcommon/src/vm_fns.rs); on LP64 the oracle's `int` params
# truncate every pointer-carrying word (GAME_NAV_* vec3 args,
# GAME_ROFF_NOTETRACK_CALLBACK's string — proven segfault hosting bots
# 2026-07-13), and `(int)ClientConnect(...)` truncates the returned
# denied-string pointer. Widen the params, the return, and that one return
# cast to GCC's builtin __INTPTR_TYPE__ (no include needed). All other
# dispatch arms narrow intptr_t -> int implicitly, which is exactly the
# 32-bit behavior for the small values they carry.
python3 - <<'PY'
p = "build/src/codemp/game/g_main.c"
s = open(p).read()

old_sig = ("int vmMain( int command, int arg0, int arg1, int arg2, int arg3, "
           "int arg4, int arg5, int arg6, int arg7, int arg8, int arg9, "
           "int arg10, int arg11  ) {")
new_sig = ("__INTPTR_TYPE__ vmMain( int command, "
           + ", ".join("__INTPTR_TYPE__ arg%d" % i for i in range(12))
           + " ) { /* referee-oracle: LP64-widened words (G1) */")
assert old_sig in s, "g_main.c vmMain signature not found — oracle changed?"
s = s.replace(old_sig, new_sig, 1)

old_cc = "return (int)ClientConnect( arg0, arg1, arg2 );"
new_cc = "return (__INTPTR_TYPE__)ClientConnect( arg0, arg1, arg2 );"
assert old_cc in s, "g_main.c ClientConnect return cast not found — oracle changed?"
s = s.replace(old_cc, new_cc, 1)

open(p, "w").write(s)
PY

C=build/src/codemp

# --- platform branch (Darwin dylib / Linux so) ---------------------------------
OS="$(uname -s)"
case "$OS" in
	Darwin) LIBOUT=build/liboraclejampgame.dylib; PICFLAG="";;
	Linux)  LIBOUT=build/liboraclejampgame.so; PICFLAG="-fPIC";;
	*) echo "error: unsupported platform '$OS' (only Darwin and Linux are supported)" >&2; exit 1;;
esac

# --- flags --------------------------------------------------------------------
# Dialect: gnu++98 — Raven is C++98-era; it also keeps `std::move`/`std::forward`
#   out of scope, which otherwise collide with Raven's global `move`/`forward`
#   vec3_t vars under a header's `using namespace std`.
# -fpermissive: downgrade the 64-bit `FOFS ((int)&ptr)` pointer->int narrowing
#   (a hard error in strict C++) to a warning — the values are small field
#   offsets, so the truncation is numerically harmless.
# -fsigned-char: pins retail `char` semantics on platforms where it defaults
#   unsigned (e.g. aarch64 Linux gcc); a no-op on Apple/x86 where char is
#   already signed. Matches OpenJK (sets it for all GNU/Clang builds) and
#   Raven's own legacy unix makefile (oracle/codemp/unix/makefile:78).
# FP regime (parity-defining): -fno-fast-math -ffp-contract=off matches what
#   rustc/LLVM do for the Rust cdylib (IEEE, NO fused multiply-add). -O2 to match
#   the release profile. See README.md.
# Defines: QAGAME (this is the game DLL) + _JK2MP (multiplayer tree: routes the
#   vehicle/NPC TUs to the MP bg_vehicles.h path, past the SP-only includes) +
#   __linux__ (wraps dllEntry/vmMain in `extern "C"` for unmangled exports) +
#   _FORTIFY_SOURCE=0 (no fortify wrappers).
# NDEBUG: retail jampgame was an MSVC Release build, so asserts were compiled
#   away — NDEBUG is the retail-faithful assert regime. It also sidesteps
#   glibc's C++ assert() (a static_cast to bool) rejecting Raven's
#   pointer-as-condition asserts under gnu++98 on the Linux lane.
CXXFLAGS="-x c++ -std=gnu++98 -fpermissive -w -O2 -fno-fast-math -ffp-contract=off \
	-fsigned-char $PICFLAG \
	-fexceptions -funwind-tables \
	-DQAGAME -D_JK2MP -D__linux__ -D_FORTIFY_SOURCE=0 -DNDEBUG \
	-include $SHIM \
	-I $C/game -I $C/qcommon -I $C/ghoul2 -I $C/cgame -I $C/icarus"

# --- compile every game TU ----------------------------------------------------
n=0
for f in "$C"/game/*.c; do
	b=$(basename "$f" .c)
	# shellcheck disable=SC2086
	"$CXX" $CXXFLAGS -c "$f" -o "build/obj/$b.o"
	n=$((n + 1))
done
echo "referee-oracle: compiled $n TUs"

# --- link the loadable module -------------------------------------------------
case "$OS" in
	Darwin) "$CXX" -dynamiclib -o "$LIBOUT" build/obj/*.o -lm;;
	Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm;;
esac
echo "referee-oracle: linked $LIBOUT"

# --- sanity: the engine entrypoints must be visible for dlsym -----------------
case "$OS" in
	Darwin)
		if ! nm -gU "$LIBOUT" | grep -qE '_dllEntry$'; then
			echo "error: dllEntry not exported" >&2; exit 1
		fi
		if ! nm -gU "$LIBOUT" | grep -qE '_vmMain$'; then
			echo "error: vmMain not exported" >&2; exit 1
		fi
		;;
	Linux)
		if ! nm -gD --defined-only "$LIBOUT" | grep -qE '\bdllEntry$'; then
			echo "error: dllEntry not exported" >&2; exit 1
		fi
		if ! nm -gD --defined-only "$LIBOUT" | grep -qE '\bvmMain$'; then
			echo "error: vmMain not exported" >&2; exit 1
		fi
		;;
esac
echo "referee-oracle: OK — dllEntry + vmMain exported"
