//! The order the instructions of a translated function are written in.
//!
//! A compiler places the rare paths of a loop after the code that follows it,
//! each ending with a jump back into the loop. The translation writes the
//! instructions in the order of their addresses: the jump back is then a jump
//! to a label written before it, over everything in between. A prover that
//! unwinds a loop for each such jump unwinds, with it, every loop written
//! between the two, once more for each time the path is taken.
//!
//! `in_line` writes such a piece just before the label it goes back to. What
//! runs does not change: a piece is moved only when nothing falls into it,
//! when it ends with a jump to a label written before it, and when every jump
//! into it is written before that label. Every jump concerned then goes down
//! the text, and no jump is made to go up it.

use regex::Regex;
use std::borrow::Cow;

/// An instruction: its label, then the lines of its operations.
struct Instruction<'a> {
    at: u32,
    lines: Vec<Cow<'a, str>>,
    /// The instructions of the function it jumps to, itself apart.
    jumps: Vec<u32>,
}

impl Instruction<'_> {
    /// Whether the instruction written next runs after this one without a jump.
    fn falls_through(&self) -> bool {
        !self.lines.last().is_some_and(|line| line.starts_with("    goto ") || line == "    return;")
    }
}

/// The address a line that only jumps goes to.
fn only_jump(line: &str) -> Option<u32> {
    let at = line.strip_prefix("    goto L_")?.strip_suffix(';')?;
    (at.len() == 4).then(|| u32::from_str_radix(at, 16).ok()).flatten()
}

/// The instructions of one function, each piece that goes back into a loop written before the label it goes back to.
fn ordered(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    // Each move takes a jump up the text away and adds none: the moves end.
    'again: loop {
        let place = |at: u32| instructions.iter().position(|instruction| instruction.at == at);
        let mut first = 0;
        while first < instructions.len() {
            let mut last = first;
            while last + 1 < instructions.len() && instructions[last].falls_through() {
                last += 1;
            }
            let back = instructions[last].lines.last().and_then(|line| only_jump(line)).and_then(|to| place(to).map(|rank| (to, rank))).filter(|(_, rank)| *rank < first);
            if let Some((to, target)) = back.filter(|_| first > 0) {
                let piece: Vec<u32> = instructions[first..=last].iter().map(|instruction| instruction.at).collect();
                let entered_from: Vec<usize> = instructions.iter().enumerate().filter(|(rank, instruction)| (*rank < first || *rank > last) && instruction.jumps.iter().any(|to| piece.contains(to))).map(|(rank, _)| rank).collect();
                if !entered_from.is_empty() && entered_from.iter().all(|rank| *rank < target) {
                    let moved: Vec<Instruction> = instructions.drain(first..=last).collect();
                    // What fell into the label now jumps to it, over the piece.
                    if target > 0 && instructions[target - 1].falls_through() {
                        instructions[target - 1].lines.push(Cow::Owned(format!("    goto L_{to:04x};")));
                    }
                    instructions.splice(target..target, moved);
                    continue 'again;
                }
            }
            first = last + 1;
        }
        return instructions;
    }
}

