/*
 * Processor in the loop, the step() API: per step, the inputs read from
 * the console (one line, values bit for bit), one step, the outputs
 * reported (arclang_pil.c). The behavior runs on the target's CPU, built
 * by its compiler; no I/O, no timer.
 */
#include <zephyr/kernel.h>
#include <string.h>

#include "arclang_app.h"
#include "arclang_pil.h"

static arclang_state state;
static arclang_inputs inputs;
static arclang_outputs outputs;

int main(void)
{
	uint32_t step = 0U;

	(void)memset(&inputs, 0, sizeof inputs);
	(void)memset(&outputs, 0, sizeof outputs);
	arclang_model_init(&state);
	arclang_pil_puts("ARCLANG READY\n");
	for (;;) {
		if (!arclang_pil_read(&inputs)) {
			arclang_pil_puts("ARCLANG ERROR an input line could not be read\n");
			continue;
		}
		{
			/* The time of step() alone, in nanoseconds of the target's clock. */
			const uint64_t before = k_cycle_get_64();

			arclang_model_step(&state, &inputs, &outputs);
			arclang_pil_report(step, k_cyc_to_ns_floor64(k_cycle_get_64() - before), 0, &outputs);
		}
		step++;
	}
	return 0;
}
