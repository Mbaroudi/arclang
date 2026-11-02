//! The pipeline files `arclang ci init` writes. Each pipeline installs the
//! release of the compiler that wrote it (checksum verified), runs
//! `arclang ci run` (compile, gate, simulation verdicts, SARIF), and on a
//! pull request builds the report and posts it, updating the same comment.
//!
//! No file holds a secret: the providers' own tokens are read from their
//! environment (`github.token`, `System.AccessToken`), and the GitLab token
//! that comments need is a CI/CD variable the user creates.

use super::report::MARKER;
use std::path::Path;

pub const RELEASE_REPO: &str = "Mbaroudi/arclang";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provider {
    Github,
    Gitlab,
    Azure,
    Jenkins,
}

impl Provider {
    pub const ALL: [Provider; 4] = [Provider::Github, Provider::Gitlab, Provider::Azure, Provider::Jenkins];

    pub fn name(self) -> &'static str {
        match self {
            Provider::Github => "github",
            Provider::Gitlab => "gitlab",
            Provider::Azure => "azure",
            Provider::Jenkins => "jenkins",
        }
    }

    pub fn parse(text: &str) -> Option<Provider> {
        match text.to_ascii_lowercase().as_str() {
            "github" | "github-actions" => Some(Provider::Github),
            "gitlab" | "gitlab-ci" => Some(Provider::Gitlab),
            "azure" | "azure-devops" | "azure-pipelines" => Some(Provider::Azure),
            "jenkins" => Some(Provider::Jenkins),
            _ => None,
        }
    }

    /// Guess the provider from a git remote URL.
    pub fn from_remote(url: &str) -> Option<Provider> {
        let url = url.to_ascii_lowercase();
        if url.contains("github.com") || url.contains("github.") {
            Some(Provider::Github)
        } else if url.contains("gitlab") {
            Some(Provider::Gitlab)
        } else if url.contains("dev.azure.com") || url.contains("visualstudio.com") || url.contains("ssh.dev.azure.com") {
            Some(Provider::Azure)
        } else {
            None
        }
    }
}

/// What the pipelines are made from.
#[derive(Clone, Debug)]
pub struct Options {
    /// The compiler release the pipeline installs.
    pub version: String,
    /// `ISO26262`, `DO178C` or `IEC61508`.
    pub standard: String,
    /// Model files, relative to the repository root; empty: every entry point.
    pub models: Vec<String>,
    /// The branch whose pushes run the pipeline.
    pub branch: String,
    /// `owner/repo` the release binaries are downloaded from.
    pub release_repo: String,
}

/// A file to write.
#[derive(Clone, Debug)]
pub struct Planned {
    /// Relative to the repository root.
    pub path: String,
    pub content: String,
}

/// A value that ends up in a shell command or a YAML scalar must be plain.
pub fn check_plain(label: &str, value: &str) -> Result<(), String> {
    let ok = !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | '+'));
    if ok && !value.starts_with('-') && !value.contains("..") {
        Ok(())
    } else {
        Err(format!("{label} '{value}' has characters that do not belong in a pipeline file (letters, digits and . _ - / + only)"))
    }
}

pub fn validate(options: &Options) -> Result<(), String> {
    check_plain("version", &options.version)?;
    check_plain("branch", &options.branch)?;
    check_plain("release repository", &options.release_repo)?;
    for model in &options.models {
        check_plain("model", model)?;
    }
    Ok(())
}

fn fill(template: &str, options: &Options, extra: &[(&str, String)]) -> String {
    let models = if options.models.is_empty() { String::new() } else { format!(" {}", options.models.join(" ")) };
    let mut out = template
        .replace("@@VERSION@@", &options.version)
        .replace("@@STANDARD@@", &options.standard.to_ascii_lowercase())
        .replace("@@MODELS@@", &models)
        .replace("@@BRANCH@@", &options.branch)
        .replace("@@REPO@@", &options.release_repo)
        .replace("@@MARKER@@", MARKER);
    for (key, value) in extra {
        out = out.replace(&format!("@@{key}@@"), value);
    }
    out
}

const GITHUB: &str = r###"# Written by `arclang ci init` (ArcLang @@VERSION@@).
# Not run against GitHub by the generator: read it once, then let the first pull request prove it.
#
# The job is named "ArcLang gate": that name is the status check to require in the
# branch protection (docs/CI_CD.md).
name: ArcLang

on:
  pull_request:
  push:
    branches: [@@BRANCH@@]
  workflow_dispatch:

