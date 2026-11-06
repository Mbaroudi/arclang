/* A room for the plant tests, as a co-simulation FMU from "another tool":
 * its temperature T follows the heat input as a first-order lag,
 * dT/dt = (heat - T) / tau, tau = 0.5 s, solved exactly over each step with
 * the input going linearly from its value along its slope (an FMU that
 * takes input derivatives). Built as FMI 2.0 (`-DFMI_VERSION=2`) or FMI 3.0
 * (`-DFMI_VERSION=3`) by tests/embedded_fmi.rs, the model identifier
 * `Room`. Value references: 1 heat, 2 T, 3 the slope of heat (FMI 3.0: an
 * input declared `derivative="1"`). */
#include <math.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#define EXPORT __attribute__((visibility("default")))
#define TAU 0.5

typedef struct {
  double t, heat, slope, temperature;
} Room;

static void advance(Room *r, double h) {
  /* T(h) = u0 + d (h - tau) + (T0 - u0 + d tau) e^(-h/tau) */
  double u0 = r->heat, d = r->slope;
  r->temperature = u0 + d * (h - TAU) + (r->temperature - u0 + d * TAU) * exp(-h / TAU);
  r->heat = u0 + d * h;
  r->t += h;
}

static int set(Room *r, unsigned vr, double v) {
  if (vr == 1) r->heat = v;
  else if (vr == 3) r->slope = v;
  else return 0;
  return 1;
}

static int get(const Room *r, unsigned vr, double *v) {
  if (vr == 1) *v = r->heat;
  else if (vr == 2) *v = r->temperature;
  else if (vr == 3) *v = r->slope;
  else return 0;
  return 1;
}

#if FMI_VERSION == 2

typedef int fmi2Status;
EXPORT void *fmi2Instantiate(const char *name, int type, const char *guid, const char *resources, const void *callbacks, int visible, int logging) {
  (void)name; (void)type; (void)guid; (void)resources; (void)callbacks; (void)visible; (void)logging;
  return calloc(1, sizeof(Room));
}
EXPORT fmi2Status fmi2SetupExperiment(void *c, int toleranceDefined, double tolerance, double start, int stopDefined, double stop) {
  (void)toleranceDefined; (void)tolerance; (void)stopDefined; (void)stop;
  ((Room *)c)->t = start;
  return 0;
}
EXPORT fmi2Status fmi2EnterInitializationMode(void *c) { (void)c; return 0; }
EXPORT fmi2Status fmi2ExitInitializationMode(void *c) { (void)c; return 0; }
EXPORT fmi2Status fmi2Terminate(void *c) { free(c); return 0; }
EXPORT fmi2Status fmi2SetReal(void *c, const unsigned *vr, size_t n, const double *v) {
  for (size_t i = 0; i < n; i++) {
    if (!set(c, vr[i], v[i])) return 3;
    if (vr[i] == 1) ((Room *)c)->slope = 0;
  }
  return 0;
}
EXPORT fmi2Status fmi2GetReal(void *c, const unsigned *vr, size_t n, double *v) {
  for (size_t i = 0; i < n; i++) if (!get(c, vr[i], &v[i])) return 3;
  return 0;
}
EXPORT fmi2Status fmi2SetRealInputDerivatives(void *c, const unsigned *vr, size_t n, const int *order, const double *v) {
  for (size_t i = 0; i < n; i++) {
    if (vr[i] != 1 || order[i] != 1) return 3;
    ((Room *)c)->slope = v[i];
  }
  return 0;
}
EXPORT fmi2Status fmi2DoStep(void *c, double now, double h, int noSetPrior) {
  (void)now; (void)noSetPrior;
  advance(c, h);
  return 0;
}

#else

typedef int fmi3Status;
EXPORT void *fmi3InstantiateCoSimulation(const char *name, const char *token, const char *resources, bool visible, bool logging, bool eventMode, bool earlyReturn, const uint32_t *required, size_t nRequired, void *env, void *log, void *update) {
  (void)name; (void)token; (void)resources; (void)visible; (void)logging; (void)eventMode; (void)earlyReturn; (void)required; (void)nRequired; (void)env; (void)log; (void)update;
  return calloc(1, sizeof(Room));
}
EXPORT fmi3Status fmi3EnterInitializationMode(void *c, bool toleranceDefined, double tolerance, double start, bool stopDefined, double stop) {
  (void)toleranceDefined; (void)tolerance; (void)stopDefined; (void)stop;
  ((Room *)c)->t = start;
  return 0;
}
EXPORT fmi3Status fmi3ExitInitializationMode(void *c) { (void)c; return 0; }
EXPORT fmi3Status fmi3Terminate(void *c) { (void)c; return 0; }
EXPORT void fmi3FreeInstance(void *c) { free(c); }
EXPORT fmi3Status fmi3SetFloat64(void *c, const uint32_t *vr, size_t n, const double *v, size_t nValues) {
  if (nValues != n) return 3;
  for (size_t i = 0; i < n; i++) {
    if (!set(c, vr[i], v[i])) return 3;
    if (vr[i] == 1) ((Room *)c)->slope = 0;
  }
  return 0;
}
EXPORT fmi3Status fmi3GetFloat64(void *c, const uint32_t *vr, size_t n, double *v, size_t nValues) {
  if (nValues != n) return 3;
  for (size_t i = 0; i < n; i++) if (!get(c, vr[i], &v[i])) return 3;
  return 0;
}
EXPORT fmi3Status fmi3DoStep(void *c, double now, double h, bool noSetPrior, bool *event, bool *terminate, bool *early, double *last) {
  (void)now; (void)noSetPrior;
  advance(c, h);
  *event = false;
  *terminate = false;
  *early = false;
  *last = ((Room *)c)->t;
  return 0;
}

#endif
