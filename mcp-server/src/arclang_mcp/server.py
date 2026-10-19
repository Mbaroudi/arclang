"""
Main MCP Server implementation for ArcLang.

The server is a thin layer over the `arclang` binary: each tool is a
subcommand of the compiler (see compiler/wrapper.py) and returns what the
compiler printed. The tool definitions live with their handlers in
`tools/`; this module only lists them and routes the calls.
"""

import asyncio
import logging
import os
import re
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp import types
from mcp.server import Server
from mcp.types import Resource, TextContent, Tool

from .compiler.wrapper import ArcLangCompiler
from .resources.syntax_guide import get_grammar, get_syntax_rules, reference_model
from .tools import (
    batch,
    core,
    edit,
    expert,
    generation,
    integration,
    safety,
    transform,
    workbench,
)
from .tools.batch import BatchTools
from .tools.common import check_arguments, mime_type
from .tools.core import CoreTools
from .tools.edit import EditTools
from .tools.expert import ExpertTools
from .tools.generation import GenerationTools
from .tools.integration import IntegrationTools
from .tools.safety import SafetyTools
from .tools.transform import TransformTools
from .tools.workbench import WorkbenchTools
from .utils.config import load_config

logger = logging.getLogger(__name__)

SYNTAX_RULES_URI = "arclang://syntax-rules"
REFERENCE_MODEL_URI = "arclang://reference-model"
METAMODEL_URI = "arclang://metamodel"
GRAMMAR_URI = "arclang://grammar"
SERVER_NAME = "arclang-mcp"
__version__ = "4.3.0"
# The oldest `arclang --version` whose command line this server is written
# for (`workbench` and `edit --request`, on top of `check --json`, `rules`,
# `transition`, `import -f sysml`). An older binary accepts another language
# and lacks commands: every answer says so.
MINIMUM_COMPILER_VERSION = "4.3.0"

Handler = Callable[[str, Dict[str, Any]], Awaitable[str]]


def version_tuple(printed: str) -> Optional["tuple[int, ...]"]:
    """The numbers of `arclang 4.1.0` (or of `4.1.0`), or None when there are none."""
    match = re.search(r"(\d+(?:\.\d+)*)\s*$", printed.strip())
    return tuple(int(part) for part in match.group(1).split(".")) if match else None


def is_at_least(printed: str, minimum: str) -> bool:
    """Whether what `arclang --version` printed is `minimum` or later. Unreadable is not."""
    found = version_tuple(printed)
    wanted = version_tuple(minimum)
    return found is not None and wanted is not None and found >= wanted


