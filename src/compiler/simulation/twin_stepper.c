/* The stepper of a digital twin or a plant: a Co-Simulation FMU, FMI 2.0 or
 * FMI 3.0, advanced as the data arrives. ArcLang writes this file beside the
 * extracted FMU and compiles it where the FMU is to run (in the OpenModelica
 * container, or with the local compiler), then talks to it line by line:
 *
 *   stepper <extracted FMU directory> <tolerance>
 *   < in NAME...           the inputs, in the order of the values to come
 *   < out NAME...          the variables read after each step
 *   < init T U...          start at T with the inputs U        > @@ok T Y...
 *   < step T U... D...     advance to T; U and D are the inputs and their
 *                          slopes at the current instant        > @@ok T Y...
 *   < quit
 *
 * The description says the version (`fmiVersion`), the library's name (the
 * co-simulation `modelIdentifier`) and the variables; the library is the
 * one under `binaries/` built for the platform this stepper was compiled on.
 * FMI 2.0 takes the slopes through fmi2SetRealInputDerivatives; FMI 3.0
 * has no such call: a slope is set on the input declared as the derivative
 * of the input (`derivative="<its value reference>"`), and an input without
 * one is held over the step.
 *
 * Anything that goes wrong is answered `@@error <why>` and the stepper ends.
 * The FMU's own messages go to stderr. */
#include <dlfcn.h>
#include <limits.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef unsigned int ValueReference;

/* FMI 2.0 */
typedef void *fmi2Component;
typedef int fmi2Integer;
typedef int fmi2Boolean;
typedef int fmi2Status;
typedef void (*fmi2Logger)(void *, const char *, fmi2Status, const char *, const char *, ...);
typedef struct {
  fmi2Logger logger;
  void *(*allocateMemory)(size_t, size_t);
  void (*freeMemory)(void *);
  void (*stepFinished)(void *, fmi2Status);
  void *componentEnvironment;
} fmi2CallbackFunctions;
typedef fmi2Component (*Instantiate2)(const char *, int, const char *, const char *, const fmi2CallbackFunctions *, fmi2Boolean, fmi2Boolean);
typedef fmi2Status (*SetupExperiment2)(fmi2Component, fmi2Boolean, double, double, fmi2Boolean, double);
typedef fmi2Status (*Mode2)(fmi2Component);
typedef fmi2Status (*GetReal2)(fmi2Component, const ValueReference *, size_t, double *);
typedef fmi2Status (*SetReal2)(fmi2Component, const ValueReference *, size_t, const double *);
typedef fmi2Status (*SetDerivatives2)(fmi2Component, const ValueReference *, size_t, const fmi2Integer *, const double *);
typedef fmi2Status (*DoStep2)(fmi2Component, double, double, fmi2Boolean);

/* FMI 3.0 (fmi3FunctionTypes.h, 3.0.2) */
typedef void *fmi3Instance;
typedef int fmi3Status;
typedef uint32_t fmi3ValueReference;
typedef void (*fmi3LogMessageCallback)(void *, fmi3Status, const char *, const char *);
typedef fmi3Instance (*Instantiate3)(const char *, const char *, const char *, bool, bool, bool, bool, const fmi3ValueReference *, size_t, void *, fmi3LogMessageCallback, void *);
typedef fmi3Status (*EnterInitialization3)(fmi3Instance, bool, double, double, bool, double);
typedef fmi3Status (*Mode3)(fmi3Instance);
typedef fmi3Status (*GetFloat64_3)(fmi3Instance, const fmi3ValueReference *, size_t, double *, size_t);
typedef fmi3Status (*SetFloat64_3)(fmi3Instance, const fmi3ValueReference *, size_t, const double *, size_t);
typedef fmi3Status (*DoStep3)(fmi3Instance, double, double, bool, bool *, bool *, bool *, double *);
typedef void (*Free3)(fmi3Instance);

#define MAX 256
#define LINE 65536

