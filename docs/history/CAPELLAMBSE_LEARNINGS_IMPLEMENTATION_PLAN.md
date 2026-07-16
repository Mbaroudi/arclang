# Capellambse Learnings: Implementation Plan for Arclang

**Date:** October 28, 2025  
**Status:** Ready for Implementation  
**Estimated Effort:** 16-20 weeks  

---

## Executive Summary

After comprehensive analysis of py-capellambse (3,993 lines of diagram code, extensive extension system, code generation framework), we've identified **key capabilities to implement in Arclang** that will significantly enhance its value proposition while maintaining Rust's performance advantages.

**Key Decision:** Implement core features in Rust, NOT port Python wholesale. Focus on high-value, high-impact features that Capella users expect.

---

## Phase 1: Enhanced Diagram Capabilities (4 weeks)

### 1.1 Context Diagrams (Week 1-2)

**What Capellambse Has:**
- Focus on single component with immediate neighbors
- Show incoming/outgoing connections
- Hide unrelated elements
- Multiple context levels (direct, indirect, full)

**Implementation for Arclang:**

```rust
// New CLI command
arclang export model.arc --diagram context --focus LC-001 --depth 1 -o context.html

// In semantic.rs
impl SemanticModel {
    pub fn generate_context_diagram(
        &self,
        focus_element_id: &str,
        depth: usize,  // 1 = direct, 2 = neighbors of neighbors, etc.
    ) -> Result<ContextDiagram, CompilerError> {
        let focus = self.find_element_by_id(focus_element_id)?;
        
        let mut context_elements = HashSet::new();
        context_elements.insert(focus_element_id.to_string());
        
        // Level 1: Direct connections
        for edge in &self.edges {
            if edge.source == focus_element_id {
                context_elements.insert(edge.target.clone());
            }
            if edge.target == focus_element_id {
                context_elements.insert(edge.source.clone());
            }
        }
        
        // Level 2+: Expand outward
        if depth > 1 {
            // Recursive expansion...
        }
        
        // Filter diagram to only context elements
        let filtered_diagram = self.filter_diagram(&context_elements)?;
        
        Ok(filtered_diagram)
    }
}
```

**New Syntax (Optional):**
```arc
context_diagram "ACC Controller Context" {
    focus: "LC-001"  // Controller component
    depth: 2
    show_edges: incoming, outgoing
    hide_labels: false
}
```

**Files to Create:**
- `src/compiler/context_diagram.rs` (300 lines)
- Update `src/compiler/mod.rs`
- Update `src/cli/commands.rs`

### 1.2 Diagram Filtering & Layers (Week 3)

**What Capellambse Has:**
- Show/hide by stereotype
- Show/hide by layer
- Filter by safety level
- Highlight selected elements

**Implementation:**

```rust
#[derive(Debug, Clone)]
pub struct DiagramFilter {
    pub show_layers: Option<Vec<String>>,  // ["OA", "SA"]
    pub hide_stereotypes: Option<Vec<String>>,
    pub safety_levels: Option<Vec<SafetyLevel>>,  // [ASIL_D, ASIL_C]
    pub highlight_uuids: Option<HashSet<String>>,
}

impl SemanticModel {
    pub fn apply_diagram_filter(
        &self,
        filter: &DiagramFilter,
    ) -> Result<FilteredDiagram, CompilerError> {
        // Filter nodes
        let filtered_nodes: Vec<_> = self.components.iter()
            .filter(|c| {
                if let Some(layers) = &filter.show_layers {
                    if !layers.contains(&c.layer) {
                        return false;
                    }
                }
                if let Some(levels) = &filter.safety_levels {
                    if let Some(level) = &c.safety_level {
                        if !levels.contains(level) {
                            return false;
                        }
                    }
                }
                true
            })
            .collect();
        
        // Filter edges (only if both endpoints visible)
        // ...
        
        Ok(FilteredDiagram { nodes: filtered_nodes, edges: filtered_edges })
    }
}
```

**CLI:**
```bash
arclang export model.arc --filter-layer LA --filter-safety ASIL_D -o filtered.html
```

**Files to Modify:**
- `src/compiler/semantic.rs` (+150 lines)
- `src/compiler/arcviz_generator.rs` (+100 lines)

