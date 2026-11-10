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