/* The binaries' directories this stepper can load, in the order tried: the
 * FMI 3.0 name, then the FMI 2.0 one (OpenModelica writes `linux64` on
 * every Linux, arm64 included). */
static const char *const PLATFORMS[] = {
#if defined(__APPLE__) && defined(__aarch64__)
    "aarch64-darwin", "darwin64",
#elif defined(__APPLE__)
    "x86_64-darwin", "darwin64",
#elif defined(__linux__) && defined(__aarch64__)
    "aarch64-linux", "linux64",
#elif defined(__linux__)
    "x86_64-linux", "linux64",
#endif
    NULL};
#if defined(__APPLE__)
#define EXTENSION "dylib"
#else
#define EXTENSION "so"
#endif

static void logger2(void *env, const char *instance, fmi2Status status, const char *category, const char *message, ...) {
  va_list args;
  (void)env;
  va_start(args, message);
  fprintf(stderr, "[%s %s %d] ", instance ? instance : "", category ? category : "", status);
  vfprintf(stderr, message, args);
  fprintf(stderr, "\n");
  va_end(args);
}

static void logger3(void *env, fmi3Status status, const char *category, const char *message) {
  (void)env;
  fprintf(stderr, "[%s %d] %s\n", category ? category : "", status, message ? message : "");
}

static void fail(const char *why, const char *what) {
  printf("@@error %s%s%s\n", why, what ? ": " : "", what ? what : "");
  fflush(stdout);
  exit(1);
}

static char *xml;
static int version;

/* The attribute's value in [from, to), or NULL. The key starts with a
 * blank: ` name="` is not the end of ` modelName="`. */
static char *attribute(const char *from, const char *to, const char *name, char *out, size_t size) {
  char key[64];
  snprintf(key, sizeof key, " %s=\"", name);
  const char *at = strstr(from, key);
  if (!at || at >= to) return NULL;
  at += strlen(key);
  const char *end = strchr(at, '"');
  if (!end || end >= to || (size_t)(end - at) >= size) return NULL;
  memcpy(out, at, end - at);
  out[end - at] = 0;
  return out;
}

/* The end of the start tag at `at` (its '>'). */
static const char *tag_end(const char *at) {
  const char *end = strchr(at, '>');
  if (!end) fail("the model description is cut short", NULL);
  return end;
}

/* Whether the start tag at `at` is the element `name`. */
static int is_element(const char *at, const char *name) {
  size_t n = strlen(name);
  return at[0] == '<' && strncmp(at + 1, name, n) == 0 && (at[n + 1] == ' ' || at[n + 1] == '\t' || at[n + 1] == '\r' || at[n + 1] == '\n' || at[n + 1] == '/' || at[n + 1] == '>');
}

/* The value reference of a real variable, by its name: a Real
 * ScalarVariable (FMI 2.0), a Float64 (FMI 3.0). */
static ValueReference reference(const char *name) {
  char found[1024], vr[32];
  if (version == 2) {
    for (const char *at = strstr(xml, "<ScalarVariable"); at; at = strstr(at + 1, "<ScalarVariable")) {
      const char *end = strstr(at, "</ScalarVariable>");
      if (!end) break;
      if (attribute(at, tag_end(at), "name", found, sizeof found) && strcmp(found, name) == 0) {
        if (!strstr(at, "<Real") || strstr(at, "<Real") > end) fail("not a Real variable", name);
        if (!attribute(at, end, "valueReference", vr, sizeof vr)) fail("no value reference", name);
        return (ValueReference)strtoul(vr, NULL, 10);
      }
    }
  } else {
    for (const char *at = strstr(xml, "<ModelVariables"); at && (at = strchr(at + 1, '<'));) {
      if (strncmp(at, "</ModelVariables", 16) == 0) break;
      if (at[1] == '/' || at[1] == '!') continue;
      const char *end = tag_end(at);
      if (attribute(at, end, "name", found, sizeof found) && strcmp(found, name) == 0) {
        if (!is_element(at, "Float64")) fail("not a Float64 variable", name);
        if (!attribute(at, end, "valueReference", vr, sizeof vr)) fail("no value reference", name);
        return (ValueReference)strtoul(vr, NULL, 10);
      }
    }
  }
  fail("no such variable in the FMU", name);
  return 0;
}

