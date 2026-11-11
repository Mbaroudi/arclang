# Reverse engineering

`arclang recover` gives the model a piece of code implements. Its first
reader takes the C the compiler itself generates (`arclang export -f c`,
see [embedded-c.md](embedded-c.md)): the header and the source of a model's
state machines.

```console
$ arclang export speed_monitor.arc -f c -o c/
$ arclang recover c/ --check -o recovered.arc
Recovered model "SpeedMonitor" from c/speed_monitor.h and c/speed_monitor.c
  state_machine "Wheel speed filter": 1 state(s), 0 transition(s), 3 variable(s), 2 equation(s)
  state_machine "Overspeed warning": 2 state(s), 2 transition(s), 3 variable(s), 0 equation(s)
  Output: recovered.arc
  note: not in the code: the name of a region (named here by its rank), whether a variable is `out` or `const`, the range of a variable, the priority of a transition
✓ Proven: the recovered model generates speed_monitor.h and speed_monitor.c as they were read
```

The input is a directory holding `<model>.h` and `<model>.c`, or one of the
two files. Without `-o` the recovered model is printed.

## What is read where

| Read in the statements of the code | Read in a comment of the generator |
|---|---|
| the machines, their states and the order of the states | the names as the model writes them |
| where a machine starts, and where a composite state does | the units of the variables |
| the regions of a machine or of a state, their states and where each starts | the kind of a pseudo-state |
| the transitions through a pseudo-state: the branches of a choice or of a junction with their guards, the targets of a fork, the sources of a join, an entry by history, a terminate | the `id` of a transition |
| the level each transition is written at (the order the code numbers them in) | a composite state that names no initial state |
| the source and the target of each transition | the text of each equation |
| its guard (`in->v_speed > st->v_on_above` is `speed > on_above`) | the text of each action and condition in prose |
| the time after which it fires, or the time it has to fire | the name of each event |
| the type and the initial value of each variable | |
| an input wired from another machine | |

A comment can say anything, and a C file can be edited by hand. So what a
comment says is never taken on trust: `--check` compiles the recovered model,
generates its C again and compares it with the C that was read, line by
line. Equal files prove the recovery, comments included. A difference is
printed with its line, and the command fails:

```console
✗ Not proven: 1 line(s) of the C differ from what the recovered model generates
  speed_monitor.c:60
    read:        st->v_filtered = (st->v_filtered + (0.25 * (in->v_raw - st->v_filtered)));
    regenerated: st->v_filtered = (st->v_filtered + (0.5 * (in->v_raw - st->v_filtered)));
```

Here the comment above the statement had been changed to say `0.5`: the
code still computes with `0.25`.

## What is not recovered

- **What the C does not hold**: whether a variable is `out` or `const`, the
  range of a variable, the priority of a transition, the names of the
  regions (a region is named by its rank: `region "Region 1"`). The recovered model
  generates the same C without them; it is the model the code implements,
  not the file the code was generated from.
- **The step period**: it comes from the `io_binding` of the model, which
  the machines' C does not hold. It is read and reported, not proven.
- **Not read**: a transition whose function enters no state and ends
  nothing. The reader leaves it out and says so in a note; the C then
  differs, and `--check` fails. None of the examples has one.
- **Other code**: C written by hand or by another generator, VHDL, and the
  rest of the model (requirements, architecture, bindings) are not read.

A region is read in the code: a transition exits the states of the region
it is in, and the states a machine or a state enters together are where its
regions start. A transition through a pseudo-state is read in the body of
its function: the `if / else if` of a choice, the `switch` of a junction
whose branch was chosen before it fired, the states and histories a fork
enters, the states a join waits for.

The recovered model is not always written as the original was: a target
written `Up` may come back as `Moving`, the state that holds it, when `Up`
is where `Moving` starts. Both generate the same C; it is what `--check`
proves.

The 27 examples of the repository that generate C are all recovered and
proven (`cargo test --test recover -- --ignored --nocapture` prints the
survey), among them the power window (regions, a choice, a junction, a
fork, a join, both histories, a terminate) and the in-flight entertainment
model imported from Capella (five machines).

