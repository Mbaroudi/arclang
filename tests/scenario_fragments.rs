//! Combined fragments of scenarios and regions of states: parsed, kept in
//! order, drawn as frames over the rows of a sequence diagram and as nested
//! boxes of a state machine.

use arclang::compiler::ast::FragmentType;
use arclang::compiler::diagram::build_diagrams;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
logical_architecture LA {
    component A { id: "LC-A" }
    component B { id: "LC-B" }
}
state_machine Modes {
    initial: "Off"
    state "Off" {}
    state "On" {
        state "Idle" {}
        state "Busy" { entry: ["work"] }
        transition "Idle" -> "Busy" { trigger: "job" }
    }
    transition "Off" -> "On" { trigger: "power" }
    transition "Busy" -> "Off" { trigger: "halt" }
}
scenario Run {
    participants: ["A", "B"]
    message A -> B "start"
    fragment loop "while running" {
        message B -> A "tick"
        fragment alt {
            operand "ok" { message A -> B "ack" }
            operand "else" { message A -> B "nack" { type: "async" } }
        }
    }
    message B -> A "done"
}
"#;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

#[test]
fn fragments_keep_the_rows_of_their_messages_and_nest() {
    let result = compile(MODEL);
    let scenario = &result.ast.scenarios[0];
    let labels: Vec<&str> = scenario.messages.iter().map(|m| m.label.as_str()).collect();
    assert_eq!(labels, ["start", "tick", "ack", "nack", "done"]);
    let outer = &scenario.fragments[0];
    assert_eq!(outer.fragment_type, FragmentType::Loop);
    assert_eq!(outer.label, "while running");
    assert_eq!((outer.start, outer.end), (1, 3));
    assert_eq!(outer.operands.len(), 1);
    let inner = &outer.operands[0].fragments[0];
    assert_eq!(inner.fragment_type, FragmentType::Alt);
    assert_eq!((inner.start, inner.end), (2, 3));
    let guards: Vec<(&str, usize, usize)> = inner.operands.iter().map(|o| (o.label.as_str(), o.start, o.end)).collect();
    assert_eq!(guards, [("ok", 2, 2), ("else", 3, 3)]);
}

#[test]
fn the_sequence_diagram_carries_the_frames() {
    let set = build_diagrams(&compile(MODEL).ast);
    let es = set.diagrams.iter().find(|d| d.id == "es:Run").expect("scenario drawn");
    assert_eq!(es.edges.len(), 5);
    let frame = &es.fragments[0];
    assert_eq!((frame.kind.as_str(), frame.start, frame.end), ("loop", 1, 3));
    assert_eq!(frame.lifelines.len(), 2);
    let nested = &frame.operands[0].fragments[0];
    assert_eq!(nested.kind, "alt");
    assert_eq!(nested.operands.iter().map(|o| o.label.as_str()).collect::<Vec<_>>(), ["ok", "else"]);
}

#[test]
fn a_state_holds_a_region_and_its_transitions_join_the_machine() {
    let result = compile(MODEL);
    let machine = &result.ast.state_machines[0];
    let on = machine.states.iter().find(|s| s.name == "On").expect("On");
    assert_eq!(on.sub_states.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Idle", "Busy"]);
    assert_eq!(machine.transitions.len(), 3);
    let set = build_diagrams(&result.ast);
    let msm = set.diagrams.iter().find(|d| d.id == "msm:Modes").expect("machine drawn");
    let on_node = msm.nodes.iter().find(|n| n.name == "On").expect("On drawn");
    assert_eq!(on_node.children.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(), ["Idle", "Busy"]);
    assert_eq!(msm.edges.len(), 4, "the initial transition and the three declared ones");
}

#[test]
fn a_fragment_mixing_operands_and_loose_messages_is_refused() {
    let bad = MODEL.replace("operand \"ok\" { message A -> B \"ack\" }", "message A -> B \"ack\"");
    let error = Compiler::new(CompilerConfig::default()).compile_string(&bad).err().expect("refused");
    assert!(error.to_string().contains("operand"), "{error}");
    let unknown = MODEL.replace("fragment loop", "fragment whenever");
    let error = Compiler::new(CompilerConfig::default()).compile_string(&unknown).err().expect("refused");
    assert!(error.to_string().contains("fragment kind"), "{error}");
}
