#!/bin/sh
# icarus-oracle — differential golden harness for the ICARUS sequencer port
# (docs/subsystems/icarus.md § Verification strategy). Compiles the UNMODIFIED
# oracle codemp/icarus TUs standalone against the real oracle header closure,
# generates the committed .IBI fixture corpus with an ibi-gen tool, and dumps
# the three canonical golden streams. See README.md.
#
#   sh build.sh          build everything, diff dumps against goldens/
#   sh build.sh --regen  regenerate fixtures/*.IBI and goldens/*
#
# oracle/ is never edited. The oracle .cpp/.h are copied into build/ and a small
# set of documented, semantics-preserving portability normalisations are applied
# to the COPIES (see "normalise" below and README § Normalizations).
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-g++-16}"
ORACLE=../../oracle
IC=$ORACLE/codemp/icarus
REGEN=0
[ "${1:-}" = "--regen" ] && REGEN=1

# Oracle-parity flags shared by all builds (match tools/gp2-oracle / jampgame):
# signed char, no FP contraction / fast-math; -D__linux__ selects the POSIX
# platform branch of the real headers; -fpermissive downgrades two MSVC-isms
# (see normalise). DEDICATED-mode dumpers add -DDEDICATED at their compile.
STD="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -D__linux__"

rm -rf build
mkdir -p build/codemp/game build/codemp/qcommon build/codemp/server \
         build/codemp/icarus build/codemp/ghoul2 build/codemp/cgame

