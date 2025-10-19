# 📚 ArcLang Documentation Index

**Complete documentation for ArcLang v1.0.0**

---

## Quick Links

- [Quick Start](QUICKSTART.md) - Get started in 5 minutes
- [Language Reference](LANGUAGE_REFERENCE.md) - Complete syntax guide
- [Examples](../examples/) - Real-world examples

---

## Core Documentation

### Getting Started

| Document | Description | Status |
|----------|-------------|--------|
| [Quick Start Guide](QUICKSTART.md) | 5-minute introduction to ArcLang | ✅ Complete |
| [Installation Guide](../README.md#installation) | Installation instructions | ✅ Complete |
| [First Model Tutorial](TUTORIALS.md#first-model) | Step-by-step first model | ✅ Complete |

### Language

| Document | Description | Status |
|----------|-------------|--------|
| [Language Specification](LANGUAGE_SPECIFICATION.md) | Formal language specification | ✅ Complete |
| [Language Reference](LANGUAGE_REFERENCE.md) | Complete syntax reference | ✅ Complete |
| [Grammar (EBNF)](LANGUAGE_SPECIFICATION.md#formal-grammar-ebnf) | Formal grammar | ✅ Complete |
| [Type System](LANGUAGE_SPECIFICATION.md#type-system) | Type system documentation | ✅ Complete |

### Compiler

| Document | Description | Status |
|----------|-------------|--------|
| [Compiler Architecture](COMPILER_ARCHITECTURE.md) | Internal compiler design | ✅ Complete |
| [Compilation Pipeline](COMPILER_ARCHITECTURE.md#compilation-pipeline) | Pipeline stages | ✅ Complete |
| [Optimization](COMPILER_ARCHITECTURE.md#optimization) | Optimization techniques | ✅ Complete |
| [Incremental Compilation](COMPILER_ARCHITECTURE.md#incremental-compilation) | Fast rebuilds | ✅ Complete |

### CLI & API

| Document | Description | Status |
|----------|-------------|--------|
| [CLI Reference](CLI_REFERENCE.md) | Command-line interface | ✅ Complete |
| [API Reference](API.md) | Rust API documentation | ✅ Complete |
| [Plugin Development](PLUGIN_DEVELOPMENT.md) | Creating compiler plugins | ✅ Complete |

---

## Integration Guides

### PLM Integration

| Document | Description | Status |
|----------|-------------|--------|
| [PLM Integration Overview](PLM_INTEGRATION.md) | PLM systems integration | ✅ Complete |
| [Windchill Integration](PLM_INTEGRATION.md#windchill) | PTC Windchill connector | ✅ Complete |
| [Teamcenter Integration](PLM_INTEGRATION.md#teamcenter) | Siemens Teamcenter | ✅ Complete |
| [SAP Integration](PLM_INTEGRATION.md#sap) | SAP PLM integration | ✅ Complete |
| [BOM Management](PLM_INTEGRATION.md#bom-management) | Bill of materials sync | ✅ Complete |

### Requirements Management

| Document | Description | Status |
|----------|-------------|--------|
| [Requirements Management](REQUIREMENTS_MANAGEMENT.md) | RM tools integration | ✅ Complete |
| [DOORS Integration](REQUIREMENTS_MANAGEMENT.md#doors) | IBM DOORS connector | ✅ Complete |
| [Polarion Integration](REQUIREMENTS_MANAGEMENT.md#polarion) | Siemens Polarion | ✅ Complete |
| [Jama Integration](REQUIREMENTS_MANAGEMENT.md#jama) | Jama Connect | ✅ Complete |
| [JIRA Integration](REQUIREMENTS_MANAGEMENT.md#jira) | Atlassian JIRA | ✅ Complete |

---

## Safety & Certification

### Safety Standards

| Document | Description | Status |
|----------|-------------|--------|
| [Safety Standards Overview](SAFETY_STANDARDS.md) | All standards overview | ✅ Complete |
| [ISO 26262 (Automotive)](SAFETY_STANDARDS.md#iso-26262-automotive) | Automotive safety | ✅ Complete |
| [DO-178C (Aerospace)](SAFETY_STANDARDS.md#do-178c-aerospace) | Aerospace software | ✅ Complete |
| [IEC 61508 (Industrial)](SAFETY_STANDARDS.md#iec-61508-industrial) | Industrial safety | ✅ Complete |

### Safety Analysis

| Document | Description | Status |
|----------|-------------|--------|
| [Traceability Guide](TRACEABILITY.md) | Requirements traceability | ✅ Complete |
| [FMEA Guide](SAFETY_CERTIFICATION.md#fmea) | Failure modes analysis | ✅ Complete |
| [FTA Guide](SAFETY_CERTIFICATION.md#fta) | Fault tree analysis | ✅ Complete |
| [Hazard Analysis](SAFETY_CERTIFICATION.md#hazard-analysis) | HARA process | ✅ Complete |
| [Safety Case](SAFETY_CERTIFICATION.md#safety-case) | Building safety cases | ✅ Complete |

---

## Tutorials & Examples

### Tutorials

| Tutorial | Description | Level | Status |
|----------|-------------|-------|--------|
| [First Model](TUTORIALS.md#first-model) | Your first ArcLang model | Beginner | ✅ Complete |
| [Automotive ACC](TUTORIALS.md#automotive-acc) | Adaptive Cruise Control | Intermediate | ✅ Complete |
| [Aerospace Flight Control](TUTORIALS.md#aerospace) | Flight control system | Advanced | ✅ Complete |
| [Safety Analysis](TUTORIALS.md#safety-analysis) | Complete safety workflow | Advanced | ✅ Complete |
| [PLM Integration](TUTORIALS.md#plm-integration) | Windchill sync | Advanced | ✅ Complete |

### Examples

| Example | Domain | Safety Level | Status |
|---------|--------|--------------|--------|
| [Automotive ACC](../examples/automotive/) | Automotive | ASIL-B | ✅ Complete |
| [Flight Control](../examples/aerospace/) | Aerospace | DAL-A | ✅ Complete |
| [Mission Computer](../examples/defense/) | Defense | SIL-3 | ✅ Complete |
| [Industrial Control](../examples/industrial/) | Industrial | SIL-2 | ✅ Complete |

---

## Best Practices

### Development

| Document | Description | Status |
|----------|-------------|--------|
| [Best Practices Guide](BEST_PRACTICES.md) | Production recommendations | ✅ Complete |
| [Model Organization](BEST_PRACTICES.md#model-organization) | File structure | ✅ Complete |
| [Naming Conventions](BEST_PRACTICES.md#naming-conventions) | Naming standards | ✅ Complete |
| [Testing Strategy](BEST_PRACTICES.md#testing-strategy) | Testing approach | ✅ Complete |

### Team Collaboration

| Document | Description | Status |
|----------|-------------|--------|
| [Version Control](BEST_PRACTICES.md#version-control) | Git workflow | ✅ Complete |
| [Code Review](BEST_PRACTICES.md#team-collaboration) | Review process | ✅ Complete |
| [CI/CD Integration](../README.md#cicd) | Continuous integration | ✅ Complete |

---

## Reference Materials

### Arcadia Methodology

| Resource | Description |
|----------|-------------|
| [Arcadia Overview](https://www.eclipse.org/capella/arcadia.html) | Official Arcadia site |
| [Capella Tool](https://www.eclipse.org/capella/) | Eclipse Capella MBSE tool |
| [Arcadia Book](https://www.amazon.com/Model-Based-Systems-Architecture-Engineering-Arcadia/dp/1785482289) | Comprehensive reference |

### Safety Standards

| Standard | Full Name | Domain |
|----------|-----------|--------|
| ISO 26262:2018 | Road vehicles – Functional safety | Automotive |
| DO-178C | Software Considerations in Airborne Systems | Aerospace |
| IEC 61508:2010 | Functional Safety of E/E/PE Systems | Industrial |
| ISO/IEC 15288 | Systems and software engineering | General |

### Technical Specifications

| Document | Description |
|----------|-------------|
| [Capella XML Format](https://www.eclipse.org/capella/schemas/) | Capella XML schema |
| [ReqIF Specification](https://www.omg.org/spec/ReqIF/) | Requirements Interchange Format |
| [SysML Specification](https://www.omg.org/spec/SysML/) | Systems Modeling Language |

---

## Contributing

### Development Guides

| Document | Description | Status |
|----------|-------------|--------|
| [Contributing Guide](../CONTRIBUTING.md) | How to contribute | ✅ Complete |
| [Development Setup](../README.md#development) | Setting up dev environment | ✅ Complete |
| [Plugin Development](PLUGIN_DEVELOPMENT.md) | Creating plugins | ✅ Complete |
| [Testing Guide](BEST_PRACTICES.md#testing-strategy) | Writing tests | ✅ Complete |

### Community

| Resource | Link |
|----------|------|
| GitHub Repository | https://github.com/Mbaroudi/arclang |
| Issue Tracker | https://github.com/Mbaroudi/arclang/issues |
| Discussions | https://github.com/Mbaroudi/arclang/discussions |
| License | [MIT License](../LICENSE) |

---

## Glossary

### Arcadia Terms

| Term | Definition |
|------|------------|
| **OA** | Operational Analysis - Stakeholder needs and operational context |
| **SA** | System Analysis - System requirements and capabilities |
| **LA** | Logical Architecture - Logical components and functions |
| **PA** | Physical Architecture - Hardware deployment |
| **EPBS** | End Product Breakdown Structure - Physical product structure |

### Safety Terms

| Term | Definition |
|------|------------|
| **ASIL** | Automotive Safety Integrity Level (QM, A, B, C, D) |
| **DAL** | Design Assurance Level (A, B, C, D, E) |
| **SIL** | Safety Integrity Level (1, 2, 3, 4) |
| **FMEA** | Failure Mode and Effects Analysis |
| **FTA** | Fault Tree Analysis |
| **HARA** | Hazard Analysis and Risk Assessment |

### Technical Terms

| Term | Definition |
|------|------------|
| **DSL** | Domain-Specific Language |
| **MBSE** | Model-Based Systems Engineering |
| **PLM** | Product Lifecycle Management |
| **RM** | Requirements Management |
| **BOM** | Bill of Materials |
| **ECO** | Engineering Change Order |

---

## Quick Reference

### File Extensions

| Extension | Type | Description |
|-----------|------|-------------|
| `.arc` | Source | ArcLang source file |
| `.json` | Output | JSON compilation output |
| `.xml` | Output | Capella XML output |
| `.html` | Diagram | Interactive HTML diagram |
| `.svg` | Diagram | SVG vector diagram |
| `.mmd` | Diagram | Mermaid flowchart |

### Common Commands

```bash
# Compile model
arclang build model.arc

# Validate model
arclang check model.arc --lint

# Generate diagram
arclang export model.arc -o diagram.html -f arc-viz-ultimate

# Check traceability
arclang trace model.arc --validate --matrix

# Show information
arclang info model.arc --metrics
```

### Configuration Files

| File | Purpose | Location |
|------|---------|----------|
| `.arclang.toml` | Project configuration | Project root |
| `arclang.toml` | User configuration | Home directory |
| `.gitignore` | Git ignore rules | Project root |
| `CLAUDE.md` | Claude Code config | Project root |

---

## Version Information

**Documentation Version**: 1.0.0  
**ArcLang Version**: 1.0.0  
**Last Updated**: 2025-10-19  
**Authors**: Malek Baroudi & Bilel Laasami  
**License**: MIT

---

## Support

### Getting Help

- 📖 **Documentation**: Read this comprehensive guide
- 💬 **Discussions**: [GitHub Discussions](https://github.com/Mbaroudi/arclang/discussions)
- 🐛 **Issues**: [GitHub Issues](https://github.com/Mbaroudi/arclang/issues)
- 📧 **Contact**: Open an issue for questions

### Reporting Issues

When reporting issues, please include:
1. ArcLang version (`arclang --version`)
2. Operating system and version
3. Complete error message
4. Minimal reproducing example
5. Expected vs actual behavior

---

**Complete**: Yes ✅  
**Certification Ready**: Yes ✅  
**Production Ready**: Yes ✅
