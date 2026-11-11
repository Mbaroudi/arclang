//! The physical architecture a device tree describes.
//!
//! A device tree is what the kernel of a board reads to know its hardware:
//! the processors, the memory, each device with what it is compatible with,
//! where it is mapped, the bus it sits on, the pins that wire it. It is
//! read here as flattened source (`dtc -I fs -O dts /proc/device-tree` on
//! the board, or `dtc -I dtb -O dts board.dtb`) and written as ArcLang: the
//! board, a node for each enabled device, a link for each bus a device sits
//! on and for each pin that wires two devices.
//!
//! Everything written is read in the tree, and nothing is guessed. What a
//! device tree does not hold is not recovered and is said in the notes: the
//! schematic, what is plugged on a connector the tree does not describe,
//! what the software does with the devices.

use crate::compiler::source_edit::string_literal;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Number(u64),
    /// `&label`
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Text(String),
    Cells(Vec<Cell>),
    Bytes,
}

#[derive(Debug, Clone, Default)]
struct Node {
    name: String,
    labels: Vec<String>,
    properties: Vec<(String, Vec<Value>)>,
    children: Vec<Node>,
}

impl Node {
    fn property(&self, name: &str) -> Option<&Vec<Value>> {
        self.properties.iter().rev().find(|(key, _)| key == name).map(|(_, values)| values)
    }
    fn texts(&self, name: &str) -> Vec<&str> {
        self.property(name).map(|values| values.iter().filter_map(|v| if let Value::Text(t) = v { Some(t.as_str()) } else { None }).collect()).unwrap_or_default()
    }
    fn cells(&self, name: &str) -> Vec<&Cell> {
        self.property(name).map(|values| values.iter().filter_map(|v| if let Value::Cells(c) = v { Some(c) } else { None }).flatten().collect()).unwrap_or_default()
    }
    fn number(&self, name: &str) -> Option<u64> {
        self.cells(name).first().and_then(|cell| if let Cell::Number(n) = cell { Some(*n) } else { None })
    }
    /// A node is enabled unless it says otherwise.
    fn enabled(&self) -> bool {
        self.texts("status").first().is_none_or(|status| matches!(*status, "okay" | "ok"))
    }
}

