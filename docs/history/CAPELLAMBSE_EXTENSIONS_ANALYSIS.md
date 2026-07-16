# Capellambse Extensions System Analysis

## Executive Summary

This document analyzes the capellambse extensions architecture to extract patterns applicable to Arclang's Rust implementation. The analysis covers four major extensions (validation, reqif, pvmt, metrics) and the core plugin pattern.

---

## 1. Extension Architecture Overview

### 1.1 Plugin Discovery Pattern

**Python Implementation:**
```python
# pyproject.toml
[project.entry-points."capellambse.model_extensions"]
filtering = "capellambse.extensions.filtering:init"
pvmt = "capellambse.extensions.pvmt:init"
reqif = "capellambse.extensions.reqif:init"
validation = "capellambse.extensions.validation:init"

# __init__.py
def load_model_extensions() -> None:
    for entrypoint in imm.entry_points(group="capellambse.model_extensions"):
        try:
            initfunc = entrypoint.load()
            initfunc()
        except Exception:
            logging.exception("Cannot load model extension...")
```

**Key Characteristics:**
- Declarative registration via entry points
- Lazy loading on first model access
- Graceful failure handling
- Each extension exports an `init()` function

**Rust Translation for Arclang:**
```rust
// Extension trait pattern
pub trait ModelExtension: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&self, model: &mut Model) -> Result<(), ExtensionError>;
    fn version(&self) -> &'static str { "1.0.0" }
}

// Registry with lazy loading
pub struct ExtensionRegistry {
    extensions: RwLock<HashMap<&'static str, Box<dyn ModelExtension>>>,
    loaded: AtomicBool,
}

impl ExtensionRegistry {
    pub fn register(&self, ext: Box<dyn ModelExtension>) {
        self.extensions.write().unwrap()
            .insert(ext.name(), ext);
    }
    
    pub fn load_all(&self, model: &mut Model) -> Result<(), Vec<ExtensionError>> {
        if self.loaded.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        
        let extensions = self.extensions.read().unwrap();
        let mut errors = Vec::new();
        
        for ext in extensions.values() {
            if let Err(e) = ext.init(model) {
                log::error!("Failed to load extension {}: {}", ext.name(), e);
                errors.push(e);
            }
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// Macro for easy extension registration
#[macro_export]
macro_rules! register_extension {
    ($ext:expr) => {
        #[ctor::ctor]
        fn register() {
            EXTENSION_REGISTRY.register(Box::new($ext));
        }
    };
}
```

### 1.2 Namespace-Based Extension Pattern

**Python Implementation:**
```python
# Extension defines its namespace
NS = m.Namespace(
    "http://www.polarsys.org/capella/filtering/{VERSION}",
    "filtering",
    "org.polarsys.capella.filtering",
    "7.0.0",
)

# Classes inherit from metamodel base
class FilteringModel(capellacore.NamedElement):
    criteria = m.Containment["FilteringCriterion"](
        "ownedFilteringCriteria", (NS, "FilteringCriterion")
    )
```

**Rust Translation:**
```rust
// Namespace as a struct with const functions
#[derive(Debug, Clone, Copy)]
pub struct Namespace {
    pub uri_template: &'static str,
    pub prefix: &'static str,
    pub symbolic_name: &'static str,
    pub version: &'static str,
}

impl Namespace {
    pub const fn new(
        uri: &'static str,
        prefix: &'static str,
        symbolic: &'static str,
        version: &'static str,
    ) -> Self {
        Self {
            uri_template: uri,
            prefix,
            symbolic_name: symbolic,
            version,
        }
    }
    
    pub fn uri(&self) -> String {
        self.uri_template.replace("{VERSION}", self.version)
    }
}

// Extension namespace declaration
pub mod filtering {
    use super::*;
    
    pub const NS: Namespace = Namespace::new(
        "http://www.polarsys.org/capella/filtering/{VERSION}",
        "filtering",
        "org.polarsys.capella.filtering",
        "7.0.0",
    );
    
    // Elements within namespace
    #[derive(Debug)]
    pub struct FilteringModel {
        pub criteria: Vec<FilteringCriterion>,
        pub criterion_pkgs: Vec<FilteringCriterionPkg>,
    }
}
```

### 1.3 Model Injection Pattern

**Python Implementation:**
```python
def init() -> None:
    # Inject properties onto existing classes
    capellambse.MelodyModel.validation = property(ModelValidation)
    m.ModelElement.validation = m.AlternateAccessor(ElementValidation)
    m.ModelElement.validate = property(lambda self: self.validation.validate)
```

**Rust Translation:**
```rust
// Extension trait pattern (Rust's alternative to monkey patching)
pub trait ValidationExt {
    fn validation(&self) -> Validation;
    fn validate(&self) -> Results;
}

impl ValidationExt for Model {
    fn validation(&self) -> Validation {
        Validation::for_model(self)
    }
    
    fn validate(&self) -> Results {
        self.validation().validate()
    }
}

impl ValidationExt for ModelElement {
    fn validation(&self) -> Validation {
        Validation::for_element(self)
    }
    
    fn validate(&self) -> Results {
        self.validation().validate()
    }
}

// Usage: bring extension trait into scope
use arclang::extensions::validation::ValidationExt;
let results = model.validate();
```

