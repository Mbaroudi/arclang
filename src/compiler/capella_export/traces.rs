//! Traces: a `realizes` trace is the realization link Capella keeps for
//! the kind of its source (a function realizes a function, an exchange an
//! exchange, a port a port...), owned by the source; an `allocates` trace
//! is the allocation the source owns (a component exchange allocates
//! functional exchanges, a component port function ports, a physical port
//! component ports, a physical link component exchanges).

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::Trace;

/// The feature and kind of the realization a source of this kind owns.
fn realization(kind: &str) -> Option<(&'static str, &'static str)> {
    let local = kind.rsplit(':').next().unwrap_or(kind);
    Some(match local {
        "SystemFunction" | "LogicalFunction" | "PhysicalFunction" | "OperationalActivity" => ("ownedFunctionRealizations", "fa:FunctionRealization"),
        "FunctionalExchange" => ("ownedFunctionalExchangeRealizations", "fa:FunctionalExchangeRealization"),
        "ComponentExchange" | "CommunicationMean" => ("ownedComponentExchangeRealizations", "fa:ComponentExchangeRealization"),
        "SystemComponent" | "LogicalComponent" | "PhysicalComponent" | "Entity" => ("ownedComponentRealizations", "cs:ComponentRealization"),
        "FunctionInputPort" | "FunctionOutputPort" | "ComponentPort" | "PhysicalPort" => ("ownedPortRealizations", "information:PortRealization"),
        "FunctionalChain" | "OperationalProcess" => ("ownedFunctionalChainRealizations", "fa:FunctionalChainRealization"),
        "State" | "Mode" | "FinalState" | "ChoicePseudoState" | "ForkPseudoState" | "JoinPseudoState" | "TerminatePseudoState" | "DeepHistoryPseudoState" | "ShallowHistoryPseudoState"
        | "EntryPointPseudoState" | "ExitPointPseudoState" | "InitialPseudoState" => ("ownedAbstractStateRealizations", "capellacommon:AbstractStateRealization"),
        "StateTransition" => ("ownedStateTransitionRealizations", "capellacommon:StateTransitionRealization"),
        "Class" | "ExchangeItem" | "NumericType" | "StringType" | "BooleanType" | "Enumeration" | "PhysicalQuantity" => ("ownedInformationRealizations", "information:InformationRealization"),
        "Scenario" => ("ownedScenarioRealization", "interaction:ScenarioRealization"),
        "Capability" | "CapabilityRealization" | "OperationalCapability" | "Threat" => ("ownedAbstractCapabilityRealizations", "interaction:AbstractCapabilityRealization"),
        "ConfigurationItem" => ("ownedPhysicalArtifactRealizations", "epbs:PhysicalArtifactRealization"),
        _ => return None,
    })
}

/// The feature and kind of the allocation a source of this kind owns.
fn allocation(kind: &str) -> Option<(&'static str, &'static str)> {
    Some(match kind.rsplit(':').next().unwrap_or(kind) {
        // A communication mean allocates the interactions it carries.
        "ComponentExchange" | "CommunicationMean" => ("ownedComponentExchangeFunctionalExchangeAllocations", "fa:ComponentExchangeFunctionalExchangeAllocation"),
        "ComponentPort" => ("ownedPortAllocations", "information:PortAllocation"),
        "PhysicalPort" => ("ownedComponentPortAllocations", "fa:ComponentPortAllocation"),
        "PhysicalLink" | "PhysicalPath" => ("ownedComponentExchangeAllocations", "fa:ComponentExchangeAllocation"),
        _ => return None,
    })
}

/// Write the traces of the model into `engineering`; the others are counted
/// in the notes. Requirement traces are written with the requirements.
pub(super) fn write(context: &mut Context, engineering: &mut El, traces: &[Trace]) {
    let mut left_out = 0;
    for trace in traces {
        let kind = if trace.trace_type == "allocates" { "allocates" } else { trace.trace_type.as_str() };
        if kind == "satisfies" || kind == "refines" {
            continue;
        }
        let (Some(source), Some(target)) = (context.resolve(&trace.from).map(str::to_string), context.resolve(&trace.to).map(str::to_string)) else {
            left_out += 1;
            continue;
        };
        let Some(owner) = engineering.find_mut(&source) else {
            left_out += 1;
            continue;
        };
        let feature = match kind {
            "realizes" => realization(owner.kind()),
            "allocates" => allocation(owner.kind()),
            _ => None,
        };
        let Some((tag, link)) = feature else {
            left_out += 1;
            continue;
        };
        // A model may realize the same element twice (Capella flags it): it is kept,
        // with an id of its own.
        let target_ref = format!("#{target}");
        let repeats = owner.children().iter().filter(|c| c.get("targetElement") == Some(target_ref.as_str()) && c.kind().ends_with(link.rsplit(':').next().unwrap_or(link))).count();
        owner.push(
            El::new(tag, link, &derived(&format!("{source}->{target}:{kind}{}", if repeats > 0 { format!(":{repeats}") } else { String::new() })))
                .attr("targetElement", &format!("#{target}"))
                .attr("sourceElement", &format!("#{source}")),
        );
    }
    if left_out > 0 {
        context.note(format!("{left_out} traces left out: an end is not exported, or Capella has no link of that kind between them"));
    }
}
