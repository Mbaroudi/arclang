# Capellambse Code Generation Patterns and Examples

## Overview

This document extracts code generation patterns from the capellambse-study repository, showing how Capella model elements are transformed into code artifacts like ROS2 IDL, Protocol Buffers, Python classes, and other formats.

## Source Files Analyzed

- **Primary Notebook**: `/docs/source/examples/07 Code Generation.ipynb`
- **Jinja Templating**: `/docs/source/examples/04 Intro to Jinja templating.ipynb`
- **Validation Template**: `/src/capellambse/extensions/validation/report-template.html.jinja`
- **ReqIF Exporter**: `/src/capellambse/extensions/reqif/exporter.py`
- **Model Structure**: `/src/capellambse/metamodel/information/__init__.py`

## Core Code Generation Patterns

### 1. ROS2 IDL Message Generation

**Pattern**: Transform Capella Class elements into ROS2 message definitions (`.msg` files)

**Source Model Elements**:
- `Class` objects from the Capella data model
- `Property` objects representing class attributes
- Type information from `type` attribute
- Cardinality from `max_card.value`

**Generated Code Structure**:
```
fieldtype1 fieldname1
fieldtype2[] fieldname2
```

**Implementation Example**:
```python
def class_to_ros2_idl(cls):
    filename = f"{cls.name}.msg"
    lines = []
    for prop in cls.properties:
        multiplicity = "[]" * (prop.max_card.value not in ("0", "1"))
        lines.append(f"{prop.type.name}{multiplicity} {prop.name}")
    text = "\n".join(lines)
    with open(filename, "w") as file:
        file.write(text)
    print(f"# file: {filename} \n{text}\n")
```

**Concrete Example**:

*Input Model*: Capella Class "Waypoint" with properties:
- `lat: float` (max_card = 1)
- `lon: float` (max_card = 1)
- `alt: float` (max_card = 1)
- `examples: Example[]` (max_card = *)

*Generated Output* (`Waypoint.msg`):
```
float lat
float lon
float alt
Example[] examples
```

**Key Mapping Rules**:
- Property name → field name (direct mapping)
- Property type.name → field type
- max_card.value determines array notation: `"0"` or `"1"` = scalar, else = array (`[]`)

---

### 2. Python Class Stub Generation

**Pattern**: Generate Python dataclass-style interfaces from Capella Classes

**Source Model Elements**:
- `Class` objects with nested class relationships
- `Property` objects with type and cardinality
- Inheritance handled through recursive generation

**Generated Code Structure**:
```python
class ClassName:
    field1: type
    field2: list[type]
```

**Implementation Example**:
```python
def class_to_python(cls, current_classes=None):
    lines = [f"class {cls.name}:"]
    current_classes = [cls]
    if not cls.properties:
        lines.append(4 * " " + "pass")
    for prop in cls.properties:
        if (
            isinstance(prop.type, mm.information.Class)
            and prop.type not in current_classes
        ):
            nested_text = class_to_python(prop.type, current_classes)
            lines = [nested_text] + ["\n"] + lines
        if prop.max_card.value in ("0", "1"):
            multiplicity = prop.type.name
        else:
            multiplicity = f"list[{prop.type.name}]"
        lines.append(4 * " " + f"{prop.name}: {multiplicity}")
    return "\n".join(lines)
```

**Concrete Example**:

*Input Model*: Capella Class "Trajectory" with:
- `waypoints: Waypoint[]` (nested class)

Where "Waypoint" contains:
- `lat: float`
- `lon: float`
- `alt: float`
- `examples: Example[]`

*Generated Output* (`trajectory.py`):
```python
class Example:
    test: str


class Waypoint:
    lat: float
    lon: float
    alt: float
    examples: list[Example]


class Trajectory:
    waypoints: list[Waypoint]
```

**Key Mapping Rules**:
- Nested classes are generated first (dependency ordering)
- Single cardinality (0,1) → direct type reference
- Multiple cardinality (*) → `list[type]` annotation
- Empty classes get `pass` statement

---

### 3. Protocol Buffers (Protobuf) Generation

**Pattern**: Generate `.proto` files with nested message definitions

**Source Model Elements**:
- `Class` objects with nested relationships
- Property ordering tracked via enumeration
- Type mapping to protobuf types