---

## 2. Validation Extension Deep Dive

### 2.1 Rule Definition Pattern

**Python Implementation:**
```python
@rule(
    category=Category.REQUIRED,
    types=[
        mm.sa.Capability,
        mm.oa.OperationalCapability,
    ],
    id="Rule-002",
    name="Capability involves an Entity / Actor",
    rationale="...",
    action="...",
)
def capability_involves_entity(obj: ModelElement) -> bool:
    if isinstance(obj, mm.oa.OperationalCapability):
        has_involvements = bool(obj.involved_entities)
    else:
        assert isinstance(obj, mm.sa.Capability)
        has_involvements = bool(obj.involved_components)
    return has_involvements or bool(obj.included_by)
```

**Key Characteristics:**
- Declarative rule registration via decorator
- Type-based filtering
- Metadata embedded in decorator
- Simple boolean return for pass/fail

**Rust Translation:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Required,
    Recommended,
    Suggested,
}

pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    pub category: RuleCategory,
    pub rationale: &'static str,
    pub action: &'static str,
    pub types: &'static [TypeId],
    pub validator: fn(&dyn ModelElement) -> bool,
}

// Macro for rule definition
macro_rules! define_rule {
    (
        id: $id:literal,
        name: $name:literal,
        category: $category:expr,
        types: [$($ty:ty),*],
        rationale: $rationale:literal,
        action: $action:literal,
        validator: $validator:expr
    ) => {
        Rule {
            id: $id,
            name: $name,
            category: $category,
            rationale: $rationale,
            action: $action,
            types: &[$(TypeId::of::<$ty>()),*],
            validator: $validator,
        }
    };
}

// Usage
static RULE_002: Rule = define_rule! {
    id: "Rule-002",
    name: "Capability involves an Entity / Actor",
    category: RuleCategory::Required,
    types: [Capability, OperationalCapability],
    rationale: "Each Capability serves a need...",
    action: "Add at least one involved Actor...",
    validator: |obj| {
        if let Some(cap) = obj.downcast_ref::<OperationalCapability>() {
            !cap.involved_entities.is_empty()
        } else if let Some(cap) = obj.downcast_ref::<Capability>() {
            !cap.involved_components.is_empty()
        } else {
            false
        }
    }
};

// Inventory pattern for rule registration
inventory::collect!(Rule);
inventory::submit! { RULE_002 }
```

### 2.2 Virtual Type System

**Python Implementation:**
```python
@virtual_type(mm.sa.SystemComponent)
def SystemActor(cmp: mm.sa.SystemComponent) -> bool:
    return cmp.is_actor

@virtual_type(mm.sa.SystemComponent)
def SystemComponent(cmp: mm.sa.SystemComponent) -> bool:
    return not cmp.is_actor
```

**Purpose:** Create refinement types based on runtime properties

**Rust Translation:**
```rust
// Type refinement with newtype pattern
pub struct SystemActor(SystemComponent);
pub struct SystemComponentNoActor(SystemComponent);

impl TryFrom<SystemComponent> for SystemActor {
    type Error = ValidationError;
    
    fn try_from(comp: SystemComponent) -> Result<Self, Self::Error> {
        if comp.is_actor {
            Ok(SystemActor(comp))
        } else {
            Err(ValidationError::NotAnActor)
        }
    }
}

// Or use a trait-based approach for more flexibility
pub trait VirtualType: Sized {
    type Base;
    fn matches(base: &Self::Base) -> bool;
    fn try_from_base(base: Self::Base) -> Result<Self, Self::Base>;
}

pub struct SystemActor;
impl VirtualType for SystemActor {
    type Base = SystemComponent;
    
    fn matches(base: &Self::Base) -> bool {
        base.is_actor
    }
    
    fn try_from_base(base: Self::Base) -> Result<Self, Self::Base> {
        if Self::matches(&base) {
            Ok(SystemActor)
        } else {
            Err(base)
        }
    }
}
```

### 2.3 Results Collection Pattern

**Python Implementation:**
```python
class Results:
    def __init__(self, results: Iterable[tuple[tuple[Rule, str], Result]] = ()):
        self.__container = dict(results)
    
    def by_rule(self, key: Rule | str) -> Results:
        # Filter by rule
    
    def by_object(self, target: str | ModelElement) -> Results:
        # Filter by object
    
    def by_category(self, category: Category | str) -> Results:
        # Filter by category
```

**Rust Translation:**
```rust
#[derive(Debug)]
pub struct ValidationResult {
    pub rule: &'static Rule,
    pub object: ElementRef,
    pub passed: bool,
}

