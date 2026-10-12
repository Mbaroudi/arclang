"""
Wrapper for the ArcLang compiler binary.

This module is the only place where `arclang` command lines are written.
Every method maps 1:1 to a subcommand of the real CLI (src/cli/mod.rs,
arclang 4.0.1) and passes only options that the CLI accepts. Nothing is
invented on the way back: a `Run` holds what the binary printed and its
exit code, and the parsers below only read lines the binary really prints.
"""

import asyncio
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

# `arclang export -f <format>` values (ExportFormat in src/cli/mod.rs).
EXPORT_FORMATS: Tuple[str, ...] = (
    "capella",
    "json",
    "yaml",
    "xml",
    "markdown",
    "mermaid",
    "plant-uml",
    "html",
    "pdf",
    "terraform",
    "sys-ml",
    "sysml-json",
    "simulink",
    "fmi",
    "req-if",
    "c-headers",
    "proto",
)

# `arclang diagram -f <format>`: one SVG per type.
DIAGRAM_TYPES: Tuple[str, ...] = (
    "operational",
    "functional",
    "component",
    "sequence",
    "state-machine",
    "physical",
    "class",
    "tree",
    "capability",
    "functional-chain",
)

# `arclang diagram -f <format>`: the Arcadia viewpoint diagrams.
VIEWPOINT_FORMATS: Tuple[str, ...] = ("viewpoints", "elk", "html")

# `arclang diagram --view <view>`.
VIEWS: Tuple[str, ...] = ("oab", "sab", "lab", "pab", "msm", "es")

SAFETY_STANDARDS: Tuple[str, ...] = ("iso26262", "do178c", "iec61508")

# Lines of `build` and `info --metrics` that carry a count.
METRIC_LABELS: Dict[str, str] = {
    "Total elements": "total_elements",
    "Requirements": "requirements",
    "Components": "components",
    "Functions": "functions",
    "Traces": "traces",
}

DEFAULT_TIMEOUT_SECONDS = 30
# `arclang review` asks one question per trace over the network.
DEFAULT_REVIEW_TIMEOUT_SECONDS = 600


@dataclass(frozen=True)
class Run:
    """What one invocation of the binary did."""

    command: Tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str

    @property
    def ok(self) -> bool:
        return self.returncode == 0

    @property
    def output(self) -> str:
        """Standard output then standard error, as a reader would see them."""
        return "\n".join(part.rstrip() for part in (self.stdout, self.stderr) if part.strip())


def absolute_binary(path: str) -> str:
    """A bare name is left to the PATH; a path with a directory is made absolute."""
    return str(Path(path).expanduser().resolve()) if os.sep in path or path.startswith("~") else path


def seconds(value: Any, default: float) -> float:
    """A timeout from the configuration: a positive number, else the default."""
    if isinstance(value, (int, float)) and not isinstance(value, bool) and value > 0:
        return float(value)
    return default


