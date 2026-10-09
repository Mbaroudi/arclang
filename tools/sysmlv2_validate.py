#!/usr/bin/env python3
"""Validate SysML v2 textual files with the OMG pilot implementation.

Runs each file through the SysML v2 Jupyter kernel (the OMG pilot
implementation) and reports the parser/validator diagnostics it produces.
Exit code 0 when every file is accepted, 1 otherwise.

Setup (once):
    conda create -n sysml-pilot -c conda-forge jupyter-sysml-kernel jupyter_client "openjdk>=21" -y
Usage:
    conda activate sysml-pilot
    python tools/sysmlv2_validate.py file.sysml [...]

The kernel launches plain `java`; this script puts the environment's JDK
(>= 21 is required by the pilot) first on PATH before starting it. Parsing
and validation run locally — nothing is sent to the remote API the kernel
spec mentions (that is only used by `%publish`).
"""
import os

_prefix = os.environ.get("CONDA_PREFIX")
if _prefix:
    os.environ["PATH"] = os.path.join(_prefix, "lib", "jvm", "bin") + os.pathsep + os.environ.get("PATH", "")
import subprocess
import sys
import queue
from jupyter_client import KernelManager


def is_diagnostic(line: str) -> bool:
    text = line.strip()
    return text.startswith(("ERROR:", "WARNING:")) and "line :" in text


def validate(code: str, timeout: float = 120.0) -> list[str]:
    """Return the diagnostics the pilot reports for `code` (empty = accepted)."""
    manager = KernelManager(kernel_name="sysml")
    # The kernel process chatters on its own stderr (JVM, log4j, channel
    # loops); diagnostics arrive through the iopub channel, so silence it.
    manager.start_kernel(stderr=subprocess.DEVNULL)
    client = manager.client()
    client.start_channels()
    try:
        client.wait_for_ready(timeout=timeout)
        msg_id = client.execute(code)
        diagnostics: list[str] = []
        while True:
            try:
                message = client.get_iopub_msg(timeout=timeout)
            except queue.Empty:
                diagnostics.append("timeout waiting for the pilot kernel")
                break
            if message.get("parent_header", {}).get("msg_id") != msg_id:
                continue
            kind = message["msg_type"]
            content = message["content"]
            if kind == "stream" and content.get("name") == "stderr":
                # The kernel also logs library loading and JVM notices on
                # stderr; keep only the validator's own diagnostics.
                for line in content["text"].splitlines():
                    if is_diagnostic(line):
                        diagnostics.append(line.rstrip())
            elif kind == "error":
                diagnostics.append("\n".join(content.get("traceback", [])) or content.get("evalue", "error"))
            elif kind == "status" and content.get("execution_state") == "idle":
                break
        reply = client.get_shell_msg(timeout=timeout)
        if reply["content"].get("status") == "error" and not diagnostics:
            diagnostics.append(reply["content"].get("evalue", "error"))
        return [d for d in diagnostics if d.strip()]
    finally:
        client.stop_channels()
        manager.shutdown_kernel(now=True)


def main(paths: list[str]) -> int:
    failed = 0
    for path in paths:
        with open(path, encoding="utf-8") as handle:
            code = handle.read()
        diagnostics = validate(code)
        if diagnostics:
            failed += 1
            print(f"FAIL {path}")
            for line in diagnostics:
                print("   ", line)
        else:
            print(f"OK   {path}")
    return 1 if failed else 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1:]))