**Generated Code Structure**:
```protobuf
syntax = "proto3";

message ClassName {
    datatype field1 = 1;
    message NestedClass {
        datatype nested_field = 1;
    }
    repeated NestedClass field2 = 2;
}
```

**Implementation Example**:
```python
def class_to_proto(cls, current_classes=None, indent=""):
    if current_classes is None:
        current_classes = [cls]
        lines = ['syntax = "proto3";\n']
        indent += " " * 4
        lines.append(f"{indent[:-4]}message  {cls.name} {{")
    else:
        lines = [f"{indent[:-4]}message  {cls.name} {{"]

    for counter, prop in enumerate(cls.properties, start=1):
        multiplicity = "[]" * (prop.max_card.value not in ("0", "1"))
        if (
            isinstance(prop.type, mm.information.Class)
            and prop.type not in current_classes
        ):
            current_classes.append(prop.type)
            nested_text = class_to_proto(
                prop.type, current_classes, indent + " " * 4
            )
            lines.append(nested_text)
            lines.append(
                f"{indent}repeated {prop.type.name}{multiplicity} {prop.name} = {counter};"
            )
        else:
            lines.append(
                f"{indent}{prop.type.name}{multiplicity} {prop.name} = {counter};"
            )
    lines.append(f"{indent[:-4]}}}")
    return "\n".join(lines)
```

**Concrete Example**:

*Input Model*: Capella Class "Trajectory"

*Generated Output* (`Trajectory.proto`):
```protobuf
syntax = "proto3";

message  Trajectory {
    message  Waypoint {
        float lat = 1;
        float lon = 2;
        float alt = 3;
        message  Example {
            str test = 1;
        }
        repeated Example[] examples = 4;
    }
    repeated Waypoint[] waypoints = 1;
}
```

**Key Mapping Rules**:
- Field numbers auto-increment starting from 1
- Nested classes become nested messages
- `repeated` keyword for arrays
- Indent management for proper nesting

---

### 4. Jinja2 Template-Based Code Generation

**Pattern**: Use Jinja2 templates to generate arbitrary text-based outputs

**Framework**: `jinja2.Environment()` with model objects passed to templates

**Common Use Cases**:
1. HTML documentation with diagrams
2. Validation reports
3. Requirement specifications (ReqIF)
4. Custom formats

**Template Example** (Actor Documentation):
```jinja2
<h1>Actor definitions</h1>
{% for actor in model.la.all_components.by_is_actor(True) %}
    <h2>{{ actor.name }}</h2>
    <h3>Actor definition</h3>
    <p>UUID: {{ actor.uuid }}</p>
    <p>{{ actor.description }}</p>
{% endfor %}
```

**Advanced Template Example** (Function Tables):
```jinja2
{% set fexs = model.la.all_actor_exchanges.map("func_exchanges") %}
{% for actor in model.la.all_actors %}
    <h2>{{ actor.name }}</h2>
    {% for fnc in actor.allocated_functions %}
    {% if loop.first %}
        <table>
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Function</th>
                    <th>Description</th>
                    <th>Involved Subsystems</th>
                </tr>
            </thead>
            <tbody>
    {% endif %}
    
    {% set outs = fexs.by_source.owner(fnc) %}
    {% set ins = fexs.by_target.owner(fnc) %}
    {% set subs = (ins + outs) | map(attribute="owner.name") | unique | sort %}
    <tr>
        <td>{{ fnc.uuid }}</td>
        <td>{{ fnc.name }}</td>
        <td>{{ fnc.description }}</td>
        <td>{{ subs | join(', ') }}</td>
    </tr>
    
    {% if loop.last %}
            </tbody>
        </table>
    {% endif %}
    {% endfor %}
{% endfor %}
```

**Key Template Features**:
- Model navigation: `model.la.all_actors`, `actor.allocated_functions`
- Filtering: `by_is_actor(True)`, `by_source`, `by_target`
- Chaining: `map("func_exchanges")`, `map(attribute="owner.name")`
- Jinja filters: `unique`, `sort`, `join(', ')`
- Conditionals: `loop.first`, `loop.last`

---

### 5. ReqIF XML Export

**Pattern**: Generate Requirements Interchange Format (ReqIF) XML from Capella requirements