### 1.3 Advanced Routing Algorithms from Capellambse (Week 4)

**Port from Python:**
1. **Manhattan routing** (orthogonal, maintain axes)
2. **Tree routing** (top-down hierarchical)
3. **Smart snapping** (direction-aware edge attachment)

```rust
pub enum RoutingStyle {
    Oblique,    // Direct line (current)
    Manhattan,  // 90° angles only
    Tree,       // Hierarchical top-down
}

impl Edge {
    pub fn route(
        &mut self,
        style: RoutingStyle,
        source_box: &ComponentBox,
        target_box: &ComponentBox,
    ) {
        match style {
            RoutingStyle::Oblique => {
                // Existing implementation
            }
            RoutingStyle::Manhattan => {
                self.route_manhattan(source_box, target_box);
            }
            RoutingStyle::Tree => {
                self.route_tree(source_box, target_box);
            }
        }
    }
    
    fn route_manhattan(&mut self, source: &ComponentBox, target: &ComponentBox) {
        // Implement Capellambse's __vector_snap_manhattan
        let axis = (target.center - source.center).closest_axis();
        
        if axis.x.abs() > axis.y.abs() {
            // Horizontal primary
            let midpoint_x = (source.center.x + target.center.x) / 2.0;
            self.waypoints = vec![
                source.center,
                Vector2D::new(midpoint_x, source.center.y),
                Vector2D::new(midpoint_x, target.center.y),
                target.center,
            ];
        } else {
            // Vertical primary
            let midpoint_y = (source.center.y + target.center.y) / 2.0;
            self.waypoints = vec![
                source.center,
                Vector2D::new(source.center.x, midpoint_y),
                Vector2D::new(target.center.x, midpoint_y),
                target.center,
            ];
        }
    }
}
```

**Files to Modify:**
- `src/compiler/arcviz_ultimate_routing.rs` (+200 lines)
- Add config option for routing style

---

## Phase 2: Model Comparison & Diff (3 weeks)

### 2.1 Model Diff Engine (Week 5-6)

**What Capellambse Has:**
- Compare two model versions
- Show added/removed/modified elements
- Highlight changes in diagrams

**Implementation:**

```rust
#[derive(Debug, Clone)]
pub enum ModelChange {
    Added { element_id: String, element_type: String },
    Removed { element_id: String, element_type: String },
    Modified { 
        element_id: String, 
        field: String, 
        old_value: String, 
        new_value: String 
    },
}

pub struct ModelDiff {
    pub changes: Vec<ModelChange>,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
}

impl SemanticModel {
    pub fn diff(&self, other: &SemanticModel) -> ModelDiff {
        let mut changes = Vec::new();
        
        // Compare components
        let self_ids: HashSet<_> = self.components.iter()
            .map(|c| &c.id).collect();
        let other_ids: HashSet<_> = other.components.iter()
            .map(|c| &c.id).collect();
        
        // Added components
        for id in other_ids.difference(&self_ids) {
            let comp = other.components.iter().find(|c| &c.id == *id).unwrap();
            changes.push(ModelChange::Added {
                element_id: id.clone(),
                element_type: "Component".to_string(),
            });
        }
        
        // Removed components
        for id in self_ids.difference(&other_ids) {
            changes.push(ModelChange::Removed {
                element_id: id.clone(),
                element_type: "Component".to_string(),
            });
        }
        
        // Modified components
        for id in self_ids.intersection(&other_ids) {
            let self_comp = self.components.iter().find(|c| &c.id == *id).unwrap();
            let other_comp = other.components.iter().find(|c| &c.id == *id).unwrap();
            
            if self_comp.name != other_comp.name {
                changes.push(ModelChange::Modified {
                    element_id: id.clone(),
                    field: "name".to_string(),
                    old_value: self_comp.name.clone(),
                    new_value: other_comp.name.clone(),
                });
            }
            // ... check other fields
        }
        
        ModelDiff {
            added_count: changes.iter().filter(|c| matches!(c, ModelChange::Added { .. })).count(),
            removed_count: changes.iter().filter(|c| matches!(c, ModelChange::Removed { .. })).count(),
            modified_count: changes.iter().filter(|c| matches!(c, ModelChange::Modified { .. })).count(),
            changes,
        }
    }
}
```

