//! Scenarios: lifelines (instance roles of the parts, or of the functions,
//! they stand for), messages with their two ends, combined fragments with
//! their operands, and the states and functions asserted on lifelines.
//!
//! Capella orders a scenario by its interaction fragments: each message has
//! a sending and a receiving end, and a fragment or an invariant is bounded
//! by two ends placed among them. The import reads that order back.

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::{CombinedFragment, FragmentType, InvariantKind, MessageType, Scenario};

/// A frame over messages `start..=end` (empty when `end < start`).
struct Frame<'a> {
    start: usize,
    end: usize,
    what: What<'a>,
}

enum What<'a> {
    Fragment(&'a CombinedFragment, String),
    Invariant { id: String, lifeline: String, related: String, state: bool },
}

fn operator(kind: FragmentType) -> &'static str {
    match kind {
        FragmentType::Alt => "ALT",
        FragmentType::Opt => "OPT",
        FragmentType::Loop => "LOOP",
        FragmentType::Par => "PAR",
        FragmentType::Critical => "CRITICAL",
        FragmentType::Seq => "SEQ",
    }
}

fn collect<'a>(fragments: &'a [CombinedFragment], scenario: &str, into: &mut Vec<Frame<'a>>) {
    for f in fragments {
        let id = derived(&format!("{scenario}:fragment:{}:{}:{}", f.start, f.end, into.len()));
        into.push(Frame { start: f.start, end: f.end, what: What::Fragment(f, id) });
        for operand in &f.operands {
            collect(&operand.fragments, scenario, into);
        }
    }
}