# security-events: SARIF upload (code scanning). pull-requests: the report comment.
# On a pull request from a fork GitHub gives the token read access only: the SARIF
# upload and the comment are then skipped without failing the gate.
permissions:
  contents: read
  pull-requests: write
  security-events: write
  actions: read

concurrency:
  group: arclang-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  ARCLANG_VERSION: "@@VERSION@@"

jobs:
  gate:
    name: ArcLang gate
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      # Full history: the report compares the pull request with its base.
      - name: Check out
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install ArcLang (release binary, checksum verified)
        shell: bash
        run: |
          set -euo pipefail
          asset="arclang-linux-x86_64.tar.gz"
          base="https://github.com/@@REPO@@/releases/download/v${ARCLANG_VERSION}"
          work="$(mktemp -d)"
          cd "$work"
          curl -fsSL --retry 3 -O "$base/$asset" -O "$base/SHA256SUMS"
          grep " ${asset}\$" SHA256SUMS | sha256sum -c -
          tar xzf "$asset"
          mkdir -p "$HOME/.local/bin"
          install -m 0755 arclang-linux-x86_64 "$HOME/.local/bin/arclang"
          echo "$HOME/.local/bin" >> "$GITHUB_PATH"

      # The verdicts of the simulations (OpenModelica) are kept between runs: a
      # verdict is reused while the model, its Modelica sources and the case are unchanged.
      - name: Restore the simulation verdicts
        uses: actions/cache@v4
        with:
          path: "**/.arclang/sim-cache"
          key: arclang-sim-${{ hashFiles('**/*.arc', '**/*.mo') }}
          restore-keys: arclang-sim-

      # Compile every model, run the production gate, simulate the cases that
      # simulate (OpenModelica through Docker, when the runner has it), write the SARIF.
      - name: Check, gate and verify every model
        run: arclang ci run --standard @@STANDARD@@ --sarif arclang-results/arclang.sarif --summary arclang-results/summary.md@@MODELS@@

      - name: Job summary
        if: ${{ !canceled() && hashFiles('arclang-results/summary.md') != '' }}
        run: cat arclang-results/summary.md >> "$GITHUB_STEP_SUMMARY"

      - name: Upload the findings to code scanning (SARIF)
        if: ${{ !canceled() && hashFiles('arclang-results/arclang.sarif') != '' }}
        uses: github/codeql-action/upload-sarif@v3
        continue-on-error: true
        with:
          sarif_file: arclang-results/arclang.sarif
          category: arclang

      - name: Pull-request report
        if: ${{ !canceled() && github.event_name == 'pull_request' }}
        continue-on-error: true
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
          BASE_REF: ${{ github.base_ref }}
          PR_NUMBER: ${{ github.event.pull_request.number }}
          RUN_URL: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
        run: |
          set -euo pipefail
          arclang report --pr --from "origin/${BASE_REF}" --standard @@STANDARD@@ \
            --out-dir arclang-results/report --format md --output arclang-results/pr-report.md \
            --note "Before and after diagrams (SVG): artifact **arclang-report** of [this run](${RUN_URL})." \
            --fail-on never@@MODELS@@
          # One comment per pull request, updated on each push (found by its hidden marker).
          id="$(gh api "repos/${GITHUB_REPOSITORY}/issues/${PR_NUMBER}/comments" --paginate \
            --jq '.[] | select(.body | contains("@@MARKER@@")) | .id' | sed -n 1p)"
          if [ -n "$id" ]; then
            gh api --method PATCH "repos/${GITHUB_REPOSITORY}/issues/comments/${id}" -F body=@arclang-results/pr-report.md > /dev/null
          else
            gh api --method POST "repos/${GITHUB_REPOSITORY}/issues/${PR_NUMBER}/comments" -F body=@arclang-results/pr-report.md > /dev/null
          fi

      - name: Keep the results
        if: ${{ !canceled() }}
        uses: actions/upload-artifact@v4
        with:
          name: arclang-report
          path: arclang-results/
          if-no-files-found: ignore
"###;

const GITLAB_ROOT: &str = r###"# Written by `arclang ci init`: includes the ArcLang pipeline.
include:
  - local: .gitlab/arclang.gitlab-ci.yml
"###;

