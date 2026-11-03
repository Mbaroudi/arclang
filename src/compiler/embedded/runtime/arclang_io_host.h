/*
 * arclang_io_host.h -- the PC implementation of arclang_io.h, for
 * software-in-the-loop runs: a simulated board whose channels a test bench
 * sets (inputs) and reads (outputs). Not for a target.
 */
#ifndef ARCLANG_IO_HOST_H
#define ARCLANG_IO_HOST_H

#include "arclang_io.h"

#define ARCLANG_IO_HOST_PINS 256U
#define ARCLANG_IO_HOST_ANALOG 64U
#define ARCLANG_IO_HOST_CAN_IDS 64U
#define ARCLANG_IO_HOST_UARTS 8U
#define ARCLANG_IO_HOST_UART_QUEUE 64U

void arclang_io_host_reset(void);

/* What the bench drives. */
arclang_io_status arclang_io_host_set_gpio(uint16_t pin, bool level);
arclang_io_status arclang_io_host_set_adc(uint16_t channel, uint16_t raw);
arclang_io_status arclang_io_host_push_can(uint32_t id, const uint8_t data[8], uint8_t length);
arclang_io_status arclang_io_host_push_uart(uint16_t port, uint8_t byte);

/* What the target drove. */
bool arclang_io_host_gpio_output(uint16_t pin);
uint16_t arclang_io_host_dac(uint16_t channel);
uint16_t arclang_io_host_pwm(uint16_t channel);
/* The last frame sent with this id; false when none was. */
bool arclang_io_host_can_sent(uint32_t id, uint8_t data[8], uint8_t *length);
/* How many bytes equal to `byte` were sent on the port since it was cleared. */
uint16_t arclang_io_host_uart_count(uint16_t port, uint8_t byte);
void arclang_io_host_uart_clear(uint16_t port);
uint32_t arclang_io_host_watchdog_kicks(void);
uint64_t arclang_io_host_time_us(void);

#endif /* ARCLANG_IO_HOST_H */
