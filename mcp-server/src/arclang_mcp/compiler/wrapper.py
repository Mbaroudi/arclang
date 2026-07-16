"""
Wrapper for ArcLang compiler binary.

Every command emitted here maps 1:1 to the real clap CLI defined in
src/cli/mod.rs. Flags that do not exist in the CLI are never emitted.
Parsers only extract data that the binary actually prints; when the
binary does not expose structured output, methods return None together
with "parse_supported": False instead of fabricating results.
"""

import asyncio
import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

# Substrings that mark a line as an error in compiler output.
ERROR_MARKERS = ("error", "Error", "✗")


class ArcLangCompiler:
    """Wrapper for ArcLang compiler binary."""

    def __init__(self, config: Dict[str, Any]):
        # Try environment variable first, then config, then default
        self.binary_path = (
            os.getenv("ARCLANG_BINARY") or
            config.get("path") or
            "arclang"
        )
        self.timeout = config.get("timeout", 30)

        # Get workspace root for resolving relative paths
        self.workspace_root = Path(os.getenv("ARCLANG_WORKSPACE", os.getcwd()))

    async def compile(
        self,
        model_path: Path,
        validate: bool = True,
        optimize: bool = False
    ) -> Dict[str, Any]:
        """Compile ArcLang model via `arclang build`.

        Note: the CLI always validates during compilation; there is no
        separate `--validate` flag. `optimize` maps to `--release`.
        """
        cmd = [self.binary_path, "build", str(model_path)]

        if optimize:
            cmd.append("--release")

        result = await self._run_command(cmd)

        return {
            "success": result["returncode"] == 0,
            "output": result["stdout"],
            "raw_output": result["stdout"],
            "errors": self._parse_errors(result["stderr"], result["stdout"]),
            "metrics": self._parse_metrics(result["stdout"]) if result["returncode"] == 0 else {}
        }

    async def validate(self, model_path: Path, strict: bool = False) -> Dict[str, Any]:
        """Validate ArcLang model via `arclang check --lint`.

        Note: the CLI has no `--strict` flag; the `strict` argument is
        accepted for API compatibility but has no effect.
        """
        cmd = [self.binary_path, "check", str(model_path), "--lint"]

        result = await self._run_command(cmd)

        return {
            "valid": result["returncode"] == 0,
            "warnings": self._parse_warnings(result["stdout"]),
            "errors": self._parse_errors(result["stderr"], result["stdout"]),
            "raw_output": result["stdout"]
        }

    async def trace_analysis(
        self,
        model_path: Path,
        show_gaps: bool = True,
        matrix: bool = False
    ) -> Dict[str, Any]:
        """Analyze traceability via `arclang trace --validate`.

        Note: the CLI has no `--gaps` flag. Traceability issues reported
        by `--validate` are returned as plain strings in "issues". A
        structured gap breakdown is not provided by the binary, so
        "gaps" is None with "parse_supported": False.
        """
        cmd = [self.binary_path, "trace", str(model_path), "--validate"]

        if matrix:
            cmd.append("--matrix")

        result = await self._run_command(cmd)

        return {
            "coverage": self._parse_coverage(result["stdout"]),
            "total_traces": self._count_traces(result["stdout"]),
            "issues": self._parse_trace_issues(result["stdout"]),
            "gaps": None,
            "parse_supported": False,
            "raw_output": result["stdout"]
        }

    async def export_diagram(
        self,
        model_path: Path,
        format_type: str = "html",
        output_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Export architecture diagram via `arclang export -o <out> -f <fmt>`."""
        if not output_path:
            output_path = str(model_path.with_suffix(".html"))

        cmd = [
            self.binary_path, "export", str(model_path),
            "-o", output_path,
            "-f", format_type
        ]

        result = await self._run_command(cmd)

        return {
            "success": result["returncode"] == 0,
            "output_path": output_path if result["returncode"] == 0 else None,
            "error": result["stderr"] if result["returncode"] != 0 else None,
            "raw_output": result["stdout"]
        }

    async def info(self, model_path: Path, detailed: bool = False) -> Dict[str, Any]:
        """Get model information via `arclang info --metrics`.

        Note: the CLI has no `--detailed` flag; the `detailed` argument
        is accepted for API compatibility but has no effect.
        """
        cmd = [self.binary_path, "info", str(model_path), "--metrics"]

        result = await self._run_command(cmd)

        return {
            "metrics": self._parse_metrics(result["stdout"]),
            "size_kb": model_path.stat().st_size / 1024 if model_path.exists() else 0,
            "raw_output": result["stdout"]
        }

    async def safety_check(
        self,
        model_path: Path,
        standard: str,
        generate_report: bool = False
    ) -> Dict[str, Any]:
        """Check safety compliance via `arclang safety --standard <std>`.

        Note: the binary does not print structured issue data, so
        "issues" is None with "parse_supported": False.
        """
        cmd = [self.binary_path, "safety", str(model_path), "--standard", standard]

        if generate_report:
            cmd.append("--report")

        result = await self._run_command(cmd)

        return {
            "compliant": result["returncode"] == 0,
            "issues": None,
            "parse_supported": False,
            "recommendations": [],
            "raw_output": result["stdout"]
        }

    async def hazard_analysis(
        self,
        model_path: Path,
        standard: str
    ) -> Dict[str, Any]:
        """Hazard analysis (HARA) is not implemented in the arclang CLI.

        The CLI `safety` subcommand only supports --fmea/--fta/--report
        and produces no hazard data.
        """
        raise NotImplementedError(
            "Hazard analysis (HARA) is not implemented in the arclang CLI: "
            "the `safety` subcommand only accepts --standard/--fmea/--fta/--report "
            "and does not produce hazard data."
        )

    async def semantic_merge(
        self,
        base_path: Path,
        ours_path: Path,
        theirs_path: Path
    ) -> Dict[str, Any]:
        """Semantic merge is not implemented in the arclang CLI."""
        raise NotImplementedError(
            "Semantic merge is not implemented in the arclang CLI: "
            "no merge subcommand exists in the compiler."
        )

    async def plm_sync(
        self,
        model_path: Path,
        system: str,
        operation: str
    ) -> Dict[str, Any]:
        """PLM synchronization.

        There is no `plm` subcommand in the arclang CLI (only `sync
        pull/push`, which are unimplemented stubs), so this returns a
        structured error instead of invoking the binary.
        """
        return {
            "success": False,
            "error": "plm sync not implemented in compiler"
        }

    async def generate_diagram(
        self,
        model_path: str,
        diagram_type: str,
        output_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Generate a specific diagram type."""
        # Resolve model path relative to workspace if not absolute
        resolved_model = Path(model_path)
        if not resolved_model.is_absolute():
            resolved_model = self.workspace_root / model_path

        # Resolve output path relative to workspace if not absolute
        if not output_path:
            output_path = str(self.workspace_root / f"{diagram_type}.svg")
        else:
            resolved_output = Path(output_path)
            if not resolved_output.is_absolute():
                output_path = str(self.workspace_root / output_path)

        # Create output directory if it doesn't exist
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)

        cmd = [
            self.binary_path,
            "diagram",
            str(resolved_model),
            "-o", output_path,
            "--format", diagram_type
        ]

        result = await self._run_command(cmd)

        if result["returncode"] != 0:
            raise Exception(f"Diagram generation failed: {result['stderr']}")

        # Get file size
        size = os.path.getsize(output_path)
        size_str = f"{size/1024:.1f}KB" if size > 1024 else f"{size}B"

        # Parse output for element count
        element_count = self._parse_element_count(result["stdout"], diagram_type)

        return {
            "output_path": output_path,
            "size": size_str,
            "element_count": element_count
        }

    async def _run_command(self, cmd: list) -> Dict[str, Any]:
        """Run command asynchronously."""
        try:
            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )

            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=self.timeout
            )

            return {
                "returncode": process.returncode,
                "stdout": stdout.decode(),
                "stderr": stderr.decode()
            }
        except asyncio.TimeoutError:
            return {
                "returncode": -1,
                "stdout": "",
                "stderr": f"Command timed out after {self.timeout} seconds"
            }
        except Exception as e:
            return {
                "returncode": -1,
                "stdout": "",
                "stderr": str(e)
            }

    def _parse_metrics(self, output: str) -> Dict[str, Any]:
        """Parse metrics from output."""
        metrics = {}
        for line in output.split("\n"):
            if "Requirements:" in line:
                metrics["requirements"] = self._parse_int(line)
            elif "Components:" in line:
                metrics["components"] = self._parse_int(line)
            elif "Functions:" in line:
                metrics["functions"] = self._parse_int(line)
            elif "Traces:" in line:
                metrics["traces"] = self._parse_int(line)
        return {k: v for k, v in metrics.items() if v is not None}

    @staticmethod
    def _parse_int(line: str) -> Optional[int]:
        """Parse the integer after the last colon in a line, or None."""
        try:
            return int(line.split(":")[-1].strip())
        except ValueError:
            return None

    def _parse_warnings(self, output: str) -> List[str]:
        """Parse warnings from `check` output.

        The CLI prints a '⚠ Traceability warnings:' header followed by
        indented warning lines; standalone lines containing 'warning:'
        are also collected.
        """
        warnings = []
        in_warning_block = False
        for line in output.split("\n"):
            if "warnings:" in line.lower():
                in_warning_block = True
                continue
            if in_warning_block:
                if line.startswith("  ") and line.strip():
                    warnings.append(line.strip())
                    continue
                in_warning_block = False
            if "warning:" in line.lower():
                warnings.append(line.strip())
        return warnings

    def _parse_errors(self, stderr: str, stdout: str = "") -> List[str]:
        """Parse errors from compiler output.

        Only lines containing an error marker ('error', 'Error', '✗')
        are treated as errors; the full stdout is preserved separately
        by callers in "raw_output".
        """
        errors = []
        for source in (stderr, stdout):
            for line in source.split("\n"):
                if any(marker in line for marker in ERROR_MARKERS):
                    errors.append(line.strip())
        return errors

    def _parse_coverage(self, output: str) -> float:
        """Parse coverage percentage (the CLI prints e.g. 'Traceability Coverage: 85.0%')."""
        for line in output.split("\n"):
            if "coverage:" in line.lower():
                match = re.search(r"([0-9]+(?:\.[0-9]+)?)\s*%", line)
                if match:
                    return float(match.group(1))
        return 0.0

    def _count_traces(self, output: str) -> int:
        """Count total traces.

        With `--matrix` the CLI prints one 'from → to' line per trace;
        otherwise fall back to a 'Traces: N' line if present.
        """
        matrix_lines = [line for line in output.split("\n") if " → " in line]
        if matrix_lines:
            return len(matrix_lines)
        for line in output.split("\n"):
            if "Traces:" in line or "traces:" in line:
                value = self._parse_int(line)
                if value is not None:
                    return value
        return 0

    def _parse_trace_issues(self, output: str) -> List[str]:
        """Parse traceability issues printed by `trace --validate`.

        The CLI prints '⚠ Traceability issues found:' followed by
        indented issue lines.
        """
        issues = []
        in_issue_block = False
        for line in output.split("\n"):
            if "issues found:" in line.lower():
                in_issue_block = True
                continue
            if in_issue_block:
                if line.startswith("  ") and line.strip():
                    issues.append(line.strip())
                else:
                    in_issue_block = False
        return issues

    def _parse_element_count(self, output: str, diagram_type: str) -> str:
        """Parse element count from CLI output."""
        patterns = {
            "operational": r"Activities: (\d+)",
            "functional": r"Functions: (\d+)",
            "component": r"Components: (\d+)",
            "sequence": r"Messages: (\d+)",
            "state-machine": r"States: (\d+)",
            "physical": r"Nodes: (\d+)",
            "class": r"Classes: (\d+)",
            "tree": r"Nodes: (\d+)",
            "capability": r"Capabilities: (\d+)",
            "functional-chain": r"Functions: (\d+)"
        }

        pattern = patterns.get(diagram_type)
        if pattern:
            match = re.search(pattern, output)
            if match:
                return match.group(1)

        return "N/A"
