#!/bin/sh
# ui-oracle: build the menu-parse dumper from the UNMODIFIED Raven
# codemp/ui/ui_shared.c (the parse half: Menu_New -> Menu_Parse ->
# dispatch_menu_keyword -> MenuParse_* -> MenuParse_itemDef -> Item_Parse ->
# dispatch_item_keyword -> ItemParse_*) and check (or, with --regen,
# regenerate) the golden dumps under golden/.
#
# PC_* strategy (porting-rules §F18 "compile the unmodified oracle TU
# standalone"): ui_shared.c's trap_PC_ReadToken/trap_PC_SourceFileAndLine
# route (through the engine, in retail) to botlib's real preprocessor —
# l_precomp.cpp/l_script.cpp. Those are self-contained enough to link
# straight into this dumper (mirrors the jampgame-oracle gcombat slice's
# "full-TU compile against the real q_shared.h" pattern, not gp2-oracle's
# from-scratch stub-header approach): main.cpp calls the UNMODIFIED
# LoadSourceMemory()/PC_ReadTokenHandle()/PC_SourceFileAndLine() directly
# (see l_precomp.h), skipping only the retail VM/syscall marshaling — same
# unsafe-free bypass gp2-oracle's dumper uses to call CGenericParser2
# methods directly. A minimal `botlib_import_t botimport` supplies
# Print/GetMemory/FreeMemory (malloc-backed); FS_* is never exercised since
# fixtures load via LoadSourceMemory, never LoadSourceFile.
#
# ui_shared.c itself is one 10k-line TU with many functions our fixtures
# never call (paint, key handling, ...); every extern it REFERENCES still
# needs a link-time definition regardless of whether it runs. `stubs.c`
# (compiled WITHOUT the game headers, K&R untyped — the C linker binds by
# name alone, same trick jampgame-oracle's gcombat slice uses) satisfies
# those. `main.cpp` implements the handful that sit on the fixtures'
# executed path with small DETERMINISTIC bodies (monotonic counters for
# registerShaderNoMip/registerModel/registerSound/R_RegisterSkin, canonical
# empty strings for cvar/GLA-name lookups, ...) — see its header comment.
#
# oracle/ is never edited; every oracle file is copied into build/ next to
# main.cpp/stubs.c so relative #includes resolve.
set -eu
cd "$(dirname "$0")"

ORACLE=../../oracle
G="$ORACLE/codemp/game"
Q="$ORACLE/codemp/qcommon"
B=build

rm -rf "$B"
mkdir -p "$B/codemp/ui" "$B/codemp/game" "$B/codemp/qcommon" \
         "$B/codemp/ghoul2" "$B/codemp/cgame" "$B/codemp/icarus" \
         "$B/codemp/botlib" "$B/ui"

# ---- ui (the TU under test + its sibling headers) -----------------------
cp "$ORACLE"/codemp/ui/*.c "$ORACLE"/codemp/ui/*.h "$B/codemp/ui/"
rm -f "$B/codemp/ui/ui_main.c" "$B/codemp/ui/ui_atoms.c" "$B/codemp/ui/ui_force.c" \
      "$B/codemp/ui/ui_gameinfo.c" "$B/codemp/ui/ui_players.c" \
      "$B/codemp/ui/ui_saber.c" "$B/codemp/ui/ui_syscalls.c" "$B/codemp/ui/ui_util.c"
cp "$ORACLE/ui/menudef.h" "$B/ui/"

# ---- game (types only — q_shared.c is the one game TU we also compile) --
cp "$G"/*.h "$B/codemp/game/"
cp "$G/q_shared.c" "$B/codemp/game/"
cp "$ORACLE/codemp/cgame/animtable.h" "$B/codemp/game/" 2>/dev/null || true

# ---- qcommon / ghoul2 / cgame / icarus header closures -------------------
cp "$Q"/*.h "$B/codemp/qcommon/"
cp "$ORACLE"/codemp/ghoul2/*.h "$B/codemp/ghoul2/" 2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h "$B/codemp/cgame/" 2>/dev/null || true
cp "$ORACLE"/codemp/icarus/*.h "$B/codemp/icarus/" 2>/dev/null || true
cp "$ORACLE/codemp/namespace_begin.h" "$ORACLE/codemp/namespace_end.h" "$B/codemp/"

# ---- botlib (the REAL preprocessor: l_precomp/l_script/l_memory) --------
cp "$ORACLE/codemp/botlib/l_precomp.h" "$ORACLE/codemp/botlib/l_precomp.cpp" \
   "$ORACLE/codemp/botlib/l_script.h" "$ORACLE/codemp/botlib/l_script.cpp" \
   "$ORACLE/codemp/botlib/l_memory.h" "$ORACLE/codemp/botlib/l_memory.cpp" \
   "$ORACLE/codemp/botlib/l_log.h" \
   "$ORACLE/codemp/botlib/be_interface.h" \
   "$ORACLE/codemp/botlib/l_libvar.h" \
   "$B/codemp/botlib/"

# main.cpp / pc_bridge.{h,cpp} / stubs.c live in this directory
# (tools/ui-oracle/), not oracle/.
#
# ui_shared.c/q_shared.c/main.cpp/stubs.c compile as plain C (`cc`) so
# stubs.c's argless-stub trick works (no C++ name mangling); l_precomp.cpp/
# l_script.cpp/l_memory.cpp keep their real C++ dialect (`c++`); pc_bridge.cpp
# is the sole C++ TU that bridges the two (see pc_bridge.h).
# shim.h (force-included first): pull real libm before the powf rename below
# dodges retail's 2-arg `float powf(float,int)` prototype colliding with
# libm's real `powf(float,float)` (same trick as jampgame-oracle's
# run_gcombat.sh/run_bgmisc.sh). -D__linux__ (below) steers q_shared.h off
# its MSVC __asm SnapVector/ID_INLINE branches.
cat > shim.h <<'EOF'
#include <math.h>
#define powf raven_powf
/* strupr is an MSVC CRT extension with no macOS/Linux libc prototype; only
   referenced from ItemParse_cvarStrList's FEEDER_PLAYER_SPECIES branch,
   which no fixture exercises (see stubs.c for the trivial definition). */
