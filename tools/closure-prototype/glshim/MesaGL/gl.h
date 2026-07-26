/* Parse-only GL typedef shim for closure.py/sweep.py on macOS.
 * Raven's qgl.h/glext.h need only the fixed-width GL scalar typedefs to
 * declare types; no GL prototypes or tokens are required at header scope.
 * GL_VERSION_1_2 is deliberately NOT defined so qgl.h supplies its own
 * PFN* typedefs.
 *
 * R1 (2026-07-25): the mp-renderer srcglob now sweeps full `tr_*.cpp` FUNCTION
 * BODIES (previously only tr_local.h et al. were parsed for types), and
 * bodies reference GL_* enum VALUES directly (`image->wrapClampMode =
 * GL_CLAMP;`) — hence the block below. Standard OpenGL 1.1 core token values
 * (stable/public since the 1992 spec); parse-only, never linked, so only
 * needs to exist as a valid integer constant, not be exhaustive. Covers every
 * GL_* name the mp-renderer body sweep currently references, grouped as the
 * real gl.h does, plus common neighbors for headroom. */
#ifndef __gl_h_
#define __gl_h_

typedef unsigned int GLenum;
typedef unsigned char GLboolean;
typedef unsigned int GLbitfield;
typedef signed char GLbyte;
typedef short GLshort;
typedef int GLint;
typedef int GLsizei;
typedef unsigned char GLubyte;
typedef unsigned short GLushort;
typedef unsigned int GLuint;
typedef float GLfloat;
typedef float GLclampf;
typedef double GLdouble;
typedef double GLclampd;
typedef void GLvoid;

/* Boolean */
#define GL_FALSE                          0x0
#define GL_TRUE                           0x1

/* Primitives */
#define GL_POINTS                         0x0000
#define GL_LINES                          0x0001
#define GL_LINE_LOOP                      0x0002
#define GL_LINE_STRIP                     0x0003
#define GL_TRIANGLES                      0x0004
#define GL_TRIANGLE_STRIP                 0x0005
#define GL_TRIANGLE_FAN                   0x0006
#define GL_QUADS                          0x0007
#define GL_QUAD_STRIP                     0x0008
#define GL_POLYGON                        0x0009

/* Matrix modes */
#define GL_MODELVIEW                      0x1700
#define GL_PROJECTION                     0x1701
#define GL_TEXTURE                        0x1702

/* Polygon/point/line raster modes */
#define GL_POINT                          0x1B00
#define GL_LINE                           0x1B01
#define GL_FILL                           0x1B02

/* glClear mask bits */
#define GL_DEPTH_BUFFER_BIT               0x00000100
#define GL_ACCUM_BUFFER_BIT               0x00000200
#define GL_STENCIL_BUFFER_BIT             0x00000400
#define GL_COLOR_BUFFER_BIT               0x00004000

/* Depth/alpha/stencil comparison funcs */
#define GL_NEVER                          0x0200
#define GL_LESS                           0x0201
#define GL_EQUAL                          0x0202
#define GL_LEQUAL                         0x0203
#define GL_GREATER                        0x0204
#define GL_NOTEQUAL                       0x0205
#define GL_GEQUAL                         0x0206
#define GL_ALWAYS                         0x0207

/* Blend factors */
#define GL_ZERO                           0
#define GL_ONE                            1
#define GL_SRC_COLOR                      0x0300
#define GL_ONE_MINUS_SRC_COLOR            0x0301
#define GL_SRC_ALPHA                      0x0302
#define GL_ONE_MINUS_SRC_ALPHA            0x0303
#define GL_DST_ALPHA                      0x0304
#define GL_ONE_MINUS_DST_ALPHA            0x0305
#define GL_DST_COLOR                      0x0306
#define GL_ONE_MINUS_DST_COLOR            0x0307
#define GL_SRC_ALPHA_SATURATE             0x0308

/* Enable/Disable capabilities */
#define GL_CULL_FACE                      0x0B44
#define GL_FOG                            0x0B60
#define GL_DEPTH_TEST                     0x0B71
#define GL_STENCIL_TEST                   0x0B90
#define GL_ALPHA_TEST                     0x0BC0
#define GL_BLEND                          0x0BE2
#define GL_SCISSOR_TEST                   0x0C11
#define GL_TEXTURE_2D                     0x0DE1
#define GL_POLYGON_OFFSET_FILL            0x8037
#define GL_CLIP_PLANE0                    0x3000
#define GL_VERTEX_ARRAY                   0x8074
#define GL_COLOR_ARRAY                    0x8076
#define GL_TEXTURE_COORD_ARRAY            0x8078

/* Face */
#define GL_FRONT                          0x0404
#define GL_BACK                           0x0405
#define GL_FRONT_AND_BACK                 0x0408
#define GL_BACK_LEFT                      0x0402
#define GL_BACK_RIGHT                     0x0403

/* Stencil ops */
#define GL_KEEP                           0x1E00
#define GL_REPLACE                        0x1E01
#define GL_INCR                           0x1E02
#define GL_DECR                           0x1E03

/* Data types */
#define GL_UNSIGNED_BYTE                  0x1401
#define GL_UNSIGNED_INT                   0x1405
#define GL_FLOAT                          0x1406

/* Texture parameters */
#define GL_TEXTURE_MAG_FILTER             0x2800
#define GL_TEXTURE_MIN_FILTER             0x2801
#define GL_TEXTURE_WRAP_S                 0x2802
#define GL_TEXTURE_WRAP_T                 0x2803
#define GL_TEXTURE_BORDER_COLOR           0x1004
#define GL_NEAREST                        0x2600
#define GL_LINEAR                         0x2601
#define GL_NEAREST_MIPMAP_NEAREST         0x2700
#define GL_LINEAR_MIPMAP_NEAREST          0x2701
#define GL_NEAREST_MIPMAP_LINEAR          0x2702
#define GL_LINEAR_MIPMAP_LINEAR           0x2703
#define GL_CLAMP                          0x2900
#define GL_REPEAT                         0x2901

/* Texture environment */
#define GL_TEXTURE_ENV_MODE               0x2200
#define GL_TEXTURE_ENV                    0x2300
#define GL_MODULATE                       0x2100
#define GL_DECAL                          0x2101
#define GL_ADD                            0x0104
#define GL_BLEND_TEXENV                   0x0BE2

/* Fog */
#define GL_FOG_MODE                       0x0B65
#define GL_FOG_DENSITY                    0x0B62
#define GL_FOG_START                      0x0B63
#define GL_FOG_END                        0x0B64
#define GL_FOG_COLOR                      0x0B66
#define GL_EXP2                           0x0801

/* Errors (glGetError) */
#define GL_NO_ERROR                       0
#define GL_INVALID_ENUM                   0x0500
#define GL_INVALID_VALUE                  0x0501
#define GL_INVALID_OPERATION              0x0502
#define GL_STACK_OVERFLOW                 0x0503
#define GL_STACK_UNDERFLOW                0x0504
#define GL_OUT_OF_MEMORY                  0x0505

/* Pixel/internal formats */
#define GL_STENCIL_INDEX                  0x1901
#define GL_DEPTH_COMPONENT                0x1902
#define GL_RGB                            0x1907
#define GL_RGBA                           0x1908
#define GL_RGB5                           0x8050
#define GL_RGB8                           0x8051
#define GL_RGBA4                          0x8056
#define GL_RGBA8                          0x8058
#define GL_RGBA16                         0x805B

/* Misc */
#define GL_NONE                           0
#define GL_COMPILE                        0x1300

#endif
