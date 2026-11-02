//! The behavior of a Capella model: state machines with their regions,
//! pseudo-states and transitions, and scenarios with their lifelines,
//! messages and combined fragments.

use super::{data, display_name, first_ref, header_attributes, refs, text_attr, Node, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{HashMap, HashSet};

// --- state machines -------------------------------------------------------------------

/// The kind a pseudo-state is written with: ArcLang draws it as a state of that kind.
fn pseudo_kind(node: &Node) -> Option<&'static str> {
    Some(match node.kind.as_str() {
        "capellacommon:ChoicePseudoState" => "choice",
        "capellacommon:ForkPseudoState" => "fork",
        "capellacommon:JoinPseudoState" => "join",
        "capellacommon:FinalState" => "final",
        "capellacommon:TerminatePseudoState" => "terminate",
        "capellacommon:DeepHistoryPseudoState" => "deep_history",
        "capellacommon:ShallowHistoryPseudoState" => "shallow_history",
        "capellacommon:EntryPointPseudoState" => "entry_point",
        "capellacommon:ExitPointPseudoState" => "exit_point",
        _ => return None,
    })
}

/// Every state machine of the model, wherever it is owned: its regions are
/// written as nested states, its transitions where they are declared.
pub(super) fn state_machines(writer: &mut Writer, engineering: &Node) {
    let mut taken: HashSet<String> = HashSet::new();
    let unique = unique_names(writer);
    // The element owning each machine, to tell machines of one name apart.
    // Whatever holds it: a component (ownedStateMachines), a port's protocol
    // (ownedProtocols)...
    let mut owners: HashMap<String, String> = HashMap::new();
    let mut owner_ids: HashMap<String, String> = HashMap::new();
    for node in engineering.descendants() {
        for machine in node.children.iter().filter(|c| c.is("capellacommon:StateMachine")) {
            owners.insert(machine.id().to_string(), display_name(node));
            owner_ids.insert(machine.id().to_string(), node.id().to_string());
        }
    }
    let all: Vec<&Node> = engineering.find_all("capellacommon:StateMachine");
    // Names counted over the machines written (those with a state): a
    // machine left out does not force a prefix on another.
    let has_states = |machine: &Node| machine.tagged("ownedRegions").any(|r| r.tagged("ownedStates").next().is_some());
    let mut names_seen: HashMap<String, usize> = HashMap::new();
    for machine in all.iter().filter(|m| has_states(m)) {
        *names_seen.entry(display_name(machine)).or_insert(0) += 1;
    }
    for machine in all {
        let regions: Vec<&Node> = machine.tagged("ownedRegions").collect();
        if regions.iter().all(|r| r.tagged("ownedStates").next().is_none()) {
            writer.count("state machines without a state");
        }
        writer.count("state machines");
        writer.remember(machine, "StateMachine");
        let base = display_name(machine);
        let preferred = match owners.get(machine.id()) {
            Some(owner) if names_seen.get(&base).copied().unwrap_or(0) > 1 => format!("{owner} / {base}"),
            _ => base,
        };
        let name = data::unique_name(&mut taken, &preferred);
        writer.names.insert(machine.id().to_string(), name.clone());
        writer.line(0, &format!("state_machine {} {{", string_literal(&name)));
        // The element owning the machine, so an export gives it back.
        if let Some(owner) = owner_ids.get(machine.id()).and_then(|id| written_holder(writer, engineering, id)) {
            writer.line(1, &format!("owner: {}", string_literal(&owner)));
        }
        writer.line(1, &format!("id: {}", string_literal(machine.id())));
        data::applied_values(writer, 1, machine, "StateMachine");
        // Every state of the machine, at any depth, by a name unique in the machine.
        let mut state_names: HashMap<String, String> = HashMap::new();
        let mut names: HashSet<String> = HashSet::new();
        fn name_states(regions: &[&Node], state_names: &mut HashMap<String, String>, names: &mut HashSet<String>) {
            for region in regions {
                for state in region.tagged("ownedStates") {
                    if state.is("capellacommon:InitialPseudoState") {
                        continue;
                    }
                    let name = data::unique_name(names, &display_name(state));
                    state_names.insert(state.id().to_string(), name);
                    let nested: Vec<&Node> = state.tagged("ownedRegions").collect();
                    name_states(&nested, state_names, names);
                }
            }
        }
        name_states(&regions, &mut state_names, &mut names);
        for (id, name) in &state_names {
            writer.names.insert(id.clone(), name.clone());
        }
        let events: HashMap<String, String> = engineering
            .descendants()
            .into_iter()
            .filter(|n| n.kind.ends_with("Event") && n.kind.starts_with("capellacommon:"))
            .map(|e| (e.id().to_string(), display_name(e)))
            .collect();
        let populated: Vec<&Node> = regions.iter().copied().filter(|r| r.tagged("ownedStates").next().is_some()).collect();
        if populated.len() > 1 {
            // Several regions, each with its initial state: `region` blocks.
            let mut region_names: HashSet<String> = HashSet::new();
            for region in &populated {
                writer.count("state machine regions (written as region blocks)");
                let region_name = data::unique_name(&mut region_names, &display_name(region));
                writer.line(1, &format!("region {} {{", string_literal(&region_name)));
                writer.line(2, &format!("id: {}", string_literal(region.id())));
                write_region(writer, 2, region, &state_names, &events, true, &unique);
                writer.line(1, "}");
            }
        } else {
            for region in &regions {
                write_region(writer, 1, region, &state_names, &events, true, &unique);
            }
        }
        writer.line(0, "}");
        writer.line(0, "");
    }
}

