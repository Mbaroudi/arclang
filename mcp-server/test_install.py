#!/usr/bin/env python3
"""Test MCP server installation."""

print("Testing ArcLang MCP Server installation...")
print()

# Test imports
try:
    from arclang_mcp import ArcLangMCPServer
    print("✅ Main server import successful")
except Exception as e:
    print(f"❌ Failed to import server: {e}")
    exit(1)

try:
    from arclang_mcp.tools import CoreTools, GenerationTools, SafetyTools, IntegrationTools
    print("✅ Tool modules import successful")
except Exception as e:
    print(f"❌ Failed to import tools: {e}")
    exit(1)

try:
    from arclang_mcp.compiler import ArcLangCompiler
    print("✅ Compiler wrapper import successful")
except Exception as e:
    print(f"❌ Failed to import compiler: {e}")
    exit(1)

try:
    from arclang_mcp.ai import AIGenerator
    print("✅ AI generator import successful")
except Exception as e:
    print(f"❌ Failed to import AI generator: {e}")
    exit(1)

try:
    from arclang_mcp.utils import load_config
    print("✅ Utils import successful")
except Exception as e:
    print(f"❌ Failed to import utils: {e}")
    exit(1)

print()
print("🎉 All imports successful!")
print()
print("Installation complete. The MCP server is ready to use.")
print()
print("Next steps:")
print("1. Configure Claude Desktop (see QUICKSTART.md)")
print("2. Set ANTHROPIC_API_KEY environment variable")
print("3. Restart Claude Desktop")
print("4. Start using AI-powered MBSE!")