#[derive(Debug, Default)]
pub struct ValidationResults {
    results: HashMap<(RuleId, Uuid), ValidationResult>,
}

impl ValidationResults {
    pub fn by_rule(&self, rule_id: &str) -> impl Iterator<Item = &ValidationResult> {
        self.results.values()
            .filter(move |r| r.rule.id == rule_id)
    }
    
    pub fn by_object(&self, uuid: Uuid) -> impl Iterator<Item = &ValidationResult> {
        self.results.values()
            .filter(move |r| r.object.uuid == uuid)
    }
    
    pub fn by_category(&self, cat: RuleCategory) -> impl Iterator<Item = &ValidationResult> {
        self.results.values()
            .filter(move |r| r.rule.category == cat)
    }
    
    pub fn by_passed(&self, passed: bool) -> impl Iterator<Item = &ValidationResult> {
        self.results.values()
            .filter(move |r| r.passed == passed)
    }
    
    // Chaining support
    pub fn filter<F>(&self, predicate: F) -> Vec<&ValidationResult>
    where
        F: Fn(&ValidationResult) -> bool,
    {
        self.results.values().filter(predicate).collect()
    }
}
```

---

## 3. ReqIF Extension Deep Dive

### 3.1 Requirements Model Structure

**Python Implementation:**
```python
class Requirement(AttributeOwner, SharedDirectAttributes):
    _xmltag = "ownedRequirements"
    
    type = m.Single["RequirementType"](
        m.Association((NS, "RequirementType"), "requirementType")
    )
    owned_relations = m.Containment["AbstractRelation"](
        "ownedRelations", (NS, "AbstractRelation")
    )
    text = m.HTMLStringPOD("ReqIFText")
```

**Key Patterns:**
- Type hierarchy (abstract base classes)
- Mix-in pattern for shared attributes
- Strong typing for relationships
- XML tag customization

**Rust Translation:**
```rust
// Trait-based composition instead of multiple inheritance
pub trait AttributeOwner {
    fn attributes(&self) -> &[Attribute];
    fn attributes_mut(&mut self) -> &mut Vec<Attribute>;
}

pub trait SharedDirectAttributes {
    fn name(&self) -> &str;
    fn prefix(&self) -> &str;
}

#[derive(Debug)]
pub struct Requirement {
    // Core fields
    uuid: Uuid,
    identifier: String,
    description: String,
    long_name: String,
    
    // SharedDirectAttributes
    name: String,
    prefix: String,
    
    // Relationships
    req_type: Option<ElementRef<RequirementType>>,
    owned_relations: Vec<AbstractRelation>,
    
    // Content
    text: HtmlString,
    attributes: Vec<Attribute>,
}

impl AttributeOwner for Requirement {
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
    
    fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attributes
    }
}

impl SharedDirectAttributes for Requirement {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn prefix(&self) -> &str {
        &self.prefix
    }
}
```

### 3.2 ReqIF Export Architecture

**Python Implementation:**
```python
def export_module(module, target, *, metadata=None, pretty=False, compress=None):
    data = etree.Element("REQ-IF", attrib={...}, nsmap=NSMAP)
    header, timestamp = _build_header(module, metadata)
    data.append(header)
    data.append(_build_content(module, timestamp))
    
    # Write to file or stream
```

**Key Patterns:**
- Builder pattern for XML construction
- Two-phase construction (header + content)
- Metadata collection and synthesis
- Flexible output (file/stream, compressed/uncompressed)

**Rust Translation:**
```rust
pub struct ReqIFExporter {
    model: Arc<Model>,
    metadata: ExportMetadata,
}

pub struct ExportMetadata {
    pub comment: Option<String>,
    pub title: Option<String>,
    pub creation_time: Option<DateTime<Utc>>,
}

impl ReqIFExporter {
    pub fn export<W: Write>(
        &self,
        module: &CapellaModule,
        writer: W,
        options: ExportOptions,
    ) -> Result<(), ExportError> {
        let mut builder = ReqIFBuilder::new(module);
        
        // Phase 1: Collect metadata
        let timestamp = self.metadata.creation_time
            .unwrap_or_else(Utc::now);
        
        // Phase 2: Build structure
        let header = builder.build_header(&self.metadata, timestamp);
        let content = builder.build_content(timestamp)?;
        
        // Phase 3: Serialize
        let doc = ReqIFDocument { header, content };
        
        if options.compress {
            self.write_compressed(doc, writer, options.pretty)
        } else {
            self.write_plain(doc, writer, options.pretty)
        }
    }
}

// Builder pattern for XML construction
struct ReqIFBuilder<'a> {
    module: &'a CapellaModule,
    datatypes: Vec<DataType>,
    spec_types: Vec<SpecType>,
    spec_objects: Vec<SpecObject>,
}

