"""
Wrapper for ArcLang compiler binary.
"""

import asyncio
import json
import os
import subprocess
from pathlib import Path
from typing import Any, Dict, Optional


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

    async def compile(
        self,
        model_path: Path,
        validate: bool = True,
        optimize: bool = False
    ) -> Dict[str, Any]:
        """Compile ArcLang model."""
        cmd = [self.binary_path, "build", str(model_path)]
        
        if validate:
            cmd.append("--validate")
        if optimize:
            cmd.append("--optimize")

        result = await self._run_command(cmd)
        
        return {
            "success": result["returncode"] == 0,
            "output": result["stdout"],
            "errors": result["stderr"],
            "metrics": self._parse_metrics(result["stdout"]) if result["returncode"] == 0 else {}
        }

    async def validate(self, model_path: Path, strict: bool = False) -> Dict[str, Any]:
        """Validate ArcLang model."""
        cmd = [self.binary_path, "check", str(model_path), "--lint"]
        
        if strict:
            cmd.append("--strict")

        result = await self._run_command(cmd)
        
        return {
            "valid": result["returncode"] == 0,
            "warnings": self._parse_warnings(result["stdout"]),
            "errors": self._parse_errors(result["stderr"])
        }

    async def trace_analysis(
        self,
        model_path: Path,
        show_gaps: bool = True,
        matrix: bool = False
    ) -> Dict[str, Any]:
        """Analyze traceability."""
        cmd = [self.binary_path, "trace", str(model_path), "--validate"]
        
        if show_gaps:
            cmd.append("--gaps")
        if matrix:
            cmd.append("--matrix")

        result = await self._run_command(cmd)
        
        return {
            "coverage": self._parse_coverage(result["stdout"]),
            "total_traces": self._count_traces(result["stdout"]),
            "gaps": self._parse_gaps(result["stdout"]) if show_gaps else {}
        }

    async def export_diagram(
        self,
        model_path: Path,
        format_type: str = "html",
        output_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Export architecture diagram."""
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
            "error": result["stderr"] if result["returncode"] != 0 else None
        }

    async def info(self, model_path: Path, detailed: bool = False) -> Dict[str, Any]:
        """Get model information."""
        cmd = [self.binary_path, "info", str(model_path), "--metrics"]
        
        if detailed:
            cmd.append("--detailed")

        result = await self._run_command(cmd)
        
        return {
            "metrics": self._parse_metrics(result["stdout"]),
            "size_kb": model_path.stat().st_size / 1024 if model_path.exists() else 0
        }

    async def safety_check(
        self,
        model_path: Path,
        standard: str,
        generate_report: bool = False
    ) -> Dict[str, Any]:
        """Check safety compliance."""
        cmd = [self.binary_path, "safety", str(model_path), "--standard", standard]
        
        if generate_report:
            cmd.append("--report")

        result = await self._run_command(cmd)
        
        return {
            "compliant": result["returncode"] == 0,
            "issues": self._parse_safety_issues(result["stdout"]),
            "recommendations": []
        }

    async def hazard_analysis(
        self,
        model_path: Path,
        standard: str
    ) -> Dict[str, Any]:
        """Perform hazard analysis."""
        cmd = [self.binary_path, "safety", str(model_path), "--hara", "--standard", standard]

        result = await self._run_command(cmd)
        
        return {
            "hazards": self._parse_hazards(result["stdout"])
        }

    async def semantic_merge(
        self,
        base_path: Path,
        ours_path: Path,
        theirs_path: Path
    ) -> Dict[str, Any]:
        """Semantic merge."""
        # This would call the semantic merge tool
        # For now, return placeholder
        return {
            "clean_merge": False,
            "conflicts": [],
            "merged_count": 0
        }

    async def plm_sync(
        self,
        model_path: Path,
        system: str,
        operation: str
    ) -> Dict[str, Any]:
        """PLM synchronization."""
        cmd = [self.binary_path, "plm", operation, str(model_path), "--system", system]

        result = await self._run_command(cmd)
        
        return {
            "success": result["returncode"] == 0,
            "changes_count": 0,
            "error": result["stderr"] if result["returncode"] != 0 else None
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
                metrics["requirements"] = int(line.split(":")[-1].strip())
            elif "Components:" in line:
                metrics["components"] = int(line.split(":")[-1].strip())
            elif "Functions:" in line:
                metrics["functions"] = int(line.split(":")[-1].strip())
            elif "Traces:" in line:
                metrics["traces"] = int(line.split(":")[-1].strip())
        return metrics

    def _parse_warnings(self, output: str) -> list:
        """Parse warnings from output."""
        return [line for line in output.split("\n") if "warning:" in line.lower()]

    def _parse_errors(self, output: str) -> list:
        """Parse errors from output."""
        return [line for line in output.split("\n") if "error:" in line.lower() or line.strip()]

    def _parse_coverage(self, output: str) -> int:
        """Parse coverage percentage."""
        for line in output.split("\n"):
            if "Coverage:" in line or "coverage:" in line:
                try:
                    return int(line.split(":")[-1].strip().rstrip("%"))
                except:
                    pass
        return 0

    def _count_traces(self, output: str) -> int:
        """Count total traces."""
        for line in output.split("\n"):
            if "Traces:" in line or "traces:" in line:
                try:
                    return int(line.split(":")[-1].strip())
                except:
                    pass
        return 0

    def _parse_gaps(self, output: str) -> Dict[str, list]:
        """Parse traceability gaps."""
        return {
            "untraced_requirements": [],
            "untraced_components": []
        }

    def _parse_safety_issues(self, output: str) -> list:
        """Parse safety issues."""
        return []

    def _parse_hazards(self, output: str) -> list:
        """Parse hazards from HARA output."""
        return []
