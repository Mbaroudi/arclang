"""Versions of a model: merge by element, history, restore, baselines, diff and impact by revision."""

import subprocess
from pathlib import Path

import pytest

GIT = ["git", "-c", "user.name=Test", "-c", "user.email=test@example.org", "-c", "commit.gpgsign=false"]


def git(directory: Path, *arguments: str) -> str:
    done = subprocess.run([*GIT, *arguments], cwd=directory, capture_output=True, text=True)
    assert done.returncode == 0, done.stderr
    return done.stdout


@pytest.fixture
def repository(workspace, model):
    """The workspace as a git repository, the reference model committed, then one rename left uncommitted."""
    git(workspace, "init", "-q", "-b", "main")
    # The compiler runs git itself (a baseline is a tag): the repository says who commits, as a machine with no global identity needs.
    git(workspace, "config", "user.name", "Test")
    git(workspace, "config", "user.email", "test@example.invalid")
    git(workspace, "add", "braking.arc")
    git(workspace, "commit", "-q", "-m", "The reference model")
    model.write_text(model.read_text().replace('component "ThreatAssessor"', 'component "ThreatJudge"'))
    return workspace


def versions(workspace, model):
    """A base and two versions of it that change different elements."""
    base = workspace / "base.arc"
    base.write_text(model.read_text())
    ours = workspace / "ours.arc"
    ours.write_text(base.read_text().replace('component "ThreatAssessor"', 'component "ThreatJudge"'))
    theirs = workspace / "theirs.arc"
    theirs.write_text(base.read_text().replace("bandwidth: 2 Mbps", "bandwidth: 5 Mbps"))
    return {"base_path": str(base), "ours_path": str(ours), "theirs_path": str(theirs)}


# ----------------------------------------------------------------------- merge


async def test_merge_takes_both_sides_by_element_and_writes_nothing(server, model, workspace):
    files = versions(workspace, model)
    answer = await server.call_tool("arclang_merge", files)
    assert answer.splitlines()[0].endswith(": clean; the merged model compiles"), answer
    assert 'component "ThreatJudge"' in answer and "bandwidth: 5 Mbps" in answer
    assert "Not written (no output_path)" in answer
    assert sorted(path.name for path in workspace.iterdir()) == ["base.arc", "braking.arc", "ours.arc", "theirs.arc"]


async def test_merge_writes_where_it_is_told(server, model, workspace):
    answer = await server.call_tool(
        "arclang_merge", {**versions(workspace, model), "output_path": "merged/result.arc"}
    )
    written = workspace / "merged" / "result.arc"
    assert f"Written to {written}" in answer
    text = written.read_text()
    assert 'component "ThreatJudge"' in text and "bandwidth: 5 Mbps" in text
    assert (await server.compiler.check(written)).ok


async def test_a_conflict_is_a_conflict_and_names_the_element(server, model, workspace):
    files = versions(workspace, model)
    Path(files["theirs_path"]).write_text(
        model.read_text().replace('component "ThreatAssessor"', 'component "ThreatSensor"')
    )
    answer = await server.call_tool("arclang_merge", files)
    first = answer.splitlines()[0]
    assert "1 CONFLICT(S)" in first and "clean" not in first, answer
    assert "- conflict on" in answer and "LC-001" in answer
    assert "<<<<<<<" in answer and ">>>>>>>" in answer


async def test_a_merge_that_does_not_compile_is_said_so(server, model, workspace):
    files = versions(workspace, model)
    # Ours removes the component their trace is added to: no conflict by element, no model either.
    base = Path(files["base_path"]).read_text()
    Path(files["theirs_path"]).write_text(
        base + 'trace "PN-001" satisfies "REQ-BRK-002" { rationale: "added" }\n'
    )
    answer = await server.call_tool("arclang_merge", files)
    first = answer.splitlines()[0]
    assert "DOES NOT COMPILE" in first, answer
    answer = await server.call_tool("arclang_merge", {**files, "compile": False})
    assert answer.splitlines()[0].endswith(": clean (not compiled: compile: false)"), answer


async def test_the_first_name_of_the_merge_still_merges(server, model, workspace):
    answer = await server.call_tool("arclang_git_merge", versions(workspace, model))
    assert answer.splitlines()[0].endswith(": clean; the merged model compiles"), answer
    assert "not available" not in answer.lower()


