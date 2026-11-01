use super::*;

const BASE: &str = "model M {\n    name: \"M\"\n}\n\nrequirements {\n    // first\n    requirement \"R-1\" {\n        description: \"a\"\n        priority: \"High\"\n    }\n    requirement \"R-2\" {\n        description: \"b\"\n    }\n}\n";

fn merged(ours: &str, theirs: &str) -> Merged {
    merge_arc(BASE, ours, theirs).expect("merges")
}

#[test]
fn changes_to_different_elements_do_not_conflict() {
    let m = merged(&BASE.replace("\"a\"", "\"a2\""), &BASE.replace("\"b\"", "\"b2\""));
    assert!(m.conflicts.is_empty());
    assert!(m.text.contains("\"a2\"") && m.text.contains("\"b2\"") && m.text.contains("// first"));
}

#[test]
fn adjacent_attributes_of_one_element_do_not_conflict() {
    let m = merged(&BASE.replace("\"a\"", "\"a2\""), &BASE.replace("\"High\"", "\"Low\""));
    assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
    assert!(m.text.contains("\"a2\"") && m.text.contains("\"Low\""));
}

#[test]
fn the_same_attribute_changed_twice_conflicts_on_the_element() {
    let m = merged(&BASE.replace("\"a\"", "\"x\""), &BASE.replace("\"a\"", "\"y\""));
    assert_eq!(m.conflicts.len(), 1);
    assert_eq!(m.conflicts[0].element, "R-1");
    assert!(m.text.matches("<<<<<<<").count() == 1 && m.text.matches('{').count() == m.text.matches('}').count());
}

#[test]
fn an_identical_change_on_both_sides_is_taken_once() {
    let both = BASE.replace("\"a\"", "\"x\"");
    let m = merged(&both, &both);
    assert!(m.conflicts.is_empty());
    assert_eq!(m.text, both);
}

#[test]
fn an_unchanged_side_returns_the_other_byte_for_byte() {
    let ours = BASE.replace("\"b\"", "\"b2\"");
    assert_eq!(merged(BASE, &ours).text, ours);
    assert_eq!(merged(&ours, BASE).text, ours);
}

const TWO: &str = "model M {}\n\nlogical_architecture \"L\" {\n    component \"A\" {\n        id: \"LC-A\"\n        latency: 1 ms\n    }\n    component \"B\" {\n        id: \"LC-B\"\n        latency: 2 ms\n    }\n}\n";

#[test]
fn each_conflict_takes_the_side_chosen_for_it_and_the_others_stay_marked() {
    let ours = TWO.replace("1 ms", "10 ms").replace("2 ms", "20 ms");
    let theirs = TWO.replace("1 ms", "11 ms").replace("2 ms", "21 ms");
    let m = merge_arc(TWO, &ours, &theirs).unwrap();
    assert_eq!(m.conflicts.len(), 2, "{:?}", m.conflicts);
    let (text, open) = resolve_conflicts(&m.text, &m.conflicts, &[("LC-A".to_string(), Side::Theirs)]);
    assert!(text.contains("latency: 11 ms") && !text.contains("latency: 10 ms"), "{text}");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].element, "LC-B");
    assert!(text.contains("<<<<<<< ours") && text.contains("20 ms") && text.contains("21 ms"), "LC-B is still in dispute");
    let (all, none_left) = resolve_conflicts(&m.text, &m.conflicts, &[("LC-A".to_string(), Side::Ours), ("component 'B' [LC-B]".to_string(), Side::Theirs)]);
    assert!(none_left.is_empty());
    assert!(all.contains("10 ms") && all.contains("21 ms") && !all.contains("<<<<<<<"), "{all}");
    assert!(crate::Compiler::new(crate::CompilerConfig::default()).compile_string(&all).is_ok(), "{all}");
}

#[test]
fn an_element_comes_back_as_it_was_even_once_removed() {
    let changed = TWO.replace("1 ms", "9 ms");
    let back = restore_element(&changed, TWO, "LC-A").unwrap().unwrap();
    assert_eq!(back, TWO, "its text replaced");
    let removed = TWO.replace("    component \"B\" {\n        id: \"LC-B\"\n        latency: 2 ms\n    }\n", "");
    let back = restore_element(&removed, TWO, "LC-B").unwrap().unwrap();
    assert!(back.contains("component \"B\" {\n        id: \"LC-B\"\n        latency: 2 ms\n    }\n}"), "put back in its parent: {back}");
    assert!(crate::Compiler::new(crate::CompilerConfig::default()).compile_string(&back).is_ok(), "{back}");
    assert_eq!(restore_element(TWO, TWO, "LC-Z").unwrap(), None, "not in the old version");
}
