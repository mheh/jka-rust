/* Parse-only shim: MP qgl.h includes qgl_linked.h on non-win/mac/linux,
 * but only the SP tree ships it. Raven qgl_linked.h just #defines qgl* to
 * gl*; nothing at header scope needs those macros. */
#pragma once
