"""
Configuration management for ArcLang MCP Server.
"""

import os
import toml
from pathlib import Path
from typing import Any, Dict


def load_config(workspace_root: Path) -> Dict[str, Any]:
    """Load configuration from .arclang-mcp.toml or environment."""
    
    # Default configuration
    config = {
        "workspace": {
            "root": str(workspace_root),
            "build_dir": "build/"
        },
        "compiler": {
            "path": "arclang",
            "timeout": 30
        },
        "ai": {
            "provider": "anthropic",
            "model": "claude-3-5-sonnet-20241022",
            "temperature": 0.3,
            "api_key": os.getenv("ANTHROPIC_API_KEY")
        },
        "cache": {
            "enabled": True,
            "ttl": 3600
        },
        "plm": {
            "enabled": False
        },
        "safety": {
            "default_standard": "iso26262",
            "strict_validation": True
        }
    }
    
    # Try to load from .arclang-mcp.toml
    config_file = workspace_root / ".arclang-mcp.toml"
    if config_file.exists():
        try:
            user_config = toml.load(config_file)
            config = _deep_merge(config, user_config)
        except Exception as e:
            print(f"Warning: Failed to load config file: {e}")
    
    # Override with environment variables
    if "ARCLANG_BINARY" in os.environ:
        config["compiler"]["path"] = os.environ["ARCLANG_BINARY"]
    
    if "ANTHROPIC_API_KEY" in os.environ:
        config["ai"]["api_key"] = os.environ["ANTHROPIC_API_KEY"]
    
    return config


def _deep_merge(base: Dict[str, Any], override: Dict[str, Any]) -> Dict[str, Any]:
    """Deep merge two dictionaries."""
    result = base.copy()
    
    for key, value in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = _deep_merge(result[key], value)
        else:
            result[key] = value
    
    return result
