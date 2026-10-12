"""Edits go through the compiler's lossless edits and write only when asked."""


async def test_set_previews_without_writing(server, model):
    before = model.read_text()
    answer = await server.call_tool(
        "arclang_set_attribute",
        {"model_path": str(model), "element": "LC-002", "key": "description", "value": '"Brakes"'},
    )
    assert "NOT written" in answer.splitlines()[0]
    assert '+            description: "Brakes"' in answer
    assert model.read_text() == before


async def test_set_writes_when_asked_and_keeps_comments(server, model):
    answer = await server.call_tool(
        "arclang_set_attribute",
        {
            "model_path": str(model),
            "element": "LF-002",
            "key": "latency",
            "value": "15 ms",
            "write": True,
        },
    )
    assert "written to braking.arc" in answer.splitlines()[0]
    text = model.read_text()
    assert "latency: 15 ms" in text
    assert "// Reference model served by the ArcLang MCP server." in text


async def test_a_value_starting_with_a_dash_is_a_value(server, model):
    answer = await server.call_tool(
        "arclang_set_attribute",
        {"model_path": str(model), "element": "LC-002", "key": "offset", "value": "-5"},
    )
    assert "+            offset: -5" in answer


async def test_set_on_an_unknown_element_fails_and_writes_nothing(server, model):
    before = model.read_text()
    answer = await server.call_tool(
        "arclang_set_attribute",
        {
            "model_path": str(model),
            "element": "NOPE",
            "key": "description",
            "value": '"x"',
            "write": True,
        },
    )
    assert "FAILED" in answer.splitlines()[0]
    assert model.read_text() == before


async def test_unset_removes_the_attribute(server, model):
    answer = await server.call_tool(
        "arclang_unset_attribute",
        {"model_path": str(model), "element": "LF-002", "key": "latency", "write": True},
    )
    assert "written" in answer.splitlines()[0]
    assert "latency: 10 ms" not in model.read_text()


async def test_rename_keeps_the_identity(server, model):
    answer = await server.call_tool(
        "arclang_rename",
        {"model_path": str(model), "element": "LC-002", "name": "Brake commander", "write": True},
    )
    assert "written" in answer.splitlines()[0]
    assert 'component "Brake commander"' in model.read_text()
    assert (await server.compiler.gate(model)).ok


async def test_format_previews_then_writes(server, workspace):
    messy = workspace / "messy.arc"
    messy.write_text(
        'model M {\n// kept\nrequirements system {\nreq "R1" "t" { description: "d" }\n}\n}\n'
    )
    before = messy.read_text()
    preview = await server.call_tool("arclang_format", {"path": str(messy)})
    assert "NOT written" in preview.splitlines()[0]
    assert messy.read_text() == before
    await server.call_tool("arclang_format", {"path": str(messy), "write": True})
    assert "// kept" in messy.read_text()
    assert (await server.compiler.format(messy, check=True)).ok


async def test_format_of_a_formatted_file_changes_nothing(server, model):
    answer = await server.call_tool("arclang_format", {"path": str(model)})
    assert answer.endswith("no change")


async def test_writing_the_value_already_there_is_not_called_a_write(server, model):
    answer = await server.call_tool(
        "arclang_set_attribute",
        {
            "model_path": str(model),
            "element": "LF-002",
            "key": "latency",
            "value": "10 ms",
            "write": True,
        },
    )
    assert answer.splitlines()[0].endswith("no change, nothing written")
