// stringed-oracle — deterministic harness host.
//
// Implements the engine services the two unmodified StringEd TUs call
// (declared in stubs/qcommon/qcommon.h + stubs/game/q_shared.h): an in-memory
// cvar registry and a fixture-backed virtual filesystem, plus the va/Q_*/Z_*
// helpers. This is the C++-side twin of the Rust MockHost (RULING 32/55): the
// cvar STRING is the single source of truth and `integer` derives from it via
// atoi, so cvar_register/cvar_string/cvar_integer/cvar_take_modified stay
// mutually consistent under test (docs/subsystems/stringed.md, SE-D3).
//
// Determinism: FS_ListFiles SORTS its results (readdir order is FS-dependent),
// so the file-list scan (Golden C) is run-twice byte-identical. oracle/ is
// never edited.

#include "stubs/game/q_shared.h"
#include "stubs/qcommon/qcommon.h"

#include <cstdarg>
#include <string>
#include <vector>
#include <map>
#include <algorithm>
#include <dirent.h>
#include <sys/stat.h>

// --------------------------------------------------------------------------
// fixture root
// --------------------------------------------------------------------------
static std::string g_root = "fixtures";
void Host_SetFixtureRoot(const char *root) { g_root = root; }

static std::string joinPath(const std::string &a, const std::string &b) {
    if (a.empty()) return b;
    if (a[a.size() - 1] == '/') return a + b;
    return a + "/" + b;
}

// --------------------------------------------------------------------------
// cvar registry — string is the single source of truth, integer = atoi(string)
// --------------------------------------------------------------------------
static std::map<std::string, cvar_t *> g_cvars;

static cvar_t *cvarLookup(const char *name) {
    std::map<std::string, cvar_t *>::iterator it = g_cvars.find(name);
    return it == g_cvars.end() ? NULL : it->second;
}

cvar_t *Cvar_Get(const char *name, const char *value, int /*flags*/) {
    // Existing cvar keeps its value (Raven only ORs flags in); creation seeds
    // string=default, integer=atoi, modified=qtrue.
    cvar_t *c = cvarLookup(name);
    if (c) return c;
    c = new cvar_t;
    c->string = strdup(value);
    c->integer = atoi(value);
    c->modified = qtrue;
    g_cvars[name] = c;
    return c;
}

// Harness-side setter (drives se_debug/sp_leet/se_language in the dumpers).
void Host_CvarSet(const char *name, const char *value) {
    cvar_t *c = Cvar_Get(name, value, 0);
    free(c->string);
    c->string = strdup(value);
    c->integer = atoi(value);
    c->modified = qtrue;
}

// com_buildScript is reached via a local `extern cvar_t*` in SE_Init, so the
// symbol must exist with external linkage. Seed integer 0 (skip the load-all
// buildscript branch by default).
cvar_t *com_buildScript = NULL;
void Host_Init() {
    com_buildScript = Cvar_Get("com_buildScript", "0", 0);
}

// --------------------------------------------------------------------------
// string helpers
// --------------------------------------------------------------------------
char *va(const char *format, ...) {
    static char buf[8][32000];
    static int idx = 0;
    char *dest = buf[idx];
    idx = (idx + 1) & 7;
    va_list ap;
    va_start(ap, format);
    vsnprintf(dest, 32000, format, ap);
    va_end(ap);
    return dest;
}

char *Q_strupr(char *s1) {
    for (char *s = s1; *s; s++) *s = toupper((unsigned char)*s);
    return s1;
}
int  Q_stricmp(const char *a, const char *b) { return strcasecmp(a, b); }
int  Q_stricmpn(const char *a, const char *b, int n) { return strncasecmp(a, b, (size_t)n); }
void Q_strncpyz(char *dest, const char *src, int destsize) {
    if (destsize <= 0) return;
    strncpy(dest, src, (size_t)destsize - 1);
    dest[destsize - 1] = '\0';
}