## The programmable logic

The VHDL the compiler generates follows the same machines as the C, in
fixed point: its words are sized from the range of each variable, which the
C does not hold. The register map written beside the VHDL
(`<block>_regs.json`) says, for each variable, its format, the range it was
sized for and where that range comes from (declared, set by a format, the
value of a constant, derived). With `--hdl`, the machines recovered from the
C say what the logic needs, and the proof is the same as for the C:

```console
$ arclang export battery_charger.arc -f c -o c/
$ arclang export battery_charger.arc -f vhdl -o hdl/
$ arclang recover c/battery_charger.h --hdl hdl/ --check -o recovered.arc
✓ Proven: the recovered model generates battery_charger.h and battery_charger.c as they were read
✓ Proven: the recovered model generates the 8 file(s) of the logic in hdl/ as they were read
```

The eight files are the VHDL of the machine, its Wishbone wrapper, the
fixed-point package, the register map (JSON and CSV), the driver and the
fixed-point C. The recovered model then declares `range 0 V .. 60 V` on a
variable whose range was declared, `const` on a constant, nothing on a
variable whose range the compiler derives, and gives each machine of the
logic a node of kind `fpga`. A statement of the VHDL changed by hand makes
`--check` fail, with the file and the line.

The logic is read with the C of the same model: the machines are read in the
C, the VHDL is proven against them. What the files of the logic do not
hold is not recovered: the device and the clock of the node, its name. A
word narrower than 32 bits, sized by a `resolution`, is not proven (a note
says so). A register map written before this version does not say the
ranges, and is refused with that reason: export the logic again.

The three examples of the repository that generate VHDL (the speed
monitor's filter, the battery charger, the power window with every
pseudo-state) are proven, C and logic.

## Drift: the code as built against the model

`arclang drift <model> <code>` says whether the C as built is still the
model, and where it is not. `<code>` is given as for `recover`.

```console
$ arclang drift speed_monitor.arc c/
Drift of c/speed_monitor.h and c/speed_monitor.c against speed_monitor.arc
✓ No drift: the code is, byte for byte, the code the model generates
```

After three edits by hand in `c/speed_monitor.c` (a guard, an initial
value, the statement of an equation):

```console
$ arclang drift speed_monitor.arc c/
Drift of c/speed_monitor.h and c/speed_monitor.c against speed_monitor.arc
  state_machine "Wheel speed filter"
    ✗ equation `filtered = filtered + 0.25 * (raw - filtered)` is computed in the code as `st->v_filtered = (st->v_filtered + (0.3 * (in->v_raw - st->v_filtered)));`; the model generates `st->v_filtered = (st->v_filtered + (0.25 * (in->v_raw - st->v_filtered)));` (speed_monitor.c:60)
  state_machine "Overspeed warning"
    ✗ variable 'on_above' starts at `135.0` in the code; the model says `130.0` (speed_monitor.c:189)
    ✗ the guard of transition Normal -> Warning is `speed >= on_above` in the code; the model says `speed > on_above` (speed_monitor.c:150)
Error: Compilation error: the code has drifted from the model: 3 difference(s) in its machines, 0 line(s) they do not explain
```

The command fails when there is a difference; `--json` prints the same
findings with their machine, file and line.

How it compares. The model generates its C; the machines are recovered from
that C and from the C as built, by the same reader, and compared: the
machines, their variables with their initial values and their wiring, their
equations (by the statement that computes each), their states with their
actions, their regions, their transitions with their guard, time and
actions. Reading both sides the same way keeps a difference of writing (a
target named by the state that holds it, a region named by its rank) from
being reported as one of behaviour.

What a comparison of machines cannot see is then looked for in the code
itself: the C as built is compared with the C its own recovered machines
generate, and the lines they do not explain are listed. This is where a
helper edited by hand shows (`speed_monitor.c:18: r = hi;`), and a comment
that no longer says what its statement does is reported as such.

