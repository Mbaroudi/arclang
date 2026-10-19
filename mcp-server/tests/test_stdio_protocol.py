"""End to end: the server started as a process and spoken to in MCP over stdio."""

import json
import selectors
import os
import subprocess
import sys

PROTOCOL_VERSION = "2024-11-05"


READ_TIMEOUT_SECONDS = 30


class Client:
    def __init__(self, process: subprocess.Popen):
        self.process = process
        self.next_id = 0

    def notify(self, method: str) -> None:
        self._write({"jsonrpc": "2.0", "method": method})

    def request(self, method: str, params: dict) -> dict:
        self.next_id += 1
        self._write({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params})
        while True:
            line = self._read_line()
            message = json.loads(line)
            if message.get("id") == self.next_id:
                assert "error" not in message, message
                return message["result"]

    def _read_line(self) -> str:
        """One line of stdout, or a failure after READ_TIMEOUT_SECONDS: a hung server must not hang the run."""
        selector = selectors.DefaultSelector()
        selector.register(self.process.stdout, selectors.EVENT_READ)
        try:
            ready = selector.select(timeout=READ_TIMEOUT_SECONDS)
        finally:
            selector.close()
        assert ready, f"no answer from the server within {READ_TIMEOUT_SECONDS} seconds"
        line = self.process.stdout.readline()
        assert line, "the server closed its output"
        return line

    def _write(self, message: dict) -> None:
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()


def test_a_client_can_list_and_call_tools_over_stdio(workspace, model):
    process = subprocess.Popen(
        [sys.executable, "-m", "arclang_mcp.server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=dict(os.environ),
    )
    try:
        client = Client(process)
        started = client.request(
            "initialize",
            {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0"},
            },
        )
        assert started["serverInfo"]["name"] == "arclang-mcp"
        assert started["serverInfo"]["version"] == "4.3.0"
        client.notify("notifications/initialized")

        tools = {tool["name"] for tool in client.request("tools/list", {})["tools"]}
        assert {
            "arclang_validate",
            "arclang_gate",
            "arclang_diff",
            "arclang_set_attribute",
            "arclang_import",
            "arclang_transition",
            "arclang_rules",
        } <= tools

        called = client.request(
            "tools/call", {"name": "arclang_gate", "arguments": {"model_path": "braking.arc"}}
        )
        assert called["content"][0]["text"].startswith("Production gate of braking.arc: PASS")

        resources = {entry["uri"] for entry in client.request("resources/list", {})["resources"]}
        assert "arclang://syntax-rules" in resources
        read = client.request("resources/read", {"uri": "arclang://syntax-rules"})
        assert 'req "REQ-001" "Title"' in read["contents"][0]["text"]
    finally:
        process.kill()
        process.wait()


def test_a_failed_call_is_marked_as_an_error_over_stdio(workspace, model):
    process = subprocess.Popen(
        [sys.executable, "-m", "arclang_mcp.server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=dict(os.environ),
    )
    try:
        client = Client(process)
        client.request(
            "initialize",
            {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "0"},
            },
        )
        client.notify("notifications/initialized")
        before = model.read_text()
        for name, arguments in (
            ("arclang_nope", {}),
            (
                "arclang_set_attribute",
                {
                    "model_path": "braking.arc",
                    "element": "LC-002",
                    "key": "k",
                    "value": '"v"',
                    "write": "no",
                },
            ),
        ):
            called = client.request("tools/call", {"name": name, "arguments": arguments})
            assert called.get("isError") is True, called
        assert model.read_text() == before
    finally:
        process.kill()
        process.wait()
