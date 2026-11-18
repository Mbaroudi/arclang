"""
What a model delivers: its tables and matrices, a document filled from a
template, a new model from a template, and what a pipeline reads (the
pull-request report, the SARIF log, the pipeline files themselves).

Each tool is a subcommand of the compiler (`arclang table`, `document`,
`new`, `report`, `sarif`, `ci`) and returns what it printed. A file is
written only where the call names it. `report`, `sarif` and `ci` work on
the git repository they are run in: `directory` names it (the workspace by
default), and the compiler runs there. `arclang_ci_init` writes pipeline
files at the top of that repository, so it takes the directory by name and
writes nothing without `write: true`.
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..compiler.wrapper import (
    CI_PROVIDERS,
    REPORT_FORMATS,
    SAFETY_STANDARDS,
    TABLE_FORMATS,
    TABLE_KINDS,
    ArcLangCompiler,
)
from .common import (
    MODEL_PATH,
    CommandLine,
    InvalidArguments,
    bounded_seconds,
    report,
    repository_root,
    resolve_path,
    schema,
)
from .core import GATE_STANDARD, RUN_SECONDS, RUN_SECONDS_MAX, RUN_TIMEOUT, STANDARD

# The formats of `arclang document`, chosen by the extension of its output.
DOCUMENT_EXTENSIONS = (".md", ".html", ".docx")
REPORT_FAIL_ON = ("blocker", "major", "never")
SARIF_FAIL_ON = ("error", "never")
CI_VERIFY = ("auto", "never")

DIRECTORY = {
    "type": "string",
    "description": "A directory of the git repository worked on (default: the workspace)",
}
MODELS = {
    "type": "array",
    "items": {"type": "string"},
    "description": "The models examined (default: every entry-point model of the repository, "
    "the .arc files no other .arc imports)",
}

TOOLS: List[Tool] = [
    Tool(
        name="arclang_table",
        description=(
            "Live tables and matrices of a model (`arclang table`), computed by the "
            "compiler. `kind` computes one the model does not declare: "
            "requirements_coverage (each requirement, what satisfies and verifies it, its "
            "verdict, its levels of the V), traceability_matrix, allocation_matrix, "
            "interface_n2, icd, elements (`row_type`: the kind of the rows), realization "
            "(the ladder of verdicts per requirement), partition, schedule, cesam_grid. "
            "`name` is a table the model declares (default: every declared table); `list: "
            "true` lists the declared tables and the kinds. `scope` is a layer (OA, SA, LA, "
            "PA, EPBS) or an element; `columns`, `rows` and `kinds` shape a matrix; `filter` "
            "keeps the rows matching (words, column:text, column=text, column!=text, "
            "is:gap, -term) and `gaps: true` only the rows with a gap or a failure. Formats: "
            "json (with ids and UUIDs), csv, tsv, md, html, xlsx (needs `output_path`). "
            "`declaration: true` prints the `table` declaration of the table asked for, as "
            "ArcLang source to paste in the model."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "list": {
                    "type": "boolean",
                    "description": "List the tables the model declares and the kinds computed",
                    "default": False,
                },
                "name": {"type": "string", "description": "A table the model declares, by name or id"},
                "kind": {
                    "type": "string",
                    "enum": list(TABLE_KINDS),
                    "description": "Compute a table the model does not declare",
                },
                "scope": {
                    "type": "string",
                    "description": "What the rows are taken from: a layer (OA, SA, LA, PA, EPBS) "
                    "or an element",
                },
                "row_type": {
                    "type": "string",
                    "description": "elements: the kind of the rows, e.g. Requirement",
                },
                "columns": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "The columns shown, by key as the JSON format names them "
                    "(id, name, safety_level: not the header's label); for a matrix, the "
                    "kinds of its columns",
                },
                "rows": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "For a matrix: the kinds (or layers) of its rows",
                },
                "kinds": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "For a traceability matrix: the trace kinds",
                },
                "filter": {
                    "type": "string",
                    "description": "Keep the rows matching: words, column:text, column=text, "
                    "column!=text, is:gap, -term",
                },
                "gaps": {
                    "type": "boolean",
                    "description": "Keep only the rows with a gap or a failure",
                    "default": False,
                },
                "format": {
                    "type": "string",
                    "enum": list(TABLE_FORMATS),
                    "description": "Format of the table",
                    "default": "json",
                },
                "output_path": {
                    "type": "string",
                    "description": "Write the table to this file (required for xlsx)",
                },
                "gate_standard": GATE_STANDARD,
                "declaration": {
                    "type": "boolean",
                    "description": "Print the `table` declaration of the table asked for",
                    "default": False,
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_document",
        description=(
            "A document filled from the model (`arclang document`): a Markdown template "
            "whose `{{ ... }}` directives the compiler fills (model, date, value, table, "
            "elements, verdicts, constraints, gate, suspects, rules). Without "
            "`template_path`, a design review of the whole model. The extension of "
            "`output_path` chooses the format: .md, .html or .docx (Word). Every figure in "
            "it is the compiler's."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "template_path": {
                    "type": "string",
                    "description": "The template: Markdown with {{ ... }} directives "
                    "(default: the design review)",
                },
                "output_path": {
                    "type": "string",
                    "description": "Where the document is written: .md, .html or .docx",
                },
            },
            "model_path",
            "output_path",
        ),
    ),
    Tool(
        name="arclang_new",
        description=(
            "Start a model from a template (`arclang new <name> --template <template>`): "
            "`cesam` writes a model following the CESAM framework, one section per vision, "
            "every element a placeholder to rename, which compiles and passes the gate. The "
            "file is `output_path`, else `<name>.arc` in the workspace; a file that exists "
            "is never overwritten."
        ),
        inputSchema=schema(
            {
                "name": {"type": "string", "description": "The name of the model"},
                "template": {
                    "type": "string",
                    "description": "The template: cesam (the compiler names those it has)",
                    "default": "cesam",
                },
                "output_path": {
                    "type": "string",
                    "description": "The .arc file to write (default: <name>.arc in the workspace)",
                },
            },
            "name",
        ),
    ),
    Tool(
        name="arclang_report",
        description=(
            "The pull-request report (`arclang report --pr --from <base>`): for each model, "
            "the semantic diff between the base and the head, the impact of the changes, "
            "the gate before and after, the requirements that are no longer satisfied or "
            "verified. `from_rev` is the base (a branch, tag or commit), `to_rev` the head "
            "(default: the working tree). `fail_on` decides when the answer is a failure: "
            "blocker (a new blocker or a model that no longer compiles), major (also a "
            "regression), never. `out_dir` writes the before and after pictures (SVG) under "
            "it."
        ),
        inputSchema=schema(
            {
                "from_rev": {
                    "type": "string",
                    "description": "The base: a branch, tag or commit",
                },
                "to_rev": {"type": "string", "description": "The head (default: the working tree)"},
                "directory": DIRECTORY,
                "models": MODELS,
                "format": {
                    "type": "string",
                    "enum": list(REPORT_FORMATS),
                    "description": "md, json or html",
                    "default": "md",
                },
                "output_path": {"type": "string", "description": "Write the report here (default: returned)"},
                "out_dir": {
                    "type": "string",
                    "description": "Write the before and after pictures under this directory, "
                    "as <dir>/diagrams/",
                },
                "standard": STANDARD,
                "fail_on": {
                    "type": "string",
                    "enum": list(REPORT_FAIL_ON),
                    "description": "When the report is a failure",
                    "default": "blocker",
                },
                "note": {"type": "string", "description": "A line of Markdown added to the report"},
                "allow_empty": {
                    "type": "boolean",
                    "description": "Succeed when there is no model at all",
                    "default": False,
                },
            },
            "from_rev",
        ),
    ),
    Tool(
        name="arclang_sarif",
        description=(
            "Compiler diagnostics and gate blockers of several models as one SARIF 2.1.0 "
            "log (`arclang sarif`), for GitHub code scanning and Azure DevOps. For one "
            "model, arclang_validate and arclang_gate take `format: sarif`. `fail_on: "
            "never` makes a model that fails a result and not a failure."
        ),
        inputSchema=schema(
            {
                "directory": DIRECTORY,
                "models": MODELS,
                "standard": STANDARD,
                "output_path": {"type": "string", "description": "Write the SARIF file here (default: returned)"},
                "fail_on": {
                    "type": "string",
                    "enum": list(SARIF_FAIL_ON),
                    "description": "error: a failure when a model does not compile or its "
                    "gate fails; never: always a result",
                    "default": "error",
                },
            }
        ),
    ),
    Tool(
        name="arclang_ci_init",
        description=(
            "The pipeline files of a repository (`arclang ci init`), for GitHub Actions, "
            "GitLab CI, Azure DevOps or Jenkins: the pipeline compiles every model, runs "
            "the production gate, writes the SARIF log and the pull-request report. "
            "`directory` is the repository (required: the files are written at its top). "
            "Without `write: true` the answer is the files as they would be written "
            "(`--dry-run`) and nothing changes; a file that exists is refused unless "
            "`force`. `provider` defaults to the one the git remote `origin` points at. The "
            "files are not run against the provider: the first pipeline run is their test."
        ),
        inputSchema=schema(
            {
                "directory": {
                    "type": "string",
                    "description": "A directory of the git repository the pipeline is for",
                },
                "provider": {
                    "type": "string",
                    "enum": list(CI_PROVIDERS),
                    "description": "The CI system (default: told from the git remote `origin`)",
                },
                "standard": STANDARD,
                "models": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "The models the pipeline checks, relative to the top of the "
                    "repository (default: every entry point)",
                },
                "write": {
                    "type": "boolean",
                    "description": "Write the files. Default: print them and change nothing",
                    "default": False,
                },
                "force": {
                    "type": "boolean",
                    "description": "With write: overwrite files that exist",
                    "default": False,
                },
                "arclang_version": {
                    "type": "string",
                    "description": "The ArcLang release the pipeline installs (default: this compiler's)",
                },
                "release_repo": {
                    "type": "string",
                    "description": "owner/repo of the GitHub releases the binary is downloaded from",
                },
                "branch": {
                    "type": "string",
                    "description": "The branch whose pushes run the pipeline (default: the "
                    "repository's default branch)",
                },
            },
            "directory",
        ),
    ),
    Tool(
        name="arclang_ci_run",
        description=(
            "What a pipeline runs, here (`arclang ci run`): compile every model of the "
            "repository, run the production gate on each, and answer PASS only when every "
            "model passes; `sarif_path` and `summary_path` write the SARIF log and a "
            "Markdown summary. `verify: auto` also simulates the cases that simulate "
            "(OpenModelica, minutes); the default here is `never`, so that nothing is "
            "simulated unless asked."
        ),
        inputSchema=schema(
            {
                "directory": DIRECTORY,
                "models": MODELS,
                "standard": STANDARD,
                "sarif_path": {"type": "string", "description": "Write the SARIF 2.1.0 file here"},
                "summary_path": {"type": "string", "description": "Write a Markdown summary here"},
                "verify": {
                    "type": "string",
                    "enum": list(CI_VERIFY),
                    "description": "auto: simulate the cases of the models that have some; "
                    "never (the default here)",
                    "default": "never",
                },
                "allow_empty": {
                    "type": "boolean",
                    "description": "Succeed when there is no model at all",
                    "default": False,
                },
                "timeout": RUN_TIMEOUT,
            }
        ),
    ),
    Tool(
        name="arclang_ci_models",
        description=(
            "The models of a repository (`arclang ci models`): the .arc files no other "
            ".arc imports, which are what `arclang_ci_run`, `arclang_sarif` and "
            "`arclang_report` examine by default."
        ),
        inputSchema=schema(
            {
                "directory": DIRECTORY,
                "as_json": {"type": "boolean", "description": "The list as JSON", "default": False},
            }
        ),
    ),
]


class DeliveryTools:
    """Tables, documents, new models, and what a pipeline reads."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_table": self._table,
            "arclang_document": self._document,
            "arclang_new": self._new,
            "arclang_report": self._report,
            "arclang_sarif": self._sarif,
            "arclang_ci_init": self._ci_init,
            "arclang_ci_run": self._ci_run,
            "arclang_ci_models": self._ci_models,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown delivery tool: {tool_name}")
        return await handler(arguments)

    def _line(self, args: Dict[str, Any], *command: str) -> CommandLine:
        return CommandLine(self.workspace_root, args, *command)

    def _directory(self, args: Dict[str, Any]) -> Path:
        directory = resolve_path(self.workspace_root, args.get("directory") or ".")
        if not directory.is_dir():
            raise InvalidArguments(f"'{args.get('directory')}' is not a directory")
        return directory

    def _models(self, args: Dict[str, Any], directory: Path) -> List[str]:
        """The models named, each a path: relative ones are from `directory`, where the compiler runs."""
        models = []
        for model in args.get("models") or []:
            path = Path(model).expanduser()
            models.append(str(resolve_path(self.workspace_root, str(path if path.is_absolute() else directory / path))))
        return models

    async def _table(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        title = f"Tables of {model.name}"
        if args.get("list", False):
            run = await self.compiler.run(self._line(args, "table", "--list").end(model))
            return report(title, run, "the declared tables and the kinds computed" if run.ok else None)
        if args.get("name") and args.get("kind"):
            raise InvalidArguments("'name' is a declared table and 'kind' a computed one: ask one")
        table_format = args.get("format", "json")
        if table_format == "xlsx" and not args.get("output_path"):
            raise InvalidArguments("'format: xlsx' is a file: give 'output_path'")
        line = self._line(args, "table").value("name", "--name").value("kind", "--kind")
        line.value("scope", "--scope").value("row_type", "--row-type")
        line.joined("columns", "--columns").joined("rows", "--rows").joined("kinds", "--kinds")
        line.value("filter", "--filter").flag("gaps", "--gaps").add(f"--format={table_format}")
        line.value("gate_standard", "--gate-standard").flag("declaration", "--declaration")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(model))
        what = args.get("kind") or args.get("name") or "every declared table"
        answer = report(f"Table {what} of {model.name}", run)
        if run.ok and output is not None and output.is_file():
            answer += f"\n\nWritten to {output}"
        return answer

    async def _document(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        output = resolve_path(self.workspace_root, args["output_path"])
        if output.suffix.lower() not in DOCUMENT_EXTENSIONS:
            raise InvalidArguments(
                f"the extension of 'output_path' chooses the format: {', '.join(DOCUMENT_EXTENSIONS)}"
            )
        output.parent.mkdir(parents=True, exist_ok=True)
        line = self._line(args, "document", f"--output={output}")
        line.path("template_path", "--template")
        run = await self.compiler.run(line.end(model))
        verdict = f"written to {output}" if run.ok and output.is_file() else None
        return report(f"Document of {model.name}", run, verdict)

    async def _new(self, args: Dict[str, Any]) -> str:
        name = args["name"]
        output = resolve_path(self.workspace_root, args.get("output_path") or f"{name}.arc")
        output.parent.mkdir(parents=True, exist_ok=True)
        template = args.get("template", "cesam")
        run = await self.compiler.run(
            ["new", f"--template={template}", f"--output={output}", "--", name]
        )
        verdict = f"written to {output}" if run.ok and output.is_file() else None
        return report(f"New model '{name}' from the {template} template", run, verdict)

    async def _report(self, args: Dict[str, Any]) -> str:
        directory = self._directory(args)
        line = self._line(args, "report", "--pr", f"--from={args['from_rev']}")
        line.value("to_rev", "--to").value("format", "--format").value("standard", "--standard")
        line.value("fail_on", "--fail-on").value("note", "--note").flag("allow_empty", "--allow-empty")
        output = line.path("output_path", "--output", make_parent=True)
        line.path("out_dir", "--out-dir")
        run = await self.compiler.run(line.end(*self._models(args, directory)), cwd=directory)
        answer = report(f"Pull-request report from {args['from_rev']}", run)
        if output is not None and output.is_file():
            answer += f"\n\nWritten to {output}"
        return answer

    async def _sarif(self, args: Dict[str, Any]) -> str:
        directory = self._directory(args)
        line = self._line(args, "sarif").value("standard", "--standard").value("fail_on", "--fail-on")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(*self._models(args, directory)), cwd=directory)
        answer = report("SARIF log", run)
        if output is not None and output.is_file():
            answer += f"\n\nWritten to {output}"
        return answer

    async def _ci_init(self, args: Dict[str, Any]) -> str:
        directory = self._directory(args)
        # The files go to the top of the repository, wherever in it the command runs.
        top = repository_root(self.workspace_root, directory)
        write = args.get("write", False)
        if args.get("force", False) and not write:
            raise InvalidArguments("'force' goes with 'write: true'")
        for model in args.get("models") or []:
            # A path from the top of the repository, as the pipeline will write it.
            if Path(model).is_absolute() or ".." in Path(model).parts:
                raise InvalidArguments(f"'{model}' must be a path from the top of the repository")
        line = self._line(args, "ci", "init").value("provider", "--provider")
        line.value("standard", "--standard").each("models", "--model")
        line.value("arclang_version", "--arclang-version").value("release_repo", "--release-repo")
        line.value("branch", "--branch")
        if write:
            line.flag("force", "--force")
        else:
            line.add("--dry-run")
        run = await self.compiler.run(line.parts, cwd=directory)
        verdict: Optional[str] = None
        if run.ok:
            verdict = f"written at the top of {top}" if write else "NOT written (pass write: true to write these files)"
        return report(f"Pipeline files for {top.name}", run, verdict)

    async def _ci_run(self, args: Dict[str, Any]) -> str:
        directory = self._directory(args)
        verify = args.get("verify", "never")
        line = self._line(args, "ci", "run", f"--verify={verify}").value("standard", "--standard")
        line.flag("allow_empty", "--allow-empty")
        line.path("sarif_path", "--sarif", make_parent=True)
        line.path("summary_path", "--summary", make_parent=True)
        limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        run = await self.compiler.run(
            line.end(*self._models(args, directory)),
            timeout=limit if verify == "auto" else max(self.compiler.timeout, 120),
            cwd=directory,
        )
        verdict = None
        if run.ok:
            verdict = "PASS" + ("" if verify == "auto" else " (compiled and gated; nothing was simulated)")
        return report(f"Pipeline run in {directory.name}", run, verdict)

    async def _ci_models(self, args: Dict[str, Any]) -> str:
        directory = self._directory(args)
        line = self._line(args, "ci", "models").flag("as_json", "--json")
        return report(f"Models of {directory.name}", await self.compiler.run(line.parts, cwd=directory))
