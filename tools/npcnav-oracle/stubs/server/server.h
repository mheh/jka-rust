// npcnav-oracle stub for oracle/codemp/server/server.h.
// navigator.h does `#include "../server.h"`; the real header drags in the whole
// server/qcommon/game surface. The TU only needs the shared types (from the
// game/q_shared.h stub) plus the SV_*/VM_Call externs, which live there.
#ifndef NPCNAV_ORACLE_SERVER_STUB
#define NPCNAV_ORACLE_SERVER_STUB

#include "../game/q_shared.h"

// VM dispatch. gvm is the game VM handle; VM_Call is variadic in Raven usage
// (GNavCallback_* pass ints/pointers). Only the game-callback path reaches it,
// and every GNavCallback_* is a no-op stub in main.cpp, so this is unused at
// runtime — declared for compilation only.
typedef int vm_t;
extern vm_t *gvm;
int VM_Call( vm_t *vm, int callnum, ... );

// GAME_NAV_* ids referenced by gameCallbacks.cpp — NOT compiled here (the
// callbacks are stubbed in main.cpp), listed as a no-op guard in case a future
// build links gameCallbacks.cpp.
enum {
    GAME_NAV_CLEARPATHTOPOINT = 400,
    GAME_NAV_CLEARLOS,
    GAME_NAV_CLEARPATHBETWEENPOINTS,
    GAME_NAV_CHECKNODEFAILEDFORENT,
    GAME_NAV_ENTISUNLOCKEDDOOR,
    GAME_NAV_ENTISDOOR,
    GAME_NAV_ENTISBREAKABLE,
    GAME_NAV_ENTISREMOVABLEUSABLE,
    GAME_NAV_FINDCOMBATPOINTWAYPOINTS
};

#endif
