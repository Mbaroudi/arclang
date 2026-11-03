/*
 * ArcLang target zephyr/litex_vexriscv: the front end of Zephyr's SJA1000
 * driver for an SJA1000-compatible CAN core on the LiteX bus, one 8-bit
 * register per 32-bit word (as the OpenCores SJA1000-compatible core is
 * wired on a 32-bit Wishbone bus). Modelled on Zephyr's Kvaser PCIcan
 * front end.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#define DT_DRV_COMPAT arclang_litex_sja1000

#include <zephyr/drivers/can/can_sja1000.h>
#include <zephyr/drivers/can.h>
#include <zephyr/kernel.h>
#include <zephyr/sys/sys_io.h>

struct can_litex_sja1000_config {
	mem_addr_t base;
	uint32_t clock_frequency;
	void (*irq_config_func)(const struct device *dev);
};

static uint8_t can_litex_sja1000_read_reg(const struct device *dev, uint8_t reg)
{
	const struct can_sja1000_config *sja1000_config = dev->config;
	const struct can_litex_sja1000_config *config = sja1000_config->custom;

	return (uint8_t)sys_read32(config->base + ((mem_addr_t)reg * 4U));
}

static void can_litex_sja1000_write_reg(const struct device *dev, uint8_t reg, uint8_t val)
{
	const struct can_sja1000_config *sja1000_config = dev->config;
	const struct can_litex_sja1000_config *config = sja1000_config->custom;

	sys_write32(val, config->base + ((mem_addr_t)reg * 4U));
}

static int can_litex_sja1000_get_core_clock(const struct device *dev, uint32_t *rate)
{
	const struct can_sja1000_config *sja1000_config = dev->config;
	const struct can_litex_sja1000_config *config = sja1000_config->custom;

	/* The internal clock runs at half of the oscillator frequency */
	*rate = config->clock_frequency / 2U;
	return 0;
}

static int can_litex_sja1000_init(const struct device *dev)
{
	const struct can_sja1000_config *sja1000_config = dev->config;
	const struct can_litex_sja1000_config *config = sja1000_config->custom;
	int err = can_sja1000_init(dev);

	if (err != 0) {
		return err;
	}
	config->irq_config_func(dev);
	return 0;
}

static DEVICE_API(can, can_litex_sja1000_api) = {
	.get_capabilities = can_sja1000_get_capabilities,
	.start = can_sja1000_start,
	.stop = can_sja1000_stop,
	.set_mode = can_sja1000_set_mode,
	.set_timing = can_sja1000_set_timing,
	.send = can_sja1000_send,
	.add_rx_filter = can_sja1000_add_rx_filter,
	.remove_rx_filter = can_sja1000_remove_rx_filter,
	.get_state = can_sja1000_get_state,
	.set_state_change_callback = can_sja1000_set_state_change_callback,
	.get_core_clock = can_litex_sja1000_get_core_clock,
	.get_max_filters = can_sja1000_get_max_filters,
#ifdef CONFIG_CAN_MANUAL_RECOVERY_MODE
	.recover = can_sja1000_recover,
#endif
	.timing_min = CAN_SJA1000_TIMING_MIN_INITIALIZER,
	.timing_max = CAN_SJA1000_TIMING_MAX_INITIALIZER,
};

#define CAN_LITEX_SJA1000_OCR (CAN_SJA1000_OCR_OCMODE_NORMAL | CAN_SJA1000_OCR_OCTN0 | CAN_SJA1000_OCR_OCTP0)
#define CAN_LITEX_SJA1000_CDR (CAN_SJA1000_CDR_CD_DIV2 | CAN_SJA1000_CDR_CLOCK_OFF)

#define CAN_LITEX_SJA1000_INIT(inst)                                                            \
	static void can_litex_sja1000_irq_##inst(const struct device *dev)                      \
	{                                                                                        \
		IRQ_CONNECT(DT_INST_IRQN(inst), DT_INST_IRQ(inst, priority), can_sja1000_isr,    \
			    DEVICE_DT_INST_GET(inst), 0);                                        \
		irq_enable(DT_INST_IRQN(inst));                                                  \
	}                                                                                        \
	static const struct can_litex_sja1000_config can_litex_sja1000_custom_##inst = {        \
		.base = DT_INST_REG_ADDR(inst),                                                  \
		.clock_frequency = DT_INST_PROP(inst, clock_frequency),                          \
		.irq_config_func = can_litex_sja1000_irq_##inst,                                 \
	};                                                                                       \
	static const struct can_sja1000_config can_sja1000_config_##inst =                       \
		CAN_SJA1000_DT_CONFIG_INST_GET(inst, &can_litex_sja1000_custom_##inst,           \
					       can_litex_sja1000_read_reg,                       \
					       can_litex_sja1000_write_reg,                      \
					       CAN_LITEX_SJA1000_OCR, CAN_LITEX_SJA1000_CDR, 0); \
	static struct can_sja1000_data can_sja1000_data_##inst =                                 \
		CAN_SJA1000_DATA_INITIALIZER(NULL);                                              \
	CAN_DEVICE_DT_INST_DEFINE(inst, can_litex_sja1000_init, NULL, &can_sja1000_data_##inst, \
				  &can_sja1000_config_##inst, POST_KERNEL,                       \
				  CONFIG_CAN_INIT_PRIORITY, &can_litex_sja1000_api);

DT_INST_FOREACH_STATUS_OKAY(CAN_LITEX_SJA1000_INIT)