# Real oracle header closure (exe_headers.h -> q_shared.h + qcommon.h, plus
# g_public.h / server.h / RoffSystem.h / platform.h and the ghoul2/cgame trees
# they pull). Copy whole dirs' headers; copy the in-scope + generator .cpp.
cp "$ORACLE"/codemp/game/*.h     build/codemp/game/
cp "$ORACLE"/codemp/qcommon/*.h  build/codemp/qcommon/
cp "$ORACLE"/codemp/server/*.h   build/codemp/server/   2>/dev/null || true
cp "$ORACLE"/codemp/ghoul2/*.h   build/codemp/ghoul2/   2>/dev/null || true
cp "$ORACLE"/codemp/cgame/*.h    build/codemp/cgame/    2>/dev/null || true
cp "$ORACLE"/codemp/namespace_begin.h "$ORACLE"/codemp/namespace_end.h build/codemp/ 2>/dev/null || true
cp "$IC"/*.h build/codemp/icarus/
cp "$IC"/*.cpp build/codemp/icarus/

IB=build/codemp/icarus

# --- documented portability normalisations on the build COPIES only ---
# (1) BlockStream.cpp: two MSVC-isms + two LP64 platform-width fixes.
#   a. GetMember/Duplicate `return false;` -> `return 0;` (lines 333,367):
#      MSVC lets a bool `false` initialise a pointer return; conforming C++ does
#      not. false==0==NULL, so this is semantics-identical.
#   b. ReadMember (lines 110,117,118): the size field is WRITTEN as int(4) by
#      WriteMember but READ back through `*(long*)` / `sizeof(long)`. On the
#      32-bit ship long==4 (self-consistent); on this LP64 host long==8 would
#      misparse. Normalise long->int (exactly the Rust port's i32 model, matching
#      the 32-bit ship & the fixtures) — porting-rules §19 platform-UB fix.
perl -i -pe 's/return false;/return 0;/ if $. == 333 || $. == 367;
             s/\*\(long \*\)/*(int *)/g if $. == 117;
             s/sizeof\( long \)/sizeof( int )/g if $. == 110 || $. == 118;' "$IB/BlockStream.cpp"
#   c. Create() header write: `fwrite(id_header,1,sizeof(id_header),...)` writes
#      sizeof(char*) bytes — 4 on the 32-bit ship (== sizeof("IBI")), 8 here.
#      The reader Open() expects a 4-byte "IBI\0" header, so pin the 32-bit ship
#      width. Writer is generator-only (§20-dropped from the port).
perl -i -pe 's/sizeof\(id_header\)/sizeof(IBI_HEADER_ID)/ if $. == 546;' "$IB/BlockStream.cpp"

# (1d) tokenizer.h: the committed oracle header's CSymbolLookup class is missing
#     two accessor declarations (GetChild / GetChildAddress) that Tokenizer.cpp
#     both DEFINES (:2824,:2829) and USES (:1917,:2476,:2486) — an oracle
#     header/impl mismatch that no conforming compiler accepts. Restore exactly
#     those two declarations (signatures taken verbatim from the .cpp defs).
#     Generator-only surface; the ported reader/registers TUs never touch it.
perl -i -pe 's/(CSymbolLookup\* GetParent\(\);)/$1\n\tCSymbolLookup** GetChildAddress();\n\tCSymbolLookup* GetChild();/' "$IB/tokenizer.h"

# (2) GameInterface.h: entlist_t/bufferlist_t pass an explicit `allocator<int>` /
#     `allocator<pscript_t*>` (value_type != pair) — an MSVC-STL laxity modern
#     libstdc++ rejects. Drop the explicit less/allocator args; the default
#     allocator yields an identical container.
perl -i -pe 's/,\s*less<string>\s*,\s*allocator<[^>]*>\s*>/>/g' "$IB/GameInterface.h"

echo "normalised oracle copies."

INC="-Ibuild/codemp/icarus -Ibuild/codemp/game -Ibuild/codemp/qcommon -Ibuild/codemp/server"
PRE="-include stubs/prelude.h"

# ---------------------------------------------------------------------------
# ibi-gen: the fixture compiler (icarus.md ruling 14). Built from the OUT-OF-SET
# oracle Interpreter.cpp + Tokenizer.cpp (permitted in the generator ONLY) plus
# the in-scope BlockStream writer half and Memory.cpp. Compiles hand-authored
# .icarus scripts into committed .IBI blobs. NOT part of the ported scope.
# ---------------------------------------------------------------------------
echo "building ibi-gen..."
$CXX $STD $INC $PRE -Istubs -include stubs/win32stub.h \
     -o build/ibigen \
     ibigen.cpp \
     "$IB/Interpreter.cpp" "$IB/Tokenizer.cpp" "$IB/BlockStream.cpp" "$IB/Memory.cpp" \
     engine_stubs_core.cpp

if [ "$REGEN" -eq 1 ]; then
	for s in fixtures/*.icarus; do
		b=$(basename "$s" .icarus)
		build/ibigen "$s" "fixtures/$b.IBI"
		echo "  compiled $b.icarus -> $b.IBI ($(wc -c < fixtures/$b.IBI) bytes)"
	done
fi

# ---------------------------------------------------------------------------
# Golden dumpers (compiled against the in-scope reader/registers TUs only).
# ---------------------------------------------------------------------------
echo "building dumpers..."
$CXX $STD -DDEDICATED $INC $PRE -o build/dump_blockstream \
     dump_blockstream.cpp "$IB/BlockStream.cpp" "$IB/Memory.cpp" engine_stubs_core.cpp
$CXX $STD -DDEDICATED $INC $PRE -o build/dump_registers \
     dump_registers.cpp "$IB/Q3_Registers.cpp" "$IB/BlockStream.cpp" "$IB/Memory.cpp" engine_stubs_core.cpp

# End-to-end: the full sequencer stack (all 10 in-scope TUs) + the MockHost.
# engine_stubs_core is NOT linked here — GameInterface/Q3_Interface provide the
# real Q3_DebugPrint/Com_* seam, and mockhost.cpp supplies the engine services.
$CXX $STD -DDEDICATED $INC $PRE -o build/dump_endtoend \
     dump_endtoend.cpp mockhost.cpp \
     "$IB/GameInterface.cpp" "$IB/Q3_Interface.cpp" "$IB/Q3_Registers.cpp" \
     "$IB/Sequencer.cpp" "$IB/Sequence.cpp" "$IB/TaskManager.cpp" "$IB/Instance.cpp" \
     "$IB/BlockStream.cpp" "$IB/Memory.cpp" "$IB/Interface.cpp"

run_or_check() { # $1 = golden basename, shift; rest = command
	name="$1"; shift
	if [ "$REGEN" -eq 1 ]; then
		"$@" > "goldens/$name"
		echo "  regenerated goldens/$name"
	else
		"$@" | diff -u "goldens/$name" - || { echo "MISMATCH: $name"; STATUS=1; }
	fi
}

STATUS=0
run_or_check q3_registers.txt build/dump_registers
for f in fixtures/*.IBI; do
	b=$(basename "$f" .IBI)
	run_or_check "blockstream_$b.txt" build/dump_blockstream "$f"
done
run_or_check endtoend_e2e.txt build/dump_endtoend fixtures/e2e

[ "$STATUS" -eq 0 ] && echo "icarus-oracle: OK"
exit "$STATUS"
