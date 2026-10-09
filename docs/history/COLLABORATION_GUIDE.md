# 🤝 Collaboration Guide

**Change tracking, conflict resolution, and Git integration for ArcLang**

---

## Table of Contents

1. [Overview](#overview)
2. [Change Tracking](#change-tracking)
3. [Git Integration](#git-integration)
4. [Conflict Resolution](#conflict-resolution)
5. [Semantic Merge](#semantic-merge)
6. [Parallel Development](#parallel-development)
7. [Real-World Scenarios](#real-world-scenarios)

---

## Overview

ArcLang enables **true Git-based collaboration** for systems engineering. Unlike binary tools (Capella, DOORS), ArcLang models are **plain text**, enabling:

✅ **Line-by-line change tracking** - See exactly what changed  
✅ **No binary merge conflicts** - Standard Git merge tools work  
✅ **Semantic merge support** - Intelligent conflict resolution  
✅ **Parallel development** - Multiple engineers, same model  
✅ **Git workflows** - Branch, merge, rebase like code  

---

## Change Tracking

### Text-Based Diffs

**Example: Requirement Change**

```bash
# Developer A modifies requirement
git diff models/requirements/safety.arc
```

**Output:**
```diff
 system_analysis "Safety Requirements" {
     requirement "REQ-SAFE-001" {
-        description: "System shall maintain safe distance"
+        description: "System shall maintain 2-second safe following distance"
         priority: "Critical"
-        safety_level: "ASIL_B"
+        safety_level: "ASIL_C"
     }
 }
```

**Benefits:**
- ✅ **Clear visibility**: Exactly what changed
- ✅ **Line-level precision**: No guessing
- ✅ **Reviewable**: Easy code review
- ✅ **Traceable**: Full Git history

### Change History

```bash
# View change history for a file
git log --oneline --follow models/requirements/safety.arc

# Output:
# a3f2b1c Update ASIL level to ASIL-C
# b8e4c3d Add 2-second time constraint
# c9f5d4e Initial safety requirements

# View specific change
git show a3f2b1c
```

**Full change details:**
```diff
commit a3f2b1c
Author: Safety Engineer <safety@company.com>
Date:   2025-10-19

    Update ASIL level to ASIL-C per hazard analysis HAZ-001
    
    Hazard analysis revealed higher severity than initially assessed.
    Updated requirement REQ-SAFE-001 from ASIL-B to ASIL-C.

diff --git a/models/requirements/safety.arc b/models/requirements/safety.arc
index 1234567..abcdefg 100644
--- a/models/requirements/safety.arc
+++ b/models/requirements/safety.arc
@@ -3,7 +3,7 @@ system_analysis "Safety Requirements" {
         description: "System shall maintain 2-second safe following distance"
         priority: "Critical"
-        safety_level: "ASIL_B"
+        safety_level: "ASIL_C"
+        rationale: "Per HAZ-001 severity analysis"
     }
 }
```

### Blame and Attribution

```bash
# Who changed each line?
git blame models/requirements/safety.arc
```

**Output:**
```
a3f2b1c (Safety Eng  2025-10-19) requirement "REQ-SAFE-001" {
a3f2b1c (Safety Eng  2025-10-19)     description: "System shall maintain 2-second..."
c9f5d4e (Systems Eng 2025-10-15)     priority: "Critical"
a3f2b1c (Safety Eng  2025-10-19)     safety_level: "ASIL_C"
a3f2b1c (Safety Eng  2025-10-19)     rationale: "Per HAZ-001 severity analysis"
c9f5d4e (Systems Eng 2025-10-15) }
```

### Visual Diff Tools

```bash
# Use visual diff tools
git difftool models/requirements/safety.arc

# Configure your preferred tool
git config --global diff.tool meld
git config --global difftool.prompt false

# Or use GitHub/GitLab web interface
git push origin feature/update-safety-req
# View PR diff in browser
```

---

## Git Integration

### Basic Git Workflow

```bash
# 1. Start new feature
git checkout -b feature/add-brake-monitor
git pull origin main --rebase

# 2. Make changes
cat >> models/architecture/brake_monitor.arc << 'EOF'
logical_architecture "Brake Monitoring" {
    component "Independent Brake Monitor" {
        id: "LC-BRAKE-MON"
        safety_level: "ASIL_D"
        
        function "Monitor Brake Status" {
            id: "LF-MON-BRAKE"
            inputs: ["brake_pedal", "brake_pressure"]
            outputs: ["brake_status", "fault_detected"]
            execution_time: "10ms"
        }
    }
}
EOF

# 3. Validate changes
arclang check models/architecture/brake_monitor.arc --lint
arclang trace models/ --validate

# 4. Stage and commit
git add models/architecture/brake_monitor.arc
git commit -m "Add independent brake monitor (ASIL-D)

Implements:
- Independent monitoring per ISO 26262 Part 6
- 10ms response time
- Fault detection capability

Addresses: REQ-SAFE-010"

# 5. Push and create PR
git push origin feature/add-brake-monitor
gh pr create --title "Add independent brake monitor" --fill
```

### Advanced Git Operations

**Rebase for Clean History:**
```bash
# Interactive rebase to clean up commits
git rebase -i HEAD~3

# Squash fixup commits
pick a1b2c3d Add brake monitor component
fixup e4f5g6h Fix typo in component ID
fixup h7i8j9k Update safety level

# Result: Single clean commit
```

**Cherry-Pick Specific Changes:**
```bash
# Apply specific commit from another branch
git cherry-pick a3f2b1c

# Cherry-pick just the brake monitor changes
git cherry-pick --no-commit a3f2b1c
git reset HEAD models/requirements/  # Don't include requirement changes
git commit -m "Cherry-pick brake monitor component only"
```

**Stash Work in Progress:**
```bash
# Save work in progress
git stash save "WIP: brake monitor implementation"

# Switch to urgent fix
git checkout main
git checkout -b hotfix/critical-bug
# ... fix bug ...
git commit -am "Fix critical sensor timeout"

# Return to original work
git checkout feature/add-brake-monitor
git stash pop
```

### Git Hooks for ArcLang

**Pre-commit Hook** - Validate before commit:
```bash
# .git/hooks/pre-commit
#!/bin/bash

echo "🔍 Validating ArcLang models..."

# Find all changed .arc files
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.arc$')

if [ -z "$CHANGED_FILES" ]; then
    echo "✅ No ArcLang files changed"
    exit 0
fi

# Validate each file
FAILED=0
for file in $CHANGED_FILES; do
    echo "Checking $file..."
    if ! arclang check "$file" --lint; then
        echo "❌ Validation failed: $file"
        FAILED=1
    fi
done

if [ $FAILED -eq 1 ]; then
    echo "❌ Pre-commit validation failed"
    echo "Fix errors and try again"
    exit 1
fi

# Check traceability
echo "🔗 Checking traceability..."
if ! arclang trace models/ --validate --fail-on-warning; then
    echo "⚠️  Traceability issues detected"
    echo "Continue anyway? (y/n)"
    read -r response
    if [ "$response" != "y" ]; then
        exit 1
    fi
fi

echo "✅ Pre-commit validation passed"
exit 0
```

**Post-merge Hook** - Validate after merge:
```bash
# .git/hooks/post-merge
#!/bin/bash

echo "🔄 Post-merge validation..."

# Check if any .arc files were merged
if git diff-tree -r --name-only --no-commit-id ORIG_HEAD HEAD | grep -q '\.arc$'; then
    echo "ArcLang models changed, validating..."
    
    # Validate all models
    if ! arclang check models/**/*.arc; then
        echo "⚠️  Validation errors after merge"
        echo "Review and fix before committing"
    fi
    
    # Rebuild
    arclang build models/**/*.arc --parallel
fi
```

---

## Conflict Resolution

### Scenario 1: Non-Overlapping Changes (Auto-Merge)

**Developer A** (10:00 AM):
```arc
// models/requirements/functional.arc
system_analysis "ACC Requirements" {
    requirement "REQ-001" {
        description: "Maintain safe distance"
        priority: "Critical"
    }
    
    // Developer A adds REQ-002
    requirement "REQ-002" {
        description: "Detect lead vehicle"
        priority: "High"
    }
}
```

**Developer B** (10:05 AM) - parallel work:
```arc
// models/requirements/functional.arc
system_analysis "ACC Requirements" {
    requirement "REQ-001" {
        description: "Maintain safe distance"
        priority: "Critical"
    }
    
    // Developer B adds REQ-003
    requirement "REQ-003" {
        description: "Adjust speed automatically"
        priority: "High"
    }
}
```

**Merge:**
```bash
# Developer A pushes first
cd /developer-a-workspace
git commit -am "Add REQ-002: Lead vehicle detection"
git push origin main

# Developer B rebases (AUTO-MERGE SUCCESS!)
cd /developer-b-workspace
git commit -am "Add REQ-003: Automatic speed adjustment"
git pull --rebase origin main

# Auto-merge successful! Result:
```

**Merged Result:**
```arc
system_analysis "ACC Requirements" {
    requirement "REQ-001" {
        description: "Maintain safe distance"
        priority: "Critical"
    }
    
    requirement "REQ-002" {
        description: "Detect lead vehicle"
        priority: "High"
    }
    
    requirement "REQ-003" {
        description: "Adjust speed automatically"
        priority: "High"
    }
}
```

✅ **No conflict!** Different sections merged automatically.

### Scenario 2: Overlapping Changes (Manual Conflict)

**Developer A**:
```arc
requirement "REQ-001" {
    description: "Maintain safe following distance of 2 seconds"
    priority: "Critical"
    safety_level: "ASIL_B"
}
```

**Developer B**:
```arc
requirement "REQ-001" {
    description: "Maintain safe distance from lead vehicle"
    priority: "High"  // Changed from Critical
    safety_level: "ASIL_C"  // Changed from ASIL_B
}
```

**Conflict:**
```bash
git pull origin main
# CONFLICT in models/requirements/functional.arc
# Automatic merge failed; fix conflicts and then commit the result.

git status
# Unmerged paths:
#   both modified:   models/requirements/functional.arc
```

**Conflict Markers:**
```arc
requirement "REQ-001" {
<<<<<<< HEAD (Developer B's version)
    description: "Maintain safe distance from lead vehicle"
    priority: "High"
    safety_level: "ASIL_C"
=======
    description: "Maintain safe following distance of 2 seconds"
    priority: "Critical"
    safety_level: "ASIL_B"
>>>>>>> origin/main (Developer A's version)
}
```

**Resolution Process:**
```bash
# 1. Open file in editor
vim models/requirements/functional.arc

# 2. Resolve conflict (combine best of both)
# Remove conflict markers and choose/combine versions:

requirement "REQ-001" {
    description: "Maintain safe following distance of 2 seconds from lead vehicle"
    priority: "Critical"  # Keep Critical (safety requirement)
    safety_level: "ASIL_C"  # Use higher ASIL level (more conservative)
    rationale: "Combined team analysis: 2-second requirement per ISO 26262, ASIL-C per HAZ-001"
}

# 3. Validate resolution
arclang check models/requirements/functional.arc --lint

# 4. Test traceability still works
arclang trace models/ --validate

# 5. Mark as resolved
git add models/requirements/functional.arc

# 6. Commit resolution
git commit -m "Resolve REQ-001 conflict: merge descriptions, keep ASIL-C"

# 7. Push
git push origin main
```

### Scenario 3: Complex Multi-File Conflict

**Situation**: Multiple files modified with cross-dependencies

```bash
# Files in conflict:
# - models/requirements/safety.arc
# - models/architecture/brake_controller.arc
# - models/traceability/traces.arc

# Strategy: Resolve in dependency order
```

**Step-by-step resolution:**

```bash
# 1. Resolve requirements first (foundational)
vim models/requirements/safety.arc
# ... resolve conflicts ...
git add models/requirements/safety.arc

# 2. Validate requirements
arclang check models/requirements/safety.arc

# 3. Resolve architecture (depends on requirements)
vim models/architecture/brake_controller.arc
# ... resolve conflicts ...
# Ensure component IDs match requirement traces
git add models/architecture/brake_controller.arc

# 4. Validate architecture
arclang check models/architecture/brake_controller.arc

# 5. Resolve traces last (depends on both)
vim models/traceability/traces.arc
# ... resolve conflicts ...
# Ensure traces reference valid IDs from both layers
git add models/traceability/traces.arc

# 6. Validate entire model
arclang check models/**/*.arc
arclang trace models/ --validate --coverage

# 7. If validation passes, commit
git commit -m "Resolve multi-file conflict: safety req + brake controller + traces"
```

---

## Semantic Merge

### ArcLang Semantic Merge Tool

**Concept**: Merge based on **semantic structure** (IDs, components) not just text lines.

**Configure Git:**
```bash
# 1. Set up .gitattributes
cat >> .gitattributes << 'EOF'
*.arc merge=arclang-semantic
EOF

# 2. Configure merge driver
git config merge.arclang-semantic.name "ArcLang semantic merge"
git config merge.arclang-semantic.driver "arclang-merge %O %A %B %P"
git config merge.arclang-semantic.recursive binary
```

### Semantic Merge Implementation

**Python-based semantic merge tool:**

```python
#!/usr/bin/env python3
"""
arclang-merge - Semantic merge tool for ArcLang models

Usage: arclang-merge <base> <ours> <theirs> <result>
"""

import sys
import re
from collections import defaultdict

def parse_arclang(content):
    """Parse ArcLang file into semantic elements by ID"""
    elements = {}
    
    # Extract requirements
    req_pattern = r'requirement\s+"([^"]+)"\s*\{([^}]+)\}'
    for match in re.finditer(req_pattern, content, re.DOTALL):
        req_id = match.group(1)
        req_body = match.group(2)
        elements[req_id] = {
            'type': 'requirement',
            'id': req_id,
            'body': req_body.strip(),
            'full': match.group(0)
        }
    
    # Extract components
    comp_pattern = r'component\s+"([^"]+)"\s*\{([^}]+)\}'
    for match in re.finditer(comp_pattern, content, re.DOTALL):
        comp_id_match = re.search(r'id:\s*"([^"]+)"', match.group(2))
        if comp_id_match:
            comp_id = comp_id_match.group(1)
            elements[comp_id] = {
                'type': 'component',
                'id': comp_id,
                'name': match.group(1),
                'body': match.group(2).strip(),
                'full': match.group(0)
            }
    
    # Extract traces
    trace_pattern = r'trace\s+"([^"]+)"\s+(\w+)\s+"([^"]+)"\s*\{([^}]+)\}'
    for match in re.finditer(trace_pattern, content, re.DOTALL):
        trace_id = f"{match.group(1)}→{match.group(3)}"
        elements[trace_id] = {
            'type': 'trace',
            'from': match.group(1),
            'relation': match.group(2),
            'to': match.group(3),
            'body': match.group(4).strip(),
            'full': match.group(0)
        }
    
    return elements

def semantic_merge(base_file, ours_file, theirs_file, result_file):
    """Perform semantic merge"""
    
    # Read files
    with open(base_file, 'r') as f:
        base_content = f.read()
    with open(ours_file, 'r') as f:
        ours_content = f.read()
    with open(theirs_file, 'r') as f:
        theirs_content = f.read()
    
    # Parse into semantic elements
    base_elements = parse_arclang(base_content)
    ours_elements = parse_arclang(ours_content)
    theirs_elements = parse_arclang(theirs_content)
    
    # Merge logic
    merged_elements = {}
    conflicts = []
    
    # Get all unique IDs
    all_ids = set(base_elements.keys()) | set(ours_elements.keys()) | set(theirs_elements.keys())
    
    for elem_id in all_ids:
        base_elem = base_elements.get(elem_id)
        ours_elem = ours_elements.get(elem_id)
        theirs_elem = theirs_elements.get(elem_id)
        
        # Case 1: Element only in ours (new addition)
        if ours_elem and not theirs_elem and not base_elem:
            merged_elements[elem_id] = ours_elem
            
        # Case 2: Element only in theirs (new addition)
        elif theirs_elem and not ours_elem and not base_elem:
            merged_elements[elem_id] = theirs_elem
            
        # Case 3: Both modified same element
        elif ours_elem and theirs_elem:
            if ours_elem['body'] == theirs_elem['body']:
                # Same modification, no conflict
                merged_elements[elem_id] = ours_elem
            else:
                # Different modifications - CONFLICT
                conflicts.append({
                    'id': elem_id,
                    'ours': ours_elem,
                    'theirs': theirs_elem,
                    'base': base_elem
                })
                # Keep ours for now, mark conflict
                merged_elements[elem_id] = ours_elem
                
        # Case 4: Deleted in ours, modified in theirs
        elif not ours_elem and theirs_elem and base_elem:
            # CONFLICT: delete vs modify
            conflicts.append({
                'id': elem_id,
                'ours': None,  # deleted
                'theirs': theirs_elem,
                'base': base_elem
            })
            # Keep theirs (prefer keeping over deleting)
            merged_elements[elem_id] = theirs_elem
            
        # Case 5: Modified in ours, deleted in theirs
        elif ours_elem and not theirs_elem and base_elem:
            # CONFLICT: modify vs delete
            conflicts.append({
                'id': elem_id,
                'ours': ours_elem,
                'theirs': None,  # deleted
                'base': base_elem
            })
            # Keep ours (prefer keeping over deleting)
            merged_elements[elem_id] = ours_elem
    
    # Write result
    if conflicts:
        # Write with conflict markers
        with open(result_file, 'w') as f:
            f.write("// ⚠️  SEMANTIC MERGE CONFLICTS DETECTED\n")
            f.write("// The following elements have conflicts:\n")
            for conflict in conflicts:
                f.write(f"//   - {conflict['id']}\n")
            f.write("// Resolve manually and validate with: arclang check\n\n")
            
            # Write merged elements with conflicts marked
            for elem_id, elem in merged_elements.items():
                # Check if this element has conflict
                conflict = next((c for c in conflicts if c['id'] == elem_id), None)
                
                if conflict:
                    f.write(f"\n// <<<<<<< CONFLICT: {elem_id}\n")
                    f.write("// OURS:\n")
                    if conflict['ours']:
                        f.write(conflict['ours']['full'])
                    else:
                        f.write("// (deleted)\n")
                    f.write("\n// =======\n")
                    f.write("// THEIRS:\n")
                    if conflict['theirs']:
                        f.write(conflict['theirs']['full'])
                    else:
                        f.write("// (deleted)\n")
                    f.write("\n// >>>>>>> END CONFLICT\n\n")
                else:
                    f.write(elem['full'])
                    f.write("\n\n")
        
        return 1  # Conflict exit code
    else:
        # Clean merge
        with open(result_file, 'w') as f:
            for elem in merged_elements.values():
                f.write(elem['full'])
                f.write("\n\n")
        
        return 0  # Success

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: arclang-merge <base> <ours> <theirs> <result>")
        sys.exit(2)
    
    base, ours, theirs, result = sys.argv[1:5]
    exit_code = semantic_merge(base, ours, theirs, result)
    sys.exit(exit_code)
```

**Install semantic merge tool:**
```bash
# 1. Save script
sudo cp arclang-merge /usr/local/bin/
sudo chmod +x /usr/local/bin/arclang-merge

# 2. Test
arclang-merge --help
```

### Semantic Merge Example

**Base version:**
```arc
requirement "REQ-001" {
    description: "System shall work"
    priority: "High"
}

requirement "REQ-002" {
    description: "System shall be safe"
    priority: "Critical"
}
```

**Developer A (ours):**
```arc
requirement "REQ-001" {
    description: "System shall maintain 2-second distance"
    priority: "High"
}

requirement "REQ-002" {
    description: "System shall be safe"
    priority: "Critical"
    safety_level: "ASIL_B"
}

// New requirement
requirement "REQ-003" {
    description: "System shall detect lead vehicle"
    priority: "High"
}
```

**Developer B (theirs):**
```arc
requirement "REQ-001" {
    description: "System shall work properly"
    priority: "Critical"  // Changed from High
}

requirement "REQ-002" {
    description: "System shall be functionally safe"
    priority: "Critical"
}

// New requirement
requirement "REQ-004" {
    description: "System shall adjust speed"
    priority: "Medium"
}
```

**Semantic merge result:**
```arc
// ⚠️  SEMANTIC MERGE CONFLICTS DETECTED
// The following elements have conflicts:
//   - REQ-001
//   - REQ-002
// Resolve manually and validate with: arclang check

// <<<<<<< CONFLICT: REQ-001
// OURS:
requirement "REQ-001" {
    description: "System shall maintain 2-second distance"
    priority: "High"
}
// =======
// THEIRS:
requirement "REQ-001" {
    description: "System shall work properly"
    priority: "Critical"
}
// >>>>>>> END CONFLICT

// <<<<<<< CONFLICT: REQ-002
// OURS:
requirement "REQ-002" {
    description: "System shall be safe"
    priority: "Critical"
    safety_level: "ASIL_B"
}
// =======
// THEIRS:
requirement "REQ-002" {
    description: "System shall be functionally safe"
    priority: "Critical"
}
// >>>>>>> END CONFLICT

// CLEAN MERGE: REQ-003 (only in ours)
requirement "REQ-003" {
    description: "System shall detect lead vehicle"
    priority: "High"
}

// CLEAN MERGE: REQ-004 (only in theirs)
requirement "REQ-004" {
    description: "System shall adjust speed"
    priority: "Medium"
}
```

**Benefits of semantic merge:**
- ✅ REQ-003 and REQ-004 merged automatically (no conflict)
- ✅ Conflicts clearly marked with element IDs
- ✅ Both versions shown for comparison
- ⚠️ Manual resolution still needed for REQ-001 and REQ-002

---

## Parallel Development

### Scenario: Large Team (10+ Engineers)

**Project**: Automotive ACC System  
**Team**: 12 engineers working in parallel

**File Structure for Parallel Work:**
```
models/
├── requirements/
│   ├── functional.arc          # Team A (2 engineers)
│   ├── performance.arc         # Team A (1 engineer)
│   └── safety.arc              # Team B (2 engineers)
├── architecture/
│   ├── sensors.arc             # Team C (2 engineers)
│   ├── controllers.arc         # Team D (2 engineers)
│   └── actuators.arc           # Team D (1 engineer)
└── traceability/
    └── traces.arc              # Team E (2 engineers - integration)
```

**Daily Workflow:**

```bash
# Morning: Each engineer syncs
git checkout main
git pull --rebase

# Work on assigned file
git checkout -b feature/my-component
# ... edit assigned .arc file ...

# Validate locally
arclang check models/architecture/sensors.arc

# Commit and push
git commit -am "Add radar sensor component"
git push origin feature/my-component

# Create PR
gh pr create --title "Add radar sensor" --fill

# Afternoon: Integration team merges
git checkout main
git merge feature/my-component

# Validate integration
arclang check models/**/*.arc
arclang trace models/ --validate --coverage
```

**Conflict Avoidance Strategy:**
- ✅ **File-level separation**: Each team works on different files
- ✅ **ID namespacing**: REQ-SENS-* (sensors), REQ-CTRL-* (controllers)
- ✅ **Daily sync**: Pull --rebase every morning
- ✅ **Small commits**: Commit frequently, push often
- ✅ **PR reviews**: Review before merging to main

### Concurrent Editing - Same File

**When unavoidable**, use structured approach:

```bash
# Engineer A: Working on sensors.arc (10:00 AM)
git checkout -b feature/radar-sensor

# Add radar at top of file
cat > models/architecture/sensors.arc << 'EOF'
logical_architecture "Sensors" {
    component "Radar Sensor" {
        id: "LC-SENS-RADAR"
        // ... details ...
    }
}
EOF

git commit -am "Add radar sensor"

# Engineer B: Working on sensors.arc (10:15 AM) - parallel
git checkout -b feature/camera-sensor

# Add camera at bottom of file
cat >> models/architecture/sensors.arc << 'EOF'

    component "Camera Sensor" {
        id: "LC-SENS-CAM"
        // ... details ...
    }
EOF

git commit -am "Add camera sensor"

# Merge: Engineer A pushes first
cd /engineer-a-workspace
git push origin feature/radar-sensor
# PR merged to main

# Engineer B rebases (auto-merge!)
cd /engineer-b-workspace
git pull --rebase origin main
# Auto-merge successful!

# Result: Both components in same file, no conflict
```

**Result:**
```arc
logical_architecture "Sensors" {
    component "Radar Sensor" {
        id: "LC-SENS-RADAR"
        // ... details ...
    }
    
    component "Camera Sensor" {
        id: "LC-SENS-CAM"
        // ... details ...
    }
}
```

---

## Real-World Scenarios

### Scenario 1: Emergency Hotfix During Active Development

**Situation**: Critical bug discovered in production while feature development ongoing

```bash
# Engineer working on feature
git checkout feature/new-safety-monitor
# ... making changes ...

# URGENT: Production bug reported!

# 1. Stash current work
git stash save "WIP: Safety monitor implementation"

# 2. Switch to main and create hotfix branch
git checkout main
git pull
git checkout -b hotfix/sensor-timeout-fix

# 3. Apply minimal fix
vim models/architecture/radar_sensor.arc

# Change:
execution_time: "50ms"  # Old value causing timeouts
# To:
execution_time: "100ms"  # More conservative timeout

# 4. Validate fix
arclang check models/architecture/radar_sensor.arc
arclang build models/**/*.arc

# 5. Commit and deploy immediately
git commit -am "HOTFIX: Increase radar sensor timeout to 100ms

Root cause: 50ms timeout too aggressive for production hardware
Impact: Prevents sensor timeout errors in field
Validation: Tested on production hardware
Urgency: Critical - customer vehicles affected

Addresses: BUG-12345"

git push origin hotfix/sensor-timeout-fix

# 6. Emergency merge (skip normal PR review)
git checkout main
git merge hotfix/sensor-timeout-fix
git push origin main

# 7. Deploy to production
./deploy_production.sh

# 8. Return to feature work
git checkout feature/new-safety-monitor
git rebase main  # Include hotfix in feature branch
git stash pop    # Restore WIP changes

# Continue feature development with hotfix included
```

### Scenario 2: Distributed Team Across Time Zones

**Team**: Global team (US, Europe, Asia) working 24/5

**US Team** (PST - Morning):
```bash
# 9:00 AM PST
git checkout -b feature/us-radar-enhancement
vim models/architecture/radar_sensor.arc

# Add advanced radar features
# ... work for 8 hours ...

git commit -am "Add radar enhancement features
- Adaptive scanning
- Multi-target tracking
- Weather compensation"

git push origin feature/us-radar-enhancement
gh pr create --title "Radar enhancement" --fill

# End of day: 5:00 PM PST (2:00 AM CET)
```

**Europe Team** (CET - Afternoon starts):
```bash
# 9:00 AM CET (12:00 AM PST - US team sleeping)
git pull origin main

# Review US team's PR
gh pr checkout 42
arclang check models/architecture/radar_sensor.arc

# Provide review comments
gh pr review 42 --comment --body "
✅ Radar enhancement looks good
⚠️  Please add safety analysis for multi-target mode
⚠️  Weather compensation needs ASIL level assignment
"

# Start own work on integration
git checkout -b feature/eu-sensor-fusion
vim models/architecture/sensor_fusion.arc

# ... work integrating US radar changes ...

git commit -am "Add sensor fusion for enhanced radar"
git push origin feature/eu-sensor-fusion

# End of day: 5:00 PM CET (8:00 AM PST - US team waking up)
```

**Asia Team** (IST - Evening starts):
```bash
# 6:00 PM IST (5:30 AM PST, 2:30 PM CET)
git pull origin main

# Both US and EU changes available
git log --oneline -5

# Review both PRs
gh pr list
gh pr view 42  # US radar enhancement
gh pr view 43  # EU sensor fusion

# Merge approved PRs
gh pr merge 42 --merge
gh pr merge 43 --merge

# Integration testing
git pull origin main
arclang check models/**/*.arc
arclang trace models/ --validate --coverage

# Generate integration report
arclang info models/ --metrics --output daily_report.json

# Post report to team chat
./post_to_slack.sh daily_report.json

# End of day: 2:00 AM IST (next day) - cycle repeats
```

**Benefits:**
- ✅ 24-hour continuous development
- ✅ Asynchronous collaboration via Git
- ✅ PR reviews happen across time zones
- ✅ No blocking on meetings

### Scenario 3: Major Refactoring with Active Feature Branches

**Situation**: Need to refactor component IDs while 5 feature branches are active

**Refactoring Plan:**
```bash
# 1. Announce refactoring to team
echo "🚨 Component ID refactoring planned for Friday
Old pattern: LC-001, LC-002, ...
New pattern: LC-SENS-001, LC-CTRL-001, ...
Freeze period: Friday 5:00 PM - Monday 9:00 AM
Action: Merge all feature branches before Friday or rebase after" | ./notify_team.sh

# 2. Friday evening: Perform refactoring
git checkout -b refactor/component-id-scheme

# Automated refactoring script
cat > refactor_ids.sh << 'EOF'
#!/bin/bash
# Refactor component IDs systematically

# Sensors: LC-001 → LC-SENS-001
find models -name "*.arc" -exec sed -i 's/LC-001/LC-SENS-001/g' {} +
find models -name "*.arc" -exec sed -i 's/LC-002/LC-SENS-002/g' {} +

# Controllers: LC-010 → LC-CTRL-001  
find models -name "*.arc" -exec sed -i 's/LC-010/LC-CTRL-001/g' {} +
find models -name "*.arc" -exec sed -i 's/LC-011/LC-CTRL-002/g' {} +

# Actuators: LC-020 → LC-ACT-001
find models -name "*.arc" -exec sed -i 's/LC-020/LC-ACT-001/g' {} +

echo "✅ Refactoring complete"
EOF

chmod +x refactor_ids.sh
./refactor_ids.sh

# Validate
arclang check models/**/*.arc
arclang trace models/ --validate

# Commit
git commit -am "REFACTOR: Standardize component ID naming scheme

Old scheme: LC-NNN (generic)
New scheme: LC-<SUBSYSTEM>-NNN (descriptive)

Changes:
- Sensors: LC-001/002 → LC-SENS-001/002
- Controllers: LC-010/011 → LC-CTRL-001/002  
- Actuators: LC-020 → LC-ACT-001

Breaking change: All component IDs renamed
Action required: Feature branches must rebase

Validation: All checks pass, traceability verified"

git push origin refactor/component-id-scheme

# Quick merge (pre-announced)
git checkout main
git merge refactor/component-id-scheme
git push origin main

# 3. Monday morning: Team rebases feature branches
# Engineer 1:
git checkout feature/my-feature
git rebase main
# CONFLICT (expected)

# Resolve using automated script
./refactor_ids.sh  # Apply same refactoring to feature branch
git add .
git rebase --continue
git push --force-with-lease origin feature/my-feature

# 4. Verify all features rebased
gh pr list --state open
# Check each PR has been updated post-refactoring
```

---

## Summary

### Key Collaboration Benefits

✅ **Text-based models** - Line-by-line diffs, no binary conflicts  
✅ **Standard Git workflows** - Branch, merge, rebase like code  
✅ **Semantic merge** - Intelligent conflict resolution by element ID  
✅ **Parallel development** - Multiple engineers, same model  
✅ **Change tracking** - Full Git history with blame and attribution  
✅ **Conflict resolution** - Clear strategies for manual resolution  
✅ **Global teams** - Asynchronous collaboration across time zones  

### Quick Commands

```bash
# Daily workflow
git pull --rebase && arclang check models/

# Make changes
git checkout -b feature/my-work
# ... edit .arc files ...
arclang check models/ --lint
git commit -am "Add component X"
git push origin feature/my-work

# Resolve conflicts
git merge main
# ... resolve conflicts ...
arclang check models/ --validate
git commit

# Semantic merge
arclang-merge base.arc ours.arc theirs.arc result.arc
```

---

**Version**: 1.0.0  
**Authors**: Malek Baroudi & Bilel Laasami  
**Status**: Production Ready ✅
