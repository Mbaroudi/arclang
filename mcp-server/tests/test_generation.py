"""Generated ArcLang is compiled before it is returned, and nothing is made up."""

import json

REQUIREMENTS = """The driver shall be warned of an imminent collision.
The system shall brake automatically, safety critical ASIL-D.
The system shall log every "emergency" event; fail-safe on sensor loss.
The radar sensor shall detect objects up to 150 m.
The brake controller shall command the actuator within 50 ms."""


async def test_requirement_template_compiles(server):
    answer = await server.call_tool(
        "arclang_generate_requirement",
        {
            "description": 'The system shall say "stop" when braking.',
            "safety_level": "ASIL_B",
            "priority": "Critical",
        },
    )
    assert "The compiler accepts it" in answer
    assert 'safety_level: "ASIL-B"' in answer
    assert "built-in template" in answer


async def test_component_templates_compile_for_each_type(server):
    for component_type in ("Logical", "Physical"):
        answer = await server.call_tool(
            "arclang_generate_component",
            {"description": "Fuses radar and camera tracks", "component_type": component_type},
        )
        assert "The compiler accepts it" in answer, answer


async def test_a_refused_snippet_is_said_to_be_refused(server):
    answer = await server.generation_tools._snippet_answer(
        'req REQ-001 "Title" { }', "requirements system"
    )
    assert "THE COMPILER REFUSES IT" in answer
    assert "accepts" not in answer


async def test_no_architecture_is_invented_without_a_model_to_ask(server):
    answer = await server.call_tool(
        "arclang_suggest_architecture", {"requirements": ["The system shall brake"]}
    )
    assert answer.startswith("No suggestion")


async def test_analysis_assumes_no_safety_level(server):
    analysis = json.loads(
        await server.call_tool("arclang_analyze_requirements", {"requirements_text": REQUIREMENTS})
    )
    levels = {
        entry["requirement"][:30]: entry["level"] for entry in analysis["safety_requirements"]
    }
    assert "ASIL-D" in levels.values()
    assert None in levels.values(), "a requirement that states no level must get none"
    assert set(levels.values()) <= {"ASIL-D", None}


async def test_skeleton_compiles_and_invents_no_trace(server, workspace):
    analysis = json.loads(
        await server.call_tool("arclang_analyze_requirements", {"requirements_text": REQUIREMENTS})
    )
    answer = await server.call_tool(
        "arclang_generate_from_requirements",
        {"analysis": analysis, "project_name": "Emergency braking", "output_path": "gen/model.arc"},
    )
    assert answer.startswith("The compiler accepts this skeleton")
    written = (workspace / "gen" / "model.arc").read_text()
    assert "trace " not in written
    assert (await server.compiler.check(workspace / "gen" / "model.arc")).ok
    for line in REQUIREMENTS.splitlines():
        assert line.replace('"', '\\"') in written


async def test_diagram_type_without_a_check_gets_no_score(server, workspace):
    svg = workspace / "tree.svg"
    svg.write_text("<svg></svg>")
    assessment = json.loads(
        await server.call_tool(
            "arclang_assess_diagram_quality", {"diagram_path": str(svg), "diagram_type": "tree"}
        )
    )
    assert assessment["assessed"] is False
    assert "quality_percentage" not in assessment


async def test_an_empty_snippet_is_not_accepted(server):
    answer = await server.generation_tools._snippet_answer("  \n", "requirements system")
    assert "THE COMPILER REFUSES IT" in answer and "empty" in answer


async def test_a_snippet_that_closes_its_enclosing_block_is_not_accepted(server):
    answer = await server.generation_tools._snippet_answer(
        "}\nrequirements other {", "requirements system"
    )
    assert "THE COMPILER REFUSES IT" in answer and "braces" in answer


async def test_braces_inside_strings_do_not_count(server):
    answer = await server.generation_tools._snippet_answer(
        'req "R-1" "Title with } brace" { description: "a { b" }', "requirements system"
    )
    assert "The compiler accepts it" in answer


async def test_a_stated_level_alone_makes_a_safety_requirement(server):
    text = "The brake controller shall meet ASIL-D.\nThe ECU shall boot in 2 s.\nThe unit shall be SIL 3 certified.\nThe cabin shall stay silent."
    analysis = json.loads(
        await server.call_tool("arclang_analyze_requirements", {"requirements_text": text})
    )
    levels = {entry["requirement"]: entry["level"] for entry in analysis["safety_requirements"]}
    assert levels == {
        "The brake controller shall meet ASIL-D.": "ASIL-D",
        "The unit shall be SIL 3 certified.": "SIL-3",
    }
    assert "The ECU shall boot in 2 s." in analysis["arcadia_mapping"]["physical"]


async def test_the_reason_a_diagram_was_not_generated_is_the_compilers(server, workspace):
    missing = workspace / "missing.arc"
    results = await server.generation_tools.generate_all(missing, workspace / "d")
    assert results and all("error" in result for result in results.values())
    for result in results.values():
        assert "missing.arc" in result["error"], result
        assert "node:internal" not in result["error"], result


async def test_a_snippet_the_compiler_warns_about_is_not_accepted(server):
    answer = await server.call_tool(
        "arclang_generate_requirement",
        {"description": "The brake shall stop the vehicle", "safety_level": "ASIL-Z"},
    )
    assert "accepts it" not in answer
    assert "WARNS" in answer and "ASIL-Z" in answer


async def test_a_skeleton_with_an_invalid_level_is_not_saved(server, workspace):
    analysis = {
        "safety_requirements": [{"requirement": "The ECU shall be SIL-9 safe", "level": "SIL-9"}]
    }
    answer = await server.call_tool(
        "arclang_generate_from_requirements",
        {"analysis": analysis, "project_name": "Bad", "output_path": "bad.arc"},
    )
    assert "WARNS" in answer and "not saved" in answer
    assert not (workspace / "bad.arc").exists()


async def test_component_hints_are_whole_words(server):
    analysis = json.loads(
        await server.call_tool(
            "arclang_analyze_requirements",
            {"requirements_text": "The uncontrolled sensory input shall be logged"},
        )
    )
    assert analysis["recommended_components"] == []


async def test_a_component_hint_cannot_add_elements_to_the_skeleton(server):
    hostile = {"recommended_components": [{"type": 'x" { } component "Evil', "hint": "h"}]}
    answer = await server.call_tool(
        "arclang_generate_from_requirements", {"analysis": hostile, "project_name": "P"}
    )
    assert 'component "Evil"' not in answer
