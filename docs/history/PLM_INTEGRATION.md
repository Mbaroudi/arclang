# 🔗 PLM Integration Guide

**Complete guide for Product Lifecycle Management integration with ArcLang**

---

## Overview

ArcLang provides bidirectional synchronization with major PLM systems including PTC Windchill, Siemens Teamcenter, and SAP PLM. This enables seamless integration of architecture models with product data, BOM management, and change control processes.

## Supported PLM Systems

| System | Vendor | Status | Features |
|--------|--------|--------|----------|
| **Windchill** | PTC | ✅ Full Support | Parts, BOM, ECO/ECR, Baselines |
| **Teamcenter** | Siemens | ✅ Full Support | Items, Structure, Change Management |
| **SAP PLM** | SAP | ✅ Full Support | Materials, BOMs, Engineering Changes |

---

## Windchill Integration

### Configuration

```toml
# .arclang.toml
[plm.windchill]
url = "https://windchill.company.com"
username = "${WINDCHILL_USER}"
password = "${WINDCHILL_PASSWORD}"
context = "wcadmin"
vault = "primary"
```

### Synchronization

```bash
# Push ArcLang model to Windchill
arclang plm push model.arc --system windchill

# Pull Windchill baseline
arclang plm pull --system windchill --baseline "REL-1.0"

# Bidirectional sync
arclang plm sync model.arc --system windchill
```

### Part Mapping

Map logical components to Windchill parts:

```arc
epbs "Product Structure" {
    configuration_item "Radar Module" {
        id: "CI-RADAR-001"
        part_number: "123-456-789"  // Windchill part number
        
        plm_properties {
            system: "windchill"
            object_id: "OR:wt.part.WTPart:12345"
            version: "A.2"
            state: "Released"
        }
    }
}
```

### ECO/ECR Creation

Automatic Engineering Change Order creation:

```bash
# Detect changes and create ECO
arclang plm change-order model.arc --system windchill --type ECO

# Output:
# ✓ Changes detected:
#   - Modified: LC-001 (Controller)
#   - Added: LF-005 (New function)
# ✓ ECO created: ECO-2025-0042
# → https://windchill.company.com/Windchill/app/#ptc1/tcomp/infoPage?oid=OR:wt.change2.WTChangeOrder:54321
```

---

## Teamcenter Integration

### Configuration

```toml
[plm.teamcenter]
url = "https://teamcenter.company.com/tc"
username = "${TC_USER}"
password = "${TC_PASSWORD}"
group = "Engineering"
role = "Designer"
```

### Item Management

```arc
configuration_item "ECU Assembly" {
    id: "CI-ECU-001"
    item_id: "ECU-CTRL-001"  // Teamcenter item ID
    
    teamcenter_properties {
        item_type: "Design"
        revision: "B"
        unit_of_measure: "Each"
        owning_group: "Electronics"
    }
}
```

### Structure Management

```bash
# Export BOM structure to Teamcenter
arclang plm export-bom model.arc --system teamcenter

# Import structure from Teamcenter
arclang plm import-structure --system teamcenter --assembly "ECU-ASM-001"
```

---

## SAP PLM Integration

### Configuration

```toml
[plm.sap]
url = "https://sap.company.com:8000"
client = "100"
username = "${SAP_USER}"
password = "${SAP_PASSWORD}"
language = "EN"
```

### Material Master Sync

```arc
configuration_item "Control Unit" {
    id: "CI-CTRL-001"
    material_number: "100-1234-56"  // SAP material number
    
    sap_properties {
        material_type: "FERT"  // Finished product
        base_unit: "PC"
        mrp_type: "PD"
        procurement_type: "F"
        valuation_class: "7920"
    }
}
```

---

## BOM Management

### BOM Export

```bash
# Export to Excel
arclang plm export-bom model.arc --format excel --output bom.xlsx

# Export to CSV
arclang plm export-bom model.arc --format csv --output bom.csv

# Push to PLM
arclang plm push-bom model.arc --system windchill
```

### BOM Structure

```
Product: Adaptive Cruise Control System
├── Hardware
│   ├── Main ECU (CI-ECU-001) → Windchill:OR:wt.part.WTPart:12345
│   ├── Radar Sensor (CI-RADAR-001) → Windchill:OR:wt.part.WTPart:12346
│   └── CAN Bus Module (CI-CAN-001) → Windchill:OR:wt.part.WTPart:12347
└── Software
    ├── Control Software (CI-SW-CTRL) → Windchill:OR:wt.soft.SoftwarePart:54321
    └── Diagnostic Software (CI-SW-DIAG) → Windchill:OR:wt.soft.SoftwarePart:54322
```

---

## Change Management

### Change Detection

```bash
# Compare with PLM baseline
arclang plm compare model.arc --baseline "REL-1.0" --system windchill

# Output:
# Comparing with baseline REL-1.0...
# 
# Changes detected:
# ──────────────────────────────────
# Modified Components: 2
#   - LC-001: Added function LF-005
#   - LC-003: Changed safety level ASIL_B → ASIL_C
# 
# New Components: 1
#   - LC-007: Safety Monitor
# 
# Deleted Components: 0
# 
# Impact Analysis:
#   - Affected parts: 3
#   - Required approvals: Safety Team, Architecture Team
#   - Estimated rework: 40 hours
```

### Automated ECO Workflow

```bash
# Create ECO with full documentation
arclang plm create-eco model.arc \
  --system windchill \
  --title "Add safety monitoring function" \
  --description "Implements REQ-SAFE-042" \
  --affected-parts auto \
  --approvers "safety-team,arch-team"

# Output:
# ✓ ECO created: ECO-2025-0043
# ✓ Affected parts: 3
#   - 123-456-789 (Radar Module)
#   - 987-654-321 (Main ECU)
#   - 555-123-456 (Software Package)
# ✓ Approval workflow started
# ✓ Notifications sent to: safety-team, arch-team
# → Track: https://windchill.company.com/...
```

---

## Best Practices

### 1. Consistent Mapping

Always use consistent ID mapping:

```arc
// ✅ Good: Clear mapping
configuration_item "Sensor Module" {
    id: "CI-SENSOR-001"
    part_number: "PLM-12345"
    plm_system: "windchill"
    plm_id: "OR:wt.part.WTPart:67890"
}

// ❌ Bad: No mapping
configuration_item "Sensor" {
    id: "C1"
}
```

### 2. Version Control

Track PLM versions:

```arc
configuration_item "ECU" {
    version: "A.3"
    plm_version: "A.3"
    baseline: "REL-2.0"
}
```

### 3. Automated Sync

Use CI/CD for automatic synchronization:

```yaml
# .github/workflows/plm-sync.yml
name: PLM Sync
on:
  push:
    branches: [main]

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Sync to Windchill
        run: arclang plm sync model.arc --system windchill
```

---

## API Reference

```rust
use arclang::plm::{PLMConnector, WindchillConnector};

async fn sync_to_plm() -> Result<(), Box<dyn std::error::Error>> {
    let connector = WindchillConnector::new("https://windchill.company.com");
    connector.authenticate("user", "pass").await?;
    
    let model = compile("model.arc")?;
    connector.push_model(&model).await?;
    
    Ok(())
}
```

---

**Status**: Production Ready ✅  
**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami
