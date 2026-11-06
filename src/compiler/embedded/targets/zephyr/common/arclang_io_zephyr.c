/*
 * arclang_io_zephyr.c -- arclang_io.h over Zephyr's device APIs, for any
 * Zephyr board. The channels are found by devicetree aliases, which the
 * target package's overlay gives:
 *
 *   arclang-gpio-in, arclang-gpio-out  gpio N is line N (read, written)
 *   arclang-adc                        adc N is channel N of that ADC
 *   arclang-dac                        dac N is channel N of that DAC
 *   arclang-pwm-0 .. arclang-pwm-7     pwm N is channel 0 of that PWM
 *   arclang-can                        every CAN identifier (<= 0x7FF standard, else extended)
 *   arclang-uart-0 .. arclang-uart-7   uart N
 *   watchdog0                          the watchdog (when the binding has one)
 *
 * A channel whose alias the board does not give answers
 * ARCLANG_IO_ERR_CHANNEL; ArcLang refuses a binding using it beforehand,
 * from the package's declaration of its channels (target.json).
 * arclang_io_config.h is generated from the binding: the CAN identifiers
 * it receives, its watchdog.
 */
#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/drivers/adc.h>
#include <zephyr/drivers/dac.h>
#include <zephyr/drivers/pwm.h>
#include <zephyr/drivers/can.h>
#include <zephyr/drivers/uart.h>
#include <zephyr/drivers/watchdog.h>
#include <string.h>

#include "arclang_io.h"
#include "arclang_io_config.h"

#ifndef ARCLANG_IO_ADC_RESOLUTION
#define ARCLANG_IO_ADC_RESOLUTION 16U
#endif
#ifndef ARCLANG_IO_PWM_HZ
#define ARCLANG_IO_PWM_HZ 1000U
#endif

#define ALIAS_DEVICE(alias) COND_CODE_1(DT_NODE_HAS_STATUS_OKAY(DT_ALIAS(alias)), (DEVICE_DT_GET(DT_ALIAS(alias))), (NULL))
#define ALIAS_LINES(alias) COND_CODE_1(DT_NODE_HAS_STATUS_OKAY(DT_ALIAS(alias)), (DT_PROP_OR(DT_ALIAS(alias), ngpios, 32)), (0))

static const struct device *const gpio_in = ALIAS_DEVICE(arclang_gpio_in);
static const struct device *const gpio_out = ALIAS_DEVICE(arclang_gpio_out);
static const uint16_t gpio_in_lines = ALIAS_LINES(arclang_gpio_in);
static const uint16_t gpio_out_lines = ALIAS_LINES(arclang_gpio_out);
static uint32_t gpio_in_ready;
static uint32_t gpio_out_ready;

static const struct device *const adc = ALIAS_DEVICE(arclang_adc);
static uint32_t adc_ready;
static const struct device *const dac = ALIAS_DEVICE(arclang_dac);
static uint32_t dac_ready;

static const struct device *const pwms[] = {
	ALIAS_DEVICE(arclang_pwm_0), ALIAS_DEVICE(arclang_pwm_1), ALIAS_DEVICE(arclang_pwm_2), ALIAS_DEVICE(arclang_pwm_3),
	ALIAS_DEVICE(arclang_pwm_4), ALIAS_DEVICE(arclang_pwm_5), ALIAS_DEVICE(arclang_pwm_6), ALIAS_DEVICE(arclang_pwm_7),
};

static const struct device *const uarts[] = {
	ALIAS_DEVICE(arclang_uart_0), ALIAS_DEVICE(arclang_uart_1), ALIAS_DEVICE(arclang_uart_2), ALIAS_DEVICE(arclang_uart_3),
	ALIAS_DEVICE(arclang_uart_4), ALIAS_DEVICE(arclang_uart_5), ALIAS_DEVICE(arclang_uart_6), ALIAS_DEVICE(arclang_uart_7),
};

static const struct device *const can = ALIAS_DEVICE(arclang_can);
static const struct device *const watchdog = ALIAS_DEVICE(watchdog0);
static int watchdog_channel = -1;

/* One mailbox per CAN identifier the binding receives: the last frame since the last read. */
typedef struct {
	uint32_t id;
	bool pending;
	uint8_t data[8];
	uint8_t length;
} can_mailbox;

#if ARCLANG_IO_CAN_RX_COUNT > 0
static const uint32_t can_rx_ids[ARCLANG_IO_CAN_RX_COUNT] = ARCLANG_IO_CAN_RX_IDS;
static can_mailbox can_rx[ARCLANG_IO_CAN_RX_COUNT];

static void can_received(const struct device *dev, struct can_frame *frame, void *user_data)
{
	can_mailbox *box = user_data;
	unsigned int key = irq_lock();

	ARG_UNUSED(dev);
	box->length = (frame->dlc > 8U) ? 8U : frame->dlc;
	memcpy(box->data, frame->data, box->length);
	box->pending = true;
	irq_unlock(key);
}
#endif

static struct k_timer period_timer;
static uint32_t period_started_us;
/* One periodic timer per task of several (arclang_io_task_wait_period). */
static struct k_timer task_timers[ARCLANG_IO_TASKS];
static uint32_t task_started_us[ARCLANG_IO_TASKS];

static bool ready(const struct device *dev)
{
	return (dev != NULL) && device_is_ready(dev);
}

static uint32_t can_flags(uint32_t id)
{
	return (id > CAN_STD_ID_MASK) ? CAN_FRAME_IDE : 0U;
}

arclang_io_status arclang_io_init(void)
{
	arclang_io_status status = ARCLANG_IO_OK;

#if ARCLANG_IO_CAN_RX_COUNT > 0
	if (!ready(can)) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	for (size_t k = 0U; k < ARCLANG_IO_CAN_RX_COUNT; k++) {
		const struct can_filter filter = {
			.id = can_rx_ids[k],
			.mask = (can_rx_ids[k] > CAN_STD_ID_MASK) ? CAN_EXT_ID_MASK : CAN_STD_ID_MASK,
			.flags = (can_rx_ids[k] > CAN_STD_ID_MASK) ? CAN_FILTER_IDE : 0U,
		};
		can_rx[k].id = can_rx_ids[k];
		can_rx[k].pending = false;
		if (can_add_rx_filter(can, can_received, &can_rx[k], &filter) < 0) {
			status = ARCLANG_IO_ERR_DEVICE;
		}
	}
#endif
	if (ready(can)) {
		int err = can_start(can);

		if ((err != 0) && (err != -EALREADY)) {
			status = ARCLANG_IO_ERR_DEVICE;
		}
	}
#if ARCLANG_IO_WATCHDOG
	if (!ready(watchdog)) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	{
		const struct wdt_timeout_cfg cfg = {
			.window = { .min = 0U, .max = ARCLANG_IO_WATCHDOG_TIMEOUT_MS },
			.callback = NULL,
			.flags = WDT_FLAG_RESET_SOC,
		};
		watchdog_channel = wdt_install_timeout(watchdog, &cfg);
		if ((watchdog_channel < 0) || (wdt_setup(watchdog, WDT_OPT_PAUSE_HALTED_BY_DBG) != 0)) {
			status = ARCLANG_IO_ERR_DEVICE;
		}
	}
#endif
	return status;
}

