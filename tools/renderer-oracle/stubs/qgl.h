/* Stub replacement for oracle/codemp/renderer/qgl.h.
 *
 * Raven's real qgl.h selects a platform header (windows.h+gl/gl.h,
 * macosx_glimp.h, GL/gl.h+GL/glx.h, ...) then declares every qgl* entry
 * point as a `(APIENTRY *)` function-pointer global (the classic Q3
 * "load GL at runtime" pattern) plus every GL/ARB/NV enum tr_*.cpp needs.
 *
 * This harness never renders — R_InitShaders/R_FindShader/ParseShader only
 * TOUCH the ARB vertex/fragment-program and NV register-combiner entry
 * points from CreateInternalShaders' glow-shader setup, and only behind a
 * `if (qglGenProgramsARB)` / `if (qglCombinerParameteriNV)` null check (see
 * README's "GL surface" section). Declaring these as extern globals that
 * main.cpp zero-initializes reproduces retail's "extension not loaded"
 * path deterministically, with none of the real qgl.h's platform-detection
 * (X11/GLX, win32 GDI, ...) or the ~200 unused entry points pulled in.
 *
 * `gl.h` here resolves to tools/closure-prototype/glshim/gl.h (on the
 * dumper's include path) for the base GLenum/GLuint/... typedefs.
 */
#ifndef __QGL_H__
#define __QGL_H__

#include "gl.h"

#ifndef APIENTRY
#define APIENTRY
#endif

// ARB vertex/fragment program + NV register-combiner enums
// (oracle/codemp/renderer/qgl.h:67-149,260,279-280 — values copied verbatim,
// this header never links against the real qgl.h/glext.h).
#define GL_TEXTURE0_ARB						0x84C0
#define GL_TEXTURE1_ARB						0x84C1
#define GL_TEXTURE2_ARB						0x84C2
#define GL_TEXTURE3_ARB						0x84C3
#define GL_COMBINER0_NV						0x8550
#define GL_COMBINER1_NV						0x8551
#define GL_NUM_GENERAL_COMBINERS_NV			0x854E
#define GL_VARIABLE_A_NV						0x8523
#define GL_VARIABLE_B_NV						0x8524
#define GL_VARIABLE_C_NV						0x8525
#define GL_VARIABLE_D_NV						0x8526
#define GL_DISCARD_NV						0x8530
#define GL_CONSTANT_COLOR0_NV				0x852A
#define GL_SPARE0_NV						0x852E
#define GL_SPARE1_NV						0x852F
#define GL_UNSIGNED_IDENTITY_NV				0x8536
#define GL_UNSIGNED_INVERT_NV				0x8537
#define GL_FRAGMENT_PROGRAM_ARB				0x8804
#define GL_VERTEX_PROGRAM_ARB				0x8620
#define GL_PROGRAM_FORMAT_ASCII_ARB			0x8875

// The exact qgl* entry points tr_shader.cpp references (grep "qgl[A-Za-z]*"
// over the oracle source); zero-initialized in main.cpp so every
// `if (qglFoo)` guard takes the "extension unavailable" branch.
extern void ( APIENTRY * qglActiveTextureARB )(GLenum texture);
extern void ( APIENTRY * qglGenProgramsARB )(GLsizei n, GLuint *programs);
extern void ( APIENTRY * qglBindProgramARB )(GLenum target, GLuint program);
extern void ( APIENTRY * qglProgramStringARB )(GLenum target, GLenum format, GLsizei len, const void *string);
extern void ( APIENTRY * qglGetIntegerv )(GLenum pname, GLint *params);
extern const GLubyte * ( APIENTRY * qglGetString )(GLenum name);
extern void ( APIENTRY * qglCombinerParameteriNV )(GLenum pname, GLint param);
extern GLuint ( APIENTRY * qglGenLists )(GLsizei range);
extern void ( APIENTRY * qglNewList )(GLuint list, GLenum mode);
extern void ( APIENTRY * qglEndList )(void);
extern void ( APIENTRY * qglCombinerInputNV )(GLenum stage, GLenum portion, GLenum variable, GLenum input, GLenum mapping, GLenum componentUsage);
extern void ( APIENTRY * qglCombinerOutputNV )(GLenum stage, GLenum portion, GLenum abOutput, GLenum cdOutput, GLenum sumOutput, GLenum scale, GLenum bias, GLboolean abDotProduct, GLboolean cdDotProduct, GLboolean muxSum);
extern void ( APIENTRY * qglFinalCombinerInputNV )(GLenum variable, GLenum input, GLenum mapping, GLenum componentUsage);

#endif // __QGL_H__