impl<'a> ReqIFBuilder<'a> {
    fn build_header(
        &mut self,
        metadata: &ExportMetadata,
        timestamp: DateTime<Utc>,
    ) -> Header {
        Header {
            identifier: format!("_{}", self.module.uuid().to_uppercase()),
            comment: metadata.comment.clone()
                .unwrap_or_else(|| self.default_comment()),
            creation_time: timestamp,
            reqif_version: "1.1",
            tool_id: format!("arclang v{}", env!("CARGO_PKG_VERSION")),
            title: metadata.title.clone()
                .unwrap_or_else(|| self.module.long_name.clone()),
        }
    }
    
    fn build_content(&mut self, timestamp: DateTime<Utc>) -> Result<Content, ExportError> {
        // Collect all requirement types and attributes
        let req_types = self.collect_requirement_types();
        
        // Build datatypes from attribute definitions
        self.datatypes = self.build_datatypes(&req_types, timestamp);
        
        // Build spec types
        self.spec_types = self.build_spec_types(&req_types, timestamp);
        
        // Build spec objects (requirements)
        self.spec_objects = self.build_spec_objects(timestamp)?;
        
        Ok(Content {
            datatypes: self.datatypes.clone(),
            spec_types: self.spec_types.clone(),
            spec_objects: self.spec_objects.clone(),
            specifications: self.build_specifications(timestamp),
            // ...
        })
    }
}
```

### 3.3 Attribute Type System

**Python Implementation:**
```python
class Attribute(IdentifiableElement, abstract=True):
    definition = m.Single["AttributeDefinition"](
        m.Association((NS, "AttributeDefinition"), "definition")
    )
    
class StringValueAttribute(Attribute):
    value = m.StringPOD("value")

class IntegerValueAttribute(Attribute):
    value = m.IntPOD("value")

class EnumerationValueAttribute(Attribute):
    values = m.Association["EnumValue"]((NS, "EnumValue"), "values")
```

**Rust Translation:**
```rust
// Enum-based variant type
#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Real(f64),
    Date(DateTime<Utc>),
    Enumeration(Vec<EnumValue>),
}

#[derive(Debug)]
pub struct Attribute {
    pub uuid: Uuid,
    pub definition: Option<ElementRef<AttributeDefinition>>,
    pub value: AttributeValue,
}

impl Attribute {
    pub fn as_string(&self) -> Option<&str> {
        match &self.value {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_integer(&self) -> Option<i64> {
        match self.value {
            AttributeValue::Integer(i) => Some(i),
            _ => None,
        }
    }
    
    // Type-safe builders
    pub fn new_string(definition: Option<ElementRef<AttributeDefinition>>, value: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            definition,
            value: AttributeValue::String(value),
        }
    }
    
    pub fn new_enum(
        definition: Option<ElementRef<AttributeDefinition>>,
        values: Vec<EnumValue>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            definition,
            value: AttributeValue::Enumeration(values),
        }
    }
}

// Type-safe attribute definition
#[derive(Debug)]
pub struct AttributeDefinition {
    pub uuid: Uuid,
    pub identifier: String,
    pub long_name: String,
    pub description: String,
    pub definition_type: Option<ElementRef<DataTypeDefinition>>,
}

#[derive(Debug)]
pub enum DataTypeDefinition {
    String { max_length: Option<usize> },
    Integer { min: i64, max: i64 },
    Real { min: f64, max: f64, accuracy: Option<u32> },
    Boolean,
    Date,
    Enumeration { values: Vec<EnumValue> },
}
```

---

## 4. PVMT Extension Deep Dive

### 4.1 Property Value Management Architecture

**Python Implementation:**
```python
class ObjectPVMT:
    """Provides access to managed property values on an element."""
    
    def __getitem__(self, key: str) -> Any:
        path = key.split(".")
        domain, groupname, *_ = path
        groupdef = self._model.pvmt.domains[domain].groups[groupname]
        group = groupdef.apply(self.owner)
        if len(path) < 3:
            return group
        return group.property_values[path[2]]
```

**Key Patterns:**
- Dictionary-like interface for property access
- Hierarchical path structure (domain.group.property)
- Lazy application of property groups
- Virtual property system

**Rust Translation:**
```rust
pub struct ObjectPVMT {
    owner: ElementRef,
    model: Arc<Model>,
}

impl ObjectPVMT {
    pub fn get(&self, path: &str) -> Result<PropertyValue, PVMTError> {
        let parts: Vec<&str> = path.split('.').collect();
        
        match parts.len() {
            2 => {
                // Return group
                let group = self.get_group(parts[0], parts[1])?;
                Ok(PropertyValue::Group(group))
            }
            3 => {
                // Return specific property
                let group = self.get_group(parts[0], parts[1])?;
                group.property_values.get(parts[2])
                    .cloned()
                    .ok_or_else(|| PVMTError::PropertyNotFound(parts[2].to_string()))
            }
            _ => Err(PVMTError::InvalidPath(path.to_string())),
        }
    }
    
    pub fn set(&mut self, path: &str, value: PropertyValue) -> Result<(), PVMTError> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() != 3 {
            return Err(PVMTError::InvalidPath(path.to_string()));
        }
        
