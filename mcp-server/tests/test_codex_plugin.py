"""The Codex edition (plugins/codex) is generated from the Claude Code plugin and stays in step with it.

Nothing here runs Codex: it checks that the files parse, that the generator and the committed output
agree, that each subagent has the fields Codex documents as required, and that every `arclang_*` tool
the text names is a tool the server lists.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

import pytest

from arclang_mcp import ArcLangMCPServer

CODEX = Path(__file__).resolve().parents[2] / "plugins" / "codex"
CLAUDE = CODEX.parent / "claude-code"
AGENTS = sorted(CODEX.glob("agents/*.toml"))
tomllib = pytest.importorskip("tomllib", reason="TOML parsing needs Python 3.11")


def test_the_generated_files_agree_with_the_claude_code_plugin():
    done = subprocess.run([sys.executable, str(CODEX / "build.py"), "--check"], capture_output=True, text=True)
    assert done.returncode == 0, done.stdout + done.stderr


def test_the_manifest_and_the_marketplace_are_json():
    manifest = json.loads((CODEX / "arclang" / "plugin.json").read_text())
    assert manifest["name"] == "arclang" and manifest["version"]
    entry = json.loads((CODEX / ".agents" / "plugins" / "marketplace.json").read_text())["plugins"][0]
    assert entry["name"] == "arclang"
    assert entry["source"]["path"].startswith("./") and (CODEX / entry["source"]["path"]).is_dir()
    assert {"installation", "authentication"} <= set(entry["policy"]) and entry["category"]


def test_every_skill_and_agent_of_the_claude_plugin_has_a_codex_twin():
    assert sorted(p.parent.name for p in CODEX.glob("arclang/skills/*/SKILL.md")) == sorted(p.parent.name for p in CLAUDE.glob("skills/*/SKILL.md"))
    assert len(AGENTS) == len(list(CLAUDE.glob("agents/*.md"))) >= 13


@pytest.mark.parametrize("path", AGENTS, ids=lambda p: p.name)
def test_a_subagent_has_the_three_required_fields(path: Path):
    agent = tomllib.loads(path.read_text())
    assert agent["name"] == path.stem and "-" not in agent["name"]
    assert len(agent["description"]) > 40 and len(agent["developer_instructions"]) > 100
    # What Codex says a subagent file cannot override must not be written there.
    assert not {"sandbox_mode", "approval_policy", "model_provider", "mcp_servers"} & set(agent)


def test_the_plugin_starts_the_server():
    servers = json.loads((CODEX / "arclang" / ".mcp.json").read_text())["mcpServers"]
    assert servers["arclang"]["command"] == "arclang-mcp"


def test_the_mcp_example_is_a_valid_table():
    table = tomllib.loads((CODEX / "config.toml.example").read_text())["mcp_servers"]["arclang"]
    assert table["command"] == "arclang-mcp" and table["tool_timeout_sec"] >= 900


def test_every_tool_named_exists_in_the_server(workspace: Path):
    listed = {tool.name for tool in ArcLangMCPServer(workspace).tools}
    named = set()
    for path in [*CODEX.rglob("*.toml"), *CODEX.rglob("*.md")]:
        named |= set(re.findall(r"\barclang_[a-z_]+\b", path.read_text()))
    unknown = sorted(name for name in named - listed if name != "arclang_mcp")
    assert named and not unknown, f"named but not offered by the server: {unknown}"
