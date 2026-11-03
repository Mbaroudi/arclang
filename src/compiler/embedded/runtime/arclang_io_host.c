/*
 * arclang_io_host.c -- arclang_io.h on a PC: a simulated board. Its state
 * is the board's, file-scope by nature (as registers are); generated code
 * never sees it.
 */
#include "arclang_io_host.h"

typedef struct {
    bool used;
    uint32_t id;
    bool pending;
    uint8_t data[8];
    uint8_t length;
} host_mailbox;

typedef struct {
    uint8_t bytes[ARCLANG_IO_HOST_UART_QUEUE];
    uint16_t head;
    uint16_t count;
} host_queue;

static bool gpio_in[ARCLANG_IO_HOST_PINS];
static bool gpio_out[ARCLANG_IO_HOST_PINS];
static uint16_t adc[ARCLANG_IO_HOST_ANALOG];
static uint16_t dac[ARCLANG_IO_HOST_ANALOG];
static uint16_t pwm[ARCLANG_IO_HOST_ANALOG];
static host_mailbox can_rx[ARCLANG_IO_HOST_CAN_IDS];
static host_mailbox can_tx[ARCLANG_IO_HOST_CAN_IDS];
static host_queue uart_rx[ARCLANG_IO_HOST_UARTS];
static host_queue uart_tx[ARCLANG_IO_HOST_UARTS];
static uint32_t kicks;
static uint64_t now_us;

void arclang_io_host_reset(void)
{
    uint16_t k;
    for (k = 0U; k < ARCLANG_IO_HOST_PINS; k++) {
        gpio_in[k] = false;
        gpio_out[k] = false;
    }
    for (k = 0U; k < ARCLANG_IO_HOST_ANALOG; k++) {
        adc[k] = 0U;
        dac[k] = 0U;
        pwm[k] = 0U;
    }
    for (k = 0U; k < ARCLANG_IO_HOST_CAN_IDS; k++) {
        can_rx[k].used = false;
        can_rx[k].pending = false;
        can_tx[k].used = false;
        can_tx[k].pending = false;
    }
    for (k = 0U; k < ARCLANG_IO_HOST_UARTS; k++) {
        uart_rx[k].head = 0U;
        uart_rx[k].count = 0U;
        uart_tx[k].head = 0U;
        uart_tx[k].count = 0U;
    }
    kicks = 0U;
    now_us = 0U;
}

static host_mailbox *mailbox(host_mailbox *boxes, uint32_t id, bool create)
{
    uint16_t k;
    for (k = 0U; k < ARCLANG_IO_HOST_CAN_IDS; k++) {
        if (boxes[k].used && (boxes[k].id == id)) {
            return &boxes[k];
        }
    }
    if (create) {
        for (k = 0U; k < ARCLANG_IO_HOST_CAN_IDS; k++) {
            if (!boxes[k].used) {
                boxes[k].used = true;
                boxes[k].id = id;
                boxes[k].pending = false;
                boxes[k].length = 0U;
                return &boxes[k];
            }
        }
    }
    return (host_mailbox *)0;
}

static arclang_io_status put(host_mailbox *box, const uint8_t data[8], uint8_t length)
{
    uint8_t k;
    if (box == (host_mailbox *)0) {
        return ARCLANG_IO_ERR_FULL;
    }
    if (length > 8U) {
        return ARCLANG_IO_ERR_DEVICE;
    }
    for (k = 0U; k < 8U; k++) {
        box->data[k] = (k < length) ? data[k] : 0U;
    }
    box->length = length;
    box->pending = true;
    return ARCLANG_IO_OK;
}

