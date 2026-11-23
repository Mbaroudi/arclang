"""The server is held to the command line: what `arclang` can do, a tool reaches.

Everything here is read from the binary's own `--help`, so nothing needs upkeep
when the CLI grows: a new subcommand, a new option, a new format, view, table
kind or edit operation fails a test until a tool offers it or `tools/cli.py`
(or the list in `compiler/wrapper.py`) leaves it out by name, with its reason.
"""

import json
import re
from typing import Dict, List, Set

import pytest

from arclang_mcp.compiler import wrapper
from arclang_mcp.tools import cli
from arclang_mcp.tools.common import input_schema

OPTION_LINE = re.compile(r"^ {2,6}(?:-\w, )?(--[a-z0-9][a-z0-9-]*)", re.M)


async def help_of(server, *command: str) -> str:
    run = await server.compiler.run([*command, "--help"])
    assert run.ok, f"arclang {' '.join(command)} --help: {run.output}"
    return run.stdout


def section(help_text: str, title: str) -> str:
    """The lines of one section of a help text (`Commands:`, `Options:`)."""
    found = re.search(rf"^{title}:\n(.*?)(?=^\S|\Z)", help_text, flags=re.M | re.S)
    return found.group(1) if found else ""


def subcommands_of(help_text: str) -> List[str]:
    return re.findall(r"^  ([a-z][a-z0-9-]*)", section(help_text, "Commands"), flags=re.M)


def options_of(help_text: str) -> Dict[str, str]:
    """Each long option of a help text, with everything the help says of it."""
    options = section(help_text, "Options")
    starts = [(found.start(), found.group(1)) for found in OPTION_LINE.finditer(options)]
    ends = [start for start, _ in starts[1:]] + [len(options)]
    return {name: options[start:end] for (start, name), end in zip(starts, ends)}


def values_of(said: str) -> List[str]:
    """The possible values the help of one option lists, in either of clap's layouts."""
    inline = re.search(r"\[possible values: ([^\]]+)\]", said)
    if inline:
        return [value.strip() for value in inline.group(1).split(",")]
    return re.findall(r"^\s+- ([A-Za-z0-9_-]+)", said, flags=re.M)


def named_in(said: str) -> Set[str]:
    """The lowercase words a help sentence enumerates (`a, b, c or d`), notes in brackets dropped."""
    said = re.sub(r"\([^)]*\)|\[default[^\]]*\]", " ", " ".join(said.split()))
    said = " ".join(said.split())
    return set(re.findall(r"(?<![A-Za-z0-9_`-])([a-z][a-z0-9_-]*)(?=\s?,|\s?;| or |\s*$|:)", said))


def properties(server, tool: str) -> Dict[str, dict]:
    return input_schema(server.tools_by_name[tool]).get("properties", {})


def edit_request_properties(server) -> Dict[str, dict]:
    return properties(server, "arclang_edit")["requests"]["items"]["properties"]


# --------------------------------------------------------------- subcommands


async def test_every_subcommand_is_run_by_a_tool_or_left_out_with_its_reason(server):
    listed = set(subcommands_of(await help_of(server)))
    assert len(listed) > 40, "the help of arclang lists its subcommands"
    covered = {command.split()[0] for command in cli.covered_subcommands()}
    neither = listed - covered - set(cli.NOT_EXPOSED)
    assert not neither, (
        f"the CLI has {sorted(neither)}: no tool runs them. Add a tool and name it in COVERS, "
        "or say in NOT_EXPOSED (tools/cli.py) why none does"
    )
    both = covered & set(cli.NOT_EXPOSED)
    assert not both, f"{sorted(both)} are run by a tool and said not to be"
    gone = (covered | set(cli.NOT_EXPOSED)) - listed
    assert not gone, f"{sorted(gone)} are named in tools/cli.py and are no subcommand of the CLI"
    assert all(reason.strip() for reason in cli.NOT_EXPOSED.values())


async def test_every_nested_subcommand_is_run_by_a_tool(server):
    covered = set(cli.covered_subcommands())
    for parent in subcommands_of(await help_of(server)):
        if parent in cli.NOT_EXPOSED:
            continue
        nested = [name for name in subcommands_of(await help_of(server, parent)) if name != "help"]
        if not nested:
            assert parent in covered, parent
            continue
        missing = {f"{parent} {name}" for name in nested} - covered
        assert not missing, f"no tool runs {sorted(missing)}: add one, named in COVERS"
        stale = {c for c in covered if c.split()[0] == parent} - {f"{parent} {n}" for n in nested}
        assert not stale, f"{sorted(stale)} are named in COVERS and are no subcommand"