        let mut group = self.get_or_create_group(parts[0], parts[1])?;
        group.property_values.insert(parts[2].to_string(), value);
        Ok(())
    }
    
    // Fluent API
    pub fn domain(&self, name: &str) -> DomainView {
        DomainView {
            pvmt: self,
            domain: name,
        }
    }
}

// Fluent API for cleaner access
pub struct DomainView<'a> {
    pvmt: &'a ObjectPVMT,
    domain: &'a str,
}

impl<'a> DomainView<'a> {
    pub fn group(&self, name: &str) -> GroupView {
        GroupView {
            pvmt: self.pvmt,
            domain: self.domain,
            group: name,
        }
    }
}

pub struct GroupView<'a> {
    pvmt: &'a ObjectPVMT,
    domain: &'a str,
    group: &'a str,
}

impl<'a> GroupView<'a> {
    pub fn get(&self, property: &str) -> Result<PropertyValue, PVMTError> {
        let path = format!("{}.{}.{}", self.domain, self.group, property);
        self.pvmt.get(&path)
    }
}

// Usage:
// obj.pvmt().domain("DarkMagic").group("Power").get("Max")?
```

### 4.2 Selector Rules System

**Python Implementation:**
```python
@dataclasses.dataclass(frozen=True)
class SelectorRules:
    raw: str
    
    @property
    def classes(self) -> tuple[type[m.ModelObject], ...]:
        # Parse [CLASS]...[/CLASS] tags
    
    @property
    def layers(self) -> tuple[type[mm.cs.BlockArchitecture], ...]:
        # Parse [ARCHITECTURE]...[/ARCHITECTURE] tags
    
    @property
    def properties(self) -> tuple[tuple[str, str, str], ...]:
        # Parse [PROPERTY]...[/PROPERTY] tags
```

**Key Patterns:**
- Domain-specific language for rule specification
- Multiple selector types (class, layer, property)
- Lazy parsing of selector string

**Rust Translation:**
```rust
#[derive(Debug, Clone)]
pub struct SelectorRules {
    raw: String,
    // Cached parsed values
    classes: OnceCell<Vec<TypeId>>,
    layers: OnceCell<Vec<LayerType>>,
    properties: OnceCell<Vec<PropertySelector>>,
}

#[derive(Debug, Clone)]
pub struct PropertySelector {
    pub property_path: String,  // "domain.group.property"
    pub operator: ComparisonOp,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    Eq,     // =
    Ne,     // !
    Contains, // ?
    StartsWith, // &
    EndsWith,   // #
    Lt,     // <
    Gt,     // >
    Le,     // %
    Ge,     // :
}

impl SelectorRules {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        Ok(Self {
            raw: raw.to_string(),
            classes: OnceCell::new(),
            layers: OnceCell::new(),
            properties: OnceCell::new(),
        })
    }
    
    pub fn classes(&self) -> &[TypeId] {
        self.classes.get_or_init(|| {
            self.parse_classes()
        })
    }
    
    pub fn layers(&self) -> &[LayerType] {
        self.layers.get_or_init(|| {
            self.parse_layers()
        })
    }
    
    pub fn properties(&self) -> &[PropertySelector] {
        self.properties.get_or_init(|| {
            self.parse_properties()
        })
    }
    
    fn parse_classes(&self) -> Vec<TypeId> {
        let re = Regex::new(r"(?m)\[CLASS\]\s*(.+?)\s*\[/CLASS\]").unwrap();
        re.captures_iter(&self.raw)
            .flat_map(|cap| {
                cap[1].split(',')
                    .filter_map(|uri| self.resolve_class_from_uri(uri.trim()))
            })
            .collect()
    }
    
    fn parse_properties(&self) -> Vec<PropertySelector> {
        let re = Regex::new(
            r"(?m)\[PROPERTY\]\s*([^.]+\.[^.]+\.[^.]+)([=!?&#<>%:])(.+?)\s*\[/PROPERTY\]"
        ).unwrap();
        
        re.captures_iter(&self.raw)
            .map(|cap| PropertySelector {
                property_path: cap[1].to_string(),
                operator: ComparisonOp::from_str(&cap[2]).unwrap(),
                value: cap[3].to_string(),
            })
            .collect()
    }
}

