"""The server reads and writes under the workspace, and where it was told to.

An agent passes the paths, and what an agent read (a model, a document, a web
page) can ask it for any path: a tool that writes must not be led to a file
outside the workspace. The directories named in ARCLANG_MCP_ROOTS are the
other places the server works in; `*` there is any path.
"""

import os
import shutil
from pathlib import Path

import pytest

from arclang_mcp.resources.syntax_guide import RESOURCES
from arclang_mcp.tools.common import ROOTS, InvalidArguments, resolve_path


@pytest.fixture
def outside(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """A directory beside the workspace, with a model in it."""
    directory = tmp_path_factory.mktemp("outside")
    shutil.copy(RESOURCES / "reference_model.arc", directory / "braking.arc")
    return directory


async def refused(server, tool: str, arguments: dict) -> str:
    with pytest.raises(InvalidArguments) as refusal:
        await server.call_tool(tool, arguments)
    return str(refusal.value)


async def test_a_model_outside_the_workspace_is_not_read(server, outside):
    said = await refused(server, "arclang_validate", {"model_path": str(outside / "braking.arc")})
    assert "is outside the workspace" in said
    assert ROOTS in said, "the refusal says how to allow it"


async def test_nothing_is_written_outside_the_workspace(server, model, outside):
    target = outside / "compiled.json"
    await refused(server, "arclang_compile", {"model_path": str(model), "output_path": str(target)})
    assert not target.exists()

    before = (outside / "braking.arc").read_text()
    await refused(
        server,
        "arclang_set_attribute",
        {
            "model_path": str(outside / "braking.arc"),
            "element": "LC-002",
            "key": "description",
            "value": '"x"',
            "write": True,
        },
    )
    assert (outside / "braking.arc").read_text() == before


async def test_a_relative_path_does_not_climb_out(server, model, workspace, outside):
    climbing = os.path.relpath(outside / "written.arc", workspace)
    assert climbing.startswith("..")
    await refused(
        server,
        "arclang_import",
        {"input_path": str(model), "format": "mermaid", "output_path": climbing},
    )
    assert not (outside / "written.arc").exists()


async def test_a_link_in_the_workspace_does_not_lead_out(server, workspace, outside):
    (workspace / "elsewhere").symlink_to(outside, target_is_directory=True)
    said = await refused(server, "arclang_validate", {"model_path": "elsewhere/braking.arc"})
    assert "is outside the workspace" in said


async def test_the_home_directory_is_not_the_workspace(server):
    await refused(server, "arclang_validate", {"model_path": "~/braking.arc"})


def test_a_directory_that_only_begins_like_the_workspace_is_outside(workspace):
    beside = workspace.parent / f"{workspace.name}-other"
    with pytest.raises(InvalidArguments):
        resolve_path(workspace, str(beside / "braking.arc"))


async def test_a_named_directory_is_worked_in(server, workspace, outside, tmp_path_factory, monkeypatch):
    other = tmp_path_factory.mktemp("other")
    monkeypatch.setenv(ROOTS, os.pathsep.join([str(other), str(outside)]))
    answer = await server.call_tool("arclang_validate", {"model_path": str(outside / "braking.arc")})
    assert "compiles" in answer.splitlines()[0]
    # What is named is that directory, not the ones beside it.
    with pytest.raises(InvalidArguments):
        resolve_path(workspace, str(outside.parent / "braking.arc"))


async def test_a_star_is_any_path(server, outside, monkeypatch):
    monkeypatch.setenv(ROOTS, "*")
    answer = await server.call_tool("arclang_validate", {"model_path": str(outside / "braking.arc")})
    assert "compiles" in answer.splitlines()[0]


def test_a_path_in_the_workspace_is_returned_as_it_was_resolved_before(workspace):
    assert resolve_path(workspace, "models/a.arc") == workspace.resolve() / "models" / "a.arc"
    assert resolve_path(workspace, str(workspace / "a.arc")) == workspace / "a.arc"
    assert resolve_path(workspace, "sub/../a.arc") == workspace.resolve() / "sub" / ".." / "a.arc"
    # The workspace itself, for the tools that take a directory.
    assert resolve_path(workspace, ".") == workspace.resolve() / "."