async def test_the_table_of_tools_is_the_list_of_tools(server):
    listed = {tool.name for tool in server.tools}
    declared = set(cli.COVERS) | set(cli.WITHOUT_SUBCOMMAND)
    assert listed - declared == set(), "a tool says in tools/cli.py which subcommand it runs"
    assert declared - listed == set(), "tools/cli.py names tools the server does not list"
    assert not set(cli.COVERS) & set(cli.WITHOUT_SUBCOMMAND)


def test_the_exclusions_are_not_a_way_to_hide_a_capability():
    """What is left out is interactive, stays running or opens a browser."""
    assert set(cli.NOT_EXPOSED) == {"serve", "lsp", "explorer", "help"}, "a subcommand added to NOT_EXPOSED is a decision: change this list with it"


async def test_what_is_said_not_implemented_still_is_not(server, model):
    """What the help of the CLI hides is still not implemented: the day it is, it returns to the
    help, and the tests above ask for a tool."""
    for arguments in (
        ["clean", "."],
        ["sync", "status", "."],
        ["plugin", "list"],
        ["info", "--dependencies", "--", str(model)],
        ["export", "--format=pdf", "--output=out.pdf", "--", str(model)],
    ):
        run = await server.compiler.run(arguments)
        assert not run.ok and "ot implemented" in run.output, (arguments, run.output)


# ------------------------------------------------------------------- options


async def test_every_option_of_a_covered_subcommand_is_reachable_or_left_out_with_its_reason(server):
    problems = []
    for subcommand in cli.covered_subcommands():
        assert subcommand in cli.OPTIONS, f"'{subcommand}' has no entry in OPTIONS (tools/cli.py)"
        declared = cli.OPTIONS[subcommand]
        printed = set(options_of(await help_of(server, *subcommand.split())))
        arguments: Set[str] = set()
        for tool in cli.tools_running(subcommand):
            arguments |= set(properties(server, tool))
        for option in sorted(printed - set(cli.GLOBAL_OPTIONS)):
            reach = declared.get(option)
            same_name = option[2:].replace("-", "_")
            if reach is None and same_name not in arguments:
                problems.append(
                    f"arclang {subcommand} {option}: no tool that runs it has an argument "
                    f"'{same_name}'; add it, or say in OPTIONS how it is reached or why not"
                )
            elif isinstance(reach, cli.Via) and reach.argument not in arguments:
                problems.append(
                    f"arclang {subcommand} {option}: said to be the argument "
                    f"'{reach.argument}', which no tool that runs it has"
                )
            elif isinstance(reach, (cli.ByServer, cli.Never)) and not reach.why.strip():
                problems.append(f"arclang {subcommand} {option}: left out without a reason")
        for option in sorted(set(declared) - printed):
            problems.append(f"arclang {subcommand} {option}: named in OPTIONS, not an option of the CLI")
    assert not problems, "\n".join(problems)


async def test_the_options_every_subcommand_has_are_those_the_table_says(server):
    """The global options are found on a subcommand that has nothing else."""
    printed = set(options_of(await help_of(server, "rules"))) - {"--format"}
    assert printed == set(cli.GLOBAL_OPTIONS), printed


async def test_an_option_never_offered_is_not_an_argument_in_disguise(server):
    """`--open`, `--dap`, `--install`: no tool has an argument that would pass them."""
    for subcommand, declared in cli.OPTIONS.items():
        for option, reach in declared.items():
            if not isinstance(reach, cli.Never):
                continue
            for tool in cli.tools_running(subcommand):
                assert option[2:].replace("-", "_") not in properties(server, tool), (tool, option)


# ------------------------------------- lists: EQUALITY with the CLI, not subset


async def test_export_formats_are_exactly_those_of_the_cli(server):
    listed = values_of(options_of(await help_of(server, "export"))["--format"])
    offered = set(wrapper.EXPORT_FORMATS)
    assert offered | set(wrapper.EXPORT_FORMATS_NOT_OFFERED) == set(listed)
    assert not offered & set(wrapper.EXPORT_FORMATS_NOT_OFFERED)
    assert set(wrapper.DIRECTORY_EXPORT_FORMATS) <= offered
    for tool in ("arclang_export", "arclang_export_diagram"):
        assert properties(server, tool)["format"]["enum"] == list(wrapper.EXPORT_FORMATS)