/// A region: its initial state, its states (nested regions inside them), its transitions.
fn write_region(writer: &mut Writer, depth: usize, region: &Node, state_names: &HashMap<String, String>, events: &HashMap<String, String>, top: bool, unique: &HashSet<String>) {
    let states: Vec<&Node> = region.tagged("ownedStates").collect();
    let initial: Option<&Node> = states.iter().copied().find(|s| s.is("capellacommon:InitialPseudoState"));
    let transitions: Vec<&Node> = region.tagged("ownedTransitions").collect();
    // The initial state: where the initial pseudo-state's transition goes.
    let initial_target = initial.and_then(|i| {
        transitions
            .iter()
            .find(|t| first_ref(t.attr("source")).as_deref() == Some(i.id()))
            .and_then(|t| first_ref(t.attr("target")))
            .and_then(|target| state_names.get(&target).cloned())
    });
    if let Some(target) = initial_target.as_ref() {
        writer.line(depth, &format!("initial: {}", string_literal(target)));
    }
    for state in states.iter().copied().filter(|s| !s.is("capellacommon:InitialPseudoState")) {
        let Some(name) = state_names.get(state.id()) else { continue };
        let keyword = if state.is("capellacommon:Mode") { "mode" } else { "state" };
        writer.count(if keyword == "mode" { "modes" } else { "states" });
        writer.remember(state, "State");
        for realization in state.tagged("ownedAbstractStateRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((state.id().to_string(), "realizes", target));
            }
        }
        writer.names.insert(state.id().to_string(), name.clone());
        writer.line(depth, &format!("{keyword} {} {{", string_literal(name)));
        header_attributes(writer, depth + 1, state, "State");
        if let Some(kind) = pseudo_kind(state) {
            writer.line(depth + 1, &format!("kind: {}", string_literal(kind)));
        }
        for (tag, key) in [("entry", "entry"), ("exit", "exit"), ("doActivity", "do")] {
            let names = behaviours_of(writer, &refs(state.attr(tag)), unique, false);
            if !names.is_empty() {
                writer.line(depth + 1, &format!("{key}: {}", super::list_literal(&names)));
            }
        }
        let nested: Vec<&Node> = state.tagged("ownedRegions").collect();
        for sub in nested {
            write_region(writer, depth + 1, sub, state_names, events, false, unique);
        }
        writer.line(depth, "}");
    }
    let _ = top;
    for transition in transitions {
        let (Some(source), Some(target)) = (first_ref(transition.attr("source")), first_ref(transition.attr("target"))) else { continue };
        if initial.is_some_and(|i| i.id() == source) {
            continue; // the initial state, written above
        }
        let (Some(from), Some(to)) = (state_names.get(&source), state_names.get(&target)) else {
            writer.count("transitions to states not written (left out)");
            continue;
        };
        writer.count("transitions");
        writer.remember(transition, "Transition");
        for realization in transition.tagged("ownedStateTransitionRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((transition.id().to_string(), "realizes", target));
            }
        }
        let mut attributes = vec![format!("id: {}", string_literal(transition.id()))];
        let trigger = text_attr(transition, "triggerDescription").unwrap_or_default();
        attributes.push(format!("trigger: {}", string_literal(&trigger)));
        let triggers: Vec<String> = refs(transition.attr("triggers")).iter().filter_map(|id| events.get(id).cloned()).collect();
        if !triggers.is_empty() {
            attributes.push(format!("events: {}", super::list_literal(&triggers)));
        }
        // Capella 6 owns the guard (`ownedGuard`); Capella 7 references one
        // of the transition's constraints (`guard="#id"`).
        let referenced = first_ref(transition.attr("guard"));
        let guard = transition
            .tagged("ownedGuard")
            .next()
            .or_else(|| transition.tagged("ownedConstraints").find(|c| referenced.as_deref() == Some(c.id())));
        if let Some(guard) = guard {
            let text = data::constraint_text(guard);
            if !text.is_empty() {
                attributes.push(format!("guard: {}", string_literal(&text)));
            }
        }
        // The effects, as a list: a name may hold a comma.
        let effects = behaviours_of(writer, &refs(transition.attr("effect")), unique, true);
        if !effects.is_empty() {
            attributes.push(format!("action: {}", super::list_literal(&effects)));
        }
        writer.line(depth, &format!("transition {} -> {} {{ {} }}", string_literal(from), string_literal(to), attributes.join(" ")));
    }
}

