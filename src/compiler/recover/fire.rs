//! The body of a transition's function, read as what it does in order: the
//! states it exits and enters, the actions it runs, and where it goes through
//! a pseudo-state: the branches of a choice (`if / else if / else`), those of
//! a junction (`switch (alternative)`), an entry by history, a terminate.

use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Step {
    Exit(usize, usize),
    Enter(usize),
    Action(String),
    /// An entry by the history pseudo-state `pseudo`; `fallback` is what is
    /// entered when nothing was remembered.
    History { pseudo: usize, fallback: Vec<Step> },
    Terminate,
    /// The branches of a choice, in order: the condition of each as the C
    /// writes it (`None` for the final `else`), and what it does.
    Choice(Vec<(Option<String>, Vec<Step>)>),
    /// The branches of a junction, by the value of `alternative`.
    Switch(Vec<(usize, Vec<Step>)>),
}

pub(super) struct Reader<'a> {
    enter: Regex,
    exit: Regex,
    history: Regex,
    branch: Regex,
    again: Regex,
    case: Regex,
    action_of: &'a dyn Fn(&str) -> Option<String>,
}

impl<'a> Reader<'a> {
    pub(super) fn new(id: &str, action_of: &'a dyn Fn(&str) -> Option<String>) -> Self {
        let re = |pattern: String| Regex::new(&pattern).expect("a pattern of the reader");
        Reader {
            enter: re(format!(r"^\s*{id}_enter_one\(st, in, out, (\d+)U\);")),
            exit: re(format!(r"^\s*{id}_exit_range\(st, in, out, (\d+)U, (\d+)U\);")),
            history: re(r"^\s*if \(st->(?:has_(\d+)|hist_(\d+) != NO_STATE)\) \{".to_string()),
            branch: re(r"^\s*if \((.+)\) \{$".to_string()),
            again: re(r"^\s*\} else if \((.+)\) \{$".to_string()),
            case: re(r"^\s*case (\d+)U:$".to_string()),
            action_of,
        }
    }

    /// The steps of `lines`, the statements of one block.
    pub(super) fn steps(&self, lines: &[&str]) -> Vec<Step> {
        let mut steps = Vec::new();
        let mut at = 0;
        while at < lines.len() {
            let line = lines[at];
            if let Some(found) = self.exit.captures(line) {
                steps.push(Step::Exit(found[1].parse().unwrap_or_default(), found[2].parse().unwrap_or_default()));
            } else if let Some(found) = self.enter.captures(line) {
                steps.push(Step::Enter(found[1].parse().unwrap_or_default()));
            } else if let Some(text) = (self.action_of)(line) {
                steps.push(Step::Action(text));
            } else if let Some(found) = self.history.captures(line) {
                // if (remembered) { ...enter what was... } else { ...the default... }
                let pseudo = found.get(1).or(found.get(2)).and_then(|n| n.as_str().parse().ok()).unwrap_or_default();
                let segments = self.chain(lines, at);
                let fallback = segments.segments.get(1).map(|(_, body)| self.steps(body)).unwrap_or_default();
                steps.push(Step::History { pseudo, fallback });
                at = segments.end;
            } else if line.trim_start().starts_with("{ /* terminate") {
                steps.push(Step::Terminate);
                at = close(lines, at);
            } else if line.trim() == "switch (alternative) {" {
                let end = close(lines, at);
                let mut cases: Vec<(usize, Vec<Step>)> = Vec::new();
                let mut inner = at + 1;
                while inner < end {
                    if let Some(value) = self.case.captures(lines[inner]) {
                        let stop = (inner + 1..end).find(|&k| lines[k].trim() == "break;").unwrap_or(end);
                        cases.push((value[1].parse().unwrap_or_default(), self.steps(&lines[inner + 1..stop])));
                        inner = stop;
                    }
                    inner += 1;
                }
                steps.push(Step::Switch(cases));
                at = end;
            } else if self.branch.is_match(line) {
                let chain = self.chain(lines, at);
                steps.push(Step::Choice(chain.segments.iter().map(|(condition, body)| (condition.clone(), self.steps(body))).collect()));
                at = chain.end;
            } else if line.trim_end().ends_with('{') {
                // A block this reader has no use for (a loop): passed over whole.
                at = close(lines, at);
            }
            at += 1;
        }
        steps
    }

