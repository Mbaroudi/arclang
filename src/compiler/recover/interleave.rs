//! An interrupt in the middle of `loop()`.
//!
//! The proof of a correspondence takes an interrupt between two `loop()`s.
//! A real one falls anywhere. What is asked here is whether that matters:
//! does an interrupt taken where the code uses something its routine
//! changes leave what an interrupt taken just before the `loop()`, or just
//! after it, would leave? Where it does, the interrupt in the middle is one
//! of the two the proof already covers. Where it does not, the code loses
//! or doubles something, and the case is shown.
//!
//! The places are found in the text of the code (the routine's body, what
//! it assigns) and the variables are watched by a definition put after
//! their declaration: each use of one, outside the routine and with
//! interrupts on, is a place where the interrupt may fall. That is between
//! two uses of a shared variable, not between two instructions of the
//! processor: `count++` is one use.

use regex::Regex;

fn uncommented(text: &str) -> Vec<String> {
    let comment = Regex::new(r"//[^\n]*").expect("a pattern of the sharing");
    text.lines().map(|line| comment.replace(line, "").into_owned()).collect()
}

/// The routine each line is in the body of, by the braces that follow its name.
fn bodies<'r>(lines: &[String], routines: &'r [String]) -> Vec<Option<&'r str>> {
    let mut inside: Vec<Option<&str>> = vec![None; lines.len()];
    for routine in routines {
        let head = Regex::new(&format!(r"\bvoid\s+{}\s*\(", regex::escape(routine))).expect("a pattern of the sharing");
        let Some(start) = lines.iter().position(|line| head.is_match(line)) else { continue };
        let (mut depth, mut opened) = (0i32, false);
        for (at, line) in lines.iter().enumerate().skip(start) {
            depth += line.matches('{').count() as i32;
            opened |= line.contains('{');
            inside[at] = Some(routine.as_str());
            depth -= line.matches('}').count() as i32;
            if opened && depth <= 0 {
                break;
            }
        }
    }
    inside
}

/// What each routine changes: (the routine, the variable), read in its body.
pub fn changed(sketches: &[(String, String)], routines: &[String]) -> Vec<(String, String)> {
    let assigned = Regex::new(r"\b([A-Za-z_]\w*)\s*(?:\+\+|--|(?:[+\-*/|&^%]|<<|>>)?=(?:[^=]|$))").expect("a pattern of the sharing");
    let before = Regex::new(r"(?:\+\+|--)\s*([A-Za-z_]\w*)").expect("a pattern of the sharing");
    let mut found = Vec::new();
    for (_, text) in sketches {
        let lines = uncommented(text);
        let inside = bodies(&lines, routines);
        for routine in routines {
            let mut names: Vec<String> = Vec::new();
            for (at, line) in lines.iter().enumerate() {
                if inside[at] == Some(routine.as_str()) {
                    names.extend(assigned.captures_iter(line).chain(before.captures_iter(line)).map(|c| c[1].to_string()));
                }
            }
            names.sort();
            names.dedup();
            found.extend(names.into_iter().map(|name| (routine.clone(), name)));
        }
    }
    found
}

/// Where the code uses, with interrupts on, what a routine changes: one
/// sentence for each variable, with its lines.
pub fn used_with_interrupts_on(sketches: &[(String, String)], routines: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for (file, text) in sketches {
        let lines = uncommented(text);
        let inside = bodies(&lines, routines);
        for (routine, variable) in changed(&[(file.clone(), text.clone())], routines) {
            let named = Regex::new(&format!(r"\b{}\b", regex::escape(&variable))).expect("a pattern of the sharing");
            let (mut depth, mut off) = (0i32, false);
            let mut places = Vec::new();
            for (at, line) in lines.iter().enumerate() {
                let within = depth > 0;
                depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if line.contains("noInterrupts()") {
                    off = true;
                }
                // Used in a function that is not a routine, with interrupts on.
                if within && inside[at].is_none() && !off && named.is_match(line) {
                    places.push((at + 1).to_string());
                }
                if line.contains("interrupts()") && !line.contains("noInterrupts()") {
                    off = false;
                }
            }
            if !places.is_empty() {
                found.push(format!("`{variable}`, which the routine `{routine}` changes, is used at line {} of {file} with interrupts on", places.join(", ")));
            }
        }
    }
    found
}

/// A declaration at the top of the code: (its type without `volatile` and `static`, its name), or what is not understood of it.
fn declaration(line: &str) -> Option<Result<(String, String), String>> {
    let scalar = Regex::new(r"^\s*((?:(?:static|volatile|const|unsigned|signed|long|short|int|char|bool|boolean|byte|word|float|double|u?int(?:8|16|32)_t)\s+)+)([A-Za-z_]\w*)\s*(?:=[^;{]*)?;\s*$").expect("a pattern of the declarations");
    let typed = Regex::new(r"^\s*(?:static|volatile|const|unsigned|signed|long|short|int|char|bool|boolean|byte|word|float|double|u?int(?:8|16|32)_t)\b").expect("a pattern of the declarations");
    if let Some(found) = scalar.captures(line) {
        let ty: Vec<&str> = found[1].split_whitespace().filter(|word| !matches!(*word, "static" | "volatile")).collect();
        return Some(Ok((ty.join(" "), found[2].to_string())));
    }
    // A declaration that is not one name of a simple type (an array, several names, a function) is told apart from a function by its `(`.
    (typed.is_match(line) && !line.contains('(') && line.contains(';')).then(|| Err(line.trim().to_string()))
}

