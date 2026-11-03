/*
 * arclang_io.h -- the one I/O interface between ArcLang generated code and
 * a target. Generated behavior (<model>.c) never calls it: only the
 * generated glue (<model>_<binding>_io.c) does, from the bindings written in
 * the model. A target package (Zephyr, Linux RT, a PLC runtime, ...)
 * implements these functions once; arclang_io_host.c implements them on a
 * PC for software-in-the-loop runs.
 *
 * Every function returns ARCLANG_IO_OK or an error; none blocks except
 * arclang_io_clock_wait_period. Channels are numbers the target maps to
 * its pins, converters, timers, controllers and ports.
 */
#ifndef ARCLANG_IO_H
#define ARCLANG_IO_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    ARCLANG_IO_OK = 0,
    /* the channel does not exist on this target */
    ARCLANG_IO_ERR_CHANNEL = 1,
    /* the hardware (or the driver) reported a fault */
    ARCLANG_IO_ERR_DEVICE = 2,
    /* a queue or mailbox is full */
    ARCLANG_IO_ERR_FULL = 3
} arclang_io_status;

/* Bring the I/O up; called once before the first cycle. */
arclang_io_status arclang_io_init(void);

/* Digital I/O: a level per pin. */
arclang_io_status arclang_io_gpio_read(uint16_t pin, bool *level);
arclang_io_status arclang_io_gpio_write(uint16_t pin, bool level);

/* Analog: raw converter counts (the target scales them, the model compares them). */
arclang_io_status arclang_io_adc_read(uint16_t channel, uint16_t *raw);
arclang_io_status arclang_io_dac_write(uint16_t channel, uint16_t raw);

/* PWM: duty cycle in per mille (0 to 1000). */
arclang_io_status arclang_io_pwm_write(uint16_t channel, uint16_t permille);

/* CAN: frames by identifier (11 or 29 bits), up to 8 data bytes.
   receive: the frame with this id received since the last call, if any. */
arclang_io_status arclang_io_can_receive(uint32_t id, uint8_t data[8], uint8_t *length, bool *received);
arclang_io_status arclang_io_can_send(uint32_t id, const uint8_t data[8], uint8_t length);

/* UART: bytes; read gives the next byte received, if any. */
arclang_io_status arclang_io_uart_read(uint16_t port, uint8_t *byte, bool *received);
arclang_io_status arclang_io_uart_write(uint16_t port, uint8_t byte);

/* Clock: block until the next period boundary (the step rate). */
arclang_io_status arclang_io_clock_wait_period(uint32_t period_us);

/* Watchdog: kicked once per cycle that read, stepped and wrote without error. */
arclang_io_status arclang_io_watchdog_kick(void);

#ifdef __cplusplus
}
#endif

#endif /* ARCLANG_IO_H */
