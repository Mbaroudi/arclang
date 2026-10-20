//! `icd`: the interface control document, one row per exchange: its ends
//! and ports, the items it carries, its protocol, bandwidth and latency.
//! An exchange of the logical level takes the protocol of its ports when
//! it states none; one of the physical level, the link it goes over.

use super::context::{layer_named, shown, Context};
use super::{label_of, Cell, CellEdit, Column, Link, Row, TableSpec};
use crate::compiler::ast::AttributeValue;
use crate::compiler::identity::element_uuid;
use crate::compiler::metamodel::Layer;
use std::collections::{BTreeMap, HashMap};

pub const DEFAULT_COLUMNS: [&str; 9] = ["name", "from", "from_port", "to", "to_port", "items", "protocol", "bandwidth", "latency"];
const ALL_COLUMNS: [(&str, &str, &str); 13] = [
    ("id", "Id", "text"),
    ("name", "Exchange", "text"),
    ("kind", "Kind", "text"),
    ("from", "From", "links"),
    ("from_port", "From port", "links"),
    ("to", "To", "links"),
    ("to_port", "To port", "links"),
    ("items", "Items", "links"),
    ("protocol", "Protocol", "text"),
    ("bandwidth", "Bandwidth", "number"),
    ("latency", "Latency", "number"),
    ("interface", "Interface", "text"),
    ("via", "Via", "links"),
];

/// One exchange, whatever its level, before it becomes a row.
struct Exchange<'m> {
    kind: &'static str,
    id: String,
    name: String,
    from: String,
    to: String,
    via: Option<String>,
    item: Option<String>,
    attributes: &'m HashMap<String, AttributeValue>,
    /// The link it goes over, for a physical exchange.
    link: Option<&'m crate::compiler::ast::PhysicalLink>,
}

fn exchanges<'m>(context: &Context<'m>, layer: Layer) -> Vec<Exchange<'m>> {
    let ast = &context.result.ast;
    let text = |attributes: &HashMap<String, AttributeValue>, key: &str| attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
    let mut out = Vec::new();
    match layer {
        Layer::Operational => {
            for oa in &ast.operational_analysis {
                for e in oa.exchanges.iter().chain(oa.communication_means.iter()) {
                    let name = e.label.clone().unwrap_or_else(|| format!("{} -> {}", e.from, e.to));
                    out.push(Exchange { kind: "OperationalExchange", id: text(&e.attributes, "id").unwrap_or_else(|| name.clone()), name, from: e.from.clone(), to: e.to.clone(), via: None, item: text(&e.attributes, "exchange_item"), attributes: &e.attributes, link: None });
                }
            }
        }
        Layer::System => {
            for sa in &ast.system_analysis {
                for e in &sa.functional_exchanges {
                    out.push(Exchange { kind: "FunctionalExchange", id: e.id.clone(), name: e.label.clone().unwrap_or_else(|| e.id.clone()), from: e.from_port.clone(), to: e.to_port.clone(), via: None, item: text(&e.attributes, "exchange_item"), attributes: &e.attributes, link: None });
                }
                for e in &sa.component_exchanges {
                    out.push(Exchange { kind: "ComponentExchange", id: e.id.clone(), name: e.label.clone().unwrap_or_else(|| e.id.clone()), from: e.from_port.clone(), to: e.to_port.clone(), via: None, item: text(&e.attributes, "exchange_item"), attributes: &e.attributes, link: None });
                }
            }
        }
        Layer::Physical => {
            for pa in &ast.physical_architecture {
                for e in &pa.physical_exchanges {
                    let link = e.via.as_ref().and_then(|v| pa.links.iter().find(|l| &l.name == v || l.attributes.get("id").and_then(|i| i.as_string()) == Some(v.as_str())));
                    out.push(Exchange {
                        kind: "PhysicalExchange",
                        id: e.id.clone(),
                        name: e.label.clone().unwrap_or_else(|| e.id.clone()),
                        from: e.from.clone(),
                        to: e.to.clone(),
                        via: e.via.clone(),
                        item: text(&e.attributes, "exchange_item").or_else(|| Some(e.message_type.clone()).filter(|m| !m.is_empty())),
                        attributes: &e.attributes,
                        link,
                    });
                }
            }
        }
        _ => {
            for la in &ast.logical_architecture {
                for e in &la.component_exchanges {
                    out.push(Exchange { kind: "ComponentExchange", id: e.id.clone(), name: e.label.clone().unwrap_or_else(|| e.id.clone()), from: e.from_port.clone(), to: e.to_port.clone(), via: None, item: text(&e.attributes, "exchange_item"), attributes: &e.attributes, link: None });
                }
            }
        }
    }
    out
}

/// The level an ICD is of: its scope's, else the logical level when it
/// has exchanges, else the first level that has some.
fn level(context: &Context, spec: &TableSpec) -> Layer {
    if let Some(layer) = spec.scope.as_deref().and_then(layer_named) {
        return layer;
    }
    [Layer::Logical, Layer::Physical, Layer::System, Layer::Operational].into_iter().find(|&l| !exchanges(context, l).is_empty()).unwrap_or(Layer::Logical)
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    let layer = level(context, spec);
    let available: Vec<Column> = ALL_COLUMNS.iter().map(|(k, l, t)| Column { key: k.to_string(), label: l.to_string(), ty: t.to_string() }).collect();
    let keys: Vec<String> = if spec.columns.is_empty() { DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect() } else { spec.columns.clone() };
    let columns = keys.iter().map(|k| available.iter().find(|c| &c.key == k).cloned().unwrap_or_else(|| Column { key: k.clone(), label: label_of(k), ty: "text".to_string() })).collect();
    let rows = exchanges(context, layer).iter().map(|e| row(context, e, &keys)).collect();
    Ok((columns, available, rows))
}

