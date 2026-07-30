#!/bin/sh
# cgame-oracle: build Raven's UNMODIFIED MP cgame module from the oracle source
# as a loadable dylib - `build/liboraclecgame.dylib`.
#
# This is the reference module for the C6b demo-referee (docs/decisions.md
# DEC-48 ruling 2): the oracle cgame DLL and our Rust `cgame` cdylib get replayed
# the same recorded engine->module input stream in lockstep and their outgoing
# trap streams are byte-diffed. This script + smoke.c only prove the artifact
# builds and a mock engine can drive its `dllEntry`/`vmMain` handshake.
#
# The oracle tree (oracle/**) is NEVER edited. Every TU is compiled from a
# throwaway COPY under build/src/, patched in-place by the python step below.
#
# TU list is authoritative from oracle/codemp/cgame/JK2_cgame.vcproj (Release):
# the cgame-local cg_*/fx_* files plus the shared-with-game bg_*/vehicle-NPC/
# q_math/q_shared TUs and ../ui/ui_shared.c. bg_lib.c is the sole listed source
# marked ExcludedFromBuild in every config (JK2_cgame.vcproj Release/Final/Debug/
# Debug(SH)), so it is dropped - its rand/srand live under #ifdef Q3_VM anyway.
#
# Compiled as C, not C++. The vcproj's `CompileAs="0"` is compile-by-extension,
# so retail built these .c files as C; cgame's headers (q_shared.h, tr_types.h,
# bg_public.h, cg_public.h, ghoul2/G2.h - all #define-only) carry no C++ classes,
# unlike the game module's ghoul2/icarus class headers. Compiling as C is both
# retail-faithful AND makes `dllEntry`/`vmMain` unmangled for free: cgame (unlike
# game/g_syscalls.c + g_main.c) never wraps them in `#ifdef __linux__ extern "C"`.
#
# Requires a real GCC (Homebrew `gcc`): the unmodified 32-bit-era source needs
# `-fpermissive` to accept its `FOFS`/`CGFOFS` pointer->int casts on a 64-bit
# host, which Apple clang does not provide. Install with `brew install gcc`.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle
C=build/src/codemp
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
echo "cgame-oracle: using $CXX"

# --- lay down a throwaway build tree (oracle is never touched) -----------------
rm -rf build
mkdir -p build/src build/obj
cp -R "$ORACLE/codemp" build/src/codemp
mkdir -p build/src/ui
cp "$ORACLE/ui/menudef.h" build/src/ui/   # cg_*/ui_shared.h: ../../ui/menudef.h

# Raven's cgame sources spell some includes with MSVC backslash separators
# (`#include "..\game\q_shared.h"`); MSVC resolves them, unix gcc reads the
# backslashes literally. Flip `\` -> `/` inside `#include "..."` directives
# across the copied tree - a path-separator normalization, ABI-neutral, copy only.
python3 - <<'PY'
import os, re
inc = re.compile(r'^(\s*#\s*include\s*")([^"]*)(".*)$')
for root, _, files in os.walk("build/src"):
    for f in files:
        if not f.endswith((".c", ".h")):
            continue
        p = os.path.join(root, f)
        lines = open(p, encoding="latin-1").read().split("\n")
        out, hit = [], False
        for ln in lines:
            m = inc.match(ln)
            if m and "\\" in m.group(2):
                ln = m.group(1) + m.group(2).replace("\\", "/") + m.group(3)
                hit = True
            out.append(ln)
        if hit:
            open(p, "w", encoding="latin-1").write("\n".join(out))
PY

# --- source normalizations (a COPY, never the oracle) --------------------------
# Same parity class as tools/referee-oracle: retail-win32 rounding/RNG/64-bit
# width. None touch cgame program logic; they pin the numeric bar the Rust
# cdylib is diffed against. The C++-only patches referee-oracle needs
# (qboolean=int, libm double-promotion macros) are absent here - as C, `bool`
# assigns to the qboolean enum fine and `sin`/`sqrt` resolve to the double libm
# without overload ambiguity, exactly as retail's C compile did.
python3 - <<'PY'
# q_shared.h: retail-win32 SnapVector + MSVC-CRT rand()/srand().
p = "build/src/codemp/game/q_shared.h"
s = open(p).read()