**CLI:**
```bash
arclang diff model_v1.arc model_v2.arc --output diff_report.html
```

**Files to Create:**
- `src/compiler/diff.rs` (500 lines)
- `src/compiler/diff_report_template.html` (HTML template for visual diff)

### 2.2 Visual Diff Diagrams (Week 7)

**Color-coded diagram showing changes:**
- Green: Added elements
- Red: Removed elements
- Yellow: Modified elements

```rust
impl SemanticModel {
    pub fn generate_diff_diagram(
        &self,
        other: &SemanticModel,
    ) -> Result<DiffDiagram, CompilerError> {
        let diff = self.diff(other);
        
        // Generate diagram with color coding
        let mut diagram_data = self.to_diagram_data()?;
        
        for change in &diff.changes {
            match change {
                ModelChange::Added { element_id, .. } => {
                    if let Some(node) = diagram_data.find_node_mut(element_id) {
                        node.stroke_color = "#00ff00";  // Green
                        node.fill_opacity = 0.3;
                    }
                }
                ModelChange::Removed { element_id, .. } => {
                    if let Some(node) = diagram_data.find_node_mut(element_id) {
                        node.stroke_color = "#ff0000";  // Red
                        node.fill_opacity = 0.3;
                        node.stroke_dasharray = Some("5,5");
                    }
                }
                ModelChange::Modified { element_id, .. } => {
                    if let Some(node) = diagram_data.find_node_mut(element_id) {
                        node.stroke_color = "#ffaa00";  // Orange
                        node.stroke_width = 3.0;
                    }
                }
            }
        }
        
        Ok(diagram_data)
    }
}
```

---

## Phase 3: Requirements Export (3 weeks)

### 3.1 Excel Export (Week 8-9)

**What Capellambse Has:**
- Export requirements to Excel spreadsheet
- Columns: ID, Description, Priority, Safety Level, Traced To
- Formatting and styles

**Implementation using `rust_xlsxwriter`:**

```rust
use rust_xlsxwriter::*;

impl SemanticModel {
    pub fn export_requirements_to_excel(
        &self,
        path: &str,
    ) -> Result<(), CompilerError> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        
        // Header row
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);
        
        worksheet.write_with_format(0, 0, "ID", &header_format)?;
        worksheet.write_with_format(0, 1, "Description", &header_format)?;
        worksheet.write_with_format(0, 2, "Priority", &header_format)?;
        worksheet.write_with_format(0, 3, "Safety Level", &header_format)?;
        worksheet.write_with_format(0, 4, "Traced To", &header_format)?;
        
        // Data rows
        let mut row = 1;
        for req in &self.requirements {
            worksheet.write(row, 0, &req.id)?;
            worksheet.write(row, 1, &req.description)?;
            worksheet.write(row, 2, &req.priority)?;
            if let Some(level) = &req.safety_level {
                worksheet.write(row, 3, format!("{:?}", level))?;
            }
            
            // Find traces
            let traces: Vec<_> = self.traces.iter()
                .filter(|t| t.from == req.id)
                .map(|t| t.to.clone())
                .collect();
            worksheet.write(row, 4, traces.join(", "))?;
            
            row += 1;
        }
        
        // Auto-fit columns
        worksheet.autofit();
        
        workbook.save(path)?;
        Ok(())
    }
}
```

**CLI:**
```bash
arclang export model.arc --format excel --output requirements.xlsx
```

**Cargo.toml:**
```toml
[dependencies]
rust_xlsxwriter = "0.69"
```

**Files to Create:**
- `src/compiler/excel_exporter.rs` (400 lines)

### 3.2 ReqIF Export (Week 10)

**ReqIF = Requirements Interchange Format (OMG standard)**

**Implementation:**

