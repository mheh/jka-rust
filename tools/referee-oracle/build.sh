#!/bin/sh
# referee-oracle: build Raven's UNMODIFIED jampgame (QAGAME) game module from the
# oracle source as a loadable dylib — `build/liboraclejampgame.dylib`.
#
# This is the reference module for the Stage-R referee differential harness: the
# oracle DLL and our Rust `jampgame` cdylib are driven over identical inputs and
# byte-diffed. Phase 1 (this script + tests/oracle_smoke.rs) only proves the
# artifact builds and our mock-engine harness can drive its full lifecycle.
#
# The oracle tree (oracle/oracle/**) is NEVER edited. Every game/*.c is compiled
# straight from a throwaway COPY under build/src/ whose only change is a one-line
# activation of Raven's OWN `#define qboolean int` (see the patch step below).
#
# Requires a real GCC (Homebrew `gcc`): the unmodified 32-bit-era C++ source needs
# `-fpermissive` to accept its `FOFS` pointer->int casts on a 64-bit host, which
# Apple clang does not provide (`-fpermissive` is a silent no-op there). Install
# with `brew install gcc`. See README.md for the full rationale.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle/oracle
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
open(p, "w").write(s.replace(old, new, 1))
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
#   Raven's own legacy unix makefile (oracle/oracle/codemp/unix/makefile:78).
# FP regime (parity-defining): -fno-fast-math -ffp-contract=off matches what
#   rustc/LLVM do for the Rust cdylib (IEEE, NO fused multiply-add). -O2 to match
#   the release profile. See README.md.
# Defines: QAGAME (this is the game DLL) + _JK2MP (multiplayer tree: routes the
#   vehicle/NPC TUs to the MP bg_vehicles.h path, past the SP-only includes) +
#   __linux__ (wraps dllEntry/vmMain in `extern "C"` for unmangled exports) +
#   _FORTIFY_SOURCE=0 (no fortify wrappers).
CXXFLAGS="-x c++ -std=gnu++98 -fpermissive -w -O2 -fno-fast-math -ffp-contract=off \
	-fsigned-char $PICFLAG \
	-fexceptions -funwind-tables \
	-DQAGAME -D_JK2MP -D__linux__ -D_FORTIFY_SOURCE=0 \
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