async def test_the_formats_that_fill_a_directory_are_those_the_cli_says(server):
    said = options_of(await help_of(server, "export"))["--format"]
    directories = set(re.findall(r"^\s+- ([a-z0-9-]+):(?:(?!^\s+- ).)*?DIRECTORY", said, flags=re.M | re.S))
    assert directories <= set(wrapper.DIRECTORY_EXPORT_FORMATS), directories


async def test_import_formats_are_exactly_those_of_the_cli(server):
    listed = values_of(options_of(await help_of(server, "import"))["--format"])
    assert set(wrapper.IMPORT_FORMATS) == set(listed)
    assert properties(server, "arclang_import")["format"]["enum"] == list(wrapper.IMPORT_FORMATS)


async def test_diagram_formats_are_exactly_those_of_the_cli(server):
    listed = values_of(options_of(await help_of(server, "diagram"))["--format"])
    offered = {*wrapper.DIAGRAM_TYPES, *wrapper.TEXT_DIAGRAM_FORMATS, *wrapper.VIEWPOINT_FORMATS}
    assert offered | set(wrapper.DIAGRAM_FORMATS_NOT_OFFERED) == set(listed)
    assert not offered & set(wrapper.DIAGRAM_FORMATS_NOT_OFFERED)
    in_the_schemas = {
        *properties(server, "arclang_viewpoint_diagrams")["format"]["enum"],
        *properties(server, "arclang_generate_diagram")["diagram_type"]["enum"],
    }
    assert in_the_schemas == offered


async def test_diagram_views_are_exactly_those_of_the_cli(server):
    said = options_of(await help_of(server, "diagram"))["--view"]
    listed = named_in(said.split("kind of view:")[1].split(": the views of")[0])
    assert set(wrapper.VIEWS) == listed, listed
    assert properties(server, "arclang_viewpoint_diagrams")["view"]["enum"] == list(wrapper.VIEWS)


async def test_diagram_themes_are_exactly_those_of_the_cli(server):
    said = options_of(await help_of(server, "diagram"))["--theme"]
    listed = set(re.search(r"a preset \(([^)]+)\)", " ".join(said.split())).group(1).split(", "))
    assert set(wrapper.THEMES) == listed


async def test_table_kinds_are_exactly_those_of_the_cli(server, model):
    run = await server.compiler.run(["table", "--list", "--", str(model)])
    assert set(wrapper.TABLE_KINDS) == set(json.loads(run.stdout)["kinds"])
    said = options_of(await help_of(server, "table"))["--kind"]
    assert set(wrapper.TABLE_KINDS) == named_in(said.split("declare:")[1])
    assert properties(server, "arclang_table")["kind"]["enum"] == list(wrapper.TABLE_KINDS)


async def test_table_formats_are_exactly_those_of_the_cli(server):
    listed = values_of(options_of(await help_of(server, "table"))["--format"])
    assert list(wrapper.TABLE_FORMATS) == listed
    assert properties(server, "arclang_table")["format"]["enum"] == listed


async def test_edit_operations_are_exactly_those_of_the_cli(server):
    help_text = await help_of(server, "edit")
    listed = re.findall(r'"([a-z_]+)"', re.search(r'\{"op": ([^.]*?), \.\.\.\}', help_text).group(1))
    assert len(listed) >= 14
    assert set(wrapper.EDIT_OPS) == set(listed)
    assert edit_request_properties(server)["op"]["enum"] == list(wrapper.EDIT_OPS)


async def test_every_field_of_an_edit_request_named_in_the_help_is_in_the_schema(server):
    """`{"op": "reconnect", "kind": K, "new_from": C?}`: each key the help writes is offered."""
    help_text = await help_of(server, "edit")
    written = set(re.findall(r'"([a-z_]+)": ', help_text)) - {"success", "source", "error"}
    assert written <= set(edit_request_properties(server)), written - set(edit_request_properties(server))


