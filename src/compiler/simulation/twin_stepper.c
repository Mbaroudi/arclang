/* The stepper of a digital twin: an OpenModelica Co-Simulation FMU (FMI 2.0)
 * advanced as the data arrives. ArcLang writes this file beside the FMU and
 * compiles it where the FMU was built (in the OpenModelica container, or
 * with the local compiler), then talks to it line by line:
 *
 *   stepper <library> <model identifier> <modelDescription.xml> <resources URI> <tolerance>
 *   < in NAME...           the inputs, in the order of the values to come
 *   < out NAME...          the variables read after each step
 *   < init T U...          start at T with the inputs U        > @@ok T Y...
 *   < step T U... D...     advance to T; U and D are the inputs and their
 *                          slopes at the current instant        > @@ok T Y...
 *   < quit
 *
 * Anything that goes wrong is answered `@@error <why>` and the stepper ends.
 * The FMU's own messages go to stderr. */
#include <dlfcn.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void *fmi2Component;
typedef unsigned int fmi2ValueReference;
typedef double fmi2Real;
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

typedef fmi2Component (*Instantiate)(const char *, int, const char *, const char *, const fmi2CallbackFunctions *, fmi2Boolean, fmi2Boolean);
typedef fmi2Status (*SetupExperiment)(fmi2Component, fmi2Boolean, fmi2Real, fmi2Real, fmi2Boolean, fmi2Real);
typedef fmi2Status (*Mode)(fmi2Component);
typedef fmi2Status (*Reals)(fmi2Component, const fmi2ValueReference *, size_t, fmi2Real *);
typedef fmi2Status (*SetReals)(fmi2Component, const fmi2ValueReference *, size_t, const fmi2Real *);
typedef fmi2Status (*SetDerivatives)(fmi2Component, const fmi2ValueReference *, size_t, const fmi2Integer *, const fmi2Real *);
typedef fmi2Status (*DoStep)(fmi2Component, fmi2Real, fmi2Real, fmi2Boolean);

#define MAX 256
#define LINE 65536

static void logger(void *env, const char *instance, fmi2Status status, const char *category, const char *message, ...) {
  va_list args;
  (void)env;
  va_start(args, message);
  fprintf(stderr, "[%s %s %d] ", instance ? instance : "", category ? category : "", status);
  vfprintf(stderr, message, args);
  fprintf(stderr, "\n");
  va_end(args);
}

static void fail(const char *why, const char *what) {
  printf("@@error %s%s%s\n", why, what ? ": " : "", what ? what : "");
  fflush(stdout);
  exit(1);
}

static char *xml;

/* The attribute's value in [from, to), or NULL. */
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

/* The value reference of a Real variable, by its name. */
static fmi2ValueReference reference(const char *name) {
  char found[1024], vr[32];
  for (const char *at = strstr(xml, "<ScalarVariable"); at; at = strstr(at + 1, "<ScalarVariable")) {
    const char *end = strstr(at, "</ScalarVariable>");
    if (!end) break;
    if (attribute(at, end, "name", found, sizeof found) && strcmp(found, name) == 0) {
      if (!strstr(at, "<Real") || strstr(at, "<Real") > end) fail("not a Real variable", name);
      if (!attribute(at, end, "valueReference", vr, sizeof vr)) fail("no value reference", name);
      return (fmi2ValueReference)strtoul(vr, NULL, 10);
    }
  }
  fail("no such variable in the FMU", name);
  return 0;
}

static void *symbol(void *library, const char *name) {
  void *f = dlsym(library, name);
  if (!f) fail("the FMU lacks", name);
  return f;
}

