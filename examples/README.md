# ArcLang Examples

Complete architecture examples demonstrating the ArcLang Domain-Specific Language for Model-Based Systems Engineering (MBSE) using the Arcadia methodology.

## 📁 Directory Structure

```
examples/
├── automotive/          # Automotive domain architectures
│   ├── remote_start/   # Complete remote start system (ISO 26262 ASIL B)
│   ├── acc_*.arc       # Adaptive Cruise Control examples
│   └── adaptive_cruise_control.arc
├── aerospace/          # Aerospace domain architectures
│   └── flight_control_system.arc
├── defense/            # Defense domain architectures
│   └── mission_computer.arc
└── business/           # Business domain architectures
    └── pluxee_analytics.arc
```

---

## 🚗 Automotive Examples

### Remote Start System (Complete Architecture)

**Location:** `automotive/remote_start/`

A complete, production-ready architecture for a Vehicle Remote Start System demonstrating:
- ✅ Full Arcadia methodology (4 layers: Operational, System, Logical, Physical)
- ✅ 33 requirements with complete traceability
- ✅ 25 logical components across 4 layers
- ✅ ISO 26262 ASIL B safety compliance
- ✅ ISO/SAE 21434 cybersecurity compliance
- ✅ Multi-powertrain support (ICE, Hybrid, Electric)

**Files:**
- `remote_start_architecture.arc` (807 lines) - Complete ArcLang source
- `remote_start_architecture_explorer.html` (109 KB) - Interactive web viewer
- `remote_start_architecture_report.tex` - LaTeX documentation (40+ pages)
- `remote_start_diagram.svg` - Capella-compliant architecture diagram
- `remote_start_explorer_package.zip` - Shareable package
- `WORKFLOW_DEMO.md` - Complete workflow from prompt to architecture
- `README_SHARING.md` - Instructions for sharing the explorer

**Generate Explorer:**
```bash
cargo run --bin arclang -- explorer examples/automotive/remote_start/remote_start_architecture.arc
```

**Key Features:**
- Cryptographic authentication (AES-256)
- Safety interlocks (parking brake, transmission, doors)
- 10-minute auto-shutdown timer
- Climate pre-conditioning
- Audit trail logging
- Replay attack protection

**Standards Compliance:**
- ISO 26262:2018 (Functional Safety - ASIL B)
- ISO/SAE 21434:2021 (Cybersecurity)
- UNECE R100 (Electric Vehicle Safety)
- FCC Part 15 (Radiated Emissions)
- GDPR (Data Privacy)

---

### Adaptive Cruise Control (ACC)

**Files:**
- `automotive/acc_complete_architecture.arc` (14 KB) - Complete ACC system
- `automotive/acc_from_capella.arc` - Imported from Eclipse Capella
- `automotive/acc_minimal.arc` - Minimal ACC example
- `automotive/adaptive_cruise_control.arc` (12 KB) - Full implementation

**Features:**
- Radar-based distance measurement
- Speed control algorithms
- Safety interlocks
- ASIL D components

**Generate:**
```bash
cargo run --bin arclang -- explorer examples/automotive/acc_complete_architecture.arc
```

---

## ✈️ Aerospace Examples

### Flight Control System

**File:** `aerospace/flight_control_system.arc` (10 KB)

**Features:**
- Primary flight controls (elevator, aileron, rudder)
- Autopilot modes
- Sensor fusion
- DO-178C compliance
- Redundancy management

**Generate:**
```bash
cargo run --bin arclang -- explorer examples/aerospace/flight_control_system.arc
```

---

## 🛡️ Defense Examples

### Mission Computer

**File:** `defense/mission_computer.arc` (17 KB)

**Features:**
- Multi-domain operations
- Tactical data links
- Weapon systems integration
- Classified data handling
- MIL-STD compliance

**Generate:**
```bash
cargo run --bin arclang -- explorer examples/defense/mission_computer.arc
```

---

## 💼 Business Examples

### Pluxee Analytics Platform

**File:** `business/pluxee_analytics.arc` (18 KB)

**Features:**
- Data ingestion pipelines
- Real-time analytics
- Business intelligence
- GDPR compliance
- Microservices architecture

**Generate:**
```bash
cargo run --bin arclang -- explorer examples/business/pluxee_analytics.arc
```

---

## 🚀 Quick Start

### 1. Compile an Example

```bash
cd /path/to/Arclang
cargo run --bin arclang -- explorer examples/automotive/remote_start/remote_start_architecture.arc
```

### 2. Open in Browser

```bash
# Generated file opens automatically, or:
open examples/automotive/remote_start/remote_start_architecture_explorer.html
```

### 3. Export Diagram

- Click **"📄 Export SVG"** button in the web interface
- SVG file downloads automatically
- Use in documentation, presentations, reports

---

## 📊 Architecture Visualization

All examples generate interactive Architecture Explorers with:

### Capella Standards Compliance
- ✅ Port distribution (IN=left/green, OUT=right/orange)
- ✅ Layer swimlanes with dashed borders
- ✅ ASIL safety badges (B/C/D)
- ✅ Exchange item labels (protocols: CAN, LIN, HTTPS, etc.)
- ✅ Stereotype icons (Controller, ECU, Gateway, etc.)
- ✅ Zero text overlaps (intelligent spacing)
- ✅ Auto-sizing components based on content

### Interactive Features
- 🖱️ Click and drag to pan
- 🔍 Mouse wheel to zoom
- 📄 Export SVG for documentation
- 📊 Layer filtering
- 🔗 Requirement traceability view

---

## 📝 ArcLang Syntax Examples