struct Reader<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn line(&self) -> usize {
        self.text[..self.at.min(self.text.len())].iter().filter(|b| **b == b'\n').count() + 1
    }
    fn fail<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("line {}: {what}", self.line()))
    }
    fn skip(&mut self) {
        loop {
            while self.at < self.text.len() && self.text[self.at].is_ascii_whitespace() {
                self.at += 1;
            }
            if self.text[self.at..].starts_with(b"/*") {
                self.at = self.text[self.at..].windows(2).position(|w| w == b"*/").map_or(self.text.len(), |end| self.at + end + 2);
            } else if self.text[self.at..].starts_with(b"//") {
                self.at = self.text[self.at..].iter().position(|b| *b == b'\n').map_or(self.text.len(), |end| self.at + end);
            } else {
                return;
            }
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.skip();
        self.text.get(self.at).copied()
    }
    fn eat(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.eat(byte) {
            Ok(())
        } else {
            self.fail(&format!("`{}` expected", byte as char))
        }
    }
    /// A node or a property name: what the specification allows in either.
    fn word(&mut self) -> String {
        self.skip();
        let start = self.at;
        while self.at < self.text.len() && (self.text[self.at].is_ascii_alphanumeric() || b",._+-@#?/".contains(&self.text[self.at])) {
            self.at += 1;
        }
        String::from_utf8_lossy(&self.text[start..self.at]).into_owned()
    }
    fn text(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = Vec::new();
        while self.at < self.text.len() && self.text[self.at] != b'"' {
            if self.text[self.at] == b'\\' && self.at + 1 < self.text.len() {
                self.at += 1;
                // `\0` ends one text of a list and starts the next: a decompiled
                // tree writes `"a\0b"` where the source wrote `"a", "b"`.
                if self.text[self.at] == b'0' {
                    out.push(0);
                    self.at += 1;
                    continue;
                }
            }
            out.push(self.text[self.at]);
            self.at += 1;
        }
        if self.at >= self.text.len() {
            return self.fail("a text is not closed");
        }
        self.at += 1;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }
    fn cells(&mut self) -> Result<Vec<Cell>, String> {
        self.expect(b'<')?;
        let mut cells = Vec::new();
        loop {
            match self.peek() {
                Some(b'>') => {
                    self.at += 1;
                    return Ok(cells);
                }
                Some(b'&') => {
                    self.at += 1;
                    // `&label`, or `&{/path}`.
                    if self.eat(b'{') {
                        let path = self.word();
                        self.expect(b'}')?;
                        cells.push(Cell::Reference(path));
                    } else {
                        cells.push(Cell::Reference(self.word()));
                    }
                }
                Some(b'(') => return self.fail("an expression in a cell: flatten the tree first (dtc -I dts -O dts)"),
                Some(byte) if byte.is_ascii_digit() => {
                    let word = self.word();
                    let number = match word.strip_prefix("0x").or_else(|| word.strip_prefix("0X")) {
                        Some(hex) => u64::from_str_radix(hex, 16),
                        None => word.parse::<u64>(),
                    };
                    match number {
                        Ok(number) => cells.push(Cell::Number(number)),
                        Err(_) => return self.fail(&format!("`{word}` is not a number")),
                    }
                }
                Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => return self.fail("a name in a cell: the tree uses macros, flatten it first (cpp, then dtc -I dts -O dts)"),
                _ => return self.fail("a cell list is not closed"),
            }
        }
    }
    fn values(&mut self) -> Result<Vec<Value>, String> {
        let mut values = Vec::new();
        loop {
            match self.peek() {
                Some(b'"') => values.extend(self.text()?.split('\0').filter(|part| !part.is_empty()).map(|part| Value::Text(part.to_string()))),
                Some(b'<') => values.push(Value::Cells(self.cells()?)),
                Some(b'[') => {
                    self.at = self.text[self.at..].iter().position(|b| *b == b']').map_or(self.text.len(), |end| self.at + end + 1);
                    values.push(Value::Bytes);
                }
                Some(b'&') => {
                    self.at += 1;
                    values.push(Value::Cells(vec![Cell::Reference(self.word())]));
                }
                Some(b'/') if self.text[self.at..].starts_with(b"/bits/") => {
                    self.at += 6;
                    self.word();
                    continue;
                }
                _ => return self.fail("a value expected (a text, <cells> or [bytes])"),
            }
            if !self.eat(b',') {
                return Ok(values);
            }
        }
    }
    /// The body of a node, after its `{`.
    fn body(&mut self, node: &mut Node) -> Result<(), String> {
        loop {
            if self.eat(b'}') {
                self.expect(b';')?;
                return Ok(());
            }
            if self.peek().is_none() {
                return self.fail("a node is not closed");
            }
            let mut labels = Vec::new();
            let mut name = self.word();
            if name.starts_with("/delete-") {
                return self.fail("a deletion: flatten the tree first (dtc -I dts -O dts)");
            }
            // `label: name`, as many labels as written.
            while self.eat(b':') {
                labels.push(name);
                name = self.word();
            }
            if name.is_empty() {
                return self.fail("a name expected");
            }
            if self.eat(b'{') {
                let mut child = Node { name: name.clone(), labels, ..Node::default() };
                self.body(&mut child)?;
                // A node written twice is one node: what comes later adds to it.
                match node.children.iter_mut().find(|known| known.name == name) {
                    Some(known) => {
                        known.labels.extend(child.labels);
                        known.properties.extend(child.properties);
                        known.children.extend(child.children);
                    }
                    None => node.children.push(child),
                }
            } else if self.eat(b'=') {
                let values = self.values()?;
                self.expect(b';')?;
                node.properties.push((name, values));
            } else {
                self.expect(b';')?;
                node.properties.push((name, Vec::new()));
            }
        }
    }
}

/// Read a flattened device tree source.
fn read(text: &str) -> Result<Node, String> {
    if let Some(line) = text.lines().position(|line| line.trim_start().starts_with("#include") || line.trim_start().starts_with("/include/")) {
        return Err(format!("line {}: the tree includes other files: give the flattened tree (on the board: dtc -I fs -O dts /proc/device-tree; from a blob: dtc -I dtb -O dts board.dtb)", line + 1));
    }
    let mut reader = Reader { text: text.as_bytes(), at: 0 };
    let mut root = Node { name: "/".to_string(), ..Node::default() };
    let mut seen = false;
    loop {
        match reader.peek() {
            None => break,
            Some(b'/') if reader.text[reader.at..].starts_with(b"/dts-v1/") || reader.text[reader.at..].starts_with(b"/plugin/") || reader.text[reader.at..].starts_with(b"/memreserve/") => {
                reader.at = reader.text[reader.at..].iter().position(|b| *b == b';').map_or(reader.text.len(), |end| reader.at + end + 1);
            }
            Some(b'/') => {
                reader.at += 1;
                reader.expect(b'{')?;
                reader.body(&mut root)?;
                seen = true;
            }
            Some(b'&') => return reader.fail("the tree changes a node by its label (an overlay): give the flattened tree (dtc -I fs -O dts /proc/device-tree on the board)"),
            _ => return reader.fail("a device tree starts with `/ {`"),
        }
    }
    if !seen {
        return Err("this is not a device tree source: no `/ { ... };`".to_string());
    }
    Ok(root)
}

