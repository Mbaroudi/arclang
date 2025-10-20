"""
ArcLang MCP Server - AI-powered Model Context Protocol server for ArcLang.

Provides intelligent assistance for Model-Based Systems Engineering (MBSE)
through integration with AI assistants like Claude and GPT-4.
"""

__version__ = "0.1.0"
__author__ = "Malek Baroudi & Bilel Laasami"
__license__ = "MIT"

from .server import ArcLangMCPServer

__all__ = ["ArcLangMCPServer"]
