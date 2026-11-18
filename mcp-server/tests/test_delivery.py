"""Tables, documents, a new model, and what a pipeline reads: all with the real binary."""

import json
import subprocess
from pathlib import Path

import pytest

from test_versioning import git


@pytest.fixture
def repository(workspace, model):
    git(workspace, "init", "-q", "-b", "main")
    git(workspace, "add", "braking.arc")
    git(workspace, "commit", "-q", "-m", "The reference model")
    return workspace


# ---------------------------------------------------------------------- tables


async def test_table_lists_the_kinds_the_compiler_computes(server, model):
    answer = await server.call_tool("arclang_table", {"model_path": str(model), "list": True})
    assert answer.splitlines()[0] == "Tables of braking.arc: the declared tables and the kinds computed"
    listed = json.loads(answer.split("\n\n", 1)[1])
    assert "requirements_coverage" in listed["kinds"] and listed["declared"] == []


async def test_a_computed_table_in_each_text_format(server, model):
    for table_format, mark in (
        ("md", "| REQ-BRK-001 |"),
        ("csv", "REQ-BRK-001,"),
        ("tsv", "REQ-BRK-001\t"),
        ("json", '"REQ-BRK-001"'),
        ("html", "<table"),
    ):
        answer = await server.call_tool(
            "arclang_table",
            {"model_path": str(model), "kind": "requirements_coverage", "format": table_format},
        )
        assert answer.splitlines()[0] == "Table requirements_coverage of braking.arc: succeeded", answer
        assert mark in answer, table_format


async def test_table_filters_scopes_and_shapes(server, model):
    gaps = await server.call_tool(
        "arclang_table", {"model_path": str(model), "kind": "requirements_coverage", "format": "md", "gaps": True}
    )
    assert "0 rows" in gaps and "REQ-BRK-001 |" not in gaps
    elements = await server.call_tool(
        "arclang_table",
        {"model_path": str(model), "kind": "elements", "row_type": "LogicalComponent", "format": "csv",
         "columns": ["id", "name"], "filter": "name:Brake"},
    )
    body = elements.split("\n\n", 1)[1]
    assert body.splitlines() == ["Id,Name", "LC-002,BrakeController"], elements
    scoped = await server.call_tool(
        "arclang_table", {"model_path": str(model), "kind": "elements", "scope": "PA", "format": "csv"}
    )
    assert "PN-001" in scoped and "LC-001," not in scoped
    matrix = await server.call_tool(
        "arclang_table",
        {"model_path": str(model), "kind": "traceability_matrix", "format": "md", "kinds": ["satisfies"],
         "gate_standard": "DO178C"},
    )
    assert matrix.splitlines()[0].endswith(": succeeded"), matrix


async def test_table_prints_the_declaration_to_paste(server, model):
    answer = await server.call_tool(
        "arclang_table", {"model_path": str(model), "kind": "requirements_coverage", "declaration": True}
    )
    assert 'table "Requirement coverage" {' in answer and "kind: requirements_coverage" in answer


async def test_table_writes_a_file_only_where_told(server, model, workspace):
    answer = await server.call_tool(
        "arclang_table",
        {"model_path": str(model), "kind": "icd", "format": "xlsx", "output_path": "out/icd.xlsx"},
    )
    written = workspace / "out" / "icd.xlsx"
    assert f"Written to {written}" in answer and written.read_bytes()[:2] == b"PK"
    with pytest.raises(Exception, match="give 'output_path'"):
        await server.call_tool("arclang_table", {"model_path": str(model), "kind": "icd", "format": "xlsx"})
    with pytest.raises(Exception, match="ask one"):
        await server.call_tool("arclang_table", {"model_path": str(model), "kind": "icd", "name": "T"})
    with pytest.raises(Exception, match="must be one of"):
        await server.call_tool("arclang_table", {"model_path": str(model), "kind": "everything"})


async def test_a_table_the_model_does_not_declare_is_a_failure(server, model):
    answer = await server.call_tool("arclang_table", {"model_path": str(model), "name": "No such table"})
    assert "FAILED" in answer.splitlines()[0], answer


# ------------------------------------------------------------ documents, new


async def test_document_is_the_design_review_or_a_template_filled(server, model, workspace):
    answer = await server.call_tool("arclang_document", {"model_path": str(model), "output_path": "review.md"})
    written = workspace / "review.md"
    assert answer.splitlines()[0] == f"Document of braking.arc: written to {written}"
    assert "EmergencyBraking" in written.read_text()
    template = workspace / "note.md"
    template.write_text("# {{ model }}\n\n{{ gate }}\n")
    await server.call_tool(
        "arclang_document",
        {"model_path": str(model), "template_path": "note.md", "output_path": "out/note.html"},
    )
    html = (workspace / "out" / "note.html").read_text()
    assert "<h1" in html and "EmergencyBraking" in html
    await server.call_tool("arclang_document", {"model_path": str(model), "output_path": "review.docx"})
    assert (workspace / "review.docx").read_bytes()[:2] == b"PK"