async def test_a_merge_of_files_that_are_not_there_is_a_failure(server):
    answer = await server.call_tool(
        "arclang_merge", {"base_path": "a.arc", "ours_path": "b.arc", "theirs_path": "c.arc"}
    )
    assert "FAILED" in answer.splitlines()[0] and "No such file" in answer


# ----------------------------------------------------- history, restore, revisions


async def test_history_lists_the_commits_that_changed_the_element(server, repository):
    answer = await server.call_tool("arclang_history", {"model_path": "braking.arc", "element": "LC-001"})
    assert answer.splitlines()[0] == "History of 'LC-001' in braking.arc: succeeded", answer
    assert "The reference model" in answer and "added" in answer
    as_json = await server.call_tool(
        "arclang_history", {"model_path": "braking.arc", "element": "LC-001", "as_json": True, "limit": 1}
    )
    assert '"entries"' in as_json


async def test_history_outside_a_repository_is_a_failure(server, model):
    answer = await server.call_tool("arclang_history", {"model_path": str(model), "element": "LC-001"})
    assert "FAILED" in answer.splitlines()[0] and "not in a git repository" in answer


async def test_restore_is_a_diff_until_it_is_told_where_to_write(server, repository, model):
    now = model.read_text()
    answer = await server.call_tool(
        "arclang_restore", {"model_path": "braking.arc", "rev": "HEAD", "elements": ["LC-001"]}
    )
    assert "accepted, NOT written (give output_path to write it)" in answer.splitlines()[0], answer
    assert '-        component "ThreatJudge" {' in answer and '+        component "ThreatAssessor" {' in answer
    assert model.read_text() == now
    answer = await server.call_tool(
        "arclang_restore", {"model_path": "braking.arc", "rev": "HEAD", "output_path": "braking.arc"}
    )
    assert "written to" in answer.splitlines()[0], answer
    assert 'component "ThreatAssessor"' in model.read_text()


async def test_restore_of_an_unknown_revision_is_a_failure(server, repository):
    answer = await server.call_tool("arclang_restore", {"model_path": "braking.arc", "rev": "no-such-rev"})
    assert "FAILED" in answer.splitlines()[0], answer


async def test_diff_between_revisions_says_what_changed_and_its_consequences(server, repository):
    answer = await server.call_tool("arclang_diff", {"model_path": "braking.arc", "from_rev": "HEAD"})
    assert answer.splitlines()[0] == (
        "Semantic diff of braking.arc: HEAD -> the working tree: the versions differ"
    ), answer
    assert '"renamed": 1' in answer and '"from": "HEAD"' in answer
    as_text = await server.call_tool(
        "arclang_diff", {"model_path": "braking.arc", "from_rev": "HEAD", "format": "md", "output_path": "diff.md"}
    )
    assert "Written to" in as_text and "ThreatJudge" in (repository / "diff.md").read_text()
    same = await server.call_tool(
        "arclang_diff", {"model_path": "braking.arc", "from_rev": "HEAD", "to_rev": "HEAD"}
    )
    assert same.splitlines()[0].endswith(": no semantic change"), same


async def test_diff_of_two_files_takes_the_rich_report_too(server, model, workspace):
    other = workspace / "other.arc"
    other.write_text(model.read_text().replace('component "ThreatAssessor"', 'component "ThreatJudge"'))
    answer = await server.call_tool(
        "arclang_diff", {"old_path": str(model), "new_path": str(other), "format": "md"}
    )
    assert answer.splitlines()[0].endswith(": the versions differ"), answer
    assert "# Semantic diff" in answer and "renamed" in answer


@pytest.mark.parametrize(
    "arguments",
    [
        {},
        {"old_path": "a.arc"},
        {"model_path": "braking.arc"},
        {"old_path": "a.arc", "new_path": "b.arc", "from_rev": "HEAD"},
        {"from_rev": "HEAD"},
    ],
)
async def test_diff_wants_its_two_versions_named_one_way(server, model, arguments):
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_diff", arguments)
    assert "old_path" in str(refused.value) or "model_path" in str(refused.value)