@pytest.mark.parametrize(
    "command, option, offered, tool, argument",
    [
        (("impact",), "--format", wrapper.IMPACT_FORMATS, "arclang_impact", "format"),
        (("diff",), "--format", wrapper.DIFF_FORMATS, "arclang_diff", "format"),
        (("baseline", "diff"), "--format", wrapper.DIFF_FORMATS, "arclang_baseline", "format"),
        (("report",), "--format", wrapper.REPORT_FORMATS, "arclang_report", "format"),
        (("check",), "--format", wrapper.DIAGNOSTIC_FORMATS, "arclang_validate", "format"),
        (("gate",), "--format", wrapper.DIAGNOSTIC_FORMATS, "arclang_gate", "format"),
        (("gate",), "--standard", wrapper.SAFETY_STANDARDS, "arclang_gate", "standard"),
        (("safety",), "--standard", wrapper.SAFETY_STANDARDS, "arclang_safety_check", "standard"),
        (("sarif",), "--standard", wrapper.SAFETY_STANDARDS, "arclang_sarif", "standard"),
        (("ci", "init"), "--provider", wrapper.CI_PROVIDERS, "arclang_ci_init", "provider"),
        (("ci", "run"), "--verify", ("auto", "never"), "arclang_ci_run", "verify"),
        (("rules",), "--format", None, "arclang_rules", "format"),
    ],
)
async def test_the_values_of_an_option_are_exactly_those_of_the_cli(
    server, command, option, offered, tool, argument
):
    said = options_of(await help_of(server, *command))[option]
    enum = properties(server, tool)[argument]["enum"]
    if offered is None:
        # No list in the help: the two values the tool offers are accepted, a third is not.
        for value in enum:
            assert (await server.compiler.run([*command, f"{option}={value}"])).ok, value
        return
    assert list(offered) == values_of(said), said
    assert enum == list(offered)


@pytest.mark.parametrize(
    "command, option, between, offered, tool, argument",
    [
        ("impact", "--kinds", ("(default: all):", None), wrapper.IMPACT_LINK_KINDS, "arclang_impact", "kinds"),
        ("impact", "--direction", (None, None), wrapper.IMPACT_DIRECTIONS, "arclang_impact", "direction"),
        ("realize", "--up-to", ("only:", None), wrapper.REALIZATION_STAGES, "arclang_realize", "up_to"),
        ("check", "--method", ("reported:", ". A model"), wrapper.METHODS, "arclang_validate", "method"),
        ("export", "--strategy", ("operations:", "[default"), wrapper.CONNECTOR_STRATEGIES, "arclang_export", "strategy"),
    ],
)
async def test_the_values_a_help_sentence_names_are_exactly_those_offered(
    server, command, option, between, offered, tool, argument
):
    said = " ".join(options_of(await help_of(server, command))[option].split())
    start, end = between
    if start:
        said = said.split(start, 1)[1]
    if end:
        said = said.split(end, 1)[0]
    said = re.sub(r"^\s*--\S+( <\S+>)?", "", said)
    assert named_in(said) == set(offered), said
    declared = properties(server, tool)[argument]
    assert (declared.get("enum") or declared["items"]["enum"]) == list(offered)


async def test_transition_steps_are_exactly_those_of_the_cli(server):
    help_text = await help_of(server, "transition")
    assert set(re.findall(r"\b[a-z]{2}-to-[a-z]{2}\b", help_text)) == set(wrapper.TRANSITION_STEPS)
    assert properties(server, "arclang_transition")["step"]["enum"] == list(wrapper.TRANSITION_STEPS)


async def test_the_gate_standards_of_the_workbench_are_those_of_the_cli(server):
    for command in ("workbench", "impact", "table", "diff"):
        said = " ".join(options_of(await help_of(server, command))["--gate-standard"].split())
        assert all(standard in said for standard in wrapper.WORKBENCH_GATE_STANDARDS), (command, said)
        assert set(re.findall(r"\b(?:ISO|DO|IEC)[0-9A-Z]+\b", said)) == set(wrapper.WORKBENCH_GATE_STANDARDS)


async def test_statistics_engines_and_build_targets_are_those_the_cli_takes(server, model, workspace):
    said = options_of(await help_of(server, "verify"))["--statistics"]
    assert set(re.findall(r"`(\w+)`", said)) == set(wrapper.STATISTICS_ENGINES)
    for target in wrapper.BUILD_TARGETS:
        run = await server.compiler.run(["build", f"--target={target}", f"--output={workspace / target}", "--", str(model)])
        assert run.ok, run.output
    refused = await server.compiler.run(["build", "--target=pdf", "--", str(model)])
    assert not refused.ok
    assert set(re.search(r"expected (.*?) \(", refused.output).group(1).replace(" or ", ", ").split(", ")) == set(wrapper.BUILD_TARGETS)
