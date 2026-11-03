# Capella 7.1 on Linux, headless, in Docker

Runs the real Capella (and the real Capella-Simulink connector) without a
window: the Capella command lines need a display, given here by Xvfb.

1. Capella: download `capella-7.1.0.202606231255-linux-gtk-aarch64.tar.gz`
   (or `-x86_64` with `--platform linux/amd64`) from
   https://download.eclipse.org/capella/core/products/releases/7.1.0/ and
   extract it to `<dir>/capella`. Add the Requirements 0.14 and Cybersecurity
   (Darc) 2.2 dropins for models that use their viewpoints. Do not add the
   STPA dropin: with it, Sirius refuses Capella's own diagram descriptions.
2. `docker build -t capella-linux:7.1.0 tools/capella_linux`
3. The connector: `build_connector.sh <connector checkout> <dir>/capella <work>`
   (its plugins compiled with javac against Capella; it has no binary release).
4. The command lines: `build_bundle.sh <dir>/capella <work>`.

Then, with `-v <dir>/capella:/opt/capella -v tools/capella_linux/container:/bin2:ro`
and a volume for the models:

- `validate_in.sh <model dir> <name.aird> <out>`: Capella's validation
  (`APPID=arclang.diag.validation` prints every status Capella's own command
  line hides behind "Status of project validation"). `validation_report.py`
  counts the report by level and rule.
- `export_in.sh <model dir> <name.aird> <out>`: every diagram as a PNG.
- `draw_in.sh <project dir> <work> <out>`: Capella draws every state machine
  (a Mode State Machine per top-level region, arranged as at its first opening).
- `csc_in.sh <project dir> <config.properties> <work> <out.m>`: the connector,
  as its tests run it (`MIGRATE=1` migrates an older model first).

`tests/fixtures/simulink_connector/m2m_linux/` was made so: `models/*.arc`
exported by ArcLang, `msmregions` from `msmgeo` by
`split_regions.py msmgeo.capella msmregions.capella "M-DUAL=M-A1+M-A2,M-B1+M-B2" "M-TRIPLE=M-X1,M-Y1+M-Y2,M-Z1"`
then `draw_in.sh`, and every configuration through `csc_in.sh`.
