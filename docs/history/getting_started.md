# Getting Started with ArcLang

This guide will help you install ArcLang and create your first project.

## Installation

### From Pre-built Binaries

Download the latest release for your platform:

```bash
# Linux
curl -L https://github.com/arclang/arclang/releases/latest/download/arclang-linux-x86_64 -o arclang
chmod +x arclang
sudo mv arclang /usr/local/bin/

# macOS
curl -L https://github.com/arclang/arclang/releases/latest/download/arclang-macos-x86_64 -o arclang
chmod +x arclang
sudo mv arclang /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/arclang/arclang/releases/latest/download/arclang-windows-x86_64.exe -OutFile arclang.exe
Move-Item arclang.exe C:\Windows\System32\
```

### From Source

```bash
git clone https://github.com/arclang/arclang.git
cd arclang
cargo install --path .
```

### Verify Installation

```bash
arclang --version
```

## Quickstart

### Create Your First Project

```bash
arclang new my-vehicle-system
cd my-vehicle-system
```

This creates a project with the following structure:

```
my-vehicle-system/
├── Arclang.toml          # Project configuration
├── README.md             # Project documentation
├── src/
│   └── main.arc          # Main model file
├── requirements/         # Requirements specifications
└── architecture/         # Architecture models
```

### Edit the Model

Open `src/main.arc` and add your system model:

```arclang
operational_analysis "Vehicle System" {
    actor "Driver" {
        description: "The person operating the vehicle"
    }
    
    operational_capability "Drive Vehicle" {
        description: "Ability to safely operate the vehicle"
        involving: ["Driver"]
    }
}

system_analysis "Vehicle Control System" {
    requirement "SYS-001" {
        description: "System shall respond to driver inputs within 100ms"
        priority: Critical
        safety_level: ASIL_B
    }
    
    system_function "Process Driver Input" {
        inputs: ["driver_command"]
        outputs: ["validated_command"]
    }
}

logical_architecture "Control Architecture" {
    component "ECU" {
        type: Logical
        
        function "Input Validation" {
            inputs: ["driver_command: CommandData"]
            outputs: ["validated_command: CommandData"]
            wcet: "10ms"
        }
    }
}
```

### Build the Project

```bash
arclang build
```

### Check for Errors

```bash
arclang check .
```

### Validate Traceability

```bash
arclang trace . --validate
```

### Generate Safety Analysis

```bash
arclang safety . --standard ISO26262 --fmea --report
```

## Project Configuration

Edit `Arclang.toml` to configure your project:

```toml
[project]
name = "my-vehicle-system"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]

[build]
target = "capella"           # Export target
optimization_level = 2        # Optimization level (0-3)
incremental = true           # Enable incremental compilation

[safety]
standard = "ISO26262"        # Safety standard
asil_level = "B"            # ASIL level

[plm]
system = "windchill"         # PLM system
url = "https://plm.company.com"
project_id = "PROJ-001"
sync_enabled = true

[requirements]
system = "doors"             # Requirements management system
url = "https://doors.company.com"
module = "Vehicle_Requirements"

[collaboration]
semantic_merge = true        # Enable semantic merge
conflict_resolution = "interactive"  # Conflict resolution mode

[plugins]
enabled = [
    "traceability-matrix",
    "architecture-diagram"
]
```

## Common Commands

### Building

```bash
# Build with default settings
arclang build

# Build for release (optimized)
arclang build --release

# Build with incremental compilation
arclang build --incremental

# Build for specific target
arclang build --target capella
```

### Checking

```bash
# Check for errors
arclang check .

# Check with linter
arclang check . --lint

# Check with safety analysis
arclang check . --safety
```

### Traceability

```bash
# Validate all traces
arclang trace . --validate

# Generate traceability matrix
arclang trace . --matrix

# Find trace path between elements
arclang trace . --from REQ-001 --to COMP-001
```

### Safety Analysis

```bash
# DO-178C analysis (aerospace)
arclang safety . --standard DO178C --report

# ISO 26262 analysis (automotive)
arclang safety . --standard ISO26262 --fmea --fta

# IEC 61508 analysis (industrial)
arclang safety . --standard IEC61508 --report
```

### Export/Import

```bash
# Export to Capella
arclang export . -o output/model.capella -f capella

# Export to JSON
arclang export . -o output/model.json -f json

# Import from DOORS
arclang import doors_export.xml -f doors -o requirements/
```

### PLM Sync

```bash
# Pull from PLM
arclang sync pull --plm windchill

# Push to PLM
arclang sync push --plm windchill

# Check sync status
arclang sync status .

# Configure PLM connection
arclang sync configure --plm-type windchill --url https://plm.company.com
```

### Plugin Management

```bash
# List installed plugins
arclang plugin list

# Install plugin
arclang plugin install traceability-matrix

# Get plugin info
arclang plugin info traceability-matrix

# Enable/disable plugin
arclang plugin enable architecture-diagram
arclang plugin disable linter
```

### Development Tools

```bash
# Start REPL
arclang repl

# Start Language Server
arclang lsp --stdio

# Format code
arclang format . --write

# Get project info
arclang info . --metrics --dependencies

# Clean build artifacts
arclang clean . --cache
```

## Next Steps

- Read the [Language Specification](language_spec.md) to learn the full syntax
- Explore [Example Projects](examples.md) for reference implementations
- Learn about [PLM Integration](plm_integration.md) for enterprise workflows
- Review [Safety & Certification](safety_certification.md) for compliance
- Check out [Plugin Development](plugin_development.md) to extend the compiler

## Getting Help

- Run `arclang --help` for CLI reference
- Run `arclang <command> --help` for command-specific help
- Visit [GitHub Discussions](https://github.com/arclang/arclang/discussions) for community support
- Check the [API Reference](api_reference.md) for programmatic usage
