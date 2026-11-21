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

1. Install the server: `pip install -e mcp-server` (it provides `arclang-mcp`).
2. Plugin: `codex plugin marketplace add ./plugins/codex`, then `codex plugin add arclang@arclang`.
   It brings the six skills and starts the MCP server (`arclang/.mcp.json`).
3. Subagents: copy `agents/*.toml` into `~/.codex/agents/` (or `.codex/agents/` in a project).
   A subagent file cannot set its own MCP servers or sandbox: they come from the session.
4. Without the plugin: `codex mcp add arclang -- arclang-mcp` (or `config.toml.example`) for the server,
   and `arclang/skills/*` copied into `~/.agents/skills/` for the skills.

The server reads `ARCLANG_BINARY` (the compiler), `ARCLANG_SERVER` (`arclang-server` of the Enterprise
edition, for `arclang_engineering`) and `ARCLANG_WORKSPACE`; `calibrate` also needs `ARCLANG_BIN` set to a
compiler that has `arclang twin`.

## What was checked, with Codex 0.171.0

Run on a Mac with a throwaway `CODEX_HOME`, so nothing of a user's configuration was touched:

- `codex plugin marketplace add` and `codex plugin add arclang@arclang` accept the marketplace and the
  root `plugin.json`; the six skills are installed, and `codex debug prompt-input` shows them as
  `arclang:engineering-*` and shows the 13 subagents with their descriptions.
- Codex starts the MCP server from the plugin's `.mcp.json` (listed under `pluginId arclang@arclang`)
  and from `[mcp_servers.arclang]`; with the branch's server it lists 65 tools, `arclang_engineering`
  with its 12 actions among them.
- `mcpServer/tool/call` of `arclang_engineering` through Codex reached the real `arclang-server`:
  `connectors` listed the four engines, `model` of the 16-point optical transform gave `every limit holds`.

- A real model session (`codex exec`, Codex logged in) was asked to judge `fft16.arc`: it read the skills
  `engineering-verdicts` and `engineering-photonics`, called `arclang_engineering` on the real
  `arclang-server`, quoted the first line (`every limit holds`) and kept the claim to "for this simulation",
  then delegated a check to the `verdict_auditor` subagent, whose conclusion said the same and added that
  the physical chip's performance is not established.

Two things learned there:

- In `codex exec` (no one to approve), an MCP tool call is refused with "requires approval, but approval
  policy is never". Allow the tools of the server explicitly:
  `default_tools_approval_mode = "approve"` under `[mcp_servers.arclang]`. Interactively, Codex asks.
- Environment variables given under `[plugins."arclang@arclang".mcp_servers.arclang.env]` did not reach the
  plugin's server in the test (the server that ran was the installed one). Use the variables of your shell, or
  register the server with `[mcp_servers.arclang.env]` instead of the plugin's own.

Not checked: the Codex desktop application.

Do not edit the generated files: edit `plugins/claude-code` and run `build.py`.
