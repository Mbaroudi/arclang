//! A complete reader of the KerML / SysML v2 textual notation: a text is
//! lexed, parsed into a syntax tree following the OMG grammars
//! (KerMLExpressions.xtext, SysML.xtext), and rendered as Systems Modeling
//! API records that `sysml_import` reads like any other listing.

pub mod lexer;
pub mod library;
mod parser;
mod records;
mod records_expr;
mod scope;
pub mod syntax;

pub use parser::{parse, parse_kerml, parse_target};
pub use records::Built;

/// Parse a text and render it as API records.
pub fn records(text: &str) -> Result<Built, String> {
    let roots = parse(text)?;
    Ok(records::build(&roots))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_package_with_parts_and_a_connection() {
        let text = "package <'P'> 'Pkg' {\n  import ISQ::*;\n  part def Vehicle :> Base { attribute mass : MassValue = 1200 [kg]; port p : ~Pt; }\n  part v : Vehicle [1..*] ordered {\n    ref part e : Engine;\n    connect e.outlet to v.p;\n    doc /* the vehicle */\n  }\n  #Meta part w;\n  flow f of Fuel from t.outlet to e.inlet;\n}\n";
        let nodes = parse(text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(nodes.len(), 1);
        let package = &nodes[0];
        assert_eq!((package.kind, package.short_name.as_deref(), package.name.as_deref()), ("Package", Some("P"), Some("Pkg")));
        assert_eq!(package.members.len(), 5);
        assert_eq!(package.members[0].kind, "NamespaceImport");
        let vehicle = &package.members[1];
        assert_eq!((vehicle.kind, vehicle.name.as_deref()), ("PartDefinition", Some("Vehicle")));
        assert_eq!(vehicle.specializations[0].kind, syntax::SpecKind::Subclassification);
        assert_eq!(vehicle.members.len(), 2);
        assert!(vehicle.members[1].specializations[0].conjugated);
        let v = &package.members[2];
        assert_eq!(v.multiplicity.as_ref().map(|m| m.ordered), Some(true));
        assert_eq!(v.members[1].kind, "ConnectionUsage");
        assert_eq!(v.members[2].kind, "Documentation");
        assert_eq!(package.members[3].metadata.len(), 1);
        assert_eq!(package.members[4].kind, "FlowUsage");
        // `flow f of Fuel ...`: the payload is a member of the flow.
        assert_eq!(package.members[4].members[0].kind, "PayloadFeature");
    }

    #[test]
    fn actions_states_and_expressions() {
        let text = "action def A {\n  in x : Real;\n  first start;\n  then action a1 { }\n  if x > 1 and not done then a2;\n  action a2;\n  fork f;\n  then a3;\n  decide d;\n  if x == 2 then a3;\n  else done;\n  accept s : Sig via p;\n  send Sig() via p to q;\n  assign x := x + 1;\n  while x < 10 { assign x := x * 2; } until x > 5;\n  for i in 1..3 { }\n  calc c { return r : Real; x ** 2 }\n}\nstate def S parallel {\n  entry; then s1;\n  state s1;\n  accept Go then s2;\n  state s2 { entry action e { } do send Done() via p; exit; }\n  transition t first s1 accept Stop if x < 1 do a then s2;\n}\n";
        let nodes = parse(text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(nodes.len(), 2);
        let a = &nodes[0];
        let kinds: Vec<&str> = a.members.iter().map(|m| m.kind).collect();
        assert_eq!(
            kinds,
            vec![
                "ReferenceUsage", "InitialNode", "ActionUsage", "TransitionUsage", "ActionUsage", "ForkNode", "SuccessionAsUsage", "DecisionNode", "TransitionUsage",
                "TransitionUsage", "AcceptActionUsage", "SendActionUsage", "AssignmentActionUsage", "WhileLoopActionUsage", "ForLoopActionUsage", "CalculationUsage"
            ]
        );
        assert_eq!(a.members[0].prefixes.direction, Some("in"));
        // `then action a1 { }`: the usage, marked as succeeding the previous member.
        assert_eq!(a.members[2].membership_kind, Some("after-then"));
        let calc = a.members.last().unwrap();
        assert_eq!(calc.members[0].membership, "ReturnParameterMembership");
        assert_eq!(calc.members[1].membership, "ResultExpressionMembership");
        let s = &nodes[1];
        assert!(s.prefixes.is_parallel);
        assert_eq!(s.members[0].membership_kind, Some("entry"));
        assert_eq!(s.members[1].kind, "TransitionUsage");
        assert_eq!(s.members[3].kind, "TransitionUsage");
        assert_eq!(s.members.last().unwrap().name.as_deref(), Some("t"));
    }

    #[test]
    fn public_imports_are_re_exported() {
        // The shape of the OMG vehicle model: the root imports `Definitions::*`,
        // which publicly imports the packages that hold the definitions.
        let text = "package Model {\n  public import Definitions::*;\n  package Definitions {\n    public import Parts::*;\n    private import Hidden::*;\n    package Parts { part def Wheel; }\n    package Hidden { part def Secret; }\n  }\n  package Config {\n    public import Parts::Wheel;\n    part w : Wheel;\n    part s : Secret;\n  }\n}\n";
        let built = records(text).unwrap_or_else(|e| panic!("{e}"));
        let unresolved: Vec<&str> = built.unresolved.iter().map(String::as_str).collect();
        assert_eq!(unresolved, vec!["FeatureTyping 'Secret' designates nothing known"]);
    }

    #[test]
    fn recursive_membership_imports_and_implicit_names() {
        // `P::**` imports P and everything under it; `perform a.b;` and
        // `:>> x` are known by the name of the feature they reference.
        let text = "package M {\n  package Config { package Tree { part v { part e { perform act.step; } } } }\n  action act { action step; }\n  package Alloc {\n    public import Config::Tree::**;\n    connect v.e to Tree.v;\n    connect v.e.step to e;\n  }\n}\n";
        let built = records(text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(built.unresolved, Vec::<String>::new());
    }

    #[test]
    fn a_performed_action_is_reached_through_its_implicit_name() {
        let text = "package M {\n  action massTests { action weigh; }\n  part context {\n    perform massTests;\n    part scale { perform massTests.weigh; }\n  }\n}\n";
        let built = records(text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(built.unresolved, Vec::<String>::new());
    }

    #[test]
    fn public_imports_that_cycle_are_followed_once() {
        // Twelve packages publicly importing each other: a name found
        // nowhere is looked for once in each, not along every path.
        let mut text = String::from("package M {\n");
        for i in 0..12 {
            let imports: String = (0..12).filter(|j| *j != i).map(|j| format!("public import P{j}::*; ")).collect();
            text.push_str(&format!("  package P{i} {{ {imports} part def D{i}; }}\n"));
        }
        text.push_str("  part x : P0::Nowhere;\n  part y : P0::D11;\n}\n");
        let started = std::time::Instant::now();
        let built = records(&text).unwrap_or_else(|e| panic!("{e}"));
        assert!(started.elapsed().as_secs() < 2, "took {:?}", started.elapsed());
        assert_eq!(built.unresolved, vec!["FeatureTyping 'P0::Nowhere' designates nothing known".to_string()]);
    }
}