const GITLAB: &str = r###"# Written by `arclang ci init` (ArcLang @@VERSION@@).
# Not run against GitLab by the generator: read it once, then let the first merge request prove it.
#
# Include it from .gitlab-ci.yml:
#   include:
#     - local: .gitlab/arclang.gitlab-ci.yml
#
# GitLab does not read SARIF itself: the file is kept as a job artifact.
# The merge-request comment needs a project access token (scope api) stored in the
# masked CI/CD variable ARCLANG_GITLAB_TOKEN; without it the report is only an artifact.
# The simulations (OpenModelica) need Docker on the runner: without it they are reported
# as inconclusive, never as passed.
stages:
  - test

arclang-gate:
  stage: test
  image: ubuntu:24.04
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == "@@BRANCH@@"
  variables:
    ARCLANG_VERSION: "@@VERSION@@"
    GIT_DEPTH: "0"
  cache:
    key: arclang-sim-$CI_COMMIT_REF_SLUG
    paths:
      - "**/.arclang/sim-cache/"
  before_script:
    - apt-get update -qq && apt-get install -y -qq curl ca-certificates git jq > /dev/null
    - |
      set -eu
      asset="arclang-linux-x86_64.tar.gz"
      base="https://github.com/@@REPO@@/releases/download/v${ARCLANG_VERSION}"
      work="$(mktemp -d)"
      (cd "$work" && curl -fsSL --retry 3 -O "$base/$asset" -O "$base/SHA256SUMS" \
        && grep " ${asset}\$" SHA256SUMS | sha256sum -c - && tar xzf "$asset")
      install -m 0755 "$work/arclang-linux-x86_64" /usr/local/bin/arclang
      arclang --version
  script:
    - |
      status=0
      arclang ci run --standard @@STANDARD@@ --sarif arclang-results/arclang.sarif --summary arclang-results/summary.md@@MODELS@@ || status=$?
      if [ -n "${CI_MERGE_REQUEST_IID:-}" ]; then
        git fetch --quiet origin "${CI_MERGE_REQUEST_TARGET_BRANCH_NAME}"
        arclang report --pr --from "origin/${CI_MERGE_REQUEST_TARGET_BRANCH_NAME}" --standard @@STANDARD@@ \
          --out-dir arclang-results/report --format md --output arclang-results/pr-report.md \
          --note "Before and after diagrams (SVG): artifacts of [this job](${CI_JOB_URL})." \
          --fail-on never@@MODELS@@ || true
        if [ -n "${ARCLANG_GITLAB_TOKEN:-}" ] && [ -f arclang-results/pr-report.md ]; then
          api="${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/merge_requests/${CI_MERGE_REQUEST_IID}/notes"
          id="$(curl -fsS --header "PRIVATE-TOKEN: ${ARCLANG_GITLAB_TOKEN}" "${api}?per_page=100" \
            | jq -r '[.[] | select(.body | contains("@@MARKER@@"))][0].id // empty')"
          if [ -n "$id" ]; then
            curl -fsS --request PUT --header "PRIVATE-TOKEN: ${ARCLANG_GITLAB_TOKEN}" --data-urlencode "body@arclang-results/pr-report.md" "${api}/${id}" > /dev/null
          else
            curl -fsS --request POST --header "PRIVATE-TOKEN: ${ARCLANG_GITLAB_TOKEN}" --data-urlencode "body@arclang-results/pr-report.md" "${api}" > /dev/null
          fi
        else
          echo "No merge-request comment: set the masked CI/CD variable ARCLANG_GITLAB_TOKEN (project access token, scope api). The report is in the job artifacts."
        fi
      fi
      exit $status
  artifacts:
    when: always
    name: arclang-report
    paths:
      - arclang-results/
    expire_in: 30 days
"###;

const AZURE: &str = r###"# Written by `arclang ci init` (ArcLang @@VERSION@@).
# Not run against Azure DevOps by the generator: read it once, then let the first pull request prove it.
#
# The job is displayed as "ArcLang gate". For the pull-request comment the build service
# identity needs "Contribute to pull requests" on the repository; the token is the
# pipeline's own (System.AccessToken), nothing is stored in this file. For a pull request
# gate, add a branch policy "Build validation" on this pipeline.
# The SARIF file is published as the CodeAnalysisLogs artifact, which the SARIF SAST Scans
# Tab extension displays.
trigger:
  branches:
    include:
      - @@BRANCH@@

pr:
  branches:
    include:
      - "*"

pool:
  vmImage: ubuntu-latest

variables:
  ARCLANG_VERSION: "@@VERSION@@"