# The x86 SnapVector uses inline `__asm fld/fistp` (q_shared.h:1408) - won't
# build on arm64, so we take the __linux__ macro branch. That branch TRUNCATES
# via (int) casts instead of rounding; retarget it to rint(), which under the
# default FP environment is round-to-nearest-even - exactly fistp's semantics
# and the port's parity bar (crates/mp/game bg_misc round_ties_even). math.h is
# already included at q_shared.h:82.
old_snap = ("#ifdef __linux__\n"
            "#define\tSnapVector(v) {v[0]=((int)(v[0]));v[1]=((int)(v[1]));v[2]=((int)(v[2]));}\n")
new_snap = ("#ifdef __linux__\n"
            "#define\tSnapVector(v) {v[0]=(float)rint(v[0]);v[1]=(float)rint(v[1]);v[2]=(float)rint(v[2]);}\n")
assert old_snap in s, "q_shared.h __linux__ SnapVector macro not found - oracle changed?"
s = s.replace(old_snap, new_snap, 1)

# retail links the MSVC CRT rand()/srand() into the native module (bg_lib.c's
# rand/srand sit under #ifdef Q3_VM and bg_lib.c is excluded here anyway). Six
# cgame TUs call rand()/srand() directly (cg_ents/localents/event/marks/players/
# weapons), so effect randomization diverges from retail unless we route to a
# retail-exact MSVC-semantics clone. State is a single shared variable defined
# in the q_math.c copy below. Same recipe as tools/referee-oracle/build.sh.
old_limitsinc = "#include <limits.h>\n"
new_limitsinc = (
    "#include <limits.h>\n"
    "/* cgame-oracle: retail-win32 rand()/srand() are the MSVC CRT LCG; this\n"
    "   host's libc differs. See tools/cgame-oracle/build.sh for rationale. */\n"
    "extern unsigned int jka_msvc_holdrand;\n"
    "static inline int jka_msvc_rand(void) {\n"
    "    jka_msvc_holdrand = jka_msvc_holdrand * 214013u + 2531011u;\n"
    "    return (int)((jka_msvc_holdrand >> 16) & 0x7fff);\n"
    "}\n"
    "static inline void jka_msvc_srand(unsigned int seed) { jka_msvc_holdrand = seed; }\n"
    "#define rand() jka_msvc_rand()\n"
    "#define srand(s) jka_msvc_srand((unsigned int)(s))\n"
)
assert old_limitsinc in s, "q_shared.h '#include <limits.h>' not found - oracle changed?"
s = s.replace(old_limitsinc, new_limitsinc, 1)
open(p, "w").write(s)

# q_math.c: MSVC-rand state + two retail-32-bit width fixes.
p = "build/src/codemp/game/q_math.c"
s = open(p).read()
s += "\n/* cgame-oracle: MSVC CRT rand() state (see q_shared.h patch). */\nunsigned int jka_msvc_holdrand = 1u;\n"

# Q_rsqrt's `long i` type-pun reads 8 bytes from a 4-byte float at LP64 width
# (UB: stack garbage feeds the shift) - retail i386 `long` is 32-bit.
old_rsqrt = "\tlong i;"
new_rsqrt = "\tint i; /* cgame-oracle: retail 32-bit long */"
assert old_rsqrt in s, "q_math.c Q_rsqrt long decl not found - oracle changed?"
s = s.replace(old_rsqrt, new_rsqrt, 1)

# flrand/irand holdrand: `unsigned long` is 32-bit on win32, 64-bit here. At
# LP64 width `holdrand >> 17` spans the full register and the draws blow past
# [0, 32767]. Force retail 32-bit state.
old_hold = "static unsigned long\tholdrand = 0x89abcdef;"
new_hold = "static unsigned int\tholdrand = 0x89abcdef; /* cgame-oracle: retail-win32 32-bit width */"
assert old_hold in s, "q_math.c holdrand decl not found - oracle changed?"
s = s.replace(old_hold, new_hold, 1)
open(p, "w").write(s)

