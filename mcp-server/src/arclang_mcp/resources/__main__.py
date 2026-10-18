"""`python -m arclang_mcp.resources` prints ARCLANG_SYNTAX_RULES.md."""

import sys

from .syntax_guide import rules_document

sys.stdout.write(rules_document())
