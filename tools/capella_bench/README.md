# Capella round-trip bench

Measures the Capella compatibility of ArcLang on a corpus of real Capella
models: every model is imported, exported back to Capella, imported again,
and the two imports are compared declaration by declaration. A model is
identical when every declaration and every trace comes back, none is lost
or changed and none appears.

```sh
cargo build --release
tools/capella_bench/fetch_corpus.sh ~/capella-corpus      # once, about 1.5 GB
python3 tools/capella_bench/bench.py ~/capella-corpus /tmp/capella-bench --json /tmp/bench.json
```

The bench exits 1 when a model is not identical, when a model fails to
import that `baseline.json` does not list as refused (the two empty XMI
files of the Capella test suite), or when fewer models or declarations are
measured than the baseline holds. When the compiler learns to carry more,
raise `models` and `declarations` in `baseline.json` with the new figures.

| File | What it does |
|---|---|
| `corpus.txt` | The corpus: each repository and the revision it is pinned to. |
| `fetch_corpus.sh` | Fetches the corpus at those revisions (one shallow fetch each). |
| `bench.py` | The round trip of the whole corpus, in parallel, against the baseline. |
| `roundtrip.py` | The round trip of one model, with what changed per keyword. |
| `shape_check.py` | Every (parent kind, feature, child kind) and attribute an export writes, against what Capella 7 models write: EMF ignores an unknown feature in silence. |
| `capella_validate.sh` | Capella's own validation of a model, headless (needs a local Capella). |

## Validation in Capella

The CI cannot run Capella; locally, with Capella 7.1 installed (and the
Requirements, Cybersecurity and STPA add-ons in its `dropins`):

```sh
python3 tools/capella_bench/roundtrip.py "Model.capella" /tmp/rt --validate
# or directly, on an exported folder:
CAPELLA_HOME=~/tools/capella/Capella.app/Contents/MacOS \
  sh tools/capella_bench/capella_validate.sh /tmp/rt/export Model.aird /tmp/validation.json
```

A model saved by an older Capella needs `MIGRATE=1` (Capella's migration
first). Compare the errors with the original's: an export should not have
more.
