/* Parse-only GL typedef shim for closure.py/sweep.py on macOS.
 * Raven's qgl.h/glext.h need only the fixed-width GL scalar typedefs to
 * declare types; no GL prototypes or tokens are required at header scope.
 * GL_VERSION_1_2 is deliberately NOT defined so qgl.h supplies its own
 * PFN* typedefs. */
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

#endif
