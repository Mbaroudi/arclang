# Recovering a model from C written by hand: design

Date: 2026-11-10. Decided with Malek in a design session; this is what was agreed.

## Goal

Take C that the compiler did not generate (first target: an Arduino sketch)
and give back an ArcLang model of it that is **proven**, not guessed.

## Decisions

- **ArcLang stays neutral.** It depends on no reverse-engineering engine and
  on no AI in particular. Engines (Ghidra through its MCP bridge, Craft
  Anything, radare2...) come later, for binaries, through MCP; none of them
  proves anything, and that is the part ArcLang brings.
- **First target: source written by hand**, because it is the only one where
  the level "proven" can be reached now.
- **Three roles, kept apart**: something reads (here, the sources as they
  are), an AI proposes (a model and a correspondence), ArcLang judges (a
  proof, or a counterexample that goes back to the AI).
- **The level is always shown**: proven, refuted with where, or neither.

## The correspondence

A sketch has its own structures; the model has the compiler's. A declaration
of the model, `correspondence`, says how they relate:

- inputs: a read of the sketch (`digitalRead(2)`) is an input of the model;
- outputs: a write of the sketch (`digitalWrite(13, ...)`) is a variable of
  the model, or a state being active;
- state: an expression over the globals of the sketch is a variable of the
  model, a state being active, or the time spent in a state.

It is written in the `.arc` file, checked by the compiler, kept under
version control, and replayed without any AI.

## The proof

Not "the two are identical" but an inductive correspondence:

1. after `setup()` and one `loop()`, and after `init` and one step, the two
   states correspond;
2. from any two states that correspond, with the same inputs, one `loop()`
   and one step leave two states that correspond.

Outputs are part of the state compared (the level of each pin). The Arduino
functions are replaced by stand-ins: a read gives the input of the step, a
write records the level of the pin, `millis()` advances by one period a turn.
CBMC translates, Bitwuzla decides (as `arclang drift --prove`).

A wrong correspondence cannot give a wrong proof: if it does not hold, step
1 or 2 fails. A read or a write of a pin the correspondence does not name
fails the proof.

Starting from states that correspond, instead of any state, also removes
the false "not equivalent" that states no run reaches gave.

### Stated with every result

- one `loop()` is one step of the model, of the period the correspondence gives;
- the sketch is in the C subset of the Arduino language (no class, no
  library beyond the stand-ins); anything else is refused with its line.

## Components

| Component | Role |
|---|---|
| `correspondence` declaration | the table; names and types checked at compile time |
| Arduino stand-ins | a C header for `digitalRead`, `digitalWrite`, `analogRead`, `millis`, `delay`, `pinMode`; a function it does not have fails with its name |
| harness generator | extends that of `drift --prove`: the assumptions and the checks, from the table |
| `arclang prove <model> <sources>` | replays the proof, no AI: the reference, and what CI runs |
| `arclang recover <sources> --propose` | the loop: AI, proof, counterexample, correction; a bounded number of turns |

## Tests

Three sketches written for it: a blinker with a button, a thermostat with
hysteresis, a debounce with a delay. For each: a right model and table are
proven; a wrong model is refuted; a wrong table is refuted. They run in the
formal image.

## Lots

1. `correspondence` and `arclang prove`, models and tables written by hand.
2. `recover --propose`: the loop with the AI.
3. Then: VHDL equivalence (Yosys), a neutral engine through MCP for
   binaries, boards plugged in (device tree first, then learning a machine
   from the pins, announced as "agrees on N runs").

## Not known yet

The time a proof takes on a real sketch has not been measured. On generated
code it grows with the events one step can take (the power window gets no
answer in 300 s).