/// The scenario element, or `None` (with a note) when no lifeline resolves.
pub(super) fn scenario(context: &mut Context, engineering: &El, s: &Scenario, name: &str) -> Option<El> {
    let id = if s.id.is_empty() { derived(&format!("scenario:{}", s.name)) } else { s.id.clone() };
    let mut el = El::new("ownedScenarios", "interaction:Scenario", &id).attr("name", name);
    if let Some(kind) = s.attributes.get("kind").and_then(|v| v.as_string()) {
        el.set("kind", kind);
    }
    // One lifeline per participant: the instance role of its part, or of the element.
    let mut role_of: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut roles: Vec<El> = Vec::new();
    for p in &s.participants {
        let Some(target) = context.resolve(&p.id).or_else(|| context.resolve(&p.name)).map(str::to_string) else { continue };
        let part = derived(&format!("{target}:part"));
        let represented = if engineering.find(&part).is_some() { part } else { target.clone() };
        let role = derived(&format!("{id}:role:{target}"));
        role_of.insert(p.id.clone(), role.clone());
        role_of.insert(p.name.clone(), role.clone());
        role_of.insert(target.clone(), role.clone());
        // Capella names a lifeline after the instance it represents.
        let label = engineering.find(&represented).and_then(|e| e.get("name")).unwrap_or(&p.name).to_string();
        roles.push(El::new("ownedInstanceRoles", "interaction:InstanceRole", &role).attr("name", &label).attr("representedInstance", &format!("#{represented}")));
    }
    if roles.is_empty() {
        context.note(format!("scenario '{}' left out: none of its participants is exported", s.name));
        return None;
    }
    let all_roles = roles.iter().map(|r| format!("#{}", r.id())).collect::<Vec<_>>().join(" ");
    let resolved: std::collections::HashMap<String, String> = s
        .participants
        .iter()
        .flat_map(|p| [p.id.clone(), p.name.clone()])
        .chain(s.messages.iter().flat_map(|m| [m.from.clone(), m.to.clone()]))
        .chain(s.invariants.iter().map(|i| i.lifeline.clone()))
        .filter_map(|r| context.resolve(&r).map(|t| (r.clone(), t.to_string())))
        .collect();
    let role = |reference: &str| role_of.get(reference).cloned().or_else(|| resolved.get(reference).and_then(|t| role_of.get(t).cloned()));

    let mut frames: Vec<Frame<'_>> = Vec::new();
    collect(&s.fragments, &id, &mut frames);
    for (index, inv) in s.invariants.iter().enumerate() {
        let (Some(lifeline), Some(related)) = (role(&inv.lifeline), context.resolve(&inv.name).map(str::to_string)) else { continue };
        let iid = if inv.id.is_empty() { derived(&format!("{id}:invariant:{index}")) } else { inv.id.clone() };
        frames.push(Frame { start: inv.start, end: inv.end, what: What::Invariant { id: iid, lifeline, related, state: inv.kind == InvariantKind::State } });
    }
    // Outer frames open first and close last.
    let span = |f: &Frame<'_>| if f.end < f.start { 0 } else { f.end - f.start + 1 };

    let mut fragments: Vec<El> = Vec::new();
    let mut messages: Vec<El> = Vec::new();
    let mut lapses: Vec<El> = Vec::new();
    let mut starts: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    let open = |f: &Frame<'_>, i: usize, fragments: &mut Vec<El>, starts: &mut std::collections::HashMap<usize, String>| {
        let fid = match &f.what {
            What::Fragment(_, fid) => fid.clone(),
            What::Invariant { id, .. } => id.clone(),
        };
        let start = derived(&format!("{fid}:start"));
        match &f.what {
            What::Fragment(..) => fragments.push(El::new("ownedInteractionFragments", "interaction:FragmentEnd", &start).attr("coveredInstanceRoles", &all_roles)),
            What::Invariant { lifeline, related, state, .. } => {
                let key = if *state { "relatedAbstractState" } else { "relatedAbstractFunction" };
                fragments.push(El::new("ownedInteractionFragments", "interaction:InteractionState", &start).attr("coveredInstanceRoles", &format!("#{lifeline}")).attr(key, &format!("#{related}")));
            }
        }
        starts.insert(i, start);
    };
    let count = s.messages.len();
    let mut dropped: Vec<String> = Vec::new();
    let mut events: Vec<El> = Vec::new();
    let mut order: Vec<usize> = (0..frames.len()).collect();
    order.sort_by_key(|i| (frames[*i].start, std::cmp::Reverse(span(&frames[*i]))));
    let mut closing: Vec<(usize, usize)> = Vec::new(); // (message index after which it closes, frame)
    let mut opened = vec![false; frames.len()];
    for position in 0..=count {
        let now: Vec<usize> = order.iter().copied().filter(|f| frames[*f].start == position && !opened[*f]).collect();
        for f in now {
            opened[f] = true;
            open(&frames[f], f, &mut fragments, &mut starts);
            if let What::Fragment(cf, fid) = &frames[f].what {
                for (n, operand) in cf.operands.iter().enumerate() {
                    if operand.start == position || (n == 0 && operand.end < operand.start) {
                        fragments.push(El::new("ownedInteractionFragments", "interaction:InteractionOperand", &derived(&format!("{fid}:operand:{n}"))).attr("coveredInstanceRoles", &all_roles));
                    }
                }
            }
            if span(&frames[f]) == 0 {
                close(&frames[f], f, &mut fragments, &starts, &mut lapses, &all_roles);
            } else {
                closing.push((frames[f].end, f));
            }
        }
        // Operands that start inside an open fragment, at this message.
        for f in 0..frames.len() {
            if let What::Fragment(cf, fid) = &frames[f].what {
                if frames[f].start < position {
                    for (n, operand) in cf.operands.iter().enumerate().skip(1) {
                        if operand.start == position && operand.end >= operand.start {
                            fragments.push(El::new("ownedInteractionFragments", "interaction:InteractionOperand", &derived(&format!("{fid}:operand:{n}"))).attr("coveredInstanceRoles", &all_roles));
                        }
                    }
                }
            }
        }
        if position == count {
            break;
        }
        let m = &s.messages[position];
        let (Some(from), Some(to)) = (role(&m.from), role(&m.to)) else {
            dropped.push(m.label.clone());
            continue;
        };
        let mid = if m.id.is_empty() { derived(&format!("{id}:message:{position}")) } else { m.id.clone() };
        let (send, receive) = (derived(&format!("{mid}:send")), derived(&format!("{mid}:receive")));
        let (sent, received) = (derived(&format!("{mid}:sent")), derived(&format!("{mid}:received")));
        fragments.push(El::new("ownedInteractionFragments", "interaction:MessageEnd", &send).attr("coveredInstanceRoles", &format!("#{from}")).attr("event", &format!("#{sent}")));
        fragments.push(El::new("ownedInteractionFragments", "interaction:MessageEnd", &receive).attr("coveredInstanceRoles", &format!("#{to}")).attr("event", &format!("#{received}")));
        events.push(El::new("ownedEvents", "interaction:EventSentOperation", &sent));
        events.push(El::new("ownedEvents", "interaction:EventReceiptOperation", &received));
        let kind = match m.message_type {
            MessageType::Synchronous => "SYNCHRONOUS_CALL",
            MessageType::Asynchronous => "ASYNCHRONOUS_CALL",
            MessageType::Return => "REPLY",
        };
        messages.push(El::new("ownedMessages", "interaction:SequenceMessage", &mid).attr("name", &m.label).attr("kind", kind).attr("sendingEnd", &format!("#{send}")).attr("receivingEnd", &format!("#{receive}")));
        // Frames ending at this message close, innermost first.
        let mut ending: Vec<usize> = closing.iter().filter(|(end, _)| *end == position).map(|(_, f)| *f).collect();
        ending.sort_by_key(|f| span(&frames[*f]));
        for f in ending {
            close(&frames[f], f, &mut fragments, &starts, &mut lapses, &all_roles);
        }
    }
    for label in dropped {
        context.note(format!("message '{label}' of scenario '{}' left out: a lifeline is not exported", s.name));
    }
    for role in roles {
        el.push(role);
    }
    for message in messages {
        el.push(message);
    }
    for fragment in fragments {
        el.push(fragment);
    }
    for lapse in lapses {
        el.push(lapse);
    }
    for event in events {
        el.push(event);
    }
    Some(el)
}