// --------------------------------------------------------------------------
// console / fatal — routed to stderr so stdout stays the dumper's clean golden
// --------------------------------------------------------------------------
void Com_Printf(const char *fmt, ...) {
    va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_DPrintf(const char *fmt, ...) {
    va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
void Com_Error(int level, const char *fmt, ...) {
    // The fixtures never trigger a fatal load; if one fires, fail loudly so the
    // golden diff catches it rather than silently truncating.
    fprintf(stderr, "COM_ERROR(%d): ", level);
    va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
    fprintf(stderr, "\n");
    exit(1);
}

// --------------------------------------------------------------------------
// misc
// --------------------------------------------------------------------------
void COM_DefaultExtension(char *path, int maxSize, const char *extension) {
    // If the filename (past the last path separator) already has a '.', leave it.
    const char *src = path + strlen(path) - 1;
    while (src >= path && *src != '/' && *src != '\\') {
        if (*src == '.') return; // already has an extension
        src--;
    }
    size_t len = strlen(path);
    size_t elen = strlen(extension);
    if (len + elen < (size_t)maxSize) {
        strcat(path, extension);
    }
}

// --------------------------------------------------------------------------
// zone allocator
// --------------------------------------------------------------------------
void *Z_Malloc(int size, int /*tag*/, qboolean zeroit) {
    void *p = malloc((size_t)size);
    if (zeroit && p) memset(p, 0, (size_t)size);
    return p;
}
void Z_Free(void *ptr) { free(ptr); }

// --------------------------------------------------------------------------
// filesystem (fixture-backed, sorted for determinism)
// --------------------------------------------------------------------------
int FS_ReadFile(const char *qpath, void **buffer) {
    std::string full = joinPath(g_root, qpath);
    FILE *fh = fopen(full.c_str(), "rb");
    if (!fh) { if (buffer) *buffer = NULL; return -1; }
    fseek(fh, 0, SEEK_END);
    long len = ftell(fh);
    fseek(fh, 0, SEEK_SET);
    if (len < 0) { fclose(fh); if (buffer) *buffer = NULL; return -1; }
    unsigned char *data = (unsigned char *)malloc((size_t)len + 1);
    size_t got = fread(data, 1, (size_t)len, fh);
    fclose(fh);
    data[got] = '\0';
    if (buffer) *buffer = data; else free(data);
    return (int)got;
}
void FS_FreeFile(void *buffer) { free(buffer); }

static bool endsWith(const std::string &s, const std::string &suf) {
    return s.size() >= suf.size() && s.compare(s.size() - suf.size(), suf.size(), suf) == 0;
}

char **FS_ListFiles(const char *directory, const char *extension, int *numfiles) {
    std::string dirPath = joinPath(g_root, directory);
    bool wantDirs = (strcmp(extension, "/") == 0);
    std::vector<std::string> names;

    DIR *d = opendir(dirPath.c_str());
    if (d) {
        struct dirent *ent;
        while ((ent = readdir(d)) != NULL) {
            std::string name = ent->d_name;
            if (name == "." || name == "..") continue;
            std::string full = joinPath(dirPath, name);
            struct stat st;
            if (stat(full.c_str(), &st) != 0) continue;
            bool isDir = S_ISDIR(st.st_mode);
            if (wantDirs) {
                if (isDir) names.push_back(name);
            } else {
                if (!isDir && endsWith(name, extension)) names.push_back(name);
            }
        }
        closedir(d);
    }
    std::sort(names.begin(), names.end()); // determinism

    int n = (int)names.size();
    char **list = (char **)malloc(sizeof(char *) * (n + 1));
    for (int i = 0; i < n; i++) list[i] = strdup(names[i].c_str());
    list[n] = NULL;
    if (numfiles) *numfiles = n;
    return list;
}

void FS_FreeFileList(char **list) {
    if (!list) return;
    for (int i = 0; list[i] != NULL; i++) free(list[i]);
    free(list);
}
