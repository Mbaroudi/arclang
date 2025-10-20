"""
Main MCP Server implementation for ArcLang.
"""

import asyncio
import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from mcp.server import Server
from mcp.types import Tool, TextContent, ImageContent, Resource

from .tools.core import CoreTools
from .resources.syntax_guide import get_syntax_rules
from .tools.generation import GenerationTools
from .tools.safety import SafetyTools
from .tools.integration import IntegrationTools
from .compiler.wrapper import ArcLangCompiler
from .utils.config import load_config

logger = logging.getLogger(__name__)


class ArcLangMCPServer:
    """MCP Server for ArcLang MBSE platform."""

    def __init__(self, workspace_root: Optional[Path] = None):
        """
        Initialize ArcLang MCP Server.

        Args:
            workspace_root: Root directory for ArcLang models
        """
        self.workspace_root = workspace_root or Path.cwd()
        self.config = load_config(self.workspace_root)
        self.compiler = ArcLangCompiler(self.config.get("compiler", {}))
        
        # Initialize tool groups
        self.core_tools = CoreTools(self.compiler, self.workspace_root)
        self.generation_tools = GenerationTools(self.compiler, self.config)
        self.safety_tools = SafetyTools(self.compiler, self.config)
        self.integration_tools = IntegrationTools(self.compiler, self.config)
        
        # Create MCP server
        self.server = Server("arclang-mcp")
        self._register_tools()

    def _register_tools(self) -> None:
        """Register all available tools and resources with the MCP server."""
        
        # Register syntax rules as a resource
        @self.server.list_resources()
        async def list_resources() -> List[Resource]:
            """List available resources including syntax rules."""
            return [
                Resource(
                    uri="arclang://syntax-rules",
                    name="ArcLang Syntax Rules",
                    description="Mandatory syntax rules for generating ArcLang models. AI clients MUST follow these rules.",
                    mimeType="text/markdown"
                )
            ]
        
        @self.server.read_resource()
        async def read_resource(uri: str) -> str:
            """Read resource content."""
            if uri == "arclang://syntax-rules":
                return get_syntax_rules()
            raise ValueError(f"Unknown resource: {uri}")
        
        # Core tools
        @self.server.list_tools()
        async def list_tools() -> List[Tool]:
            """List all available tools."""
            return [
                # Core compilation and validation
                Tool(
                    name="arclang_compile",
                    description="Compile ArcLang model to Capella XML format. Validates syntax and semantics.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "validate": {
                                "type": "boolean",
                                "description": "Run validation checks",
                                "default": True
                            },
                            "optimize": {
                                "type": "boolean",
                                "description": "Enable optimizations",
                                "default": False
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                Tool(
                    name="arclang_validate",
                    description="Validate ArcLang model syntax and semantics without compilation.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "strict": {
                                "type": "boolean",
                                "description": "Enable strict validation mode",
                                "default": False
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                Tool(
                    name="arclang_trace_analysis",
                    description="Analyze traceability coverage and find gaps in requirements/component traces.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file or directory"
                            },
                            "show_gaps": {
                                "type": "boolean",
                                "description": "Show untraced elements",
                                "default": True
                            },
                            "matrix": {
                                "type": "boolean",
                                "description": "Generate traceability matrix",
                                "default": False
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                Tool(
                    name="arclang_export_diagram",
                    description="Generate architecture diagram from ArcLang model.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "format": {
                                "type": "string",
                                "enum": ["capella", "json", "yaml", "xml", "markdown", "html", "pdf"],
                                "description": "Output diagram format",
                                "default": "html"
                            },
                            "output_path": {
                                "type": "string",
                                "description": "Output file path"
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                Tool(
                    name="arclang_info",
                    description="Get model metrics and statistics (requirements count, components, coverage, etc.).",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file or directory"
                            },
                            "detailed": {
                                "type": "boolean",
                                "description": "Include detailed metrics",
                                "default": False
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                # Generation tools
                Tool(
                    name="arclang_generate_requirement",
                    description="""Generate ArcLang requirement from natural language description.
                    
MANDATORY SYNTAX: Use 'req ID \"Title\" { }' format inside 'requirements stakeholder/system/safety { }' block.
CORRECT: requirements stakeholder { req STK-001 \"Title\" { description: \"Text\" } }
WRONG: requirement \"REQ-001\" { } or req { id: \"REQ-001\" }""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "description": {
                                "type": "string",
                                "description": "Natural language description of requirement"
                            },
                            "safety_level": {
                                "type": "string",
                                "enum": ["ASIL_A", "ASIL_B", "ASIL_C", "ASIL_D", "DAL_A", "DAL_B", "DAL_C", "DAL_D", "SIL_1", "SIL_2", "SIL_3", "SIL_4"],
                                "description": "Safety integrity level"
                            },
                            "priority": {
                                "type": "string",
                                "enum": ["Critical", "High", "Medium", "Low"],
                                "description": "Requirement priority",
                                "default": "High"
                            }
                        },
                        "required": ["description"]
                    }
                ),
                
                Tool(
                    name="arclang_generate_component",
                    description="""Generate ArcLang component architecture from description.
                    
MANDATORY SYNTAX: Use 'component Name \"Display\" { }' inside 'architecture logical/physical { }' block.
CORRECT: component SensorSubsystem \"Sensor\" { provides interface IData { } }
WRONG: component \"Name\" { port \"input\" { } } or component { name: \"Name\" }""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "description": {
                                "type": "string",
                                "description": "Natural language description of component"
                            },
                            "component_type": {
                                "type": "string",
                                "enum": ["Logical", "Physical", "Operational"],
                                "description": "Component type",
                                "default": "Logical"
                            },
                            "safety_level": {
                                "type": "string",
                                "description": "Safety integrity level (optional)"
                            }
                        },
                        "required": ["description"]
                    }
                ),
                
                Tool(
                    name="arclang_suggest_architecture",
                    description="Get AI-powered architecture suggestions based on requirements.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "requirements": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "List of requirement descriptions"
                            },
                            "domain": {
                                "type": "string",
                                "enum": ["automotive", "aerospace", "defense", "industrial"],
                                "description": "Application domain"
                            }
                        },
                        "required": ["requirements"]
                    }
                ),
                
                # Safety tools
                Tool(
                    name="arclang_safety_check",
                    description="Validate model against safety standards (ISO 26262, DO-178C, IEC 61508).",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "standard": {
                                "type": "string",
                                "enum": ["iso26262", "do178c", "iec61508"],
                                "description": "Safety standard to validate against"
                            },
                            "generate_report": {
                                "type": "boolean",
                                "description": "Generate detailed HTML report",
                                "default": False
                            }
                        },
                        "required": ["model_path", "standard"]
                    }
                ),
                
                Tool(
                    name="arclang_hazard_analysis",
                    description="Perform HARA (Hazard Analysis and Risk Assessment) on model.",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "standard": {
                                "type": "string",
                                "enum": ["iso26262", "iec61508"],
                                "description": "Safety standard",
                                "default": "iso26262"
                            }
                        },
                        "required": ["model_path"]
                    }
                ),
                
                # Integration tools
                Tool(
                    name="arclang_git_merge",
                    description="Semantic merge assistance for ArcLang models (resolves conflicts by component ID).",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "base_path": {
                                "type": "string",
                                "description": "Path to base version"
                            },
                            "ours_path": {
                                "type": "string",
                                "description": "Path to our version"
                            },
                            "theirs_path": {
                                "type": "string",
                                "description": "Path to their version"
                            }
                        },
                        "required": ["base_path", "ours_path", "theirs_path"]
                    }
                ),
            ]

        @self.server.call_tool()
        async def call_tool(name: str, arguments: Dict[str, Any]) -> List[TextContent | ImageContent]:
            """Execute a tool with given arguments."""
            try:
                # Route to appropriate tool handler
                if name.startswith("arclang_compile") or name in ["arclang_validate", "arclang_trace_analysis", "arclang_export_diagram", "arclang_info"]:
                    result = await self.core_tools.execute(name, arguments)
                elif name.startswith("arclang_generate") or name == "arclang_suggest_architecture":
                    result = await self.generation_tools.execute(name, arguments)
                elif name.startswith("arclang_safety") or name == "arclang_hazard_analysis":
                    result = await self.safety_tools.execute(name, arguments)
                elif name.startswith("arclang_git") or name.startswith("arclang_plm"):
                    result = await self.integration_tools.execute(name, arguments)
                else:
                    raise ValueError(f"Unknown tool: {name}")
                
                return [TextContent(type="text", text=result)]
            
            except Exception as e:
                logger.error(f"Error executing tool {name}: {e}")
                return [TextContent(type="text", text=f"Error: {str(e)}")]

    async def run(self) -> None:
        """Run the MCP server."""
        from mcp.server.stdio import stdio_server
        
        async with stdio_server() as (read_stream, write_stream):
            await self.server.run(
                read_stream,
                write_stream,
                self.server.create_initialization_options()
            )


def main() -> None:
    """Main entry point for the MCP server."""
    import os
    
    # Get workspace from environment or use current directory
    workspace = os.getenv("ARCLANG_WORKSPACE")
    workspace_root = Path(workspace) if workspace else Path.cwd()
    
    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    
    # Create and run server
    server = ArcLangMCPServer(workspace_root)
    asyncio.run(server.run())


if __name__ == "__main__":
    main()