async def test_document_refuses_a_format_it_does_not_write(server, model, workspace):
    with pytest.raises(Exception, match="chooses the format"):
        await server.call_tool("arclang_document", {"model_path": str(model), "output_path": "review.pdf"})
    answer = await server.call_tool(
        "arclang_document", {"model_path": str(model), "template_path": "missing.md", "output_path": "r.md"}
    )
    assert "FAILED" in answer.splitlines()[0] and not (workspace / "r.md").exists()


async def test_new_writes_a_model_that_compiles_and_never_over_one(server, workspace):
    answer = await server.call_tool("arclang_new", {"name": "door"})
    written = workspace / "door.arc"
    assert answer.splitlines()[0] == f"New model 'door' from the cesam template: written to {written}"
    assert (await server.compiler.check(written)).ok
    assert "PRODUCTION GATE: PASS" in (await server.compiler.gate(written)).stdout
    before = written.read_text()
    again = await server.call_tool("arclang_new", {"name": "door"})
    assert "FAILED" in again.splitlines()[0] and "exists already" in again
    assert written.read_text() == before
    elsewhere = await server.call_tool("arclang_new", {"name": "lock", "output_path": "models/lock.arc"})
    assert (workspace / "models" / "lock.arc").is_file(), elsewhere
    unknown = await server.call_tool("arclang_new", {"name": "x", "template": "no-such-template"})
    assert "FAILED" in unknown.splitlines()[0] and not (workspace / "x.arc").exists()


# ------------------------------------------------------- report, sarif, ci


async def test_the_pull_request_report_compares_the_base_and_the_tree(server, repository, model):
    model.write_text(model.read_text().replace('priority: "Critical"', 'priority: "High"'))
    answer = await server.call_tool("arclang_report", {"from_rev": "HEAD"})
    assert answer.splitlines()[0] == "Pull-request report from HEAD: succeeded", answer
    assert "## ArcLang gate: **PASS**" in answer and "`braking.arc`" in answer
    as_json = await server.call_tool(
        "arclang_report",
        {"from_rev": "HEAD", "format": "json", "output_path": "out/report.json", "models": ["braking.arc"],
         "note": "from the test", "standard": "iso26262", "fail_on": "never"},
    )
    report = json.loads((repository / "out" / "report.json").read_text())
    assert "Written to" in as_json and report["models"][0]["model"] == "braking.arc"


async def test_a_report_that_makes_the_gate_worse_is_a_failure(server, repository, model):
    model.write_text(model.read_text().replace('verdict: "pass"', 'verdict: "fail"'))
    answer = await server.call_tool("arclang_report", {"from_rev": "HEAD"})
    assert "FAILED" in answer.splitlines()[0] and "makes the gate worse" in answer
    lenient = await server.call_tool("arclang_report", {"from_rev": "HEAD", "fail_on": "never"})
    assert lenient.splitlines()[0].endswith(": succeeded")


async def test_sarif_is_one_log_of_the_models(server, repository, model):
    answer = await server.call_tool("arclang_sarif", {"output_path": "out/gate.sarif"})
    log = json.loads((repository / "out" / "gate.sarif").read_text())
    assert answer.splitlines()[0] == "SARIF log: succeeded" and log["version"] == "2.1.0"
    model.write_text(model.read_text().replace('verdict: "pass"', 'verdict: "fail"'))
    failing = await server.call_tool("arclang_sarif", {"models": ["braking.arc"]})
    assert "FAILED" in failing.splitlines()[0] and '"version": "2.1.0"' in failing
    result = await server.call_tool("arclang_sarif", {"models": ["braking.arc"], "fail_on": "never"})
    assert result.splitlines()[0] == "SARIF log: succeeded"


async def test_one_model_as_sarif_from_check_and_from_the_gate(server, model):
    checked = await server.call_tool("arclang_validate", {"model_path": str(model), "format": "sarif"})
    assert checked.splitlines()[0] == "Check of braking.arc: SARIF 2.1.0 log"
    assert json.loads(checked.split("\n\n", 1)[1])["version"] == "2.1.0"
    gated = await server.call_tool("arclang_gate", {"model_path": str(model), "format": "sarif"})
    assert gated.splitlines()[0] == "Production gate of braking.arc: PASS; SARIF 2.1.0 log"
    model.write_text(model.read_text().replace('verdict: "pass"', 'verdict: "fail"'))
    gated = await server.call_tool("arclang_gate", {"model_path": str(model), "format": "sarif"})
    assert gated.splitlines()[0] == "Production gate of braking.arc: FAIL; SARIF 2.1.0 log"
    assert json.loads(gated.split("\n\n", 1)[1])["runs"]


