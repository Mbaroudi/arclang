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

### The same question of the VHDL

With `--hdl <dir>`, `--prove` asks it of the programmable logic too: each
entity whose VHDL differs from the model's (all of them when the package
they share differs) is compared as built against as generated.

```
$ arclang drift speed_monitor.arc c/ --hdl vhdl/ --prove
    ✗ vhdl/speed_monitor_wheel_speed_filter.vhd:109: 1 line(s) of the file as built where the model generates 1
✓ Proven equivalent (6.8 s) entity speed_monitor_wheel_speed_filter: from reset and for any inputs, the logic as built and the logic the model generates agree on 1403 signals, their outputs among them, at every clock
```

GHDL synthesizes each side to a netlist, and the answer is one of three:

- **proven equivalent**, by an induction on the clock in two parts. The
  step, by Yosys: every signal the two netlists share by name is a point
  where they must agree, and points equal for two clocks are proven to stay
  equal. The base, by SymbiYosys with Bitwuzla: from reset, with any inputs,
  those same points are equal for the first clocks. A VHDL file that
  differs and whose entity is proven is no longer a drift, and the command
  succeeds;
- **not equivalent**: from reset and with the same inputs, an output of the
  two differs. The clock at which it does and the outputs, with both
  values, are printed, and the command fails:

  ```
  ✗ Not equivalent (4.7 s) entity battery_charger_charger: 3936 clock(s) after reset, on a run of random inputs simulated on both, the logic as built does not give what the model's gives:
      o_st_charging_bulk = 0b0 as built, 0b1 for the model
      o_st_charging_absorption = 0b1 as built, 0b0 for the model
  ```

  The run is looked for twice. Before any proof, GHDL simulates the two
  side by side from reset on 200,000 clocks of random inputs (always the
  same run: its seed is fixed) and stops at the first clock an output
  differs; that reaches a change far from reset. When the proof then
  fails, SymbiYosys searches the first 400 clocks exhaustively. A run that
  shows a difference is a refutation whoever found it; a run that shows
  none proves nothing, and the proof is still asked.

- **neither proven nor refuted**, said as such and never as a proof: the
  time limit, a VHDL that GHDL does not accept, an entity with other ports
  (a logic with another interface is not compared), an entity with no
  `rst` to start from.

What the proof is of, exactly, and where it stops:

- the netlists GHDL synthesizes, from reset, for any inputs at any clock;
  the bus wrapper of a block is an entity like another;
- the points are the signals with the same name in both. GHDL gives a
  number to the wires it creates, and two netlists may give the same number
  to wires that are not the same: such a point, when it is not proven, is
  no claim of anyone's and is dropped, and the whole proof is run again on
  the points left, until every one of them is proven. Their count is
  printed. A signal with a name of the design is never dropped: one that is
  not proven ends the proof;
- a VHDL rewritten so far that its registers no longer carry the numbers of
  the model's is not proven by this route; it is then searched for a
  difference, and reported as neither when none is found;
- a change that neither the random run reaches nor the search of the first
  400 clocks finds is **not** refuted: the answer is "neither", and the
  cases of the model run on the logic as built (`arclang hdl`) are the
  other way to find it. Random inputs reach what a few conditions lead to;
  a state behind a precise sequence of events may never be reached by them.

Measured, on a laptop: the speed monitor (a filter with a 32-bit product,
1,400 points) is proven in 7 s after two terms of a sum are exchanged. The
battery charger (regions, a PID, sixteen events a step; 12,000 points) is
proven in a little over two minutes after two guards are rewritten. The
same charger is refuted in 5 s once the target of one transition is
changed, a difference that shows 3,936 clocks after reset and that the
exhaustive search alone did not find in 400 s.

The tools are GHDL, Yosys and SymbiYosys with Bitwuzla, in the same formal
image; `ARCLANG_GHDL`, `ARCLANG_YOSYS` and `ARCLANG_SBY` name them when they
are elsewhere.

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
  the value of the variable. `analogWrite(PIN) is duty` relates the value
  written (0 to 255) to an Integer variable of the machine.
