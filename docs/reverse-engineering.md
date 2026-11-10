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