# cg_main.c: LP64-widen vmMain's words + its pointer-returning arms. Raven's
# `int vmMain(int command, int arg0..arg11)` is 32-bit-era; our engine's VM_Call
# passes 12 pointer-width words (RawVmMain). On LP64 the `int` params truncate
# every pointer-carrying arg (CG_GET_ORIGIN's `(float*)arg1`,
# CG_ROFF_NOTETRACK_CALLBACK's `(const char*)arg1`, ...) and the four
# `return (int)ptr` arms truncate handles/trajectory pointers the engine reads
# back. Widen params, return, and those four casts to GCC's builtin
# __INTPTR_TYPE__ (no include). Other arms narrow intptr_t->int implicitly,
# which is the 32-bit behavior for the small values they carry. Mirrors
# referee-oracle's g_main.c vmMain widening (G1).
p = "build/src/codemp/cgame/cg_main.c"
s = open(p).read()
old_sig = ("int vmMain( int command, int arg0, int arg1, int arg2, int arg3, "
           "int arg4, int arg5, int arg6, int arg7, int arg8, int arg9, "
           "int arg10, int arg11  ) {")
new_sig = ("__INTPTR_TYPE__ vmMain( int command, "
           + ", ".join("__INTPTR_TYPE__ arg%d" % i for i in range(12))
           + " ) { /* cgame-oracle: LP64-widened words */")
assert old_sig in s, "cg_main.c vmMain signature not found - oracle changed?"
s = s.replace(old_sig, new_sig, 1)

for ptr_ret in (
    "return (int)cg_entities[arg0].ghoul2;",
    "return (int)cgs.gameModels;",
    "return (int)&cg_entities[arg0].nextState.pos;",
    "return (int)&cg_entities[arg0].nextState.apos;",
):
    assert ptr_ret in s, "cg_main.c vmMain pointer-return arm not found: %s" % ptr_ret
    s = s.replace(ptr_ret, ptr_ret.replace("(int)", "(__INTPTR_TYPE__)", 1), 1)
open(p, "w").write(s)
PY

# --- platform branch (Darwin dylib / Linux so) ---------------------------------
OS="$(uname -s)"
case "$OS" in
	Darwin) LIBOUT=build/liboraclecgame.dylib; PICFLAG="";;
	Linux)  LIBOUT=build/liboraclecgame.so; PICFLAG="-fPIC";;
	*) echo "error: unsupported platform '$OS' (only Darwin and Linux are supported)" >&2; exit 1;;
esac

# --- flags --------------------------------------------------------------------
# Compiled AS C (-x c -std=gnu99): retail-faithful (vcproj CompileAs=default) and
#   it leaves dllEntry/vmMain with natural C linkage - no name mangling, no
#   extern "C" wrappers to add (cgame ships none).
# -fpermissive: downgrade the 64-bit FOFS/CGFOFS `((int)&ptr)` pointer->int
#   narrowing to a warning. GCC-only (Apple clang no-ops it).
# -fsigned-char: pins retail `char` semantics on platforms defaulting unsigned.
# FP regime (parity-defining): -fno-fast-math -ffp-contract=off matches what
#   rustc/LLVM do for the Rust cdylib (IEEE, no fused multiply-add). -O2 = release.
# -fno-builtin sin/cos family: GCC -O2 fuses a same-argument sin+cos pair into
#   one __builtin_cexpi call, which lands on Apple's cexp - and Apple's cexp
#   returns +0.0 imag for a -0.0 angle, where sin(-0.0) is -0.0. AngleVectors
#   hit this (509 shield-axis sign-of-zero diffs in the C6b referee). These
#   flags keep every sin/cos a real libm call, the regime the line above claims.
# Defines: from JK2_cgame.vcproj Release (NDEBUG;WIN32;_WINDOWS;MISSIONPACK;_JK2;
#   CGAME) with the win32 pair swapped for __linux__ - the host branch that
#   selects the macro SnapVector (past the x86 __asm one) and `ID_INLINE inline`.
#   _JK2 (not _JK2MP) is what the vcproj ships; the four vehicle-NPC TUs bridge
#   _JK2 -> _JK2MP themselves, bg_vehicleLoad.c self-#defines _JK2MP, and no
#   other TU here reads _JK2MP. _FORTIFY_SOURCE=0 = no fortify wrappers.
CFLAGS="-x c -std=gnu99 -fpermissive -w -O2 -fno-fast-math -ffp-contract=off \
	-fno-builtin-sin -fno-builtin-cos -fno-builtin-sinf -fno-builtin-cosf \
	-fno-builtin-sincos -fno-builtin-sincosf \
	-fsigned-char $PICFLAG \
	-DNDEBUG -DMISSIONPACK -D_JK2 -DCGAME -D__linux__ -D_FORTIFY_SOURCE=0 \
	-include $SHIM \
	-I $C/game -I $C/cgame -I $C/qcommon -I $C/ui"