- `state`: an expression of the code on the left, what it is in the model on
  the right: `in <State>` (the expression holds exactly when the state is
  active), `time in <State>` (the expression, in milliseconds, is the time
  the state has been held, while it is active), or a variable of the machine.
- `events`: an edge of a pin taken by an interrupt, and the event of the
  machine each such edge is: `"RISING on SENSOR is pulse"` (or `FALLING`,
  `CHANGE`). See "Interrupts" below.
  A time read with `micros()` is related in milliseconds:
  `(micros() - since) / 1000 is time in Rising`.
- `invariant`: what always holds of the code's own variables. It is proven
  with the rest, not taken on trust.
- `int`: the bits of an `int` on the board, `16` on an AVR board (the Uno,
  the Nano, the Mega) or `32` (the default). A product that fits in 32 bits
  and overflows in 16 is another behaviour, and another proof. A `long`
  has 32 bits on both, so `millis()` counts on 32 bits as on the board. The
  proof is compiled for these widths with fixed-width types of its own
  (`uint32_t` and the others), not with those of the computer that runs it.

The compiler checks what a correspondence names when it compiles the model
(`arclang check`): the machine, the states and the variables are those of
the model, a read is an input, every line says what it is in the model.
Whether the lines hold is the proof's to say.

```
$ arclang prove debounce.arc
✓ Proven (1.5 s) correspondence "Debounce sketch": debounce.ino is state_machine "Debounced button"
    for one loop() a step of 10 ms, an int of 32 bits and a long of 32, the pins the correspondence names, and the C subset of the Arduino language
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
  `digitalRead`, `analogRead`, `digitalWrite`, `analogWrite`, `millis`,
  `micros`, `delay`, `delayMicroseconds`, `min`, `max`, `abs`, `constrain`,
  `map`. A pin the code writes reads back as it was last written. What the
  code prints on `Serial` is not looked at (it is no part of the claim);
  what it would read from it is an input nobody named, and fails the proof
  as a pin would. Interrupts are read as events (below). Libraries and C++
  classes are not read;
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

One sketch went all the way to a board. `uno_lamp.ino` (a lamp toggled by a
button, a press counting once held 50 ms) had its model proposed and proven
at the first answer for an `int` of 16 bits, was compiled with avr-gcc and
flashed on an Arduino UNO R3 (ATmega328P). Tried by hand, a wire for the
button: the lamp is dark at start, changes once at each press, holds while
the wire stays, and alternates over repeated presses, as the model says.
That is what a hand can try, not a measure.

It was then measured. A second board (the ESP32 of a UDOO KEY, running
MicroPython) pressed the button and timed the lamp, through a resistor
divider: 300 clean presses, the time from the press to the lamp's change
read to a fraction of a millisecond. The model says 50 to 60 ms (five steps
of 10 ms after the first one that sees the press). The board answered in
50.2 to 68.3 ms: 281 times within the model's window (allowing half a
millisecond for the instrument), and 19 times (6 %) later, by up to one
loop.

The board does not do exactly what the proof assumes. The proof takes
`millis()` to advance by the period at each `loop()`. On an AVR, `delay(10)`
does wait 10 ms, but `millis()` counts ticks of 1.024 ms: over five loops it
can advance by 49, 50 or 51. When it has advanced by 49, `millis() - since
>= 50` is false at the fifth loop and the lamp changes at the sixth. That
explanation was then tested: the same sketch timing with `micros()` instead
of `millis()` (no tick of 1.024 ms) was flashed and measured the same way,
and its 300 answers all fell between 50.2 and 60.2 ms, none late.

`arclang prove` now asks that question itself. For a board whose clock is
known not to be the one the proof states (`int: 16`, an AVR), the step is
proven a second time on the clock of the board: a time read can be one
millisecond short, so a duration can be one millisecond off either way.

```
✓ Proven (18.1 s) correspondence "Uno lamp sketch": uno_lamp.ino is state_machine "Lamp toggled by a button"
    on the clock of the board (an AVR's millis() counts ticks of 1.024 ms: a time read can be 1 ms short): NOT proven. ...
      a case that shows it (the code's side and the model's side of each line):
        `millis() - since` and `time in Releasing`: before 49 and 50, after 60 and -
      a duration compared with an exact number of loops is on the edge: the code can act one loop later than the machine
```

That case is the one the bench measured: the sketch reads 49 where the
machine has 50, and waits one loop more. The first line stays what it was,
a proof for the clock it states; the second says it does not carry to the
board. A sketch that reads no clock is "proven too". So is this one once its
comparison is moved off the edge: with `millis() - since >= 45` and the same
model, both proofs hold, and the board agreed when asked: that variant was
flashed, and its 300 answers fell between 50.2 and 60.1 ms, none late.

A sketch that times with `micros()` is proven on the clock of the board too:
that clock's tick is 4 µs, and its reads are never a millisecond short. The
board agreed again: the lamp timed with `micros()`, observed over 400 changes
of its button, made its 103 changes at the model's step, none a loop late.

The clock of a board that is not an AVR is not asked about: it has not been
measured here.

## Interrupts

A sketch that reads a pin once a loop misses a pulse shorter than its loop.
One that takes it by an interrupt does not: a function of the code runs at
the edge itself.

```c
volatile int pulses = 0;
void count() { pulses++; }
void setup() { ... attachInterrupt(digitalPinToInterrupt(SENSOR), count, RISING); }
void loop() {
  noInterrupts(); int seen = pulses; pulses = 0; interrupts();
  if (seen % 2 == 1) { lit = !lit; digitalWrite(LAMP, lit); }
  delay(10);
}
```

On the model's side an edge is an **event**, the trigger of a transition
(`transition Dark -> Lit { trigger: "pulse" ... }`), and the correspondence
says which: `events: ["RISING on SENSOR is pulse"]`. In the proof, the
events of a step call the routine the code attached to that pin, once each
and in the order they came, before `loop()` runs; the machine is given the
same events. The routine is the one read in the code's `attachInterrupt`,
and a line of the correspondence holds only while it is attached to that
pin on that edge.

The proof takes an interrupt **between two `loop()`s**. A real one falls
anywhere, so a second question is asked for each routine: does an interrupt
taken in the middle of `loop()` leave what one taken just before the
`loop()`, or just after it, would leave? Where it does, the interrupt in the
middle is one of the two the proof covers. The places it may fall are the
uses, outside the routine and with interrupts on, of what the routine
changes; the code is run three times from the same state (the routine
before, after, and at one of those uses) and the three states are compared,
from any two states that correspond and after none, one or two interrupts
already taken.

```
✓ Proven (2.7 s) correspondence "Flag lamp sketch": flag_lamp.ino is state_machine "Lamp"
    interrupts: taken between two loop()s, or in the middle of one: proven that an interrupt at any use of what its routine changes leaves what one taken just before the loop(), or just after it, leaves (one interrupt in the loop(), after none, one or two since the last)
      `pulsed`, which the routine `mark` changes, is used at line 20, 21 of flag_lamp.ino with interrupts on
```

That sketch raises a flag in its routine and lowers it in `loop()` with
interrupts on: two lines use it, and it is safe. The sketch that counts its
pulses, written without `noInterrupts()` / `interrupts()`, is not:

```
    interrupts: taken between two loop()s. In the middle of one, NOT the same: an interrupt of `count` taken at one of these uses can leave what neither an interrupt just before the loop() nor one just after it leaves (something is lost, or counted twice):
      `pulses`, which the routine `count` changes, is used at line 20, 21 of pulse_lamp.ino with interrupts on
```

Those two lines are the read of the count and its reset: a pulse taken
between them is lost. With the protected section the count is only used
with interrupts off, and there is nothing to ask.

Where this stops: an interrupt falls between two *uses* of a shared variable,
not between two instructions of the processor (`count++` in `loop()` is one
use, where an AVR takes three instructions); one interrupt in a `loop()` is
followed, not several; the places are found in the text of the code (the
routine's body, what it assigns), and a state that holds an array or a
structure is not compared: the answer is then "not asked", with why.

Measured on the board: this sketch on an UNO, 200 pulses sent to its
interrupt pin by the instrument, 19 of them shorter than its loop (the
shortest 5 ms), and 200 changes of its lamp, each at the model's step.

## A binary, without its sources

A sketch may be lost, or never have been given: what is left is what was
flashed. `arclang prove` reads that too. A correspondence names a `binary`
(an Intel HEX file) in place of a `source`, and the proof is of the machine
code itself:

```
correspondence "Uno lamp binary" {
    machine: "Lamp toggled by a button"
    binary: "uno_lamp.hex"
    period: 10 ms
    functions: [
        "loop at 0x0102",
        "pinMode at 0x043e",
        "digitalRead at 0x0514",
        "digitalWrite at 0x04b6",
        "millis at 0x029c",
        "delay at 0x02fe"
    ]
    reads: ["digitalRead(2) is button"]
    writes: ["digitalWrite(13) is led"]
    state: [
        "word(0x0106) == 0 && word(0x0104) == 0 is in Up",
        "word(0x0106) == 0 && word(0x0104) == 1 is in Arming",
        "word(0x0106) == 1 && word(0x0104) == 0 is in Down",
        "word(0x0106) == 1 && word(0x0104) == 1 is in Releasing",
        "millis() - dword(0x0100) is time in Arming",
        "millis() - dword(0x0100) is time in Releasing",
        "word(0x0108) == 1 is led"
    ]
    invariant: ["word(0x0106) == 0 || word(0x0106) == 1", ...]
}
```

A binary has no names. A function is where it is (`loop at 0x0102`, the
address a disassembler prints), and a variable is the bytes of the memory
that hold it: `byte(at)`, `word(at)` for two bytes (an `int`), `dword(at)`
for four (a `long`), low byte first. A pin is its number.

```
$ arclang prove uno_lamp_binary.arc
✓ Proven (206.9 s) correspondence "Uno lamp binary": uno_lamp.hex is state_machine "Lamp toggled by a button"
    of the machine code of uno_lamp.hex itself, translated one instruction at a time (ATmega328P, Ghidra's description of the AVR): no source is read
    proven of the machine code of the functions it does not run, from any memory and registers, on the pins of an Arduino UNO:
      pinMode at 0x043e makes the pin an input, an input pulled up or an output, for each of the pins 0 to 19, and changes nothing else
      digitalRead at 0x0514 returns the level of the pin, for each of the pins 0 to 19, and changes nothing but the PWM of the pin, turned off
      digitalWrite at 0x04b6 sets the level of the pin and of no other, for each of the pins 0 to 19, and changes nothing else but the PWM of the pin, turned off
      millis at 0x029c returns the four bytes of the memory at 0x010b and changes nothing
    taken on trust: these are the Arduino functions they are said to be, and are not run (delay at 0x02fe)
    proven of the clock of the binary:
      the interrupt of timer 0 (at 0x0208) adds 1.024 to the four bytes millis returns at each overflow of the timer, and changes nothing else but its own count of turns
      the start leaves timer 0 overflowing every 256 x 64 cycles of the clock (1.024 ms at 16 MHz), its overflow an interrupt, and interrupts on
      it is the only interrupt of the binary, and the code of loop() reads none of its bytes: an interrupt changes nothing loop() reads
    taken on trust: the clock of the board is 16 MHz, of which timer 0 counts the cycles
    on the clock of the board (an AVR's millis() counts ticks of 1.024 ms: a time read can be 1 ms short): NOT proven. ...
```

**How the machine code is read.** Nothing is decompiled: a decompiler
gives C a person can read, and loses things on the way (the argument of a
call, a comparison on four bytes). Ghidra is asked something else: the
meaning its description of the processor gives each instruction, as a few
elementary operations. Each is written as one line of C: a register is a
cell of an array, the memory an array of bytes, a jump a `goto`, a call a
call, the flags of an addition the flags of that addition. That C is long
and unreadable, and it is what the chip does. It takes the place of the
sketch in the proof of the section above, unchanged otherwise:

- **the start** runs the code from its reset (its own initialisation of
  the memory, the Arduino core's `init`, the sketch's `setup`) to its first
  call of `loop`;
- **the step** runs `loop` once from any memory and any registers that
  correspond to a state of the machine.

**What stands in.** The Arduino functions the correspondence names are not
run: they are the stand-ins of the proof, called as the compiler of the
board calls them (the first argument in r24, the result in r24, or in r22
to r25 for a `long`), and what such a function may change beside its
result is anything after it. That the function at 0x0514 is `digitalRead`
is what the correspondence says, and it is not taken on trust: before the
proof of the correspondence, each named function is proven, of its own
machine code and of what it calls, to be the function it is said to be on
an Arduino UNO (pins 0 to 7 on port D, 8 to 13 on port B, 14 to 19 on port
C). From any memory and any registers, for every pin:

- `digitalRead(pin)` returns the bit of the pin in the input register of
  its port;
- `digitalWrite(pin, level)` sets the bit of the pin in the output register
  of its port to the level, and no other bit;
- `pinMode(pin, mode)` makes the pin an input, an input pulled up or an
  output, and touches no other pin;
- `millis()` returns the four bytes of the memory it reads;

and each runs only instructions the translation covers, returns with the
stack and the registers a function must keep as they were, changes no byte
of the memory and no other register of the chip (the PWM of the pin apart,
which a digital read or write turns off), and leaves interrupts as they
were. A function that is not the one it is said to be refutes the proof,
with what it does not do: `digitalRead` and `digitalWrite` taken for each
other, or a binary whose table of pins puts pin 13 on another port.

**The clock.** `millis` returns four bytes of the memory; that they count
the time is proven too, of the binary's own interrupt and of its own start:

- the routine of the overflow of timer 0 (found by its interrupt vector),
  run from any memory and registers, adds 1 to those four bytes, and 1 more
  every time a fraction it adds 3 to passes 125 (1.024 a turn, as the
  Arduino core counts), counts its turns, changes no other byte of the
  memory and no register of the chip, and returns with the stack, every
  register and every flag as they were;
- the start, run from the reset of the chip to the first `loop`, leaves
  timer 0 counting one in 64 cycles of the clock and overflowing every 256
  counts (1.024 ms at 16 MHz), its overflow an interrupt, and interrupts
  on;
- when that routine is the only interrupt of the binary (every other vector
  goes where the vectors nobody takes go) and the code of `loop` reads none
  of its bytes by address and uses no pointer: an interrupt changes nothing
  `loop` reads.

A binary whose routine counts otherwise, or whose start sets the timer
otherwise, refutes the proof. What is left on trust is said under it:
`delay`, `micros` and the analog functions, and the 16 MHz of the board's
clock, which is not in the binary.

A function that is not named is run, instruction by instruction: leave
`pinMode` out and its code writes the direction of the port itself, which
the proof refuses (*the machine code writes no port of the chip itself*),
since a pin written that way would be a write nobody compared.

**What the proof holds besides the lines of the correspondence**, each a
line of its own that can fail: the code runs only instructions the
translation covers (a computed jump is not one); it stays within the memory
and the flash of the chip; it reaches `loop` from its reset; between two
`loop()`s, r1 is zero (the compiler's convention) and the stack is where
it was.

**What is tried against it** (the tests): the binary with one instruction
changed (`sbiw r24, 0x32`, the 50 ms of the press, made 51) is refuted; so
are a correspondence that reads the wrong bytes of the memory, and one
that takes `digitalRead` and `digitalWrite` for each other.

**What it does not do yet.** The chip is an ATmega328P (an Arduino UNO),
and the image a program that starts at address 0. An interrupt of the
binary other than the one that keeps the time is not read. A binary built with the Arduino IDE's link-time
optimisation has `setup` and `loop` folded into `main`, and its pin
functions specialised: there is then no `loop` to name.
The binary of the example was read back from the board itself, through its
bootloader, and is byte for byte the one its sketch compiles to.

### Where the functions are: `arclang recover firmware.hex`

The addresses need not be read in a disassembly. `arclang recover` proposes
them, with the pins, the period and the memory the code keeps its state in:

```
$ arclang recover uno_lamp.hex
// setup() calls: pinMode(2, 2), pinMode(13, 1), digitalWrite(13, 0)
// loop() calls:  digitalRead(2), millis(), millis(), digitalWrite(13, ?), delay(10)
// loop() keeps its state in the memory at: 0x0100 to 0x0109 (10 bytes)

correspondence "uno_lamp binary" {
    machine: "..."
    binary: "uno_lamp.hex"
    period: 10 ms
    functions: [
        "loop at 0x0102",  // main (0x01ea) calls it in its cycle
        "millis at 0x029c",  // it copies the four bytes at 0x010b with interrupts masked, and touches nothing else
        "delay at 0x02fe",  // it calls micros (0x02b4) in a cycle and keeps nothing in the memory
        "pinMode at 0x043e",  // it reads the pin tables of the flash and writes the direction and the level of one port ...
        "digitalWrite at 0x04b6",  // it reads the pin tables of the flash, turns the PWM of the pin off and writes one port ...
        "digitalRead at 0x0514"  // it reads the pin tables of the flash, turns the PWM of the pin off, reads one port ...
    ]
    reads: ["digitalRead(2) is ..."]
    writes: ["digitalWrite(13) is ..."]
    state: ["... is in ..."]
}
```

Each function is found by the shape of its machine code, and the reason is
written beside it. `main` is the first function, from the reset, that calls
in a cycle and calls nothing after it; `loop` is what it calls there,
`setup` what it calls just before. An Arduino function is known by what
only it does on this chip: `millis` copies four bytes of the memory with
interrupts masked and touches nothing else, `micros` also reads the counter
of timer 0, `digitalRead` reads the pin tables of the flash and one port
through a pointer and writes nothing. A pin is the constant the caller
loads in the register of the first argument just before the call, and the
period the constant `loop` passes to its one `delay`.

What cannot be told is said and not guessed: two functions with the same
shape name neither; a pin that is not a constant is a `?`; a binary whose
`main` has no call in a cycle (link-time optimisation) gives no `loop`.

It is a proposal. The machine is left to write, by a person or an AI, from
those calls and from the bytes of the state; `arclang prove` then decides.
A shape is a reason to name a function, not a proof that it is that
function: the proof is `arclang prove`'s, for the functions of the pins and
`millis`; the others it takes on trust and says so.

### The machine, proposed by an AI: `arclang recover firmware.hex --propose`

The loop of the section *A model proposed by an AI, judged by the proof*
takes a binary as it takes a sketch:

```
$ arclang recover firmware.hex --propose --with "<command>" -o firmware.arc
```

The proposer is given what `arclang recover` found (the correspondence
started, with its functions, its pins and its period) and the machine code
of `setup` and of `loop` as a listing, one line an instruction, the calls
of the named functions said. It is given no source: there is none. It
answers with the machine and the completed correspondence; the answer is
compiled and proven against the machine code; what is refuted goes back to
it, with the case. A model is written only once its proof holds, and
`arclang prove` replays that proof with no AI.

One thing a proposal cannot do: name functions. A function a correspondence
names is not run in the proof, so a proposal that called a piece of the
code `delay` would put it out of the proof. The functions are those found
in the machine code, where they were found; a proposal that adds, moves or
renames one is refused before any proof.

**The tools.** Ghidra (`analyzeHeadless`, or the command `ARCLANG_GHIDRA`
names) beside CBMC and Bitwuzla. `tools/embedded/reverse` builds an image
that holds the three.

## A board observed against the model

The bench that found this is a command. An **instrument** drives the inputs
of the board and times its outputs: a second board running MicroPython, its
console on a serial port, wired to the pins of the first.

```
$ arclang observe uno_lamp.arc --instrument /dev/cu.usbserial-2140 \
      --drive button=13:1 --watch led=35 --changes 400
Playing 400 change(s) of button on the board through /dev/cu.usbserial-2140 (about 34 s)
Observation of correspondence "Uno lamp sketch": the board against state_machine "Lamp toggled by a button"
  played: 400 change(s) of the inputs over 31.6 s; the board changed its outputs 99 time(s)
  the board's loop takes 10.024 ms (the model steps every 10 ms); its changes are within 0.11 ms of that loop
  2 input change(s) fell too close to a read of the board for the instrument to say on which side: taken on the side the board showed
✓ Agrees on this run: the same 99 change(s) in the same order; 90 at the model's step, 9 one loop later
    a measure of what was played, not a proof: `arclang prove` is the proof
```

- `--drive NAME=PIN[:1]`: an input of the machine (one the correspondence
  reads from a pin), or an event it takes from a pin by an interrupt (each
  edge is then given to the model in the step that follows it), and the
  instrument's pin wired to it; `:1` when the pin
  rests high. `--watch NAME=PIN`: what the correspondence writes to a pin (a
  variable, or a state), and the instrument's pin that reads it.
- `--watch NAME=PIN:duty`: a value the correspondence writes with
  `analogWrite`. The instrument does not follow the level of that pin, it
  measures it: the share of each of its periods the pin is high, 0 to 255
  (a pin that stays put is 0 or 255). One output at least is followed by its
  level: the loop of the board is found from those.
- The instrument plays `--changes` changes of the inputs, held for random
  times, some too short to count; `--seed` plays the same run again.
- `--keep FILE` keeps the recording, `--recorded FILE` compares one kept
  earlier, with no board.

How the comparison is made:

- The board's loop is not seen, but its outputs only change when it runs:
  the moments of the changes lie on a regular lattice. Its period is the
  **real period of the loop** and its phase is where the loop stands against
  the instrument's clock. Both are measured from the changes alone, with no
  help from the model, and how far the changes are from that lattice is
  printed (here 0.11 ms: the loop is regular).
- The inputs played are replayed on the model, by the executor of the model
  (the one `arclang verify` runs), read at the moments the board's loop ran.
- The outputs are compared change by change: the same change at the model's
  step, or one loop later (counted apart: on an AVR that is its clock, see
  above), or a disagreement, with the first one and its time.
- An input change within 0.6 ms of a moment the board reads cannot be placed
  by the instrument; it is taken on the side the board showed, and counted.
- A measured value is compared in value (within 20 of 255: what timing one
  period of the pin by polling can be off by) and in step (the loop just
  before the change was seen, or the one beside: a value shows up to 7.5 ms
  after the loop that made it). A model that differs by less than that
  tolerance is not told apart: 80 for 85 would pass.

What it says and does not:

- "Agrees on this run" is an agreement on what was played. Another run may
  disagree; it is a measure, and the proof is `arclang prove`. Its worth is
  in what it measures that a proof assumes: the period of the loop (10.024 ms
  where the model says 10), the clock, the compiler's translation.
- It catches a model the board does not follow: against the same recording, a
  model that waits 100 ms instead of 50, or whose lamp only turns on,
  disagrees from the first change.
- The inputs and outputs are digital pins. The model starts from its state at
  rest: a board that was not at rest when the run began is compared by its
  changes, not by its levels.

The dimmer of the tests (`dimmer.ino`: two buttons, a heartbeat that
changes at every loop, a lamp written with `analogWrite`) was observed so:
two inputs driven, the heartbeat followed, the lamp measured.

```
$ arclang observe dimmer.arc --instrument /dev/cu.usbserial-2140 \
      --drive up=13:0 --drive off=26:0 --watch beat=35 --watch duty=14:duty --changes 150
  the board's loop takes 20.053 ms (the model steps every 20 ms); its changes are within 0.48 ms of that loop
  measured values: 27 change(s), each the model's value at the model's step (or the one beside: a value is seen up to 7.5 ms after the loop that made it)
✓ Agrees on this run: the same 645 change(s) in the same order; 645 at the model's step, 0 one loop later
```

The wiring is the instrument's affair and a board's pins are not all alike:
an instrument at 3.3 V reading a board at 5 V needs a divider (here 10 kΩ
over 20 kΩ), and checking the wiring by software before the two boards are
joined (a loopback on the instrument alone, then the divider measured) is
what kept this one safe. The sketch toggles on the same presses as
the model, in the same order; it is the *moment* that can be one loop late. A proof by `arclang prove` is a proof
for the clock it states, and this is the measure of how far a real one is
from it: a sketch that compares `millis()` with an exact multiple of its
period sits on that edge.

## A board, from its device tree

The code is one half of a board; the other is its hardware. On a board that
runs Linux (a Raspberry Pi, a Zynq, most ARM and RISC-V boards) and on
those Zephyr builds for, the hardware is described by a **device tree**:
what the kernel itself reads to know its processors, its memory, each
device, where it is mapped, the bus it sits on and the pins that wire it.
`arclang recover` reads it:

```
# on the board
$ dtc -I fs -O dts /proc/device-tree > board.dts
# or from the blob the board boots with
$ dtc -I dtb -O dts board.dtb > board.dts

$ arclang recover board.dts -o board.arc
Recovered board "Zynq Zed Development Board" from board.dts: 35 device(s), 35 link(s), read in the tree
  note: 11 node(s) the tree marks as not enabled (`status`) are not written, nor what they hold
```

What is written is a physical architecture:

- the **board**: what it is compatible with, its processors (`1 x
  arm,cortex-a9`), its memory (the sum of the ranges the tree gives);
- a **node** for each enabled device that says what it is compatible with,
  named by its label when it has one, with its path in the tree;
- a **link** for the bus each device sits on, with where: `I2C, address
  0x76`, `SPI, chip select 0`, `memory-mapped, 0xe000a000`;
- a **link** for each pin of a GPIO controller that wires a device, with
  what the tree calls it: `GPIO 17 (reset)`.

Every line is read in the tree, and the level is that one: **read**, not
guessed and not proven. There is nothing to prove: the tree is the
description the board's own kernel trusts, and the model is checked to
compile before it is written. What the tree does not hold is not recovered,
and is said with the result:

- a device the tree marks as not enabled, and a node that does not say what
  it is compatible with, are not written (their count is given);
- interrupt lines, clocks and power supplies are in the tree and are not
  written as links yet;
- the schematic, what is plugged on a connector the tree does not describe
  (a sensor on a Raspberry Pi's header that no overlay declares), and what
  the software does with the devices are not in a device tree at all.

The tree must be flattened, as `dtc` writes it: a source with `#include`,
macros, deletions or an overlay (`&label { ... }`) is refused, with the
command that flattens it.

Tried on two real trees, decompiled from the blobs Zephyr ships for its
emulated boards: a ZedBoard (Zynq-7000: 35 devices, 35 links) and a ZCU102
(Zynq UltraScale+: 348 devices, 585 links, 237 of them pins). Both models
compile. No board was plugged in for these runs: reading `/proc/device-tree`
on a board is the command above, run by whoever has the board.

A board without a device tree (an Arduino Uno, a bare microcontroller) has
no such description to read: its hardware is in its schematic.
