# 📋 Requirements Management Integration

**Complete guide for Requirements Management tools integration**

---

## Overview

ArcLang integrates with major Requirements Management (RM) tools to provide bidirectional traceability, automated synchronization, and comprehensive coverage analysis.

## Supported RM Tools

| Tool | Vendor | Status | Key Features |
|------|--------|--------|--------------|
| **DOORS** | IBM | ✅ Full Support | Modules, Baselines, Links, Attributes |
| **Polarion** | Siemens | ✅ Full Support | Work Items, Documents, Traceability |
| **Jama Connect** | Jama | ✅ Full Support | Items, Sets, Relationships |
| **JIRA** | Atlassian | ✅ Full Support | Issues, Epics, Links |

---

## IBM DOORS Integration

### Configuration

```toml
[rm.doors]
url = "https://doors.company.com:9443/dwa"
username = "${DOORS_USER}"
password = "${DOORS_PASSWORD}"
project = "ACC_Project"
module = "System_Requirements"
```

### Synchronization

```bash
# Pull requirements from DOORS
arclang rm pull --system doors --module "System_Requirements"

# Push to DOORS
arclang rm push model.arc --system doors

# Bidirectional sync
arclang rm sync model.arc --system doors
```

### Requirement Mapping

```arc
system_analysis "Requirements" {
    requirement "REQ-ACC-001" {
        description: "System shall maintain safe distance"
        
        doors_properties {
            object_id: "12345"
            module: "System_Requirements"
            absolute_number: "SYS-001"
            heading_number: "3.1.2"
        }
    }
}
```

### Traceability Links

```bash
# Export traceability to DOORS
arclang rm export-traces model.arc --system doors

# Import DOORS link modules
arclang rm import-links --system doors --module "Trace_Matrix"
```

---

## Polarion Integration

### Configuration

```toml
[rm.polarion]
url = "https://polarion.company.com/polarion"
username = "${POLARION_USER}"
password = "${POLARION_PASSWORD}"
project = "acc_system"
space = "requirements"
```

### Work Item Sync

```arc
requirement "REQ-ACC-001" {
    description: "Maintain safe distance"
    
    polarion_properties {
        work_item_id: "ACC-REQ-42"
        type: "requirement"
        status: "approved"
        priority: "high"
        assigned_to: "engineering-team"
    }
}
```

### Document Generation

```bash
# Generate Polarion document
arclang rm export-document model.arc --system polarion --template "SRS_Template"
```

---

## Jama Connect Integration

### Configuration

```toml
[rm.jama]
url = "https://company.jamacloud.com"
api_key = "${JAMA_API_KEY}"
project_id = "12345"
```

### Item Synchronization

```arc
requirement "REQ-ACC-001" {
    jama_properties {
        item_id: 67890
        item_type: "requirement"
        set_id: 123
        parent_id: 456
    }
}
```

---

## JIRA Integration

### Configuration

```toml
[rm.jira]
url = "https://company.atlassian.net"
email = "${JIRA_EMAIL}"
api_token = "${JIRA_API_TOKEN}"
project_key = "ACC"
```

### Issue Mapping

```arc
requirement "REQ-ACC-001" {
    jira_properties {
        issue_key: "ACC-42"
        issue_type: "Story"
        epic: "ACC-1"
        sprint: "Sprint 5"
    }
}
```

---

## Traceability Matrix

### Generate Matrix

```bash
# HTML matrix
arclang rm matrix model.arc --output matrix.html

# Excel matrix
arclang rm matrix model.arc --format excel --output matrix.xlsx

# Push to RM tool
arclang rm push-matrix model.arc --system doors
```

### Matrix Output

```
Traceability Matrix
═══════════════════════════════════════════════════════════════

Requirement     | Component  | Function   | Test       | Status
────────────────┼────────────┼────────────┼────────────┼────────
REQ-ACC-001     | LC-001     | LF-001     | TC-001     | ✓ Complete
REQ-ACC-002     | LC-001     | LF-002     | TC-002     | ✓ Complete  
REQ-ACC-003     | LC-002     | —          | —          | ⚠ Missing test
REQ-ACC-004     | —          | —          | —          | ✗ Not traced

Coverage: 75% (3/4 requirements fully traced)
```

---

## Coverage Analysis

```bash
# Analyze coverage
arclang rm coverage model.arc

# Output:
# Requirements Coverage Analysis
# ══════════════════════════════════════
# 
# Total Requirements: 25
# Traced Requirements: 23 (92%)
# Untraceable Requirements: 2 (8%)
# 
# Component Coverage: 100%
# Test Coverage: 85%
# 
# Gaps:
#   - REQ-003: No test case
#   - REQ-007: No test case
#   - REQ-015: No component allocation
# 
# Recommendations:
#   1. Add test cases for REQ-003, REQ-007
#   2. Allocate REQ-015 to component
#   3. Review untraceable requirements
```

---

## Best Practices

### 1. Unique IDs

```arc
// ✅ Good: Unique, descriptive IDs
requirement "REQ-ACC-DIST-001" {
    description: "Measure distance to lead vehicle"
    doors_id: "SYS-REQ-42"
}

// ❌ Bad: Generic IDs
requirement "R1" {
    description: "Distance"
}
```

### 2. Bidirectional Sync

```bash
# Regular bidirectional sync in CI/CD
arclang rm sync model.arc --system doors --bidirectional
```

### 3. Baseline Management

```bash
# Create baseline before major changes
arclang rm create-baseline model.arc --system doors --name "REL-1.0"

# Compare with baseline
arclang rm compare model.arc --baseline "REL-1.0"
```

---

**Status**: Production Ready ✅  
**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami
