/* referee-oracle compile shim — force-included before every oracle TU.
 *
 * The ONLY job here is to defuse Raven's `float powf(float x, int y)`
 * (q_shared.h:1242 / q_math.c:1476), whose 2-arg-with-int signature conflicts
 * with libm's `float powf(float, float)` on a native host. We include <math.h>
 * FIRST (so the real powf is declared under its true name behind its include
 * guard), then rename every subsequent `powf` token to `raven_powf` — Raven's
 * declaration, definition, and all callers move out of libm's way together.
 * The oracle tree is never touched.
 */
#include <math.h>
#define powf raven_powf
