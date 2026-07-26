#!/bin/sh
# renderer-oracle: build the shader-parse dumper from the UNMODIFIED Raven
# oracle/codemp/renderer/tr_shader.cpp (R_InitShaders -> ScanAndLoadShaderFiles
# -> R_FindShader -> ParseShader -> FinishShader) and check (or, with --regen,
# regenerate) the golden dumps under golden/.
#
# Strategy (porting-rules §18 / DEC-37 ruling 15): the "full header closure"
# pattern from tools/ui-oracle (tr_shader.cpp is too large and too entangled
# with tr_local.h's renderer-wide type closure for gp2-oracle's from-scratch
# stub-header approach). oracle/ is never edited; every oracle file this needs
# is copied into build/ next to this directory's main.cpp and the GL/qgl stub
# in stubs/qgl.h (which REPLACES the copied real qgl.h -- see its header
# comment and README.md's "GL surface" section). See README.md for the full
# stub inventory (every non-oracle symbol this links against) and for the
# fixture/keyword-coverage checklist.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle
GLSHIM=../closure-prototype/glshim
B=build

rm -rf "$B"
mkdir -p "$B/codemp/renderer" "$B/codemp/game" "$B/codemp/qcommon" \
         "$B/codemp/ghoul2" "$B/codemp/cgame" "$B/codemp/RMG" "$B/codemp"

# ---- renderer: the TU under test + its full sibling-header closure --------
cp "$ORACLE/codemp/renderer/tr_shader.cpp" "$B/codemp/renderer/"
cp "$ORACLE"/codemp/renderer/*.h "$B/codemp/renderer/"
# stubs/qgl.h REPLACES the real qgl.h (avoids the GLX/win32/macosx_glimp
# platform-detect branches the real header takes; see its own header comment
# for exactly which oracle grammar it reproduces vs. stubs). glext.h/
# qgl_console.h/glext_console.h are never referenced once qgl.h is replaced.
cp stubs/qgl.h "$B/codemp/renderer/qgl.h"
rm -f "$B/codemp/renderer/glext.h" "$B/codemp/renderer/glext_console.h" "$B/codemp/renderer/qgl_console.h"

# ---- game: q_shared.c + q_math.c (real, unmodified) for COM_*/Q_*/Vector*/
# va(); every other game/*.h (types only, nothing else compiled) ----------
cp "$ORACLE/codemp/game/q_shared.c" "$ORACLE/codemp/game/q_math.c" "$B/codemp/game/"
cp "$ORACLE"/codemp/game/*.h "$B/codemp/game/"

# ---- qcommon: exe_headers.h (tr_shader.cpp's first #include) + headers ----
cp "$ORACLE/codemp/qcommon/exe_headers.h" "$B/codemp/qcommon/"
cp "$ORACLE"/codemp/qcommon/*.h "$B/codemp/qcommon/"

# ---- ghoul2/cgame/RMG: header-only (pulled in by tr_local.h's own
# #include "../ghoul2/ghoul2_shared.h" and tr_public.h's tr_types.h) --------
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h "$B/codemp/cgame/" 2>/dev/null || true
cp "$ORACLE"/codemp/RMG/*.h "$B/codemp/RMG/" 2>/dev/null || true
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/" 2>/dev/null || true

# main.cpp lives in this directory (tools/renderer-oracle/), not oracle/.
#
# Everything compiles as C++ (tr_shader.cpp always was; q_shared.c/q_math.c
# are compiled -x c++ here too, unlike ui-oracle's C/C++ split, since nothing
# in this closure needs C linkage -- there is no cross-language boundary to
# bridge, so no pc_bridge.cpp-style shim is needed).
#
# -std=c++14 (not c++17): libc++'s C++17 <cstddef> pulls `std::byte` into
# scope, which collides with Raven's own `byte` typedef (q_shared.h:349)
# every time tr_local.h uses it unqualified -- pin c++14 like gp2-oracle/
# mp-renderer's own profile, which sidesteps the clash entirely.
# -D__linux__: routes q_shared.h's SnapVector off its MSVC __asm branch (the
# same fix ui-oracle's run.sh uses) -- harmless for qgl.h since stubs/qgl.h
# already replaced it, so the real qgl.h's `#elif defined(__linux__)` GLX
# branch is never reached.
# -Wno-c++11-narrowing: CONTENTS_TRANSLUCENT (surfaceflags.h) is 0x80000000,
# which narrows against `int` in infoParms[]'s aggregate init -- Raven's own
# source, not something to fix; downgrade to the (accepted) implementation-
# defined narrowing instead of erroring.
DEFS="-DNDEBUG -D__linux__ \
 -DLPCTSTR=\"const char *\" -DLPCSTR=\"const char *\" -DCOLORREF=\"unsigned int\" \
 -DDWORD=\"unsigned int\" -DWORD=\"unsigned short\" -DBYTE=\"unsigned char\" \
 -DHANDLE=\"void *\" -DLPVOID=\"void *\" -D__int64=\"long long\" \
 -Dstricmp=strcasecmp -Dstrnicmp=strncasecmp -Dstrcmpi=strcasecmp \
 -DUSHORT=\"unsigned short\" -DBOOL=int -DUINT=\"unsigned int\" -DFLOAT=float \
 -DHDC=\"void *\" -DHGLRC=\"void *\" -DHPBUFFERARB=\"void *\""
INCLUDES="-I $B -I $B/codemp/renderer -I $B/codemp/game -I $B/codemp/qcommon \
 -I $B/codemp/ghoul2 -I $B/codemp/cgame -I $GLSHIM"
CXXFLAGS="-std=c++14 -w -Wno-c++11-narrowing -fdeclspec $DEFS $INCLUDES"

OBJS=""
n=0
for f in "$B/codemp/renderer/tr_shader.cpp" "$B/codemp/game/q_shared.c" "$B/codemp/game/q_math.c" main.cpp; do
	o="$B/$(basename "$f").o"
	# DEFS carries embedded double-quoted values (e.g. -DLPCTSTR="const char
	# *"); a plain unquoted $DEFS expansion word-splits on the spaces INSIDE
	# those quotes (the quote characters are literal data at this point, not
	# shell syntax) -- eval re-parses the whole line so the shell's own
	# quoting rules apply, same as ui-oracle/gp2-oracle's simpler (no
	# embedded-space defines) invocations don't need to worry about.
	eval c++ "$CXXFLAGS" -x c++ -c "\"$f\"" -o "\"$o\""
	OBJS="$OBJS $o"
	n=$((n + 1))
done

echo "renderer-oracle: compiled $n TUs (tr_shader.cpp + q_shared.c + q_math.c + main.cpp)"
# shellcheck disable=SC2086
c++ $OBJS -o "$B/rdump"
echo "renderer-oracle: linked $B/rdump"

mkdir -p golden
status=0
run_one() {
	fixture="$1"
	shadersdir="$B/shaders_$fixture"
	rm -rf "$shadersdir"
	mkdir -p "$shadersdir"
	cp "fixtures/$fixture.shader" "$shadersdir/"
	if [ "${REGEN:-0}" = "1" ]; then
		"$B/rdump" "$shadersdir" "fixtures/$fixture.names" >"golden/$fixture.txt"
		echo "regenerated $fixture.txt"
	else
		"$B/rdump" "$shadersdir" "fixtures/$fixture.names" | diff -u "golden/$fixture.txt" - || status=1
	fi
}

[ "${1:-}" = "--regen" ] && REGEN=1 || REGEN=0

run_one general_keywords
run_one stage_keywords
run_one edge_cases

[ "$status" -eq 0 ] && echo "renderer-oracle: OK"
exit "$status"