fn close(f: &Frame<'_>, index: usize, fragments: &mut Vec<El>, starts: &std::collections::HashMap<usize, String>, lapses: &mut Vec<El>, all_roles: &str) {
    let Some(start) = starts.get(&index) else { return };
    match &f.what {
        What::Fragment(cf, fid) => {
            let finish = derived(&format!("{fid}:finish"));
            fragments.push(El::new("ownedInteractionFragments", "interaction:FragmentEnd", &finish).attr("coveredInstanceRoles", all_roles));
            let operands = (0..cf.operands.len().max(1)).map(|n| format!("#{}", derived(&format!("{fid}:operand:{n}")))).collect::<Vec<_>>().join(" ");
            let mut lapse = El::new("ownedTimeLapses", "interaction:CombinedFragment", fid).attr("start", &format!("#{start}")).attr("finish", &format!("#{finish}")).attr("operator", operator(cf.fragment_type));
            if !cf.label.is_empty() {
                lapse.set("name", &cf.label);
            }
            lapse.set("referencedOperands", &operands);
            lapses.push(lapse);
        }
        What::Invariant { id, lifeline, related, state } => {
            let finish = derived(&format!("{id}:finish"));
            let key = if *state { "relatedAbstractState" } else { "relatedAbstractFunction" };
            fragments.push(El::new("ownedInteractionFragments", "interaction:InteractionState", &finish).attr("coveredInstanceRoles", &format!("#{lifeline}")).attr(key, &format!("#{related}")));
            lapses.push(El::new("ownedTimeLapses", "interaction:StateFragment", id).attr("start", &format!("#{start}")).attr("finish", &format!("#{finish}")).attr(key, &format!("#{related}")));
        }
    }
}

/// The capability that owns a scenario: the one its name starts with
/// (`Owner / Scenario`), else the first capability of the level of its
/// first lifeline.
pub(super) fn owner<'a>(context: &Context, engineering: &El, s: &'a Scenario) -> (Option<String>, &'a str) {
    if let Some((owner, name)) = s.name.split_once(" / ") {
        if let Some(id) = context.resolve(owner) {
            return (Some(id.to_string()), name);
        }
    }
    let first = s.participants.iter().find_map(|p| context.resolve(&p.id).or_else(|| context.resolve(&p.name)));
    let level = first.and_then(|id| engineering.children().iter().find(|level| level.find(id).is_some()));
    let capability = level.and_then(|level| first_capability(level)).map(str::to_string);
    (capability, s.name.as_str())
}

fn first_capability(el: &El) -> Option<&str> {
    let kind = el.kind();
    if kind.ends_with(":Capability") || kind.ends_with(":OperationalCapability") || kind.ends_with(":CapabilityRealization") {
        return Some(el.id());
    }
    el.children().iter().find_map(first_capability)
}