/// A device read in the tree.
struct Device<'a> {
    node: &'a Node,
    path: String,
    /// The device it is a child of, when one of its ancestors is a device.
    parent: Option<usize>,
    /// The node it sits in, whether or not that node is a device.
    bus: &'a Node,
    id: String,
    name: String,
}

/// What a bus is, by the name of its node (the generic names of the specification).
fn bus_of(node: &Node) -> Option<&'static str> {
    // `i2c@...`, and `i2c0@...` as some trees number their buses.
    let name = node.name.split('@').next().unwrap_or_default().trim_end_matches(|c: char| c.is_ascii_digit());
    let known: [(&[&str], &str); 10] = [
        (&["i2c", "i2c-mux", "i2c-gpio"], "I2C"),
        (&["spi", "qspi"], "SPI"),
        (&["serial", "uart"], "UART"),
        (&["usb", "usb-hub", "hub"], "USB"),
        (&["mmc", "mmcnr", "sdhci", "sdhost", "sdio"], "SD/MMC"),
        (&["mdio"], "MDIO"),
        (&["ethernet"], "Ethernet"),
        (&["pci", "pcie"], "PCI"),
        (&["can"], "CAN"),
        (&["onewire", "w1"], "1-Wire"),
    ];
    known.iter().find(|(names, _)| names.contains(&name)).map(|(_, bus)| *bus)
}

/// The size in bytes of the memory a tree declares.
fn memory(root: &Node) -> u64 {
    let (address_cells, size_cells) = (root.number("#address-cells").unwrap_or(2) as usize, root.number("#size-cells").unwrap_or(1) as usize);
    let mut bytes = 0u64;
    for node in root.children.iter().filter(|node| node.name == "memory" || node.name.starts_with("memory@") || node.texts("device_type").first() == Some(&"memory")) {
        let cells: Vec<u64> = node.cells("reg").iter().filter_map(|cell| if let Cell::Number(n) = cell { Some(*n) } else { None }).collect();
        for range in cells.chunks(address_cells + size_cells).filter(|range| range.len() == address_cells + size_cells) {
            bytes = bytes.saturating_add(range[address_cells..].iter().fold(0u64, |size, cell| (size << 32) | cell));
        }
    }
    bytes
}

/// A size in the largest binary unit that writes it whole.
fn size(bytes: u64) -> String {
    for (unit, scale) in [("GiB", 1u64 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)] {
        if bytes >= scale && bytes % scale == 0 {
            return format!("{} {unit}", bytes / scale);
        }
    }
    format!("{bytes} B")
}

/// What was recovered, and what was not.
pub struct Recovered {
    pub source: String,
    pub board: String,
    pub devices: usize,
    pub links: usize,
    pub notes: Vec<String>,
}