/* FMI 3.0: the input declared as the derivative of the input `of`. */
static bool derivative_of(ValueReference of, ValueReference *derivative) {
  char text[32], causality[32], vr[32];
  for (const char *at = strstr(xml, "<ModelVariables"); at && (at = strchr(at + 1, '<'));) {
    if (strncmp(at, "</ModelVariables", 16) == 0) break;
    if (!is_element(at, "Float64")) continue;
    const char *end = tag_end(at);
    if (attribute(at, end, "derivative", text, sizeof text) && strtoul(text, NULL, 10) == of && attribute(at, end, "causality", causality, sizeof causality) && strcmp(causality, "input") == 0 && attribute(at, end, "valueReference", vr, sizeof vr)) {
      *derivative = (ValueReference)strtoul(vr, NULL, 10);
      return true;
    }
  }
  return false;
}

static void *symbol(void *library, const char *name) {
  void *f = dlsym(library, name);
  if (!f) fail("the FMU lacks", name);
  return f;
}

static char *read_file(const char *path) {
  FILE *file = fopen(path, "rb");
  if (!file) fail("cannot read", path);
  fseek(file, 0, SEEK_END);
  long length = ftell(file);
  rewind(file);
  char *text = malloc(length + 1);
  if (fread(text, 1, length, file) != (size_t)length) fail("cannot read", path);
  text[length] = 0;
  fclose(file);
  return text;
}

/* The library of the FMU for this platform, loaded. */
static void *load(const char *dir, const char *identifier) {
  static char why[4096];
  char path[PATH_MAX];
  why[0] = 0;
  for (int i = 0; PLATFORMS[i]; i++) {
    snprintf(path, sizeof path, "%s/binaries/%s/%s.%s", dir, PLATFORMS[i], identifier, EXTENSION);
    if (access(path, R_OK) != 0) continue;
    void *library = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (library) return library;
    snprintf(why + strlen(why), sizeof why - strlen(why), "%s%s", why[0] ? "; " : "", dlerror());
  }
  if (why[0]) fail("cannot load the FMU", why);
  snprintf(why, sizeof why, "binaries/{");
  for (int i = 0; PLATFORMS[i]; i++) snprintf(why + strlen(why), sizeof why - strlen(why), "%s%s", i ? "," : "", PLATFORMS[i]);
  snprintf(why + strlen(why), sizeof why - strlen(why), "}/%s.%s", identifier, EXTENSION);
  fail("the FMU has no binary for this platform", why);
  return NULL;
}