// Application of selector rules
impl ManagedGroup {
    pub fn applies_to(&self, obj: &dyn ModelElement) -> bool {
        let rules = &self.selector;
        
        // Check class constraints
        if !rules.classes().is_empty() {
            let obj_type = obj.type_id();
            if !rules.classes().contains(&obj_type) {
                return false;
            }
        }
        
        // Check layer constraints
        if !rules.layers().is_empty() {
            if let Some(layer) = obj.layer() {
                if !rules.layers().contains(&layer.layer_type()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check property constraints
        for prop_sel in rules.properties() {
            if !self.check_property(obj, prop_sel) {
                return false;
            }
        }
        
        true
    }
}
```

---

## 5. Metrics Extension

### 5.1 Model Quantification Pattern

**Python Implementation:**
```python
OBJECTS_OF_INTEREST: dict[str, list[type[m.ModelElement]]] = {
    "oa": [
        *COMMON_OBJECTS,
        mm.oa.OperationalCapability,
        mm.oa.OperationalActivity,
    ],
    # ...
}

def quantify_model_layers(model):
    objects = []
    diagrams = []
    for layer, object_types in OBJECTS_OF_INTEREST.items():
        layer_obj = getattr(model.project.model_root, layer, None)
        if layer_obj:
            layer_objects = len(model.search(*object_types, below=layer_obj))
            layer_diagrams = len(layer_obj.diagrams)
        else:
            layer_objects = 0
            layer_diagrams = 0
        objects.append(layer_objects)
        diagrams.append(layer_diagrams)
    return objects, diagrams
```

**Rust Translation:**
```rust
pub struct ModelMetrics {
    pub layers: HashMap<LayerType, LayerMetrics>,
}

pub struct LayerMetrics {
    pub object_count: usize,
    pub diagram_count: usize,
    pub object_types: HashMap<TypeId, usize>,
}

pub struct MetricsCollector {
    model: Arc<Model>,
    object_types_of_interest: HashMap<LayerType, Vec<TypeId>>,
}

impl MetricsCollector {
    pub fn new(model: Arc<Model>) -> Self {
        let mut types_of_interest = HashMap::new();
        
        types_of_interest.insert(
            LayerType::OperationalAnalysis,
            vec![
                TypeId::of::<OperationalCapability>(),
                TypeId::of::<OperationalActivity>(),
                TypeId::of::<Entity>(),
                // Common objects
                TypeId::of::<StateMachine>(),
                TypeId::of::<FunctionalExchange>(),
            ],
        );
        
        // Similar for other layers...
        
        Self {
            model,
            object_types_of_interest: types_of_interest,
        }
    }
    
    pub fn collect(&self) -> ModelMetrics {
        let mut layers = HashMap::new();
        
        for (layer_type, type_ids) in &self.object_types_of_interest {
            if let Some(layer_obj) = self.model.layer(*layer_type) {
                let metrics = self.collect_layer_metrics(layer_obj, type_ids);
                layers.insert(*layer_type, metrics);
            }
        }
        
        ModelMetrics { layers }
    }
    
    fn collect_layer_metrics(
        &self,
        layer: &Layer,
        type_ids: &[TypeId],
    ) -> LayerMetrics {
        let mut object_types = HashMap::new();
        let mut total_count = 0;
        
        for &type_id in type_ids {
            let count = self.model.search_below(layer, type_id).count();
            object_types.insert(type_id, count);
            total_count += count;
        }
        
        LayerMetrics {
            object_count: total_count,
            diagram_count: layer.diagrams().count(),
            object_types,
        }
    }
}

// Visualization support
pub trait MetricsVisualizer {
    fn render(&self, metrics: &ModelMetrics) -> String;
}

pub struct SvgBadgeVisualizer;

impl MetricsVisualizer for SvgBadgeVisualizer {
    fn render(&self, metrics: &ModelMetrics) -> String {
        // Generate SVG badge similar to composer.py
        let mut svg = SvgBuilder::new(134, 30);
        
        // Add bars for each layer
        for (layer, layer_metrics) in &metrics.layers {
            // Draw bars, legends, etc.
        }
        
        svg.build()
    }
}
```

---

## 6. Common Patterns & Best Practices

### 6.1 AlternateAccessor Pattern

**Python Implementation:**
```python
class AlternateAccessor:
    """Provides alternate view of model element."""
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return self.alternate_class.from_model(obj._model, obj._element)

m.ModelElement.pvmt = m.AlternateAccessor(ObjectPVMT)
```

**Purpose:** Provide specialized views of objects without modifying core class

**Rust Translation:**
```rust
// Extension method pattern
pub trait PVMTAccessor {
    fn pvmt(&self) -> ObjectPVMT;
}

impl PVMTAccessor for ModelElement {
    fn pvmt(&self) -> ObjectPVMT {
        ObjectPVMT::from_element(self)
    }
}

// Or using associated types for zero-cost abstraction
pub trait ElementView<V> {
    fn view(&self) -> V;
}

impl ElementView<ObjectPVMT> for ModelElement {
    fn view(&self) -> ObjectPVMT {
        ObjectPVMT::from_element(self)
    }
}

// Usage
let pvmt_view: ObjectPVMT = element.view();
```

### 6.2 Registry Pattern with Static Initialization

**Python Pattern:**
```python
_VALIDATION_RULES = Rules()

@rule(id="Rule-001", ...)
def my_rule(obj):
    return check_condition(obj)

# Rule auto-registers on import
```

**Rust Translation:**
```rust
// Using inventory crate for distributed registration
use inventory;

pub struct Rule {
    pub id: &'static str,
    // ...
}

inventory::collect!(Rule);

// In extension modules
inventory::submit! {
    Rule {
        id: "Rule-001",
        // ...
    }
}

// Access all registered rules
pub fn all_rules() -> impl Iterator<Item = &'static Rule> {
    inventory::iter::<Rule>()
}

// Or using lazy_static for centralized registry
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    static ref RULE_REGISTRY: RwLock<HashMap<&'static str, Rule>> = {
        RwLock::new(HashMap::new())
    };
}