// --- scenarios -----------------------------------------------------------------------

/// The nearest written element holding `id`: the owner itself, else the
/// first written ancestor (a machine owned by a pruned root component
/// belongs to the element above it).
fn written_holder(writer: &Writer, engineering: &Node, id: &str) -> Option<String> {
    let mut parent_of: HashMap<&str, &str> = HashMap::new();
    for node in engineering.descendants() {
        for child in &node.children {
            if !child.id().is_empty() {
                parent_of.insert(child.id(), node.id());
            }
        }
    }
    let mut current = id;
    loop {
        if writer.written(current) {
            return Some(writer.ids[current].clone());
        }
        // The level itself (its block carries the architecture's id) when
        // nothing written lies between.
        if let Some(level) = engineering.tagged("ownedArchitectures").find(|a| a.id() == current) {
            return Some(level.id().to_string());
        }
        current = parent_of.get(current)?;
    }
}

/// What a state's entry, exit or do, or a transition's effect, names: a
/// function or activity, by its name when no other has it, else by its id.
/// Another written element (an exchange item...) is kept by id when
/// `keep_others` (a transition effect may be one); else left out.
fn behaviours_of(writer: &mut Writer, ids: &[String], unique: &HashSet<String>, keep_others: bool) -> Vec<String> {
    let mut named = Vec::new();
    for id in ids {
        match writer.kinds.get(id).copied() {
            Some("SystemFunction" | "LogicalFunction" | "PhysicalFunction" | "OperationalActivity") => {
                let name = writer.names.get(id).cloned().unwrap_or_default();
                named.push(if unique.contains(&format!("function:{name}")) { name } else { writer.ids[id].clone() });
            }
            Some(_) if keep_others => named.push(writer.ids[id].clone()),
            Some(_) => writer.count("state entry/exit/do naming something that is not a function (left out)"),
            None => writer.count("state entry/exit/do or effect naming an element not written (left out)"),
        }
    }
    named
}

/// What a lifeline stands for: the element its instance role represents.
fn represented(writer: &Writer, role: &Node) -> Option<String> {
    let instance = first_ref(role.attr("representedInstance"))?;
    if writer.written(&instance) {
        return Some(writer.ids[&instance].clone());
    }
    let typed = writer.part_type.get(&instance)?;
    writer.ids.get(typed).cloned()
}

/// A combined fragment or an operand, as a range of fragment indexes; or a
/// state fragment, the state or function a lifeline is in over that range.
struct Frame<'a> {
    node: &'a Node,
    start: usize,
    end: usize,
    operands: Vec<(Option<&'a Node>, usize, usize)>,
    invariant: Option<Invariant>,
}

/// What a state fragment asserts: `state`/`function`, on which lifeline,
/// which element (by name when unique, else by id).
struct Invariant {
    keyword: &'static str,
    lifeline: String,
    reference: String,
}