arclang_io_status arclang_io_gpio_read(uint16_t pin, bool *level)
{
	int value;

	if (!ready(gpio_in) || (pin >= gpio_in_lines) || (pin >= 32U)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if ((gpio_in_ready & BIT(pin)) == 0U) {
		if (gpio_pin_configure(gpio_in, (gpio_pin_t)pin, GPIO_INPUT) != 0) {
			return ARCLANG_IO_ERR_DEVICE;
		}
		gpio_in_ready |= BIT(pin);
	}
	value = gpio_pin_get_raw(gpio_in, (gpio_pin_t)pin);
	if (value < 0) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	*level = (value != 0);
	return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_gpio_write(uint16_t pin, bool level)
{
	if (!ready(gpio_out) || (pin >= gpio_out_lines) || (pin >= 32U)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if ((gpio_out_ready & BIT(pin)) == 0U) {
		if (gpio_pin_configure(gpio_out, (gpio_pin_t)pin, GPIO_OUTPUT) != 0) {
			return ARCLANG_IO_ERR_DEVICE;
		}
		gpio_out_ready |= BIT(pin);
	}
	return (gpio_pin_set_raw(gpio_out, (gpio_pin_t)pin, level ? 1 : 0) == 0) ? ARCLANG_IO_OK : ARCLANG_IO_ERR_DEVICE;
}

arclang_io_status arclang_io_adc_read(uint16_t channel, uint16_t *raw)
{
	uint16_t sample = 0U;
	struct adc_sequence sequence = {
		.channels = BIT(channel),
		.buffer = &sample,
		.buffer_size = sizeof(sample),
		.resolution = ARCLANG_IO_ADC_RESOLUTION,
	};

	if (!ready(adc) || (channel >= 32U)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if ((adc_ready & BIT(channel)) == 0U) {
		const struct adc_channel_cfg cfg = {
			.gain = ADC_GAIN_1,
			.reference = ADC_REF_INTERNAL,
			.acquisition_time = ADC_ACQ_TIME_DEFAULT,
			.channel_id = (uint8_t)channel,
		};
		if (adc_channel_setup(adc, &cfg) != 0) {
			return ARCLANG_IO_ERR_CHANNEL;
		}
		adc_ready |= BIT(channel);
	}
	if (adc_read(adc, &sequence) != 0) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	*raw = sample;
	return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_dac_write(uint16_t channel, uint16_t raw)
{
	if (!ready(dac) || (channel >= 32U)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if ((dac_ready & BIT(channel)) == 0U) {
		const struct dac_channel_cfg cfg = { .channel_id = (uint8_t)channel, .resolution = 16U };

		if (dac_channel_setup(dac, &cfg) != 0) {
			return ARCLANG_IO_ERR_CHANNEL;
		}
		dac_ready |= BIT(channel);
	}
	return (dac_write_value(dac, (uint8_t)channel, raw) == 0) ? ARCLANG_IO_OK : ARCLANG_IO_ERR_DEVICE;
}

arclang_io_status arclang_io_pwm_write(uint16_t channel, uint16_t permille)
{
	uint64_t cycles_per_sec = 0U;
	uint32_t period;
	const struct device *dev = (channel < ARRAY_SIZE(pwms)) ? pwms[channel] : NULL;

	if (!ready(dev) || (permille > 1000U)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if (pwm_get_cycles_per_sec(dev, 0U, &cycles_per_sec) != 0) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	/* A period of a whole number of thousandths, so the duty is exact. */
	period = (uint32_t)((cycles_per_sec / ARCLANG_IO_PWM_HZ) / 1000U) * 1000U;
	if (period == 0U) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	return (pwm_set_cycles(dev, 0U, period, (period / 1000U) * permille, PWM_POLARITY_NORMAL) == 0) ? ARCLANG_IO_OK : ARCLANG_IO_ERR_DEVICE;
}

arclang_io_status arclang_io_can_receive(uint32_t id, uint8_t data[8], uint8_t *length, bool *received)
{
	*received = false;
	*length = 0U;
	if (!ready(can)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
#if ARCLANG_IO_CAN_RX_COUNT > 0
	for (size_t k = 0U; k < ARCLANG_IO_CAN_RX_COUNT; k++) {
		if (can_rx[k].id == id) {
			unsigned int key = irq_lock();

			if (can_rx[k].pending) {
				memcpy(data, can_rx[k].data, 8U);
				*length = can_rx[k].length;
				*received = true;
				can_rx[k].pending = false;
			}
			irq_unlock(key);
			return ARCLANG_IO_OK;
		}
	}
#endif
	ARG_UNUSED(data);
	/* An identifier no filter was installed for (not in the binding). */
	return ARCLANG_IO_ERR_CHANNEL;
}

arclang_io_status arclang_io_can_send(uint32_t id, const uint8_t data[8], uint8_t length)
{
	struct can_frame frame = { .id = id, .dlc = length, .flags = can_flags(id) };

	if (!ready(can)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if (length > 8U) {
		return ARCLANG_IO_ERR_DEVICE;
	}
	memcpy(frame.data, data, length);
	switch (can_send(can, &frame, K_NO_WAIT, NULL, NULL)) {
	case 0:
		return ARCLANG_IO_OK;
	case -EAGAIN:
		return ARCLANG_IO_ERR_FULL;
	default:
		return ARCLANG_IO_ERR_DEVICE;
	}
}

arclang_io_status arclang_io_uart_read(uint16_t port, uint8_t *byte, bool *received)
{
	unsigned char c;
	const struct device *dev = (port < ARRAY_SIZE(uarts)) ? uarts[port] : NULL;
	int err;

	*received = false;
	if (!ready(dev)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	err = uart_poll_in(dev, &c);
	if (err == 0) {
		*byte = c;
		*received = true;
		return ARCLANG_IO_OK;
	}
	return (err == -1) ? ARCLANG_IO_OK : ARCLANG_IO_ERR_DEVICE;
}

arclang_io_status arclang_io_uart_write(uint16_t port, uint8_t byte)
{
	const struct device *dev = (port < ARRAY_SIZE(uarts)) ? uarts[port] : NULL;

	if (!ready(dev)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	uart_poll_out(dev, byte);
	return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_clock_wait_period(uint32_t period_us)
{
	/* A periodic kernel timer, started at the first call: the cycles keep
	   the period whatever each one took (no drift), and a cycle longer
	   than the period is followed at once by the next. */
	if (period_started_us != period_us) {
		k_timer_init(&period_timer, NULL, NULL);
		k_timer_start(&period_timer, K_USEC(period_us), K_USEC(period_us));
		period_started_us = period_us;
	}
	(void)k_timer_status_sync(&period_timer);
	return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_task_wait_period(uint8_t task, uint32_t period_us)
{
	/* As the clock above, one timer per task: each keeps its own period
	   whatever the others take; the kernel schedules them by priority. */
	if (task >= ARCLANG_IO_TASKS) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	if (task_started_us[task] != period_us) {
		k_timer_init(&task_timers[task], NULL, NULL);
		k_timer_start(&task_timers[task], K_USEC(period_us), K_USEC(period_us));
		task_started_us[task] = period_us;
	}
	(void)k_timer_status_sync(&task_timers[task]);
	return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_watchdog_kick(void)
{
	if (!ready(watchdog) || (watchdog_channel < 0)) {
		return ARCLANG_IO_ERR_CHANNEL;
	}
	return (wdt_feed(watchdog, watchdog_channel) == 0) ? ARCLANG_IO_OK : ARCLANG_IO_ERR_DEVICE;
}
