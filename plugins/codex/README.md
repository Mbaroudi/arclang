# ArcLang for Codex

The Codex edition of the ArcLang plugin: the same skills and agents as `plugins/claude-code`,
**generated from them** by `build.py` (one source, so the two cannot drift: `python3 plugins/codex/build.py --check`
fails when they do, and the tests run it).

| What | Where | Codex reads it as |
|---|---|---|
| MCP server | `config.toml.example` | `[mcp_servers.arclang]` in `~/.codex/config.toml`, or `codex mcp add arclang -- arclang-mcp` |
| Skills (6) | `arclang/skills/*/SKILL.md` | the same `SKILL.md` (name, description, body) |
| Plugin manifest | `arclang/plugin.json` | a plugin, offered by the marketplace `.agents/plugins/marketplace.json` |
| Subagents (13) | `agents/*.toml` | custom subagents (`name`, `description`, `developer_instructions`) |
| Project guidance | `AGENTS.md` | the project's `AGENTS.md` |

## Use

1. Install the server: `pip install -e mcp-server`, then register it: `codex mcp add arclang -- arclang-mcp`
   (or copy `config.toml.example`). The tools are not in the plugin: register the server first.
2. Skills only, without the plugin: copy `arclang/skills/*` into `~/.agents/skills/` (or `.agents/skills/` in a repo).
3. Plugin: `codex plugin marketplace add ./plugins/codex`, then install `arclang` from it
   (the Codex documentation says installing and testing a plugin is done in the desktop app).
4. Subagents: copy `agents/*.toml` into `~/.codex/agents/` (or `.codex/agents/` in a project).
   A subagent file cannot set its own MCP servers or sandbox: they come from the session.

## Status: written from the documentation, not run in Codex

Codex is not installed where this was written, so **none of it has been loaded in Codex**. What is
checked here: the files parse (JSON, TOML), the skills are the Claude Code ones byte for byte, each
subagent has the three required fields, and every `arclang_*` tool they name exists in the server.
What is not: that Codex accepts the manifest (the portable root `plugin.json` is used; the
`.codex-plugin/plugin.json` layout is the documented fallback), the marketplace path, the underscore
names of the subagents, and the MCP table. The plugin does not bundle the MCP server because the
documented plugin `mcp.json` example is for an HTTP server, not a stdio command.
The agents that name Claude Code tools by their Claude Code names have none: they name only `arclang_*` tools.

Do not edit the generated files: edit `plugins/claude-code` and run `build.py`.