pub fn register_rule(rule: Rule) {
    RULE_REGISTRY.write().unwrap()
        .insert(rule.id, rule);
}
```

### 6.3 Flexible Output Pattern

**Python Pattern:**
```python
def export(target: str | os.PathLike | IO[bytes], ...):
    if isinstance(target, str | os.PathLike):
        ctx = open(target, "wb")
    else:
        ctx = contextlib.nullcontext(target)
    
    with ctx as file:
        write_data(file)
```

**Rust Translation:**
```rust
// Using trait objects for output flexibility
pub trait OutputTarget: Write {}
impl OutputTarget for File {}
impl OutputTarget for Vec<u8> {}
impl OutputTarget for &mut dyn Write {}

pub fn export(
    target: impl Into<Box<dyn OutputTarget>>,
    options: ExportOptions,
) -> Result<(), ExportError> {
    let mut writer = target.into();
    write_data(&mut writer, options)
}

// Or using Path + Write separately
pub fn export_to_path(
    path: impl AsRef<Path>,
    options: ExportOptions,
) -> Result<(), ExportError> {
    let file = File::create(path)?;
    export_to_writer(file, options)
}

pub fn export_to_writer(
    mut writer: impl Write,
    options: ExportOptions,
) -> Result<(), ExportError> {
    // Export logic
    Ok(())
}
```

---

## 7. Arclang Implementation Recommendations

### 7.1 Extension System Architecture

```rust
// Core extension traits
pub trait ModelExtension: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> semver::Version;
    fn init(&self, model: &mut Model) -> Result<(), ExtensionError>;
    fn namespace(&self) -> Option<&Namespace> { None }
}

// Extension registry
pub struct ExtensionRegistry {
    extensions: DashMap<&'static str, Arc<dyn ModelExtension>>,
    initialized: AtomicBool,
}

impl ExtensionRegistry {
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<ExtensionRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| ExtensionRegistry::new())
    }
    
    pub fn register(&self, ext: Arc<dyn ModelExtension>) {
        self.extensions.insert(ext.name(), ext);
    }
    
    pub fn init_all(&self, model: &mut Model) -> Result<(), Vec<ExtensionError>> {
        if self.initialized.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        
        let mut errors = Vec::new();
        for ext in self.extensions.iter() {
            if let Err(e) = ext.value().init(model) {
                log::error!("Failed to initialize extension '{}': {}", ext.key(), e);
                errors.push(e);
            }
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

// Macro for extension registration
#[macro_export]
macro_rules! register_extension {
    ($ext:ty) => {
        #[ctor::ctor]
        fn __register_extension() {
            ExtensionRegistry::global().register(
                Arc::new(<$ext>::default())
            );
        }
    };
}
```

### 7.2 Validation System

```rust
// Rule definition
pub struct ValidationRule {
    pub id: Cow<'static, str>,
    pub name: Cow<'static, str>,
    pub category: RuleCategory,
    pub rationale: Cow<'static, str>,
    pub action: Cow<'static, str>,
    pub applies_to: Box<dyn Fn(&dyn ModelElement) -> bool + Send + Sync>,
    pub validate: Box<dyn Fn(&dyn ModelElement) -> bool + Send + Sync>,
}

// Rule registration using inventory
inventory::collect!(ValidationRule);

// Extension implementation
pub struct ValidationExtension;

impl ModelExtension for ValidationExtension {
    fn name(&self) -> &'static str { "validation" }
    
    fn version(&self) -> semver::Version {
        semver::Version::new(1, 0, 0)
    }
    
    fn init(&self, model: &mut Model) -> Result<(), ExtensionError> {
        // Register validation methods on model
        Ok(())
    }
}

register_extension!(ValidationExtension);

// Validation execution
pub struct Validator {
    rules: Vec<&'static ValidationRule>,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            rules: inventory::iter::<ValidationRule>()
                .collect::<Vec<_>>(),
        }
    }
    
    pub fn validate_model(&self, model: &Model) -> ValidationResults {
        let mut results = ValidationResults::default();
        
        for rule in &self.rules {
            for element in model.all_elements() {
                if (rule.applies_to)(element.as_ref()) {
                    let passed = (rule.validate)(element.as_ref());
                    results.add(ValidationResult {
                        rule,
                        element: element.clone(),
                        passed,
                    });
                }
            }
        }
        
        results
    }
}
```

### 7.3 Requirements/ReqIF System

```rust
// Namespace definition
pub mod reqif {
    use super::*;
    
