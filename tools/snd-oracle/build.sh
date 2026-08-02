#!/bin/sh
# snd-oracle — differential golden harness for the MP sound port (DEC-57.2,
# wayfinder ticket gh#23). Compiles the UNMODIFIED Raven sound TUs
# (codemp/client/snd_dma.cpp, snd_mem.cpp, snd_mix.cpp, snd_music.cpp,
# snd_ambient.cpp) against harness stubs, links them with a deterministic host,
# and builds the `snd_dump` driver. run.sh drives the scenarios.
#
#   sh build.sh    build build/snd_dump
#
# oracle/ is never edited. The oracle sources are COPIED into build/ so their
# relative #includes resolve, and only four header names are replaced by stubs:
# the OpenAL and EAX headers that DEC-57.4 drops.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-clang++}"
ORACLE=../../oracle

# Oracle-parity flags shared with tools/gp2-oracle and tools/ghoul2-server-oracle:
# signed char, no FP contraction, no fast math. `-D__linux__` selects Raven's
# POSIX branch in q_shared.h, which is the little-endian branch that matches the
# shipped x86 build: `LittleShort` is a no-op there and `BigShort` swaps. The
# MACOS_X branch has those two the other way round, because it targeted PPC.
# `-fpermissive` downgrades the LP64 pointer-width warnings.
FLAGS="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -D__linux__ -DFINAL_BUILD -DNDEBUG"

rm -rf build
mkdir -p build/codemp/client build/codemp/qcommon build/codemp/game \
	build/codemp/renderer build/codemp/ui build/codemp/cgame \
	build/codemp/mp3code build/inc

# Unmodified oracle TUs.
cp "$ORACLE/codemp/client/snd_dma.cpp" "$ORACLE/codemp/client/snd_mem.cpp" \
	"$ORACLE/codemp/client/snd_mix.cpp" "$ORACLE/codemp/client/snd_music.cpp" \
	"$ORACLE/codemp/client/snd_ambient.cpp" build/codemp/client/
cp "$ORACLE/codemp/qcommon/GenericParser2.cpp" build/codemp/qcommon/
# The shared string and math helpers come from the oracle too, so name handling
# (COM_DefaultExtension, Q_strncpyz, Q_stricmp) stays faithful.
cp "$ORACLE/codemp/game/q_shared.c" "$ORACLE/codemp/game/q_math.c" build/codemp/game/

# Unmodified oracle headers, whole directories, so every relative include lands.
cp "$ORACLE"/codemp/client/*.h    build/codemp/client/
cp "$ORACLE"/codemp/qcommon/*.h   build/codemp/qcommon/
cp "$ORACLE"/codemp/game/*.h      build/codemp/game/
cp "$ORACLE"/codemp/renderer/*.h  build/codemp/renderer/
cp "$ORACLE"/codemp/ui/*.h        build/codemp/ui/
cp "$ORACLE"/codemp/cgame/*.h     build/codemp/cgame/
cp "$ORACLE"/codemp/mp3code/*.h   build/codemp/mp3code/
cp "$ORACLE"/codemp/namespace_*.h build/codemp/

# Harness stubs for the dropped OpenAL/EAX arm. Raven writes these includes with
# a Windows path separator (snd_local.h:12-15), so the copies keep the backslash
# in the file name. The repo stores them under portable names.
mkdir -p build/inc
cp stubs/openal_al.h     'build/inc/openal\al.h'
cp stubs/openal_alc.h    'build/inc/openal\alc.h'
cp stubs/eax_eax.h       'build/inc/eax\eax.h'
cp stubs/eax_eaxman.h    'build/inc/eax\eaxman.h'
cp stubs/win_shim.h      build/inc/win_shim.h

# Normalisations on the COPIES only, in the tools/icarus-oracle style. Raven
# built with MSVC, whose `for (int i=...)` leaks `i` into the enclosing scope.
# Two later loops reuse the leaked name, which no conforming compiler accepts.
# Both edits hoist the declaration and change no behaviour, and neither site is
# on a golden path (S_StartBackgroundTrack, SND_FreeOldestSound).
perl -i -pe 's/for \(i=0; i<eBGRNDTRACK_NUMBEROF; i\+\+\)/for (int i=0; i<eBGRNDTRACK_NUMBEROF; i++)/' build/codemp/client/snd_dma.cpp
perl -i -pe 's/for \(int iChannel=0; iChannel<MAX_CHANNELS; iChannel\+\+\)/int iChannel; for (iChannel=0; iChannel<MAX_CHANNELS; iChannel++)/' build/codemp/client/snd_dma.cpp
# snd_mem.cpp:98 casts a pointer to int. That is exact on the 32-bit ship and
# lossy here, so the copy widens the cast. The line is inside DumpChunks, which
# only runs under `s_show`, and no golden ever prints an address.
perl -i -pe 's/\(int\)\(data_p - 4\)/(int)(size_t)(data_p - 4)/' build/codemp/client/snd_mem.cpp

INC="-Ibuild/inc -Ibuild/codemp/client -Ibuild/codemp/qcommon -Ibuild/codemp/game -I."
# The Win32 shim leads every translation unit, the way MSVC's forced include did.
FLAGS="$FLAGS -include build/inc/win_shim.h"

echo "compiling oracle TUs..."
for tu in snd_dma snd_mem snd_mix snd_music snd_ambient; do
	$CXX $FLAGS $INC -c "build/codemp/client/$tu.cpp" -o "build/$tu.o"
done
$CXX $FLAGS $INC -c build/codemp/qcommon/GenericParser2.cpp -o build/GenericParser2.o
$CXX $FLAGS $INC -c build/codemp/game/q_shared.c -o build/q_shared.o
$CXX $FLAGS $INC -c build/codemp/game/q_math.c -o build/q_math.o

echo "compiling harness..."
$CXX $FLAGS $INC -c host.cpp -o build/host.o
$CXX $FLAGS $INC -c main.cpp -o build/main.o

echo "linking..."
# -Wl,-dead_strip drops every function the driver never reaches, so only the
# live-path engine-seam symbols (host.cpp) need a body.
$CXX $FLAGS -Wl,-dead_strip -o build/snd_dump \
	build/main.o build/host.o build/snd_dma.o build/snd_mem.o build/snd_mix.o \
	build/snd_music.o build/snd_ambient.o build/GenericParser2.o \
	build/q_shared.o build/q_math.o

echo "snd-oracle: built build/snd_dump"