**Key Components**:
- **Datatypes**: Define STRING, XHTML, INTEGER, REAL, BOOLEAN, ENUMERATION
- **Spec Types**: Define requirement types with attributes
- **Spec Objects**: Individual requirements
- **Specifications**: Hierarchical requirement structures

**Code Generation Structure**:

```python
def export_module(module, target, metadata=None, pretty=False, compress=False):
    data = etree.Element("REQ-IF", attrib={...}, nsmap=NSMAP)
    header, timestamp = _build_header(module, metadata)
    data.append(header)
    data.append(_build_content(module, timestamp))
    # Write to file or zip
```

**XML Builder Pattern** (using lxml.builder):
```python
from lxml import builder

E = builder.ElementMaker(namespace=NS)

# Build structured XML
spec_type = E(
    "SPECIFICATION-TYPE",
    {
        "IDENTIFIER": f"_{identifier}",
        "LAST-CHANGE": timestamp,
        "LONG-NAME": module_type.long_name,
    },
    E("SPEC-ATTRIBUTES", 
        E("ATTRIBUTE-DEFINITION-XHTML", {...})
    )
)
```

**Mapping Example**:

*Capella Requirement*:
```
name: "Safety Requirement 1"
identifier: "REQ-001"
text: "System shall..."
type: SafetyRequirement
```

*Generated ReqIF*:
```xml
<SPEC-OBJECT IDENTIFIER="_uuid123" LAST-CHANGE="2024-01-01T00:00:00Z">
    <VALUES>
        <ATTRIBUTE-VALUE-STRING THE-VALUE="REQ-001">
            <DEFINITION>
                <ATTRIBUTE-DEFINITION-STRING-REF>_STD-ATTRIBUTE-type-ReqIF.ForeignID</ATTRIBUTE-DEFINITION-STRING-REF>
            </DEFINITION>
        </ATTRIBUTE-VALUE-STRING>
        <ATTRIBUTE-VALUE-XHTML>
            <DEFINITION>
                <ATTRIBUTE-DEFINITION-XHTML-REF>_STD-ATTRIBUTE-type-ReqIF.Text</ATTRIBUTE-DEFINITION-XHTML-REF>
            </DEFINITION>
            <THE-VALUE><div>System shall...</div></THE-VALUE>
        </ATTRIBUTE-VALUE-XHTML>
    </VALUES>
    <TYPE>
        <SPEC-OBJECT-TYPE-REF>_SAFETYREQ_UUID</SPEC-OBJECT-TYPE-REF>
    </TYPE>
</SPEC-OBJECT>
```

---

## Common Capella Model Element API

### Accessing Model Elements

```python
import capellambse

# Load model
model = capellambse.MelodyModel("path/to/model.aird")

# Access layers
model.oa  # Operational Architecture
model.sa  # System Architecture
model.la  # Logical Architecture
model.pa  # Physical Architecture

# Access data packages
data_pkg = model.oa.data_pkg
classes = data_pkg.classes

# Find by name
cls = classes.by_name("MyClass")

# Filter by type
actors = model.la.all_components.by_is_actor(True)
functions = model.la.all_functions
```

### Class API

```python
# Class properties
cls.name              # str: class name
cls.uuid              # str: unique identifier
cls.description       # str: documentation
cls.properties        # ElementList[Property]: all properties (owned + inherited)
cls.owned_properties  # ElementList[Property]: only directly owned
cls.super             # Class | None: parent class
cls.is_primitive      # bool: primitive type flag

# Property iteration
for prop in cls.properties:
    print(f"{prop.name}: {prop.type.name}")
    print(f"  min_card: {prop.min_card}")
    print(f"  max_card: {prop.max_card.value}")
```

### Property API

```python
# Property attributes
prop.name             # str: property name
prop.type             # Type: property type (could be Class, DataType, etc.)
prop.type.name        # str: type name
prop.min_card         # NumericValue: minimum cardinality
prop.max_card         # NumericValue: maximum cardinality
prop.max_card.value   # str: "0", "1", "*", etc.
prop.description      # str: documentation
prop.aggregation_kind # AggregationKind: UNSET, ASSOCIATION, AGGREGATION, COMPOSITION
```

### Type Checking

```python
import capellambse.metamodel as mm

# Check if type is a Class
if isinstance(prop.type, mm.information.Class):
    # It's a nested class
    nested_class = prop.type
```

---

