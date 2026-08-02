#!/bin/sh
# cin-oracle: differential golden harness for the MP RoQ cinematic decoder
# (DEC-55.3, wayfinder ticket gh#28). Compiles the UNMODIFIED Raven TU
# codemp/client/cl_cin.cpp against harness stubs, links it with a deterministic
# host, and builds the `cin_dump` driver. run.sh drives the scenarios.
#
#   sh build.sh    build build/cin_dump
#
# oracle/ is never edited. The oracle sources are COPIED into build/ so their
# relative #includes resolve, and only the four OpenAL and EAX header names that
# snd_local.h reaches are replaced by stubs.
#
# main.cpp #includes the copied cl_cin.cpp, because every gated function is
# `static` in that TU. The driver therefore compiles as one translation unit
# with the oracle text.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-clang++}"
ORACLE=../../oracle

# Oracle-parity flags shared with tools/snd-oracle, tools/gp2-oracle and
# tools/ghoul2-server-oracle: signed char, no FP contraction, no fast math.
# `-D__linux__` selects Raven's POSIX branch in q_shared.h, which is the
# little-endian branch that matches the shipped x86 build. The MACOS_X branch
# would take `cl_cin.cpp`'s other `yuv_to_rgb24` body and its `drawX` typo arm,
# neither of which the PC build compiles. `-fpermissive` downgrades the LP64
# pointer-width warnings.
FLAGS="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -D__linux__ -DFINAL_BUILD -DNDEBUG"

rm -rf build
mkdir -p build/codemp/client build/codemp/qcommon build/codemp/game \
	build/codemp/renderer build/codemp/ui build/codemp/cgame \
	build/codemp/mp3code build/inc

# The unmodified oracle TU.
cp "$ORACLE/codemp/client/cl_cin.cpp" build/codemp/client/

# Unmodified oracle headers, whole directories, so every relative include lands.
cp "$ORACLE"/codemp/client/*.h    build/codemp/client/
cp "$ORACLE"/codemp/qcommon/*.h   build/codemp/qcommon/
cp "$ORACLE"/codemp/game/*.h      build/codemp/game/
cp "$ORACLE"/codemp/renderer/*.h  build/codemp/renderer/
cp "$ORACLE"/codemp/ui/*.h        build/codemp/ui/
cp "$ORACLE"/codemp/cgame/*.h     build/codemp/cgame/
cp "$ORACLE"/codemp/mp3code/*.h   build/codemp/mp3code/
cp "$ORACLE"/codemp/namespace_*.h build/codemp/

# Harness stubs for the OpenAL and EAX arm that snd_local.h includes. Raven
# writes those includes with a Windows path separator (snd_local.h:12-15), so
# the copies keep the backslash in the file name. The repo stores them under
# portable names.
cp stubs/openal_al.h     'build/inc/openal\al.h'
cp stubs/openal_alc.h    'build/inc/openal\alc.h'
cp stubs/eax_eax.h       'build/inc/eax\eax.h'
cp stubs/eax_eaxman.h    'build/inc/eax\eaxman.h'
cp stubs/win_shim.h      build/inc/win_shim.h

# Normalisations on the COPY only, in the tools/snd-oracle style. Both restore
# the shipped 32-bit build's arithmetic on an LP64 host; see README.md.
#
# 1. cl_cin.cpp:806-807 casts `cin.linbuf` to `unsigned int`. That is exact on
#    the 32-bit ship and a hard error under LP64, so the copy routes the cast
#    through `size_t`. The surrounding algebra cancels the address either way,
#    so `t[0]` stays `screenDelta` and `t[1]` stays `-screenDelta`.
perl -i -pe 's/\(unsigned int\)cin\.linbuf/(unsigned int)(size_t)cin.linbuf/g' build/codemp/client/cl_cin.cpp
# 2. cl_cin.cpp:515,523 add `cin.mcomp[...]`, an `unsigned int` holding a signed
#    delta, to a `byte *`. On the 32-bit ship that wraps and walks backwards. An
#    LP64 pointer zero-extends instead and walks off the surface, so the copy
#    casts to `int` first. The Rust port spells the same `as i32 as isize` step.
perl -i -pe 's/status\[index\] \+ cin\.mcomp\[\(\*data\)\]/status[index] + (int)cin.mcomp[(*data)]/g' build/codemp/client/cl_cin.cpp
# 3. cl_cin.cpp:1197 calls `abs` on an `unsigned int` difference. MSVC resolves
#    that to `abs(int)`; libc++ finds the float overloads too and calls it
#    ambiguous. The copy spells the MSVC choice. Site: CIN_RunCinematic, which
#    is outside the byte gate and off every golden path.
perl -i -pe 's/abs\(thisTime - cinTable\[currentHandle\]\.lastTime\)/abs((int)(thisTime - cinTable[currentHandle].lastTime))/' build/codemp/client/cl_cin.cpp

INC="-Ibuild -Ibuild/inc -Ibuild/codemp/client -Ibuild/codemp/qcommon -Ibuild/codemp/game -I."
# The Win32 shim leads every translation unit, the way MSVC's forced include did.
FLAGS="$FLAGS -include build/inc/win_shim.h"

echo "compiling harness..."
$CXX $FLAGS $INC -c host.cpp -o build/host.o
echo "compiling the oracle TU inside the driver..."
$CXX $FLAGS $INC -c main.cpp -o build/main.o

echo "linking..."
# -Wl,-dead_strip drops every function the driver never reaches, so only the
# live-path engine-seam symbols (host.cpp) need a real body.
$CXX $FLAGS -Wl,-dead_strip -o build/cin_dump build/main.o build/host.o

echo "cin-oracle: built build/cin_dump"
