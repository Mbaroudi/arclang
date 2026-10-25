use super::*;

fn read_ok(text: &str) -> Vec<Declaration> {
    read(text).expect("the export reads")
}

fn summary(declarations: &[Declaration]) -> Vec<String> {
    declarations
        .iter()
        .map(|d| format!("{} {} via {}", d.metaclass, d.name.as_deref().unwrap_or("-"), d.membership))
        .collect()
}

#[test]
fn definitions_and_usages_get_their_metaclass_and_membership() {
    let declarations = read_ok(
        "package M {\n  private import SI::*;\n  part def <'LC-1'> Controller :> Unit {\n    doc /* The controller */\n    attribute latency : DurationValue = 25 [ms];\n    port cmd : Out_Port;\n    perform action perform_Decide : Decide;\n  }\n  part p_LC_1 : Controller;\n  abstract part def Unit;\n}\n",
    );
    assert_eq!(
        summary(&declarations),
        [
            "Package M via OwningMembership",
            "PartDefinition Controller via OwningMembership",
            "Documentation - via OwningMembership",
            "AttributeUsage latency via FeatureMembership",
            "PortUsage cmd via FeatureMembership",
            "PerformActionUsage perform_Decide via FeatureMembership",
            "PartUsage p_LC_1 via OwningMembership",
            "PartDefinition Unit via OwningMembership",
        ]
    );
    let controller = &declarations[1];
    assert_eq!(controller.short_name.as_deref(), Some("LC-1"));
    assert_eq!(controller.specializes, ["Unit"]);
    assert_eq!(declarations[2].body.as_deref(), Some("The controller"));
    assert_eq!(declarations[3].typed_by, ["DurationValue"]);
    assert_eq!(declarations[3].expression.as_deref(), Some("25 [ms]"));
    assert!(declarations[7].is_abstract);
}

#[test]
fn connectors_keep_their_ends() {
    let declarations = read_ok(
        "package M {\n  connect p_A.out to p_B.inp;  // note\n  allocate p_A to p_N;\n  flow from a_F to a_G;\n  connection Bus : CAN connect p_N to p_O {\n    attribute rate : Real = 2;\n  }\n  satisfy req_R by p_A;\n  dependency refines from A to B;\n}\n",
    );
    let ends: Vec<(&str, Option<(String, String)>)> = declarations[1..].iter().map(|d| (d.metaclass, d.ends.clone())).collect();
    let pair = |a: &str, b: &str| Some((a.to_string(), b.to_string()));
    assert_eq!(
        ends,
        [
            ("ConnectionUsage", pair("p_A.out", "p_B.inp")),
            ("AllocationUsage", pair("p_A", "p_N")),
            ("FlowUsage", pair("a_F", "a_G")),
            ("ConnectionUsage", pair("p_N", "p_O")),
            ("AttributeUsage", None),
            ("SatisfyRequirementUsage", pair("p_A", "req_R")),
            ("Dependency", pair("A", "B")),
        ]
    );
    assert_eq!(declarations[4].typed_by, ["CAN"]);
}

#[test]
fn a_multiplicity_is_a_range_owned_by_its_usage() {
    let declarations = read_ok(
        "package M {\n  part a : D [4];\n  part b : D [0..1];\n  part c : D [2..*];\n  part d : D [*];\n  part e : D;\n}\n",
    );
    let ranges: Vec<(Option<usize>, Option<(Option<u64>, Option<u64>)>)> = declarations
        .iter()
        .filter(|d| d.metaclass == "MultiplicityRange")
        .map(|d| (d.owner, d.bounds))
        .collect();
    assert_eq!(
        ranges,
        [
            (Some(1), Some((None, Some(4)))),
            (Some(3), Some((Some(0), Some(1)))),
            (Some(5), Some((Some(2), None))),
            (Some(7), Some((None, None))),
        ]
    );
    assert_eq!(declarations[1].typed_by, ["D"]);
    assert!(read("package M {\n  part a : D [4;\n}\n").is_err());
}