char *strupr(char *s);
/* l_precomp.cpp/l_script.cpp (real C++) call q_shared.h's Com_Error/
   Com_Memcpy/Com_Memset/Com_sprintf/COM_Compress/Q_stricmp, which q_shared.c
   defines with plain-C linkage (compiled via `cc`, not `c++` — see
   pc_bridge.h). Force-including q_shared.h inside extern "C" here (BEFORE
   any oracle header's own #include "q_shared.h", so the include guard makes
   every later one a no-op) gives the C++ TUs the matching unmangled names. */
#ifdef __cplusplus
extern "C" {
#include "q_shared.h"
}
#endif
EOF

INCLUDES="-I $B -I $B/codemp/ui -I $B/codemp/game -I $B/codemp/qcommon \
	-I $B/codemp/ghoul2 -I $B/codemp/cgame -I $B/codemp/icarus -I $B/codemp/botlib -I ."
CFLAGS="-std=gnu11 -fgnu89-inline -w -D__linux__ -include $(pwd)/shim.h $INCLUDES"
CXXFLAGS="-std=gnu++14 -fpermissive -w -D__linux__ -DBOTLIB -include $(pwd)/shim.h $INCLUDES"

OBJS=""
n=0
for f in "$B/codemp/ui/ui_shared.c" "$B/codemp/game/q_shared.c"; do
	o="$B/$(basename "$f").o"
	# shellcheck disable=SC2086
	cc $CFLAGS -c "$f" -o "$o"
	OBJS="$OBJS $o"
	n=$((n + 1))
done
for f in "$B/codemp/botlib/l_precomp.cpp" "$B/codemp/botlib/l_script.cpp" \
         "$B/codemp/botlib/l_memory.cpp" "pc_bridge.cpp"; do
	o="$B/$(basename "$f").o"
	# shellcheck disable=SC2086
	c++ $CXXFLAGS -c "$f" -o "$o"
	OBJS="$OBJS $o"
	n=$((n + 1))
done
# shellcheck disable=SC2086
cc $CFLAGS -x c -c main.cpp -o "$B/main.o"
OBJS="$OBJS $B/main.o"
# stubs.c is compiled WITHOUT the game headers on purpose (argless K&R
# stubs; the linker binds by symbol name alone, same trick as
# jampgame-oracle's stubs_gcombat.c).
cc -std=gnu11 -w -c stubs.c -o "$B/stubs.o"
OBJS="$OBJS $B/stubs.o"

echo "ui-oracle: compiled $n oracle TUs + main.cpp + pc_bridge.cpp + stubs.c"
# shellcheck disable=SC2086
c++ $OBJS -lm -o "$B/ui_dump"
echo "ui-oracle: linked $B/ui_dump"

mkdir -p golden
status=0
run_one() {
	fixture="$1"; attempts="$2"; out="$3"
	if [ "${REGEN:-0}" = "1" ]; then
		"$B/ui_dump" "$fixture" "$attempts" >"golden/$out"
		echo "regenerated $out"
	else
		"$B/ui_dump" "$fixture" "$attempts" | diff -u "golden/$out" - || status=1
	fi
}

[ "${1:-}" = "--regen" ] && REGEN=1 || REGEN=0

run_one fixtures/retail.menu 1 retail.txt
run_one fixtures/all_menu_keywords.menu 1 all_menu_keywords.txt
run_one fixtures/broad_item_keywords.menu 1 broad_item_keywords.txt
run_one fixtures/edge_cases.menu 8 edge_cases.txt

[ "$status" -eq 0 ] && echo "ui-oracle: OK"
exit "$status"
