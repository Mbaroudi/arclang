"""
The server against the command line: which subcommand of `arclang` each tool
runs, and what of the command line is left out on purpose.

The rule this file holds the server to: every capability of the compiler is
reachable through a tool. `tests/test_cli_parity.py` reads the CLI's own
`--help` and fails when

- a subcommand is neither run by a tool (`COVERS`) nor left out here with
  its reason (`NOT_EXPOSED`);
- a long option of a covered subcommand is neither an argument of a tool
  that runs it (same name, `-` written `_`), nor declared in `OPTIONS` as
  reached under another name (`Via`), passed by the server itself
  (`ByServer`), or left out with its reason (`Never`).

So a subcommand or an option the CLI gains fails a test until it is offered
or named here. Nothing else needs upkeep: the lists of formats, views,
kinds and operations are checked for equality with the CLI in the same test
file, from the lists of `compiler/wrapper.py`.
"""

from dataclasses import dataclass
from typing import Dict, Tuple, Union

# Tool -> the subcommands it runs. A nested subcommand is written with its
# parent ("baseline create"). A tool that runs none of them is not listed.
COVERS: Dict[str, Tuple[str, ...]] = {
    # core.py
    "arclang_compile": ("build",),
    "arclang_validate": ("check",),
    "arclang_rules": ("rules",),
    "arclang_trace_analysis": ("trace",),
    "arclang_info": ("info",),
    "arclang_gate": ("gate",),
    "arclang_realize": ("realize",),
    "arclang_partition": ("partition", "table"),
    "arclang_cesam": ("table", "check"),
    "arclang_schedule": ("schedule",),
    "arclang_diff": ("diff",),
    "arclang_impact": ("impact",),
    "arclang_export": ("export",),
    "arclang_export_diagram": ("export",),
    "arclang_metamodel": ("metamodel",),
    "arclang_review": ("review",),
    # edit.py
    "arclang_format": ("format",),
    "arclang_set_attribute": ("set",),
    "arclang_unset_attribute": ("unset",),
    "arclang_rename": ("rename",),
    # workbench.py
    "arclang_workbench": ("workbench",),
    "arclang_edit": ("edit",),
    # transform.py
    "arclang_import": ("import",),
    "arclang_transition": ("transition",),
    # generation.py
    "arclang_viewpoint_diagrams": ("diagram",),
    "arclang_generate_diagram": ("diagram",),
    "arclang_generate_all_diagrams": ("diagram",),
    # safety.py
    "arclang_safety_check": ("safety",),
    "arclang_hazard_analysis": ("export",),
    # recover.py
    "arclang_prove": ("prove",),
    "arclang_recover": ("recover",),
    "arclang_drift": ("drift",),
    # simulation.py
    "arclang_verify": ("verify",),
    "arclang_cosim": ("cosim",),
    "arclang_sil": ("sil",),
    "arclang_pil": ("pil",),
    "arclang_hdl": ("hdl",),
    "arclang_soc": ("soc",),
    "arclang_hil": ("hil",),
    "arclang_twin": ("twin",),
    "arclang_observe": ("observe",),
    "arclang_debug": ("debug",),
    # versioning.py
    "arclang_merge": ("merge",),
    "arclang_git_merge": ("merge",),
    "arclang_history": ("history",),
    "arclang_restore": ("restore",),
    "arclang_baseline": ("baseline create", "baseline list", "baseline diff", "baseline verify"),
    # delivery.py
    "arclang_table": ("table",),
    "arclang_document": ("document",),
    "arclang_new": ("new",),
    "arclang_report": ("report",),
    "arclang_sarif": ("sarif",),
    "arclang_ci_init": ("ci init",),
    "arclang_ci_run": ("ci run",),
    "arclang_ci_models": ("ci models",),
}