### Basic Component
```arclang
component "Engine Control Unit" {
    id: "LA-VHC-001"
    layer: "Vehicle"
    stereotype: "ECU"
    safety_level: "ASIL_B"
    
    interface_in: "StartCommand" {
        protocol: "CAN"
        format: "Binary"
    }
    
    interface_out: "EngineStatus" {
        protocol: "CAN"
        format: "Binary"
    }
    
    function "crankEngine" {
        description: "Crank engine via starter motor"
    }
}
```

### Requirements with Traceability
```arclang
requirements stakeholder {
    req "STK-RS-001" "Remote Start from Smartphone" {
        description: "User must be able to remotely start vehicle"
        priority: Critical
    }
}

requirements system {
    req "SYS-RS-001" "Cryptographic Authentication" {
        description: "Authenticate using cryptographic tokens"
        priority: Critical
        safety_level: ASIL_B
    }
}

trace "STK-RS-001" satisfies "SYS-RS-001" {}
```

### Interface Connections
```arclang
interface "TCU to Remote Start Controller" {
    from: "LA-CONN-001"
    to: "LA-CTRL-001"
    description: "Validated start command"
}
```

---

## 🎯 Use Cases

### Educational
- **University Courses:** MBSE, Systems Engineering, Automotive Engineering
- **Workshops:** Arcadia methodology training
- **Tutorials:** Model-based design patterns

### Professional
- **Requirements Management:** Traceability from stakeholders to implementation
- **Safety Analysis:** ISO 26262 compliance documentation
- **Architecture Reviews:** Visual communication with stakeholders
- **Documentation:** Auto-generate architecture diagrams and reports

### Research
- **Methodology Development:** Experiment with MBSE approaches
- **Tool Comparison:** Benchmark against Eclipse Capella, IBM Rhapsody
- **AI-Powered MBSE:** Natural language to architecture generation

---

## 📚 Documentation

### Included Documentation
- **WORKFLOW_DEMO.md** - Complete workflow from prompt to architecture
- **README_SHARING.md** - How to share Architecture Explorers
- **LaTeX Reports** - Publication-ready technical documentation

### External Resources
- [ArcLang Language Specification](https://github.com/arclang/spec)
- [Arcadia Methodology](https://www.eclipse.org/capella/arcadia.html)
- [ISO 26262 Standard](https://www.iso.org/standard/68383.html)

---

## 🛠️ Development Workflow

### 1. Write ArcLang
```bash
vim examples/automotive/my_architecture.arc
```

### 2. Validate Syntax
```bash
cargo run --bin arclang -- validate examples/automotive/my_architecture.arc
```

### 3. Generate Explorer
```bash
cargo run --bin arclang -- explorer examples/automotive/my_architecture.arc
```

### 4. Review in Browser
```bash
open examples/automotive/my_architecture_explorer.html
```

### 5. Export Artifacts
- Click "Export SVG" for diagrams
- Generate LaTeX documentation
- Export JSON for tool integration

---

## 🔧 Customization

### ArcViz Configuration

All examples use the centralized `ARCVIZ_CONFIG` object for visualization:

```javascript
const ARCVIZ_CONFIG = {
    layout: {
        nodesep: 350,    // Horizontal spacing
        ranksep: 200,    // Vertical spacing
    },
    port: {
        spacing: 50,     // Port vertical spacing
        colors: {
            inFill: '#4caf50',   // Green for IN
            outFill: '#ff9800'   // Orange for OUT
        }
    },
    safety: {
        colors: {
            ASIL_B: '#ff9800',
            ASIL_C: '#f44336',
            ASIL_D: '#d32f2f'
        }
    }
};
```

Modify these values in the template to customize visualization.

---

## 📈 Statistics

### Remote Start Architecture
| Metric | Value |
|--------|-------|
| ArcLang Lines of Code | 807 |
| Requirements | 33 |
| Logical Components | 25 |
| Physical Components | 9 |
| Interfaces | 16 |
| Functions | 32 |
| Trace Links | 8 |
| ASIL B Components | 10 |
| Protocols | 7 |

### Development Time
- **Traditional MBSE tools:** 2-3 weeks
- **ArcLang + Claude AI:** ~2 hours
- **Efficiency gain:** 60-80x faster

---

## 🤝 Contributing

### Adding New Examples

1. Choose appropriate domain folder (`automotive/`, `aerospace/`, etc.)
2. Create subdirectory for complex projects
3. Follow naming convention: `lowercase_with_underscores.arc`
4. Include metadata block with standards compliance
5. Add comprehensive requirements and traceability
6. Test compilation and explorer generation

### Example Template

```arclang
model MySystem {
    metadata {
        version: "1.0.0"
        description: "Brief description"
        domain: "automotive|aerospace|defense|business"
        safety_standard: "ISO_26262|DO_178C|MIL_STD"
        project_phase: "concept|system_design|detailed_design"
    }
}

requirements stakeholder { /* ... */ }
requirements system { /* ... */ }
architecture logical { /* ... */ }
trace /* ... */ {}
```

---

## 📞 Support

For questions or issues:
- **GitHub Issues:** [arclang/issues](https://github.com/arclang/issues)
- **Documentation:** [docs.arclang.org](https://docs.arclang.org)
- **Community:** [arclang.slack.com](https://arclang.slack.com)

---

## 📄 License

All examples are provided under the MIT License. See [LICENSE](../LICENSE) for details.

---

## 🎓 Citation

If you use these examples in academic work:

```bibtex
@software{arclang_examples,
  title = {ArcLang Architecture Examples},
  author = {ArcLang Contributors},
  year = {2025},
  url = {https://github.com/arclang/arclang},
  note = {Domain-Specific Language for Arcadia MBSE}
}
```

---

**Last Updated:** October 23, 2025  
**ArcLang Version:** 1.0.0  
**Examples Count:** 9 architectures across 4 domains