#[test]
fn then_declares_the_succession_before_the_step() {
    let declarations = read_ok(
        "package M {\n  action def Chain {\n    action step1 : A;\n    then action step2 : B;\n  }\n}\n",
    );
    assert_eq!(
        summary(&declarations[2..]),
        [
            "ActionUsage step1 via FeatureMembership",
            "SuccessionAsUsage - via FeatureMembership",
            "ActionUsage step2 via FeatureMembership",
        ]
    );
    assert_eq!(declarations[3].ends, Some(("step1".to_string(), "step2".to_string())));
}

#[test]
fn a_constraint_keeps_its_expression_as_text() {
    let declarations = read_ok(
        "package M {\n  assert constraint <'C-1'> Margin {\n    doc /* margin */\n    (a_F.latency + a_G.latency) <= (a_C.budget * 0.8)\n  }\n}\n",
    );
    assert_eq!(declarations[1].metaclass, "AssertConstraintUsage");
    assert_eq!(declarations[1].expression.as_deref(), Some("(a_F.latency + a_G.latency) <= (a_C.budget * 0.8)"));
}

#[test]
fn expressions_are_parsed_with_the_notation_precedence() {
    let declarations = read_ok(
        "package M {\n  attribute a : DurationValue = 25 [ms];\n  attribute b : Real = -3;\n  attribute c : String = \"x \\\"y\\\"\";\n  attribute d : Boolean = true;\n  attribute e default = 0.5;\n  assert constraint C {\n    (p.x + p.y * 2) <= DataFunctions::max(q.z, 4 [kg])\n  }\n}\n",
    );
    let reference = |name: &str| Expr::Reference(name.to_string());
    let chain = |base: &str, feature: &str| Expr::Chain(Box::new(reference(base)), feature.to_string());
    assert_eq!(declarations[1].value, Some(Expr::Operator("[", vec![Expr::Integer(25), reference("ms")])));
    assert_eq!(declarations[2].value, Some(Expr::Operator("-", vec![Expr::Integer(3)])));
    assert_eq!(declarations[3].value, Some(Expr::Text("x \"y\"".to_string())));
    assert_eq!(declarations[4].value, Some(Expr::Boolean(true)));
    assert_eq!((declarations[5].value.clone(), declarations[5].is_default), (Some(Expr::Rational(0.5)), true));
    assert_eq!(
        declarations[6].value,
        Some(Expr::Operator(
            "<=",
            vec![
                Expr::Operator("+", vec![chain("p", "x"), Expr::Operator("*", vec![chain("p", "y"), Expr::Integer(2)])]),
                Expr::Invocation(
                    "DataFunctions::max".to_string(),
                    vec![chain("q", "z"), Expr::Operator("[", vec![Expr::Integer(4), reference("kg")])]
                ),
            ]
        ))
    );
}

#[test]
fn a_redefining_feature_goes_by_the_name_it_redefines() {
    let declarations = read_ok(
        "package M {\n  action def Task {\n    attribute period : DurationValue;\n  }\n  action def Detect :> Task {\n    attribute :>> period = 33 [ms];\n  }\n  action a : Detect;\n}\n",
    );
    let scope = Scope::new(&declarations);
    let period = scope.resolve("a.period", Some(0)).unwrap();
    assert_eq!(scope.qualified_name(period).as_deref(), Some("M::Detect::period"));
    assert_eq!(declarations[period].name, None, "it declares no name of its own");
}

#[test]
fn a_short_name_resolves_like_a_name() {
    let declarations = read_ok("package M {\n  package Units {\n    attribute <ms> millisecond : DurationUnit;\n  }\n  private import Units::*;\n}\n");
    let scope = Scope::new(&declarations);
    assert_eq!(scope.resolve("ms", Some(0)), Some(2));
    assert_eq!(scope.resolve("Units::ms", Some(0)), Some(2));
}

#[test]
fn names_resolve_through_owners_types_and_packages() {
    let declarations = read_ok(
        "package M {\n  part def Camera {\n    port lane : P;\n  }\n  part p_CAM : Camera;\n  connect p_CAM.lane to p_CAM.lane;\n  package Units {\n    attribute <ms> millisecond : DurationUnit;\n  }\n  private import Units::*;\n}\n",
    );
    let scope = Scope::new(&declarations);
    let lane = scope.resolve("p_CAM.lane", Some(0)).expect("the port resolves through the part's type");
    assert_eq!(scope.qualified_name(lane).as_deref(), Some("M::Camera::lane"));
    assert_eq!(scope.resolve("millisecond", Some(1)).map(|i| declarations[i].metaclass), Some("AttributeUsage"));
    assert_eq!(scope.resolve("DurationUnit", Some(0)), None, "library names are not declared in the model");
}