## Code Generation Frameworks Used

### 1. Direct Python String Building
- Simple concatenation for straightforward formats
- List comprehensions for iteration
- String formatting with f-strings

### 2. Jinja2 Templating
- **When to use**: Complex text generation with conditionals, loops
- **Features**: Template inheritance, macros, filters
- **Best for**: HTML, documentation, reports

### 3. lxml XML Building
- **When to use**: Structured XML/HTML output
- **Features**: ElementMaker pattern, XPath, namespace handling
- **Best for**: ReqIF, XML schemas, SVG

### 4. Pandas DataFrames
- **When to use**: Tabular data export
- **Features**: Excel, CSV export, data manipulation
- **Best for**: Requirements traceability matrices, allocation tables

---

## Related Capellambse Extensions

### capella-ros-tools
- **Purpose**: Import/export ROS message files
- **Direction**: Bidirectional (model ↔ code)
- **Formats**: `*.msg` files, declarative YAML
- **Pattern**: Similar to ROS2 IDL example but with full import capability

### capellambse-context-diagrams
- **Purpose**: Auto-generate context diagrams
- **Exposes**: `.context_diagram`, `.tree_view` attributes on elements
- **Pattern**: Algorithmic diagram generation from model structure

---

## Code Generation Best Practices from Capellambse

### 1. Model Navigation Pattern
```python
def collect_all_dependencies(cls, visited=None):
    """Recursively collect all dependent classes"""
    if visited is None:
        visited = set()
    
    if cls.uuid in visited:
        return []
    visited.add(cls.uuid)
    
    deps = []
    for prop in cls.properties:
        if isinstance(prop.type, mm.information.Class):
            deps.append(prop.type)
            deps.extend(collect_all_dependencies(prop.type, visited))
    
    return deps
```

### 2. Cardinality Handling Pattern
```python
def get_multiplicity_notation(prop, format="python"):
    """Get appropriate multiplicity notation for different formats"""
    is_array = prop.max_card.value not in ("0", "1")
    
    if format == "python":
        return f"list[{prop.type.name}]" if is_array else prop.type.name
    elif format == "ros2":
        return f"{prop.type.name}[]" if is_array else prop.type.name
    elif format == "proto":
        return f"repeated {prop.type.name}" if is_array else prop.type.name
```

### 3. Type Mapping Pattern
```python
TYPE_MAPPINGS = {
    "ros2": {
        "Boolean": "bool",
        "Integer": "int32",
        "Float": "float32",
        "String": "string",
    },
    "python": {
        "Boolean": "bool",
        "Integer": "int",
        "Float": "float",
        "String": "str",
    },
    "proto": {
        "Boolean": "bool",
        "Integer": "int32",
        "Float": "float",
        "String": "string",
    }
}

def map_type(capella_type, target_format):
    return TYPE_MAPPINGS[target_format].get(capella_type, capella_type)
```

### 4. File Generation Pattern
```python
def generate_file(cls, format="python"):
    """Generate code file for a class"""
    generators = {
        "python": class_to_python,
        "ros2": class_to_ros2_idl,
        "proto": class_to_proto,
    }
    
    generator = generators[format]
    content = generator(cls)
    
    extensions = {
        "python": ".py",
        "ros2": ".msg",
        "proto": ".proto",
    }
    
    filename = f"{cls.name}{extensions[format]}"
    with open(filename, "w") as f:
        f.write(content)
    
    return filename
```

---

## Summary

Capellambse provides a **read-only model access** framework that excels at:

1. **Interface Code Generation**: ROS2 IDL, Protobuf, Python stubs
2. **Documentation Generation**: HTML reports, validation reports, requirement specs
3. **Data Export**: ReqIF XML, Excel, CSV
4. **Diagram Rendering**: SVG, PNG export

**Key Strengths**:
- Clean Python API for model navigation
- Jinja2 integration for flexible templating
- Comprehensive metamodel coverage
- Extension architecture for custom generators

**Limitations**:
- No write-back to Capella models (read-only)
- Focused on interface/data structure generation
- Not a full code generator (no implementation logic)

**Arclang Integration Opportunities**:
- Arclang could learn similar patterns for generating ROS2/Proto from its models
- Jinja2 templating approach is applicable
- Model navigation API patterns are instructive
- Type mapping strategies are reusable
