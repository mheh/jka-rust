/* cgame-oracle compile shim - force-included before every oracle TU.
 *
 * Same job as tools/referee-oracle/shim/oracle_shim.h: defuse Raven's
 * `float powf(float x, int y)` (q_shared.h:1242 / q_math.c:1476). Its
 * 2-arg-with-int signature conflicts with libm's `float powf(float, float)`
 * on a native host - a hard error in C ("conflicting types for powf"). We
 * include <math.h> FIRST (real powf declared behind its include guard), then
 * rename every subsequent `powf` token to `raven_powf` so Raven's declaration,
 * definition, and all callers move out of libm's way together. The oracle tree
 * is never touched.
 */
#include <math.h>
#define powf raven_powf

/* MSVC's <stdlib.h> defines min/max macros on the retail PC build; Raven only
 * declares its own under Q3_VM/_XBOX (q_shared.h:76-77,94-95), so on the normal
 * PC compile cg_players.c's bare `max(...)` resolves to the CRT macro. gcc's
 * stdlib has no such macro, so supply the MSVC-identical pair here. */
#ifndef max
#define max(a,b) (((a) > (b)) ? (a) : (b))
#endif
#ifndef min
#define min(a,b) (((a) < (b)) ? (a) : (b))
#endif