```rust
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};

impl SemanticModel {
    pub fn export_to_reqif(&self, path: &str) -> Result<(), CompilerError> {
        let mut writer = Writer::new_with_indent(File::create(path)?, b' ', 2);
        
        // XML Declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0", Some("UTF-8"), None
        )))?;
        
        // ReqIF root
        let mut reqif = BytesStart::new("REQ-IF");
        reqif.push_attribute(("xmlns", "http://www.omg.org/spec/ReqIF/20110401/reqif.xsd"));
        reqif.push_attribute(("xmlns:xhtml", "http://www.w3.org/1999/xhtml"));
        writer.write_event(Event::Start(reqif))?;
        
        // THE-HEADER
        writer.write_event(Event::Start(BytesStart::new("THE-HEADER")))?;
        writer.write_event(Event::Start(BytesStart::new("REQ-IF-HEADER")))?;
        writer.write_event(Event::Start(BytesStart::new("IDENTIFIER")))?;
        writer.write_event(Event::Text(BytesText::new("arclang-export")))?;
        writer.write_event(Event::End(BytesEnd::new("IDENTIFIER")))?;
        writer.write_event(Event::End(BytesEnd::new("REQ-IF-HEADER")))?;
        writer.write_event(Event::End(BytesEnd::new("THE-HEADER")))?;
        
        // CORE-CONTENT
        writer.write_event(Event::Start(BytesStart::new("CORE-CONTENT")))?;
        writer.write_event(Event::Start(BytesStart::new("REQ-IF-CONTENT")))?;
        
        // DATATYPES
        self.write_reqif_datatypes(&mut writer)?;
        
        // SPEC-TYPES
        self.write_reqif_spec_types(&mut writer)?;
        
        // SPEC-OBJECTS (Requirements)
        writer.write_event(Event::Start(BytesStart::new("SPEC-OBJECTS")))?;
        for req in &self.requirements {
            self.write_reqif_spec_object(&mut writer, req)?;
        }
        writer.write_event(Event::End(BytesEnd::new("SPEC-OBJECTS")))?;
        
        // SPEC-RELATIONS (Traces)
        writer.write_event(Event::Start(BytesStart::new("SPEC-RELATIONS")))?;
        for trace in &self.traces {
            if let Some(req) = self.requirements.iter().find(|r| r.id == trace.from) {
                self.write_reqif_spec_relation(&mut writer, trace)?;
            }
        }
        writer.write_event(Event::End(BytesEnd::new("SPEC-RELATIONS")))?;
        
        writer.write_event(Event::End(BytesEnd::new("REQ-IF-CONTENT")))?;
        writer.write_event(Event::End(BytesEnd::new("CORE-CONTENT")))?;
        
        writer.write_event(Event::End(BytesEnd::new("REQ-IF")))?;
        
        Ok(())
    }
    
    fn write_reqif_spec_object(
        &self,
        writer: &mut Writer<File>,
        req: &Requirement,
    ) -> Result<(), CompilerError> {
        let mut spec_object = BytesStart::new("SPEC-OBJECT");
        spec_object.push_attribute(("IDENTIFIER", req.id.as_str()));
        writer.write_event(Event::Start(spec_object))?;
        
        // VALUES
        writer.write_event(Event::Start(BytesStart::new("VALUES")))?;
        
        // Description
        self.write_reqif_attribute_value(
            writer, 
            "AT_Description", 
            &req.description
        )?;
        
        // Priority
        if let Some(priority) = req.attributes.get("priority") {
            self.write_reqif_attribute_value(
                writer,
                "AT_Priority",
                priority
            )?;
        }
        
        writer.write_event(Event::End(BytesEnd::new("VALUES")))?;
        writer.write_event(Event::End(BytesEnd::new("SPEC-OBJECT")))?;
        
        Ok(())
    }
}
```

**CLI:**
```bash
arclang export model.arc --format reqif --output requirements.reqif
```

**Files to Create:**
- `src/compiler/reqif_exporter.rs` (600 lines)

---

## Phase 4: Code Generation Framework (4 weeks)

### 4.1 ROS2 IDL Generation (Week 11-12)

**Generate ROS2 message definitions from data types:**