/// The physical architecture of a flattened device tree, as ArcLang.
pub fn to_arclang(text: &str, origin: &str) -> Result<Recovered, String> {
    let root = read(text)?;
    let board = root.texts("model").first().map(|m| m.to_string()).or_else(|| root.texts("compatible").first().map(|c| c.to_string())).unwrap_or_else(|| "Board".to_string());

    // The enabled nodes that say what they are compatible with, in the order of the tree.
    let mut devices: Vec<Device> = Vec::new();
    let (mut disabled, mut unnamed) = (0usize, 0usize);
    fn walk<'a>(node: &'a Node, path: &str, parent: Option<usize>, devices: &mut Vec<Device<'a>>, disabled: &mut usize, unnamed: &mut usize) {
        for child in &node.children {
            let here = format!("{}/{}", path.trim_end_matches('/'), child.name);
            if !child.enabled() {
                *disabled += 1;
                continue;
            }
            // The processors and the memory are attributes of the board; the tree's own
            // bookkeeping (aliases, chosen, symbols) describes no hardware.
            if path == "/" && matches!(child.name.split('@').next().unwrap_or_default(), "cpus" | "memory" | "aliases" | "chosen" | "__symbols__" | "__overrides__" | "__fixups__" | "__local_fixups__" | "reserved-memory") {
                continue;
            }
            let mut next = parent;
            if child.texts("compatible").is_empty() {
                if !child.children.is_empty() || child.properties.is_empty() {
                    // A grouping node: only its children count.
                } else {
                    *unnamed += 1;
                }
            } else {
                devices.push(Device { node: child, path: here.clone(), parent, bus: node, id: String::new(), name: String::new() });
                next = Some(devices.len() - 1);
            }
            walk(child, &here, next, devices, disabled, unnamed);
        }
    }
    walk(&root, "/", None, &mut devices, &mut disabled, &mut unnamed);
    if devices.is_empty() {
        return Err("the tree declares no enabled device (a node with `compatible`)".to_string());
    }

    // Each device a name and an identifier of its own.
    let mut taken: HashMap<String, usize> = HashMap::new();
    for device in &mut devices {
        let wanted = device.node.labels.first().cloned().unwrap_or_else(|| device.node.name.clone());
        let count = taken.entry(wanted.clone()).or_insert(0);
        *count += 1;
        device.name = if *count == 1 { wanted } else { device.path.trim_start_matches('/').to_string() };
    }
    let mut ids: HashMap<String, usize> = HashMap::new();
    for device in &mut devices {
        let base: String = device.name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '-' }).collect();
        let count = ids.entry(base.clone()).or_insert(0);
        *count += 1;
        device.id = if *count == 1 { format!("PN-{base}") } else { format!("PN-{base}-{count}") };
    }

    // The phandles and the labels of the tree, to the device they name.
    let mut by_phandle: BTreeMap<u64, usize> = BTreeMap::new();
    let mut by_label: HashMap<&str, usize> = HashMap::new();
    for (index, device) in devices.iter().enumerate() {
        for name in ["phandle", "linux,phandle"] {
            if let Some(phandle) = device.node.number(name) {
                by_phandle.insert(phandle, index);
            }
        }
        for label in &device.node.labels {
            by_label.insert(label.as_str(), index);
        }
    }
    let named = |cell: &Cell| match cell {
        Cell::Number(phandle) => by_phandle.get(phandle).copied(),
        Cell::Reference(label) => by_label.get(label.as_str()).copied(),
    };

    let mut out = String::new();
    out.push_str(&format!("// Recovered from the device tree {origin}: read, not guessed.\n// Each node is an enabled device of the tree; each link, the bus it sits on or a pin that wires it.\n"));
    out.push_str(&format!("model {} {{\n    version: \"1.0.0\"\n}}\n\nphysical_architecture {} {{\n", string_literal(&board), string_literal(&board)));

    // The board: what it is, its processors, its memory.
    out.push_str(&format!("    node {} {{\n        id: \"PN-BOARD\"\n", string_literal(&board)));
    if let Some(compatible) = root.texts("compatible").first() {
        out.push_str(&format!("        type: {}\n", string_literal(compatible)));
    }
    if let Some(cpus) = root.children.iter().find(|node| node.name == "cpus") {
        let mut kinds: Vec<(String, usize)> = Vec::new();
        for cpu in cpus.children.iter().filter(|node| node.name.split('@').next() == Some("cpu") && node.enabled()) {
            let kind = cpu.texts("compatible").first().map(|k| k.to_string()).unwrap_or_else(|| "cpu".to_string());
            match kinds.iter_mut().find(|(known, _)| *known == kind) {
                Some((_, count)) => *count += 1,
                None => kinds.push((kind, 1)),
            }
        }
        if !kinds.is_empty() {
            out.push_str(&format!("        processor: {}\n", string_literal(&kinds.iter().map(|(kind, count)| format!("{count} x {kind}")).collect::<Vec<_>>().join(", "))));
        }
    }
    let bytes = memory(&root);
    if bytes > 0 {
        out.push_str(&format!("        ram: {}\n", size(bytes)));
    }
    out.push_str("    }\n");

    for device in &devices {
        let compatible = device.node.texts("compatible");
        out.push_str(&format!("    node {} {{\n        id: {}\n        type: {}\n", string_literal(&device.name), string_literal(&device.id), string_literal(compatible[0])));
        let mut said = vec![format!("{} in the tree", device.path)];
        if compatible.len() > 1 {
            said.push(format!("also compatible with {}", compatible[1..].join(", ")));
        }
        out.push_str(&format!("        description: {}\n    }}\n", string_literal(&said.join("; "))));
    }

    // A link for the bus each device sits on.
    let mut links = 0usize;
    let mut link = |out: &mut String, name: String, from: &str, to: &str, protocol: String| {
        links += 1;
        out.push_str(&format!("    physical_link {} {{ id: \"PL-{links}\" from: {} to: {} protocol: {} }}\n", string_literal(&name), string_literal(from), string_literal(to), string_literal(&protocol)));
    };
    for device in &devices {
        let (from_id, from_name) = match device.parent {
            Some(parent) => (devices[parent].id.as_str(), devices[parent].name.as_str()),
            None => ("PN-BOARD", board.as_str()),
        };
        let at = device.node.name.split_once('@').map(|(_, address)| address);
        let protocol = match (bus_of(device.bus), at) {
            (Some("I2C"), Some(address)) => format!("I2C, address 0x{address}"),
            (Some("SPI"), Some(select)) => format!("SPI, chip select {select}"),
            (Some(bus), Some(address)) => format!("{bus}, {address}"),
            (Some(bus), None) => bus.to_string(),
            (None, Some(address)) => format!("memory-mapped, 0x{address}"),
            (None, None) => "on the board".to_string(),
        };
        link(&mut out, format!("{from_name} to {}", device.name), from_id, &device.id, protocol);
    }
    // A link for each pin of a GPIO controller that wires a device.
    for device in &devices {
        for (property, values) in &device.node.properties {
            let Some(role) = property.strip_suffix("-gpios").or_else(|| property.strip_suffix("-gpio")).or_else(|| (property == "gpios" || property == "gpio").then_some("")) else {
                continue;
            };
            let cells: Vec<&Cell> = values.iter().filter_map(|v| if let Value::Cells(c) = v { Some(c) } else { None }).flatten().collect();
            let mut at = 0;
            while at < cells.len() {
                let Some(controller) = named(cells[at]) else { break };
                let width = devices[controller].node.number("#gpio-cells").unwrap_or(2) as usize;
                let pin = cells.get(at + 1).and_then(|cell| if let Cell::Number(pin) = cell { Some(*pin) } else { None });
                if let (Some(pin), true) = (pin, controller != devices.iter().position(|d| std::ptr::eq(d, device)).unwrap_or(usize::MAX)) {
                    let what = if role.is_empty() { String::new() } else { format!(" ({role})") };
                    link(&mut out, format!("{} pin {pin} to {}{what}", devices[controller].name, device.name), &devices[controller].id, &device.id, format!("GPIO {pin}{what}"));
                }
                at += 1 + width;
            }
        }
    }
    out.push_str("}\n");

    let mut notes = Vec::new();
    if disabled > 0 {
        notes.push(format!("{disabled} node(s) the tree marks as not enabled (`status`) are not written, nor what they hold"));
    }
    if unnamed > 0 {
        notes.push(format!("{unnamed} node(s) that do not say what they are compatible with are not written"));
    }
    notes.push("interrupt lines, clocks and power supplies are in the tree and are not written as links".to_string());
    notes.push("a device tree does not hold the schematic, what is plugged on a connector it does not describe, nor what the software does with the devices".to_string());
    Ok(Recovered { source: out, board, devices: devices.len(), links, notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: &str = r#"/dts-v1/;
/ {
    model = "Acme Sensor Board";
    compatible = "acme,sensor-board", "acme,soc-a1";
    #address-cells = <1>;
    #size-cells = <1>;
    cpus {
        cpu@0 { compatible = "arm,cortex-a53"; };
        cpu@1 { compatible = "arm,cortex-a53"; };
    };
    memory@0 { device_type = "memory"; reg = <0x0 0x40000000>; };
    soc {
        compatible = "simple-bus";
        gpio: gpio@7e200000 { compatible = "acme,gpio"; #gpio-cells = <2>; phandle = <0x0a>; };
        i2c1: i2c@7e804000 {
            compatible = "acme,i2c";
            /* a pressure sensor on the bus */
            bme280@76 { compatible = "bosch,bme280"; reg = <0x76>; };
        };
        spi@7e204000 {
            compatible = "acme,spi";
            flash@0 { compatible = "jedec,spi-nor"; reg = <0>; };
        };
        serial@7e201000 { compatible = "arm,pl011"; status = "disabled"; };
    };
    leds {
        compatible = "gpio-leds";
        act { gpios = <&gpio 42 0>; };
    };
    display {
        compatible = "acme,panel";
        reset-gpios = <0x0a 17 1>;
    };
};
"#;

    #[test]
    fn a_board_is_read_with_its_devices_buses_and_pins() {
        let recovered = to_arclang(BOARD, "board.dts").unwrap();
        let source = &recovered.source;
        assert_eq!(recovered.board, "Acme Sensor Board");
        // The board: its processors and its memory (1 GiB in one range).
        assert!(source.contains("processor: \"2 x arm,cortex-a53\"") && source.contains("ram: 1 GiB"), "{source}");
        // A device, named by its label, with what it is.
        assert!(source.contains("node \"i2c1\" {\n        id: \"PN-I2C1\"\n        type: \"acme,i2c\""), "{source}");
        // The bus each sits on, with its address.
        assert!(source.contains("from: \"PN-I2C1\" to: \"PN-BME280-76\" protocol: \"I2C, address 0x76\""), "{source}");
        assert!(source.contains("to: \"PN-FLASH-0\" protocol: \"SPI, chip select 0\""), "{source}");
        assert!(source.contains("from: \"PN-SOC\" to: \"PN-I2C1\" protocol: \"memory-mapped, 0x7e804000\""), "{source}");
        // A pin by the phandle of its controller: wired to the panel, for its reset.
        assert!(source.contains("physical_link \"gpio pin 17 to display (reset)\"") && source.contains("protocol: \"GPIO 17 (reset)\""), "{source}");
        // A bus numbered in its name is the same bus.
        assert_eq!(bus_of(&Node { name: "i2c0@ff020000".to_string(), ..Node::default() }), Some("I2C"));
        assert_eq!(bus_of(&Node { name: "amba".to_string(), ..Node::default() }), None);
        // A disabled device is not written, and that is said.
        assert!(!source.contains("pl011"), "{source}");
        assert!(recovered.notes.iter().any(|note| note.starts_with("1 node(s) the tree marks as not enabled")), "{:?}", recovered.notes);
    }

    #[test]
    fn a_list_of_texts_written_as_one_by_a_decompiled_tree_is_a_list() {
        let recovered = to_arclang("/ { model = \"B\"; compatible = \"acme,board\\0acme,soc\"; dev { compatible = \"acme,first\\0acme,second\"; }; };", "b.dts").unwrap();
        assert!(recovered.source.contains("type: \"acme,board\"") && recovered.source.contains("type: \"acme,first\""), "{}", recovered.source);
        assert!(recovered.source.contains("also compatible with acme,second"), "{}", recovered.source);
    }

    #[test]
    fn the_recovered_board_compiles() {
        let recovered = to_arclang(BOARD, "board.dts").unwrap();
        let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_string(&recovered.source);
        assert!(compiled.is_ok(), "{:?}\n{}", compiled.err(), recovered.source);
    }

    #[test]
    fn a_tree_that_is_not_flattened_is_refused_with_how_to_flatten_it() {
        let included = to_arclang("#include \"soc.dtsi\"\n/ { };", "b.dts").err().unwrap();
        assert!(included.contains("give the flattened tree") && included.contains("dtc -I fs -O dts /proc/device-tree"), "{included}");
        let macros = to_arclang("/ { a { compatible = \"x\"; reg = <GPIO_ACTIVE_HIGH>; }; };", "b.dts").err().unwrap();
        assert!(macros.contains("line 1") && macros.contains("macros"), "{macros}");
        let overlay = to_arclang("/ { };\n&i2c1 { status = \"okay\"; };", "b.dts").err().unwrap();
        assert!(overlay.contains("line 2") && overlay.contains("overlay"), "{overlay}");
        assert!(to_arclang("int main(void) { return 0; }", "b.dts").is_err());
    }

    #[test]
    fn the_memory_is_the_sum_of_its_ranges_on_the_cells_the_tree_says() {
        // Two cells of address, two of size: 4 GiB less a hole, in two ranges.
        let root = read("/ { #address-cells = <2>; #size-cells = <2>; memory@0 { device_type = \"memory\"; reg = <0x0 0x0 0x0 0x40000000 0x1 0x0 0x0 0x80000000>; }; };").unwrap();
        assert_eq!(memory(&root), 0x4000_0000 + 0x8000_0000);
        assert_eq!(size(memory(&root)), "3 GiB");
        assert_eq!(size(948 << 20), "948 MiB");
    }
}