/// The state fragments of a scenario, as frames; counts the ones whose
/// state or function is not written.
fn state_fragments<'a>(writer: &mut Writer, scenario: &'a Node, index_of: &HashMap<String, usize>, fragments: &[&Node], covered: &dyn Fn(&Node) -> Option<String>, unique: &HashSet<String>) -> Vec<Frame<'a>> {
    let mut frames: Vec<Frame<'a>> = Vec::new();
    for lapse in scenario.tagged("ownedTimeLapses").filter(|l| l.is("interaction:StateFragment")) {
        let (Some(start), Some(end)) = (
            first_ref(lapse.attr("start")).and_then(|e| index_of.get(&e).copied()),
            first_ref(lapse.attr("finish")).and_then(|e| index_of.get(&e).copied()),
        ) else { continue };
        let Some(lifeline) = covered(fragments[start]) else {
            writer.count("state fragments on a lifeline not written (left out)");
            continue;
        };
        let (keyword, related) = match (first_ref(lapse.attr("relatedAbstractState")), first_ref(lapse.attr("relatedAbstractFunction"))) {
            (Some(state), _) => ("state", state),
            (None, Some(function)) => ("function", function),
            (None, None) => { writer.count("state fragments asserting nothing (left out)"); continue }
        };
        if !writer.written(&related) {
            writer.count("state fragments of a state or function not written (left out)");
            continue;
        }
        let name = writer.names.get(&related).cloned().unwrap_or_default();
        let reference = if unique.contains(&format!("{keyword}:{name}")) { name } else { writer.ids[&related].clone() };
        frames.push(Frame { node: lapse, start, end, operands: Vec::new(), invariant: Some(Invariant { keyword, lifeline, reference }) });
    }
    frames
}

/// The names that denote one state (or one function) only, as `state:Name`
/// / `function:Name`: an invariant names its element so, else by id.
fn unique_names(writer: &Writer) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (id, kind) in &writer.kinds {
        let group = match *kind {
            "State" => "state",
            "SystemFunction" | "LogicalFunction" | "PhysicalFunction" | "OperationalActivity" => "function",
            _ => continue,
        };
        if let Some(name) = writer.names.get(id) {
            *counts.entry(format!("{group}:{name}")).or_insert(0) += 1;
        }
    }
    counts.into_iter().filter(|(_, n)| *n == 1).map(|(key, _)| key).collect()
}