```rust
impl SemanticModel {
    pub fn generate_ros2_messages(&self, output_dir: &Path) -> Result<(), CompilerError> {
        for datatype in &self.data_types {
            let msg_content = self.datatype_to_ros2_msg(datatype)?;
            let filename = format!("{}.msg", datatype.name);
            let filepath = output_dir.join(filename);
            std::fs::write(filepath, msg_content)?;
        }
        Ok(())
    }
    
    fn datatype_to_ros2_msg(&self, datatype: &DataType) -> Result<String, CompilerError> {
        let mut msg = String::new();
        
        // Header comment
        msg.push_str(&format!("# Generated from Arclang model\n"));
        msg.push_str(&format!("# Type: {}\n\n", datatype.name));
        
        // Fields
        for field in &datatype.fields {
            let ros2_type = self.capella_type_to_ros2(&field.type_name)?;
            
            // Handle arrays
            if field.is_array {
                msg.push_str(&format!("{}[] {}\n", ros2_type, field.name));
            } else {
                msg.push_str(&format!("{} {}\n", ros2_type, field.name));
            }
        }
        
        Ok(msg)
    }
    
    fn capella_type_to_ros2(&self, capella_type: &str) -> Result<String, CompilerError> {
        Ok(match capella_type {
            "Boolean" => "bool",
            "Byte" | "Char" => "byte",
            "Double" => "float64",
            "Float" => "float32",
            "Integer" | "Long" => "int32",
            "Short" => "int16",
            "String" => "string",
            "UnsignedInteger" => "uint32",
            "UnsignedShort" => "uint16",
            _ => capella_type,  // Custom type
        }.to_string())
    }
}
```

**Example Output (`Waypoint.msg`):**
```
# Generated from Arclang model
# Type: Waypoint

float64 latitude
float64 longitude
float32 altitude
```

**New ArcLang Syntax (optional):**
```arc
datatype "Waypoint" {
    field "latitude" {
        type: "Double"
        description: "GPS latitude in degrees"
    }
    field "longitude" {
        type: "Double"
    }
    field "altitude" {
        type: "Float"
        unit: "meters"
    }
}
```

**CLI:**
```bash
arclang codegen model.arc --format ros2 --output-dir ./ros2_msgs/
```

**Files to Create:**
- `src/compiler/codegen/` directory
- `src/compiler/codegen/ros2.rs` (400 lines)
- `src/compiler/codegen/mod.rs`

### 4.2 Protocol Buffers Generation (Week 13)

```rust
impl SemanticModel {
    pub fn generate_protobuf(&self, output_dir: &Path) -> Result<(), CompilerError> {
        for datatype in &self.data_types {
            let proto_content = self.datatype_to_protobuf(datatype)?;
            let filename = format!("{}.proto", datatype.name.to_lowercase());
            let filepath = output_dir.join(filename);
            std::fs::write(filepath, proto_content)?;
        }
        Ok(())
    }
    
    fn datatype_to_protobuf(&self, datatype: &DataType) -> Result<String, CompilerError> {
        let mut proto = String::new();
        
        proto.push_str("syntax = \"proto3\";\n\n");
        proto.push_str(&format!("message {} {{\n", datatype.name));
        
        for (index, field) in datatype.fields.iter().enumerate() {
            let proto_type = self.capella_type_to_protobuf(&field.type_name)?;
            let field_num = index + 1;
            
            if field.is_array {
                proto.push_str(&format!(
                    "  repeated {} {} = {};\n",
                    proto_type, field.name, field_num
                ));
            } else {
                proto.push_str(&format!(
                    "  {} {} = {};\n",
                    proto_type, field.name, field_num
                ));
            }
        }
        
        proto.push_str("}\n");
        Ok(proto)
    }
    
    fn capella_type_to_protobuf(&self, capella_type: &str) -> Result<String, CompilerError> {
        Ok(match capella_type {
            "Boolean" => "bool",
            "Double" => "double",
            "Float" => "float",
            "Integer" | "Long" => "int32",
            "String" => "string",
            "Byte" => "bytes",
            _ => capella_type,
        }.to_string())
    }
}
```

**Example Output (`waypoint.proto`):**
```protobuf
syntax = "proto3";

message Waypoint {
  double latitude = 1;
  double longitude = 2;
  float altitude = 3;
}
```

**Files to Create:**
- `src/compiler/codegen/protobuf.rs` (350 lines)

### 4.3 Python/TypeScript Stub Generation (Week 14)

