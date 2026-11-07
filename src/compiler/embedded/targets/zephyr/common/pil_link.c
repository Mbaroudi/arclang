/*
 * The link of processor-in-the-loop runs: the console UART, polled.
 */
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/drivers/uart.h>

#include "arclang_pil.h"

static const struct device *const console = DEVICE_DT_GET(DT_CHOSEN(zephyr_console));

void arclang_pil_putc(char c)
{
	uart_poll_out(console, (unsigned char)c);
}

int arclang_pil_getc(void)
{
	unsigned char c;

	while (uart_poll_in(console, &c) != 0) {
		k_yield();
	}
	return (int)c;
}