pub(super) fn scenarios(writer: &mut Writer, engineering: &Node) {
    let mut taken: HashSet<String> = HashSet::new();
    let unique = unique_names(writer);
    let all: Vec<&Node> = engineering.find_all("interaction:Scenario");
    // Names counted over the scenarios written (one lifeline stands for a
    // written element): a scenario left out does not force a prefix.
    let has_lifeline = |scenario: &Node| scenario.tagged("ownedInstanceRoles").any(|role| represented(writer, role).is_some());
    let mut names_seen: HashMap<String, usize> = HashMap::new();
    for scenario in all.iter().filter(|s| has_lifeline(s)) {
        *names_seen.entry(display_name(scenario)).or_insert(0) += 1;
    }
    let mut owners: HashMap<String, String> = HashMap::new();
    let mut owner_ids: HashMap<String, String> = HashMap::new();
    for node in engineering.descendants() {
        for scenario in node.tagged("ownedScenarios") {
            owners.insert(scenario.id().to_string(), display_name(node));
            owner_ids.insert(scenario.id().to_string(), node.id().to_string());
        }
    }
    for scenario in all {
        // Lifelines, and what they stand for.
        let mut lifeline_of: HashMap<String, String> = HashMap::new();
        let mut participants: Vec<String> = Vec::new();
        let mut unresolved = 0;
        for role in scenario.tagged("ownedInstanceRoles") {
            match represented(writer, role) {
                Some(id) => {
                    if !participants.contains(&id) {
                        participants.push(id.clone());
                    }
                    lifeline_of.insert(role.id().to_string(), id);
                }
                None => unresolved += 1,
            }
        }
        if participants.is_empty() {
            writer.count("scenarios whose lifelines are not imported elements (left out)");
            continue;
        }
        if unresolved > 0 {
            writer.count_by("lifelines of elements not written (left out)", unresolved);
        }
        let fragments: Vec<&Node> = scenario.tagged("ownedInteractionFragments").collect();
        let index_of: HashMap<String, usize> = fragments.iter().enumerate().map(|(i, f)| (f.id().to_string(), i)).collect();
        // What each message carries: the operation of its sending end's event.
        let events: HashMap<String, &Node> = scenario.tagged("ownedEvents").map(|e| (e.id().to_string(), e)).collect();
        let mut carried: HashMap<String, String> = HashMap::new();
        for message in scenario.tagged("ownedMessages") {
            let operation = first_ref(message.attr("sendingEnd"))
                .and_then(|end| index_of.get(&end).map(|i| fragments[*i]))
                .and_then(|end| first_ref(end.attr("event")))
                .and_then(|event| events.get(&event).copied())
                .and_then(|event| first_ref(event.attr("operation")));
            if let Some(operation) = operation.filter(|id| writer.written(id)) {
                carried.insert(message.id().to_string(), writer.ids[&operation].clone());
            }
        }
        let covered = |fragment: &Node| first_ref(fragment.attr("coveredInstanceRoles")).and_then(|role| lifeline_of.get(&role).cloned());
        // Messages, in the order of their sending ends.
        let mut messages: Vec<(usize, &Node, String, String)> = Vec::new();
        let mut dropped = 0;
        for message in scenario.tagged("ownedMessages") {
            let ends = (
                first_ref(message.attr("sendingEnd")).and_then(|e| index_of.get(&e).copied()),
                first_ref(message.attr("receivingEnd")).and_then(|e| index_of.get(&e).copied()),
            );
            let (Some(send), Some(receive)) = ends else { dropped += 1; continue };
            let (Some(from), Some(to)) = (covered(fragments[send]), covered(fragments[receive])) else { dropped += 1; continue };
            messages.push((send, message, from, to));
        }
        if dropped > 0 {
            writer.count_by("messages of lifelines not written (left out)", dropped);
        }
        messages.sort_by_key(|(at, _, _, _)| *at);
        if messages.is_empty() {
            writer.count("scenarios without a message");
        }
        // Combined fragments: a frame over a range of fragment indexes, with operands.
        let mut frames: Vec<Frame> = Vec::new();
        for lapse in scenario.tagged("ownedTimeLapses").filter(|l| l.is("interaction:CombinedFragment")) {
            let (Some(start), Some(end)) = (
                first_ref(lapse.attr("start")).and_then(|e| index_of.get(&e).copied()),
                first_ref(lapse.attr("finish")).and_then(|e| index_of.get(&e).copied()),
            ) else { continue };
            let mut operands: Vec<(Option<&Node>, usize, usize)> = Vec::new();
            let operand_ids = refs(lapse.attr("referencedOperands"));
            let mut starts: Vec<(usize, &Node)> = operand_ids.iter().filter_map(|id| index_of.get(id).map(|i| (*i, fragments[*i]))).collect();
            starts.sort_by_key(|(i, _)| *i);
            for (n, (at, operand)) in starts.iter().enumerate() {
                let until = starts.get(n + 1).map(|(next, _)| *next).unwrap_or(end);
                operands.push((Some(operand), *at, until));
            }
            frames.push(Frame { node: lapse, start, end, operands, invariant: None });
        }
        frames.extend(state_fragments(writer, scenario, &index_of, &fragments, &covered, &unique));
        frames.sort_by_key(|f| (f.start, std::cmp::Reverse(f.end)));

        writer.count("scenarios");
        writer.remember(scenario, "Scenario");
        let base = display_name(scenario);
        let preferred = match owners.get(scenario.id()) {
            Some(owner) if names_seen.get(&base).copied().unwrap_or(0) > 1 => format!("{owner} / {base}"),
            _ => base,
        };
        let name = data::unique_name(&mut taken, &preferred);
        // The compiler knows a scenario by its name: a trace names it so.
        writer.names.insert(scenario.id().to_string(), name.clone());
        writer.line(0, &format!("scenario {} {{", string_literal(&name)));
        header_attributes(writer, 1, scenario, "Scenario");
        if let Some(owner) = owner_ids.get(scenario.id()).and_then(|id| written_holder(writer, engineering, id)) {
            writer.line(1, &format!("owner: {}", string_literal(&owner)));
        }
        if let Some(kind) = scenario.attr("kind") {
            writer.line(1, &format!("kind: {}", string_literal(kind)));
        }
        writer.line(1, &format!("participants: {}", super::list_literal(&participants)));
        let count = write_frame_body(writer, 1, &messages, &frames, 0, usize::MAX, &mut 0, &carried);
        writer.count_by("messages", count);
        writer.line(0, "}");
        writer.line(0, "");
        for realization in scenario.tagged("ownedScenarioRealization") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((scenario.id().to_string(), "realizes", target));
            }
        }
    }
}

