// Stub for MSVC <direct.h>, included by the out-of-scope oracle Interpreter.cpp
// (fixture-generator only). Interpreter.cpp uses it solely for getcwd() in its
// Error() path; route to the POSIX getcwd.
#pragma once
#include <unistd.h>
