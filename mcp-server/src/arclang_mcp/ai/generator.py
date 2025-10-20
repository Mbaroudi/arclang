"""
AI-powered code generation for ArcLang.
"""

from typing import Any, Dict, List, Optional
import anthropic


class AIGenerator:
    """AI-powered ArcLang code generator."""

    def __init__(self, config: Dict[str, Any]):
        self.provider = config.get("provider", "anthropic")
        self.model = config.get("model", "claude-3-5-sonnet-20241022")
        self.temperature = config.get("temperature", 0.3)
        
        if self.provider == "anthropic":
            api_key = config.get("api_key") or os.getenv("ANTHROPIC_API_KEY")
            self.client = anthropic.Anthropic(api_key=api_key) if api_key else None

    async def generate_requirement(
        self,
        description: str,
        safety_level: Optional[str] = None,
        priority: str = "High"
    ) -> str:
        """Generate ArcLang requirement from description."""
        
        prompt = f"""Generate an ArcLang requirement based on this description:

Description: {description}
Safety Level: {safety_level or 'Not specified'}
Priority: {priority}

Generate a complete ArcLang requirement block following this format:

requirement "REQ-XXX-YYY" {{
    description: "..."
    priority: "{priority}"
    {f'safety_level: "{safety_level}"' if safety_level else '// safety_level: "TBD"'}
    type: "Functional"  // or Performance, Safety, etc.
    verification_method: "Test"  // or Analysis, Inspection, Demonstration
}}

Important:
- Use descriptive requirement ID (e.g., REQ-BRAKE-001)
- Keep description concise and verifiable
- Choose appropriate type based on description
- Follow ISO 26262 / DO-178C conventions if safety level specified

Generate only the ArcLang code, no explanation."""

        if self.client:
            message = self.client.messages.create(
                model=self.model,
                max_tokens=1024,
                temperature=self.temperature,
                messages=[{"role": "user", "content": prompt}]
            )
            return message.content[0].text.strip()
        else:
            # Fallback template
            req_id = self._generate_requirement_id(description)
            return f'''requirement "{req_id}" {{
    description: "{description}"
    priority: "{priority}"
    {f'safety_level: "{safety_level}"' if safety_level else '// safety_level: "TBD"'}
    type: "Functional"
    verification_method: "Test"
}}'''

    async def generate_component(
        self,
        description: str,
        component_type: str = "Logical",
        safety_level: Optional[str] = None
    ) -> str:
        """Generate ArcLang component from description."""
        
        prompt = f"""Generate an ArcLang component based on this description:

Description: {description}
Type: {component_type}
Safety Level: {safety_level or 'Not specified'}

Generate a complete ArcLang component block following this format:

component "Component Name" {{
    id: "LC-XXX-YYY"
    type: "{component_type}"
    description: "..."
    {f'safety_level: "{safety_level}"' if safety_level else ''}
    
    function "Main Function" {{
        id: "LF-XXX-YYY"
        inputs: ["input1", "input2"]
        outputs: ["output1"]
        execution_time: "XYms"
    }}
}}

Important:
- Use descriptive component and function names
- Component ID pattern: LC-<SUBSYSTEM>-<NUM>
- Function ID pattern: LF-<FUNCTION>-<NUM>
- Include realistic inputs/outputs
- Add execution_time for real-time systems

Generate only the ArcLang code, no explanation."""

        if self.client:
            message = self.client.messages.create(
                model=self.model,
                max_tokens=2048,
                temperature=self.temperature,
                messages=[{"role": "user", "content": prompt}]
            )
            return message.content[0].text.strip()
        else:
            # Fallback template
            comp_id = self._generate_component_id(description)
            return f'''component "{description}" {{
    id: "{comp_id}"
    type: "{component_type}"
    description: "{description}"
    {f'safety_level: "{safety_level}"' if safety_level else ''}
    
    function "Process" {{
        id: "LF-PROCESS"
        inputs: ["input_data"]
        outputs: ["output_data"]
    }}
}}'''

    async def suggest_architecture(
        self,
        requirements: List[str],
        domain: str = "automotive"
    ) -> Dict[str, Any]:
        """Suggest architecture based on requirements."""
        
        req_list = "\n".join([f"- {req}" for req in requirements])
        
        prompt = f"""Given these requirements for a {domain} system, suggest an appropriate architecture:

Requirements:
{req_list}

Suggest:
1. Main components (5-10 components)
2. Component types (Logical/Physical)
3. Safety levels if applicable
4. Recommended architectural patterns

Provide suggestions in JSON format:
{{
    "components": [
        {{
            "name": "Component Name",
            "type": "Logical",
            "description": "Brief description",
            "safety_level": "ASIL_B"  // if applicable
        }}
    ],
    "patterns": ["Pattern 1", "Pattern 2"]
}}"""

        if self.client:
            message = self.client.messages.create(
                model=self.model,
                max_tokens=2048,
                temperature=self.temperature,
                messages=[{"role": "user", "content": prompt}]
            )
            try:
                import json
                return json.loads(message.content[0].text.strip())
            except:
                return {"components": [], "patterns": []}
        else:
            # Fallback
            return {
                "components": [
                    {"name": "Sensor Interface", "type": "Logical", "description": "Handles sensor data"},
                    {"name": "Controller", "type": "Logical", "description": "Main control logic"},
                    {"name": "Actuator Interface", "type": "Logical", "description": "Controls actuators"}
                ],
                "patterns": ["Layered Architecture", "Sense-Process-Actuate"]
            }

    def _generate_requirement_id(self, description: str) -> str:
        """Generate requirement ID from description."""
        words = description.split()[:2]
        prefix = "".join([w[0].upper() for w in words if w])
        return f"REQ-{prefix}-001"

    def _generate_component_id(self, description: str) -> str:
        """Generate component ID from description."""
        words = description.split()[:2]
        prefix = "".join([w[0].upper() for w in words if w])
        return f"LC-{prefix}-001"


import os