static arclang_io_status enqueue(host_queue *queue, uint8_t byte)
{
    if (queue->count >= ARCLANG_IO_HOST_UART_QUEUE) {
        return ARCLANG_IO_ERR_FULL;
    }
    queue->bytes[(queue->head + queue->count) % ARCLANG_IO_HOST_UART_QUEUE] = byte;
    queue->count++;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_init(void)
{
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_gpio_read(uint16_t pin, bool *level)
{
    if (pin >= ARCLANG_IO_HOST_PINS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    *level = gpio_in[pin];
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_gpio_write(uint16_t pin, bool level)
{
    if (pin >= ARCLANG_IO_HOST_PINS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    gpio_out[pin] = level;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_adc_read(uint16_t channel, uint16_t *raw)
{
    if (channel >= ARCLANG_IO_HOST_ANALOG) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    *raw = adc[channel];
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_dac_write(uint16_t channel, uint16_t raw)
{
    if (channel >= ARCLANG_IO_HOST_ANALOG) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    dac[channel] = raw;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_pwm_write(uint16_t channel, uint16_t permille)
{
    if ((channel >= ARCLANG_IO_HOST_ANALOG) || (permille > 1000U)) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    pwm[channel] = permille;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_can_receive(uint32_t id, uint8_t data[8], uint8_t *length, bool *received)
{
    uint8_t k;
    host_mailbox *box = mailbox(can_rx, id, false);
    *received = false;
    *length = 0U;
    if ((box != (host_mailbox *)0) && box->pending) {
        for (k = 0U; k < 8U; k++) {
            data[k] = box->data[k];
        }
        *length = box->length;
        *received = true;
        box->pending = false;
    }
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_can_send(uint32_t id, const uint8_t data[8], uint8_t length)
{
    return put(mailbox(can_tx, id, true), data, length);
}

arclang_io_status arclang_io_uart_read(uint16_t port, uint8_t *byte, bool *received)
{
    host_queue *queue;
    if (port >= ARCLANG_IO_HOST_UARTS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    queue = &uart_rx[port];
    *received = false;
    if (queue->count > 0U) {
        *byte = queue->bytes[queue->head];
        queue->head = (uint16_t)((queue->head + 1U) % ARCLANG_IO_HOST_UART_QUEUE);
        queue->count--;
        *received = true;
    }
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_uart_write(uint16_t port, uint8_t byte)
{
    if (port >= ARCLANG_IO_HOST_UARTS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    return enqueue(&uart_tx[port], byte);
}

arclang_io_status arclang_io_clock_wait_period(uint32_t period_us)
{
    now_us += period_us;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_watchdog_kick(void)
{
    kicks++;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_host_set_gpio(uint16_t pin, bool level)
{
    if (pin >= ARCLANG_IO_HOST_PINS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    gpio_in[pin] = level;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_host_set_adc(uint16_t channel, uint16_t raw)
{
    if (channel >= ARCLANG_IO_HOST_ANALOG) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    adc[channel] = raw;
    return ARCLANG_IO_OK;
}

arclang_io_status arclang_io_host_push_can(uint32_t id, const uint8_t data[8], uint8_t length)
{
    return put(mailbox(can_rx, id, true), data, length);
}

arclang_io_status arclang_io_host_push_uart(uint16_t port, uint8_t byte)
{
    if (port >= ARCLANG_IO_HOST_UARTS) {
        return ARCLANG_IO_ERR_CHANNEL;
    }
    return enqueue(&uart_rx[port], byte);
}

bool arclang_io_host_gpio_output(uint16_t pin)
{
    return (pin < ARCLANG_IO_HOST_PINS) ? gpio_out[pin] : false;
}

uint16_t arclang_io_host_dac(uint16_t channel)
{
    return (channel < ARCLANG_IO_HOST_ANALOG) ? dac[channel] : 0U;
}

uint16_t arclang_io_host_pwm(uint16_t channel)
{
    return (channel < ARCLANG_IO_HOST_ANALOG) ? pwm[channel] : 0U;
}

bool arclang_io_host_can_sent(uint32_t id, uint8_t data[8], uint8_t *length)
{
    uint8_t k;
    host_mailbox *box = mailbox(can_tx, id, false);
    if ((box == (host_mailbox *)0) || !box->pending) {
        return false;
    }
    for (k = 0U; k < 8U; k++) {
        data[k] = box->data[k];
    }
    *length = box->length;
    return true;
}

uint16_t arclang_io_host_uart_count(uint16_t port, uint8_t byte)
{
    uint16_t matches = 0U;
    uint16_t k;
    const host_queue *queue;
    if (port >= ARCLANG_IO_HOST_UARTS) {
        return 0U;
    }
    queue = &uart_tx[port];
    for (k = 0U; k < queue->count; k++) {
        if (queue->bytes[(queue->head + k) % ARCLANG_IO_HOST_UART_QUEUE] == byte) {
            matches++;
        }
    }
    return matches;
}

void arclang_io_host_uart_clear(uint16_t port)
{
    if (port < ARCLANG_IO_HOST_UARTS) {
        uart_tx[port].head = 0U;
        uart_tx[port].count = 0U;
    }
}

uint32_t arclang_io_host_watchdog_kicks(void)
{
    return kicks;
}

uint64_t arclang_io_host_time_us(void)
{
    return now_us;
}
