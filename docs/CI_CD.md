# CI/CD: the gate in your pipeline

ArcLang checks a model the way a pipeline checks code: every model compiled, the production gate, the simulations that simulate, a SARIF file for code scanning, and on a pull request one comment that says what the change does to the model. Everything is computed by the compiler (`arclang`); the pipeline files only call it.

What was run, and what was not: the generated pipelines are tested as files (valid YAML, no secret, pinned versions, the commands they call run in the tests), **not against GitHub, GitLab, Azure DevOps or Jenkins themselves**. The first pipeline run is their test. Each section says what is expected to need a look.

## Quick start

```text
arclang ci init                       # the provider is read from the git remote
arclang ci init --provider all        # GitHub, GitLab, Azure DevOps and Jenkins
arclang ci init --dry-run             # print the files, write nothing
```

`ci init` never overwrites a file: if one exists, nothing is written and the command says which (`--force` replaces them). Options: `--provider github|gitlab|azure|jenkins|all`, `--standard iso26262|do178c|iec61508`, `--model <file>` (repeatable; default: every entry-point model, i.e. every `.arc` that no other `.arc` imports, see `arclang ci models`), `--arclang-version`, `--release-repo`, `--branch`.

In VS Code: *ArcLang: Configure CI* (a preview of each file, then *Write*).

| provider | file written | job |
|----------|--------------|-----|
| GitHub Actions | `.github/workflows/arclang.yml` | `ArcLang gate` |
| GitLab CI | `.gitlab/arclang.gitlab-ci.yml`, plus `.gitlab-ci.yml` including it when the project has none | `arclang-gate` |
| Azure DevOps | `azure-pipelines.yml` | `ArcLang gate` |
| Jenkins | `Jenkinsfile` | stage `ArcLang gate` |

Every pipeline: installs the **release binary of the compiler that wrote it** (`ARCLANG_VERSION`, the version of the `arclang` that ran `ci init`; `SHA256SUMS` of the release is checked before the binary is used), then runs

```text
arclang ci run --standard iso26262 --sarif arclang-results/arclang.sarif --summary arclang-results/summary.md
```

and, on a pull request, `arclang report --pr` and posts the report. The binary comes from the GitHub release of `Mbaroudi/arclang` (`--release-repo` for a mirror). The release must contain the `ci` commands, i.e. be the first one after this feature: generating a pipeline with an older `--arclang-version` produces a pipeline that fails on `arclang ci run`.

## `arclang ci run`

For each model: compile, run the simulations of the test cases that `simulate:` (OpenModelica; see below), run the production gate. It prints one line per model, writes the SARIF file and a Markdown summary, and exits non-zero unless **every** model compiles and passes its gate and no simulation fails.

- No model found is a failure (nothing was checked), unless `--allow-empty`.
- A simulation that could not run (no OpenModelica, a stale cache) is **inconclusive**: it is a `verification.inconclusive` warning in the SARIF file and the summary, never a pass. The gate counts a requirement whose verification is inconclusive as not verified.
- `--verify never` skips the simulations.

## The pull-request report

```text
arclang report --pr --from origin/main [--to <head>] [models...]
                   [-f md|json|html] [-o file] [--out-dir dir]
                   [--fail-on blocker|major|never] [--standard ...] [--note "..."]
```

Per model, base against head: the **gate before and after** with the **new blockers** named (and the ones already at the base, and the ones resolved), the coverage change (requirements satisfied and verified), the latency budgets, the constraints that flip, the simulation verdicts (`pass -> fail`, stale), the impact (how many elements the changes reach), the semantic diff by layer (folded; `arclang diff --from` has all of it), and the **before and after pictures**.

- The verdict is the gate's. A model that does not compile at the head, or whose gate fails, is **FAIL**, and the report never says PASS in that case, whatever `--fail-on` says. No model at all is "no model found", not a pass.
- *Exit status*: `--fail-on blocker` (default): 1 when a blocker appears that the base did not have, or a model no longer compiles. `major`: also on a regression (a requirement no longer satisfied or verified, a constraint that stops holding, a budget newly exceeded, a simulation that stops passing). `never`: 0. A gate that already failed at the base and fails the same way is reported FAIL but is not "new": it is the job `ArcLang gate` (`ci run`) that stays red.
- A new model is reported as new (all its blockers count as new); a base that did not compile says so; `.arc` files removed since the base are named.
- **A hidden marker** `<!-- arclang-pr-report -->` is the first line: the pipelines look for it to **update the same comment** instead of adding one per push (GitHub: `gh api` PATCH or POST; GitLab: notes API PUT or POST; Azure DevOps: thread comment PATCH or POST).
- Comments are cut under 60 000 characters (GitHub's limit is 65 536), with a line saying so.
- **Pictures**: `--out-dir D` writes `D/diagrams/<model>-before.svg` and `-after.svg` (an architecture map: the elements by Arcadia layer; added green, modified amber, removed dashed red; not a drawing of the architecture, `arclang diagram` is) and the report links them by relative path to the report file. A comment cannot display a file of the workspace: the pipelines upload `arclang-results/` as an artifact (GitHub: `arclang-report`; GitLab: the job artifacts; Azure DevOps: the `arclang-report` artifact; Jenkins: archived) and `--note` adds the line that says where. Images are SVG only: there is no rasterizer in the compiler.
- The simulation verdicts of the **base** are read from the verdict cache of that revision, which is not in git: a base verdict is "inconclusive" unless the cache is there.

## GitHub Actions

`.github/workflows/arclang.yml`, one job named **`ArcLang gate`**: checkout with full history, install, restore the simulation cache, `ci run`, job summary, SARIF upload (`github/codeql-action/upload-sarif`, `category: arclang`), the report and its comment on pull requests, upload of `arclang-results/`. Permissions: `contents: read`, `pull-requests: write`, `security-events: write`, `actions: read`. The token is `github.token`; nothing else is read.

For a pull request from a fork GitHub gives the token read access only: the SARIF upload and the comment fail and are allowed to (`continue-on-error` on both steps); the gate itself still runs and decides.

### SARIF in code scanning

`arclang sarif [models...]` (also `arclang check --format sarif`, `arclang gate --format sarif`) writes a SARIF 2.1.0 log (validated against the OASIS schema in the tests): one run, one rule per rule id with its meaning and how to answer it (`help`), one result per diagnostic with the file and line, the element as a logical location, a stable fingerprint, and the fix in the message and in `properties.fixHint`.

| rule id | level | what |
|---------|-------|------|
| `compile.error` | error | the model does not compile; line and column from the compiler |
| `gate.<check>` (`gate.timing.budget`, `gate.requirements.satisfaction`, ...) | error for a blocker, warning otherwise | the production gate |
| the rules of `arclang rules` (`trace.requirement`, `metamodel.type`, ...) | per rule | compiler findings; the gate check that also reports it is in `properties.gateCheck` |
| `verification.failed` / `verification.inconclusive` | error / warning | simulations |

The line is exact for a compile error; for the others it is the line where the element the message names is declared (`properties.locationPrecision`: `element`), or the first line of the file (`file`) when the message names none. Code scanning shows the results as alerts on the pull request and the Security tab. It needs GitHub Advanced Security on a private repository.

`arclang sarif` exits 1 when a model does not compile or its gate fails (`--fail-on never` for 0), so that `if: always()` steps can upload it.

### The gate as a required check

Branch protection, by hand: Settings, Branches, the rule of your default branch, *Require status checks to pass*, add **`ArcLang gate`** (it appears in the list after one run). By script (prints the command and runs nothing unless `--apply`):

```text
tools/ci/require-arclang-gate.sh OWNER/REPO [BRANCH]
```

which is, in substance (not executed by ArcLang; `PUT` replaces the whole protection, read the current one first):

```text
gh api repos/OWNER/REPO/branches/main/protection
gh api --method PUT repos/OWNER/REPO/branches/main/protection --input - <<'JSON'
{ "required_status_checks": { "strict": true, "contexts": ["ArcLang gate"] },
  "enforce_admins": false, "required_pull_request_reviews": null, "restrictions": null }
JSON
```

The check name is the job name: do not rename the job without updating the protection. Azure DevOps: a *Build validation* branch policy on the pipeline. GitLab: *Pipelines must succeed* in the merge request settings. Jenkins: the status your SCM plugin publishes.

## GitLab CI

`.gitlab/arclang.gitlab-ci.yml` (job `arclang-gate`, image `ubuntu:24.04`, `GIT_DEPTH: 0`). GitLab does not read SARIF: the file is a job artifact. The merge-request comment needs a **project access token** (scope `api`) that you store in the masked CI/CD variable `ARCLANG_GITLAB_TOKEN`: `CI_JOB_TOKEN` cannot write merge-request notes. Without the variable the job says so and the report stays in the artifacts. The simulations need Docker on the runner; with none they are inconclusive.

## Azure DevOps

`azure-pipelines.yml`, job `ArcLangGate` displayed as `ArcLang gate`. The comment uses `System.AccessToken`: give the project's build service identity *Contribute to pull requests* on the repository. The SARIF file is published as the `CodeAnalysisLogs` artifact, which the *SARIF SAST Scans Tab* extension displays. The simulation cache is restored from the repository root only.

## Jenkins

`Jenkinsfile` (declarative; an existing one is never overwritten). A multibranch pipeline with a pull-request source sets `CHANGE_ID`/`CHANGE_TARGET`: the report is built, printed in the console and archived. **Posting it as a comment is not done**: it depends on the SCM plugin. The SARIF file is read with `recordIssues(tool: sarif(...))` when the Warnings Next Generation plugin has it (inside `catchError`: without it the step is skipped).

## Caching the simulation verdicts

`arclang verify` keeps one verdict per simulated test case in `.arclang/sim-cache/` beside the model, keyed by the generated Modelica package, every Modelica source and the case's parameters and expectations: a verdict is reused while none of them changes, and is "stale" (inconclusive) when one does. The pipelines cache that directory (GitHub: `actions/cache` on `**/.arclang/sim-cache`, key from the hash of every `.arc` and `.mo`; GitLab: `cache:paths`; Azure DevOps: `Cache@2`). `ci run` still runs `verify`: a cached verdict whose key matches is not simulated again.

OpenModelica comes from `ARCLANG_OMC` (a local `omc`), `ARCLANG_OMC_DOCKER` (an image) or Docker with the default pinned image `openmodelica/openmodelica:v1.27.1-minimal`. GitHub's `ubuntu-latest` has Docker; the first run pulls the image and installs the Modelica Standard Library in a Docker volume (neither is cached by the generated workflow).

## Security notes

- **No secret in a generated file.** The tests fail on a literal token or key. The tokens are the provider's own, read at run time: `github.token`, `System.AccessToken`; GitLab's `ARCLANG_GITLAB_TOKEN` is a variable you create.
- **Pinned**: the compiler version is the one that wrote the file, the binary is checked against the release's `SHA256SUMS` (the release also carries a build provenance attestation, `gh attestation verify`), and actions are pinned to a major version (`@v4`). Pin them to a commit SHA if your policy asks for it.
- Values that end up in a shell command or a YAML scalar (`--model`, `--branch`, the version, the release repository) must be plain (`A-Za-z0-9._-/+`) or `ci init` refuses them. Branch names from the event are passed through environment variables, not interpolated into scripts.
- The pull-request report is computed from the head's files: on `pull_request` (not `pull_request_target`) from a fork the token is read-only, which is what you want for code you did not write.
- In VS Code the CI status reads the latest run through the GitHub CLI and its login: ArcLang stores no token, runs nothing in an untrusted workspace, and the executable is the user-level setting `arclang.ci.gh`.

## Limits

- Not run against the four providers by the generator (see the top).
- GitLab: no SARIF ingestion; the comment needs a token you provide. Jenkins: no comment.
- The pictures are SVG, an element map, not the diagrams.
- Models are found by `import`: a model in a repository with no `.arc` entry point is "no model found"; name them with `--model` or on the command line.
- The release binary is Linux x86-64 (the release's asset); other runners need the compiler built or installed another way.
- The base's simulation verdicts are only known when the cache is there.
