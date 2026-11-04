/*
 * The controller: the generated glue's cycle every period. Exported by
 * ArcLang; arclang_app.h names the model and the binding.
 */
#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>
#include <string.h>

#include "arclang_app.h"

static arclang_state state;
static arclang_inputs inputs;
static arclang_outputs outputs;
static arclang_glue glue;

int main(void)
{
	if (arclang_io_init() != ARCLANG_IO_OK) {
		printk("arclang: the I/O did not come up; no cycle runs\n");
		return 0;
	}
	(void)memset(&inputs, 0, sizeof inputs);
	(void)memset(&outputs, 0, sizeof outputs);
	arclang_model_init(&state);
	arclang_glue_init(&glue);
	for (;;) {
		/* Every cycle starts on the period's boundary (the first one
		   period after start), whatever the one before took. */
		(void)arclang_io_clock_wait_period(ARCLANG_PERIOD_US);
		/* A failed cycle has not stepped; the watchdog is not kicked. */
		(void)arclang_cycle(&glue, &state, &inputs, &outputs);
	}
	return 0;
}
