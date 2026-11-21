#!/usr/bin/env python3
"""Builds the Codex edition of the ArcLang plugin from the Claude Code one: one source, two outputs.

    python3 plugins/codex/build.py          write plugins/codex/{arclang,agents}
    python3 plugins/codex/build.py --check  fail if what is written differs from what the source gives

The skills are the same `SKILL.md` (a name, a description, a body) and are copied as they are.
A Claude Code agent is a Markdown file; a Codex subagent is a TOML file with `name`,
`description` and `developer_instructions`, so each agent is converted. Nothing else is invented.
Only the standard library is used.
"""

import json
import re
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = HERE.parent / "claude-code"
FRONT = re.compile(r"\A---\nname: (?P<name>[^\n]+)\ndescription: (?P<description>[^\n]+)\n---\n(?P<body>.*)\Z", re.S)

MANIFEST = {
    "name": "arclang",
    "version": "0.1.0",
    "description": "ArcLang for Codex: the MCP server (arclang-mcp) and the skills for engineering verdicts (electronics, mechanics, chip, photonics).",
    "author": {"name": "Malek Baroudi"},
    "homepage": "https://arclang.fr",
    "repository": "https://github.com/Mbaroudi/arclang",
    "license": "BUSL-1.1",
}
# Codex starts this server with the plugin (checked with Codex 0.171.0: the server is listed under the plugin).
MCP = {"mcpServers": {"arclang": {"command": "arclang-mcp"}}}
MARKETPLACE = {
    "name": "arclang",
    "interface": {"displayName": "ArcLang"},
    "plugins": [
        {
            "name": "arclang",
            "source": {"source": "local", "path": "./arclang"},
            "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
            "category": "Engineering",
        }
    ],
}


def toml_text(value: str) -> str:
    """A TOML basic string, escaped."""
    return json.dumps(value, ensure_ascii=False)


def agent_toml(path: Path) -> str:
    match = FRONT.match(path.read_text())
    if not match:
        raise SystemExit(f"{path}: front matter must be name then description")
    body = match["body"].strip("\n") + "\n"
    if "'''" in body:
        raise SystemExit(f"{path}: a ''' cannot go in a TOML literal string")
    # The name is a word with underscores, as Codex's own examples write them.
    return (
        f"name = {toml_text(match['name'].replace('-', '_'))}\n"
        f"description = {toml_text(match['description'])}\n"
        f"developer_instructions = '''\n{body}'''\n"
    )


def expected() -> dict:
    """Every file to write: relative path -> text."""
    files = {
        "arclang/plugin.json": json.dumps(MANIFEST, indent=2, ensure_ascii=False) + "\n",
        "arclang/.mcp.json": json.dumps(MCP, indent=2) + "\n",
        ".agents/plugins/marketplace.json": json.dumps(MARKETPLACE, indent=2) + "\n",
    }
    for skill in sorted(SOURCE.glob("skills/*/SKILL.md")):
        files[f"arclang/skills/{skill.parent.name}/SKILL.md"] = skill.read_text()
    for agent in sorted(SOURCE.glob("agents/*.md")):
        files[f"agents/{agent.stem.replace('-', '_')}.toml"] = agent_toml(agent)
    return files


GENERATED = ("arclang", "agents", ".agents")


def main() -> int:
    files = expected()
    if "--check" in sys.argv:
        found = {str(p.relative_to(HERE)) for top in GENERATED for p in (HERE / top).rglob("*") if p.is_file()}
        stale = sorted(found ^ set(files))
        stale += [name for name, text in files.items() if (HERE / name).is_file() and (HERE / name).read_text() != text]
        if stale:
            print("out of date with plugins/claude-code (run: python3 plugins/codex/build.py):", *sorted(set(stale)), sep="\n  ")
            return 1
        print(f"{len(files)} files agree with plugins/claude-code")
        return 0
    for top in GENERATED:
        shutil.rmtree(HERE / top, ignore_errors=True)
    for name, text in files.items():
        target = HERE / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)
    print(f"{len(files)} files written under {HERE}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