    pub const REQUIREMENTS_NS: Namespace = Namespace::new(
        "http://www.polarsys.org/kitalpha/requirements",
        "Requirements",
        "org.polarsys.kitalpha.vp.requirements",
    );
    
    pub const CAPELLA_REQUIREMENTS_NS: Namespace = Namespace::new(
        "http://www.polarsys.org/capella/requirements",
        "CapellaRequirements",
        "org.polarsys.capella.vp.requirements",
    );
}

// Type-safe requirement model
#[derive(Debug)]
pub struct Requirement {
    base: ModelElement,
    req_type: Option<ElementRef<RequirementType>>,
    text: HtmlString,
    attributes: Vec<Attribute>,
    relations: Vec<AbstractRelation>,
}

// Export functionality
pub struct ReqIFExporter {
    options: ExportOptions,
}

impl ReqIFExporter {
    pub fn export_module(
        &self,
        module: &CapellaModule,
        writer: impl Write,
    ) -> Result<(), ExportError> {
        let builder = ReqIFDocumentBuilder::new(module, &self.options);
        let document = builder.build()?;
        document.write_to(writer)?;
        Ok(())
    }
}
```

### 7.4 PVMT System

```rust
// Domain-specific language for property management
pub struct PVMTConfiguration {
    domains: HashMap<String, ManagedDomain>,
}

pub struct ManagedDomain {
    name: String,
    version: String,
    groups: HashMap<String, ManagedGroup>,
    types: Vec<EnumerationPropertyType>,
}

pub struct ManagedGroup {
    name: String,
    selector: SelectorRules,
    properties: HashMap<String, PropertyDefinition>,
}

// Property access API
pub trait PVMTAccessor {
    fn pvmt(&self) -> ObjectPVMT;
}

impl PVMTAccessor for ModelElement {
    fn pvmt(&self) -> ObjectPVMT {
        ObjectPVMT::new(self)
    }
}

// Fluent API
pub struct ObjectPVMT {
    element: ElementRef,
    config: Arc<PVMTConfiguration>,
}

impl ObjectPVMT {
    pub fn get(&self, path: &str) -> Result<PropertyValue> {
        // Parse "domain.group.property" path
        // Apply group if not already applied
        // Return property value
    }
    
    pub fn set(&mut self, path: &str, value: PropertyValue) -> Result<()> {
        // Apply group if needed
        // Set property value
    }
    
    // Builder-style API
    pub fn domain(&self, name: &str) -> DomainBuilder {
        DomainBuilder::new(self, name)
    }
}
```

---

## 8. Key Takeaways

### 8.1 Design Principles

1. **Modularity**: Each extension is self-contained with clear boundaries
2. **Declarative Registration**: Use metadata for discovery rather than explicit linking
3. **Lazy Initialization**: Extensions load only when needed
4. **Graceful Degradation**: Missing extensions don't break core functionality
5. **Type Safety**: Strong typing throughout, even in dynamic contexts

### 8.2 Rust-Specific Advantages

1. **Zero-cost abstractions**: Trait-based extensions with no runtime overhead
2. **Thread safety**: Extensions can be safely shared across threads
3. **Compile-time guarantees**: Many errors caught at compile time
4. **Memory safety**: No need for garbage collection
5. **Performance**: Native code generation for validators and processors

### 8.3 Implementation Priority

For Arclang's initial implementation:

1. **Phase 1 - Core Extension System**
   - Extension trait and registry
   - Namespace system
   - Basic model injection via traits

2. **Phase 2 - Validation Extension**
   - Rule definition framework
   - Registry using inventory
   - Basic validators for model integrity

3. **Phase 3 - Requirements/ReqIF**
   - Requirement model types
   - Basic ReqIF import
   - ReqIF export functionality

4. **Phase 4 - PVMT**
   - Property value system
   - Selector rules
   - Dictionary-like access API

5. **Phase 5 - Metrics**
   - Model quantification
   - Visualization support
   - Report generation

### 8.4 Critical Differences from Python

| Aspect | Python (capellambse) | Rust (Arclang) |
|--------|---------------------|----------------|
| Extension Discovery | Entry points via setuptools | inventory crate or manual registration |
| Model Injection | Runtime monkey patching | Compile-time trait implementation |
| Type System | Duck typing | Strong static typing |
| Error Handling | Exceptions | Result types |
| Concurrency | GIL limitations | True parallelism with thread safety |
| Memory Model | Reference counting + GC | Ownership + borrowing |

---

## Conclusion

The capellambse extension system demonstrates a mature, well-architected approach to extensibility. While Python's dynamic nature enables certain patterns (like monkey patching), Rust's trait system provides superior type safety and performance. The key is translating the declarative, metadata-driven approach of Python extensions into Rust's compile-time trait-based system while maintaining the same flexibility and ease of use.

The patterns identified here provide a solid foundation for implementing a robust extension system in Arclang that leverages Rust's strengths while preserving the architectural insights from capellambse.
