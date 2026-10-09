# ArcLang Documentation

Welcome to the ArcLang documentation. ArcLang is an industrial-grade programming language and compiler for **Arcadia-as-Code** (also known as **Capella-as-Code**), bringing the power of code-based systems engineering to the Arcadia/Capella methodology.

## Table of Contents

1. [Introduction](INTRODUCTION.md)
2. [Getting Started](getting_started.md)
3. [Language Specification](language_spec.md)
4. [Compiler Architecture](COMPILER_ARCHITECTURE.md)
5. [CLI Reference](CLI_REFERENCE.md)
6. [API Reference](api_reference.md)
7. [PLM Integration](PLM_INTEGRATION.md)
8. [Requirements Management](REQUEREMENTS_MANAGEMENT.md)
9. [Safety & Certification](SAFETY_CERTIFICATION.md)
10. [Plugin Development](PLUGIN_DEVELOPMENT.md)
11. [Examples & Tutorials](TUTORIALS.md)
12. [Best Practices](BEST_PRACTICES.md)

## Quick Links

- **Installation**: See [Getting Started](getting_started.md#installation)
- **First Project**: Follow the [Quickstart Tutorial](getting_started.md#quickstart)
- **Language Reference**: Browse the [Language Specification](language_spec.md)
- **Example Projects**: Explore [Examples](examples.md)

## What is ArcLang?

ArcLang enables systems engineers to describe complex aerospace, automotive, and defense systems using a textual, version-controllable language that follows the **Arcadia methodology**. The language covers all 5 Arcadia levels:

1. **Operational Analysis (OA)** - Actors, capabilities, activities
2. **System Analysis (SA)** - Requirements, functions, system capabilities
3. **Logical Architecture (LA)** - Components, functions, interfaces
4. **Physical Architecture (PA)** - Nodes, deployment, physical links
5. **EPBS** - End Product Breakdown Structure

## Key Features

### 🏭 Industrial-Grade Compiler
- **8-pass compilation pipeline** with semantic validation
- **Incremental compilation** for large projects (<5min full build, <30s incremental)
- **Content-based caching** for optimal performance
- **Multi-threading support** for parallel compilation

### 🔗 Enterprise Integration
- **PLM Systems**: Windchill, Teamcenter, 3DEXPERIENCE, SAP PLM
- **Requirements Management**: DOORS, Polarion, Jama, JIRA
- **Bidirectional synchronization** with 95%+ automation

### 🛡️ Safety & Certification
- **ISO 26262** (Automotive - ASIL A/B/C/D)
- **DO-178C** (Aerospace - DAL A/B/C/D/E)
- **IEC 61508** (Industrial - SIL 1/2/3/4)
- **Automated FMEA/FTA generation**
- **Compliance checking and reporting**

### 🤝 Collaboration
- **Git-first architecture** with semantic merge
- **Multi-user collaboration** (100+ concurrent engineers)
- **Conflict resolution** with intelligent strategies
- **Code review integration**

### 🔌 Extensibility
- **Plugin system** for custom analyses
- **Code generators** for multiple targets
- **Linters and formatters**
- **Visualization plugins**

## System Requirements

- **OS**: Linux, macOS, Windows
- **Memory**: 8GB RAM minimum, 16GB recommended
- **Disk**: 500MB for compiler, 1GB+ for large projects
- **Dependencies**: Rust 1.70+, Git 2.30+

## License

ArcLang is released under the MIT License. See [LICENSE](../LICENSE) for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/arclang/arclang/issues)
- **Discussions**: [GitHub Discussions](https://github.com/arclang/arclang/discussions)
- **Email**: support@arclang.org

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Community

- **Slack**: [Join our Slack](https://arclang.slack.com)
- **Twitter**: [@arclang](https://twitter.com/arclang)
- **Blog**: [blog.arclang.org](https://blog.arclang.org)
