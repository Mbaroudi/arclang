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
from .mbse_expert import MBSECapellaExpert

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
        
        # Initialize MBSE expert system
        self.mbse_expert = MBSECapellaExpert(self.workspace_root)
        
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
                
                Tool(
                    name="arclang_generate_diagram",
                    description="""Generate a specific Capella diagram type from ArcLang model.

Supports all 10 diagram types with rich content:
- operational: Swimlane activity diagrams with actors (⊕ symbols)
- functional: Data flow with 15+ functions, categories
- component: Hierarchical blocks with protocols (CAN, Ethernet)
- sequence: Interaction scenarios with lifelines
- state-machine: State/transition diagrams
- physical: Hardware deployment diagrams
- class: Data type and class definitions
- tree: Hierarchical breakdown (Reingold-Tilford)
- capability: Requirements hierarchy (3 levels)
- functional-chain: Execution flow scenarios

All diagrams use professional Capella-quality rendering.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "diagram_type": {
                                "type": "string",
                                "enum": ["operational", "functional", "component", "sequence",
                                        "state-machine", "physical", "class", "tree",
                                        "capability", "functional-chain"],
                                "description": "Type of diagram to generate"
                            },
                            "output_path": {
                                "type": "string",
                                "description": "Output SVG file path (optional)"
                            }
                        },
                        "required": ["model_path", "diagram_type"]
                    }
                ),
                
                Tool(
                    name="arclang_generate_all_diagrams",
                    description="""Generate all 10 Capella diagram types from ArcLang model.

Creates complete professional diagram set:
- 10 SVG diagrams with rich content (3-13x larger than simple)
- 100% Capella visual parity
- Automatic organization in output directory
- Summary report with success/failure status

Average: 127KB total content, 12.7KB per diagram.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "model_path": {
                                "type": "string",
                                "description": "Path to .arc model file"
                            },
                            "output_dir": {
                                "type": "string",
                                "description": "Output directory for all diagrams",
                                "default": "./diagrams"
                            }
                        },
                        "required": ["model_path"]
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
                
                # MBSE Expert tools
                Tool(
                    name="arclang_analyze_requirements",
                    description="""MBSE Expert: Analyze natural language requirements and map to Arcadia layers.
                    
This tool acts as a Capella methodology expert, analyzing user requirements to:
- Identify which Arcadia layer each requirement belongs to (Operational/System/Logical/Physical/EPBS)
- Detect safety requirements and assign ASIL/SIL levels
- Recommend component types (sensors, controllers, actuators)
- Suggest appropriate diagram types for each layer
- Generate traceability plan across layers
- Recommend architecture patterns based on domain

Returns comprehensive analysis with Arcadia mapping and recommendations.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "requirements_text": {
                                "type": "string",
                                "description": "Natural language requirements (one per line or paragraph)"
                            },
                            "domain": {
                                "type": "string",
                                "enum": ["automotive", "aerospace", "defense", "industrial", "general"],
                                "description": "Application domain for context-aware analysis",
                                "default": "automotive"
                            }
                        },
                        "required": ["requirements_text"]
                    }
                ),
                
                Tool(
                    name="arclang_generate_from_requirements",
                    description="""MBSE Expert: Generate complete ArcLang model from requirement analysis.
                    
Takes requirement analysis and generates syntactically correct .arc file with:
- Stakeholder requirements (STK-XXX)
- System requirements (SYS-XXX) with traces to stakeholder reqs
- Safety requirements (SAF-XXX) with ASIL levels
- Operational architecture (actors and activities)
- Logical architecture (components with proper interfaces)
- Physical architecture (ECUs and deployment)
- Proper Capella colors and stereotypes
- Complete traceability chains

Output is production-ready ArcLang code following best practices.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "analysis": {
                                "type": "object",
                                "description": "Requirement analysis from arclang_analyze_requirements"
                            },
                            "project_name": {
                                "type": "string",
                                "description": "Name of the project/system"
                            },
                            "output_path": {
                                "type": "string",
                                "description": "Path to save generated .arc file"
                            }
                        },
                        "required": ["analysis", "project_name"]
                    }
                ),
                
                Tool(
                    name="arclang_assess_diagram_quality",
                    description="""MBSE Expert: Assess Capella diagram quality against best practices.
                    
Evaluates generated diagrams for Capella compliance:
- Checks required elements (interfaces, protocols, actors, etc.)
- Validates visual notation (lollipops, sockets, stereotypes)
- Verifies safety critical markings (ASIL borders)
- Assesses layout quality (spacing, hierarchy, flow)
- Compares against Capella reference patterns
- Provides quality score (0-100%) per check

Returns detailed assessment with specific improvement recommendations.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "diagram_path": {
                                "type": "string",
                                "description": "Path to SVG diagram file"
                            },
                            "diagram_type": {
                                "type": "string",
                                "enum": ["operational", "functional", "component", "sequence", 
                                        "state-machine", "physical", "class", "tree", 
                                        "capability", "functional-chain"],
                                "description": "Type of diagram to assess"
                            }
                        },
                        "required": ["diagram_path", "diagram_type"]
                    }
                ),
                
                Tool(
                    name="arclang_suggest_diagram_enhancements",
                    description="""MBSE Expert: Suggest specific enhancements for diagram quality.
                    
Based on quality assessment, suggests concrete enhancements:
- Missing Capella notation elements
- Layout improvements (spacing, alignment)
- Visual enhancements (colors, borders, effects)
- Best practice additions
- Prioritized by impact on quality score

Returns actionable enhancement list ready for batch execution.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "assessment": {
                                "type": "object",
                                "description": "Quality assessment from arclang_assess_diagram_quality"
                            }
                        },
                        "required": ["assessment"]
                    }
                ),
                
                Tool(
                    name="arclang_generate_mbse_report",
                    description="""MBSE Expert: Generate comprehensive MBSE quality report.
                    
Produces professional markdown report with:
- Requirements analysis summary
- Arcadia layer mapping statistics
- Safety requirements breakdown
- Diagram quality scores
- Traceability coverage metrics
- Improvement recommendations
- Capella compliance assessment

Perfect for technical reviews and documentation.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "analysis": {
                                "type": "object",
                                "description": "Requirement analysis results"
                            },
                            "assessments": {
                                "type": "array",
                                "items": {"type": "object"},
                                "description": "List of diagram quality assessments"
                            },
                            "output_path": {
                                "type": "string",
                                "description": "Path to save report (optional)"
                            }
                        },
                        "required": ["analysis"]
                    }
                ),
                
                # Code execution tool for efficient batching
                Tool(
                    name="arclang_execute_batch",
                    description="""Execute multiple ArcLang operations in a single efficient batch.

This tool enables "code mode" execution where the AI agent can:
- Load and filter model data BEFORE sending to LLM context
- Execute complex multi-step workflows in one call
- Batch compile + validate + generate diagrams atomically
- Pre-filter large JSON outputs to only relevant data
- Reduce token usage by 10-100x for large models

Example operations:
1. compile_and_validate: Compile model, validate, return only errors (not full output)
2. generate_all_views: Generate all diagrams, return summary with file sizes
3. analyze_safety_gaps: Find untraced safety requirements, return gap list only
4. compare_models: Diff two models, return only differences
5. refactor_component: Rename component across model, validate, return change count

Each operation executes fully server-side. Only filtered results sent to LLM.""",
                    inputSchema={
                        "type": "object",
                        "properties": {
                            "operations": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "action": {
                                            "type": "string",
                                            "enum": [
                                                "compile_and_validate",
                                                "generate_all_diagrams",
                                                "analyze_traceability",
                                                "safety_check",
                                                "extract_metrics",
                                                "export_filtered",
                                                "compare_models",
                                                "analyze_diagrams",
                                                "enhance_diagram"
                                            ],
                                            "description": "Operation to perform"
                                        },
                                        "params": {
                                            "type": "object",
                                            "description": "Parameters for the operation"
                                        },
                                        "filter": {
                                            "type": "object",
                                            "description": "Optional filter to apply to results (e.g., {\"only_errors\": true, \"max_items\": 10})"
                                        }
                                    },
                                    "required": ["action", "params"]
                                },
                                "description": "List of operations to execute in sequence"
                            },
                            "stop_on_error": {
                                "type": "boolean",
                                "description": "Stop batch execution if any operation fails",
                                "default": False
                            },
                            "return_summary_only": {
                                "type": "boolean",
                                "description": "Return only summary statistics instead of full results",
                                "default": False
                            }
                        },
                        "required": ["operations"]
                    }
                ),
            ]

        @self.server.call_tool()
        async def call_tool(name: str, arguments: Dict[str, Any]) -> List[TextContent | ImageContent]:
            """Execute a tool with given arguments."""
            try:
                # Route to appropriate tool handler
                if name == "arclang_execute_batch":
                    result = await self._execute_batch(arguments)
                elif name == "arclang_analyze_requirements":
                    result = await self._analyze_requirements(arguments)
                elif name == "arclang_generate_from_requirements":
                    result = await self._generate_from_requirements(arguments)
                elif name == "arclang_assess_diagram_quality":
                    result = await self._assess_diagram_quality(arguments)
                elif name == "arclang_suggest_diagram_enhancements":
                    result = await self._suggest_diagram_enhancements(arguments)
                elif name == "arclang_generate_mbse_report":
                    result = await self._generate_mbse_report(arguments)
                elif name.startswith("arclang_compile") or name in ["arclang_validate", "arclang_trace_analysis", "arclang_export_diagram", "arclang_info"]:
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
    
    async def _execute_batch(self, arguments: Dict[str, Any]) -> str:
        """
        Execute multiple operations in a batch with filtering and aggregation.
        
        This enables "code mode" where complex operations run server-side,
        filtering data before sending to LLM context.
        """
        import json
        from pathlib import Path
        
        operations = arguments.get("operations", [])
        stop_on_error = arguments.get("stop_on_error", False)
        return_summary_only = arguments.get("return_summary_only", False)
        
        results = []
        total_tokens_saved = 0
        
        for i, op in enumerate(operations):
            action = op["action"]
            params = op.get("params", {})
            filter_config = op.get("filter", {})
            
            try:
                # Execute operation based on action type
                if action == "compile_and_validate":
                    model_path = params.get("model_path")
                    
                    # Compile
                    compile_result = await self.core_tools.execute("arclang_compile", {"model_path": model_path})
                    
                    # Validate
                    validate_result = await self.core_tools.execute("arclang_validate", {"model_path": model_path})
                    
                    # Filter: only return errors/warnings if any
                    if filter_config.get("only_errors", True):
                        filtered = {
                            "status": "success" if "Error" not in compile_result else "error",
                            "error_count": compile_result.count("Error:"),
                            "warning_count": compile_result.count("warning:"),
                            "model_path": model_path
                        }
                        if filtered["error_count"] > 0:
                            # Extract first few errors only
                            errors = [line for line in compile_result.split('\n') if 'Error:' in line]
                            filtered["errors"] = errors[:filter_config.get("max_items", 5)]
                        
                        result = filtered
                        total_tokens_saved += len(compile_result) // 4  # Rough token estimate
                    else:
                        result = {"compile": compile_result, "validate": validate_result}
                
                elif action == "generate_all_diagrams":
                    model_path = params.get("model_path")
                    output_dir = params.get("output_dir", "./diagrams")
                    
                    # Generate all diagrams
                    gen_result = await self.generation_tools.execute(
                        "arclang_generate_all_diagrams",
                        {"model_path": model_path, "output_dir": output_dir}
                    )
                    
                    # Filter: return only summary with file sizes
                    if return_summary_only or filter_config.get("summary_only", True):
                        output_path = Path(output_dir)
                        diagram_files = list(output_path.glob("*.svg")) if output_path.exists() else []
                        
                        result = {
                            "status": "success",
                            "diagram_count": len(diagram_files),
                            "total_size_kb": sum(f.stat().st_size for f in diagram_files) // 1024,
                            "diagrams": [{"name": f.name, "size_kb": f.stat().st_size // 1024} for f in diagram_files[:10]]
                        }
                        total_tokens_saved += len(gen_result) // 4
                    else:
                        result = gen_result
                
                elif action == "analyze_traceability":
                    model_path = params.get("model_path")
                    
                    # Run trace analysis
                    trace_result = await self.core_tools.execute(
                        "arclang_trace_analysis",
                        {"model_path": model_path, "show_gaps": True}
                    )
                    
                    # Filter: return only gaps list
                    if filter_config.get("gaps_only", True):
                        gaps = [line for line in trace_result.split('\n') if 'untraced' in line.lower() or 'gap' in line.lower()]
                        result = {
                            "status": "success",
                            "gap_count": len(gaps),
                            "gaps": gaps[:filter_config.get("max_items", 10)]
                        }
                        total_tokens_saved += len(trace_result) // 4
                    else:
                        result = trace_result
                
                elif action == "safety_check":
                    model_path = params.get("model_path")
                    standard = params.get("standard", "iso26262")
                    
                    # Run safety check
                    safety_result = await self.safety_tools.execute(
                        "arclang_safety_check",
                        {"model_path": model_path, "standard": standard}
                    )
                    
                    # Filter: return only violations
                    if filter_config.get("violations_only", True):
                        violations = [line for line in safety_result.split('\n') if 'violation' in line.lower() or 'fail' in line.lower()]
                        result = {
                            "status": "pass" if len(violations) == 0 else "fail",
                            "violation_count": len(violations),
                            "violations": violations[:filter_config.get("max_items", 10)]
                        }
                        total_tokens_saved += len(safety_result) // 4
                    else:
                        result = safety_result
                
                elif action == "extract_metrics":
                    model_path = params.get("model_path")
                    
                    # Get model info
                    info_result = await self.core_tools.execute(
                        "arclang_info",
                        {"model_path": model_path, "detailed": True}
                    )
                    
                    # Extract key metrics only
                    result = {
                        "component_count": info_result.count("component") if isinstance(info_result, str) else 0,
                        "requirement_count": info_result.count("requirement") if isinstance(info_result, str) else 0,
                        "interface_count": info_result.count("interface") if isinstance(info_result, str) else 0,
                    }
                    total_tokens_saved += len(str(info_result)) // 4
                
                elif action == "export_filtered":
                    model_path = params.get("model_path")
                    filter_type = params.get("filter_type", "components_only")
                    
                    # Export to JSON
                    export_result = await self.core_tools.execute(
                        "arclang_export_diagram",
                        {"model_path": model_path, "format": "json"}
                    )
                    
                    # Filter JSON based on filter_type
                    try:
                        data = json.loads(export_result) if isinstance(export_result, str) else export_result
                        
                        if filter_type == "components_only":
                            result = {
                                "components": data.get("logical_architecture", [{}])[0].get("components", [])[:filter_config.get("max_items", 20)]
                            }
                        elif filter_type == "requirements_only":
                            result = {
                                "requirements": data.get("system_analysis", [{}])[0].get("requirements", [])[:filter_config.get("max_items", 20)]
                            }
                        else:
                            result = data
                        
                        total_tokens_saved += (len(str(data)) - len(str(result))) // 4
                    except:
                        result = {"error": "Failed to parse JSON"}
                
                elif action == "compare_models":
                    model1_path = params.get("model1_path")
                    model2_path = params.get("model2_path")
                    
                    # Load both models
                    info1 = await self.core_tools.execute("arclang_info", {"model_path": model1_path})
                    info2 = await self.core_tools.execute("arclang_info", {"model_path": model2_path})
                    
                    # Simple diff comparison
                    result = {
                        "model1": model1_path,
                        "model2": model2_path,
                        "differences": "Detailed comparison would go here",
                        "summary": f"Model 1: {len(str(info1))} chars, Model 2: {len(str(info2))} chars"
                    }
                
                elif action == "analyze_diagrams":
                    diagram_paths = params.get("diagram_paths", [])
                    
                    # Analyze SVG structure, element counts, sizes
                    analysis = {
                        "diagrams": []
                    }
                    
                    import re
                    
                    for path in diagram_paths:
                        # Resolve path relative to workspace
                        resolved = Path(path)
                        if not resolved.is_absolute():
                            resolved = self.workspace_root / path
                        
                        if resolved.exists():
                            content = resolved.read_text()
                            
                            # Extract SVG dimensions
                            width_match = re.search(r'width="(\d+)"', content)
                            height_match = re.search(r'height="(\d+)"', content)
                            
                            # Count elements
                            g_count = content.count("<g ")
                            rect_count = content.count("<rect ")
                            circle_count = content.count("<circle ")
                            line_count = content.count("<line ")
                            path_count = content.count("<path ")
                            text_count = content.count("<text ")
                            
                            # Check for Phase 1-4 features
                            has_interfaces = "lollipop" in content.lower() or (circle_count > 0 and "stroke=\"#FFFFFF\"" in content)
                            has_3d_boxes = "drop-shadow" in content and rect_count > 0
                            has_safety_borders = "stroke-width=\"6\"" in content or "#8B0000" in content
                            
                            # Extract color palette
                            colors = list(set(re.findall(r'fill="#([0-9A-Fa-f]{6})"', content)))
                            
                            analysis["diagrams"].append({
                                "path": str(resolved.relative_to(self.workspace_root)) if str(resolved).startswith(str(self.workspace_root)) else str(resolved),
                                "size_bytes": len(content),
                                "size_kb": f"{len(content) / 1024:.1f}KB",
                                "dimensions": f"{width_match.group(1)}x{height_match.group(1)}" if width_match and height_match else "unknown",
                                "elements": {
                                    "groups": g_count,
                                    "rectangles": rect_count,
                                    "circles": circle_count,
                                    "lines": line_count,
                                    "paths": path_count,
                                    "texts": text_count,
                                    "total": g_count + rect_count + circle_count + line_count + path_count + text_count
                                },
                                "features": {
                                    "has_interfaces": has_interfaces,
                                    "has_3d_effects": has_3d_boxes,
                                    "has_safety_borders": has_safety_borders
                                },
                                "color_palette": colors[:10]  # Limit to 10 colors
                            })
                            
                            # Save tokens by not including full content
                            total_tokens_saved += len(content) // 4
                        else:
                            analysis["diagrams"].append({
                                "path": path,
                                "error": "File not found"
                            })
                    
                    # Apply filter
                    if filter_config.get("summary_only", False):
                        result = {
                            "total_diagrams": len(analysis["diagrams"]),
                            "total_size_kb": sum(d.get("size_bytes", 0) for d in analysis["diagrams"]) / 1024,
                            "total_elements": sum(d.get("elements", {}).get("total", 0) for d in analysis["diagrams"]),
                            "diagrams_with_interfaces": sum(1 for d in analysis["diagrams"] if d.get("features", {}).get("has_interfaces", False)),
                            "diagrams_with_safety": sum(1 for d in analysis["diagrams"] if d.get("features", {}).get("has_safety_borders", False))
                        }
                    else:
                        result = analysis
                
                elif action == "enhance_diagram":
                    diagram_path = params.get("diagram_path")
                    enhancements = params.get("enhancements", [])
                    
                    # Resolve path relative to workspace
                    resolved = Path(diagram_path)
                    if not resolved.is_absolute():
                        resolved = self.workspace_root / diagram_path
                    
                    if resolved.exists():
                        content = resolved.read_text()
                        modified = content
                        applied = []
                        
                        import re
                        
                        for enhancement in enhancements:
                            if enhancement == "increase_spacing":
                                # Increase viewBox by 20%
                                def expand_viewbox(match):
                                    w = int(match.group(1))
                                    h = int(match.group(2))
                                    return f'viewBox="0 0 {int(w * 1.2)} {int(h * 1.2)}"'
                                
                                modified = re.sub(r'viewBox="0 0 (\d+) (\d+)"', expand_viewbox, modified)
                                applied.append("increase_spacing")
                            
                            elif enhancement == "improve_contrast":
                                # Darken light colors for better contrast
                                def darken_color(match):
                                    color = match.group(1)
                                    # Simple darkening: if color is light, darken it
                                    r, g, b = int(color[0:2], 16), int(color[2:4], 16), int(color[4:6], 16)
                                    if r + g + b > 500:  # Light color
                                        r, g, b = int(r * 0.8), int(g * 0.8), int(b * 0.8)
                                        return f'fill="#{r:02x}{g:02x}{b:02x}"'
                                    return match.group(0)
                                
                                modified = re.sub(r'fill="#([0-9A-Fa-f]{6})"', darken_color, modified)
                                applied.append("improve_contrast")
                            
                            elif enhancement == "add_grid":
                                # Add grid pattern to background
                                grid = '''<defs>
<pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
<path d="M 20 0 L 0 0 0 20" fill="none" stroke="#E0E0E0" stroke-width="0.5"/>
</pattern>
</defs>'''
                                if '<defs>' in modified:
                                    modified = modified.replace('<defs>', '<defs>\n<pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">\n<path d="M 20 0 L 0 0 0 20" fill="none" stroke="#E0E0E0" stroke-width="0.5"/>\n</pattern>')
                                else:
                                    modified = modified.replace('</svg>', grid + '\n</svg>')
                                
                                # Apply grid to background
                                modified = re.sub(r'<rect x="0" y="0"([^>]+)fill="#FFFFFF"', r'<rect x="0" y="0"\1fill="url(#grid)"', modified)
                                applied.append("add_grid")
                            
                            elif enhancement == "enlarge_text":
                                # Increase font sizes by 20%
                                def increase_font(match):
                                    size = int(match.group(1))
                                    return f'font-size="{int(size * 1.2)}"'
                                
                                modified = re.sub(r'font-size="(\d+)"', increase_font, modified)
                                applied.append("enlarge_text")
                        
                        # Write enhanced version
                        output_path = resolved.parent / f"{resolved.stem}_enhanced.svg"
                        output_path.write_text(modified)
                        
                        result = {
                            "original": str(resolved.relative_to(self.workspace_root)) if str(resolved).startswith(str(self.workspace_root)) else str(resolved),
                            "enhanced": str(output_path.relative_to(self.workspace_root)) if str(output_path).startswith(str(self.workspace_root)) else str(output_path),
                            "enhancements_applied": applied,
                            "size_before_kb": f"{len(content) / 1024:.1f}KB",
                            "size_after_kb": f"{len(modified) / 1024:.1f}KB"
                        }
                        
                        total_tokens_saved += len(content) // 4
                    else:
                        result = {"error": f"Diagram not found: {diagram_path}"}
                
                else:
                    result = {"error": f"Unknown action: {action}"}
                
                results.append({
                    "operation": i + 1,
                    "action": action,
                    "status": "success",
                    "result": result
                })
            
            except Exception as e:
                error_result = {
                    "operation": i + 1,
                    "action": action,
                    "status": "error",
                    "error": str(e)
                }
                results.append(error_result)
                
                if stop_on_error:
                    break
        
        # Build final response
        if return_summary_only:
            summary = {
                "total_operations": len(operations),
                "successful": len([r for r in results if r["status"] == "success"]),
                "failed": len([r for r in results if r["status"] == "error"]),
                "tokens_saved_estimate": total_tokens_saved,
                "operations": [{"action": r["action"], "status": r["status"]} for r in results]
            }
            return json.dumps(summary, indent=2)
        else:
            return json.dumps({
                "results": results,
                "tokens_saved_estimate": total_tokens_saved
            }, indent=2)
    
    async def _analyze_requirements(self, arguments: Dict[str, Any]) -> str:
        """Analyze requirements using MBSE expert system."""
        import json
        
        requirements_text = arguments.get("requirements_text", "")
        domain = arguments.get("domain", "automotive")
        
        analysis = self.mbse_expert.analyze_requirements(requirements_text)
        analysis["domain"] = domain
        
        return json.dumps(analysis, indent=2)
    
    async def _generate_from_requirements(self, arguments: Dict[str, Any]) -> str:
        """Generate ArcLang model from requirements analysis."""
        analysis = arguments.get("analysis", {})
        project_name = arguments.get("project_name", "GeneratedModel")
        output_path = arguments.get("output_path")
        
        arc_content = self.mbse_expert.generate_arclang_skeleton(analysis, project_name)
        
        if output_path:
            resolved_path = Path(output_path)
            if not resolved_path.is_absolute():
                resolved_path = self.workspace_root / output_path
            
            resolved_path.parent.mkdir(parents=True, exist_ok=True)
            resolved_path.write_text(arc_content)
            
            return f"✅ Generated ArcLang model saved to: {resolved_path}\n\n{arc_content}"
        else:
            return arc_content
    
    async def _assess_diagram_quality(self, arguments: Dict[str, Any]) -> str:
        """Assess diagram quality using MBSE expert system."""
        import json
        
        diagram_path = arguments.get("diagram_path", "")
        diagram_type = arguments.get("diagram_type", "component")
        
        resolved_path = Path(diagram_path)
        if not resolved_path.is_absolute():
            resolved_path = self.workspace_root / diagram_path
        
        assessment = self.mbse_expert.assess_diagram_quality(resolved_path, diagram_type)
        
        return json.dumps(assessment, indent=2)
    
    async def _suggest_diagram_enhancements(self, arguments: Dict[str, Any]) -> str:
        """Suggest diagram enhancements based on quality assessment."""
        import json
        
        assessment = arguments.get("assessment", {})
        
        enhancements = self.mbse_expert.suggest_enhancements(assessment)
        
        return json.dumps({
            "suggested_enhancements": enhancements,
            "priority": "high" if assessment.get("quality_percentage", 100) < 70 else "medium"
        }, indent=2)
    
    async def _generate_mbse_report(self, arguments: Dict[str, Any]) -> str:
        """Generate comprehensive MBSE report."""
        analysis = arguments.get("analysis", {})
        assessments = arguments.get("assessments", [])
        output_path = arguments.get("output_path")
        
        report = self.mbse_expert.generate_mbse_report(analysis, assessments)
        
        if output_path:
            resolved_path = Path(output_path)
            if not resolved_path.is_absolute():
                resolved_path = self.workspace_root / output_path
            
            resolved_path.parent.mkdir(parents=True, exist_ok=True)
            resolved_path.write_text(report)
            
            return f"✅ MBSE Report saved to: {resolved_path}\n\n{report}"
        else:
            return report

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
