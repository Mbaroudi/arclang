# Examples Migration Status

## ✅ Updated Examples (New Capella Viz Template)

These examples have been upgraded to use the new ArcViz Capella visualization template with:
- Port distribution (IN=left/green, OUT=right/orange)
- Layer swimlanes
- ASIL safety badges
- Exchange item labels
- Zero text overlaps

### Working Examples:
1. **automotive/remote_start/** - Complete remote start system (ISO 26262 ASIL B)
   - 807 lines, 33 requirements, 25 components
   - Full LaTeX documentation
   - Interactive HTML explorer
   - ✅ Ready to use

2. **data_platform_migration.arc** - Enterprise data platform migration
   - 868 lines, 27 requirements, 24 components
   - Databricks/Oracle/Snowflake
   - ✅ Ready to use

---

## ⚠️ Examples Requiring Syntax Migration

The following examples use older ArcLang syntax that needs updating to work with the current parser:

### Needs Migration:
1. **aerospace/flight_control_system.arc**
   - Issue: Uses old `epbs`, `physical_link` syntax
   - Error: `Expected identifier, got Inputs`
   
2. **automotive/acc_complete_architecture.arc**
   - Issue: Old syntax constructs
   - Needs: Update to current `architecture logical {}` format

3. **automotive/acc_from_capella.arc**
   - Issue: Capella import format outdated
   - Needs: Re-import with updated importer

4. **automotive/acc_minimal.arc**
   - Issue: Old syntax
   - Needs: Update to current format

5. **automotive/adaptive_cruise_control.arc**
   - Issue: Old syntax constructs
   - Needs: Update to current format

6. **business/pluxee_analytics.arc**
   - Issue: `Expected identifier, got Stakeholder`
   - Needs: Update requirements syntax

7. **defense/mission_computer.arc**
   - Issue: `Expected identifier, got Inputs`
   - Needs: Update to current format

---

## 📝 Migration Guide

### Old Syntax → New Syntax

**Requirements:**
```arclang
// OLD (not supported)
Stakeholder Requirements {
    ...
}

// NEW (current)
requirements stakeholder {
    ...
}
```

**Architecture:**
```arclang
// OLD (not supported)
epbs "System Name" {
    ...
}

physical_link "Link Name" {
    ...
}

// NEW (current)
architecture logical {
    component "Component Name" {
        id: "LA-001"
        layer: "LayerName"
        ...
    }
}

architecture physical {
    component "Hardware Name" {
        id: "PA-001"
        layer: "Physical"
        ...
    }
}
```

**Interfaces:**
```arclang
// OLD (not supported)
Inputs {
    ...
}
Outputs {
    ...
}

// NEW (current)
interface_in: "InputName" {
    protocol: "CAN"
    format: "Binary"
}

interface_out: "OutputName" {
    protocol: "CAN"
    format: "Binary"
}
```

---

## 🔧 How to Migrate

### Option 1: Manual Update
1. Open the `.arc` file
2. Replace old syntax with new syntax (see guide above)
3. Test with: `cargo run --bin arclang -- explorer examples/your_file.arc`
4. Fix any remaining errors

### Option 2: AI-Assisted Migration
Use Claude Code to migrate:
```
Claude, please update this ArcLang file to use the current syntax:
- Update requirements blocks to use `requirements stakeholder/system/functional {}`
- Update architecture to use `architecture logical/physical {}`
- Update interfaces to use `interface_in`/`interface_out`
- Remove deprecated constructs like `epbs`, `physical_link`
```

### Option 3: Re-create from Scratch
For complex files, it may be faster to:
1. Extract requirements and component list
2. Create new file with current syntax
3. Use remote_start_architecture.arc as template

---

## 📊 Migration Priority

**High Priority** (commonly used):
- ✅ data_platform_migration.arc (DONE)
- ⏳ automotive/adaptive_cruise_control.arc
- ⏳ business/pluxee_analytics.arc

**Medium Priority**:
- ⏳ aerospace/flight_control_system.arc
- ⏳ defense/mission_computer.arc

**Low Priority** (specific examples):
- ⏳ automotive/acc_*.arc files

---

## 🎯 Current Status

| Example | Status | Explorer | Notes |
|---------|--------|----------|-------|
| automotive/remote_start/ | ✅ Working | ✅ | Complete with docs |
| data_platform_migration | ✅ Working | ✅ | Freshly upgraded |
| aerospace/flight_control | ❌ Broken | ❌ | Needs migration |
| automotive/acc_complete | ❌ Broken | ❌ | Needs migration |
| automotive/acc_minimal | ❌ Broken | ❌ | Needs migration |
| automotive/adaptive_cruise | ❌ Broken | ❌ | Needs migration |
| business/pluxee_analytics | ❌ Broken | ❌ | Needs migration |
| defense/mission_computer | ❌ Broken | ❌ | Needs migration |

---

## 🤝 Contributing

If you migrate an example:
1. Test it generates successfully
2. Open the HTML explorer to verify visualization
3. Update this file with ✅ status
4. Commit with message: `chore: migrate [example_name] to current syntax`

---

**Last Updated:** October 23, 2025  
**Migrated Examples:** 2/8 (25%)  
**Next Target:** automotive/adaptive_cruise_control.arc
