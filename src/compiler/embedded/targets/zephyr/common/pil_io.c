/*
 * Processor in the loop, through the I/O: the controller's loop (the
 * glue's cycle every period, the kernel timer keeping the rate), and after
 * each cycle a report on the console: the step, its time, the cycle's
 * status and the outputs (arclang_pil.c). The bench drives the bound
 * input channels between two cycles and reads the bound output channels
 * after each. It starts on a 'G' from the console, so step 0 sees the
 * inputs the bench set for it; every cycle starts on a boundary of the
 * period (the first one a period after the 'G').
 */
#include <zephyr/kernel.h>
#include <string.h>

#include "arclang_app.h"
#include "arclang_pil.h"

static arclang_state state;
static arclang_inputs inputs;
static arclang_outputs outputs;
static arclang_glue glue;

int main(void)
{
	uint32_t step = 0U;
	arclang_io_status status = arclang_io_init();

	(void)memset(&inputs, 0, sizeof inputs);
	(void)memset(&outputs, 0, sizeof outputs);
	arclang_model_init(&state);
	arclang_glue_init(&glue);
	arclang_pil_puts(status == ARCLANG_IO_OK ? "ARCLANG READY\n" : "ARCLANG ERROR the I/O did not come up\n");
	while (arclang_pil_getc() != 'G') {
	}
	for (;;) {
		uint64_t now_us;

		(void)arclang_io_clock_wait_period(ARCLANG_PERIOD_US);
		now_us = k_cyc_to_us_floor64(k_cycle_get_64());
		status = arclang_cycle(&glue, &state, &inputs, &outputs);
		arclang_pil_report(step, now_us, (int)status, &outputs);
		step++;
	}
	return 0;
}