# Tools that run no subcommand of their own, each with what they are instead.
WITHOUT_SUBCOMMAND: Dict[str, str] = {
    "arclang_generate_requirement": "writes a snippet from a description, then compiles it (check)",
    "arclang_generate_component": "writes a snippet from a description, then compiles it (check)",
    "arclang_suggest_architecture": "asks a Claude model; runs no command",
    "arclang_analyze_requirements": "sorts sentences by their wording; runs no command",
    "arclang_generate_from_requirements": "writes a skeleton from that analysis, then compiles it (check)",
    "arclang_assess_diagram_quality": "string checks on an SVG; runs no command",
    "arclang_suggest_diagram_enhancements": "names enhancements from those checks; runs no command",
    "arclang_generate_mbse_report": "a Markdown summary of an analysis; runs no command",
    "arclang_execute_batch": "several of the operations above in one call, each through its own command",
}

# Subcommands no tool runs, each with its reason. Anything interactive, anything
# that stays running, anything that opens a browser, and what the compiler does
# not implement.
NOT_EXPOSED: Dict[str, str] = {
    "serve": "a server that stays running (the modeling API over HTTP): a tool call "
    "answers and ends; start it yourself, it is another way in, not a capability missing here",
    "lsp": "the language server of an editor: a conversation on the standard input",
    "repl": "an interactive prompt",
    "explorer": "writes a page for a person to browse and can open a browser; an agent "
    "reads the same model through arclang_workbench and draws it with arclang_viewpoint_diagrams",
    "clean": "the CLI answers 'Not implemented'",
    "sync": "the CLI answers 'Not implemented' (there is no PLM connector)",
    "plugin": "the CLI answers 'Not implemented' (there is no plugin system)",
    "help": "clap's own: the help of a subcommand, which the tool descriptions carry",
}


@dataclass(frozen=True)
class Via:
    """The option is reached through this argument of a tool that runs the subcommand."""

    argument: str


@dataclass(frozen=True)
class ByServer:
    """The server passes the option itself, always or from the other arguments."""

    why: str


@dataclass(frozen=True)
class Never:
    """The option is deliberately not offered."""

    why: str


Reach = Union[Via, ByServer, Never]

# Options every subcommand has, never offered.
GLOBAL_OPTIONS: Dict[str, Never] = {
    "--help": Never("the tool descriptions carry it"),
    "--verbose": Never("adds the compiler's progress lines to an answer read by a program"),
    "--as": Never(
        "says which file an editor's unsaved buffer stands for; the server reads files, "
        "it has no buffer"
    ),
}

OUTPUT = Via("output_path")
JSON = Via("as_json")
JSON_ALWAYS = ByServer("always passed: the first line of the answer is read from the JSON")
KEEP = Via("keep_dir")
OPENS_A_BROWSER = Never("opens a browser on the user's screen")

