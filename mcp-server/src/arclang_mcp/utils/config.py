"""
Configuration management for ArcLang MCP Server.
"""

import logging
import os
from pathlib import Path
from typing import Any, Dict

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10
    import tomli as tomllib  # type: ignore[no-redef]

logger = logging.getLogger(__name__)

CONFIG_FILE = ".arclang-mcp.toml"


def load_config(workspace_root: Path) -> Dict[str, Any]:
    """Load configuration from .arclang-mcp.toml and the environment."""
    config: Dict[str, Any] = {
        "workspace": {"root": str(workspace_root), "build_dir": "build/"},
        "compiler": {"path": "arclang", "timeout": 30, "review_timeout": 600},
        "ai": {
            "provider": "anthropic",
            "model": "claude-opus-5-5",
            "api_key": os.getenv("ANTHROPIC_API_KEY"),
        },
        "safety": {"default_standard": "iso26262"},
    }

    config_file = workspace_root / CONFIG_FILE
    if config_file.exists():
        try:
            with config_file.open("rb") as handle:
                given = tomllib.load(handle)
            # A workspace can be a cloned repository: it must not be able to
            # choose the program this server runs, nor the key it sends.
            if "path" in given.get("compiler", {}):
                given["compiler"].pop("path")
                logger.warning(
                    "%s: [compiler] path is ignored; set ARCLANG_BINARY instead", config_file
                )
            if "api_key" in given.get("ai", {}):
                given["ai"].pop("api_key")
                logger.warning(
                    "%s: [ai] api_key is ignored; set ANTHROPIC_API_KEY instead", config_file
                )
            config = _deep_merge(config, given)
        except (OSError, tomllib.TOMLDecodeError) as failure:
            # stdout is the protocol channel: never print there.
            logger.warning("%s ignored: %s", config_file, failure)

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
