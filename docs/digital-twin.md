# Digital twins

A digital twin connects the physics of a model to real data. ArcLang runs it
in OpenModelica, the same way as a simulated test case: the measurements
drive the model (`drives:`), and the twin compares the model's answer to what
was measured (`compares:`). The requirements it names are checked on that
run (`expects:`).

```arc
twin "January cold day, standard unit" {
    id: "TW-JANUARY"
    verifies: ["REQ-COMFORT"]
    simulate: { model: "HeatingPhysics.StandardDay" configuration: "CFG-STANDARD" sources: ["physics/HeatingPhysics.mo"] }
    data: { file: "data/house-2026-01-14.csv" time: "timestamp" }
    drives: ["\"PN-OUTDOOR\".T <- outdoor_c °C"]
    compares: ["\"PN-SPACE\".room.T ~ indoor_c °C within 0.5 K for 99 %"]
    expects: ["REQ-COMFORT: min(\"PN-SPACE\".room.T) >= 20 °C"]
}
```

```bash
arclang twin examples/building/heat-pump/heat_pump_heating.arc          # each twin, its verdict
arclang twin model.arc --twin TW-JANUARY --json                          # one twin, with its curves
arclang twin model.arc --list
```

In VS Code, *ArcLang: Run the digital twins* runs the model's twins and shows
each one in a view:
- its verdict, its data and their span;
- for each comparison, the measurement and the model on one chart, with the
  tolerance band and the first instant the model goes beyond it;
- the requirements checked on the run.

## The declaration

| Key | What it is |
|---|---|
| `simulate:` | The Modelica class the data drives (`model`), its `configuration`, the hand-written `sources`, an integration `tolerance`. There is no `stop_time`: the run lasts as long as the data. |
| `data:` | `file`: the measurements, a CSV file in the model's folder; `time`: the column of the instants (default `time`), as numbers (in `time_unit`, seconds by default) or ISO 8601 date-times (`2026-01-14T08:00:00Z`); `delimiter` when it is neither `,` `;` nor a tab. A line starting with `#` is a comment. |
| `drives:` | `variable <- column [unit]`: the column, converted from its unit, is imposed on the variable, linearly interpolated between the samples. An ArcLang id names a part (`"PN-OUTDOOR".T`). |
| `compares:` | `variable ~ column [unit] within tolerance [unit] [for N %]`: at each measured instant, the deviation of the model from the measurement. The comparison holds when at least N % of the samples are within the tolerance (100 % without `for`; real sensors have outliers). A tolerance is a difference: `0.5 K` and `0.5 °C` are the same. |
| `expects:` | Expectations on the run, written as a simulated case's (`min`, `max`, `final`, `mean`, `at`), each naming the requirement it checks. |
| `verifies:` | The requirements the twin checks on real data. |

## How it runs

ArcLang writes a model that extends the class and binds each driven variable
to an input of its own (`extends StandardDay(outdoorAir.T = twin_in_0)`). It
gives OpenModelica the measurements as those inputs (`-csvInput`) and
simulates over the span of the data. Only a variable can be driven. If the
target is a parameter of the physics, the twin says so: declare it as a
variable (or an input) in the Modelica source. A variable that a formula
defines (the weather of a design day, say) is driven: the measurement
replaces the formula.

For each comparison the twin reports:
- the largest deviation and the RMS deviation, in the column's unit;
- the share of the samples within the tolerance;
- the first instant beyond it, in seconds from the start of the data.

## What comes next

- **Parquet** files.
- **Live data** in the same `data:` block: MQTT, OPC UA, Kafka and HTTP, with
  the model advanced as the data arrives (an FMU, `buildModelFMU`).
- **Online**: on arclang.fr, signed in, the twins run on the server, as the
  simulations do.

The example in `examples/building/heat-pump/` carries a twin and a sample of
data. **The sample is synthetic**: its header says how it was made. Replace
it with a logger's export that has the same columns.