int main(int argc, char **argv) {
  if (argc != 3) fail("usage: stepper <extracted FMU directory> <tolerance>", NULL);
  char dir[PATH_MAX], path[PATH_MAX + 64], resources[PATH_MAX + 64];
  if (!realpath(argv[1], dir)) fail("no such directory", argv[1]);
  snprintf(path, sizeof path, "%s/modelDescription.xml", dir);
  xml = read_file(path);
  const char *root = strstr(xml, "<fmiModelDescription");
  if (!root) fail("not a model description", path);
  const char *root_end = tag_end(root);
  char text[256], token[256], identifier[256];
  if (!attribute(root, root_end, "fmiVersion", text, sizeof text)) fail("no fmiVersion in", path);
  version = text[0] == '2' ? 2 : text[0] == '3' ? 3 : 0;
  if (!version) fail("an FMI version this stepper does not know (it knows 2.0 and 3.0)", text);
  if (!attribute(root, root_end, version == 2 ? "guid" : "instantiationToken", token, sizeof token)) fail(version == 2 ? "no guid in" : "no instantiationToken in", path);
  const char *cs = strstr(xml, "<CoSimulation");
  if (!cs) fail("not a co-simulation FMU (no <CoSimulation> in its description)", NULL);
  if (!attribute(cs, tag_end(cs), "modelIdentifier", identifier, sizeof identifier)) fail("no modelIdentifier in", "<CoSimulation>");
  void *library = load(dir, identifier);
  double tolerance = atof(argv[2]);

  /* FMI 2.0 */
  Instantiate2 instantiate2 = NULL;
  SetupExperiment2 setup2 = NULL;
  Mode2 enter2 = NULL, exit2 = NULL, terminate2 = NULL;
  GetReal2 get2 = NULL;
  SetReal2 set2 = NULL;
  SetDerivatives2 derivatives2 = NULL;
  DoStep2 step2 = NULL;
  /* FMI 3.0 */
  EnterInitialization3 enter3 = NULL;
  Mode3 exit3 = NULL, terminate3 = NULL;
  GetFloat64_3 get3 = NULL;
  SetFloat64_3 set3 = NULL;
  DoStep3 step3 = NULL;
  Free3 free3 = NULL;
  void *c;
  if (version == 2) {
    instantiate2 = (Instantiate2)symbol(library, "fmi2Instantiate");
    setup2 = (SetupExperiment2)symbol(library, "fmi2SetupExperiment");
    enter2 = (Mode2)symbol(library, "fmi2EnterInitializationMode");
    exit2 = (Mode2)symbol(library, "fmi2ExitInitializationMode");
    get2 = (GetReal2)symbol(library, "fmi2GetReal");
    set2 = (SetReal2)symbol(library, "fmi2SetReal");
    derivatives2 = (SetDerivatives2)symbol(library, "fmi2SetRealInputDerivatives");
    step2 = (DoStep2)symbol(library, "fmi2DoStep");
    terminate2 = (Mode2)symbol(library, "fmi2Terminate");
    static fmi2CallbackFunctions callbacks = {logger2, calloc, free, NULL, NULL};
    snprintf(resources, sizeof resources, "file://%s/resources", dir);
    c = instantiate2(identifier, 1 /* co-simulation */, token, resources, &callbacks, 0, 0);
  } else {
    Instantiate3 instantiate3 = (Instantiate3)symbol(library, "fmi3InstantiateCoSimulation");
    enter3 = (EnterInitialization3)symbol(library, "fmi3EnterInitializationMode");
    exit3 = (Mode3)symbol(library, "fmi3ExitInitializationMode");
    get3 = (GetFloat64_3)symbol(library, "fmi3GetFloat64");
    set3 = (SetFloat64_3)symbol(library, "fmi3SetFloat64");
    step3 = (DoStep3)symbol(library, "fmi3DoStep");
    terminate3 = (Mode3)symbol(library, "fmi3Terminate");
    free3 = (Free3)symbol(library, "fmi3FreeInstance");
    snprintf(resources, sizeof resources, "%s/resources/", dir);
    /* No event mode, no early return: a step always ends where it was asked to. */
    c = instantiate3(identifier, token, resources, false, false, false, false, NULL, 0, NULL, logger3, NULL);
  }
  if (!c) fail("the FMU cannot be instantiated (see stderr)", NULL);

  ValueReference in[MAX], out[MAX], slope_vr[MAX];
  size_t slope_of[MAX];
  fmi2Integer orders[MAX];
  double values[MAX], slopes[MAX], results[MAX], slope_values[MAX];
  size_t inputs = 0, outputs = 0, sloped = 0;
  double now = 0;
  int started = 0;
  static char line[LINE];
  for (size_t i = 0; i < MAX; i++) orders[i] = 1;
  printf("@@ready\n");
  fflush(stdout);
  while (fgets(line, sizeof line, stdin)) {
    char *word = strtok(line, " \t\r\n");
    if (!word) continue;
    if (strcmp(word, "quit") == 0) break;
    if (strcmp(word, "in") == 0 || strcmp(word, "out") == 0) {
      int isIn = word[0] == 'i';
      size_t n = 0;
      for (char *name = strtok(NULL, " \t\r\n"); name; name = strtok(NULL, " \t\r\n")) {
        if (n == MAX) fail("too many variables", NULL);
        (isIn ? in : out)[n++] = reference(name);
      }
      if (isIn) {
        inputs = n;
        sloped = 0;
        for (size_t i = 0; version == 3 && i < n; i++) {
          if (derivative_of(in[i], &slope_vr[sloped])) slope_of[sloped++] = i;
        }
      } else {
        outputs = n;
      }
      printf("@@ok\n");
      fflush(stdout);
      continue;
    }
    int init = strcmp(word, "init") == 0;
    if (!init && strcmp(word, "step") != 0) fail("unknown command", word);
    char *value = strtok(NULL, " \t\r\n");
    if (!value) fail("a time is missing", word);
    double t = atof(value);
    size_t wanted = init ? inputs : 2 * inputs;
    for (size_t i = 0; i < wanted; i++) {
      value = strtok(NULL, " \t\r\n");
      if (!value) fail("a value is missing", word);
      if (i < inputs) values[i] = atof(value); else slopes[i - inputs] = atof(value);
    }
    if (init) {
      if (started) fail("init twice", NULL);
      if (version == 2) {
        if (setup2(c, tolerance > 0, tolerance, t, 0, 0) > 1) fail("fmi2SetupExperiment failed", NULL);
        if (enter2(c) > 1) fail("fmi2EnterInitializationMode failed", NULL);
        if (inputs && set2(c, in, inputs, values) > 1) fail("fmi2SetReal failed", NULL);
        if (exit2(c) > 1) fail("the model does not initialize (see stderr)", NULL);
      } else {
        if (enter3(c, tolerance > 0, tolerance, t, false, 0) > 1) fail("fmi3EnterInitializationMode failed", NULL);
        if (inputs && set3(c, in, inputs, values, inputs) > 1) fail("fmi3SetFloat64 failed", NULL);
        if (exit3(c) > 1) fail("the model does not initialize (see stderr)", NULL);
      }
      started = 1;
      now = t;
    } else {
      if (!started) fail("step before init", NULL);
      if (t < now) fail("a step goes back in time", value);
      if (version == 2) {
        if (inputs && set2(c, in, inputs, values) > 1) fail("fmi2SetReal failed", NULL);
        if (inputs && derivatives2(c, in, inputs, orders, slopes) > 1) fail("fmi2SetRealInputDerivatives failed", NULL);
        if (t > now && step2(c, now, t - now, 1) > 1) fail("the step failed (see stderr)", NULL);
      } else {
        if (inputs && set3(c, in, inputs, values, inputs) > 1) fail("fmi3SetFloat64 failed", NULL);
        for (size_t i = 0; i < sloped; i++) slope_values[i] = slopes[slope_of[i]];
        if (sloped && set3(c, slope_vr, sloped, slope_values, sloped) > 1) fail("fmi3SetFloat64 failed (input derivatives)", NULL);
        if (t > now) {
          bool event = false, terminate = false, early = false;
          double reached = t;
          if (step3(c, now, t - now, true, &event, &terminate, &early, &reached) > 1) fail("the step failed (see stderr)", NULL);
          if (terminate) fail("the FMU ends the simulation", NULL);
        }
      }
      now = t > now ? t : now;
    }
    if (outputs && (version == 2 ? get2(c, out, outputs, results) : get3(c, out, outputs, results, outputs)) > 1) fail(version == 2 ? "fmi2GetReal failed" : "fmi3GetFloat64 failed", NULL);
    printf("@@ok %.17g", now);
    for (size_t i = 0; i < outputs; i++) printf(" %.17g", results[i]);
    printf("\n");
    fflush(stdout);
  }
  if (version == 2) {
    terminate2(c);
  } else {
    terminate3(c);
    free3(c);
  }
  return 0;
}
