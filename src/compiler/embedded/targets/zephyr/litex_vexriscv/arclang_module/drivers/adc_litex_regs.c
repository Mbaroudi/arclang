/*
 * ArcLang target zephyr/litex_vexriscv: the ADC register bank, one 32-bit
 * read-only register per channel holding the last raw conversion, behind
 * Zephyr's ADC API. A read copies the registers of the selected channels,
 * lowest channel first; any resolution up to 16 bits is accepted, the raw
 * value is returned as the register holds it.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#define DT_DRV_COMPAT arclang_litex_adc

#include <zephyr/device.h>
#include <zephyr/drivers/adc.h>
#include <zephyr/sys/sys_io.h>
#include <zephyr/sys/util.h>
#include <errno.h>

struct adc_litex_regs_config {
	mem_addr_t base;
	uint8_t channels;
};

static int adc_litex_regs_channel_setup(const struct device *dev, const struct adc_channel_cfg *cfg)
{
	const struct adc_litex_regs_config *config = dev->config;

	if (cfg->channel_id >= config->channels || cfg->differential) {
		return -EINVAL;
	}
	return 0;
}

static int adc_litex_regs_read(const struct device *dev, const struct adc_sequence *sequence)
{
	const struct adc_litex_regs_config *config = dev->config;
	uint16_t *out = sequence->buffer;
	size_t used = 0U;

	if ((sequence->resolution == 0U) || (sequence->resolution > 16U)) {
		return -EINVAL;
	}
	if ((sequence->channels >> config->channels) != 0U) {
		return -EINVAL;
	}
	for (uint8_t k = 0U; k < config->channels; k++) {
		if ((sequence->channels & BIT(k)) == 0U) {
			continue;
		}
		if ((used + 1U) * sizeof(uint16_t) > sequence->buffer_size) {
			return -ENOMEM;
		}
		out[used] = (uint16_t)sys_read32(config->base + ((mem_addr_t)k * 4U));
		used++;
	}
	return 0;
}

static DEVICE_API(adc, adc_litex_regs_api) = {
	.channel_setup = adc_litex_regs_channel_setup,
	.read = adc_litex_regs_read,
	.ref_internal = 3300,
};

#define ADC_LITEX_REGS_INIT(inst)                                                    \
	static const struct adc_litex_regs_config adc_litex_regs_config_##inst = {  \
		.base = DT_INST_REG_ADDR(inst),                                      \
		.channels = DT_INST_PROP(inst, channels),                            \
	};                                                                           \
	DEVICE_DT_INST_DEFINE(inst, NULL, NULL, NULL, &adc_litex_regs_config_##inst, \
			      POST_KERNEL, CONFIG_ADC_INIT_PRIORITY, &adc_litex_regs_api);

DT_INST_FOREACH_STATUS_OKAY(ADC_LITEX_REGS_INIT)