async def test_a_diff_at_a_revision_that_does_not_exist_is_a_failure_not_a_difference(server, repository):
    answer = await server.call_tool("arclang_diff", {"model_path": "braking.arc", "from_rev": "no-such-rev"})
    first = answer.splitlines()[0]
    assert "FAILED" in first and "differ" not in first, answer


async def test_the_workbench_reads_the_model_as_committed(server, repository):
    now = await server.call_tool("arclang_workbench", {"path": "braking.arc", "element": "LC-001"})
    then = await server.call_tool("arclang_workbench", {"path": "braking.arc", "element": "LC-001", "rev": "HEAD"})
    assert "ThreatJudge" in now and "ThreatAssessor" not in now.split("Findings")[0].split("1 element")[1]
    assert then.startswith("Workbench of braking.arc at HEAD: compiles")
    assert "LC-001 | LogicalComponent | ThreatAssessor" in then


async def test_impact_of_the_changes_since_a_revision(server, repository):
    answer = await server.call_tool("arclang_impact", {"model_path": "braking.arc", "since": "HEAD"})
    assert answer.splitlines()[0] == "Impact of the change in braking.arc: succeeded", answer
    assert "the changes since HEAD" in answer and "LC-002" in answer


# ------------------------------------------------------------------- baselines


async def test_baselines_are_listed_made_verified_and_compared(server, repository, model):
    listed = await server.call_tool("arclang_baseline", {"action": "list"})
    assert "No baseline" in listed
    # A tree with an uncommitted change is refused: a baseline is of a commit.
    dirty = await server.call_tool("arclang_baseline", {"action": "create", "name": "v1"})
    assert "FAILED" in dirty.splitlines()[0], dirty
    assert git(repository, "tag") == ""
    git(repository, "commit", "-q", "-am", "Rename the assessor")
    made = await server.call_tool("arclang_baseline", {"action": "create", "name": "v1", "require_pass": True})
    assert made.splitlines()[0] == (
        "Baseline 'v1': created (a directory under baselines/ and a tag, not pushed)"
    ), made
    assert (repository / "baselines" / "v1" / "gate_report.json").is_file()
    assert git(repository, "tag").split() == ["baseline/v1"]
    git(repository, "add", "baselines")
    git(repository, "commit", "-q", "-m", "Baseline v1")
    verified = await server.call_tool("arclang_baseline", {"action": "verify", "name": "v1"})
    assert verified.splitlines()[0].endswith(": matches its manifest and its tag"), verified
    as_json = await server.call_tool("arclang_baseline", {"action": "list", "as_json": True})
    assert '"v1"' in as_json

    model.write_text(model.read_text().replace("bandwidth: 2 Mbps", "bandwidth: 5 Mbps"))
    git(repository, "commit", "-q", "-am", "A faster bus")
    await server.call_tool("arclang_baseline", {"action": "create", "name": "v2"})
    compared = await server.call_tool("arclang_baseline", {"action": "diff", "a": "v1", "b": "v2", "format": "md"})
    # Like `diff`: exit 1 when the baselines differ, and the first line says so.
    assert compared.splitlines()[0] == "Baseline diff v1 -> v2: the baselines differ", compared
    assert 'bandwidth: "2 Mbps" -> "5 Mbps"' in compared

    # A file of the baseline changed by hand no longer matches its manifest.
    report = repository / "baselines" / "v1" / "gate_report.md"
    report.write_text(report.read_text() + "\nedited\n")
    broken = await server.call_tool("arclang_baseline", {"action": "verify", "name": "v1"})
    assert "DOES NOT MATCH" in broken.splitlines()[0], broken


@pytest.mark.parametrize(
    "arguments, mention",
    [
        ({"action": "create"}, "needs 'name'"),
        ({"action": "diff", "a": "v1"}, "needs 'b'"),
        ({"action": "list", "name": "v1"}, "mean(s) nothing"),
        ({"action": "verify", "name": "v1", "allow_dirty": True}, "mean(s) nothing"),
        ({"action": "delete", "name": "v1"}, "must be one of"),
    ],
)
async def test_a_baseline_action_takes_its_own_arguments(server, repository, arguments, mention):
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_baseline", arguments)
    assert mention in str(refused.value)
    assert git(repository, "tag") == ""