/// The translation, the pieces of each function that go back into a loop written before the label they go back to.
pub fn in_line(lifted: &str) -> String {
    let head = Regex::new(r"^void f_[0-9a-f]{4}\(void\)$").expect("a pattern of the order");
    let label = Regex::new(r"^L_([0-9a-f]{4}): /\*").expect("a pattern of the order");
    let jump = Regex::new(r"goto L_([0-9a-f]{4})(?:_\d+)?;").expect("a pattern of the order");
    let lines: Vec<&str> = lifted.lines().collect();
    let mut written: Vec<Cow<str>> = Vec::with_capacity(lines.len());
    let mut rank = 0;
    while rank < lines.len() {
        if !head.is_match(lines[rank]) {
            written.push(Cow::Borrowed(lines[rank]));
            rank += 1;
            continue;
        }
        let end = lines[rank..].iter().position(|line| *line == "}").map_or(lines.len(), |length| rank + length);
        let body = &lines[rank..end];
        // The last line of a function says it ran off its end: it stays last.
        let (Some(first), Some(tail)) = (body.iter().position(|line| label.is_match(line)), body.len().checked_sub(1)) else {
            written.extend(body.iter().map(|line| Cow::Borrowed(*line)));
            rank = end;
            continue;
        };
        let mut instructions: Vec<Instruction> = Vec::new();
        for line in &body[first..tail.max(first)] {
            if let Some(found) = label.captures(line) {
                instructions.push(Instruction { at: u32::from_str_radix(&found[1], 16).unwrap_or_default(), lines: Vec::new(), jumps: Vec::new() });
            }
            if let Some(instruction) = instructions.last_mut() {
                instruction.lines.push(Cow::Borrowed(*line));
                let at = instruction.at;
                instruction.jumps.extend(jump.captures_iter(line).filter_map(|to| u32::from_str_radix(&to[1], 16).ok()).filter(|to| *to != at));
            }
        }
        written.extend(body[..first].iter().map(|line| Cow::Borrowed(*line)));
        written.extend(ordered(instructions).into_iter().flat_map(|instruction| instruction.lines));
        written.extend(body[tail.max(first)..].iter().map(|line| Cow::Borrowed(*line)));
        rank = end;
    }
    written.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOOP: &str = "void f_0100(void)\n{\n    goto L_0100;\nL_0100: /* cpi */\n    if (a) goto L_0110;\nL_0102: /* body */\n    x = 1;\nL_0104: /* dec */\n    if (b) goto L_0100;\nL_0106: /* inner loop */\n    if (c) goto L_0106;\nL_0108: /* ret */\n    return;\nL_0110: /* rare */\n    y = 1;\nL_0112: /* rjmp 0x102 */\n    goto L_0102;\n    arclang_uncovered = true; /* the function runs off its end */\n}\n";

    #[test]
    fn a_piece_that_goes_back_into_a_loop_is_written_before_the_label_it_goes_back_to() {
        let written = in_line(LOOP);
        let order: Vec<&str> = written.lines().filter(|line| line.starts_with("L_")).map(|line| &line[..6]).collect();
        assert_eq!(order, ["L_0100", "L_0110", "L_0112", "L_0102", "L_0104", "L_0106", "L_0108"]);
        // What fell into the label jumps over the piece; the function still ends on the line that says it ran off its end.
        assert!(written.contains("    if (a) goto L_0110;\n    goto L_0102;\nL_0110: /* rare */"), "{written}");
        assert!(written.ends_with("    return;\n    arclang_uncovered = true; /* the function runs off its end */\n}\n"), "{written}");
        // Every line is kept, and one jump is added.
        assert_eq!(written.lines().count(), LOOP.lines().count() + 1);
    }

    #[test]
    fn what_is_already_in_line_and_what_is_no_function_are_left_as_they_are() {
        let written = in_line(LOOP);
        assert_eq!(in_line(&written), written);
        let other = "static void arclang_ijmp(uint64_t word)\n{\n    switch (word) {\n    case 0x0u: c_0000(); return;\n    }\n}\n";
        assert_eq!(in_line(other), other);
    }

    #[test]
    fn a_piece_entered_from_inside_the_loop_it_goes_back_to_is_not_moved() {
        // The tail of a loop, placed after its head and jumped to from its body: moved up, the jump to it would go up the text.
        let tail = "void f_0200(void)\n{\n    goto L_0200;\nL_0200: /* head */\n    x = 1;\nL_0202: /* rjmp 0x210 */\n    goto L_0210;\nL_0204: /* ret */\n    return;\nL_0210: /* tail */\n    y = 1;\nL_0212: /* rjmp 0x200 */\n    goto L_0200;\n    arclang_uncovered = true; /* the function runs off its end */\n}\n";
        assert_eq!(in_line(tail), tail);
    }

    #[test]
    fn a_piece_something_falls_into_is_not_moved() {
        let falls = "void f_0300(void)\n{\n    goto L_0300;\nL_0300: /* head */\n    if (a) goto L_0304;\nL_0302: /* body */\n    x = 1;\nL_0304: /* rjmp 0x300 */\n    goto L_0300;\n    arclang_uncovered = true; /* the function runs off its end */\n}\n";
        assert_eq!(in_line(falls), falls);
    }
}
