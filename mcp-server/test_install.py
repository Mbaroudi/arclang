#!/usr/bin/env python3
"""Check an installation: the package imports and the compiler answers."""

import asyncio
import sys


def main() -> int:
    try:
        from arclang_mcp import ArcLangMCPServer, __version__
        from arclang_mcp.server import MINIMUM_COMPILER_VERSION, is_at_least
    except Exception as failure:  # an install check reports any import failure
        print(f"FAILED: the server does not import: {failure}")
        return 1
    print(f"arclang-mcp {__version__} imports")

    server = ArcLangMCPServer()
    print(f"{len(server.tools)} tools, {len(server.list_resources())} resources")

    run = asyncio.run(server.compiler.version())
    if not run.ok:
        print(f"FAILED: the compiler does not run: {run.stderr.strip()}")
        print("Build it (cargo build --release) and put it on the PATH, or set ARCLANG_BINARY.")
        return 1
    compiler = run.stdout.strip()
    print(f"compiler: {compiler}")
    if not is_at_least(compiler, MINIMUM_COMPILER_VERSION):
        print(f"FAILED: this server is written for arclang {MINIMUM_COMPILER_VERSION} or later,")
        print(f"and '{compiler}'")
        print(
            f"is the one it would run ({server.compiler.binary_path}). Its tools would be refused"
        )
        print("or answer for an older language. Install the current build (./install-arclang.sh")
        print("from the repository root) or set ARCLANG_BINARY to it.")
        return 1
    print("Ready. See QUICKSTART.md to connect a client.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
