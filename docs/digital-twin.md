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

## Parquet

`data: { file: "data/house.parquet" time: "timestamp" }` reads a Parquet
file as pandas and pyarrow write it:
- compression Snappy, zstd or LZ4 (not gzip);
- the time column a timestamp (milli, micro or nanoseconds, the schema says
  which), a number in `time_unit`, or an ISO 8601 text;
- the other columns numbers or booleans (1 and 0), null where there is no
  value.

## Live data (MQTT)

```arc
twin "House 042, live" {
    verifies: ["REQ-COMFORT"]
    simulate: { model: "HeatingPhysics.StandardDay" configuration: "CFG-STANDARD" sources: ["physics/HeatingPhysics.mo"] }
    data: { mqtt: "mqtt://broker.local:1883" topics: ["house/042/#"] window: 24 h refresh: 1 min record: "data/live.csv" }
    drives: ["\"PN-OUTDOOR\".T <- outdoor_c °C"]
    compares: ["\"PN-SPACE\".room.T ~ indoor_c °C within 0.5 K for 99 %"]
}
```

```bash
arclang twin model.arc --live                  # follows the broker until stopped
arclang twin model.arc --live --json --for 600 # one JSON line per run, for ten minutes
arclang twin model.arc                         # replays what was recorded (record:)
```

`--live` subscribes to the topics. Every `refresh` (one minute by default)
it runs the twin over the latest `window` of data (one hour by default),
then prints the verdict, or a JSON line with `--json`. The window is
counted in the data's own time, so a fast replay of past data is a live
run too. In VS Code, *ArcLang: Watch a digital twin live* follows the twin
in its view, with a banner saying how the connection goes and a Stop
button.

**Messages.** Either a number, or a JSON object:
- A number gives one sample. Its column is the topic's last level:
  `house/042/indoor_c` gives `indoor_c`.
- A JSON object gives one column per numeric or boolean field, for example
  `{"time": "2026-01-14T08:10:00Z", "outdoor_c": -9.5, "indoor_c": 21.0}`.
  Its `time` is an ISO 8601 date-time or seconds (or milliseconds) since
  1970. Without a `time`, the sample is dated by its arrival.

Only the columns the twin reads (those of `drives:` and `compares:`) are kept.

**The broker.** `mqtt://host[:port]` (1883 by default), or `mqtts://` for TLS
(8883). The credentials are never written in the model: set
`ARCLANG_MQTT_USERNAME` and `ARCLANG_MQTT_PASSWORD`. `client_id` names the
client. When the broker cannot be reached, the twin says so and tries again
every two seconds.

**Record.** With `record:`, every sample received is appended to a CSV:
the time, then the columns. A later `arclang twin`, without `--live`,
replays it like any data file.

**What live means here.** The twin runs the model again over the window at
every refresh. It does not yet advance the model message by message: the
verdict comes at most one `refresh` after the data. That is right for
watching a building, a data center or a fleet. It is not right for
closing a control loop.

## What comes next

- OPC UA, Kafka and HTTP sources, in the same `data:` block.
- The model advanced as the data arrives, with an FMU (`buildModelFMU`).
- Twins online: on arclang.fr, signed in, they run on the server, as the
  simulations do.

The example in `examples/building/heat-pump/` carries a twin and a sample of
data. **The sample is synthetic**: its header says how it was made. Replace
it with a logger's export that has the same columns.
