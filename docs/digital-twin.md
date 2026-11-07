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
replays it like any data file. Stopping the run (Ctrl-C, SIGTERM, a line
`stop` on its standard input, or Stop in VS Code) writes what was received
since the last record before it ends.

## Live data (OPC UA)

```arc
twin "House 042, from the controller" {
    verifies: ["REQ-COMFORT"]
    simulate: { model: "HeatingPhysics.StandardDay" configuration: "CFG-STANDARD" sources: ["physics/HeatingPhysics.mo"] }
    data: {
        opcua: "opc.tcp://plc.local:4840"
        nodes: { outdoor_c: "ns=2;s=House.Outdoor.T" indoor_c: "ns=2;s=House.Room.T" }
        security: "sign_and_encrypt"
        sampling: 1 s
        window: 24 h
        refresh: 1 min
        record: "data/live.csv"
    }
    drives: ["\"PN-OUTDOOR\".T <- outdoor_c °C"]
    compares: ["\"PN-SPACE\".room.T ~ indoor_c °C within 0.5 K for 99 %"]
}
```

`--live` opens a session on the server and subscribes to the nodes, one
column per node. The server samples them every `sampling` (one second by
default) and sends every new reading, even one equal to the last. A
reading is dated by its source timestamp, else the server's. A value whose
status is not good is left out, and a boolean counts as 1 or 0.

**Nodes.** In their text form: `ns=2;s=House.Room.T`, `ns=2;i=1002`,
`i=2258`, `ns=3;g=<guid>`, `ns=1;b=<base64>`, or `nsu=<namespace uri>;s=...`.

**Security.** `security:` is `"none"` (the default), `"sign"` or
`"sign_and_encrypt"`. `policy:` is `Basic256Sha256` by default, or
`Aes128Sha256RsaOaep`, `Aes256Sha256RsaPss` or `Basic128Rsa15`. ArcLang
makes its own certificate the first time, in `.arclang/opcua-pki/` beside
the model (`ARCLANG_OPCUA_PKI` names another folder).

The first connection to a secured server is refused, on purpose. The
server's certificate is put in `rejected/`, and the message says so. Once
you have checked it, move it to `trusted/`; the server must also trust
ArcLang's certificate (`own/cert.der`).

**Identity.** The session is anonymous, unless `ARCLANG_OPCUA_USERNAME` and
`ARCLANG_OPCUA_PASSWORD` are set: credentials are never written in the
model. When the server cannot be reached, or the session is lost, the twin
says so and opens a new session every two seconds.

When the server's host name gives several addresses (`localhost` is often
`::1` and then 127.0.0.1), the twin connects to the first that answers and
says which. For a secured session, the server's certificate must then name
that address.

`record:` and the window work as for MQTT.

## Live data (Kafka)

```arc
data: {
    kafka: "kafka://broker-1:9092,broker-2:9092"
    topics: ["house.telemetry"]
    window: 24 h
    refresh: 1 min
    record: "data/live.csv"
}
```

`--live` reads every partition of the topics. Kafka keeps what was sent, so
the twin starts `window` ago and its first run needs no wait; after that it
reads the messages as they come. It reads without a consumer group: it
commits no offset and does not move any other consumer's. `client_id`
names the twin to the brokers.

A message is decoded as an MQTT message is: a JSON object (a column per
field, at its `time`) or a bare number. A number's column is the message's
key, else the topic's last level (`house.indoor_c` gives `indoor_c`). With
no `time` in the message, the instant is the message's own Kafka timestamp.

`kafkas://` connects over TLS and checks the broker's certificate against
the system's trusted roots. A user and a password come from
`ARCLANG_KAFKA_USERNAME` and `ARCLANG_KAFKA_PASSWORD`, never from the model.
The mechanism is `sasl:` `"plain"` (the default), `"scram-sha-256"` or
`"scram-sha-512"`; a broker that turns the user down is reported as
*refused*. When the broker stops answering, the twin says so within a few
seconds and reconnects every two seconds. It takes up where it stopped, so
nothing is read twice.

## Live data (HTTP)

```arc
data: {
    http: "https://api.example.com/houses/042/latest"
    sampling: 30 s
    window: 24 h
    refresh: 5 min
}
```

`--live` sends a GET every `sampling` (ten seconds by default). The answer
is a JSON object, as an MQTT message's, or a list of them. Only what is
newer than what was already read is kept, column by column. An API that
answers its latest hour at every call therefore adds only the new samples.
A bearer token comes from `ARCLANG_HTTP_TOKEN`, never from the model. A 401
or a 403 is reported as *refused*.

**What live means here.** By default (`advance: "window"`), the twin runs
the model again over the window at every refresh: the verdict comes at most
one `refresh` after the data, and the model starts each window from its
initial state.

With `advance: "step"`, the model advances with the data instead:

```arc
data: { opcua: "opc.tcp://plc:4840" nodes: { ... } window: 24 h refresh: 1 min advance: "step" }
```

The model is built once as an FMU: OpenModelica's Co-Simulation FMU (FMI
2.0), solved by CVODE. That takes a few seconds, while the source is
already followed. A small stepper then advances the FMU to each new instant
of the data. It is compiled where the FMU was built: in the OpenModelica
image with Docker, or with the local C compiler (`cc`, or `$CC`) next to a
local `omc`.
- **Inputs.** Between two instants, each driven input goes linearly from one
  sample to the next, as with a file.
- **State.** The model keeps its state from one sample to the next. It is
  never run again from the start of a window, so a refresh only judges what
  the model has already computed and costs next to nothing.
- **Timing.** An instant is stepped to once the next one has arrived, since
  an OPC UA instant comes one node at a time. A sample older than the
  model's time drives nothing, but it is still compared.
- **Failures.** If the stepper fails, the twin says so, and the model starts
  again from its initial state at the next sample.

On the heat pump's day, both ways give the same verdict to the last digit.

The model is still not driven by a control loop of its own: the twin
watches, it does not command.

## Online

On arclang.fr, signed in, "ArcLang: Run the digital twins" sends the
model's folder (the model, its Modelica sources, the measurements in CSV or
Parquet beside it) to the same sandboxed OpenModelica as the simulations,
counted in the same daily runs, and shows the twins in the same view as the
desktop. A job there has no network: a twin on live data (MQTT, OPC UA,
Kafka, HTTP) runs in the desktop extension, which reaches the engineer's
network.

The example in `examples/building/heat-pump/` carries a twin and a sample of
data. **The sample is synthetic**: its header says how it was made. Replace
it with a logger's export that has the same columns.
