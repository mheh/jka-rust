// Force-included first into every ICARUS oracle TU.
//
// Raven built these TUs behind an MSVC precompiled header (`exe_headers.h`) that
// had already pulled the STL and <string.h> in before any icarus header parsed.
// On a conforming compiler the icarus headers reference `string`/`map`/`list`/
// `vector` (and the C string fns) without including them, so we restore that
// ambient state here. No behaviour change — pure include-order restoration.
#ifndef ICARUS_ORACLE_PRELUDE_H
#define ICARUS_ORACLE_PRELUDE_H

#include <cstring>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <map>
#include <list>
#include <vector>
#include <algorithm>
using namespace std;

// icarus.h declares these at the very end, after blockstream.h's templates
// (CBlockMember::WriteData/WriteDataPointer) already reference them. GCC's
// two-phase template lookup needs the declaration visible at template-definition
// time, so hoist the two decls here (identical signatures to icarus.h:29-30).
extern void *ICARUS_Malloc(int iSize);
extern void  ICARUS_Free(void *pMem);

#endif
