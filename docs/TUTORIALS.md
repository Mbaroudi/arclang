# 📚 ArcLang Tutorials & Examples

**Step-by-step tutorials for learning ArcLang**

---

## Table of Contents

1. [First Model Tutorial](#first-model-tutorial)
2. [Automotive ACC System](#automotive-acc-system)
3. [Aerospace Flight Control](#aerospace-flight-control)
4. [Safety Analysis Workflow](#safety-analysis-workflow)
5. [PLM Integration](#plm-integration-tutorial)

---

## First Model Tutorial

### Step 1: Create Your First Model

Create a file `hello_world.arc`:

```arc
// My First ArcLang Model

system_analysis "Hello World System" {
    requirement "REQ-001" {
        description: "System shall greet users"
        priority: "High"
        verification_method: "Test"
    }
}

logical_architecture "Greeting Architecture" {
    component "Greeter" {
        id: "LC-001"
        type: "Logical"
        description: "Greets users"
        
        function "Say Hello" {
            id: "LF-001"
            outputs: ["greeting_message"]
        }
    }
}

trace "LC-001" satisfies "REQ-001" {
    rationale: "Greeter component implements greeting functionality"
}
```

### Step 2: Compile

```bash
arclang build hello_world.arc
```

**Output:**
```
✓ Compilation successful
  Requirements: 1
  Components: 1
  Functions: 1
  Traces: 1
```

### Step 3: Generate Diagram

```bash
arclang export hello_world.arc -o diagram.html -f arc-viz-ultimate
open diagram.html
```

---

## Automotive ACC System

### Complete ACC Model

```arc
// ═══════════════════════════════════════════════════════════
// Adaptive Cruise Control (ACC) System
// Safety Level: ASIL-B
// ═══════════════════════════════════════════════════════════

// ─── OPERATIONAL ANALYSIS ───────────────────────────────────

operational_analysis "ACC Operations" {
    actor "Driver" {
        id: "ACT-001"
        description: "Vehicle operator"
        role: "Primary user"
    }
    
    capability "Maintain Safe Distance" {
        id: "CAP-001"
        description: "Automatically maintain safe following distance"
        actors: ["ACT-001"]
    }
}

// ─── SYSTEM ANALYSIS (Requirements) ─────────────────────────

system_analysis "ACC Requirements" {
    requirement "REQ-ACC-001" {
        description: "System shall maintain 2-second following distance"
        priority: "Critical"
        type: "Functional"
        safety_level: "ASIL_B"
        verification_method: "Test"
    }
    
    requirement "REQ-ACC-002" {
        description: "Response time shall be < 100ms"
        priority: "High"
        type: "Performance"
        derived_from: ["REQ-ACC-001"]
    }
    
    requirement "REQ-ACC-003" {
        description: "System shall handle sensor failures"
        priority: "Critical"
        type: "Safety"
        safety_level: "ASIL_B"
    }
}

// ─── LOGICAL ARCHITECTURE ───────────────────────────────────

logical_architecture "ACC Architecture" {
    component "Radar Sensor" {
        id: "LC-RADAR"
        type: "Logical"
        description: "77GHz radar for distance measurement"
        
        function "Measure Distance" {
            id: "LF-MEASURE"
            outputs: ["distance_data", "relative_speed"]
            execution_time: "10ms"
        }
        
        function "Self-Diagnostic" {
            id: "LF-DIAG"
            outputs: ["sensor_status"]
            execution_time: "50ms"
        }
    }
    
    component "ACC Controller" {
        id: "LC-CTRL"
        type: "Logical"
        safety_level: "ASIL_B"
        
        function "Calculate Target Speed" {
            id: "LF-CALC"
            inputs: ["distance_data", "relative_speed", "set_speed"]
            outputs: ["target_speed"]
            execution_time: "20ms"
        }
        
        function "Control Throttle" {
            id: "LF-THROTTLE"
            inputs: ["target_speed", "current_speed"]
            outputs: ["throttle_command"]
        }
        
        function "Monitor System Health" {
            id: "LF-HEALTH"
            inputs: ["sensor_status"]
            outputs: ["system_status"]
        }
    }
    
    component "Safety Monitor" {
        id: "LC-SAFETY"
        type: "Logical"
        safety_level: "ASIL_B"
        
        function "Monitor Controller" {
            id: "LF-MON"
            inputs: ["system_status", "throttle_command"]
            outputs: ["safety_status"]
        }
    }
}

// ─── PHYSICAL ARCHITECTURE ──────────────────────────────────

physical_architecture "ACC Hardware" {
    node "Front ECU" {
        id: "PN-FRONT"
        type: "ECU"
        processor: "ARM Cortex-M7 @ 400MHz"
        memory: "2MB Flash, 512KB RAM"
        os: "AUTOSAR Classic 4.3"
        
        deploys "LC-RADAR"
        deploys "LC-CTRL"
        deploys "LC-SAFETY"
    }
    
    link "CAN Bus" {
        id: "PL-CAN"
        type: "CAN"
        bandwidth: "500 kbps"
        protocol: "CAN 2.0B"
        connects: ["PN-FRONT"]
    }
}

// ─── EPBS (Product Structure) ───────────────────────────────

epbs "ACC Product" {
    configuration_item "Radar Module" {
        id: "CI-RADAR"
        type: "Hardware"
        part_number: "77GHz-RADAR-001"
        supplier: "Bosch"
        version: "2.5"
        
        implements "PN-FRONT"
    }
    
    configuration_item "ACC Software" {
        id: "CI-SW"
        type: "Software"
        version: "1.0.0"
        
        implements "PN-FRONT"
    }
}

// ─── TRACEABILITY ───────────────────────────────────────────

trace "CAP-001" derives_from "ACT-001" {
    rationale: "Driver capability to maintain safe distance"
}

trace "REQ-ACC-001" satisfies "CAP-001" {
    rationale: "Requirement implements operational capability"
}

trace "LC-RADAR" satisfies "REQ-ACC-001" {
    rationale: "Radar provides distance measurement for safe following"
}

trace "LC-CTRL" satisfies "REQ-ACC-001" {
    rationale: "Controller maintains target distance"
}

trace "LF-CALC" satisfies "REQ-ACC-002" {
    rationale: "Calculation completes within time constraint"
}

trace "LC-SAFETY" satisfies "REQ-ACC-003" {
    rationale: "Safety monitor handles sensor failures"
}

// ─── SAFETY ANALYSIS ────────────────────────────────────────

hazard "HAZ-ACC-001" {
    description: "Unintended acceleration during cruise control"
    severity: "S3"
    exposure: "E4"
    controllability: "C2"
    asil: "ASIL_C"
    mitigation: ["REQ-ACC-SAFE-001"]
}

fmea "FMEA-RADAR-001" {
    component: "LC-RADAR"
    failure_mode: "Stuck sensor reading"
    effects: "Incorrect distance measurement leading to wrong speed"
    causes: "Sensor hardware failure, signal processing fault"
    severity: 9
    occurrence: 3
    detection: 6
    rpn: 162
    actions: ["Add redundant sensor", "Plausibility check"]
}
```

### Build and Export

```bash
# Compile
arclang build acc_complete.arc --optimize --validate

# Generate diagram
arclang export acc_complete.arc -o acc_diagram.html -f arc-viz-ultimate

# Check traceability
arclang trace acc_complete.arc --validate --matrix

# Show metrics
arclang info acc_complete.arc --metrics --safety
```

---

## Aerospace Flight Control

### Flight Control System Model

```arc
// ═══════════════════════════════════════════════════════════
// Primary Flight Control System
// Safety Level: DAL-A
// ═══════════════════════════════════════════════════════════

system_analysis "Flight Control Requirements" {
    requirement "REQ-FC-001" {
        description: "System shall maintain stable pitch control"
        dal: "DAL_A"
        criticality: "Critical"
        failure_condition: "Catastrophic"
        failure_probability: "< 1E-9 per flight hour"
        
        verification_method: "Test"
        test_coverage: [
            "Statement coverage: 100%",
            "Decision coverage: 100%",
            "MC/DC coverage: 100%"
        ]
    }
    
    requirement "REQ-FC-002" {
        description: "System shall detect control surface failures"
        dal: "DAL_A"
        type: "Safety"
    }
}

logical_architecture "Flight Control Architecture" {
    component "Primary Flight Computer" {
        id: "LC-PFC"
        dal: "DAL_A"
        
        function "Compute Control Laws" {
            id: "LF-CTRL-LAW"
            inputs: ["sensor_data", "pilot_input"]
            outputs: ["control_commands"]
            execution_time: "10ms"
            wcet: "15ms"  // Worst Case Execution Time
        }
        
        function "Monitor Health" {
            id: "LF-HEALTH"
            outputs: ["health_status"]
        }
    }
    
    component "Backup Flight Computer" {
        id: "LC-BFC"
        dal: "DAL_A"
        independence: "Physical"
        diverse_technology: "Different processor architecture"
        
        function "Backup Control Laws" {
            id: "LF-BACKUP"
            inputs: ["sensor_data", "pilot_input"]
            outputs: ["backup_commands"]
        }
    }
}

physical_architecture "IMA Platform" {
    node "Integrated Modular Avionics" {
        id: "PN-IMA"
        processor: "PowerPC"
        os: "VxWorks 653"  // ARINC 653 partitioned OS
        
        partition "Flight Control Partition" {
            id: "PART-FC"
            dal: "DAL_A"
            memory: "4MB"
            time_slots: "100ms per 500ms window"
            memory_protection: "MMU"
            time_protection: "ARINC 653 scheduler"
            
            deploys "LC-PFC"
        }
        
        partition "Backup Partition" {
            id: "PART-BFC"
            dal: "DAL_A"
            memory: "4MB"
            
            deploys "LC-BFC"
        }
    }
}

trace "LC-PFC" satisfies "REQ-FC-001" {
    rationale: "Primary flight computer implements pitch control"
    verification: "Test case TC-FC-001 validates all scenarios"
    independence: "Verified by independent V&V team"
}
```

---

## Safety Analysis Workflow

### Complete Safety Workflow

```bash
#\!/bin/bash
# Complete safety analysis workflow

echo "═══════════════════════════════════════════════════════════"
echo "Safety Analysis Workflow"
echo "═══════════════════════════════════════════════════════════"

# Step 1: Compile model
echo "\n[1/7] Compiling model..."
arclang build acc_system.arc --validate

# Step 2: Generate safety report
echo "\n[2/7] Generating safety report..."
arclang safety acc_system.arc --standard iso26262 --report --output safety_report.html

# Step 3: Validate traceability
echo "\n[3/7] Validating traceability..."
arclang trace acc_system.arc --validate --coverage

# Step 4: Generate FMEA report
echo "\n[4/7] Generating FMEA report..."
arclang safety acc_system.arc --fmea --output fmea_report.html

# Step 5: Generate traceability matrix
echo "\n[5/7] Generating traceability matrix..."
arclang trace acc_system.arc --matrix --output trace_matrix.html

# Step 6: Generate architecture diagram
echo "\n[6/7] Generating architecture diagram..."
arclang export acc_system.arc -o architecture.html -f arc-viz-ultimate

# Step 7: Package all artifacts
echo "\n[7/7] Packaging artifacts..."
mkdir -p safety_package
mv safety_report.html safety_package/
mv fmea_report.html safety_package/
mv trace_matrix.html safety_package/
mv architecture.html safety_package/
cp acc_system.arc safety_package/

echo "\n✓ Safety analysis complete\!"
echo "→ Safety package: ./safety_package/"
```

---

## PLM Integration Tutorial

### Step-by-Step Windchill Integration

```bash
# Step 1: Configure Windchill connection
cat > .arclang.toml << 'EOF'
[plm.windchill]
url = "https://windchill.company.com"
username = "${WINDCHILL_USER}"
password = "${WINDCHILL_PASSWORD}"
vault = "primary"
EOF

# Step 2: Pull baseline from Windchill
arclang plm pull --system windchill --baseline "REL-1.0"

# Step 3: Edit model
# ... make changes to model.arc

# Step 4: Compare with baseline
arclang plm compare model.arc --baseline "REL-1.0"

# Step 5: Create ECO for changes
arclang plm create-eco model.arc \
  --title "Add safety monitoring" \
  --affected-parts auto

# Step 6: Push changes
arclang plm push model.arc --system windchill

# Step 7: Sync BOM
arclang plm push-bom model.arc --system windchill

echo "✓ PLM sync complete\!"
```

---

## Quick Reference

### Common Commands

```bash
# Compile
arclang build model.arc

# Validate
arclang check model.arc --lint --strict

# Generate diagram
arclang export model.arc -o diagram.html -f arc-viz-ultimate

# Traceability
arclang trace model.arc --validate --matrix

# Safety analysis
arclang safety model.arc --standard iso26262 --report

# PLM sync
arclang plm sync model.arc --system windchill

# Show info
arclang info model.arc --metrics
```

---

**Status**: Complete ✅  
**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami
