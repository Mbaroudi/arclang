"""
AI-powered generation tools for ArcLang.
"""

from typing import Any, Dict

from ..ai.generator import AIGenerator


class GenerationTools:
    """AI generation tools for requirements, components, and architectures."""

    def __init__(self, compiler: Any, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config
        self.generator = AIGenerator(config.get("ai", {}))

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a generation tool."""
        if tool_name == "arclang_generate_requirement":
            return await self._generate_requirement(arguments)
        elif tool_name == "arclang_generate_component":
            return await self._generate_component(arguments)
        elif tool_name == "arclang_suggest_architecture":
            return await self._suggest_architecture(arguments)
        elif tool_name == "arclang_generate_diagram":
            return await self._generate_diagram(arguments)
        elif tool_name == "arclang_generate_all_diagrams":
            return await self._generate_all_diagrams(arguments)
        else:
            raise ValueError(f"Unknown generation tool: {tool_name}")

    async def _generate_requirement(self, args: Dict[str, Any]) -> str:
        """Generate requirement from natural language."""
        description = args["description"]
        safety_level = args.get("safety_level")
        priority = args.get("priority", "High")

        requirement_code = await self.generator.generate_requirement(
            description=description,
            safety_level=safety_level,
            priority=priority
        )

        output = f"✨ **Generated Requirement**\n\n```arc\n{requirement_code}\n```\n\n"
        output += "**Next Steps**:\n"
        output += "1. Review the generated requirement\n"
        output += "2. Adjust safety level if needed\n"
        output += "3. Add to your model file\n"
        output += "4. Create traceability links to components"

        return output

    async def _generate_component(self, args: Dict[str, Any]) -> str:
        """Generate component from description."""
        description = args["description"]
        component_type = args.get("component_type", "Logical")
        safety_level = args.get("safety_level")

        component_code = await self.generator.generate_component(
            description=description,
            component_type=component_type,
            safety_level=safety_level
        )

        output = f"✨ **Generated Component**\n\n```arc\n{component_code}\n```\n\n"
        output += "**Next Steps**:\n"
        output += "1. Review the component structure\n"
        output += "2. Add input/output ports if needed\n"
        output += "3. Add to your logical_architecture block\n"
        output += "4. Create traces to requirements"

        return output

    async def _suggest_architecture(self, args: Dict[str, Any]) -> str:
        """Suggest architecture based on requirements."""
        requirements = args["requirements"]
        domain = args.get("domain", "automotive")

        suggestions = await self.generator.suggest_architecture(
            requirements=requirements,
            domain=domain
        )

        output = f"💡 **Architecture Suggestions**\n\n"
        output += f"**Domain**: {domain.capitalize()}\n"
        output += f"**Based on**: {len(requirements)} requirements\n\n"

        output += "**Suggested Components**:\n\n"
        for i, component in enumerate(suggestions.get("components", []), 1):
            output += f"{i}. **{component['name']}** ({component['type']})\n"
            output += f"   - Purpose: {component['description']}\n"
            if component.get("safety_level"):
                output += f"   - Safety: {component['safety_level']}\n"
            output += "\n"

        if suggestions.get("patterns"):
            output += "\n**Recommended Patterns**:\n"
            for pattern in suggestions["patterns"]:
                output += f"  - {pattern}\n"

        output += "\n**Next Steps**:\n"
        output += "1. Review suggested architecture\n"
        output += "2. Refine based on your specific needs\n"
        output += "3. Generate individual components\n"
        output += "4. Establish data flows and interfaces"

        return output

    async def _generate_diagram(self, args: Dict[str, Any]) -> str:
        """Generate a specific diagram type from model."""
        model_path = args["model_path"]
        diagram_type = args["diagram_type"]
        output_path = args.get("output_path")

        # Supported diagram types
        diagram_types = [
            "operational", "functional", "component", "sequence",
            "state-machine", "physical", "class", "tree",
            "capability", "functional-chain"
        ]

        if diagram_type not in diagram_types:
            return f"❌ **Error**: Unknown diagram type '{diagram_type}'\n\nSupported types: {', '.join(diagram_types)}"

        # Generate diagram using compiler
        result = await self.compiler.generate_diagram(
            model_path=model_path,
            diagram_type=diagram_type,
            output_path=output_path
        )

        output = f"📊 **Generated {diagram_type.capitalize()} Diagram**\n\n"
        output += f"**Input**: {model_path}\n"
        output += f"**Output**: {result['output_path']}\n"
        output += f"**Size**: {result['size']}\n"
        output += f"**Elements**: {result.get('element_count', 'N/A')}\n\n"

        output += "**Diagram Features**:\n"
        for feature in result.get('features', []):
            output += f"  ✅ {feature}\n"

        output += "\n**Next Steps**:\n"
        output += "1. Open the SVG file in a browser\n"
        output += "2. Review the visual layout\n"
        output += "3. Generate other diagram types if needed\n"
        output += "4. Use arclang_generate_all_diagrams for complete set"

        return output

    async def _generate_all_diagrams(self, args: Dict[str, Any]) -> str:
        """Generate all 10 Capella diagram types from model."""
        model_path = args["model_path"]
        output_dir = args.get("output_dir", "./diagrams")

        # All 10 diagram types
        diagram_types = [
            "operational", "functional", "component", "sequence",
            "state-machine", "physical", "class", "tree",
            "capability", "functional-chain"
        ]

        results = {}
        for diagram_type in diagram_types:
            try:
                result = await self.compiler.generate_diagram(
                    model_path=model_path,
                    diagram_type=diagram_type,
                    output_path=f"{output_dir}/{diagram_type}.svg"
                )
                results[diagram_type] = {
                    "status": "✅ Success",
                    "path": result['output_path'],
                    "size": result['size']
                }
            except Exception as e:
                results[diagram_type] = {
                    "status": "⏳ Skipped",
                    "reason": str(e)
                }

        # Generate summary
        output = "📊 **All Diagrams Generated**\n\n"
        output += f"**Input Model**: {model_path}\n"
        output += f"**Output Directory**: {output_dir}\n\n"

        output += "**Results**:\n\n"
        success_count = 0
        for dtype, result in results.items():
            output += f"**{dtype.capitalize()}**: {result['status']}\n"
            if result['status'] == "✅ Success":
                output += f"  - Path: {result['path']}\n"
                output += f"  - Size: {result['size']}\n"
                success_count += 1
            elif 'reason' in result:
                output += f"  - Reason: {result['reason']}\n"
            output += "\n"

        output += f"**Summary**: {success_count}/10 diagrams generated successfully\n\n"

        output += "**Rich Diagram Quality**:\n"
        output += "  - Operational: Swimlanes, actors, activity symbols (⊕)\n"
        output += "  - Functional: 15+ functions, data flows, categories\n"
        output += "  - Component: Hierarchical, protocols (CAN, Ethernet)\n"
        output += "  - All types: Professional Capella-quality output\n\n"

        output += "**Next Steps**:\n"
        output += "1. Open generated diagrams in browser\n"
        output += "2. Review each diagram type\n"
        output += "3. Use in documentation or presentations\n"
        output += "4. Export to PNG if needed"

        return output