jobs:
  - job: ArcLangGate
    displayName: ArcLang gate
    timeoutInMinutes: 30
    steps:
      - checkout: self
        fetchDepth: 0

      - bash: |
          set -euo pipefail
          asset="arclang-linux-x86_64.tar.gz"
          base="https://github.com/@@REPO@@/releases/download/v${ARCLANG_VERSION}"
          work="$(mktemp -d)"
          cd "$work"
          curl -fsSL --retry 3 -O "$base/$asset" -O "$base/SHA256SUMS"
          grep " ${asset}\$" SHA256SUMS | sha256sum -c -
          tar xzf "$asset"
          mkdir -p "$HOME/.local/bin"
          install -m 0755 arclang-linux-x86_64 "$HOME/.local/bin/arclang"
          echo "##vso[task.prependpath]$HOME/.local/bin"
        displayName: Install ArcLang (release binary, checksum verified)

      - task: Cache@2
        displayName: Restore the simulation verdicts
        inputs:
          key: 'arclang-sim | "$(Agent.OS)" | **/*.arc, **/*.mo'
          restoreKeys: |
            arclang-sim | "$(Agent.OS)"
          path: $(Build.SourcesDirectory)/.arclang/sim-cache

      - bash: arclang ci run --standard @@STANDARD@@ --sarif arclang-results/sarif/arclang.sarif --summary arclang-results/summary.md@@MODELS@@
        displayName: Check, gate and verify every model

      - bash: |
          set -euo pipefail
          target="${TARGET_BRANCH#refs/heads/}"
          git fetch --quiet origin "$target"
          arclang report --pr --from "origin/${target}" --standard @@STANDARD@@ \
            --out-dir arclang-results/report --format md --output arclang-results/pr-report.md \
            --note "Before and after diagrams (SVG): artifact **arclang-report** of this build." \
            --fail-on never@@MODELS@@
          api="${COLLECTION_URI}${PROJECT}/_apis/git/repositories/${REPO_ID}/pullRequests/${PR_ID}/threads"
          auth="Authorization: Bearer ${SYSTEM_ACCESSTOKEN}"
          found="$(curl -fsS -H "$auth" "${api}?api-version=7.1" \
            | jq -r '[.value[] | select(.comments[0].content // "" | contains("@@MARKER@@")) | {thread: .id, comment: .comments[0].id}][0] // empty')"
          if [ -n "$found" ]; then
            thread="$(echo "$found" | jq -r .thread)"; comment="$(echo "$found" | jq -r .comment)"
            jq -n --rawfile body arclang-results/pr-report.md '{content: $body}' \
              | curl -fsS -X PATCH -H "$auth" -H "Content-Type: application/json" -d @- "${api}/${thread}/comments/${comment}?api-version=7.1" > /dev/null
          else
            jq -n --rawfile body arclang-results/pr-report.md '{comments: [{parentCommentId: 0, content: $body, commentType: 1}], status: 1}' \
              | curl -fsS -X POST -H "$auth" -H "Content-Type: application/json" -d @- "${api}?api-version=7.1" > /dev/null
          fi
        displayName: Pull-request report
        condition: and(succeededOrFailed(), eq(variables['Build.Reason'], 'PullRequest'))
        env:
          SYSTEM_ACCESSTOKEN: $(System.AccessToken)
          TARGET_BRANCH: $(System.PullRequest.TargetBranch)
          PR_ID: $(System.PullRequest.PullRequestId)
          REPO_ID: $(Build.Repository.ID)
          COLLECTION_URI: $(System.CollectionUri)
          PROJECT: $(System.TeamProject)

      - task: PublishPipelineArtifact@1
        displayName: Publish the SARIF file
        condition: succeededOrFailed()
        inputs:
          targetPath: arclang-results/sarif
          artifact: CodeAnalysisLogs

      - task: PublishPipelineArtifact@1
        displayName: Publish the report
        condition: succeededOrFailed()
        inputs:
          targetPath: arclang-results
          artifact: arclang-report
"###;