async def test_ci_init_prints_the_files_and_writes_them_only_when_told(server, repository):
    answer = await server.call_tool("arclang_ci_init", {"directory": ".", "provider": "github"})
    assert "NOT written (pass write: true to write these files)" in answer.splitlines()[0], answer
    assert "--- .github/workflows/arclang.yml" in answer and "Dry run: nothing was written." in answer
    assert not (repository / ".github").exists()
    written = await server.call_tool(
        "arclang_ci_init",
        {"directory": ".", "provider": "github", "write": True, "models": ["braking.arc"],
         "standard": "do178c", "branch": "main", "arclang_version": "4.12.0"},
    )
    workflow = (repository / ".github" / "workflows" / "arclang.yml").read_text()
    assert "written at the top of" in written.splitlines()[0], written
    assert "braking.arc" in workflow and "do178c" in workflow and "4.12.0" in workflow
    again = await server.call_tool("arclang_ci_init", {"directory": ".", "provider": "github", "write": True})
    assert "FAILED" in again.splitlines()[0] and "nothing was written" in again
    forced = await server.call_tool(
        "arclang_ci_init", {"directory": ".", "provider": "github", "write": True, "force": True}
    )
    assert "overwrote" in forced


async def test_ci_init_writes_at_the_top_of_the_repository_and_not_above_the_workspace(
    tmp_path, binary, monkeypatch
):
    """A workspace inside a repository: the files would go above it, where the server does not write."""
    from arclang_mcp import ArcLangMCPServer

    git(tmp_path, "init", "-q", "-b", "main")
    inner = tmp_path / "models"
    inner.mkdir()
    monkeypatch.setenv("ARCLANG_BINARY", binary)
    monkeypatch.setenv("ARCLANG_WORKSPACE", str(inner))
    server = ArcLangMCPServer(inner)
    for write in (False, True):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_ci_init", {"directory": ".", "provider": "github", "write": write})
        assert "outside the workspace" in str(refused.value)
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_baseline", {"action": "create", "name": "v1"})
    assert "outside the workspace" in str(refused.value)
    assert not (tmp_path / ".github").exists() and not (tmp_path / "baselines").exists()


async def test_ci_init_refuses_what_is_not_its_own(server, repository):
    for arguments, mention in (
        ({"directory": ".", "force": True}, "goes with 'write: true'"),
        ({"directory": ".", "provider": "github", "models": ["../other.arc"]}, "from the top of the repository"),
        ({"directory": ".", "provider": "travis"}, "must be one of"),
        ({"provider": "github"}, "'directory' is required"),
        ({"directory": "no-such-directory", "provider": "github"}, "is not a directory"),
    ):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_ci_init", arguments)
        assert mention in str(refused.value), arguments
    assert not (repository / ".github").exists()


async def test_ci_run_compiles_and_gates_and_simulates_nothing_unless_asked(server, repository, model, recorded):
    await server.call_tool("arclang_ci_run", {})
    assert recorded.last[:3] == ["ci", "run", "--verify=never"]
    await server.call_tool("arclang_ci_run", {"verify": "auto", "allow_empty": True, "standard": "iec61508"})
    assert {"--verify=auto", "--allow-empty", "--standard=iec61508"} <= set(recorded.last)


async def test_ci_run_and_ci_models_on_a_repository(server, repository, model):
    models = await server.call_tool("arclang_ci_models", {})
    assert models.split("\n\n", 1)[1].split() == ["braking.arc"]
    assert json.loads((await server.call_tool("arclang_ci_models", {"as_json": True})).split("\n\n", 1)[1]) == ["braking.arc"]
    answer = await server.call_tool(
        "arclang_ci_run", {"sarif_path": "out/ci.sarif", "summary_path": "out/summary.md"}
    )
    assert answer.splitlines()[0].endswith(": PASS (compiled and gated; nothing was simulated)"), answer
    assert (repository / "out" / "ci.sarif").is_file()
    assert "**PASS**" in (repository / "out" / "summary.md").read_text()
    model.write_text(model.read_text().replace('verdict: "pass"', 'verdict: "fail"'))
    failing = await server.call_tool("arclang_ci_run", {})
    first = failing.splitlines()[0]
    assert "FAILED" in first and "PASS" not in first, failing


async def test_the_repository_commands_run_in_the_directory_they_are_given(server, workspace, model):
    """Two repositories in one workspace: each command works on the one it is sent to."""
    for name in ("one", "two"):
        (workspace / name).mkdir()
        git(workspace / name, "init", "-q", "-b", "main")
        (workspace / name / f"{name}.arc").write_text(model.read_text())
    for name in ("one", "two"):
        models = await server.call_tool("arclang_ci_models", {"directory": name})
        assert models.split("\n\n", 1)[1].split() == [f"{name}.arc"]
    # A model named from the directory is found there.
    answer = await server.call_tool("arclang_sarif", {"directory": "one", "models": ["one.arc"]})
    assert answer.splitlines()[0] == "SARIF log: succeeded" and "one.arc" in answer
    assert subprocess.run(["git", "status", "--short"], cwd=workspace / "two", capture_output=True, text=True).stdout.split() == ["??", "two.arc"]