/// The messages and fragments between fragment indexes `from` and `to`,
/// the fragments nested as they are in Capella; returns the messages written.
fn write_frame_body(writer: &mut Writer, depth: usize, messages: &[(usize, &Node, String, String)], frames: &[Frame], from: usize, to: usize, next_frame: &mut usize, carried: &HashMap<String, String>) -> usize {
    let mut written = 0;
    let mut position = from;
    loop {
        // The next frame starting in this range, if any, else the messages to the end.
        let frame = frames.get(*next_frame).filter(|f| f.start >= position && f.start < to && f.end <= to.max(f.end));
        let stop = frame.map(|f| f.start).unwrap_or(to);
        for (at, message, source, target) in messages.iter().filter(|(at, _, _, _)| *at >= position && *at < stop) {
            let _ = at;
            write_message(writer, depth, message, source, target, carried.get(message.id()).map(String::as_str));
            written += 1;
        }
        let Some(frame) = frame else { break };
        *next_frame += 1;
        if let Some(invariant) = &frame.invariant {
            // A state fragment: the lifeline is in that state (or runs that
            // function) over the messages of the range.
            writer.count(if invariant.keyword == "state" { "state fragments (states on lifelines)" } else { "state fragments (functions on lifelines)" });
            writer.remember(frame.node, "StateFragment");
            writer.line(depth, &format!("{} {} {} {{", invariant.keyword, string_literal(&invariant.lifeline), string_literal(&invariant.reference)));
            writer.line(depth + 1, &format!("id: {}", string_literal(frame.node.id())));
            written += write_frame_body(writer, depth + 1, messages, frames, frame.start, frame.end, next_frame, carried);
            writer.line(depth, "}");
            position = frame.end;
            if position >= to {
                break;
            }
            continue;
        }
        let kind = match frame.node.attr("operator").unwrap_or("SEQ") {
            "ALT" => "alt",
            "OPT" => "opt",
            "LOOP" => "loop",
            "PAR" => "par",
            "CRITICAL" => "critical",
            _ => "seq",
        };
        let operator = frame.node.attr("operator").unwrap_or("SEQ");
        let label = text_attr(frame.node, "name").filter(|n| n != "combined fragment").map(|n| format!(" {}", string_literal(&n))).unwrap_or_default();
        writer.count("combined fragments");
        if matches!(operator, "ASSERT" | "NEG" | "IGNORE" | "CONSIDER" | "STRICT" | "WEAK") {
            writer.count("combined fragments of an operator ArcLang has no frame for (written as seq)");
        }
        writer.line(depth, &format!("fragment {kind}{label} {{"));
        let (start, end) = (frame.start, frame.end);
        if frame.operands.len() > 1 && matches!(kind, "alt" | "par") {
            for (operand, at, until) in &frame.operands {
                let guard = operand.and_then(|o| o.tagged("guard").next()).map(data::constraint_text).filter(|g| !g.is_empty()).map(|g| format!(" {}", string_literal(&g))).unwrap_or_default();
                writer.line(depth + 1, &format!("operand{guard} {{"));
                written += write_frame_body(writer, depth + 2, messages, frames, *at, *until, next_frame, carried);
                writer.line(depth + 1, "}");
            }
        } else {
            written += write_frame_body(writer, depth + 1, messages, frames, start, end, next_frame, carried);
        }
        writer.line(depth, "}");
        position = end;
        if position >= to {
            break;
        }
    }
    written
}

fn write_message(writer: &mut Writer, depth: usize, message: &Node, source: &str, target: &str, carried: Option<&str>) {
    writer.remember(message, "Message");
    let kind = message.attr("kind").unwrap_or("UNSET");
    let mut attributes = vec![format!("id: {}", string_literal(message.id()))];
    if let Some(exchange) = carried {
        writer.count("messages carrying an exchange");
        attributes.push(format!("exchange: {}", string_literal(exchange)));
    }
    match kind {
        "SYNCHRONOUS_CALL" => attributes.push("type: \"sync\"".to_string()),
        "ASYNCHRONOUS_CALL" => attributes.push("type: \"async\"".to_string()),
        _ => {}
    }
    if kind != "UNSET" {
        attributes.push(format!("kind: {}", string_literal(kind)));
    }
    let items = writer.known(&refs(message.attr("exchangedItems")));
    if !items.is_empty() {
        attributes.push(format!("exchange_items: {}", super::list_literal(&items)));
    }
    let label = display_name(message);
    writer.line(depth, &format!("message {} -> {} {} {{ {} }}", string_literal(source), string_literal(target), string_literal(&label), attributes.join(" ")));
}
