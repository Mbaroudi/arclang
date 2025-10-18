# 🚀 ArcLang Quick Start Guide

**Get started with ArcLang in 5 minutes!**

---

## ⚡ Installation

### Prerequisites
- Rust 1.70 or higher
- Git (optional)

### Install ArcLang

```bash
# Clone the repository
git clone https://github.com/yourusername/arclang.git
cd arclang

# Build and install
cargo install --path .

# Verify installation
arclang --version
# Output: arclang 1.0.0
```

**Installation time**: ~1-2 minutes (depending on your system)

---

## 📝 Your First Model (2 minutes)

### Step 1: Create a Model File

Create a file named `hello.arc`:

```arc
system_analysis "Hello World System" {
    requirement "REQ-001" {
        description: "System shall greet users"
        priority: "High"
    }
}

logical_architecture "Greeting Architecture" {
    component "Greeter" {
        id: "LC-001"
        type: "Logical"
        
        function "Say Hello" {
            id: "LF-001"
            outputs: ["greeting"]
        }
    }
}

trace "LC-001" satisfies "REQ-001" {
    rationale: "Greeter component implements greeting requirement"
}
```

### Step 2: Compile Your Model

```bash
arclang build hello.arc
```

**Output:**
```
✓ Compilation successful
  Output: hello.json
  Requirements: 1
  Components: 1
  Functions: 1
  Traces: 1
```

**That's it!** You've compiled your first ArcLang model! 🎉

---

## 🎨 Generate a Diagram (1 minute)

### Create Professional Diagram

```bash
arclang export hello.arc -o hello_diagram.html -f arc-viz-ultimate
```

### View Your Diagram

```bash
# macOS
open hello_diagram.html

# Linux
xdg-open hello_diagram.html

# Windows
start hello_diagram.html
```

**Result**: Interactive HTML diagram with:
- ✅ Zero crossings
- ✅ Professional appearance
- ✅ Zoom/pan controls
- ✅ SVG export button

---

## 🎯 Next Steps (Choose Your Path)

### Path 1: Learn by Example

Try the included examples:

```bash
# Automotive example (9 components)
arclang build examples/automotive/acc_complete_architecture.arc
arclang export examples/automotive/acc_complete_architecture.arc -o acc.html -f arc-viz-ultimate
open acc.html

# Aerospace example (3 components)
arclang build examples/aerospace/flight_control_system.arc
arclang export examples/aerospace/flight_control_system.arc -o flight.html -f arc-viz-ultimate
open flight.html

# Defense example (6 components)
arclang build examples/defense/mission_computer.arc
```

### Path 2: Build Your Own Model

Start with a simple template:

```arc
system_analysis "My System" {
    requirement "REQ-001" {
        description: "Your requirement here"
        priority: "High"
    }
}

logical_architecture "My Architecture" {
    component "MyComponent" {
        id: "LC-001"
        type: "Logical"
        
        function "MyFunction" {
            id: "LF-001"
            inputs: ["data_in"]
            outputs: ["data_out"]
        }
    }
}

trace "LC-001" satisfies "REQ-001" {
    rationale: "Component realizes requirement"
}
```

Save as `my_model.arc` and run:

```bash
arclang build my_model.arc
arclang export my_model.arc -o my_diagram.html -f arc-viz-ultimate
```

---

## 🛠️ Essential Commands

### Compilation
```bash
arclang build model.arc              # Compile to Capella XML
arclang build model.arc -o out.json  # Specify output file
```

### Validation
```bash
arclang check model.arc              # Validate model structure
arclang check model.arc --lint       # Include linting checks
```

### Diagram Generation
```bash
arclang export model.arc -o diagram.html -f arc-viz-ultimate  # Best quality
arclang export model.arc -o diagram.mmd -f mermaid            # Mermaid format
arclang export model.arc -o diagram.puml -f plant-uml         # PlantUML format
```

### Traceability Analysis
```bash
arclang trace model.arc --validate   # Check traceability
arclang trace model.arc --matrix     # Show traceability matrix
```

### Model Information
```bash
arclang info model.arc               # Show model statistics
arclang info model.arc --metrics     # Detailed metrics
```

---

## 📚 Language Basics (5-Minute Overview)

### 1. System Analysis (Requirements)

```arc
system_analysis "Requirements" {
    requirement "REQ-001" {
        description: "System shall..."
        priority: "High"              // High, Medium, Low, Critical
        safety_level: "ASIL_B"        // For automotive (ISO 26262)
    }
}
```

### 2. Logical Architecture (Components)

```arc
logical_architecture "Architecture" {
    component "Controller" {
        id: "LC-001"
        type: "Logical"
        description: "Main controller"
        
        function "Process" {
            id: "LF-001"
            inputs: ["sensor_data"]
            outputs: ["control_signal"]
        }
    }
}
```

### 3. Physical Architecture (Hardware)

```arc
physical_architecture "Hardware" {
    node "ECU" {
        id: "PN-001"
        processor: "ARM Cortex-M7"
        memory: "2MB Flash, 512KB RAM"
        
        deploys "LC-001"              // Deploy logical component
    }
}
```

### 4. Traceability

```arc
trace "LC-001" satisfies "REQ-001" {
    rationale: "Controller implements requirement"
}

trace "LF-001" implements "LC-001" {
    rationale: "Function realizes component"
}
```

---

## 🎓 Learning Resources

### Official Documentation
- [Language Reference](LANGUAGE_REFERENCE.md) - Complete syntax guide
- [Safety Standards](SAFETY_STANDARDS.md) - ISO 26262, DO-178C
- [Diagram Guide](../CAPELLA_DIAGRAMS_FINAL.md) - Professional diagrams