Every other file the model generates is compared too, whole, with the file
as built: the I/O interface, the glue of each binding and the host
implementation beside the machines' C, and, with `--hdl <dir>`, the files of
the programmable logic (`arclang export -f vhdl`: the VHDL, the register
map, the driver, the fixed-point C). For these files the difference is
shown as lines, the file as built against what the model generates, not in
the terms of the model:

```console
    ✗ hdl/speed_monitor_wheel_speed_filter.vhd:76: 2 line(s) of the file as built where the model generates 1
        built:         ready <= '0';
        built:         -- patched on the bench
        model:         ready <= '1';
```

A generated file that is not among the files as built is said, and is not a
drift. Every line found is listed: there is no limit.

Not compared: anything the C does not hold (see above), and code the
compiler did not generate. A drift needs the code the model generates to
compare with; C or VHDL written by hand has no such reference, and reading
a model out of it is another problem than this one.

## Proving that two codes do the same

`drift` says where the C as built differs from the model's C. It cannot say
that two codes written differently do the same: a temporary variable, a
comparison turned round, a loop unrolled are differences of text. `--prove`
answers that question:

```console
$ arclang drift speed_monitor.arc c/ --prove
  state_machine "Overspeed warning"
    ✗ the guard of transition Normal -> Warning is `on_above < speed` in the code; the model says `speed > on_above` (speed_monitor.c:150)
✓ Proven equivalent (0.6 s): from any state and for any inputs, one step of the code as built gives the state and the outputs one step of the model's code gives (19 fields compared)
```

One step of the code as built and one step of the model's code are run from
the same state, any state, with the same inputs, any inputs; every field of
the state and of the outputs is compared after the step, the floating-point
values bit for bit. If nothing makes a field differ, each step of one is a
step of the other, and so is every run. CBMC translates the two programs
and the comparison to one formula, every loop unwound to its bound (which is
checked); Bitwuzla decides it.

- `unsat`: **proven equivalent**. The text of the code may differ; the
  command succeeds.
- `sat`: **not equivalent**, with the state and the inputs from which one
  step differs:

  ```console
  ✗ Not equivalent (0.8 s): a state and inputs from which one step of the code as built does not give what the model's code gives:
      inputs.overspeed_warning.v_speed = 0
      state.overspeed_warning.v_on_above = 0
      state.overspeed_warning_started = false
      ...
  ```

  (the guard had become `>=`: at the threshold, the code as built warns and
  the model does not.)
- anything else (the time limit of `--timeout`, 300 s by default, or a tool
  that failed): **neither proven nor refuted**, said as such, never as a
  proof.

What the proof is of, exactly: the two step functions, for the structures of
the header, which must be the same in both codes (a code with another
interface is not compared). The starting state is any value of the state
structure, reachable or not, but for one condition: a shallow history names
a state of its machine or none. Two codes that differ only from a state no
run reaches are reported as not equivalent. The I/O glue and the host files
are not part of it.

How far it goes today, on the code the model generates itself (so the
answer is known): the speed monitor (floating-point blocks) is proven in
about a second, the level crossing (sixteen events a step) in half a
minute, the battery charger (regions, a PID, timed transitions, sixteen
events) in a minute and a half. The power window (regions, both histories, every
pseudo-state, sixteen events a step) gets no answer within 300 s: it is
reported as neither proven nor refuted. The cost grows with the number of
events one step can take.

The tools are not on a usual computer: `tools/embedded/formal` builds an
image with CBMC (from its source, version 6.4.1) on top of the FPGA image,
which holds Bitwuzla. `ARCLANG_CBMC` and `ARCLANG_BITWUZLA` name the tools
when they are elsewhere.

## Code written by hand: a correspondence, and its proof

The code above is the code the compiler generated, or a copy of it someone
edited: the model's own structures are in it. A sketch written by hand has
none of them. What ties it to a machine of the model is written down, in a
`correspondence`, and proven by `arclang prove`:

```
state_machine "Debounced button" {
    variables: ["in button: Boolean", "led: Boolean = false"]
    initial: "Low"
    state Low {}
    state Rising {}
    state High { entry: "led := true" }
    state Falling {}
    transition Low -> Rising { guard: "button" }
    transition Rising -> High { guard: "button" timing: 50 ms }
    ...
}

correspondence "Debounce sketch" {
    machine: "Debounced button"
    source: "debounce.ino"
    period: 10 ms
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LED) is led"]
    state: [
        "stable == 0 && counting == 0 is in Low",
        "stable == 0 && counting == 1 is in Rising",
        "stable == 1 && counting == 0 is in High",
        "stable == 1 && counting == 1 is in Falling",
        "millis() - since is time in Rising",
        "millis() - since is time in Falling",
        "stable == 1 is led"
    ]
    invariant: ["stable == 0 || stable == 1", "counting == 0 || counting == 1"]
}
```

- `source`: the file of the code (or a list of them), beside the model.
- `period`: the time of one `loop()`, which is one step of the machine.
- `reads`: the pin each input of the machine is read from.
- `writes`: the pin each output is written to; the level last written is
  the value of the variable.
- `state`: an expression of the code on the left, what it is in the model on
  the right: `in <State>` (the expression holds exactly when the state is
  active), `time in <State>` (the expression, in milliseconds, is the time
  the state has been held, while it is active), or a variable of the machine.
- `invariant`: what always holds of the code's own variables. It is proven
  with the rest, not taken on trust.
- `int`: the bits of an `int` on the board, `16` on an AVR board (the Uno,
  the Nano, the Mega) or `32` (the default). A product that fits in 32 bits
  and overflows in 16 is another behaviour, and another proof.

The compiler checks what a correspondence names when it compiles the model
(`arclang check`): the machine, the states and the variables are those of
the model, a read is an input, every line says what it is in the model.
Whether the lines hold is the proof's to say.

```
$ arclang prove debounce.arc
✓ Proven (1.5 s) correspondence "Debounce sketch": debounce.ino is state_machine "Debounced button"
    for one loop() a step of 10 ms, an int of 32 bits, the pins the correspondence names, and the C subset of the Arduino language
```

The proof has two parts, each asked of CBMC and Bitwuzla as the equivalence
above:

- **start**: after `setup()` and one `loop()`, and after the machine's start
  and one step with the same inputs, every line of the correspondence
  holds;
- **step**: from *any* state of the code and *any* state of the machine for
  which every line holds, one `loop()` and one step with the same inputs
  leave two states for which every line holds.

Both proven, every line holds after every `loop()` of every run, whatever
the inputs: the pins the sketch drives are the outputs of the machine, for
ever. That is the whole claim, and it is not a test: no run is sampled.

One refuted, the command fails and names the lines that do not hold after
it:

```
✗ Refuted correspondence "Debounce sketch": debounce.ino is state_machine "Debounced button"
    from two states that correspond, one loop() and one step can leave two that do not
    does not hold after it: digitalWrite(LED) is led
    does not hold after it: stable == 0 && counting == 1 is in Rising
    does not hold after it: stable == 1 && counting == 0 is in High
    does not hold after it: stable == 1 is led
    a case that shows it (the code's side and the model's side of each line):
      with button = 1
      `the level of LED` and `led`: before 0 and 0, after 0 and 1  <- differ
      `stable == 0 && counting == 1` and `in Rising`: before 1 and 1, after 1 and 0  <- differ
      `stable == 1 && counting == 0` and `in High`: before 0 and 0, after 0 and 1  <- differ
      `millis() - since` and `time in Rising`: before 40 and 40, after 50 and -
      ...
```

(the model here waits 40 ms where the sketch waits 50: at 40 ms the machine
is in `High` with the lamp lit while the sketch still counts. The case gives
the inputs of the step and, for each line, the value of the code's side and
of the model's side before and after; `-` is the time in a state that is not
active.)