```rust
impl SemanticModel {
    pub fn generate_python_stubs(&self, output_dir: &Path) -> Result<(), CompilerError> {
        for datatype in &self.data_types {
            let py_content = self.datatype_to_python(datatype)?;
            let filename = format!("{}.py", datatype.name.to_snake_case());
            let filepath = output_dir.join(filename);
            std::fs::write(filepath, py_content)?;
        }
        Ok(())
    }
    
    fn datatype_to_python(&self, datatype: &DataType) -> Result<String, CompilerError> {
        let mut py = String::new();
        
        py.push_str("from dataclasses import dataclass\n");
        py.push_str("from typing import Optional, List\n\n");
        py.push_str(&format!("@dataclass\n"));
        py.push_str(&format!("class {}:\n", datatype.name));
        
        if datatype.fields.is_empty() {
            py.push_str("    pass\n");
        } else {
            for field in &datatype.fields {
                let py_type = self.capella_type_to_python(&field.type_name)?;
                
                if field.is_array {
                    py.push_str(&format!("    {}: List[{}]\n", field.name, py_type));
                } else if field.optional {
                    py.push_str(&format!("    {}: Optional[{}] = None\n", field.name, py_type));
                } else {
                    py.push_str(&format!("    {}: {}\n", field.name, py_type));
                }
            }
        }
        
        Ok(py)
    }
}
```

**Example Output (`waypoint.py`):**
```python
from dataclasses import dataclass
from typing import Optional, List

@dataclass
class Waypoint:
    latitude: float
    longitude: float
    altitude: float
```

**Files to Create:**
- `src/compiler/codegen/python.rs` (300 lines)
- `src/compiler/codegen/typescript.rs` (350 lines)

---

## Phase 5: Extension System Architecture (3 weeks)

### 5.1 Plugin Framework (Week 15-16)

**Design extensible architecture:**

```rust
// src/extensions/mod.rs
pub trait ArclangExtension: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    fn on_model_loaded(&self, model: &mut SemanticModel) -> Result<(), ExtensionError>;
    fn on_export(&self, model: &SemanticModel, format: &str) -> Result<Option<Vec<u8>>, ExtensionError>;
    fn validate(&self, model: &SemanticModel) -> Result<Vec<ValidationIssue>, ExtensionError>;
}

// Extension registry using inventory
inventory::collect!(Box<dyn ArclangExtension>);

pub struct ExtensionManager {
    extensions: Vec<Box<dyn ArclangExtension>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        let extensions: Vec<_> = inventory::iter::<Box<dyn ArclangExtension>>
            .map(|ext| ext.clone())
            .collect();
        
        Self { extensions }
    }
    
    pub fn run_validators(&self, model: &SemanticModel) -> Vec<ValidationIssue> {
        let mut all_issues = Vec::new();
        for ext in &self.extensions {
            if let Ok(issues) = ext.validate(model) {
                all_issues.extend(issues);
            }
        }
        all_issues
    }
}
```

**Example Extension:**

```rust
// src/extensions/iso26262_validator.rs
pub struct ISO26262Validator;

impl ArclangExtension for ISO26262Validator {
    fn name(&self) -> &str {
        "ISO 26262 Validator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_model_loaded(&self, _model: &mut SemanticModel) -> Result<(), ExtensionError> {
        Ok(())
    }
    
    fn on_export(&self, _model: &SemanticModel, _format: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        Ok(None)
    }
    
    fn validate(&self, model: &SemanticModel) -> Result<Vec<ValidationIssue>, ExtensionError> {
        let mut issues = Vec::new();
        
        // Rule: All ASIL_D components must have hazard analysis
        for comp in &model.components {
            if comp.safety_level == Some(SafetyLevel::ASIL_D) {
                let has_hazard = model.hazards.iter()
                    .any(|h| h.related_component == Some(comp.id.clone()));
                
                if !has_hazard {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        element_id: comp.id.clone(),
                        rule: "ISO26262_HAZARD_REQUIRED".to_string(),
                        message: format!(
                            "ASIL-D component '{}' must have associated hazard analysis",
                            comp.name
                        ),
                    });
                }
            }
        }
        
        Ok(issues)
    }
}

// Register extension
inventory::submit! {
    Box::new(ISO26262Validator) as Box<dyn ArclangExtension>
}
```

**Files to Create:**
- `src/extensions/mod.rs` (200 lines)
- `src/extensions/iso26262_validator.rs` (400 lines)
- `src/extensions/do178c_validator.rs` (350 lines)

### 5.2 Validation Rule System (Week 17)

**Declarative validation rules:**