### Examples
- `examples/automotive/` - Automotive ACC system
- `examples/aerospace/` - Flight control system
- `examples/defense/` - Mission computer

### Community
- GitHub Issues - Report bugs or ask questions
- GitHub Discussions - Share ideas and best practices

---

## 💡 Common Tasks

### Task 1: Add a New Requirement

```arc
system_analysis "Requirements" {
    requirement "REQ-002" {
        description: "System shall validate input data"
        priority: "High"
        verification_method: "Test"
    }
}
```

### Task 2: Add a New Component

```arc
logical_architecture "Architecture" {
    component "DataValidator" {
        id: "LC-002"
        type: "Logical"
        
        function "Validate" {
            id: "LF-002"
            inputs: ["raw_data"]
            outputs: ["validated_data"]
        }
    }
}
```

### Task 3: Create Traceability

```arc
trace "LC-002" satisfies "REQ-002" {
    rationale: "DataValidator implements validation requirement"
}
```

### Task 4: Generate Diagram

```bash
arclang export model.arc -o updated_diagram.html -f arc-viz-ultimate
open updated_diagram.html
```

---

## 🔧 Troubleshooting

### Issue: Command not found

```bash
# Make sure ArcLang is installed
which arclang

# If not found, install again
cd arclang
cargo install --path .
```

### Issue: Compilation errors

```bash
# Check your syntax
arclang check model.arc

# Common fixes:
# - Ensure all IDs are unique
# - Check for missing quotes around strings
# - Verify all blocks are properly closed with }
```

### Issue: Diagram not displaying

```bash
# Regenerate the diagram
arclang export model.arc -o diagram.html -f arc-viz-ultimate

# Check the HTML file was created
ls -lh diagram.html

# Open in browser
open diagram.html
```

---

## 📊 Quick Reference Card

### File Structure
```
project/
├── model.arc          # Your model file
├── model.json         # Compiled output
└── diagram.html       # Generated diagram
```

### Command Cheat Sheet
```bash
arclang build model.arc              # Compile
arclang check model.arc              # Validate
arclang export model.arc -o out.html # Generate diagram
arclang trace model.arc --matrix     # Traceability
arclang info model.arc               # Statistics
```

### Common File Extensions
- `.arc` - ArcLang source files
- `.json` - Compiled JSON output
- `.xml` - Capella XML export
- `.html` - Interactive diagrams
- `.mmd` - Mermaid diagrams
- `.puml` - PlantUML diagrams

---

## 🎯 5-Minute Challenge

**Can you create a simple ACC (Adaptive Cruise Control) model?**

Requirements:
1. One requirement: "Maintain safe distance"
2. Two components: "Radar Sensor" and "Distance Controller"
3. Traceability between them
4. Generate a diagram

**Solution** (spoiler below):

<details>
<summary>Click to see solution</summary>

```arc
system_analysis "ACC Requirements" {
    requirement "REQ-ACC-001" {
        description: "System shall maintain safe following distance"
        priority: "Critical"
        safety_level: "ASIL_B"
    }
}

logical_architecture "ACC Architecture" {
    component "Radar Sensor" {
        id: "LC-RADAR"
        type: "Logical"
        
        function "Measure Distance" {
            id: "LF-MEASURE"
            outputs: ["distance_data"]
        }
    }
    
    component "Distance Controller" {
        id: "LC-CONTROLLER"
        type: "Logical"
        
        function "Control Speed" {
            id: "LF-CONTROL"
            inputs: ["distance_data"]
            outputs: ["throttle_command"]
        }
    }
}

trace "LC-RADAR" satisfies "REQ-ACC-001" {
    rationale: "Radar provides distance measurement"
}

trace "LC-CONTROLLER" satisfies "REQ-ACC-001" {
    rationale: "Controller maintains safe distance"
}
```

Generate diagram:
```bash
arclang export acc_model.arc -o acc_diagram.html -f arc-viz-ultimate
open acc_diagram.html
```

</details>

---

## 🚀 What's Next?

### Beginner Level
1. ✅ Complete this quick start
2. → Try all example models
3. → Create your own simple model
4. → Read [Language Reference](LANGUAGE_REFERENCE.md)

### Intermediate Level
1. → Learn all 5 Arcadia levels
2. → Master traceability
3. → Explore safety standards
4. → Create complex models

### Advanced Level
1. → Integrate with Capella
2. → Build certification documentation
3. → Contribute to ArcLang
4. → Share your models

---

## ✅ Quick Start Checklist

- [ ] Install ArcLang
- [ ] Create hello.arc
- [ ] Compile your first model
- [ ] Generate a diagram
- [ ] Try an example
- [ ] Read language basics
- [ ] Complete 5-minute challenge
- [ ] Explore advanced features

---

## 🎉 Congratulations!

You've completed the ArcLang Quick Start! You now know how to:
- ✅ Install ArcLang
- ✅ Write basic models
- ✅ Compile and validate
- ✅ Generate professional diagrams
- ✅ Use essential commands

**Ready for more?** Check out:
- [Language Reference](LANGUAGE_REFERENCE.md) - Complete syntax
- [Safety Standards](SAFETY_STANDARDS.md) - ISO 26262, DO-178C
- [Examples](../examples/) - Real-world models

---

**Time invested**: 5 minutes  
**Skills learned**: ArcLang basics  
**Next step**: Build your own model! 🚀

**Questions?** Open an issue on GitHub or check the documentation!
