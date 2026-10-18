"""
Fixtures: the tests run the real `arclang` binary, never a stand-in.

The binary is ARCLANG_BINARY, else the repository's release or debug build,
else `arclang` on the PATH. Without one the tests are skipped: they would
prove nothing.
"""

import os
import shutil
from pathlib import Path
from typing import Optional

import pytest

from arclang_mcp import ArcLangMCPServer
from arclang_mcp.resources.syntax_guide import RESOURCES

REPOSITORY = Path(__file__).resolve().parents[2]


def find_binary() -> Optional[str]:
    given = os.getenv("ARCLANG_BINARY")
    if given:
        return given
    for build in ("release", "debug"):
        candidate = REPOSITORY / "target" / build / "arclang"
        if candidate.is_file():
            return str(candidate)
    return shutil.which("arclang")


@pytest.fixture(scope="session")
def binary() -> str:
    found = find_binary()
    if not found and os.getenv("CI"):
        pytest.fail("no arclang binary in CI: nothing would be tested")
    if not found:
        pytest.skip("no arclang binary: build the compiler or set ARCLANG_BINARY")
    return found


@pytest.fixture
def workspace(tmp_path: Path, binary: str, monkeypatch: pytest.MonkeyPatch) -> Path:
    """An empty workspace, with no API key so that nothing can leave the machine."""
    monkeypatch.setenv("ARCLANG_BINARY", binary)
    monkeypatch.setenv("ARCLANG_WORKSPACE", str(tmp_path))
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    monkeypatch.delenv("TYPESAFE_API_KEY", raising=False)
    return tmp_path


@pytest.fixture
def server(workspace: Path) -> ArcLangMCPServer:
    return ArcLangMCPServer(workspace)


@pytest.fixture
def model(workspace: Path) -> Path:
    """A copy of the reference model in the workspace."""
    target = workspace / "braking.arc"
    shutil.copy(RESOURCES / "reference_model.arc", target)
    return target


@pytest.fixture
def broken(workspace: Path) -> Path:
    """A model in the syntax the server used to teach: the compiler refuses it."""
    target = workspace / "broken.arc"
    target.write_text('model M {\n  requirements system {\n    req REQ-001 "Title" { }\n  }\n}\n')
    return target