```rust
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub check: Box<dyn Fn(&SemanticModel, &str) -> bool>,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

pub struct ValidationEngine {
    rules: Vec<ValidationRule>,
}

impl ValidationEngine {
    pub fn with_default_rules() -> Self {
        let mut engine = Self { rules: Vec::new() };
        
        // Rule: All requirements must be traced
        engine.add_rule(ValidationRule {
            id: "REQ_TRACED".to_string(),
            name: "Requirement Traceability".to_string(),
            description: "All requirements must be traced to at least one component".to_string(),
            severity: Severity::Error,
            check: Box::new(|model, req_id| {
                model.traces.iter().any(|t| t.from == req_id)
            }),
        });
        
        // Rule: No orphan components
        engine.add_rule(ValidationRule {
            id: "NO_ORPHAN_COMPONENTS".to_string(),
            name: "No Orphan Components".to_string(),
            description: "All components must be deployed to physical nodes".to_string(),
            severity: Severity::Warning,
            check: Box::new(|model, comp_id| {
                model.traces.iter().any(|t| {
                    t.from == comp_id && t.trace_type == "deploys"
                })
            }),
        });
        
        engine
    }
    
    pub fn validate(&self, model: &SemanticModel) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        for req in &model.requirements {
            for rule in &self.rules {
                if !(rule.check)(model, &req.id) {
                    issues.push(ValidationIssue {
                        severity: rule.severity.clone(),
                        element_id: req.id.clone(),
                        rule: rule.id.clone(),
                        message: format!("{}: {}", rule.name, rule.description),
                    });
                }
            }
        }
        
        issues
    }
}
```

**CLI:**
```bash
arclang validate model.arc --profile ISO26262_ASIL_D
```

**Files to Create:**
- `src/compiler/validation.rs` (500 lines)
- `src/compiler/validation_rules.rs` (600 lines)

---

## Phase 6: Python Bridge (Optional, 3 weeks)

### 6.1 PyO3 Bindings (Week 18-19)

**Expose Arclang to Python:**

```rust
// src/python/mod.rs
use pyo3::prelude::*;

#[pyclass]
struct PyArclangModel {
    inner: SemanticModel,
}

#[pymethods]
impl PyArclangModel {
    #[staticmethod]
    fn load(path: String) -> PyResult<Self> {
        let model = SemanticModel::from_file(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!("Failed to load model: {}", e)
            ))?;
        
        Ok(Self { inner: model })
    }
    
    #[getter]
    fn components(&self) -> Vec<PyComponent> {
        self.inner.components.iter()
            .map(|c| PyComponent::from(c))
            .collect()
    }
    
    fn generate_diagram(&self, diagram_type: String) -> PyResult<String> {
        // Generate and return SVG string
        Ok("<!-- SVG content -->".to_string())
    }
    
    fn export_to_excel(&self, path: String) -> PyResult<()> {
        self.inner.export_requirements_to_excel(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!("Export failed: {}", e)
            ))
    }
}

#[pymodule]
fn arclang(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyArclangModel>()?;
    Ok(())
}
```

**Python Usage:**
```python
import arclang

# Load model
model = arclang.PyArclangModel.load("model.arc")

# Access components
for comp in model.components:
    print(f"{comp.id}: {comp.name}")

# Export
model.export_to_excel("requirements.xlsx")
```

**Cargo.toml:**
```toml
[lib]
name = "arclang"
crate-type = ["cdylib", "rlib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
```

**Files to Create:**
- `src/python/mod.rs` (600 lines)
- `python/arclang/__init__.py` (wrapper)
- `pyproject.toml` for Python packaging

### 6.2 Capellambse Interop (Week 20)

**Python bridge to use both libraries:**

```python
# python/arclang/capellambse_bridge.py
import arclang
from capellambse import MelodyModel

class ArclangCapellamseBridge:
    """Bridge between Arclang and Capellambse."""
    
    def __init__(self, arclang_path: str):
        self.arclang_model = arclang.PyArclangModel.load(arclang_path)
        # Export to Capella XML
        self.arclang_model.export("temp.capella", format="capella")
        # Load in Capellambse
        self.capellambse_model = MelodyModel("temp.capella")
    
    def query_with_capellambse(self, xpath: str):
        """Use Capellambse's advanced querying."""
        return self.capellambse_model.xpath(xpath)
    
    def analyze_complexity(self):
        """Use Capellambse's complexity metrics."""
        from capellambse.extensions.metrics import calculate_complexity
        return calculate_complexity(self.capellambse_model)
    
    def generate_with_arclang(self, diagram_type: str):
        """Use Arclang's fast diagram generation."""
        return self.arclang_model.generate_diagram(diagram_type)
```

