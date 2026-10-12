"""
ArcLang snippets from a description.

Without an API key the snippets come from the templates below. With
ANTHROPIC_API_KEY they are asked from a Claude model, which is given the
compiler's own reference model as the syntax to follow. Either way the
caller (tools/generation.py) compiles the snippet before returning it:
this module never claims that what it wrote is valid.
"""

import json
import os
import re
from typing import Any, Dict, List, Optional

import anthropic

from ..resources.syntax_guide import reference_model

DEFAULT_MODEL = "claude-opus-5-5"

# Arcadia layer of a component, and the block that declares it.
COMPONENT_BLOCKS = {
    "Logical": ("architecture logical", "component", "LC"),
    "Physical": ("architecture physical", "node", "PN"),
}


def arc_string(text: str) -> str:
    """`text` as an ArcLang string literal."""
    escaped = text.replace("\\", "\\\\").replace('"', '\\"')
    return '"' + " ".join(escaped.split()) + '"'


def safety_level(level: Optional[str]) -> Optional[str]:
    """`ASIL_B` or `asil b` as the metamodel spells it: `ASIL-B`."""
    if not level:
        return None
    return re.sub(r"[\s_]+", "-", level.strip().upper())


def acronym(description: str, fallback: str) -> str:
    """Up to three initials of the description, for an identifier."""
    initials = "".join(word[0] for word in re.findall(r"[A-Za-z]+", description)[:3]).upper()
    return initials or fallback


def title_of(description: str, limit: int = 60) -> str:
    """A short title: the first sentence, cut at `limit` characters."""
    first = re.split(r"(?<=[.!?])\s", description.strip(), maxsplit=1)[0].rstrip(".")
    return first if len(first) <= limit else first[: limit - 3].rstrip() + "..."


class AIGenerator:
    """Writes ArcLang snippets, from templates or from a Claude model."""

    def __init__(self, config: Dict[str, Any]):
        self.provider = config.get("provider", "anthropic")
        self.model = config.get("model", DEFAULT_MODEL)
        self.temperature = config.get("temperature")
        self.client: Optional[anthropic.AsyncAnthropic] = None

        if self.provider == "anthropic":
            api_key = config.get("api_key") or os.getenv("ANTHROPIC_API_KEY")
            if api_key:
                self.client = anthropic.AsyncAnthropic(api_key=api_key)

    @property
    def uses_model(self) -> bool:
        return self.client is not None

    def requirement_template(
        self, description: str, safety: Optional[str] = None, priority: str = "High"
    ) -> str:
        """A requirement, to be placed inside a `requirements <kind> { }` block."""
        lines = [
            f'req "REQ-{acronym(description, "X")}-001" {arc_string(title_of(description))} {{',
            f"  description: {arc_string(description)}",
            f"  priority: {arc_string(priority)}",
        ]
        level = safety_level(safety)
        if level:
            lines.append(f"  safety_level: {arc_string(level)}")
        return "\n".join([*lines, "}"])

    def component_template(
        self, description: str, component_type: str = "Logical", safety: Optional[str] = None
    ) -> str:
        """A component or a node, to be placed inside its architecture block."""
        _, keyword, prefix = COMPONENT_BLOCKS.get(component_type, COMPONENT_BLOCKS["Logical"])
        lines = [
            f"{keyword} {arc_string(title_of(description, 40))} {{",
            f'  id: "{prefix}-{acronym(description, "X")}-001"',
            f"  description: {arc_string(description)}",
        ]
        level = safety_level(safety)
        if level:
            lines.append(f"  safety_level: {arc_string(level)}")
        return "\n".join([*lines, "}"])

    async def generate_requirement(
        self, description: str, safety_level: Optional[str] = None, priority: str = "High"
    ) -> str:
        """An ArcLang requirement for the description."""
        template = self.requirement_template(description, safety_level, priority)
        if not self.client:
            return template
        return await self._ask(
            "Write ONE ArcLang requirement (a `req` declaration only, without the "
            "enclosing `requirements` block) for this description.\n\n"
            f"Description: {description}\nPriority: {priority}\n"
            f"Safety level: {safety_level or 'none'}\n\n"
            f"Shape to follow:\n{template}",
            max_tokens=1024,
        )

    async def generate_component(
        self,
        description: str,
        component_type: str = "Logical",
        safety_level: Optional[str] = None,
    ) -> str:
        """An ArcLang component (or physical node) for the description."""
        template = self.component_template(description, component_type, safety_level)
        if not self.client:
            return template
        return await self._ask(
            f"Write ONE ArcLang {component_type.lower()} element (the declaration only, "
            "without the enclosing architecture block) for this description. Give it "
            "ports and functions only when the description states them.\n\n"
            f"Description: {description}\nSafety level: {safety_level or 'none'}\n\n"
            f"Shape to follow:\n{template}",
            max_tokens=2048,
        )

    async def suggest_architecture(
        self, requirements: List[str], domain: str = "automotive"
    ) -> Optional[Dict[str, Any]]:
        """Components suggested for the requirements, or None without a model to ask."""
        if not self.client:
            return None
        listed = "\n".join(f"- {requirement}" for requirement in requirements)
        answer = await self._ask(
            f"Given these requirements for a {domain} system, suggest an architecture.\n\n"
            f"Requirements:\n{listed}\n\n"
            "Answer with JSON only, of this shape:\n"
            '{"components": [{"name": "...", "type": "Logical", "description": "...", '
            '"safety_level": "ASIL-B"}], "patterns": ["..."]}',
            max_tokens=2048,
            with_syntax=False,
        )
        try:
            return json.loads(answer)
        except json.JSONDecodeError:
            return {"components": [], "patterns": [], "unparsed_answer": answer}

    async def _ask(self, request: str, max_tokens: int, with_syntax: bool = True) -> str:
        assert self.client is not None
        prompt = request
        if with_syntax:
            prompt = (
                "ArcLang is a strict language: anything outside its grammar is a compile "
                "error. This model compiles and shows the syntax to follow:\n\n"
                f"{reference_model()}\n\n{request}\n\n"
                "Answer with the ArcLang code only: no explanation, no code fence."
            )
        parameters: Dict[str, Any] = {
            "model": self.model,
            "max_tokens": max_tokens,
            "messages": [{"role": "user", "content": prompt}],
        }
        if self.temperature is not None:
            parameters["temperature"] = self.temperature
        message = await self.client.messages.create(**parameters)
        text = "".join(block.text for block in message.content if block.type == "text").strip()
        fenced = re.fullmatch(r"```[a-zA-Z]*\n(.*?)\n?```", text, flags=re.DOTALL)
        return fenced.group(1).strip() if fenced else text