# --- the authoritative TU list (JK2_cgame.vcproj Source Files, bg_lib dropped) --
CGAME_TUS="cg_consolecmds cg_draw cg_drawtools cg_effects cg_ents cg_event \
	cg_info cg_light cg_localents cg_main cg_marks cg_newDraw cg_players \
	cg_playerstate cg_predict cg_saga cg_scoreboard cg_servercmds cg_snapshot \
	cg_strap cg_syscalls cg_turret cg_view cg_weaponinit cg_weapons \
	fx_blaster fx_bowcaster fx_bryarpistol fx_demp2 fx_disruptor fx_flechette \
	fx_force fx_heavyrepeater fx_rocketlauncher"
GAME_TUS="AnimalNPC FighterNPC SpeederNPC WalkerNPC bg_g2_utils bg_misc \
	bg_panimate bg_pmove bg_saber bg_saberLoad bg_saga bg_slidemove \
	bg_vehicleLoad bg_weapons q_math q_shared"

n=0
for b in $CGAME_TUS; do
	# shellcheck disable=SC2086
	"$CXX" $CFLAGS -c "$C/cgame/$b.c" -o "build/obj/$b.o"
	n=$((n + 1))
done
for b in $GAME_TUS; do
	# shellcheck disable=SC2086
	"$CXX" $CFLAGS -c "$C/game/$b.c" -o "build/obj/$b.o"
	n=$((n + 1))
done
# shellcheck disable=SC2086
"$CXX" $CFLAGS -c "$C/ui/ui_shared.c" -o "build/obj/ui_shared.o"
n=$((n + 1))
echo "cgame-oracle: compiled $n TUs"

# --- link the loadable module -------------------------------------------------
case "$OS" in
	Darwin) "$CXX" -dynamiclib -o "$LIBOUT" build/obj/*.o -lm;;
	Linux)  "$CXX" -shared -o "$LIBOUT" build/obj/*.o -lm;;
esac
echo "cgame-oracle: linked $LIBOUT"

# --- sanity: the engine entrypoints must be visible for dlsym -----------------
case "$OS" in
	Darwin)
		nm -gU "$LIBOUT" | grep -qE '_dllEntry$' || { echo "error: dllEntry not exported" >&2; exit 1; }
		nm -gU "$LIBOUT" | grep -qE '_vmMain$'   || { echo "error: vmMain not exported" >&2; exit 1; }
		;;
	Linux)
		nm -gD --defined-only "$LIBOUT" | grep -qE '\bdllEntry$' || { echo "error: dllEntry not exported" >&2; exit 1; }
		nm -gD --defined-only "$LIBOUT" | grep -qE '\bvmMain$'   || { echo "error: vmMain not exported" >&2; exit 1; }
		;;
esac
echo "cgame-oracle: OK - dllEntry + vmMain exported"

# --- smoke: dlopen the module and drive the vmMain default arm -----------------
"$CXX" -x c -std=gnu99 -O2 -o build/smoke smoke.c
./build/smoke "$LIBOUT"
