"""
ArcLang MCP Server - Model Context Protocol server for the ArcLang compiler.

Gives an AI assistant the compiler's own commands (check, gate, diff,
impact, lossless edits, exports) and its metamodel as the reference.
"""

__author__ = "Malek Baroudi & Bilel Laasami"
__license__ = "MIT"

from .server import ArcLangMCPServer, __version__

__all__ = ["ArcLangMCPServer", "__version__"]
