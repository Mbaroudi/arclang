# The drivers ArcLang's LiteX target adds: an SJA1000-compatible CAN core
# (the front end of Zephyr's SJA1000 driver) and an ADC register bank.
zephyr_library_named(arclang_litex)
zephyr_library_sources_ifdef(CONFIG_CAN_ARCLANG_LITEX_SJA1000 drivers/can_litex_sja1000.c)
zephyr_library_sources_ifdef(CONFIG_ADC_ARCLANG_LITEX drivers/adc_litex_regs.c)
zephyr_library_include_directories(${ZEPHYR_BASE}/drivers/can)