#[test]
fn a_package_is_searched_only_when_imported() {
    let declarations = read_ok("package M {\n  package Data {\n    item def String;\n  }\n  package Use {\n    attribute a : String;\n  }\n}\n");
    let scope = Scope::new(&declarations);
    assert_eq!(scope.resolve("String", Some(3)), None, "the library's String, not the model's Data::String");
    let declarations = read_ok("package M {\n  library package Profile {\n    metadata def Actor;\n  }\n  #Actor part def A;\n}\n");
    let scope = Scope::new(&declarations);
    assert_eq!(scope.resolve("Actor", Some(0)).map(|i| declarations[i].metaclass), Some("MetadataDefinition"));
}

#[test]
fn an_unknown_statement_is_an_error() {
    let error = read("package M {\n  viewpoint V;\n}\n").unwrap_err();
    assert!(error.contains("unknown statement `viewpoint V`"), "{error}");
    assert!(read("package M {").is_err());
}

#[test]
fn a_chain_reads_its_control_nodes_and_successions() {
    let text = "package P {\n    action c {\n        action step1;\n        fork <'n1'> 'AND 1';\n        #Iterate merge <'n2'> 'IT';\n        succession <'s1'> first step1 then 'n1';\n        succession <'s2'> first 'n1' then step1 {\n            attribute condition : String = \"A\";\n        }\n    }\n}\n";
    let declarations = read_ok(text);
    let fork = declarations.iter().find(|d| d.metaclass == "ForkNode").expect("a fork");
    assert_eq!((fork.short_name.as_deref(), fork.name.as_deref(), fork.membership), (Some("n1"), Some("AND 1"), "FeatureMembership"));
    assert!(declarations.iter().any(|d| d.metaclass == "MergeNode" && d.short_name.as_deref() == Some("n2")));
    let links: Vec<_> = declarations.iter().filter(|d| d.metaclass == "SuccessionAsUsage").collect();
    assert_eq!(links.len(), 2);
    assert_eq!((links[0].short_name.as_deref(), links[0].name.as_deref()), (Some("s1"), None));
    assert_eq!(links[1].ends, Some(("n1".to_string(), "step1".to_string())));
}

#[test]
fn a_connection_of_more_than_two_ends_keeps_them_all() {
    let declarations = read_ok("package M {\n  connection bus connect ('LC-a'.p, 'LC-b'.q, 'LC-c'.r);\n  connect (x, y);\n}\n");
    assert_eq!(declarations[1].ends, Some(("LC-a.p".to_string(), "LC-b.q".to_string())));
    assert_eq!(declarations[1].more_ends, vec!["LC-c.r".to_string()]);
    assert_eq!(declarations[2].ends, Some(("x".to_string(), "y".to_string())));
    assert!(declarations[2].more_ends.is_empty());
}

#[test]
fn a_quoted_name_holding_a_dot_is_one_segment_of_a_path() {
    assert_eq!(super::split_path("'FC-1.Boundary approached?'.x", "."), vec!["'FC-1.Boundary approached?'", "x"]);
    assert_eq!(super::split_path("P::'a::b'.c", "::"), vec!["P", "'a::b'.c"]);
    assert_eq!(super::unquoted("'FC-1.B'"), "FC-1.B");
    let declarations = read_ok("package M {\n  action c {\n    merge <'C.1'> 'Done?';\n    succession first x then 'C.1';\n  }\n}\n");
    assert_eq!(declarations.last().unwrap().ends.as_ref().map(|e| e.1.as_str()), Some("'C.1'"));
}

#[test]
fn a_path_with_a_character_beyond_ascii_splits_whole() {
    // `SI::'°'` (the degree) once cut the `°` in two and panicked.
    assert_eq!(super::split_path("SI::'°'", "::"), ["SI", "'°'"]);
    assert_eq!(super::split_path("Café.Résumé", "."), ["Café", "Résumé"]);
}
