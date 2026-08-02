#!/bin/sh
# fx-oracle - differential golden harness for the MP FX port (DEC-61.5,
# wayfinder ticket gh#26). Compiles the UNMODIFIED Raven FX TUs
# (codemp/client/FxSystem.cpp, FxScheduler.cpp, FxPrimitives.cpp,
# FxTemplate.cpp, FxUtil.cpp, FXExport.cpp) against harness stubs, links them
# with a deterministic host, and builds the `fx_dump` driver. run.sh drives the
# scenarios.
#
#   sh build.sh    build build/fx_dump
#
# oracle/ is never edited. The oracle sources are COPIED into build/ so their
# relative #includes resolve, and four header names are replaced by stubs:
# client.h, exe_headers.h and G2_local.h, each of which drags a whole subsystem
# into every FX TU. README.md lists every normalisation applied to a copy.
set -eu
cd "$(dirname "$0")"

CXX="${CXX:-clang++}"
ORACLE=../../oracle

# Oracle-parity flags shared with tools/snd-oracle and tools/gp2-oracle: signed
# char, no FP contraction, no fast math. `-D__linux__` selects Raven's POSIX
# branch in q_shared.h, which is the little-endian branch that matches the
# shipped x86 build. `-fpermissive` downgrades the LP64 pointer-width warnings.
# `-ffp-contract=off` is load-bearing: an FMA anywhere in the update chain would
# change the float bits every golden carries.
FLAGS="-std=c++14 -w -fpermissive -fsigned-char -ffp-contract=off -fno-fast-math -D__linux__ -DFINAL_BUILD -DNDEBUG"

rm -rf build
mkdir -p build/codemp/client build/codemp/qcommon build/codemp/game \
	build/codemp/cgame build/codemp/ghoul2 build/inc

# Unmodified oracle TUs.
cp "$ORACLE/codemp/client/FxSystem.cpp" "$ORACLE/codemp/client/FxScheduler.cpp" \
	"$ORACLE/codemp/client/FxPrimitives.cpp" "$ORACLE/codemp/client/FxTemplate.cpp" \
	"$ORACLE/codemp/client/FxUtil.cpp" "$ORACLE/codemp/client/FXExport.cpp" \
	build/codemp/client/
# The .efx parser.
cp "$ORACLE/codemp/qcommon/GenericParser2.cpp" "$ORACLE/codemp/qcommon/GenericParser2.h" \
	build/codemp/qcommon/
# The shared math and string helpers, so flrand, irand, COM_StripExtension and
# the vector math stay faithful.
cp "$ORACLE/codemp/game/q_shared.c" "$ORACLE/codemp/game/q_math.c" build/codemp/game/

# Unmodified oracle headers.
cp "$ORACLE"/codemp/client/Fx*.h "$ORACLE"/codemp/client/FX*.h build/codemp/client/
cp "$ORACLE"/codemp/game/q_shared.h "$ORACLE"/codemp/game/bg_lib.h \
	"$ORACLE"/codemp/game/teams.h "$ORACLE"/codemp/game/surfaceflags.h \
	build/codemp/game/
cp "$ORACLE"/codemp/qcommon/disablewarnings.h "$ORACLE"/codemp/qcommon/tags.h \
	build/codemp/qcommon/
cp "$ORACLE"/codemp/cgame/tr_types.h "$ORACLE"/codemp/cgame/cg_public.h build/codemp/cgame/
cp "$ORACLE"/codemp/ghoul2/G2.h build/codemp/ghoul2/

# Harness stubs. Each replaces an oracle header that would otherwise pull a
# whole subsystem into every FX TU.
cp stubs/client.h build/codemp/client/client.h
cp stubs/exe_headers.h build/codemp/qcommon/exe_headers.h
cp stubs/G2_local.h build/codemp/ghoul2/G2_local.h
cp stubs/win_shim.h build/inc/win_shim.h

# --- normalisations on the COPIES only -------------------------------------
# oracle/ stays untouched. README.md carries the justification for each.

# 1. FxPrimitives.cpp:423-451 - `VectorToInt` is an MSVC 32-bit x87 `_asm`
#    block. Clang compiles it on no target we have. The replacement body is
#    bit-identical: x87 `fistp` rounds half to even under the default control
#    word, and the pack order and the 0xff alpha byte match the assembly.
perl -0777 -i -pe 's/\t_asm\r?\n\t\{\r?\n.*?\r?\n\t\}\r?\n/\ttmp = 0;\n\t(void)tmp;\n\tint r = (int)nearbyintf(vec[0]) \& 0xff;\n\tint g = (int)nearbyintf(vec[1]) \& 0xff;\n\tint b = (int)nearbyintf(vec[2]) \& 0xff;\n\tretval = (int)(0xff000000u | ((unsigned)b << 16) | ((unsigned)g << 8) | (unsigned)r);\n/s' build/codemp/client/FxPrimitives.cpp
grep -q "nearbyintf" build/codemp/client/FxPrimitives.cpp || { echo "fx-oracle: the VectorToInt normalisation did not apply"; exit 1; }

# 2. q_math.c:1432 - `holdrand` is `unsigned long`, 32 bit on the ship and 64
#    bit under LP64. The LCG must stay 32 bit or every flrand draw diverges.
perl -i -pe 's/^static unsigned long\tholdrand = 0x89abcdef;/static unsigned int\tholdrand = 0x89abcdef;/' build/codemp/game/q_math.c

# 3. FxSystem.h:218 and FxPrimitives.h:310,315 - MSVC accepts an extra class
#    qualification on an in-class declaration. No conforming compiler does. The
#    copies drop the redundant `SFxHelper::` and `CParticle::` prefixes, which
#    changes nothing else about the declarations.
perl -i -pe 's/\tqboolean SFxHelper::GetOriginAxisFromBolt\(/\tqboolean GetOriginAxisFromBolt(/' build/codemp/client/FxSystem.h
perl -i -pe 's/\tinline CParticle::CParticle\(void\)/\tinline CParticle(void)/; s/\tvirtual CParticle::~CParticle\(void\)/\tvirtual ~CParticle(void)/' build/codemp/client/FxPrimitives.h

# 4. FxTemplate.cpp:2335 - `strcpy( mName, val )` writes an unbounded name into
#    a 32-byte field. The fixtures keep every name short, and the copy bounds
#    the write so a fixture typo cannot corrupt the template next door.
perl -i -pe 's/\t\t\t\tstrcpy\( mName, val \);/\t\t\t\tQ_strncpyz( mName, val, sizeof( mName ) );/' build/codemp/client/FxTemplate.cpp

INC="-Ibuild/inc -Ibuild/codemp/client -Ibuild/codemp/qcommon -Ibuild/codemp/game -I."
# The Win32 shim leads every translation unit, the way MSVC's forced include did.
FLAGS="$FLAGS -include build/inc/win_shim.h"

echo "compiling oracle TUs..."
for tu in FxSystem FxScheduler FxPrimitives FxTemplate FxUtil FXExport; do
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
$CXX $FLAGS -Wl,-dead_strip -o build/fx_dump \
	build/main.o build/host.o build/FxSystem.o build/FxScheduler.o \
	build/FxPrimitives.o build/FxTemplate.o build/FxUtil.o build/FXExport.o \
	build/GenericParser2.o build/q_shared.o build/q_math.o

echo "fx-oracle: built build/fx_dump"
