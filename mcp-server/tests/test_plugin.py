"""The Claude Code plugin (plugins/claude-code) stays coherent with the server it registers.

Skills and agents are Markdown with a front matter; a typo in one is silent. Here each one must
have a name equal to its file or folder, a description, and every `arclang_*` tool it names must
be a tool the server really lists.
"""

import json
import re
from pathlib import Path

import pytest

from arclang_mcp import ArcLangMCPServer

PLUGIN = Path(__file__).resolve().parents[2] / "plugins" / "claude-code"
FRONT = re.compile(r"\A---\nname: (?P<name>[^\n]+)\ndescription: (?P<description>[^\n]+)\n---\n", re.S)
FILES = sorted([*PLUGIN.glob("skills/*/SKILL.md"), *PLUGIN.glob("agents/*.md")])


def test_the_manifest_and_the_server_registration_are_json():
    manifest = json.loads((PLUGIN / ".claude-plugin" / "plugin.json").read_text())
    assert manifest["name"] == "arclang"
    servers = json.loads((PLUGIN / ".mcp.json").read_text())["mcpServers"]
    assert servers["arclang"]["command"] == "arclang-mcp"


def test_there_are_skills_and_agents():
    assert len(list(PLUGIN.glob("skills/*/SKILL.md"))) >= 6
    assert len(list(PLUGIN.glob("agents/*.md"))) >= 13


@pytest.mark.parametrize("path", FILES, ids=lambda p: str(p.relative_to(PLUGIN)))
def test_the_front_matter_names_the_file(path: Path):
    match = FRONT.match(path.read_text())
    assert match, f"{path}: front matter must be name then description"
    expected = path.parent.name if path.name == "SKILL.md" else path.stem
    assert match["name"] == expected
    assert len(match["description"]) > 40


def test_every_tool_a_skill_or_an_agent_names_exists(workspace: Path):
    listed = {tool.name for tool in ArcLangMCPServer(workspace).tools}
    named = {name for path in FILES for name in re.findall(r"\barclang_[a-z_]+\b", path.read_text())}
    # `arclang_mcp` is the Python package, not a tool.
    unknown = sorted(name for name in named - listed if name != "arclang_mcp")
    assert not unknown, f"named but not offered by the server: {unknown}"
