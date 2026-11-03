# The bench of ArcLang's processor in the loop, run inside Renode (its
# IronPython 2.7): it loads the SoC and the image, then, step by step,
# drives the bound input channels (or sends the step's inputs on the
# console), runs the emulation until the target reports the step, and reads
# the bound output channels. Everything runs on Renode's virtual time: the
# same image and the same plan give the same results, whatever the host.
#
# The plan (ARCLANG_PIL_PLAN, written by `arclang pil`) is a list of lines:
#   mode io|direct, elf, platform, peripheral (C# model), channel names,
#   output <kind> ..., quantum_us, boot_us, step_us, result, then per step
#   `step`, its operations (g pin level, a channel raw, c id byte0,
#   u port byte, line <text>) and `.`.
# The result file: `ready <us>`, one line per step `R ...|io values`, or
# `error <why>`.
import System
from Antmicro.Renode.Core import EmulationManager
from Antmicro.Renode.Time import TimeInterval

plan_path = System.Environment.GetEnvironmentVariable("ARCLANG_PIL_PLAN")
config = {"peripheral": [], "output": [], "pwm": {}, "uart": {}}
steps = []
current = None
for raw in open(plan_path).read().splitlines():
    if raw.strip() == "":
        continue
    if current is not None:
        if raw == ".":
            steps.append(current)
            current = None
        else:
            current.append(raw)
        continue
    word, _, rest = raw.partition(" ")
    if word == "step":
        current = []
    elif word in ("peripheral", "output"):
        config[word].append(rest)
    elif word in ("pwm", "uart"):
        k, _, name = rest.partition(" ")
        config[word][int(k)] = name
    else:
        config[word] = rest

result = open(config["result"], "w")


def out(text):
    result.write(text + "\n")
    result.flush()


for path in config["peripheral"]:
    monitor.Parse("include @" + path)
monitor.Parse("mach create")
monitor.Parse("machine LoadPlatformDescription @" + config["platform"])
monitor.Parse("sysbus LoadELF @" + config["elf"])
emulation = EmulationManager.Instance.CurrentEmulation
machine = list(emulation.Machines)[0]
bus = machine.SystemBus

console = machine[config["console"]]
lines = []
partial = []


def on_console(c):
    if c == 10:
        lines.append("".join(partial))
        del partial[:]
    elif c != 13:
        partial.append(chr(c))


console.CharReceived += on_console

sent = {}


def watch_uart(port, peripheral):
    def on_byte(c):
        key = (port, c)
        sent[key] = sent.get(key, 0) + 1
    peripheral.CharReceived += on_byte


uarts = {}
for port, name in config["uart"].items():
    uarts[port] = machine[name]
    watch_uart(port, uarts[port])

quantum = TimeInterval.FromMicroseconds(int(config.get("quantum_us", "100")))


def now_us():
    return machine.ElapsedVirtualTime.TimeElapsed.TotalMicroseconds


def run_until(predicate, budget_us):
    start = now_us()
    while not predicate():
        if now_us() - start > budget_us:
            return False
        emulation.RunFor(quantum)
    return True


def errors():
    return [l for l in lines if l.startswith("ARCLANG ERROR")]


def apply(op):
    word, _, rest = op.partition(" ")
    if word == "line":
        for ch in rest:
            console.WriteChar(ord(ch))
        console.WriteChar(10)
        return
    a, b = [int(x, 0) for x in rest.split()]
    if word == "g":
        machine[config["gpio_in"]].OnGPIO(a, b != 0)
    elif word == "a":
        machine[config["adc"]].SetRaw(a, b)
    elif word == "c":
        machine[config["can"]].Receive(a, b, 1)
    elif word == "u":
        uarts[a].WriteChar(b)
    else:
        raise Exception("unknown operation " + op)


def outputs():
    values = []
    for spec in config["output"]:
        words = spec.split()
        kind = words[0]
        if kind == "gpio":
            register = bus.ReadDoubleWord(int(config["gpio_out"], 0))
            values.append((register >> int(words[1])) & 1)
        elif kind == "pwm":
            values.append(machine[config["pwm"][int(words[1])]].Permille)
        elif kind == "can":
            values.append(machine[config["can"]].LastSent(int(words[1], 0)))
        elif kind == "uart":
            values.append(sent.get((int(words[1]), int(words[2], 0)), 0))
        elif kind == "watchdog":
            values.append(machine[config["watchdog"]].Feeds - kicks_at_start[0])
        else:
            raise Exception("unknown output " + spec)
    return ",".join([str(v) for v in values])


kicks_at_start = [0]
try:
    if not run_until(lambda: any(l.startswith("ARCLANG READY") or l.startswith("ARCLANG ERROR") for l in lines), int(config.get("boot_us", "2000000"))):
        out("error the target did not report ready within %s us of virtual time (console: %s)" % (config.get("boot_us"), " / ".join(lines[-5:] + ["".join(partial)])))
    elif errors():
        out("error " + errors()[0][len("ARCLANG ERROR "):])
    else:
        out("ready %d" % now_us())
        if "watchdog" in config:
            kicks_at_start[0] = machine[config["watchdog"]].Feeds
        budget = int(config.get("step_us", "1000000"))
        for k, ops in enumerate(steps):
            for op in ops:
                apply(op)
            if k == 0 and config["mode"] == "io":
                console.WriteChar(ord("G"))
            for key in list(sent.keys()):
                sent[key] = 0
            reports = [l for l in lines if l.startswith("R ")]
            wanted = len(reports) + 1
            if not run_until(lambda: len([l for l in lines if l.startswith("R ")]) >= wanted or errors(), budget):
                out("error no report of step %d within %d us of virtual time" % (k, budget))
                break
            if errors():
                out("error at step %d: %s" % (k, errors()[0]))
                break
            report = [l for l in lines if l.startswith("R ")][wanted - 1]
            out(report + "|" + outputs())
            if config["mode"] == "io":
                # The outputs of the step are read; the next step's inputs
                # go in before its cycle (well inside the period).
                pass
except Exception as e:
    out("error the bench failed: %s" % e)
result.close()