int main(int argc, char **argv) {
  if (argc != 6) fail("usage: stepper <library> <model identifier> <modelDescription.xml> <resources URI> <tolerance>", NULL);
  FILE *file = fopen(argv[3], "rb");
  if (!file) fail("cannot read", argv[3]);
  fseek(file, 0, SEEK_END);
  long length = ftell(file);
  rewind(file);
  xml = malloc(length + 1);
  if (fread(xml, 1, length, file) != (size_t)length) fail("cannot read", argv[3]);
  xml[length] = 0;
  fclose(file);
  char guid[256];
  const char *root = strstr(xml, "<fmiModelDescription");
  if (!root || !attribute(root, xml + length, "guid", guid, sizeof guid)) fail("no guid in", argv[3]);

  void *library = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
  if (!library) fail("cannot load the FMU", dlerror());
  Instantiate instantiate = (Instantiate)symbol(library, "fmi2Instantiate");
  SetupExperiment setup = (SetupExperiment)symbol(library, "fmi2SetupExperiment");
  Mode enter = (Mode)symbol(library, "fmi2EnterInitializationMode");
  Mode exitInit = (Mode)symbol(library, "fmi2ExitInitializationMode");
  Reals getReal = (Reals)symbol(library, "fmi2GetReal");
  SetReals setReal = (SetReals)symbol(library, "fmi2SetReal");
  SetDerivatives setDerivatives = (SetDerivatives)symbol(library, "fmi2SetRealInputDerivatives");
  DoStep doStep = (DoStep)symbol(library, "fmi2DoStep");
  Mode terminate = (Mode)symbol(library, "fmi2Terminate");

  static fmi2CallbackFunctions callbacks = {logger, calloc, free, NULL, NULL};
  fmi2Component c = instantiate(argv[2], 1 /* co-simulation */, guid, argv[4], &callbacks, 0, 0);
  if (!c) fail("the FMU cannot be instantiated (see stderr)", NULL);
  double tolerance = atof(argv[5]);

  fmi2ValueReference in[MAX], out[MAX];
  fmi2Integer orders[MAX];
  fmi2Real values[MAX], slopes[MAX], results[MAX];
  size_t inputs = 0, outputs = 0;
  double now = 0;
  int started = 0;
  static char line[LINE];
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
      if (isIn) inputs = n; else outputs = n;
      for (size_t i = 0; i < MAX; i++) orders[i] = 1;
      printf("@@ok\n");
      fflush(stdout);
      continue;
    }
    int init = strcmp(word, "init") == 0;
    if (!init && strcmp(word, "step") != 0) fail("unknown command", word);
    char *text = strtok(NULL, " \t\r\n");
    if (!text) fail("a time is missing", word);
    double t = atof(text);
    size_t wanted = init ? inputs : 2 * inputs;
    for (size_t i = 0; i < wanted; i++) {
      text = strtok(NULL, " \t\r\n");
      if (!text) fail("a value is missing", word);
      if (i < inputs) values[i] = atof(text); else slopes[i - inputs] = atof(text);
    }
    if (init) {
      if (started) fail("init twice", NULL);
      if (setup(c, tolerance > 0, tolerance, t, 0, 0) > 1) fail("fmi2SetupExperiment failed", NULL);
      if (enter(c) > 1) fail("fmi2EnterInitializationMode failed", NULL);
      if (inputs && setReal(c, in, inputs, values) > 1) fail("fmi2SetReal failed", NULL);
      if (exitInit(c) > 1) fail("the model does not initialize (see stderr)", NULL);
      started = 1;
      now = t;
    } else {
      if (!started) fail("step before init", NULL);
      if (t < now) fail("a step goes back in time", text);
      if (inputs && setReal(c, in, inputs, values) > 1) fail("fmi2SetReal failed", NULL);
      if (inputs && setDerivatives(c, in, inputs, orders, slopes) > 1) fail("fmi2SetRealInputDerivatives failed", NULL);
      if (t > now) {
        if (doStep(c, now, t - now, 1) > 1) fail("the step failed (see stderr)", NULL);
        now = t;
      }
    }
    if (outputs && getReal(c, out, outputs, results) > 1) fail("fmi2GetReal failed", NULL);
    printf("@@ok %.17g", now);
    for (size_t i = 0; i < outputs; i++) printf(" %.17g", results[i]);
    printf("\n");
    fflush(stdout);
  }
  terminate(c);
  return 0;
}
