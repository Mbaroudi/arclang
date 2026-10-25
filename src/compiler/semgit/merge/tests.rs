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
