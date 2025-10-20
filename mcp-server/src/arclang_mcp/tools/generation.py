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