class ArcLangMCPServer:
    """MCP Server for the ArcLang compiler."""

    def __init__(self, workspace_root: Optional[Path] = None):
        """
        Initialize ArcLang MCP Server.

        Args:
            workspace_root: Root directory for ArcLang models
        """
        self.workspace_root = (workspace_root or Path.cwd()).resolve()
        self.config = load_config(self.workspace_root)
        self.compiler = ArcLangCompiler(self.config.get("compiler", {}))
        self.compiler.workspace_root = self.workspace_root

        self.core_tools = CoreTools(self.compiler, self.workspace_root)
        self.edit_tools = EditTools(self.compiler, self.workspace_root)
        self.workbench_tools = WorkbenchTools(self.compiler, self.workspace_root)
        self.generation_tools = GenerationTools(self.compiler, self.config)
        self.safety_tools = SafetyTools(self.compiler, self.config)
        self.integration_tools = IntegrationTools(self.compiler, self.config)
        self.transform_tools = TransformTools(self.compiler, self.workspace_root)
        self.expert_tools = ExpertTools(self.compiler, self.workspace_root)
        self.batch_tools = BatchTools(self.compiler, self.generation_tools, self.workspace_root)
        self.mbse_expert = self.expert_tools.expert

        groups = [
            (core.TOOLS, self.core_tools.execute),
            (edit.TOOLS, self.edit_tools.execute),
            (workbench.TOOLS, self.workbench_tools.execute),
            (transform.TOOLS, self.transform_tools.execute),
            (generation.TOOLS, self.generation_tools.execute),
            (safety.TOOLS, self.safety_tools.execute),
            (integration.TOOLS, self.integration_tools.execute),
            (expert.TOOLS, self.expert_tools.execute),
            (batch.TOOLS, self.batch_tools.execute),
        ]
        self.tools: List[Tool] = [tool for tools, _ in groups for tool in tools]
        self.handlers: Dict[str, Handler] = {
            tool.name: handler for tools, handler in groups for tool in tools
        }

        self.tools_by_name: Dict[str, Tool] = {tool.name: tool for tool in self.tools}
        self._compiler_note: Optional[str] = None
        self.server = self._build_server()

    def list_resources(self) -> List[Resource]:
        """The resources this server can serve from where it runs."""
        resources = [
            Resource(
                uri=SYNTAX_RULES_URI,
                name="ArcLang Syntax Rules",
                description="Read this before writing ArcLang: the rules most often broken, "
                "and a model that compiles and passes the production gate.",
                mimeType="text/markdown",
            ),
            Resource(
                uri=REFERENCE_MODEL_URI,
                name="ArcLang Reference Model",
                description="A complete .arc model, compiled and gated by the server's test suite.",
                mimeType="text/plain",
            ),
            Resource(
                uri=METAMODEL_URI,
                name="ArcLang Metamodel",
                description="Element kinds, typed attributes, enumerations, units and trace "
                "rules, printed by the compiler this server runs.",
                mimeType="text/markdown",
            ),
        ]
        if get_grammar() is not None:
            resources.append(
                Resource(
                    uri=GRAMMAR_URI,
                    name="ArcLang Grammar",
                    description="The canonical grammar (spec/GRAMMAR.ebnf).",
                    mimeType="text/plain",
                )
            )
        return resources

    async def read_resource(self, uri: str) -> str:
        """The content of a resource."""
        if uri == SYNTAX_RULES_URI:
            return get_syntax_rules()
        if uri == REFERENCE_MODEL_URI:
            return reference_model()
        if uri == METAMODEL_URI:
            run = await self.compiler.metamodel()
            if not run.ok:
                raise ValueError(f"the compiler did not print its metamodel: {run.output}")
            return run.stdout
        grammar = get_grammar()
        if uri == GRAMMAR_URI and grammar is not None:
            return grammar
        raise ValueError(f"Unknown resource: {uri}")

    async def call_tool(self, name: str, arguments: Dict[str, Any]) -> str:
        """Run a tool. An unknown tool or a failing handler raises."""
        handler = self.handlers.get(name)
        if handler is None:
            raise ValueError(f"Unknown tool: {name}")
        arguments = arguments or {}
        check_arguments(self.tools_by_name[name], arguments)
        answer = await handler(name, arguments)
        note = await self.compiler_note()
        return f"{answer}\n\n{note}" if note else answer

    async def compiler_note(self) -> str:
        """A warning when the compiler is older than the one this server is written for.

        An older `arclang` left on the PATH accepts another language and
        lacks commands: every answer says so rather than let it pass.
        """
        if self._compiler_note is None:
            run = await self.compiler.version()
            found = run.stdout.strip()
            if not run.ok:
                # A compiler that cannot run says so in every answer; the
                # version is asked again next time rather than remembered.
                return ""
            if is_at_least(found, MINIMUM_COMPILER_VERSION):
                self._compiler_note = ""
            else:
                self._compiler_note = (
                    f"WARNING: the compiler at '{self.compiler.binary_path}' is '{found}'; "
                    f"this server is written for arclang {MINIMUM_COMPILER_VERSION} or later. "
                    "Answers may be those of an older language. Install the current build or "
                    "set ARCLANG_BINARY."
                )
        return self._compiler_note

    async def _answer(self, name: str, arguments: Optional[Dict[str, Any]]) -> "tuple[str, bool]":
        """A tool's text and whether it is an error. A failure is reported, never swallowed."""
        try:
            return await self.call_tool(name, arguments or {}), False
        except Exception as failure:
            logger.exception("Error executing tool %s", name)
            note = await self.compiler_note()
            text = f"Error: {failure}"
            return (f"{text}\n\n{note}" if note else text), True

    def _build_server(self) -> Server:
        """The MCP server, for either generation of the Python SDK.

        SDK 1.x registers handlers with decorators; SDK 2.x takes them as
        constructor arguments and returns result objects.
        """
        if hasattr(Server, "list_tools"):
            return self._build_server_v1()
        return self._build_server_v2()

    def _build_server_v1(self) -> Server:
        server = Server(SERVER_NAME, version=__version__)

        @server.list_resources()
        async def list_resources() -> List[Resource]:
            return self.list_resources()

        @server.read_resource()
        async def read_resource(uri: Any) -> str:
            return await self.read_resource(str(uri))

        @server.list_tools()
        async def list_tools() -> List[Tool]:
            return self.tools

        @server.call_tool()
        async def call_tool(name: str, arguments: Dict[str, Any]) -> List[TextContent]:
            text, is_error = await self._answer(name, arguments)
            if is_error:
                # SDK 1.x marks the result as an error when the handler raises.
                raise RuntimeError(text)
            return [TextContent(type="text", text=text)]

        return server

    def _build_server_v2(self) -> Server:
        async def on_list_resources(context: Any, params: Any) -> Any:
            return types.ListResourcesResult(resources=self.list_resources())

        async def on_read_resource(context: Any, params: Any) -> Any:
            uri = str(params.uri)
            mime_types = {
                str(resource.uri): mime_type(resource) for resource in self.list_resources()
            }
            content = types.TextResourceContents(
                uri=uri, mimeType=mime_types.get(uri), text=await self.read_resource(uri)
            )
            return types.ReadResourceResult(contents=[content])

        async def on_list_tools(context: Any, params: Any) -> Any:
            return types.ListToolsResult(tools=self.tools)

        async def on_call_tool(context: Any, params: Any) -> Any:
            text, is_error = await self._answer(params.name, params.arguments)
            return types.CallToolResult(
                content=[TextContent(type="text", text=text)], isError=is_error
            )

        return Server(
            SERVER_NAME,
            version=__version__,
            on_list_resources=on_list_resources,
            on_read_resource=on_read_resource,
            on_list_tools=on_list_tools,
            on_call_tool=on_call_tool,
        )

    async def run(self) -> None:
        """Run the MCP server."""
        from mcp.server.stdio import stdio_server

        async with stdio_server() as (read_stream, write_stream):
            await self.server.run(
                read_stream,
                write_stream,
                self.server.create_initialization_options(),
            )


def main() -> None:
    """Main entry point for the MCP server."""
    workspace = os.getenv("ARCLANG_WORKSPACE")
    workspace_root = Path(workspace) if workspace else Path.cwd()

    # stdout carries the protocol: logs go to stderr (the logging default).
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    server = ArcLangMCPServer(workspace_root)
    asyncio.run(server.run())


if __name__ == "__main__":
    main()