/// The variables of the code that hold its state: its top-level names of a
/// simple type that are not constants. An array or a structure is not kept:
/// it is said, and the question is then not asked.
pub fn state(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut depth = 0i32;
    let mut kept = Vec::new();
    for line in uncommented(text) {
        if depth == 0 {
            match declaration(&line) {
                Some(Ok((ty, name))) if !ty.split_whitespace().any(|word| word == "const") => kept.push((ty, name)),
                Some(Ok(_)) | None => {}
                Some(Err(line)) => return Err(format!("the state of the code cannot be kept to compare: `{line}` is not one name of a simple type")),
            }
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
    }
    Ok(kept)
}

/// The code with each variable given watched: after its declaration, a
/// definition makes every later use of it pass by `arclang_point()`.
pub fn watched(text: &str, variables: &[String]) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    for line in text.lines() {
        out.push_str(line);
        out.push('\n');
        let bare = uncommented(line).pop().unwrap_or_default();
        if depth == 0 {
            if let Some(Ok((_, name))) = declaration(&bare) {
                if variables.contains(&name) {
                    out.push_str(&format!("#define {name} (*(arclang_point(), &{name})) /* watched: a routine of interrupt changes it */\n"));
                }
            }
        }
        depth += bare.matches('{').count() as i32 - bare.matches('}').count() as i32;
    }
    out
}

/// The C that keeps the state of the code, puts it back and compares two:
/// its variables, and what the stand-ins hold of its pins.
pub fn keeper(state: &[(String, String)], levels: usize, duties: usize) -> String {
    let mut fields: Vec<(String, String, String)> = state.iter().map(|(ty, name)| (ty.clone(), format!("v_{name}"), name.clone())).collect();
    fields.extend((0..levels).map(|k| ("bool".to_string(), format!("level_{k}"), format!("arclang_level_{k}"))));
    fields.extend((0..duties).map(|k| ("int".to_string(), format!("duty_{k}"), format!("arclang_duty_{k}"))));
    fields.push(("bool".to_string(), "unnamed".to_string(), "arclang_unnamed".to_string()));
    let mut c = String::from("\n/* ---- the state of the code, kept and compared ---- */\nstruct arclang_kept {\n");
    for (ty, field, _) in &fields {
        c.push_str(&format!("    {ty} {field};\n"));
    }
    c.push_str("};\nstatic void arclang_keep(struct arclang_kept *k)\n{\n");
    for (_, field, name) in &fields {
        c.push_str(&format!("    k->{field} = {name};\n"));
    }
    c.push_str("}\nstatic void arclang_put_back(const struct arclang_kept *k)\n{\n");
    for (_, field, name) in &fields {
        c.push_str(&format!("    {name} = k->{field};\n"));
    }
    c.push_str("}\nstatic bool arclang_same(const struct arclang_kept *a, const struct arclang_kept *b)\n{\n    return ");
    c.push_str(&fields.iter().map(|(_, field, _)| format!("a->{field} == b->{field}")).collect::<Vec<_>>().join(" && "));
    c.push_str(";\n}\n");
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKETCH: &str = "const int SENSOR = 2;\nvolatile int n = 0; // counted by the routine\nint seen = 0;\nunsigned long since;\nvoid tick() {\n  n++;\n}\nvoid loop() {\n  noInterrupts();\n  seen = n;\n  n = 0;\n  interrupts();\n  if (n > 3) { seen = 0; }\n}\n";

    fn sketch(text: &str) -> Vec<(String, String)> {
        vec![("s.ino".to_string(), text.to_string())]
    }

    #[test]
    fn what_a_routine_changes_and_where_the_code_uses_it_with_interrupts_on() {
        let routines = ["tick".to_string()];
        assert_eq!(changed(&sketch(SKETCH), &routines), [("tick".to_string(), "n".to_string())]);
        // Lines 10 and 11 are protected; line 13 is not. The declaration (line 2) is no use.
        assert_eq!(used_with_interrupts_on(&sketch(SKETCH), &routines), ["`n`, which the routine `tick` changes, is used at line 13 of s.ino with interrupts on"]);
        assert!(used_with_interrupts_on(&sketch(&SKETCH.replace("  if (n > 3) { seen = 0; }\n", "")), &routines).is_empty());
    }

    #[test]
    fn the_state_of_a_sketch_is_its_variables_not_its_constants() {
        assert_eq!(state(SKETCH).unwrap(), [("int".to_string(), "n".to_string()), ("int".to_string(), "seen".to_string()), ("unsigned long".to_string(), "since".to_string())]);
        // An array is not kept, and that is said.
        let refused = state("int samples[8];\nvoid loop() { }\n").unwrap_err();
        assert!(refused.contains("`int samples[8];` is not one name of a simple type"), "{refused}");
        // A function is not a declaration of state.
        assert!(state("int twice(int x);\nvoid loop() { int local = 3; }\n").unwrap().is_empty());
    }

    #[test]
    fn a_watched_variable_passes_by_the_point_at_each_use_after_its_declaration() {
        let text = watched(SKETCH, &["n".to_string()]);
        assert!(text.contains("volatile int n = 0; // counted by the routine\n#define n (*(arclang_point(), &n))"), "{text}");
        assert_eq!(text.matches("#define").count(), 1);
        let c = keeper(&state(SKETCH).unwrap(), 1, 0);
        assert!(c.contains("int v_n;") && c.contains("k->v_seen = seen;") && c.contains("arclang_level_0 = k->level_0;") && c.contains("a->v_since == b->v_since"), "{c}");
    }
}