/// The element owning a port, or the element itself: the end as a block.
fn end_of(context: &Context, port: Option<usize>) -> Option<usize> {
    let port = port?;
    if context.element(port).kind.ends_with("Port") {
        return context.owners(port).into_iter().find(|&o| !context.element(o).kind.ends_with("Port"));
    }
    Some(port)
}

fn row(context: &Context, exchange: &Exchange, keys: &[String]) -> Row {
    let from_port = context.resolve(&exchange.from);
    let to_port = context.resolve(&exchange.to);
    let (from, to) = (end_of(context, from_port), end_of(context, to_port));
    let port_of = |index: Option<usize>| index.filter(|&i| context.element(i).kind.ends_with("Port"));
    let attribute = |index: Option<usize>, key: &str| index.and_then(|i| context.element(i).attributes.get(key)).map(shown);
    let own = |key: &str| exchange.attributes.get(key).map(shown);
    let link_attribute = |key: &str| exchange.link.and_then(|l| match key {
        "protocol" => Some(l.protocol.clone()).filter(|p| !p.is_empty()),
        "bandwidth" => l.attributes.get("bandwidth").map(shown).or_else(|| l.bandwidth.clone()),
        other => l.attributes.get(other).map(shown),
    });
    let set = |element: &str, key: &str, value_type: &str| Some(CellEdit { op: "set".to_string(), element: element.to_string(), key: Some(key.to_string()), value_type: Some(value_type.to_string()), values: Vec::new() });
    // An attribute is edited where it is written: the exchange, else the port that states it, else the exchange.
    let sourced = |key: &str, value_type: &str| -> Cell {
        if let Some(value) = own(key) {
            return Cell::text(value).editable(set(&exchange.id, key, value_type));
        }
        for port in [port_of(from_port), port_of(to_port)].into_iter().flatten() {
            if let Some(value) = attribute(Some(port), key) {
                return Cell::text(value).editable(set(&context.element(port).id, key, value_type));
            }
        }
        if let (Some(value), Some(link)) = (link_attribute(key), exchange.link) {
            let id = link.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&link.name).to_string();
            return Cell::text(value).editable(set(&id, key, value_type));
        }
        Cell::text("").editable(set(&exchange.id, key, value_type))
    };
    let item_links: Vec<Link> = exchange.item.iter().filter_map(|i| context.resolve(i)).map(|i| context.link(i)).collect();
    let mut cells = BTreeMap::new();
    for key in keys {
        let link_or_text = |index: Option<usize>, written: &str| index.map(|i| Cell::links(vec![context.link(i)])).unwrap_or_else(|| Cell::text(written));
        let cell = match key.as_str() {
            "id" => Cell::text(exchange.id.clone()),
            "name" => Cell::text(exchange.name.clone()),
            "kind" => Cell::text(exchange.kind),
            "from" => link_or_text(from, &exchange.from),
            "to" => link_or_text(to, &exchange.to),
            "from_port" => port_of(from_port).map(|p| Cell::links(vec![context.link(p)])).unwrap_or_default(),
            "to_port" => port_of(to_port).map(|p| Cell::links(vec![context.link(p)])).unwrap_or_default(),
            "items" => match (&exchange.item, item_links.is_empty()) {
                (Some(_), false) => Cell::links(item_links.clone()),
                (Some(written), true) => Cell::text(written.clone()),
                (None, _) => Cell::text("").with_status("gap"),
            },
            "protocol" => sourced("protocol", "Text"),
            "bandwidth" => sourced("bandwidth", "Quantity"),
            "latency" => sourced("latency", "Quantity"),
            "interface" => sourced("interface", "Text"),
            "via" => exchange.via.as_ref().map(|v| link_or_text(context.resolve(v), v)).unwrap_or_default(),
            other => Cell::text(own(other).unwrap_or_default()).editable(set(&exchange.id, other, "Text")),
        };
        cells.insert(key.clone(), cell);
    }
    let mut issues = Vec::new();
    if from.is_none() || to.is_none() {
        issues.push("an end does not resolve".to_string());
    }
    if exchange.item.is_none() {
        issues.push("carries no exchange item".to_string());
    }
    let protocol = cells.get("protocol").map(|c| !c.text.is_empty()).unwrap_or_else(|| own("protocol").is_some());
    if !protocol && keys.iter().any(|k| k == "protocol") {
        issues.push("states no protocol".to_string());
    }
    let status = if from.is_none() || to.is_none() || exchange.item.is_none() { "gap" } else if issues.is_empty() { "ok" } else { "warn" };
    if let Some(cell) = cells.get_mut("protocol").filter(|c| c.text.is_empty()) {
        cell.status = Some("warn".to_string());
    }
    let uuid = relationship_uuid(context, exchange, from_port, to_port).unwrap_or_else(|| element_uuid(&format!("icd:{}", exchange.kind), &exchange.id));
    Row { id: exchange.id.clone(), uuid, kind: exchange.kind.to_string(), name: exchange.name.clone(), status: status.to_string(), issues, cells }
}

/// The UUID the element graph gives the exchange: same kind, same ends.
fn relationship_uuid(context: &Context, exchange: &Exchange, from: Option<usize>, to: Option<usize>) -> Option<String> {
    let (from, to) = (context.element(from?).uuid.clone(), context.element(to?).uuid.clone());
    let candidates: Vec<_> = context.relationships(exchange.kind).filter(|r| r.source == from && r.target == to).collect();
    candidates.iter().find(|r| r.name.as_deref() == Some(exchange.name.as_str())).or(candidates.first()).map(|r| r.uuid.clone())
}
