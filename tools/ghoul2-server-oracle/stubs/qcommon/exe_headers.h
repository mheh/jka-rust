// ghoul2-server-oracle stub (seeded from tools/trmodel-oracle) for codemp/qcommon/exe_headers.h
//
// Raven's real exe_headers.h is an MSVC precompiled-header umbrella pulling the
// whole engine closure. tr_model.cpp only needs q_shared + the engine seam, so
// this stub pulls exactly those two, mirroring the gp2-oracle exe_headers stub.
#ifndef GHOUL2_ORACLE_EXE_HEADERS_H
#define GHOUL2_ORACLE_EXE_HEADERS_H

#include "../game/q_shared.h"
#include "qcommon.h"

#endif // GHOUL2_ORACLE_EXE_HEADERS_H