# Subcommand -> its long options that are not an argument of the same name.
OPTIONS: Dict[str, Dict[str, Reach]] = {
    "build": {"--output": OUTPUT, "--release": Via("optimize")},
    "check": {
        "--lint": ByServer("always passed: a check without the advisories says less"),
        "--json": JSON,
    },
    "transition": {},
    "rules": {},
    "format": {
        "--check": ByServer("passed for a directory without `write`: which files are not formatted"),
    },
    "new": {"--output": OUTPUT},
    "trace": {
        "--validate": ByServer("passed unless `suspects` or `review` is asked"),
        "--print": ByServer("passed with `review` unless `write`: the answer is then a diff"),
    },
    "gate": {},
    "diff": {
        "--json": ByServer("passed for two files when no `format` is asked"),
        "--from": Via("from_rev"),
        "--to": Via("to_rev"),
        "--output": OUTPUT,
    },
    "history": {"--json": JSON},
    "merge": {
        "--output": OUTPUT,
        "--path": Via("repository_path"),
        "--json": JSON_ALWAYS,
        "--no-compile": Via("compile"),
        "--install": Never(
            "sets git up on the user's machine to use this merge (.git/config, "
            ".gitattributes), after a question on the terminal: a one-time setup for a person"
        ),
        "--yes": Never("goes with --install"),
        "--dry-run": Never("goes with --install"),
        "--binary": Never("goes with --install"),
    },
    "restore": {"--element": Via("elements"), "--output": OUTPUT},
    "baseline create": {"--model": Via("model_path")},
    "baseline list": {"--json": JSON},
    "baseline diff": {},
    "baseline verify": {"--json": JSON},
    "ci init": {
        "--model": Via("models"),
        "--dry-run": ByServer("passed unless `write: true`"),
    },
    "ci run": {"--sarif": Via("sarif_path"), "--summary": Via("summary_path")},
    "ci models": {"--json": JSON},
    "report": {
        "--pr": ByServer("always passed: it is the only report there is"),
        "--from": Via("from_rev"),
        "--to": Via("to_rev"),
        "--output": OUTPUT,
    },
    "sarif": {"--output": OUTPUT},
    "impact": {"--element": Via("elements"), "--current": Via("current_path"), "--output": OUTPUT},
    "debug": {
        "--set": Via("conditions"),
        "--json": JSON,
        "--dap": Never(
            "the Debug Adapter Protocol of VS Code: a conversation on the standard input, "
            "which a tool call does not have"
        ),
    },
    "verify": {"--json": JSON},
    "cosim": {"--keep": KEEP, "--json": JSON},
    "hil": {"--timeout": Via("row_timeout"), "--json": JSON},
    "sil": {"--keep": KEEP, "--json": JSON},
    "pil": {"--keep": KEEP, "--json": JSON},
    "hdl": {"--keep": KEEP, "--json": JSON},
    "soc": {"--keep": KEEP, "--json": JSON},
    "partition": {
        "--clock": Via("clock_mhz"),
        "--dry-run": ByServer("passed unless `run: true`"),
        "--json": JSON,
    },
    "schedule": {"--keep": KEEP, "--json": JSON_ALWAYS},
    "recover": {
        "--output": OUTPUT,
        "--hdl": Via("hdl_path"),
        "--keep": KEEP,
        "--model": Via("model_path"),
        "--view": Via("view_path"),
        "--open": OPENS_A_BROWSER,
    },
    "observe": {"--recorded": Via("recorded_path"), "--keep": Via("keep_path"), "--json": JSON},
    "prove": {"--keep": KEEP, "--json": JSON},
    "drift": {"--hdl": Via("hdl_path"), "--json": JSON},
    "realize": {
        "--dry-run": ByServer("passed unless `run: true`"),
        "--json": JSON_ALWAYS,
    },
    "twin": {"--json": JSON, "--for": Via("for_seconds")},
    "document": {"--template": Via("template_path"), "--output": OUTPUT},
    "export": {"--output": OUTPUT, "--element": Via("elements")},
    "import": {"--output": OUTPUT},
    "safety": {"--output": OUTPUT},
    "info": {
        "--metrics": ByServer("always passed: the counts are what the tool is for"),
        "--dependencies": Never("the CLI answers 'Not implemented'"),
    },
    "set": {},
    "review": {
        "--json": JSON_ALWAYS,
        "--dry-run": ByServer("passed unless `send: true`: nothing leaves the machine by default"),
    },
    "rename": {},
    "unset": {},
    "workbench": {},
    "edit": {
        "--request": Via("requests"),
        "--write": ByServer(
            "never passed: the requests run on a private copy and the server writes the "
            "file once, with the final text, when `write: true`"
        ),
    },
    "table": {"--output": OUTPUT},
    "metamodel": {},
    "diagram": {
        "--output": OUTPUT,
        "--format": Via("diagram_type"),
        "--layout": Via("layout_path"),
        "--open": OPENS_A_BROWSER,
    },
}


def tools_running(subcommand: str) -> Tuple[str, ...]:
    """The tools that run a subcommand."""
    return tuple(tool for tool, commands in COVERS.items() if subcommand in commands)


def covered_subcommands() -> Tuple[str, ...]:
    """Every subcommand a tool runs, in the order of the table."""
    return tuple(dict.fromkeys(command for commands in COVERS.values() for command in commands))