class ArcLangCompiler:
    """Runs the ArcLang compiler binary."""

    def __init__(self, config: Dict[str, Any]):
        # The compiler runs with the workspace as working directory: a relative
        # binary path is resolved now, from where the server was started.
        self.binary_path = absolute_binary(
            os.getenv("ARCLANG_BINARY") or config.get("path") or "arclang"
        )
        self.timeout = seconds(config.get("timeout"), DEFAULT_TIMEOUT_SECONDS)
        self.review_timeout = seconds(config.get("review_timeout"), DEFAULT_REVIEW_TIMEOUT_SECONDS)
        self.workspace_root = Path(os.getenv("ARCLANG_WORKSPACE", os.getcwd())).resolve()

    async def run(self, arguments: Sequence[str], timeout: Optional[float] = None) -> Run:
        """Run `arclang <arguments>`. A binary that cannot run is a failed Run."""
        command = (self.binary_path, *arguments)
        limit = timeout or self.timeout
        try:
            process = await asyncio.create_subprocess_exec(
                *command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=str(self.workspace_root) if self.workspace_root.is_dir() else None,
            )
        except OSError as failure:
            return Run(command, -1, "", f"cannot run '{self.binary_path}': {failure}")
        try:
            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=limit)
        except asyncio.TimeoutError:
            process.kill()
            await process.wait()
            return Run(command, -1, "", f"command timed out after {limit} seconds")
        except asyncio.CancelledError:
            # The client gave up: do not leave the compiler running.
            process.kill()
            raise
        return Run(
            command,
            process.returncode if process.returncode is not None else -1,
            stdout.decode(errors="replace"),
            stderr.decode(errors="replace"),
        )

    async def version(self) -> Run:
        return await self.run(["--version"])

    async def check(self, model: Path) -> Run:
        """`arclang check --lint`: compile, then traceability and methodology advisories."""
        return await self.run(["check", "--lint", "--", str(model)])

    async def build(self, model: Path, output: Optional[Path] = None, release: bool = False) -> Run:
        """`arclang build`: compile to the model's JSON."""
        arguments = ["build"]
        if output:
            arguments += ["-o", str(output)]
        if release:
            arguments.append("--release")
        return await self.run([*arguments, "--", str(model)])

    async def trace(self, model: Path, matrix: bool = False) -> Run:
        """`arclang trace --validate [--matrix]`."""
        arguments = ["trace", "--validate"]
        if matrix:
            arguments.append("--matrix")
        return await self.run([*arguments, "--", str(model)])

    async def info(self, model: Path) -> Run:
        """`arclang info --metrics`."""
        return await self.run(["info", "--metrics", "--", str(model)])

    async def gate(self, model: Path, standard: str = "iso26262") -> Run:
        """`arclang gate`: exit 0 on PASS, 1 on FAIL."""
        return await self.run(["gate", "--standard", standard, "--", str(model)])

    async def diff(self, old: Path, new: Path, as_json: bool = True) -> Run:
        """`arclang diff`: semantic diff by stable identity. Exits 1 when they differ."""
        arguments = ["diff"]
        if as_json:
            arguments.append("--json")
        return await self.run([*arguments, "--", str(old), str(new)])

    async def impact(self, model: Path, element: str) -> Run:
        """`arclang impact`: what a change to one element reaches."""
        return await self.run(["impact", "--", str(model), element])

    async def safety(self, model: Path, standard: str, report: bool = False) -> Run:
        """`arclang safety --standard <standard> [--report]`."""
        arguments = ["safety", "--standard", standard]
        if report:
            arguments.append("--report")
        return await self.run([*arguments, "--", str(model)])

    async def export(self, model: Path, export_format: str, output: Path) -> Run:
        """`arclang export -f <format> -o <output>`."""
        return await self.run(["export", "-f", export_format, "-o", str(output), "--", str(model)])

    async def diagram(
        self,
        model: Path,
        diagram_format: str,
        output: Path,
        view: Optional[str] = None,
    ) -> Run:
        """`arclang diagram -f <format> -o <output> [--view <view>]`."""
        arguments = ["diagram", "-f", diagram_format, "-o", str(output)]
        if view:
            arguments += ["--view", view]
        return await self.run([*arguments, "--", str(model)])

    async def format(self, target: Path, write: bool = False, check: bool = False) -> Run:
        """`arclang format`: whitespace only; prints the result unless told to write."""
        arguments = ["format"]
        if write:
            arguments.append("--write")
        if check:
            arguments.append("--check")
        return await self.run([*arguments, "--", str(target)])

    async def set_attribute(
        self, model: Path, element: str, key: str, value: str, write: bool = False
    ) -> Run:
        """`arclang set`: the value is ArcLang source (`"text"`, `25 ms`, `[A, B]`)."""
        return await self.run([*self._edit("set", write), str(model), element, key, value])

    async def unset_attribute(
        self, model: Path, element: str, key: str, write: bool = False
    ) -> Run:
        """`arclang unset`."""
        return await self.run([*self._edit("unset", write), str(model), element, key])

    async def rename(self, model: Path, element: str, name: str, write: bool = False) -> Run:
        """`arclang rename`."""
        return await self.run([*self._edit("rename", write), str(model), element, name])

    async def review(
        self,
        model: Path,
        send: bool = False,
        suggest: bool = False,
        threshold: Optional[float] = None,
        max_questions: Optional[int] = None,
    ) -> Run:
        """`arclang review --json`. Without `send` it is a dry run: nothing leaves the machine."""
        arguments = ["review", "--json"]
        if not send:
            arguments.append("--dry-run")
        if suggest:
            arguments.append("--suggest")
        if threshold is not None:
            arguments += ["--threshold", str(threshold)]
        if max_questions is not None:
            arguments += ["--max-questions", str(max_questions)]
        return await self.run([*arguments, "--", str(model)], timeout=self.review_timeout)

    async def metamodel(self, as_json: bool = False) -> Run:
        """`arclang metamodel --format markdown|json`."""
        return await self.run(["metamodel", "--format", "json" if as_json else "markdown"])

    @staticmethod
    def _edit(command: str, write: bool) -> List[str]:
        # `--` so that a value such as `-5` is never read as an option.
        return [command, "--write", "--"] if write else [command, "--"]


def parse_diff(run: Run) -> Optional[Dict[str, List[Any]]]:
    """The changes printed by `arclang diff --json`, or None when it printed none.

    Like `diff`, the command exits 1 when the two models differ: the exit
    code alone does not tell a difference from a failure, the JSON does.
    """
    try:
        changes = json.loads(run.stdout)
    except json.JSONDecodeError:
        return None
    if not isinstance(changes, dict) or not all(isinstance(v, list) for v in changes.values()):
        return None
    return changes


def parse_metrics(output: str) -> Dict[str, int]:
    """Counts printed by `build`, `check --lint` and `info --metrics`."""
    metrics: Dict[str, int] = {}
    for line in output.splitlines():
        label, _, value = line.strip().partition(":")
        key = METRIC_LABELS.get(label)
        if key and value.strip().isdigit():
            metrics[key] = int(value.strip())
    return metrics


def parse_coverage(output: str) -> Optional[float]:
    """The traceability coverage the compiler printed, or None when it printed none.

    Only the compiler's own lines count (`Traceability Coverage: 85.0%`,
    `Traceability: 85.0%`), never a rationale or a name that mentions a percentage.
    """
    found = re.findall(
        r"^\s*Traceability(?: [Cc]overage)?:\s*([0-9]+(?:\.[0-9]+)?)\s*%\s*$", output, flags=re.M
    )
    return float(found[-1]) if found else None


def parse_block(output: str, header: str) -> List[str]:
    """The indented lines that follow the line containing `header`."""
    lines: List[str] = []
    inside = False
    for line in output.splitlines():
        if header.lower() in line.lower():
            inside = True
            continue
        if inside:
            if line.startswith("  ") and line.strip():
                lines.append(line.strip())
            else:
                inside = False
    return lines


def parse_blockers(output: str) -> List[str]:
    """The blockers of `arclang gate`, one `[rule] message` per line."""
    return parse_block(output, "blocker(s):")