A refuted step does not always mean the code is wrong. The step is asked
from any two states the lines allow, reachable or not: a correspondence that
says too little allows states no run reaches, and the proof fails from
them. The remedy is to say more (how a variable of the code relates to the
output, an `invariant`), never to trust the result less: a correspondence
is only ever proven when it holds from every state it allows.

A pin the code reads or writes that the correspondence does not name is
never passed over: the proof is refuted on the line "the code reads and
writes only the pins the correspondence names". A `loop()` whose `delay()`s
add up to another time than the period is refuted the same way. A variable
of the machine that no line speaks of does not fail the proof, since
nothing was claimed of it; it is named under the answer ("the proof says
nothing of this variable of the machine"), so that a proof is never read as
covering more than it does. A loop of the code the
bound does not cover, a function with no body, the time limit: neither
proven nor refuted, said as such.

What the proof takes as given, and prints with its answer:

- the first `loop()` runs as `millis()` starts counting (at 0);
- the machine steps at the `period` of the correspondence: a time of 500 ms
  with a `loop()` of 20 ms is 25 steps;
- one `loop()` is one step of `period`: `millis()` is constant within a
  `loop()` and advances by the period between two; the time the statements
  themselves take is not counted. A sketch whose `loop()` takes another
  time on the board is another machine;
- an input has one value in a step, read by the code and by the machine;
- the code is the C subset of the Arduino language: `pinMode`,
  `digitalRead`, `analogRead`, `digitalWrite`, `millis`, `delay`. Interrupts,
  the serial port, libraries and C++ classes are not read;
- the machine is the C the model generates, the same that runs in `sil`.

`--correspondence <name>` proves one of several, `--keep <dir>` keeps the
files of the proof (`proof.c`, the model's C, the code), `--json` gives the
outcomes to a tool. The declaration is exported to SysML v2 as a
`#Correspondence` part and read back as written.

## A model proposed by an AI, judged by the proof

Writing the machine and the correspondence of a sketch is work an AI does
well, and nothing it says is trusted:

```
$ arclang recover door.ino --propose --with "<command>"
Turn 1 of 4: asking the proposer
  not proven: START is refuted: after setup() and one loop(), ...
Turn 2 of 4: asking the proposer
✓ Proven (1.4 s) door.ino is state_machine "Door lock"
  Output: door.arc
```

Three roles, kept apart:

- the **sketch** is read as it is;
- a **proposer** proposes: `--with` names any command that reads a prompt
  on its input and prints its answer (the command line of an AI, local or
  not; `ARCLANG_PROPOSER` when `--with` is not given). The prompt holds the
  sketch, how a machine and a correspondence are written, and how the answer
  is judged;
- **ArcLang judges**: the answer is compiled and proven exactly as `arclang
  prove` does. Refuted, the lines that do not hold and the case that shows
  it go back to the proposer, which answers again, `--turns` times at most
  (4 by default).

The model is written (`-o`, or `<sketch>.arc`) only once its proof holds,
with the command that replays the proof with no AI in its first lines. No
answer proven in the turns given: the command fails and writes nothing.
`--keep <dir>` keeps each answer and the files of its proof.

ArcLang depends on no AI and calls none by itself: the sketch is sent to the
command you name, and nowhere else.

What a proof of a proposed model says, and what it does not:

- it says the sketch drives its pins as the machine drives its outputs, on
  every run, under the assumptions printed with it. A proposer cannot get a
  wrong model through: the proof is of the sketch that was given, whatever
  source the answer names, and every pin the sketch touches must be named;
- it does not say the model is the one an engineer would write. A proposer
  may count a time in a variable where a `timing:` would say it better, or
  keep the sketch's bookkeeping as variables of the machine. The model is
  right; it may still deserve a rewrite, which `arclang prove` then judges
  the same way.

Tried with an AI's command line on sketches written for it: a thermostat
and a blinker were proven at the first answer, a door lock (two inputs, two
outputs, a timer) at the second, after the first was refuted at the start.
Four sketches are not a measure; a sketch outside the C subset read here is
not proven by any number of turns.