**Usage:**
```python
bridge = ArclangCapellamseBridge("model.arc")

# Use Arclang for fast operations
svg = bridge.generate_with_arclang("component")

# Use Capellambse for complex queries
results = bridge.query_with_capellambse("//oa:OperationalActivity")

# Use Capellambse for metrics
complexity = bridge.analyze_complexity()
```

---

## Implementation Priority Summary

### Must Have (Weeks 1-14):
1. ✅ Context diagrams
2. ✅ Diagram filtering
3. ✅ Model diff/comparison
4. ✅ Excel export
5. ✅ ReqIF export
6. ✅ ROS2/Protobuf code generation

### Should Have (Weeks 15-17):
7. ✅ Extension system
8. ✅ Validation rules

### Nice to Have (Weeks 18-20):
9. ⚠️ Python bindings
10. ⚠️ Capellambse interop

---

## Updated Roadmap Integration

Add to `/Users/malek/Arclang/README.md`:

```markdown
### Version 1.3 (Enhanced Capabilities - Capellambse-Inspired) - Q1 2025
- [ ] Context diagrams (focused component views) ✨
- [ ] Diagram filtering and layers (show/hide by criteria) ✨
- [ ] Model comparison and diff visualization ✨
- [ ] Requirements export (Excel, ReqIF) ✨
- [ ] Advanced routing (Manhattan, Tree) ✨
- [ ] Code generation framework (ROS2, Protobuf, Python) ✨

### Version 1.4 (Extension System) - Q2 2025
- [ ] Plugin architecture with extension traits ✨
- [ ] Validation rule system (declarative) ✨
- [ ] ISO 26262 validation extension ✨
- [ ] DO-178C validation extension ✨

### Version 1.5 (Python Ecosystem Bridge) - Q3 2025
- [ ] PyO3 bindings for Python integration
- [ ] Capellambse interop layer
- [ ] Jupyter notebook kernel
```

---

## Success Metrics

### Phase 1-2 (Diagrams & Diff):
- Context diagrams generate in <500ms
- Diagram filtering supports 10+ criteria
- Model diff handles 1000+ element models

### Phase 3 (Export):
- Excel export for 500+ requirements in <2s
- ReqIF validation passes OMG standard

### Phase 4 (Code Gen):
- ROS2 messages compile without errors
- Protobuf definitions pass `protoc` validation
- Python stubs pass `mypy` type checking

### Phase 5 (Extensions):
- 5+ validation rules per standard
- Extension overhead <10ms per rule
- 90% of Capella validation rules covered

### Phase 6 (Python):
- Python bindings achieve 80% API coverage
- Performance within 2x of pure Rust
- Seamless Capellambse interop

---

## Risk Assessment

### Low Risk:
- Context diagrams (clear requirements)
- Excel export (mature crate)
- ROS2 generation (simple text templates)

### Medium Risk:
- Model diff (complex change detection)
- ReqIF export (complex XML standard)
- Extension system (API design critical)

### High Risk:
- Python bindings (PyO3 learning curve)
- Capellambse interop (two-library coordination)

---

## Conclusion

This plan transforms Arclang from a "good" MBSE compiler into an **exceptional, Capella-competitive** tool by implementing the most valuable Capellambse features in high-performance Rust.

**Key Philosophy:**
- Learn from Capellambse's design
- Implement in idiomatic Rust
- Maintain performance advantage
- Provide optional Python bridge for ecosystem access

**Total Effort:** 16-20 weeks (4-5 months)  
**Team Size:** 1-2 developers  
**Impact:** Transform Arclang into enterprise-grade MBSE toolchain

---

**Next Steps:**
1. Review and approve this plan
2. Create GitHub project board with milestones
3. Begin Phase 1: Context Diagrams (Week 1)
4. Iterative development with weekly demos

**Status:** ✅ Ready to Start