const JENKINS: &str = r###"// Written by `arclang ci init` (ArcLang @@VERSION@@).
// Not run against Jenkins by the generator: read it once, then let the first build prove it.
//
// A multibranch pipeline with a pull-request source (GitHub, GitLab, Bitbucket) sets
// CHANGE_ID and CHANGE_TARGET; the report is then built and archived, and printed in the
// console. Posting it as a comment depends on the SCM plugin of your controller and is not
// done here. The SARIF file is read by the Warnings Next Generation plugin when it is
// installed (recordIssues with the sarif tool); without it that step is skipped.
pipeline {
    agent any

    options {
        timeout(time: 30, unit: 'MINUTES')
        timestamps()
    }

    environment {
        ARCLANG_VERSION = '@@VERSION@@'
        PATH = "${env.HOME}/.local/bin:${env.PATH}"
    }

    stages {
        stage('Install ArcLang') {
            steps {
                sh '''
                    set -eu
                    asset="arclang-linux-x86_64.tar.gz"
                    base="https://github.com/@@REPO@@/releases/download/v${ARCLANG_VERSION}"
                    work="$(mktemp -d)"
                    cd "$work"
                    curl -fsSL --retry 3 -O "$base/$asset" -O "$base/SHA256SUMS"
                    grep " ${asset}\\$" SHA256SUMS | sha256sum -c -
                    tar xzf "$asset"
                    mkdir -p "$HOME/.local/bin"
                    install -m 0755 arclang-linux-x86_64 "$HOME/.local/bin/arclang"
                '''
            }
        }

        stage('ArcLang gate') {
            steps {
                sh 'arclang ci run --standard @@STANDARD@@ --sarif arclang-results/arclang.sarif --summary arclang-results/summary.md@@MODELS@@'
            }
        }
    }

    post {
        always {
            script {
                if (env.CHANGE_ID) {
                    sh '''
                        git fetch --quiet origin "+refs/heads/${CHANGE_TARGET}:refs/remotes/origin/${CHANGE_TARGET}"
                        arclang report --pr --from "origin/${CHANGE_TARGET}" --standard @@STANDARD@@ \\
                          --out-dir arclang-results/report --format md --output arclang-results/pr-report.md \\
                          --note "Before and after diagrams (SVG): archived with this build." \\
                          --fail-on never@@MODELS@@ || true
                        cat arclang-results/pr-report.md || true
                    '''
                }
            }
            catchError(buildResult: 'SUCCESS', stageResult: 'SUCCESS') {
                recordIssues(tool: sarif(pattern: 'arclang-results/arclang.sarif'))
            }
            archiveArtifacts artifacts: 'arclang-results/**', allowEmptyArchive: true, fingerprint: true
        }
    }
}
"###;

/// The files of one provider.
pub fn render(provider: Provider, options: &Options) -> Vec<Planned> {
    let one = |path: &str, template: &str| Planned { path: path.to_string(), content: fill(template, options, &[]) };
    match provider {
        Provider::Github => vec![one(".github/workflows/arclang.yml", GITHUB)],
        Provider::Gitlab => vec![one(".gitlab/arclang.gitlab-ci.yml", GITLAB)],
        Provider::Azure => vec![one("azure-pipelines.yml", AZURE)],
        Provider::Jenkins => vec![one("Jenkinsfile", JENKINS)],
    }
}

/// The root `.gitlab-ci.yml` that includes the GitLab pipeline: written only
/// when the project has none.
pub fn gitlab_root() -> Planned {
    Planned { path: ".gitlab-ci.yml".to_string(), content: GITLAB_ROOT.to_string() }
}

/// What a generated file still says to do: for the message after writing.
pub fn next_step(provider: Provider) -> &'static str {
    match provider {
        Provider::Github => "Commit it and open a pull request; make the job \"ArcLang gate\" a required check (docs/CI_CD.md).",
        Provider::Gitlab => "Commit it; set the masked variable ARCLANG_GITLAB_TOKEN for merge-request comments (docs/CI_CD.md).",
        Provider::Azure => "Create the pipeline from the file; add a Build validation branch policy (docs/CI_CD.md).",
        Provider::Jenkins => "Create a multibranch pipeline on the repository (docs/CI_CD.md).",
    }
}

/// The default branch of a repository: `origin/HEAD`, else the current
/// branch when it is `main` or `master`, else `main`.
pub fn default_branch(root: &Path) -> String {
    use crate::compiler::semgit::git;
    if let Ok(out) = git::run(root, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]) {
        if let Some(name) = out.trim().strip_prefix("origin/") {
            if check_plain("branch", name).is_ok() {
                return name.to_string();
            }
        }
    }
    if let Ok(out) = git::run(root, &["symbolic-ref", "--short", "HEAD"]) {
        let name = out.trim();
        if name == "main" || name == "master" {
            return name.to_string();
        }
    }
    "main".to_string()
}

/// The provider the `origin` remote points at.
pub fn detect(root: &Path) -> Option<Provider> {
    use crate::compiler::semgit::git;
    let url = git::run(root, &["remote", "get-url", "origin"]).ok()?;
    Provider::from_remote(url.trim())
}