    /// The `if / else if / else` chain that opens at `lines[at]`.
    fn chain<'l>(&self, lines: &'l [&'l str], at: usize) -> Chain<'l> {
        let mut segments = Vec::new();
        let mut condition = self.branch.captures(lines[at]).map(|found| found[1].to_string());
        let mut from = at + 1;
        let mut depth = 1usize;
        let mut k = at + 1;
        while k < lines.len() {
            let line = lines[k].trim();
            if depth == 1 && line.starts_with('}') {
                segments.push((condition.take(), &lines[from..k]));
                if line == "}" {
                    return Chain { segments, end: k };
                }
                // `} else if (c) {` or `} else {`: the next branch.
                condition = self.again.captures(lines[k]).map(|found| found[1].to_string());
                from = k + 1;
            } else {
                depth = (depth + line.matches('{').count()).saturating_sub(line.matches('}').count());
            }
            k += 1;
        }
        Chain { segments, end: lines.len().saturating_sub(1) }
    }
}

struct Chain<'l> {
    segments: Vec<(Option<String>, &'l [&'l str])>,
    /// The line of the closing brace.
    end: usize,
}

/// The line that closes the block opened at `lines[at]`.
fn close(lines: &[&str], at: usize) -> usize {
    let mut depth = 0usize;
    for (k, line) in lines.iter().enumerate().skip(at) {
        depth += line.matches('{').count();
        depth = depth.saturating_sub(line.matches('}').count());
        if depth == 0 {
            return k;
        }
    }
    lines.len().saturating_sub(1)
}

/// The states the steps enter, in order, each sequence being one way through
/// them: the steps themselves, and each branch of a choice or of a junction.
pub(super) fn sequences(steps: &[Step]) -> Vec<Vec<usize>> {
    let mut own = Vec::new();
    let mut all = Vec::new();
    for step in steps {
        match step {
            Step::Enter(state) => own.push(*state),
            Step::History { fallback, .. } => own.extend(entered(fallback)),
            Step::Choice(branches) => all.extend(branches.iter().flat_map(|(_, body)| sequences(body))),
            Step::Switch(cases) => all.extend(cases.iter().flat_map(|(_, body)| sequences(body))),
            _ => {}
        }
    }
    all.insert(0, own);
    all
}

/// The states the steps enter themselves, branches apart.
pub(super) fn entered(steps: &[Step]) -> Vec<usize> {
    steps.iter().filter_map(|step| if let Step::Enter(state) = step { Some(*state) } else { None }).collect()
}

/// The first range of states the steps exit, wherever it is written.
pub(super) fn first_exit(steps: &[Step]) -> Option<(usize, usize)> {
    steps.iter().find_map(|step| match step {
        Step::Exit(lo, hi) => Some((*lo, *hi)),
        Step::Choice(branches) => branches.iter().find_map(|(_, body)| first_exit(body)),
        Step::Switch(cases) => cases.iter().find_map(|(_, body)| first_exit(body)),
        _ => None,
    })
}

pub(super) fn actions(steps: &[Step]) -> Vec<String> {
    steps.iter().filter_map(|step| if let Step::Action(text) = step { Some(text.clone()) } else { None }).collect()
}

/// The history pseudo-states the steps enter by.
pub(super) fn histories(steps: &[Step]) -> Vec<usize> {
    steps.iter().filter_map(|step| if let Step::History { pseudo, .. } = step { Some(*pseudo) } else { None }).collect()
}
